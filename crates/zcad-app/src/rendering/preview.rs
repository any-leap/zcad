//! 预览绘制

use zcad_core::geometry::{Arc, Circle, Dimension, DimensionType, Geometry, Line, Polyline, Text};
use zcad_core::math::Point2;
use zcad_core::properties::Color;
use zcad_ui::state::{DrawingTool, EditState};

use super::RenderContext;
use super::geometry::draw_geometry;
use super::dimension::draw_dimension;

/// 绘制预览
pub fn draw_preview(
    ctx: &RenderContext,
    edit_state: &EditState,
    effective_point: Point2,
) {
    if let EditState::Drawing { tool, points, .. } = edit_state {
        if points.is_empty() {
            return;
        }
        
        let preview_color = Color::from_hex(0xFF00FF);
        let mouse_pos = effective_point;

        match tool {
            DrawingTool::Line => {
                let line = Line::new(*points.last().unwrap(), mouse_pos);
                draw_geometry(ctx, &Geometry::Line(line), preview_color);
            }
            DrawingTool::Circle => {
                let radius = (mouse_pos - points[0]).norm();
                if radius > 0.01 {
                    let circle = Circle::new(points[0], radius);
                    draw_geometry(ctx, &Geometry::Circle(circle), preview_color);
                }
            }
            DrawingTool::Rectangle => {
                let p1 = points[0];
                let rect_geom = Polyline::from_points(
                    [
                        Point2::new(p1.x, p1.y),
                        Point2::new(mouse_pos.x, p1.y),
                        Point2::new(mouse_pos.x, mouse_pos.y),
                        Point2::new(p1.x, mouse_pos.y),
                    ],
                    true,
                );
                draw_geometry(ctx, &Geometry::Polyline(rect_geom), preview_color);
            }
            DrawingTool::Arc => {
                if points.len() == 1 {
                    // 只有起点，画到鼠标的直线预览
                    let line = Line::new(points[0], mouse_pos);
                    draw_geometry(ctx, &Geometry::Line(line), preview_color);
                } else if points.len() == 2 {
                    // 有两个点，尝试预览圆弧
                    if let Some(arc) = Arc::from_three_points(points[0], points[1], mouse_pos) {
                        draw_geometry(ctx, &Geometry::Arc(arc), preview_color);
                    } else {
                        // 共线，画两条线
                        let line1 = Line::new(points[0], points[1]);
                        let line2 = Line::new(points[1], mouse_pos);
                        draw_geometry(ctx, &Geometry::Line(line1), preview_color);
                        draw_geometry(ctx, &Geometry::Line(line2), preview_color);
                    }
                }
            }
            DrawingTool::Polyline => {
                // 绘制已有的线段
                for i in 0..points.len().saturating_sub(1) {
                    let line = Line::new(points[i], points[i + 1]);
                    draw_geometry(ctx, &Geometry::Line(line), preview_color);
                }
                // 绘制到鼠标的预览线段
                if let Some(&last) = points.last() {
                    let line = Line::new(last, mouse_pos);
                    draw_geometry(ctx, &Geometry::Line(line), preview_color);
                }
            }
            DrawingTool::Dimension => {
                // 标注预览
                if points.len() == 1 {
                    // 只有一个点，显示到鼠标的直线
                    let line = Line::new(points[0], mouse_pos);
                    draw_geometry(ctx, &Geometry::Line(line), preview_color);
                } else if points.len() == 2 {
                    // 有两个点，显示标注预览
                    let dim = Dimension::new(points[0], points[1], mouse_pos);
                    draw_dimension(ctx, &dim, preview_color);
                }
            }
            DrawingTool::DimensionRadius => {
                if points.len() == 2 {
                    // points[0] = center, points[1] = point on circle
                    let mut dim = Dimension::new(points[0], points[1], mouse_pos);
                    dim.dim_type = DimensionType::Radius;
                    draw_dimension(ctx, &dim, preview_color);
                }
            }
            DrawingTool::DimensionDiameter => {
                if points.len() == 2 {
                    // points[0] = center, points[1] = point representing radius
                    let center = points[0];
                    let radius = (points[1] - center).norm();
                    let text_pos = mouse_pos;
                    
                    let dir = if (text_pos - center).norm() > 0.001 {
                        (text_pos - center).normalize()
                    } else {
                        zcad_core::math::Vector2::x()
                    };
                    let p1 = center - dir * radius;
                    let p2 = center + dir * radius;
                    
                    let mut dim = Dimension::new(p1, p2, text_pos);
                    dim.dim_type = DimensionType::Diameter;
                    draw_dimension(ctx, &dim, preview_color);
                }
            }
            _ => {}
        }
    }
    
    // 文本输入预览
    if let EditState::TextInput { position, content, height } = edit_state {
        let preview_color = Color::from_hex(0xFF00FF);
        let display_content = if content.is_empty() {
            "输入文本...".to_string()
        } else {
            content.clone()
        };
        let text = Text::new(*position, display_content, *height);
        draw_geometry(ctx, &Geometry::Text(text), preview_color);
    }
}
