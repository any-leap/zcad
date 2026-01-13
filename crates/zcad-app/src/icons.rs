//! 自定义矢量图标
//!
//! 使用 egui Painter API 绘制专业 CAD 风格的图标

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

/// 图标绘制 trait
pub trait IconPainter {
    fn paint(&self, painter: &egui::Painter, rect: Rect, color: Color32);
}

/// 图标大小
pub const ICON_SIZE: f32 = 16.0;

/// 绘制选择工具图标 (箭头)
pub fn draw_select_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let s = rect.width() * 0.35;
    
    // 箭头形状
    let points = [
        Pos2::new(c.x - s * 0.6, c.y - s),       // 顶点
        Pos2::new(c.x - s * 0.6, c.y + s * 0.3), // 左下
        Pos2::new(c.x - s * 0.1, c.y),           // 拐点
        Pos2::new(c.x + s * 0.3, c.y + s * 0.8), // 右下角
        Pos2::new(c.x + s * 0.5, c.y + s * 0.5), // 右下
        Pos2::new(c.x + s * 0.1, c.y - s * 0.1), // 拐点右
        Pos2::new(c.x + s * 0.4, c.y - s),       // 右上
    ];
    
    painter.add(egui::Shape::convex_polygon(
        points.to_vec(),
        color,
        Stroke::NONE,
    ));
}

/// 绘制直线图标
pub fn draw_line_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let margin = rect.width() * 0.2;
    let start = Pos2::new(rect.left() + margin, rect.bottom() - margin);
    let end = Pos2::new(rect.right() - margin, rect.top() + margin);
    
    painter.line_segment([start, end], Stroke::new(2.0, color));
    
    // 端点
    painter.circle_filled(start, 2.0, color);
    painter.circle_filled(end, 2.0, color);
}

/// 绘制圆形图标
pub fn draw_circle_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let center = rect.center();
    let radius = rect.width() * 0.35;
    
    painter.circle_stroke(center, radius, Stroke::new(1.5, color));
    
    // 圆心点
    painter.circle_filled(center, 1.5, color);
}

/// 绘制矩形图标
pub fn draw_rectangle_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let margin = rect.width() * 0.22;
    let inner_rect = Rect::from_min_max(
        Pos2::new(rect.left() + margin, rect.top() + margin * 1.2),
        Pos2::new(rect.right() - margin, rect.bottom() - margin * 1.2),
    );
    
    painter.rect_stroke(inner_rect, 0.0, Stroke::new(1.5, color), egui::StrokeKind::Outside);
}

/// 绘制圆弧图标
pub fn draw_arc_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let center = Pos2::new(rect.center().x, rect.bottom() - rect.height() * 0.3);
    let radius = rect.width() * 0.35;
    
    // 绘制圆弧 (半圆)
    let n_points = 16;
    let mut points = Vec::with_capacity(n_points);
    for i in 0..=n_points {
        let t = std::f32::consts::PI * (i as f32 / n_points as f32);
        let x = center.x + radius * t.cos();
        let y = center.y - radius * t.sin();
        points.push(Pos2::new(x, y));
    }
    
    painter.add(egui::Shape::line(points, Stroke::new(1.5, color)));
    
    // 端点
    painter.circle_filled(Pos2::new(center.x - radius, center.y), 2.0, color);
    painter.circle_filled(Pos2::new(center.x + radius, center.y), 2.0, color);
}

/// 绘制多段线图标
pub fn draw_polyline_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let margin = rect.width() * 0.18;
    let points = [
        Pos2::new(rect.left() + margin, rect.bottom() - margin),
        Pos2::new(rect.left() + margin * 1.8, rect.top() + margin * 1.5),
        Pos2::new(rect.right() - margin * 1.8, rect.center().y + margin * 0.5),
        Pos2::new(rect.right() - margin, rect.top() + margin),
    ];
    
    painter.add(egui::Shape::line(points.to_vec(), Stroke::new(1.5, color)));
    
    // 端点
    for p in &points {
        painter.circle_filled(*p, 2.0, color);
    }
}

