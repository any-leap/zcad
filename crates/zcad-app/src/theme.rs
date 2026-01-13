//! ZCAD 现代 UI 主题系统
//! 
//! 采用专业 CAD 软件的深色设计语言（参考 Blender/Fusion 360）

use eframe::egui::{self, Color32, FontId, CornerRadius, Shadow, Stroke, Vec2, FontFamily, Margin, StrokeKind};

/// 主题配色方案
pub struct ThemeColors {
    // 背景色层次
    pub bg_base: Color32,           // 最深的背景 (绘图区)
    pub bg_panel: Color32,          // 面板背景
    pub bg_header: Color32,         // 标题栏/工具栏背景
    pub bg_elevated: Color32,       // 悬浮元素背景
    pub bg_input: Color32,          // 输入框背景
    
    // 边框和分隔
    pub border_subtle: Color32,     // 微妙边框
    pub border_normal: Color32,     // 普通边框
    pub border_strong: Color32,     // 强调边框
    
    // 文字颜色
    pub text_primary: Color32,      // 主要文字
    pub text_secondary: Color32,    // 次要文字
    pub text_muted: Color32,        // 静默文字
    pub text_accent: Color32,       // 强调文字
    
    // 交互状态
    pub accent_primary: Color32,    // 主强调色 (选中/活动)
    pub accent_secondary: Color32,  // 次强调色
    pub hover: Color32,             // 悬停背景
    pub active: Color32,            // 按下/激活背景
    pub selected: Color32,          // 选中项背景
    
    // 功能色
    pub success: Color32,           // 成功/确认
    pub warning: Color32,           // 警告
    pub error: Color32,             // 错误
    pub info: Color32,              // 信息
    
    // CAD 特有色
    pub grid_major: Color32,        // 主网格线
    pub grid_minor: Color32,        // 次网格线
    pub snap_marker: Color32,       // 捕捉标记
    pub crosshair: Color32,         // 十字光标
}

impl Default for ThemeColors {
    fn default() -> Self {
        // 深邃蓝色调主题 - 专业 CAD 风格
        Self {
            // 背景层次 - 冷色调深色
            bg_base: Color32::from_rgb(18, 20, 28),      // 深邃的绘图区
            bg_panel: Color32::from_rgb(28, 31, 42),     // 面板背景
            bg_header: Color32::from_rgb(35, 38, 52),    // 工具栏
            bg_elevated: Color32::from_rgb(42, 46, 62),  // 弹出层
            bg_input: Color32::from_rgb(22, 24, 34),     // 输入框
            
            // 边框
            border_subtle: Color32::from_rgb(45, 48, 65),
            border_normal: Color32::from_rgb(55, 60, 80),
            border_strong: Color32::from_rgb(70, 75, 100),
            
            // 文字
            text_primary: Color32::from_rgb(230, 233, 240),
            text_secondary: Color32::from_rgb(170, 175, 190),
            text_muted: Color32::from_rgb(110, 115, 135),
            text_accent: Color32::from_rgb(100, 180, 255),
            
            // 交互 - 采用蓝色系主色调
            accent_primary: Color32::from_rgb(56, 139, 253),   // GitHub 蓝
            accent_secondary: Color32::from_rgb(88, 166, 255),
            hover: Color32::from_rgb(45, 50, 68),
            active: Color32::from_rgb(50, 55, 75),
            selected: Color32::from_rgb(40, 70, 120),
            
            // 功能色
            success: Color32::from_rgb(46, 160, 67),
            warning: Color32::from_rgb(210, 153, 34),
            error: Color32::from_rgb(218, 54, 51),
            info: Color32::from_rgb(56, 139, 253),
            
            // CAD 特有
            grid_major: Color32::from_rgba_premultiplied(80, 85, 105, 80),
            grid_minor: Color32::from_rgba_premultiplied(50, 55, 75, 40),
            snap_marker: Color32::from_rgb(255, 200, 50),
            crosshair: Color32::from_rgb(255, 100, 80),
        }
    }
}

/// 主题间距常量
pub struct ThemeSpacing {
    pub tiny: f32,      // 2
    pub small: f32,     // 4
    pub medium: f32,    // 8
    pub large: f32,     // 12
    pub xlarge: f32,    // 16
    pub xxlarge: f32,   // 24
}

