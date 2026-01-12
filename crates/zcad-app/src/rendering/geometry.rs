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
            // 文本绘制（支持高度缩放和对齐）
            let screen = ctx.world_to_screen(text.position);
            
            // 计算屏幕上的字体大小（考虑缩放）
            let font_size = (text.height * ctx.camera_zoom).max(8.0).min(200.0) as f32;
            let font_id = egui::FontId::proportional(font_size);
            
            // 根据对齐方式设置锚点
            use zcad_core::geometry::TextAlignment;
            let align = match text.alignment {
                TextAlignment::Left => egui::Align2::LEFT_BOTTOM,
                TextAlignment::Center => egui::Align2::CENTER_BOTTOM,
                TextAlignment::Right => egui::Align2::RIGHT_BOTTOM,
            };
            
            // 如果有旋转，使用 TextShape
            if text.rotation.abs() > 1e-6 {
                let galley = painter.layout_no_wrap(
                    text.content.clone(),
                    font_id,
                    stroke_color,
                );
                
                let galley_size = galley.rect.size();
                let half_size = galley_size * 0.5;
                let angle = -text.rotation as f32; // egui Y轴向下，所以取反
                
                let rot = egui::emath::Rot2::from_angle(angle);
                let offset = match text.alignment {
                    TextAlignment::Left => egui::vec2(0.0, half_size.y),
                    TextAlignment::Center => egui::vec2(half_size.x, half_size.y),
                    TextAlignment::Right => egui::vec2(galley_size.x, half_size.y),
                };
                let draw_pos = screen - rot * offset;
                
                painter.add(egui::Shape::Text(egui::epaint::TextShape {
                    pos: draw_pos,
                    galley,
                    underline: egui::Stroke::NONE,
                    override_text_color: Some(stroke_color),
                    angle,
                    fallback_color: stroke_color,
                    opacity_factor: 1.0,
                }));
            } else {
                painter.text(
                    screen,
                    align,
                    &text.content,
                    font_id,
                    stroke_color,
                );
            }
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
        Geometry::Table(table) => {
            // 表格绘制
            let border_stroke = egui::Stroke::new(1.0, stroke_color);
            let text_color = stroke_color;
            
            // 绘制所有单元格
            for row in 0..table.rows {
                for col in 0..table.columns {
                    let cell_pos = table.cell_position(row, col);
                    let (cell_width, cell_height) = table.cell_size(row, col);
                    
                    // 单元格四个角（世界坐标）
                    let top_left = cell_pos;
                    let top_right = Point2::new(cell_pos.x + cell_width, cell_pos.y);
                    let bottom_left = Point2::new(cell_pos.x, cell_pos.y - cell_height);
                    let bottom_right = Point2::new(cell_pos.x + cell_width, cell_pos.y - cell_height);
                    
                    // 转换为屏幕坐标
                    let s_tl = ctx.world_to_screen(top_left);
                    let s_tr = ctx.world_to_screen(top_right);
                    let s_bl = ctx.world_to_screen(bottom_left);
                    let s_br = ctx.world_to_screen(bottom_right);
                    
                    // 绘制单元格边框
                    if table.style.show_grid {
                        painter.line_segment([s_tl, s_tr], border_stroke);
                        painter.line_segment([s_tr, s_br], border_stroke);
                        painter.line_segment([s_br, s_bl], border_stroke);
                        painter.line_segment([s_bl, s_tl], border_stroke);
                    }
                    
                    // 绘制单元格内容
                    if let Some(cell) = table.get_cell(row, col) {
                        if !cell.content.is_empty() {
                            // 计算文本位置（考虑边距）
                            let margin = table.style.cell_margin;
                            let text_pos = Point2::new(
                                cell_pos.x + margin,
                                cell_pos.y - margin - table.style.text_height,
                            );
                            let s_text = ctx.world_to_screen(text_pos);
                            
                            // 计算字体大小
                            let font_size = (table.style.text_height * ctx.camera_zoom)
                                .max(6.0)
                                .min(100.0) as f32;
                            
                            painter.text(
                                s_text,
                                egui::Align2::LEFT_TOP,
                                &cell.content,
                                egui::FontId::proportional(font_size),
                                text_color,
                            );
                        }
                    }
                }
            }
        }
        // 其他几何类型暂不渲染详细图形
        Geometry::Spline(_) | Geometry::Hatch(_) | Geometry::Leader(_) => {
            // TODO: 实现详细渲染
        }
    }
}
