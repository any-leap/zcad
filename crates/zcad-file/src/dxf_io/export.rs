//! DXF 导出

use std::path::Path;
use crate::document::Document;
use crate::dxf_raw::DxfWriter;
use crate::error::FileError;
use zcad_core::entity::Entity;
use zcad_core::geometry::Geometry;
use zcad_core::layout::{Layout, Viewport, ViewportStatus};

use super::conversion::{color_to_aci, convert_to_dxf_entity};

/// 导出到DXF文件
pub fn export(document: &Document, path: &Path) -> Result<(), FileError> {
    let mut drawing = dxf::Drawing::new();

    // 导出图层
    for layer in document.layers.all_layers() {
        let mut dxf_layer = dxf::tables::Layer::default();
        dxf_layer.name = layer.name.clone();
        dxf_layer.color = dxf::Color::from_index(color_to_aci(&layer.color));
        drawing.add_layer(dxf_layer);
    }

    // 导出模型空间实体
    for entity in document.all_entities() {
        if let Some(dxf_entity) = convert_to_dxf_entity(entity) {
            drawing.add_entity(dxf_entity);
        }
    }

    // 导出图纸空间实体（如果有）
    export_paper_space_entities(document, &mut drawing);

    drawing
        .save_file(path)
        .map_err(|e| FileError::Dxf(e.to_string()))?;

    Ok(())
}

/// 导出图纸空间实体和视口
fn export_paper_space_entities(document: &Document, drawing: &mut dxf::Drawing) {
    // 遍历所有布局
    for layout in document.layout_manager.layouts() {
        // 导出图纸空间实体
        for entity in &layout.paper_space_entities {
            if let Some(dxf_entity) = convert_to_dxf_entity(entity) {
                drawing.add_entity(dxf_entity);
            }
        }
    }
}

/// 使用原始写入器导出完整的 DXF（包括布局和视口）
/// 
/// 此函数生成包含完整 Layout/Viewport 信息的 DXF 文件
#[allow(dead_code)]
pub fn export_full(document: &Document, path: &Path) -> Result<(), FileError> {
    let mut writer = DxfWriter::new();
    
    // 1. 写入 HEADER 段
    write_header_section(&mut writer);
    
    // 2. 写入 TABLES 段
    write_tables_section(&mut writer, document);
    
    // 3. 写入 BLOCKS 段
    write_blocks_section(&mut writer, document);
    
    // 4. 写入 ENTITIES 段
    write_entities_section(&mut writer, document);
    
    // 5. 写入 OBJECTS 段
    write_objects_section(&mut writer, document);
    
    // 保存文件
    writer.save_to_file(path)
}

/// 写入 HEADER 段
fn write_header_section(writer: &mut DxfWriter) {
    writer.begin_section("HEADER");
    
    // AutoCAD 版本
    writer.write_pair(9, "$ACADVER");
    writer.write_pair(1, "AC1027"); // AutoCAD 2013 格式
    
    // 默认图层
    writer.write_pair(9, "$CLAYER");
    writer.write_pair(8, "0");
    
    writer.end_section();
}

/// 写入 TABLES 段
fn write_tables_section(writer: &mut DxfWriter, document: &Document) {
    writer.begin_section("TABLES");
    
    // VPORT 表
    writer.write_pair(0, "TABLE");
    writer.write_pair(2, "VPORT");
    writer.write_handle_only();
    writer.write_pair(70, 1);
    writer.write_pair(0, "ENDTAB");
    
    // LTYPE 表
    writer.write_pair(0, "TABLE");
    writer.write_pair(2, "LTYPE");
    writer.write_handle_only();
    writer.write_pair(70, 1);
    
    // CONTINUOUS 线型
    writer.write_pair(0, "LTYPE");
    writer.write_handle_only();
    writer.write_pair(2, "CONTINUOUS");
    writer.write_pair(70, 0);
    writer.write_pair(3, "Solid line");
    writer.write_pair(72, 65);
    writer.write_pair(73, 0);
    writer.write_pair(40, 0.0);
    
    writer.write_pair(0, "ENDTAB");
    
    // LAYER 表
    writer.write_pair(0, "TABLE");
    writer.write_pair(2, "LAYER");
    writer.write_handle_only();
    writer.write_pair(70, document.layers.all_layers().len() as i32);
    
    for layer in document.layers.all_layers() {
        writer.write_pair(0, "LAYER");
        writer.write_handle_only();
        writer.write_pair(2, &layer.name);
        writer.write_pair(70, if layer.visible { 0 } else { 1 });
        writer.write_pair(62, color_to_aci(&layer.color) as i32);
        writer.write_pair(6, "CONTINUOUS");
    }
    
    writer.write_pair(0, "ENDTAB");
    
    // BLOCK_RECORD 表
    let model_handle = writer.new_handle();
    let paper_handle = writer.new_handle();
    
    writer.write_pair(0, "TABLE");
    writer.write_pair(2, "BLOCK_RECORD");
    writer.write_handle_only();
    writer.write_pair(70, 2 + document.layout_manager.layouts().len() as i32);
    
    // *Model_Space
    writer.write_pair(0, "BLOCK_RECORD");
    writer.write_pair(5, &model_handle);
    writer.write_pair(2, "*Model_Space");
    
    // *Paper_Space
    writer.write_pair(0, "BLOCK_RECORD");
    writer.write_pair(5, &paper_handle);
    writer.write_pair(2, "*Paper_Space");
    
    writer.write_pair(0, "ENDTAB");
    
    writer.end_section();
}

