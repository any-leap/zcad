//! 顶部菜单栏

use eframe::egui;
use zcad_ui::state::DrawingTool;

/// 菜单操作结果
pub struct MenuResult {
    pub new_document: bool,
    pub open_dialog: bool,
    pub save: bool,
    pub save_as: bool,
    pub exit: bool,
    pub delete: bool,
    pub undo: bool,
    pub redo: bool,
    pub zoom_fit: bool,
    pub toggle_grid: bool,
    pub toggle_ortho: bool,
    pub set_tool: Option<DrawingTool>,
}

impl Default for MenuResult {
    fn default() -> Self {
        Self {
            new_document: false,
            open_dialog: false,
            save: false,
            save_as: false,
            exit: false,
            delete: false,
            undo: false,
            redo: false,
            zoom_fit: false,
            toggle_grid: false,
            toggle_ortho: false,
            set_tool: None,
        }
    }
}

/// 显示顶部菜单
#[allow(deprecated)]
pub fn show_menu(ctx: &egui::Context, show_grid: bool, ortho_mode: bool) -> MenuResult {
    let mut result = MenuResult::default();
    
    egui::TopBottomPanel::top("menu").show(ctx, |ui| {
        egui::menu::bar(ui, |ui| {
            ui.menu_button("文件", |ui| {
                if ui.button("📄 新建 (Ctrl+N)").clicked() {
                    result.new_document = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("📂 打开 (Ctrl+O)").clicked() {
                    result.open_dialog = true;
                    ui.close_menu();
                }
                if ui.button("💾 保存 (Ctrl+S)").clicked() {
                    result.save = true;
                    ui.close_menu();
                }
                if ui.button("💾 另存为 (Ctrl+Shift+S)").clicked() {
                    result.save_as = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("🚪 退出").clicked() {
                    result.exit = true;
                    ui.close_menu();
                }
            });
            ui.menu_button("编辑", |ui| {
                if ui.button("🗑 删除 (Del)").clicked() {
                    result.delete = true;
                    ui.close_menu();
                }
                ui.separator();
                if ui.button("↩ 撤销 (Ctrl+Z)").clicked() {
                    result.undo = true;
                    ui.close_menu();
                }
                if ui.button("↪ 重做 (Ctrl+Y)").clicked() {
                    result.redo = true;
                    ui.close_menu();
                }
            });
            ui.menu_button("视图", |ui| {
                if ui.button("📐 缩放至全部 (Z)").clicked() {
                    result.zoom_fit = true;
                    ui.close_menu();
                }
                if ui.button(format!("{} 网格 (G)", if show_grid { "☑" } else { "☐" })).clicked() {
                    result.toggle_grid = true;
                    ui.close_menu();
                }
                if ui.button(format!("{} 正交 (F8)", if ortho_mode { "☑" } else { "☐" })).clicked() {
                    result.toggle_ortho = true;
                    ui.close_menu();
                }
            });
            ui.menu_button("绘图", |ui| {
                if ui.button("╱ 直线 (L)").clicked() {
                    result.set_tool = Some(DrawingTool::Line);
                    ui.close_menu();
                }
                if ui.button("○ 圆 (C)").clicked() {
                    result.set_tool = Some(DrawingTool::Circle);
                    ui.close_menu();
                }
                if ui.button("▭ 矩形 (R)").clicked() {
                    result.set_tool = Some(DrawingTool::Rectangle);
                    ui.close_menu();
                }
                if ui.button("◠ 圆弧 (A)").clicked() {
                    result.set_tool = Some(DrawingTool::Arc);
                    ui.close_menu();
                }
                if ui.button("⌇ 多段线 (PL)").clicked() {
                    result.set_tool = Some(DrawingTool::Polyline);
                    ui.close_menu();
                }
                ui.separator();
                ui.menu_button("标注", |ui| {
                    if ui.button("📏 线性标注 (DIM)").clicked() {
                        result.set_tool = Some(DrawingTool::Dimension);
                        ui.close_menu();
                    }
                    if ui.button("⊛ 半径标注 (DIMR)").clicked() {
                        result.set_tool = Some(DrawingTool::DimensionRadius);
                        ui.close_menu();
                    }
                    if ui.button("⊚ 直径标注 (DIMD)").clicked() {
                        result.set_tool = Some(DrawingTool::DimensionDiameter);
                        ui.close_menu();
                    }
                });
            });
        });
    });
    
    result
}
