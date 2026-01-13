//! 瓦片缓存渲染系统
//!
//! 将画布分成固定大小的瓦片，每个瓦片独立缓存。
//! 平移时只渲染新出现的瓦片，缩放时才需要全部重新渲染。
//! 这是地图软件和专业 CAD 软件常用的成熟技术。

use eframe::egui;
use std::collections::HashMap;
use tiny_skia::{Color, Paint, PathBuilder, Pixmap, Stroke, Transform};
use zcad_core::entity::Entity;
use zcad_core::geometry::Geometry;
use zcad_core::layer::LayerManager;
use zcad_core::math::{BoundingBox2, Point2};

/// 瓦片大小（像素）
const TILE_SIZE: u32 = 256;

/// 瓦片键（缩放级别 + 瓦片坐标）
#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
struct TileKey {
    /// 缩放级别（量化后的 zoom）
    zoom_level: i32,
    /// 瓦片 X 坐标
    tile_x: i32,
    /// 瓦片 Y 坐标
    tile_y: i32,
}

/// 单个瓦片
struct Tile {
    /// 渲染的位图
    pixmap: Pixmap,
    /// 实体版本（用于判断是否需要重新渲染）
    entity_version: u64,
}

/// 瓦片缓存渲染器
pub struct TileCacheRenderer {
    /// 瓦片缓存
    tiles: HashMap<TileKey, Tile>,
    /// 合成后的最终图像
    composite_pixmap: Option<Pixmap>,
    /// egui 纹理句柄
    texture_handle: Option<egui::TextureHandle>,
    /// 缓存的视口大小
    cached_viewport_size: (u32, u32),
    /// 缓存的缩放级别
    cached_zoom_level: i32,
    /// 最大缓存瓦片数
    max_tiles: usize,
}

impl TileCacheRenderer {
    pub fn new() -> Self {
        Self {
            tiles: HashMap::new(),
            composite_pixmap: None,
            texture_handle: None,
            cached_viewport_size: (0, 0),
            cached_zoom_level: 0,
            max_tiles: 100, // 最多缓存 100 个瓦片
        }
    }

    /// 将 zoom 量化为缩放级别
    fn zoom_to_level(zoom: f64) -> i32 {
        // 使用对数量化，每 2 倍缩放增加一级
        (zoom.log2() * 2.0).round() as i32
    }

    /// 将缩放级别转换回 zoom
    fn level_to_zoom(level: i32) -> f64 {
        2.0_f64.powf(level as f64 / 2.0)
    }

    /// 计算世界坐标到瓦片坐标
    fn world_to_tile(world_x: f64, world_y: f64, zoom: f64) -> (i32, i32) {
        let pixel_x = world_x * zoom;
        let pixel_y = world_y * zoom;
        let tile_x = (pixel_x / TILE_SIZE as f64).floor() as i32;
        let tile_y = (pixel_y / TILE_SIZE as f64).floor() as i32;
        (tile_x, tile_y)
    }

