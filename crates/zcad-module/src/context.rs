//! Module context - provides access to ZCAD services

use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Module context provides access to ZCAD core services.
///
/// Modules use this context to:
/// - Access shared services (file I/O, rendering, etc.)
/// - Store module-specific state
/// - Communicate with other modules
pub struct ModuleContext {
    /// Shared services registry
    services: HashMap<String, Arc<dyn Any + Send + Sync>>,

    /// Module-specific storage
    storage: RwLock<HashMap<String, Box<dyn Any + Send + Sync>>>,

    /// Configuration directory path
    config_dir: std::path::PathBuf,

    /// Data directory path
    data_dir: std::path::PathBuf,
}

impl ModuleContext {
    /// Create a new module context
    pub fn new(config_dir: std::path::PathBuf, data_dir: std::path::PathBuf) -> Self {
        Self {
            services: HashMap::new(),
            storage: RwLock::new(HashMap::new()),
            config_dir,
            data_dir,
        }
    }

    /// Get the configuration directory for this module
    pub fn config_dir(&self) -> &std::path::Path {
        &self.config_dir
    }

    /// Get the data directory for this module
    pub fn data_dir(&self) -> &std::path::Path {
        &self.data_dir
    }

    /// Register a shared service
    pub fn register_service<T: Any + Send + Sync + 'static>(
        &mut self,
        name: impl Into<String>,
        service: Arc<T>,
    ) {
        self.services.insert(name.into(), service);
    }

    /// Get a shared service by name
    pub fn get_service<T: Any + Send + Sync + 'static>(&self, name: &str) -> Option<Arc<T>> {
        self.services
            .get(name)
            .and_then(|s| s.clone().downcast::<T>().ok())
    }

    /// Store module-specific data
    pub fn store<T: Any + Send + Sync + 'static>(&self, key: impl Into<String>, value: T) {
        let mut storage = self.storage.write().unwrap();
        storage.insert(key.into(), Box::new(value));
    }

    /// Retrieve module-specific data
    pub fn retrieve<T: Any + Send + Sync + Clone + 'static>(&self, key: &str) -> Option<T> {
        let storage = self.storage.read().unwrap();
        storage
            .get(key)
            .and_then(|v| v.downcast_ref::<T>())
            .cloned()
    }

    /// Remove module-specific data
    pub fn remove(&self, key: &str) -> bool {
        let mut storage = self.storage.write().unwrap();
        storage.remove(key).is_some()
    }

    /// Check if a service exists
    pub fn has_service(&self, name: &str) -> bool {
        self.services.contains_key(name)
    }

    /// List all registered service names
    pub fn service_names(&self) -> Vec<&str> {
        self.services.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ModuleContext {
    fn default() -> Self {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("zcad");
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("zcad");

        Self::new(config_dir, data_dir)
    }
}

/// Service trait for typed service access
pub trait Service: Any + Send + Sync {
    /// Service name for registration
    fn service_name() -> &'static str
    where
        Self: Sized;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_storage() {
        let ctx = ModuleContext::default();
        
        ctx.store("test_key", 42i32);
        assert_eq!(ctx.retrieve::<i32>("test_key"), Some(42));
        
        ctx.store("string_key", String::from("hello"));
        assert_eq!(ctx.retrieve::<String>("string_key"), Some(String::from("hello")));
        
        assert!(ctx.remove("test_key"));
        assert_eq!(ctx.retrieve::<i32>("test_key"), None);
    }

    #[test]
    fn test_service_registration() {
        let mut ctx = ModuleContext::default();
        
        let service = Arc::new(String::from("test service"));
        ctx.register_service("my_service", service);
        
        assert!(ctx.has_service("my_service"));
        
        let retrieved: Option<Arc<String>> = ctx.get_service("my_service");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().as_str(), "test service");
    }
}
