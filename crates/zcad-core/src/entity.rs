//! 实体标识和管理
//!
//! ZCAD 核心数据模型：Entity as Object
//!
//! 设计哲学：
//! - 几何是**载体**，数据和关系才是**核心**
//! - 每个实体不只是形状，而是**带属性的对象**
//! - 支持层级结构（parent/children）
//! - 开放属性系统（PropertyMap）
//! - 实体间关系（Relations）

use crate::geometry::Geometry;
use crate::math::{Point2, Vector2};
use crate::properties::Properties;
use crate::transform::Transform2D;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
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

// ============================================================================
// 属性系统 (Property System)
// ============================================================================

/// 属性值类型
///
/// 支持多种数据类型，实现开放式属性系统
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PropertyValue {
    /// 字符串值
    String(String),
    /// 数值
    Number(f64),
    /// 布尔值
    Bool(bool),
    /// 点坐标
    Point(Point2),
    /// 引用另一个实体
    EntityRef(EntityId),
    /// 值列表
    List(Vec<PropertyValue>),
    /// 键值映射
    Map(HashMap<String, PropertyValue>),
}

impl PropertyValue {
    /// 尝试获取字符串值
    pub fn as_string(&self) -> Option<&str> {
        match self {
            PropertyValue::String(s) => Some(s),
            _ => None,
        }
    }

    /// 尝试获取数值
    pub fn as_number(&self) -> Option<f64> {
        match self {
            PropertyValue::Number(n) => Some(*n),
            _ => None,
        }
    }

    /// 尝试获取布尔值
    pub fn as_bool(&self) -> Option<bool> {
        match self {
            PropertyValue::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// 尝试获取实体引用
    pub fn as_entity_ref(&self) -> Option<EntityId> {
        match self {
            PropertyValue::EntityRef(id) => Some(*id),
            _ => None,
        }
    }
}

impl From<String> for PropertyValue {
    fn from(s: String) -> Self {
        PropertyValue::String(s)
    }
}

impl From<&str> for PropertyValue {
    fn from(s: &str) -> Self {
        PropertyValue::String(s.to_string())
    }
}

impl From<f64> for PropertyValue {
    fn from(n: f64) -> Self {
        PropertyValue::Number(n)
    }
}

impl From<i32> for PropertyValue {
    fn from(n: i32) -> Self {
        PropertyValue::Number(n as f64)
    }
}

impl From<bool> for PropertyValue {
    fn from(b: bool) -> Self {
        PropertyValue::Bool(b)
    }
}

impl From<EntityId> for PropertyValue {
    fn from(id: EntityId) -> Self {
        PropertyValue::EntityRef(id)
    }
}

/// 实体属性表（开放式，可扩展）
pub type PropertyMap = HashMap<String, PropertyValue>;

// ============================================================================
// 实体内容类型 (Entity Content)
// ============================================================================

/// 实体内容类型
///
/// 统一几何实体、块定义、块参照、外部参照、组等概念
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityContent {
    /// 几何实体（线、圆、弧等）
    Geometry(Geometry),

    /// 块定义（容器，存储子实体的模板）
    BlockDefinition {
        /// 块名称（必须唯一）
        name: String,
        /// 基点（插入点的参考）
        base_point: Point2,
        /// 块说明
        description: String,
    },

    /// 块参照（引用块定义，带变换）
    BlockReference {
        /// 指向 BlockDefinition 实体的 ID
        block_id: EntityId,
        /// 插入点
        insertion_point: Point2,
        /// 缩放
        scale: Vector2,
        /// 旋转角度（弧度）
        rotation: f64,
        /// 块属性值
        attributes: HashMap<String, String>,
    },

    /// 外部参照（懒加载的外部文档）
    ExternalReference {
        /// 外部文件路径
        file_path: PathBuf,
        /// 插入点
        insertion_point: Point2,
        /// 缩放
        scale: Vector2,
        /// 旋转角度（弧度）
        rotation: f64,
        /// 是否已加载
        loaded: bool,
    },

    /// 组（临时集合，不影响子实体的独立性）
    Group {
        /// 组名称（可选）
        name: Option<String>,
    },
}

impl EntityContent {
    /// 获取内容类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            EntityContent::Geometry(g) => g.type_name(),
            EntityContent::BlockDefinition { .. } => "BlockDefinition",
            EntityContent::BlockReference { .. } => "BlockReference",
            EntityContent::ExternalReference { .. } => "ExternalReference",
            EntityContent::Group { .. } => "Group",
        }
    }

