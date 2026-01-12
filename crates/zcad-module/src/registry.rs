//! Module registry - manages loading and lifecycle of modules

use crate::command::CommandRegistry;
use crate::context::ModuleContext;
use crate::element::ElementTypeRegistry;
use crate::error::{ModuleError, Result};
use crate::metadata::{ModuleCategory, ModuleMetadata};
use crate::traits::{ModuleState, ZcadModule};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};
use tracing::{debug, error, info};

/// Entry for a registered module
struct ModuleEntry {
    module: Arc<RwLock<Box<dyn ZcadModule>>>,
    metadata: ModuleMetadata,
    state: ModuleState,
}

/// Module registry - central manager for all ZCAD modules
pub struct ModuleRegistry {
    /// Registered modules
    modules: HashMap<String, ModuleEntry>,

    /// Module load order (for proper initialization/shutdown)
    load_order: Vec<String>,

    /// Enabled modules
    enabled: HashSet<String>,

    /// Command registry (shared across all modules)
    commands: CommandRegistry,

    /// Element type registry (shared across all modules)
    element_types: ElementTypeRegistry,

    /// Module context
    context: ModuleContext,
}

impl ModuleRegistry {
    /// Create a new module registry
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            load_order: Vec::new(),
            enabled: HashSet::new(),
            commands: CommandRegistry::new(),
            element_types: ElementTypeRegistry::new(),
            context: ModuleContext::default(),
        }
    }

    /// Create with custom context
    pub fn with_context(context: ModuleContext) -> Self {
        Self {
            modules: HashMap::new(),
            load_order: Vec::new(),
            enabled: HashSet::new(),
            commands: CommandRegistry::new(),
            element_types: ElementTypeRegistry::new(),
            context,
        }
    }

    /// Register a module
    pub fn register<M: ZcadModule + 'static>(&mut self, module: M) -> Result<()> {
        let metadata = module.metadata();
        let module_id = metadata.id.clone();

        if self.modules.contains_key(&module_id) {
            return Err(ModuleError::AlreadyLoaded(module_id));
        }

        info!("Registering module: {} v{}", metadata.name, metadata.version);

        self.modules.insert(
            module_id.clone(),
            ModuleEntry {
                module: Arc::new(RwLock::new(Box::new(module))),
                metadata,
                state: ModuleState::Discovered,
            },
        );

        Ok(())
    }

    /// Load and initialize a module
    pub fn load(&mut self, module_id: &str) -> Result<()> {
        // Check if module exists
        if !self.modules.contains_key(module_id) {
            return Err(ModuleError::NotFound(module_id.to_string()));
        }

        // Check dependencies first
        self.check_dependencies(module_id)?;

        // Update state to loading
        if let Some(entry) = self.modules.get_mut(module_id) {
            entry.state = ModuleState::Loading;
        }

        // Load dependencies first (recursive)
        let deps: Vec<String> = self
            .modules
            .get(module_id)
            .map(|e| {
                e.metadata
                    .dependencies
                    .iter()
                    .filter(|d| !d.optional)
                    .map(|d| d.module_id.clone())
                    .collect()
            })
            .unwrap_or_default();

        for dep_id in deps {
            if !self.is_loaded(&dep_id) {
                self.load(&dep_id)?;
            }
        }

        // Initialize the module
        let entry = self.modules.get(module_id).unwrap();
        let module_arc = entry.module.clone();

        {
            let mut module = module_arc.write().unwrap();
            if let Err(e) = module.initialize(&mut self.context) {
                error!("Failed to initialize module {}: {}", module_id, e);
                if let Some(entry) = self.modules.get_mut(module_id) {
                    entry.state = ModuleState::Failed;
                }
                return Err(ModuleError::InitializationFailed(e.to_string()));
            }

            // Register commands
            module.register_commands(&mut self.commands);

            // Register element types
            module.register_element_types(&mut self.element_types);
        }

        // Update state
        if let Some(entry) = self.modules.get_mut(module_id) {
            entry.state = ModuleState::Loaded;
            self.load_order.push(module_id.to_string());
            self.enabled.insert(module_id.to_string());
            info!("Module loaded: {}", module_id);
        }

        Ok(())
    }

    /// Unload a module
    pub fn unload(&mut self, module_id: &str) -> Result<()> {
        if !self.modules.contains_key(module_id) {
            return Err(ModuleError::NotFound(module_id.to_string()));
        }

        // Check if other modules depend on this one
        for (id, entry) in &self.modules {
            if id != module_id {
                for dep in &entry.metadata.dependencies {
                    if dep.module_id == module_id && !dep.optional {
                        return Err(ModuleError::DependencyNotSatisfied {
                            module: id.clone(),
                            dependency: module_id.to_string(),
                        });
                    }
                }
            }
        }

        // Update state
        if let Some(entry) = self.modules.get_mut(module_id) {
            entry.state = ModuleState::Unloading;
        }

        // Unregister commands from this module
        let commands_to_remove: Vec<String> = self
            .commands
            .commands_by_module(module_id)
            .iter()
            .map(|c| c.name().to_string())
            .collect();

        for cmd_name in commands_to_remove {
            self.commands.unregister(&cmd_name);
        }

        // Shutdown the module
        if let Some(entry) = self.modules.get(module_id) {
            let mut module = entry.module.write().unwrap();
            module.shutdown();
        }

        // Remove from load order
        self.load_order.retain(|id| id != module_id);
        self.enabled.remove(module_id);

        // Remove from registry
        self.modules.remove(module_id);

        info!("Module unloaded: {}", module_id);
        Ok(())
    }

    /// Enable a loaded module
    pub fn enable(&mut self, module_id: &str) -> Result<()> {
        if let Some(entry) = self.modules.get_mut(module_id) {
            if entry.state == ModuleState::Loaded || entry.state == ModuleState::Disabled {
                entry.state = ModuleState::Active;
                self.enabled.insert(module_id.to_string());

                let mut module = entry.module.write().unwrap();
                module.set_enabled(true);

                debug!("Module enabled: {}", module_id);
                Ok(())
            } else {
                Err(ModuleError::Other(format!(
                    "Module {} is not in a loadable state",
                    module_id
                )))
            }
        } else {
            Err(ModuleError::NotFound(module_id.to_string()))
        }
    }

    /// Disable a module (without unloading)
    pub fn disable(&mut self, module_id: &str) -> Result<()> {
        if let Some(entry) = self.modules.get_mut(module_id) {
            if entry.state == ModuleState::Active || entry.state == ModuleState::Loaded {
                entry.state = ModuleState::Disabled;
                self.enabled.remove(module_id);

                let mut module = entry.module.write().unwrap();
                module.set_enabled(false);

                debug!("Module disabled: {}", module_id);
                Ok(())
            } else {
                Err(ModuleError::Other(format!(
                    "Module {} is not active",
                    module_id
                )))
            }
        } else {
            Err(ModuleError::NotFound(module_id.to_string()))
        }
    }

    /// Check if a module is loaded
    pub fn is_loaded(&self, module_id: &str) -> bool {
        self.modules
            .get(module_id)
            .map(|e| matches!(e.state, ModuleState::Loaded | ModuleState::Active))
            .unwrap_or(false)
    }

    /// Check if a module is enabled
    pub fn is_enabled(&self, module_id: &str) -> bool {
        self.enabled.contains(module_id)
    }

    /// Get module metadata
    pub fn get_metadata(&self, module_id: &str) -> Option<&ModuleMetadata> {
        self.modules.get(module_id).map(|e| &e.metadata)
    }

    /// Get module state
    pub fn get_state(&self, module_id: &str) -> Option<ModuleState> {
        self.modules.get(module_id).map(|e| e.state)
    }

    /// List all registered module IDs
    pub fn module_ids(&self) -> Vec<&str> {
        self.modules.keys().map(|s| s.as_str()).collect()
    }

    /// List modules by category
    pub fn modules_by_category(&self, category: ModuleCategory) -> Vec<&ModuleMetadata> {
        self.modules
            .values()
            .filter(|e| e.metadata.category == category)
            .map(|e| &e.metadata)
            .collect()
    }

    /// List enabled modules
    pub fn enabled_modules(&self) -> Vec<&ModuleMetadata> {
        self.enabled
            .iter()
            .filter_map(|id| self.get_metadata(id))
            .collect()
    }

    /// Get command registry
    pub fn commands(&self) -> &CommandRegistry {
        &self.commands
    }

    /// Get mutable command registry
    pub fn commands_mut(&mut self) -> &mut CommandRegistry {
        &mut self.commands
    }

    /// Get element type registry
    pub fn element_types(&self) -> &ElementTypeRegistry {
        &self.element_types
    }

    /// Get mutable element type registry
    pub fn element_types_mut(&mut self) -> &mut ElementTypeRegistry {
        &mut self.element_types
    }

    /// Get module context
    pub fn context(&self) -> &ModuleContext {
        &self.context
    }

    /// Get mutable module context
    pub fn context_mut(&mut self) -> &mut ModuleContext {
        &mut self.context
    }

    /// Check dependencies for a module
    fn check_dependencies(&self, module_id: &str) -> Result<()> {
        let entry = self
            .modules
            .get(module_id)
            .ok_or_else(|| ModuleError::NotFound(module_id.to_string()))?;

        for dep in &entry.metadata.dependencies {
            if dep.optional {
                continue;
            }

            // Check if dependency exists
            if let Some(dep_entry) = self.modules.get(&dep.module_id) {
                // Check version compatibility
                if !dep_entry
                    .metadata
                    .version
                    .is_compatible_with(&dep.min_version)
                {
                    return Err(ModuleError::VersionIncompatible {
                        module: module_id.to_string(),
                        required: dep.min_version.to_string(),
                        found: dep_entry.metadata.version.to_string(),
                    });
                }
            } else {
                return Err(ModuleError::DependencyNotSatisfied {
                    module: module_id.to_string(),
                    dependency: dep.module_id.clone(),
                });
            }
        }

        // Check for circular dependencies
        self.check_circular_dependencies(module_id, &mut HashSet::new())?;

        Ok(())
    }

    /// Check for circular dependencies
    fn check_circular_dependencies(
        &self,
        module_id: &str,
        visited: &mut HashSet<String>,
    ) -> Result<()> {
        if visited.contains(module_id) {
            return Err(ModuleError::CircularDependency(module_id.to_string()));
        }

        visited.insert(module_id.to_string());

        if let Some(entry) = self.modules.get(module_id) {
            for dep in &entry.metadata.dependencies {
                if !dep.optional {
                    self.check_circular_dependencies(&dep.module_id, visited)?;
                }
            }
        }

        visited.remove(module_id);
        Ok(())
    }

    /// Shutdown all modules in reverse load order
    pub fn shutdown_all(&mut self) {
        info!("Shutting down all modules...");

        for module_id in self.load_order.iter().rev() {
            if let Some(entry) = self.modules.get(module_id) {
                debug!("Shutting down module: {}", module_id);
                let mut module = entry.module.write().unwrap();
                module.shutdown();
            }
        }

        self.load_order.clear();
        self.enabled.clear();
        self.modules.clear();

        info!("All modules shut down");
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for ModuleRegistry {
    fn drop(&mut self) {
        self.shutdown_all();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::metadata::{ModuleDependency, Version};

    struct TestModule {
        id: String,
        deps: Vec<ModuleDependency>,
    }

    impl ZcadModule for TestModule {
        fn metadata(&self) -> ModuleMetadata {
            let mut meta = ModuleMetadata::new(&self.id, "Test Module", ModuleCategory::Core);
            meta.dependencies = self.deps.clone();
            meta
        }

        fn initialize(&mut self, _ctx: &mut ModuleContext) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn test_module_registration() {
        let mut registry = ModuleRegistry::new();

        let module = TestModule {
            id: "zcad.test".to_string(),
            deps: vec![],
        };

        registry.register(module).unwrap();
        assert!(registry.modules.contains_key("zcad.test"));
    }

    #[test]
    fn test_module_loading() {
        let mut registry = ModuleRegistry::new();

        let module = TestModule {
            id: "zcad.test".to_string(),
            deps: vec![],
        };

        registry.register(module).unwrap();
        registry.load("zcad.test").unwrap();

        assert!(registry.is_loaded("zcad.test"));
        assert!(registry.is_enabled("zcad.test"));
    }

    #[test]
    fn test_dependency_resolution() {
        let mut registry = ModuleRegistry::new();

        // Register core module first
        let core = TestModule {
            id: "zcad.core".to_string(),
            deps: vec![],
        };
        registry.register(core).unwrap();

        // Register module that depends on core
        let dependent = TestModule {
            id: "zcad.dependent".to_string(),
            deps: vec![ModuleDependency::required(
                "zcad.core",
                Version::new(0, 1, 0),
            )],
        };
        registry.register(dependent).unwrap();

        // Load dependent - should auto-load core first
        registry.load("zcad.dependent").unwrap();

        assert!(registry.is_loaded("zcad.core"));
        assert!(registry.is_loaded("zcad.dependent"));
    }
}
