//! 布局系统（Layout System）
//!
//! 实现 AutoCAD 风格的模型空间/图纸空间设计：
//! - **模型空间（Model Space）**：1:1 比例绘制实际几何图形
//! - **图纸空间（Paper Space/Layout）**：用于打印输出的虚拟图纸
//! - **视口（Viewport）**：在图纸空间中显示模型空间内容的"窗口"
//!
//! # 架构设计
//!
//! ```text
//! Document
//! ├── Model Space (entities)     <- 所有几何图形存放在这里
//! └── Layouts[]
//!     ├── Layout1
//!     │   ├── Paper Settings (A3, 横向...)
//!     │   ├── Paper Space Entities (图框、标题栏...)
//!     │   └── Viewports[]
//!     │       ├── Viewport1 (比例 1:100, 显示整体)
//!     │       └── Viewport2 (比例 1:10, 显示详图)
//!     └── Layout2
//!         └── ...
//! ```

use crate::entity::{Entity, EntityId};
use crate::math::{Point2, Vector2};
use serde::{Deserialize, Serialize};

/// 布局 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayoutId(pub u64);

impl LayoutId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 视口 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewportId(pub u64);

impl ViewportId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 纸张大小（用于布局）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PaperSize {
    /// A4 (210 x 297 mm)
    A4,
    /// A3 (297 x 420 mm)
    A3,
    /// A2 (420 x 594 mm)
    A2,
    /// A1 (594 x 841 mm)
    A1,
    /// A0 (841 x 1189 mm)
    A0,
    /// Letter (8.5 x 11 in)
    Letter,
    /// Legal (8.5 x 14 in)
    Legal,
    /// Tabloid (11 x 17 in)
    Tabloid,
    /// 自定义尺寸 (宽, 高) mm
    Custom { width: f64, height: f64 },
}

impl Default for PaperSize {
    fn default() -> Self {
        PaperSize::A3
    }
}

impl PaperSize {
    /// 获取纸张尺寸（毫米）
    pub fn dimensions_mm(&self) -> (f64, f64) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::A3 => (297.0, 420.0),
            PaperSize::A2 => (420.0, 594.0),
            PaperSize::A1 => (594.0, 841.0),
            PaperSize::A0 => (841.0, 1189.0),
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::Legal => (215.9, 355.6),
            PaperSize::Tabloid => (279.4, 431.8),
            PaperSize::Custom { width, height } => (*width, *height),
        }
    }

    /// 获取纸张名称
    pub fn name(&self) -> &'static str {
        match self {
            PaperSize::A4 => "A4",
            PaperSize::A3 => "A3",
            PaperSize::A2 => "A2",
            PaperSize::A1 => "A1",
            PaperSize::A0 => "A0",
            PaperSize::Letter => "Letter",
            PaperSize::Legal => "Legal",
            PaperSize::Tabloid => "Tabloid",
            PaperSize::Custom { .. } => "Custom",
        }
    }
}

/// 纸张方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaperOrientation {
    /// 纵向（竖放）
    Portrait,
    /// 横向（横放）
    #[default]
    Landscape,
}

/// 视口状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ViewportStatus {
    /// 激活（可编辑模型空间）
    Active,
    /// 非激活（正常显示）
    #[default]
    Inactive,
    /// 锁定（不能修改视口设置）
    Locked,
    /// 隐藏
    Hidden,
}

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

