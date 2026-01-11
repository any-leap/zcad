//! 偏移命令 Action
//!
//! 支持线段、圆、圆弧、多段线的偏移操作

use crate::action::{
    Action, ActionContext, ActionResult, ActionType, MouseButton, PreviewGeometry,
};
use zcad_core::entity::EntityId;
use zcad_core::geometry::{Arc, Circle, Geometry, Line, Polyline, PolylineVertex};
use zcad_core::math::{Point2, Vector2, EPSILON};

/// 偏移状态
#[derive(Debug, Clone, PartialEq)]
enum Status {
    /// 等待输入偏移距离
    SetDistance,
    /// 等待选择要偏移的对象
    SelectObject,
    /// 等待指定偏移方向
    SelectSide,
}

/// 偏移命令 Action
pub struct OffsetAction {
    status: Status,
    /// 偏移距离
    distance: f64,
    /// 选中的实体
    selected_entity: Option<EntityId>,
    /// 选中的几何体（缓存）
    selected_geometry: Option<Geometry>,
}

impl OffsetAction {
    pub fn new() -> Self {
        Self {
            status: Status::SetDistance,
            distance: 0.0,
            selected_entity: None,
            selected_geometry: None,
        }
    }
}

impl Default for OffsetAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for OffsetAction {
    fn action_type(&self) -> ActionType {
        ActionType::Offset
    }

    fn reset(&mut self) {
        self.status = Status::SetDistance;
        self.distance = 0.0;
        self.selected_entity = None;
        self.selected_geometry = None;
    }

    fn on_mouse_move(&mut self, _ctx: &ActionContext) -> ActionResult {
        ActionResult::Continue
    }

    fn on_mouse_click(&mut self, ctx: &ActionContext, button: MouseButton) -> ActionResult {
        match button {
            MouseButton::Left => {
                let point = ctx.effective_point();
                match self.status {
                    Status::SetDistance => {
                        // 不能通过点击设置距离
                        ActionResult::Continue
                    }
                    Status::SelectObject => {
                        // 尝试选择对象
                        if let Some(entity) = self.find_entity_at_point(ctx, point) {
                            if Self::can_offset(&entity.geometry) {
                                self.selected_entity = Some(entity.id);
                                self.selected_geometry = Some(entity.geometry.clone());
                                self.status = Status::SelectSide;
                            }
                        }
                        ActionResult::Continue
                    }
                    Status::SelectSide => {
                        // 根据点击位置确定偏移方向
                        if let Some(geom) = &self.selected_geometry {
                            let offset_geom = self.offset_geometry(geom, point);
                            if let Some(new_geom) = offset_geom {
                                // 重置以便继续偏移其他对象
                                self.selected_entity = None;
                                self.selected_geometry = None;
                                self.status = Status::SelectObject;
                                return ActionResult::CreateEntities(vec![new_geom]);
                            }
                        }
                        ActionResult::Continue
                    }
                }
            }
            MouseButton::Right => {
                match self.status {
                    Status::SetDistance => ActionResult::Cancel,
                    Status::SelectObject => ActionResult::Cancel,
                    Status::SelectSide => {
                        // 取消当前选择，回到选择对象状态
                        self.selected_entity = None;
                        self.selected_geometry = None;
                        self.status = Status::SelectObject;
                        ActionResult::Continue
                    }
                }
            }
            MouseButton::Middle => ActionResult::Continue,
        }
    }

    fn on_coordinate(&mut self, _ctx: &ActionContext, _coord: Point2) -> ActionResult {
        ActionResult::Continue
    }

    fn on_command(&mut self, _ctx: &ActionContext, cmd: &str) -> Option<ActionResult> {
        let cmd_upper = cmd.to_uppercase();
        match cmd_upper.as_str() {
            "T" | "THROUGH" => {
                // 通过点模式（未实现）
                None
            }
            _ => None,
        }
    }