impl Default for ThemeSpacing {
    fn default() -> Self {
        Self {
            tiny: 2.0,
            small: 4.0,
            medium: 8.0,
            large: 12.0,
            xlarge: 16.0,
            xxlarge: 24.0,
        }
    }
}

/// 主题圆角
pub struct ThemeRounding {
    pub none: CornerRadius,
    pub small: CornerRadius,
    pub medium: CornerRadius,
    pub large: CornerRadius,
    pub full: CornerRadius,
}

impl Default for ThemeRounding {
    fn default() -> Self {
        Self {
            none: CornerRadius::ZERO,
            small: CornerRadius::same(3),
            medium: CornerRadius::same(6),
            large: CornerRadius::same(10),
            full: CornerRadius::same(100),
        }
    }
}

/// 完整主题配置
pub struct Theme {
    pub colors: ThemeColors,
    pub spacing: ThemeSpacing,
    pub rounding: ThemeRounding,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            colors: ThemeColors::default(),
            spacing: ThemeSpacing::default(),
            rounding: ThemeRounding::default(),
        }
    }
}

impl Theme {
    /// 应用主题到 egui Context
    pub fn apply(&self, ctx: &egui::Context) {
        let mut visuals = egui::Visuals::dark();
        let c = &self.colors;
        
        // 窗口样式
        visuals.window_fill = c.bg_elevated;
        visuals.window_stroke = Stroke::new(1.0, c.border_normal);
        visuals.window_shadow = Shadow {
            offset: [0, 4],
            blur: 12,
            spread: 0,
            color: Color32::from_black_alpha(100),
        };
        
        // 面板样式
        visuals.panel_fill = c.bg_panel;
        
        // 极端背景 (用于绘图区等)
        visuals.extreme_bg_color = c.bg_base;
        visuals.faint_bg_color = c.bg_input;
        
        // 按钮和控件
        visuals.widgets.noninteractive.bg_fill = c.bg_panel;
        visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, c.text_secondary);
        visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, c.border_subtle);
        
        visuals.widgets.inactive.bg_fill = c.bg_header;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, c.text_primary);
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, c.border_subtle);
        
        visuals.widgets.hovered.bg_fill = c.hover;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, c.text_primary);
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, c.border_normal);
        
        visuals.widgets.active.bg_fill = c.active;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, c.text_primary);
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, c.accent_primary);
        
        visuals.widgets.open.bg_fill = c.selected;
        visuals.widgets.open.fg_stroke = Stroke::new(1.0, c.text_primary);
        visuals.widgets.open.bg_stroke = Stroke::new(1.0, c.accent_primary);
        
        // 选择高亮
        visuals.selection.bg_fill = c.selected;
        visuals.selection.stroke = Stroke::new(1.0, c.accent_primary);
        
        // 超链接
        visuals.hyperlink_color = c.accent_primary;
        
        // 滚动条
        visuals.widgets.inactive.expansion = 0.0;
        
        // 条纹背景（用于列表等）
        visuals.striped = true;
        
        // 文本光标
        visuals.text_cursor.stroke = Stroke::new(2.0, c.accent_primary);
        
        ctx.set_visuals(visuals);
        
        // 设置全局样式
        let mut style = (*ctx.style()).clone();
        style.spacing.item_spacing = Vec2::new(self.spacing.medium, self.spacing.small);
        style.spacing.window_margin = Margin::same(self.spacing.large as i8);
        style.spacing.button_padding = Vec2::new(self.spacing.large, self.spacing.small);
        style.spacing.indent = self.spacing.xlarge;
        style.spacing.icon_width = 18.0;
        style.spacing.icon_spacing = self.spacing.small;
        
        // 交互样式
        style.interaction.tooltip_delay = 0.3;
        
        ctx.set_style(style);
    }
    
    /// 获取工具按钮的 Frame
    pub fn tool_button_frame(&self, selected: bool) -> egui::Frame {
        let c = &self.colors;
        egui::Frame::NONE
            .fill(if selected { c.selected } else { Color32::TRANSPARENT })
            .stroke(Stroke::new(1.0, if selected { c.accent_primary } else { Color32::TRANSPARENT }))
            .corner_radius(self.rounding.small)
            .inner_margin(Margin::same(self.spacing.small as i8))
    }
    
    /// 获取面板的 Frame
    pub fn panel_frame(&self) -> egui::Frame {
        let c = &self.colors;
        egui::Frame::NONE
            .fill(c.bg_panel)
            .stroke(Stroke::new(1.0, c.border_subtle))
    }
    
    /// 获取分组容器的 Frame
    pub fn group_frame(&self) -> egui::Frame {
        let c = &self.colors;
        egui::Frame::NONE
            .fill(c.bg_header)
            .stroke(Stroke::new(1.0, c.border_subtle))
            .corner_radius(self.rounding.small)
            .inner_margin(Margin::same(self.spacing.medium as i8))
    }
    
    /// 获取标题栏的 Frame
    pub fn header_frame(&self) -> egui::Frame {
        let c = &self.colors;
        egui::Frame::NONE
            .fill(c.bg_header)
            .stroke(Stroke::new(1.0, c.border_subtle))
            .inner_margin(Margin::symmetric(self.spacing.medium as i8, self.spacing.small as i8))
    }
    
    /// 获取状态栏的 Frame
    pub fn statusbar_frame(&self) -> egui::Frame {
        let c = &self.colors;
        egui::Frame::NONE
            .fill(c.bg_header)
            .stroke(Stroke::new(1.0, c.border_subtle))
            .inner_margin(Margin::symmetric(self.spacing.large as i8, self.spacing.small as i8))
    }
    
    /// 获取绘图区的 Frame
    pub fn canvas_frame(&self) -> egui::Frame {
        let c = &self.colors;
        egui::Frame::NONE.fill(c.bg_base)
    }
}