    /// 渲染视口
    pub fn render(
        &mut self,
        entities: &[&Entity],
        layers: &LayerManager,
        center: Point2,
        zoom: f64,
        viewport_width: u32,
        viewport_height: u32,
        entity_version: u64,
        show_grid: bool,
    ) {
        let zoom_level = Self::zoom_to_level(zoom);
        let effective_zoom = Self::level_to_zoom(zoom_level);

        // 如果缩放级别变化，清除所有瓦片
        if zoom_level != self.cached_zoom_level {
            self.tiles.clear();
            self.cached_zoom_level = zoom_level;
        }

        // 计算需要的瓦片范围
        let half_w = viewport_width as f64 / 2.0 / effective_zoom;
        let half_h = viewport_height as f64 / 2.0 / effective_zoom;
        
        let (min_tile_x, min_tile_y) = Self::world_to_tile(center.x - half_w, center.y - half_h, effective_zoom);
        let (max_tile_x, max_tile_y) = Self::world_to_tile(center.x + half_w, center.y + half_h, effective_zoom);

        // 确保合成图像大小正确
        if self.composite_pixmap.is_none()
            || self.cached_viewport_size != (viewport_width, viewport_height)
        {
            self.composite_pixmap = Pixmap::new(viewport_width, viewport_height);
            self.cached_viewport_size = (viewport_width, viewport_height);
        }

        let composite = match self.composite_pixmap.as_mut() {
            Some(p) => p,
            None => return,
        };

        // 清空合成图像
        composite.fill(Color::from_rgba8(30, 30, 35, 255));

        // 第一阶段：渲染需要的瓦片
        let mut tiles_to_render = Vec::new();
        for tile_y in min_tile_y..=max_tile_y {
            for tile_x in min_tile_x..=max_tile_x {
                let key = TileKey {
                    zoom_level,
                    tile_x,
                    tile_y,
                };

                let needs_render = match self.tiles.get(&key) {
                    Some(tile) => tile.entity_version != entity_version,
                    None => true,
                };

                if needs_render {
                    tiles_to_render.push((key, tile_x, tile_y));
                }
            }
        }

        // 渲染所有需要的瓦片
        for (key, tile_x, tile_y) in tiles_to_render {
            if let Some(tile_pixmap) = Self::render_tile_static(
                entities,
                layers,
                tile_x,
                tile_y,
                effective_zoom,
                entity_version,
                show_grid,
            ) {
                // 限制缓存大小
                if self.tiles.len() >= self.max_tiles {
                    if let Some(old_key) = self.tiles.keys().next().cloned() {
                        self.tiles.remove(&old_key);
                    }
                }
                self.tiles.insert(key, Tile {
                    pixmap: tile_pixmap,
                    entity_version,
                });
            }
        }

        // 第二阶段：合成瓦片到最终图像
        for tile_y in min_tile_y..=max_tile_y {
            for tile_x in min_tile_x..=max_tile_x {
                let key = TileKey {
                    zoom_level,
                    tile_x,
                    tile_y,
                };

                if let Some(tile) = self.tiles.get(&key) {
                    let tile_world_x = tile_x as f64 * TILE_SIZE as f64 / effective_zoom;
                    let tile_world_y = tile_y as f64 * TILE_SIZE as f64 / effective_zoom;
                    
                    let screen_x = ((tile_world_x - center.x) * effective_zoom + viewport_width as f64 / 2.0) as i32;
                    let screen_y = (viewport_height as f64 / 2.0 - (tile_world_y - center.y) * effective_zoom - TILE_SIZE as f64) as i32;

                    Self::blit_tile_static(composite, &tile.pixmap, screen_x, screen_y);
                }
            }
        }
    }

    /// 渲染单个瓦片
    fn render_tile_static(
        entities: &[&Entity],
        layers: &LayerManager,
        tile_x: i32,
        tile_y: i32,
        zoom: f64,
        _entity_version: u64,
        show_grid: bool,
    ) -> Option<Pixmap> {
        let mut pixmap = Pixmap::new(TILE_SIZE, TILE_SIZE)?;
        pixmap.fill(Color::from_rgba8(30, 30, 35, 255));

        // 计算瓦片的世界坐标范围
        let world_left = tile_x as f64 * TILE_SIZE as f64 / zoom;
        let world_bottom = tile_y as f64 * TILE_SIZE as f64 / zoom;
        let world_right = world_left + TILE_SIZE as f64 / zoom;
        let world_top = world_bottom + TILE_SIZE as f64 / zoom;

        let tile_bounds = BoundingBox2::new(
            Point2::new(world_left, world_bottom),
            Point2::new(world_right, world_top),
        );

        // 绘制网格
        if show_grid {
            Self::draw_grid_on_tile(&mut pixmap, world_left, world_bottom, zoom);
        }

        // 绘制与此瓦片相交的实体
        for entity in entities {
            if let Some(geometry) = entity.geometry() {
                let entity_bounds = geometry.bounding_box();
                if tile_bounds.intersects(&entity_bounds) {
                    let color = if entity.visual_properties.color.is_by_layer() {
                        layers
                            .get_layer_by_id(entity.layer_id)
                            .map(|l| l.color)
                            .unwrap_or(zcad_core::properties::Color::WHITE)
                    } else {
                        entity.visual_properties.color
                    };
                    Self::draw_geometry_on_tile(
                        &mut pixmap,
                        geometry,
                        color,
                        world_left,
                        world_top, // 注意 Y 轴方向
                        zoom,
                    );
                }
            }
        }

        Some(pixmap)
    }

