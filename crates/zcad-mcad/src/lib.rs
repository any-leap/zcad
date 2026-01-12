//! ZCAD MCAD Module
//!
//! Mechanical Computer-Aided Design module for ZCAD.
//!
//! # Features
//!
//! - **Part Design**: Feature-based solid modeling
//! - **Assembly Design**: Component relationships and constraints
//! - **Drawing Generation**: 2D views from 3D models
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │              zcad-mcad                   │
//! ├──────────────────────────────────────────┤
//! │  part/           - Part design           │
//! │    features/     - Modeling features     │
//! │    sketch/       - 2D sketches           │
//! │  assembly/       - Assembly design       │
//! │  drawing/        - Engineering drawings  │
//! │  material/       - Material properties   │
//! │  tolerance/      - GD&T                  │
//! └──────────────────────────────────────────┘
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use zcad_mcad::prelude::*;
//!
//! // Create a new part
//! let mut part = Part::new("Bracket");
//!
//! // Add a base extrusion
//! let sketch = Sketch::rectangle(100.0, 50.0);
//! part.add_feature(Feature::extrude(&sketch, 10.0));
//!
//! // Add a hole
//! part.add_feature(Feature::hole(Point3::new(25.0, 25.0, 10.0), 5.0, 10.0));
//! ```

pub mod assembly;
pub mod drawing;
pub mod error;
pub mod feature;
pub mod material;
pub mod module;
pub mod part;
pub mod sketch;
pub mod tolerance;

pub mod prelude {
    //! Convenient re-exports for MCAD development
    pub use crate::assembly::{Assembly, AssemblyConstraint, Component};
    pub use crate::error::{McadError, Result};
    pub use crate::feature::{Feature, FeatureId, FeatureTree};
    pub use crate::material::{Material, MaterialLibrary};
    pub use crate::part::{Part, PartId};
    pub use crate::sketch::{Sketch, SketchConstraint, SketchEntity};
    pub use crate::tolerance::{Tolerance, ToleranceType};
}
