//! GPU 渲染器 - 统一的多层渲染架构
//!
//! 架构：
//! - Layer 0: wgpu 3D 场景（IFC / 模型）
//! - Layer 1: Vello 2D 矢量（剖面线、标注、尺寸、路径）
//! - Layer 2: egui UI（面板、参数、调试）
//!
//! 注意：由于 vello 和 egui 使用不同版本的 wgpu，
//! Vello 渲染器使用独立的 wgpu 设备，结果通过 CPU 传输到 egui 纹理

use std::sync::Arc;
use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};
// 使用 vello 重新导出的 wgpu 类型
use vello::wgpu;
use vello::{AaConfig, Renderer as VelloRenderer, RendererOptions, Scene};
use vello::kurbo::{Affine, BezPath, Cap, Circle as KurboCircle, Join, Line as KurboLine, Point as KurboPoint, Rect, Stroke};
use vello::peniko::{Color as VelloColor, Fill};
use zcad_core::entity::Entity;
use zcad_core::geometry::Geometry;
use zcad_core::layer::LayerManager;
use zcad_core::math::Point2;

/// GPU 渲染器状态
pub struct GpuRenderer {
    /// Vello 独立的 wgpu 设备
    device: Option<Arc<wgpu::Device>>,
    queue: Option<Arc<wgpu::Queue>>,
    /// Vello 渲染器
    vello_renderer: Option<VelloRenderer>,
    /// 渲染目标纹理
    target_texture: Option<wgpu::Texture>,
    /// 用于读取渲染结果的缓冲区
    read_buffer: Option<wgpu::Buffer>,
    /// egui 纹理句柄
    texture: Option<TextureHandle>,
    /// 缓存的尺寸
    cached_size: (u32, u32),
    /// 缓存的相机状态
    cached_center: Point2,
    cached_zoom: f64,
    /// 缓存的实体版本
    cached_entity_version: u64,
    /// 是否已初始化
    initialized: bool,
}

impl GpuRenderer {
    pub fn new() -> Self {
        Self {
            device: None,
            queue: None,
            vello_renderer: None,
            target_texture: None,
            read_buffer: None,
            texture: None,
            cached_size: (0, 0),
            cached_center: Point2::origin(),
            cached_zoom: 1.0,
            cached_entity_version: 0,
            initialized: false,
        }
    }

