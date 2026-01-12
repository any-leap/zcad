//! Geometric Dimensioning and Tolerancing (GD&T)
//!
//! Provides tolerance annotations for engineering drawings.

use serde::{Deserialize, Serialize};

/// Tolerance type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToleranceType {
    /// Plus/minus symmetric tolerance
    Symmetric(f64),

    /// Asymmetric tolerance (upper, lower)
    Asymmetric { upper: f64, lower: f64 },

    /// Limit tolerance (min, max values)
    Limits { min: f64, max: f64 },

    /// ISO tolerance grade (e.g., H7, g6)
    IsoFit {
        grade: String,
        deviation: char,
    },

    /// Basic dimension (enclosed in box)
    Basic,

    /// Reference dimension (parentheses)
    Reference,
}

/// GD&T characteristic symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GdtCharacteristic {
    // Form
    /// ⌓ Straightness
    Straightness,
    /// ⌖ Flatness
    Flatness,
    /// ○ Circularity (Roundness)
    Circularity,
    /// ⌭ Cylindricity
    Cylindricity,

    // Profile
    /// ⌒ Profile of a Line
    ProfileLine,
    /// ⌓ Profile of a Surface
    ProfileSurface,

    // Orientation
    /// ∥ Parallelism
    Parallelism,
    /// ⊥ Perpendicularity
    Perpendicularity,
    /// ∠ Angularity
    Angularity,

    // Location
    /// ⊕ Position
    Position,
    /// ◎ Concentricity
    Concentricity,
    /// ≡ Symmetry
    Symmetry,

    // Runout
    /// ↗ Circular Runout
    CircularRunout,
    /// ⇗ Total Runout
    TotalRunout,
}

impl GdtCharacteristic {
    /// Get the Unicode symbol for this characteristic
    pub fn symbol(&self) -> &'static str {
        match self {
            GdtCharacteristic::Straightness => "⌓",
            GdtCharacteristic::Flatness => "⌖",
            GdtCharacteristic::Circularity => "○",
            GdtCharacteristic::Cylindricity => "⌭",
            GdtCharacteristic::ProfileLine => "⌒",
            GdtCharacteristic::ProfileSurface => "⌓",
            GdtCharacteristic::Parallelism => "∥",
            GdtCharacteristic::Perpendicularity => "⊥",
            GdtCharacteristic::Angularity => "∠",
            GdtCharacteristic::Position => "⊕",
            GdtCharacteristic::Concentricity => "◎",
            GdtCharacteristic::Symmetry => "≡",
            GdtCharacteristic::CircularRunout => "↗",
            GdtCharacteristic::TotalRunout => "⇗",
        }
    }

    /// Get the name of this characteristic
    pub fn name(&self) -> &'static str {
        match self {
            GdtCharacteristic::Straightness => "Straightness",
            GdtCharacteristic::Flatness => "Flatness",
            GdtCharacteristic::Circularity => "Circularity",
            GdtCharacteristic::Cylindricity => "Cylindricity",
            GdtCharacteristic::ProfileLine => "Profile of a Line",
            GdtCharacteristic::ProfileSurface => "Profile of a Surface",
            GdtCharacteristic::Parallelism => "Parallelism",
            GdtCharacteristic::Perpendicularity => "Perpendicularity",
            GdtCharacteristic::Angularity => "Angularity",
            GdtCharacteristic::Position => "Position",
            GdtCharacteristic::Concentricity => "Concentricity",
            GdtCharacteristic::Symmetry => "Symmetry",
            GdtCharacteristic::CircularRunout => "Circular Runout",
            GdtCharacteristic::TotalRunout => "Total Runout",
        }
    }
}

/// Material condition modifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MaterialCondition {
    /// No modifier
    None,
    /// Ⓜ Maximum Material Condition
    Maximum,
    /// Ⓛ Least Material Condition
    Least,
    /// Ⓢ Regardless of Feature Size
    RegardlessOfFeatureSize,
}

/// A tolerance specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tolerance {
    /// Tolerance type
    pub tolerance_type: ToleranceType,

    /// Nominal value (for dimensional tolerances)
    pub nominal: Option<f64>,

    /// Unit (mm, in, etc.)
    pub unit: String,
}

impl Tolerance {
    /// Create a symmetric tolerance
    pub fn symmetric(nominal: f64, tolerance: f64) -> Self {
        Self {
            tolerance_type: ToleranceType::Symmetric(tolerance),
            nominal: Some(nominal),
            unit: "mm".to_string(),
        }
    }

