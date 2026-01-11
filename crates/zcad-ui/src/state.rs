//! UI状态管理

use zcad_core::entity::EntityId;
use zcad_core::math::Point2;
use zcad_core::snap::{SnapConfig, SnapEngine, SnapPoint, SnapType};

/// 当前绘图工具
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawingTool {
    None,
    Select,
    Line,
    Circle,
    Arc,
    Polyline,
    Rectangle,
    Point,
    Text,
}

impl DrawingTool {
    pub fn name(&self) -> &'static str {
        match self {
            DrawingTool::None => "None",
            DrawingTool::Select => "Select",
            DrawingTool::Line => "Line",
            DrawingTool::Circle => "Circle",
            DrawingTool::Arc => "Arc",
            DrawingTool::Polyline => "Polyline",
            DrawingTool::Rectangle => "Rectangle",
            DrawingTool::Point => "Point",
            DrawingTool::Text => "Text",
        }
    }

    pub fn shortcut(&self) -> Option<&'static str> {
        match self {
            DrawingTool::Select => Some("Space"),
            DrawingTool::Line => Some("L"),
            DrawingTool::Circle => Some("C"),
            DrawingTool::Arc => Some("A"),
            DrawingTool::Polyline => Some("P"),
            DrawingTool::Rectangle => Some("R"),
            DrawingTool::Point => Some("."),
            DrawingTool::Text => Some("T"),
            DrawingTool::None => None,
        }
    }
}

/// 捕捉模式（保留向后兼容，实际使用SnapEngine）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SnapMode {
    pub endpoint: bool,
    pub midpoint: bool,
    pub center: bool,
    pub intersection: bool,
    pub perpendicular: bool,
    pub tangent: bool,
    pub nearest: bool,
    pub grid: bool,
}

impl Default for SnapMode {
    fn default() -> Self {
        Self {
            endpoint: true,
            midpoint: true,
            center: true,
            intersection: true,
            perpendicular: false,
            tangent: false,
            nearest: false,
            grid: false,
        }
    }
}

/// 当前捕捉状态
#[derive(Debug, Clone)]
pub struct SnapState {
    /// 捕捉引擎
    engine: SnapEngine,
    /// 当前捕捉到的点
    pub current_snap: Option<SnapPoint>,
    /// 是否启用捕捉
    pub enabled: bool,
}

impl SnapState {
    pub fn new() -> Self {
        Self {
            engine: SnapEngine::default(),
            current_snap: None,
            enabled: true,
        }
    }

    /// 获取捕捉引擎的可变引用
    pub fn engine_mut(&mut self) -> &mut SnapEngine {
        &mut self.engine
    }

    /// 获取捕捉引擎的引用
    pub fn engine(&self) -> &SnapEngine {
        &self.engine
    }

    /// 获取捕捉配置
    pub fn config(&self) -> &SnapConfig {
        self.engine.config()
    }

    /// 获取捕捉配置（可变）
    pub fn config_mut(&mut self) -> &mut SnapConfig {
        self.engine.config_mut()
    }

    /// 切换捕捉类型
    pub fn toggle_snap_type(&mut self, snap_type: SnapType) {
        self.engine.config_mut().enabled_types.toggle(snap_type);
    }

    /// 检查捕捉类型是否启用
    pub fn is_snap_type_enabled(&self, snap_type: SnapType) -> bool {
        self.engine.config().enabled_types.is_enabled(snap_type)
    }
}

impl Default for SnapState {
    fn default() -> Self {
        Self::new()
    }
}

/// 编辑状态
#[derive(Debug, Clone)]
pub enum EditState {
    /// 空闲
    Idle,
    /// 正在绘制
    Drawing {
        tool: DrawingTool,
        points: Vec<Point2>,
    },
    /// 选择中
    Selecting {
        start: Point2,
    },
    /// 移动选择的对象
    Moving {
        start: Point2,
        offset: Point2,
    },
    /// 等待命令输入
    Command {
        input: String,
    },
    /// 正在输入文本
    TextInput {
        position: Point2,
        content: String,
        height: f64,
    },
}

impl Default for EditState {
    fn default() -> Self {
        Self::Idle
    }
}

/// UI状态
#[derive(Debug)]
pub struct UiState {
    /// 当前工具
    pub current_tool: DrawingTool,

    /// 编辑状态
    pub edit_state: EditState,

    /// 选中的实体
    pub selected_entities: Vec<EntityId>,

    /// 鼠标在世界坐标中的位置（原始位置）
    pub mouse_world_pos: Point2,

    /// 捕捉状态
    pub snap_state: SnapState,

    /// 捕捉到的点（如果有）- 保留向后兼容
    pub snap_point: Option<Point2>,

    /// 捕捉模式 - 保留向后兼容
    pub snap_mode: SnapMode,

