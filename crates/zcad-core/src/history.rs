//! 历史树系统
//!
//! 实现类似SolidWorks的特征历史管理，支持：
//! - 操作历史记录
//! - 撤销/重做
//! - 分支历史
//! - 操作依赖管理
//! - 历史压缩和优化

use crate::entity::{Entity, EntityId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 操作ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct OperationId(pub u64);

impl OperationId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    pub fn null() -> Self {
        Self(0)
    }

    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

/// 操作类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OperationType {
    /// 创建实体
    CreateEntity {
        entity: Entity,
    },

    /// 删除实体
    DeleteEntity {
        entity_id: EntityId,
        previous_entity: Option<Entity>,
    },

    /// 修改实体
    ModifyEntity {
        entity_id: EntityId,
        previous_geometry: crate::geometry::Geometry,
        new_geometry: crate::geometry::Geometry,
    },

    /// 移动实体
    MoveEntities {
        entity_ids: Vec<EntityId>,
        offset: crate::math::Vector2,
        previous_positions: Vec<crate::math::Point2>,
    },

    /// 旋转实体
    RotateEntities {
        entity_ids: Vec<EntityId>,
        center: crate::math::Point2,
        angle: f64,
        previous_angles: Vec<f64>,
    },

    /// 缩放实体
    ScaleEntities {
        entity_ids: Vec<EntityId>,
        center: crate::math::Point2,
        scale: f64,
        previous_scales: Vec<f64>,
    },

    /// 布尔运算
    BooleanOperation {
        operation: crate::parametric::BooleanOp,
        entity1: EntityId,
        entity2: EntityId,
        result_entities: Vec<Entity>,
        previous_entities: Vec<Entity>,
    },

    /// 添加约束
    AddConstraint {
        constraint: crate::parametric::Constraint,
    },

    /// 删除约束
    RemoveConstraint {
        constraint_id: crate::parametric::ConstraintId,
        previous_constraint: Option<crate::parametric::Constraint>,
    },

    /// 修改变量
    ModifyVariable {
        variable_id: crate::parametric::VariableId,
        previous_value: f64,
        new_value: f64,
    },

    /// 分组操作（复合操作）
    GroupOperation {
        name: String,
        operations: Vec<Operation>,
    },

    /// 自定义操作
    Custom {
        name: String,
        data: Vec<u8>,
    },
}

/// 操作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Operation {
    /// 操作ID
    pub id: OperationId,

    /// 操作类型
    pub operation_type: OperationType,

    /// 时间戳
    pub timestamp: std::time::SystemTime,

    /// 用户描述
    pub description: String,

    /// 是否可以撤销
    pub can_undo: bool,

    /// 依赖的操作ID
    pub dependencies: Vec<OperationId>,

    /// 受影响的实体ID
    pub affected_entities: Vec<EntityId>,
}

impl Operation {
    /// 创建新操作
    pub fn new(operation_type: OperationType, description: impl Into<String>) -> Self {
        static NEXT_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

        Self {
            id: OperationId(NEXT_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed)),
            operation_type,
            timestamp: std::time::SystemTime::now(),
            description: description.into(),
            can_undo: true,
            dependencies: Vec::new(),
            affected_entities: Vec::new(),
        }
    }

    /// 设置依赖
    pub fn with_dependencies(mut self, dependencies: Vec<OperationId>) -> Self {
        self.dependencies = dependencies;
        self
    }

    /// 设置受影响的实体
    pub fn with_affected_entities(mut self, entities: Vec<EntityId>) -> Self {
        self.affected_entities = entities;
        self
    }

    /// 设置是否可以撤销
    pub fn with_undo(mut self, can_undo: bool) -> Self {
        self.can_undo = can_undo;
        self
    }
}

/// 历史节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryNode {
    /// 节点ID
    pub id: OperationId,

    /// 操作
    pub operation: Operation,

    /// 父节点
    pub parent: Option<OperationId>,

    /// 子节点（分支）
    pub children: Vec<OperationId>,

    /// 节点深度（根节点为0）
    pub depth: usize,

    /// 是否是当前活动分支
    pub is_active: bool,
}

impl HistoryNode {
    pub fn new(operation: Operation, parent: Option<OperationId>, depth: usize) -> Self {
        Self {
            id: operation.id,
            operation,
            parent,
            children: Vec::new(),
            depth,
            is_active: true,
        }
    }
}

/// 历史树
///
/// 管理CAD操作的历史，支持撤销/重做和分支历史
pub struct HistoryTree {
    /// 所有历史节点
    nodes: HashMap<OperationId, HistoryNode>,

