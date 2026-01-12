//! ZCAD Unified Error Handling
//!
//! This crate provides a unified error handling system for ZCAD, including:
//!
//! - **Error codes**: Numeric codes for documentation and i18n
//! - **Error context**: Chain of context for debugging
//! - **User-friendly display**: Separate messages for users vs developers
//! - **Tracing integration**: Automatic error logging
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                      ZcadError                               │
//! │  (Top-level error type that wraps all domain errors)        │
//! ├─────────────────────────────────────────────────────────────┤
//! │                                                              │
//! │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐       │
//! │  │ CoreError│ │FileError │ │GeomError │ │ ModError │ ...   │
//! │  └──────────┘ └──────────┘ └──────────┘ └──────────┘       │
//! │                                                              │
//! │  ┌────────────────────────────────────────────────────────┐ │
//! │  │                    ErrorContext                         │ │
//! │  │  (Provides .context() and .with_context() methods)     │ │
//! │  └────────────────────────────────────────────────────────┘ │
//! │                                                              │
//! │  ┌────────────────────────────────────────────────────────┐ │
//! │  │                   UserFriendly                          │ │
//! │  │  (Provides user_message(), suggestion(), etc.)         │ │
//! │  └────────────────────────────────────────────────────────┘ │
//! │                                                              │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use zcad_error::prelude::*;
//!
//! fn load_file(path: &Path) -> Result<Data, ZcadError> {
//!     let content = std::fs::read(path)
//!         .map_err(|e| ZcadError::io(e))
//!         .with_context(|| format!("Failed to read file: {:?}", path))?;
//!     
//!     parse_data(&content)
//!         .context("Invalid file format")
//! }
//! ```

pub mod code;
pub mod context;
pub mod display;
pub mod error;
pub mod location;

pub mod prelude {
    //! Convenient re-exports for error handling
    pub use crate::code::ErrorCode;
    pub use crate::context::{ErrorContext, ResultExt};
    pub use crate::display::UserFriendly;
    pub use crate::error::{ErrorKind, ZcadError};
    pub use crate::location::ErrorLocation;
    pub use crate::Result;
}

/// Standard Result type for ZCAD operations
pub type Result<T, E = error::ZcadError> = std::result::Result<T, E>;
