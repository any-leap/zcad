//! 状态栏

use eframe::egui;
use zcad_core::math::Point2;

/// 状态栏操作结果
pub struct StatusbarResult {
    pub toggle_snap: bool,
}

impl Default for StatusbarResult {
    fn default() -> Self {
        Self {
            toggle_snap: false,
        }
    }
}

/// 显示状态栏
pub fn show_statusbar(
    ctx: &egui::Context,
    status_message: &str,
    snap_enabled: bool,
    snap_info: Option<(&str, Point2)>,
    effective_pos: Point2,
    entity_count: usize,
    selected_count: usize,
) -> StatusbarResult {
    let mut result = StatusbarResult::default();
    
    egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
        ui.horizontal(|ui| {
            ui.label(status_message);
            
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