    fn on_value(&mut self, _ctx: &ActionContext, value: f64) -> ActionResult {
        match self.status {
            Status::SetDistance => {
                if value > EPSILON {
                    self.distance = value;
                    self.status = Status::SelectObject;
                }
                ActionResult::Continue
            }
            _ => ActionResult::Continue,
        }
    }

    fn get_prompt(&self) -> &str {
        match self.status {
            Status::SetDistance => "指定偏移距离",
            Status::SelectObject => "选择要偏移的对象",
            Status::SelectSide => "指定点以确定偏移侧",
        }
    }

    fn get_preview(&self, ctx: &ActionContext) -> Vec<PreviewGeometry> {
        let mut previews = Vec::new();
        
        if self.status == Status::SelectSide {
            if let Some(geom) = &self.selected_geometry {
                let mouse = ctx.effective_point();
                if let Some(preview_geom) = self.offset_geometry(geom, mouse) {
                    previews.push(PreviewGeometry::new(preview_geom));
                }
            }
        }
        
        previews
    }
}

impl OffsetAction {
    /// 检查几何体是否可以偏移
    fn can_offset(geometry: &Geometry) -> bool {
        matches!(
            geometry,
            Geometry::Line(_) | Geometry::Circle(_) | Geometry::Arc(_) | Geometry::Polyline(_)
        )
    }

