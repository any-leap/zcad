//! DXF 导入

use std::path::Path;
use crate::document::Document;
use crate::dxf_raw::{DxfRawParser, parse_layouts, parse_viewports};
use crate::error::FileError;
use zcad_core::layout::{PaperSize, PaperOrientation, Viewport, ViewportId, ViewportStatus};
use zcad_core::math::Point2;

use super::conversion::{aci_to_color, convert_dxf_entity, get_entity_bounds};

/// 从DXF文件导入
pub fn import(path: &Path) -> Result<Document, FileError> {
    let drawing = dxf::Drawing::load_file(path).map_err(|e| FileError::Dxf(e.to_string()))?;

    let mut document = Document::new();

    // 导入图层
    for layer in drawing.layers() {
        let color = aci_to_color(layer.color.index().unwrap_or(7) as u8);
        let new_layer = zcad_core::layer::Layer::new(&layer.name).with_color(color);
        document.layers.add_layer(new_layer);
    }

    // 导入模型空间实体
    for entity in drawing.entities() {
        if let Some(zcad_entity) = convert_dxf_entity(entity) {
            document.add_entity(zcad_entity);
        }
    }

    // 使用原始解析器导入完整的布局和视口信息
    if let Ok(mut raw_parser) = DxfRawParser::load(path) {
        import_layouts_full(&mut raw_parser, &drawing, &mut document);
    } else {
        // 回退到简化模式
        import_layouts_simplified(&drawing, &mut document);
    }

    // 设置文件路径
    document.set_file_path(path);

    Ok(document)
}

/// 完整的布局导入（使用原始解析器）
fn import_layouts_full(
    raw_parser: &mut DxfRawParser,
    drawing: &dxf::Drawing,
    document: &mut Document,
) {
    // 1. 解析 LAYOUT 对象
    let dxf_layouts = parse_layouts(raw_parser);
    
    // 2. 解析 VIEWPORT 实体
    let dxf_viewports = parse_viewports(raw_parser);
    
    // 3. 计算模型空间边界（用于设置默认视图）
    let model_bounds = calculate_model_bounds(drawing);
    
    // 4. 更新或创建布局
    for dxf_layout in &dxf_layouts {
        // 跳过模型空间
        if dxf_layout.is_model_space {
            continue;
        }
        
        // 确定图纸尺寸
        let paper_size = determine_paper_size(dxf_layout.paper_width, dxf_layout.paper_height);
        let orientation = if dxf_layout.paper_width > dxf_layout.paper_height {
            PaperOrientation::Landscape
        } else {
            PaperOrientation::Portrait
        };
        
        // 查找属于此布局的视口
        let layout_viewports: Vec<Viewport> = dxf_viewports
            .iter()
            .filter(|vp| vp.owner_handle == dxf_layout.block_record_handle || 
                         vp.owner_handle.is_empty())
            .enumerate()
            .map(|(idx, dxf_vp)| {
                convert_raw_viewport_to_zcad(dxf_vp, idx as u64 + 1, &model_bounds)
            })
            .collect();
        
        // 更新现有布局或添加新布局
        if let Some(existing) = document.layout_manager.get_layout_by_name(&dxf_layout.name) {
            let existing_id = existing.id;
            if let Some(layout) = document.layout_manager.get_layout_mut(existing_id) {
                layout.paper_size = paper_size;
                layout.orientation = orientation;
                layout.margins = (
                    dxf_layout.top_margin,
                    dxf_layout.right_margin,
                    dxf_layout.bottom_margin,
                    dxf_layout.left_margin,
                );
                if !layout_viewports.is_empty() {
                    layout.viewports = layout_viewports;
                }
            }
        } else {
            // 添加新布局
            let layout_id = document.layout_manager.add_layout(&dxf_layout.name);
            if let Some(layout) = document.layout_manager.get_layout_mut(layout_id) {
                layout.paper_size = paper_size;
                layout.orientation = orientation;
                layout.margins = (
                    dxf_layout.top_margin,
                    dxf_layout.right_margin,
                    dxf_layout.bottom_margin,
                    dxf_layout.left_margin,
                );
                if !layout_viewports.is_empty() {
                    layout.viewports = layout_viewports;
                }
            }
        }
    }
    
    // 如果没有解析到任何布局，使用简化模式
    if dxf_layouts.iter().filter(|l| !l.is_model_space).count() == 0 {
        import_layouts_simplified(drawing, document);
    }
}

