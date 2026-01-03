//! 图层面板

use zcad_core::layer::LayerManager;

/// 渲染图层面板
pub fn show_layers_panel(ctx: &egui::Context, layers: &mut LayerManager, show: &mut bool) {
    egui::SidePanel::right("layers_panel")
        .resizable(true)
        .default_width(200.0)
        .show_animated(ctx, *show, |ui| {
            ui.heading("Layers");
            ui.separator();

            // 图层列表
            egui::ScrollArea::vertical().show(ui, |ui| {
                let current_layer_name = layers.current_layer().name.clone();

                for layer in layers.all_layers() {
                    let is_current = layer.name == current_layer_name;

                    ui.horizontal(|ui| {
                        // 可见性按钮
                        let vis_icon = if layer.visible { "👁" } else { "👁‍🗨" };
                        if ui.small_button(vis_icon).clicked() {
                            // TODO: 切换可见性
                        }

                        // 锁定按钮
                        let lock_icon = if layer.locked { "🔒" } else { "🔓" };
                        if ui.small_button(lock_icon).clicked() {
                            // TODO: 切换锁定
                        }

                        // 颜色指示器
                        let color = egui::Color32::from_rgb(
                            layer.color.r,
                            layer.color.g,
                            layer.color.b,
                        );
                        let (rect, _response) =
                            ui.allocate_exact_size(egui::vec2(16.0, 16.0), egui::Sense::click());
                        ui.painter().rect_filled(rect, 2.0, color);

                        // 图层名称
                        let text = if is_current {
                            egui::RichText::new(&layer.name).strong()
                        } else {
                            egui::RichText::new(&layer.name)
                        };

                        if ui.selectable_label(is_current, text).clicked() {
                            // TODO: 设置当前图层
                        }
                    });
                }
            });

            ui.separator();

            // 图层操作按钮
            ui.horizontal(|ui| {
                if ui.button("➕ Add").clicked() {
                    // TODO: 添加图层
                }
                if ui.button("➖ Delete").clicked() {
                    // TODO: 删除图层
                }
            });
        });
}

