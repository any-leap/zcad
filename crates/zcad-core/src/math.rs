//! 数学基础类型
//!
//! 基于 nalgebra 提供的向量和点类型的别名。

use nalgebra as na;
use serde::{Deserialize, Serialize};

/// 2D点类型
pub type Point2 = na::Point2<f64>;

/// 3D点类型
pub type Point3 = na::Point3<f64>;

/// 2D向量类型
pub type Vector2 = na::Vector2<f64>;

/// 3D向量类型
pub type Vector3 = na::Vector3<f64>;

/// 2D变换矩阵
pub type Matrix3 = na::Matrix3<f64>;

/// 3D变换矩阵
pub type Matrix4 = na::Matrix4<f64>;

/// 数值容差，用于几何比较
pub const EPSILON: f64 = 1e-10;

/// 判断两个浮点数是否近似相等
#[inline]
pub fn approx_eq(a: f64, b: f64) -> bool {
    (a - b).abs() < EPSILON
}

/// 判断两个2D点是否近似相等
#[inline]
pub fn points_approx_eq(a: &Point2, b: &Point2) -> bool {
    approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
}

/// 2D包围盒
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BoundingBox2 {
    pub min: Point2,
    pub max: Point2,
}

impl BoundingBox2 {
    /// 创建新的包围盒
    pub fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    /// 创建空的包围盒（无效状态）
    pub fn empty() -> Self {
        Self {
            min: Point2::new(f64::MAX, f64::MAX),
            max: Point2::new(f64::MIN, f64::MIN),
        }
    }

    /// 从点集创建包围盒
    pub fn from_points(points: impl IntoIterator<Item = Point2>) -> Self {
        let mut bbox = Self::empty();
        for p in points {
            bbox.expand_to_include(&p);
        }
        bbox
    }

    /// 扩展包围盒以包含指定点
    pub fn expand_to_include(&mut self, point: &Point2) {
        self.min.x = self.min.x.min(point.x);
        self.min.y = self.min.y.min(point.y);
        self.max.x = self.max.x.max(point.x);
        self.max.y = self.max.y.max(point.y);
    }

    /// 合并两个包围盒
    pub fn union(&self, other: &Self) -> Self {
        Self {
            min: Point2::new(self.min.x.min(other.min.x), self.min.y.min(other.min.y)),
            max: Point2::new(self.max.x.max(other.max.x), self.max.y.max(other.max.y)),
        }
    }

    /// 检查是否与另一个包围盒相交
    pub fn intersects(&self, other: &Self) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// 检查是否包含指定点
    pub fn contains(&self, point: &Point2) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// 获取中心点
    pub fn center(&self) -> Point2 {
        Point2::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }

    /// 获取宽度
    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    /// 获取高度
    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bounding_box() {
        let bbox = BoundingBox2::from_points([
            Point2::new(0.0, 0.0),
            Point2::new(10.0, 5.0),
            Point2::new(-5.0, 8.0),
        ]);

        assert!(approx_eq(bbox.min.x, -5.0));
        assert!(approx_eq(bbox.min.y, 0.0));
        assert!(approx_eq(bbox.max.x, 10.0));
        assert!(approx_eq(bbox.max.y, 8.0));
        assert!(bbox.contains(&Point2::new(0.0, 4.0)));
        assert!(!bbox.contains(&Point2::new(20.0, 4.0)));
    }
}

