//! 状态栏和命令行

use eframe::egui;
use zcad_core::math::Point2;

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
pub fn show_statusbar(
    ctx: &egui::Context,
    status_message: &str,
    snap_enabled: bool,
    snap_info: Option<(&str, Point2)>,
    effective_pos: Point2,
    entity_count: usize,
    selected_count: usize,
    command_input: &mut String,
    should_focus: &mut bool,
) -> StatusbarResult {
    let mut result = StatusbarResult::default();
    
    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal(|ui| {
            // 状态消息
            ui.label(status_message);
            
            ui.separator();
            
            // 命令行输入
            ui.label("Command:");
            let response = ui.add(
                egui::TextEdit::singleline(command_input)
                    .desired_width(200.0)
                    .hint_text("输入命令或数据..."),
            );
            
            // 自动聚焦
            if *should_focus {
                response.request_focus();
                *should_focus = false;
            }
            
            // 回车执行命令
            if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                let input = std::mem::take(command_input);
                if !input.is_empty() || true { // 允许空命令（重复上一个命令）
                    result.command_input = Some(input);
                }
                response.request_focus();
            }
            
            // 捕捉状态显示
            if let Some((snap_name, _)) = snap_info {
                ui.separator();
                ui.colored_label(egui::Color32::YELLOW, format!("⊕ {}", snap_name));
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(format!("X:{:>8.2} Y:{:>8.2}", effective_pos.x, effective_pos.y));
                ui.separator();
                ui.label(format!("实体: {}", entity_count));
                if selected_count > 0 {
                    ui.separator();
                    ui.label(format!("选中: {}", selected_count));
                }
                ui.separator();
                // 捕捉开关
                let snap_text = if snap_enabled { "🔗 捕捉" } else { "🔗" };
                if ui.selectable_label(snap_enabled, snap_text).on_hover_text("对象捕捉 (F3)").clicked() {
                    result.toggle_snap = true;
                }
            });
        });
    });
    
    result
}