/// 全局主题实例 (线程安全的延迟初始化)
pub static THEME: std::sync::LazyLock<Theme> = std::sync::LazyLock::new(Theme::default);

/// 图标常量 - 使用兼容性更好的符号
pub mod icons {
    // 工具图标 - 使用简单符号或文字
    pub const SELECT: &str = "▸";
    pub const LINE: &str = "/";
    pub const CIRCLE: &str = "O";
    pub const RECTANGLE: &str = "□";
    pub const ARC: &str = "⌒";
    pub const POLYLINE: &str = "~";
    
    // 标注
    pub const DIMENSION: &str = "↔";
    pub const RADIUS: &str = "R";
    pub const DIAMETER: &str = "Ø";
    
    // 操作
    pub const DELETE: &str = "×";
    pub const UNDO: &str = "←";
    pub const REDO: &str = "→";
    pub const ORTHO: &str = "⊥";
    pub const GRID: &str = "#";
    pub const ZOOM_FIT: &str = "□";
    pub const SNAP: &str = "+";
    
    // 文件
    pub const NEW: &str = "+";
    pub const OPEN: &str = "📂";
    pub const SAVE: &str = "💾";
    pub const EXIT: &str = "×";
    
    // 状态
    pub const CHECK: &str = "√";
    pub const UNCHECK: &str = "○";
}

/// 按钮变体
#[derive(Default, Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    #[default]
    Default,
    Primary,
    Ghost,
    Danger,
}

/// 美化的图标按钮
pub fn icon_button(ui: &mut egui::Ui, icon: &str, tooltip: &str, selected: bool) -> egui::Response {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let desired_size = Vec2::splat(28.0);
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    
    if ui.is_rect_visible(rect) {
        // 背景
        let bg_color = if selected {
            c.selected
        } else if response.hovered() {
            c.hover
        } else {
            Color32::TRANSPARENT
        };
        
        let stroke_color = if selected {
            c.accent_primary
        } else if response.hovered() {
            c.border_normal
        } else {
            Color32::TRANSPARENT
        };
        
        ui.painter().rect(
            rect,
            theme.rounding.small,
            bg_color,
            Stroke::new(1.0, stroke_color),
            StrokeKind::Outside,
        );
        
        // 图标
        let text_color = if selected {
            c.accent_secondary
        } else if response.hovered() {
            c.text_primary
        } else {
            c.text_secondary
        };
        
        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(16.0),
            text_color,
        );
    }
    
    response.on_hover_text(tooltip)
}

