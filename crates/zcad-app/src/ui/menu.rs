//! 现代化顶部菜单栏

use eframe::egui;
use zcad_ui::state::DrawingTool;

use crate::theme::{icons, THEME};

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
                        if menu_item(ui, icons::NEW, "新建", Some("Ctrl+N")) {
                            result.new_document = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, icons::OPEN, "打开", Some("Ctrl+O")) {
                            result.open_dialog = true;
                            ui.close_menu();
                        }
                        if menu_item(ui, icons::SAVE, "保存", Some("Ctrl+S")) {
                            result.save = true;
                            ui.close_menu();
                        }
                        if menu_item(ui, icons::SAVE, "另存为", Some("Ctrl+Shift+S")) {
                            result.save_as = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, icons::EXIT, "退出", None) {
                            result.exit = true;
                            ui.close_menu();
                        }
                    }
                );
                
                // 编辑菜单
                ui.menu_button(
                    egui::RichText::new("编辑").color(c.text_primary),
                    |ui| {
                        if menu_item(ui, icons::UNDO, "撤销", Some("Ctrl+Z")) {
                            result.undo = true;
                            ui.close_menu();
                        }
                        if menu_item(ui, icons::REDO, "重做", Some("Ctrl+Y")) {
                            result.redo = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item(ui, icons::DELETE, "删除", Some("Del")) {
                            result.delete = true;
                            ui.close_menu();
                        }
                    }
                );
                
                // 视图菜单
                ui.menu_button(
                    egui::RichText::new("视图").color(c.text_primary),
                    |ui| {
                        if menu_item(ui, icons::ZOOM_FIT, "缩放至全部", Some("Z")) {
                            result.zoom_fit = true;
                            ui.close_menu();
                        }
                        ui.separator();
                        if menu_item_toggle(ui, icons::GRID, "网格", Some("G"), show_grid) {
                            result.toggle_grid = true;
                            ui.close_menu();
                        }
                        if menu_item_toggle(ui, icons::ORTHO, "正交模式", Some("F8"), ortho_mode) {
                            result.toggle_ortho = true;
                            ui.close_menu();
                        }
                    }
                );
                
                // 绘图菜单
                ui.menu_button(
                    egui::RichText::new("绘图").color(c.text_primary),
                    |ui| {
                        if menu_item(ui, icons::LINE, "直线", Some("L")) {
                            result.set_tool = Some(DrawingTool::Line);
                            ui.close_menu();
                        }
                        if menu_item(ui, icons::CIRCLE, "圆", Some("C")) {
                            result.set_tool = Some(DrawingTool::Circle);
                            ui.close_menu();
                        }
                        if menu_item(ui, icons::RECTANGLE, "矩形", Some("R")) {
                            result.set_tool = Some(DrawingTool::Rectangle);
                            ui.close_menu();
                        }
                        if menu_item(ui, icons::ARC, "圆弧", Some("A")) {
                            result.set_tool = Some(DrawingTool::Arc);
                            ui.close_menu();
                        }
                        if menu_item(ui, icons::POLYLINE, "多段线", Some("PL")) {
                            result.set_tool = Some(DrawingTool::Polyline);
                            ui.close_menu();
                        }
                        ui.separator();
                        ui.menu_button(
                            egui::RichText::new("标注").color(c.text_primary),
                            |ui| {
                                if menu_item(ui, icons::DIMENSION, "线性标注", Some("DIM")) {
                                    result.set_tool = Some(DrawingTool::Dimension);
                                    ui.close_menu();
                                }
                                if menu_item(ui, icons::RADIUS, "半径标注", Some("DIMR")) {
                                    result.set_tool = Some(DrawingTool::DimensionRadius);
                                    ui.close_menu();
                                }
                                if menu_item(ui, icons::DIAMETER, "直径标注", Some("DIMD")) {
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
                        if menu_item(ui, "ℹ", "关于 ZCAD", None) {
                            // TODO: 显示关于对话框
                            ui.close_menu();
                        }
                    }
                );
            });
        });
    
    result
}

/// 菜单项
fn menu_item(ui: &mut egui::Ui, icon: &str, label: &str, shortcut: Option<&str>) -> bool {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let button_text = format!("{}  {}", icon, label);
    
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::Button::new(
                egui::RichText::new(&button_text)
                    .color(c.text_primary)
                    .size(12.0)
            )
            .fill(egui::Color32::TRANSPARENT)
            .min_size(egui::vec2(140.0, 24.0))
        );
        
        if let Some(sc) = shortcut {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(sc)
                        .color(c.text_muted)
                        .size(11.0)
                );
            });
        }
        
        response.clicked()
    }).inner
}

/// 带开关状态的菜单项
fn menu_item_toggle(ui: &mut egui::Ui, icon: &str, label: &str, shortcut: Option<&str>, enabled: bool) -> bool {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let check_icon = if enabled { icons::CHECK } else { icons::UNCHECK };
    let button_text = format!("{}  {}  {}", check_icon, icon, label);
    
    ui.horizontal(|ui| {
        let response = ui.add(
            egui::Button::new(
                egui::RichText::new(&button_text)
                    .color(if enabled { c.accent_secondary } else { c.text_primary })
                    .size(12.0)
            )
            .fill(egui::Color32::TRANSPARENT)
            .min_size(egui::vec2(140.0, 24.0))
        );
        
        if let Some(sc) = shortcut {
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(sc)
                        .color(c.text_muted)
                        .size(11.0)
                );
            });
        }
        
        response.clicked()
    }).inner
}
