//! ZCAD OpenCASCADE Bindings
//!
//! This crate provides bindings to OpenCASCADE Technology (OCCT) for
//! advanced solid modeling operations.
//!
//! # Features
//!
//! - **Boolean operations**: Union, intersection, difference
//! - **Filleting and chamfering**: Edge rounding and beveling
//! - **Extrusion and revolution**: Creating solids from profiles
//! - **Sweeping**: Solids from path + profile
//! - **STEP/IGES I/O**: CAD file format support
//!
//! # Architecture
//!
//! This crate wraps OCCT functionality and converts between OCCT types
//! and ZCAD's native `zcad-geometry-3d` types.
//!
//! When OCCT is not available (feature disabled), operations fall back
//! to simplified implementations or return errors.
//!
//! # Example
//!
//! ```rust,ignore
//! use zcad_occt::prelude::*;
//!
//! // Create a box
//! let box_shape = OcctShape::make_box(100.0, 50.0, 30.0)?;
//!
//! // Create a cylinder
//! let cylinder = OcctShape::make_cylinder(25.0, 60.0)?;
//!
//! // Boolean subtraction
//! let result = box_shape.boolean_cut(&cylinder)?;
//!
//! // Fillet edges
//! let filleted = result.fillet_all_edges(5.0)?;
//! ```

pub mod boolean;
pub mod builder;
pub mod error;
pub mod export;
pub mod fillet;
pub mod shape;
pub mod tessellate;

pub mod prelude {
    //! Convenient re-exports for OCCT operations
    pub use crate::boolean::BooleanOp;
    pub use crate::builder::{ExtrudeBuilder, RevolveBuilder, SweepBuilder};
    pub use crate::error::{OcctError, Result};
    pub use crate::fillet::{ChamferBuilder, FilletBuilder};
    pub use crate::shape::OcctShape;
    pub use crate::tessellate::{TessellationParams, Tessellator};
}
