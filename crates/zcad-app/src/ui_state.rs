//! UI 状态管理 trait
//! 
//! 提供一个抽象接口，避免模块之间的循环依赖

use zcad_core::entity::EntityId;

/// UI 状态管理接口
pub trait UiStateManager {
    /// 清除选择
    fn clear_selection(&mut self);
    
    /// 设置状态消息
    fn set_status_message(&mut self, message: String);
    
    /// 添加到选择集
    fn add_to_selection(&mut self, id: EntityId);
}
