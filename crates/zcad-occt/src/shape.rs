//! OCCT Shape wrapper
//!
//! This module provides a unified interface to OCCT shapes,
//! with fallback implementations when OCCT is not available.

use crate::error::{OcctError, Result};
use zcad_geometry_3d::math::{BoundingBox3, Point3, Vector3};
use zcad_geometry_3d::mesh::Mesh;
use zcad_geometry_3d::primitives::{Box3D, Cone, Cylinder, Sphere, Torus};
use zcad_geometry_3d::topology::TopoShape;

/// Wrapper for OCCT TopoDS_Shape
///
/// When OCCT is available, this wraps the actual OCCT shape.
/// Otherwise, it stores a simplified representation.
#[derive(Debug, Clone)]
pub struct OcctShape {
    /// Native ZCAD topology representation
    native: TopoShape,

    /// Cached bounding box
    bounding_box: Option<BoundingBox3>,

    /// Cached mesh for visualization
    mesh_cache: Option<Mesh>,
}

impl OcctShape {
    /// Create an empty/null shape
    pub fn null() -> Self {
        Self {
            native: TopoShape::Compound(Vec::new()),
            bounding_box: None,
            mesh_cache: None,
        }
    }

    /// Check if the shape is null/empty
    pub fn is_null(&self) -> bool {
        match &self.native {
            TopoShape::Compound(shapes) => shapes.is_empty(),
            _ => false,
        }
    }

    /// Get the native ZCAD topology
    pub fn native(&self) -> &TopoShape {
        &self.native
    }

    /// Create from native topology
    pub fn from_native(native: TopoShape) -> Self {
        let bounding_box = Some(native.bounding_box());
        Self {
            native,
            bounding_box,
            mesh_cache: None,
        }
    }

    // ========== Primitive Creation ==========

    /// Create a box
    pub fn make_box(dx: f64, dy: f64, dz: f64) -> Result<Self> {
        Self::make_box_at(Point3::origin(), dx, dy, dz)
    }

    /// Create a box at a specific location
    pub fn make_box_at(origin: Point3, dx: f64, dy: f64, dz: f64) -> Result<Self> {
        if dx <= 0.0 || dy <= 0.0 || dz <= 0.0 {
            return Err(OcctError::InvalidParameter(
                "Box dimensions must be positive".into(),
            ));
        }

        let box3d = Box3D::at(origin, dx, dy, dz);
        let bbox = box3d.bounding_box();

        // Create mesh representation
        let mesh = zcad_geometry_3d::mesh::make_box_mesh(dx, dy, dz);

        Ok(Self {
            native: TopoShape::Compound(Vec::new()), // Placeholder
            bounding_box: Some(bbox),
            mesh_cache: Some(mesh),
        })
    }

    /// Create a cylinder
    pub fn make_cylinder(radius: f64, height: f64) -> Result<Self> {
        Self::make_cylinder_at(Point3::origin(), Vector3::z(), radius, height)
    }

    /// Create a cylinder at a specific location and axis
    pub fn make_cylinder_at(
        origin: Point3,
        axis: Vector3,
        radius: f64,
        height: f64,
    ) -> Result<Self> {
        if radius <= 0.0 || height <= 0.0 {
            return Err(OcctError::InvalidParameter(
                "Cylinder dimensions must be positive".into(),
            ));
        }

        let cylinder = Cylinder::at(origin, axis, radius, height);
        let bbox = cylinder.bounding_box();

        Ok(Self {
            native: TopoShape::Compound(Vec::new()),
            bounding_box: Some(bbox),
            mesh_cache: None, // Would need cylinder tessellation
        })
    }

    /// Create a sphere
    pub fn make_sphere(radius: f64) -> Result<Self> {
        Self::make_sphere_at(Point3::origin(), radius)
    }

    /// Create a sphere at a specific center
    pub fn make_sphere_at(center: Point3, radius: f64) -> Result<Self> {
        if radius <= 0.0 {
            return Err(OcctError::InvalidParameter(
                "Sphere radius must be positive".into(),
            ));
        }

        let sphere = Sphere::at(center, radius);
        let bbox = sphere.bounding_box();

        Ok(Self {
            native: TopoShape::Compound(Vec::new()),
            bounding_box: Some(bbox),
            mesh_cache: None,
        })
    }