/// 布局（Layout）
/// 
/// 代表一张虚拟图纸，用于打印输出
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layout {
    /// 布局 ID
    pub id: LayoutId,
    
    /// 布局名称
    pub name: String,
    
    // ===== 纸张设置 =====
    /// 纸张大小
    pub paper_size: PaperSize,
    /// 纸张方向
    pub orientation: PaperOrientation,
    /// 边距 (上, 右, 下, 左) mm
    pub margins: (f64, f64, f64, f64),
    
    // ===== 视口 =====
    /// 视口列表
    pub viewports: Vec<Viewport>,
    /// 下一个视口 ID
    next_viewport_id: u64,
    
    // ===== 图纸空间实体 =====
    /// 图纸空间的实体（图框、标题栏、注释等）
    /// 这些实体只属于此布局，不在模型空间中
    pub paper_space_entities: Vec<Entity>,
    
    // ===== 打印设置 =====
    /// 打印比例（图纸单位:打印单位）
    pub plot_scale: f64,
    /// 打印区域偏移
    pub plot_offset: (f64, f64),
    /// 是否居中打印
    pub center_plot: bool,
}

impl Layout {
    /// 创建新布局
    pub fn new(id: LayoutId, name: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            paper_size: PaperSize::A3,
            orientation: PaperOrientation::Landscape,
            margins: (10.0, 10.0, 10.0, 10.0),
            viewports: Vec::new(),
            next_viewport_id: 1,
            paper_space_entities: Vec::new(),
            plot_scale: 1.0,
            plot_offset: (0.0, 0.0),
            center_plot: true,
        }
    }

    /// 获取纸张实际尺寸（考虑方向）
    pub fn paper_dimensions(&self) -> (f64, f64) {
        let (w, h) = self.paper_size.dimensions_mm();
        match self.orientation {
            PaperOrientation::Portrait => (w, h),
            PaperOrientation::Landscape => (h, w),
        }
    }

    /// 获取可打印区域尺寸
    pub fn printable_area(&self) -> (f64, f64) {
        let (w, h) = self.paper_dimensions();
        let (top, right, bottom, left) = self.margins;
        (w - left - right, h - top - bottom)
    }

    /// 获取可打印区域的边界
    pub fn printable_bounds(&self) -> (Point2, Point2) {
        let (w, h) = self.paper_dimensions();
        let (top, right, bottom, left) = self.margins;
        (
            Point2::new(left, bottom),
            Point2::new(w - right, h - top),
        )
    }

    /// 添加视口
    pub fn add_viewport(&mut self, position: Point2, width: f64, height: f64) -> ViewportId {
        let id = ViewportId::new(self.next_viewport_id);
        self.next_viewport_id += 1;
        
        let viewport = Viewport::new(id, position, width, height);
        self.viewports.push(viewport);
        id
    }

    /// 添加默认视口（填满可打印区域）
    pub fn add_default_viewport(&mut self) -> ViewportId {
        let (min, max) = self.printable_bounds();
        self.add_viewport(min, max.x - min.x, max.y - min.y)
    }

    /// 获取视口
    pub fn get_viewport(&self, id: ViewportId) -> Option<&Viewport> {
        self.viewports.iter().find(|v| v.id == id)
    }

    /// 获取视口（可变）
    pub fn get_viewport_mut(&mut self, id: ViewportId) -> Option<&mut Viewport> {
        self.viewports.iter_mut().find(|v| v.id == id)
    }

    /// 删除视口
    pub fn remove_viewport(&mut self, id: ViewportId) -> bool {
        if let Some(pos) = self.viewports.iter().position(|v| v.id == id) {
            self.viewports.remove(pos);
            true
        } else {
            false
        }
    }

    /// 查找包含指定点的视口
    pub fn viewport_at_point(&self, point: Point2) -> Option<&Viewport> {
        self.viewports.iter().find(|v| v.contains_paper_point(point))
    }

    /// 添加图纸空间实体
    pub fn add_paper_entity(&mut self, entity: Entity) {
        self.paper_space_entities.push(entity);
    }
}

/// 当前空间类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpaceType {
    /// 模型空间
    #[default]
    Model,
    /// 图纸空间（指定布局）
    Paper(LayoutId),
}

/// 布局管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutManager {
    /// 所有布局
    layouts: Vec<Layout>,
    /// 下一个布局 ID
    next_layout_id: u64,
    /// 当前空间
    current_space: SpaceType,
    /// 当前激活的视口（如果在图纸空间）
    active_viewport: Option<ViewportId>,
}

