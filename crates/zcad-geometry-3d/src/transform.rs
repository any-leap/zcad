//! 3D transformations

use crate::math::{Matrix4, Point3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};

/// 3D affine transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform3D {
    /// 4x4 transformation matrix
    matrix: Matrix4,
}

impl Transform3D {
    /// Create an identity transform
    pub fn identity() -> Self {
        Self {
            matrix: Matrix4::identity(),
        }
    }

    /// Create a translation transform
    pub fn translation(dx: f64, dy: f64, dz: f64) -> Self {
        Self {
            matrix: Matrix4::new_translation(&Vector3::new(dx, dy, dz)),
        }
    }

    /// Create a translation from a vector
    pub fn from_translation(v: Vector3) -> Self {
        Self::translation(v.x, v.y, v.z)
    }

    /// Create a rotation around X axis
    pub fn rotation_x(angle: f64) -> Self {
        let q = UnitQuaternion::from_axis_angle(&Vector3::x_axis(), angle);
        Self {
            matrix: q.to_homogeneous(),
        }
    }

    /// Create a rotation around Y axis
    pub fn rotation_y(angle: f64) -> Self {
        let q = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), angle);
        Self {
            matrix: q.to_homogeneous(),
        }
    }

    /// Create a rotation around Z axis
    pub fn rotation_z(angle: f64) -> Self {
        let q = UnitQuaternion::from_axis_angle(&Vector3::z_axis(), angle);
        Self {
            matrix: q.to_homogeneous(),
        }
    }

    /// Create a rotation around an arbitrary axis
    pub fn rotation(axis: Vector3, angle: f64) -> Self {
        if let Some(unit_axis) = nalgebra::Unit::try_new(axis, 1e-10) {
            let q = UnitQuaternion::from_axis_angle(&unit_axis, angle);
            Self {
                matrix: q.to_homogeneous(),
            }
        } else {
            Self::identity()
        }
    }

    /// Create a uniform scale transform
    pub fn scale(factor: f64) -> Self {
        Self {
            matrix: Matrix4::new_scaling(factor),
        }
    }

    /// Create a non-uniform scale transform
    pub fn scale_xyz(sx: f64, sy: f64, sz: f64) -> Self {
        Self {
            matrix: Matrix4::new_nonuniform_scaling(&Vector3::new(sx, sy, sz)),
        }
    }

    /// Create a mirror transform across XY plane
    pub fn mirror_xy() -> Self {
        Self::scale_xyz(1.0, 1.0, -1.0)
    }

    /// Create a mirror transform across XZ plane
    pub fn mirror_xz() -> Self {
        Self::scale_xyz(1.0, -1.0, 1.0)
    }

    /// Create a mirror transform across YZ plane
    pub fn mirror_yz() -> Self {
        Self::scale_xyz(-1.0, 1.0, 1.0)
    }

    /// Get the transformation matrix
    pub fn matrix(&self) -> &Matrix4 {
        &self.matrix
    }

    /// Create from a matrix
    pub fn from_matrix(matrix: Matrix4) -> Self {
        Self { matrix }
    }

    /// Transform a point
    pub fn transform_point(&self, point: &Point3) -> Point3 {
        self.matrix.transform_point(point)
    }

    /// Transform a vector (ignores translation)
    pub fn transform_vector(&self, vector: &Vector3) -> Vector3 {
        self.matrix.transform_vector(vector)
    }

    /// Compose two transforms (self then other)
    pub fn then(&self, other: &Transform3D) -> Transform3D {
        Transform3D {
            matrix: other.matrix * self.matrix,
        }
    }

    /// Get the inverse transform
    pub fn inverse(&self) -> Option<Transform3D> {
        self.matrix.try_inverse().map(|m| Transform3D { matrix: m })
    }

    /// Check if this is the identity transform
    pub fn is_identity(&self) -> bool {
        (self.matrix - Matrix4::identity()).norm() < 1e-10
    }

    /// Get translation component
    pub fn translation_component(&self) -> Vector3 {
        Vector3::new(self.matrix[(0, 3)], self.matrix[(1, 3)], self.matrix[(2, 3)])
    }

    /// Create a transform that moves from one point to another
    pub fn point_to_point(from: &Point3, to: &Point3) -> Self {
        let delta = to - from;
        Self::from_translation(delta)
    }

    /// Create a transform from 3 points to 3 points
    /// Used for aligning coordinate systems
    pub fn align_points(
        from_origin: &Point3,
        from_x: &Point3,
        from_y: &Point3,
        to_origin: &Point3,
        to_x: &Point3,
        to_y: &Point3,
    ) -> Option<Self> {
        // Build source coordinate system
        let from_x_dir = (from_x - from_origin).normalize();
        let from_y_dir = (from_y - from_origin).normalize();
        let from_z_dir = from_x_dir.cross(&from_y_dir).normalize();
        let from_y_dir = from_z_dir.cross(&from_x_dir); // Orthogonalize

        // Build target coordinate system
        let to_x_dir = (to_x - to_origin).normalize();
        let to_y_dir = (to_y - to_origin).normalize();
        let to_z_dir = to_x_dir.cross(&to_y_dir).normalize();
        let to_y_dir = to_z_dir.cross(&to_x_dir);

        // Build rotation matrix
        let from_rot = nalgebra::Matrix3::from_columns(&[from_x_dir, from_y_dir, from_z_dir]);
        let to_rot = nalgebra::Matrix3::from_columns(&[to_x_dir, to_y_dir, to_z_dir]);
        
        let rotation = to_rot * from_rot.transpose();

        // Build full transform: translate to origin, rotate, translate to target
        let t1 = Self::from_translation(-from_origin.coords);
        let r = Self::from_matrix(rotation.to_homogeneous());
        let t2 = Self::from_translation(to_origin.coords);

        Some(t1.then(&r).then(&t2))
    }
}