    /// 当前节点（撤销/重做位置）
    current_node: Option<OperationId>,

    /// 根节点
    root_node: Option<OperationId>,

    /// 操作栈（用于快速撤销/重做）
    undo_stack: Vec<OperationId>,
    redo_stack: Vec<OperationId>,

    /// 分支管理
    branches: HashMap<String, OperationId>,

    /// 统计信息
    stats: HistoryStats,

    /// 最大历史深度
    max_depth: usize,
}

#[derive(Debug, Clone, Default)]
pub struct HistoryStats {
    /// 总操作数
    pub total_operations: usize,

    /// 当前深度
    pub current_depth: usize,

    /// 分支数
    pub branch_count: usize,

    /// 压缩节省的空间
    pub compression_savings: usize,

    /// 最后操作时间
    pub last_operation_time: Option<std::time::SystemTime>,
}

impl HistoryTree {
    /// 创建新的历史树
    pub fn new(max_depth: usize) -> Self {
        Self {
            nodes: HashMap::new(),
            current_node: None,
            root_node: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            branches: HashMap::new(),
            stats: HistoryStats::default(),
            max_depth,
        }
    }

    /// 添加操作
    pub fn add_operation(&mut self, operation: Operation) -> Result<(), String> {
        let operation_id = operation.id;

        // 检查深度限制
        if self.stats.total_operations >= self.max_depth {
            self.compress_history()?;
        }

        // 创建新节点
        let parent = self.current_node;
        let depth = parent.map_or(0, |p| self.nodes[&p].depth + 1);

        let node = HistoryNode::new(operation.clone(), parent, depth);
        self.nodes.insert(operation_id, node.clone());

        // 更新父节点的子节点列表
        if let Some(parent_id) = parent {
            if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
                parent_node.children.push(operation_id);
            }
        } else {
            // 这是根节点
            self.root_node = Some(operation_id);
        }

        // 设置为当前节点
        self.set_current_node(Some(operation_id));

        // 添加到撤销栈
        self.undo_stack.push(operation_id);

        // 清空重做栈
        self.redo_stack.clear();

        // 更新统计
        self.stats.total_operations += 1;
        self.stats.current_depth = depth;
        self.stats.last_operation_time = Some(operation.timestamp);