    /// 初始化 Vello 渲染器（使用独立的 wgpu 设备）
    fn ensure_initialized(&mut self) {
        if self.initialized {
            return;
        }

        // 创建独立的 wgpu 实例和设备
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = match pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: None,
        })) {
            Ok(a) => a,
            Err(e) => {
                tracing::error!("Failed to get wgpu adapter for Vello: {:?}", e);
                return;
            }
        };

        let (device, queue) = match pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label: Some("vello_device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            },
        )) {
            Ok((d, q)) => (d, q),
            Err(e) => {
                tracing::error!("Failed to create wgpu device for Vello: {:?}", e);
                return;
            }
        };

        let device = Arc::new(device);
        let queue = Arc::new(queue);

        // 创建 Vello 渲染器
        let options = RendererOptions {
            use_cpu: false,
            antialiasing_support: vello::AaSupport::all(),
            num_init_threads: None,
            pipeline_cache: None,
        };

        match VelloRenderer::new(&device, options) {
            Ok(renderer) => {
                tracing::info!("Vello GPU renderer initialized with independent device");
                self.vello_renderer = Some(renderer);
                self.device = Some(device);
                self.queue = Some(queue);
                self.initialized = true;
            }
            Err(e) => {
                tracing::error!("Failed to create Vello renderer: {:?}", e);
            }
        }
    }

    /// 确保渲染目标纹理存在且尺寸正确
    fn ensure_target_texture(&mut self, width: u32, height: u32) {
        let device = match &self.device {
            Some(d) => d,
            None => return,
        };

        if self.cached_size != (width, height) || self.target_texture.is_none() {
            let width = width.max(1);
            let height = height.max(1);

            // 创建渲染目标纹理
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("vello_target"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8Unorm,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::STORAGE_BINDING,
                view_formats: &[],
            });

            // 创建读取缓冲区
            let bytes_per_row = ((width * 4 + 255) / 256) * 256; // 对齐到 256
            let buffer_size = (bytes_per_row * height) as u64;
            let read_buffer = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("vello_read_buffer"),
                size: buffer_size,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });

            self.target_texture = Some(texture);
            self.read_buffer = Some(read_buffer);
            self.cached_size = (width, height);
        }
    }

    /// 检查是否需要重新渲染
    fn needs_update(&self, center: Point2, zoom: f64, entity_version: u64) -> bool {
        if self.cached_entity_version != entity_version {
            return true;
        }
        // 使用相对阈值，避免视野大时频繁触发
        let center_threshold = 1.0 / zoom.max(0.0001); // 视野越大，阈值越大
        let zoom_threshold = self.cached_zoom * 0.001; // 0.1% 的缩放变化
        
        if (self.cached_center.x - center.x).abs() > center_threshold
            || (self.cached_center.y - center.y).abs() > center_threshold
            || (self.cached_zoom - zoom).abs() > zoom_threshold
        {
            return true;
        }
        false
    }

    /// 标记为需要重新渲染
    pub fn invalidate(&mut self) {
        self.cached_entity_version = 0;
    }

    /// 渲染 CAD 内容到 GPU 纹理
    pub fn render(
        &mut self,
        ctx: &egui::Context,
        entities: &[&Entity],
        layers: &LayerManager,
        center: Point2,
        zoom: f64,
        width: u32,
        height: u32,
        entity_version: u64,
        show_grid: bool,
    ) -> Option<egui::TextureId> {
        if width == 0 || height == 0 {
            return None;
        }

        // 确保初始化
        self.ensure_initialized();
        if !self.initialized {
            return None;
        }

        // 检查是否需要重新渲染
        let needs_render = self.needs_update(center, zoom, entity_version) 
            || self.texture.is_none()
            || self.cached_size != (width, height);

        if !needs_render {
            return self.texture.as_ref().map(|t| t.id());
        }

        // 确保渲染目标纹理存在
        self.ensure_target_texture(width, height);

        let device = self.device.as_ref()?;
        let queue = self.queue.as_ref()?;
        let renderer = self.vello_renderer.as_mut()?;
        let texture = self.target_texture.as_ref()?;
        let read_buffer = self.read_buffer.as_ref()?;

        // 创建 Vello 场景
        let mut scene = Scene::new();
        
        let half_w = width as f64 / 2.0;
        let half_h = height as f64 / 2.0;

        // 绘制背景
        let bg_rect = Rect::new(0.0, 0.0, width as f64, height as f64);
        scene.fill(Fill::NonZero, Affine::IDENTITY, VelloColor::new([0.118f32, 0.118, 0.137, 1.0]), None, &bg_rect);

        // 绘制网格
        if show_grid {
            Self::draw_grid(&mut scene, center, zoom, width, height);
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
                Self::draw_geometry(&mut scene, geometry, color, center, zoom, half_w, half_h);
            }
        }

        // 渲染到纹理
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let render_params = vello::RenderParams {
            base_color: VelloColor::TRANSPARENT,
            width,
            height,
            antialiasing_method: AaConfig::Msaa16,
        };

        if let Err(e) = renderer.render_to_texture(device, queue, &scene, &view, &render_params) {
            tracing::error!("Vello render error: {:?}", e);
            return None;
        }

        // 将渲染结果复制到 CPU 缓冲区
        let bytes_per_row = ((width * 4 + 255) / 256) * 256;
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("vello_copy_encoder"),
        });

        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: read_buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        queue.submit(Some(encoder.finish()));

        // 映射缓冲区并读取数据
        let buffer_slice = read_buffer.slice(..);
        let (tx, rx) = std::sync::mpsc::channel();
        buffer_slice.map_async(wgpu::MapMode::Read, move |result| {
            tx.send(result).unwrap();
        });
        device.poll(wgpu::PollType::Wait).ok();

        if rx.recv().ok()?.is_err() {
            return None;
        }

        // 读取像素数据
        let data = buffer_slice.get_mapped_range();
        let mut pixels = Vec::with_capacity((width * height * 4) as usize);
        
        for row in 0..height {
            let start = (row * bytes_per_row) as usize;
            let end = start + (width * 4) as usize;
            pixels.extend_from_slice(&data[start..end]);
        }
        drop(data);
        read_buffer.unmap();

        // 创建 egui ColorImage
        let image = ColorImage::from_rgba_unmultiplied([width as usize, height as usize], &pixels);

        // 更新 egui 纹理
        match &mut self.texture {
            Some(handle) => {
                handle.set(image, TextureOptions::LINEAR);
            }
            None => {
                self.texture = Some(ctx.load_texture(
                    "vello_gpu_canvas",
                    image,
                    TextureOptions::LINEAR,
                ));
            }
        }

        // 更新缓存状态
        self.cached_center = center;
        self.cached_zoom = zoom;
        self.cached_entity_version = entity_version;

        self.texture.as_ref().map(|t| t.id())
    }

    /// 绘制网格（限制最大线条数量）
    fn draw_grid(scene: &mut Scene, center: Point2, zoom: f64, width: u32, height: u32) {
        const MAX_GRID_LINES: usize = 100; // 限制最大网格线数量
        
        let half_w = width as f64 / 2.0;
        let half_h = height as f64 / 2.0;

        let grid_color = VelloColor::new([0.196f32, 0.196, 0.235, 1.0]);
        let axis_color = VelloColor::new([0.314f32, 0.314, 0.392, 1.0]);
        let stroke = Stroke::new(1.0);

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

        // 坐标转换
        let to_screen = |p: Point2| -> KurboPoint {
            let x = half_w + (p.x - center.x) * zoom;
            let y = half_h - (p.y - center.y) * zoom;
            KurboPoint::new(x, y)
        };

        // 垂直线
        let start_x = (left / spacing).floor() * spacing;
        let mut x = start_x;
        let mut count = 0;
        while x <= right && count < MAX_GRID_LINES {
            let p1 = to_screen(Point2::new(x, bottom));
            let p2 = to_screen(Point2::new(x, top));
            let line = KurboLine::new(p1, p2);
            scene.stroke(&stroke, Affine::IDENTITY, grid_color, None, &line);
            x += spacing;
            count += 1;
        }

        // 水平线
        let start_y = (bottom / spacing).floor() * spacing;
        let mut y = start_y;
        count = 0;
        while y <= top && count < MAX_GRID_LINES {
            let p1 = to_screen(Point2::new(left, y));
            let p2 = to_screen(Point2::new(right, y));
            let line = KurboLine::new(p1, p2);
            scene.stroke(&stroke, Affine::IDENTITY, grid_color, None, &line);
            y += spacing;
            count += 1;
        }

        // 坐标轴
        if bottom <= 0.0 && top >= 0.0 {
            let p1 = to_screen(Point2::new(left, 0.0));
            let p2 = to_screen(Point2::new(right, 0.0));
            let line = KurboLine::new(p1, p2);
            scene.stroke(&stroke, Affine::IDENTITY, axis_color, None, &line);
        }
        if left <= 0.0 && right >= 0.0 {
            let p1 = to_screen(Point2::new(0.0, bottom));
            let p2 = to_screen(Point2::new(0.0, top));
            let line = KurboLine::new(p1, p2);
            scene.stroke(&stroke, Affine::IDENTITY, axis_color, None, &line);
        }
    }

    /// 绘制几何体
    fn draw_geometry(
        scene: &mut Scene,
        geometry: &Geometry,
        color: zcad_core::properties::Color,
        center: Point2,
        zoom: f64,
        half_w: f64,
        half_h: f64,
    ) {
        let vello_color = VelloColor::new([
            color.r as f32 / 255.0,
            color.g as f32 / 255.0,
            color.b as f32 / 255.0,
            1.0f32
        ]);
        let stroke = Stroke::new(1.5).with_caps(Cap::Round).with_join(Join::Round);

        // 坐标转换
        let to_screen = |p: Point2| -> KurboPoint {
            let x = half_w + (p.x - center.x) * zoom;
            let y = half_h - (p.y - center.y) * zoom;
            KurboPoint::new(x, y)
        };

        match geometry {
            Geometry::Point(point) => {
                let screen = to_screen(point.position);
                let circle = KurboCircle::new(screen, 3.0);
                scene.fill(Fill::NonZero, Affine::IDENTITY, vello_color, None, &circle);
            }
            Geometry::Line(line) => {
                let p1 = to_screen(line.start);
                let p2 = to_screen(line.end);
                let kurbo_line = KurboLine::new(p1, p2);
                scene.stroke(&stroke, Affine::IDENTITY, vello_color, None, &kurbo_line);
            }
            Geometry::Circle(circle) => {
                let screen_center = to_screen(circle.center);
                let screen_radius = circle.radius * zoom;
                if screen_radius < 0.5 {
                    return;
                }
                let kurbo_circle = KurboCircle::new(screen_center, screen_radius);
                scene.stroke(&stroke, Affine::IDENTITY, vello_color, None, &kurbo_circle);
            }
            Geometry::Arc(arc) => {
                let screen_radius = arc.radius * zoom;
                if screen_radius < 0.5 {
                    return;
                }
                
                let mut path = BezPath::new();
                let sweep = arc.sweep_angle();
                let segments = ((screen_radius * sweep.abs()) / 5.0).ceil() as usize;
                let segments = segments.clamp(4, 64);
                let angle_step = sweep / segments as f64;

                let start_p = arc.start_point();
                let screen_start = to_screen(start_p);
                path.move_to(screen_start);

                for i in 1..=segments {
                    let angle = arc.start_angle + i as f64 * angle_step;
                    let p = Point2::new(
                        arc.center.x + arc.radius * angle.cos(),
                        arc.center.y + arc.radius * angle.sin(),
                    );
                    let screen_p = to_screen(p);
                    path.line_to(screen_p);
                }

                scene.stroke(&stroke, Affine::IDENTITY, vello_color, None, &path);
            }
            Geometry::Polyline(polyline) => {
                if polyline.vertices.len() < 2 {
                    return;
                }
                
                let mut path = BezPath::new();
                let first = to_screen(polyline.vertices[0].point);
                path.move_to(first);
                
                for v in polyline.vertices.iter().skip(1) {
                    let p = to_screen(v.point);
                    path.line_to(p);
                }
                
                if polyline.closed {
                    path.close_path();
                }
                
                scene.stroke(&stroke, Affine::IDENTITY, vello_color, None, &path);
            }
            Geometry::Ellipse(ellipse) => {
                let points = ellipse.sample_points(48);
                if points.len() < 2 {
                    return;
                }
                
                let mut path = BezPath::new();
                let first = to_screen(points[0]);
                path.move_to(first);
                
                for p in points.iter().skip(1) {
                    let screen_p = to_screen(*p);
                    path.line_to(screen_p);
                }
                path.close_path();
                
                scene.stroke(&stroke, Affine::IDENTITY, vello_color, None, &path);
            }
            Geometry::Text(text) => {
                // TODO: 使用 Parley 进行文字布局
                // 目前用矩形表示
                let screen = to_screen(text.position);
                let h = text.height * zoom;
                if h < 2.0 {
                    return;
                }
                let w = h * text.content.chars().count() as f64 * 0.6;
                let rect = Rect::new(screen.x, screen.y - h, screen.x + w, screen.y);
                let thin_stroke = Stroke::new(1.0);
                scene.stroke(&thin_stroke, Affine::IDENTITY, vello_color, None, &rect);
            }
            Geometry::Spline(spline) => {
                if spline.control_points.len() < 2 {
                    return;
                }
                
                let samples = 50;
                let mut path = BezPath::new();
                
                for i in 0..=samples {
                    let t = i as f64 / samples as f64;
                    let nalgebra_p = spline.point_at_param(t);
                    let p = Point2::new(nalgebra_p.x, nalgebra_p.y);
                    let screen_p = to_screen(p);
                    if i == 0 {
                        path.move_to(screen_p);
                    } else {
                        path.line_to(screen_p);
                    }
                }
                
                scene.stroke(&stroke, Affine::IDENTITY, vello_color, None, &path);
            }
            Geometry::Dimension(dim) => {
                Self::draw_dimension(scene, dim, vello_color, center, zoom, half_w, half_h);
            }
            _ => {}
        }
    }

    /// 绘制标注
    fn draw_dimension(
        scene: &mut Scene,
        dim: &zcad_core::geometry::Dimension,
        color: VelloColor,
        center: Point2,
        zoom: f64,
        half_w: f64,
        half_h: f64,
    ) {
        let stroke = Stroke::new(1.0).with_caps(Cap::Round).with_join(Join::Round);

        let to_screen = |p: Point2| -> KurboPoint {
            let x = half_w + (p.x - center.x) * zoom;
            let y = half_h - (p.y - center.y) * zoom;
            KurboPoint::new(x, y)
        };

        match dim.dim_type {
            zcad_core::geometry::DimensionType::Aligned | zcad_core::geometry::DimensionType::Linear => {
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
                let dim_scale = 1.0 / zoom;
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
                let ext_line1 = KurboLine::new(to_screen(ext_start_p1), to_screen(ext_end_p1));
                let ext_line2 = KurboLine::new(to_screen(ext_start_p2), to_screen(ext_end_p2));
                scene.stroke(&stroke, Affine::IDENTITY, color, None, &ext_line1);
                scene.stroke(&stroke, Affine::IDENTITY, color, None, &ext_line2);
                
                // 绘制尺寸线
                let dim_line = KurboLine::new(to_screen(dim_p1), to_screen(dim_p2));
                scene.stroke(&stroke, Affine::IDENTITY, color, None, &dim_line);
                
                // 绘制箭头
                Self::draw_arrow(scene, to_screen(dim_p1), to_screen(dim_p2), color);
                Self::draw_arrow(scene, to_screen(dim_p2), to_screen(dim_p1), color);
                
                // 绘制文本占位符（矩形）
                let text_pos = dim.get_text_position();
                let screen_pos = to_screen(text_pos);
                let text_height = dim.text_height * zoom;
                if text_height > 2.0 {
                    let text_content = dim.display_text();
                    let text_width = text_height * text_content.chars().count() as f64 * 0.6;
                    let rect = Rect::new(
                        screen_pos.x - text_width / 2.0,
                        screen_pos.y - text_height / 2.0,
                        screen_pos.x + text_width / 2.0,
                        screen_pos.y + text_height / 2.0,
                    );
                    scene.stroke(&stroke, Affine::IDENTITY, color, None, &rect);
                }
            }
            zcad_core::geometry::DimensionType::Radius => {
                let p1 = dim.definition_point1;  // 圆心
                let p2 = dim.definition_point2;  // 圆上点
                
                // 绘制半径线
                let radius_line = KurboLine::new(to_screen(p1), to_screen(p2));
                scene.stroke(&stroke, Affine::IDENTITY, color, None, &radius_line);
                
                // 绘制箭头
                Self::draw_arrow(scene, to_screen(p1), to_screen(p2), color);
                
                // 绘制文本
                let text_pos = dim.get_text_position();
                let screen_pos = to_screen(text_pos);
                let text_height = dim.text_height * zoom;
                if text_height > 2.0 {
                    let text_content = dim.display_text().replace("%%C", "Ø");
                    let text_width = text_height * text_content.chars().count() as f64 * 0.6;
                    let rect = Rect::new(
                        screen_pos.x - text_width / 2.0,
                        screen_pos.y - text_height / 2.0,
                        screen_pos.x + text_width / 2.0,
                        screen_pos.y + text_height / 2.0,
                    );
                    scene.stroke(&stroke, Affine::IDENTITY, color, None, &rect);
                }
            }
            zcad_core::geometry::DimensionType::Diameter => {
                let p1 = dim.definition_point1;
                let p2 = dim.definition_point2;
                let center_point = Point2::new((p1.x + p2.x) / 2.0, (p1.y + p2.y) / 2.0);
                
                // 绘制直径线
                let diameter_line = KurboLine::new(to_screen(p1), to_screen(p2));
                scene.stroke(&stroke, Affine::IDENTITY, color, None, &diameter_line);
                
                // 绘制箭头
                Self::draw_arrow(scene, to_screen(center_point), to_screen(p1), color);
                Self::draw_arrow(scene, to_screen(center_point), to_screen(p2), color);
                
                // 绘制文本
                let text_pos = dim.get_text_position();
                let screen_pos = to_screen(text_pos);
                let text_height = dim.text_height * zoom;
                if text_height > 2.0 {
                    let text_content = dim.display_text().replace("%%C", "Ø");
                    let text_width = text_height * text_content.chars().count() as f64 * 0.6;
                    let rect = Rect::new(
                        screen_pos.x - text_width / 2.0,
                        screen_pos.y - text_height / 2.0,
                        screen_pos.x + text_width / 2.0,
                        screen_pos.y + text_height / 2.0,
                    );
                    scene.stroke(&stroke, Affine::IDENTITY, color, None, &rect);
                }
            }
            _ => {
                // 简化绘制
                let p1 = to_screen(dim.definition_point1);
                let p2 = to_screen(dim.definition_point2);
                let line_loc = to_screen(dim.line_location);
                
                let line1 = KurboLine::new(p1, line_loc);
                let line2 = KurboLine::new(p2, line_loc);
                scene.stroke(&stroke, Affine::IDENTITY, color, None, &line1);
                scene.stroke(&stroke, Affine::IDENTITY, color, None, &line2);
            }
        }
    }

    /// 绘制箭头
    fn draw_arrow(scene: &mut Scene, from: KurboPoint, to: KurboPoint, color: VelloColor) {
        let arrow_len = 10.0;
        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.001 {
            return;
        }
        let dir_x = dx / len;
        let dir_y = dy / len;
        
        let arrow1_end = KurboPoint::new(
            to.x - dir_x * arrow_len + (-dir_y) * arrow_len * 0.3,
            to.y - dir_y * arrow_len + dir_x * arrow_len * 0.3,
        );
        let arrow2_end = KurboPoint::new(
            to.x - dir_x * arrow_len - (-dir_y) * arrow_len * 0.3,
            to.y - dir_y * arrow_len - dir_x * arrow_len * 0.3,
        );
        
        let stroke = Stroke::new(1.0);
        let line1 = KurboLine::new(to, arrow1_end);
        let line2 = KurboLine::new(to, arrow2_end);
        scene.stroke(&stroke, Affine::IDENTITY, color, None, &line1);
        scene.stroke(&stroke, Affine::IDENTITY, color, None, &line2);
    }
}

impl Default for GpuRenderer {
    fn default() -> Self {
        Self::new()
    }
}
