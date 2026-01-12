//! 撤销/重做操作模块

use zcad_core::entity::{Entity, EntityContent, EntityId};
use zcad_core::history::{HistoryTree, OperationType, operations as hist_ops};
use zcad_file::Document;

use crate::ui_state::UiStateManager;

/// 历史记录最大深度
pub const HISTORY_MAX_DEPTH: usize = 500;

/// 历史操作管理器
pub struct HistoryOperations {
    /// 历史树
    pub history: HistoryTree,
}

impl Default for HistoryOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl HistoryOperations {
    pub fn new() -> Self {
        Self {
            history: HistoryTree::new(HISTORY_MAX_DEPTH),
        }
    }

    /// 删除选中的实体（带撤销支持）
    pub fn delete_selected_entities<U: UiStateManager>(
        &mut self,
        document: &mut Document,
        ui_state: &mut U,
        selected_ids: &[EntityId],
    ) {
        if selected_ids.is_empty() {
            return;
        }
        
        // 使用分组操作记录多个删除
        let mut operations = Vec::new();
        for id in selected_ids {
            if let Some(entity) = document.remove_entity(id) {
                let op = hist_ops::delete_entity(*id, Some(entity), "删除实体");
                operations.push(op);
            }
        }
        
        if !operations.is_empty() {
            let count = operations.len();
            // 如果只有一个操作，直接添加；否则使用分组操作
            if operations.len() == 1 {
                let _ = self.history.add_operation(operations.remove(0));
            } else {
                let group_op = hist_ops::group_operation(
                    "批量删除",
                    operations,
                    format!("删除 {} 个实体", count),
                );
                let _ = self.history.add_operation(group_op);
            }
            ui_state.set_status_message(format!("已删除 {} 个实体", count));
        }
        ui_state.clear_selection();
    }

    /// 添加实体并记录历史（用于创建操作）
    pub fn add_entity_with_history(
        &mut self,
        document: &mut Document,
        entity: Entity,
        description: &str,
    ) -> EntityId {
        let id = document.add_entity(entity.clone());
        let op = hist_ops::create_entity(entity, description);
        let _ = self.history.add_operation(op);
        id
    }

    /// 执行撤销操作
    pub fn do_undo<U: UiStateManager>(&mut self, document: &mut Document, ui_state: &mut U) {
        // 先获取操作并克隆，避免借用问题
        let op_type = self.history.undo().map(|op| (op.operation_type.clone(), op.description.clone()));
        if let Some((op_type, desc)) = op_type {
            self.apply_undo_operation(document, &op_type);
            ui_state.set_status_message(format!("撤销: {}", desc));
        } else {
            ui_state.set_status_message("没有可撤销的操作".to_string());
        }
    }

    /// 执行重做操作
    pub fn do_redo<U: UiStateManager>(&mut self, document: &mut Document, ui_state: &mut U) {
        // 先获取操作并克隆，避免借用问题
        let op_type = self.history.redo().map(|op| (op.operation_type.clone(), op.description.clone()));
        if let Some((op_type, desc)) = op_type {
            self.apply_redo_operation(document, &op_type);
            ui_state.set_status_message(format!("重做: {}", desc));
        } else {
            ui_state.set_status_message("没有可重做的操作".to_string());
        }
    }

    /// 应用撤销操作（反向执行）
    fn apply_undo_operation(&self, document: &mut Document, op_type: &OperationType) {
        match op_type {
            OperationType::CreateEntity { entity } => {
                // 撤销创建：删除实体
                document.remove_entity(&entity.id);
            }
            OperationType::DeleteEntity { previous_entity, .. } => {
                // 撤销删除：恢复实体
                if let Some(entity) = previous_entity {
                    document.add_entity(entity.clone());
                }
            }
            OperationType::ModifyEntity { entity_id, previous_geometry, .. } => {
                // 撤销修改：恢复到之前的几何
                if let Some(entity) = document.get_entity(entity_id) {
                    let mut restored = entity.clone();
                    restored.content = EntityContent::Geometry(previous_geometry.clone());
                    document.update_entity(entity_id, restored);
                }
            }
            OperationType::MoveEntities { .. } => {
                // TODO: 移动操作的撤销需要额外的几何体变换支持
            }
            OperationType::RotateEntities { .. } => {
                // TODO: 旋转操作的撤销需要额外的几何体变换支持
            }
            OperationType::ScaleEntities { .. } => {
                // TODO: 缩放操作的撤销需要额外的几何体变换支持
            }
            OperationType::GroupOperation { operations, .. } => {
                // 反向撤销分组中的所有操作
                for op in operations.iter().rev() {
                    self.apply_undo_operation(document, &op.operation_type);
                }
            }
            _ => {
                // 其他操作类型暂不支持
            }
        }
    }

    /// 应用重做操作（正向执行）
    fn apply_redo_operation(&self, document: &mut Document, op_type: &OperationType) {
        match op_type {
            OperationType::CreateEntity { entity } => {
                // 重做创建：添加实体
                document.add_entity(entity.clone());
            }
            OperationType::DeleteEntity { entity_id, .. } => {
                // 重做删除：删除实体
                document.remove_entity(entity_id);
            }
            OperationType::ModifyEntity { entity_id, new_geometry, .. } => {
                // 重做修改：应用新几何
                if let Some(entity) = document.get_entity(entity_id) {
                    let mut modified = entity.clone();
                    modified.content = EntityContent::Geometry(new_geometry.clone());
                    document.update_entity(entity_id, modified);
                }
            }
            OperationType::MoveEntities { .. } => {
                // TODO: 移动操作的重做需要额外的几何体变换支持
            }
            OperationType::RotateEntities { .. } => {
                // TODO: 旋转操作的重做需要额外的几何体变换支持
            }
            OperationType::ScaleEntities { .. } => {
                // TODO: 缩放操作的重做需要额外的几何体变换支持
            }
            OperationType::GroupOperation { operations, .. } => {
                // 正向重做分组中的所有操作
                for op in operations {
                    self.apply_redo_operation(document, &op.operation_type);
                }
            }
            _ => {
                // 其他操作类型暂不支持
            }
        }
    }
}
