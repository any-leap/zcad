//! 标注绘制

use eframe::egui;
use zcad_core::geometry::{Dimension, DimensionType};
use zcad_core::math::Point2;
use zcad_core::properties::Color;

use super::RenderContext;

/// 绘制标注
pub fn draw_dimension(ctx: &RenderContext, dim: &Dimension, color: Color) {
    let stroke_color = egui::Color32::from_rgb(color.r, color.g, color.b);
    let stroke = egui::Stroke::new(1.0, stroke_color); // 标注线通常细一些
    let painter = ctx.painter;
    let rect = ctx.rect;
    
    match dim.dim_type {
        DimensionType::Aligned | DimensionType::Linear => {
            // 对齐标注逻辑 (参考 LibreCAD 实现)
            // 标注样式参数（单位：世界坐标）
            let dim_scale = 1.0 / ctx.camera_zoom; // 根据缩放调整
            let ext_line_offset = 0.625 * dim_scale; // 界线起点偏移 (DIMEXO)
            let ext_line_extension = 1.25 * dim_scale; // 界线超出标注线的距离 (DIMEXE)
            
            // 1. 计算标注线的方向向量 (平行于 p1->p2)
            let dir = (dim.definition_point2 - dim.definition_point1).normalize();
            // 2. 计算标注线的法向量 (垂直于 p1->p2)
            let perp = zcad_core::math::Vector2::new(-dir.y, dir.x);
            
            // 3. 计算标注线在法向量方向上的投影距离
            let v_loc = dim.line_location - dim.definition_point1;
            let dist = v_loc.dot(&perp);
            let sign = if dist.abs() < 1e-10 { 1.0 } else { dist.signum() };
            
            // 4. 计算标注线的两个端点
            let dim_p1 = dim.definition_point1 + perp * dist;
            let dim_p2 = dim.definition_point2 + perp * dist;
            
            let dim_p1_s = ctx.world_to_screen(dim_p1);
            let dim_p2_s = ctx.world_to_screen(dim_p2);
            
            // 绘制界线 (Extension lines)
            // 界线起点：从定义点偏移 ext_line_offset
            let ext_start_offset = perp * (ext_line_offset * sign);
            let ext_start_p1 = dim.definition_point1 + ext_start_offset;
            let ext_start_p2 = dim.definition_point2 + ext_start_offset;
            
            // 界线终点：标注线位置 + 超出距离
            let ext_end_offset = perp * (dist + ext_line_extension * sign);
            let ext_end_p1 = dim.definition_point1 + ext_end_offset;
            let ext_end_p2 = dim.definition_point2 + ext_end_offset;
            
            painter.line_segment([ctx.world_to_screen(ext_start_p1), ctx.world_to_screen(ext_end_p1)], stroke);
            painter.line_segment([ctx.world_to_screen(ext_start_p2), ctx.world_to_screen(ext_end_p2)], stroke);
            
            // 绘制尺寸线 (Dimension line)
            painter.line_segment([dim_p1_s, dim_p2_s], stroke);
            
            // 绘制箭头
            draw_arrow(painter, dim_p1_s, dim_p2_s, stroke);
            draw_arrow(painter, dim_p2_s, dim_p1_s, stroke);
            
            // 绘制文本
            let text_content = dim.display_text().replace("%%C", "Ø");
            let mid_point = dim.get_text_position();
            
            // 计算旋转角度
            let diff = dim.definition_point2 - dim.definition_point1;
            let mut angle = diff.y.atan2(diff.x);
            // 标准化角度：保持文字直立（从底部或右侧可读）
            if angle.abs() > std::f64::consts::FRAC_PI_2 {
                angle += std::f64::consts::PI;
            }
            if angle > std::f64::consts::FRAC_PI_2 {
                angle -= std::f64::consts::PI * 2.0;
            }

            draw_dimension_text(ctx, mid_point, &text_content, dim.text_height, stroke_color, angle as f32);
        }
        DimensionType::Radius => {
            // 半径标注：p1=圆心, p2=圆上一点
            let center = dim.definition_point1;
            let text_pos = dim.get_text_position();
            
            let radius = (dim.definition_point2 - center).norm();
            let dir = (text_pos - center).normalize();
            
            // 箭头位置在圆弧上
            let arrow_pos = center + dir * radius;
            
            let center_s = ctx.world_to_screen(center);
            let arrow_s = ctx.world_to_screen(arrow_pos);
            let text_s = ctx.world_to_screen(text_pos);
            
            // 绘制圆心标记（小十字）
            let cross_size = 3.0;
            painter.line_segment(
                [egui::pos2(center_s.x - cross_size, center_s.y), egui::pos2(center_s.x + cross_size, center_s.y)],
                stroke,
            );
            painter.line_segment(
                [egui::pos2(center_s.x, center_s.y - cross_size), egui::pos2(center_s.x, center_s.y + cross_size)],
                stroke,
            );
            
            // 绘制从圆心到文本的线
            painter.line_segment([center_s, text_s], stroke);
            
            // 绘制箭头（指向圆弧）
            draw_arrow(painter, center_s, arrow_s, stroke);
            
            // 绘制文本
            let text_content = dim.display_text().replace("%%C", "Ø");
            let diff = text_pos - center;
            let mut angle = diff.y.atan2(diff.x);
            if angle.abs() > std::f64::consts::FRAC_PI_2 {
                angle += std::f64::consts::PI;
            }
            if angle > std::f64::consts::FRAC_PI_2 {
                angle -= std::f64::consts::PI * 2.0;
            }
            
            draw_dimension_text(ctx, text_pos, &text_content, dim.text_height, stroke_color, angle as f32);
        }
        DimensionType::Diameter => {
            // 直径标注：p1, p2 是直径端点
            let center = (dim.definition_point1 + dim.definition_point2.coords) * 0.5;
            let p1 = dim.definition_point1;
            let p2 = dim.definition_point2;
            
            let p1_s = ctx.world_to_screen(p1);
            let p2_s = ctx.world_to_screen(p2);
            
            // 绘制直径线
            painter.line_segment([p1_s, p2_s], stroke);
            
            // 绘制箭头（向外）
            let center_s = ctx.world_to_screen(center);
            draw_arrow(painter, center_s, p1_s, stroke);
            draw_arrow(painter, center_s, p2_s, stroke);
            
            // 绘制文本
            let text_content = dim.display_text().replace("%%C", "Ø");
            let text_pos = dim.get_text_position();
            let diff = p2 - p1;
            let mut angle = diff.y.atan2(diff.x);
            if angle.abs() > std::f64::consts::FRAC_PI_2 {
                angle += std::f64::consts::PI;
            }
            if angle > std::f64::consts::FRAC_PI_2 {
                angle -= std::f64::consts::PI * 2.0;
            }
            
            draw_dimension_text(ctx, text_pos, &text_content, dim.text_height, stroke_color, angle as f32);
        }
        _ => {
            // 其他类型暂用简化绘制
            let p1 = ctx.world_to_screen(dim.definition_point1);
            let p2 = ctx.world_to_screen(dim.definition_point2);
            let line_loc = ctx.world_to_screen(dim.line_location);
            painter.line_segment([p1, line_loc], stroke);
            painter.line_segment([p2, line_loc], stroke);
        }
    }
}

