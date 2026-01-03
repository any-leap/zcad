//! 属性面板

use crate::state::UiState;
use zcad_file::Document;

/// 渲染属性面板
pub fn show_properties_panel(
    ctx: &egui::Context,
    document: &Document,
    ui_state: &UiState,
    show: &mut bool,
) {
    egui::SidePanel::left("properties_panel")
        .resizable(true)
        .default_width(220.0)
        .show_animated(ctx, *show, |ui| {
            ui.heading("Properties");
            ui.separator();

            // 选择信息
            let selected_count = ui_state.selected_entities.len();
            if selected_count == 0 {
                ui.label("No selection");
            } else if selected_count == 1 {
                // 单个实体的详细属性
                if let Some(entity) = document.get_entity(&ui_state.selected_entities[0]) {
                    ui.label(format!("Type: {}", entity.geometry.type_name()));
                    ui.separator();

                    egui::Grid::new("entity_props")
                        .num_columns(2)
                        .spacing([10.0, 4.0])
                        .show(ui, |ui| {
                            // 颜色
                            ui.label("Color:");
                            let color = &entity.properties.color;
                            let c32 = egui::Color32::from_rgb(color.r, color.g, color.b);
                            let (rect, _) = ui.allocate_exact_size(
                                egui::vec2(60.0, 16.0),
                                egui::Sense::click(),
                            );
                            ui.painter().rect_filled(rect, 2.0, c32);
                            ui.end_row();

                            // 图层
                            ui.label("Layer:");
                            if let Some(layer) = document.layers.get_layer_by_id(entity.layer_id) {
                                ui.label(&layer.name);
                            } else {
                                ui.label("0");
                            }
                            ui.end_row();

                            // 根据几何类型显示特定属性
                            match &entity.geometry {
                                zcad_core::geometry::Geometry::Line(line) => {
                                    ui.label("Start:");
                                    ui.label(format!(
                                        "{:.2}, {:.2}",
                                        line.start.x, line.start.y
                                    ));
                                    ui.end_row();

                                    ui.label("End:");
                                    ui.label(format!("{:.2}, {:.2}", line.end.x, line.end.y));
                                    ui.end_row();

                                    ui.label("Length:");
                                    ui.label(format!("{:.4}", line.length()));
                                    ui.end_row();
                                }
                                zcad_core::geometry::Geometry::Circle(circle) => {
                                    ui.label("Center:");
                                    ui.label(format!(
                                        "{:.2}, {:.2}",
                                        circle.center.x, circle.center.y
                                    ));
                                    ui.end_row();

                                    ui.label("Radius:");
                                    ui.label(format!("{:.4}", circle.radius));
                                    ui.end_row();

                                    ui.label("Circumference:");
                                    ui.label(format!("{:.4}", circle.circumference()));
                                    ui.end_row();

                                    ui.label("Area:");
                                    ui.label(format!("{:.4}", circle.area()));
                                    ui.end_row();
                                }
                                zcad_core::geometry::Geometry::Polyline(pl) => {
                                    ui.label("Vertices:");
                                    ui.label(format!("{}", pl.vertex_count()));
                                    ui.end_row();

                                    ui.label("Closed:");
                                    ui.label(if pl.closed { "Yes" } else { "No" });
                                    ui.end_row();

                                    ui.label("Length:");
                                    ui.label(format!("{:.4}", pl.length()));
                                    ui.end_row();
                                }
                                _ => {}
                            }
                        });
                }
            } else {
                // 多选
                ui.label(format!("{} objects selected", selected_count));

                // 统计类型
                let mut type_counts = std::collections::HashMap::new();
                for id in &ui_state.selected_entities {
                    if let Some(entity) = document.get_entity(id) {
                        *type_counts
                            .entry(entity.geometry.type_name())
                            .or_insert(0) += 1;
                    }
                }

                ui.separator();
                for (type_name, count) in &type_counts {
                    ui.label(format!("{}: {}", type_name, count));
                }
            }

            ui.separator();

            // 鼠标位置
            ui.heading("Cursor");
            ui.label(format!(
                "X: {:.4}  Y: {:.4}",
                ui_state.mouse_world_pos.x, ui_state.mouse_world_pos.y
            ));
        });
}

