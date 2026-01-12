//! Part design
//!
//! A Part represents a single solid body with a feature tree.

use crate::error::Result;
use crate::feature::{Feature, FeatureId, FeatureTree};
use crate::material::MaterialId;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use zcad_geometry_3d::math::{BoundingBox3, Point3};
use zcad_occt::shape::OcctShape;

static PART_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Part ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PartId(pub u64);

impl PartId {
    pub fn new() -> Self {
        Self(PART_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for PartId {
    fn default() -> Self {
        Self::new()
    }
}

/// Part properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartProperties {
    /// Part name
    pub name: String,
    /// Part number/ID
    pub part_number: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Material ID
    pub material_id: Option<MaterialId>,
    /// Author
    pub author: Option<String>,
    /// Creation date
    pub created: Option<String>,
    /// Last modified date
    pub modified: Option<String>,
}

impl PartProperties {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            part_number: None,
            description: None,
            material_id: None,
            author: None,
            created: None,
            modified: None,
        }
    }
}

/// A part with feature-based modeling
#[derive(Debug)]
pub struct Part {
    /// Unique ID
    pub id: PartId,

    /// Part properties
    pub properties: PartProperties,

    /// Feature tree
    pub features: FeatureTree,

    /// Current solid body (result of feature tree)
    body: Option<OcctShape>,

    /// Is the body up to date?
    body_valid: bool,
}

impl Part {
    /// Create a new empty part
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: PartId::new(),
            properties: PartProperties::new(name),
            features: FeatureTree::new(),
            body: None,
            body_valid: false,
        }
    }

    /// Get the part name
    pub fn name(&self) -> &str {
        &self.properties.name
    }

    /// Set the part name
    pub fn set_name(&mut self, name: impl Into<String>) {
        self.properties.name = name.into();
    }

    /// Add a feature to the part
    pub fn add_feature(&mut self, feature: Feature) -> FeatureId {
        let id = self.features.add_feature(feature);
        self.body_valid = false;
        id
    }

    /// Remove a feature
    pub fn remove_feature(&mut self, id: FeatureId) -> bool {
        let removed = self.features.remove_feature(id);
        if removed {
            self.body_valid = false;
        }
        removed
    }

    /// Suppress a feature (disable without removing)
    pub fn suppress_feature(&mut self, id: FeatureId, suppressed: bool) {
        self.features.set_suppressed(id, suppressed);
        self.body_valid = false;
    }

    /// Rebuild the part geometry
    pub fn rebuild(&mut self) -> Result<()> {
        if self.body_valid {
            return Ok(());
        }

        // Start with null shape
        let mut body = OcctShape::null();

        // Apply features in order
        for feature in self.features.ordered_features() {
            if !feature.suppressed {
                body = feature.apply(&body)?;
            }
        }

        self.body = Some(body);
        self.body_valid = true;
        Ok(())
    }

    /// Get the solid body
    pub fn body(&self) -> Option<&OcctShape> {
        self.body.as_ref()
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        self.body
            .as_ref()
            .map(|b| b.bounding_box())
            .unwrap_or_else(BoundingBox3::empty)
    }

    /// Calculate mass properties (requires material)
    pub fn mass(&self, density: f64) -> Result<f64> {
        if let Some(body) = &self.body {
            let volume = body.volume()?;
            Ok(volume * density)
        } else {
            Ok(0.0)
        }
    }

    /// Get feature count
    pub fn feature_count(&self) -> usize {
        self.features.len()
    }

    /// Check if the part needs rebuilding
    pub fn needs_rebuild(&self) -> bool {
        !self.body_valid
    }

    /// Mark the part as needing rebuild
    pub fn invalidate(&mut self) {
        self.body_valid = false;
    }
}

impl Clone for Part {
    fn clone(&self) -> Self {
        Self {
            id: PartId::new(), // New ID for cloned part
            properties: self.properties.clone(),
            features: self.features.clone(),
            body: self.body.clone(),
            body_valid: self.body_valid,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_creation() {
        let part = Part::new("Test Part");
        assert_eq!(part.name(), "Test Part");
        assert_eq!(part.feature_count(), 0);
    }
}
