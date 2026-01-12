//! 引线

use crate::math::{BoundingBox2, Point2, Vector2};
use serde::{Deserialize, Serialize};

use super::Line;

/// 箭头类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArrowType {
    /// 无箭头
    None,
    /// 闭合填充箭头（默认）
    #[default]
    ClosedFilled,
    /// 闭合空心箭头
    ClosedBlank,
    /// 开口箭头
    Open,
    /// 点
    Dot,
    /// 圆
    Circle,
}

/// 引线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leader {
    /// 顶点列表（从箭头到文本）
    pub vertices: Vec<Point2>,
    /// 箭头类型
    pub arrow_type: ArrowType,
    /// 箭头大小
    pub arrow_size: f64,
    /// 关联的文本
    pub text: Option<String>,
    /// 文本高度
    pub text_height: f64,
}

impl Leader {
    /// 创建新的引线
    pub fn new(vertices: Vec<Point2>) -> Self {
        Self {
            vertices,
            arrow_type: ArrowType::ClosedFilled,
            arrow_size: 3.0,
            text: None,
            text_height: 2.5,
        }
    }

    /// 设置箭头类型
    pub fn with_arrow(mut self, arrow_type: ArrowType, size: f64) -> Self {
        self.arrow_type = arrow_type;
        self.arrow_size = size;
        self
    }

    /// 设置文本
    pub fn with_text(mut self, text: impl Into<String>, height: f64) -> Self {
        self.text = Some(text.into());
        self.text_height = height;
        self
    }

    /// 获取箭头位置（第一个顶点）
    pub fn arrow_point(&self) -> Option<Point2> {
        self.vertices.first().copied()
    }

    /// 获取箭头方向
    pub fn arrow_direction(&self) -> Option<Vector2> {
        if self.vertices.len() >= 2 {
            Some((self.vertices[0] - self.vertices[1]).normalize())
        } else {
            None
        }
    }

    /// 获取文本位置（最后一个顶点）
    pub fn text_position(&self) -> Option<Point2> {
        self.vertices.last().copied()
    }

    /// 计算总长度
    pub fn length(&self) -> f64 {
        let mut total = 0.0;
        for i in 0..self.vertices.len().saturating_sub(1) {
            total += (self.vertices[i + 1] - self.vertices[i]).norm();
        }
        total
    }

    /// 计算点到引线的距离
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        let mut min_dist = f64::MAX;
        for i in 0..self.vertices.len().saturating_sub(1) {
            let line = Line::new(self.vertices[i], self.vertices[i + 1]);
            min_dist = min_dist.min(line.distance_to_point(point));
        }
        min_dist
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        if self.vertices.is_empty() {
            return BoundingBox2::empty();
        }
        BoundingBox2::from_points(self.vertices.iter().copied())
    }
}