impl Default for LayoutManager {
    fn default() -> Self {
        Self::new()
    }
}

impl LayoutManager {
    /// 创建新的布局管理器（带默认 Layout1）
    pub fn new() -> Self {
        let mut manager = Self {
            layouts: Vec::new(),
            next_layout_id: 1,
            current_space: SpaceType::Model,
            active_viewport: None,
        };
        
        // 创建默认布局
        let mut layout1 = manager.create_layout("Layout1");
        layout1.add_default_viewport();
        manager.layouts.push(layout1);
        
        manager
    }

    /// 创建新布局
    pub fn create_layout(&mut self, name: &str) -> Layout {
        let id = LayoutId::new(self.next_layout_id);
        self.next_layout_id += 1;
        Layout::new(id, name)
    }

    /// 添加布局
    pub fn add_layout(&mut self, name: &str) -> LayoutId {
        let layout = self.create_layout(name);
        let id = layout.id;
        self.layouts.push(layout);
        id
    }

    /// 获取布局
    pub fn get_layout(&self, id: LayoutId) -> Option<&Layout> {
        self.layouts.iter().find(|l| l.id == id)
    }

    /// 获取布局（可变）
    pub fn get_layout_mut(&mut self, id: LayoutId) -> Option<&mut Layout> {
        self.layouts.iter_mut().find(|l| l.id == id)
    }

    /// 按名称获取布局
    pub fn get_layout_by_name(&self, name: &str) -> Option<&Layout> {
        self.layouts.iter().find(|l| l.name == name)
    }

    /// 删除布局
    pub fn remove_layout(&mut self, id: LayoutId) -> bool {
        // 不能删除最后一个布局
        if self.layouts.len() <= 1 {
            return false;
        }
        
        if let Some(pos) = self.layouts.iter().position(|l| l.id == id) {
            self.layouts.remove(pos);
            
            // 如果删除的是当前布局，切换到模型空间
            if self.current_space == SpaceType::Paper(id) {
                self.current_space = SpaceType::Model;
            }
            true
        } else {
            false
        }
    }

    /// 获取所有布局
    pub fn layouts(&self) -> &[Layout] {
        &self.layouts
    }

    /// 获取所有布局（可变）
    pub fn layouts_mut(&mut self) -> &mut [Layout] {
        &mut self.layouts
    }

    /// 获取布局名称列表
    pub fn layout_names(&self) -> Vec<&str> {
        self.layouts.iter().map(|l| l.name.as_str()).collect()
    }

    /// 获取当前空间类型
    pub fn current_space(&self) -> SpaceType {
        self.current_space
    }

    /// 切换到模型空间
    pub fn switch_to_model(&mut self) {
        self.current_space = SpaceType::Model;
        self.active_viewport = None;
    }

    /// 切换到指定布局
    pub fn switch_to_layout(&mut self, id: LayoutId) -> bool {
        if self.layouts.iter().any(|l| l.id == id) {
            self.current_space = SpaceType::Paper(id);
            self.active_viewport = None;
            true
        } else {
            false
        }
    }

    /// 按名称切换布局
    pub fn switch_to_layout_by_name(&mut self, name: &str) -> bool {
        if let Some(layout) = self.layouts.iter().find(|l| l.name == name) {
            self.current_space = SpaceType::Paper(layout.id);
            self.active_viewport = None;
            true
        } else if name == "Model" || name == "模型" {
            self.switch_to_model();
            true
        } else {
            false
        }
    }

    /// 获取当前布局（如果在图纸空间）
    pub fn current_layout(&self) -> Option<&Layout> {
        match self.current_space {
            SpaceType::Model => None,
            SpaceType::Paper(id) => self.get_layout(id),
        }
    }

    /// 获取当前布局（可变）
    pub fn current_layout_mut(&mut self) -> Option<&mut Layout> {
        match self.current_space {
            SpaceType::Model => None,
            SpaceType::Paper(id) => self.get_layout_mut(id),
        }
    }