        Ok(())
    }

    /// 撤销操作
    pub fn undo(&mut self) -> Option<&Operation> {
        if let Some(current_id) = self.current_node {
            if let Some(operation) = self.undo_stack.pop() {
                self.redo_stack.push(operation);
                self.set_current_node(self.nodes[&current_id].parent);
                return Some(&self.nodes[&operation].operation);
            }
        }
        None
    }

    /// 重做操作
    pub fn redo(&mut self) -> Option<&Operation> {
        if let Some(operation_id) = self.redo_stack.pop() {
            self.undo_stack.push(operation_id);
            self.set_current_node(Some(operation_id));
            return Some(&self.nodes[&operation_id].operation);
        }
        None
    }

    /// 跳转到指定操作
    pub fn goto_operation(&mut self, operation_id: OperationId) -> Result<(), String> {
        if !self.nodes.contains_key(&operation_id) {
            return Err(format!("Operation {:?} not found", operation_id));
        }

        // 重新构建撤销/重做栈
        self.rebuild_stacks(operation_id);
        self.set_current_node(Some(operation_id));

        Ok(())
    }

    /// 创建分支
    pub fn create_branch(&mut self, branch_name: String, from_operation: OperationId) -> Result<(), String> {
        if !self.nodes.contains_key(&from_operation) {
            return Err(format!("Operation {:?} not found", from_operation));
        }

        if self.branches.contains_key(&branch_name) {
            return Err(format!("Branch '{}' already exists", branch_name));
        }

        self.branches.insert(branch_name.clone(), from_operation);
        self.stats.branch_count += 1;

        Ok(())
    }

    /// 切换分支
    pub fn switch_branch(&mut self, branch_name: &str) -> Result<(), String> {
        if let Some(&operation_id) = self.branches.get(branch_name) {
            self.goto_operation(operation_id)?;
            Ok(())
        } else {
            Err(format!("Branch '{}' not found", branch_name))
        }
    }

    /// 获取当前操作序列
    pub fn current_operations(&self) -> Vec<&Operation> {
        let mut operations = Vec::new();
        let mut current = self.current_node;

        while let Some(node_id) = current {
            if let Some(node) = self.nodes.get(&node_id) {
                operations.push(&node.operation);
                current = node.parent;
            } else {
                break;
            }
        }

        operations.reverse();
        operations
    }

    /// 获取操作依赖图
    pub fn dependency_graph(&self) -> HashMap<OperationId, Vec<OperationId>> {
        let mut graph = HashMap::new();

        for node in self.nodes.values() {
            graph.insert(node.id, node.operation.dependencies.clone());

            // 添加隐式依赖（父子关系）
            if let Some(parent) = node.parent {
                graph.entry(node.id).or_insert_with(Vec::new).push(parent);
            }
        }

        graph
    }

    /// 压缩历史（合并相似操作）
    pub fn compress_history(&mut self) -> Result<(), String> {
        // 简化的压缩策略：合并连续的相同类型的操作
        let mut compressed = 0;
        let mut nodes_to_remove = Vec::new();

        // 首先收集所有节点信息
        let node_info: Vec<_> = self.nodes.iter().map(|(id, node)| (*id, node.parent)).collect();

        for (id, parent_id) in node_info {
            if let Some(parent_id) = parent_id {
                if let (Some(node), Some(parent)) = (self.nodes.get(&id), self.nodes.get(&parent_id)) {
                    if self.can_merge_operations(&node.operation, &parent.operation) {
                        // 合并操作
                        if let Some(merged) = self.merge_operations(&parent.operation, &node.operation) {
                            // 更新节点
                            if let Some(parent_node) = self.nodes.get_mut(&parent_id) {
                                parent_node.operation = merged;
                                nodes_to_remove.push(id);
                                compressed += 1;
                            }
                        }
                    }
                }
            }
        }

        // 移除合并的节点
        for id in nodes_to_remove {
            self.nodes.remove(&id);
        }

        self.stats.compression_savings += compressed;
        Ok(())
    }

    /// 检查两个操作是否可以合并
    fn can_merge_operations(&self, op1: &Operation, op2: &Operation) -> bool {
        match (&op1.operation_type, &op2.operation_type) {
            (OperationType::MoveEntities { .. }, OperationType::MoveEntities { .. }) => true,
            (OperationType::ModifyVariable { variable_id: id1, .. }, OperationType::ModifyVariable { variable_id: id2, .. }) => id1 == id2,
            _ => false,
        }
    }

    /// 合并两个操作
    fn merge_operations(&self, op1: &Operation, op2: &Operation) -> Option<Operation> {
        match (&op1.operation_type, &op2.operation_type) {
            (OperationType::MoveEntities { entity_ids: ids1, offset: offset1, .. },
             OperationType::MoveEntities { entity_ids: ids2, offset: offset2, .. }) => {
                if ids1 == ids2 {
                    Some(Operation::new(
                        OperationType::MoveEntities {
                            entity_ids: ids1.clone(),
                            offset: *offset1 + *offset2,
                            previous_positions: Vec::new(), // 合并时简化处理
                        },
                        format!("Merged move operations: {} + {}", op1.description, op2.description),
                    ))
                } else {
                    None
                }
            }
            (OperationType::ModifyVariable { variable_id, previous_value, .. },
             OperationType::ModifyVariable { new_value, .. }) => {
                Some(Operation::new(
                    OperationType::ModifyVariable {
                        variable_id: *variable_id,
                        previous_value: *previous_value,
                        new_value: *new_value,
                    },
                    format!("Merged variable modifications: {} -> {}", op1.description, op2.description),
                ))
            }
            _ => None,
        }
    }

    /// 设置当前节点
    fn set_current_node(&mut self, node_id: Option<OperationId>) {
        // 取消之前当前节点的活动状态
        if let Some(current) = self.current_node {
            if let Some(node) = self.nodes.get_mut(&current) {
                node.is_active = false;
            }
        }

        self.current_node = node_id;

        // 设置新当前节点的活动状态
        if let Some(current) = node_id {
            if let Some(node) = self.nodes.get_mut(&current) {
                node.is_active = true;
            }
        }
    }

    /// 重新构建撤销/重做栈
    fn rebuild_stacks(&mut self, target_operation: OperationId) {
        self.undo_stack.clear();
        self.redo_stack.clear();

        // 从目标操作到根的路径
        let mut path = Vec::new();
        let mut current = Some(target_operation);

        while let Some(node_id) = current {
            path.push(node_id);
            if let Some(node) = self.nodes.get(&node_id) {
                current = node.parent;
            } else {
                break;
            }
        }

        path.reverse();
        self.undo_stack = path;
    }

    /// 获取统计信息
    pub fn stats(&self) -> &HistoryStats {
        &self.stats
    }

    /// 获取分支列表
    pub fn branches(&self) -> &HashMap<String, OperationId> {
        &self.branches
    }

    /// 查找操作
    pub fn find_operation(&self, id: &OperationId) -> Option<&Operation> {
        self.nodes.get(id).map(|node| &node.operation)
    }

    /// 获取操作树的可视化表示
    pub fn tree_string(&self) -> String {
        let mut result = String::new();
        self.build_tree_string(self.root_node, 0, &mut result);
        result
    }

    fn build_tree_string(&self, node_id: Option<OperationId>, depth: usize, result: &mut String) {
        if let Some(id) = node_id {
            if let Some(node) = self.nodes.get(&id) {
                let indent = "  ".repeat(depth);
                let marker = if node.is_active { "* " } else { "  " };
                result.push_str(&format!("{}{}{}: {}\n", indent, marker, id.0, node.operation.description));

                for child_id in &node.children {
                    self.build_tree_string(Some(*child_id), depth + 1, result);
                }
            }
        }
    }
}

