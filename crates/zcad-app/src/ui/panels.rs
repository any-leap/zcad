//! 现代化侧边面板
//!
//! 图层面板和属性面板，采用折叠组设计

use eframe::egui::{self, Margin, Color32, Stroke, Vec2, StrokeKind};
use zcad_core::geometry::Geometry;
use zcad_core::math::Point2;
use zcad_ui::state::DrawingTool;

use crate::theme::THEME;

/// 图层信息
pub struct LayerInfo {
    pub name: String,
    pub color: (u8, u8, u8),
    pub is_current: bool,
}

/// 选中实体信息
pub struct SelectedEntityInfo {
    pub type_name: String,
    pub properties: Vec<String>,
}

/// 显示图层面板（右侧）
pub fn show_layers_panel(ctx: &egui::Context, layers: &[LayerInfo]) {
    let theme = &*THEME;
    let c = &theme.colors;
    
    egui::SidePanel::right("layers")
        .default_width(180.0)
        .min_width(140.0)
        .frame(theme.panel_frame().inner_margin(Margin::ZERO))
        .show(ctx, |ui| {
            // 面板标题
            ui.add_space(theme.spacing.small);
            ui.horizontal(|ui| {
                ui.add_space(theme.spacing.large);
                ui.label(
                    egui::RichText::new("📑 图层")
                        .color(c.text_primary)
                        .size(14.0)
                        .strong()
                );
            });
            
            ui.add_space(theme.spacing.small);
            ui.separator();
            ui.add_space(theme.spacing.small);
            
            // 图层列表
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(theme.spacing.tiny);
                    
                    for layer in layers {
                        let is_current = layer.is_current;
                        
                        // 图层行
                        let response = ui.horizontal(|ui| {
                            ui.add_space(theme.spacing.medium);
                            
                            // 颜色方块
                            let color = Color32::from_rgb(layer.color.0, layer.color.1, layer.color.2);
                            let (rect, _) = ui.allocate_exact_size(Vec2::splat(12.0), egui::Sense::hover());
                            ui.painter().rect_filled(rect, theme.rounding.small, color);
                            ui.painter().rect_stroke(rect, theme.rounding.small, Stroke::new(1.0, c.border_normal), StrokeKind::Outside);
                            
                            ui.add_space(theme.spacing.small);
                            
                            // 图层名称
                            let text = if is_current {
                                egui::RichText::new(&layer.name)
                                    .color(c.accent_secondary)
                                    .strong()
                            } else {
                                egui::RichText::new(&layer.name)
                                    .color(c.text_primary)
                            };
                            
                            ui.label(text);
                            
                            // 当前图层标记
                            if is_current {
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    ui.add_space(theme.spacing.medium);
                                    ui.label(
                                        egui::RichText::new("✓")
                                            .color(c.accent_primary)
                                            .size(11.0)
                                    );
                                });
                            }
                        });
                        
                        // 分隔
                        ui.add_space(theme.spacing.tiny);
                    }
                    
                    ui.add_space(theme.spacing.medium);
                });
        });
}

