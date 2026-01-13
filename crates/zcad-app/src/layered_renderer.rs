//! 三层缓冲渲染器
//!
//! 借鉴 LibreCAD 的渲染架构：
//! - Layer 1: 背景（网格）- 仅在缩放/平移时重绘
//! - Layer 2: 实体 - 仅在实体变化时重绘
//! - Layer 3: 叠加层（光标、捕捉、预览）- 每帧重绘
//!
//! 关键优化：鼠标移动时只需要重绘 Layer 3！

use eframe::egui;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};
use zcad_core::entity::Entity;
use zcad_core::geometry::Geometry;
use zcad_core::layer::LayerManager;
use zcad_core::math::Point2;

/// 重绘标志（借鉴 LibreCAD 的 RS2::RedrawMethod）
#[derive(Clone, Copy, PartialEq)]
pub struct RedrawFlags(u8);

impl RedrawFlags {
    pub const NONE: Self = Self(0);
    pub const GRID: Self = Self(1);       // Layer 1
    pub const ENTITIES: Self = Self(2);   // Layer 2
    pub const OVERLAY: Self = Self(4);    // Layer 3
    pub const ALL: Self = Self(0xFF);

    pub fn contains(&self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }

    pub fn add(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    pub fn clear(&mut self) {
        self.0 = 0;
    }
}

/// 三层缓冲渲染器
pub struct LayeredRenderer {
    /// Layer 1: 背景 + 网格
    layer_background: Option<Pixmap>,
    /// Layer 2: 所有实体
    layer_entities: Option<Pixmap>,
    /// Layer 3: 叠加层（最终合成结果）
    layer_composite: Option<Pixmap>,
    
    /// egui 纹理句柄
    texture_handle: Option<egui::TextureHandle>,
    
    /// 缓存的相机状态
    cached_center: Point2,
    cached_zoom: f64,
    cached_size: (u32, u32),
    
    /// 缓存的实体版本
    cached_entity_version: u64,
    
    /// 需要重绘的层
    redraw_flags: RedrawFlags,
    
    /// 最小渲染阈值（像素）- 借鉴 LibreCAD
    min_circle_radius: f64,
    min_arc_radius: f64,
    min_line_len: f64,
    min_text_height: f64,
}

impl LayeredRenderer {
    pub fn new() -> Self {
        Self {
            layer_background: None,
            layer_entities: None,
            layer_composite: None,
            texture_handle: None,
            cached_center: Point2::origin(),
            cached_zoom: 1.0,
            cached_size: (0, 0),
            cached_entity_version: 0,
            redraw_flags: RedrawFlags::ALL,
            // LibreCAD 默认值
            min_circle_radius: 2.0,
            min_arc_radius: 0.8,
            min_line_len: 2.0,
            min_text_height: 4.0,
        }
    }

    /// 检测相机变化，更新重绘标志
    pub fn update_camera(&mut self, center: Point2, zoom: f64, width: u32, height: u32) {
        // 尺寸变化 → 全部重绘
        if self.cached_size != (width, height) {
            self.redraw_flags = RedrawFlags::ALL;
            self.cached_size = (width, height);
        }
        // 缩放变化 → 重绘网格和实体
        else if (self.cached_zoom - zoom).abs() > 0.0001 {
            self.redraw_flags.add(RedrawFlags::GRID);
            self.redraw_flags.add(RedrawFlags::ENTITIES);
        }
        // 平移变化 → 重绘网格和实体
        else if (self.cached_center.x - center.x).abs() > 0.001
            || (self.cached_center.y - center.y).abs() > 0.001
        {
            self.redraw_flags.add(RedrawFlags::GRID);
            self.redraw_flags.add(RedrawFlags::ENTITIES);
        }
        
        self.cached_center = center;
        self.cached_zoom = zoom;
    }

    /// 标记实体变化
    pub fn mark_entities_changed(&mut self, version: u64) {
        if self.cached_entity_version != version {
            self.redraw_flags.add(RedrawFlags::ENTITIES);
            self.cached_entity_version = version;
        }
    }

