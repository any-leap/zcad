//! 几何图元定义
//!
//! 支持的基本图元：
//! - 点 (Point)
//! - 线段 (Line)
//! - 圆 (Circle)
//! - 圆弧 (Arc)
//! - 多段线 (Polyline)
//! - 文本 (Text)
//! - 椭圆 (Ellipse)
//! - 样条曲线 (Spline)
//! - 填充 (Hatch)
//! - 引线 (Leader)
//! - 标注 (Dimension)

mod point;
mod line;
mod circle;
mod arc;
mod polyline;
mod text;
mod dimension;
mod ellipse;
mod spline;
mod hatch;
mod leader;

pub use point::Point;
pub use line::Line;
pub use circle::Circle;
pub use arc::Arc;
pub use polyline::{Polyline, PolylineVertex};
pub use text::{Text, TextAlignment};
pub use dimension::{Dimension, DimensionType};
pub use ellipse::Ellipse;
pub use spline::{Spline, SplineType};
pub use hatch::{Hatch, HatchBoundary, HatchBoundaryElement, HatchPatternType, HatchPatternLine};
pub use leader::{Leader, ArrowType};

use crate::math::{BoundingBox2, Point2};
use serde::{Deserialize, Serialize};

/// 几何类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Geometry {
    Point(Point),
    Line(Line),
    Circle(Circle),
    Arc(Arc),
    Polyline(Polyline),
    Text(Text),
    Dimension(Dimension),
    Ellipse(Ellipse),
    Spline(Spline),
    Hatch(Hatch),
    Leader(Leader),
}

impl Geometry {
    /// 获取几何的包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        match self {
            Geometry::Point(p) => p.bounding_box(),
            Geometry::Line(l) => l.bounding_box(),
            Geometry::Circle(c) => c.bounding_box(),
            Geometry::Arc(a) => a.bounding_box(),
            Geometry::Polyline(pl) => pl.bounding_box(),
            Geometry::Text(t) => t.bounding_box(),
            Geometry::Dimension(d) => d.bounding_box(),
            Geometry::Ellipse(e) => e.bounding_box(),
            Geometry::Spline(s) => s.bounding_box(),
            Geometry::Hatch(h) => h.bounding_box(),
            Geometry::Leader(l) => l.bounding_box(),
        }
    }

    /// 获取几何的类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            Geometry::Point(_) => "Point",
            Geometry::Line(_) => "Line",
            Geometry::Circle(_) => "Circle",
            Geometry::Arc(_) => "Arc",
            Geometry::Polyline(_) => "Polyline",
            Geometry::Text(_) => "Text",
            Geometry::Dimension(_) => "Dimension",
            Geometry::Ellipse(_) => "Ellipse",
            Geometry::Spline(_) => "Spline",
            Geometry::Hatch(_) => "Hatch",
            Geometry::Leader(_) => "Leader",
        }
    }

    /// 检查点是否在几何上（考虑容差）
    pub fn contains_point(&self, point: &Point2, tolerance: f64) -> bool {
        match self {
            Geometry::Point(p) => (p.position - point).norm() <= tolerance,
            Geometry::Line(l) => l.distance_to_point(point) <= tolerance,
            Geometry::Circle(c) => (c.distance_to_point(point)).abs() <= tolerance,
            Geometry::Arc(a) => a.distance_to_point(point) <= tolerance,
            Geometry::Polyline(pl) => pl.distance_to_point(point) <= tolerance,
            Geometry::Text(t) => t.contains_point(point, tolerance),
            Geometry::Dimension(d) => d.contains_point(point, tolerance),
            Geometry::Ellipse(e) => e.distance_to_point(point) <= tolerance,
            Geometry::Spline(s) => s.distance_to_point(point) <= tolerance,
            Geometry::Hatch(h) => h.contains_point(point, tolerance),
            Geometry::Leader(l) => l.distance_to_point(point) <= tolerance,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::EPSILON;

    #[test]
    fn test_line_length() {
        let line = Line::new(Point2::new(0.0, 0.0), Point2::new(3.0, 4.0));
        assert!((line.length() - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_circle_area() {
        let circle = Circle::new(Point2::origin(), 1.0);
        assert!((circle.area() - std::f64::consts::PI).abs() < EPSILON);
    }

    #[test]
    fn test_polyline_explode() {
        let pl = Polyline::from_points(
            [
                Point2::new(0.0, 0.0),
                Point2::new(10.0, 0.0),
                Point2::new(10.0, 10.0),
            ],
            false,
        );

        let exploded = pl.explode();
        assert_eq!(exploded.len(), 2);
        assert!(matches!(exploded[0], Geometry::Line(_)));
        assert!(matches!(exploded[1], Geometry::Line(_)));
    }
}
