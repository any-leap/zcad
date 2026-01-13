//! 高质量缓存渲染器
//!
//! 使用 tiny-skia 进行几何渲染，ab_glyph 进行文字渲染。
//! 渲染结果缓存到位图，只在相机/实体变化时重新渲染。

use ab_glyph::{Font, FontRef, PxScale, ScaleFont};
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform, LineCap, LineJoin, FillRule};
use zcad_core::entity::Entity;
use zcad_core::geometry::Geometry;
use zcad_core::layer::LayerManager;
use zcad_core::math::Point2;

/// 缓存渲染器
pub struct VelloRenderer {
    /// 缓存的位图
    pixmap: Option<Pixmap>,
    /// egui 纹理
    texture: Option<TextureHandle>,
    /// 缓存对应的相机状态
    cached_center: Point2,
    cached_zoom: f64,
    cached_size: (u32, u32),
    /// 缓存的实体版本
    cached_entity_version: u64,
    /// 字体数据
    font_data: Option<Vec<u8>>,
}

impl VelloRenderer {
    pub fn new() -> Self {
        // 尝试加载系统字体
        let font_data = Self::load_system_font();
        
        Self {
            pixmap: None,
            texture: None,
            cached_center: Point2::origin(),
            cached_zoom: 1.0,
            cached_size: (0, 0),
            cached_entity_version: 0,
            font_data,
        }
    }

    /// 加载系统字体
    fn load_system_font() -> Option<Vec<u8>> {
        // Windows 常见字体路径
        let font_paths = [
            "C:\\Windows\\Fonts\\msyh.ttc",      // 微软雅黑
            "C:\\Windows\\Fonts\\simhei.ttf",    // 黑体
            "C:\\Windows\\Fonts\\simsun.ttc",    // 宋体
            "C:\\Windows\\Fonts\\arial.ttf",     // Arial
        ];
        
        for path in &font_paths {
            if let Ok(data) = std::fs::read(path) {
                tracing::info!("Loaded font: {}", path);
                return Some(data);
            }
        }
        
        tracing::warn!("No system font found, text rendering will be limited");
        None
    }

    /// 检查是否需要重新渲染
    fn needs_update(&self, center: Point2, zoom: f64, width: u32, height: u32, entity_version: u64) -> bool {
        if self.pixmap.is_none() {
            return true;
        }
        if self.cached_size != (width, height) {
            return true;
        }
        if self.cached_entity_version != entity_version {
            return true;
        }
        // 相机变化 - 使用相对阈值，避免大坐标时频繁触发
        // 阈值 = 1 像素对应的世界坐标距离
        let pixel_threshold = 1.0 / zoom.max(0.0001);
        if (self.cached_center.x - center.x).abs() > pixel_threshold
            || (self.cached_center.y - center.y).abs() > pixel_threshold
        {
            return true;
        }
        // 缩放变化 - 使用相对阈值 (0.1% 变化)
        let zoom_threshold = self.cached_zoom * 0.001;
        if (self.cached_zoom - zoom).abs() > zoom_threshold {
            return true;
        }
        false
    }

    /// 标记为需要重新渲染
    pub fn invalidate(&mut self) {
        self.pixmap = None;
    }