    /// 是否是几何类型
    pub fn is_geometry(&self) -> bool {
        matches!(self, EntityContent::Geometry(_))
    }

    /// 是否是容器类型（可以有子实体）
    pub fn is_container(&self) -> bool {
        matches!(
            self,
            EntityContent::BlockDefinition { .. }
                | EntityContent::BlockReference { .. }
                | EntityContent::ExternalReference { .. }
                | EntityContent::Group { .. }
        )
    }

    /// 获取几何数据（如果是几何类型）
    pub fn as_geometry(&self) -> Option<&Geometry> {
        match self {
            EntityContent::Geometry(g) => Some(g),
            _ => None,
        }
    }

    /// 获取几何数据的可变引用
    pub fn as_geometry_mut(&mut self) -> Option<&mut Geometry> {
        match self {
            EntityContent::Geometry(g) => Some(g),
            _ => None,
        }
    }
}

impl From<Geometry> for EntityContent {
    fn from(geometry: Geometry) -> Self {
        EntityContent::Geometry(geometry)
    }
}

// ============================================================================
// 实体关系 (Relations)
// ============================================================================

/// 实体间关系
///
/// 表示实体之间的语义关系，如"属于"、"连接到"、"依赖于"等
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// 关系类型（如 "belongs_to", "connects_to", "part_of"）
    pub relation_type: String,
    /// 目标实体 ID
    pub target_id: EntityId,
    /// 关系元数据
    pub metadata: PropertyMap,
}

impl Relation {
    /// 创建新关系
    pub fn new(relation_type: impl Into<String>, target_id: EntityId) -> Self {
        Self {
            relation_type: relation_type.into(),
            target_id,
            metadata: HashMap::new(),
        }
    }

    /// 添加元数据
    pub fn with_metadata(mut self, key: impl Into<String>, value: PropertyValue) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

// ============================================================================
// Entity（核心实体结构）
// ============================================================================

/// CAD实体
///
/// ZCAD 的核心数据结构，支持：
/// - 多种内容类型（几何、块、参照等）
/// - 层级关系（parent/children）
/// - 开放属性系统（PropertyMap）
/// - 标签分类（tags）
/// - 实体间关系（relations）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// 唯一标识符
    pub id: EntityId,

    /// 实体内容（几何、块定义、块参照等）
    pub content: EntityContent,

    /// 视觉属性（颜色、线型、线宽）
    pub visual_properties: Properties,

    /// 所属图层ID
    pub layer_id: EntityId,

    // === 层级关系 ===
    /// 父实体 ID
    pub parent: Option<EntityId>,
    /// 子实体 ID 列表
    pub children: Vec<EntityId>,
    /// 局部变换（相对于父实体）
    pub local_transform: Transform2D,

    // === 开放属性系统 ===
    /// 任意键值属性
    pub properties: PropertyMap,
    /// 标签（用于分类/查询）
    pub tags: HashSet<String>,

    // === 关系 ===
    /// 与其他实体的关系
    pub relations: Vec<Relation>,

    // === 状态 ===
    /// 是否可见
    pub visible: bool,
    /// 是否锁定（不可编辑）
    pub locked: bool,
    /// 是否选中
    pub selected: bool,
}

