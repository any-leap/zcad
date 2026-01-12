//! BIM Core concepts
//!
//! Defines the fundamental building blocks for BIM modeling.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use zcad_geometry_3d::math::{BoundingBox3, Point3, Vector3};
use zcad_geometry_3d::transform::Transform3D;
use zcad_occt::shape::OcctShape;

static ELEMENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// BIM Element ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BimElementId(pub u64);

impl BimElementId {
    pub fn new() -> Self {
        Self(ELEMENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for BimElementId {
    fn default() -> Self {
        Self::new()
    }
}

/// BIM Element category (IFC-aligned)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ElementCategory {
    // Structural
    Beam,
    Column,
    Slab,
    Wall,
    Footing,
    Pile,
    Plate,

    // Architectural
    Door,
    Window,
    Stair,
    Ramp,
    Railing,
    Roof,
    Ceiling,
    Covering,
    Curtainwall,

    // MEP
    Duct,
    Pipe,
    Cable,
    CableTray,
    Equipment,
    Fitting,
    Valve,

    // Other
    Furniture,
    Annotation,
    Grid,
    Space,
    Zone,
    
    // Generic
    BuildingElementProxy,
}

impl ElementCategory {
    /// Get the IFC entity type for this category
    pub fn ifc_type(&self) -> &'static str {
        match self {
            ElementCategory::Beam => "IfcBeam",
            ElementCategory::Column => "IfcColumn",
            ElementCategory::Slab => "IfcSlab",
            ElementCategory::Wall => "IfcWall",
            ElementCategory::Footing => "IfcFooting",
            ElementCategory::Pile => "IfcPile",
            ElementCategory::Plate => "IfcPlate",
            ElementCategory::Door => "IfcDoor",
            ElementCategory::Window => "IfcWindow",
            ElementCategory::Stair => "IfcStair",
            ElementCategory::Ramp => "IfcRamp",
            ElementCategory::Railing => "IfcRailing",
            ElementCategory::Roof => "IfcRoof",
            ElementCategory::Ceiling => "IfcCeiling",
            ElementCategory::Covering => "IfcCovering",
            ElementCategory::Curtainwall => "IfcCurtainWall",
            ElementCategory::Duct => "IfcDuctSegment",
            ElementCategory::Pipe => "IfcPipeSegment",
            ElementCategory::Cable => "IfcCableSegment",
            ElementCategory::CableTray => "IfcCableTray",
            ElementCategory::Equipment => "IfcDistributionElement",
            ElementCategory::Fitting => "IfcFitting",
            ElementCategory::Valve => "IfcValve",
            ElementCategory::Furniture => "IfcFurniture",
            ElementCategory::Annotation => "IfcAnnotation",
            ElementCategory::Grid => "IfcGrid",
            ElementCategory::Space => "IfcSpace",
            ElementCategory::Zone => "IfcZone",
            ElementCategory::BuildingElementProxy => "IfcBuildingElementProxy",
        }
    }
}

/// Properties common to all BIM elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BimElementProperties {
    /// Element name
    pub name: String,

    /// Element type/family name
    pub type_name: Option<String>,

    /// Description
    pub description: Option<String>,

    /// Mark/tag
    pub mark: Option<String>,

    /// Material
    pub material: Option<String>,

    /// Phase (design, construction, etc.)
    pub phase: Option<String>,

    /// Custom properties
    pub custom: HashMap<String, serde_json::Value>,
}

impl BimElementProperties {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            type_name: None,
            description: None,
            mark: None,
            material: None,
            phase: None,
            custom: HashMap::new(),
        }
    }

    pub fn with_type(mut self, type_name: impl Into<String>) -> Self {
        self.type_name = Some(type_name.into());
        self
    }

    pub fn with_material(mut self, material: impl Into<String>) -> Self {
        self.material = Some(material.into());
        self
    }

    pub fn with_mark(mut self, mark: impl Into<String>) -> Self {
        self.mark = Some(mark.into());
        self
    }
}

/// A BIM element (building component)
#[derive(Debug, Clone)]
pub struct BimElement {
    /// Unique ID
    pub id: BimElementId,

    /// Element category
    pub category: ElementCategory,

    /// Properties
    pub properties: BimElementProperties,

    /// Local transformation
    pub transform: Transform3D,

    /// 3D geometry
    pub geometry: Option<OcctShape>,

    /// Parent element (spatial containment)
    pub parent_id: Option<BimElementId>,

    /// Level/storey this element is on
    pub level_id: Option<BimElementId>,