    /// 渲染到位图缓存
    pub fn render(
        &mut self,
        entities: &[&Entity],
        layers: &LayerManager,
        center: Point2,
        zoom: f64,
        width: u32,
        height: u32,
        entity_version: u64,
        show_grid: bool,
    ) {
        // 检查是否需要更新
        if !self.needs_update(center, zoom, width, height, entity_version) {
            return;
        }

        // 创建或调整位图大小
        if self.pixmap.is_none() || self.cached_size != (width, height) {
            self.pixmap = Pixmap::new(width, height);
        }

        let pixmap = match self.pixmap.as_mut() {
            Some(p) => p,
            None => return,
        };

        // 清空背景
        pixmap.fill(Color::from_rgba8(30, 30, 35, 255));

        let half_w = width as f32 / 2.0;
        let half_h = height as f32 / 2.0;
        let zoom_f = zoom as f32;

        // 坐标转换函数 - 使用 f64 计算差值避免精度丢失
        let center_x = center.x;
        let center_y = center.y;
        let to_screen = move |p: Point2| -> (f32, f32) {
            // 先在 f64 下计算相对位置，再转为 f32
            let rel_x = (p.x - center_x) * zoom;
            let rel_y = (p.y - center_y) * zoom;
            let x = half_w + rel_x as f32;
            let y = half_h - rel_y as f32;
            (x, y)
        };

        // 绘制网格
        if show_grid {
            Self::draw_grid(pixmap, center, zoom, width, height);
        }

        // 配置画笔 - 使用抗锯齿和圆角线帽
        let mut stroke = Stroke::default();
        stroke.width = 1.5;
        stroke.line_cap = LineCap::Round;
        stroke.line_join = LineJoin::Round;

        // 准备字体
        let font = self.font_data.as_ref().and_then(|data| FontRef::try_from_slice(data).ok());

        // 绘制所有实体
        for entity in entities {
            let color = if entity.visual_properties.color.is_by_layer() {
                layers
                    .get_layer_by_id(entity.layer_id)
                    .map(|l| l.color)
                    .unwrap_or(zcad_core::properties::Color::WHITE)
            } else {
                entity.visual_properties.color
            };

            let mut paint = Paint::default();
            paint.set_color(Color::from_rgba8(color.r, color.g, color.b, 255));
            paint.anti_alias = true;

            if let Some(geometry) = entity.geometry() {
                Self::draw_geometry(pixmap, geometry, &paint, &stroke, &to_screen, zoom_f, font.as_ref());
            }
        }

        // 更新缓存状态
        self.cached_center = center;
        self.cached_zoom = zoom;
        self.cached_size = (width, height);
        self.cached_entity_version = entity_version;
    }

    /// 绘制网格（限制最大线条数量）
    fn draw_grid(pixmap: &mut Pixmap, center: Point2, zoom: f64, width: u32, height: u32) {
        const MAX_GRID_LINES: usize = 100;
        
        let half_w = width as f64 / 2.0;
        let half_h = height as f64 / 2.0;

        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(50, 50, 60, 255));
        paint.anti_alias = false;

        let stroke = Stroke { width: 1.0, ..Default::default() };

        let left = center.x - half_w / zoom;
        let right = center.x + half_w / zoom;
        let bottom = center.y - half_h / zoom;
        let top = center.y + half_h / zoom;
        
        let view_width = right - left;
        let view_height = top - bottom;

        // 动态计算网格间距，确保网格线数量不超过限制
        let min_spacing_x = view_width / MAX_GRID_LINES as f64;
        let min_spacing_y = view_height / MAX_GRID_LINES as f64;
        let min_spacing = min_spacing_x.max(min_spacing_y);
        
        // 选择 1, 2, 5, 10, 20, 50, 100... 系列的间距
        let order = min_spacing.log10().floor();
        let base = 10f64.powf(order);
        let spacing = if min_spacing <= base {
            base
        } else if min_spacing <= base * 2.0 {
            base * 2.0
        } else if min_spacing <= base * 5.0 {
            base * 5.0
        } else {
            base * 10.0
        };

        // 垂直线
        let start_x = (left / spacing).floor() * spacing;
        let mut x = start_x;
        let mut count = 0;
        while x <= right && count < MAX_GRID_LINES {
            let sx = (half_w + (x - center.x) * zoom) as f32;
            let mut pb = PathBuilder::new();
            pb.move_to(sx, 0.0);
            pb.line_to(sx, height as f32);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
            x += spacing;
            count += 1;
        }

