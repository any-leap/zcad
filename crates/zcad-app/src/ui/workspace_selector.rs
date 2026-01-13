//! 工作空间选择器
//! 
//! 提供启动时的工作空间选择界面和运行时切换功能

use eframe::egui;
use zcad_module::workspace::WorkspaceType;

use crate::theme::THEME;

/// 工作空间选择器结果
pub struct WorkspaceSelectorResult {
    /// 选择的工作空间类型
    pub selected: Option<WorkspaceType>,
    /// 是否关闭选择器（不做选择）
    pub dismissed: bool,
}

impl Default for WorkspaceSelectorResult {
    fn default() -> Self {
        Self {
            selected: None,
            dismissed: false,
        }
    }
}

/// 显示工作空间选择器对话框（启动时的欢迎界面）
pub fn show_workspace_selector(ctx: &egui::Context, current: WorkspaceType) -> WorkspaceSelectorResult {
    let mut result = WorkspaceSelectorResult::default();
    let theme = &*THEME;
    let c = &theme.colors;
    
    // 半透明背景遮罩
    egui::Area::new(egui::Id::new("workspace_selector_overlay"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .show(ctx, |ui| {
            let screen_rect = ctx.screen_rect();
            ui.painter().rect_filled(
                screen_rect,
                0.0,
                egui::Color32::from_black_alpha(180),
            );
        });
    
    // 居中的选择器窗口
    egui::Window::new("选择工作空间")
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .fixed_size([700.0, 500.0])
        .frame(egui::Frame::window(&ctx.style())
            .fill(c.bg_panel)
            .stroke(egui::Stroke::new(1.0, c.border_normal))
            .corner_radius(theme.rounding.large)
            .shadow(egui::Shadow {
                offset: [0, 8],
                blur: 24,
                spread: 0,
                color: egui::Color32::from_black_alpha(100),
            }))
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(16.0);
                
                // 标题
                ui.label(
                    egui::RichText::new("欢迎使用 ZCAD")
                        .size(28.0)
                        .color(c.text_primary)
                        .strong()
                );
                
                ui.add_space(8.0);
                
                ui.label(
                    egui::RichText::new("请选择您的工作空间类型")
                        .size(14.0)
                        .color(c.text_secondary)
                );
                
                ui.add_space(24.0);
            });
            
            // 工作空间网格
            egui::ScrollArea::vertical()
                .max_height(350.0)
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        ui.spacing_mut().item_spacing = egui::vec2(12.0, 12.0);
                        
                        for ws_type in WorkspaceType::all() {
                            let is_current = *ws_type == current;
                            if workspace_card(ui, *ws_type, is_current) {
                                result.selected = Some(*ws_type);
                            }
                        }
                    });
                });
            
            ui.add_space(16.0);
            
            // 底部按钮
            ui.horizontal(|ui| {
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.add(
                        egui::Button::new(
                            egui::RichText::new("取消")
                                .color(c.text_secondary)
                        )
                        .fill(c.bg_header)
                        .stroke(egui::Stroke::new(1.0, c.border_normal))
                        .corner_radius(theme.rounding.small)
                        .min_size(egui::vec2(80.0, 32.0))
                    ).clicked() {
                        result.dismissed = true;
                    }
                });
            });
        });
    
    result
}

/// 工作空间卡片组件
fn workspace_card(ui: &mut egui::Ui, ws_type: WorkspaceType, is_current: bool) -> bool {
    let theme = &*THEME;
    let c = &theme.colors;
    
    let card_size = egui::vec2(200.0, 100.0);
    let (rect, response) = ui.allocate_exact_size(card_size, egui::Sense::click());
    
    if ui.is_rect_visible(rect) {
        // 背景
        let bg_color = if is_current {
            c.selected
        } else if response.hovered() {
            c.hover
        } else {
            c.bg_header
        };
        
        let stroke_color = if is_current {
            c.accent_primary
        } else if response.hovered() {
            c.border_normal
        } else {
            c.border_subtle
        };
        
        ui.painter().rect(
            rect,
            theme.rounding.medium,
            bg_color,
            egui::Stroke::new(if is_current { 2.0 } else { 1.0 }, stroke_color),
            egui::StrokeKind::Inside,
        );
        
        // 图标
        let icon = workspace_icon(ws_type);
        let icon_pos = egui::pos2(rect.left() + 16.0, rect.top() + 20.0);
        ui.painter().text(
            icon_pos,
            egui::Align2::LEFT_TOP,
            icon,
            egui::FontId::proportional(24.0),
            if is_current { c.accent_secondary } else { c.text_accent },
        );
        
        // 名称
        let name_pos = egui::pos2(rect.left() + 16.0, rect.top() + 50.0);
        ui.painter().text(
            name_pos,
            egui::Align2::LEFT_TOP,
            ws_type.display_name(),
            egui::FontId::proportional(14.0),
            c.text_primary,
        );
        
        // 描述（截断）
        let desc = ws_type.description();
        let short_desc = if desc.len() > 35 {
            format!("{}...", &desc[..32])
        } else {
            desc.to_string()
        };
        let desc_pos = egui::pos2(rect.left() + 16.0, rect.top() + 72.0);
        ui.painter().text(
            desc_pos,
            egui::Align2::LEFT_TOP,
            short_desc,
            egui::FontId::proportional(11.0),
            c.text_muted,
        );
        
        // 当前标记
        if is_current {
            let badge_pos = egui::pos2(rect.right() - 8.0, rect.top() + 8.0);
            ui.painter().text(
                badge_pos,
                egui::Align2::RIGHT_TOP,
                "✓",
                egui::FontId::proportional(14.0),
                c.accent_primary,
            );
        }
    }
    
    response.clicked()
}

/// 获取工作空间类型的图标
fn workspace_icon(ws_type: WorkspaceType) -> &'static str {
    match ws_type {
        WorkspaceType::Drafting2D => "📐",
        WorkspaceType::Modeling3D => "🧊",
        WorkspaceType::MechanicalDesign => "⚙️",
        WorkspaceType::SteelStructure => "🏗️",
        WorkspaceType::ConcreteStructure => "🧱",
        WorkspaceType::Architecture => "🏛️",
        WorkspaceType::Mep => "🔧",
        WorkspaceType::ElectronicPcb => "🔌",
        WorkspaceType::ElectronicSchematic => "📋",
        WorkspaceType::Custom => "🎨",
    }
}

/// 获取工作空间类型的中文名称
pub fn workspace_display_name_zh(ws_type: WorkspaceType) -> &'static str {
    match ws_type {
        WorkspaceType::Drafting2D => "2D 制图",
        WorkspaceType::Modeling3D => "3D 建模",
        WorkspaceType::MechanicalDesign => "机械设计",
        WorkspaceType::SteelStructure => "钢结构",
        WorkspaceType::ConcreteStructure => "混凝土结构",
        WorkspaceType::Architecture => "建筑设计",
        WorkspaceType::Mep => "机电管道",
        WorkspaceType::ElectronicPcb => "PCB 设计",
        WorkspaceType::ElectronicSchematic => "电路原理图",
        WorkspaceType::Custom => "自定义",
    }
}
