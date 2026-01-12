//! Feature-based modeling
//!
//! Features are parametric operations that create or modify solid geometry.

use crate::error::{McadError, Result};
use crate::sketch::Sketch;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use zcad_geometry_3d::math::{Point3, Vector3};
use zcad_occt::shape::OcctShape;

static FEATURE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Feature ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FeatureId(pub u64);

impl FeatureId {
    pub fn new() -> Self {
        Self(FEATURE_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for FeatureId {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FeatureType {
    /// Extrude a sketch profile
    Extrude {
        sketch: Sketch,
        distance: f64,
        direction: Vector3,
        symmetric: bool,
    },

    /// Revolve a sketch profile
    Revolve {
        sketch: Sketch,
        axis_point: Point3,
        axis_direction: Vector3,
        angle: f64,
    },

    /// Create a hole
    Hole {
        position: Point3,
        direction: Vector3,
        diameter: f64,
        depth: f64,
        through: bool,
    },

    /// Fillet edges
    Fillet {
        radius: f64,
        edge_ids: Vec<u64>,
    },

    /// Chamfer edges
    Chamfer {
        distance: f64,
        edge_ids: Vec<u64>,
    },

    /// Shell (hollow out)
    Shell {
        thickness: f64,
        faces_to_remove: Vec<u64>,
    },

    /// Draft (taper faces)
    Draft {
        angle: f64,
        neutral_plane: Point3,
        direction: Vector3,
    },

    /// Boolean union (references another part/body)
    Union {
        tool_body_id: u64,
    },

    /// Boolean subtraction (references another part/body)
    Cut {
        tool_body_id: u64,
    },

    /// Boolean intersection (references another part/body)
    Intersect {
        tool_body_id: u64,
    },

    /// Pattern (linear)
    LinearPattern {
        source_features: Vec<FeatureId>,
        direction: Vector3,
        count: u32,
        spacing: f64,
    },

    /// Pattern (circular)
    CircularPattern {
        source_features: Vec<FeatureId>,
        axis_point: Point3,
        axis_direction: Vector3,
        count: u32,
        angle: f64,
    },

    /// Mirror
    Mirror {
        source_features: Vec<FeatureId>,
        plane_point: Point3,
        plane_normal: Vector3,
    },

    /// Primitive: Box
    PrimitiveBox {
        origin: Point3,
        dx: f64,
        dy: f64,
        dz: f64,
    },

    /// Primitive: Cylinder
    PrimitiveCylinder {
        origin: Point3,
        axis: Vector3,
        radius: f64,
        height: f64,
    },

    /// Primitive: Sphere
    PrimitiveSphere {
        center: Point3,
        radius: f64,
    },
}

/// A feature in the feature tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Unique ID
    pub id: FeatureId,

    /// Feature name
    pub name: String,

    /// Feature type and parameters
    pub feature_type: FeatureType,

    /// Is the feature suppressed?
    pub suppressed: bool,

    /// Parent feature IDs (dependencies)
    pub parents: Vec<FeatureId>,
}

impl Feature {
    /// Create a new feature
    pub fn new(name: impl Into<String>, feature_type: FeatureType) -> Self {
        Self {
            id: FeatureId::new(),
            name: name.into(),
            feature_type,
            suppressed: false,
            parents: Vec::new(),
        }
    }

    /// Apply this feature to a body
    pub fn apply(&self, body: &OcctShape) -> Result<OcctShape> {
        match &self.feature_type {
            FeatureType::PrimitiveBox { origin, dx, dy, dz } => {
                let new_shape = OcctShape::make_box_at(*origin, *dx, *dy, *dz)?;
                if body.is_null() {
                    Ok(new_shape)
                } else {
                    body.boolean_fuse(&new_shape)
                        .map_err(|e| McadError::FeatureFailed(e.to_string()))
                }
            }
            FeatureType::PrimitiveCylinder { origin, axis, radius, height } => {
                let new_shape = OcctShape::make_cylinder_at(*origin, *axis, *radius, *height)?;
                if body.is_null() {
                    Ok(new_shape)
                } else {
                    body.boolean_fuse(&new_shape)
                        .map_err(|e| McadError::FeatureFailed(e.to_string()))
                }
            }
            FeatureType::PrimitiveSphere { center, radius } => {
                let new_shape = OcctShape::make_sphere_at(*center, *radius)?;
                if body.is_null() {
                    Ok(new_shape)
                } else {
                    body.boolean_fuse(&new_shape)
                        .map_err(|e| McadError::FeatureFailed(e.to_string()))
                }
            }
            FeatureType::Cut { tool_body_id: _ } => {
                // Would need to lookup the tool body by ID
                Err(McadError::FeatureFailed(
                    "Boolean cut requires tool body lookup".into(),
                ))
            }
            FeatureType::Union { tool_body_id: _ } => {
                // Would need to lookup the tool body by ID
                Err(McadError::FeatureFailed(
                    "Boolean union requires tool body lookup".into(),
                ))
            }
            FeatureType::Fillet { radius, .. } => {
                body.fillet_all_edges(*radius)
                    .map_err(|e| McadError::FeatureFailed(e.to_string()))
            }
            FeatureType::Chamfer { distance, .. } => {
                body.chamfer_all_edges(*distance)
                    .map_err(|e| McadError::FeatureFailed(e.to_string()))
            }
            _ => {
                // Other features require OCCT implementation
                Err(McadError::FeatureFailed(
                    "Feature type not yet implemented".into(),
                ))
            }
        }
    }

    // ========== Convenience constructors ==========

    /// Create a box feature
    pub fn make_box(origin: Point3, dx: f64, dy: f64, dz: f64) -> Self {
        Self::new(
            "Box",
            FeatureType::PrimitiveBox { origin, dx, dy, dz },
        )
    }

    /// Create a cylinder feature
    pub fn make_cylinder(origin: Point3, radius: f64, height: f64) -> Self {
        Self::new(
            "Cylinder",
            FeatureType::PrimitiveCylinder {
                origin,
                axis: Vector3::z(),
                radius,
                height,
            },
        )
    }

    /// Create a sphere feature
    pub fn make_sphere(center: Point3, radius: f64) -> Self {
        Self::new(
            "Sphere",
            FeatureType::PrimitiveSphere { center, radius },
        )
    }

    /// Create an extrude feature
    pub fn extrude(sketch: Sketch, distance: f64) -> Self {
        Self::new(
            "Extrude",
            FeatureType::Extrude {
                sketch,
                distance,
                direction: Vector3::z(),
                symmetric: false,
            },
        )
    }

    /// Create a hole feature
    pub fn hole(position: Point3, diameter: f64, depth: f64) -> Self {
        Self::new(
            "Hole",
            FeatureType::Hole {
                position,
                direction: -Vector3::z(),
                diameter,
                depth,
                through: false,
            },
        )
    }

    /// Create a through hole feature
    pub fn through_hole(position: Point3, diameter: f64) -> Self {
        Self::new(
            "Through Hole",
            FeatureType::Hole {
                position,
                direction: -Vector3::z(),
                diameter,
                depth: 0.0,
                through: true,
            },
        )
    }

    /// Create a fillet feature
    pub fn fillet(radius: f64) -> Self {
        Self::new(
            "Fillet",
            FeatureType::Fillet {
                radius,
                edge_ids: vec![],
            },
        )
    }

    /// Create a chamfer feature
    pub fn chamfer(distance: f64) -> Self {
        Self::new(
            "Chamfer",
            FeatureType::Chamfer {
                distance,
                edge_ids: vec![],
            },
        )
    }
}

/// Feature tree - ordered collection of features
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FeatureTree {
    /// Features in order
    features: Vec<Feature>,
}

impl FeatureTree {
    /// Create a new empty feature tree
    pub fn new() -> Self {
        Self {
            features: Vec::new(),
        }
    }

    /// Add a feature
    pub fn add_feature(&mut self, feature: Feature) -> FeatureId {
        let id = feature.id;
        self.features.push(feature);
        id
    }

    /// Remove a feature by ID
    pub fn remove_feature(&mut self, id: FeatureId) -> bool {
        if let Some(pos) = self.features.iter().position(|f| f.id == id) {
            self.features.remove(pos);
            true
        } else {
            false
        }
    }

    /// Get a feature by ID
    pub fn get_feature(&self, id: FeatureId) -> Option<&Feature> {
        self.features.iter().find(|f| f.id == id)
    }

    /// Get a mutable feature by ID
    pub fn get_feature_mut(&mut self, id: FeatureId) -> Option<&mut Feature> {
        self.features.iter_mut().find(|f| f.id == id)
    }

    /// Set feature suppression state
    pub fn set_suppressed(&mut self, id: FeatureId, suppressed: bool) {
        if let Some(feature) = self.get_feature_mut(id) {
            feature.suppressed = suppressed;
        }
    }

    /// Get features in order (for rebuilding)
    pub fn ordered_features(&self) -> &[Feature] {
        &self.features
    }

    /// Get the number of features
    pub fn len(&self) -> usize {
        self.features.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.features.is_empty()
    }

    /// Move a feature to a new position
    pub fn reorder(&mut self, id: FeatureId, new_position: usize) {
        if let Some(pos) = self.features.iter().position(|f| f.id == id) {
            let feature = self.features.remove(pos);
            let insert_pos = new_position.min(self.features.len());
            self.features.insert(insert_pos, feature);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_creation() {
        let feature = Feature::make_box(Point3::origin(), 10.0, 20.0, 30.0);
        assert_eq!(feature.name, "Box");
        assert!(!feature.suppressed);
    }

    #[test]
    fn test_feature_tree() {
        let mut tree = FeatureTree::new();

        let f1 = Feature::make_box(Point3::origin(), 100.0, 50.0, 10.0);
        let id1 = tree.add_feature(f1);

        let f2 = Feature::make_cylinder(Point3::new(25.0, 25.0, 0.0), 5.0, 10.0);
        let _id2 = tree.add_feature(f2);

        assert_eq!(tree.len(), 2);
        assert!(tree.get_feature(id1).is_some());
    }
}
