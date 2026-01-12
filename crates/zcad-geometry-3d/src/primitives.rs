//! 3D primitive shapes
//!
//! This module provides factory methods for creating common 3D shapes.

use crate::error::Result;
use crate::math::{BoundingBox3, Point3, Vector3};
use crate::topology::Solid;
use serde::{Deserialize, Serialize};

/// Box (rectangular prism) primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Box3D {
    pub origin: Point3,
    pub dx: f64,
    pub dy: f64,
    pub dz: f64,
}

impl Box3D {
    /// Create a new box at origin with given dimensions
    pub fn new(dx: f64, dy: f64, dz: f64) -> Self {
        Self {
            origin: Point3::origin(),
            dx,
            dy,
            dz,
        }
    }

    /// Create a box at a specific origin
    pub fn at(origin: Point3, dx: f64, dy: f64, dz: f64) -> Self {
        Self { origin, dx, dy, dz }
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        BoundingBox3::new(
            self.origin,
            Point3::new(
                self.origin.x + self.dx,
                self.origin.y + self.dy,
                self.origin.z + self.dz,
            ),
        )
    }

    /// Get the center point
    pub fn center(&self) -> Point3 {
        Point3::new(
            self.origin.x + self.dx / 2.0,
            self.origin.y + self.dy / 2.0,
            self.origin.z + self.dz / 2.0,
        )
    }

    /// Get the 8 corner vertices
    pub fn vertices(&self) -> [Point3; 8] {
        let (x0, y0, z0) = (self.origin.x, self.origin.y, self.origin.z);
        let (x1, y1, z1) = (x0 + self.dx, y0 + self.dy, z0 + self.dz);

        [
            Point3::new(x0, y0, z0),
            Point3::new(x1, y0, z0),
            Point3::new(x1, y1, z0),
            Point3::new(x0, y1, z0),
            Point3::new(x0, y0, z1),
            Point3::new(x1, y0, z1),
            Point3::new(x1, y1, z1),
            Point3::new(x0, y1, z1),
        ]
    }

    /// Calculate volume
    pub fn volume(&self) -> f64 {
        self.dx * self.dy * self.dz
    }

    /// Calculate surface area
    pub fn surface_area(&self) -> f64 {
        2.0 * (self.dx * self.dy + self.dy * self.dz + self.dz * self.dx)
    }
}

/// Cylinder primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cylinder {
    pub origin: Point3,
    pub axis: Vector3,
    pub radius: f64,
    pub height: f64,
}

impl Cylinder {
    /// Create a cylinder along Z axis at origin
    pub fn new(radius: f64, height: f64) -> Self {
        Self {
            origin: Point3::origin(),
            axis: Vector3::z(),
            radius,
            height,
        }
    }

    /// Create a cylinder at a specific location and axis
    pub fn at(origin: Point3, axis: Vector3, radius: f64, height: f64) -> Self {
        Self {
            origin,
            axis: axis.normalize(),
            radius,
            height,
        }
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        // Simplified: assumes axis-aligned cylinder along Z
        BoundingBox3::new(
            Point3::new(
                self.origin.x - self.radius,
                self.origin.y - self.radius,
                self.origin.z,
            ),
            Point3::new(
                self.origin.x + self.radius,
                self.origin.y + self.radius,
                self.origin.z + self.height,
            ),
        )
    }

    /// Calculate volume
    pub fn volume(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius * self.height
    }

    /// Calculate lateral surface area
    pub fn lateral_area(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius * self.height
    }

    /// Calculate total surface area (including caps)
    pub fn surface_area(&self) -> f64 {
        self.lateral_area() + 2.0 * std::f64::consts::PI * self.radius * self.radius
    }
}

/// Sphere primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
}

impl Sphere {
    /// Create a sphere at origin
    pub fn new(radius: f64) -> Self {
        Self {
            center: Point3::origin(),
            radius,
        }
    }

    /// Create a sphere at a specific center
    pub fn at(center: Point3, radius: f64) -> Self {
        Self { center, radius }
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        BoundingBox3::new(
            Point3::new(
                self.center.x - self.radius,
                self.center.y - self.radius,
                self.center.z - self.radius,
            ),
            Point3::new(
                self.center.x + self.radius,
                self.center.y + self.radius,
                self.center.z + self.radius,
            ),
        )
    }

