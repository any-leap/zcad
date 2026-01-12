//! Reinforced concrete elements
//!
//! Provides concrete beams, columns, slabs, and rebar detailing.

use serde::{Deserialize, Serialize};

/// Concrete grade/class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConcreteGrade {
    /// Grade name (e.g., "C30/37", "4000 PSI")
    pub name: String,

    /// Characteristic compressive strength (MPa)
    pub fck: f64,

    /// Mean compressive strength (MPa)
    pub fcm: f64,

    /// Modulus of elasticity (GPa)
    pub ecm: f64,
}

impl ConcreteGrade {
    /// C30/37 grade
    pub fn c30_37() -> Self {
        Self {
            name: "C30/37".to_string(),
            fck: 30.0,
            fcm: 38.0,
            ecm: 33.0,
        }
    }

    /// C40/50 grade
    pub fn c40_50() -> Self {
        Self {
            name: "C40/50".to_string(),
            fck: 40.0,
            fcm: 48.0,
            ecm: 35.0,
        }
    }
}

/// Rebar grade
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebarGrade {
    /// Grade name (e.g., "B500B", "Grade 60")
    pub name: String,

    /// Yield strength (MPa)
    pub fy: f64,

    /// Ultimate strength (MPa)
    pub fu: f64,

    /// Modulus of elasticity (GPa)
    pub es: f64,
}

impl RebarGrade {
    /// B500B grade (European)
    pub fn b500b() -> Self {
        Self {
            name: "B500B".to_string(),
            fy: 500.0,
            fu: 540.0,
            es: 200.0,
        }
    }

    /// Grade 60 (US)
    pub fn grade_60() -> Self {
        Self {
            name: "Grade 60".to_string(),
            fy: 420.0,
            fu: 620.0,
            es: 200.0,
        }
    }
}

/// Rebar diameter (standard sizes)
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum RebarDiameter {
    D6,
    D8,
    D10,
    D12,
    D14,
    D16,
    D20,
    D25,
    D32,
    D40,
    Custom(f64),
}

impl RebarDiameter {
    /// Get diameter in mm
    pub fn diameter_mm(&self) -> f64 {
        match self {
            RebarDiameter::D6 => 6.0,
            RebarDiameter::D8 => 8.0,
            RebarDiameter::D10 => 10.0,
            RebarDiameter::D12 => 12.0,
            RebarDiameter::D14 => 14.0,
            RebarDiameter::D16 => 16.0,
            RebarDiameter::D20 => 20.0,
            RebarDiameter::D25 => 25.0,
            RebarDiameter::D32 => 32.0,
            RebarDiameter::D40 => 40.0,
            RebarDiameter::Custom(d) => *d,
        }
    }

    /// Get cross-sectional area in mm²
    pub fn area(&self) -> f64 {
        let d = self.diameter_mm();
        std::f64::consts::PI * d * d / 4.0
    }
}

/// A rebar bar shape
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebarShape {
    /// Shape code (e.g., "00", "11", "21")
    pub code: String,

    /// Segments (lengths and bend angles)
    pub segments: Vec<RebarSegment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebarSegment {
    pub length: f64,
    pub bend_angle: Option<f64>,
    pub bend_radius: Option<f64>,
}

/// A rebar set (group of bars)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RebarSet {
    pub id: u64,
    pub mark: String,
    pub diameter: RebarDiameter,
    pub grade: RebarGrade,
    pub shape: RebarShape,
    pub count: u32,
    pub spacing: Option<f64>,
}

impl RebarSet {
    /// Create a new rebar set
    pub fn new(mark: impl Into<String>, diameter: RebarDiameter, count: u32) -> Self {
        Self {
            id: 1,
            mark: mark.into(),
            diameter,
            grade: RebarGrade::b500b(),
            shape: RebarShape {
                code: "00".to_string(),
                segments: vec![],
            },
            count,
            spacing: None,
        }
    }

    /// Set spacing
    pub fn with_spacing(mut self, spacing: f64) -> Self {
        self.spacing = Some(spacing);
        self
    }

    /// Calculate total weight (kg)
    pub fn weight(&self) -> f64 {
        let area = self.diameter.area(); // mm²
        let length = self.shape.segments.iter().map(|s| s.length).sum::<f64>(); // mm
        let density = 7850.0; // kg/m³

        let volume = area * length / 1e9; // m³
        volume * density * self.count as f64
    }
}

// Placeholder for concrete beam, column, slab, etc.
// These would be similar to steel but with rebar definitions

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rebar_area() {
        let d12 = RebarDiameter::D12;
        let area = d12.area();
        // π * 12² / 4 ≈ 113.1
        assert!((area - 113.1).abs() < 0.1);
    }
}