    /// 是否在模型空间
    pub fn is_model_space(&self) -> bool {
        self.current_space == SpaceType::Model
    }

    /// 是否在图纸空间
    pub fn is_paper_space(&self) -> bool {
        matches!(self.current_space, SpaceType::Paper(_))
    }

    /// 激活视口（双击视口进入模型空间编辑）
    pub fn activate_viewport(&mut self, viewport_id: ViewportId) {
        self.active_viewport = Some(viewport_id);
    }

    /// 退出视口编辑
    pub fn deactivate_viewport(&mut self) {
        self.active_viewport = None;
    }

    /// 获取激活的视口
    pub fn active_viewport(&self) -> Option<ViewportId> {
        self.active_viewport
    }

    /// 重命名布局
    pub fn rename_layout(&mut self, id: LayoutId, new_name: &str) -> bool {
        // 检查名称是否已存在
        if self.layouts.iter().any(|l| l.name == new_name && l.id != id) {
            return false;
        }
        
        if let Some(layout) = self.get_layout_mut(id) {
            layout.name = new_name.to_string();
            true
        } else {
            false
        }
    }
}

/// 常用标准比例列表
pub const STANDARD_SCALES: &[(&str, f64)] = &[
    ("1:1", 1.0),
    ("1:2", 2.0),
    ("1:5", 5.0),
    ("1:10", 10.0),
    ("1:20", 20.0),
    ("1:25", 25.0),
    ("1:50", 50.0),
    ("1:100", 100.0),
    ("1:200", 200.0),
    ("1:500", 500.0),
    ("1:1000", 1000.0),
    ("2:1", 0.5),
    ("5:1", 0.2),
    ("10:1", 0.1),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_viewport_coordinate_transform() {
        let viewport = Viewport {
            id: ViewportId::new(1),
            name: "Test".to_string(),
            position: Point2::new(50.0, 50.0),
            width: 200.0,
            height: 150.0,
            view_center: Point2::new(5000.0, 3000.0),
            scale: 100.0,  // 1:100
            rotation: 0.0,
            status: ViewportStatus::Inactive,
            show_border: true,
            border_color: (0, 0, 0),
            frozen_layers: Vec::new(),
        };

        // 测试视图中心应该映射到视口中心
        let paper_point = viewport.model_to_paper(viewport.view_center);
        let expected_center = Point2::new(150.0, 125.0); // 50 + 200/2, 50 + 150/2
        assert!((paper_point.x - expected_center.x).abs() < 0.001);
        assert!((paper_point.y - expected_center.y).abs() < 0.001);

        // 测试逆变换
        let model_point = viewport.paper_to_model(expected_center);
        assert!((model_point.x - viewport.view_center.x).abs() < 0.001);
        assert!((model_point.y - viewport.view_center.y).abs() < 0.001);
    }

    #[test]
    fn test_layout_manager() {
        let mut manager = LayoutManager::new();
        
        // 应该有一个默认布局
        assert_eq!(manager.layouts().len(), 1);
        assert_eq!(manager.layouts()[0].name, "Layout1");
        
        // 默认在模型空间
        assert!(manager.is_model_space());
        
        // 添加新布局
        let id = manager.add_layout("Layout2");
        assert_eq!(manager.layouts().len(), 2);
        
        // 切换到布局
        manager.switch_to_layout(id);
        assert!(manager.is_paper_space());
        
        // 切换回模型空间
        manager.switch_to_model();
        assert!(manager.is_model_space());
    }

    #[test]
    fn test_paper_size() {
        let a3 = PaperSize::A3;
        assert_eq!(a3.dimensions_mm(), (297.0, 420.0));
        
        let layout = Layout::new(LayoutId::new(1), "Test");
        // A3 横向
        assert_eq!(layout.paper_dimensions(), (420.0, 297.0));
    }
}
