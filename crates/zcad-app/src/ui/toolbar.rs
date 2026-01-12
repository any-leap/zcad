//! 现代化工具栏
//! 
//! 采用图标按钮设计，清晰的分组

use eframe::egui;
use zcad_ui::state::DrawingTool;

use crate::theme::{self, icons, THEME};

/// 工具栏操作结果
pub struct ToolbarResult {
    pub set_tool: Option<DrawingTool>,
    pub delete: bool,
    pub undo: bool,
    pub redo: bool,
    pub toggle_ortho: bool,
    pub toggle_grid: bool,
    pub zoom_fit: bool,
}

impl Default for ToolbarResult {
    fn default() -> Self {
        Self {
            set_tool: None,
            delete: false,
            undo: false,
            redo: false,
            toggle_ortho: false,
            toggle_grid: false,
            zoom_fit: false,
        }
    }
}

/// 显示工具栏
pub fn show_toolbar(
    ctx: &egui::Context,
    current_tool: DrawingTool,
    ortho_mode: bool,
    show_grid: bool,
) -> ToolbarResult {
    let mut result = ToolbarResult::default();
    let theme = &*THEME;
    let c = &theme.colors;
    
    egui::TopBottomPanel::top("toolbar")
        .frame(theme.header_frame())
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.add_space(theme.spacing.small);
                
                // ===== 选择工具 =====
                if theme::tool_button(ui, icons::SELECT, "选择", current_tool == DrawingTool::Select).clicked() {
                    result.set_tool = Some(DrawingTool::Select);
                }
                
                theme::vseparator(ui);
                
                // ===== 绘图工具组 =====
                ui.horizontal(|ui| {
                    ui.add_space(theme.spacing.tiny);
                    
                    // 直线
                    if theme::icon_button(ui, icons::LINE, "直线 (L)", current_tool == DrawingTool::Line).clicked() {
                        result.set_tool = Some(DrawingTool::Line);
                    }
                    
                    // 圆
                    if theme::icon_button(ui, icons::CIRCLE, "圆 (C)", current_tool == DrawingTool::Circle).clicked() {
                        result.set_tool = Some(DrawingTool::Circle);
                    }
                    
                    // 矩形
                    if theme::icon_button(ui, icons::RECTANGLE, "矩形 (R)", current_tool == DrawingTool::Rectangle).clicked() {
                        result.set_tool = Some(DrawingTool::Rectangle);
                    }
                    
                    // 圆弧
                    if theme::icon_button(ui, icons::ARC, "圆弧 (A)", current_tool == DrawingTool::Arc).clicked() {
                        result.set_tool = Some(DrawingTool::Arc);
                    }
                    
                    // 多段线
                    if theme::icon_button(ui, icons::POLYLINE, "多段线 (PL)", current_tool == DrawingTool::Polyline).clicked() {
                        result.set_tool = Some(DrawingTool::Polyline);
                    }
                });
                
                theme::vseparator(ui);
                
                // ===== 标注工具组 =====
                ui.horizontal(|ui| {
                    ui.add_space(theme.spacing.tiny);
                    
                    // 线性标注
                    if theme::icon_button(ui, icons::DIMENSION, "线性标注 (DIM)", current_tool == DrawingTool::Dimension).clicked() {
                        result.set_tool = Some(DrawingTool::Dimension);
                    }
                    
                    // 半径标注
                    if theme::icon_button(ui, icons::RADIUS, "半径标注 (DIMR)", current_tool == DrawingTool::DimensionRadius).clicked() {
                        result.set_tool = Some(DrawingTool::DimensionRadius);
                    }
                    
                    // 直径标注
                    if theme::icon_button(ui, icons::DIAMETER, "直径标注 (DIMD)", current_tool == DrawingTool::DimensionDiameter).clicked() {
                        result.set_tool = Some(DrawingTool::DimensionDiameter);
                    }
                });
                
                theme::vseparator(ui);
                
                // ===== 编辑操作组 =====
                ui.horizontal(|ui| {
                    ui.add_space(theme.spacing.tiny);
                    
                    // 删除
                    if theme::icon_button(ui, icons::DELETE, "删除选中 (Del)", false).clicked() {
                        result.delete = true;
                    }
                    
                    // 撤销
                    if theme::icon_button(ui, icons::UNDO, "撤销 (Ctrl+Z)", false).clicked() {
                        result.undo = true;
                    }
                    
                    // 重做
                    if theme::icon_button(ui, icons::REDO, "重做 (Ctrl+Y)", false).clicked() {
                        result.redo = true;
                    }
                });
                
                theme::vseparator(ui);
                
                // ===== 视图选项组 =====
                ui.horizontal(|ui| {
                    ui.add_space(theme.spacing.tiny);
                    
                    // 正交模式
                    if theme::icon_button(ui, icons::ORTHO, "正交模式 (F8)", ortho_mode).clicked() {
                        result.toggle_ortho = true;
                    }
                    
                    // 网格
                    if theme::icon_button(ui, icons::GRID, "网格 (G)", show_grid).clicked() {
                        result.toggle_grid = true;
                    }
                    
                    // 缩放至全部
                    if theme::icon_button(ui, icons::ZOOM_FIT, "缩放至全部 (Z)", false).clicked() {
                        result.zoom_fit = true;
                    }
                });
                
                // 右侧状态显示
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.add_space(theme.spacing.medium);
                    
                    // 当前工具名称
                    ui.label(
                        egui::RichText::new(current_tool.name())
                            .color(c.text_accent)
                            .size(12.0)
                    );
                    
                    ui.label(
                        egui::RichText::new("当前工具:")
                            .color(c.text_muted)
                            .size(11.0)
                    );
                });
            });
        });
    
    result
}
