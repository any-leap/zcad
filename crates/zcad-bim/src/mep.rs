//! MEP (Mechanical, Electrical, Plumbing) elements
//!
//! Ducts, pipes, cables, equipment, etc.

use serde::{Deserialize, Serialize};

/// Duct shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DuctShape {
    Rectangular { width: f64, height: f64 },
    Round { diameter: f64 },
    Oval { width: f64, height: f64 },
}

/// Duct segment placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuctSegment {
    pub id: u64,
    pub name: String,
    pub shape: DuctShape,
    pub length: f64,
    pub insulation_thickness: Option<f64>,
}

/// Pipe material
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PipeMaterial {
    Steel,
    Copper,
    PVC,
    CPVC,
    PEX,
    CastIron,
}

/// Pipe segment placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipeSegment {
    pub id: u64,
    pub name: String,
    pub outer_diameter: f64,
    pub wall_thickness: f64,
    pub material: PipeMaterial,
    pub length: f64,
}

/// Cable tray type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CableTrayType {
    Ladder,
    Solid,
    Perforated,
    Wire,
}

/// Cable tray placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CableTray {
    pub id: u64,
    pub name: String,
    pub tray_type: CableTrayType,
    pub width: f64,
    pub height: f64,
    pub length: f64,
}

// More MEP elements would be defined here