    /// 渲染（按需重绘）
    pub fn render(
        &mut self,
        entities: &[&Entity],
        layers: &LayerManager,
        center: Point2,
        zoom: f64,
        width: u32,
        height: u32,
        show_grid: bool,
        // 叠加层内容
        cursor_pos: Option<Point2>,
        snap_point: Option<Point2>,
        selected_entities: &[&Entity],
    ) {
        // 确保所有层都有正确的大小
        self.ensure_layers(width, height);
        
        let half_w = width as f32 / 2.0;
        let half_h = height as f32 / 2.0;
        let zoom_f = zoom as f32;

        // === Layer 1: 背景 ===
        if self.redraw_flags.contains(RedrawFlags::GRID) {
            if let Some(layer) = self.layer_background.as_mut() {
                layer.fill(Color::from_rgba8(30, 30, 35, 255));
                if show_grid {
                    Self::draw_grid(layer, center, zoom, width, height);
                }
            }
        }

        // === Layer 2: 实体 ===
        if self.redraw_flags.contains(RedrawFlags::ENTITIES) {
            if let Some(layer) = self.layer_entities.as_mut() {
                // 透明背景
                layer.fill(Color::TRANSPARENT);
                
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

                    if let Some(geometry) = entity.geometry() {
                        Self::draw_geometry_static(
                            layer, geometry, color, center, zoom_f, half_w, half_h,
                            self.min_circle_radius, self.min_arc_radius, self.min_line_len,
                        );
                    }
                }
            }
        }

        // === Layer 3: 合成 + 叠加 ===
        // 叠加层每帧都需要重绘
        if let Some(composite) = self.layer_composite.as_mut() {
            // 复制背景层
            if let Some(bg) = self.layer_background.as_ref() {
                composite.data_mut().copy_from_slice(bg.data());
            }
            
            // 叠加实体层
            if let Some(entities_layer) = self.layer_entities.as_ref() {
                Self::blend_layer(composite, entities_layer);
            }
            
            // 绘制选中实体高亮
            for entity in selected_entities {
                if let Some(geometry) = entity.geometry() {
                    Self::draw_geometry_static(
                        composite,
                        geometry,
                        zcad_core::properties::Color::from_hex(0x00FF00),
                        center,
                        zoom_f,
                        half_w,
                        half_h,
                        self.min_circle_radius,
                        self.min_arc_radius,
                        self.min_line_len,
                    );
                }
            }
            
            // 绘制捕捉点
            if let Some(snap) = snap_point {
                Self::draw_snap_marker(composite, snap, center, zoom_f, half_w, half_h);
            }
            
            // 绘制十字光标
            if let Some(cursor) = cursor_pos {
                Self::draw_crosshair(composite, cursor, center, zoom_f, half_w, half_h, width, height);
            }
        }

        // 清除重绘标志
        self.redraw_flags.clear();
    }

    /// 确保所有层都有正确的大小
    fn ensure_layers(&mut self, width: u32, height: u32) {
        let need_resize = self.layer_background.is_none()
            || self.cached_size != (width, height);

        if need_resize {
            self.layer_background = Pixmap::new(width, height);
            self.layer_entities = Pixmap::new(width, height);
            self.layer_composite = Pixmap::new(width, height);
        }
    }

    /// 绘制网格
    fn draw_grid(pixmap: &mut Pixmap, center: Point2, zoom: f64, width: u32, height: u32) {
        let half_w = width as f64 / 2.0;
        let half_h = height as f64 / 2.0;

        // 计算网格间距
        let mut spacing = 50.0;
        while spacing * zoom < 20.0 {
            spacing *= 5.0;
        }
        while spacing * zoom > 200.0 {
            spacing /= 5.0;
        }

        let left = center.x - half_w / zoom;
        let right = center.x + half_w / zoom;
        let bottom = center.y - half_h / zoom;
        let top = center.y + half_h / zoom;

        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(60, 60, 70, 255));
        paint.anti_alias = false;

        let stroke = Stroke { width: 1.0, ..Default::default() };

        // 垂直线
        let start_x = (left / spacing).floor() * spacing;
        let mut x = start_x;
        while x <= right {
            let sx = (half_w + (x - center.x) * zoom) as f32;
            let mut pb = PathBuilder::new();
            pb.move_to(sx, 0.0);
            pb.line_to(sx, height as f32);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
            x += spacing;
        }

