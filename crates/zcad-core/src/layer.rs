//! 图层管理
//!
//! 图层是CAD中组织实体的重要方式。

use crate::entity::EntityId;
use crate::properties::{Color, LineType, LineWeight};
use serde::{Deserialize, Serialize};

/// 图层定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Layer {
    /// 图层ID
    pub id: EntityId,

    /// 图层名称
    pub name: String,

    /// 图层颜色
    pub color: Color,

    /// 图层线型
    pub line_type: LineType,

    /// 图层线宽
    pub line_weight: LineWeight,

    /// 是否可见
    pub visible: bool,

    /// 是否锁定
    pub locked: bool,

    /// 是否冻结
    pub frozen: bool,

    /// 是否可打印
    pub plottable: bool,

    /// 描述
    pub description: String,
}

impl Layer {
    /// 创建新图层
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: EntityId::new(),
            name: name.into(),
            color: Color::WHITE,
            line_type: LineType::Continuous,
            line_weight: LineWeight::Default,
            visible: true,
            locked: false,
            frozen: false,
            plottable: true,
            description: String::new(),
        }
    }

    /// 默认图层（0层）
    pub fn default_layer() -> Self {
        Self {
            id: EntityId::from_raw(1, 0), // 固定ID
            name: "0".to_string(),
            color: Color::WHITE,
            line_type: LineType::Continuous,
            line_weight: LineWeight::Default,
            visible: true,
            locked: false,
            frozen: false,
            plottable: true,
            description: "Default layer".to_string(),
        }
    }

    /// 设置颜色
    pub fn with_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// 设置线型
    pub fn with_line_type(mut self, line_type: LineType) -> Self {
        self.line_type = line_type;
        self
    }

    /// 检查图层是否可编辑
    pub fn is_editable(&self) -> bool {
        !self.locked && !self.frozen
    }

    /// 检查图层上的实体是否应该显示
    pub fn should_display(&self) -> bool {
        self.visible && !self.frozen
    }
}

/// 图层管理器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerManager {
    /// 所有图层
    layers: Vec<Layer>,

    /// 当前活动图层的索引
    current_layer_index: usize,
}

impl LayerManager {
    /// 创建新的图层管理器
    pub fn new() -> Self {
        Self {
            layers: vec![Layer::default_layer()],
            current_layer_index: 0,
        }
    }

    /// 获取当前图层
    pub fn current_layer(&self) -> &Layer {
        &self.layers[self.current_layer_index]
    }

    /// 获取当前图层（可变）
    pub fn current_layer_mut(&mut self) -> &mut Layer {
        &mut self.layers[self.current_layer_index]
    }

    /// 设置当前图层
    pub fn set_current_layer(&mut self, name: &str) -> bool {
        if let Some(idx) = self.layers.iter().position(|l| l.name == name) {
            self.current_layer_index = idx;
            true
        } else {
            false
        }
    }

    /// 添加新图层
    pub fn add_layer(&mut self, layer: Layer) -> EntityId {
        let id = layer.id;
        self.layers.push(layer);
        id
    }

    /// 创建并添加新图层
    pub fn create_layer(&mut self, name: impl Into<String>) -> EntityId {
        let layer = Layer::new(name);
        self.add_layer(layer)
    }

    /// 获取图层（按名称）
    pub fn get_layer(&self, name: &str) -> Option<&Layer> {
        self.layers.iter().find(|l| l.name == name)
    }

    /// 获取图层（按ID）
    pub fn get_layer_by_id(&self, id: EntityId) -> Option<&Layer> {
        self.layers.iter().find(|l| l.id == id)
    }

    /// 获取图层（可变，按名称）
    pub fn get_layer_mut(&mut self, name: &str) -> Option<&mut Layer> {
        self.layers.iter_mut().find(|l| l.name == name)
    }

    /// 获取所有图层
    pub fn all_layers(&self) -> &[Layer] {
        &self.layers
    }

    /// 删除图层
    ///
    /// 注意：不能删除图层0和当前图层
    pub fn delete_layer(&mut self, name: &str) -> Result<(), LayerError> {
        if name == "0" {
            return Err(LayerError::CannotDeleteLayerZero);
        }

        if let Some(idx) = self.layers.iter().position(|l| l.name == name) {
            if idx == self.current_layer_index {
                return Err(LayerError::CannotDeleteCurrentLayer);
            }

            self.layers.remove(idx);

            // 调整当前图层索引
            if idx < self.current_layer_index {
                self.current_layer_index -= 1;
            }

            Ok(())
        } else {
            Err(LayerError::LayerNotFound(name.to_string()))
        }
    }

    /// 重命名图层
    pub fn rename_layer(&mut self, old_name: &str, new_name: &str) -> Result<(), LayerError> {
        if old_name == "0" {
            return Err(LayerError::CannotRenameLayerZero);
        }

        if self.layers.iter().any(|l| l.name == new_name) {
            return Err(LayerError::LayerAlreadyExists(new_name.to_string()));
        }

        if let Some(layer) = self.get_layer_mut(old_name) {
            layer.name = new_name.to_string();
            Ok(())
        } else {
            Err(LayerError::LayerNotFound(old_name.to_string()))
        }
    }

    /// 图层数量
    pub fn count(&self) -> usize {
        self.layers.len()
    }
}

impl Default for LayerManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 图层操作错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum LayerError {
    #[error("Cannot delete layer 0")]
    CannotDeleteLayerZero,

    #[error("Cannot delete current layer")]
    CannotDeleteCurrentLayer,

    #[error("Cannot rename layer 0")]
    CannotRenameLayerZero,

    #[error("Layer not found: {0}")]
    LayerNotFound(String),

    #[error("Layer already exists: {0}")]
    LayerAlreadyExists(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_manager() {
        let mut manager = LayerManager::new();

        assert_eq!(manager.count(), 1);
        assert_eq!(manager.current_layer().name, "0");

        manager.create_layer("Layer1");
        manager.create_layer("Layer2");

        assert_eq!(manager.count(), 3);
        assert!(manager.set_current_layer("Layer1"));
        assert_eq!(manager.current_layer().name, "Layer1");

        assert!(manager.delete_layer("Layer2").is_ok());
        assert_eq!(manager.count(), 2);
    }
}