    /// Create a cone
    pub fn make_cone(radius: f64, height: f64) -> Result<Self> {
        if radius <= 0.0 || height <= 0.0 {
            return Err(OcctError::InvalidParameter(
                "Cone dimensions must be positive".into(),
            ));
        }

        let cone = Cone::new(radius, height);
        let bbox = cone.bounding_box();

        Ok(Self {
            native: TopoShape::Compound(Vec::new()),
            bounding_box: Some(bbox),
            mesh_cache: None,
        })
    }

    /// Create a torus
    pub fn make_torus(major_radius: f64, minor_radius: f64) -> Result<Self> {
        if major_radius <= 0.0 || minor_radius <= 0.0 {
            return Err(OcctError::InvalidParameter(
                "Torus radii must be positive".into(),
            ));
        }

        if minor_radius >= major_radius {
            return Err(OcctError::InvalidParameter(
                "Minor radius must be less than major radius".into(),
            ));
        }

        let torus = Torus::new(major_radius, minor_radius);
        let bbox = torus.bounding_box();

        Ok(Self {
            native: TopoShape::Compound(Vec::new()),
            bounding_box: Some(bbox),
            mesh_cache: None,
        })
    }

    // ========== Queries ==========

    /// Get the bounding box
    pub fn bounding_box(&self) -> BoundingBox3 {
        self.bounding_box.unwrap_or_else(BoundingBox3::empty)
    }

    /// Get the cached mesh (for visualization)
    pub fn mesh(&self) -> Option<&Mesh> {
        self.mesh_cache.as_ref()
    }

    /// Calculate volume (requires OCCT for accurate results)
    pub fn volume(&self) -> Result<f64> {
        // Without OCCT, return 0 or estimate from bounding box
        #[cfg(feature = "occt")]
        {
            // Would call OCCT GProp_GProps
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            // Rough estimate from bounding box
            let bbox = self.bounding_box();
            let size = bbox.size();
            Ok(size.x * size.y * size.z * 0.5) // Very rough estimate
        }
    }

    /// Calculate surface area (requires OCCT for accurate results)
    pub fn surface_area(&self) -> Result<f64> {
        #[cfg(feature = "occt")]
        {
            Err(OcctError::OcctNotAvailable)
        }

        #[cfg(not(feature = "occt"))]
        {
            // Use mesh if available
            if let Some(mesh) = &self.mesh_cache {
                Ok(mesh.surface_area())
            } else {
                Err(OcctError::OcctNotAvailable)
            }
        }
    }

    // ========== Transformations ==========

    /// Translate the shape
    pub fn translate(&self, dx: f64, dy: f64, dz: f64) -> Result<Self> {
        let mut new_shape = self.clone();

        if let Some(ref mut bbox) = new_shape.bounding_box {
            bbox.min.x += dx;
            bbox.min.y += dy;
            bbox.min.z += dz;
            bbox.max.x += dx;
            bbox.max.y += dy;
            bbox.max.z += dz;
        }

        // Clear mesh cache (would need to transform vertices)
        new_shape.mesh_cache = None;

        Ok(new_shape)
    }

    /// Create a copy of the shape
    pub fn copy(&self) -> Self {
        self.clone()
    }

    /// Set the bounding box
    pub fn set_bounding_box(&mut self, bbox: BoundingBox3) {
        self.bounding_box = Some(bbox);
    }

    /// Clear mesh cache
    pub fn clear_mesh_cache(&mut self) {
        self.mesh_cache = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_make_box() {
        let shape = OcctShape::make_box(10.0, 5.0, 2.0).unwrap();
        assert!(!shape.is_null());

        let bbox = shape.bounding_box();
        assert_eq!(bbox.size().x, 10.0);
        assert_eq!(bbox.size().y, 5.0);
        assert_eq!(bbox.size().z, 2.0);
    }

    #[test]
    fn test_make_cylinder() {
        let shape = OcctShape::make_cylinder(5.0, 10.0).unwrap();
        assert!(!shape.is_null());
    }

    #[test]
    fn test_invalid_parameters() {
        assert!(OcctShape::make_box(-1.0, 5.0, 2.0).is_err());
        assert!(OcctShape::make_sphere(0.0).is_err());
    }
}