impl Entity {
    /// 创建新的几何实体
    pub fn new(geometry: Geometry) -> Self {
        Self {
            id: EntityId::new(),
            content: EntityContent::Geometry(geometry),
            visual_properties: Properties::default(),
            layer_id: EntityId::NULL,
            parent: None,
            children: Vec::new(),
            local_transform: Transform2D::identity(),
            properties: HashMap::new(),
            tags: HashSet::new(),
            relations: Vec::new(),
            visible: true,
            locked: false,
            selected: false,
        }
    }

    /// 从内容创建实体
    pub fn from_content(content: EntityContent) -> Self {
        Self {
            id: EntityId::new(),
            content,
            visual_properties: Properties::default(),
            layer_id: EntityId::NULL,
            parent: None,
            children: Vec::new(),
            local_transform: Transform2D::identity(),
            properties: HashMap::new(),
            tags: HashSet::new(),
            relations: Vec::new(),
            visible: true,
            locked: false,
            selected: false,
        }
    }

    /// 创建块定义实体
    pub fn block_definition(name: impl Into<String>, base_point: Point2) -> Self {
        Self::from_content(EntityContent::BlockDefinition {
            name: name.into(),
            base_point,
            description: String::new(),
        })
    }

    /// 创建块参照实体
    pub fn block_reference(
        block_id: EntityId,
        insertion_point: Point2,
        scale: Vector2,
        rotation: f64,
    ) -> Self {
        let mut entity = Self::from_content(EntityContent::BlockReference {
            block_id,
            insertion_point,
            scale,
            rotation,
            attributes: HashMap::new(),
        });
        // 设置局部变换
        entity.local_transform = Transform2D::translation(insertion_point.x, insertion_point.y)
            .then(&Transform2D::rotation(rotation))
            .then(&Transform2D::scale(scale.x, scale.y));
        entity
    }

    /// 创建组实体
    pub fn group(name: Option<String>) -> Self {
        Self::from_content(EntityContent::Group { name })
    }

    // === 向后兼容的便捷方法 ===

    /// 获取几何数据（向后兼容）
    pub fn geometry(&self) -> Option<&Geometry> {
        self.content.as_geometry()
    }

    /// 获取几何数据的可变引用
    pub fn geometry_mut(&mut self) -> Option<&mut Geometry> {
        self.content.as_geometry_mut()
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> crate::math::BoundingBox2 {
        match &self.content {
            EntityContent::Geometry(g) => g.bounding_box(),
            _ => crate::math::BoundingBox2::empty(),
        }
    }

    // === Builder 方法 ===

    /// 使用指定的图层
    pub fn with_layer(mut self, layer_id: EntityId) -> Self {
        self.layer_id = layer_id;
        self
    }

    /// 使用指定的视觉属性
    pub fn with_properties(mut self, properties: Properties) -> Self {
        self.visual_properties = properties;
        self
    }

    /// 添加属性
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<PropertyValue>) -> Self {
        self.properties.insert(key.into(), value.into());
        self
    }

