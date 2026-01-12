//! 交点计算
//!
//! 几何体之间的交点计算逻辑

use crate::geometry::{Arc, Circle, Geometry, Line, Polyline};
use crate::math::{Point2, Vector2, EPSILON};

/// 交点查找器
pub struct IntersectionFinder;

impl IntersectionFinder {
    /// 计算两个几何体的交点
    pub fn find_intersections(geom1: &Geometry, geom2: &Geometry) -> Vec<Point2> {
        match (geom1, geom2) {
            (Geometry::Line(l1), Geometry::Line(l2)) => {
                Self::line_line_intersection(l1, l2).into_iter().collect()
            }
            (Geometry::Line(line), Geometry::Circle(circle)) 
            | (Geometry::Circle(circle), Geometry::Line(line)) => {
                Self::line_circle_intersection(line, circle)
            }
            (Geometry::Circle(c1), Geometry::Circle(c2)) => {
                Self::circle_circle_intersection(c1, c2)
            }
            (Geometry::Line(line), Geometry::Arc(arc))
            | (Geometry::Arc(arc), Geometry::Line(line)) => {
                Self::line_arc_intersection(line, arc)
            }
            (Geometry::Line(line), Geometry::Polyline(poly))
            | (Geometry::Polyline(poly), Geometry::Line(line)) => {
                Self::line_polyline_intersection(line, poly)
            }
            _ => vec![],
        }
    }

    /// 线段-线段交点
    pub fn line_line_intersection(l1: &Line, l2: &Line) -> Option<Point2> {
        let d1 = l1.end - l1.start;
        let d2 = l2.end - l2.start;

        let cross = d1.x * d2.y - d1.y * d2.x;
        
        // 平行
        if cross.abs() < EPSILON {
            return None;
        }

        let d = l2.start - l1.start;
        let t1 = (d.x * d2.y - d.y * d2.x) / cross;
        let t2 = (d.x * d1.y - d.y * d1.x) / cross;

        // 检查交点是否在两条线段上
        if t1 >= 0.0 && t1 <= 1.0 && t2 >= 0.0 && t2 <= 1.0 {
            Some(l1.start + d1 * t1)
        } else {
            None
        }
    }

    /// 线段-圆交点
    pub fn line_circle_intersection(line: &Line, circle: &Circle) -> Vec<Point2> {
        let d = line.end - line.start;
        let f = line.start - circle.center;

        let a = d.dot(&d);
        let b = 2.0 * f.dot(&d);
        let c = f.dot(&f) - circle.radius * circle.radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return vec![];
        }

        let mut intersections = Vec::new();

        if discriminant.abs() < EPSILON {
            // 一个交点（相切）
            let t = -b / (2.0 * a);
            if t >= 0.0 && t <= 1.0 {
                intersections.push(line.start + d * t);
            }
        } else {
            // 两个交点
            let sqrt_disc = discriminant.sqrt();
            let t1 = (-b - sqrt_disc) / (2.0 * a);
            let t2 = (-b + sqrt_disc) / (2.0 * a);

            if t1 >= 0.0 && t1 <= 1.0 {
                intersections.push(line.start + d * t1);
            }
            if t2 >= 0.0 && t2 <= 1.0 {
                intersections.push(line.start + d * t2);
            }
        }

        intersections
    }

    /// 圆-圆交点
    pub fn circle_circle_intersection(c1: &Circle, c2: &Circle) -> Vec<Point2> {
        let d = (c2.center - c1.center).norm();

        // 不相交情况
        if d > c1.radius + c2.radius || d < (c1.radius - c2.radius).abs() || d < EPSILON {
            return vec![];
        }

        let a = (c1.radius * c1.radius - c2.radius * c2.radius + d * d) / (2.0 * d);
        let h = (c1.radius * c1.radius - a * a).sqrt();

        let p = c1.center + (c2.center - c1.center) * (a / d);

        let dir = (c2.center - c1.center) / d;
        let perp = Vector2::new(-dir.y, dir.x);

        if h < EPSILON {
            // 一个交点（相切）
            vec![p]
        } else {
            // 两个交点
            vec![
                p + perp * h,
                p - perp * h,
            ]
        }
    }

    /// 线段-圆弧交点
    pub fn line_arc_intersection(line: &Line, arc: &Arc) -> Vec<Point2> {
        // 先求线段-完整圆的交点，再过滤在弧范围内的
        let circle = Circle::new(arc.center, arc.radius);
        let circle_intersections = Self::line_circle_intersection(line, &circle);

        circle_intersections
            .into_iter()
            .filter(|p| arc_contains_point(arc, p))
            .collect()
    }

    /// 线段-多段线交点
    pub fn line_polyline_intersection(line: &Line, polyline: &Polyline) -> Vec<Point2> {
        let mut intersections = Vec::new();

        for i in 0..polyline.segment_count() {
            let v1 = &polyline.vertices[i];
            let v2 = &polyline.vertices[(i + 1) % polyline.vertices.len()];

            if v1.bulge.abs() < EPSILON {
                // 直线段
                let seg = Line::new(v1.point, v2.point);
                if let Some(p) = Self::line_line_intersection(line, &seg) {
                    intersections.push(p);
                }
            }
            // TODO: 处理弧线段
        }

        intersections
    }
}

/// 检查点是否在弧上（角度范围内）
fn arc_contains_point(arc: &Arc, point: &Point2) -> bool {
    let angle = (point.y - arc.center.y).atan2(point.x - arc.center.x);
    arc.contains_angle(angle)
}
