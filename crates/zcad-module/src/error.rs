//! Module system error types

use thiserror::Error;

/// Module system errors
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Module not found: {0}")]
    NotFound(String),

    #[error("Module already loaded: {0}")]
    AlreadyLoaded(String),

    #[error("Module dependency not satisfied: {module} requires {dependency}")]
    DependencyNotSatisfied { module: String, dependency: String },

    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),

    #[error("Module initialization failed: {0}")]
    InitializationFailed(String),

    #[error("Module version incompatible: {module} requires {required}, found {found}")]
    VersionIncompatible {
        module: String,
        required: String,
        found: String,
    },

    #[error("Invalid module configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Command already registered: {0}")]
    CommandAlreadyRegistered(String),

    #[error("Element type already registered: {0}")]
    ElementTypeAlreadyRegistered(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other error: {0}")]
    Other(String),
}

/// Result type for module operations
pub type Result<T> = std::result::Result<T, ModuleError>;
