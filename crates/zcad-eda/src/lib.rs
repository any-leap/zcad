//! ZCAD EDA Module
//!
//! Electronic Design Automation module for schematic capture and PCB design.
//!
//! # Features
//!
//! - **Schematic Capture**: Circuit diagram creation and editing
//! - **Component Library**: Symbol, footprint, and 3D model management
//! - **PCB Layout**: Board design with multi-layer support
//! - **Design Rules**: DRC (Design Rule Check) and ERC (Electrical Rule Check)
//! - **Output**: Gerber, drill files, BOM generation
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │               zcad-eda                   │
//! ├──────────────────────────────────────────┤
//! │  component/    - Component library       │
//! │  schematic/    - Schematic capture       │
//! │  pcb/          - PCB layout              │
//! │  netlist/      - Netlist management      │
//! │  drc/          - Design rule checks      │
//! │  output/       - Manufacturing output    │
//! └──────────────────────────────────────────┘
//! ```

pub mod component;
pub mod drc;
pub mod error;
pub mod module;
pub mod netlist;
pub mod output;
pub mod pcb;
pub mod schematic;

pub mod prelude {
    //! Convenient re-exports for EDA development
    pub use crate::component::{Component, ComponentId, Footprint, Symbol};
    pub use crate::error::{EdaError, Result};
    pub use crate::netlist::{Net, NetId, Netlist};
    pub use crate::pcb::{Board, Layer, LayerId, Pad, Track, Via};
    pub use crate::schematic::{SchematicPage, SchematicSymbol, Wire};
}
