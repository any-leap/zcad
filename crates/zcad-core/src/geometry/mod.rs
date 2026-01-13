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
//! - 表格 (Table)

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
mod table;

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
pub use table::{Table, TableCell, TableStyle, CellAlignment};

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
    Table(Table),
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
            Geometry::Table(t) => t.bounding_box(),
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
            Geometry::Table(_) => "Table",
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
            Geometry::Table(t) => t.contains_point(point, tolerance),
        }
    }

    /// 平移几何体
    pub fn translate(&mut self, offset: crate::math::Vector2) {
        match self {
            Geometry::Point(p) => p.position += offset,
            Geometry::Line(l) => {
                l.start += offset;
                l.end += offset;
            }
            Geometry::Circle(c) => c.center += offset,
            Geometry::Arc(a) => a.center += offset,
            Geometry::Polyline(pl) => {
                for v in &mut pl.vertices {
                    v.point += offset;
                }
            }
            Geometry::Text(t) => t.position += offset,
            Geometry::Dimension(d) => {
                d.definition_point1 += offset;
                d.definition_point2 += offset;
                d.line_location += offset;
            }
            Geometry::Ellipse(e) => e.center += offset,
            Geometry::Spline(s) => {
                for p in &mut s.control_points {
                    *p += offset;
                }
                for p in &mut s.fit_points {
                    *p += offset;
                }
            }
            Geometry::Hatch(h) => {
                for boundary in &mut h.boundaries {
                    for elem in &mut boundary.elements {
                        match elem {
                            HatchBoundaryElement::Line(line) => {
                                line.start += offset;
                                line.end += offset;
                            }
                            HatchBoundaryElement::Arc(arc) => {
                                arc.center += offset;
                            }
                            HatchBoundaryElement::Ellipse(ellipse) => {
                                ellipse.center += offset;
                            }
                            HatchBoundaryElement::Spline(spline) => {
                                for p in &mut spline.control_points {
                                    *p += offset;
                                }
                                for p in &mut spline.fit_points {
                                    *p += offset;
                                }
                            }
                        }
                    }
                }
            }
            Geometry::Leader(l) => {
                for p in &mut l.vertices {
                    *p += offset;
                }
            }
            Geometry::Table(t) => t.position += offset,
        }
    }

    /// 缩放几何体（相对于原点）
    pub fn scale(&mut self, factor: f64) {
        match self {
            Geometry::Point(p) => {
                p.position.x *= factor;
                p.position.y *= factor;
            }
            Geometry::Line(l) => {
                l.start.x *= factor;
                l.start.y *= factor;
                l.end.x *= factor;
                l.end.y *= factor;
            }
            Geometry::Circle(c) => {
                c.center.x *= factor;
                c.center.y *= factor;
                c.radius *= factor;
            }
            Geometry::Arc(a) => {
                a.center.x *= factor;
                a.center.y *= factor;
                a.radius *= factor;
            }
            Geometry::Polyline(pl) => {
                for v in &mut pl.vertices {
                    v.point.x *= factor;
                    v.point.y *= factor;
                }
            }
            Geometry::Text(t) => {
                t.position.x *= factor;
                t.position.y *= factor;
                t.height *= factor;
            }
            Geometry::Dimension(d) => {
                d.definition_point1.x *= factor;
                d.definition_point1.y *= factor;
                d.definition_point2.x *= factor;
                d.definition_point2.y *= factor;
                d.line_location.x *= factor;
                d.line_location.y *= factor;
                d.text_height *= factor;
            }
            Geometry::Ellipse(e) => {
                e.center.x *= factor;
                e.center.y *= factor;
                e.major_axis.x *= factor;
                e.major_axis.y *= factor;
                // ratio 不变，因为长短轴同比例缩放
            }
            Geometry::Spline(s) => {
                for p in &mut s.control_points {
                    p.x *= factor;
                    p.y *= factor;
                }
                for p in &mut s.fit_points {
                    p.x *= factor;
                    p.y *= factor;
                }
            }
            Geometry::Hatch(h) => {
                for boundary in &mut h.boundaries {
                    for elem in &mut boundary.elements {
                        match elem {
                            HatchBoundaryElement::Line(line) => {
                                line.start.x *= factor;
                                line.start.y *= factor;
                                line.end.x *= factor;
                                line.end.y *= factor;
                            }
                            HatchBoundaryElement::Arc(arc) => {
                                arc.center.x *= factor;
                                arc.center.y *= factor;
                                arc.radius *= factor;
                            }
                            HatchBoundaryElement::Ellipse(ellipse) => {
                                ellipse.center.x *= factor;
                                ellipse.center.y *= factor;
                                ellipse.major_axis.x *= factor;
                                ellipse.major_axis.y *= factor;
                            }
                            HatchBoundaryElement::Spline(spline) => {
                                for p in &mut spline.control_points {
                                    p.x *= factor;
                                    p.y *= factor;
                                }
                                for p in &mut spline.fit_points {
                                    p.x *= factor;
                                    p.y *= factor;
                                }
                            }
                        }
                    }
                }
            }
            Geometry::Leader(l) => {
                for p in &mut l.vertices {
                    p.x *= factor;
                    p.y *= factor;
                }
                l.arrow_size *= factor;
                l.text_height *= factor;
            }
            Geometry::Table(t) => {
                t.position.x *= factor;
                t.position.y *= factor;
                t.style.row_height *= factor;
                t.style.column_width *= factor;
                t.style.text_height *= factor;
                for w in &mut t.column_widths {
                    *w *= factor;
                }
                for h in &mut t.row_heights {
                    *h *= factor;
                }
            }
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
