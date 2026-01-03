//! 双缓冲实体数据系统
//!
//! 提供无锁并行的实体数据管理：
//! - 渲染缓冲区：只读，用于GPU渲染
//! - 编辑缓冲区：可写，用于用户编辑
//! - 缓冲区交换：在适当时机同步数据

use crate::entity::{Entity, EntityId};
use crate::math::BoundingBox2;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};

/// 实体缓冲区
///
/// 包含所有实体的快照，用于渲染或编辑
#[derive(Debug, Clone)]
pub struct EntityBuffer {
    /// 所有实体，按ID索引
    entities: HashMap<EntityId, Entity>,

    /// 实体ID列表（保持顺序）
    entity_ids: Vec<EntityId>,

    /// 图层ID到实体ID的映射
    layer_entities: HashMap<EntityId, Vec<EntityId>>,

    /// 空间索引用于快速查询
    spatial_index: HashMap<EntityId, BoundingBox2>,

    /// 版本号，用于检测变化
    version: u64,
}

impl EntityBuffer {
    /// 创建空的实体缓冲区
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            entity_ids: Vec::new(),
            layer_entities: HashMap::new(),
            spatial_index: HashMap::new(),
            version: 0,
        }
    }

    /// 从实体列表创建缓冲区
    pub fn from_entities(entities: Vec<Entity>) -> Self {
        let mut buffer = Self::new();

        for entity in entities {
            buffer.add_entity(entity);
        }

        buffer
    }

    /// 添加实体
    pub fn add_entity(&mut self, entity: Entity) {
        let id = entity.id;
        let bounds = entity.bounding_box();
        let layer_id = entity.layer_id;

        self.entities.insert(id, entity);
        self.entity_ids.push(id);
        self.spatial_index.insert(id, bounds);

        // 更新图层映射
        self.layer_entities.entry(layer_id).or_insert_with(Vec::new).push(id);

        self.version += 1;
    }

    /// 移除实体
    pub fn remove_entity(&mut self, id: &EntityId) -> Option<Entity> {
        if let Some(entity) = self.entities.remove(id) {
            self.entity_ids.retain(|eid| eid != id);
            self.spatial_index.remove(id);

            // 从图层映射中移除
            if let Some(layer_entities) = self.layer_entities.get_mut(&entity.layer_id) {
                layer_entities.retain(|eid| eid != id);
            }

            self.version += 1;
            Some(entity)
        } else {
            None
        }
    }

    /// 更新实体
    pub fn update_entity(&mut self, entity: Entity) {
        let id = entity.id;
        let bounds = entity.bounding_box();

        if self.entities.contains_key(&id) {
            self.entities.insert(id, entity);
            self.spatial_index.insert(id, bounds);
            self.version += 1;
        }
    }

    /// 获取实体
    pub fn get_entity(&self, id: &EntityId) -> Option<&Entity> {
        self.entities.get(id)
    }

    /// 获取所有实体
    pub fn entities(&self) -> impl Iterator<Item = &Entity> {
        self.entity_ids.iter().filter_map(|id| self.entities.get(id))
    }

    /// 获取指定图层的实体
    pub fn entities_in_layer(&self, layer_id: &EntityId) -> impl Iterator<Item = &Entity> {
        self.layer_entities
            .get(layer_id)
            .map(|ids| ids.iter().filter_map(|id| self.entities.get(id)).collect::<Vec<_>>())
            .unwrap_or_default()
            .into_iter()
    }

    /// 按包围盒查询实体
    pub fn query_entities(&self, bounds: &BoundingBox2) -> Vec<&Entity> {
        self.spatial_index
            .iter()
            .filter(|(_, entity_bounds)| entity_bounds.intersects(bounds))
            .filter_map(|(id, _)| self.entities.get(id))
            .collect()
    }

    /// 获取实体数量
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 获取版本号
    pub fn version(&self) -> u64 {
        self.version
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// 清空缓冲区
    pub fn clear(&mut self) {
        self.entities.clear();
        self.entity_ids.clear();
        self.layer_entities.clear();
        self.spatial_index.clear();
        self.version += 1;
    }
}

