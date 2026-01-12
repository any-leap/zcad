//! MCAD error types

use thiserror::Error;

/// MCAD errors
#[derive(Error, Debug)]
pub enum McadError {
    #[error("Part not found: {0}")]
    PartNotFound(String),

    #[error("Feature failed: {0}")]
    FeatureFailed(String),

    #[error("Sketch error: {0}")]
    SketchError(String),

    #[error("Assembly constraint failed: {0}")]
    ConstraintFailed(String),

    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    #[error("Material not found: {0}")]
    MaterialNotFound(String),

    #[error("Circular dependency in feature tree")]
    CircularDependency,

    #[error("Geometry error: {0}")]
    GeometryError(#[from] zcad_geometry_3d::error::GeometryError),

    #[error("OCCT error: {0}")]
    OcctError(#[from] zcad_occt::error::OcctError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for MCAD operations
pub type Result<T> = std::result::Result<T, McadError>;