/// 写入 BLOCKS 段
fn write_blocks_section(writer: &mut DxfWriter, _document: &Document) {
    writer.begin_section("BLOCKS");
    
    // *Model_Space 块
    writer.write_pair(0, "BLOCK");
    writer.write_handle_only();
    writer.write_pair(8, "0");
    writer.write_pair(2, "*Model_Space");
    writer.write_pair(70, 0);
    writer.write_pair(10, 0.0);
    writer.write_pair(20, 0.0);
    writer.write_pair(30, 0.0);
    writer.write_pair(0, "ENDBLK");
    writer.write_handle_only();
    writer.write_pair(8, "0");
    
    // *Paper_Space 块
    writer.write_pair(0, "BLOCK");
    writer.write_handle_only();
    writer.write_pair(8, "0");
    writer.write_pair(2, "*Paper_Space");
    writer.write_pair(70, 0);
    writer.write_pair(10, 0.0);
    writer.write_pair(20, 0.0);
    writer.write_pair(30, 0.0);
    writer.write_pair(0, "ENDBLK");
    writer.write_handle_only();
    writer.write_pair(8, "0");
    
    writer.end_section();
}

/// 写入 ENTITIES 段
fn write_entities_section(writer: &mut DxfWriter, document: &Document) {
    writer.begin_section("ENTITIES");
    
    // 导出模型空间实体
    for entity in document.all_entities() {
        write_entity(writer, entity, false);
    }
    
    // 导出视口和图纸空间实体
    for layout in document.layout_manager.layouts() {
        // 导出视口
        for viewport in &layout.viewports {
            write_viewport(writer, viewport);
        }
        
        // 导出图纸空间实体
        for entity in &layout.paper_space_entities {
            write_entity(writer, entity, true);
        }
    }
    
    writer.end_section();
}

/// 写入单个实体
fn write_entity(writer: &mut DxfWriter, entity: &Entity, is_paper_space: bool) {
    let Some(geometry) = entity.geometry() else { return; };
    match geometry {
        Geometry::Line(line) => {
            writer.write_pair(0, "LINE");
            writer.write_handle_only();
            if is_paper_space {
                writer.write_pair(67, 1);
            }
            writer.write_pair(8, "0");
            writer.write_pair(10, line.start.x);
            writer.write_pair(20, line.start.y);
            writer.write_pair(30, 0.0);
            writer.write_pair(11, line.end.x);
            writer.write_pair(21, line.end.y);
            writer.write_pair(31, 0.0);
        }
        Geometry::Circle(circle) => {
            writer.write_pair(0, "CIRCLE");
            writer.write_handle_only();
            if is_paper_space {
                writer.write_pair(67, 1);
            }
            writer.write_pair(8, "0");
            writer.write_pair(10, circle.center.x);
            writer.write_pair(20, circle.center.y);
            writer.write_pair(30, 0.0);
            writer.write_pair(40, circle.radius);
        }
        Geometry::Arc(arc) => {
            writer.write_pair(0, "ARC");
            writer.write_handle_only();
            if is_paper_space {
                writer.write_pair(67, 1);
            }
            writer.write_pair(8, "0");
            writer.write_pair(10, arc.center.x);
            writer.write_pair(20, arc.center.y);
            writer.write_pair(30, 0.0);
            writer.write_pair(40, arc.radius);
            writer.write_pair(50, arc.start_angle.to_degrees());
            writer.write_pair(51, arc.end_angle.to_degrees());
        }
        Geometry::Polyline(polyline) => {
            writer.write_pair(0, "LWPOLYLINE");
            writer.write_handle_only();
            if is_paper_space {
                writer.write_pair(67, 1);
            }
            writer.write_pair(8, "0");
            writer.write_pair(90, polyline.vertices.len() as i32);
            writer.write_pair(70, if polyline.closed { 1 } else { 0 });
            
            for vertex in &polyline.vertices {
                writer.write_pair(10, vertex.point.x);
                writer.write_pair(20, vertex.point.y);
                writer.write_pair(42, vertex.bulge);
            }
        }
        Geometry::Text(text) => {
            writer.write_pair(0, "TEXT");
            writer.write_handle_only();
            if is_paper_space {
                writer.write_pair(67, 1);
            }
            writer.write_pair(8, "0");
            writer.write_pair(10, text.position.x);
            writer.write_pair(20, text.position.y);
            writer.write_pair(30, 0.0);
            writer.write_pair(40, text.height);
            writer.write_pair(1, &text.content);
            writer.write_pair(50, text.rotation.to_degrees());
        }
        _ => {
            // 其他几何类型暂不支持
        }
    }
}