    /// IFC GUID (for interoperability)
    pub ifc_guid: Option<String>,
}

impl BimElement {
    /// Create a new BIM element
    pub fn new(category: ElementCategory, name: impl Into<String>) -> Self {
        Self {
            id: BimElementId::new(),
            category,
            properties: BimElementProperties::new(name),
            transform: Transform3D::identity(),
            geometry: None,
            parent_id: None,
            level_id: None,
            ifc_guid: None,
        }
    }

    /// Set the element's geometry
    pub fn with_geometry(mut self, geometry: OcctShape) -> Self {
        self.geometry = Some(geometry);
        self
    }

    /// Set the element's transform
    pub fn with_transform(mut self, transform: Transform3D) -> Self {
        self.transform = transform;
        self
    }

    /// Set the parent element
    pub fn with_parent(mut self, parent_id: BimElementId) -> Self {
        self.parent_id = Some(parent_id);
        self
    }

    /// Set the level/storey
    pub fn with_level(mut self, level_id: BimElementId) -> Self {
        self.level_id = Some(level_id);
        self
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        self.geometry
            .as_ref()
            .map(|g| g.bounding_box())
            .unwrap_or_else(BoundingBox3::empty)
    }

    /// Get the IFC entity type
    pub fn ifc_type(&self) -> &'static str {
        self.category.ifc_type()
    }
}

// ========== Spatial Hierarchy ==========

/// A project (top-level container)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: BimElementId,
    pub name: String,
    pub description: Option<String>,
    pub sites: Vec<BimElementId>,
    pub units: ProjectUnits,
}

impl Project {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: BimElementId::new(),
            name: name.into(),
            description: None,
            sites: Vec::new(),
            units: ProjectUnits::default(),
        }
    }
}

/// Project units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectUnits {
    pub length: LengthUnit,
    pub area: AreaUnit,
    pub volume: VolumeUnit,
    pub angle: AngleUnit,
}

impl Default for ProjectUnits {
    fn default() -> Self {
        Self {
            length: LengthUnit::Millimeter,
            area: AreaUnit::SquareMeter,
            volume: VolumeUnit::CubicMeter,
            angle: AngleUnit::Degree,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum LengthUnit {
    Millimeter,
    Centimeter,
    Meter,
    Inch,
    Foot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AreaUnit {
    SquareMillimeter,
    SquareMeter,
    SquareFoot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum VolumeUnit {
    CubicMillimeter,
    CubicMeter,
    CubicFoot,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum AngleUnit {
    Degree,
    Radian,
}

/// A site
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Site {
    pub id: BimElementId,
    pub name: String,
    pub buildings: Vec<BimElementId>,
    pub ref_latitude: Option<f64>,
    pub ref_longitude: Option<f64>,
    pub ref_elevation: Option<f64>,
}

impl Site {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: BimElementId::new(),
            name: name.into(),
            buildings: Vec::new(),
            ref_latitude: None,
            ref_longitude: None,
            ref_elevation: None,
        }
    }
}

/// A building storey/level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingStorey {
    pub id: BimElementId,
    pub name: String,
    pub elevation: f64,
    pub height: Option<f64>,
    pub elements: Vec<BimElementId>,
}

impl BuildingStorey {
    pub fn new(name: impl Into<String>, elevation: f64) -> Self {
        Self {
            id: BimElementId::new(),
            name: name.into(),
            elevation,
            height: None,
            elements: Vec::new(),
        }
    }

    pub fn with_height(mut self, height: f64) -> Self {
        self.height = Some(height);
        self
    }
}

/// Alias for BuildingStorey
pub type Level = BuildingStorey;

/// Spatial element (room, space, zone)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpatialElement {
    pub id: BimElementId,
    pub name: String,
    pub category: SpatialCategory,
    pub boundary: Option<Vec<Point3>>,
    pub area: Option<f64>,
    pub volume: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SpatialCategory {
    Space,
    Room,
    Zone,
    Area,
}

impl SpatialElement {
    pub fn new(name: impl Into<String>, category: SpatialCategory) -> Self {
        Self {
            id: BimElementId::new(),
            name: name.into(),
            category,
            boundary: None,
            area: None,
            volume: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_element_creation() {
        let beam = BimElement::new(ElementCategory::Beam, "B-001");
        assert_eq!(beam.properties.name, "B-001");
        assert_eq!(beam.ifc_type(), "IfcBeam");
    }

    #[test]
    fn test_project_creation() {
        let project = Project::new("Test Project");
        assert!(project.sites.is_empty());
    }
}
