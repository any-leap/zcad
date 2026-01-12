//! 绘制表格 Action

use crate::action::{
    Action, ActionContext, ActionResult, ActionType, MouseButton, PreviewGeometry,
};
use zcad_core::geometry::{Geometry, Table, Line};
use zcad_core::math::Point2;

/// 表格绘制状态
#[derive(Debug, Clone, PartialEq, Eq)]
enum Status {
    /// 等待设置插入点
    SetPosition,
    /// 等待设置行列数
    SetSize,
    /// 等待设置对角点（确定大小）
    SetCorner,
}

/// 绘制表格 Action
pub struct DrawTableAction {
    status: Status,
    position: Option<Point2>,
    rows: usize,
    columns: usize,
}

impl DrawTableAction {
    pub fn new() -> Self {
        Self {
            status: Status::SetPosition,
            position: None,
            rows: 3,     // 默认 3 行
            columns: 3,  // 默认 3 列
        }
    }

    /// 创建表格预览
    fn create_preview(&self, corner: Point2) -> Option<Table> {
        if let Some(pos) = self.position {
            let width = (corner.x - pos.x).abs();
            let height = (corner.y - pos.y).abs();
            
            if width > 0.1 && height > 0.1 {
                let top_left = Point2::new(
                    pos.x.min(corner.x),
                    pos.y.max(corner.y),
                );
                
                let mut table = Table::new(top_left, self.rows, self.columns);
                
                // 设置列宽和行高
                let col_width = width / self.columns as f64;
                let row_height = height / self.rows as f64;
                
                for i in 0..self.columns {
                    table.set_column_width(i, col_width);
                }
                for i in 0..self.rows {
                    table.set_row_height(i, row_height);
                }
                
                // 调整文本高度
                table.style.text_height = (row_height * 0.5).min(2.5);
                
                return Some(table);
            }
        }
        None
    }
}

impl Default for DrawTableAction {
    fn default() -> Self {
        Self::new()
    }
}

impl Action for DrawTableAction {
    fn action_type(&self) -> ActionType {
        ActionType::DrawTable
    }

    fn reset(&mut self) {
        self.status = Status::SetPosition;
        self.position = None;
        self.rows = 3;
        self.columns = 3;
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
                    Status::SetSize | Status::SetCorner => {
                        self.reset();
                        ActionResult::Continue
                    }
                }
            }
            MouseButton::Middle => ActionResult::Continue,
        }
    }

    fn on_coordinate(&mut self, ctx: &ActionContext, coord: Point2) -> ActionResult {
        match self.status {
            Status::SetPosition => {
                self.position = Some(coord);
                self.status = Status::SetCorner;
                ActionResult::Continue
            }
            Status::SetCorner => {
                if let Some(table) = self.create_preview(coord) {
                    self.reset();
                    return ActionResult::CreateEntities(vec![Geometry::Table(table)]);
                }
                ActionResult::Continue
            }
            Status::SetSize => {
                // 这个状态目前不使用，保留以备将来扩展
                ActionResult::Continue
            }
        }
    }

    fn on_command(&mut self, _ctx: &ActionContext, cmd: &str) -> Option<ActionResult> {
        let cmd_upper = cmd.to_uppercase();
        
        // 解析行列数设置
        if let Some(stripped) = cmd_upper.strip_prefix("R") {
            if let Ok(rows) = stripped.parse::<usize>() {
                if rows > 0 && rows <= 100 {
                    self.rows = rows;
                    return Some(ActionResult::Continue);
                }
            }
        }
        
        if let Some(stripped) = cmd_upper.strip_prefix("C") {
            if let Ok(cols) = stripped.parse::<usize>() {
                if cols > 0 && cols <= 100 {
                    self.columns = cols;
                    return Some(ActionResult::Continue);
                }
            }
        }
        
        // 解析 "行x列" 格式，如 "3x4"
        if let Some((rows_str, cols_str)) = cmd.split_once('x').or_else(|| cmd.split_once('X')) {
            if let (Ok(rows), Ok(cols)) = (rows_str.trim().parse::<usize>(), cols_str.trim().parse::<usize>()) {
                if rows > 0 && rows <= 100 && cols > 0 && cols <= 100 {
                    self.rows = rows;
                    self.columns = cols;
                    return Some(ActionResult::Continue);
                }
            }
        }
        
        None
    }

    fn on_value(&mut self, _ctx: &ActionContext, _value: f64) -> ActionResult {
        ActionResult::Continue
    }

    fn get_prompt(&self) -> &str {
        match self.status {
            Status::SetPosition => "指定表格插入点 或 [行数(Rn)/列数(Cn)/行x列]:",
            Status::SetCorner => "指定对角点 (确定表格大小):",
            Status::SetSize => "输入行列数 (如: 3x4):",
        }
    }

    fn get_available_commands(&self) -> Vec<&str> {
        match self.status {
            Status::SetPosition => vec!["R3 (3行)", "C4 (4列)", "3x4 (3行4列)"],
            _ => vec![],
        }
    }

    fn get_preview(&self, ctx: &ActionContext) -> Vec<PreviewGeometry> {
        let mut previews = Vec::new();
        
        match self.status {
            Status::SetPosition => {
                // 在鼠标位置显示预览表格框架
                let preview_size = 30.0; // 预览大小
                let pos = ctx.effective_point();
                
                // 绘制简单的网格预览
                let col_width = preview_size / self.columns as f64;
                let row_height = preview_size / self.rows as f64;
                
                // 垂直线
                for i in 0..=self.columns {
                    let x = pos.x + i as f64 * col_width;
                    let line = Line::new(
                        Point2::new(x, pos.y),
                        Point2::new(x, pos.y - preview_size),
                    );
                    previews.push(PreviewGeometry::new(Geometry::Line(line)));
                }
                
                // 水平线
                for i in 0..=self.rows {
                    let y = pos.y - i as f64 * row_height;
                    let line = Line::new(
                        Point2::new(pos.x, y),
                        Point2::new(pos.x + preview_size, y),
                    );
                    previews.push(PreviewGeometry::new(Geometry::Line(line)));
                }
            }
            Status::SetCorner => {
                if let Some(table) = self.create_preview(ctx.effective_point()) {
                    previews.push(PreviewGeometry::new(Geometry::Table(table)));
                }
            }
            _ => {}
        }
        
        previews
    }
}