        // 水平线
        let start_y = (bottom / spacing).floor() * spacing;
        let mut y = start_y;
        while y <= top {
            let sy = (half_h - (y - center.y) * zoom) as f32;
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, sy);
            pb.line_to(width as f32, sy);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
            }
            y += spacing;
        }

        // 坐标轴
        let mut axis_paint = Paint::default();
        axis_paint.set_color(Color::from_rgba8(100, 100, 120, 255));
        
        // X 轴
        if bottom <= 0.0 && top >= 0.0 {
            let sy = (half_h - (0.0 - center.y) * zoom) as f32;
            let mut pb = PathBuilder::new();
            pb.move_to(0.0, sy);
            pb.line_to(width as f32, sy);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &axis_paint, &stroke, Transform::identity(), None);
            }
        }
        
        // Y 轴
        if left <= 0.0 && right >= 0.0 {
            let sx = (half_w + (0.0 - center.x) * zoom) as f32;
            let mut pb = PathBuilder::new();
            pb.move_to(sx, 0.0);
            pb.line_to(sx, height as f32);
            if let Some(path) = pb.finish() {
                pixmap.stroke_path(&path, &axis_paint, &stroke, Transform::identity(), None);
            }
        }
    }

    /// 绘制几何体（静态方法，避免借用冲突）
    fn draw_geometry_static(
        pixmap: &mut Pixmap,
        geometry: &Geometry,
        color: zcad_core::properties::Color,
        center: Point2,
        zoom: f32,
        half_w: f32,
        half_h: f32,
        min_circle_radius: f64,
        min_arc_radius: f64,
        min_line_len: f64,
    ) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color.r, color.g, color.b, 255));
        paint.anti_alias = true;

        // 使用 1.5 像素线宽 + 圆角线帽，改善抗锯齿效果
        let stroke = Stroke {
            width: 1.5,
            line_cap: tiny_skia::LineCap::Round,
            line_join: tiny_skia::LineJoin::Round,
            ..Default::default()
        };

        let to_screen = |p: Point2| -> (f32, f32) {
            let x = half_w + ((p.x - center.x) as f32 * zoom);
            let y = half_h - ((p.y - center.y) as f32 * zoom);
            (x, y)
        };

        match geometry {
            Geometry::Point(p) => {
                let (sx, sy) = to_screen(p.position);
                let mut pb = PathBuilder::new();
                pb.push_circle(sx, sy, 2.0);
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            }
            Geometry::Line(line) => {
                // 最小长度检查
                let screen_len = line.length() as f32 * zoom;
                if screen_len < min_line_len as f32 {
                    return;
                }
                
                let (x1, y1) = to_screen(line.start);
                let (x2, y2) = to_screen(line.end);
                let mut pb = PathBuilder::new();
                pb.move_to(x1, y1);
                pb.line_to(x2, y2);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            Geometry::Circle(circle) => {
                let screen_radius = circle.radius as f32 * zoom;
                
                // 最小半径检查
                if screen_radius < min_circle_radius as f32 {
                    // 太小，画一个点
                    let (cx, cy) = to_screen(circle.center);
                    let mut pb = PathBuilder::new();
                    pb.push_circle(cx, cy, 1.0);
                    if let Some(path) = pb.finish() {
                        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                    }
                    return;
                }
                
                let (cx, cy) = to_screen(circle.center);
                let mut pb = PathBuilder::new();
                pb.push_circle(cx, cy, screen_radius);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            Geometry::Arc(arc) => {
                let screen_radius = arc.radius as f32 * zoom;
                
                // 最小半径检查
                if screen_radius < min_arc_radius as f32 {
                    let (x1, y1) = to_screen(arc.start_point());
                    let (x2, y2) = to_screen(arc.end_point());
                    let mut pb = PathBuilder::new();
                    pb.move_to(x1, y1);
                    pb.line_to(x2, y2);
                    if let Some(path) = pb.finish() {
                        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                    }
                    return;
                }
                
                let sweep = arc.sweep_angle();
                let segments = ((screen_radius * sweep.abs() as f32) / 8.0).ceil() as usize;
                let segments = segments.clamp(4, 64);
                let angle_step = sweep / segments as f64;

                let mut pb = PathBuilder::new();
                for i in 0..=segments {
                    let angle = arc.start_angle + i as f64 * angle_step;
                    let p = Point2::new(
                        arc.center.x + arc.radius * angle.cos(),
                        arc.center.y + arc.radius * angle.sin(),
                    );
                    let (sx, sy) = to_screen(p);
                    if i == 0 {
                        pb.move_to(sx, sy);
                    } else {
                        pb.line_to(sx, sy);
                    }
                }
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
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
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            Geometry::Ellipse(ellipse) => {
                let (cx, cy) = to_screen(ellipse.center);
                let major = ellipse.major_axis.norm() as f32 * zoom;
                let minor = major * ellipse.ratio as f32;
                
                if major < 1.0 {
                    let mut pb = PathBuilder::new();
                    pb.push_circle(cx, cy, 1.0);
                    if let Some(path) = pb.finish() {
                        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                    }
                    return;
                }
                
                let segments = 24;
                let mut pb = PathBuilder::new();
                for i in 0..=segments {
                    let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
                    let x = cx + major * angle.cos();
                    let y = cy + minor * angle.sin();
                    if i == 0 {
                        pb.move_to(x, y);
                    } else {
                        pb.line_to(x, y);
                    }
                }
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            _ => {}
        }
    }

    /// 绘制十字光标
    fn draw_crosshair(
        pixmap: &mut Pixmap,
        cursor: Point2,
        center: Point2,
        zoom: f32,
        half_w: f32,
        half_h: f32,
        width: u32,
        height: u32,
    ) {
        let sx = half_w + ((cursor.x - center.x) as f32 * zoom);
        let sy = half_h - ((cursor.y - center.y) as f32 * zoom);

        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(255, 255, 255, 200));
        paint.anti_alias = false;

        let stroke = Stroke { width: 1.0, ..Default::default() };

        // 水平线（全屏宽度）
        let mut pb = PathBuilder::new();
        pb.move_to(0.0, sy);
        pb.line_to(width as f32, sy);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }

        // 垂直线（全屏高度）
        let mut pb = PathBuilder::new();
        pb.move_to(sx, 0.0);
        pb.line_to(sx, height as f32);
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    /// 绘制捕捉标记
    fn draw_snap_marker(
        pixmap: &mut Pixmap,
        snap: Point2,
        center: Point2,
        zoom: f32,
        half_w: f32,
        half_h: f32,
    ) {
        let sx = half_w + ((snap.x - center.x) as f32 * zoom);
        let sy = half_h - ((snap.y - center.y) as f32 * zoom);

        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(255, 255, 0, 255));
        paint.anti_alias = true;

        let stroke = Stroke { width: 2.0, ..Default::default() };

        // 画一个小方块
        let size = 5.0;
        let mut pb = PathBuilder::new();
        pb.move_to(sx - size, sy - size);
        pb.line_to(sx + size, sy - size);
        pb.line_to(sx + size, sy + size);
        pb.line_to(sx - size, sy + size);
        pb.close();
        if let Some(path) = pb.finish() {
            pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
        }
    }

    /// 混合两个图层（简单的 alpha 混合）
    fn blend_layer(dest: &mut Pixmap, src: &Pixmap) {
        let dest_data = dest.data_mut();
        let src_data = src.data();
        
        for i in (0..dest_data.len()).step_by(4) {
            let sa = src_data[i + 3] as f32 / 255.0;
            if sa > 0.0 {
                let da = dest_data[i + 3] as f32 / 255.0;
                let out_a = sa + da * (1.0 - sa);
                
                if out_a > 0.0 {
                    dest_data[i] = ((src_data[i] as f32 * sa + dest_data[i] as f32 * da * (1.0 - sa)) / out_a) as u8;
                    dest_data[i + 1] = ((src_data[i + 1] as f32 * sa + dest_data[i + 1] as f32 * da * (1.0 - sa)) / out_a) as u8;
                    dest_data[i + 2] = ((src_data[i + 2] as f32 * sa + dest_data[i + 2] as f32 * da * (1.0 - sa)) / out_a) as u8;
                    dest_data[i + 3] = (out_a * 255.0) as u8;
                }
            }
        }
    }

    /// 获取 egui 纹理
    pub fn get_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureId> {
        let pixmap = self.layer_composite.as_ref()?;
        let size = [pixmap.width() as usize, pixmap.height() as usize];
        let image = egui::ColorImage::from_rgba_premultiplied(size, pixmap.data());

        match &mut self.texture_handle {
            Some(handle) => {
                handle.set(image, egui::TextureOptions::NEAREST);
            }
            None => {
                self.texture_handle = Some(ctx.load_texture(
                    "layered_canvas",
                    image,
                    egui::TextureOptions::NEAREST,
                ));
            }
        }

        self.texture_handle.as_ref().map(|h| h.id())
    }

    /// 强制全部重绘
    pub fn invalidate_all(&mut self) {
        self.redraw_flags = RedrawFlags::ALL;
    }
}

impl Default for LayeredRenderer {
    fn default() -> Self {
        Self::new()
    }
}