    /// 添加标签
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.insert(tag.into());
        self
    }

    /// 添加多个标签
    pub fn with_tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for tag in tags {
            self.tags.insert(tag.into());
        }
        self
    }

    /// 添加关系
    pub fn with_relation(mut self, relation: Relation) -> Self {
        self.relations.push(relation);
        self
    }

    /// 设置父实体
    pub fn with_parent(mut self, parent_id: EntityId) -> Self {
        self.parent = Some(parent_id);
        self
    }

    /// 设置局部变换
    pub fn with_transform(mut self, transform: Transform2D) -> Self {
        self.local_transform = transform;
        self
    }

    // === 属性操作 ===

    /// 获取属性值
    pub fn get_property(&self, key: &str) -> Option<&PropertyValue> {
        self.properties.get(key)
    }

    /// 设置属性值
    pub fn set_property(&mut self, key: impl Into<String>, value: impl Into<PropertyValue>) {
        self.properties.insert(key.into(), value.into());
    }

    /// 移除属性
    pub fn remove_property(&mut self, key: &str) -> Option<PropertyValue> {
        self.properties.remove(key)
    }

    /// 检查是否有指定标签
    pub fn has_tag(&self, tag: &str) -> bool {
        self.tags.contains(tag)
    }

    /// 添加标签
    pub fn add_tag(&mut self, tag: impl Into<String>) {
        self.tags.insert(tag.into());
    }

    /// 移除标签
    pub fn remove_tag(&mut self, tag: &str) -> bool {
        self.tags.remove(tag)
    }

    // === 关系操作 ===

    /// 添加关系
    pub fn add_relation(&mut self, relation: Relation) {
        self.relations.push(relation);
    }

    /// 获取指定类型的所有关系
    pub fn relations_of_type<'a>(&'a self, relation_type: &'a str) -> impl Iterator<Item = &'a Relation> + 'a {
        self.relations
            .iter()
            .filter(move |r| r.relation_type == relation_type)
    }

    /// 获取指定类型的所有目标实体 ID
    pub fn related_ids(&self, relation_type: &str) -> Vec<EntityId> {
        self.relations_of_type(relation_type)
            .map(|r| r.target_id)
            .collect()
    }
}

// ============================================================================
// 测试
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Line;
    use crate::math::Point2;

    #[test]
    fn test_entity_creation() {
        let line = Line::new(Point2::new(0.0, 0.0), Point2::new(10.0, 10.0));
        let entity = Entity::new(Geometry::Line(line));

        assert!(!entity.id.is_null());
        assert!(entity.content.is_geometry());
        assert!(entity.parent.is_none());
        assert!(entity.children.is_empty());
    }

    #[test]
    fn test_property_system() {
        let line = Line::new(Point2::new(0.0, 0.0), Point2::new(10.0, 10.0));
        let entity = Entity::new(Geometry::Line(line))
            .with_property("material", "steel")
            .with_property("thickness", 10.0)
            .with_tag("structural")
            .with_tag("load-bearing");

        assert_eq!(
            entity.get_property("material").and_then(|v| v.as_string()),
            Some("steel")
        );
        assert_eq!(
            entity.get_property("thickness").and_then(|v| v.as_number()),
            Some(10.0)
        );
        assert!(entity.has_tag("structural"));
        assert!(entity.has_tag("load-bearing"));
        assert!(!entity.has_tag("decorative"));
    }

    #[test]
    fn test_relations() {
        let room_id = EntityId::new();
        let line = Line::new(Point2::new(0.0, 0.0), Point2::new(10.0, 10.0));
        let entity = Entity::new(Geometry::Line(line))
            .with_relation(Relation::new("belongs_to", room_id))
            .with_relation(Relation::new("part_of", room_id));

        let belongs_to: Vec<_> = entity.relations_of_type("belongs_to").collect();
        assert_eq!(belongs_to.len(), 1);
        assert_eq!(belongs_to[0].target_id, room_id);

        assert_eq!(entity.related_ids("belongs_to"), vec![room_id]);
    }

    #[test]
    fn test_block_definition() {
        let block = Entity::block_definition("Door", Point2::new(0.0, 0.0));
        
        match &block.content {
            EntityContent::BlockDefinition { name, base_point, .. } => {
                assert_eq!(name, "Door");
                assert_eq!(*base_point, Point2::new(0.0, 0.0));
            }
            _ => panic!("Expected BlockDefinition"),
        }
    }

    #[test]
    fn test_block_reference() {
        let block_id = EntityId::new();
        let reference = Entity::block_reference(
            block_id,
            Point2::new(100.0, 200.0),
            Vector2::new(1.0, 1.0),
            0.0,
        );

        match &reference.content {
            EntityContent::BlockReference {
                block_id: ref_id,
                insertion_point,
                ..
            } => {
                assert_eq!(*ref_id, block_id);
                assert_eq!(*insertion_point, Point2::new(100.0, 200.0));
            }
            _ => panic!("Expected BlockReference"),
        }
    }
}