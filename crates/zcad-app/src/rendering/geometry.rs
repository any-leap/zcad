//! 几何体绘制
//!
//! 使用批量渲染优化性能：收集所有线段后一次性绘制

use eframe::egui::{self, Shape, Stroke};
use zcad_core::geometry::Geometry;
use zcad_core::math::Point2;
use zcad_core::properties::Color;

use super::RenderContext;
use super::dimension::draw_dimension;

/// 批量渲染收集器
/// 
/// 收集所有要绘制的线段，最后一次性提交给 GPU
pub struct BatchRenderer {
    shapes: Vec<Shape>,
}

impl BatchRenderer {
    pub fn new() -> Self {
        Self {
            shapes: Vec::with_capacity(10000), // 预分配容量
        }
    }

    /// 添加线段
    #[inline]
    pub fn add_line(&mut self, p1: egui::Pos2, p2: egui::Pos2, stroke: Stroke) {
        self.shapes.push(Shape::line_segment([p1, p2], stroke));
    }

    /// 添加圆（使用 egui 原生圆）
    #[inline]
    pub fn add_circle(&mut self, center: egui::Pos2, radius: f32, stroke: Stroke) {
        self.shapes.push(Shape::circle_stroke(center, radius, stroke));
    }

    /// 添加填充圆点
    #[inline]
    pub fn add_circle_filled(&mut self, center: egui::Pos2, radius: f32, color: egui::Color32) {
        self.shapes.push(Shape::circle_filled(center, radius, color));
    }

    /// 提交所有形状到画布
    pub fn flush(self, painter: &egui::Painter) {
        if !self.shapes.is_empty() {
            painter.extend(self.shapes);
        }
    }

    /// 形状数量
    pub fn len(&self) -> usize {
        self.shapes.len()
    }
}

/// 根据屏幕像素大小计算弧线分段数（LOD）
/// 
/// 极端简化：只保证视觉上可接受，最小化顶点数
fn calculate_arc_segments(screen_radius: f64, sweep: f64) -> usize {
    // 极端优化：每段弧长约 8-10 像素
    let arc_length = screen_radius * sweep.abs();
    let segments = (arc_length / 10.0).ceil() as usize;
    
    // 非常小的圆/弧用极少分段
    if screen_radius < 5.0 {
        return 4; // 极小的圆用四边形近似
    }
    if screen_radius < 20.0 {
        return segments.clamp(4, 8);
    }
    
    // 限制最多 24 段（极端减少）
    segments.clamp(4, 24)
}

/// 根据屏幕像素大小计算圆的分段数（LOD）
fn calculate_circle_segments(screen_radius: f64) -> usize {
    calculate_arc_segments(screen_radius, std::f64::consts::TAU)
}

/// 检查几何体是否值得渲染（基于屏幕大小的快速剔除）
/// 
/// 注意：剔除条件应该非常宽松，只跳过真正看不见的几何体
fn is_worth_rendering(geometry: &Geometry, camera_zoom: f64) -> bool {
    // 极小的阈值，只跳过真正微不可见的
    let min_pixels = 0.1;
    
    match geometry {
        // 点总是渲染
        Geometry::Point(_) => true,
        // 文本：极小时不渲染（< 0.5 像素）
        Geometry::Text(text) => {
            let screen_height = text.height * camera_zoom;
            screen_height >= 0.5
        }
        // 标注总是渲染
        Geometry::Dimension(_) => true,
        // 线条检查长度
        Geometry::Line(line) => {
            let screen_len = line.length() * camera_zoom;
            screen_len > min_pixels
        }
        // 圆检查半径
        Geometry::Circle(circle) => {
            let screen_radius = circle.radius * camera_zoom;
            screen_radius > min_pixels
        }
        // 弧检查半径
        Geometry::Arc(arc) => {
            let screen_radius = arc.radius * camera_zoom;
            screen_radius > min_pixels
        }
        // 多段线 - 几乎总是渲染
        Geometry::Polyline(polyline) => {
            polyline.vertices.len() >= 2
        }
        // 椭圆检查大小
        Geometry::Ellipse(ellipse) => {
            let screen_size = ellipse.major_axis.norm() * camera_zoom;
            screen_size > min_pixels
        }
        // 其他类型默认渲染
        _ => true,
    }
}

