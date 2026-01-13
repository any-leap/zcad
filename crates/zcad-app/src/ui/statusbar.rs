//! 现代化状态栏和命令行
//!
//! 底部信息栏，包含命令输入、坐标显示、状态指示

use eframe::egui::{self, Color32, Margin, Stroke, Vec2, FontFamily, StrokeKind};
use zcad_core::math::Point2;

use crate::theme::THEME;

/// 状态栏操作结果
pub struct StatusbarResult {
    pub toggle_snap: bool,
    /// 用户输入的命令（如果有）
    pub command_input: Option<String>,
}

impl Default for StatusbarResult {
    fn default() -> Self {
        Self {
            toggle_snap: false,
            command_input: None,
        }
    }
}

/// 显示状态栏（包含命令行）
#[allow(clippy::too_many_arguments)]
pub fn show_statusbar(
    ctx: &egui::Context,
    status_message: &str,
    snap_enabled: bool,
    snap_info: Option<(&str, Point2)>,
    effective_pos: Point2,
    entity_count: usize,
    visible_count: usize,
    selected_count: usize,
    command_input: &mut String,
    should_focus: &mut bool,
) -> StatusbarResult {
    let mut result = StatusbarResult::default();
    let theme = &*THEME;
    let c = &theme.colors;
    
    egui::TopBottomPanel::bottom("status")
        .frame(theme.statusbar_frame())
        .show(ctx, |ui| {
            ui.horizontal(|ui| {
                // ===== 状态消息 =====
                ui.label(
                    egui::RichText::new(status_message)
                        .color(c.text_secondary)
                        .size(11.0)
                );
                
                ui.add_space(theme.spacing.large);
                
                // ===== 命令行输入框 =====
                ui.horizontal(|ui| {
                    ui.label(
                        egui::RichText::new("›")
                            .color(c.accent_primary)
                            .size(14.0)
                            .strong()
                    );
                    
                    let input_width = 220.0;
                    let response = ui.add(
                        egui::TextEdit::singleline(command_input)
                            .desired_width(input_width)
                            .font(egui::FontId::new(12.0, FontFamily::Monospace))
                            .hint_text(
                                egui::RichText::new("输入命令或数据...")
                                    .color(c.text_muted)
                            )
                            .margin(Margin::symmetric(theme.spacing.medium as i8, theme.spacing.small as i8))
                    );
                    
                    // 自动聚焦
                    if *should_focus {
                        response.request_focus();
                        *should_focus = false;
                    }
                    
                    // 回车执行命令
                    if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        let input = std::mem::take(command_input);
                        result.command_input = Some(input);
                        response.request_focus();
                    }
                });
                
                // ===== 捕捉状态显示 =====
                if let Some((snap_name, _)) = snap_info {
                    ui.add_space(theme.spacing.medium);
                    
                    // 捕捉标记徽章
                    let badge_text = format!("⊕ {}", snap_name);
                    let text_layout = ui.painter().layout_no_wrap(
                        badge_text.clone(),
                        egui::FontId::proportional(11.0),
                        c.snap_marker,
                    );
                    
                    let padding = theme.spacing.small;
                    let badge_size = Vec2::new(
                        text_layout.rect.width() + padding * 2.0,
                        text_layout.rect.height() + padding,
                    );
                    
                    let (rect, _) = ui.allocate_exact_size(badge_size, egui::Sense::hover());
                    
                    if ui.is_rect_visible(rect) {
                        ui.painter().rect(
                            rect,
                            theme.rounding.small,
                            Color32::from_rgba_unmultiplied(255, 200, 50, 30),
                            Stroke::new(1.0, c.snap_marker),
                            StrokeKind::Outside,
                        );
                        
                        ui.painter().galley(
                            egui::pos2(
                                rect.left() + padding,
                                rect.center().y - text_layout.rect.height() / 2.0,
                            ),
                            text_layout,
                            c.snap_marker,
                        );
                    }
                }
                
                // ===== 右侧信息区域 =====
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    // 坐标显示
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{:>9.3}", effective_pos.y))
                                .color(c.text_primary)
                                .size(11.0)
                                .family(FontFamily::Monospace)
                        );
                        ui.label(
                            egui::RichText::new("Y:")
                                .color(c.success)
                                .size(11.0)
                                .strong()
                        );
                        
                        ui.add_space(theme.spacing.medium);
                        
                        ui.label(
                            egui::RichText::new(format!("{:>9.3}", effective_pos.x))
                                .color(c.text_primary)
                                .size(11.0)
                                .family(FontFamily::Monospace)
                        );
                        ui.label(
                            egui::RichText::new("X:")
                                .color(c.error)
                                .size(11.0)
                                .strong()
                        );
                    });
                    
                    // 分隔线
                    ui.add_space(theme.spacing.medium);
                    ui.label(
                        egui::RichText::new("|")
                            .color(c.border_normal)
                    );
                    ui.add_space(theme.spacing.medium);
                    
                    // 渲染/总实体计数
                    ui.label(
                        egui::RichText::new(format!("{}/{}", visible_count, entity_count))
                            .color(c.text_primary)
                            .size(11.0)
                    );
                    ui.label(
                        egui::RichText::new("渲染:")
                            .color(c.text_muted)
                            .size(11.0)
                    );
                    
                    // 选中计数
                    if selected_count > 0 {
                        ui.add_space(theme.spacing.medium);
                        ui.label(
                            egui::RichText::new("|")
                                .color(c.border_normal)
                        );
                        ui.add_space(theme.spacing.medium);
                        
                        ui.label(
                            egui::RichText::new(format!("{}", selected_count))
                                .color(c.warning)
                                .size(11.0)
                                .strong()
                        );
                        ui.label(
                            egui::RichText::new("选中:")
                                .color(c.text_muted)
                                .size(11.0)
                        );
                    }
                    
                    ui.add_space(theme.spacing.medium);
                    ui.label(
                        egui::RichText::new("|")
                            .color(c.border_normal)
                    );
                    ui.add_space(theme.spacing.medium);
                    
                    // 捕捉开关按钮
                    let snap_label = if snap_enabled { "⊕ 捕捉" } else { "⊕" };
                    let snap_button = ui.add(
                        egui::Button::new(
                            egui::RichText::new(snap_label)
                            .color(if snap_enabled { c.accent_primary } else { c.text_muted })
                            .size(11.0)
                        )
                        .fill(if snap_enabled { c.selected } else { Color32::TRANSPARENT })
                        .stroke(Stroke::new(
                            1.0, 
                            if snap_enabled { c.accent_primary } else { Color32::TRANSPARENT }
                        ))
                        .corner_radius(theme.rounding.small)
                    );
                    
                    if snap_button.on_hover_text("对象捕捉 (F3)").clicked() {
                        result.toggle_snap = true;
                    }
                });
            });
        });
    
    result
}
