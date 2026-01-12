//! 布局（Layout）

use crate::entity::Entity;
use crate::math::Point2;
use serde::{Deserialize, Serialize};

use super::types::{LayoutId, PaperOrientation, PaperSize, ViewportId};
use super::viewport::Viewport;

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
