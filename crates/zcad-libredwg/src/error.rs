//! Error types for LibreDWG operations

use thiserror::Error;

/// Result type for LibreDWG operations
pub type Result<T> = std::result::Result<T, DwgError>;

/// Errors that can occur when working with DWG files
#[derive(Debug, Error)]
pub enum DwgError {
    /// File not found
    #[error("DWG file not found: {0}")]
    FileNotFound(String),

    /// Failed to open the DWG file
    #[error("Failed to open DWG file: {0}")]
    OpenFailed(String),

    /// Failed to read the DWG file
    #[error("Failed to read DWG file: {0}")]
    ReadFailed(String),

    /// Unsupported DWG version
    #[error("Unsupported DWG version: {0}")]
    UnsupportedVersion(String),

    /// Invalid or corrupted DWG file
    #[error("Invalid or corrupted DWG file: {0}")]
    InvalidFile(String),

    /// Entity conversion error
    #[error("Failed to convert entity: {0}")]
    EntityConversion(String),

    /// LibreDWG internal error
    #[error("LibreDWG error code: {0}")]
    LibreDwgError(i32),

    /// LibreDWG is not available
    #[error("LibreDWG library is not available. Please install it.")]
    NotAvailable,
}

impl DwgError {
    /// Create error from LibreDWG error code
    pub fn from_error_code(code: i32) -> Self {
        match code {
            0 => panic!("from_error_code called with success code"),
            1 => DwgError::ReadFailed("Out of memory".to_string()),
            2 => DwgError::InvalidFile("Invalid DWG format".to_string()),
            3 => DwgError::UnsupportedVersion("DWG version not supported".to_string()),
            _ => DwgError::LibreDwgError(code),
        }
    }
}
