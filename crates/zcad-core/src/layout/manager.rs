//! 布局管理器

use serde::{Deserialize, Serialize};

use super::layout::Layout;
use super::types::{LayoutId, SpaceType, ViewportId};

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Point2;
    use crate::layout::{PaperSize, Viewport, ViewportStatus};

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
