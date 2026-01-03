//! 2D变换操作
//!
//! 支持平移、旋转、缩放、镜像等变换。

use crate::math::{Matrix3, Point2, Vector2};
use serde::{Deserialize, Serialize};

/// 2D仿射变换
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Transform2D {
    matrix: Matrix3,
}

impl Transform2D {
    /// 创建单位变换
    pub fn identity() -> Self {
        Self {
            matrix: Matrix3::identity(),
        }
    }

    /// 创建平移变换
    pub fn translation(dx: f64, dy: f64) -> Self {
        Self {
            matrix: Matrix3::new(
                1.0, 0.0, dx,
                0.0, 1.0, dy,
                0.0, 0.0, 1.0,
            ),
        }
    }

    /// 创建旋转变换（绕原点）
    pub fn rotation(angle: f64) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            matrix: Matrix3::new(
                cos, -sin, 0.0,
                sin, cos, 0.0,
                0.0, 0.0, 1.0,
            ),
        }
    }

    /// 创建绕指定点的旋转变换
    pub fn rotation_around(center: Point2, angle: f64) -> Self {
        Self::translation(center.x, center.y)
            .then(&Self::rotation(angle))
            .then(&Self::translation(-center.x, -center.y))
    }

    /// 创建缩放变换（绕原点）
    pub fn scale(sx: f64, sy: f64) -> Self {
        Self {
            matrix: Matrix3::new(
                sx, 0.0, 0.0,
                0.0, sy, 0.0,
                0.0, 0.0, 1.0,
            ),
        }
    }

    /// 创建均匀缩放变换
    pub fn uniform_scale(s: f64) -> Self {
        Self::scale(s, s)
    }

    /// 创建绕指定点的缩放变换
    pub fn scale_around(center: Point2, sx: f64, sy: f64) -> Self {
        Self::translation(center.x, center.y)
            .then(&Self::scale(sx, sy))
            .then(&Self::translation(-center.x, -center.y))
    }

    /// 创建镜像变换（相对于X轴）
    pub fn mirror_x() -> Self {
        Self::scale(1.0, -1.0)
    }

    /// 创建镜像变换（相对于Y轴）
    pub fn mirror_y() -> Self {
        Self::scale(-1.0, 1.0)
    }

    /// 创建相对于任意直线的镜像变换
    pub fn mirror_line(p1: Point2, p2: Point2) -> Self {
        let dx = p2.x - p1.x;
        let dy = p2.y - p1.y;
        let angle = dy.atan2(dx);

        // 平移到原点 -> 旋转使直线与X轴对齐 -> 镜像 -> 反向旋转 -> 反向平移
        Self::translation(p1.x, p1.y)
            .then(&Self::rotation(angle))
            .then(&Self::mirror_x())
            .then(&Self::rotation(-angle))
            .then(&Self::translation(-p1.x, -p1.y))
    }

    /// 组合两个变换（self 在后，other 在前）
    pub fn then(&self, other: &Transform2D) -> Self {
        Self {
            matrix: self.matrix * other.matrix,
        }
    }

    /// 变换一个点
    pub fn transform_point(&self, point: &Point2) -> Point2 {
        let v = self.matrix * nalgebra::Vector3::new(point.x, point.y, 1.0);
        Point2::new(v.x, v.y)
    }

    /// 变换一个向量（不受平移影响）
    pub fn transform_vector(&self, vector: &Vector2) -> Vector2 {
        let v = self.matrix * nalgebra::Vector3::new(vector.x, vector.y, 0.0);
        Vector2::new(v.x, v.y)
    }

    /// 获取逆变换
    pub fn inverse(&self) -> Option<Self> {
        self.matrix.try_inverse().map(|m| Self { matrix: m })
    }

    /// 获取变换矩阵
    pub fn matrix(&self) -> &Matrix3 {
        &self.matrix
    }

    /// 从矩阵创建变换
    pub fn from_matrix(matrix: Matrix3) -> Self {
        Self { matrix }
    }

    /// 提取平移分量
    pub fn translation_component(&self) -> Vector2 {
        Vector2::new(self.matrix[(0, 2)], self.matrix[(1, 2)])
    }

    /// 提取旋转角度
    pub fn rotation_angle(&self) -> f64 {
        self.matrix[(1, 0)].atan2(self.matrix[(0, 0)])
    }

    /// 提取缩放分量
    pub fn scale_component(&self) -> (f64, f64) {
        let sx = (self.matrix[(0, 0)].powi(2) + self.matrix[(1, 0)].powi(2)).sqrt();
        let sy = (self.matrix[(0, 1)].powi(2) + self.matrix[(1, 1)].powi(2)).sqrt();
        (sx, sy)
    }
}

impl Default for Transform2D {
    fn default() -> Self {
        Self::identity()
    }
}

impl std::ops::Mul for Transform2D {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            matrix: self.matrix * rhs.matrix,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::approx_eq;

    #[test]
    fn test_translation() {
        let t = Transform2D::translation(10.0, 20.0);
        let p = Point2::new(5.0, 5.0);
        let result = t.transform_point(&p);

        assert!(approx_eq(result.x, 15.0));
        assert!(approx_eq(result.y, 25.0));
    }

    #[test]
    fn test_rotation() {
        let t = Transform2D::rotation(std::f64::consts::PI / 2.0);
        let p = Point2::new(1.0, 0.0);
        let result = t.transform_point(&p);

        assert!(approx_eq(result.x, 0.0));
        assert!(approx_eq(result.y, 1.0));
    }

    #[test]
    fn test_scale() {
        let t = Transform2D::scale(2.0, 3.0);
        let p = Point2::new(5.0, 10.0);
        let result = t.transform_point(&p);

        assert!(approx_eq(result.x, 10.0));
        assert!(approx_eq(result.y, 30.0));
    }

    #[test]
    fn test_inverse() {
        let t = Transform2D::translation(10.0, 20.0)
            .then(&Transform2D::rotation(0.5))
            .then(&Transform2D::scale(2.0, 3.0));

        let inv = t.inverse().unwrap();
        let p = Point2::new(100.0, 200.0);

        let transformed = t.transform_point(&p);
        let restored = inv.transform_point(&transformed);

        assert!(approx_eq(restored.x, p.x));
        assert!(approx_eq(restored.y, p.y));
    }
}

