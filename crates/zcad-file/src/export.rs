//! 导出模块
//!
//! 支持将 CAD 图纸导出为多种格式：PDF、SVG、PNG、JPG

use crate::error::FileError;
use zcad_core::entity::Entity;
use zcad_core::geometry::Geometry;
use zcad_core::math::{Point2, Vector2};
use zcad_core::properties::Color;

/// 纸张大小
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaperSize {
    A4,
    A3,
    A2,
    A1,
    A0,
    Letter,
    Legal,
    Tabloid,
    Custom { width: f64, height: f64 },
}

impl PaperSize {
    /// 获取纸张尺寸（毫米）
    pub fn dimensions_mm(&self) -> (f64, f64) {
        match self {
            PaperSize::A4 => (210.0, 297.0),
            PaperSize::A3 => (297.0, 420.0),
            PaperSize::A2 => (420.0, 594.0),
            PaperSize::A1 => (594.0, 841.0),
            PaperSize::A0 => (841.0, 1189.0),
            PaperSize::Letter => (215.9, 279.4),
            PaperSize::Legal => (215.9, 355.6),
            PaperSize::Tabloid => (279.4, 431.8),
            PaperSize::Custom { width, height } => (*width, *height),
        }
    }
}

/// 纸张方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Orientation {
    Portrait,
    Landscape,
}

/// 页面设置
#[derive(Debug, Clone)]
pub struct PageSetup {
    /// 纸张大小
    pub paper_size: PaperSize,
    /// 纸张方向
    pub orientation: Orientation,
    /// 边距（毫米）：上、右、下、左
    pub margins: (f64, f64, f64, f64),
    /// 缩放比例（1:X）
    pub scale: f64,
    /// 是否适应页面
    pub fit_to_page: bool,
    /// 打印范围：None = 全部，Some = 指定区域
    pub print_area: Option<PrintArea>,
}

impl Default for PageSetup {
    fn default() -> Self {
        Self {
            paper_size: PaperSize::A4,
            orientation: Orientation::Landscape,
            margins: (10.0, 10.0, 10.0, 10.0), // 10mm 边距
            scale: 1.0,
            fit_to_page: true,
            print_area: None,
        }
    }
}

impl PageSetup {
    /// 获取可打印区域尺寸（毫米）
    pub fn printable_size(&self) -> (f64, f64) {
        let (paper_w, paper_h) = self.paper_size.dimensions_mm();
        let (w, h) = match self.orientation {
            Orientation::Portrait => (paper_w, paper_h),
            Orientation::Landscape => (paper_h, paper_w),
        };
        let (top, right, bottom, left) = self.margins;
        (w - left - right, h - top - bottom)
    }
}

/// 打印区域
#[derive(Debug, Clone)]
pub struct PrintArea {
    /// 最小点
    pub min: Point2,
    /// 最大点
    pub max: Point2,
}

impl PrintArea {
    pub fn new(min: Point2, max: Point2) -> Self {
        Self { min, max }
    }

    pub fn width(&self) -> f64 {
        self.max.x - self.min.x
    }

    pub fn height(&self) -> f64 {
        self.max.y - self.min.y
    }

    pub fn center(&self) -> Point2 {
        Point2::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }
}

/// SVG 导出器
pub struct SvgExporter {
    page_setup: PageSetup,
}

impl SvgExporter {
    pub fn new(page_setup: PageSetup) -> Self {
        Self { page_setup }
    }

    /// 将 LineWeight 转换为毫米值
    fn line_weight_to_mm(&self, line_weight: &zcad_core::properties::LineWeight) -> f64 {
        use zcad_core::properties::LineWeight;
        match line_weight {
            LineWeight::Default => 0.25,
            LineWeight::ByLayer => 0.25,
            LineWeight::ByBlock => 0.25,
            LineWeight::Width(w) => *w,
        }
    }

    /// 导出实体为 SVG 字符串
    pub fn export(&self, entities: &[Entity]) -> Result<String, FileError> {
        // 计算所有实体的包围盒
        let bounds = self.calculate_bounds(entities);
        
        // 获取页面尺寸
        let (page_width, page_height) = self.page_setup.printable_size();
        
        // 计算缩放和偏移
        let (scale, offset) = self.calculate_transform(&bounds, page_width, page_height);
        
        // 生成 SVG
        let mut svg = String::new();
        
        // SVG 头部
        svg.push_str(&format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<svg xmlns="http://www.w3.org/2000/svg" 
     width="{:.2}mm" height="{:.2}mm"
     viewBox="0 0 {:.2} {:.2}">
  <g transform="translate({:.2},{:.2}) scale({:.6},-{:.6})">
"#,
            page_width, page_height,
            page_width, page_height,
            offset.x, page_height - offset.y,
            scale, scale
        ));

