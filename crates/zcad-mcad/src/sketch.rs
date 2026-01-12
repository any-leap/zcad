//! 2D Sketches for profile-based features
//!
//! Sketches are 2D constraint-based drawings used to create 3D features.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};
use zcad_core::math::Point2;

static SKETCH_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Sketch entity ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SketchEntityId(pub u64);

impl SketchEntityId {
    pub fn new() -> Self {
        Self(SKETCH_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for SketchEntityId {
    fn default() -> Self {
        Self::new()
    }
}

/// Sketch entity types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SketchEntity {
    /// Line segment
    Line {
        id: SketchEntityId,
        start: Point2,
        end: Point2,
    },

    /// Arc (center, radius, start/end angles)
    Arc {
        id: SketchEntityId,
        center: Point2,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
    },

    /// Full circle
    Circle {
        id: SketchEntityId,
        center: Point2,
        radius: f64,
    },

    /// Point (construction)
    Point {
        id: SketchEntityId,
        position: Point2,
    },

    /// Spline (control points)
    Spline {
        id: SketchEntityId,
        control_points: Vec<Point2>,
    },

    /// Ellipse
    Ellipse {
        id: SketchEntityId,
        center: Point2,
        major_radius: f64,
        minor_radius: f64,
        rotation: f64,
    },
}

impl SketchEntity {
    pub fn id(&self) -> SketchEntityId {
        match self {
            SketchEntity::Line { id, .. } => *id,
            SketchEntity::Arc { id, .. } => *id,
            SketchEntity::Circle { id, .. } => *id,
            SketchEntity::Point { id, .. } => *id,
            SketchEntity::Spline { id, .. } => *id,
            SketchEntity::Ellipse { id, .. } => *id,
        }
    }

    /// Create a line entity
    pub fn line(start: Point2, end: Point2) -> Self {
        SketchEntity::Line {
            id: SketchEntityId::new(),
            start,
            end,
        }
    }

    /// Create a circle entity
    pub fn circle(center: Point2, radius: f64) -> Self {
        SketchEntity::Circle {
            id: SketchEntityId::new(),
            center,
            radius,
        }
    }

    /// Create an arc entity
    pub fn arc(center: Point2, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        SketchEntity::Arc {
            id: SketchEntityId::new(),
            center,
            radius,
            start_angle,
            end_angle,
        }
    }

    /// Create a point entity
    pub fn point(position: Point2) -> Self {
        SketchEntity::Point {
            id: SketchEntityId::new(),
            position,
        }
    }
}

/// Sketch constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SketchConstraint {
    /// Fixed position
    Fixed {
        entity_id: SketchEntityId,
    },

    /// Horizontal constraint
    Horizontal {
        entity_id: SketchEntityId,
    },

    /// Vertical constraint
    Vertical {
        entity_id: SketchEntityId,
    },

    /// Coincident points
    Coincident {
        entity1_id: SketchEntityId,
        point1_index: usize,
        entity2_id: SketchEntityId,
        point2_index: usize,
    },

    /// Parallel lines
    Parallel {
        entity1_id: SketchEntityId,
        entity2_id: SketchEntityId,
    },

    /// Perpendicular lines
    Perpendicular {
        entity1_id: SketchEntityId,
        entity2_id: SketchEntityId,
    },

    /// Tangent constraint
    Tangent {
        entity1_id: SketchEntityId,
        entity2_id: SketchEntityId,
    },

    /// Equal length/radius
    Equal {
        entity1_id: SketchEntityId,
        entity2_id: SketchEntityId,
    },

    /// Concentric circles/arcs
    Concentric {
        entity1_id: SketchEntityId,
        entity2_id: SketchEntityId,
    },

    /// Distance dimension
    Distance {
        entity1_id: SketchEntityId,
        entity2_id: Option<SketchEntityId>,
        value: f64,
    },

    /// Angle dimension
    Angle {
        entity1_id: SketchEntityId,
        entity2_id: SketchEntityId,
        value: f64,
    },

    /// Radius dimension
    Radius {
        entity_id: SketchEntityId,
        value: f64,
    },

    /// Symmetric about a line
    Symmetric {
        entity1_id: SketchEntityId,
        entity2_id: SketchEntityId,
        axis_id: SketchEntityId,
    },
}

/// A 2D sketch (profile for features)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Sketch {
    /// Sketch entities
    pub entities: Vec<SketchEntity>,

    /// Constraints
    pub constraints: Vec<SketchConstraint>,

    /// Sketch origin (on the work plane)
    pub origin: Point2,

    /// Is the sketch fully constrained?
    pub fully_constrained: bool,
}

impl Sketch {
    /// Create a new empty sketch
    pub fn new() -> Self {
        Self {
            entities: Vec::new(),
            constraints: Vec::new(),
            origin: Point2::origin(),
            fully_constrained: false,
        }
    }

    /// Add an entity
    pub fn add_entity(&mut self, entity: SketchEntity) -> SketchEntityId {
        let id = entity.id();
        self.entities.push(entity);
        id
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: SketchConstraint) {
        self.constraints.push(constraint);
    }

    /// Get entity by ID
    pub fn get_entity(&self, id: SketchEntityId) -> Option<&SketchEntity> {
        self.entities.iter().find(|e| e.id() == id)
    }

    /// Create a rectangle sketch
    pub fn rectangle(width: f64, height: f64) -> Self {
        let mut sketch = Self::new();

        let p1 = Point2::new(0.0, 0.0);
        let p2 = Point2::new(width, 0.0);
        let p3 = Point2::new(width, height);
        let p4 = Point2::new(0.0, height);

        sketch.add_entity(SketchEntity::line(p1, p2));
        sketch.add_entity(SketchEntity::line(p2, p3));
        sketch.add_entity(SketchEntity::line(p3, p4));
        sketch.add_entity(SketchEntity::line(p4, p1));

        sketch.fully_constrained = true;
        sketch
    }

    /// Create a circle sketch
    pub fn circle(radius: f64) -> Self {
        let mut sketch = Self::new();
        sketch.add_entity(SketchEntity::circle(Point2::origin(), radius));
        sketch.fully_constrained = true;
        sketch
    }

    /// Create a centered rectangle sketch
    pub fn centered_rectangle(width: f64, height: f64) -> Self {
        let mut sketch = Self::new();

        let hw = width / 2.0;
        let hh = height / 2.0;

        let p1 = Point2::new(-hw, -hh);
        let p2 = Point2::new(hw, -hh);
        let p3 = Point2::new(hw, hh);
        let p4 = Point2::new(-hw, hh);

        sketch.add_entity(SketchEntity::line(p1, p2));
        sketch.add_entity(SketchEntity::line(p2, p3));
        sketch.add_entity(SketchEntity::line(p3, p4));
        sketch.add_entity(SketchEntity::line(p4, p1));

        sketch.fully_constrained = true;
        sketch
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sketch_rectangle() {
        let sketch = Sketch::rectangle(100.0, 50.0);
        assert_eq!(sketch.entities.len(), 4);
        assert!(sketch.fully_constrained);
    }

    #[test]
    fn test_sketch_circle() {
        let sketch = Sketch::circle(25.0);
        assert_eq!(sketch.entities.len(), 1);
    }
}
