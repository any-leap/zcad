//! ZCAD BIM Module
//!
//! Building Information Modeling module for architecture, engineering,
//! and construction (AEC) industry.
//!
//! # Sub-modules
//!
//! - **Steel Structure**: Steel beams, columns, plates, connections
//! - **Concrete**: Reinforced concrete elements, rebar detailing
//! - **Architecture**: Walls, floors, roofs, doors, windows
//! - **MEP**: Mechanical, electrical, plumbing systems
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │               zcad-bim                   │
//! ├──────────────────────────────────────────┤
//! │  core/         - BIM core concepts       │
//! │    element.rs  - Building elements       │
//! │    spatial.rs  - Spatial hierarchy       │
//! │    relation.rs - Element relationships   │
//! │  steel/        - Steel structure         │
//! │  concrete/     - Reinforced concrete     │
//! │  arch/         - Architecture            │
//! │  mep/          - MEP systems             │
//! │  ifc/          - IFC interoperability    │
//! └──────────────────────────────────────────┘
//! ```

pub mod arch;
pub mod concrete;
pub mod core;
pub mod error;
pub mod ifc;
pub mod mep;
pub mod module;
pub mod steel;

pub mod prelude {
    //! Convenient re-exports for BIM development
    pub use crate::core::{
        BimElement, BimElementId, BuildingStorey, Level, Project, Site, SpatialElement,
    };
    pub use crate::error::{BimError, Result};
    pub use crate::steel::{Beam, Column, Plate, SteelConnection, SteelSection};
}
