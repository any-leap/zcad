//! 工具栏

use eframe::egui;
use zcad_ui::state::DrawingTool;

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
    
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            if ui.selectable_label(current_tool == DrawingTool::Select, "⬚ 选择").clicked() {
                result.set_tool = Some(DrawingTool::Select);
            }
            ui.separator();
            if ui.selectable_label(current_tool == DrawingTool::Line, "╱ 直线").clicked() {
                result.set_tool = Some(DrawingTool::Line);
            }
            if ui.selectable_label(current_tool == DrawingTool::Circle, "○ 圆").clicked() {
                result.set_tool = Some(DrawingTool::Circle);
            }
            if ui.selectable_label(current_tool == DrawingTool::Rectangle, "▭ 矩形").clicked() {
                result.set_tool = Some(DrawingTool::Rectangle);
            }
            if ui.selectable_label(current_tool == DrawingTool::Arc, "◠ 圆弧").clicked() {
                result.set_tool = Some(DrawingTool::Arc);
            }
            if ui.selectable_label(current_tool == DrawingTool::Polyline, "⌇ 多段线").clicked() {
                result.set_tool = Some(DrawingTool::Polyline);
            }
            ui.separator();
            if ui.selectable_label(current_tool == DrawingTool::Dimension, "📏 标注").clicked() {
                result.set_tool = Some(DrawingTool::Dimension);
            }
            if ui.selectable_label(current_tool == DrawingTool::DimensionRadius, "⊛ 半径").clicked() {
                result.set_tool = Some(DrawingTool::DimensionRadius);
            }
            if ui.selectable_label(current_tool == DrawingTool::DimensionDiameter, "⊚ 直径").clicked() {
                result.set_tool = Some(DrawingTool::DimensionDiameter);
            }
            ui.separator();
            if ui.button("🗑").on_hover_text("删除选中").clicked() {
                result.delete = true;
            }
            if ui.button("↩").on_hover_text("撤销 (Ctrl+Z)").clicked() {
                result.undo = true;
            }
            if ui.button("↪").on_hover_text("重做 (Ctrl+Y)").clicked() {
                result.redo = true;
            }
            ui.separator();
            if ui.selectable_label(ortho_mode, "⊥").on_hover_text("正交模式 (F8)").clicked() {
                result.toggle_ortho = true;
            }
            if ui.selectable_label(show_grid, "#").on_hover_text("网格 (G)").clicked() {
                result.toggle_grid = true;
            }
            if ui.button("⊞").on_hover_text("缩放至全部 (Z)").clicked() {
                result.zoom_fit = true;
            }
        });
    });
    
    result
}