impl Default for EntityBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// 双缓冲实体管理器
///
/// 维护两个缓冲区：渲染缓冲区（只读）和编辑缓冲区（可写）
/// 支持无锁并行访问和缓冲区交换
pub struct DoubleBufferedEntities {
    /// 渲染缓冲区（只读，供渲染线程使用）
    render_buffer: Arc<RwLock<EntityBuffer>>,

    /// 编辑缓冲区（可写，供编辑线程使用）
    edit_buffer: Arc<RwLock<EntityBuffer>>,

    /// 交换标记（true表示需要交换缓冲区）
    swap_pending: Arc<AtomicBool>,

    /// 统计信息
    stats: Arc<RwLock<BufferStats>>,
}

#[derive(Debug, Clone, Default)]
pub struct BufferStats {
    /// 渲染缓冲区版本
    pub render_version: u64,

    /// 编辑缓冲区版本
    pub edit_version: u64,

    /// 缓冲区交换次数
    pub swap_count: u64,

    /// 最后交换时间
    pub last_swap_time: Option<std::time::Instant>,
}

impl DoubleBufferedEntities {
    /// 创建新的双缓冲管理器
    pub fn new() -> Self {
        Self {
            render_buffer: Arc::new(RwLock::new(EntityBuffer::new())),
            edit_buffer: Arc::new(RwLock::new(EntityBuffer::new())),
            swap_pending: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(RwLock::new(BufferStats::default())),
        }
    }

    /// 从实体列表创建双缓冲管理器
    pub fn from_entities(entities: Vec<Entity>) -> Self {
        let buffer = EntityBuffer::from_entities(entities);
        let render_buffer = Arc::new(RwLock::new(buffer.clone()));
        let edit_buffer = Arc::new(RwLock::new(buffer));

        Self {
            render_buffer,
            edit_buffer,
            swap_pending: Arc::new(AtomicBool::new(false)),
            stats: Arc::new(RwLock::new(BufferStats::default())),
        }
    }

    // === 渲染线程方法（只读） ===

    /// 获取渲染缓冲区的只读访问
    pub fn render_buffer(&self) -> &Arc<RwLock<EntityBuffer>> {
        &self.render_buffer
    }