/// 绘制几何体到批量渲染器
pub fn draw_geometry_batched(ctx: &RenderContext, geometry: &Geometry, color: Color, batch: &mut BatchRenderer) {
    // 快速剔除：太小的几何体不渲染
    if !is_worth_rendering(geometry, ctx.camera_zoom) {
        return;
    }
    
    let stroke_color = egui::Color32::from_rgb(color.r, color.g, color.b);
    let stroke = Stroke::new(1.0, stroke_color);

    match geometry {
        Geometry::Point(p) => {
            let screen = ctx.world_to_screen(p.position);
            batch.add_circle_filled(screen, 2.0, stroke_color);
        }
        Geometry::Line(line) => {
            let start = ctx.world_to_screen(line.start);
            let end = ctx.world_to_screen(line.end);
            batch.add_line(start, end, stroke);
        }
        Geometry::Circle(circle) => {
            let center = ctx.world_to_screen(circle.center);
            let screen_radius = (circle.radius * ctx.camera_zoom) as f32;
            
            if screen_radius < 1.0 {
                batch.add_circle_filled(center, 1.0, stroke_color);
            } else if screen_radius < 300.0 {
                // 使用 egui 原生圆
                batch.add_circle(center, screen_radius, stroke);
            } else {
                // 大圆用线段近似
                let segments = calculate_circle_segments(screen_radius as f64);
                let angle_step = std::f64::consts::TAU / segments as f64;
                for i in 0..segments {
                    let a1 = i as f64 * angle_step;
                    let a2 = (i + 1) as f64 * angle_step;
                    let p1 = Point2::new(
                        circle.center.x + circle.radius * a1.cos(),
                        circle.center.y + circle.radius * a1.sin(),
                    );
                    let p2 = Point2::new(
                        circle.center.x + circle.radius * a2.cos(),
                        circle.center.y + circle.radius * a2.sin(),
                    );
                    batch.add_line(ctx.world_to_screen(p1), ctx.world_to_screen(p2), stroke);
                }
            }
        }
        Geometry::Arc(arc) => {
            let screen_radius = arc.radius * ctx.camera_zoom;
            let sweep = arc.sweep_angle();
            
            if screen_radius < 1.0 {
                let start = ctx.world_to_screen(arc.start_point());
                let end = ctx.world_to_screen(arc.end_point());
                batch.add_line(start, end, stroke);
            } else {
                let segments = calculate_arc_segments(screen_radius, sweep);
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
                    batch.add_line(ctx.world_to_screen(p1), ctx.world_to_screen(p2), stroke);
                }
            }
        }
        Geometry::Polyline(polyline) => {
            if polyline.vertices.len() < 2 {
                return;
            }
            for i in 0..polyline.segment_count() {
                let v1 = &polyline.vertices[i];
                let v2 = &polyline.vertices[(i + 1) % polyline.vertices.len()];
                batch.add_line(ctx.world_to_screen(v1.point), ctx.world_to_screen(v2.point), stroke);
            }
        }
        Geometry::Ellipse(ellipse) => {
            let major_len = ellipse.major_axis.norm();
            let minor_len = major_len * ellipse.ratio;
            let screen_size = major_len.max(minor_len) * ctx.camera_zoom;
            
            if screen_size < 1.0 {
                batch.add_circle_filled(ctx.world_to_screen(ellipse.center), 1.0, stroke_color);
            } else {
                let segments = calculate_circle_segments(screen_size);
                let angle_step = std::f64::consts::TAU / segments as f64;
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
                    batch.add_line(ctx.world_to_screen(p1), ctx.world_to_screen(p2), stroke);
                }
            }
        }
        // 文本和标注使用原有的即时渲染（因为它们特殊）
        Geometry::Text(_) | Geometry::Dimension(_) => {
            // 这些在 draw_geometry 中单独处理
        }
        // 其他类型暂不渲染
        _ => {}
    }
}