/// 写入视口
fn write_viewport(writer: &mut DxfWriter, viewport: &Viewport) {
    writer.write_pair(0, "VIEWPORT");
    writer.write_handle_only();
    writer.write_pair(67, 1); // 图纸空间标记
    writer.write_pair(8, "0");
    writer.write_pair(100, "AcDbEntity");
    writer.write_pair(100, "AcDbViewport");
    
    // 视口中心（图纸空间）
    let center_x = viewport.position.x + viewport.width / 2.0;
    let center_y = viewport.position.y + viewport.height / 2.0;
    writer.write_pair(10, center_x);
    writer.write_pair(20, center_y);
    writer.write_pair(30, 0.0);
    
    // 视口尺寸
    writer.write_pair(40, viewport.width);
    writer.write_pair(41, viewport.height);
    
    // 视口 ID
    writer.write_pair(69, viewport.id.0 as i32 + 1);
    
    // 视图中心（模型空间）
    writer.write_pair(12, viewport.view_center.x);
    writer.write_pair(22, viewport.view_center.y);
    
    // 视图高度
    writer.write_pair(45, viewport.height * viewport.scale);
    
    // 视口状态
    let status = match viewport.status {
        ViewportStatus::Active => 1,
        ViewportStatus::Inactive => 1,
        ViewportStatus::Locked => 1,
        ViewportStatus::Hidden => 0,
    };
    writer.write_pair(68, status);
    
    // 标准标志
    writer.write_pair(90, 32864);
}

/// 写入 OBJECTS 段
fn write_objects_section(writer: &mut DxfWriter, document: &Document) {
    writer.begin_section("OBJECTS");
    
    // 写入字典
    let dict_handle = writer.new_handle();
    writer.write_pair(0, "DICTIONARY");
    writer.write_pair(5, &dict_handle);
    writer.write_pair(100, "AcDbDictionary");
    
    // 布局字典
    let layout_dict_handle = writer.new_handle();
    writer.write_pair(3, "ACAD_LAYOUT");
    writer.write_pair(350, &layout_dict_handle);
    
    // 布局字典内容
    writer.write_pair(0, "DICTIONARY");
    writer.write_pair(5, &layout_dict_handle);
    writer.write_pair(100, "AcDbDictionary");
    
    // 写入每个布局
    for layout in document.layout_manager.layouts() {
        let layout_obj_handle = writer.new_handle();
        writer.write_pair(3, &layout.name);
        writer.write_pair(350, &layout_obj_handle);
        
        // 写入 LAYOUT 对象
        write_layout_object(writer, layout, &layout_obj_handle, &layout_dict_handle);
    }
    
    writer.end_section();
}

/// 写入 LAYOUT 对象
fn write_layout_object(
    writer: &mut DxfWriter,
    layout: &Layout,
    handle: &str,
    owner_handle: &str,
) {
    let (width, height) = layout.paper_size.dimensions_mm();
    
    writer.write_pair(0, "LAYOUT");
    writer.write_pair(5, handle);
    writer.write_pair(330, owner_handle);
    writer.write_pair(100, "AcDbPlotSettings");
    
    // 图纸设置
    writer.write_pair(1, ""); // 页面设置名
    writer.write_pair(2, "none_device"); // 打印机
    writer.write_pair(4, ""); // 图纸尺寸名
    
    // 边距
    writer.write_pair(40, layout.margins.3); // 左
    writer.write_pair(41, layout.margins.2); // 下
    writer.write_pair(42, layout.margins.1); // 右
    writer.write_pair(43, layout.margins.0); // 上
    
    // 图纸尺寸
    writer.write_pair(44, width);
    writer.write_pair(45, height);
    
    writer.write_pair(100, "AcDbLayout");
    
    // 布局名称
    writer.write_pair(1, &layout.name);
    
    // 布局标志
    writer.write_pair(70, 1);
    
    // 布局顺序
    writer.write_pair(71, layout.id.0 as i32);
}