/// 绘制标注图标 (尺寸线)
pub fn draw_dimension_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let margin = rect.width() * 0.15;
    let y = rect.center().y;
    
    // 水平尺寸线
    let left = Pos2::new(rect.left() + margin, y);
    let right = Pos2::new(rect.right() - margin, y);
    painter.line_segment([left, right], Stroke::new(1.5, color));
    
    // 箭头
    let arrow_size = 3.0;
    // 左箭头
    painter.line_segment([left, Pos2::new(left.x + arrow_size, y - arrow_size)], Stroke::new(1.5, color));
    painter.line_segment([left, Pos2::new(left.x + arrow_size, y + arrow_size)], Stroke::new(1.5, color));
    // 右箭头
    painter.line_segment([right, Pos2::new(right.x - arrow_size, y - arrow_size)], Stroke::new(1.5, color));
    painter.line_segment([right, Pos2::new(right.x - arrow_size, y + arrow_size)], Stroke::new(1.5, color));
    
    // 延伸线
    painter.line_segment(
        [Pos2::new(left.x, y - margin * 1.2), Pos2::new(left.x, y + margin * 1.2)],
        Stroke::new(1.0, color),
    );
    painter.line_segment(
        [Pos2::new(right.x, y - margin * 1.2), Pos2::new(right.x, y + margin * 1.2)],
        Stroke::new(1.0, color),
    );
}

/// 绘制删除图标 (垃圾桶)
pub fn draw_delete_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let w = rect.width() * 0.25;
    let h = rect.height() * 0.35;
    
    // 桶身
    let body = Rect::from_center_size(Pos2::new(c.x, c.y + h * 0.15), Vec2::new(w * 1.6, h * 1.5));
    painter.rect_stroke(body, 1.0, Stroke::new(1.5, color), egui::StrokeKind::Outside);
    
    // 盖子
    let lid_y = c.y - h * 0.55;
    painter.line_segment(
        [Pos2::new(c.x - w * 1.0, lid_y), Pos2::new(c.x + w * 1.0, lid_y)],
        Stroke::new(1.5, color),
    );
    
    // 把手
    painter.line_segment(
        [Pos2::new(c.x - w * 0.3, lid_y), Pos2::new(c.x - w * 0.3, lid_y - h * 0.3)],
        Stroke::new(1.5, color),
    );
    painter.line_segment(
        [Pos2::new(c.x - w * 0.3, lid_y - h * 0.3), Pos2::new(c.x + w * 0.3, lid_y - h * 0.3)],
        Stroke::new(1.5, color),
    );
    painter.line_segment(
        [Pos2::new(c.x + w * 0.3, lid_y - h * 0.3), Pos2::new(c.x + w * 0.3, lid_y)],
        Stroke::new(1.5, color),
    );
}

/// 绘制撤销图标
pub fn draw_undo_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = rect.width() * 0.3;
    
    // 圆弧
    let n_points = 12;
    let mut points = Vec::with_capacity(n_points);
    for i in 0..=n_points {
        let t = std::f32::consts::PI * 0.8 * (i as f32 / n_points as f32) + std::f32::consts::PI * 0.6;
        let x = c.x + r * t.cos();
        let y = c.y + r * t.sin();
        points.push(Pos2::new(x, y));
    }
    painter.add(egui::Shape::line(points.clone(), Stroke::new(1.5, color)));
    
    // 箭头
    if let Some(start) = points.first() {
        let arrow_size = 3.0;
        painter.line_segment([*start, Pos2::new(start.x - arrow_size, start.y - arrow_size * 0.5)], Stroke::new(1.5, color));
        painter.line_segment([*start, Pos2::new(start.x + arrow_size * 0.3, start.y - arrow_size)], Stroke::new(1.5, color));
    }
}

