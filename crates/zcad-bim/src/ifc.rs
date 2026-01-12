//! IFC (Industry Foundation Classes) interoperability
//!
//! Export and import of IFC files for BIM data exchange.

use crate::error::{BimError, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// IFC schema version
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IfcSchema {
    Ifc2x3,
    Ifc4,
    Ifc4x3,
}

impl Default for IfcSchema {
    fn default() -> Self {
        IfcSchema::Ifc4
    }
}

/// IFC export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfcExportOptions {
    /// Schema version
    pub schema: IfcSchema,

    /// Include geometry
    pub include_geometry: bool,

    /// Include property sets
    pub include_properties: bool,

    /// Include quantities
    pub include_quantities: bool,

    /// Include relationships
    pub include_relationships: bool,

    /// Export coordinate system
    pub coordinate_system: CoordinateSystem,

    /// Application name
    pub application_name: String,

    /// Application version
    pub application_version: String,
}

impl Default for IfcExportOptions {
    fn default() -> Self {
        Self {
            schema: IfcSchema::Ifc4,
            include_geometry: true,
            include_properties: true,
            include_quantities: true,
            include_relationships: true,
            coordinate_system: CoordinateSystem::WorldCoordinates,
            application_name: "ZCAD".to_string(),
            application_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Coordinate system for export
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CoordinateSystem {
    /// World coordinates (absolute)
    WorldCoordinates,
    /// Project coordinates (relative to project origin)
    ProjectCoordinates,
    /// Shared coordinates (for multi-model coordination)
    SharedCoordinates,
}

/// IFC import options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfcImportOptions {
    /// Import geometry
    pub import_geometry: bool,

    /// Import property sets
    pub import_properties: bool,

    /// Import spatial structure
    pub import_spatial: bool,

    /// Element types to include (empty = all)
    pub include_types: Vec<String>,

    /// Element types to exclude
    pub exclude_types: Vec<String>,
}

impl Default for IfcImportOptions {
    fn default() -> Self {
        Self {
            import_geometry: true,
            import_properties: true,
            import_spatial: true,
            include_types: Vec::new(),
            exclude_types: Vec::new(),
        }
    }
}

/// IFC exporter
pub struct IfcExporter {
    options: IfcExportOptions,
}

impl IfcExporter {
    pub fn new() -> Self {
        Self {
            options: IfcExportOptions::default(),
        }
    }

    pub fn with_options(options: IfcExportOptions) -> Self {
        Self { options }
    }

    /// Export to IFC file
    pub fn export<P: AsRef<Path>>(&self, _path: P) -> Result<()> {
        // TODO: Implement IFC export
        // This would use an IFC library to write the file
        Err(BimError::IfcError(
            "IFC export not yet implemented".to_string(),
        ))
    }
}

impl Default for IfcExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// IFC importer
pub struct IfcImporter {
    options: IfcImportOptions,
}

impl IfcImporter {
    pub fn new() -> Self {
        Self {
            options: IfcImportOptions::default(),
        }
    }

    pub fn with_options(options: IfcImportOptions) -> Self {
        Self { options }
    }

    /// Import from IFC file
    pub fn import<P: AsRef<Path>>(&self, _path: P) -> Result<IfcModel> {
        // TODO: Implement IFC import
        Err(BimError::IfcError(
            "IFC import not yet implemented".to_string(),
        ))
    }
}

impl Default for IfcImporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Imported IFC model
#[derive(Debug, Default)]
pub struct IfcModel {
    /// Schema version of the imported file
    pub schema: IfcSchema,

    /// Project information
    pub project_name: Option<String>,

    /// Number of elements imported
    pub element_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_options() {
        let options = IfcExportOptions::default();
        assert_eq!(options.schema, IfcSchema::Ifc4);
        assert!(options.include_geometry);
    }
}
