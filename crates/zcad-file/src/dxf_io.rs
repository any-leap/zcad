//! DXF文件导入/导出
//!
//! 支持AutoCAD DXF格式的读写，包括：
//! - 模型空间实体
//! - 图纸空间（Layout）
//! - 视口（Viewport）

use crate::document::Document;
use crate::error::FileError;
use std::path::Path;
use zcad_core::entity::Entity;
use zcad_core::geometry::{
    Arc, Circle, Ellipse, Geometry, Leader, Line, Polyline, PolylineVertex, 
    Spline, Text,
};
// 布局相关导入（用于简化的布局导入功能）
// use zcad_core::layout::{Layout, Viewport, ViewportId};
use zcad_core::math::{Point2, Vector2};
use zcad_core::properties::{Color, Properties};

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

    // 导入布局信息
    // 注意：dxf crate 对 Layout/Viewport 的支持有限
    // 我们创建一个默认视口来显示模型空间的范围
    import_layouts_simplified(&drawing, &mut document);

    // 设置文件路径
    document.set_file_path(path);

    Ok(document)
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

/// 获取实体的边界范围
fn get_entity_bounds(entity: &dxf::entities::Entity) -> Option<(f64, f64, f64, f64)> {
    match &entity.specific {
        dxf::entities::EntityType::Line(line) => {
            let min_x = line.p1.x.min(line.p2.x);
            let min_y = line.p1.y.min(line.p2.y);
            let max_x = line.p1.x.max(line.p2.x);
            let max_y = line.p1.y.max(line.p2.y);
            Some((min_x, min_y, max_x, max_y))
        }
        dxf::entities::EntityType::Circle(circle) => {
            let r = circle.radius;
            Some((
                circle.center.x - r,
                circle.center.y - r,
                circle.center.x + r,
                circle.center.y + r,
            ))
        }
        dxf::entities::EntityType::Arc(arc) => {
            let r = arc.radius;
            Some((
                arc.center.x - r,
                arc.center.y - r,
                arc.center.x + r,
                arc.center.y + r,
            ))
        }
        dxf::entities::EntityType::LwPolyline(lwpoly) => {
            if lwpoly.vertices.is_empty() {
                return None;
            }
            let min_x = lwpoly.vertices.iter().map(|v| v.x).fold(f64::MAX, f64::min);
            let min_y = lwpoly.vertices.iter().map(|v| v.y).fold(f64::MAX, f64::min);
            let max_x = lwpoly.vertices.iter().map(|v| v.x).fold(f64::MIN, f64::max);
            let max_y = lwpoly.vertices.iter().map(|v| v.y).fold(f64::MIN, f64::max);
            Some((min_x, min_y, max_x, max_y))
        }
        _ => None,
    }
}

