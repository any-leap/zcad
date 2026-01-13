//! 现代化顶部菜单栏

use eframe::egui;
use zcad_module::workspace::WorkspaceType;
use zcad_ui::state::DrawingTool;

use crate::theme::THEME;
use crate::ui::workspace_selector::workspace_display_name_zh;

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
    /// 切换到指定工作空间
    pub switch_workspace: Option<WorkspaceType>,
    /// 显示工作空间选择器
    pub show_workspace_selector: bool,
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
            switch_workspace: None,
            show_workspace_selector: false,
        }
    }
}

/// 显示顶部菜单
#[allow(deprecated)]
pub fn show_menu(ctx: &egui::Context, show_grid: bool, ortho_mode: bool) -> MenuResult {
    let mut result = MenuResult::default();
    let theme = &*THEME;
    let c = &theme.colors;
    
    egui::TopBottomPanel::top("menu")
        .frame(theme.header_frame())
        .show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // 文件菜单
                ui.menu_button(
                    egui::RichText::new("文件").color(c.text_primary),
                    |ui| {
                        if menu_item(ui, "新建", Some("Ctrl+N")) {
                            result.new_document = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, "打开", Some("Ctrl+O")) {
                            result.open_dialog = true;
                            ui.close_menu();
                        }
                        if menu_item(ui, "保存", Some("Ctrl+S")) {
                            result.save = true;
                            ui.close_menu();
                        }
                        if menu_item(ui, "另存为", Some("Ctrl+Shift+S")) {
                            result.save_as = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, "退出", None) {
                            result.exit = true;
                            ui.close_menu();
                        }
                    }
                );
                
                // 编辑菜单
                ui.menu_button(
                    egui::RichText::new("编辑").color(c.text_primary),
                    |ui| {
                        if menu_item(ui, "撤销", Some("Ctrl+Z")) {
                            result.undo = true;
                            ui.close_menu();
                        }
                        if menu_item(ui, "重做", Some("Ctrl+Y")) {
                            result.redo = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, "删除", Some("Del")) {
                            result.delete = true;
                            ui.close_menu();
                        }
                    }
                );
                
                // 视图菜单
                ui.menu_button(
                    egui::RichText::new("视图").color(c.text_primary),
                    |ui| {
                        if menu_item(ui, "缩放至全部", Some("Z")) {
                            result.zoom_fit = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item_toggle(ui, "网格", Some("G"), show_grid) {
                            result.toggle_grid = true;
                            ui.close_menu();
                        }
                        if menu_item_toggle(ui, "正交模式", Some("F8"), ortho_mode) {
                            result.toggle_ortho = true;
                            ui.close_menu();
                        }
                    }
                );
                
                // 绘图菜单
                ui.menu_button(
                    egui::RichText::new("绘图").color(c.text_primary),
                    |ui| {
                        if menu_item(ui, "直线", Some("L")) {
                            result.set_tool = Some(DrawingTool::Line);
                            ui.close_menu();
                        }
                        if menu_item(ui, "圆", Some("C")) {
                            result.set_tool = Some(DrawingTool::Circle);
                            ui.close_menu();
                        }
                        if menu_item(ui, "矩形", Some("R")) {
                            result.set_tool = Some(DrawingTool::Rectangle);
                            ui.close_menu();
                        }
                        if menu_item(ui, "圆弧", Some("A")) {
                            result.set_tool = Some(DrawingTool::Arc);
                            ui.close_menu();
                        }
                        if menu_item(ui, "多段线", Some("PL")) {
                            result.set_tool = Some(DrawingTool::Polyline);
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.menu_button(
                            egui::RichText::new("标注").color(c.text_primary),
                            |ui| {
                                if menu_item(ui, "线性标注", Some("DIM")) {
                                    result.set_tool = Some(DrawingTool::Dimension);
                                    ui.close_menu();
                                }
                                if menu_item(ui, "半径标注", Some("DIMR")) {
                                    result.set_tool = Some(DrawingTool::DimensionRadius);
                                    ui.close_menu();
                                }
                                if menu_item(ui, "直径标注", Some("DIMD")) {
                                    result.set_tool = Some(DrawingTool::DimensionDiameter);
                                    ui.close_menu();
                                }
                            }
                        );
                    }
                );
                
                // 帮助菜单
                ui.menu_button(
                    egui::RichText::new("帮助").color(c.text_primary),
                    |ui| {
                        if menu_item(ui, "关于 ZCAD", None) {
                            // TODO: 显示关于对话框
                            ui.close_menu();
                        }
                    }
                );
            });
        });
    
    result
}

/// 菜单项 - 紧凑设计
fn menu_item(ui: &mut egui::Ui, label: &str, shortcut: Option<&str>) -> bool {
    let theme = &*THEME;
    let c = &theme.colors;
    
    // 构建带快捷键的文本
    let text = if let Some(sc) = shortcut {
        format!("{}  \t{}", label, sc)
    } else {
        label.to_string()
    };
    
    ui.add(
        egui::Button::new(
            egui::RichText::new(&text)
                .color(c.text_primary)
                .size(12.0)
        )
        .fill(egui::Color32::TRANSPARENT)
        .min_size(egui::vec2(100.0, 22.0))
    ).clicked()
}

/// 带开关状态的菜单项 - 紧凑设计
fn menu_item_toggle(ui: &mut egui::Ui, label: &str, shortcut: Option<&str>, enabled: bool) -> bool {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let check = if enabled { "✓ " } else { "   " };
    
    // 构建带快捷键的文本
    let text = if let Some(sc) = shortcut {
        format!("{}{}  \t{}", check, label, sc)
    } else {
        format!("{}{}", check, label)
    };
    
    ui.add(
        egui::Button::new(
            egui::RichText::new(&text)
                .color(if enabled { c.accent_secondary } else { c.text_primary })
                .size(12.0)
        )
        .fill(egui::Color32::TRANSPARENT)
        .min_size(egui::vec2(100.0, 22.0))
    ).clicked()
}
