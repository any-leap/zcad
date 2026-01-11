//! 倒角命令 Action
//!
//! 在两条线段之间创建倒角（斜切）

use crate::action::{
    Action, ActionContext, ActionResult, ActionType, MouseButton, PreviewGeometry,
};
use zcad_core::entity::EntityId;
use zcad_core::geometry::{Geometry, Line};
use zcad_core::math::{Point2, EPSILON};

/// 倒角状态
#[derive(Debug, Clone, PartialEq)]
enum Status {
    /// 设置第一个距离
    SetDistance1,
    /// 设置第二个距离
    SetDistance2,
    /// 选择第一条线
    SelectFirst,
    /// 选择第二条线
    SelectSecond,
}

/// 倒角命令 Action
pub struct ChamferAction {
    status: Status,
    distance1: f64,
    distance2: f64,
    first_entity: Option<EntityId>,
    first_line: Option<Line>,
}

impl ChamferAction {
    pub fn new() -> Self {
        Self {
            status: Status::SetDistance1,
            distance1: 10.0,
            distance2: 10.0,
            first_entity: None,
            first_line: None,
        }
    }
}

impl Default for ChamferAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for ChamferAction {
    fn action_type(&self) -> ActionType {
        ActionType::Chamfer
    }

    fn reset(&mut self) {
        self.status = Status::SetDistance1;
        self.first_entity = None;
        self.first_line = None;
    }

    fn on_mouse_move(&mut self, _ctx: &ActionContext) -> ActionResult {
        ActionResult::Continue
    }

    fn on_mouse_click(&mut self, ctx: &ActionContext, button: MouseButton) -> ActionResult {
        match button {
            MouseButton::Left => {
                let point = ctx.effective_point();
                match self.status {
                    Status::SetDistance1 | Status::SetDistance2 => ActionResult::Continue,
                    Status::SelectFirst => {
                        if let Some(entity) = self.find_line_at_point(ctx, point) {
                            if let Geometry::Line(line) = &entity.geometry {
                                self.first_entity = Some(entity.id);
                                self.first_line = Some(line.clone());
                                self.status = Status::SelectSecond;
                            }
                        }
                        ActionResult::Continue
                    }
                    Status::SelectSecond => {
                        if let Some(entity) = self.find_line_at_point(ctx, point) {
                            if let Geometry::Line(line2) = &entity.geometry {
                                if let Some(result) = self.create_chamfer(&self.first_line.clone().unwrap(), line2, self.first_entity.unwrap(), entity.id) {
                                    self.first_entity = None;
                                    self.first_line = None;
                                    self.status = Status::SelectFirst;
                                    return result;
                                }
                            }
                        }
                        ActionResult::Continue
                    }
                }
            }
            MouseButton::Right => ActionResult::Cancel,
            MouseButton::Middle => ActionResult::Continue,
        }
    }

    fn on_coordinate(&mut self, _ctx: &ActionContext, _coord: Point2) -> ActionResult {
        ActionResult::Continue
    }

    fn on_command(&mut self, _ctx: &ActionContext, cmd: &str) -> Option<ActionResult> {
        let cmd_upper = cmd.to_uppercase();
        if cmd_upper == "D" || cmd_upper == "DISTANCE" {
            self.status = Status::SetDistance1;
            return Some(ActionResult::Continue);
        }
        None
    }

    fn on_value(&mut self, _ctx: &ActionContext, value: f64) -> ActionResult {
        match self.status {
            Status::SetDistance1 => {
                if value >= 0.0 {
                    self.distance1 = value;
                    self.status = Status::SetDistance2;
                }
            }
            Status::SetDistance2 => {
                if value >= 0.0 {
                    self.distance2 = value;
                    self.status = Status::SelectFirst;
                }
            }
            _ => {}
        }
        ActionResult::Continue
    }

    fn get_prompt(&self) -> &str {
        match self.status {
            Status::SetDistance1 => "输入第一个倒角距离",
            Status::SetDistance2 => "输入第二个倒角距离",
            Status::SelectFirst => "选择第一条线",
            Status::SelectSecond => "选择第二条线",
        }
    }

    fn get_preview(&self, _ctx: &ActionContext) -> Vec<PreviewGeometry> {
        Vec::new()
    }
}

impl ChamferAction {
    fn find_line_at_point<'a>(&self, ctx: &'a ActionContext, point: Point2) -> Option<&'a zcad_core::entity::Entity> {
        let tolerance = 5.0;
        ctx.entities.iter().find(|e| {
            matches!(&e.geometry, Geometry::Line(_)) && e.geometry.contains_point(&point, tolerance)
        })
    }

    fn create_chamfer(&self, line1: &Line, line2: &Line, id1: EntityId, id2: EntityId) -> Option<ActionResult> {
        // 找到两条线的交点
        let intersection = self.find_intersection(line1, line2)?;
        
        if self.distance1 < EPSILON && self.distance2 < EPSILON {
            // 距离为0：只修剪到交点
            let new_line1 = self.trim_line_to_point(line1, intersection);
            let new_line2 = self.trim_line_to_point(line2, intersection);
            return Some(ActionResult::ModifyEntities(vec![
                (id1, Geometry::Line(new_line1)),
                (id2, Geometry::Line(new_line2)),
            ]));
        }
        
        // 计算倒角点
        let dir1 = line1.direction();
        let dir2 = line2.direction();
        
        // 计算从交点向线段方向的倒角点
        let chamfer_pt1 = self.find_chamfer_point(line1, intersection, self.distance1)?;
        let chamfer_pt2 = self.find_chamfer_point(line2, intersection, self.distance2)?;
        
        // 修剪线段
        let new_line1 = self.trim_line_to_point(line1, chamfer_pt1);
        let new_line2 = self.trim_line_to_point(line2, chamfer_pt2);
        
        // 创建倒角线
        let _chamfer_line = Line::new(chamfer_pt1, chamfer_pt2);
        
        // 返回修改操作
        Some(ActionResult::ModifyEntities(vec![
            (id1, Geometry::Line(new_line1)),
            (id2, Geometry::Line(new_line2)),
        ]))
        // TODO: 需要同时创建倒角线
    }

    fn find_intersection(&self, l1: &Line, l2: &Line) -> Option<Point2> {
        let d1 = l1.end - l1.start;
        let d2 = l2.end - l2.start;

        let cross = d1.x * d2.y - d1.y * d2.x;
        
        if cross.abs() < EPSILON {
            return None;
        }

        let d = l2.start - l1.start;
        let t1 = (d.x * d2.y - d.y * d2.x) / cross;

        Some(l1.start + d1 * t1)
    }

    fn find_chamfer_point(&self, line: &Line, intersection: Point2, distance: f64) -> Option<Point2> {
        let dir = line.direction();
        
        // 确定从交点向线段内部的方向
        let to_start = (line.start - intersection).norm();
        let to_end = (line.end - intersection).norm();
        
        if to_start < to_end {
            // 交点靠近起点，向终点方向偏移
            Some(intersection + dir * distance)
        } else {
            // 交点靠近终点，向起点方向偏移
            Some(intersection - dir * distance)
        }
    }

    fn trim_line_to_point(&self, line: &Line, point: Point2) -> Line {
        let start_dist = (line.start - point).norm();
        let end_dist = (line.end - point).norm();
        
        if start_dist < end_dist {
            Line::new(point, line.end)
        } else {
            Line::new(line.start, point)
        }
    }
}
