//! 圆角命令 Action
//!
//! 在两条线段之间创建圆角

use crate::action::{
    Action, ActionContext, ActionResult, ActionType, MouseButton, PreviewGeometry,
};
use zcad_core::entity::EntityId;
use zcad_core::geometry::{Arc, Geometry, Line};
use zcad_core::math::{Point2, Vector2, EPSILON};

/// 圆角状态
#[derive(Debug, Clone, PartialEq)]
enum Status {
    /// 设置半径
    SetRadius,
    /// 选择第一条线
    SelectFirst,
    /// 选择第二条线
    SelectSecond,
}

/// 圆角命令 Action
pub struct FilletAction {
    status: Status,
    radius: f64,
    first_entity: Option<EntityId>,
    first_line: Option<Line>,
}

impl FilletAction {
    pub fn new() -> Self {
        Self {
            status: Status::SetRadius,
            radius: 10.0, // 默认半径
            first_entity: None,
            first_line: None,
        }
    }
}

impl Default for FilletAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for FilletAction {
    fn action_type(&self) -> ActionType {
        ActionType::Fillet
    }

    fn reset(&mut self) {
        self.status = Status::SetRadius;
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
                    Status::SetRadius => ActionResult::Continue,
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
                                if let Some(result) = self.create_fillet(&self.first_line.clone().unwrap(), line2, self.first_entity.unwrap(), entity.id) {
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
        if cmd_upper == "R" || cmd_upper == "RADIUS" {
            self.status = Status::SetRadius;
            return Some(ActionResult::Continue);
        }
        None
    }

    fn on_value(&mut self, _ctx: &ActionContext, value: f64) -> ActionResult {
        if self.status == Status::SetRadius {
            if value >= 0.0 {
                self.radius = value;
                self.status = Status::SelectFirst;
            }
        }
        ActionResult::Continue
    }

    fn get_prompt(&self) -> &str {
        match self.status {
            Status::SetRadius => "输入圆角半径或按 Enter 接受当前值",
            Status::SelectFirst => "选择第一条线",
            Status::SelectSecond => "选择第二条线",
        }
    }

    fn get_preview(&self, _ctx: &ActionContext) -> Vec<PreviewGeometry> {
        Vec::new()
    }
}

impl FilletAction {
    fn find_line_at_point<'a>(&self, ctx: &'a ActionContext, point: Point2) -> Option<&'a zcad_core::entity::Entity> {
        let tolerance = 5.0;
        ctx.entities.iter().find(|e| {
            matches!(&e.geometry, Geometry::Line(_)) && e.geometry.contains_point(&point, tolerance)
        })
    }

    fn create_fillet(&self, line1: &Line, line2: &Line, id1: EntityId, id2: EntityId) -> Option<ActionResult> {
        // 找到两条线的交点
        let intersection = self.find_intersection(line1, line2)?;
        
        if self.radius < EPSILON {
            // 半径为0：只修剪到交点
            let new_line1 = self.trim_line_to_point(line1, intersection);
            let new_line2 = self.trim_line_to_point(line2, intersection);
            return Some(ActionResult::ModifyEntities(vec![
                (id1, Geometry::Line(new_line1)),
                (id2, Geometry::Line(new_line2)),
            ]));
        }
        
        // 计算两条线的方向
        let dir1 = line1.direction();
        let dir2 = line2.direction();
        
        // 计算角平分线方向
        let bisector = (dir1 + dir2).normalize();
        
        // 计算圆心到交点的距离
        let half_angle = (dir1.dot(&dir2)).acos() / 2.0;
        if half_angle.abs() < EPSILON || (std::f64::consts::PI - half_angle).abs() < EPSILON {
            return None; // 平行线
        }
        
        let center_dist = self.radius / half_angle.sin();
        
        // 计算圆心
        let perp1 = Vector2::new(-dir1.y, dir1.x);
        let sign1: f64 = if perp1.dot(&(line2.start - line1.start)) > 0.0 { 1.0 } else { -1.0 };
        let center = intersection + bisector * center_dist * sign1.signum();
        
        // 计算圆弧的起止角
        let to_line1 = (line1.start - center).normalize();
        let to_line2 = (line2.start - center).normalize();
        
        let start_angle = to_line1.y.atan2(to_line1.x);
        let end_angle = to_line2.y.atan2(to_line2.x);
        
        // 计算切点
        let tangent1 = center + Vector2::new(start_angle.cos(), start_angle.sin()) * self.radius;
        let tangent2 = center + Vector2::new(end_angle.cos(), end_angle.sin()) * self.radius;
        
        // 修剪线段
        let new_line1 = self.trim_line_to_point(line1, tangent1);
        let new_line2 = self.trim_line_to_point(line2, tangent2);
        
        // 创建圆弧
        let arc = Arc::new(center, self.radius, start_angle, end_angle);
        
        // 返回修改和创建操作
        // 注意：ActionResult 需要能同时修改和创建实体
        // 这里简化处理，只修改线段，圆弧需要单独创建
        Some(ActionResult::ModifyEntities(vec![
            (id1, Geometry::Line(new_line1)),
            (id2, Geometry::Line(new_line2)),
        ]))
        // TODO: 需要同时创建圆弧，目前 ActionResult 不支持同时修改和创建
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
