//! 圆弧

use crate::math::{BoundingBox2, Point2, EPSILON};
use serde::{Deserialize, Serialize};

/// 圆弧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arc {
    pub center: Point2,
    pub radius: f64,
    /// 起始角度（弧度）
    pub start_angle: f64,
    /// 终止角度（弧度）
    pub end_angle: f64,
}

impl Arc {
    pub fn new(center: Point2, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        Self {
            center,
            radius,
            start_angle,
            end_angle,
        }
    }

    /// 从三点创建圆弧
    pub fn from_three_points(p1: Point2, p2: Point2, p3: Point2) -> Option<Self> {
        // 计算圆心
        let d = 2.0
            * (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y));

        if d.abs() < EPSILON {
            return None; // 三点共线
        }

        let ux = ((p1.x * p1.x + p1.y * p1.y) * (p2.y - p3.y)
            + (p2.x * p2.x + p2.y * p2.y) * (p3.y - p1.y)
            + (p3.x * p3.x + p3.y * p3.y) * (p1.y - p2.y))
            / d;
        let uy = ((p1.x * p1.x + p1.y * p1.y) * (p3.x - p2.x)
            + (p2.x * p2.x + p2.y * p2.y) * (p1.x - p3.x)
            + (p3.x * p3.x + p3.y * p3.y) * (p2.x - p1.x))
            / d;

        let center = Point2::new(ux, uy);
        let radius = (p1 - center).norm();

        let start_angle = (p1.y - center.y).atan2(p1.x - center.x);
        let end_angle = (p3.y - center.y).atan2(p3.x - center.x);

        Some(Self::new(center, radius, start_angle, end_angle))
    }

    /// 计算弧长
    pub fn length(&self) -> f64 {
        self.sweep_angle().abs() * self.radius
    }

    /// 计算扫过的角度
    pub fn sweep_angle(&self) -> f64 {
        let mut sweep = self.end_angle - self.start_angle;
        while sweep < 0.0 {
            sweep += 2.0 * std::f64::consts::PI;
        }
        while sweep > 2.0 * std::f64::consts::PI {
            sweep -= 2.0 * std::f64::consts::PI;
        }
        sweep
    }

    /// 获取起点
    pub fn start_point(&self) -> Point2 {
        Point2::new(
            self.center.x + self.radius * self.start_angle.cos(),
            self.center.y + self.radius * self.start_angle.sin(),
        )
    }

    /// 获取终点
    pub fn end_point(&self) -> Point2 {
        Point2::new(
            self.center.x + self.radius * self.end_angle.cos(),
            self.center.y + self.radius * self.end_angle.sin(),
        )
    }

    /// 计算点到圆弧的距离
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        let angle = (point.y - self.center.y).atan2(point.x - self.center.x);

        // 检查角度是否在弧的范围内
        if self.contains_angle(angle) {
            ((point - self.center).norm() - self.radius).abs()
        } else {
            // 返回到端点的最小距离
            let d1 = (point - self.start_point()).norm();
            let d2 = (point - self.end_point()).norm();
            d1.min(d2)
        }
    }

    /// 检查角度是否在弧的范围内
    pub fn contains_angle(&self, angle: f64) -> bool {
        let mut a = angle;
        let mut start = self.start_angle;
        let mut end = self.end_angle;

        // 归一化到 [0, 2π)
        while a < 0.0 {
            a += 2.0 * std::f64::consts::PI;
        }
        while start < 0.0 {
            start += 2.0 * std::f64::consts::PI;
        }
        while end < 0.0 {
            end += 2.0 * std::f64::consts::PI;
        }

        if start <= end {
            a >= start && a <= end
        } else {
            a >= start || a <= end
        }
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        let mut bbox = BoundingBox2::from_points([self.start_point(), self.end_point()]);

        // 检查象限点
        let pi = std::f64::consts::PI;
        for angle in [0.0, pi / 2.0, pi, 3.0 * pi / 2.0] {
            if self.contains_angle(angle) {
                bbox.expand_to_include(&Point2::new(
                    self.center.x + self.radius * angle.cos(),
                    self.center.y + self.radius * angle.sin(),
                ));
            }
        }

        bbox
    }
}