/// 显示属性面板（左侧）
pub fn show_properties_panel(
    ctx: &egui::Context,
    selected_info: Option<&SelectedEntityInfo>,
    selected_count: usize,
    current_tool: DrawingTool,
    mouse_world_pos: Point2,
) {
    let theme = &*THEME;
    let c = &theme.colors;
    
    egui::SidePanel::left("props")
        .default_width(200.0)
        .min_width(160.0)
        .frame(theme.panel_frame().inner_margin(Margin::ZERO))
        .show(ctx, |ui| {
            egui::ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    ui.add_space(theme.spacing.small);
                    
                    // ===== 属性区域 =====
                    ui.horizontal(|ui| {
                        ui.add_space(theme.spacing.large);
                        ui.label(
                            egui::RichText::new("📋 属性")
                                .color(c.text_primary)
                                .size(14.0)
                                .strong()
                        );
                    });
                    
                    ui.add_space(theme.spacing.small);
                    ui.separator();
                    ui.add_space(theme.spacing.medium);
                    
                    // 属性内容
                    ui.horizontal(|ui| {
                        ui.add_space(theme.spacing.large);
                        ui.vertical(|ui| {
                            if let Some(info) = selected_info {
                                // 单个实体选中
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("类型")
                                            .color(c.text_muted)
                                            .size(11.0)
                                    );
                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(
                                            egui::RichText::new(&info.type_name)
                                                .color(c.accent_secondary)
                                                .size(11.0)
                                                .strong()
                                        );
                                    });
                                });
                                
                                ui.add_space(theme.spacing.small);
                                
                                // 几何属性
                                for prop in &info.properties {
                                    ui.label(
                                        egui::RichText::new(prop)
                                            .color(c.text_secondary)
                                            .size(11.0)
                                    );
                                }
                            } else if selected_count > 1 {
                                // 多选
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("已选中")
                                            .color(c.text_muted)
                                            .size(11.0)
                                    );
                                    ui.label(
                                        egui::RichText::new(format!("{} 个对象", selected_count))
                                            .color(c.warning)
                                            .size(11.0)
                                            .strong()
                                    );
                                });
                            } else {
                                // 无选择 - 显示当前工具
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("当前工具")
                                            .color(c.text_muted)
                                            .size(11.0)
                                    );
                                    ui.label(
                                        egui::RichText::new(current_tool.name())
                                            .color(c.text_accent)
                                            .size(11.0)
                                            .strong()
                                    );
                                });
                            }
                        });
                        ui.add_space(theme.spacing.large);
                    });
                    
                    ui.add_space(theme.spacing.xlarge);
                    
                    // ===== 坐标区域 =====
                    ui.horizontal(|ui| {
                        ui.add_space(theme.spacing.large);
                        ui.label(
                            egui::RichText::new("📍 坐标")
                                .color(c.text_primary)
                                .size(14.0)
                                .strong()
                        );
                    });
                    
                    ui.add_space(theme.spacing.small);
                    ui.separator();
                    ui.add_space(theme.spacing.medium);
                    
                    // 坐标显示
                    ui.horizontal(|ui| {
                        ui.add_space(theme.spacing.large);
                        ui.vertical(|ui| {
                            // X 坐标
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("X")
                                        .color(c.error)
                                        .size(12.0)
                                        .strong()
                                );
                                ui.add_space(theme.spacing.small);
                                ui.label(
                                    egui::RichText::new(format!("{:>10.4}", mouse_world_pos.x))
                                        .color(c.text_primary)
                                        .size(12.0)
                                        .family(egui::FontFamily::Monospace)
                                );
                            });
                            
                            ui.add_space(theme.spacing.tiny);
                            
                            // Y 坐标
                            ui.horizontal(|ui| {
                                ui.label(
                                    egui::RichText::new("Y")
                                        .color(c.success)
                                        .size(12.0)
                                        .strong()
                                );
                                ui.add_space(theme.spacing.small);
                                ui.label(
                                    egui::RichText::new(format!("{:>10.4}", mouse_world_pos.y))
                                        .color(c.text_primary)
                                        .size(12.0)
                                        .family(egui::FontFamily::Monospace)
                                );
                            });
                        });
                    });
                    
                    ui.add_space(theme.spacing.xlarge);
                });
        });
}

/// 从几何体提取属性信息
pub fn extract_geometry_properties(geometry: &Geometry) -> Vec<String> {
    match geometry {
        Geometry::Line(l) => vec![
            format!("起点: ({:.2}, {:.2})", l.start.x, l.start.y),
            format!("终点: ({:.2}, {:.2})", l.end.x, l.end.y),
            format!("长度: {:.3}", l.length()),
        ],
        Geometry::Circle(c) => vec![
            format!("圆心: ({:.2}, {:.2})", c.center.x, c.center.y),
            format!("半径: {:.3}", c.radius),
            format!("周长: {:.3}", std::f64::consts::PI * 2.0 * c.radius),
            format!("面积: {:.3}", std::f64::consts::PI * c.radius * c.radius),
        ],
        Geometry::Arc(a) => vec![
            format!("圆心: ({:.2}, {:.2})", a.center.x, a.center.y),
            format!("半径: {:.3}", a.radius),
            format!("起始角: {:.1}°", a.start_angle.to_degrees()),
            format!("终止角: {:.1}°", a.end_angle.to_degrees()),
        ],
        Geometry::Polyline(p) => vec![
            format!("顶点数: {}", p.vertex_count()),
            format!("长度: {:.3}", p.length()),
            format!("闭合: {}", if p.closed { "是" } else { "否" }),
        ],
        Geometry::Ellipse(e) => vec![
            format!("中心: ({:.2}, {:.2})", e.center.x, e.center.y),
            format!("长轴: {:.3}", e.major_radius()),
            format!("短轴: {:.3}", e.minor_radius()),
        ],
        Geometry::Text(t) => vec![
            format!("位置: ({:.2}, {:.2})", t.position.x, t.position.y),
            format!("内容: {}", t.content),
            format!("高度: {:.1}", t.height),
        ],
        Geometry::Point(p) => vec![
            format!("位置: ({:.4}, {:.4})", p.position.x, p.position.y),
        ],
        Geometry::Dimension(d) => vec![
            format!("标注类型: 线性"),
            format!("测量值: {:.3}", d.measurement()),
        ],
        _ => vec![],
    }
}