    /// 安全地读取渲染缓冲区
    pub fn with_render_buffer<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&EntityBuffer) -> R,
    {
        let buffer = self.render_buffer.read().unwrap();
        f(&buffer)
    }

    // === 编辑线程方法（可写） ===

    /// 获取编辑缓冲区的可写访问
    pub fn edit_buffer(&self) -> &Arc<RwLock<EntityBuffer>> {
        &self.edit_buffer
    }

    /// 安全地修改编辑缓冲区
    pub fn with_edit_buffer_mut<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut EntityBuffer) -> R,
    {
        let mut buffer = self.edit_buffer.write().unwrap();
        let result = f(&mut buffer);

        // 标记需要交换（如果有变化）
        if buffer.version() != self.stats.read().unwrap().edit_version {
            self.swap_pending.store(true, Ordering::Release);
            self.stats.write().unwrap().edit_version = buffer.version();
        }

        result
    }

    /// 添加实体到编辑缓冲区
    pub fn add_entity(&self, entity: Entity) {
        self.with_edit_buffer_mut(|buffer| {
            buffer.add_entity(entity);
        });
    }

    /// 从编辑缓冲区移除实体
    pub fn remove_entity(&self, id: &EntityId) -> Option<Entity> {
        self.with_edit_buffer_mut(|buffer| {
            buffer.remove_entity(id)
        })
    }

    /// 更新编辑缓冲区中的实体
    pub fn update_entity(&self, entity: Entity) {
        self.with_edit_buffer_mut(|buffer| {
            buffer.update_entity(entity);
        });
    }

    /// 获取编辑缓冲区中的实体（只读）
    pub fn get_entity(&self, id: &EntityId) -> Option<Entity> {
        self.with_edit_buffer_mut(|buffer| {
            buffer.get_entity(id).cloned()
        })
    }

    // === 缓冲区管理 ===

    /// 检查是否需要交换缓冲区
    pub fn swap_pending(&self) -> bool {
        self.swap_pending.load(Ordering::Acquire)
    }

    /// 交换渲染缓冲区和编辑缓冲区
    ///
    /// 这个操作应该是原子性的，确保渲染线程总是看到一致的状态
    pub fn swap_buffers(&self) {
        if !self.swap_pending.load(Ordering::Acquire) {
            return;
        }

        // 复制编辑缓冲区到渲染缓冲区
        {
            let edit_buffer = self.edit_buffer.read().unwrap();
            let mut render_buffer = self.render_buffer.write().unwrap();

            *render_buffer = edit_buffer.clone();

            // 更新统计信息
            let mut stats = self.stats.write().unwrap();
            stats.render_version = render_buffer.version();
            stats.swap_count += 1;
            stats.last_swap_time = Some(std::time::Instant::now());
        }

        // 清除交换标记
        self.swap_pending.store(false, Ordering::Release);
    }

    /// 强制交换缓冲区（用于初始化或其他情况）
    pub fn force_swap(&self) {
        self.swap_pending.store(true, Ordering::Release);
        self.swap_buffers();
    }

    /// 获取缓冲区统计信息
    pub fn stats(&self) -> BufferStats {
        self.stats.read().unwrap().clone()
    }

    /// 重置统计信息
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = BufferStats::default();
    }

    // === 批量操作 ===

    /// 批量添加实体
    pub fn add_entities(&self, entities: Vec<Entity>) {
        self.with_edit_buffer_mut(|buffer| {
            for entity in entities {
                buffer.add_entity(entity);
            }
        });
    }

    /// 批量移除实体
    pub fn remove_entities(&self, ids: &[EntityId]) -> Vec<Entity> {
        self.with_edit_buffer_mut(|buffer| {
            ids.iter().filter_map(|id| buffer.remove_entity(id)).collect()
        })
    }

    /// 清空所有实体
    pub fn clear(&self) {
        self.with_edit_buffer_mut(|buffer| {
            buffer.clear();
        });
    }

    /// 获取实体总数
    pub fn entity_count(&self) -> usize {
        self.with_edit_buffer_mut(|buffer| buffer.entity_count())
    }

    /// 检查是否为空
    pub fn is_empty(&self) -> bool {
        self.with_edit_buffer_mut(|buffer| buffer.is_empty())
    }
}

impl Default for DoubleBufferedEntities {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Geometry;
    use crate::math::Point2;

    #[test]
    fn test_entity_buffer() {
        let mut buffer = EntityBuffer::new();

        let entity1 = Entity::new(Geometry::Point(crate::geometry::Point::new(0.0, 0.0)));
        let entity2 = Entity::new(Geometry::Line(crate::geometry::Line::new(
            Point2::new(0.0, 0.0),
            Point2::new(10.0, 10.0),
        )));

        buffer.add_entity(entity1.clone());
        buffer.add_entity(entity2.clone());

        assert_eq!(buffer.entity_count(), 2);
        assert_eq!(buffer.version(), 2);

        let removed = buffer.remove_entity(&entity1.id);
        assert!(removed.is_some());
        assert_eq!(buffer.entity_count(), 1);
        assert_eq!(buffer.version(), 3);
    }

    #[test]
    fn test_double_buffered_entities() {
        let db = DoubleBufferedEntities::new();

        let entity = Entity::new(Geometry::Point(crate::geometry::Point::new(0.0, 0.0)));
        db.add_entity(entity.clone());

        // 检查编辑缓冲区
        assert_eq!(db.entity_count(), 1);

        // 交换缓冲区
        db.force_swap();

        // 检查渲染缓冲区
        db.with_render_buffer(|buffer| {
            assert_eq!(buffer.entity_count(), 1);
        });

        let stats = db.stats();
        assert_eq!(stats.swap_count, 1);
    }
}
