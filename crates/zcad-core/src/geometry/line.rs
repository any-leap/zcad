//! 线段

use crate::math::{BoundingBox2, Point2, Vector2};
use serde::{Deserialize, Serialize};

/// 线段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub start: Point2,
    pub end: Point2,
}

impl Line {
    pub fn new(start: Point2, end: Point2) -> Self {
        Self { start, end }
    }

    /// 计算线段长度
    pub fn length(&self) -> f64 {
        (self.end - self.start).norm()
    }

    /// 计算线段方向向量（单位向量）
    pub fn direction(&self) -> Vector2 {
        (self.end - self.start).normalize()
    }

    /// 计算线段中点
    pub fn midpoint(&self) -> Point2 {
        Point2::new(
            (self.start.x + self.end.x) / 2.0,
            (self.start.y + self.end.y) / 2.0,
        )
    }

    /// 计算点到线段的距离
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        let v = self.end - self.start;
        let w = point - self.start;

        let c1 = w.dot(&v);
        if c1 <= 0.0 {
            return (point - self.start).norm();
        }

        let c2 = v.dot(&v);
        if c2 <= c1 {
            return (point - self.end).norm();
        }

        let b = c1 / c2;
        let pb = self.start + v * b;
        (point - pb).norm()
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::from_points([self.start, self.end])
    }
}