        // 水平线
        let start_y = (bottom / spacing).floor() * spacing;
        let mut y = start_y;
        count = 0;
        while y <= top && count < MAX_GRID_LINES {
            let sy = (half_h - (y - center.y) * zoom) as f32;
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, sy);
            pb.line_to(width as f32, sy);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
            y += spacing;
            count += 1;
        }

        // 坐标轴
        paint.set_color(Color::from_rgba8(80, 80, 100, 255));
        if bottom <= 0.0 && top >= 0.0 {
            let sy = (half_h - (0.0 - center.y) * zoom) as f32;
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, sy);
            pb.line_to(width as f32, sy);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
        if left <= 0.0 && right >= 0.0 {
            let sx = (half_w + (0.0 - center.x) * zoom) as f32;
            let mut pb = PathBuilder::new();
            pb.move_to(sx, 0.0);
            pb.line_to(sx, height as f32);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
        }
    }

    /// 绘制几何体
    fn draw_geometry<F>(
        pixmap: &mut Pixmap,
        geometry: &Geometry,
        paint: &Paint,
        stroke: &Stroke,
        to_screen: &F,
        zoom: f32,
        font: Option<&FontRef>,
    ) where F: Fn(Point2) -> (f32, f32) {
        match geometry {
            Geometry::Point(p) => {
                let (sx, sy) = to_screen(p.position);
                if let Some(path) = PathBuilder::from_circle(sx, sy, 2.0) {
                    pixmap.fill_path(&path, paint, FillRule::Winding, Transform::identity(), None);
                }
            }
            Geometry::Line(line) => {
                let (x1, y1) = to_screen(line.start);
                let (x2, y2) = to_screen(line.end);
                let mut pb = PathBuilder::new();
                pb.move_to(x1, y1);
                pb.line_to(x2, y2);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
                }
            }
            Geometry::Circle(circle) => {
                let (cx, cy) = to_screen(circle.center);
                let r = (circle.radius * zoom as f64) as f32;
                if r < 0.5 {
                    return;
                }
                if let Some(path) = PathBuilder::from_circle(cx, cy, r) {
                    pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
                }
            }
            Geometry::Arc(arc) => {
                let screen_radius = arc.radius * zoom as f64;
                if screen_radius < 0.5 {
                    return;
                }
                let sweep = arc.sweep_angle();
                let segments = ((screen_radius * sweep.abs()) / 5.0).ceil() as usize;
                let segments = segments.clamp(4, 64);
                let angle_step = sweep / segments as f64;

                let mut pb = PathBuilder::new();
                let start_p = arc.start_point();
                let (sx, sy) = to_screen(start_p);
                pb.move_to(sx, sy);

                for i in 1..=segments {
                    let angle = arc.start_angle + i as f64 * angle_step;
                    let p = Point2::new(
                        arc.center.x + arc.radius * angle.cos(),
                        arc.center.y + arc.radius * angle.sin(),
                    );
                    let (px, py) = to_screen(p);
                    pb.line_to(px, py);
                }
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
                }
            }
            Geometry::Polyline(polyline) => {
                if polyline.vertices.len() < 2 {
                    return;
                }
                let mut pb = PathBuilder::new();
                let (x, y) = to_screen(polyline.vertices[0].point);
                pb.move_to(x, y);
                for v in polyline.vertices.iter().skip(1) {
                    let (x, y) = to_screen(v.point);
                    pb.line_to(x, y);
                }
                if polyline.closed {
                    pb.close();
                }
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
                }
            }
            Geometry::Ellipse(ellipse) => {
                let points = ellipse.sample_points(48);
                if points.len() < 2 {
                    return;
                }
                let mut pb = PathBuilder::new();
                let (x, y) = to_screen(points[0]);
                pb.move_to(x, y);
                for p in points.iter().skip(1) {
                    let (x, y) = to_screen(*p);
                    pb.line_to(x, y);
                }
                pb.close();
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
                }
            }
            Geometry::Text(text) => {
                // 使用 ab_glyph 绘制文字
                Self::draw_text(pixmap, text, to_screen, zoom, paint, font);
            }
            Geometry::Spline(spline) => {
                if spline.control_points.len() < 2 {
                    return;
                }
                let samples = 50;
                let mut pb = PathBuilder::new();
                
                for i in 0..=samples {
                    let t = i as f64 / samples as f64;
                    let nalgebra_p = spline.point_at_param(t);
                    let p = Point2::new(nalgebra_p.x, nalgebra_p.y);
                    let (x, y) = to_screen(p);
                    if i == 0 {
                        pb.move_to(x, y);
                    } else {
                        pb.line_to(x, y);
                    }
                }
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
                }
            }
            Geometry::Dimension(dim) => {
                Self::draw_dimension(pixmap, dim, paint, stroke, to_screen, zoom);
            }
            _ => {}
        }
    }

    /// 绘制文字
    fn draw_text<F>(
        pixmap: &mut Pixmap,
        text: &zcad_core::geometry::Text,
        to_screen: &F,
        zoom: f32,
        paint: &Paint,
        font: Option<&FontRef>,
    ) where F: Fn(Point2) -> (f32, f32) {
        let (x, y) = to_screen(text.position);
        let screen_height = (text.height * zoom as f64) as f32;
        
        // 如果文字太小，用下划线表示
        if screen_height < 2.0 {
            return;
        }
        
        let font = match font {
            Some(f) => f,
            None => {
                // 没有字体，用下划线表示
                let w = screen_height * text.content.chars().count() as f32 * 0.6;
                let mut pb = PathBuilder::new();
                pb.move_to(x, y);
                pb.line_to(x + w, y);
                if let Some(path) = pb.finish() {
                    let stroke = Stroke { width: 1.5, ..Default::default() };
                    pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
                }
                return;
            }
        };
        
        // 使用 ab_glyph 渲染文字
        let scale = PxScale::from(screen_height);
        let scaled_font = font.as_scaled(scale);
        
        // 获取颜色
        let (r, g, b, a): (u8, u8, u8, u8) = if let tiny_skia::Shader::SolidColor(c) = &paint.shader {
            ((c.red() * 255.0) as u8, (c.green() * 255.0) as u8, (c.blue() * 255.0) as u8, (c.alpha() * 255.0) as u8)
        } else {
            (255, 255, 255, 255)
        };
        
        // 获取像素缓冲区
        let width = pixmap.width();
        let height = pixmap.height();
        let pixels = pixmap.pixels_mut();
        
        // 计算每个字符的位置并渲染
        let mut cursor_x = x;
        let baseline_y = y;
        
        for c in text.content.chars() {
            let glyph_id = font.glyph_id(c);
            let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(cursor_x, baseline_y));
            
            if let Some(outlined) = font.outline_glyph(glyph) {
                let bounds = outlined.px_bounds();
                
                // 绘制每个像素
                outlined.draw(|px, py, coverage| {
                    if coverage > 0.1 {
                        let pixel_x = (bounds.min.x + px as f32) as i32;
                        let pixel_y = (bounds.min.y + py as f32) as i32;
                        
                        if pixel_x >= 0 && pixel_x < width as i32 
                            && pixel_y >= 0 && pixel_y < height as i32 {
                            let alpha = (coverage * a as f32) as u8;
                            if alpha > 0 {
                                let idx = (pixel_y as u32 * width + pixel_x as u32) as usize;
                                if idx < pixels.len() {
                                    let dst = pixels[idx];
                                    // 简单的 alpha 混合
                                    let src_a = alpha as f32 / 255.0;
                                    let dst_a = dst.alpha() as f32 / 255.0;
                                    let out_a = src_a + dst_a * (1.0 - src_a);
                                    
                                    if out_a > 0.0 {
                                        let blend = |src: u8, dst: u8| -> u8 {
                                            let src_f = src as f32 / 255.0;
                                            let dst_f = dst as f32 / 255.0;
                                            let out = (src_f * src_a + dst_f * dst_a * (1.0 - src_a)) / out_a;
                                            (out * 255.0) as u8
                                        };
                                        
                                        if let Some(new_px) = tiny_skia::PremultipliedColorU8::from_rgba(
                                            blend(r, dst.red()),
                                            blend(g, dst.green()),
                                            blend(b, dst.blue()),
                                            (out_a * 255.0) as u8,
                                        ) {
                                            pixels[idx] = new_px;
                                        }
                                    }
                                }
                            }
                        }
                    }
                });
            }
            
            // 移动光标
            cursor_x += scaled_font.h_advance(glyph_id);
        }
    }

    /// 绘制标注
    fn draw_dimension<F>(
        pixmap: &mut Pixmap,
        dim: &zcad_core::geometry::Dimension,
        paint: &Paint,
        stroke: &Stroke,
        to_screen: &F,
        zoom: f32,
    ) where F: Fn(Point2) -> (f32, f32) {
        use zcad_core::geometry::DimensionType;
        
        match dim.dim_type {
            DimensionType::Aligned | DimensionType::Linear => {
                // 计算标注线的方向向量
                let dir = (dim.definition_point2 - dim.definition_point1).normalize();
                let perp = zcad_core::math::Vector2::new(-dir.y, dir.x);
                
                // 计算标注线在法向量方向上的投影距离
                let v_loc = dim.line_location - dim.definition_point1;
                let dist = v_loc.dot(&perp);
                let sign = if dist.abs() < 1e-10 { 1.0 } else { dist.signum() };
                
                // 计算标注线的两个端点
                let dim_p1 = dim.definition_point1 + perp * dist;
                let dim_p2 = dim.definition_point2 + perp * dist;
                
                // 界线参数
                let dim_scale = 1.0 / zoom as f64;
                let ext_line_offset = 0.625 * dim_scale;
                let ext_line_extension = 1.25 * dim_scale;
                
                // 界线起点和终点
                let ext_start_offset = perp * (ext_line_offset * sign);
                let ext_end_offset = perp * (dist + ext_line_extension * sign);
                
                let ext_start_p1 = dim.definition_point1 + ext_start_offset;
                let ext_end_p1 = dim.definition_point1 + ext_end_offset;
                let ext_start_p2 = dim.definition_point2 + ext_start_offset;
                let ext_end_p2 = dim.definition_point2 + ext_end_offset;
                
                // 绘制界线
                Self::draw_line_segment(pixmap, ext_start_p1, ext_end_p1, paint, stroke, to_screen);
                Self::draw_line_segment(pixmap, ext_start_p2, ext_end_p2, paint, stroke, to_screen);
                
                // 绘制尺寸线
                Self::draw_line_segment(pixmap, dim_p1, dim_p2, paint, stroke, to_screen);
                
                // 绘制箭头
                Self::draw_arrow(pixmap, dim_p1, dim_p2, paint, to_screen, zoom);
                Self::draw_arrow(pixmap, dim_p2, dim_p1, paint, to_screen, zoom);
            }
            DimensionType::Radius => {
                let p1 = dim.definition_point1;  // 圆心
                let p2 = dim.definition_point2;  // 圆上点
                
                // 绘制半径线
                Self::draw_line_segment(pixmap, p1, p2, paint, stroke, to_screen);
                
                // 绘制箭头
                Self::draw_arrow(pixmap, p1, p2, paint, to_screen, zoom);
            }
            DimensionType::Diameter => {
                let p1 = dim.definition_point1;
                let p2 = dim.definition_point2;
                let center_point = Point2::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
                
                // 绘制直径线
                Self::draw_line_segment(pixmap, p1, p2, paint, stroke, to_screen);
                
                // 绘制箭头
                Self::draw_arrow(pixmap, center_point, p1, paint, to_screen, zoom);
                Self::draw_arrow(pixmap, center_point, p2, paint, to_screen, zoom);
            }
            _ => {
                // 简化绘制
                Self::draw_line_segment(pixmap, dim.definition_point1, dim.line_location, paint, stroke, to_screen);
                Self::draw_line_segment(pixmap, dim.definition_point2, dim.line_location, paint, stroke, to_screen);
            }
        }
    }

    /// 绘制线段
    fn draw_line_segment<F>(
        pixmap: &mut Pixmap,
        p1: Point2,
        p2: Point2,
        paint: &Paint,
        stroke: &Stroke,
        to_screen: &F,
    ) where F: Fn(Point2) -> (f32, f32) {
        let (x1, y1) = to_screen(p1);
        let (x2, y2) = to_screen(p2);
        let mut pb = PathBuilder::new();
        pb.move_to(x1, y1);
        pb.line_to(x2, y2);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, paint, stroke, Transform::identity(), None);
        }
    }

    /// 绘制箭头
    fn draw_arrow<F>(
        pixmap: &mut Pixmap,
        from: Point2,
        to: Point2,
        paint: &Paint,
        to_screen: &F,
        _zoom: f32,
    ) where F: Fn(Point2) -> (f32, f32) {
        let (x1, y1) = to_screen(from);
        let (x2, y2) = to_screen(to);
        
        let dx = x2 - x1;
        let dy = y2 - y1;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }
        let dir_x = dx / len;
        let dir_y = dy / len;
        
        let arrow_len = 10.0;
        let arrow_end1_x = x2 - dir_x * arrow_len + (-dir_y) * arrow_len * 0.3;
        let arrow_end1_y = y2 - dir_y * arrow_len + dir_x * arrow_len * 0.3;
        let arrow_end2_x = x2 - dir_x * arrow_len - (-dir_y) * arrow_len * 0.3;
        let arrow_end2_y = y2 - dir_y * arrow_len - dir_x * arrow_len * 0.3;
        
        let stroke = Stroke { width: 1.0, ..Default::default() };
        
        let mut pb = PathBuilder::new();
        pb.move_to(x2, y2);
        pb.line_to(arrow_end1_x, arrow_end1_y);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
        }
        
        let mut pb = PathBuilder::new();
        pb.move_to(x2, y2);
        pb.line_to(arrow_end2_x, arrow_end2_y);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, paint, &stroke, Transform::identity(), None);
        }
    }

    /// 获取或创建 egui 纹理
    pub fn get_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureId> {
        let pixmap = self.pixmap.as_ref()?;
        
        let size = [pixmap.width() as usize, pixmap.height() as usize];
        let image = ColorImage::from_rgba_unmultiplied(size, pixmap.data());

        match &mut self.texture {
            Some(handle) => {
                handle.set(image, TextureOptions::LINEAR);
            }
            None => {
                self.texture = Some(ctx.load_texture(
                    "cached_canvas",
                    image,
                    TextureOptions::LINEAR,
                ));
            }
        }

        self.texture.as_ref().map(|h| h.id())
    }
}

impl Default for VelloRenderer {
    fn default() -> Self {
        Self::new()
    }
}
