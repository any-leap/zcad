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
        // LibreDWG error codes (from dwg.h DWG_ERROR enum)
        match code {
            0 => panic!("from_error_code called with success code"),
            1 => DwgError::ReadFailed("Out of memory".to_string()),
            2 => DwgError::InvalidFile("Invalid DWG format".to_string()),
            3 => DwgError::UnsupportedVersion("DWG version not supported by LibreDWG".to_string()),
            4 => DwgError::ReadFailed("Invalid type".to_string()),
            8 => DwgError::ReadFailed("Invalid handle".to_string()),
            16 => DwgError::ReadFailed("Invalid code".to_string()),
            32 => DwgError::ReadFailed("Invalid class".to_string()),
            64 => DwgError::ReadFailed("Invalid object".to_string()),
            128 => DwgError::InvalidFile("Section not found".to_string()),
            256 => DwgError::InvalidFile("Page not found".to_string()),
            512 => DwgError::InvalidFile("Unknown class".to_string()),
            1024 => DwgError::InvalidFile("Unknown entity type".to_string()),
            2048 => DwgError::InvalidFile("Unhandled class".to_string()),
            4096 => DwgError::UnsupportedVersion(
                "DWG version not supported. Try saving as AutoCAD 2013 or older format, or convert to DXF.".to_string()
            ),
            8192 => DwgError::ReadFailed("Invalid bitcode".to_string()),
            16384 => DwgError::InvalidFile("Invalid layer index".to_string()),
            32768 => DwgError::ReadFailed("Decode error".to_string()),
            _ => DwgError::LibreDwgError(code),
        }
    }
}