    /// 在点处查找实体
    fn find_entity_at_point<'a>(&self, ctx: &'a ActionContext, point: Point2) -> Option<&'a zcad_core::entity::Entity> {
        let tolerance = 5.0; // 像素容差转换为世界坐标
        ctx.entities.iter().find(|e| e.geometry.contains_point(&point, tolerance))
    }

    /// 执行偏移操作
    fn offset_geometry(&self, geometry: &Geometry, side_point: Point2) -> Option<Geometry> {
        match geometry {
            Geometry::Line(line) => self.offset_line(line, side_point),
            Geometry::Circle(circle) => self.offset_circle(circle, side_point),
            Geometry::Arc(arc) => self.offset_arc(arc, side_point),
            Geometry::Polyline(polyline) => self.offset_polyline(polyline, side_point),
            _ => None,
        }
    }

    /// 偏移线段
    fn offset_line(&self, line: &Line, side_point: Point2) -> Option<Geometry> {
        let dir = line.direction();
        let perp = Vector2::new(-dir.y, dir.x);
        
        // 确定偏移方向
        let mid = line.midpoint();
        let to_side = side_point - mid;
        let sign = if to_side.dot(&perp) > 0.0 { 1.0 } else { -1.0 };
        
        let offset = perp * (self.distance * sign);
        
        Some(Geometry::Line(Line::new(
            line.start + offset,
            line.end + offset,
        )))
    }

    /// 偏移圆
    fn offset_circle(&self, circle: &Circle, side_point: Point2) -> Option<Geometry> {
        let to_side = side_point - circle.center;
        let dist_to_center = to_side.norm();
        
        // 判断是向内还是向外偏移
        let new_radius = if dist_to_center > circle.radius {
            // 外侧
            circle.radius + self.distance
        } else {
            // 内侧
            let r = circle.radius - self.distance;
            if r < EPSILON {
                return None; // 半径太小
            }
            r
        };
        
        Some(Geometry::Circle(Circle::new(circle.center, new_radius)))
    }

    /// 偏移圆弧
    fn offset_arc(&self, arc: &Arc, side_point: Point2) -> Option<Geometry> {
        let to_side = side_point - arc.center;
        let dist_to_center = to_side.norm();
        
        // 判断是向内还是向外偏移
        let new_radius = if dist_to_center > arc.radius {
            // 外侧
            arc.radius + self.distance
        } else {
            // 内侧
            let r = arc.radius - self.distance;
            if r < EPSILON {
                return None;
            }
            r
        };
        
        Some(Geometry::Arc(Arc::new(
            arc.center,
            new_radius,
            arc.start_angle,
            arc.end_angle,
        )))
    }

    /// 偏移多段线
    fn offset_polyline(&self, polyline: &Polyline, side_point: Point2) -> Option<Geometry> {
        if polyline.vertices.len() < 2 {
            return None;
        }

        // 简化实现：逐段偏移线段，然后延伸/裁剪交点
        let mut new_vertices = Vec::with_capacity(polyline.vertices.len());
        
        // 确定偏移方向
        let offset_sign = self.determine_offset_side(polyline, side_point);
        
        for i in 0..polyline.vertices.len() {
            let v1 = &polyline.vertices[i];
            
            if i == 0 {
                // 第一个点：计算第一段的偏移方向
                let v2 = &polyline.vertices[1];
                let dir = (v2.point - v1.point).normalize();
                let perp = Vector2::new(-dir.y, dir.x) * offset_sign;
                let offset = perp * self.distance;
                new_vertices.push(PolylineVertex::with_bulge(v1.point + offset, v1.bulge));
            } else if i == polyline.vertices.len() - 1 && !polyline.closed {
                // 最后一个点（开放多段线）
                let v0 = &polyline.vertices[i - 1];
                let dir = (v1.point - v0.point).normalize();
                let perp = Vector2::new(-dir.y, dir.x) * offset_sign;
                let offset = perp * self.distance;
                new_vertices.push(PolylineVertex::with_bulge(v1.point + offset, v1.bulge));
            } else {
                // 中间点：计算两条边的偏移线交点
                let prev_idx = if i == 0 { polyline.vertices.len() - 1 } else { i - 1 };
                let next_idx = (i + 1) % polyline.vertices.len();
                
                let v_prev = &polyline.vertices[prev_idx];
                let v_next = &polyline.vertices[next_idx];
                
                // 前一段的方向
                let dir1 = (v1.point - v_prev.point).normalize();
                let perp1 = Vector2::new(-dir1.y, dir1.x) * offset_sign;
                
                // 后一段的方向
                let dir2 = (v_next.point - v1.point).normalize();
                let perp2 = Vector2::new(-dir2.y, dir2.x) * offset_sign;
                
                // 偏移后的线段
                let p1a = v_prev.point + perp1 * self.distance;
                let p1b = v1.point + perp1 * self.distance;
                let p2a = v1.point + perp2 * self.distance;
                let p2b = v_next.point + perp2 * self.distance;
                
                // 计算交点
                if let Some(intersection) = Self::line_line_intersection(p1a, p1b, p2a, p2b) {
                    new_vertices.push(PolylineVertex::with_bulge(intersection, v1.bulge));
                } else {
                    // 平行线：使用中点
                    let offset = (perp1 + perp2).normalize() * self.distance;
                    new_vertices.push(PolylineVertex::with_bulge(v1.point + offset, v1.bulge));
                }
            }
        }
        
        Some(Geometry::Polyline(Polyline::new(new_vertices, polyline.closed)))
    }

    /// 确定多段线偏移方向
    fn determine_offset_side(&self, polyline: &Polyline, side_point: Point2) -> f64 {
        // 简化：使用第一段确定方向
        if polyline.vertices.len() < 2 {
            return 1.0;
        }
        
        let v1 = &polyline.vertices[0];
        let v2 = &polyline.vertices[1];
        let dir = (v2.point - v1.point).normalize();
        let perp = Vector2::new(-dir.y, dir.x);
        let mid = Point2::new((v1.point.x + v2.point.x) / 2.0, (v1.point.y + v2.point.y) / 2.0);
        let to_side = side_point - mid;
        
        if to_side.dot(&perp) > 0.0 { 1.0 } else { -1.0 }
    }

    /// 计算两条直线的交点
    fn line_line_intersection(p1: Point2, p2: Point2, p3: Point2, p4: Point2) -> Option<Point2> {
        let d1 = p2 - p1;
        let d2 = p4 - p3;
        
        let cross = d1.x * d2.y - d1.y * d2.x;
        
        if cross.abs() < EPSILON {
            return None; // 平行
        }
        
        let d = p3 - p1;
        let t = (d.x * d2.y - d.y * d2.x) / cross;
        
        Some(p1 + d1 * t)
    }
}
