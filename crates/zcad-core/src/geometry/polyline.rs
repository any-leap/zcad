//! 多段线

use crate::math::{BoundingBox2, Point2, Vector2, EPSILON};
use serde::{Deserialize, Serialize};

use super::{Arc, Geometry, Line};

/// 多段线顶点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolylineVertex {
    pub point: Point2,
    /// 凸度（bulge）- 用于弧线段，0表示直线
    pub bulge: f64,
}

impl PolylineVertex {
    pub fn new(point: Point2) -> Self {
        Self { point, bulge: 0.0 }
    }

    pub fn with_bulge(point: Point2, bulge: f64) -> Self {
        Self { point, bulge }
    }
}

/// 多段线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polyline {
    pub vertices: Vec<PolylineVertex>,
    /// 是否闭合
    pub closed: bool,
}

impl Polyline {
    pub fn new(vertices: Vec<PolylineVertex>, closed: bool) -> Self {
        Self { vertices, closed }
    }

    /// 从点列表创建（所有顶点都是直线连接）
    pub fn from_points(points: impl IntoIterator<Item = Point2>, closed: bool) -> Self {
        Self {
            vertices: points
                .into_iter()
                .map(PolylineVertex::new)
                .collect(),
            closed,
        }
    }

    /// 顶点数量
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 线段数量
    pub fn segment_count(&self) -> usize {
        if self.vertices.len() < 2 {
            return 0;
        }
        if self.closed {
            self.vertices.len()
        } else {
            self.vertices.len() - 1
        }
    }

    /// 计算总长度
    pub fn length(&self) -> f64 {
        if self.vertices.len() < 2 {
            return 0.0;
        }

        let mut total = 0.0;
        for i in 0..self.segment_count() {
            let v1 = &self.vertices[i];
            let v2 = &self.vertices[(i + 1) % self.vertices.len()];

            if v1.bulge.abs() < EPSILON {
                // 直线段
                total += (v2.point - v1.point).norm();
            } else {
                // 弧线段
                total += self.arc_segment_length(v1, v2);
            }
        }
        total
    }

    /// 计算弧线段长度
    fn arc_segment_length(&self, v1: &PolylineVertex, v2: &PolylineVertex) -> f64 {
        let chord = (v2.point - v1.point).norm();
        let s = chord / 2.0;
        let bulge = v1.bulge.abs();
        let radius = s * (1.0 + bulge * bulge) / (2.0 * bulge);
        let angle = 4.0 * bulge.atan();
        radius * angle.abs()
    }

    /// 计算点到多段线的距离
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        if self.vertices.is_empty() {
            return f64::MAX;
        }
        if self.vertices.len() == 1 {
            return (point - self.vertices[0].point).norm();
        }

        let mut min_dist = f64::MAX;
        for i in 0..self.segment_count() {
            let v1 = &self.vertices[i];
            let v2 = &self.vertices[(i + 1) % self.vertices.len()];

            let dist = if v1.bulge.abs() < EPSILON {
                // 直线段
                let line = Line::new(v1.point, v2.point);
                line.distance_to_point(point)
            } else {
                // 弧线段 - 简化处理，使用直线近似
                let line = Line::new(v1.point, v2.point);
                line.distance_to_point(point)
            };

            min_dist = min_dist.min(dist);
        }
        min_dist
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        if self.vertices.is_empty() {
            return BoundingBox2::empty();
        }
        BoundingBox2::from_points(self.vertices.iter().map(|v| v.point))
    }

    /// 爆炸为独立的线段/圆弧
    ///
    /// 这是我们要做好的功能 - 智能爆炸，只生成需要的几何体
    pub fn explode(&self) -> Vec<Geometry> {
        if self.vertices.len() < 2 {
            return vec![];
        }

        let mut result = Vec::with_capacity(self.segment_count());

        for i in 0..self.segment_count() {
            let v1 = &self.vertices[i];
            let v2 = &self.vertices[(i + 1) % self.vertices.len()];

            if v1.bulge.abs() < EPSILON {
                // 直线段
                result.push(Geometry::Line(Line::new(v1.point, v2.point)));
            } else {
                // 弧线段
                if let Some(arc) = self.vertex_pair_to_arc(v1, v2) {
                    result.push(Geometry::Arc(arc));
                } else {
                    // 回退到直线
                    result.push(Geometry::Line(Line::new(v1.point, v2.point)));
                }
            }
        }

        result
    }

    /// 将顶点对转换为圆弧
    fn vertex_pair_to_arc(&self, v1: &PolylineVertex, v2: &PolylineVertex) -> Option<Arc> {
        let chord = v2.point - v1.point;
        let chord_len = chord.norm();

        if chord_len < EPSILON {
            return None;
        }

        let bulge = v1.bulge;
        let s = chord_len / 2.0;
        let h = s * bulge; // 弧高

        // 计算圆心
        let mid = Point2::new(
            (v1.point.x + v2.point.x) / 2.0,
            (v1.point.y + v2.point.y) / 2.0,
        );

        let radius = (s * s + h * h) / (2.0 * h.abs());
        let d = radius - h.abs(); // 圆心到弦的距离

        // 弦的垂直方向
        let perp = if bulge > 0.0 {
            Vector2::new(-chord.y, chord.x).normalize()
        } else {
            Vector2::new(chord.y, -chord.x).normalize()
        };

        let center = mid + perp * d;

        let start_angle = (v1.point.y - center.y).atan2(v1.point.x - center.x);
        let end_angle = (v2.point.y - center.y).atan2(v2.point.x - center.x);

        Some(Arc::new(center, radius, start_angle, end_angle))
    }
}