/// 带文字的工具按钮
pub fn tool_button(ui: &mut egui::Ui, icon: &str, label: &str, selected: bool) -> egui::Response {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let text_layout = ui.painter().layout_no_wrap(
        label.to_string(),
        FontId::proportional(12.0),
        c.text_primary,
    );
    
    let icon_size = 16.0;
    let padding = theme.spacing.medium;
    let gap = theme.spacing.small;
    
    let desired_size = Vec2::new(
        padding * 2.0 + icon_size + gap + text_layout.rect.width(),
        28.0,
    );
    
    let (rect, response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    
    if ui.is_rect_visible(rect) {
        // 背景
        let bg_color = if selected {
            c.selected
        } else if response.hovered() {
            c.hover
        } else {
            Color32::TRANSPARENT
        };
        
        let stroke_color = if selected {
            c.accent_primary
        } else if response.hovered() {
            c.border_normal
        } else {
            Color32::TRANSPARENT
        };
        
        ui.painter().rect(
            rect,
            theme.rounding.small,
            bg_color,
            Stroke::new(1.0, stroke_color),
            StrokeKind::Outside,
        );
        
        // 图标
        let icon_pos = egui::pos2(rect.left() + padding + icon_size / 2.0, rect.center().y);
        let text_color = if selected {
            c.accent_secondary
        } else if response.hovered() {
            c.text_primary
        } else {
            c.text_secondary
        };
        
        ui.painter().text(
            icon_pos,
            egui::Align2::CENTER_CENTER,
            icon,
            FontId::proportional(14.0),
            text_color,
        );
        
        // 文字
        let text_pos = egui::pos2(
            rect.left() + padding + icon_size + gap,
            rect.center().y - text_layout.rect.height() / 2.0,
        );
        ui.painter().galley(text_pos, text_layout, text_color);
    }
    
    response
}

/// 分隔线（垂直）
pub fn vseparator(ui: &mut egui::Ui) {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let height = ui.available_height().min(24.0);
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, height), egui::Sense::hover());
    
    if ui.is_rect_visible(rect) {
        ui.painter().line_segment(
            [rect.center_top(), rect.center_bottom()],
            Stroke::new(1.0, c.border_subtle),
        );
    }
    
    ui.add_space(theme.spacing.medium);
}

/// 分组标题
pub fn group_header(ui: &mut egui::Ui, title: &str) {
    let theme = &*THEME;
    let c = &theme.colors;
    
    ui.add_space(theme.spacing.small);
    ui.horizontal(|ui| {
        ui.add_space(theme.spacing.tiny);
        ui.label(
            egui::RichText::new(title)
                .color(c.text_muted)
                .size(11.0)
                .strong()
        );
    });
    ui.add_space(theme.spacing.tiny);
}

/// 折叠面板
pub fn collapsing_group(
    ui: &mut egui::Ui, 
    title: &str, 
    default_open: bool,
    add_contents: impl FnOnce(&mut egui::Ui),
) {
    let theme = &*THEME;
    let c = &theme.colors;
    
    egui::CollapsingHeader::new(
        egui::RichText::new(title)
            .color(c.text_primary)
            .size(12.0)
    )
    .default_open(default_open)
    .show(ui, |ui| {
        ui.add_space(theme.spacing.tiny);
        add_contents(ui);
        ui.add_space(theme.spacing.small);
    });
}

/// 状态指示器
pub fn status_indicator(ui: &mut egui::Ui, enabled: bool, label: &str, tooltip: &str) -> egui::Response {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let text = egui::RichText::new(label)
        .size(11.0)
        .color(if enabled { c.accent_primary } else { c.text_muted });
    
    let response = ui.add(
        egui::Label::new(text)
            .selectable(false)
            .sense(egui::Sense::click())
    );
    
    response.on_hover_text(tooltip)
}

/// 属性行
pub fn property_row(ui: &mut egui::Ui, label: &str, value: &str) {
    let theme = &*THEME;
    let c = &theme.colors;
    
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(label)
                .color(c.text_muted)
                .size(11.0)
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(value)
                    .color(c.text_primary)
                    .size(11.0)
            );
        });
    });
}

/// 坐标显示
pub fn coordinate_display(ui: &mut egui::Ui, x: f64, y: f64) {
    let theme = &*THEME;
    let c = &theme.colors;
    
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("X")
                .color(c.error)
                .size(11.0)
                .strong()
        );
        ui.label(
            egui::RichText::new(format!("{:>9.3}", x))
                .color(c.text_primary)
                .size(11.0)
                .family(FontFamily::Monospace)
        );
        
        ui.add_space(theme.spacing.medium);
        
        ui.label(
            egui::RichText::new("Y")
                .color(c.success)
                .size(11.0)
                .strong()
        );
        ui.label(
            egui::RichText::new(format!("{:>9.3}", y))
                .color(c.text_primary)
                .size(11.0)
                .family(FontFamily::Monospace)
        );
    });
}