/// 绘制重做图标
pub fn draw_redo_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r = rect.width() * 0.3;
    
    // 圆弧 (镜像)
    let n_points = 12;
    let mut points = Vec::with_capacity(n_points);
    for i in 0..=n_points {
        let t = -std::f32::consts::PI * 0.8 * (i as f32 / n_points as f32) - std::f32::consts::PI * 0.6 + std::f32::consts::PI;
        let x = c.x + r * t.cos();
        let y = c.y + r * t.sin();
        points.push(Pos2::new(x, y));
    }
    painter.add(egui::Shape::line(points.clone(), Stroke::new(1.5, color)));
    
    // 箭头
    if let Some(start) = points.first() {
        let arrow_size = 3.0;
        painter.line_segment([*start, Pos2::new(start.x + arrow_size, start.y - arrow_size * 0.5)], Stroke::new(1.5, color));
        painter.line_segment([*start, Pos2::new(start.x - arrow_size * 0.3, start.y - arrow_size)], Stroke::new(1.5, color));
    }
}

/// 绘制正交模式图标
pub fn draw_ortho_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let margin = rect.width() * 0.2;
    
    // 水平线
    painter.line_segment(
        [Pos2::new(rect.left() + margin, rect.center().y), Pos2::new(rect.right() - margin, rect.center().y)],
        Stroke::new(1.5, color),
    );
    
    // 垂直线
    painter.line_segment(
        [Pos2::new(rect.center().x, rect.top() + margin), Pos2::new(rect.center().x, rect.bottom() - margin)],
        Stroke::new(1.5, color),
    );
    
    // 直角标记
    let corner = rect.width() * 0.12;
    painter.line_segment(
        [Pos2::new(rect.center().x + corner, rect.center().y), Pos2::new(rect.center().x + corner, rect.center().y - corner)],
        Stroke::new(1.0, color),
    );
    painter.line_segment(
        [Pos2::new(rect.center().x + corner, rect.center().y - corner), Pos2::new(rect.center().x, rect.center().y - corner)],
        Stroke::new(1.0, color),
    );
}

/// 绘制网格图标
pub fn draw_grid_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let margin = rect.width() * 0.18;
    let step = (rect.width() - margin * 2.0) / 3.0;
    
    // 垂直线
    for i in 0..=3 {
        let x = rect.left() + margin + step * i as f32;
        painter.line_segment(
            [Pos2::new(x, rect.top() + margin), Pos2::new(x, rect.bottom() - margin)],
            Stroke::new(1.0, color),
        );
    }
    
    // 水平线
    for i in 0..=3 {
        let y = rect.top() + margin + step * i as f32;
        painter.line_segment(
            [Pos2::new(rect.left() + margin, y), Pos2::new(rect.right() - margin, y)],
            Stroke::new(1.0, color),
        );
    }
}

/// 绘制适应缩放图标
pub fn draw_zoom_fit_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let margin = rect.width() * 0.15;
    let corner_len = rect.width() * 0.2;
    
    // 四个角的括号
    let corners = [
        (rect.left() + margin, rect.top() + margin, 1.0, 1.0),      // 左上
        (rect.right() - margin, rect.top() + margin, -1.0, 1.0),    // 右上
        (rect.left() + margin, rect.bottom() - margin, 1.0, -1.0),  // 左下
        (rect.right() - margin, rect.bottom() - margin, -1.0, -1.0), // 右下
    ];
    
    for (x, y, dx, dy) in corners {
        painter.line_segment(
            [Pos2::new(x, y), Pos2::new(x + corner_len * dx, y)],
            Stroke::new(1.5, color),
        );
        painter.line_segment(
            [Pos2::new(x, y), Pos2::new(x, y + corner_len * dy)],
            Stroke::new(1.5, color),
        );
    }
}

/// 绘制捕捉图标 (十字准心)
pub fn draw_snap_icon(painter: &egui::Painter, rect: Rect, color: Color32) {
    let c = rect.center();
    let r_outer = rect.width() * 0.35;
    let r_inner = rect.width() * 0.15;
    let cross = rect.width() * 0.12;
    
    // 外圆
    painter.circle_stroke(c, r_outer, Stroke::new(1.5, color));
    
    // 十字
    painter.line_segment([Pos2::new(c.x - cross, c.y), Pos2::new(c.x + cross, c.y)], Stroke::new(1.5, color));
    painter.line_segment([Pos2::new(c.x, c.y - cross), Pos2::new(c.x, c.y + cross)], Stroke::new(1.5, color));
}
