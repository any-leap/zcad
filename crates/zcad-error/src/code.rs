//! Error codes for documentation and internationalization
//!
//! Error codes are organized by category:
//! - 1000-1999: Core/System errors
//! - 2000-2999: File I/O errors
//! - 3000-3999: Geometry errors
//! - 4000-4999: Module system errors
//! - 5000-5999: MCAD errors
//! - 6000-6999: BIM errors
//! - 7000-7999: EDA errors
//! - 8000-8999: Rendering errors
//! - 9000-9999: UI errors

use serde::{Deserialize, Serialize};
use std::fmt;

/// Error code for categorization and documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(u16)]
pub enum ErrorCode {
    // === Core/System errors (1000-1999) ===
    /// Unknown error
    Unknown = 1000,
    /// Operation cancelled by user
    Cancelled = 1001,
    /// Operation timed out
    Timeout = 1002,
    /// Invalid argument provided
    InvalidArgument = 1003,
    /// Operation not supported
    NotSupported = 1004,
    /// Internal error (bug)
    Internal = 1005,
    /// Resource not found
    NotFound = 1006,
    /// Resource already exists
    AlreadyExists = 1007,
    /// Permission denied
    PermissionDenied = 1008,

    // === File I/O errors (2000-2999) ===
    /// File not found
    FileNotFound = 2001,
    /// Failed to read file
    FileReadError = 2002,
    /// Failed to write file
    FileWriteError = 2003,
    /// Invalid file format
    InvalidFileFormat = 2004,
    /// Unsupported file version
    UnsupportedVersion = 2005,
    /// File is corrupted
    FileCorrupted = 2006,
    /// DXF parsing error
    DxfError = 2007,
    /// STEP/IGES error
    StepError = 2008,
    /// IFC error
    IfcError = 2009,

    // === Geometry errors (3000-3999) ===
    /// Invalid geometry
    InvalidGeometry = 3001,
    /// Degenerate geometry (zero-length line, etc.)
    DegenerateGeometry = 3002,
    /// Boolean operation failed
    BooleanFailed = 3003,
    /// Topology error
    TopologyError = 3004,
    /// Tessellation failed
    TessellationFailed = 3005,
    /// Transform error
    TransformError = 3006,
    /// Constraint solver failed
    ConstraintSolverFailed = 3007,

    // === Module system errors (4000-4999) ===
    /// Module not found
    ModuleNotFound = 4001,
    /// Module already loaded
    ModuleAlreadyLoaded = 4002,
    /// Module dependency not satisfied
    DependencyNotSatisfied = 4003,
    /// Circular dependency detected
    CircularDependency = 4004,
    /// Module version incompatible
    VersionIncompatible = 4005,
    /// Module initialization failed
    ModuleInitFailed = 4006,
    /// Command not found
    CommandNotFound = 4007,
    /// Command already registered
    CommandAlreadyRegistered = 4008,

    // === MCAD errors (5000-5999) ===
    /// Part not found
    PartNotFound = 5001,
    /// Feature operation failed
    FeatureFailed = 5002,
    /// Sketch error
    SketchError = 5003,
    /// Assembly constraint failed
    AssemblyConstraintFailed = 5004,
    /// Material not found
    MaterialNotFound = 5005,
    /// Feature tree error
    FeatureTreeError = 5006,

    // === BIM errors (6000-6999) ===
    /// BIM element not found
    BimElementNotFound = 6001,
    /// Invalid element configuration
    InvalidElementConfig = 6002,
    /// Section not found
    SectionNotFound = 6003,
    /// Connection failed
    ConnectionFailed = 6004,
    /// Spatial relationship error
    SpatialError = 6005,
    /// Level not found
    LevelNotFound = 6006,

    // === EDA errors (7000-7999) ===
    /// Component not found
    ComponentNotFound = 7001,
    /// Symbol not found
    SymbolNotFound = 7002,
    /// Footprint not found
    FootprintNotFound = 7003,
    /// Net error
    NetError = 7004,
    /// DRC violation
    DrcViolation = 7005,
    /// ERC violation
    ErcViolation = 7006,
    /// Invalid pin
    InvalidPin = 7007,
    /// Invalid layer
    InvalidLayer = 7008,
    /// Gerber export error
    GerberError = 7009,

    // === Rendering errors (8000-8999) ===
    /// GPU initialization failed
    GpuInitFailed = 8001,
    /// Shader compilation failed
    ShaderCompileFailed = 8002,
    /// Texture creation failed
    TextureError = 8003,
    /// Buffer creation failed
    BufferError = 8004,

    // === UI errors (9000-9999) ===
    /// UI initialization failed
    UiInitFailed = 9001,
    /// Invalid input
    InvalidInput = 9002,
    /// Action failed
    ActionFailed = 9003,
}

impl ErrorCode {
    /// Get the numeric value of this error code
    pub fn as_u16(self) -> u16 {
        self as u16
    }

