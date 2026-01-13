//! ZCAD LibreDWG bindings
//!
//! This crate provides Rust bindings to the LibreDWG library for reading DWG files.
//!
//! # Usage
//!
//! ```rust,ignore
//! use zcad_libredwg::DwgFile;
//!
//! let dwg = DwgFile::open("drawing.dwg")?;
//! for entity in dwg.entities() {
//!     println!("{:?}", entity);
//! }
//! ```
//!
//! # Requirements
//!
//! LibreDWG must be installed on the system:
//! - Windows: `vcpkg install libredwg` or set `LIBREDWG_DIR`
//! - macOS: `brew install libredwg`
//! - Linux: `apt install libredwg-dev`

mod dwg;
mod entity;
mod error;
#[allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code,
    clippy::all
)]
mod sys;

pub use dwg::DwgFile;
pub use entity::{DwgEntity, DwgEntityType, DwgPoint2, DwgPoint3, DwgColor};
pub use error::{DwgError, Result};
