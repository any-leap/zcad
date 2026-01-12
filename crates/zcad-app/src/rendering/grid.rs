//! 网格绘制

use eframe::egui;
use zcad_core::math::Point2;

use super::RenderContext;

/// 绘制网格
pub fn draw_grid(ctx: &RenderContext, show_grid: bool) {
    if !show_grid {
        return;
    }

    let painter = ctx.painter;
    let rect = ctx.rect;

    // 根据缩放级别调整网格间距
    let mut spacing = 50.0;
    while spacing * ctx.camera_zoom < 20.0 {
        spacing *= 5.0;
    }
    while spacing * ctx.camera_zoom > 200.0 {
        spacing /= 5.0;
    }

    // 计算可见范围
    let top_left = screen_to_world(rect.left_top(), rect, ctx.camera_center, ctx.camera_zoom);
    let bottom_right = screen_to_world(rect.right_bottom(), rect, ctx.camera_center, ctx.camera_zoom);

    let start_x = (top_left.x / spacing).floor() * spacing;
    let end_x = (bottom_right.x / spacing).ceil() * spacing;
    let start_y = (bottom_right.y / spacing).floor() * spacing;
    let end_y = (top_left.y / spacing).ceil() * spacing;

    let grid_color = egui::Color32::from_rgb(50, 50, 60);
    let axis_color = egui::Color32::from_rgb(80, 80, 100);

    // 绘制垂直线
    let mut x = start_x;
    while x <= end_x {
        let screen_x = ctx.world_to_screen(Point2::new(x, 0.0)).x;
        if screen_x >= rect.left() && screen_x <= rect.right() {
            let color = if x.abs() < 0.001 { axis_color } else { grid_color };
            painter.line_segment(
                [egui::Pos2::new(screen_x, rect.top()), egui::Pos2::new(screen_x, rect.bottom())],
                egui::Stroke::new(1.0, color),
            );
        }
        x += spacing;
    }

    // 绘制水平线
    let mut y = start_y;
    while y <= end_y {
        let screen_y = ctx.world_to_screen(Point2::new(0.0, y)).y;
        if screen_y >= rect.top() && screen_y <= rect.bottom() {
            let color = if y.abs() < 0.001 { axis_color } else { grid_color };
            painter.line_segment(
                [egui::Pos2::new(rect.left(), screen_y), egui::Pos2::new(rect.right(), screen_y)],
                egui::Stroke::new(1.0, color),
            );
        }
        y += spacing;
    }
}

/// 屏幕坐标转世界坐标（辅助函数）
fn screen_to_world(pos: egui::Pos2, rect: &egui::Rect, camera_center: Point2, camera_zoom: f64) -> Point2 {
    let center = rect.center();
    let x = camera_center.x + ((pos.x - center.x) as f64 / camera_zoom);
    let y = camera_center.y - ((pos.y - center.y) as f64 / camera_zoom);
    Point2::new(x, y)
}
