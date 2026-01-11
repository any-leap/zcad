//! 夹点编辑 Action
//!
//! 允许用户通过拖动夹点来直接编辑几何图形。

use crate::action::{
    Action, ActionContext, ActionResult, ActionType, MouseButton, PreviewGeometry,
};
use zcad_core::entity::EntityId;
use zcad_core::geometry::{Geometry, Line};
use zcad_core::grip::{get_grips_for_geometry, update_geometry_by_grip, Grip, GripType};
use zcad_core::math::Point2;

/// 夹点编辑状态
#[derive(Debug, Clone, Copy, PartialEq)]
enum Status {
    /// 等待选择夹点
    SelectGrip,
    /// 拖动夹点中
    DraggingGrip,
}

/// 夹点编辑历史操作
#[derive(Debug, Clone)]
enum HistoryAction {
    /// 选择了夹点
    SelectGrip {
        entity_id: EntityId,
        grip_index: usize,
    },
    /// 移动了夹点
    MoveGrip {
        old_geometry: Geometry,
    },
}

/// 夹点编辑 Action
pub struct GripEditAction {
    status: Status,
    /// 当前选中的实体及其夹点
    selected_entity: Option<(EntityId, Geometry, Vec<Grip>)>,
    /// 当前激活（正在拖动）的夹点
    active_grip: Option<Grip>,
    /// 夹点的原始位置
    original_grip_position: Option<Point2>,
    /// 预览几何体
    preview_geometry: Option<Geometry>,
    /// 历史记录
    history: Vec<HistoryAction>,
    /// 夹点选择容差（屏幕像素）
    grip_tolerance: f64,
}

impl GripEditAction {
    pub fn new() -> Self {
        Self {
            status: Status::SelectGrip,
            selected_entity: None,
            active_grip: None,
            original_grip_position: None,
            preview_geometry: None,
            history: Vec::new(),
            grip_tolerance: 10.0, // 10 像素容差
        }
    }
    
    /// 设置要编辑的实体
    pub fn set_entity(&mut self, entity_id: EntityId, geometry: Geometry) {
        let grips = get_grips_for_geometry(&geometry);
        self.selected_entity = Some((entity_id, geometry, grips));
        self.status = Status::SelectGrip;
    }
    
    /// 查找最近的夹点
    fn find_nearest_grip(&self, world_pos: Point2, tolerance: f64) -> Option<&Grip> {
        if let Some((_, _, grips)) = &self.selected_entity {
            let mut nearest: Option<&Grip> = None;
            let mut min_dist = tolerance;
            
            for grip in grips {
                let dist = (grip.position - world_pos).norm();
                if dist < min_dist {
                    min_dist = dist;
                    nearest = Some(grip);
                }
            }
            
            nearest
        } else {
            None
        }
    }
    
    /// 更新预览
    fn update_preview(&mut self, ctx: &ActionContext) {
        if let (Some((_, geometry, _)), Some(grip)) = (&self.selected_entity, &self.active_grip) {
            // 使用 grip 模块的更新函数
            self.preview_geometry = update_geometry_by_grip(geometry, grip, ctx.mouse_pos);
        }
    }
}

impl Default for GripEditAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for GripEditAction {
    fn action_type(&self) -> ActionType {
        ActionType::GripEdit
    }
    
    fn reset(&mut self) {
        self.status = Status::SelectGrip;
        self.selected_entity = None;
        self.active_grip = None;
        self.original_grip_position = None;
        self.preview_geometry = None;
        self.history.clear();
    }
    
    fn on_mouse_move(&mut self, ctx: &ActionContext) -> ActionResult {
        match self.status {
            Status::SelectGrip => {
                // 高亮最近的夹点（如果有）
                ActionResult::Continue
            }
            Status::DraggingGrip => {
                self.update_preview(ctx);
                ActionResult::Continue
            }
        }
    }
    
    fn on_mouse_click(&mut self, ctx: &ActionContext, button: MouseButton) -> ActionResult {
        match button {
            MouseButton::Left => {
                let point = ctx.effective_point();
                self.on_coordinate(ctx, point)
            }
            MouseButton::Right => {
                if self.status == Status::DraggingGrip {
                    // 取消当前拖动
                    self.status = Status::SelectGrip;
                    self.active_grip = None;
                    self.original_grip_position = None;
                    self.preview_geometry = None;
                    ActionResult::Continue
                } else {
                    ActionResult::Cancel
                }
            }
            MouseButton::Middle => ActionResult::Continue,
        }
    }
    
