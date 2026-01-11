//! 块定义和块参照系统
//!
//! 块是一组实体的集合，可以被重复使用。
//! 块参照是块的一个实例，可以有自己的位置、旋转和缩放。

use crate::entity::Entity;
use crate::math::{Point2, Vector2};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 块 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(pub u64);

impl BlockId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 块定义
/// 
/// 块是一组实体的集合，定义在其自己的坐标系中
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// 块 ID
    pub id: BlockId,
    /// 块名称（必须唯一）
    pub name: String,
    /// 基点（插入点的参考）
    pub base_point: Point2,
    /// 块中的实体
    pub entities: Vec<Entity>,
    /// 块说明
    pub description: String,
    /// 是否是匿名块（用于 Hatch 等）
    pub is_anonymous: bool,
}

impl Block {
    /// 创建新块
    pub fn new(name: impl Into<String>, base_point: Point2) -> Self {
        static mut NEXT_ID: u64 = 1;
        let id = unsafe {
            let id = NEXT_ID;
            NEXT_ID += 1;
            id
        };
        
        Self {
            id: BlockId::new(id),
            name: name.into(),
            base_point,
            entities: Vec::new(),
            description: String::new(),
            is_anonymous: false,
        }
    }

    /// 添加实体到块
    pub fn add_entity(&mut self, entity: Entity) {
        self.entities.push(entity);
    }

    /// 添加多个实体
    pub fn add_entities(&mut self, entities: impl IntoIterator<Item = Entity>) {
        self.entities.extend(entities);
    }

    /// 获取实体数量
    pub fn entity_count(&self) -> usize {
        self.entities.len()
    }

    /// 设置说明
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// 标记为匿名块
    pub fn anonymous(mut self) -> Self {
        self.is_anonymous = true;
        self
    }
}

/// 块参照
/// 
/// 块参照是块定义的一个实例，可以有位置、旋转和缩放
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockReference {
    /// 参照的块名称
    pub block_name: String,
    /// 插入点
    pub insertion_point: Point2,
    /// X 方向缩放
    pub scale_x: f64,
    /// Y 方向缩放
    pub scale_y: f64,
    /// 旋转角度（弧度）
    pub rotation: f64,
    /// 列数（用于阵列插入）
    pub column_count: u32,
    /// 行数（用于阵列插入）
    pub row_count: u32,
    /// 列间距
    pub column_spacing: f64,
    /// 行间距
    pub row_spacing: f64,
}

impl BlockReference {
    /// 创建块参照
    pub fn new(block_name: impl Into<String>, insertion_point: Point2) -> Self {
        Self {
            block_name: block_name.into(),
            insertion_point,
            scale_x: 1.0,
            scale_y: 1.0,
            rotation: 0.0,
            column_count: 1,
            row_count: 1,
            column_spacing: 0.0,
            row_spacing: 0.0,
        }
    }

    /// 设置缩放
    pub fn with_scale(mut self, scale_x: f64, scale_y: f64) -> Self {
        self.scale_x = scale_x;
        self.scale_y = scale_y;
        self
    }

    /// 设置均匀缩放
    pub fn with_uniform_scale(mut self, scale: f64) -> Self {
        self.scale_x = scale;
        self.scale_y = scale;
        self
    }

    /// 设置旋转（弧度）
    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置旋转（度）
    pub fn with_rotation_degrees(mut self, degrees: f64) -> Self {
        self.rotation = degrees.to_radians();
        self
    }

    /// 设置阵列参数
    pub fn with_array(mut self, columns: u32, rows: u32, col_spacing: f64, row_spacing: f64) -> Self {
        self.column_count = columns.max(1);
        self.row_count = rows.max(1);
        self.column_spacing = col_spacing;
        self.row_spacing = row_spacing;
        self
    }

    /// 变换点（从块坐标到世界坐标）
    pub fn transform_point(&self, point: Point2, base_point: Point2) -> Point2 {
        // 相对于基点
        let relative = point - base_point;
        
        // 缩放
        let scaled = Vector2::new(relative.x * self.scale_x, relative.y * self.scale_y);
        
        // 旋转
        let cos_r = self.rotation.cos();
        let sin_r = self.rotation.sin();
        let rotated = Vector2::new(
            scaled.x * cos_r - scaled.y * sin_r,
            scaled.x * sin_r + scaled.y * cos_r,
        );
        
        // 平移到插入点
        Point2::new(
            self.insertion_point.x + rotated.x,
            self.insertion_point.y + rotated.y,
        )
    }

