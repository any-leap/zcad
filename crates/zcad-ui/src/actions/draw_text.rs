//! 绘制文本 Action

use crate::action::{
    Action, ActionContext, ActionResult, ActionType, MouseButton, PreviewGeometry,
};
use zcad_core::geometry::{Geometry, Text, TextAlignment};
use zcad_core::math::Point2;

/// 文本绘制状态
#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    /// 等待设置插入点
    SetPosition,
    /// 等待输入文本内容
    SetContent,
    /// 等待设置高度（可选）
    SetHeight,
}

/// 绘制文本 Action
pub struct DrawTextAction {
    status: Status,
    position: Option<Point2>,
    content: String,
    height: f64,
    rotation: f64,
    alignment: TextAlignment,
}

impl DrawTextAction {
    pub fn new() -> Self {
        Self {
            status: Status::SetPosition,
            position: None,
            content: String::new(),
            height: 2.5, // 默认文本高度
            rotation: 0.0,
            alignment: TextAlignment::Left,
        }
    }

    /// 创建文本实体
    fn create_text(&self) -> Option<Text> {
        if let Some(pos) = self.position {
            if !self.content.is_empty() {
                let mut text = Text::new(pos, &self.content, self.height);
                text.rotation = self.rotation;
                text.alignment = self.alignment;
                return Some(text);
            }
        }
        None
    }
}

impl Default for DrawTextAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for DrawTextAction {
    fn action_type(&self) -> ActionType {
        ActionType::DrawText
    }

    fn reset(&mut self) {
        self.status = Status::SetPosition;
        self.position = None;
        self.content.clear();
        self.height = 2.5;
        self.rotation = 0.0;
        self.alignment = TextAlignment::Left;
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
                match self.status {
                    Status::SetPosition => ActionResult::Cancel,
                    Status::SetContent | Status::SetHeight => {
                        // 如果已经有内容，尝试创建文本
                        if let Some(text) = self.create_text() {
                            self.reset();
                            return ActionResult::CreateEntities(vec![Geometry::Text(text)]);
                        }
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
            Status::SetPosition => {
                self.position = Some(coord);
                self.status = Status::SetContent;
                ActionResult::Continue
            }
            Status::SetContent | Status::SetHeight => {
                // 如果在输入内容或高度时点击了新位置，完成当前文本并开始新文本
                if let Some(text) = self.create_text() {
                    self.content.clear();
                    self.position = Some(coord);
                    self.status = Status::SetContent;
                    return ActionResult::CreateEntities(vec![Geometry::Text(text)]);
                }
                // 没有内容，直接移动位置
                self.position = Some(coord);
                self.status = Status::SetContent;
                ActionResult::Continue
            }
        }
    }

    fn on_command(&mut self, _ctx: &ActionContext, cmd: &str) -> Option<ActionResult> {
        let cmd_upper = cmd.to_uppercase();
        
        match self.status {
            Status::SetPosition => {
                // 在设置位置时，处理子命令
                match cmd_upper.as_str() {
                    "H" | "HEIGHT" => {
                        self.status = Status::SetHeight;
                        return Some(ActionResult::Continue);
                    }
                    "L" | "LEFT" => {
                        self.alignment = TextAlignment::Left;
                        return Some(ActionResult::Continue);
                    }
                    "C" | "CENTER" => {
                        self.alignment = TextAlignment::Center;
                        return Some(ActionResult::Continue);
                    }
                    "R" | "RIGHT" => {
                        self.alignment = TextAlignment::Right;
                        return Some(ActionResult::Continue);
                    }
                    _ => None,
                }
            }
            Status::SetContent => {
                // 检查是否是子命令
                match cmd_upper.as_str() {
                    "H" | "HEIGHT" => {
                        self.status = Status::SetHeight;
                        return Some(ActionResult::Continue);
                    }
                    "" => {
                        // 空输入，如果已有内容则创建文本
                        if let Some(text) = self.create_text() {
                            self.reset();
                            return Some(ActionResult::CreateEntities(vec![Geometry::Text(text)]));
                        }
                        return Some(ActionResult::Continue);
                    }
                    _ => {
                        // 其他输入作为文本内容
                        self.content = cmd.to_string();
                        // 创建文本
                        if let Some(text) = self.create_text() {
                            // 保持位置，准备输入下一行
                            if let Some(pos) = self.position {
                                // 下一行位置向下偏移
                                self.position = Some(Point2::new(pos.x, pos.y - self.height * 1.5));
                            }
                            self.content.clear();
                            return Some(ActionResult::CreateEntities(vec![Geometry::Text(text)]));
                        }
                        return Some(ActionResult::Continue);
                    }
                }
            }
            Status::SetHeight => {
                // 尝试解析高度值
                if let Ok(h) = cmd.parse::<f64>() {
                    if h > 0.0 {
                        self.height = h;
                        self.status = Status::SetContent;
                        return Some(ActionResult::Continue);
                    }
                }
                None
            }
        }
    }

    fn on_value(&mut self, _ctx: &ActionContext, value: f64) -> ActionResult {
        match self.status {
            Status::SetHeight => {
                if value > 0.0 {
                    self.height = value;
                    self.status = Status::SetContent;
                }
            }
            Status::SetPosition => {
                // 直接输入高度
                if value > 0.0 {
                    self.height = value;
                }
            }
            _ => {}
        }
        ActionResult::Continue
    }

    fn get_prompt(&self) -> &str {
        match self.status {
            Status::SetPosition => "指定文本插入点 或 [高度(H)/对齐(L/C/R)]:",
            Status::SetContent => "输入文本内容 (回车确认，继续输入下一行):",
            Status::SetHeight => "指定文本高度:",
        }
    }

    fn get_available_commands(&self) -> Vec<&str> {
        match self.status {
            Status::SetPosition => vec!["Height", "Left", "Center", "Right"],
            Status::SetContent => vec!["Height"],
            Status::SetHeight => vec![],
        }
    }

    fn get_preview(&self, ctx: &ActionContext) -> Vec<PreviewGeometry> {
        let mut previews = Vec::new();
        
        match self.status {
            Status::SetPosition => {
                // 在鼠标位置显示预览文本
                let preview_text = Text::new(
                    ctx.effective_point(),
                    "Text",
                    self.height,
                );
                previews.push(PreviewGeometry::new(Geometry::Text(preview_text)));
            }
            Status::SetContent | Status::SetHeight => {
                if let Some(pos) = self.position {
                    // 显示当前输入的文本预览
                    let content = if self.content.is_empty() {
                        "Text".to_string()
                    } else {
                        self.content.clone()
                    };
                    let mut preview_text = Text::new(pos, content, self.height);
                    preview_text.alignment = self.alignment;
                    preview_text.rotation = self.rotation;
                    previews.push(PreviewGeometry::new(Geometry::Text(preview_text)));
                }
            }
        }
        
        previews
    }
}
