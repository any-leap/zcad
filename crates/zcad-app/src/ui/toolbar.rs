//! 现代化工具栏
//! 
//! 使用自定义矢量图标 + 文字标签

use eframe::egui::{self, Color32, Rect, Stroke, Vec2, StrokeKind};
use zcad_ui::state::DrawingTool;

use crate::icons;
use crate::theme::THEME;

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

/// 图标类型
#[derive(Clone, Copy)]
enum IconType {
    Select,
    Line,
    Circle,
    Rectangle,
    Arc,
    Polyline,
    Dimension,
    Delete,
    Undo,
    Redo,
    Ortho,
    Grid,
    ZoomFit,
    Snap,
}

impl IconType {
    fn draw(&self, painter: &egui::Painter, rect: Rect, color: Color32) {
        match self {
            IconType::Select => icons::draw_select_icon(painter, rect, color),
            IconType::Line => icons::draw_line_icon(painter, rect, color),
            IconType::Circle => icons::draw_circle_icon(painter, rect, color),
            IconType::Rectangle => icons::draw_rectangle_icon(painter, rect, color),
            IconType::Arc => icons::draw_arc_icon(painter, rect, color),
            IconType::Polyline => icons::draw_polyline_icon(painter, rect, color),
            IconType::Dimension => icons::draw_dimension_icon(painter, rect, color),
            IconType::Delete => icons::draw_delete_icon(painter, rect, color),
            IconType::Undo => icons::draw_undo_icon(painter, rect, color),
            IconType::Redo => icons::draw_redo_icon(painter, rect, color),
            IconType::Ortho => icons::draw_ortho_icon(painter, rect, color),
            IconType::Grid => icons::draw_grid_icon(painter, rect, color),
            IconType::ZoomFit => icons::draw_zoom_fit_icon(painter, rect, color),
            IconType::Snap => icons::draw_snap_icon(painter, rect, color),
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
                ui.add_space(theme.spacing.medium);
                
                // ===== 选择工具 =====
                if icon_text_button(ui, IconType::Select, "选择", current_tool == DrawingTool::Select, "选择工具 (Esc)") {
                    result.set_tool = Some(DrawingTool::Select);
                }
                
                separator(ui);
                
                // ===== 绘图工具组 =====
                if icon_text_button(ui, IconType::Line, "直线", current_tool == DrawingTool::Line, "直线 (L)") {
                    result.set_tool = Some(DrawingTool::Line);
                }
                if icon_text_button(ui, IconType::Circle, "圆", current_tool == DrawingTool::Circle, "圆 (C)") {
                    result.set_tool = Some(DrawingTool::Circle);
                }
                if icon_text_button(ui, IconType::Rectangle, "矩形", current_tool == DrawingTool::Rectangle, "矩形 (R)") {
                    result.set_tool = Some(DrawingTool::Rectangle);
                }
                if icon_text_button(ui, IconType::Arc, "弧", current_tool == DrawingTool::Arc, "圆弧 (A)") {
                    result.set_tool = Some(DrawingTool::Arc);
                }
                if icon_text_button(ui, IconType::Polyline, "多段线", current_tool == DrawingTool::Polyline, "多段线 (PL)") {
                    result.set_tool = Some(DrawingTool::Polyline);
                }
                
                separator(ui);
                
                // ===== 标注工具组 =====
                if icon_text_button(ui, IconType::Dimension, "标注", current_tool == DrawingTool::Dimension, "线性标注 (DIM)") {
                    result.set_tool = Some(DrawingTool::Dimension);
                }
                
                separator(ui);
                
                // ===== 编辑操作组 =====
                if icon_button(ui, IconType::Delete, false, "删除选中 (Del)") {
                    result.delete = true;
                }
                if icon_button(ui, IconType::Undo, false, "撤销 (Ctrl+Z)") {
                    result.undo = true;
                }
                if icon_button(ui, IconType::Redo, false, "重做 (Ctrl+Y)") {
                    result.redo = true;
                }
                
                separator(ui);
                
                // ===== 视图选项组 =====
                if icon_button(ui, IconType::Ortho, ortho_mode, "正交模式 (F8)") {
                    result.toggle_ortho = true;
                }
                if icon_button(ui, IconType::Grid, show_grid, "网格 (G)") {
                    result.toggle_grid = true;
                }
                if icon_button(ui, IconType::ZoomFit, false, "缩放至全部 (Z)") {
                    result.zoom_fit = true;
                }
                
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

/// 图标+文字按钮
fn icon_text_button(ui: &mut egui::Ui, icon: IconType, label: &str, selected: bool, tooltip: &str) -> bool {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let icon_size = 16.0;
    let font_size = 11.0;
    let padding_x = 6.0;
    let padding_y = 4.0;
    let gap = 4.0;
    
    // 计算文字宽度
    let text_galley = ui.painter().layout_no_wrap(
        label.to_string(),
        egui::FontId::proportional(font_size),
        c.text_primary,
    );
    
    let desired_size = Vec2::new(
        padding_x * 2.0 + icon_size + gap + text_galley.rect.width(),
        26.0,
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
        
        let stroke = if selected {
            Stroke::new(1.0, c.accent_primary)
        } else {
            Stroke::NONE
        };
        
        ui.painter().rect(rect, theme.rounding.small, bg_color, stroke, StrokeKind::Outside);
        
        // 图标
        let icon_color = if selected {
            c.accent_secondary
        } else if response.hovered() {
            c.text_primary
        } else {
            c.text_secondary
        };
        
        let icon_rect = Rect::from_center_size(
            egui::pos2(rect.left() + padding_x + icon_size / 2.0, rect.center().y),
            Vec2::splat(icon_size),
        );
        icon.draw(ui.painter(), icon_rect, icon_color);
        
        // 文字
        let text_color = if selected {
            c.accent_secondary
        } else if response.hovered() {
            c.text_primary
        } else {
            c.text_secondary
        };
        
        let text_pos = egui::pos2(
            rect.left() + padding_x + icon_size + gap,
            rect.center().y - text_galley.rect.height() / 2.0,
        );
        ui.painter().galley(text_pos, text_galley, text_color);
    }
    
    response.on_hover_text(tooltip).clicked()
}

/// 纯图标按钮
fn icon_button(ui: &mut egui::Ui, icon: IconType, selected: bool, tooltip: &str) -> bool {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let size = 26.0;
    let icon_size = 16.0;
    
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(size), egui::Sense::click());
    
    if ui.is_rect_visible(rect) {
        // 背景
        let bg_color = if selected {
            c.selected
        } else if response.hovered() {
            c.hover
        } else {
            Color32::TRANSPARENT
        };
        
        let stroke = if selected {
            Stroke::new(1.0, c.accent_primary)
        } else {
            Stroke::NONE
        };
        
        ui.painter().rect(rect, theme.rounding.small, bg_color, stroke, StrokeKind::Outside);
        
        // 图标
        let icon_color = if selected {
            c.accent_primary
        } else if response.hovered() {
            c.text_primary
        } else {
            c.text_secondary
        };
        
        let icon_rect = Rect::from_center_size(rect.center(), Vec2::splat(icon_size));
        icon.draw(ui.painter(), icon_rect, icon_color);
    }
    
    response.on_hover_text(tooltip).clicked()
}

/// 分隔线
fn separator(ui: &mut egui::Ui) {
    let theme = &*THEME;
    let c = &theme.colors;
    
    ui.add_space(theme.spacing.small);
    
    let height = 18.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(1.0, height), egui::Sense::hover());
    
    if ui.is_rect_visible(rect) {
        ui.painter().line_segment(
            [rect.center_top(), rect.center_bottom()],
            Stroke::new(1.0, c.border_subtle),
        );
    }
    
    ui.add_space(theme.spacing.small);
}