        // 添加背景（可选）
        svg.push_str(&format!(
            r#"    <rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="white" transform="scale(1,-1) translate(0,-{:.2})"/>
"#,
            0.0, 0.0, page_width / scale, page_height / scale, page_height / scale
        ));

        // 渲染每个实体
        for entity in entities {
            let color = &entity.properties.color;
            let stroke_width = self.line_weight_to_mm(&entity.properties.line_weight).max(0.1);
            
            if let Some(svg_elem) = self.geometry_to_svg(&entity.geometry, color, stroke_width) {
                svg.push_str(&format!("    {}\n", svg_elem));
            }
        }

        // SVG 尾部
        svg.push_str("  </g>\n</svg>\n");

        Ok(svg)
    }

    /// 计算所有实体的包围盒
    fn calculate_bounds(&self, entities: &[Entity]) -> PrintArea {
        if entities.is_empty() {
            return PrintArea::new(Point2::origin(), Point2::new(100.0, 100.0));
        }

        let mut min_x = f64::MAX;
        let mut min_y = f64::MAX;
        let mut max_x = f64::MIN;
        let mut max_y = f64::MIN;

        for entity in entities {
            let bbox = entity.geometry.bounding_box();
            min_x = min_x.min(bbox.min.x);
            min_y = min_y.min(bbox.min.y);
            max_x = max_x.max(bbox.max.x);
            max_y = max_y.max(bbox.max.y);
        }

        // 如果有指定打印区域，使用它
        if let Some(ref area) = self.page_setup.print_area {
            return area.clone();
        }

        PrintArea::new(Point2::new(min_x, min_y), Point2::new(max_x, max_y))
    }

    /// 计算变换参数
    fn calculate_transform(&self, bounds: &PrintArea, page_width: f64, page_height: f64) -> (f64, Vector2) {
        let content_width = bounds.width();
        let content_height = bounds.height();
        
        let scale = if self.page_setup.fit_to_page {
            let scale_x = page_width / content_width;
            let scale_y = page_height / content_height;
            scale_x.min(scale_y) * 0.95 // 留一点边距
        } else {
            self.page_setup.scale
        };
        
        // 居中偏移
        let scaled_width = content_width * scale;
        let scaled_height = content_height * scale;
        let offset_x = (page_width - scaled_width) / 2.0 - bounds.min.x * scale;
        let offset_y = (page_height - scaled_height) / 2.0 - bounds.min.y * scale;
        
        (scale, Vector2::new(offset_x, offset_y))
    }

    /// 将几何体转换为 SVG 元素
    fn geometry_to_svg(&self, geometry: &Geometry, color: &Color, stroke_width: f64) -> Option<String> {
        let stroke_color = format!("rgb({},{},{})", color.r, color.g, color.b);
        let style = format!(
            r#"stroke="{}" stroke-width="{:.2}" fill="none""#,
            stroke_color, stroke_width
        );

        match geometry {
            Geometry::Line(line) => Some(format!(
                r#"<line x1="{:.4}" y1="{:.4}" x2="{:.4}" y2="{:.4}" {}/>"#,
                line.start.x, line.start.y, line.end.x, line.end.y, style
            )),
            Geometry::Circle(circle) => Some(format!(
                r#"<circle cx="{:.4}" cy="{:.4}" r="{:.4}" {}/>"#,
                circle.center.x, circle.center.y, circle.radius, style
            )),
            Geometry::Arc(arc) => {
                let start_x = arc.center.x + arc.radius * arc.start_angle.cos();
                let start_y = arc.center.y + arc.radius * arc.start_angle.sin();
                let end_x = arc.center.x + arc.radius * arc.end_angle.cos();
                let end_y = arc.center.y + arc.radius * arc.end_angle.sin();
                
                let sweep_angle = arc.end_angle - arc.start_angle;
                let large_arc = if sweep_angle.abs() > std::f64::consts::PI { 1 } else { 0 };
                let sweep = if sweep_angle > 0.0 { 1 } else { 0 };
                
                Some(format!(
                    r#"<path d="M {:.4} {:.4} A {:.4} {:.4} 0 {} {} {:.4} {:.4}" {}/>"#,
                    start_x, start_y,
                    arc.radius, arc.radius,
                    large_arc, sweep,
                    end_x, end_y,
                    style
                ))
            }
            Geometry::Point(point) => {
                let size = 1.0;
                Some(format!(
                    r#"<circle cx="{:.4}" cy="{:.4}" r="{:.4}" fill="{}" stroke="none"/>"#,
                    point.position.x, point.position.y, size, stroke_color
                ))
            }
            Geometry::Polyline(polyline) => {
                if polyline.vertices.is_empty() {
                    return None;
                }
                
                let mut path = String::new();
                for (i, vertex) in polyline.vertices.iter().enumerate() {
                    if i == 0 {
                        path.push_str(&format!("M {:.4} {:.4}", vertex.point.x, vertex.point.y));
                    } else {
                        path.push_str(&format!(" L {:.4} {:.4}", vertex.point.x, vertex.point.y));
                    }
                }
                if polyline.closed {
                    path.push_str(" Z");
                }
                
                Some(format!(r#"<path d="{}" {}/>"#, path, style))
            }
            Geometry::Ellipse(ellipse) => {
                let major_len = ellipse.major_axis.norm();
                let minor_len = major_len * ellipse.ratio;
                let rotation = ellipse.major_axis.y.atan2(ellipse.major_axis.x) * 180.0 / std::f64::consts::PI;
                
                Some(format!(
                    r#"<ellipse cx="{:.4}" cy="{:.4}" rx="{:.4}" ry="{:.4}" transform="rotate({:.2} {:.4} {:.4})" {}/>"#,
                    ellipse.center.x, ellipse.center.y,
                    major_len, minor_len,
                    rotation, ellipse.center.x, ellipse.center.y,
                    style
                ))
            }
            Geometry::Spline(spline) => {
                // 简化处理：使用多段线近似
                if spline.control_points.len() < 2 {
                    return None;
                }
                
                let mut path = String::new();
                let first = &spline.control_points[0];
                path.push_str(&format!("M {:.4} {:.4}", first.x, first.y));
                
                // 对于简单情况，直接连接控制点
                for point in spline.control_points.iter().skip(1) {
                    path.push_str(&format!(" L {:.4} {:.4}", point.x, point.y));
                }
                
                if spline.closed {
                    path.push_str(" Z");
                }
                
                Some(format!(r#"<path d="{}" {}/>"#, path, style))
            }
            Geometry::Text(text) => {
                // 简单的文本渲染
                let font_size = text.height;
                Some(format!(
                    r#"<text x="{:.4}" y="{:.4}" font-size="{:.2}" fill="{}" transform="scale(1,-1) translate(0,{:.4})">{}</text>"#,
                    text.position.x, -text.position.y, font_size, stroke_color,
                    -2.0 * text.position.y,
                    text.content
                ))
            }
            Geometry::Leader(leader) => {
                if leader.vertices.is_empty() {
                    return None;
                }
                
                let mut path = String::new();
                for (i, vertex) in leader.vertices.iter().enumerate() {
                    if i == 0 {
                        path.push_str(&format!("M {:.4} {:.4}", vertex.x, vertex.y));
                    } else {
                        path.push_str(&format!(" L {:.4} {:.4}", vertex.x, vertex.y));
                    }
                }
                
                // 添加箭头
                if leader.vertices.len() >= 2 {
                    let p0 = &leader.vertices[0];
                    let p1 = &leader.vertices[1];
                    let dir = Vector2::new(p1.x - p0.x, p1.y - p0.y).normalize();
                    let arrow_len = 3.0;
                    let arrow_width = 1.0;
                    
                    let perp = Vector2::new(-dir.y, dir.x);
                    let arrow1 = Point2::new(
                        p0.x + dir.x * arrow_len + perp.x * arrow_width,
                        p0.y + dir.y * arrow_len + perp.y * arrow_width,
                    );
                    let arrow2 = Point2::new(
                        p0.x + dir.x * arrow_len - perp.x * arrow_width,
                        p0.y + dir.y * arrow_len - perp.y * arrow_width,
                    );
                    
                    path.push_str(&format!(
                        " M {:.4} {:.4} L {:.4} {:.4} L {:.4} {:.4}",
                        arrow1.x, arrow1.y, p0.x, p0.y, arrow2.x, arrow2.y
                    ));
                }
                
                Some(format!(r#"<path d="{}" {}/>"#, path, style))
            }
            Geometry::Dimension(dim) => {
                // 标注渲染较复杂，这里简化处理
                let p1 = dim.definition_point1;
                let p2 = dim.definition_point2;
                let text_pos = dim.get_text_position();
                
                let mut elements = vec![];
                
                // 绘制标注线
                elements.push(format!(
                    r#"<line x1="{:.4}" y1="{:.4}" x2="{:.4}" y2="{:.4}" {}/>"#,
                    p1.x, p1.y, p2.x, p2.y, style
                ));
                
                // 绘制文本
                elements.push(format!(
                    r#"<text x="{:.4}" y="{:.4}" font-size="{:.2}" fill="{}" text-anchor="middle" transform="scale(1,-1) translate(0,{:.4})">{}</text>"#,
                    text_pos.x, -text_pos.y, dim.text_height, stroke_color,
                    -2.0 * text_pos.y,
                    dim.display_text()
                ));
                
                Some(elements.join("\n    "))
            }
            Geometry::Hatch(_) => {
                // 填充渲染需要更复杂的处理
                None
            }
        }
    }

    /// 导出到文件
    pub fn export_to_file(&self, entities: &[Entity], path: &std::path::Path) -> Result<(), FileError> {
        let svg = self.export(entities)?;
        std::fs::write(path, svg)?;
        Ok(())
    }
}

/// PDF 导出器（使用 SVG 转换）
pub struct PdfExporter {
    page_setup: PageSetup,
}

impl PdfExporter {
    pub fn new(page_setup: PageSetup) -> Self {
        Self { page_setup }
    }

    /// 导出为 PDF
    /// 
    /// 注意：实际的 PDF 生成需要额外的库（如 printpdf）
    /// 这里提供一个简化的接口
    pub fn export(&self, entities: &[Entity]) -> Result<Vec<u8>, FileError> {
        // 首先生成 SVG
        let svg_exporter = SvgExporter::new(self.page_setup.clone());
        let svg_content = svg_exporter.export(entities)?;
        
        // 注意：完整的 PDF 导出需要使用 printpdf 或类似的库
        // 这里返回 SVG 内容的字节，作为占位符
        // 在实际实现中，应该使用 svg2pdf 或直接使用 printpdf 渲染
        
        // 简化实现：返回包装的 SVG
        let pdf_placeholder = format!(
            "PDF Export Placeholder\n\
             Paper: {:?}\n\
             Orientation: {:?}\n\
             Entities: {}\n\n\
             SVG Content:\n{}",
            self.page_setup.paper_size,
            self.page_setup.orientation,
            entities.len(),
            svg_content
        );
        
        Ok(pdf_placeholder.into_bytes())
    }

    /// 导出到文件
    pub fn export_to_file(&self, entities: &[Entity], path: &std::path::Path) -> Result<(), FileError> {
        let pdf_data = self.export(entities)?;
        std::fs::write(path, pdf_data)?;
        Ok(())
    }
}

/// 导出格式
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExportFormat {
    Svg,
    Pdf,
    Png,
    Jpg,
}

/// 通用导出函数
pub fn export_entities(
    entities: &[Entity],
    format: ExportFormat,
    page_setup: PageSetup,
    path: &std::path::Path,
) -> Result<(), FileError> {
    match format {
        ExportFormat::Svg => {
            let exporter = SvgExporter::new(page_setup);
            exporter.export_to_file(entities, path)
        }
        ExportFormat::Pdf => {
            let exporter = PdfExporter::new(page_setup);
            exporter.export_to_file(entities, path)
        }
        ExportFormat::Png | ExportFormat::Jpg => {
            // PNG/JPG 导出需要图像渲染库
            Err(FileError::InvalidFormat(format!(
                "{:?} export requires image rendering (not yet implemented)",
                format
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paper_size_dimensions() {
        let a4 = PaperSize::A4.dimensions_mm();
        assert_eq!(a4, (210.0, 297.0));
        
        let a3 = PaperSize::A3.dimensions_mm();
        assert_eq!(a3, (297.0, 420.0));
    }

    #[test]
    fn test_page_setup_printable_size() {
        let setup = PageSetup {
            paper_size: PaperSize::A4,
            orientation: Orientation::Portrait,
            margins: (10.0, 10.0, 10.0, 10.0),
            ..Default::default()
        };
        
        let (w, h) = setup.printable_size();
        assert_eq!(w, 190.0);
        assert_eq!(h, 277.0);
    }
}
