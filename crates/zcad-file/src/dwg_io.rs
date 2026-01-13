//! DWG 文件导入
//!
//! 使用 LibreDWG 库读取 DWG 文件并转换为 ZCAD 文档。

use std::path::Path;

use crate::document::Document;
use crate::dxf_io::conversion::aci_to_color;
use crate::error::FileError;
use zcad_core::entity::Entity;
use zcad_core::geometry::{
    Arc, Circle, Ellipse, Geometry, Line, Point, Polyline, PolylineVertex, Spline, Text,
};
use zcad_core::math::{Point2, Vector2};
use zcad_core::properties::{Color, Properties};

use zcad_libredwg::{DwgEntity, DwgEntityType, DwgFile, DwgPoint2, DwgPoint3};

/// 从 DWG 文件导入
pub fn import(path: &Path) -> Result<Document, FileError> {
    // 打开 DWG 文件
    let dwg_file = DwgFile::open(path).map_err(|e| FileError::Dwg(e.to_string()))?;

    let mut document = Document::new();

    // 导入图层
    for layer_name in dwg_file.layers() {
        if layer_name != "0" {
            // "0" 图层已经默认存在
            let layer = zcad_core::layer::Layer::new(layer_name);
            document.layers.add_layer(layer);
        }
    }

    // 导入实体
    for dwg_entity in dwg_file.entities() {
        if let Some(zcad_entity) = convert_dwg_entity(dwg_entity) {
            document.add_entity(zcad_entity);
        }
    }

    // 设置文件路径
    document.set_file_path(path);

    Ok(document)
}

/// 将 DWG 实体转换为 ZCAD 实体
fn convert_dwg_entity(dwg_entity: &DwgEntity) -> Option<Entity> {
    let geometry = convert_entity_type(&dwg_entity.entity_type)?;

    // 转换颜色
    let color = if let Some(index) = dwg_entity.color.index {
        aci_to_color(index)
    } else {
        Color::new(dwg_entity.color.r, dwg_entity.color.g, dwg_entity.color.b)
    };

    let properties = Properties::with_color(color);

    Some(Entity::new(geometry).with_properties(properties))
}

/// 转换 DWG 实体类型到 ZCAD 几何体
fn convert_entity_type(entity_type: &DwgEntityType) -> Option<Geometry> {
    match entity_type {
        DwgEntityType::Line { start, end } => {
            let line = Line::new(point3_to_point2(start), point3_to_point2(end));
            Some(Geometry::Line(line))
        }

        DwgEntityType::Circle { center, radius } => {
            let circle = Circle::new(point3_to_point2(center), *radius);
            Some(Geometry::Circle(circle))
        }

        DwgEntityType::Arc {
            center,
            radius,
            start_angle,
            end_angle,
        } => {
            let arc = Arc::new(point3_to_point2(center), *radius, *start_angle, *end_angle);
            Some(Geometry::Arc(arc))
        }

        DwgEntityType::LwPolyline {
            points,
            bulges,
            closed,
        } => {
            let vertices: Vec<PolylineVertex> = points
                .iter()
                .zip(bulges.iter())
                .map(|(p, b)| PolylineVertex::with_bulge(dwg_point2_to_point2(p), *b))
                .collect();

            if vertices.is_empty() {
                return None;
            }

            Some(Geometry::Polyline(Polyline::new(vertices, *closed)))
        }

        DwgEntityType::Polyline { points, closed } => {
            let vertices: Vec<PolylineVertex> = points
                .iter()
                .map(|p| PolylineVertex::new(point3_to_point2(p)))
                .collect();

            if vertices.is_empty() {
                return None;
            }

            Some(Geometry::Polyline(Polyline::new(vertices, *closed)))
        }

        DwgEntityType::Point { position } => {
            let point = Point::from_point2(point3_to_point2(position));
            Some(Geometry::Point(point))
        }

        DwgEntityType::Text {
            position,
            text,
            height,
            rotation,
        } => {
            let mut zcad_text = Text::new(point3_to_point2(position), text.clone(), *height);
            zcad_text.rotation = *rotation;
            Some(Geometry::Text(zcad_text))
        }

        DwgEntityType::MText {
            position,
            text,
            height,
            width: _,
        } => {
            // MText 内容可能包含格式代码，简化处理
            let content = text
                .replace("\\P", "\n")
                .replace("\\p", "\n")
                .replace("{", "")
                .replace("}", "");
            let zcad_text = Text::new(point3_to_point2(position), content, *height);
            Some(Geometry::Text(zcad_text))
        }

        DwgEntityType::Ellipse {
            center,
            major_axis,
            ratio,
            start_angle,
            end_angle,
        } => {
            let center_pt = point3_to_point2(center);
            let major = Vector2::new(major_axis.x, major_axis.y);
            let ellipse = Ellipse::arc(center_pt, major, *ratio, *start_angle, *end_angle);
            Some(Geometry::Ellipse(ellipse))
        }

        DwgEntityType::Spline {
            control_points,
            knots,
            degree,
            closed,
        } => {
            let mut spline = Spline::new(*degree as u8);
            spline.control_points = control_points.iter().map(point3_to_point2).collect();
            spline.knots = knots.clone();
            spline.closed = *closed;
            Some(Geometry::Spline(spline))
        }

        DwgEntityType::Insert { .. } => {
            // INSERT (块引用) 需要更复杂的处理
            // 目前跳过，后续可以扩展支持
            None
        }

        DwgEntityType::Dimension { .. } => {
            // 标注需要更复杂的处理
            // 目前跳过
            None
        }

        DwgEntityType::Hatch { .. } => {
            // 填充需要更复杂的处理
            // 目前跳过
            None
        }

        DwgEntityType::Unknown { .. } => None,
    }
}

/// DwgPoint3 转换为 Point2
#[inline]
fn point3_to_point2(p: &DwgPoint3) -> Point2 {
    Point2::new(p.x, p.y)
}

/// DwgPoint2 转换为 Point2
#[inline]
fn dwg_point2_to_point2(p: &DwgPoint2) -> Point2 {
    Point2::new(p.x, p.y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_conversion() {
        let dwg_pt = DwgPoint3::new(10.0, 20.0, 30.0);
        let pt = point3_to_point2(&dwg_pt);
        assert_eq!(pt.x, 10.0);
        assert_eq!(pt.y, 20.0);
    }
}