    fn on_coordinate(&mut self, ctx: &ActionContext, coord: Point2) -> ActionResult {
        match self.status {
            Status::SelectGrip => {
                // 将屏幕容差转换为世界坐标容差
                let world_tolerance = self.grip_tolerance / ctx.zoom.max(0.001);
                
                // 先克隆需要的数据，避免借用冲突
                let grip_data = self.find_nearest_grip(coord, world_tolerance)
                    .map(|g| (g.clone(), g.index));
                
                if let Some((grip, grip_index)) = grip_data {
                    // 选中夹点
                    let grip_position = grip.position;
                    self.active_grip = Some(grip);
                    self.original_grip_position = Some(grip_position);
                    self.status = Status::DraggingGrip;
                    
                    if let Some((entity_id, _, _)) = &self.selected_entity {
                        self.history.push(HistoryAction::SelectGrip {
                            entity_id: *entity_id,
                            grip_index,
                        });
                    }
                }
                ActionResult::Continue
            }
            Status::DraggingGrip => {
                // 完成拖动，应用修改
                if let Some((entity_id, geometry, _)) = &self.selected_entity {
                    if let Some(grip) = &self.active_grip {
                        if let Some(new_geometry) = update_geometry_by_grip(geometry, grip, coord) {
                            // 保存历史
                            self.history.push(HistoryAction::MoveGrip {
                                old_geometry: geometry.clone(),
                            });
                            
                            // 返回修改结果
                            let result = ActionResult::ModifyEntity(*entity_id, new_geometry);
                            
                            // 重置状态，准备下一次编辑
                            self.status = Status::SelectGrip;
                            self.active_grip = None;
                            self.original_grip_position = None;
                            self.preview_geometry = None;
                            
                            return result;
                        }
                    }
                }
                ActionResult::Continue
            }
        }
    }
    
    fn on_command(&mut self, _ctx: &ActionContext, _cmd: &str) -> Option<ActionResult> {
        None
    }
    
    fn on_value(&mut self, ctx: &ActionContext, value: f64) -> ActionResult {
        match self.status {
            Status::DraggingGrip => {
                // 使用数值输入来精确移动
                if let Some(original_pos) = self.original_grip_position {
                    // 沿当前鼠标方向移动指定距离
                    let dir = (ctx.mouse_pos - original_pos).normalize();
                    let new_pos = original_pos + dir * value;
                    self.on_coordinate(ctx, new_pos)
                } else {
                    ActionResult::Continue
                }
            }
            _ => ActionResult::Continue,
        }
    }
    
    fn get_prompt(&self) -> &str {
        match self.status {
            Status::SelectGrip => "选择夹点进行编辑",
            Status::DraggingGrip => "指定新位置或输入距离",
        }
    }
    
    fn get_preview(&self, ctx: &ActionContext) -> Vec<PreviewGeometry> {
        let mut previews = Vec::new();
        
        // 绘制预览几何体
        if let Some(ref preview_geom) = self.preview_geometry {
            previews.push(PreviewGeometry::new(preview_geom.clone()));
        }
        
        // 绘制夹点
        if let Some((_, _, grips)) = &self.selected_entity {
            for grip in grips {
                // 用小方块表示夹点
                let size = 3.0 / ctx.zoom.max(0.001);
                let is_active = self.active_grip.as_ref().map_or(false, |ag| {
                    ag.grip_type == grip.grip_type && ag.index == grip.index
                });
                
                // 使用十字线表示夹点位置
                let p = grip.position;
                previews.push(PreviewGeometry::reference(Geometry::Line(
                    Line::new(
                        Point2::new(p.x - size, p.y),
                        Point2::new(p.x + size, p.y),
                    )
                )));
                previews.push(PreviewGeometry::reference(Geometry::Line(
                    Line::new(
                        Point2::new(p.x, p.y - size),
                        Point2::new(p.x, p.y + size),
                    )
                )));
                
                // 如果是激活的夹点，加大尺寸
                if is_active {
                    let big_size = size * 2.0;
                    previews.push(PreviewGeometry::new(Geometry::Line(
                        Line::new(
                            Point2::new(p.x - big_size, p.y - big_size),
                            Point2::new(p.x + big_size, p.y + big_size),
                        )
                    )));
                    previews.push(PreviewGeometry::new(Geometry::Line(
                        Line::new(
                            Point2::new(p.x - big_size, p.y + big_size),
                            Point2::new(p.x + big_size, p.y - big_size),
                        )
                    )));
                }
            }
        }
        
        // 如果正在拖动，绘制从原始位置到当前位置的虚线
        if self.status == Status::DraggingGrip {
            if let Some(original_pos) = self.original_grip_position {
                previews.push(PreviewGeometry::reference(Geometry::Line(
                    Line::new(original_pos, ctx.mouse_pos)
                )));
            }
        }
        
        previews
    }
    
    fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }
    
    fn undo(&mut self) {
        if let Some(last_action) = self.history.pop() {
            match last_action {
                HistoryAction::SelectGrip { .. } => {
                    self.active_grip = None;
                    self.original_grip_position = None;
                    self.status = Status::SelectGrip;
                    self.preview_geometry = None;
                }
                HistoryAction::MoveGrip { old_geometry } => {
                    if let Some((entity_id, _, grips)) = &self.selected_entity {
                        let new_grips = get_grips_for_geometry(&old_geometry);
                        self.selected_entity = Some((*entity_id, old_geometry, new_grips));
                    }
                }
            }
        }
    }
}
