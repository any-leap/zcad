//! DWG 文件导入
//!
//! 支持两种方式：
//! 1. LibreDWG 直接解析（支持较旧版本）
//! 2. ODA File Converter 转换为 DXF（支持所有版本，需要用户安装）

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::document::Document;
use crate::dxf_io::conversion::aci_to_color;
use crate::error::FileError;
use tracing::{info, warn};
use zcad_core::entity::Entity;
use zcad_core::geometry::{
    Arc, Circle, Ellipse, Geometry, Line, Point, Polyline, PolylineVertex, Spline, Text,
};
use zcad_core::math::{Point2, Vector2};
use zcad_core::properties::{Color, Properties};

use zcad_libredwg::{DwgEntity, DwgEntityType, DwgFile, DwgPoint2, DwgPoint3};

/// 从 DWG 文件导入
/// 
/// 首先尝试使用 LibreDWG 直接解析，如果失败则尝试使用 ODA File Converter 转换为 DXF
pub fn import(path: &Path) -> Result<Document, FileError> {
    // 首先尝试 LibreDWG
    match import_with_libredwg(path) {
        Ok(doc) => {
            info!("Successfully opened DWG with LibreDWG: {}", path.display());
            return Ok(doc);
        }
        Err(e) => {
            warn!("LibreDWG failed to open {}: {}", path.display(), e);
            
            // 尝试使用 ODA File Converter
            if let Some(converter_path) = find_oda_converter() {
                info!("Trying ODA File Converter at: {}", converter_path.display());
                match import_with_oda_converter(path, &converter_path) {
                    Ok(doc) => {
                        info!("Successfully opened DWG via ODA converter");
                        return Ok(doc);
                    }
                    Err(oda_err) => {
                        warn!("ODA converter also failed: {}", oda_err);
                        // 返回原始 LibreDWG 错误，因为它更有意义
                        return Err(FileError::Dwg(format!(
                            "LibreDWG: {}. ODA converter: {}", e, oda_err
                        )));
                    }
                }
            } else {
                // 没有找到 ODA converter，返回带建议的错误
                return Err(FileError::Dwg(format!(
                    "{}. 建议: 安装免费的 ODA File Converter (https://www.opendesign.com/guestfiles/oda_file_converter) 以支持所有 DWG 版本，或将 DWG 另存为 DXF 格式。",
                    e
                )));
            }
        }
    }
}