    /// 是否显示网格
    pub show_grid: bool,

    /// 网格间距
    pub grid_spacing: f64,

    /// 命令行输入
    pub command_input: String,

    /// 命令历史
    pub command_history: Vec<String>,

    /// 状态栏消息
    pub status_message: String,

    /// 是否显示图层面板
    pub show_layers_panel: bool,

    /// 是否显示属性面板
    pub show_properties_panel: bool,

    /// 正交模式
    pub ortho_mode: bool,
}

impl UiState {
    /// 获取实际使用的点（优先使用捕捉点）
    pub fn effective_point(&self) -> Point2 {
        if let Some(ref snap) = self.snap_state.current_snap {
            if self.snap_state.enabled {
                return snap.point;
            }
        }
        self.mouse_world_pos
    }

    /// 获取当前捕捉点信息
    pub fn current_snap(&self) -> Option<&SnapPoint> {
        if self.snap_state.enabled {
            self.snap_state.current_snap.as_ref()
        } else {
            None
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            current_tool: DrawingTool::Select,
            edit_state: EditState::Idle,
            selected_entities: Vec::new(),
            mouse_world_pos: Point2::origin(),
            snap_state: SnapState::default(),
            snap_point: None,
            snap_mode: SnapMode::default(),
            show_grid: true,
            grid_spacing: 10.0,
            command_input: String::new(),
            command_history: Vec::new(),
            status_message: "Ready".to_string(),
            show_layers_panel: true,
            show_properties_panel: true,
            ortho_mode: false,
        }
    }
}

impl UiState {
    /// 设置当前工具
    pub fn set_tool(&mut self, tool: DrawingTool) {
        self.current_tool = tool;
        self.edit_state = EditState::Idle;
        self.status_message = format!("{} tool selected", tool.name());
    }

    /// 取消当前操作
    pub fn cancel(&mut self) {
        self.edit_state = EditState::Idle;
        self.status_message = "Cancelled".to_string();
    }

    /// 清空选择
    pub fn clear_selection(&mut self) {
        self.selected_entities.clear();
    }

    /// 添加到选择
    pub fn add_to_selection(&mut self, id: EntityId) {
        if !self.selected_entities.contains(&id) {
            self.selected_entities.push(id);
        }
    }

    /// 从选择中移除
    pub fn remove_from_selection(&mut self, id: &EntityId) {
        self.selected_entities.retain(|e| e != id);
    }

    /// 切换选择状态
    pub fn toggle_selection(&mut self, id: EntityId) {
        if self.selected_entities.contains(&id) {
            self.remove_from_selection(&id);
        } else {
            self.add_to_selection(id);
        }
    }

    /// 执行命令
    pub fn execute_command(&mut self, command: &str) -> Option<Command> {
        let trimmed = command.trim().to_uppercase();

        if trimmed.is_empty() {
            return None;
        }

        // 添加到历史
        self.command_history.push(command.to_string());

        // 解析命令
        let cmd = match trimmed.as_str() {
            "L" | "LINE" => Some(Command::SetTool(DrawingTool::Line)),
            "C" | "CIRCLE" => Some(Command::SetTool(DrawingTool::Circle)),
            "A" | "ARC" => Some(Command::SetTool(DrawingTool::Arc)),
            "P" | "PL" | "PLINE" | "POLYLINE" => Some(Command::SetTool(DrawingTool::Polyline)),
            "R" | "REC" | "RECTANGLE" => Some(Command::SetTool(DrawingTool::Rectangle)),
            "T" | "TEXT" | "DTEXT" | "MTEXT" => Some(Command::SetTool(DrawingTool::Text)),
            "E" | "ERASE" | "DELETE" => Some(Command::DeleteSelected),
            "M" | "MOVE" => Some(Command::Move),
            "CO" | "COPY" => Some(Command::Copy),
            "RO" | "ROTATE" => Some(Command::Rotate),
            "SC" | "SCALE" => Some(Command::Scale),
            "MI" | "MIRROR" => Some(Command::Mirror),
            "Z" | "ZOOM" => Some(Command::ZoomExtents),
            "ZE" | "ZOOM EXTENTS" => Some(Command::ZoomExtents),
            "U" | "UNDO" => Some(Command::Undo),
            "REDO" => Some(Command::Redo),
            "ESC" => {
                self.cancel();
                None
            }
            _ => {
                self.status_message = format!("Unknown command: {}", command);
                None
            }
        };

        if let Some(ref c) = cmd {
            self.status_message = format!("Command: {:?}", c);
        }

        cmd
    }
}

/// 命令类型
#[derive(Debug, Clone)]
pub enum Command {
    SetTool(DrawingTool),
    DeleteSelected,
    Move,
    Copy,
    Rotate,
    Scale,
    Mirror,
    ZoomExtents,
    Undo,
    Redo,
}

