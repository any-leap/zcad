//! 侧边面板

use eframe::egui;
use zcad_core::geometry::Geometry;
use zcad_core::math::Point2;
use zcad_ui::state::DrawingTool;

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

/// 显示图层面板
pub fn show_layers_panel(ctx: &egui::Context, layers: &[LayerInfo]) {
    egui::SidePanel::right("layers").default_width(150.0).show(ctx, |ui| {
        ui.heading("图层");
        ui.separator();
        for layer in layers {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.0, egui::Color32::from_rgb(layer.color.0, layer.color.1, layer.color.2));
                let txt = if layer.is_current { 
                    egui::RichText::new(&layer.name).strong() 
                } else { 
                    egui::RichText::new(&layer.name) 
                };
                ui.label(txt);
            });
        }
    });
}

/// 显示属性面板
pub fn show_properties_panel(
    ctx: &egui::Context,
    selected_info: Option<&SelectedEntityInfo>,
    selected_count: usize,
    current_tool: DrawingTool,
    mouse_world_pos: Point2,
) {
    egui::SidePanel::left("props").default_width(170.0).show(ctx, |ui| {
        ui.heading("属性");
        ui.separator();
        if let Some(info) = selected_info {
            ui.label(format!("类型: {}", info.type_name));
            ui.separator();
            for p in &info.properties { 
                ui.label(p); 
            }
        } else if selected_count > 1 {
            ui.label(format!("{} 个对象", selected_count));
        } else {
            ui.label(format!("工具: {}", current_tool.name()));
        }
        ui.separator();
        ui.label(format!("X: {:.4}", mouse_world_pos.x));
        ui.label(format!("Y: {:.4}", mouse_world_pos.y));
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
        ],
        Geometry::Polyline(p) => vec![
            format!("顶点数: {}", p.vertex_count()),
            format!("长度: {:.3}", p.length()),
        ],
        _ => vec![],
    }
}
