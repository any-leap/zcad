//! 延伸命令 Action
//!
//! 延伸到边界边缘

use crate::action::{
    Action, ActionContext, ActionResult, ActionType, MouseButton, PreviewGeometry,
};
use zcad_core::entity::EntityId;
use zcad_core::geometry::{Geometry, Line};
use zcad_core::math::{Point2, EPSILON};

/// 延伸状态
#[derive(Debug, Clone, PartialEq)]
enum Status {
    /// 选择边界对象
    SelectBoundary,
    /// 选择要延伸的对象
    SelectToExtend,
}

/// 延伸命令 Action
pub struct ExtendAction {
    status: Status,
    /// 边界实体 ID 列表
    boundary_entities: Vec<EntityId>,
}

impl ExtendAction {
    pub fn new() -> Self {
        Self {
            status: Status::SelectBoundary,
            boundary_entities: Vec::new(),
        }
    }
}

impl Default for ExtendAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for ExtendAction {
    fn action_type(&self) -> ActionType {
        ActionType::Extend
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
                        if let Some(entity) = self.find_entity_at_point(ctx, point) {
                            if !self.boundary_entities.contains(&entity.id) {
                                self.boundary_entities.push(entity.id);
                            }
                        }
                        ActionResult::Continue
                    }
                    Status::SelectToExtend => {
                        if let Some(entity) = self.find_entity_at_point(ctx, point) {
                            if let Some(extended) = self.extend_entity(ctx, &entity.geometry, point) {
                                return ActionResult::ModifyEntities(vec![(entity.id, extended)]);
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
                            self.status = Status::SelectToExtend;
                            ActionResult::Continue
                        }
                    }
                    Status::SelectToExtend => ActionResult::Cancel,
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
            Status::SelectToExtend => "选择要延伸的对象",
        }
    }

    fn get_preview(&self, _ctx: &ActionContext) -> Vec<PreviewGeometry> {
        Vec::new()
    }
}

impl ExtendAction {
    fn find_entity_at_point<'a>(&self, ctx: &'a ActionContext, point: Point2) -> Option<&'a zcad_core::entity::Entity> {
        let tolerance = 5.0;
        ctx.entities.iter().find(|e| e.geometry.contains_point(&point, tolerance))
    }

    fn extend_entity(&self, ctx: &ActionContext, geometry: &Geometry, click_point: Point2) -> Option<Geometry> {
        match geometry {
            Geometry::Line(line) => self.extend_line(ctx, line, click_point),
            _ => None,
        }
    }

    fn extend_line(&self, ctx: &ActionContext, line: &Line, click_point: Point2) -> Option<Geometry> {
        // 确定延伸哪一端（离点击点更近的那端）
        let start_dist = (line.start - click_point).norm();
        let end_dist = (line.end - click_point).norm();
        
        let extend_from_start = start_dist < end_dist;
        
        // 找到与边界的最近交点
        let mut best_intersection: Option<Point2> = None;
        let mut best_dist = f64::MAX;
        
        // 延伸线段方向
        let dir = line.direction();
        let extend_dir = if extend_from_start { -dir } else { dir };
        let extend_point = if extend_from_start { line.start } else { line.end };
        
        // 创建一个延伸射线（足够长）
        let ray_end = extend_point + extend_dir * 10000.0;
        let ray = Line::new(extend_point, ray_end);
        
        for boundary_id in &self.boundary_entities {
            if let Some(boundary) = ctx.entities.iter().find(|e| e.id == *boundary_id) {
                let intersections = self.find_intersections(&Geometry::Line(ray.clone()), &boundary.geometry);
                
                for p in intersections {
                    let dist = (p - extend_point).norm();
                    if dist > EPSILON && dist < best_dist {
                        best_dist = dist;
                        best_intersection = Some(p);
                    }
                }
            }
        }
        
        if let Some(new_point) = best_intersection {
            if extend_from_start {
                Some(Geometry::Line(Line::new(new_point, line.end)))
            } else {
                Some(Geometry::Line(Line::new(line.start, new_point)))
            }
        } else {
            None
        }
    }

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

        // t1 可以 > 1（射线延伸），t2 必须在 [0, 1] 内（边界线段内）
        if t1 >= 0.0 && t2 >= 0.0 && t2 <= 1.0 {
            Some(l1.start + d1 * t1)
        } else {
            None
        }
    }

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

        if t1 >= 0.0 {
            intersections.push(line.start + d * t1);
        }
        if t2 >= 0.0 && (t2 - t1).abs() > EPSILON {
            intersections.push(line.start + d * t2);
        }

        intersections
    }
}
