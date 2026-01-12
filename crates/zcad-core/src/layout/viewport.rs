//! 视口（Viewport）

use crate::math::{Point2, Vector2};
use serde::{Deserialize, Serialize};

use super::types::{ViewportId, ViewportStatus};

/// 视口（Viewport）
/// 
/// 在图纸空间中显示模型空间内容的"窗口"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    /// 视口 ID
    pub id: ViewportId,
    
    /// 视口名称
    pub name: String,
    
    // ===== 图纸空间中的位置和大小 =====
    /// 视口在图纸空间的左下角位置（mm）
    pub position: Point2,
    /// 视口在图纸空间的宽度（mm）
    pub width: f64,
    /// 视口在图纸空间的高度（mm）
    pub height: f64,
    
    // ===== 模型空间的视图设置 =====
    /// 模型空间的中心点（视口显示的中心位置）
    pub view_center: Point2,
    /// 视图比例（1:scale，例如 100 表示 1:100）
    pub scale: f64,
    /// 视图旋转角度（弧度）
    pub rotation: f64,
    
    // ===== 视口状态 =====
    /// 视口状态
    pub status: ViewportStatus,
    /// 是否显示边框
    pub show_border: bool,
    /// 边框颜色
    pub border_color: (u8, u8, u8),
    
    // ===== 图层可见性（可选）=====
    /// 冻结的图层列表（在此视口中不显示）
    pub frozen_layers: Vec<String>,
}

impl Viewport {
    /// 创建新视口
    pub fn new(id: ViewportId, position: Point2, width: f64, height: f64) -> Self {
        Self {
            id,
            name: format!("Viewport{}", id.0),
            position,
            width,
            height,
            view_center: Point2::origin(),
            scale: 1.0,
            rotation: 0.0,
            status: ViewportStatus::Inactive,
            show_border: true,
            border_color: (0, 0, 0),
            frozen_layers: Vec::new(),
        }
    }

    /// 设置标准比例
    pub fn set_scale(&mut self, scale: f64) {
        self.scale = scale;
    }

    /// 设置常用比例
    pub fn set_standard_scale(&mut self, scale_str: &str) {
        self.scale = match scale_str {
            "1:1" => 1.0,
            "1:2" => 2.0,
            "1:5" => 5.0,
            "1:10" => 10.0,
            "1:20" => 20.0,
            "1:25" => 25.0,
            "1:50" => 50.0,
            "1:100" => 100.0,
            "1:200" => 200.0,
            "1:500" => 500.0,
            "1:1000" => 1000.0,
            "2:1" => 0.5,
            "5:1" => 0.2,
            "10:1" => 0.1,
            _ => 1.0,
        };
    }

    /// 获取视口在图纸空间的边界框
    pub fn paper_bounds(&self) -> (Point2, Point2) {
        let min = self.position;
        let max = Point2::new(self.position.x + self.width, self.position.y + self.height);
        (min, max)
    }

    /// 获取视口显示的模型空间范围
    pub fn model_bounds(&self) -> (Point2, Point2) {
        // 视口在图纸上的尺寸 * 比例 = 模型空间的范围
        let model_width = self.width * self.scale;
        let model_height = self.height * self.scale;
        
        let min = Point2::new(
            self.view_center.x - model_width / 2.0,
            self.view_center.y - model_height / 2.0,
        );
        let max = Point2::new(
            self.view_center.x + model_width / 2.0,
            self.view_center.y + model_height / 2.0,
        );
        (min, max)
    }

    /// 将模型空间坐标转换为图纸空间坐标
    pub fn model_to_paper(&self, model_point: Point2) -> Point2 {
        // 1. 相对于视图中心的偏移
        let offset = model_point - self.view_center;
        
        // 2. 应用比例
        let scaled = offset / self.scale;
        
        // 3. 应用旋转
        let rotated = if self.rotation.abs() > 1e-10 {
            let cos_r = self.rotation.cos();
            let sin_r = self.rotation.sin();
            Vector2::new(
                scaled.x * cos_r - scaled.y * sin_r,
                scaled.x * sin_r + scaled.y * cos_r,
            )
        } else {
            scaled
        };
        
        // 4. 转换到视口中心
        let viewport_center = Point2::new(
            self.position.x + self.width / 2.0,
            self.position.y + self.height / 2.0,
        );
        
        viewport_center + rotated
    }

    /// 将图纸空间坐标转换为模型空间坐标
    pub fn paper_to_model(&self, paper_point: Point2) -> Point2 {
        // 视口中心
        let viewport_center = Point2::new(
            self.position.x + self.width / 2.0,
            self.position.y + self.height / 2.0,
        );
        
        // 1. 相对于视口中心的偏移
        let offset = paper_point - viewport_center;
        
        // 2. 逆旋转
        let rotated = if self.rotation.abs() > 1e-10 {
            let cos_r = (-self.rotation).cos();
            let sin_r = (-self.rotation).sin();
            Vector2::new(
                offset.x * cos_r - offset.y * sin_r,
                offset.x * sin_r + offset.y * cos_r,
            )
        } else {
            offset
        };
        
        // 3. 逆比例
        let unscaled = rotated * self.scale;
        
        // 4. 加上视图中心
        self.view_center + unscaled
    }

    /// 检查图纸空间的点是否在视口内
    pub fn contains_paper_point(&self, point: Point2) -> bool {
        point.x >= self.position.x
            && point.x <= self.position.x + self.width
            && point.y >= self.position.y
            && point.y <= self.position.y + self.height
    }

    /// 缩放以适应指定的模型空间范围
    pub fn zoom_to_fit(&mut self, model_min: Point2, model_max: Point2) {
        let model_width = model_max.x - model_min.x;
        let model_height = model_max.y - model_min.y;
        
        // 计算需要的比例
        let scale_x = model_width / self.width;
        let scale_y = model_height / self.height;
        
        // 使用较大的比例（确保完全显示）
        self.scale = scale_x.max(scale_y) * 1.1; // 留 10% 边距
        
        // 设置视图中心
        self.view_center = Point2::new(
            (model_min.x + model_max.x) / 2.0,
            (model_min.y + model_max.y) / 2.0,
        );
    }
}
