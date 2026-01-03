//! 工具栏

use crate::state::{DrawingTool, UiState};

/// 渲染工具栏
pub fn show_toolbar(ctx: &egui::Context, ui_state: &mut UiState) {
    egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 4.0;

            // 文件操作
            if ui.button("📄 New").clicked() {
                // TODO: 新建文件
            }
            if ui.button("📂 Open").clicked() {
                // TODO: 打开文件
            }
            if ui.button("💾 Save").clicked() {
                // TODO: 保存文件
            }

            ui.separator();

            // 绘图工具
            tool_button(ui, ui_state, DrawingTool::Select, "⬚", "Select (Space)");
            tool_button(ui, ui_state, DrawingTool::Line, "╱", "Line (L)");
            tool_button(ui, ui_state, DrawingTool::Circle, "○", "Circle (C)");
            tool_button(ui, ui_state, DrawingTool::Arc, "◠", "Arc (A)");
            tool_button(ui, ui_state, DrawingTool::Polyline, "⌇", "Polyline (P)");
            tool_button(ui, ui_state, DrawingTool::Rectangle, "▭", "Rectangle (R)");
            tool_button(ui, ui_state, DrawingTool::Point, "•", "Point (.)");

            ui.separator();

            // 修改工具
            if ui.button("↔ Move").clicked() {
                // TODO
            }
            if ui.button("⎘ Copy").clicked() {
                // TODO
            }
            if ui.button("↻ Rotate").clicked() {
                // TODO
            }
            if ui.button("⤢ Scale").clicked() {
                // TODO
            }

            ui.separator();

            // 视图控制
            if ui
                .button(if ui_state.ortho_mode { "⊥ ON" } else { "⊥ OFF" })
                .on_hover_text("Ortho Mode (F8)")
                .clicked()
            {
                ui_state.ortho_mode = !ui_state.ortho_mode;
            }

            if ui
                .button(if ui_state.show_grid { "# ON" } else { "# OFF" })
                .on_hover_text("Toggle Grid")
                .clicked()
            {
                ui_state.show_grid = !ui_state.show_grid;
            }
        });
    });
}

fn tool_button(
    ui: &mut egui::Ui,
    ui_state: &mut UiState,
    tool: DrawingTool,
    icon: &str,
    tooltip: &str,
) {
    let selected = ui_state.current_tool == tool;

    let button = egui::Button::new(icon).selected(selected);

    if ui.add(button).on_hover_text(tooltip).clicked() {
        ui_state.set_tool(tool);
    }
}

