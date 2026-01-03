//! 实体标识和管理
//!
//! 采用生成式ID设计，支持撤销/重做时的实体复用。

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

/// 全局实体ID生成器
static ENTITY_COUNTER: AtomicU64 = AtomicU64::new(1);

/// 实体唯一标识符
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId {
    /// 唯一ID
    pub id: u64,
    /// 代数（用于撤销/重做时区分同一ID的不同版本）
    pub generation: u32,
}

impl EntityId {
    /// 创建新的实体ID
    pub fn new() -> Self {
        Self {
            id: ENTITY_COUNTER.fetch_add(1, Ordering::Relaxed),
            generation: 0,
        }
    }

    /// 从指定值创建（用于文件加载）
    pub fn from_raw(id: u64, generation: u32) -> Self {
        Self { id, generation }
    }

    /// 空ID（无效）
    pub const NULL: EntityId = EntityId {
        id: 0,
        generation: 0,
    };

    /// 检查是否为空ID
    pub fn is_null(&self) -> bool {
        self.id == 0
    }
}

impl Default for EntityId {
    fn default() -> Self {
        Self::new()
    }
}

/// CAD实体
///
/// 一个实体包含几何数据和属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 唯一标识符
    pub id: EntityId,

    /// 几何类型和数据
    pub geometry: crate::geometry::Geometry,

    /// 视觉属性
    pub properties: crate::properties::Properties,

    /// 所属图层ID
    pub layer_id: EntityId,

    /// 是否可见
    pub visible: bool,

    /// 是否锁定（不可编辑）
    pub locked: bool,
}

impl Entity {
    /// 创建新实体
    pub fn new(geometry: crate::geometry::Geometry) -> Self {
        Self {
            id: EntityId::new(),
            geometry,
            properties: crate::properties::Properties::default(),
            layer_id: EntityId::NULL,
            visible: true,
            locked: false,
        }
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> crate::math::BoundingBox2 {
        self.geometry.bounding_box()
    }

    /// 使用指定的图层
    pub fn with_layer(mut self, layer_id: EntityId) -> Self {
        self.layer_id = layer_id;
        self
    }

    /// 使用指定的属性
    pub fn with_properties(mut self, properties: crate::properties::Properties) -> Self {
        self.properties = properties;
        self
    }
}