/// 绘制箭头
pub fn draw_arrow(painter: &egui::Painter, from: egui::Pos2, to: egui::Pos2, stroke: egui::Stroke) {
    let arrow_len = 10.0;
    let dir = (to - from).normalized();
    if dir.length() > 0.0 {
        let arrow1_end = to - dir * arrow_len + egui::vec2(-dir.y, dir.x) * arrow_len * 0.3;
        let arrow2_end = to - dir * arrow_len - egui::vec2(-dir.y, dir.x) * arrow_len * 0.3;
        painter.line_segment([to, arrow1_end], stroke);
        painter.line_segment([to, arrow2_end], stroke);
    }
}

/// 绘制标注文本（带旋转和背景）
pub fn draw_dimension_text(
    ctx: &RenderContext,
    world_pos: Point2,
    text: &str,
    height: f64,
    color: egui::Color32,
    angle: f32,
) {
    let painter = ctx.painter;
    let font_id = egui::FontId::proportional((height * ctx.camera_zoom).max(8.0) as f32);
    let galley = painter.layout_no_wrap(text.to_string(), font_id, color);
    
    let text_screen_pos = ctx.world_to_screen(world_pos);
    
    let galley_size = galley.rect.size();
    let half_size = galley_size * 0.5;
    
    // 使用 egui 的 Rot2 进行旋转计算
    let rot = egui::emath::Rot2::from_angle(angle);
    let offset = rot * half_size;
    let draw_pos = text_screen_pos - offset;
    
    // 绘制背景（旋转的矩形）
    let bg_expand = 2.0;
    let bg_half_size = half_size + egui::vec2(bg_expand, bg_expand);
    
    let corners = [
        text_screen_pos + rot * egui::vec2(-bg_half_size.x, -bg_half_size.y),
        text_screen_pos + rot * egui::vec2(bg_half_size.x, -bg_half_size.y),
        text_screen_pos + rot * egui::vec2(bg_half_size.x, bg_half_size.y),
        text_screen_pos + rot * egui::vec2(-bg_half_size.x, bg_half_size.y),
    ];
    
    painter.add(egui::Shape::convex_polygon(
        corners.to_vec(),
        egui::Color32::from_rgb(30, 30, 46), // 背景色
        egui::Stroke::NONE,
    ));
    
    // 绘制旋转文本
    painter.add(egui::Shape::Text(egui::epaint::TextShape {
        pos: draw_pos,
        galley,
        underline: egui::Stroke::NONE,
        override_text_color: Some(color),
        angle,
        fallback_color: color,
        opacity_factor: 1.0,
    }));
}
