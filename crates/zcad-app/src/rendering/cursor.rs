//! 光标和捕捉标记绘制

use eframe::egui;
use zcad_core::math::Point2;
use zcad_core::snap::SnapType;

use super::RenderContext;

/// 绘制十字光标
pub fn draw_crosshair(ctx: &RenderContext, world_pos: Point2) {
    let screen = ctx.world_to_screen(world_pos);
    let size = 15.0;
    let color = egui::Color32::WHITE;
    let stroke = egui::Stroke::new(1.0, color);
    let painter = ctx.painter;

    painter.line_segment(
        [egui::Pos2::new(screen.x - size, screen.y), egui::Pos2::new(screen.x + size, screen.y)],
        stroke,
    );
    painter.line_segment(
        [egui::Pos2::new(screen.x, screen.y - size), egui::Pos2::new(screen.x, screen.y + size)],
        stroke,
    );
}

/// 绘制捕捉标记
pub fn draw_snap_marker(ctx: &RenderContext, snap_type: SnapType, world_pos: Point2) {
    let screen = ctx.world_to_screen(world_pos);
    let size = 8.0;
    let stroke = egui::Stroke::new(2.0, egui::Color32::YELLOW);
    let painter = ctx.painter;

    match snap_type {
        SnapType::Endpoint => {
            // 方形标记
            painter.rect_stroke(
                egui::Rect::from_center_size(screen, egui::vec2(size * 2.0, size * 2.0)),
                egui::CornerRadius::ZERO,
                stroke,
                egui::StrokeKind::Outside,
            );
        }
        SnapType::Midpoint => {
            // 三角形标记
            let points = [
                egui::Pos2::new(screen.x, screen.y - size),
                egui::Pos2::new(screen.x - size, screen.y + size),
                egui::Pos2::new(screen.x + size, screen.y + size),
            ];
            painter.add(egui::Shape::closed_line(points.to_vec(), stroke));
        }
        SnapType::Center => {
            // 圆形标记
            painter.circle_stroke(screen, size, stroke);
        }
        SnapType::Intersection => {
            // X形标记
            painter.line_segment(
                [egui::Pos2::new(screen.x - size, screen.y - size), egui::Pos2::new(screen.x + size, screen.y + size)],
                stroke,
            );
            painter.line_segment(
                [egui::Pos2::new(screen.x - size, screen.y + size), egui::Pos2::new(screen.x + size, screen.y - size)],
                stroke,
            );
        }
        SnapType::Perpendicular => {
            // 垂直标记（直角符号）
            painter.line_segment(
                [egui::Pos2::new(screen.x - size, screen.y), egui::Pos2::new(screen.x, screen.y)],
                stroke,
            );
            painter.line_segment(
                [egui::Pos2::new(screen.x, screen.y), egui::Pos2::new(screen.x, screen.y + size)],
                stroke,
            );
        }
        SnapType::Tangent => {
            // 切点标记（圆+线）
            painter.circle_stroke(screen, size * 0.6, stroke);
            painter.line_segment(
                [egui::Pos2::new(screen.x - size, screen.y + size), egui::Pos2::new(screen.x + size, screen.y - size)],
                stroke,
            );
        }
        SnapType::Nearest => {
            // 最近点标记（沙漏形）
            let half = size * 0.7;
            painter.line_segment(
                [egui::Pos2::new(screen.x - half, screen.y - size), egui::Pos2::new(screen.x + half, screen.y - size)],
                stroke,
            );
            painter.line_segment(
                [egui::Pos2::new(screen.x - half, screen.y - size), egui::Pos2::new(screen.x + half, screen.y + size)],
                stroke,
            );
            painter.line_segment(
                [egui::Pos2::new(screen.x + half, screen.y - size), egui::Pos2::new(screen.x - half, screen.y + size)],
                stroke,
            );
            painter.line_segment(
                [egui::Pos2::new(screen.x - half, screen.y + size), egui::Pos2::new(screen.x + half, screen.y + size)],
                stroke,
            );
        }
        SnapType::Grid => {
            // 网格点标记（小+形）
            let small = size * 0.5;
            painter.line_segment(
                [egui::Pos2::new(screen.x - small, screen.y), egui::Pos2::new(screen.x + small, screen.y)],
                stroke,
            );
            painter.line_segment(
                [egui::Pos2::new(screen.x, screen.y - small), egui::Pos2::new(screen.x, screen.y + small)],
                stroke,
            );
        }
        SnapType::Quadrant => {
            // 象限点标记（菱形）
            let points = [
                egui::Pos2::new(screen.x, screen.y - size),
                egui::Pos2::new(screen.x + size, screen.y),
                egui::Pos2::new(screen.x, screen.y + size),
                egui::Pos2::new(screen.x - size, screen.y),
            ];
            painter.add(egui::Shape::closed_line(points.to_vec(), stroke));
        }
    }
}

/// 绘制正交辅助线
pub fn draw_ortho_guides(ctx: &RenderContext, reference: Point2) {
    let screen = ctx.world_to_screen(reference);
    let guide_color = egui::Color32::from_rgba_unmultiplied(0, 255, 255, 80); // 半透明青色
    let stroke = egui::Stroke::new(1.0, guide_color);
    let painter = ctx.painter;
    let rect = ctx.rect;

    // 水平辅助线
    painter.line_segment(
        [egui::Pos2::new(rect.left(), screen.y), egui::Pos2::new(rect.right(), screen.y)],
        stroke,
    );

    // 垂直辅助线
    painter.line_segment(
        [egui::Pos2::new(screen.x, rect.top()), egui::Pos2::new(screen.x, rect.bottom())],
        stroke,
    );
}
