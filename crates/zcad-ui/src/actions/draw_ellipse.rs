//! 绘制椭圆 Action

use crate::action::{
    Action, ActionContext, ActionResult, ActionType, MouseButton, PreviewGeometry,
};
use zcad_core::geometry::{Ellipse, Geometry};
use zcad_core::math::{Point2, Vector2};

/// 椭圆绘制状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
    /// 等待设置中心
    SetCenter,
    /// 等待设置长轴端点
    SetMajorAxis,
    /// 等待设置短轴比例
    SetMinorRatio,
}

/// 绘制椭圆 Action
pub struct DrawEllipseAction {
    status: Status,
    center: Option<Point2>,
    major_axis: Option<Vector2>,
}

impl DrawEllipseAction {
    pub fn new() -> Self {
        Self {
            status: Status::SetCenter,
            center: None,
            major_axis: None,
        }
    }
}

impl Default for DrawEllipseAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for DrawEllipseAction {
    fn action_type(&self) -> ActionType {
        ActionType::DrawEllipse
    }

    fn reset(&mut self) {
        self.status = Status::SetCenter;
        self.center = None;
        self.major_axis = None;
    }

    fn on_mouse_move(&mut self, _ctx: &ActionContext) -> ActionResult {
        ActionResult::Continue
    }

    fn on_mouse_click(&mut self, ctx: &ActionContext, button: MouseButton) -> ActionResult {
        match button {
            MouseButton::Left => {
                let point = ctx.effective_point();
                self.on_coordinate(ctx, point)
            }
            MouseButton::Right => {
                // 右键取消当前步骤或整个操作
                match self.status {
                    Status::SetCenter => ActionResult::Cancel,
                    _ => {
                        self.reset();
                        ActionResult::Continue
                    }
                }
            }
            MouseButton::Middle => ActionResult::Continue,
        }
    }

    fn on_coordinate(&mut self, _ctx: &ActionContext, coord: Point2) -> ActionResult {
        match self.status {
            Status::SetCenter => {
                self.center = Some(coord);
                self.status = Status::SetMajorAxis;
                ActionResult::Continue
            }
            Status::SetMajorAxis => {
                if let Some(center) = self.center {
                    let major_axis = coord - center;
                    if major_axis.norm() > 1e-6 {
                        self.major_axis = Some(major_axis);
                        self.status = Status::SetMinorRatio;
                    }
                }
                ActionResult::Continue
            }
            Status::SetMinorRatio => {
                if let (Some(center), Some(major_axis)) = (self.center, self.major_axis) {
                    // 计算短轴比例：鼠标到长轴的距离 / 长轴长度
                    let major_len = major_axis.norm();
                    let major_dir = major_axis / major_len;
                    let minor_dir = Vector2::new(-major_dir.y, major_dir.x);
                    let to_mouse = coord - center;
                    let minor_len = to_mouse.dot(&minor_dir).abs();
                    let ratio = (minor_len / major_len).clamp(0.01, 1.0);
                    
                    let ellipse = Ellipse::new(center, major_axis, ratio);
                    self.reset();
                    return ActionResult::CreateEntities(vec![Geometry::Ellipse(ellipse)]);
                }
                ActionResult::Continue
            }
        }
    }

    fn on_command(&mut self, _ctx: &ActionContext, _cmd: &str) -> Option<ActionResult> {
        None
    }

    fn on_value(&mut self, _ctx: &ActionContext, value: f64) -> ActionResult {
        match self.status {
            Status::SetMajorAxis => {
                // 输入长轴长度
                if let Some(center) = self.center {
                    if value > 1e-6 {
                        // 默认水平方向
                        self.major_axis = Some(Vector2::new(value, 0.0));
                        self.status = Status::SetMinorRatio;
                    }
                }
                ActionResult::Continue
            }
            Status::SetMinorRatio => {
                // 输入短轴比例或长度
                if let (Some(center), Some(major_axis)) = (self.center, self.major_axis) {
                    let major_len = major_axis.norm();
                    // 如果输入值 <= 1，视为比例；否则视为短轴长度
                    let ratio = if value <= 1.0 {
                        value.clamp(0.01, 1.0)
                    } else {
                        (value / major_len).clamp(0.01, 1.0)
                    };
                    
                    let ellipse = Ellipse::new(center, major_axis, ratio);
                    self.reset();
                    return ActionResult::CreateEntities(vec![Geometry::Ellipse(ellipse)]);
                }
                ActionResult::Continue
            }
            _ => ActionResult::Continue,
        }
    }

    fn get_prompt(&self) -> &str {
        match self.status {
            Status::SetCenter => "指定椭圆中心点",
            Status::SetMajorAxis => "指定长轴端点或输入长度",
            Status::SetMinorRatio => "指定短轴端点或输入比例",
        }
    }

    fn get_preview(&self, ctx: &ActionContext) -> Vec<PreviewGeometry> {
        let mut previews = Vec::new();
        
        match self.status {
            Status::SetMajorAxis => {
                if let Some(center) = self.center {
                    let mouse = ctx.effective_point();
                    let major_axis = mouse - center;
                    if major_axis.norm() > 1e-6 {
                        // 预览椭圆（默认比例 0.5）
                        let ellipse = Ellipse::new(center, major_axis, 0.5);
                        previews.push(PreviewGeometry::new(Geometry::Ellipse(ellipse)));
                    }
                }
            }
            Status::SetMinorRatio => {
                if let (Some(center), Some(major_axis)) = (self.center, self.major_axis) {
                    let mouse = ctx.effective_point();
                    let major_len = major_axis.norm();
                    let major_dir = major_axis / major_len;
                    let minor_dir = Vector2::new(-major_dir.y, major_dir.x);
                    let to_mouse = mouse - center;
                    let minor_len = to_mouse.dot(&minor_dir).abs();
                    let ratio = (minor_len / major_len).clamp(0.01, 1.0);
                    
                    let ellipse = Ellipse::new(center, major_axis, ratio);
                    previews.push(PreviewGeometry::new(Geometry::Ellipse(ellipse)));
                }
            }
            _ => {}
        }
        
        previews
    }
}
