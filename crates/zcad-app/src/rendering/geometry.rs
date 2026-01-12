//! 几何体绘制

use eframe::egui;
use zcad_core::geometry::Geometry;
use zcad_core::math::Point2;
use zcad_core::properties::Color;

use super::RenderContext;
use super::dimension::draw_dimension;

/// 绘制几何体
pub fn draw_geometry(ctx: &RenderContext, geometry: &Geometry, color: Color) {
    let stroke_color = egui::Color32::from_rgb(color.r, color.g, color.b);
    let stroke = egui::Stroke::new(1.5, stroke_color);
    let painter = ctx.painter;

    match geometry {
        Geometry::Point(p) => {
            let screen = ctx.world_to_screen(p.position);
            painter.circle_filled(screen, 3.0, stroke_color);
        }
        Geometry::Line(line) => {
            let start = ctx.world_to_screen(line.start);
            let end = ctx.world_to_screen(line.end);
            painter.line_segment([start, end], stroke);
        }
        Geometry::Circle(circle) => {
            let center = ctx.world_to_screen(circle.center);
            let radius = (circle.radius * ctx.camera_zoom) as f32;
            painter.circle_stroke(center, radius, stroke);
        }
        Geometry::Arc(arc) => {
            // 简化：用线段近似弧线
            let segments = 32;
            let sweep = arc.sweep_angle();
            let angle_step = sweep / segments as f64;
            
            for i in 0..segments {
                let a1 = arc.start_angle + i as f64 * angle_step;
                let a2 = arc.start_angle + (i + 1) as f64 * angle_step;
                
                let p1 = Point2::new(
                    arc.center.x + arc.radius * a1.cos(),
                    arc.center.y + arc.radius * a1.sin(),
                );
                let p2 = Point2::new(
                    arc.center.x + arc.radius * a2.cos(),
                    arc.center.y + arc.radius * a2.sin(),
                );
                
                let s1 = ctx.world_to_screen(p1);
                let s2 = ctx.world_to_screen(p2);
                painter.line_segment([s1, s2], stroke);
            }
        }
        Geometry::Polyline(polyline) => {
            if polyline.vertices.len() < 2 {
                return;
            }
            
            for i in 0..polyline.segment_count() {
                let v1 = &polyline.vertices[i];
                let v2 = &polyline.vertices[(i + 1) % polyline.vertices.len()];
                
                let s1 = ctx.world_to_screen(v1.point);
                let s2 = ctx.world_to_screen(v2.point);
                painter.line_segment([s1, s2], stroke);
            }
        }
        Geometry::Text(text) => {
            // 简化的文本绘制
            let screen = ctx.world_to_screen(text.position);
            painter.text(
                screen,
                egui::Align2::LEFT_BOTTOM,
                &text.content,
                egui::FontId::proportional(12.0),
                stroke_color,
            );
        }
        Geometry::Dimension(dim) => {
            draw_dimension(ctx, dim, color);
        }
        Geometry::Ellipse(ellipse) => {
            // 用线段近似椭圆
            let segments = 32;
            let angle_step = std::f64::consts::TAU / segments as f64;
            let major_len = ellipse.major_axis.norm();
            let minor_len = major_len * ellipse.ratio;
            
            for i in 0..segments {
                let a1 = i as f64 * angle_step;
                let a2 = (i + 1) as f64 * angle_step;
                
                let p1 = Point2::new(
                    ellipse.center.x + major_len * a1.cos(),
                    ellipse.center.y + minor_len * a1.sin(),
                );
                let p2 = Point2::new(
                    ellipse.center.x + major_len * a2.cos(),
                    ellipse.center.y + minor_len * a2.sin(),
                );
                
                let s1 = ctx.world_to_screen(p1);
                let s2 = ctx.world_to_screen(p2);
                painter.line_segment([s1, s2], stroke);
            }
        }
        // 其他几何类型暂不渲染详细图形
        Geometry::Spline(_) | Geometry::Hatch(_) | Geometry::Leader(_) => {
            // TODO: 实现详细渲染
        }
    }
}
