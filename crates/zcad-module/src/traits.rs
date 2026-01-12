//! Core module trait definition

use crate::command::CommandRegistry;
use crate::context::ModuleContext;
use crate::element::ElementTypeRegistry;
use crate::error::Result;
use crate::metadata::ModuleMetadata;

/// The core trait that all ZCAD modules must implement.
///
/// A module represents a domain-specific extension to ZCAD, such as:
/// - MCAD: Mechanical design (parts, assemblies, drawings)
/// - BIM: Building information modeling (beams, columns, walls)
/// - EDA: Electronic design (schematics, PCB, components)
///
/// # Lifecycle
///
/// 1. Module is discovered and metadata is read
/// 2. Dependencies are resolved
/// 3. `initialize()` is called
/// 4. Module is active and can be used
/// 5. `shutdown()` is called when unloading
///
/// # Example
///
/// ```rust,ignore
/// pub struct SteelBimModule {
///     section_catalog: SectionCatalog,
/// }
///
/// impl ZcadModule for SteelBimModule {
///     fn metadata(&self) -> ModuleMetadata {
///         ModuleMetadata::new("zcad.bim.steel", "Steel Structure BIM", ModuleCategory::Aec)
///             .with_version(Version::new(0, 1, 0))
///             .with_dependency(ModuleDependency::required("zcad.bim.core", Version::new(0, 1, 0)))
///     }
///
///     fn initialize(&mut self, ctx: &mut ModuleContext) -> Result<()> {
///         self.section_catalog = SectionCatalog::load_default()?;
///         Ok(())
///     }
///
///     fn register_commands(&self, registry: &mut CommandRegistry) {
///         registry.register("BEAM", BeamCommand::new());
///         registry.register("COLUMN", ColumnCommand::new());
///     }
/// }
/// ```
pub trait ZcadModule: Send + Sync {
    /// Get module metadata (identity, version, dependencies)
    fn metadata(&self) -> ModuleMetadata;

    /// Initialize the module.
    /// Called after all dependencies are loaded.
    fn initialize(&mut self, ctx: &mut ModuleContext) -> Result<()>;

    /// Register commands provided by this module.
    /// Called after initialization.
    fn register_commands(&self, _registry: &mut CommandRegistry) {
        // Default: no commands
    }

    /// Register element types provided by this module.
    /// Called after initialization.
    fn register_element_types(&self, _registry: &mut ElementTypeRegistry) {
        // Default: no element types
    }

    /// Called when the module is being unloaded.
    /// Clean up resources here.
    fn shutdown(&mut self) {
        // Default: no cleanup needed
    }

    /// Check if this module is enabled.
    /// Modules can be disabled without unloading.
    fn is_enabled(&self) -> bool {
        true
    }

    /// Enable or disable the module.
    fn set_enabled(&mut self, _enabled: bool) {
        // Default: ignore (always enabled)
    }
}

/// A module that can be dynamically loaded from a shared library.
/// This is for future plugin support.
pub trait DynamicModule: ZcadModule {
    /// Get the module factory function name.
    /// This is used to load the module from a dynamic library.
    fn factory_symbol() -> &'static str {
        "zcad_module_create"
    }
}

/// Module state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleState {
    /// Module is discovered but not loaded
    Discovered,
    /// Module is loading (resolving dependencies)
    Loading,
    /// Module is loaded and initialized
    Loaded,
    /// Module is enabled and active
    Active,
    /// Module is disabled (loaded but not active)
    Disabled,
    /// Module failed to load
    Failed,
    /// Module is unloading
    Unloading,
}

impl ModuleState {
    pub fn is_usable(&self) -> bool {
        matches!(self, ModuleState::Active)
    }
}
