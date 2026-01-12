//! EDA error types

use thiserror::Error;

/// EDA errors
#[derive(Error, Debug)]
pub enum EdaError {
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    #[error("Symbol not found: {0}")]
    SymbolNotFound(String),

    #[error("Footprint not found: {0}")]
    FootprintNotFound(String),

    #[error("Net error: {0}")]
    NetError(String),

    #[error("DRC violation: {0}")]
    DrcViolation(String),

    #[error("ERC violation: {0}")]
    ErcViolation(String),

    #[error("Invalid pin: {0}")]
    InvalidPin(String),

    #[error("Invalid layer: {0}")]
    InvalidLayer(String),

    #[error("Gerber export error: {0}")]
    GerberError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for EDA operations
pub type Result<T> = std::result::Result<T, EdaError>;
