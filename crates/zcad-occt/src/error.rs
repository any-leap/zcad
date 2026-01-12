//! Error types for OCCT operations

use thiserror::Error;

/// OCCT operation errors
#[derive(Error, Debug)]
pub enum OcctError {
    #[error("Shape is null or invalid")]
    NullShape,

    #[error("Boolean operation failed: {0}")]
    BooleanFailed(String),

    #[error("Fillet operation failed: {0}")]
    FilletFailed(String),

    #[error("Chamfer operation failed: {0}")]
    ChamferFailed(String),

    #[error("Extrusion failed: {0}")]
    ExtrusionFailed(String),

    #[error("Revolution failed: {0}")]
    RevolutionFailed(String),

    #[error("Sweep failed: {0}")]
    SweepFailed(String),

    #[error("Tessellation failed: {0}")]
    TessellationFailed(String),

    #[error("Export failed: {0}")]
    ExportFailed(String),

    #[error("Import failed: {0}")]
    ImportFailed(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("OCCT not available (feature disabled)")]
    OcctNotAvailable,

    #[error("Geometry error: {0}")]
    GeometryError(#[from] zcad_geometry_3d::error::GeometryError),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for OCCT operations
pub type Result<T> = std::result::Result<T, OcctError>;
