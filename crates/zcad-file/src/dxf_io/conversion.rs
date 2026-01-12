//! DXF 与 ZCAD 之间的转换

use zcad_core::entity::Entity;
use zcad_core::geometry::{
    Arc, Circle, Ellipse, Geometry, Leader, Line, Polyline, PolylineVertex, 
    Spline, Text,
};
use zcad_core::math::{Point2, Vector2};
use zcad_core::properties::{Color, Properties};

/// 将DXF实体转换为ZCAD实体
pub fn convert_dxf_entity(entity: &dxf::entities::Entity) -> Option<Entity> {
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
            let p1 = Point2::new(dim.definition_point_2.x, dim.definition_point_2.y);
            let p2 = Point2::new(dim.definition_point_3.x, dim.definition_point_3.y);
            let location = Point2::new(dim.dimension_base.definition_point_1.x, dim.dimension_base.definition_point_1.y);
            
            let mut zcad_dim = zcad_core::geometry::Dimension::new(p1, p2, location);
            
            match dim.dimension_base.dimension_type {
                dxf::enums::DimensionType::Aligned => {
                    zcad_dim.dim_type = zcad_core::geometry::DimensionType::Aligned;
                }
                _ => {
                    zcad_dim.dim_type = zcad_core::geometry::DimensionType::Linear;
                }
            }
            
            if !dim.dimension_base.text.is_empty() && dim.dimension_base.text != "<>" {
                zcad_dim.text_override = Some(dim.dimension_base.text.clone());
            }
            
            let text_pos = Point2::new(dim.dimension_base.text_mid_point.x, dim.dimension_base.text_mid_point.y);
            if text_pos.x.abs() > 1e-6 || text_pos.y.abs() > 1e-6 {
                zcad_dim.text_position = Some(text_pos);
            }
            
            Geometry::Dimension(zcad_dim)
        }

        dxf::entities::EntityType::RadialDimension(dim) => {
            let center = Point2::new(dim.dimension_base.definition_point_1.x, dim.dimension_base.definition_point_1.y);
            let point_on_curve = Point2::new(dim.definition_point_2.x, dim.definition_point_2.y);
            let text_pos = Point2::new(dim.dimension_base.text_mid_point.x, dim.dimension_base.text_mid_point.y);

            let mut zcad_dim = zcad_core::geometry::Dimension::new(center, point_on_curve, text_pos);
            zcad_dim.dim_type = zcad_core::geometry::DimensionType::Radius;

            if !dim.dimension_base.text.is_empty() && dim.dimension_base.text != "<>" {
                zcad_dim.text_override = Some(dim.dimension_base.text.clone());
            }
            
            zcad_dim.text_position = Some(text_pos);

            Geometry::Dimension(zcad_dim)
        }

        dxf::entities::EntityType::DiameterDimension(dim) => {
            let p1 = Point2::new(dim.definition_point_2.x, dim.definition_point_2.y);
            let p2 = Point2::new(dim.dimension_base.definition_point_1.x, dim.dimension_base.definition_point_1.y);
            
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

/// 将ZCAD实体转换为DXF实体
pub fn convert_to_dxf_entity(entity: &Entity) -> Option<dxf::entities::Entity> {
    let geometry = entity.geometry()?;

    let dxf_entity = match geometry {
        Geometry::Line(line) => {
            let dxf_line = dxf::entities::Line::new(
                dxf::Point::new(line.start.x, line.start.y, 0.0),
                dxf::Point::new(line.end.x, line.end.y, 0.0),
            );
            dxf::entities::Entity::new(dxf::entities::EntityType::Line(dxf_line))
        }

        Geometry::Circle(circle) => {
            let dxf_circle = dxf::entities::Circle {
                center: dxf::Point::new(circle.center.x, circle.center.y, 0.0),
                radius: circle.radius,
                ..Default::default()
            };
            dxf::entities::Entity::new(dxf::entities::EntityType::Circle(dxf_circle))
        }

        Geometry::Arc(arc) => {
            let dxf_arc = dxf::entities::Arc {
                center: dxf::Point::new(arc.center.x, arc.center.y, 0.0),
                radius: arc.radius,
                start_angle: arc.start_angle.to_degrees(),
                end_angle: arc.end_angle.to_degrees(),
                ..Default::default()
            };
            dxf::entities::Entity::new(dxf::entities::EntityType::Arc(dxf_arc))
        }

        Geometry::Polyline(polyline) => {
            let mut dxf_lwpoly = dxf::entities::LwPolyline::default();
            dxf_lwpoly.set_is_closed(polyline.closed);
            for v in &polyline.vertices {
                let vertex = dxf::LwPolylineVertex {
                    x: v.point.x,
                    y: v.point.y,
                    id: 0,
                    starting_width: 0.0,
                    ending_width: 0.0,
                    bulge: v.bulge,
                };
                dxf_lwpoly.vertices.push(vertex);
            }
            dxf::entities::Entity::new(dxf::entities::EntityType::LwPolyline(dxf_lwpoly))
        }

        Geometry::Text(text) => {
            let dxf_text = dxf::entities::Text {
                location: dxf::Point::new(text.position.x, text.position.y, 0.0),
                text_height: text.height,
                value: text.content.clone(),
                rotation: text.rotation.to_degrees(),
                ..Default::default()
            };
            dxf::entities::Entity::new(dxf::entities::EntityType::Text(dxf_text))
        }

        Geometry::Dimension(dim) => {
            let mut dxf_dim = dxf::entities::RotatedDimension::default();
            dxf_dim.definition_point_2 = dxf::Point::new(dim.definition_point1.x, dim.definition_point1.y, 0.0);
            dxf_dim.definition_point_3 = dxf::Point::new(dim.definition_point2.x, dim.definition_point2.y, 0.0);
            dxf_dim.dimension_base.definition_point_1 = dxf::Point::new(dim.line_location.x, dim.line_location.y, 0.0);
            
            if let Some(override_text) = &dim.text_override {
                dxf_dim.dimension_base.text = override_text.clone();
            }
            
            if let Some(text_pos) = dim.text_position {
                dxf_dim.dimension_base.text_mid_point = dxf::Point::new(text_pos.x, text_pos.y, 0.0);
            }
            
            dxf::entities::Entity::new(dxf::entities::EntityType::RotatedDimension(dxf_dim))
        }

        Geometry::Point(p) => {
            let dxf_point = dxf::entities::ModelPoint {
                location: dxf::Point::new(p.position.x, p.position.y, 0.0),
                ..Default::default()
            };
            dxf::entities::Entity::new(dxf::entities::EntityType::ModelPoint(dxf_point))
        }

        Geometry::Ellipse(ellipse) => {
            let dxf_ellipse = dxf::entities::Ellipse {
                center: dxf::Point::new(ellipse.center.x, ellipse.center.y, 0.0),
                major_axis: dxf::Vector::new(ellipse.major_axis.x, ellipse.major_axis.y, 0.0),
                minor_axis_ratio: ellipse.ratio,
                start_parameter: ellipse.start_param,
                end_parameter: ellipse.end_param,
                ..Default::default()
            };
            dxf::entities::Entity::new(dxf::entities::EntityType::Ellipse(dxf_ellipse))
        }

        Geometry::Spline(spline) => {
            let mut dxf_spline = dxf::entities::Spline::default();
            dxf_spline.degree_of_curve = spline.degree as i32;
            dxf_spline.control_points = spline.control_points
                .iter()
                .map(|p| dxf::Point::new(p.x, p.y, 0.0))
                .collect();
            dxf_spline.knot_values = spline.knots.clone();
            dxf_spline.fit_points = spline.fit_points
                .iter()
                .map(|p| dxf::Point::new(p.x, p.y, 0.0))
                .collect();
            dxf::entities::Entity::new(dxf::entities::EntityType::Spline(dxf_spline))
        }

        Geometry::Leader(leader) => {
            let mut dxf_leader = dxf::entities::Leader::default();
            dxf_leader.vertices = leader.vertices
                .iter()
                .map(|p| dxf::Point::new(p.x, p.y, 0.0))
                .collect();
            dxf::entities::Entity::new(dxf::entities::EntityType::Leader(dxf_leader))
        }

        _ => return None,
    };

    let mut dxf_entity = dxf_entity;
    dxf_entity.common.color =
            dxf::Color::from_index(color_to_aci(&entity.visual_properties.color));

    Some(dxf_entity)
}

/// 获取实体的边界范围
pub fn get_entity_bounds(entity: &dxf::entities::Entity) -> Option<(f64, f64, f64, f64)> {
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

/// AutoCAD颜色索引(ACI)转ZCAD颜色
pub fn aci_to_color(aci: u8) -> Color {
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
pub fn color_to_aci(color: &Color) -> u8 {
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
