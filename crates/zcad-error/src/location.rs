//! Error location tracking
//!
//! Captures file, line, and column information for debugging.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Location where an error occurred
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorLocation {
    /// File path
    pub file: &'static str,
    /// Line number
    pub line: u32,
    /// Column number
    pub column: u32,
}

impl ErrorLocation {
    /// Create a new error location
    pub const fn new(file: &'static str, line: u32, column: u32) -> Self {
        Self { file, line, column }
    }
}

impl fmt::Display for ErrorLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Macro to capture the current source location
#[macro_export]
macro_rules! here {
    () => {
        $crate::location::ErrorLocation::new(file!(), line!(), column!())
    };
}

/// Macro to create a ZcadError with location
#[macro_export]
macro_rules! zcad_error {
    ($code:expr, $msg:expr) => {
        $crate::error::ZcadError::new($code, $msg).with_location($crate::here!())
    };
    ($code:expr, $fmt:expr, $($arg:tt)*) => {
        $crate::error::ZcadError::new($code, format!($fmt, $($arg)*)).with_location($crate::here!())
    };
}

/// Macro to create a ZcadError and return early
#[macro_export]
macro_rules! bail {
    ($code:expr, $msg:expr) => {
        return Err($crate::zcad_error!($code, $msg))
    };
    ($code:expr, $fmt:expr, $($arg:tt)*) => {
        return Err($crate::zcad_error!($code, $fmt, $($arg)*))
    };
}

/// Macro to ensure a condition, otherwise return an error
#[macro_export]
macro_rules! ensure {
    ($cond:expr, $code:expr, $msg:expr) => {
        if !$cond {
            $crate::bail!($code, $msg);
        }
    };
    ($cond:expr, $code:expr, $fmt:expr, $($arg:tt)*) => {
        if !$cond {
            $crate::bail!($code, $fmt, $($arg)*);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_display() {
        let loc = ErrorLocation::new("src/lib.rs", 42, 10);
        assert_eq!(loc.to_string(), "src/lib.rs:42:10");
    }

    #[test]
    fn test_here_macro() {
        let loc = here!();
        assert!(loc.file.ends_with("location.rs"));
        assert!(loc.line > 0);
    }
}
