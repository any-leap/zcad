//! 标注

use crate::math::{BoundingBox2, Point2, Vector2, EPSILON};
use serde::{Deserialize, Serialize};

use super::{Text, TextAlignment};

/// 标注类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DimensionType {
    /// 对齐标注 (Aligned) - 默认
    #[default]
    Aligned,
    /// 线性标注 (Linear) - 水平或垂直
    Linear,
    /// 半径标注
    Radius,
    /// 直径标注
    Diameter,
    /// 角度标注
    Angular,
    /// 弧长标注
    ArcLength,
    /// 坐标标注
    Ordinate,
}

/// 尺寸标注
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    /// 第一个测量点
    pub definition_point1: Point2,
    /// 第二个测量点
    pub definition_point2: Point2,
    /// 标注线位置点 (决定标注线的高度/距离)
    pub line_location: Point2,
    /// 标注类型
    pub dim_type: DimensionType,
    /// 覆盖文本 (如果为空则显示测量值)
    pub text_override: Option<String>,
    /// 文本高度
    pub text_height: f64,
    /// 文本位置 (如果为None，则自动计算默认位置)
    pub text_position: Option<Point2>,
}

impl Dimension {
    pub fn new(p1: Point2, p2: Point2, location: Point2) -> Self {
        Self {
            definition_point1: p1,
            definition_point2: p2,
            line_location: location,
            dim_type: DimensionType::Aligned,
            text_override: None,
            text_height: 10.0, // 默认高度
            text_position: None,
        }
    }

    /// 获取文本的实际显示位置（如果未设置，则计算默认位置）
    pub fn get_text_position(&self) -> Point2 {
        if let Some(pos) = self.text_position {
            return pos;
        }
        self.default_text_position()
    }

    /// 计算默认文本位置
    pub fn default_text_position(&self) -> Point2 {
        match self.dim_type {
            DimensionType::Aligned | DimensionType::Linear => {
                let dir = (self.definition_point2 - self.definition_point1).normalize();
                let perp = Vector2::new(-dir.y, dir.x);
                let v_loc = self.line_location - self.definition_point1;
                let dist = v_loc.dot(&perp);
                
                // 确保signum不为0，如果为0，默认向上
                let sign = if dist.abs() < EPSILON { 1.0 } else { dist.signum() };
                
                // 默认偏移量：dist + 0.8 * text_height * sign
                let total_dist = dist + sign * (self.text_height * 0.8);
                let offset_vec = perp * total_dist;
                
                self.definition_point1 + (self.definition_point2 - self.definition_point1) * 0.5 + offset_vec
            }
            DimensionType::Radius | DimensionType::Diameter => {
                // 默认位置就是 line_location (用户点击的位置)
                self.line_location
            }
            DimensionType::Angular | DimensionType::ArcLength => {
                // 角度标注：文本位于角平分线上
                let v1 = (self.definition_point2 - self.definition_point1).normalize();
                let v2 = (self.line_location - self.definition_point1).normalize();
                let bisector = (v1 + v2).normalize();
                let radius = (self.definition_point2 - self.definition_point1).norm() * 0.6;
                self.definition_point1 + bisector * radius
            }
            DimensionType::Ordinate => {
                // 坐标标注：文本位于 line_location
                self.line_location
            }
        }
    }

    /// 获取文本包围盒
    pub fn text_bounding_box(&self) -> BoundingBox2 {
        let pos = self.get_text_position();
        let content = self.display_text();
        let text = Text::new(pos, content, self.text_height)
            .with_alignment(TextAlignment::Center); // 标注文本通常是居中绘制
        text.bounding_box()
    }

    /// 获取测量值
    pub fn measurement(&self) -> f64 {
        match self.dim_type {
            DimensionType::Aligned => (self.definition_point2 - self.definition_point1).norm(),
            DimensionType::Linear => {
                // 线性标注通常根据line_location的位置决定是水平还是垂直
                // 这里简化处理：根据两点的主要差异方向决定
                let dx = (self.definition_point2.x - self.definition_point1.x).abs();
                let dy = (self.definition_point2.y - self.definition_point1.y).abs();
                if dx >= dy { dx } else { dy }
            }
            DimensionType::Radius => {
                // 对于半径标注，p1是圆心，p2是圆上一点
                (self.definition_point2 - self.definition_point1).norm()
            }
            DimensionType::Diameter => {
                // 对于直径标注，p1是圆心，p2是圆上一点，测量值为半径 * 2
                (self.definition_point2 - self.definition_point1).norm() * 2.0
            }
            DimensionType::Angular => {
                // 角度标注：p1 是顶点，p2 是第一条边上的点，line_location 是第二条边上的点
                let v1 = self.definition_point2 - self.definition_point1;
                let v2 = self.line_location - self.definition_point1;
                let dot = v1.x * v2.x + v1.y * v2.y;
                let cross = v1.x * v2.y - v1.y * v2.x;
                cross.atan2(dot).abs().to_degrees()
            }
            DimensionType::ArcLength => {
                // 弧长标注：p1 是圆心，p2 是起点，line_location 是终点
                let radius = (self.definition_point2 - self.definition_point1).norm();
                let v1 = self.definition_point2 - self.definition_point1;
                let v2 = self.line_location - self.definition_point1;
                let angle = (v1.x * v2.y - v1.y * v2.x).atan2(v1.x * v2.x + v1.y * v2.y).abs();
                radius * angle
            }
            DimensionType::Ordinate => {
                // 坐标标注：显示 x 或 y 坐标
                // 根据 line_location 相对于 definition_point1 的位置决定
                let dx = (self.line_location.x - self.definition_point1.x).abs();
                let dy = (self.line_location.y - self.definition_point1.y).abs();
                if dx > dy {
                    self.definition_point1.x
                } else {
                    self.definition_point1.y
                }
            }
        }
    }

    /// 获取显示的文本
    pub fn display_text(&self) -> String {
        if let Some(text) = &self.text_override {
            text.clone()
        } else {
            let val = self.measurement();
            match self.dim_type {
                DimensionType::Radius => format!("R{:.2}", val),
                DimensionType::Diameter => format!("%%C{:.2}", val), // %%C 是 CAD 中直径符号的转义
                DimensionType::Angular => format!("{:.1}°", val),
                DimensionType::ArcLength => format!("⌒{:.2}", val),
                DimensionType::Ordinate => format!("{:.2}", val),
                _ => format!("{:.2}", val),
            }
        }
    }

    /// 计算包围盒 (简化估算)
    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::from_points([
            self.definition_point1,
            self.definition_point2,
            self.line_location,
        ])
    }

    /// 检查点是否在标注上 (简化：检查是否在包围盒内)
    pub fn contains_point(&self, point: &Point2, tolerance: f64) -> bool {
        let bbox = self.bounding_box();
        let expanded = BoundingBox2::new(
            Point2::new(bbox.min.x - tolerance, bbox.min.y - tolerance),
            Point2::new(bbox.max.x + tolerance, bbox.max.y + tolerance),
        );
        expanded.contains(point)
    }
}
