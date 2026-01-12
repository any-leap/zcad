//! 3D math types and utilities

use nalgebra as na;
use serde::{Deserialize, Serialize};

/// 3D point type
pub type Point3 = na::Point3<f64>;

/// 3D vector type
pub type Vector3 = na::Vector3<f64>;

/// 4x4 transformation matrix
pub type Matrix4 = na::Matrix4<f64>;

/// 3x3 rotation matrix
pub type Matrix3 = na::Matrix3<f64>;

/// Unit quaternion for rotations
pub type UnitQuaternion = na::UnitQuaternion<f64>;

/// Numerical tolerance for geometry comparisons
pub const TOLERANCE: f64 = 1e-9;

/// Angular tolerance (radians)
pub const ANGULAR_TOLERANCE: f64 = 1e-9;

/// Check if two floats are approximately equal
#[inline]
pub fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < TOLERANCE
}

/// Check if a float is approximately zero
#[inline]
pub fn approx_zero(a: f64) -> bool {
    a.abs() < TOLERANCE
}

/// Check if two points are approximately equal
#[inline]
pub fn points_approx_eq(a: &Point3, b: &Point3) -> bool {
    (a - b).norm() < TOLERANCE
}

/// Check if two vectors are approximately equal
#[inline]
pub fn vectors_approx_eq(a: &Vector3, b: &Vector3) -> bool {
    (a - b).norm() < TOLERANCE
}

/// 3D axis-aligned bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox3 {
    pub min: Point3,
    pub max: Point3,
}

impl BoundingBox3 {
    /// Create a new bounding box
    pub fn new(min: Point3, max: Point3) -> Self {
        Self { min, max }
    }

    /// Create an empty (invalid) bounding box
    pub fn empty() -> Self {
        Self {
            min: Point3::new(f64::MAX, f64::MAX, f64::MAX),
            max: Point3::new(f64::MIN, f64::MIN, f64::MIN),
        }
    }

    /// Check if the bounding box is valid (non-empty)
    pub fn is_valid(&self) -> bool {
        self.min.x <= self.max.x && self.min.y <= self.max.y && self.min.z <= self.max.z
    }

    /// Create a bounding box from a set of points
    pub fn from_points(points: impl IntoIterator<Item = Point3>) -> Self {
        let mut bbox = Self::empty();
        for p in points {
            bbox.expand_to_include(&p);
        }
        bbox
    }

    /// Expand the bounding box to include a point
    pub fn expand_to_include(&mut self, point: &Point3) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.min.z = self.min.z.min(point.z);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
        self.max.z = self.max.z.max(point.z);
    }

    /// Union of two bounding boxes
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: Point3::new(
                self.min.x.min(other.min.x),
                self.min.y.min(other.min.y),
                self.min.z.min(other.min.z),
            ),
            max: Point3::new(
                self.max.x.max(other.max.x),
                self.max.y.max(other.max.y),
                self.max.z.max(other.max.z),
            ),
        }
    }

    /// Check if two bounding boxes intersect
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
            && self.min.z <= other.max.z
            && self.max.z >= other.min.z
    }

    /// Check if the bounding box contains a point
    pub fn contains(&self, point: &Point3) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
            && point.z >= self.min.z
            && point.z <= self.max.z
    }

    /// Get the center of the bounding box
    pub fn center(&self) -> Point3 {
        Point3::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
            (self.min.z + self.max.z) / 2.0,
        )
    }

    /// Get the size (dimensions) of the bounding box
    pub fn size(&self) -> Vector3 {
        Vector3::new(
            self.max.x - self.min.x,
            self.max.y - self.min.y,
            self.max.z - self.min.z,
        )
    }

    /// Get the diagonal length
    pub fn diagonal(&self) -> f64 {
        self.size().norm()
    }

    /// Expand the bounding box by a margin
    pub fn expand(&self, margin: f64) -> Self {
        Self {
            min: Point3::new(
                self.min.x - margin,
                self.min.y - margin,
                self.min.z - margin,
            ),
            max: Point3::new(
                self.max.x + margin,
                self.max.y + margin,
                self.max.z + margin,
            ),
        }
    }
}

impl Default for BoundingBox3 {
    fn default() -> Self {
        Self::empty()
    }
}

/// Plane in 3D space (ax + by + cz + d = 0)
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Plane {
    /// Normal vector (a, b, c)
    pub normal: Vector3,
    /// Distance from origin (d)
    pub d: f64,
}

impl Plane {
    /// Create a plane from normal and a point on the plane
    pub fn from_normal_and_point(normal: Vector3, point: &Point3) -> Self {
        let n = normal.normalize();
        let d = -n.dot(&point.coords);
        Self { normal: n, d }
    }

    /// Create a plane from three points
    pub fn from_three_points(p1: &Point3, p2: &Point3, p3: &Point3) -> Option<Self> {
        let v1 = p2 - p1;
        let v2 = p3 - p1;
        let normal = v1.cross(&v2);

        if approx_zero(normal.norm()) {
            return None; // Points are collinear
        }

        Some(Self::from_normal_and_point(normal, p1))
    }

    /// XY plane (z = 0)
    pub fn xy() -> Self {
        Self {
            normal: Vector3::z(),
            d: 0.0,
        }
    }

    /// XZ plane (y = 0)
    pub fn xz() -> Self {
        Self {
            normal: Vector3::y(),
            d: 0.0,
        }
    }

    /// YZ plane (x = 0)
    pub fn yz() -> Self {
        Self {
            normal: Vector3::x(),
            d: 0.0,
        }
    }

    /// Signed distance from a point to the plane
    pub fn signed_distance(&self, point: &Point3) -> f64 {
        self.normal.dot(&point.coords) + self.d
    }

    /// Project a point onto the plane
    pub fn project_point(&self, point: &Point3) -> Point3 {
        let dist = self.signed_distance(point);
        point - self.normal * dist
    }

    /// Check if a point is on the plane (within tolerance)
    pub fn contains_point(&self, point: &Point3) -> bool {
        approx_zero(self.signed_distance(point))
    }
}

/// Axis enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Axis {
    X,
    Y,
    Z,
}

impl Axis {
    /// Get the unit vector for this axis
    pub fn unit_vector(&self) -> Vector3 {
        match self {
            Axis::X => Vector3::x(),
            Axis::Y => Vector3::y(),
            Axis::Z => Vector3::z(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box() {
        let mut bbox = BoundingBox3::empty();
        bbox.expand_to_include(&Point3::new(0.0, 0.0, 0.0));
        bbox.expand_to_include(&Point3::new(10.0, 5.0, 3.0));

        assert!(bbox.is_valid());
        assert_eq!(bbox.center(), Point3::new(5.0, 2.5, 1.5));
        assert!(bbox.contains(&Point3::new(5.0, 2.5, 1.5)));
        assert!(!bbox.contains(&Point3::new(15.0, 0.0, 0.0)));
    }

    #[test]
    fn test_plane() {
        let plane = Plane::xy();
        assert!(plane.contains_point(&Point3::new(1.0, 2.0, 0.0)));
        assert!(!plane.contains_point(&Point3::new(1.0, 2.0, 1.0)));

        let projected = plane.project_point(&Point3::new(1.0, 2.0, 5.0));
        assert!(points_approx_eq(&projected, &Point3::new(1.0, 2.0, 0.0)));
    }
}