    /// 在瓦片上绘制网格
    fn draw_grid_on_tile(pixmap: &mut Pixmap, world_left: f64, world_bottom: f64, zoom: f64) {
        let tile_size = TILE_SIZE as f64;
        let world_size = tile_size / zoom;

        // 计算网格间距
        let mut spacing = 50.0;
        while spacing * zoom < 20.0 {
            spacing *= 5.0;
        }
        while spacing * zoom > 200.0 {
            spacing /= 5.0;
        }

        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(60, 60, 70, 255));
        paint.anti_alias = false;

        let stroke = Stroke {
            width: 1.0,
            ..Default::default()
        };

        // 垂直线
        let start_x = (world_left / spacing).floor() * spacing;
        let end_x = world_left + world_size;
        let mut x = start_x;
        while x <= end_x {
            let sx = ((x - world_left) * zoom) as f32;
            if sx >= 0.0 && sx <= tile_size as f32 {
                let mut pb = PathBuilder::new();
                pb.move_to(sx, 0.0);
                pb.line_to(sx, tile_size as f32);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            x += spacing;
        }

        // 水平线
        let start_y = (world_bottom / spacing).floor() * spacing;
        let end_y = world_bottom + world_size;
        let mut y = start_y;
        while y <= end_y {
            let sy = tile_size as f32 - ((y - world_bottom) * zoom) as f32;
            if sy >= 0.0 && sy <= tile_size as f32 {
                let mut pb = PathBuilder::new();
                pb.move_to(0.0, sy);
                pb.line_to(tile_size as f32, sy);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            y += spacing;
        }
    }

    /// 在瓦片上绘制几何体
    fn draw_geometry_on_tile(
        pixmap: &mut Pixmap,
        geometry: &Geometry,
        color: zcad_core::properties::Color,
        world_left: f64,
        world_top: f64,
        zoom: f64,
    ) {
        let mut paint = Paint::default();
        paint.set_color(Color::from_rgba8(color.r, color.g, color.b, 255));
        paint.anti_alias = true;

        let stroke = Stroke {
            width: 1.0,
            ..Default::default()
        };

        // 世界坐标转瓦片像素坐标
        let to_tile = |p: Point2| -> (f32, f32) {
            let x = ((p.x - world_left) * zoom) as f32;
            let y = ((world_top - p.y) * zoom) as f32;
            (x, y)
        };

        match geometry {
            Geometry::Point(p) => {
                let (sx, sy) = to_tile(p.position);
                let mut pb = PathBuilder::new();
                pb.push_circle(sx, sy, 2.0);
                if let Some(path) = pb.finish() {
                    pixmap.fill_path(&path, &paint, tiny_skia::FillRule::Winding, Transform::identity(), None);
                }
            }
            Geometry::Line(line) => {
                let (x1, y1) = to_tile(line.start);
                let (x2, y2) = to_tile(line.end);
                let mut pb = PathBuilder::new();
                pb.move_to(x1, y1);
                pb.line_to(x2, y2);
                if let Some(path) = pb.finish() {
                    pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                }
            }
            Geometry::Circle(circle) => {
                let (cx, cy) = to_tile(circle.center);
                let r = (circle.radius * zoom) as f32;
                if r > 0.5 {
                    let mut pb = PathBuilder::new();
                    pb.push_circle(cx, cy, r);
                    if let Some(path) = pb.finish() {
                        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                    }
                }
            }
            Geometry::Arc(arc) => {
                let screen_radius = arc.radius * zoom;
                let sweep = arc.sweep_angle();
                let segments = ((screen_radius * sweep.abs()) / 8.0).ceil() as usize;
                let segments = segments.clamp(4, 32);
                let angle_step = sweep / segments as f64;

                let mut pb = PathBuilder::new();
                let mut first = true;
                for i in 0..=segments {
                    let angle = arc.start_angle + i as f64 * angle_step;
                    let p = Point2::new(
                        arc.center.x + arc.radius * angle.cos(),
                        arc.center.y + arc.radius * angle.sin(),
                    );
                    let (sx, sy) = to_tile(p);
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
                if polyline.vertices.len() >= 2 {
                    let mut pb = PathBuilder::new();
                    let (x, y) = to_tile(polyline.vertices[0].point);
                    pb.move_to(x, y);
                    for v in polyline.vertices.iter().skip(1) {
                        let (x, y) = to_tile(v.point);
                        pb.line_to(x, y);
                    }
                    if polyline.closed {
                        pb.close();
                    }
                    if let Some(path) = pb.finish() {
                        pixmap.stroke_path(&path, &paint, &stroke, Transform::identity(), None);
                    }
                }
            }
            Geometry::Ellipse(ellipse) => {
                let (cx, cy) = to_tile(ellipse.center);
                let major = (ellipse.major_axis.norm() * zoom) as f32;
                let minor = major * ellipse.ratio as f32;
                if major > 0.5 {
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
            _ => {}
        }
    }

    /// 将瓦片复制到合成图像
    fn blit_tile_static(dest: &mut Pixmap, src: &Pixmap, x: i32, y: i32) {
        let dest_width = dest.width() as i32;
        let dest_height = dest.height() as i32;
        let src_width = src.width() as i32;
        let src_height = src.height() as i32;

        // 计算有效的复制区域
        let src_x_start = (-x).max(0);
        let src_y_start = (-y).max(0);
        let dest_x_start = x.max(0);
        let dest_y_start = y.max(0);

        let copy_width = (src_width - src_x_start).min(dest_width - dest_x_start);
        let copy_height = (src_height - src_y_start).min(dest_height - dest_y_start);

        if copy_width <= 0 || copy_height <= 0 {
            return;
        }

        let src_data = src.data();
        let dest_data = dest.data_mut();

        for row in 0..copy_height {
            let src_row = (src_y_start + row) as usize;
            let dest_row = (dest_y_start + row) as usize;
            let src_offset = (src_row * src_width as usize + src_x_start as usize) * 4;
            let dest_offset = (dest_row * dest_width as usize + dest_x_start as usize) * 4;
            let bytes = copy_width as usize * 4;

            dest_data[dest_offset..dest_offset + bytes]
                .copy_from_slice(&src_data[src_offset..src_offset + bytes]);
        }
    }

    /// 获取 egui 纹理
    pub fn get_texture(&mut self, ctx: &egui::Context) -> Option<egui::TextureId> {
        let pixmap = self.composite_pixmap.as_ref()?;
        let size = [pixmap.width() as usize, pixmap.height() as usize];
        let image = egui::ColorImage::from_rgba_premultiplied(size, pixmap.data());

        match &mut self.texture_handle {
            Some(handle) => {
                handle.set(image, egui::TextureOptions::NEAREST);
            }
            None => {
                self.texture_handle = Some(ctx.load_texture(
                    "tile_cache",
                    image,
                    egui::TextureOptions::NEAREST,
                ));
            }
        }

        self.texture_handle.as_ref().map(|h| h.id())
    }

    /// 清除所有缓存
    pub fn clear(&mut self) {
        self.tiles.clear();
    }

    /// 标记需要重新渲染（实体变化时调用）
    pub fn invalidate(&mut self) {
        self.tiles.clear();
    }

    /// 获取缓存统计
    pub fn stats(&self) -> (usize, usize) {
        (self.tiles.len(), self.max_tiles)
    }
}

impl Default for TileCacheRenderer {
    fn default() -> Self {
        Self::new()
    }
}
