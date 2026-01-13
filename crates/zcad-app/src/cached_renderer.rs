//! 缓存渲染引擎
//!
//! 使用 tiny-skia 将 CAD 视图渲染成位图，只在相机变化时重新渲染。
//! 这大大减少了每帧的渲染开销。

use eframe::egui;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};
use zcad_core::entity::Entity;
use zcad_core::geometry::Geometry;
use zcad_core::math::Point2;

/// 缓存渲染器
pub struct CachedRenderer {
    /// 渲染的位图缓存
    pixmap: Option<Pixmap>,
    /// egui 纹理句柄
    texture_handle: Option<egui::TextureHandle>,
    /// 缓存的相机中心
    cached_center: Point2,
    /// 缓存的缩放
    cached_zoom: f64,
    /// 缓存的视口大小
    cached_size: (u32, u32),
    /// 缓存的实体版本号（用于检测变化）
    cached_entity_version: u64,
    /// 是否需要重新渲染
    dirty: bool,
}

impl CachedRenderer {
    pub fn new() -> Self {
        Self {
            pixmap: None,
            texture_handle: None,
            cached_center: Point2::origin(),
            cached_zoom: 1.0,
            cached_size: (0, 0),
            cached_entity_version: 0,
            dirty: true,
        }
    }

    /// 检查是否需要重新渲染
    pub fn needs_render(
        &self,
        center: Point2,
        zoom: f64,
        width: u32,
        height: u32,
        entity_version: u64,
    ) -> bool {
        self.dirty
            || (self.cached_center.x - center.x).abs() > 0.001
            || (self.cached_center.y - center.y).abs() > 0.001
            || (self.cached_zoom - zoom).abs() > 0.0001
            || self.cached_size != (width, height)
            || self.cached_entity_version != entity_version
    }

