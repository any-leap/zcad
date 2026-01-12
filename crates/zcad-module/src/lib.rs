//! ZCAD Module System
//!
//! This crate provides the plugin/module architecture for ZCAD, enabling
//! domain-specific extensions such as:
//! - MCAD (Mechanical CAD)
//! - AEC/BIM (Architecture, Engineering, Construction)
//! - EDA (Electronic Design Automation)
//!
//! # Architecture
//!
//! The module system follows a layered architecture:
//! - Layer 0: Foundation (geometry, rendering, UI framework)
//! - Layer 1: CAD Core (document, parametric, module system)
//! - Layer 2: Domain Modules (MCAD, BIM, EDA, etc.)
//!
//! # Example
//!
//! ```rust,ignore
//! use zcad_module::prelude::*;
//!
//! // Define a custom module
//! pub struct MyModule;
//!
//! impl ZcadModule for MyModule {
//!     fn metadata(&self) -> ModuleMetadata {
//!         ModuleMetadata {
//!             id: "com.example.mymodule",
//!             name: "My Custom Module",
//!             version: Version::new(1, 0, 0),
//!             category: ModuleCategory::ThirdParty,
//!             dependencies: vec![],
//!             description: "A custom ZCAD module",
//!         }
//!     }
//!     
//!     fn initialize(&mut self, ctx: &mut ModuleContext) -> Result<()> {
//!         // Register commands, UI elements, etc.
//!         Ok(())
//!     }
//! }
//! ```

pub mod command;
pub mod context;
pub mod element;
pub mod error;
pub mod metadata;
pub mod registry;
pub mod traits;
pub mod workspace;

pub mod prelude {
    //! Convenient re-exports for module development
    pub use crate::command::{Command, CommandCategory, CommandContext, CommandResult, CommandRegistry};
    pub use crate::context::ModuleContext;
    pub use crate::element::{ElementDomain, ElementType, ElementTypeId, ElementTypeRegistry, PropertyDefinition};
    pub use crate::error::{ModuleError, Result};
    pub use crate::metadata::{ModuleCategory, ModuleDependency, ModuleMetadata, Version};
    pub use crate::registry::ModuleRegistry;
    pub use crate::traits::ZcadModule;
    pub use crate::workspace::{Workspace, WorkspaceType};
}
