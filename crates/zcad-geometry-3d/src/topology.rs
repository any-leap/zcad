//! B-Rep Topology structures
//!
//! This module defines the boundary representation (B-Rep) topology
//! used for solid modeling.
//!
//! # Topology Hierarchy
//!
//! ```text
//! Compound
//!   └── Solid
//!         └── Shell
//!               └── Face
//!                     └── Wire
//!                           └── Edge
//!                                 └── Vertex
//! ```

use crate::math::{BoundingBox3, Point3, Vector3};
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

// ID generators
static VERTEX_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static EDGE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static WIRE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static FACE_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static SHELL_ID_COUNTER: AtomicU64 = AtomicU64::new(1);
static SOLID_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

/// Vertex ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VertexId(pub u64);

impl VertexId {
    pub fn new() -> Self {
        Self(VERTEX_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for VertexId {
    fn default() -> Self {
        Self::new()
    }
}

/// Edge ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EdgeId(pub u64);

impl EdgeId {
    pub fn new() -> Self {
        Self(EDGE_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for EdgeId {
    fn default() -> Self {
        Self::new()
    }
}

/// Wire ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WireId(pub u64);

impl WireId {
    pub fn new() -> Self {
        Self(WIRE_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for WireId {
    fn default() -> Self {
        Self::new()
    }
}

/// Face ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct FaceId(pub u64);

impl FaceId {
    pub fn new() -> Self {
        Self(FACE_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for FaceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Shell ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ShellId(pub u64);

impl ShellId {
    pub fn new() -> Self {
        Self(SHELL_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for ShellId {
    fn default() -> Self {
        Self::new()
    }
}

/// Solid ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SolidId(pub u64);

impl SolidId {
    pub fn new() -> Self {
        Self(SOLID_ID_COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for SolidId {
    fn default() -> Self {
        Self::new()
    }
}

/// A vertex in 3D space (0D topology)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vertex {
    pub id: VertexId,
    pub point: Point3,
}

impl Vertex {
    pub fn new(point: Point3) -> Self {
        Self {
            id: VertexId::new(),
            point,
        }
    }

    pub fn at(x: f64, y: f64, z: f64) -> Self {
        Self::new(Point3::new(x, y, z))
    }
}

/// Curve type for edges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Curve {
    /// Line segment
    Line { start: Point3, end: Point3 },
    /// Circular arc
    Arc {
        center: Point3,
        radius: f64,
        start_angle: f64,
        end_angle: f64,
        axis: Vector3,
    },
    /// Full circle
    Circle {
        center: Point3,
        radius: f64,
        axis: Vector3,
    },
    /// Ellipse arc
    Ellipse {
        center: Point3,
        major_axis: Vector3,
        minor_axis: Vector3,
        start_angle: f64,
        end_angle: f64,
    },
    /// B-Spline curve
    BSpline {
        degree: u32,
        control_points: Vec<Point3>,
        knots: Vec<f64>,
        weights: Option<Vec<f64>>,
    },
}

impl Curve {
    /// Evaluate the curve at parameter t (0..1)
    pub fn point_at(&self, t: f64) -> Point3 {
        match self {
            Curve::Line { start, end } => {
                Point3::new(
                    start.x + t * (end.x - start.x),
                    start.y + t * (end.y - start.y),
                    start.z + t * (end.z - start.z),
                )
            }
            Curve::Circle { center, radius, axis } => {
                let angle = t * 2.0 * std::f64::consts::PI;
                // Compute perpendicular vectors
                let u = if axis.x.abs() < 0.9 {
                    axis.cross(&Vector3::x()).normalize()
                } else {
                    axis.cross(&Vector3::y()).normalize()
                };
                let v = axis.cross(&u);
                Point3::new(
                    center.x + radius * (angle.cos() * u.x + angle.sin() * v.x),
                    center.y + radius * (angle.cos() * u.y + angle.sin() * v.y),
                    center.z + radius * (angle.cos() * u.z + angle.sin() * v.z),
                )
            }
            Curve::Arc { center, radius, start_angle, end_angle, axis } => {
                let angle = start_angle + t * (end_angle - start_angle);
                let u = if axis.x.abs() < 0.9 {
                    axis.cross(&Vector3::x()).normalize()
                } else {
                    axis.cross(&Vector3::y()).normalize()
                };
                let v = axis.cross(&u);
                Point3::new(
                    center.x + radius * (angle.cos() * u.x + angle.sin() * v.x),
                    center.y + radius * (angle.cos() * u.y + angle.sin() * v.y),
                    center.z + radius * (angle.cos() * u.z + angle.sin() * v.z),
                )
            }
            _ => {
                // For complex curves, return a linear approximation
                // Real implementation would use proper curve evaluation
                Point3::origin()
            }
        }
    }

    /// Get the start point
    pub fn start_point(&self) -> Point3 {
        self.point_at(0.0)
    }

    /// Get the end point
    pub fn end_point(&self) -> Point3 {
        self.point_at(1.0)
    }
}

/// An edge in 3D space (1D topology)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Edge {
    pub id: EdgeId,
    pub curve: Curve,
    pub start_vertex: VertexId,
    pub end_vertex: VertexId,
}

impl Edge {
    pub fn new(curve: Curve, start: VertexId, end: VertexId) -> Self {
        Self {
            id: EdgeId::new(),
            curve,
            start_vertex: start,
            end_vertex: end,
        }
    }

    /// Create a line edge
    pub fn line(start: Point3, end: Point3) -> Self {
        let start_v = Vertex::new(start);
        let end_v = Vertex::new(end);
        Self::new(
            Curve::Line { start, end },
            start_v.id,
            end_v.id,
        )
    }
}

/// A wire is a connected sequence of edges (1D boundary)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Wire {
    pub id: WireId,
    pub edges: Vec<EdgeId>,
    pub is_closed: bool,
}

impl Wire {
    pub fn new(edges: Vec<EdgeId>, is_closed: bool) -> Self {
        Self {
            id: WireId::new(),
            edges,
            is_closed,
        }
    }

    pub fn empty() -> Self {
        Self {
            id: WireId::new(),
            edges: Vec::new(),
            is_closed: false,
        }
    }
}

/// Surface type for faces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Surface {
    /// Planar surface
    Plane {
        origin: Point3,
        normal: Vector3,
        u_dir: Vector3,
        v_dir: Vector3,
    },
    /// Cylindrical surface
    Cylinder {
        origin: Point3,
        axis: Vector3,
        radius: f64,
    },
    /// Conical surface
    Cone {
        apex: Point3,
        axis: Vector3,
        half_angle: f64,
    },
    /// Spherical surface
    Sphere {
        center: Point3,
        radius: f64,
    },
    /// Toroidal surface
    Torus {
        center: Point3,
        axis: Vector3,
        major_radius: f64,
        minor_radius: f64,
    },
    /// B-Spline surface
    BSpline {
        degree_u: u32,
        degree_v: u32,
        control_points: Vec<Vec<Point3>>,
        knots_u: Vec<f64>,
        knots_v: Vec<f64>,
        weights: Option<Vec<Vec<f64>>>,
    },
}

impl Surface {
    /// Create a planar surface in the XY plane
    pub fn xy_plane() -> Self {
        Surface::Plane {
            origin: Point3::origin(),
            normal: Vector3::z(),
            u_dir: Vector3::x(),
            v_dir: Vector3::y(),
        }
    }

    /// Get the normal at a point on the surface
    pub fn normal_at(&self, _u: f64, _v: f64) -> Vector3 {
        match self {
            Surface::Plane { normal, .. } => *normal,
            Surface::Sphere { .. } => Vector3::z(), // Simplified
            _ => Vector3::z(), // Placeholder
        }
    }
}

/// A face is a bounded portion of a surface (2D topology)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Face {
    pub id: FaceId,
    pub surface: Surface,
    pub outer_wire: WireId,
    pub inner_wires: Vec<WireId>, // Holes
    pub reversed: bool,
}

impl Face {
    pub fn new(surface: Surface, outer_wire: WireId) -> Self {
        Self {
            id: FaceId::new(),
            surface,
            outer_wire,
            inner_wires: Vec::new(),
            reversed: false,
        }
    }

    pub fn with_holes(mut self, holes: Vec<WireId>) -> Self {
        self.inner_wires = holes;
        self
    }
}

/// A shell is a connected set of faces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shell {
    pub id: ShellId,
    pub faces: Vec<FaceId>,
    pub is_closed: bool,
}

impl Shell {
    pub fn new(faces: Vec<FaceId>, is_closed: bool) -> Self {
        Self {
            id: ShellId::new(),
            faces,
            is_closed,
        }
    }
}

/// A solid is a closed volume bounded by shells
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Solid {
    pub id: SolidId,
    /// Outer shell (required)
    pub outer_shell: ShellId,
    /// Inner shells (voids/cavities)
    pub inner_shells: Vec<ShellId>,
    /// Cached bounding box
    pub bounding_box: Option<BoundingBox3>,
}

impl Solid {
    pub fn new(outer_shell: ShellId) -> Self {
        Self {
            id: SolidId::new(),
            outer_shell,
            inner_shells: Vec::new(),
            bounding_box: None,
        }
    }

    pub fn with_voids(mut self, voids: Vec<ShellId>) -> Self {
        self.inner_shells = voids;
        self
    }

    pub fn with_bounding_box(mut self, bbox: BoundingBox3) -> Self {
        self.bounding_box = Some(bbox);
        self
    }
}

/// Unified topological shape type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TopoShape {
    Vertex(Vertex),
    Edge(Edge),
    Wire(Wire),
    Face(Face),
    Shell(Shell),
    Solid(Solid),
    Compound(Vec<TopoShape>),
}

impl TopoShape {
    /// Get the bounding box of the shape
    pub fn bounding_box(&self) -> BoundingBox3 {
        match self {
            TopoShape::Vertex(v) => BoundingBox3::from_points([v.point]),
            TopoShape::Solid(s) => s.bounding_box.unwrap_or_else(BoundingBox3::empty),
            TopoShape::Compound(shapes) => {
                let mut bbox = BoundingBox3::empty();
                for shape in shapes {
                    bbox = bbox.union(&shape.bounding_box());
                }
                bbox
            }
            _ => BoundingBox3::empty(), // Simplified
        }
    }

    /// Check if this is a solid
    pub fn is_solid(&self) -> bool {
        matches!(self, TopoShape::Solid(_))
    }

    /// Try to get as a solid
    pub fn as_solid(&self) -> Option<&Solid> {
        match self {
            TopoShape::Solid(s) => Some(s),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vertex_creation() {
        let v1 = Vertex::at(1.0, 2.0, 3.0);
        let v2 = Vertex::at(4.0, 5.0, 6.0);

        assert_ne!(v1.id, v2.id);
        assert_eq!(v1.point, Point3::new(1.0, 2.0, 3.0));
    }

    #[test]
    fn test_line_curve() {
        let curve = Curve::Line {
            start: Point3::new(0.0, 0.0, 0.0),
            end: Point3::new(10.0, 0.0, 0.0),
        };

        let mid = curve.point_at(0.5);
        assert_eq!(mid, Point3::new(5.0, 0.0, 0.0));
    }
}