    /// 获取所有插入点（考虑阵列）
    pub fn all_insertion_points(&self) -> Vec<Point2> {
        let mut points = Vec::with_capacity((self.column_count * self.row_count) as usize);
        
        for col in 0..self.column_count {
            for row in 0..self.row_count {
                let offset = Vector2::new(
                    col as f64 * self.column_spacing,
                    row as f64 * self.row_spacing,
                );
                
                // 应用旋转到偏移
                let cos_r = self.rotation.cos();
                let sin_r = self.rotation.sin();
                let rotated_offset = Vector2::new(
                    offset.x * cos_r - offset.y * sin_r,
                    offset.x * sin_r + offset.y * cos_r,
                );
                
                points.push(Point2::new(
                    self.insertion_point.x + rotated_offset.x,
                    self.insertion_point.y + rotated_offset.y,
                ));
            }
        }
        
        points
    }
}

/// 块表 - 管理所有块定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTable {
    /// 块定义（按名称索引）
    blocks: HashMap<String, Block>,
}

impl BlockTable {
    /// 创建空的块表
    pub fn new() -> Self {
        Self {
            blocks: HashMap::new(),
        }
    }

    /// 添加块定义
    pub fn add_block(&mut self, block: Block) -> bool {
        if self.blocks.contains_key(&block.name) {
            false
        } else {
            self.blocks.insert(block.name.clone(), block);
            true
        }
    }

    /// 获取块定义
    pub fn get_block(&self, name: &str) -> Option<&Block> {
        self.blocks.get(name)
    }

    /// 获取块定义（可变）
    pub fn get_block_mut(&mut self, name: &str) -> Option<&mut Block> {
        self.blocks.get_mut(name)
    }

    /// 移除块定义
    pub fn remove_block(&mut self, name: &str) -> Option<Block> {
        self.blocks.remove(name)
    }

    /// 重命名块
    pub fn rename_block(&mut self, old_name: &str, new_name: &str) -> bool {
        if self.blocks.contains_key(new_name) {
            return false;
        }
        
        if let Some(mut block) = self.blocks.remove(old_name) {
            block.name = new_name.to_string();
            self.blocks.insert(new_name.to_string(), block);
            true
        } else {
            false
        }
    }

    /// 检查块是否存在
    pub fn contains(&self, name: &str) -> bool {
        self.blocks.contains_key(name)
    }

    /// 获取所有块名称
    pub fn block_names(&self) -> Vec<&str> {
        self.blocks.keys().map(|s| s.as_str()).collect()
    }

    /// 获取块数量
    pub fn block_count(&self) -> usize {
        self.blocks.len()
    }

    /// 迭代所有块
    pub fn iter(&self) -> impl Iterator<Item = &Block> {
        self.blocks.values()
    }
}

impl Default for BlockTable {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::{Geometry, Line};

    #[test]
    fn test_block_creation() {
        let mut block = Block::new("TestBlock", Point2::origin());
        
        let line = Line::new(Point2::new(0.0, 0.0), Point2::new(10.0, 10.0));
        block.add_entity(Entity::new(Geometry::Line(line)));
        
        assert_eq!(block.name, "TestBlock");
        assert_eq!(block.entity_count(), 1);
    }

    #[test]
    fn test_block_reference_transform() {
        let ref_ = BlockReference::new("Test", Point2::new(100.0, 100.0))
            .with_uniform_scale(2.0)
            .with_rotation_degrees(90.0);
        
        let base = Point2::origin();
        let point = Point2::new(10.0, 0.0);
        
        let transformed = ref_.transform_point(point, base);
        
        // 缩放 2x：(20, 0)
        // 旋转 90°：(0, 20)
        // 平移到 (100, 100)：(100, 120)
        assert!((transformed.x - 100.0).abs() < 0.001);
        assert!((transformed.y - 120.0).abs() < 0.001);
    }

    #[test]
    fn test_block_table() {
        let mut table = BlockTable::new();
        
        let block1 = Block::new("Block1", Point2::origin());
        let block2 = Block::new("Block2", Point2::new(10.0, 10.0));
        
        assert!(table.add_block(block1));
        assert!(table.add_block(block2));
        assert!(!table.add_block(Block::new("Block1", Point2::origin()))); // 重复名称
        
        assert_eq!(table.block_count(), 2);
        assert!(table.contains("Block1"));
        assert!(table.contains("Block2"));
    }
}
