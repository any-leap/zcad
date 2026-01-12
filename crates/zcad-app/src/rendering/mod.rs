//! 渲染模块 - 所有绘制逻辑

mod grid;
mod geometry;
mod cursor;
mod preview;
mod dimension;

pub use grid::draw_grid;
pub use geometry::draw_geometry;
pub use cursor::{draw_crosshair, draw_snap_marker, draw_ortho_guides};
pub use preview::draw_preview;

use eframe::egui;
use zcad_core::math::Point2;

/// 渲染上下文，包含绘制所需的所有信息
pub struct RenderContext<'a> {
    pub painter: &'a egui::Painter,
    pub rect: &'a egui::Rect,
    pub camera_center: Point2,
    pub camera_zoom: f64,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        painter: &'a egui::Painter,
        rect: &'a egui::Rect,
        camera_center: Point2,
        camera_zoom: f64,
    ) -> Self {
        Self {
            painter,
            rect,
            camera_center,
            camera_zoom,
        }
    }

    /// 世界坐标转屏幕坐标
    pub fn world_to_screen(&self, point: Point2) -> egui::Pos2 {
        let center = self.rect.center();
        let x = center.x + ((point.x - self.camera_center.x) * self.camera_zoom) as f32;
        let y = center.y - ((point.y - self.camera_center.y) * self.camera_zoom) as f32;
        egui::Pos2::new(x, y)
    }
}
