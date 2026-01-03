//! 2D相机
//!
//! 处理平移、缩放和视口变换。

use crate::vertex::CameraUniform;
use zcad_core::math::{BoundingBox2, Point2, Vector2};

/// 2D相机
#[derive(Debug, Clone)]
pub struct Camera2D {
    /// 相机中心位置（世界坐标）
    pub center: Point2,

    /// 缩放级别（像素/单位）
    pub zoom: f64,

    /// 视口宽度（像素）
    pub viewport_width: u32,

    /// 视口高度（像素）
    pub viewport_height: u32,

    /// 最小缩放
    pub min_zoom: f64,

    /// 最大缩放
    pub max_zoom: f64,
}

impl Camera2D {
    /// 创建新的相机
    pub fn new(viewport_width: u32, viewport_height: u32) -> Self {
        Self {
            center: Point2::origin(),
            zoom: 1.0,
            viewport_width,
            viewport_height,
            min_zoom: 0.001,
            max_zoom: 10000.0,
        }
    }

    /// 更新视口大小
    pub fn set_viewport(&mut self, width: u32, height: u32) {
        self.viewport_width = width;
        self.viewport_height = height;
    }

    /// 平移相机
    pub fn pan(&mut self, delta: Vector2) {
        self.center += delta / self.zoom;
    }

    /// 缩放相机（以指定屏幕点为中心）
    pub fn zoom_at(&mut self, screen_point: Point2, factor: f64) {
        let world_before = self.screen_to_world(screen_point);

        self.zoom = (self.zoom * factor).clamp(self.min_zoom, self.max_zoom);

        let world_after = self.screen_to_world(screen_point);
        self.center += world_before - world_after;
    }

    /// 缩放到指定区域
    pub fn zoom_to_fit(&mut self, bbox: &BoundingBox2, padding: f64) {
        let width = bbox.width() + padding * 2.0;
        let height = bbox.height() + padding * 2.0;

        let zoom_x = self.viewport_width as f64 / width;
        let zoom_y = self.viewport_height as f64 / height;

        self.zoom = zoom_x.min(zoom_y).clamp(self.min_zoom, self.max_zoom);
        self.center = bbox.center();
    }

    /// 屏幕坐标转世界坐标
    pub fn screen_to_world(&self, screen: Point2) -> Point2 {
        let x = (screen.x - self.viewport_width as f64 / 2.0) / self.zoom + self.center.x;
        let y = (self.viewport_height as f64 / 2.0 - screen.y) / self.zoom + self.center.y;
        Point2::new(x, y)
    }

    /// 世界坐标转屏幕坐标
    pub fn world_to_screen(&self, world: Point2) -> Point2 {
        let x = (world.x - self.center.x) * self.zoom + self.viewport_width as f64 / 2.0;
        let y = self.viewport_height as f64 / 2.0 - (world.y - self.center.y) * self.zoom;
        Point2::new(x, y)
    }

    /// 获取当前可见的世界区域
    pub fn visible_bounds(&self) -> BoundingBox2 {
        let half_width = self.viewport_width as f64 / 2.0 / self.zoom;
        let half_height = self.viewport_height as f64 / 2.0 / self.zoom;

        BoundingBox2::new(
            Point2::new(self.center.x - half_width, self.center.y - half_height),
            Point2::new(self.center.x + half_width, self.center.y + half_height),
        )
    }

    /// 获取视图投影矩阵
    pub fn view_projection_matrix(&self) -> [[f32; 4]; 4] {
        let scale_x = 2.0 * self.zoom / self.viewport_width as f64;
        let scale_y = 2.0 * self.zoom / self.viewport_height as f64;
        let tx = -self.center.x * scale_x;
        let ty = -self.center.y * scale_y;

        [
            [scale_x as f32, 0.0, 0.0, 0.0],
            [0.0, scale_y as f32, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [tx as f32, ty as f32, 0.0, 1.0],
        ]
    }

    /// 获取相机Uniform数据
    pub fn to_uniform(&self) -> CameraUniform {
        CameraUniform {
            view_proj: self.view_projection_matrix(),
        }
    }

    /// 重置相机到原点
    pub fn reset(&mut self) {
        self.center = Point2::origin();
        self.zoom = 1.0;
    }

    /// 获取当前单位像素比（1单位对应多少像素）
    pub fn units_per_pixel(&self) -> f64 {
        1.0 / self.zoom
    }
}

impl Default for Camera2D {
    fn default() -> Self {
        Self::new(800, 600)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zcad_core::math::approx_eq;

    #[test]
    fn test_coordinate_conversion() {
        let camera = Camera2D::new(800, 600);

        let world = Point2::new(100.0, 50.0);
        let screen = camera.world_to_screen(world);
        let back = camera.screen_to_world(screen);

        assert!(approx_eq(world.x, back.x));
        assert!(approx_eq(world.y, back.y));
    }
}