    /// Calculate volume
    pub fn volume(&self) -> f64 {
        (4.0 / 3.0) * std::f64::consts::PI * self.radius.powi(3)
    }

    /// Calculate surface area
    pub fn surface_area(&self) -> f64 {
        4.0 * std::f64::consts::PI * self.radius * self.radius
    }
}

/// Cone primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cone {
    pub origin: Point3,
    pub axis: Vector3,
    pub radius: f64,
    pub height: f64,
}

impl Cone {
    /// Create a cone along Z axis at origin
    pub fn new(radius: f64, height: f64) -> Self {
        Self {
            origin: Point3::origin(),
            axis: Vector3::z(),
            radius,
            height,
        }
    }

    /// Create a cone at a specific location
    pub fn at(origin: Point3, axis: Vector3, radius: f64, height: f64) -> Self {
        Self {
            origin,
            axis: axis.normalize(),
            radius,
            height,
        }
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        BoundingBox3::new(
            Point3::new(
                self.origin.x - self.radius,
                self.origin.y - self.radius,
                self.origin.z,
            ),
            Point3::new(
                self.origin.x + self.radius,
                self.origin.y + self.radius,
                self.origin.z + self.height,
            ),
        )
    }

    /// Calculate volume
    pub fn volume(&self) -> f64 {
        (1.0 / 3.0) * std::f64::consts::PI * self.radius * self.radius * self.height
    }

    /// Calculate lateral surface area
    pub fn lateral_area(&self) -> f64 {
        let slant = (self.radius * self.radius + self.height * self.height).sqrt();
        std::f64::consts::PI * self.radius * slant
    }

    /// Calculate total surface area (including base)
    pub fn surface_area(&self) -> f64 {
        self.lateral_area() + std::f64::consts::PI * self.radius * self.radius
    }
}

/// Torus primitive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Torus {
    pub center: Point3,
    pub axis: Vector3,
    pub major_radius: f64,
    pub minor_radius: f64,
}

impl Torus {
    /// Create a torus at origin along Z axis
    pub fn new(major_radius: f64, minor_radius: f64) -> Self {
        Self {
            center: Point3::origin(),
            axis: Vector3::z(),
            major_radius,
            minor_radius,
        }
    }

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        let r = self.major_radius + self.minor_radius;
        BoundingBox3::new(
            Point3::new(
                self.center.x - r,
                self.center.y - r,
                self.center.z - self.minor_radius,
            ),
            Point3::new(
                self.center.x + r,
                self.center.y + r,
                self.center.z + self.minor_radius,
            ),
        )
    }

    /// Calculate volume
    pub fn volume(&self) -> f64 {
        2.0 * std::f64::consts::PI.powi(2)
            * self.major_radius
            * self.minor_radius
            * self.minor_radius
    }

    /// Calculate surface area
    pub fn surface_area(&self) -> f64 {
        4.0 * std::f64::consts::PI.powi(2) * self.major_radius * self.minor_radius
    }
}

/// Trait for primitive shapes that can be converted to solids
pub trait ToSolid {
    /// Convert to a solid (requires OCCT for actual implementation)
    fn to_solid(&self) -> Result<Solid>;
}

// Note: Actual ToSolid implementations will be in zcad-occt
// Here we provide placeholder implementations

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_box_volume() {
        let box3d = Box3D::new(10.0, 5.0, 2.0);
        assert_eq!(box3d.volume(), 100.0);
    }

    #[test]
    fn test_sphere_volume() {
        let sphere = Sphere::new(1.0);
        let expected = (4.0 / 3.0) * std::f64::consts::PI;
        assert!((sphere.volume() - expected).abs() < 1e-10);
    }

    #[test]
    fn test_cylinder_volume() {
        let cylinder = Cylinder::new(1.0, 10.0);
        let expected = std::f64::consts::PI * 10.0;
        assert!((cylinder.volume() - expected).abs() < 1e-10);
    }
}