    /// Get the category name for this error code
    pub fn category(&self) -> &'static str {
        match self.as_u16() {
            1000..=1999 => "Core",
            2000..=2999 => "File",
            3000..=3999 => "Geometry",
            4000..=4999 => "Module",
            5000..=5999 => "MCAD",
            6000..=6999 => "BIM",
            7000..=7999 => "EDA",
            8000..=8999 => "Rendering",
            9000..=9999 => "UI",
            _ => "Unknown",
        }
    }

    /// Check if this is a recoverable error
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            ErrorCode::Cancelled
                | ErrorCode::InvalidArgument
                | ErrorCode::NotFound
                | ErrorCode::FileNotFound
                | ErrorCode::InvalidInput
                | ErrorCode::DrcViolation
                | ErrorCode::ErcViolation
        )
    }

    /// Get a brief description of this error code
    pub fn description(&self) -> &'static str {
        match self {
            // Core
            ErrorCode::Unknown => "An unknown error occurred",
            ErrorCode::Cancelled => "Operation was cancelled",
            ErrorCode::Timeout => "Operation timed out",
            ErrorCode::InvalidArgument => "Invalid argument provided",
            ErrorCode::NotSupported => "Operation not supported",
            ErrorCode::Internal => "Internal error",
            ErrorCode::NotFound => "Resource not found",
            ErrorCode::AlreadyExists => "Resource already exists",
            ErrorCode::PermissionDenied => "Permission denied",

            // File
            ErrorCode::FileNotFound => "File not found",
            ErrorCode::FileReadError => "Failed to read file",
            ErrorCode::FileWriteError => "Failed to write file",
            ErrorCode::InvalidFileFormat => "Invalid file format",
            ErrorCode::UnsupportedVersion => "Unsupported file version",
            ErrorCode::FileCorrupted => "File is corrupted",
            ErrorCode::DxfError => "DXF file error",
            ErrorCode::StepError => "STEP/IGES file error",
            ErrorCode::IfcError => "IFC file error",

            // Geometry
            ErrorCode::InvalidGeometry => "Invalid geometry",
            ErrorCode::DegenerateGeometry => "Degenerate geometry",
            ErrorCode::BooleanFailed => "Boolean operation failed",
            ErrorCode::TopologyError => "Topology error",
            ErrorCode::TessellationFailed => "Tessellation failed",
            ErrorCode::TransformError => "Transform error",
            ErrorCode::ConstraintSolverFailed => "Constraint solver failed",

            // Module
            ErrorCode::ModuleNotFound => "Module not found",
            ErrorCode::ModuleAlreadyLoaded => "Module already loaded",
            ErrorCode::DependencyNotSatisfied => "Module dependency not satisfied",
            ErrorCode::CircularDependency => "Circular dependency detected",
            ErrorCode::VersionIncompatible => "Module version incompatible",
            ErrorCode::ModuleInitFailed => "Module initialization failed",
            ErrorCode::CommandNotFound => "Command not found",
            ErrorCode::CommandAlreadyRegistered => "Command already registered",

            // MCAD
            ErrorCode::PartNotFound => "Part not found",
            ErrorCode::FeatureFailed => "Feature operation failed",
            ErrorCode::SketchError => "Sketch error",
            ErrorCode::AssemblyConstraintFailed => "Assembly constraint failed",
            ErrorCode::MaterialNotFound => "Material not found",
            ErrorCode::FeatureTreeError => "Feature tree error",

            // BIM
            ErrorCode::BimElementNotFound => "BIM element not found",
            ErrorCode::InvalidElementConfig => "Invalid element configuration",
            ErrorCode::SectionNotFound => "Section not found",
            ErrorCode::ConnectionFailed => "Connection failed",
            ErrorCode::SpatialError => "Spatial relationship error",
            ErrorCode::LevelNotFound => "Level not found",

            // EDA
            ErrorCode::ComponentNotFound => "Component not found",
            ErrorCode::SymbolNotFound => "Symbol not found",
            ErrorCode::FootprintNotFound => "Footprint not found",
            ErrorCode::NetError => "Net error",
            ErrorCode::DrcViolation => "Design rule check violation",
            ErrorCode::ErcViolation => "Electrical rule check violation",
            ErrorCode::InvalidPin => "Invalid pin",
            ErrorCode::InvalidLayer => "Invalid layer",
            ErrorCode::GerberError => "Gerber export error",

            // Rendering
            ErrorCode::GpuInitFailed => "GPU initialization failed",
            ErrorCode::ShaderCompileFailed => "Shader compilation failed",
            ErrorCode::TextureError => "Texture error",
            ErrorCode::BufferError => "Buffer error",

            // UI
            ErrorCode::UiInitFailed => "UI initialization failed",
            ErrorCode::InvalidInput => "Invalid input",
            ErrorCode::ActionFailed => "Action failed",
        }
    }
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "E{:04}", self.as_u16())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::Unknown.to_string(), "E1000");
        assert_eq!(ErrorCode::FileNotFound.to_string(), "E2001");
        assert_eq!(ErrorCode::DrcViolation.to_string(), "E7005");
    }

    #[test]
    fn test_error_code_category() {
        assert_eq!(ErrorCode::Unknown.category(), "Core");
        assert_eq!(ErrorCode::FileNotFound.category(), "File");
        assert_eq!(ErrorCode::InvalidGeometry.category(), "Geometry");
        assert_eq!(ErrorCode::DrcViolation.category(), "EDA");
    }

    #[test]
    fn test_error_code_recoverable() {
        assert!(ErrorCode::Cancelled.is_recoverable());
        assert!(ErrorCode::DrcViolation.is_recoverable());
        assert!(!ErrorCode::Internal.is_recoverable());
    }
}