    /// Create an asymmetric tolerance
    pub fn asymmetric(nominal: f64, upper: f64, lower: f64) -> Self {
        Self {
            tolerance_type: ToleranceType::Asymmetric { upper, lower },
            nominal: Some(nominal),
            unit: "mm".to_string(),
        }
    }

    /// Create a limits tolerance
    pub fn limits(min: f64, max: f64) -> Self {
        Self {
            tolerance_type: ToleranceType::Limits { min, max },
            nominal: Some((min + max) / 2.0),
            unit: "mm".to_string(),
        }
    }

    /// Create an ISO fit tolerance
    pub fn iso_fit(nominal: f64, fit: &str) -> Self {
        // Parse fit string like "H7" or "g6"
        let mut chars = fit.chars();
        let deviation = chars.next().unwrap_or('H');
        let grade = chars.collect::<String>();

        Self {
            tolerance_type: ToleranceType::IsoFit { grade, deviation },
            nominal: Some(nominal),
            unit: "mm".to_string(),
        }
    }

    /// Format the tolerance as a string
    pub fn format(&self) -> String {
        match &self.tolerance_type {
            ToleranceType::Symmetric(t) => {
                if let Some(nom) = self.nominal {
                    format!("{} ± {}", nom, t)
                } else {
                    format!("± {}", t)
                }
            }
            ToleranceType::Asymmetric { upper, lower } => {
                if let Some(nom) = self.nominal {
                    format!("{} +{} / -{}", nom, upper, lower.abs())
                } else {
                    format!("+{} / -{}", upper, lower.abs())
                }
            }
            ToleranceType::Limits { min, max } => {
                format!("{} - {}", min, max)
            }
            ToleranceType::IsoFit { grade, deviation } => {
                if let Some(nom) = self.nominal {
                    format!("{} {}{}", nom, deviation, grade)
                } else {
                    format!("{}{}", deviation, grade)
                }
            }
            ToleranceType::Basic => {
                if let Some(nom) = self.nominal {
                    format!("[{}]", nom)
                } else {
                    "BASIC".to_string()
                }
            }
            ToleranceType::Reference => {
                if let Some(nom) = self.nominal {
                    format!("({})", nom)
                } else {
                    "REF".to_string()
                }
            }
        }
    }
}

/// Feature control frame (GD&T annotation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureControlFrame {
    /// Geometric characteristic
    pub characteristic: GdtCharacteristic,

    /// Tolerance value
    pub tolerance: f64,

    /// Tolerance zone shape (⌀ for diameter)
    pub diameter_zone: bool,

    /// Material condition
    pub material_condition: MaterialCondition,

    /// Primary datum reference
    pub datum_primary: Option<String>,

    /// Secondary datum reference
    pub datum_secondary: Option<String>,

    /// Tertiary datum reference
    pub datum_tertiary: Option<String>,
}

impl FeatureControlFrame {
    /// Create a new feature control frame
    pub fn new(characteristic: GdtCharacteristic, tolerance: f64) -> Self {
        Self {
            characteristic,
            tolerance,
            diameter_zone: false,
            material_condition: MaterialCondition::None,
            datum_primary: None,
            datum_secondary: None,
            datum_tertiary: None,
        }
    }

    /// Set diameter tolerance zone
    pub fn with_diameter_zone(mut self) -> Self {
        self.diameter_zone = true;
        self
    }

    /// Set primary datum
    pub fn with_datum_a(mut self, datum: impl Into<String>) -> Self {
        self.datum_primary = Some(datum.into());
        self
    }

    /// Set secondary datum
    pub fn with_datum_b(mut self, datum: impl Into<String>) -> Self {
        self.datum_secondary = Some(datum.into());
        self
    }

    /// Set tertiary datum
    pub fn with_datum_c(mut self, datum: impl Into<String>) -> Self {
        self.datum_tertiary = Some(datum.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symmetric_tolerance() {
        let tol = Tolerance::symmetric(10.0, 0.1);
        assert_eq!(tol.format(), "10 ± 0.1");
    }

    #[test]
    fn test_iso_fit() {
        let tol = Tolerance::iso_fit(25.0, "H7");
        assert_eq!(tol.format(), "25 H7");
    }

    #[test]
    fn test_feature_control_frame() {
        let fcf = FeatureControlFrame::new(GdtCharacteristic::Position, 0.05)
            .with_diameter_zone()
            .with_datum_a("A")
            .with_datum_b("B");

        assert_eq!(fcf.characteristic, GdtCharacteristic::Position);
        assert_eq!(fcf.datum_primary, Some("A".to_string()));
    }
}
