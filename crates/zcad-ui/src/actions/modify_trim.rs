//! 修剪命令 Action
//!
//! 修剪到边界边缘

use crate::action::{
    Action, ActionContext, ActionResult, ActionType, MouseButton, PreviewGeometry,
};
use zcad_core::entity::EntityId;
use zcad_core::geometry::{Arc, Geometry, Line};
use zcad_core::math::{Point2, EPSILON};

/// 修剪状态
#[derive(Debug, Clone, PartialEq)]
enum Status {
    /// 选择边界对象
    SelectBoundary,
    /// 选择要修剪的对象
    SelectToTrim,
}

/// 修剪命令 Action
pub struct TrimAction {
    status: Status,
    /// 边界实体 ID 列表
    boundary_entities: Vec<EntityId>,
}

impl TrimAction {
    pub fn new() -> Self {
        Self {
            status: Status::SelectBoundary,
            boundary_entities: Vec::new(),
        }
    }
}

impl Default for TrimAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for TrimAction {
    fn action_type(&self) -> ActionType {
        ActionType::Trim
    }

    fn reset(&mut self) {
        self.status = Status::SelectBoundary;
        self.boundary_entities.clear();
    }

    fn on_mouse_move(&mut self, _ctx: &ActionContext) -> ActionResult {
        ActionResult::Continue
    }

    fn on_mouse_click(&mut self, ctx: &ActionContext, button: MouseButton) -> ActionResult {
        match button {
            MouseButton::Left => {
                let point = ctx.effective_point();
                match self.status {
                    Status::SelectBoundary => {
                        // 选择边界对象
                        if let Some(entity) = self.find_entity_at_point(ctx, point) {
                            if !self.boundary_entities.contains(&entity.id) {
                                self.boundary_entities.push(entity.id);
                            }
                        }
                        ActionResult::Continue
                    }
                    Status::SelectToTrim => {
                        // 选择要修剪的对象并执行修剪
                        if let Some(entity) = self.find_entity_at_point(ctx, point) {
                            if let Some(trimmed) = self.trim_entity(ctx, &entity.geometry, point) {
                                return ActionResult::ModifyEntities(vec![(entity.id, trimmed)]);
                            }
                        }
                        ActionResult::Continue
                    }
                }
            }
            MouseButton::Right => {
                match self.status {
                    Status::SelectBoundary => {
                        if self.boundary_entities.is_empty() {
                            ActionResult::Cancel
                        } else {
                            // 切换到修剪模式
                            self.status = Status::SelectToTrim;
                            ActionResult::Continue
                        }
                    }
                    Status::SelectToTrim => ActionResult::Cancel,
                }
            }
            MouseButton::Middle => ActionResult::Continue,
        }
    }

    fn on_coordinate(&mut self, _ctx: &ActionContext, _coord: Point2) -> ActionResult {
        ActionResult::Continue
    }

    fn on_command(&mut self, _ctx: &ActionContext, _cmd: &str) -> Option<ActionResult> {
        None
    }

    fn get_prompt(&self) -> &str {
        match self.status {
            Status::SelectBoundary => "选择边界对象，右键确认",
            Status::SelectToTrim => "选择要修剪的对象",
        }
    }

    fn get_preview(&self, _ctx: &ActionContext) -> Vec<PreviewGeometry> {
        Vec::new()
    }
}

