//! Error types for 3D geometry operations

use thiserror::Error;

/// 3D geometry errors
#[derive(Error, Debug)]
pub enum GeometryError {
    #[error("Invalid geometry: {0}")]
    InvalidGeometry(String),

    #[error("Degenerate geometry: {0}")]
    DegenerateGeometry(String),

    #[error("Boolean operation failed: {0}")]
    BooleanFailed(String),

    #[error("Topology error: {0}")]
    TopologyError(String),

    #[error("Tessellation failed: {0}")]
    TessellationFailed(String),

    #[error("Transform error: {0}")]
    TransformError(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("OCCT error: {0}")]
    OcctError(String),
}

/// Result type for geometry operations
pub type Result<T> = std::result::Result<T, GeometryError>;
