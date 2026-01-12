//! 填充

use crate::math::{BoundingBox2, Point2, Vector2};
use serde::{Deserialize, Serialize};

use super::{Arc, Ellipse, Line, Spline};

/// 填充边界类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HatchBoundaryElement {
    /// 线段
    Line(Line),
    /// 圆弧
    Arc(Arc),
    /// 椭圆弧
    Ellipse(Ellipse),
    /// 样条
    Spline(Spline),
}

/// 填充边界
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchBoundary {
    /// 边界元素
    pub elements: Vec<HatchBoundaryElement>,
    /// 是否为外边界（false 表示孔洞）
    pub is_outer: bool,
}

impl HatchBoundary {
    pub fn new(elements: Vec<HatchBoundaryElement>, is_outer: bool) -> Self {
        Self { elements, is_outer }
    }

    /// 获取边界的包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        let mut bbox = BoundingBox2::empty();
        for elem in &self.elements {
            let elem_bbox = match elem {
                HatchBoundaryElement::Line(l) => l.bounding_box(),
                HatchBoundaryElement::Arc(a) => a.bounding_box(),
                HatchBoundaryElement::Ellipse(e) => e.bounding_box(),
                HatchBoundaryElement::Spline(s) => s.bounding_box(),
            };
            bbox = bbox.union(&elem_bbox);
        }
        bbox
    }
}

/// 填充图案类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HatchPatternType {
    /// 实心填充
    Solid,
    /// 预定义图案
    Predefined(String),
    /// 用户自定义图案
    Custom {
        /// 图案线定义
        lines: Vec<HatchPatternLine>,
    },
}

/// 填充图案线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchPatternLine {
    /// 角度（弧度）
    pub angle: f64,
    /// 起点
    pub base_point: Point2,
    /// 偏移（用于平行线）
    pub offset: Vector2,
    /// 虚线模式（正数表示实线，负数表示间隙）
    pub dash_pattern: Vec<f64>,
}

/// 填充
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hatch {
    /// 边界
    pub boundaries: Vec<HatchBoundary>,
    /// 图案类型
    pub pattern_type: HatchPatternType,
    /// 图案角度（弧度）
    pub angle: f64,
    /// 图案比例
    pub scale: f64,
}

impl Hatch {
    /// 创建实心填充
    pub fn solid(boundaries: Vec<HatchBoundary>) -> Self {
        Self {
            boundaries,
            pattern_type: HatchPatternType::Solid,
            angle: 0.0,
            scale: 1.0,
        }
    }

    /// 创建图案填充
    pub fn pattern(boundaries: Vec<HatchBoundary>, pattern_name: &str, angle: f64, scale: f64) -> Self {
        Self {
            boundaries,
            pattern_type: HatchPatternType::Predefined(pattern_name.to_string()),
            angle,
            scale,
        }
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        let mut bbox = BoundingBox2::empty();
        for boundary in &self.boundaries {
            bbox = bbox.union(&boundary.bounding_box());
        }
        bbox
    }

    /// 检查点是否在填充区域内
    pub fn contains_point(&self, _point: &Point2, _tolerance: f64) -> bool {
        // TODO: 实现点在多边形内的判断（射线法）
        false
    }
}
