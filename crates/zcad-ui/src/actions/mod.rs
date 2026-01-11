//! 具体的 Action 实现
//!
//! 每个绘图/编辑工具对应一个 Action 实现

mod draw_line;
mod draw_circle;
mod draw_arc;
mod draw_polyline;
mod draw_rectangle;
mod draw_point;
mod draw_ellipse;
mod select;
mod modify_move;
mod modify_copy;
mod modify_rotate;
mod modify_scale;
mod modify_mirror;
mod modify_offset;
mod modify_trim;
mod modify_extend;
mod modify_fillet;
mod modify_chamfer;
mod grip_edit;

pub use draw_line::DrawLineAction;
pub use draw_circle::DrawCircleAction;
pub use draw_arc::DrawArcAction;
pub use draw_polyline::DrawPolylineAction;
pub use draw_rectangle::DrawRectangleAction;
pub use draw_point::DrawPointAction;
pub use draw_ellipse::DrawEllipseAction;
pub use select::SelectAction;
pub use modify_move::MoveAction;
pub use modify_copy::CopyAction;
pub use modify_rotate::RotateAction;
pub use modify_scale::ScaleAction;
pub use modify_mirror::MirrorAction;
pub use modify_offset::OffsetAction;
pub use modify_trim::TrimAction;
pub use modify_extend::ExtendAction;
pub use modify_fillet::FilletAction;
pub use modify_chamfer::ChamferAction;
pub use grip_edit::GripEditAction;

use crate::action::{Action, ActionType};

/// 创建指定类型的 Action
pub fn create_action(action_type: ActionType) -> Box<dyn Action> {
    match action_type {
        ActionType::Select => Box::new(SelectAction::new()),
        ActionType::DrawLine => Box::new(DrawLineAction::new()),
        ActionType::DrawCircle => Box::new(DrawCircleAction::new()),
        ActionType::DrawArc => Box::new(DrawArcAction::new()),
        ActionType::DrawPolyline => Box::new(DrawPolylineAction::new()),
        ActionType::DrawRectangle => Box::new(DrawRectangleAction::new()),
        ActionType::DrawPoint => Box::new(DrawPointAction::new()),
        ActionType::DrawEllipse => Box::new(DrawEllipseAction::new()),
        ActionType::Move => Box::new(MoveAction::new()),
        ActionType::Copy => Box::new(CopyAction::new()),
        ActionType::Rotate => Box::new(RotateAction::new()),
        ActionType::Scale => Box::new(ScaleAction::new()),
        ActionType::Mirror => Box::new(MirrorAction::new()),
        ActionType::Offset => Box::new(OffsetAction::new()),
        ActionType::Trim => Box::new(TrimAction::new()),
        ActionType::Extend => Box::new(ExtendAction::new()),
        ActionType::Fillet => Box::new(FilletAction::new()),
        ActionType::Chamfer => Box::new(ChamferAction::new()),
        ActionType::GripEdit => Box::new(GripEditAction::new()),
        _ => Box::new(SelectAction::new()),
    }
}
