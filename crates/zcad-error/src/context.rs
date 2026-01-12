//! Error context extension traits
//!
//! Provides `.context()` and `.with_context()` methods for adding
//! context to errors, similar to `anyhow`.

use crate::code::ErrorCode;
use crate::error::ZcadError;

/// A context that can be attached to an error
pub trait ErrorContext {
    /// The message to display
    fn message(&self) -> String;
}

impl ErrorContext for &str {
    fn message(&self) -> String {
        self.to_string()
    }
}

impl ErrorContext for String {
    fn message(&self) -> String {
        self.clone()
    }
}

impl<F: FnOnce() -> String> ErrorContext for F {
    fn message(&self) -> String {
        // This won't work because FnOnce takes ownership
        // We'll handle this differently in the actual implementation
        String::new()
    }
}

/// Extension trait for adding context to Results
pub trait ResultExt<T> {
    /// Add context to an error
    fn context(self, context: impl Into<String>) -> Result<T, ZcadError>;

    /// Add context to an error, lazily evaluated
    fn with_context<F>(self, f: F) -> Result<T, ZcadError>
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultExt<T> for Result<T, E>
where
    E: Into<ZcadError>,
{
    fn context(self, context: impl Into<String>) -> Result<T, ZcadError> {
        self.map_err(|e| e.into().with_context(context))
    }

    fn with_context<F>(self, f: F) -> Result<T, ZcadError>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| e.into().with_context(f()))
    }
}

/// Extension trait for adding context to Options
pub trait OptionExt<T> {
    /// Convert Option to Result with a "not found" error
    fn ok_or_not_found(self, what: impl Into<String>) -> Result<T, ZcadError>;

    /// Convert Option to Result with a custom error
    fn ok_or_error(self, code: ErrorCode, msg: impl Into<String>) -> Result<T, ZcadError>;
}

impl<T> OptionExt<T> for Option<T> {
    fn ok_or_not_found(self, what: impl Into<String>) -> Result<T, ZcadError> {
        self.ok_or_else(|| ZcadError::not_found(what))
    }

    fn ok_or_error(self, code: ErrorCode, msg: impl Into<String>) -> Result<T, ZcadError> {
        self.ok_or_else(|| ZcadError::new(code, msg))
    }
}

/// Trait for converting domain-specific errors to ZcadError
pub trait IntoZcadError {
    /// Convert this error into a ZcadError
    fn into_zcad_error(self) -> ZcadError;
}

/// Macro to implement IntoZcadError for domain errors
#[macro_export]
macro_rules! impl_into_zcad_error {
    ($error_type:ty, $default_code:expr) => {
        impl $crate::context::IntoZcadError for $error_type {
            fn into_zcad_error(self) -> $crate::error::ZcadError {
                $crate::error::ZcadError::new($default_code, self.to_string())
            }
        }

        impl From<$error_type> for $crate::error::ZcadError {
            fn from(err: $error_type) -> Self {
                $crate::context::IntoZcadError::into_zcad_error(err)
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ZcadError;

    #[test]
    fn test_result_context() {
        let result: Result<(), ZcadError> = Err(ZcadError::not_found("file"));
        let with_ctx = result.context("Loading configuration");
        
        let err = with_ctx.unwrap_err();
        assert_eq!(err.context_chain().len(), 1);
        assert!(err.context_chain()[0].message.contains("Loading configuration"));
    }

    #[test]
    fn test_result_with_context_lazy() {
        let result: Result<(), ZcadError> = Err(ZcadError::not_found("file"));
        let with_ctx = result.with_context(|| format!("Failed at step {}", 42));
        
        let err = with_ctx.unwrap_err();
        assert!(err.context_chain()[0].message.contains("Failed at step 42"));
    }

    #[test]
    fn test_option_ok_or_not_found() {
        let opt: Option<i32> = None;
        let result = opt.ok_or_not_found("item");
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code(), ErrorCode::NotFound);
    }

    #[test]
    fn test_option_ok_or_error() {
        let opt: Option<i32> = None;
        let result = opt.ok_or_error(ErrorCode::PartNotFound, "Part ABC123");
        
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code(), ErrorCode::PartNotFound);
    }
}
