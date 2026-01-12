//! 相机/视图坐标变换模块

use eframe::egui;
use zcad_core::math::Point2;

/// 相机状态，用于视图变换
#[derive(Debug, Clone)]
pub struct Camera {
    /// 相机中心点（世界坐标）
    pub center: Point2,
    /// 缩放级别
    pub zoom: f64,
    /// 视口大小
    pub viewport_size: (f32, f32),
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            center: Point2::new(250.0, 100.0),
            zoom: 1.5,
            viewport_size: (800.0, 600.0),
        }
    }
}

impl Camera {
    /// 世界坐标转屏幕坐标
    pub fn world_to_screen(&self, point: Point2, rect: &egui::Rect) -> egui::Pos2 {
        let center = rect.center();
        let x = center.x + ((point.x - self.center.x) * self.zoom) as f32;
        let y = center.y - ((point.y - self.center.y) * self.zoom) as f32; // Y轴翻转
        egui::Pos2::new(x, y)
    }

    /// 屏幕坐标转世界坐标
    pub fn screen_to_world(&self, pos: egui::Pos2, rect: &egui::Rect) -> Point2 {
        let center = rect.center();
        let x = self.center.x + ((pos.x - center.x) as f64 / self.zoom);
        let y = self.center.y - ((pos.y - center.y) as f64 / self.zoom); // Y轴翻转
        Point2::new(x, y)
    }

    /// 处理滚轮缩放
    pub fn handle_scroll_zoom(&mut self, scroll_delta_y: f32, hover_pos: egui::Pos2, rect: &egui::Rect) {
        if scroll_delta_y.abs() > 0.0 {
            let zoom_factor = if scroll_delta_y > 0.0 { 1.1 } else { 0.9 };
            
            // 缩放时保持鼠标位置不变
            let world_before = self.screen_to_world(hover_pos, rect);
            self.zoom *= zoom_factor;
            self.zoom = self.zoom.clamp(0.01, 100.0);
            let world_after = self.screen_to_world(hover_pos, rect);
            self.center.x += world_before.x - world_after.x;
            self.center.y += world_before.y - world_after.y;
        }
    }

    /// 处理平移
    pub fn handle_pan(&mut self, delta: egui::Vec2) {
        self.center.x -= (delta.x as f64) / self.zoom;
        self.center.y += (delta.y as f64) / self.zoom;
    }

    /// 缩放到适合视图
    pub fn zoom_to_fit(&mut self, bounds: Option<&zcad_core::math::BoundingBox2>) {
        if let Some(bounds) = bounds {
            self.center = Point2::new(
                (bounds.min.x + bounds.max.x) / 2.0,
                (bounds.min.y + bounds.max.y) / 2.0,
            );
            
            let width = bounds.max.x - bounds.min.x;
            let height = bounds.max.y - bounds.min.y;
            
            let zoom_x = (self.viewport_size.0 as f64 - 100.0) / width.max(1.0);
            let zoom_y = (self.viewport_size.1 as f64 - 100.0) / height.max(1.0);
            
            self.zoom = zoom_x.min(zoom_y).clamp(0.01, 100.0);
        }
    }
}
