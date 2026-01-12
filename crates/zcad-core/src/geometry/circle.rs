//! 圆

use crate::math::{BoundingBox2, Point2};
use serde::{Deserialize, Serialize};

/// 圆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point2,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point2, radius: f64) -> Self {
        Self { center, radius }
    }

    /// 计算周长
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    /// 计算面积
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    /// 计算点到圆的距离（负值表示在圆内）
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        (point - self.center).norm() - self.radius
    }

    /// 获取圆上指定角度的点
    pub fn point_at_angle(&self, angle: f64) -> Point2 {
        Point2::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
        )
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::new(
            Point2::new(self.center.x - self.radius, self.center.y - self.radius),
            Point2::new(self.center.x + self.radius, self.center.y + self.radius),
        )
    }
}