/// 使用 LibreDWG 直接导入
fn import_with_libredwg(path: &Path) -> Result<Document, FileError> {
    let dwg_file = DwgFile::open(path).map_err(|e| FileError::Dwg(e.to_string()))?;

    let mut document = Document::new();

    // 导入图层
    for layer_name in dwg_file.layers() {
        if layer_name != "0" {
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

    document.set_file_path(path);
    Ok(document)
}

/// 查找 ODA File Converter
fn find_oda_converter() -> Option<PathBuf> {
    // Windows 常见安装路径
    #[cfg(windows)]
    {
        // 先尝试在 ODA 目录下搜索任何版本
        let oda_dir = PathBuf::from(r"C:\Program Files\ODA");
        if oda_dir.exists() {
            if let Ok(entries) = std::fs::read_dir(&oda_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        let converter = path.join("ODAFileConverter.exe");
                        if converter.exists() {
                            return Some(converter);
                        }
                    }
                }
            }
        }
        
        // 备用固定路径
        let common_paths = [
            r"C:\Program Files\ODA\ODAFileConverter\ODAFileConverter.exe",
            r"C:\Program Files (x86)\ODA\ODAFileConverter\ODAFileConverter.exe",
            r"C:\Program Files\ODA\ODAFileConverter 26.10.0\ODAFileConverter.exe",
            r"C:\Program Files\ODA\ODAFileConverter 25.12\ODAFileConverter.exe",
            r"C:\Program Files\ODA\ODAFileConverter 24.12\ODAFileConverter.exe",
        ];
        
        for path_str in &common_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            }
        }
        
        // 尝试在 PATH 中查找
        if let Ok(output) = Command::new("where").arg("ODAFileConverter").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = path_str.lines().next() {
                    let path = PathBuf::from(first_line.trim());
                    if path.exists() {
                        return Some(path);
                    }
                }
            }
        }
    }
    
    // macOS/Linux
    #[cfg(not(windows))]
    {
        let common_paths = [
            "/usr/bin/ODAFileConverter",
            "/usr/local/bin/ODAFileConverter",
            "/opt/ODAFileConverter/ODAFileConverter",
        ];
        
        for path_str in &common_paths {
            let path = PathBuf::from(path_str);
            if path.exists() {
                return Some(path);
            }
        }
        
        // 尝试 which
        if let Ok(output) = Command::new("which").arg("ODAFileConverter").output() {
            if output.status.success() {
                let path_str = String::from_utf8_lossy(&output.stdout);
                let path = PathBuf::from(path_str.trim());
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }
    
    None
}

/// 使用 ODA File Converter 转换并导入
fn import_with_oda_converter(dwg_path: &Path, converter_path: &Path) -> Result<Document, FileError> {
    // 创建临时目录
    let temp_dir = std::env::temp_dir().join("zcad_dwg_convert");
    std::fs::create_dir_all(&temp_dir)
        .map_err(|e| FileError::Dwg(format!("Failed to create temp dir: {}", e)))?;
    
    // 获取输入文件所在目录和文件名
    let input_dir = dwg_path.parent()
        .ok_or_else(|| FileError::Dwg("Invalid DWG path".to_string()))?;
    let file_name = dwg_path.file_stem()
        .ok_or_else(|| FileError::Dwg("Invalid DWG filename".to_string()))?;
    
    // ODA File Converter 命令行参数:
    // ODAFileConverter <input_folder> <output_folder> <output_version> <output_type> <recurse> <audit> [filter]
    // output_version: ACAD2018, ACAD2013, ACAD2010, ACAD2007, ACAD2004, ACAD2000, ACAD14, ACAD13, ACAD12
    // output_type: DWG, DXF, DXB
    
    let dwg_filename = dwg_path.file_name()
        .ok_or_else(|| FileError::Dwg("Invalid filename".to_string()))?
        .to_string_lossy();
    
    info!("Converting {} to DXF using ODA File Converter...", dwg_filename);
    
    let output = Command::new(converter_path)
        .arg(input_dir)
        .arg(&temp_dir)
        .arg("ACAD2018")  // 输出版本
        .arg("DXF")       // 输出格式
        .arg("0")         // 不递归
        .arg("1")         // 审核修复
        .arg(format!("{}.dwg", file_name.to_string_lossy()))  // 只转换这个文件
        .output()
        .map_err(|e| FileError::Dwg(format!("Failed to run ODA converter: {}", e)))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(FileError::Dwg(format!("ODA converter failed: {}", stderr)));
    }
    
    // 查找转换后的 DXF 文件
    let dxf_path = temp_dir.join(format!("{}.dxf", file_name.to_string_lossy()));
    
    if !dxf_path.exists() {
        // 尝试其他可能的命名
        let entries = std::fs::read_dir(&temp_dir)
            .map_err(|e| FileError::Dwg(format!("Failed to read temp dir: {}", e)))?;
        
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "dxf").unwrap_or(false) {
                // 找到 DXF 文件，导入它
                let result = crate::dxf_io::import(&path);
                // 清理临时文件
                let _ = std::fs::remove_file(&path);
                return result;
            }
        }
        
        return Err(FileError::Dwg("ODA converter did not produce DXF output".to_string()));
    }
    
    // 导入转换后的 DXF
    let result = crate::dxf_io::import(&dxf_path);
    
    // 清理临时文件
    let _ = std::fs::remove_file(&dxf_path);
    
    // 更新文件路径为原始 DWG 路径
    result.map(|mut doc| {
        doc.set_file_path(dwg_path);
        doc
    })
}

/// 将 DWG 实体转换为 ZCAD 实体
fn convert_dwg_entity(dwg_entity: &DwgEntity) -> Option<Entity> {
    let geometry = convert_entity_type(&dwg_entity.entity_type)?;

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

        DwgEntityType::Insert { .. } => None,
        DwgEntityType::Dimension { .. } => None,
        DwgEntityType::Hatch { .. } => None,
        DwgEntityType::Unknown { .. } => None,
    }
}

#[inline]
fn point3_to_point2(p: &DwgPoint3) -> Point2 {
    Point2::new(p.x, p.y)
}

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
    
    #[test]
    fn test_find_oda_converter() {
        // 这个测试只验证函数不会崩溃
        let _ = find_oda_converter();
    }
}
