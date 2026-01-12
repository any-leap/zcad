//! Steel structure elements
//!
//! Provides steel beams, columns, plates, and connections.

use crate::core::{BimElement, BimElementId, ElementCategory};
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use zcad_geometry_3d::math::{Point3, Vector3};
use zcad_occt::shape::OcctShape;

/// Steel section profile type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SectionProfileType {
    /// I-beam / H-beam / Wide Flange
    ISection,
    /// Channel (C-section)
    Channel,
    /// Angle (L-section)
    Angle,
    /// Hollow Structural Section (Square/Rectangular)
    HSS,
    /// Circular Hollow Section (Pipe)
    CHS,
    /// T-section
    TSection,
    /// Flat plate
    Plate,
    /// Custom profile
    Custom,
}

/// Steel section definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteelSection {
    /// Section name (e.g., "W10x49", "HSS4x4x1/4")
    pub name: String,

    /// Profile type
    pub profile_type: SectionProfileType,

    /// Depth/height (mm)
    pub depth: f64,

    /// Width (mm)
    pub width: f64,

    /// Web thickness (mm)
    pub web_thickness: f64,

    /// Flange thickness (mm)
    pub flange_thickness: f64,

    /// Corner radius (mm)
    pub corner_radius: f64,

    /// Cross-sectional area (mm²)
    pub area: f64,

    /// Moment of inertia about major axis (mm⁴)
    pub ix: f64,

    /// Moment of inertia about minor axis (mm⁴)
    pub iy: f64,

    /// Section modulus about major axis (mm³)
    pub sx: f64,

    /// Section modulus about minor axis (mm³)
    pub sy: f64,

    /// Weight per unit length (kg/m)
    pub weight_per_meter: f64,
}

impl SteelSection {
    /// Create a new section
    pub fn new(name: impl Into<String>, profile_type: SectionProfileType) -> Self {
        Self {
            name: name.into(),
            profile_type,
            depth: 0.0,
            width: 0.0,
            web_thickness: 0.0,
            flange_thickness: 0.0,
            corner_radius: 0.0,
            area: 0.0,
            ix: 0.0,
            iy: 0.0,
            sx: 0.0,
            sy: 0.0,
            weight_per_meter: 0.0,
        }
    }

    /// Create a W-section (Wide Flange)
    pub fn w_section(name: &str, depth: f64, width: f64, web: f64, flange: f64) -> Self {
        let mut section = Self::new(name, SectionProfileType::ISection);
        section.depth = depth;
        section.width = width;
        section.web_thickness = web;
        section.flange_thickness = flange;
        // Calculate area (approximate)
        section.area = 2.0 * width * flange + (depth - 2.0 * flange) * web;
        section
    }

    /// Create a HSS (rectangular tube)
    pub fn hss_section(name: &str, depth: f64, width: f64, thickness: f64) -> Self {
        let mut section = Self::new(name, SectionProfileType::HSS);
        section.depth = depth;
        section.width = width;
        section.web_thickness = thickness;
        section.flange_thickness = thickness;
        // Calculate area
        section.area = 2.0 * (depth + width - 2.0 * thickness) * thickness;
        section
    }

    /// Create a CHS (circular pipe)
    pub fn chs_section(name: &str, diameter: f64, thickness: f64) -> Self {
        let mut section = Self::new(name, SectionProfileType::CHS);
        section.depth = diameter;
        section.width = diameter;
        section.web_thickness = thickness;
        // Calculate area
        let outer_r = diameter / 2.0;
        let inner_r = outer_r - thickness;
        section.area = std::f64::consts::PI * (outer_r * outer_r - inner_r * inner_r);
        section
    }

    /// Create an angle section
    pub fn angle_section(name: &str, leg_a: f64, leg_b: f64, thickness: f64) -> Self {
        let mut section = Self::new(name, SectionProfileType::Angle);
        section.depth = leg_a;
        section.width = leg_b;
        section.web_thickness = thickness;
        section.flange_thickness = thickness;
        // Approximate area
        section.area = (leg_a + leg_b - thickness) * thickness;
        section
    }
}

/// Steel section catalog
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SectionCatalog {
    sections: HashMap<String, SteelSection>,
}

impl SectionCatalog {
    pub fn new() -> Self {
        Self {
            sections: HashMap::new(),
        }
    }

    /// Load default AISC sections
    pub fn with_aisc_sections() -> Self {
        let mut catalog = Self::new();
        
        // Add some common W sections
        catalog.add(SteelSection::w_section("W10x49", 253.0, 254.0, 8.6, 14.2));
        catalog.add(SteelSection::w_section("W12x65", 307.0, 305.0, 9.9, 15.4));
        catalog.add(SteelSection::w_section("W14x82", 363.0, 257.0, 10.9, 18.8));
        catalog.add(SteelSection::w_section("W16x100", 420.0, 267.0, 11.8, 21.6));
        catalog.add(SteelSection::w_section("W18x119", 478.0, 282.0, 13.3, 24.0));

        // Add some HSS sections
        catalog.add(SteelSection::hss_section("HSS4x4x1/4", 101.6, 101.6, 6.35));
        catalog.add(SteelSection::hss_section("HSS6x6x3/8", 152.4, 152.4, 9.53));
        catalog.add(SteelSection::hss_section("HSS8x8x1/2", 203.2, 203.2, 12.7));

        // Add some pipe sections
        catalog.add(SteelSection::chs_section("Pipe4STD", 114.3, 6.02));
        catalog.add(SteelSection::chs_section("Pipe6STD", 168.3, 7.11));

        catalog
    }

