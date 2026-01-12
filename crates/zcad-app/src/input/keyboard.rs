//! 键盘快捷键处理

use eframe::egui;
use zcad_ui::state::{DrawingTool, UiState};

/// 键盘事件结果
pub struct KeyboardResult {
    pub should_new_document: bool,
    pub should_open_dialog: bool,
    pub should_save: bool,
    pub should_save_as: bool,
    pub should_delete: bool,
    pub should_undo: bool,
    pub should_redo: bool,
    pub should_zoom_fit: bool,
}

impl Default for KeyboardResult {
    fn default() -> Self {
        Self {
            should_new_document: false,
            should_open_dialog: false,
            should_save: false,
            should_save_as: false,
            should_delete: false,
            should_undo: false,
            should_redo: false,
            should_zoom_fit: false,
        }
    }
}

/// 处理键盘快捷键
pub fn handle_keyboard_shortcuts(
    ctx: &egui::Context,
    ui_state: &mut UiState,
) -> KeyboardResult {
    let mut result = KeyboardResult::default();
    
    ctx.input(|i| {
        // 文件操作
        if i.modifiers.command && i.key_pressed(egui::Key::N) {
            result.should_new_document = true;
        }
        if i.modifiers.command && i.key_pressed(egui::Key::O) {
            result.should_open_dialog = true;
        }
        if i.modifiers.command && i.key_pressed(egui::Key::S) {
            if i.modifiers.shift {
                result.should_save_as = true;
            } else {
                result.should_save = true;
            }
        }
        
        // 编辑操作
        if i.key_pressed(egui::Key::Escape) {
            ui_state.cancel();
        }
        if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
            result.should_delete = true;
        }
        // 撤销 Ctrl+Z
        if i.modifiers.command && i.key_pressed(egui::Key::Z) && !i.modifiers.shift {
            result.should_undo = true;
        }
        // 重做 Ctrl+Y 或 Ctrl+Shift+Z
        if i.modifiers.command && (i.key_pressed(egui::Key::Y) || (i.key_pressed(egui::Key::Z) && i.modifiers.shift)) {
            result.should_redo = true;
        }
        
        // 绘图工具
        if i.key_pressed(egui::Key::L) {
            ui_state.set_tool(DrawingTool::Line);
        }
        if i.key_pressed(egui::Key::C) {
            ui_state.set_tool(DrawingTool::Circle);
        }
        if i.key_pressed(egui::Key::R) {
            ui_state.set_tool(DrawingTool::Rectangle);
        }
        if i.key_pressed(egui::Key::Space) {
            ui_state.set_tool(DrawingTool::Select);
        }
        
        // 视图操作（确保不与 Ctrl+Z 撤销冲突）
        if i.key_pressed(egui::Key::Z) && !i.modifiers.command && !i.modifiers.ctrl {
            result.should_zoom_fit = true;
        }
        if i.key_pressed(egui::Key::G) {
            ui_state.show_grid = !ui_state.show_grid;
        }
        if i.key_pressed(egui::Key::F3) {
            ui_state.snap_state.enabled = !ui_state.snap_state.enabled;
            let status = if ui_state.snap_state.enabled { "捕捉已启用" } else { "捕捉已禁用" };
            ui_state.status_message = status.to_string();
        }
        if i.key_pressed(egui::Key::F8) {
            ui_state.ortho_mode = !ui_state.ortho_mode;
            let status = if ui_state.ortho_mode { "正交模式已启用" } else { "正交模式已禁用" };
            ui_state.status_message = status.to_string();
        }
        // 圆弧快捷键
        if i.key_pressed(egui::Key::A) {
            ui_state.set_tool(DrawingTool::Arc);
        }
        // 多段线快捷键
        if i.key_pressed(egui::Key::P) {
            ui_state.set_tool(DrawingTool::Polyline);
        }
    });
    
    result
}