impl Default for HistoryTree {
    fn default() -> Self {
        Self::new(1000) // 默认最大1000个操作
    }
}

/// 操作构造器
pub mod operations {
    use super::*;

    /// 创建实体操作
    pub fn create_entity(entity: Entity, description: impl Into<String>) -> Operation {
        Operation::new(
            OperationType::CreateEntity { entity },
            description,
        )
    }

    /// 删除实体操作
    pub fn delete_entity(entity_id: EntityId, previous_entity: Option<Entity>, description: impl Into<String>) -> Operation {
        Operation::new(
            OperationType::DeleteEntity {
                entity_id,
                previous_entity,
            },
            description,
        )
    }

    /// 修改实体操作
    pub fn modify_entity(
        entity_id: EntityId,
        previous_geometry: crate::geometry::Geometry,
        new_geometry: crate::geometry::Geometry,
        description: impl Into<String>,
    ) -> Operation {
        Operation::new(
            OperationType::ModifyEntity {
                entity_id,
                previous_geometry,
                new_geometry,
            },
            description,
        )
    }

    /// 移动实体操作
    pub fn move_entities(
        entity_ids: Vec<EntityId>,
        offset: crate::math::Vector2,
        previous_positions: Vec<crate::math::Point2>,
        description: impl Into<String>,
    ) -> Operation {
        Operation::new(
            OperationType::MoveEntities {
                entity_ids,
                offset,
                previous_positions,
            },
            description,
        )
    }

    /// 布尔运算操作
    pub fn boolean_operation(
        operation: crate::parametric::BooleanOp,
        entity1: EntityId,
        entity2: EntityId,
        result_entities: Vec<Entity>,
        previous_entities: Vec<Entity>,
        description: impl Into<String>,
    ) -> Operation {
        Operation::new(
            OperationType::BooleanOperation {
                operation,
                entity1,
                entity2,
                result_entities,
                previous_entities,
            },
            description,
        )
    }

    /// 分组操作
    pub fn group_operation(name: impl Into<String>, operations: Vec<Operation>, description: impl Into<String>) -> Operation {
        Operation::new(
            OperationType::GroupOperation {
                name: name.into(),
                operations,
            },
            description,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Geometry;
    use crate::math::Point2;

    #[test]
    fn test_history_tree() {
        let mut history = HistoryTree::new(100);

        // 添加一些操作
        let op1 = operations::create_entity(
            Entity::new(Geometry::Point(crate::geometry::Point::new(0.0, 0.0))),
            "Create point",
        );
        history.add_operation(op1).unwrap();

        let op2 = operations::create_entity(
            Entity::new(Geometry::Line(crate::geometry::Line::new(
                Point2::new(0.0, 0.0),
                Point2::new(10.0, 10.0),
            ))),
            "Create line",
        );
        history.add_operation(op2).unwrap();

        // 检查当前操作
        let current_ops = history.current_operations();
        assert_eq!(current_ops.len(), 2);

        // 撤销
        let undone = history.undo();
        assert!(undone.is_some());
        assert_eq!(history.current_operations().len(), 1);

        // 重做
        let redone = history.redo();
        assert!(redone.is_some());
        assert_eq!(history.current_operations().len(), 2);

        // 统计
        let stats = history.stats();
        assert_eq!(stats.total_operations, 2);
        assert_eq!(stats.current_depth, 1);
    }
}