impl Default for Transform3D {
    fn default() -> Self {
        Self::identity()
    }
}

impl std::ops::Mul for Transform3D {
    type Output = Transform3D;

    fn mul(self, rhs: Transform3D) -> Self::Output {
        self.then(&rhs)
    }
}

impl std::ops::Mul<&Transform3D> for Transform3D {
    type Output = Transform3D;

    fn mul(self, rhs: &Transform3D) -> Self::Output {
        self.then(rhs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::approx_eq;

    #[test]
    fn test_identity() {
        let t = Transform3D::identity();
        let p = Point3::new(1.0, 2.0, 3.0);
        let transformed = t.transform_point(&p);
        assert_eq!(transformed, p);
    }

    #[test]
    fn test_translation() {
        let t = Transform3D::translation(10.0, 20.0, 30.0);
        let p = Point3::origin();
        let transformed = t.transform_point(&p);
        assert_eq!(transformed, Point3::new(10.0, 20.0, 30.0));
    }

    #[test]
    fn test_rotation() {
        let t = Transform3D::rotation_z(std::f64::consts::FRAC_PI_2);
        let p = Point3::new(1.0, 0.0, 0.0);
        let transformed = t.transform_point(&p);
        
        assert!(approx_eq(transformed.x, 0.0));
        assert!(approx_eq(transformed.y, 1.0));
        assert!(approx_eq(transformed.z, 0.0));
    }

    #[test]
    fn test_composition() {
        let t1 = Transform3D::translation(10.0, 0.0, 0.0);
        let t2 = Transform3D::translation(0.0, 20.0, 0.0);
        let combined = t1.then(&t2);

        let p = Point3::origin();
        let transformed = combined.transform_point(&p);
        assert_eq!(transformed, Point3::new(10.0, 20.0, 0.0));
    }

    #[test]
    fn test_inverse() {
        let t = Transform3D::translation(10.0, 20.0, 30.0);
        let inv = t.inverse().unwrap();

        let p = Point3::new(10.0, 20.0, 30.0);
        let transformed = inv.transform_point(&p);
        
        assert!(approx_eq(transformed.x, 0.0));
        assert!(approx_eq(transformed.y, 0.0));
        assert!(approx_eq(transformed.z, 0.0));
    }
}