impl TrimAction {
    /// 在点处查找实体
    fn find_entity_at_point<'a>(&self, ctx: &'a ActionContext, point: Point2) -> Option<&'a zcad_core::entity::Entity> {
        let tolerance = 5.0;
        ctx.entities.iter().find(|e| e.geometry.contains_point(&point, tolerance))
    }

    /// 修剪实体
    fn trim_entity(&self, ctx: &ActionContext, geometry: &Geometry, click_point: Point2) -> Option<Geometry> {
        match geometry {
            Geometry::Line(line) => self.trim_line(ctx, line, click_point),
            Geometry::Arc(arc) => self.trim_arc(ctx, arc, click_point),
            _ => None,
        }
    }

    /// 修剪线段
    fn trim_line(&self, ctx: &ActionContext, line: &Line, click_point: Point2) -> Option<Geometry> {
        // 找到所有与边界的交点
        let mut intersections = Vec::new();
        
        for boundary_id in &self.boundary_entities {
            if let Some(boundary) = ctx.entities.iter().find(|e| e.id == *boundary_id) {
                let points = self.find_intersections(&Geometry::Line(line.clone()), &boundary.geometry);
                intersections.extend(points);
            }
        }
        
        if intersections.is_empty() {
            return None;
        }
        
        // 按到起点的参数排序
        let line_vec = line.end - line.start;
        let line_len = line_vec.norm();
        
        let mut params: Vec<f64> = intersections
            .iter()
            .map(|p| {
                let v = *p - line.start;
                v.dot(&line_vec) / (line_len * line_len)
            })
            .filter(|&t| t > EPSILON && t < 1.0 - EPSILON)
            .collect();
        
        if params.is_empty() {
            return None;
        }
        
        params.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        // 确定点击位置的参数
        let click_vec = click_point - line.start;
        let click_t = click_vec.dot(&line_vec) / (line_len * line_len);
        
        // 找到包含点击位置的段
        let mut segment_start = 0.0;
        let mut segment_end = 1.0;
        
        for &t in &params {
            if click_t < t {
                segment_end = t;
                break;
            }
            segment_start = t;
        }
        
        // 删除点击的段，保留剩余部分
        // 这里简化处理：返回离点击点较远的那一段
        let start_dist = (click_point - line.start).norm();
        let end_dist = (click_point - line.end).norm();
        
        if start_dist < end_dist && segment_start > EPSILON {
            // 保留终点侧
            let new_start = line.start + line_vec * segment_start;
            Some(Geometry::Line(Line::new(new_start, line.end)))
        } else if segment_end < 1.0 - EPSILON {
            // 保留起点侧
            let new_end = line.start + line_vec * segment_end;
            Some(Geometry::Line(Line::new(line.start, new_end)))
        } else {
            None
        }
    }

    /// 修剪圆弧
    fn trim_arc(&self, _ctx: &ActionContext, arc: &Arc, _click_point: Point2) -> Option<Geometry> {
        // 简化实现：暂不支持圆弧修剪
        Some(Geometry::Arc(arc.clone()))
    }

    /// 计算两个几何体的交点
    fn find_intersections(&self, geom1: &Geometry, geom2: &Geometry) -> Vec<Point2> {
        match (geom1, geom2) {
            (Geometry::Line(l1), Geometry::Line(l2)) => {
                self.line_line_intersection(l1, l2).into_iter().collect()
            }
            (Geometry::Line(line), Geometry::Circle(circle)) => {
                self.line_circle_intersection(line, circle)
            }
            _ => vec![],
        }
    }

    /// 线段-线段交点
    fn line_line_intersection(&self, l1: &Line, l2: &Line) -> Option<Point2> {
        let d1 = l1.end - l1.start;
        let d2 = l2.end - l2.start;

        let cross = d1.x * d2.y - d1.y * d2.x;
        
        if cross.abs() < EPSILON {
            return None;
        }

        let d = l2.start - l1.start;
        let t1 = (d.x * d2.y - d.y * d2.x) / cross;
        let t2 = (d.x * d1.y - d.y * d1.x) / cross;

        if t1 >= 0.0 && t1 <= 1.0 && t2 >= 0.0 && t2 <= 1.0 {
            Some(l1.start + d1 * t1)
        } else {
            None
        }
    }

    /// 线段-圆交点
    fn line_circle_intersection(&self, line: &Line, circle: &zcad_core::geometry::Circle) -> Vec<Point2> {
        let d = line.end - line.start;
        let f = line.start - circle.center;

        let a = d.dot(&d);
        let b = 2.0 * f.dot(&d);
        let c = f.dot(&f) - circle.radius * circle.radius;

        let discriminant = b * b - 4.0 * a * c;

        if discriminant < 0.0 {
            return vec![];
        }

        let mut intersections = Vec::new();
        let sqrt_disc = discriminant.sqrt();
        
        let t1 = (-b - sqrt_disc) / (2.0 * a);
        let t2 = (-b + sqrt_disc) / (2.0 * a);

        if t1 >= 0.0 && t1 <= 1.0 {
            intersections.push(line.start + d * t1);
        }
        if t2 >= 0.0 && t2 <= 1.0 && (t2 - t1).abs() > EPSILON {
            intersections.push(line.start + d * t2);
        }

        intersections
    }
}