/// 计算模型空间边界
fn calculate_model_bounds(drawing: &dxf::Drawing) -> Option<(f64, f64, f64, f64)> {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut has_entities = false;
    
    for entity in drawing.entities() {
        if let Some(bbox) = get_entity_bounds(entity) {
            min_x = min_x.min(bbox.0);
            min_y = min_y.min(bbox.1);
            max_x = max_x.max(bbox.2);
            max_y = max_y.max(bbox.3);
            has_entities = true;
        }
    }
    
    if has_entities {
        Some((min_x, min_y, max_x, max_y))
    } else {
        None
    }
}

/// 根据尺寸确定标准图纸大小
fn determine_paper_size(width: f64, height: f64) -> PaperSize {
    let (w, h) = if width > height { (width, height) } else { (height, width) };
    
    // 检查常见纸张尺寸（允许 5mm 误差）
    let tolerance = 5.0;
    
    if (w - 1189.0).abs() < tolerance && (h - 841.0).abs() < tolerance {
        PaperSize::A0
    } else if (w - 841.0).abs() < tolerance && (h - 594.0).abs() < tolerance {
        PaperSize::A1
    } else if (w - 594.0).abs() < tolerance && (h - 420.0).abs() < tolerance {
        PaperSize::A2
    } else if (w - 420.0).abs() < tolerance && (h - 297.0).abs() < tolerance {
        PaperSize::A3
    } else if (w - 297.0).abs() < tolerance && (h - 210.0).abs() < tolerance {
        PaperSize::A4
    } else {
        PaperSize::Custom { width, height }
    }
}

/// 将原始 DXF 视口转换为 ZCAD 视口
fn convert_raw_viewport_to_zcad(
    dxf_vp: &crate::dxf_raw::DxfViewport,
    id: u64,
    _model_bounds: &Option<(f64, f64, f64, f64)>,
) -> Viewport {
    // 计算视口位置（从中心转换为左下角）
    let position = Point2::new(
        dxf_vp.center.x - dxf_vp.width / 2.0,
        dxf_vp.center.y - dxf_vp.height / 2.0,
    );
    
    let mut viewport = Viewport::new(ViewportId::new(id), position, dxf_vp.width, dxf_vp.height);
    
    // 设置视图中心
    viewport.view_center = dxf_vp.view_center;
    
    // 计算比例
    if dxf_vp.view_height > 0.0 && dxf_vp.height > 0.0 {
        viewport.scale = dxf_vp.view_height / dxf_vp.height;
    }
    
    // 设置状态
    viewport.status = if dxf_vp.status > 0 {
        ViewportStatus::Inactive
    } else {
        ViewportStatus::Hidden
    };
    
    viewport
}

/// 简化的布局导入
/// 
/// 由于 dxf crate 对 VIEWPORT 实体的支持有限，
/// 这里使用简化的方式：基于模型空间范围创建默认视口
fn import_layouts_simplified(drawing: &dxf::Drawing, document: &mut Document) {
    // 计算模型空间的边界
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;
    let mut has_entities = false;
    
    for entity in drawing.entities() {
        if let Some(bbox) = get_entity_bounds(entity) {
            min_x = min_x.min(bbox.0);
            min_y = min_y.min(bbox.1);
            max_x = max_x.max(bbox.2);
            max_y = max_y.max(bbox.3);
            has_entities = true;
        }
    }
    
    // 如果有实体，更新默认视口的视图范围
    if has_entities {
        if let Some(layout) = document.layout_manager.get_layout_by_name("Layout1") {
            let layout_id = layout.id;
            if let Some(layout) = document.layout_manager.get_layout_mut(layout_id) {
                // 更新第一个视口的视图中心和比例
                if let Some(viewport) = layout.viewports.first_mut() {
                    let model_width = max_x - min_x;
                    let model_height = max_y - min_y;
                    
                    // 设置视图中心
                    viewport.view_center = Point2::new(
                        (min_x + max_x) / 2.0,
                        (min_y + max_y) / 2.0,
                    );
                    
                    // 计算合适的比例
                    let scale_x = model_width / viewport.width;
                    let scale_y = model_height / viewport.height;
                    viewport.scale = scale_x.max(scale_y) * 1.1; // 留 10% 边距
                }
            }
        }
    }
}
