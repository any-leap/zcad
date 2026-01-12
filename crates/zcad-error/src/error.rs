//! Core error types

use crate::code::ErrorCode;
use crate::location::ErrorLocation;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// The kind of error (for categorization)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorKind {
    /// Core system error
    Core,
    /// File I/O error
    File,
    /// Geometry error
    Geometry,
    /// Module system error
    Module,
    /// MCAD error
    Mcad,
    /// BIM error
    Bim,
    /// EDA error
    Eda,
    /// Rendering error
    Rendering,
    /// UI error
    Ui,
    /// Other/unknown
    Other,
}

impl ErrorKind {
    /// Get from error code
    pub fn from_code(code: ErrorCode) -> Self {
        match code.as_u16() {
            1000..=1999 => ErrorKind::Core,
            2000..=2999 => ErrorKind::File,
            3000..=3999 => ErrorKind::Geometry,
            4000..=4999 => ErrorKind::Module,
            5000..=5999 => ErrorKind::Mcad,
            6000..=6999 => ErrorKind::Bim,
            7000..=7999 => ErrorKind::Eda,
            8000..=8999 => ErrorKind::Rendering,
            9000..=9999 => ErrorKind::Ui,
            _ => ErrorKind::Other,
        }
    }
}

/// Context entry in the error chain
#[derive(Debug, Clone)]
pub struct ContextEntry {
    /// Context message
    pub message: String,
    /// Optional location
    pub location: Option<ErrorLocation>,
}

impl ContextEntry {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            location: None,
        }
    }

    pub fn with_location(mut self, location: ErrorLocation) -> Self {
        self.location = Some(location);
        self
    }
}

impl fmt::Display for ContextEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(loc) = &self.location {
            write!(f, "{} (at {})", self.message, loc)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

/// Unified error type for ZCAD
///
/// This error type provides:
/// - Error codes for categorization
/// - Context chain for debugging
/// - Source location tracking
/// - User-friendly messages
#[derive(Error, Debug)]
pub struct ZcadError {
    /// Error code
    code: ErrorCode,

    /// Primary error message
    message: String,

    /// Context chain (outermost first)
    context: Vec<ContextEntry>,

    /// Source location where error originated
    location: Option<ErrorLocation>,

    /// Source error (if wrapping another error)
    #[source]
    source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
}

impl fmt::Display for ZcadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Format: [E1001] Message
        write!(f, "[{}] {}", self.code, self.message)?;

        // Add context chain
        for ctx in &self.context {
            write!(f, "\n  caused by: {}", ctx)?;
        }

        // Add location if available
        if let Some(loc) = &self.location {
            write!(f, "\n  at {}", loc)?;
        }

        Ok(())
    }
}

impl ZcadError {
    /// Create a new error with code and message
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            context: Vec::new(),
            location: None,
            source: None,
        }
    }

    /// Create an error from an error code (uses default message)
    pub fn from_code(code: ErrorCode) -> Self {
        Self::new(code, code.description())
    }

    /// Add source location
    pub fn with_location(mut self, location: ErrorLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add a source error
    pub fn with_source<E>(mut self, source: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        self.source = Some(Box::new(source));
        self
    }

    /// Add context to this error
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context.push(ContextEntry::new(context));
        self
    }

    /// Get the error code
    pub fn code(&self) -> ErrorCode {
        self.code
    }

    /// Get the error kind
    pub fn kind(&self) -> ErrorKind {
        ErrorKind::from_code(self.code)
    }

    /// Get the primary message
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Get the context chain
    pub fn context_chain(&self) -> &[ContextEntry] {
        &self.context
    }

    /// Get the source location
    pub fn location(&self) -> Option<&ErrorLocation> {
        self.location.as_ref()
    }

    /// Check if this error is recoverable
    pub fn is_recoverable(&self) -> bool {
        self.code.is_recoverable()
    }

    // === Convenience constructors ===

    /// Create an IO error
    pub fn io(err: std::io::Error) -> Self {
        let code = match err.kind() {
            std::io::ErrorKind::NotFound => ErrorCode::FileNotFound,
            std::io::ErrorKind::PermissionDenied => ErrorCode::PermissionDenied,
            _ => ErrorCode::FileReadError,
        };
        Self::new(code, err.to_string()).with_source(err)
    }

    /// Create a "not found" error
    pub fn not_found(what: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, format!("Not found: {}", what.into()))
    }

    /// Create an "invalid argument" error
    pub fn invalid_argument(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidArgument, msg)
    }

    /// Create an "internal" error (for bugs)
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, msg)
    }

    /// Create a "not supported" error
    pub fn not_supported(feature: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::NotSupported,
            format!("Not supported: {}", feature.into()),
        )
    }

    /// Create a "cancelled" error
    pub fn cancelled() -> Self {
        Self::from_code(ErrorCode::Cancelled)
    }

    // === Domain-specific constructors ===

    /// Create a geometry error
    pub fn geometry(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidGeometry, msg)
    }

    /// Create a module error
    pub fn module_not_found(module_id: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::ModuleNotFound,
            format!("Module not found: {}", module_id.into()),
        )
    }

    /// Create a file format error
    pub fn invalid_format(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::InvalidFileFormat, msg)
    }

    /// Create a DRC violation error
    pub fn drc_violation(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::DrcViolation, msg)
    }

    /// Create an ERC violation error
    pub fn erc_violation(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::ErcViolation, msg)
    }
}

// === From implementations for common error types ===

impl From<std::io::Error> for ZcadError {
    fn from(err: std::io::Error) -> Self {
        ZcadError::io(err)
    }
}

impl From<ErrorCode> for ZcadError {
    fn from(code: ErrorCode) -> Self {
        ZcadError::from_code(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let err = ZcadError::new(ErrorCode::FileNotFound, "config.json not found");
        assert_eq!(err.code(), ErrorCode::FileNotFound);
        assert_eq!(err.message(), "config.json not found");
    }

    #[test]
    fn test_error_display() {
        let err = ZcadError::new(ErrorCode::FileNotFound, "config.json");
        let display = err.to_string();
        assert!(display.contains("[E2001]"));
        assert!(display.contains("config.json"));
    }

    #[test]
    fn test_error_with_context() {
        let err = ZcadError::new(ErrorCode::FileNotFound, "config.json")
            .with_context("Loading configuration")
            .with_context("Starting application");

        assert_eq!(err.context_chain().len(), 2);
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ZcadError = io_err.into();
        assert_eq!(err.code(), ErrorCode::FileNotFound);
    }
}