    pub fn add(&mut self, section: SteelSection) {
        self.sections.insert(section.name.clone(), section);
    }

    pub fn get(&self, name: &str) -> Option<&SteelSection> {
        self.sections.get(name)
    }

    pub fn names(&self) -> Vec<&str> {
        self.sections.keys().map(|s| s.as_str()).collect()
    }
}

/// A steel beam
#[derive(Debug, Clone)]
pub struct Beam {
    /// Base BIM element
    pub element: BimElement,

    /// Section profile
    pub section: SteelSection,

    /// Start point
    pub start_point: Point3,

    /// End point
    pub end_point: Point3,

    /// Rotation about longitudinal axis (degrees)
    pub rotation: f64,

    /// Start offset
    pub start_offset: Vector3,

    /// End offset
    pub end_offset: Vector3,
}

impl Beam {
    /// Create a new beam
    pub fn new(
        name: impl Into<String>,
        section: SteelSection,
        start: Point3,
        end: Point3,
    ) -> Self {
        Self {
            element: BimElement::new(ElementCategory::Beam, name),
            section,
            start_point: start,
            end_point: end,
            rotation: 0.0,
            start_offset: Vector3::zeros(),
            end_offset: Vector3::zeros(),
        }
    }

    /// Set rotation
    pub fn with_rotation(mut self, degrees: f64) -> Self {
        self.rotation = degrees;
        self
    }

    /// Get beam length
    pub fn length(&self) -> f64 {
        (self.end_point - self.start_point).norm()
    }

    /// Get beam direction (unit vector)
    pub fn direction(&self) -> Vector3 {
        (self.end_point - self.start_point).normalize()
    }

    /// Generate 3D geometry
    pub fn generate_geometry(&self) -> Result<OcctShape> {
        // Create a box approximation for now
        // Real implementation would extrude the section profile
        let length = self.length();
        OcctShape::make_box(self.section.width, self.section.depth, length)
            .map_err(|e| crate::error::BimError::OcctError(e))
    }
}

/// A steel column
#[derive(Debug, Clone)]
pub struct Column {
    /// Base BIM element
    pub element: BimElement,

    /// Section profile
    pub section: SteelSection,

    /// Base point
    pub base_point: Point3,

    /// Height
    pub height: f64,

    /// Rotation about vertical axis (degrees)
    pub rotation: f64,

    /// Base offset
    pub base_offset: f64,

    /// Top offset
    pub top_offset: f64,
}

impl Column {
    /// Create a new column
    pub fn new(
        name: impl Into<String>,
        section: SteelSection,
        base: Point3,
        height: f64,
    ) -> Self {
        Self {
            element: BimElement::new(ElementCategory::Column, name),
            section,
            base_point: base,
            height,
            rotation: 0.0,
            base_offset: 0.0,
            top_offset: 0.0,
        }
    }

    /// Set rotation
    pub fn with_rotation(mut self, degrees: f64) -> Self {
        self.rotation = degrees;
        self
    }

    /// Get top point
    pub fn top_point(&self) -> Point3 {
        Point3::new(
            self.base_point.x,
            self.base_point.y,
            self.base_point.z + self.height,
        )
    }
}

/// A steel plate
#[derive(Debug, Clone)]
pub struct Plate {
    /// Base BIM element
    pub element: BimElement,

    /// Plate thickness
    pub thickness: f64,

    /// Plate outline points (in local XY plane)
    pub outline: Vec<Point3>,
}

impl Plate {
    /// Create a rectangular plate
    pub fn rectangular(name: impl Into<String>, width: f64, height: f64, thickness: f64) -> Self {
        let outline = vec![
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(width, 0.0, 0.0),
            Point3::new(width, height, 0.0),
            Point3::new(0.0, height, 0.0),
        ];

        Self {
            element: BimElement::new(ElementCategory::Plate, name),
            thickness,
            outline,
        }
    }
}

/// Steel connection type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConnectionType {
    /// Bolted end plate
    BoltedEndPlate,
    /// Bolted fin plate (shear tab)
    BoltedFinPlate,
    /// Bolted angle
    BoltedAngle,
    /// Welded
    Welded,
    /// Base plate
    BasePlate,
    /// Splice
    Splice,
}

/// A steel connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SteelConnection {
    pub id: BimElementId,
    pub name: String,
    pub connection_type: ConnectionType,
    pub main_member_id: BimElementId,
    pub secondary_member_ids: Vec<BimElementId>,
    pub location: Point3,
}

impl SteelConnection {
    pub fn new(
        name: impl Into<String>,
        connection_type: ConnectionType,
        main_member: BimElementId,
        location: Point3,
    ) -> Self {
        Self {
            id: BimElementId::new(),
            name: name.into(),
            connection_type,
            main_member_id: main_member,
            secondary_member_ids: Vec::new(),
            location,
        }
    }

    pub fn add_secondary_member(&mut self, member_id: BimElementId) {
        self.secondary_member_ids.push(member_id);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_beam_creation() {
        let section = SteelSection::w_section("W10x49", 253.0, 254.0, 8.6, 14.2);
        let beam = Beam::new(
            "B-001",
            section,
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(6000.0, 0.0, 0.0),
        );

        assert_eq!(beam.length(), 6000.0);
    }

    #[test]
    fn test_section_catalog() {
        let catalog = SectionCatalog::with_aisc_sections();
        assert!(catalog.get("W10x49").is_some());
    }
}