/// 绘制几何体（原有接口，用于特殊几何体）
pub fn draw_geometry(ctx: &RenderContext, geometry: &Geometry, color: Color) {
    // 快速剔除：太小的几何体不渲染
    if !is_worth_rendering(geometry, ctx.camera_zoom) {
        return;
    }
    
    let stroke_color = egui::Color32::from_rgb(color.r, color.g, color.b);
    let stroke = Stroke::new(1.0, stroke_color);
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
            let screen_radius = circle.radius * ctx.camera_zoom;
            
            // 如果太小，画一个点
            if screen_radius < 1.0 {
                painter.circle_filled(center, 1.0, stroke_color);
                return;
            }
            
            // 使用 LOD 计算分段数
            let segments = calculate_circle_segments(screen_radius);
            
            // 如果分段数很少或半径适中，直接用 egui 的圆
            if segments >= 16 && screen_radius < 500.0 {
                painter.circle_stroke(center, screen_radius as f32, stroke);
            } else {
                // 手动绘制多段线近似圆
                let angle_step = std::f64::consts::TAU / segments as f64;
                for i in 0..segments {
                    let a1 = i as f64 * angle_step;
                    let a2 = (i + 1) as f64 * angle_step;
                    
                    let p1 = Point2::new(
                        circle.center.x + circle.radius * a1.cos(),
                        circle.center.y + circle.radius * a1.sin(),
                    );
                    let p2 = Point2::new(
                        circle.center.x + circle.radius * a2.cos(),
                        circle.center.y + circle.radius * a2.sin(),
                    );
                    
                    let s1 = ctx.world_to_screen(p1);
                    let s2 = ctx.world_to_screen(p2);
                    painter.line_segment([s1, s2], stroke);
                }
            }
        }
        Geometry::Arc(arc) => {
            let screen_radius = arc.radius * ctx.camera_zoom;
            let sweep = arc.sweep_angle();
            
            // 如果太小，画一条直线
            if screen_radius < 1.0 {
                let start = ctx.world_to_screen(arc.start_point());
                let end = ctx.world_to_screen(arc.end_point());
                painter.line_segment([start, end], stroke);
                return;
            }
            
            // 使用 LOD 计算分段数
            let segments = calculate_arc_segments(screen_radius, sweep);
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
            
            // 计算屏幕上的字体大小（考虑缩放）
            let font_size = (text.height * ctx.camera_zoom) as f32;
            
            // 只有当字体极其微小时才不渲染（< 0.5 像素基本看不见了）
            if font_size < 0.5 {
                return;
            }
            
            // 限制字体范围：最小 1 像素（egui 限制），最大 200 像素
            let font_size = font_size.clamp(1.0, 200.0);
            let screen = ctx.world_to_screen(text.position);
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
            let major_len = ellipse.major_axis.norm();
            let minor_len = major_len * ellipse.ratio;
            
            // 使用较大轴计算屏幕大小
            let screen_radius = major_len.max(minor_len) * ctx.camera_zoom;
            
            // 如果太小，画一个点
            if screen_radius < 1.0 {
                let center = ctx.world_to_screen(ellipse.center);
                painter.circle_filled(center, 1.0, stroke_color);
                return;
            }
            
            // 使用 LOD 计算分段数
            let segments = calculate_circle_segments(screen_radius);
            let angle_step = std::f64::consts::TAU / segments as f64;
            
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

/// 极简渲染模式：大量实体时只画边界框/直线
/// 
/// 这个函数用于当实体数量非常多时，跳过复杂的圆弧等计算，
/// 只绘制最简单的线段来表示几何体位置
pub fn draw_geometry_simplified(
    ctx: &RenderContext, 
    geometry: &Geometry, 
    color: Color,
    shapes: &mut Vec<egui::Shape>,
) {
    let stroke_color = egui::Color32::from_rgb(color.r, color.g, color.b);
    let stroke = Stroke::new(1.0, stroke_color);

    match geometry {
        Geometry::Point(p) => {
            let screen = ctx.world_to_screen(p.position);
            shapes.push(Shape::circle_filled(screen, 1.5, stroke_color));
        }
        Geometry::Line(line) => {
            let start = ctx.world_to_screen(line.start);
            let end = ctx.world_to_screen(line.end);
            shapes.push(Shape::line_segment([start, end], stroke));
        }
        Geometry::Circle(circle) => {
            // 简化：用 4 条线段画正方形
            let center = ctx.world_to_screen(circle.center);
            let r = (circle.radius * ctx.camera_zoom) as f32;
            if r < 1.0 {
                shapes.push(Shape::circle_filled(center, 1.0, stroke_color));
            } else {
                // 画十字 + 正方形外框
                let pts = [
                    egui::pos2(center.x - r, center.y),
                    egui::pos2(center.x + r, center.y),
                    egui::pos2(center.x, center.y - r),
                    egui::pos2(center.x, center.y + r),
                ];
                shapes.push(Shape::line_segment([pts[0], pts[1]], stroke));
                shapes.push(Shape::line_segment([pts[2], pts[3]], stroke));
            }
        }
        Geometry::Arc(arc) => {
            // 简化：只画起点到终点的直线
            let start = ctx.world_to_screen(arc.start_point());
            let end = ctx.world_to_screen(arc.end_point());
            shapes.push(Shape::line_segment([start, end], stroke));
        }
        Geometry::Polyline(polyline) => {
            // 简化：只画第一段和最后一段
            if polyline.vertices.len() >= 2 {
                let first = ctx.world_to_screen(polyline.vertices[0].point);
                let second = ctx.world_to_screen(polyline.vertices[1].point);
                shapes.push(Shape::line_segment([first, second], stroke));
                
                if polyline.vertices.len() > 2 {
                    let last_idx = polyline.vertices.len() - 1;
                    let prev = ctx.world_to_screen(polyline.vertices[last_idx - 1].point);
                    let last = ctx.world_to_screen(polyline.vertices[last_idx].point);
                    shapes.push(Shape::line_segment([prev, last], stroke));
                }
            }
        }
        Geometry::Ellipse(ellipse) => {
            // 简化：画中心十字
            let center = ctx.world_to_screen(ellipse.center);
            let r = (ellipse.major_axis.norm() * ctx.camera_zoom) as f32;
            if r < 1.0 {
                shapes.push(Shape::circle_filled(center, 1.0, stroke_color));
            } else {
                shapes.push(Shape::line_segment(
                    [egui::pos2(center.x - r, center.y), egui::pos2(center.x + r, center.y)],
                    stroke,
                ));
            }
        }
        // 跳过文本、标注、表格 - 太复杂
        Geometry::Text(_) | Geometry::Dimension(_) | Geometry::Table(_) => {}
        // 其他类型跳过
        _ => {}
    }
}
