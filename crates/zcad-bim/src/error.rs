//! BIM error types

use thiserror::Error;

/// BIM errors
#[derive(Error, Debug)]
pub enum BimError {
    #[error("Element not found: {0}")]
    ElementNotFound(String),

    #[error("Invalid element configuration: {0}")]
    InvalidConfiguration(String),

    #[error("Section not found: {0}")]
    SectionNotFound(String),

    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    #[error("IFC error: {0}")]
    IfcError(String),

    #[error("Spatial error: {0}")]
    SpatialError(String),

    #[error("Geometry error: {0}")]
    GeometryError(#[from] zcad_geometry_3d::error::GeometryError),

    #[error("OCCT error: {0}")]
    OcctError(#[from] zcad_occt::error::OcctError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for BIM operations
pub type Result<T> = std::result::Result<T, BimError>;
