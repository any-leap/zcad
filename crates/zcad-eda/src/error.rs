//! EDA error types
//!
//! Domain-specific errors for EDA operations, with integration
//! into the unified ZCAD error system.

use thiserror::Error;
use zcad_error::code::ErrorCode;
use zcad_error::error::ZcadError;

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

// === Integration with ZcadError ===

impl From<EdaError> for ZcadError {
    fn from(err: EdaError) -> Self {
        let (code, message) = match &err {
            EdaError::ComponentNotFound(name) => {
                (ErrorCode::ComponentNotFound, format!("Component not found: {}", name))
            }
            EdaError::SymbolNotFound(name) => {
                (ErrorCode::SymbolNotFound, format!("Symbol not found: {}", name))
            }
            EdaError::FootprintNotFound(name) => {
                (ErrorCode::FootprintNotFound, format!("Footprint not found: {}", name))
            }
            EdaError::NetError(msg) => {
                (ErrorCode::NetError, format!("Net error: {}", msg))
            }
            EdaError::DrcViolation(msg) => {
                (ErrorCode::DrcViolation, format!("DRC violation: {}", msg))
            }
            EdaError::ErcViolation(msg) => {
                (ErrorCode::ErcViolation, format!("ERC violation: {}", msg))
            }
            EdaError::InvalidPin(name) => {
                (ErrorCode::InvalidPin, format!("Invalid pin: {}", name))
            }
            EdaError::InvalidLayer(name) => {
                (ErrorCode::InvalidLayer, format!("Invalid layer: {}", name))
            }
            EdaError::GerberError(msg) => {
                (ErrorCode::GerberError, format!("Gerber error: {}", msg))
            }
            EdaError::IoError(e) => {
                (ErrorCode::FileReadError, format!("IO error: {}", e))
            }
        };
        
        ZcadError::new(code, message)
    }
}

/// Extension trait for converting EDA results to ZcadError results
pub trait EdaResultExt<T> {
    /// Convert to ZcadError result with context
    fn into_zcad(self) -> std::result::Result<T, ZcadError>;
    
    /// Convert to ZcadError result with additional context
    fn into_zcad_with_context(self, context: impl Into<String>) -> std::result::Result<T, ZcadError>;
}

impl<T> EdaResultExt<T> for Result<T> {
    fn into_zcad(self) -> std::result::Result<T, ZcadError> {
        self.map_err(|e| e.into())
    }
    
    fn into_zcad_with_context(self, context: impl Into<String>) -> std::result::Result<T, ZcadError> {
        self.map_err(|e| {
            let zcad_err: ZcadError = e.into();
            zcad_err.with_context(context)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eda_error_to_zcad_error() {
        let eda_err = EdaError::DrcViolation("Track too narrow".to_string());
        let zcad_err: ZcadError = eda_err.into();
        
        assert_eq!(zcad_err.code(), ErrorCode::DrcViolation);
        assert!(zcad_err.message().contains("Track too narrow"));
    }

    #[test]
    fn test_result_extension() {
        let result: Result<()> = Err(EdaError::ComponentNotFound("R1".to_string()));
        let zcad_result = result.into_zcad_with_context("Loading schematic");
        
        let err = zcad_result.unwrap_err();
        assert_eq!(err.code(), ErrorCode::ComponentNotFound);
        assert!(!err.context_chain().is_empty());
    }
}