/// 将DXF实体转换为ZCAD实体
fn convert_dxf_entity(entity: &dxf::entities::Entity) -> Option<Entity> {
    let geometry = match &entity.specific {
        dxf::entities::EntityType::Line(line) => {
            let start = Point2::new(line.p1.x, line.p1.y);
            let end = Point2::new(line.p2.x, line.p2.y);
            Geometry::Line(Line::new(start, end))
        }

        dxf::entities::EntityType::Circle(circle) => {
            let center = Point2::new(circle.center.x, circle.center.y);
            Geometry::Circle(Circle::new(center, circle.radius))
        }

        dxf::entities::EntityType::Arc(arc) => {
            let center = Point2::new(arc.center.x, arc.center.y);
            let start_angle = arc.start_angle.to_radians();
            let end_angle = arc.end_angle.to_radians();
            Geometry::Arc(Arc::new(center, arc.radius, start_angle, end_angle))
        }

        dxf::entities::EntityType::LwPolyline(lwpoly) => {
            let vertices: Vec<PolylineVertex> = lwpoly
                .vertices
                .iter()
                .map(|v| PolylineVertex::with_bulge(Point2::new(v.x, v.y), v.bulge))
                .collect();

            Geometry::Polyline(Polyline::new(vertices, lwpoly.is_closed()))
        }

        dxf::entities::EntityType::Polyline(poly) => {
            let vertices: Vec<PolylineVertex> = poly
                .vertices()
                .map(|v| {
                    PolylineVertex::with_bulge(Point2::new(v.location.x, v.location.y), v.bulge)
                })
                .collect();

            Geometry::Polyline(Polyline::new(vertices, poly.is_closed()))
        }

        dxf::entities::EntityType::Text(text) => {
            let position = Point2::new(text.location.x, text.location.y);
            let height = text.text_height;
            let rotation = text.rotation.to_radians();
            let mut zcad_text = Text::new(position, text.value.clone(), height);
            zcad_text.rotation = rotation;
            Geometry::Text(zcad_text)
        }

        dxf::entities::EntityType::MText(mtext) => {
            let position = Point2::new(mtext.insertion_point.x, mtext.insertion_point.y);
            let height = mtext.initial_text_height;
            let rotation = mtext.rotation_angle.to_radians();
            // MText 内容可能包含格式代码，这里简化处理
            let content = mtext.text.replace("\\P", "\n"); // 简单的换行处理
            let mut zcad_text = Text::new(position, content, height);
            zcad_text.rotation = rotation;
            Geometry::Text(zcad_text)
        }

        dxf::entities::EntityType::ModelPoint(point) => {
            let position = Point2::new(point.location.x, point.location.y);
            Geometry::Point(zcad_core::geometry::Point::from_point2(position))
        }

        dxf::entities::EntityType::Ellipse(ellipse) => {
            let center = Point2::new(ellipse.center.x, ellipse.center.y);
            let major_axis = Vector2::new(ellipse.major_axis.x, ellipse.major_axis.y);
            let ratio = ellipse.minor_axis_ratio;
            let start_param = ellipse.start_parameter;
            let end_param = ellipse.end_parameter;
            Geometry::Ellipse(Ellipse::arc(center, major_axis, ratio, start_param, end_param))
        }

        dxf::entities::EntityType::Spline(spline) => {
            let degree = spline.degree_of_curve as u8;
            let control_points: Vec<Point2> = spline
                .control_points
                .iter()
                .map(|p| Point2::new(p.x, p.y))
                .collect();
            let knots: Vec<f64> = spline.knot_values.clone();
            let fit_points: Vec<Point2> = spline
                .fit_points
                .iter()
                .map(|p| Point2::new(p.x, p.y))
                .collect();
            let closed = spline.is_closed();
            
            let mut zcad_spline = Spline::new(degree);
            zcad_spline.control_points = control_points;
            zcad_spline.knots = knots;
            zcad_spline.fit_points = fit_points;
            zcad_spline.closed = closed;
            
            Geometry::Spline(zcad_spline)
        }

        dxf::entities::EntityType::Leader(leader) => {
            let vertices: Vec<Point2> = leader
                .vertices
                .iter()
                .map(|p| Point2::new(p.x, p.y))
                .collect();
            
            let zcad_leader = Leader::new(vertices);
            
            Geometry::Leader(zcad_leader)
        }

        dxf::entities::EntityType::RotatedDimension(dim) => {
            // RotatedDimension (AcDbRotatedDimension/AcDbAlignedDimension)
            // definition_point_2 (13) = Extension line 1 origin (Start point)
            // definition_point_3 (14) = Extension line 2 origin (End point)
            // definition_point_1 (10 in base) = Dimension line definition point
            
            let p1 = Point2::new(dim.definition_point_2.x, dim.definition_point_2.y);
            let p2 = Point2::new(dim.definition_point_3.x, dim.definition_point_3.y);
            let location = Point2::new(dim.dimension_base.definition_point_1.x, dim.dimension_base.definition_point_1.y);
            
            let mut zcad_dim = zcad_core::geometry::Dimension::new(p1, p2, location);
            
            match dim.dimension_base.dimension_type {
                dxf::enums::DimensionType::Aligned => {
                    zcad_dim.dim_type = zcad_core::geometry::DimensionType::Aligned;
                }
                _ => {
                    // Default to Linear for RotatedHorizontalOrVertical or others
                    zcad_dim.dim_type = zcad_core::geometry::DimensionType::Linear;
                }
            }
            
            if !dim.dimension_base.text.is_empty() && dim.dimension_base.text != "<>" {
                zcad_dim.text_override = Some(dim.dimension_base.text.clone());
            }
            
            // 读取文本位置 (11)
            let text_pos = Point2::new(dim.dimension_base.text_mid_point.x, dim.dimension_base.text_mid_point.y);
            // 检查是否是有效位置 (0,0可能是未设置)
            if text_pos.x.abs() > 1e-6 || text_pos.y.abs() > 1e-6 {
                zcad_dim.text_position = Some(text_pos);
            }
            
            Geometry::Dimension(zcad_dim)
        }

        dxf::entities::EntityType::RadialDimension(dim) => {
            // 10: Center (definition_point_1 in base)
            // 15: Point on curve (definition_point_2)
            let center = Point2::new(dim.dimension_base.definition_point_1.x, dim.dimension_base.definition_point_1.y);
            let point_on_curve = Point2::new(dim.definition_point_2.x, dim.definition_point_2.y);
            let text_pos = Point2::new(dim.dimension_base.text_mid_point.x, dim.dimension_base.text_mid_point.y);

            let mut zcad_dim = zcad_core::geometry::Dimension::new(center, point_on_curve, text_pos);
            zcad_dim.dim_type = zcad_core::geometry::DimensionType::Radius;

            if !dim.dimension_base.text.is_empty() && dim.dimension_base.text != "<>" {
                zcad_dim.text_override = Some(dim.dimension_base.text.clone());
            }
            
            // 半径/直径标注的 text_pos 总是有效的
            zcad_dim.text_position = Some(text_pos);

            Geometry::Dimension(zcad_dim)
        }

        dxf::entities::EntityType::DiameterDimension(dim) => {
            // 15: Point on curve (definition_point_2)
            // 10: Opposite point on curve (definition_point_1 in base)
            let p1 = Point2::new(dim.definition_point_2.x, dim.definition_point_2.y);
            let p2 = Point2::new(dim.dimension_base.definition_point_1.x, dim.dimension_base.definition_point_1.y);
            
            // Calculate center as midpoint
            let center = p1 + (p2 - p1) * 0.5;
            let text_pos = Point2::new(dim.dimension_base.text_mid_point.x, dim.dimension_base.text_mid_point.y);

            let mut zcad_dim = zcad_core::geometry::Dimension::new(center, p1, text_pos);
            zcad_dim.dim_type = zcad_core::geometry::DimensionType::Diameter;

            if !dim.dimension_base.text.is_empty() && dim.dimension_base.text != "<>" {
                zcad_dim.text_override = Some(dim.dimension_base.text.clone());
            }
            
            zcad_dim.text_position = Some(text_pos);

            Geometry::Dimension(zcad_dim)
        }

        // TODO: 支持更多实体类型
        _ => return None,
    };

    // 提取属性
    let color = entity
        .common
        .color
        .index()
        .map(|i| aci_to_color(i as u8))
        .unwrap_or(Color::BY_LAYER);

    let properties = Properties::with_color(color);

    Some(Entity::new(geometry).with_properties(properties))
}

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