    /// 渲染所有实体到缓存
    pub fn render(
        &mut self,
        entities: &[&Entity],
        layers: &zcad_core::layer::LayerManager,
        center: Point2,
        zoom: f64,
        width: u32,
        height: u32,
        entity_version: u64,
        show_grid: bool,
    ) {
        // 创建或重用 pixmap
        if self.pixmap.is_none()
            || self.cached_size != (width, height)
        {
            self.pixmap = Pixmap::new(width, height);
        }

        let pixmap = match self.pixmap.as_mut() {
            Some(p) => p,
            None => return,
        };

        // 清空背景（深色）
        pixmap.fill(Color::from_rgba8(30, 30, 35, 255));

        // 计算变换矩阵：世界坐标 -> 屏幕坐标
        let half_w = width as f32 / 2.0;
        let half_h = height as f32 / 2.0;
        let zoom_f = zoom as f32;

        // 绘制网格
        if show_grid {
            Self::draw_grid_static(pixmap, center, zoom, width, height);
        }

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
                Self::draw_geometry_static(pixmap, geometry, color, center, zoom_f, half_w, half_h);
            }
        }

        // 更新缓存状态
        self.cached_center = center;
        self.cached_zoom = zoom;
        self.cached_size = (width, height);
        self.cached_entity_version = entity_version;
        self.dirty = false;
    }

    /// 绘制网格
    fn draw_grid_static(
        pixmap: &mut Pixmap,
        center: Point2,
        zoom: f64,
        width: u32,
        height: u32,
    ) {
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

        let start_x = (left / spacing).floor() * spacing;
        let end_x = (right / spacing).ceil() * spacing;
        let start_y = (bottom / spacing).floor() * spacing;
        let end_y = (top / spacing).ceil() * spacing;

        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(60, 60, 70, 255));
        paint.anti_alias = false;

        let stroke = Stroke {
            width: 1.0,
            ..Default::default()
        };

        // 垂直线
        let mut x = start_x;
        while x <= end_x {
            let sx = half_w + (x - center.x) * zoom;
            if sx >= 0.0 && sx <= width as f64 {
                let mut pb = PathBuilder::new();
                pb.move_to(sx as f32, 0.0);
                pb.line_to(sx as f32, height as f32);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            x += spacing;
        }

        // 水平线
        let mut y = start_y;
        while y <= end_y {
            let sy = half_h - (y - center.y) * zoom;
            if sy >= 0.0 && sy <= height as f64 {
                let mut pb = PathBuilder::new();
                pb.move_to(0.0, sy as f32);
                pb.line_to(width as f32, sy as f32);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            y += spacing;
        }
    }

    /// 绘制几何体
    fn draw_geometry_static(
        pixmap: &mut Pixmap,
        geometry: &Geometry,
        color: zcad_core::properties::Color,
        center: Point2,
        zoom: f32,
        half_w: f32,
        half_h: f32,
    ) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color.r, color.g, color.b, 255));
        paint.anti_alias = true;

        let stroke = Stroke {
            width: 1.0,
            ..Default::default()
        };

        // 世界坐标转屏幕坐标的闭包
        let to_screen = |p: Point2| -> (f32, f32) {
            let x = half_w + ((p.x - center.x) as f32 * zoom);
            let y = half_h - ((p.y - center.y) as f32 * zoom);
            (x, y)
        };

        match geometry {
            Geometry::Point(p) => {
                let (sx, sy) = to_screen(p.position);
                // 画一个小圆点
                let mut pb = PathBuilder::new();
                pb.push_circle(sx, sy, 2.0);
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            }
            Geometry::Line(line) => {
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
                let (cx, cy) = to_screen(circle.center);
                let r = (circle.radius as f32) * zoom;
                
                if r < 1.0 {
                    // 太小，画点
                    let mut pb = PathBuilder::new();
                    pb.push_circle(cx, cy, 1.0);
                    if let Some(path) = pb.finish() {
                        pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                    }
                } else {
                    let mut pb = PathBuilder::new();
                    pb.push_circle(cx, cy, r);
                    if let Some(path) = pb.finish() {
                        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                    }
                }
            }
            Geometry::Arc(arc) => {
                let screen_radius = arc.radius as f32 * zoom;
                let sweep = arc.sweep_angle();
                
                // 计算分段数（LOD）
                let segments = if screen_radius < 5.0 {
                    4
                } else if screen_radius < 20.0 {
                    8
                } else {
                    ((screen_radius * sweep.abs() as f32) / 8.0).ceil() as usize
                }.clamp(4, 32);

                let angle_step = sweep / segments as f64;
                let mut pb = PathBuilder::new();
                let mut first = true;
                
                for i in 0..=segments {
                    let angle = arc.start_angle + i as f64 * angle_step;
                    let p = Point2::new(
                        arc.center.x + arc.radius * angle.cos(),
                        arc.center.y + arc.radius * angle.sin(),
                    );
                    let (sx, sy) = to_screen(p);
                    if first {
                        pb.move_to(sx, sy);
                        first = false;
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
                
                for i in 1..polyline.vertices.len() {
                    let (x, y) = to_screen(polyline.vertices[i].point);
                    pb.line_to(x, y);
                }
                
                if polyline.closed && polyline.vertices.len() > 2 {
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
                } else {
                    // 用多边形近似椭圆
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
            }
            // 文本、标注等复杂类型跳过（由 egui 叠加渲染）
            _ => {}
        }
    }

    /// 获取或更新 egui 纹理
    pub fn get_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureId> {
        let pixmap = self.pixmap.as_ref()?;
        
        // 转换为 egui 格式
        let size = [pixmap.width() as usize, pixmap.height() as usize];
        let image = egui::ColorImage::from_rgba_premultiplied(size, pixmap.data());

        // 创建或更新纹理
        match &mut self.texture_handle {
            Some(handle) => {
                handle.set(image, egui::TextureOptions::NEAREST);
            }
            None => {
                self.texture_handle = Some(ctx.load_texture(
                    "cad_canvas",
                    image,
                    egui::TextureOptions::NEAREST,
                ));
            }
        }

        self.texture_handle.as_ref().map(|h| h.id())
    }

    /// 标记为需要重新渲染
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Default for CachedRenderer {
    fn default() -> Self {
        Self::new()
    }
}
