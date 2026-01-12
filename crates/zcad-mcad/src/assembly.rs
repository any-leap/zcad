//! Assembly design
//!
//! Assemblies are collections of parts with constraints between them.

use crate::error::Result;
use crate::part::{Part, PartId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use zcad_geometry_3d::math::{Point3, Vector3};
use zcad_geometry_3d::transform::Transform3D;

static ASSEMBLY_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static COMPONENT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Assembly ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct AssemblyId(pub u64);

impl AssemblyId {
    pub fn new() -> Self {
        Self(ASSEMBLY_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for AssemblyId {
    fn default() -> Self {
        Self::new()
    }
}

/// Component ID (instance of a part in an assembly)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ComponentId(pub u64);

impl ComponentId {
    pub fn new() -> Self {
        Self(COMPONENT_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ComponentId {
    fn default() -> Self {
        Self::new()
    }
}

/// A component (instance of a part) in an assembly
#[derive(Debug, Clone)]
pub struct Component {
    /// Component ID
    pub id: ComponentId,

    /// Reference to the source part
    pub part_id: PartId,

    /// Instance name
    pub name: String,

    /// Transformation (position/orientation)
    pub transform: Transform3D,

    /// Is the component suppressed?
    pub suppressed: bool,

    /// Is the component fixed (cannot move)?
    pub fixed: bool,
}

impl Component {
    /// Create a new component
    pub fn new(part_id: PartId, name: impl Into<String>) -> Self {
        Self {
            id: ComponentId::new(),
            part_id,
            name: name.into(),
            transform: Transform3D::identity(),
            suppressed: false,
            fixed: false,
        }
    }

    /// Set the component transform
    pub fn with_transform(mut self, transform: Transform3D) -> Self {
        self.transform = transform;
        self
    }

    /// Set the component as fixed
    pub fn fixed(mut self) -> Self {
        self.fixed = true;
        self
    }

    /// Move the component
    pub fn translate(&mut self, dx: f64, dy: f64, dz: f64) {
        self.transform = self.transform.then(&Transform3D::translation(dx, dy, dz));
    }

    /// Rotate the component
    pub fn rotate(&mut self, axis: Vector3, angle: f64) {
        self.transform = self.transform.then(&Transform3D::rotation(axis, angle));
    }
}

/// Assembly constraint types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssemblyConstraint {
    /// Fix a component in place
    Fixed {
        component_id: ComponentId,
    },

    /// Coincident (points, faces, or axes)
    Coincident {
        component1_id: ComponentId,
        geometry1: ConstraintGeometry,
        component2_id: ComponentId,
        geometry2: ConstraintGeometry,
    },

    /// Parallel faces/axes
    Parallel {
        component1_id: ComponentId,
        geometry1: ConstraintGeometry,
        component2_id: ComponentId,
        geometry2: ConstraintGeometry,
    },

    /// Perpendicular faces/axes
    Perpendicular {
        component1_id: ComponentId,
        geometry1: ConstraintGeometry,
        component2_id: ComponentId,
        geometry2: ConstraintGeometry,
    },

    /// Distance between geometries
    Distance {
        component1_id: ComponentId,
        geometry1: ConstraintGeometry,
        component2_id: ComponentId,
        geometry2: ConstraintGeometry,
        value: f64,
    },

    /// Angle between geometries
    Angle {
        component1_id: ComponentId,
        geometry1: ConstraintGeometry,
        component2_id: ComponentId,
        geometry2: ConstraintGeometry,
        value: f64,
    },

    /// Concentric (cylindrical faces)
    Concentric {
        component1_id: ComponentId,
        geometry1: ConstraintGeometry,
        component2_id: ComponentId,
        geometry2: ConstraintGeometry,
    },

    /// Tangent constraint
    Tangent {
        component1_id: ComponentId,
        geometry1: ConstraintGeometry,
        component2_id: ComponentId,
        geometry2: ConstraintGeometry,
    },
}

/// Geometry reference for constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintGeometry {
    /// A point
    Point(Point3),
    /// An axis (origin + direction)
    Axis { origin: Point3, direction: Vector3 },
    /// A plane (origin + normal)
    Plane { origin: Point3, normal: Vector3 },
    /// A face by ID
    Face(u64),
    /// An edge by ID
    Edge(u64),
    /// A vertex by ID
    Vertex(u64),
}

/// An assembly of components
#[derive(Debug)]
pub struct Assembly {
    /// Assembly ID
    pub id: AssemblyId,

    /// Assembly name
    pub name: String,

    /// Components (instances)
    pub components: HashMap<ComponentId, Component>,

    /// Constraints
    pub constraints: Vec<AssemblyConstraint>,

    /// Sub-assemblies
    pub sub_assemblies: Vec<AssemblyId>,
}

impl Assembly {
    /// Create a new empty assembly
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: AssemblyId::new(),
            name: name.into(),
            components: HashMap::new(),
            constraints: Vec::new(),
            sub_assemblies: Vec::new(),
        }
    }

    /// Add a component (part instance)
    pub fn add_component(&mut self, component: Component) -> ComponentId {
        let id = component.id;
        self.components.insert(id, component);
        id
    }

    /// Add a part as a new component
    pub fn add_part(&mut self, part_id: PartId, name: impl Into<String>) -> ComponentId {
        let component = Component::new(part_id, name);
        self.add_component(component)
    }

    /// Remove a component
    pub fn remove_component(&mut self, id: ComponentId) -> bool {
        // Also remove constraints referencing this component
        self.constraints.retain(|c| {
            !matches!(c,
                AssemblyConstraint::Fixed { component_id } if *component_id == id
            )
        });
        self.components.remove(&id).is_some()
    }

    /// Get a component
    pub fn get_component(&self, id: ComponentId) -> Option<&Component> {
        self.components.get(&id)
    }

    /// Get a mutable component
    pub fn get_component_mut(&mut self, id: ComponentId) -> Option<&mut Component> {
        self.components.get_mut(&id)
    }

    /// Add a constraint
    pub fn add_constraint(&mut self, constraint: AssemblyConstraint) {
        self.constraints.push(constraint);
    }

    /// Solve assembly constraints (updates component transforms)
    pub fn solve(&mut self) -> Result<()> {
        // TODO: Implement constraint solver
        // This would use an iterative solver to position components
        Ok(())
    }

    /// Get component count
    pub fn component_count(&self) -> usize {
        self.components.len()
    }

    /// Get constraint count
    pub fn constraint_count(&self) -> usize {
        self.constraints.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assembly_creation() {
        let mut assembly = Assembly::new("Test Assembly");

        let part_id = PartId::new();
        let comp_id = assembly.add_part(part_id, "Component 1");

        assert_eq!(assembly.component_count(), 1);
        assert!(assembly.get_component(comp_id).is_some());
    }
}