/// 导出图纸空间实体
/// 
/// 注意：dxf crate 对 VIEWPORT 实体的支持有限
/// 这里只导出图纸空间的普通实体（如图框、标题栏等）
fn export_paper_space_entities(document: &Document, drawing: &mut dxf::Drawing) {
    // 遍历所有布局
    for layout in document.layout_manager.layouts() {
        // 导出图纸空间实体
        for entity in &layout.paper_space_entities {
            if let Some(dxf_entity) = convert_to_dxf_entity(entity) {
                // 注意：完整的图纸空间支持需要将实体放入正确的块记录
                // dxf crate 可能不完全支持此功能
                // 当前实现将图纸空间实体也放入模型空间
                drawing.add_entity(dxf_entity);
            }
        }
    }
    
    // 注意：完整的布局/视口导出需要：
    // 1. 创建 BLOCK_RECORD 表项
    // 2. 创建 LAYOUT 对象
    // 3. 创建 VIEWPORT 实体
    // dxf crate 可能不完全支持这些高级功能
}

/// 将ZCAD实体转换为DXF实体
fn convert_to_dxf_entity(entity: &Entity) -> Option<dxf::entities::Entity> {
    let specific = match &entity.geometry {
        Geometry::Line(line) => {
            let mut dxf_line = dxf::entities::Line::default();
            dxf_line.p1 = dxf::Point::new(line.start.x, line.start.y, 0.0);
            dxf_line.p2 = dxf::Point::new(line.end.x, line.end.y, 0.0);
            dxf::entities::EntityType::Line(dxf_line)
        }

        Geometry::Circle(circle) => {
            let mut dxf_circle = dxf::entities::Circle::default();
            dxf_circle.center = dxf::Point::new(circle.center.x, circle.center.y, 0.0);
            dxf_circle.radius = circle.radius;
            dxf::entities::EntityType::Circle(dxf_circle)
        }

        Geometry::Arc(arc) => {
            let mut dxf_arc = dxf::entities::Arc::default();
            dxf_arc.center = dxf::Point::new(arc.center.x, arc.center.y, 0.0);
            dxf_arc.radius = arc.radius;
            dxf_arc.start_angle = arc.start_angle.to_degrees();
            dxf_arc.end_angle = arc.end_angle.to_degrees();
            dxf::entities::EntityType::Arc(dxf_arc)
        }

        Geometry::Polyline(polyline) => {
            let mut lwpoly = dxf::entities::LwPolyline::default();
            lwpoly.set_is_closed(polyline.closed);
            lwpoly.vertices = polyline
                .vertices
                .iter()
                .map(|v| {
                    let mut vertex = dxf::LwPolylineVertex::default();
                    vertex.x = v.point.x;
                    vertex.y = v.point.y;
                    vertex.bulge = v.bulge;
                    vertex
                })
                .collect();
            dxf::entities::EntityType::LwPolyline(lwpoly)
        }

        Geometry::Point(point) => {
            let mut dxf_point = dxf::entities::ModelPoint::default();
            dxf_point.location = dxf::Point::new(point.position.x, point.position.y, 0.0);
            dxf::entities::EntityType::ModelPoint(dxf_point)
        }

        Geometry::Text(text) => {
            let mut dxf_text = dxf::entities::Text::default();
            dxf_text.location = dxf::Point::new(text.position.x, text.position.y, 0.0);
            dxf_text.text_height = text.height;
            dxf_text.value = text.content.clone();
            dxf_text.rotation = text.rotation.to_degrees();
            dxf::entities::EntityType::Text(dxf_text)
        }
        Geometry::Dimension(dim) => {
            let mut base = dxf::entities::DimensionBase::default();
            
            // 设置文本位置 (11)
            // base.text_mid_point = dxf::Point::new(dim.line_location.x, dim.line_location.y, 0.0);
            
            // 设置文本内容
            if let Some(text) = &dim.text_override {
                base.text = text.clone();
            } else {
                // 空字符串表示使用测量值
                base.text = String::new();
            }

            // 设置文本位置 (11) - 如果有自定义位置，使用它；否则使用默认计算位置
            let text_pos = dim.get_text_position();
            base.text_mid_point = dxf::Point::new(text_pos.x, text_pos.y, 0.0);
            
            match dim.dim_type {
                zcad_core::geometry::DimensionType::Radius => {
                    base.dimension_type = dxf::enums::DimensionType::Radius;
                    
                    // 10: Center (p1)
                    base.definition_point_1 = dxf::Point::new(dim.definition_point1.x, dim.definition_point1.y, 0.0);
                    
                    let mut dxf_dim = dxf::entities::RadialDimension::default();
                    dxf_dim.dimension_base = base;
                    
                    // 15: Point on curve (p2)
                    dxf_dim.definition_point_2 = dxf::Point::new(dim.definition_point2.x, dim.definition_point2.y, 0.0);
                    
                    dxf::entities::EntityType::RadialDimension(dxf_dim)
                },
                zcad_core::geometry::DimensionType::Diameter => {
                    base.dimension_type = dxf::enums::DimensionType::Diameter;
                    
                    // 10: Opposite point
                    let opposite = dim.definition_point1 + (dim.definition_point1 - dim.definition_point2);
                    base.definition_point_1 = dxf::Point::new(opposite.x, opposite.y, 0.0);
                    
                    let mut dxf_dim = dxf::entities::DiameterDimension::default();
                    dxf_dim.dimension_base = base;
                    
                    // 15: Point on curve (p2)
                    dxf_dim.definition_point_2 = dxf::Point::new(dim.definition_point2.x, dim.definition_point2.y, 0.0);
                    
                    dxf::entities::EntityType::DiameterDimension(dxf_dim)
                },
                _ => {
                    // definition_point_1 (10) = Dimension line definition point
                    base.definition_point_1 = dxf::Point::new(dim.line_location.x, dim.line_location.y, 0.0);
                    
                    let mut dxf_dim = dxf::entities::RotatedDimension::default();
                    
                    if dim.dim_type == zcad_core::geometry::DimensionType::Aligned {
                         base.dimension_type = dxf::enums::DimensionType::Aligned;
                    } else {
                         base.dimension_type = dxf::enums::DimensionType::RotatedHorizontalOrVertical;
                    }

                    dxf_dim.dimension_base = base;
                    
                    // definition_point_2 (13) = Extension line 1 origin (Start point)
                    dxf_dim.definition_point_2 = dxf::Point::new(dim.definition_point1.x, dim.definition_point1.y, 0.0);
                    // definition_point_3 (14) = Extension line 2 origin (End point)
                    dxf_dim.definition_point_3 = dxf::Point::new(dim.definition_point2.x, dim.definition_point2.y, 0.0);
                    
                    // insertion_point (12)
                    dxf_dim.insertion_point = dxf::Point::new(dim.line_location.x, dim.line_location.y, 0.0);
                    
                    dxf::entities::EntityType::RotatedDimension(dxf_dim)
                }
            }
        }

        Geometry::Ellipse(ellipse) => {
            let mut dxf_ellipse = dxf::entities::Ellipse::default();
            dxf_ellipse.center = dxf::Point::new(ellipse.center.x, ellipse.center.y, 0.0);
            dxf_ellipse.major_axis = dxf::Vector::new(ellipse.major_axis.x, ellipse.major_axis.y, 0.0);
            dxf_ellipse.minor_axis_ratio = ellipse.ratio;
            dxf_ellipse.start_parameter = ellipse.start_param;
            dxf_ellipse.end_parameter = ellipse.end_param;
            dxf::entities::EntityType::Ellipse(dxf_ellipse)
        }

        Geometry::Spline(spline) => {
            let mut dxf_spline = dxf::entities::Spline::default();
            dxf_spline.degree_of_curve = spline.degree as i32;
            dxf_spline.control_points = spline
                .control_points
                .iter()
                .map(|p| dxf::Point::new(p.x, p.y, 0.0))
                .collect();
            dxf_spline.knot_values = spline.knots.clone();
            dxf_spline.fit_points = spline
                .fit_points
                .iter()
                .map(|p| dxf::Point::new(p.x, p.y, 0.0))
                .collect();
            if spline.closed {
                dxf_spline.flags |= 1; // Closed spline
            }
            dxf::entities::EntityType::Spline(dxf_spline)
        }

        Geometry::Hatch(_hatch) => {
            // TODO: 实现完整的 Hatch 导出
            // 当前跳过填充，因为 DXF Hatch 结构复杂
            return None;
        }

        Geometry::Leader(leader) => {
            let mut dxf_leader = dxf::entities::Leader::default();
            dxf_leader.vertices = leader
                .vertices
                .iter()
                .map(|p| dxf::Point::new(p.x, p.y, 0.0))
                .collect();
            dxf::entities::EntityType::Leader(dxf_leader)
        }
    };

    let mut dxf_entity = dxf::entities::Entity::new(specific);

    // 设置颜色
    if !entity.properties.color.is_by_layer() {
        dxf_entity.common.color =
            dxf::Color::from_index(color_to_aci(&entity.properties.color));
    }

    Some(dxf_entity)
}

/// AutoCAD颜色索引(ACI)转ZCAD颜色
fn aci_to_color(aci: u8) -> Color {
    match aci {
        1 => Color::RED,
        2 => Color::YELLOW,
        3 => Color::GREEN,
        4 => Color::CYAN,
        5 => Color::BLUE,
        6 => Color::MAGENTA,
        7 => Color::WHITE,
        8 => Color::GRAY,
        _ => Color::WHITE,
    }
}

/// ZCAD颜色转AutoCAD颜色索引
fn color_to_aci(color: &Color) -> u8 {
    if color.is_by_layer() || color.is_by_block() {
        return 7; // 默认白色（ByLayer/ByBlock在其他地方处理）
    }

    // 简单的颜色匹配
    match (color.r, color.g, color.b) {
        (255, 0, 0) => 1,
        (255, 255, 0) => 2,
        (0, 255, 0) => 3,
        (0, 255, 255) => 4,
        (0, 0, 255) => 5,
        (255, 0, 255) => 6,
        (255, 255, 255) => 7,
        (128, 128, 128) => 8,
        _ => 7, // 默认白色
    }
}

