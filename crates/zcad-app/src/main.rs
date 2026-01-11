//! ZCAD 主应用程序入口
//! 使用 eframe 作为应用框架，提供完整的 egui + wgpu 集成

use anyhow::Result;
use eframe::egui;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use zcad_core::entity::Entity;
use zcad_core::geometry::{Arc, Circle, Geometry, Line, Point, Polyline, Text};
use zcad_core::math::Point2;
use zcad_core::properties::Color;
use zcad_core::snap::SnapType;
use zcad_file::Document;
use zcad_ui::state::{DrawingTool, EditState, UiState};

/// ZCAD 应用程序
struct ZcadApp {
    document: Document,
    ui_state: UiState,
    
    // 视图状态
    camera_center: Point2,
    camera_zoom: f64,
    viewport_size: (f32, f32),
    
    // 文件操作状态
    pending_file_op: Option<FileOperation>,
    
    // 剪贴板（存储复制的几何体）
    clipboard: Vec<Geometry>,
}

/// 文件操作类型
#[derive(Debug, Clone)]
enum FileOperation {
    Open(std::path::PathBuf),
    Save(std::path::PathBuf),
}

impl Default for ZcadApp {
    fn default() -> Self {
        let mut app = Self {
            document: Document::new(),
            ui_state: UiState::default(),
            camera_center: Point2::new(250.0, 100.0),
            camera_zoom: 1.5,
            viewport_size: (800.0, 600.0),
            pending_file_op: None,
            clipboard: Vec::new(),
        };
        app.create_demo_content();
        app
    }
}

impl ZcadApp {
    fn create_demo_content(&mut self) {
        // 创建示例线条
        for i in 0..10 {
            let x = i as f64 * 50.0;
            let line = Line::new(Point2::new(x, 0.0), Point2::new(x, 200.0));
            let mut entity = Entity::new(Geometry::Line(line));
            entity.properties.color = Color::CYAN;
            self.document.add_entity(entity);
        }

        // 创建圆
        let circle = Circle::new(Point2::new(250.0, 100.0), 80.0);
        let mut entity = Entity::new(Geometry::Circle(circle));
        entity.properties.color = Color::YELLOW;
        self.document.add_entity(entity);

        // 创建矩形
        let rect = Polyline::from_points(
            [
                Point2::new(400.0, 50.0),
                Point2::new(550.0, 50.0),
                Point2::new(550.0, 150.0),
                Point2::new(400.0, 150.0),
            ],
            true,
        );
        let mut entity = Entity::new(Geometry::Polyline(rect));
        entity.properties.color = Color::GREEN;
        self.document.add_entity(entity);

        info!("Created {} demo entities", self.document.entity_count());
    }

    /// 世界坐标转屏幕坐标
    fn world_to_screen(&self, point: Point2, rect: &egui::Rect) -> egui::Pos2 {
        let center = rect.center();
        let x = center.x + ((point.x - self.camera_center.x) * self.camera_zoom) as f32;
        let y = center.y - ((point.y - self.camera_center.y) * self.camera_zoom) as f32; // Y轴翻转
        egui::Pos2::new(x, y)
    }

    /// 屏幕坐标转世界坐标
    fn screen_to_world(&self, pos: egui::Pos2, rect: &egui::Rect) -> Point2 {
        let center = rect.center();
        let x = self.camera_center.x + ((pos.x - center.x) as f64 / self.camera_zoom);
        let y = self.camera_center.y - ((pos.y - center.y) as f64 / self.camera_zoom); // Y轴翻转
        Point2::new(x, y)
    }

    /// 绘制网格
    fn draw_grid(&self, painter: &egui::Painter, rect: &egui::Rect) {
        if !self.ui_state.show_grid {
            return;
        }

        // 根据缩放级别调整网格间距
        let mut spacing = 50.0;
        while spacing * self.camera_zoom < 20.0 {
            spacing *= 5.0;
        }
        while spacing * self.camera_zoom > 200.0 {
            spacing /= 5.0;
        }

        // 计算可见范围
        let top_left = self.screen_to_world(rect.left_top(), rect);
        let bottom_right = self.screen_to_world(rect.right_bottom(), rect);

        let start_x = (top_left.x / spacing).floor() * spacing;
        let end_x = (bottom_right.x / spacing).ceil() * spacing;
        let start_y = (bottom_right.y / spacing).floor() * spacing;
        let end_y = (top_left.y / spacing).ceil() * spacing;

        let grid_color = egui::Color32::from_rgb(50, 50, 60);
        let axis_color = egui::Color32::from_rgb(80, 80, 100);

        // 绘制垂直线
        let mut x = start_x;
        while x <= end_x {
            let screen_x = self.world_to_screen(Point2::new(x, 0.0), rect).x;
            if screen_x >= rect.left() && screen_x <= rect.right() {
                let color = if x.abs() < 0.001 { axis_color } else { grid_color };
                painter.line_segment(
                    [egui::Pos2::new(screen_x, rect.top()), egui::Pos2::new(screen_x, rect.bottom())],
                    egui::Stroke::new(1.0, color),
                );
            }
            x += spacing;
        }

        // 绘制水平线
        let mut y = start_y;
        while y <= end_y {
            let screen_y = self.world_to_screen(Point2::new(0.0, y), rect).y;
            if screen_y >= rect.top() && screen_y <= rect.bottom() {
                let color = if y.abs() < 0.001 { axis_color } else { grid_color };
                painter.line_segment(
                    [egui::Pos2::new(rect.left(), screen_y), egui::Pos2::new(rect.right(), screen_y)],
                    egui::Stroke::new(1.0, color),
                );
            }
            y += spacing;
        }
    }

    /// 绘制几何体
    fn draw_geometry(&self, painter: &egui::Painter, rect: &egui::Rect, geometry: &Geometry, color: Color) {
        let stroke_color = egui::Color32::from_rgb(color.r, color.g, color.b);
        let stroke = egui::Stroke::new(1.5, stroke_color);

        match geometry {
            Geometry::Point(p) => {
                let screen = self.world_to_screen(p.position, rect);
                painter.circle_filled(screen, 3.0, stroke_color);
            }
            Geometry::Line(line) => {
                let start = self.world_to_screen(line.start, rect);
                let end = self.world_to_screen(line.end, rect);
                painter.line_segment([start, end], stroke);
            }
            Geometry::Circle(circle) => {
                let center = self.world_to_screen(circle.center, rect);
                let radius = (circle.radius * self.camera_zoom) as f32;
                painter.circle_stroke(center, radius, stroke);
            }
            Geometry::Arc(arc) => {
                // 简化：用线段近似弧线
                let segments = 32;
                let sweep = arc.sweep_angle();
                let angle_step = sweep / segments as f64;
                
                for i in 0..segments {
                    let a1 = arc.start_angle + i as f64 * angle_step;
                    let a2 = arc.start_angle + (i + 1) as f64 * angle_step;
                    
                    let p1 = Point2::new(
                        arc.center.x + arc.radius * a1.cos(),
                        arc.center.y + arc.radius * a1.sin(),
                    );
                    let p2 = Point2::new(
                        arc.center.x + arc.radius * a2.cos(),
                        arc.center.y + arc.radius * a2.sin(),
                    );
                    
                    let s1 = self.world_to_screen(p1, rect);
                    let s2 = self.world_to_screen(p2, rect);
                    painter.line_segment([s1, s2], stroke);
                }
            }
            Geometry::Polyline(polyline) => {
                if polyline.vertices.len() < 2 {
                    return;
                }
                
                for i in 0..polyline.segment_count() {
                    let v1 = &polyline.vertices[i];
                    let v2 = &polyline.vertices[(i + 1) % polyline.vertices.len()];
                    
                    let s1 = self.world_to_screen(v1.point, rect);
                    let s2 = self.world_to_screen(v2.point, rect);
                    painter.line_segment([s1, s2], stroke);
                }
            }
            Geometry::Text(text) => {
                self.draw_text(painter, rect, text, color);
            }
        }
    }

    /// 绘制文本
    fn draw_text(&self, painter: &egui::Painter, rect: &egui::Rect, text: &Text, color: Color) {
        let screen_pos = self.world_to_screen(text.position, rect);
        let screen_height = (text.height * self.camera_zoom) as f32;
        
        // 限制最小显示字号
        if screen_height < 4.0 {
            // 太小时显示一个占位符点
            painter.circle_filled(screen_pos, 2.0, egui::Color32::from_rgb(color.r, color.g, color.b));
            return;
        }

        let font_id = egui::FontId::proportional(screen_height.clamp(8.0, 200.0));
        let text_color = egui::Color32::from_rgb(color.r, color.g, color.b);
        
        // 创建文本绘制任务
        let galley = painter.layout_no_wrap(
            text.content.clone(),
            font_id,
            text_color,
        );
        
        // 计算对齐偏移
        let text_width = galley.rect.width();
        let align_offset = match text.alignment {
            zcad_core::geometry::TextAlignment::Left => 0.0,
            zcad_core::geometry::TextAlignment::Center => -text_width / 2.0,
            zcad_core::geometry::TextAlignment::Right => -text_width,
        };
        
        // Y轴翻转：egui的Y轴向下，CAD的Y轴向上
        // 文本的position是基线位置，需要调整
        let draw_pos = egui::Pos2::new(
            screen_pos.x + align_offset,
            screen_pos.y - screen_height, // 向上偏移一个字高
        );
        
        // 如果有旋转，需要使用变换
        if text.rotation.abs() > 0.001 {
            // egui不直接支持旋转文本，这里简化处理
            // 可以通过mesh来实现，但这里先用简单方式
            painter.galley(draw_pos, galley, text_color);
        } else {
            painter.galley(draw_pos, galley, text_color);
        }
    }

    /// 绘制十字光标
    fn draw_crosshair(&self, painter: &egui::Painter, rect: &egui::Rect, world_pos: Point2) {
        let screen = self.world_to_screen(world_pos, rect);
        let size = 15.0;
        let color = egui::Color32::WHITE;
        let stroke = egui::Stroke::new(1.0, color);

        painter.line_segment(
            [egui::Pos2::new(screen.x - size, screen.y), egui::Pos2::new(screen.x + size, screen.y)],
            stroke,
        );
        painter.line_segment(
            [egui::Pos2::new(screen.x, screen.y - size), egui::Pos2::new(screen.x, screen.y + size)],
            stroke,
        );
    }

    /// 绘制捕捉标记
    fn draw_snap_marker(&self, painter: &egui::Painter, rect: &egui::Rect, snap_type: SnapType, world_pos: Point2) {
        let screen = self.world_to_screen(world_pos, rect);
        let size = 8.0;
        let stroke = egui::Stroke::new(2.0, egui::Color32::YELLOW);

        match snap_type {
            SnapType::Endpoint => {
                // 方形标记
                painter.rect_stroke(
                    egui::Rect::from_center_size(screen, egui::vec2(size * 2.0, size * 2.0)),
                    egui::CornerRadius::ZERO,
                    stroke,
                    egui::StrokeKind::Outside,
                );
            }
            SnapType::Midpoint => {
                // 三角形标记
                let points = [
                    egui::Pos2::new(screen.x, screen.y - size),
                    egui::Pos2::new(screen.x - size, screen.y + size),
                    egui::Pos2::new(screen.x + size, screen.y + size),
                ];
                painter.add(egui::Shape::closed_line(points.to_vec(), stroke));
            }
            SnapType::Center => {
                // 圆形标记
                painter.circle_stroke(screen, size, stroke);
            }
            SnapType::Intersection => {
                // X形标记
                painter.line_segment(
                    [egui::Pos2::new(screen.x - size, screen.y - size), egui::Pos2::new(screen.x + size, screen.y + size)],
                    stroke,
                );
                painter.line_segment(
                    [egui::Pos2::new(screen.x - size, screen.y + size), egui::Pos2::new(screen.x + size, screen.y - size)],
                    stroke,
                );
            }
            SnapType::Perpendicular => {
                // 垂直标记（直角符号）
                painter.line_segment(
                    [egui::Pos2::new(screen.x - size, screen.y), egui::Pos2::new(screen.x, screen.y)],
                    stroke,
                );
                painter.line_segment(
                    [egui::Pos2::new(screen.x, screen.y), egui::Pos2::new(screen.x, screen.y + size)],
                    stroke,
                );
            }
            SnapType::Tangent => {
                // 切点标记（圆+线）
                painter.circle_stroke(screen, size * 0.6, stroke);
                painter.line_segment(
                    [egui::Pos2::new(screen.x - size, screen.y + size), egui::Pos2::new(screen.x + size, screen.y - size)],
                    stroke,
                );
            }
            SnapType::Nearest => {
                // 最近点标记（沙漏形）
                let half = size * 0.7;
                painter.line_segment(
                    [egui::Pos2::new(screen.x - half, screen.y - size), egui::Pos2::new(screen.x + half, screen.y - size)],
                    stroke,
                );
                painter.line_segment(
                    [egui::Pos2::new(screen.x - half, screen.y - size), egui::Pos2::new(screen.x + half, screen.y + size)],
                    stroke,
                );
                painter.line_segment(
                    [egui::Pos2::new(screen.x + half, screen.y - size), egui::Pos2::new(screen.x - half, screen.y + size)],
                    stroke,
                );
                painter.line_segment(
                    [egui::Pos2::new(screen.x - half, screen.y + size), egui::Pos2::new(screen.x + half, screen.y + size)],
                    stroke,
                );
            }
            SnapType::Grid => {
                // 网格点标记（小+形）
                let small = size * 0.5;
                painter.line_segment(
                    [egui::Pos2::new(screen.x - small, screen.y), egui::Pos2::new(screen.x + small, screen.y)],
                    stroke,
                );
                painter.line_segment(
                    [egui::Pos2::new(screen.x, screen.y - small), egui::Pos2::new(screen.x, screen.y + small)],
                    stroke,
                );
            }
            SnapType::Quadrant => {
                // 象限点标记（菱形）
                let points = [
                    egui::Pos2::new(screen.x, screen.y - size),
                    egui::Pos2::new(screen.x + size, screen.y),
                    egui::Pos2::new(screen.x, screen.y + size),
                    egui::Pos2::new(screen.x - size, screen.y),
                ];
                painter.add(egui::Shape::closed_line(points.to_vec(), stroke));
            }
        }
    }

    /// 绘制正交辅助线
    fn draw_ortho_guides(&self, painter: &egui::Painter, rect: &egui::Rect, reference: Point2) {
        let screen = self.world_to_screen(reference, rect);
        let guide_color = egui::Color32::from_rgba_unmultiplied(0, 255, 255, 80); // 半透明青色
        let stroke = egui::Stroke::new(1.0, guide_color);

        // 水平辅助线
        painter.line_segment(
            [egui::Pos2::new(rect.left(), screen.y), egui::Pos2::new(rect.right(), screen.y)],
            stroke,
        );

        // 垂直辅助线
        painter.line_segment(
            [egui::Pos2::new(screen.x, rect.top()), egui::Pos2::new(screen.x, rect.bottom())],
            stroke,
        );
    }

    /// 更新捕捉点
    fn update_snap(&mut self) {
        // 获取当前视图内的实体
        let entities: Vec<&Entity> = self.document.all_entities().collect();

        // 获取参考点（绘图状态下的起始点）
        let reference_point = match &self.ui_state.edit_state {
            EditState::Drawing { points, .. } if !points.is_empty() => Some(points[0]),
            _ => None,
        };

        // 查找捕捉点
        let mut snap = self.ui_state.snap_state.engine_mut().find_snap_point(
            self.ui_state.mouse_world_pos,
            &entities,
            self.camera_zoom,
            reference_point,
        );

        // 特殊处理：绘制多段线时，检查是否接近起点（用于闭合）
        if let EditState::Drawing { tool: DrawingTool::Polyline, points } = &self.ui_state.edit_state {
            if points.len() >= 2 {
                let start_point = points[0];
                let world_tolerance = self.ui_state.snap_state.config().tolerance / self.camera_zoom;
                let dist_to_start = (self.ui_state.mouse_world_pos - start_point).norm();
                
                if dist_to_start <= world_tolerance {
                    // 比当前捕捉点更近，或者没有当前捕捉点
                    let should_use_start = match &snap {
                        Some(existing) => dist_to_start < existing.distance,
                        None => true,
                    };
                    
                    if should_use_start {
                        snap = Some(zcad_core::snap::SnapPoint::new(
                            start_point,
                            zcad_core::snap::SnapType::Endpoint,
                            None,
                            dist_to_start,
                        ));
                    }
                }
            }
        }

        // 同样处理圆弧：可以捕捉到第一个点
        if let EditState::Drawing { tool: DrawingTool::Arc, points } = &self.ui_state.edit_state {
            if !points.is_empty() {
                let first_point = points[0];
                let world_tolerance = self.ui_state.snap_state.config().tolerance / self.camera_zoom;
                let dist_to_first = (self.ui_state.mouse_world_pos - first_point).norm();
                
                if dist_to_first <= world_tolerance {
                    let should_use_first = match &snap {
                        Some(existing) => dist_to_first < existing.distance,
                        None => true,
                    };
                    
                    if should_use_first {
                        snap = Some(zcad_core::snap::SnapPoint::new(
                            first_point,
                            zcad_core::snap::SnapType::Endpoint,
                            None,
                            dist_to_first,
                        ));
                    }
                }
            }
        }

        self.ui_state.snap_state.current_snap = snap;
    }

    /// 应用正交约束
    /// 
    /// 将目标点约束到从参考点出发的水平或垂直方向
    fn apply_ortho_constraint(&self, reference: Point2, target: Point2) -> Point2 {
        if !self.ui_state.ortho_mode {
            return target;
        }

        let dx = (target.x - reference.x).abs();
        let dy = (target.y - reference.y).abs();

        if dx > dy {
            // 水平方向更近，约束到水平线
            Point2::new(target.x, reference.y)
        } else {
            // 垂直方向更近，约束到垂直线
            Point2::new(reference.x, target.y)
        }
    }

    /// 获取有效的绘图点（应用捕捉和正交约束）
    fn get_effective_draw_point(&self) -> Point2 {
        let base_point = self.ui_state.effective_point();

        // 如果正在绘图且有参考点，应用正交约束
        if let EditState::Drawing { points, .. } = &self.ui_state.edit_state {
            if !points.is_empty() && self.ui_state.ortho_mode {
                let reference = *points.last().unwrap();
                return self.apply_ortho_constraint(reference, base_point);
            }
        }

        base_point
    }

    /// 绘制预览
    fn draw_preview(&self, painter: &egui::Painter, rect: &egui::Rect) {
        if let EditState::Drawing { tool, points } = &self.ui_state.edit_state {
            if points.is_empty() {
                return;
            }
            
            let preview_color = Color::from_hex(0xFF00FF);
            // 使用捕捉点和正交约束
            let mouse_pos = self.get_effective_draw_point();

            match tool {
                DrawingTool::Line => {
                    let line = Line::new(*points.last().unwrap(), mouse_pos);
                    self.draw_geometry(painter, rect, &Geometry::Line(line), preview_color);
                }
                DrawingTool::Circle => {
                    let radius = (mouse_pos - points[0]).norm();
                    if radius > 0.01 {
                        let circle = Circle::new(points[0], radius);
                        self.draw_geometry(painter, rect, &Geometry::Circle(circle), preview_color);
                    }
                }
                DrawingTool::Rectangle => {
                    let p1 = points[0];
                    let rect_geom = Polyline::from_points(
                        [
                            Point2::new(p1.x, p1.y),
                            Point2::new(mouse_pos.x, p1.y),
                            Point2::new(mouse_pos.x, mouse_pos.y),
                            Point2::new(p1.x, mouse_pos.y),
                        ],
                        true,
                    );
                    self.draw_geometry(painter, rect, &Geometry::Polyline(rect_geom), preview_color);
                }
                DrawingTool::Arc => {
                    if points.len() == 1 {
                        // 只有起点，画到鼠标的直线预览
                        let line = Line::new(points[0], mouse_pos);
                        self.draw_geometry(painter, rect, &Geometry::Line(line), preview_color);
                    } else if points.len() == 2 {
                        // 有两个点，尝试预览圆弧
                        if let Some(arc) = Arc::from_three_points(points[0], points[1], mouse_pos) {
                            self.draw_geometry(painter, rect, &Geometry::Arc(arc), preview_color);
                        } else {
                            // 共线，画两条线
                            let line1 = Line::new(points[0], points[1]);
                            let line2 = Line::new(points[1], mouse_pos);
                            self.draw_geometry(painter, rect, &Geometry::Line(line1), preview_color);
                            self.draw_geometry(painter, rect, &Geometry::Line(line2), preview_color);
                        }
                    }
                }
                DrawingTool::Polyline => {
                    // 绘制已有的线段
                    for i in 0..points.len().saturating_sub(1) {
                        let line = Line::new(points[i], points[i + 1]);
                        self.draw_geometry(painter, rect, &Geometry::Line(line), preview_color);
                    }
                    // 绘制到鼠标的预览线段
                    if let Some(&last) = points.last() {
                        let line = Line::new(last, mouse_pos);
                        self.draw_geometry(painter, rect, &Geometry::Line(line), preview_color);
                    }
                }
                _ => {}
            }
        }
        
        // 移动预览
        if let EditState::MovingEntities { start_pos, entity_ids } = &self.ui_state.edit_state {
            let preview_color = Color::from_hex(0x00FFFF); // 青色
            let mouse_pos = self.ui_state.mouse_world_pos;
            let offset = mouse_pos - *start_pos;
            
            for id in entity_ids {
                if let Some(entity) = self.document.get_entity(id) {
                    let mut preview_geom = entity.geometry.clone();
                    self.apply_offset_to_geometry_preview(&mut preview_geom, offset);
                    self.draw_geometry(painter, rect, &preview_geom, preview_color);
                }
            }
        }
    }
    
    /// 对几何体应用偏移（预览用，不修改原始）
    fn apply_offset_to_geometry_preview(&self, geometry: &mut Geometry, offset: zcad_core::math::Vector2) {
        match geometry {
            Geometry::Point(p) => {
                p.position = p.position + offset;
            }
            Geometry::Line(l) => {
                l.start = l.start + offset;
                l.end = l.end + offset;
            }
            Geometry::Circle(c) => {
                c.center = c.center + offset;
            }
            Geometry::Arc(a) => {
                a.center = a.center + offset;
            }
            Geometry::Polyline(pl) => {
                for v in &mut pl.vertices {
                    v.point = v.point + offset;
                }
            }
            Geometry::Text(t) => {
                t.position = t.position + offset;
            }
        }
    }

    /// 处理左键点击
    fn handle_left_click(&mut self) {
        // 使用捕捉点和正交约束
        let world_pos = self.get_effective_draw_point();

        match &self.ui_state.edit_state {
            EditState::Idle => match self.ui_state.current_tool {
                DrawingTool::Line => {
                    self.ui_state.edit_state = EditState::Drawing {
                        tool: DrawingTool::Line,
                        points: vec![world_pos],
                    };
                    self.ui_state.status_message = "指定下一点:".to_string();
                }
                DrawingTool::Circle => {
                    self.ui_state.edit_state = EditState::Drawing {
                        tool: DrawingTool::Circle,
                        points: vec![world_pos],
                    };
                    self.ui_state.status_message = "指定半径:".to_string();
                }
                DrawingTool::Rectangle => {
                    self.ui_state.edit_state = EditState::Drawing {
                        tool: DrawingTool::Rectangle,
                        points: vec![world_pos],
                    };
                    self.ui_state.status_message = "指定对角点:".to_string();
                }
                DrawingTool::Arc => {
                    self.ui_state.edit_state = EditState::Drawing {
                        tool: DrawingTool::Arc,
                        points: vec![world_pos],
                    };
                    self.ui_state.status_message = "圆弧: 指定第二点:".to_string();
                }
                DrawingTool::Polyline => {
                    self.ui_state.edit_state = EditState::Drawing {
                        tool: DrawingTool::Polyline,
                        points: vec![world_pos],
                    };
                    self.ui_state.status_message = "多段线: 指定下一点 (右键结束):".to_string();
                }
                DrawingTool::Point => {
                    // 点直接创建，不需要绘图状态
                    let point = Point::from_point2(world_pos);
                    let entity = Entity::new(Geometry::Point(point));
                    self.document.add_entity(entity);
                    self.ui_state.status_message = "点已创建".to_string();
                }
                DrawingTool::Text => {
                    // 进入文本输入模式
                    self.ui_state.edit_state = EditState::TextInput {
                        position: world_pos,
                        content: String::new(),
                        height: 10.0, // 默认文本高度
                    };
                    self.ui_state.status_message = "输入文本内容，按 Enter 确认:".to_string();
                }
                DrawingTool::Select => {
                    let hits = self.document.query_point(&world_pos, 5.0 / self.camera_zoom);
                    self.ui_state.clear_selection();
                    if let Some(entity) = hits.first() {
                        self.ui_state.add_to_selection(entity.id);
                        self.ui_state.status_message = format!("已选择: {}", entity.geometry.type_name());
                    } else {
                        self.ui_state.status_message.clear();
                    }
                }
                DrawingTool::None => {}
            },
            EditState::Drawing { tool, points } => {
                let tool = *tool;
                let mut new_points = points.clone();
                new_points.push(world_pos);

                match tool {
                    DrawingTool::Line => {
                        if new_points.len() >= 2 {
                            let line = Line::new(new_points[0], new_points[1]);
                            let entity = Entity::new(Geometry::Line(line));
                            self.document.add_entity(entity);
                            self.ui_state.edit_state = EditState::Drawing {
                                tool: DrawingTool::Line,
                                points: vec![new_points[1]],
                            };
                            self.ui_state.status_message = "直线已创建。下一点:".to_string();
                        }
                    }
                    DrawingTool::Circle => {
                        if new_points.len() >= 2 {
                            let radius = (new_points[1] - new_points[0]).norm();
                            let circle = Circle::new(new_points[0], radius);
                            let entity = Entity::new(Geometry::Circle(circle));
                            self.document.add_entity(entity);
                            self.ui_state.edit_state = EditState::Idle;
                            self.ui_state.status_message = "圆已创建".to_string();
                        }
                    }
                    DrawingTool::Rectangle => {
                        if new_points.len() >= 2 {
                            let p1 = new_points[0];
                            let p2 = new_points[1];
                            let rect = Polyline::from_points(
                                [
                                    Point2::new(p1.x, p1.y),
                                    Point2::new(p2.x, p1.y),
                                    Point2::new(p2.x, p2.y),
                                    Point2::new(p1.x, p2.y),
                                ],
                                true,
                            );
                            let entity = Entity::new(Geometry::Polyline(rect));
                            self.document.add_entity(entity);
                            self.ui_state.edit_state = EditState::Idle;
                            self.ui_state.status_message = "矩形已创建".to_string();
                        }
                    }
                    DrawingTool::Arc => {
                        // 三点圆弧：起点、经过点、终点
                        if new_points.len() == 2 {
                            // 第二个点
                            self.ui_state.edit_state = EditState::Drawing {
                                tool: DrawingTool::Arc,
                                points: new_points,
                            };
                            self.ui_state.status_message = "圆弧: 指定终点:".to_string();
                        } else if new_points.len() >= 3 {
                            // 三个点，创建圆弧
                            if let Some(arc) = Arc::from_three_points(
                                new_points[0],
                                new_points[1],
                                new_points[2],
                            ) {
                                let entity = Entity::new(Geometry::Arc(arc));
                                self.document.add_entity(entity);
                                self.ui_state.status_message = "圆弧已创建".to_string();
                            } else {
                                self.ui_state.status_message = "无法创建圆弧（三点共线）".to_string();
                            }
                            self.ui_state.edit_state = EditState::Idle;
                        }
                    }
                    DrawingTool::Polyline => {
                        // 检查是否点击了起点（闭合多段线）
                        if new_points.len() >= 3 {
                            let start = new_points[0];
                            let current = new_points[new_points.len() - 1];
                            let tolerance = 0.001; // 很小的容差，因为捕捉已经对齐了
                            
                            if (current - start).norm() < tolerance {
                                // 点击了起点，创建闭合多段线
                                new_points.pop(); // 移除重复的终点
                                let polyline = Polyline::from_points(new_points, true); // closed = true
                                let entity = Entity::new(Geometry::Polyline(polyline));
                                self.document.add_entity(entity);
                                self.ui_state.edit_state = EditState::Idle;
                                self.ui_state.status_message = "闭合多段线已创建".to_string();
                                return;
                            }
                        }
                        
                        // 否则继续添加点
                        self.ui_state.edit_state = EditState::Drawing {
                            tool: DrawingTool::Polyline,
                            points: new_points,
                        };
                        self.ui_state.status_message = "多段线: 指定下一点 (右键结束, 点击起点闭合):".to_string();
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    /// 处理右键点击（结束多段线等）
    fn handle_right_click(&mut self) {
        if let EditState::Drawing { tool, points } = &self.ui_state.edit_state {
            match tool {
                DrawingTool::Polyline => {
                    if points.len() >= 2 {
                        // 创建多段线
                        let polyline = Polyline::from_points(points.clone(), false);
                        let entity = Entity::new(Geometry::Polyline(polyline));
                        self.document.add_entity(entity);
                        self.ui_state.status_message = format!("多段线已创建 ({} 个点)", points.len());
                    } else {
                        self.ui_state.status_message = "取消".to_string();
                    }
                    self.ui_state.edit_state = EditState::Idle;
                }
                _ => {
                    // 其他工具右键取消
                    self.ui_state.cancel();
                }
            }
        } else {
            self.ui_state.cancel();
        }
    }

    /// 处理双击（编辑文本）
    fn handle_double_click(&mut self) {
        let world_pos = self.ui_state.mouse_world_pos;
        let hits = self.document.query_point(&world_pos, 5.0 / self.camera_zoom);
        
        if let Some(entity) = hits.first() {
            if let Geometry::Text(text) = &entity.geometry {
                // 进入文本编辑模式
                self.ui_state.edit_state = EditState::TextEdit {
                    entity_id: entity.id,
                    position: text.position,
                    content: text.content.clone(),
                    height: text.height,
                };
                self.ui_state.status_message = "编辑文本，点击确定保存:".to_string();
            }
        }
    }

    /// 处理拖拽
    fn handle_drag(&mut self, _delta: egui::Vec2) {
        match &self.ui_state.edit_state {
            EditState::MovingEntities { .. } => {
                // 移动过程中不需要额外处理，预览会自动更新
            }
            EditState::Idle if self.ui_state.current_tool == DrawingTool::Select => {
                // 选中状态下开始拖拽 = 开始移动
                if !self.ui_state.selected_entities.is_empty() {
                    let world_pos = self.ui_state.mouse_world_pos;
                    self.ui_state.edit_state = EditState::MovingEntities {
                        start_pos: world_pos,
                        entity_ids: self.ui_state.selected_entities.clone(),
                    };
                    self.ui_state.status_message = "移动实体，释放鼠标确认:".to_string();
                }
            }
            _ => {}
        }
    }

    /// 处理拖拽结束
    fn handle_drag_end(&mut self) {
        if let EditState::MovingEntities { start_pos, entity_ids } = &self.ui_state.edit_state {
            let end_pos = self.ui_state.mouse_world_pos;
            let offset = end_pos - *start_pos;
            
            // 应用移动
            for id in entity_ids.clone() {
                if let Some(entity) = self.document.get_entity(&id) {
                    let mut new_entity = entity.clone();
                    self.apply_offset_to_geometry(&mut new_entity.geometry, offset);
                    self.document.update_entity(&id, new_entity);
                }
            }
            
            self.ui_state.status_message = format!("已移动 {} 个实体", entity_ids.len());
            self.ui_state.edit_state = EditState::Idle;
        }
    }

    /// 对几何体应用偏移
    fn apply_offset_to_geometry(&self, geometry: &mut Geometry, offset: zcad_core::math::Vector2) {
        match geometry {
            Geometry::Point(p) => {
                p.position = p.position + offset;
            }
            Geometry::Line(l) => {
                l.start = l.start + offset;
                l.end = l.end + offset;
            }
            Geometry::Circle(c) => {
                c.center = c.center + offset;
            }
            Geometry::Arc(a) => {
                a.center = a.center + offset;
            }
            Geometry::Polyline(pl) => {
                for v in &mut pl.vertices {
                    v.point = v.point + offset;
                }
            }
            Geometry::Text(t) => {
                t.position = t.position + offset;
            }
        }
    }

    /// 复制选中的实体
    fn copy_selected(&mut self) {
        self.clipboard.clear();
        for id in &self.ui_state.selected_entities {
            if let Some(entity) = self.document.get_entity(id) {
                self.clipboard.push(entity.geometry.clone());
            }
        }
        if !self.clipboard.is_empty() {
            self.ui_state.status_message = format!("已复制 {} 个实体", self.clipboard.len());
        }
    }

    /// 粘贴实体
    fn paste_entities(&mut self) {
        if self.clipboard.is_empty() {
            return;
        }
        
        let mouse_pos = self.ui_state.mouse_world_pos;
        self.ui_state.clear_selection();
        
        for geom in &self.clipboard {
            let mut new_geom = geom.clone();
            // 计算原始中心到鼠标位置的偏移
            let bbox = new_geom.bounding_box();
            let center = Point2::new(
                (bbox.min.x + bbox.max.x) / 2.0,
                (bbox.min.y + bbox.max.y) / 2.0,
            );
            let offset = mouse_pos - center;
            self.apply_offset_to_geometry(&mut new_geom, offset);
            
            let entity = Entity::new(new_geom);
            let id = self.document.add_entity(entity);
            self.ui_state.add_to_selection(id);
        }
        
        self.ui_state.status_message = format!("已粘贴 {} 个实体", self.clipboard.len());
    }

    /// 缩放到适合视图
    fn zoom_to_fit(&mut self) {
        if let Some(bounds) = self.document.bounds() {
            self.camera_center = Point2::new(
                (bounds.min.x + bounds.max.x) / 2.0,
                (bounds.min.y + bounds.max.y) / 2.0,
            );
            
            let width = bounds.max.x - bounds.min.x;
            let height = bounds.max.y - bounds.min.y;
            
            let zoom_x = (self.viewport_size.0 as f64 - 100.0) / width.max(1.0);
            let zoom_y = (self.viewport_size.1 as f64 - 100.0) / height.max(1.0);
            
            self.camera_zoom = zoom_x.min(zoom_y).clamp(0.01, 100.0);
        }
    }

    /// 打开文件对话框 - 打开文件
    fn show_open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ZCAD Files", &["zcad"])
            .add_filter("DXF Files", &["dxf"])
            .add_filter("All Files", &["*"])
            .set_title("打开文件")
            .pick_file()
        {
            self.pending_file_op = Some(FileOperation::Open(path));
        }
    }

    /// 打开文件对话框 - 保存文件
    fn show_save_dialog(&mut self) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("ZCAD Files", &["zcad"])
            .add_filter("DXF Files", &["dxf"])
            .set_title("保存文件");

        // 如果已有文件名，使用它
        if let Some(path) = self.document.file_path() {
            if let Some(file_name) = path.file_name() {
                dialog = dialog.set_file_name(file_name.to_string_lossy().as_ref());
            }
        }

        if let Some(path) = dialog.save_file() {
            self.pending_file_op = Some(FileOperation::Save(path));
        }
    }

    /// 处理文件操作
    fn process_file_operations(&mut self) {
        if let Some(op) = self.pending_file_op.take() {
            match op {
                FileOperation::Open(path) => {
                    match Document::open(&path) {
                        Ok(doc) => {
                            self.document = doc;
                            self.ui_state.clear_selection();
                            self.zoom_to_fit();
                            self.ui_state.status_message = 
                                format!("已打开: {}", path.display());
                            info!("Opened file: {}", path.display());
                        }
                        Err(e) => {
                            self.ui_state.status_message = 
                                format!("打开失败: {}", e);
                            tracing::error!("Failed to open file: {}", e);
                        }
                    }
                }
                FileOperation::Save(path) => {
                    match self.document.save_as(&path) {
                        Ok(_) => {
                            self.ui_state.status_message = 
                                format!("已保存: {}", path.display());
                            info!("Saved file: {}", path.display());
                        }
                        Err(e) => {
                            self.ui_state.status_message = 
                                format!("保存失败: {}", e);
                            tracing::error!("Failed to save file: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// 快速保存（已有路径）
    fn quick_save(&mut self) {
        if self.document.file_path().is_some() {
            match self.document.save() {
                Ok(_) => {
                    self.ui_state.status_message = "已保存".to_string();
                    info!("Quick saved file");
                }
                Err(e) => {
                    self.ui_state.status_message = format!("保存失败: {}", e);
                    tracing::error!("Failed to quick save: {}", e);
                }
            }
        } else {
            // 没有路径，显示另存为对话框
            self.show_save_dialog();
        }
    }
}

impl eframe::App for ZcadApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // 处理文件操作
        self.process_file_operations();
        
        // 更新窗口标题
        let title = if let Some(path) = self.document.file_path() {
            let modified = if self.document.is_modified() { "*" } else { "" };
            format!("ZCAD - {}{}", path.display(), modified)
        } else {
            let modified = if self.document.is_modified() { "*" } else { "" };
            format!("ZCAD - Untitled{}", modified)
        };
        ctx.send_viewport_cmd(egui::ViewportCommand::Title(title));
        
        // 深色主题
        ctx.set_visuals(egui::Visuals::dark());

        // UI状态快照
        let current_tool = self.ui_state.current_tool;
        let ortho = self.ui_state.ortho_mode;
        let grid = self.ui_state.show_grid;
        let status = self.ui_state.status_message.clone();
        let mouse_world = self.ui_state.mouse_world_pos;
        let entity_count = self.document.entity_count();
        let selected_count = self.ui_state.selected_entities.len();

        // 选中实体信息
        let selected_info: Option<(String, Vec<String>)> = if selected_count == 1 {
            self.document.get_entity(&self.ui_state.selected_entities[0]).map(|e| {
                let name = e.geometry.type_name().to_string();
                let props: Vec<String> = match &e.geometry {
                    Geometry::Line(l) => vec![
                        format!("起点: ({:.2}, {:.2})", l.start.x, l.start.y),
                        format!("终点: ({:.2}, {:.2})", l.end.x, l.end.y),
                        format!("长度: {:.3}", l.length()),
                    ],
                    Geometry::Circle(c) => vec![
                        format!("圆心: ({:.2}, {:.2})", c.center.x, c.center.y),
                        format!("半径: {:.3}", c.radius),
                    ],
                    Geometry::Polyline(p) => vec![
                        format!("顶点数: {}", p.vertex_count()),
                        format!("长度: {:.3}", p.length()),
                    ],
                    Geometry::Text(t) => vec![
                        format!("内容: {}", t.content),
                        format!("位置: ({:.2}, {:.2})", t.position.x, t.position.y),
                        format!("高度: {:.3}", t.height),
                    ],
                    #[allow(unreachable_patterns)]
                    _ => vec![],
                };
                (name, props)
            })
        } else { None };

        // 图层信息
        let layers_info: Vec<_> = self.document.layers.all_layers().iter()
            .map(|l| (l.name.clone(), l.color.r, l.color.g, l.color.b, l.name == self.document.layers.current_layer().name))
            .collect();

        // ===== 顶部菜单 =====
        #[allow(deprecated)]
        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("文件", |ui| {
                    if ui.button("📄 新建 (Ctrl+N)").clicked() {
                        self.document = Document::new();
                        self.ui_state.clear_selection();
                        self.ui_state.status_message = "新文档".to_string();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("📂 打开 (Ctrl+O)").clicked() {
                        self.show_open_dialog();
                        ui.close();
                    }
                    if ui.button("💾 保存 (Ctrl+S)").clicked() {
                        self.quick_save();
                        ui.close();
                    }
                    if ui.button("💾 另存为 (Ctrl+Shift+S)").clicked() {
                        self.show_save_dialog();
                        ui.close();
                    }
                    ui.separator();
                    if ui.button("🚪 退出").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.menu_button("编辑", |ui| {
                    if ui.button("🗑 删除 (Del)").clicked() {
                        for id in self.ui_state.selected_entities.clone() {
                            self.document.remove_entity(&id);
                        }
                        self.ui_state.clear_selection();
                        ui.close();
                    }
                });
                ui.menu_button("视图", |ui| {
                    if ui.button("📐 缩放至全部 (Z)").clicked() {
                        self.zoom_to_fit();
                        ui.close();
                    }
                    if ui.button(format!("{} 网格 (G)", if grid { "☑" } else { "☐" })).clicked() {
                        self.ui_state.show_grid = !self.ui_state.show_grid;
                        ui.close();
                    }
                    if ui.button(format!("{} 正交 (F8)", if ortho { "☑" } else { "☐" })).clicked() {
                        self.ui_state.ortho_mode = !self.ui_state.ortho_mode;
                        ui.close();
                    }
                });
                ui.menu_button("绘图", |ui| {
                    if ui.button("╱ 直线 (L)").clicked() {
                        self.ui_state.set_tool(DrawingTool::Line);
                        ui.close();
                    }
                    if ui.button("○ 圆 (C)").clicked() {
                        self.ui_state.set_tool(DrawingTool::Circle);
                        ui.close();
                    }
                    if ui.button("▭ 矩形 (R)").clicked() {
                        self.ui_state.set_tool(DrawingTool::Rectangle);
                        ui.close();
                    }
                    if ui.button("A 文本 (T)").clicked() {
                        self.ui_state.set_tool(DrawingTool::Text);
                        ui.close();
                    }
                });
            });
        });

        // ===== 工具栏 =====
        egui::TopBottomPanel::top("toolbar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.selectable_label(current_tool == DrawingTool::Select, "⬚ 选择").clicked() {
                    self.ui_state.set_tool(DrawingTool::Select);
                }
                ui.separator();
                if ui.selectable_label(current_tool == DrawingTool::Line, "╱ 直线").clicked() {
                    self.ui_state.set_tool(DrawingTool::Line);
                }
                if ui.selectable_label(current_tool == DrawingTool::Circle, "○ 圆").clicked() {
                    self.ui_state.set_tool(DrawingTool::Circle);
                }
                if ui.selectable_label(current_tool == DrawingTool::Rectangle, "▭ 矩形").clicked() {
                    self.ui_state.set_tool(DrawingTool::Rectangle);
                }
                if ui.selectable_label(current_tool == DrawingTool::Arc, "◠ 圆弧").clicked() {
                    self.ui_state.set_tool(DrawingTool::Arc);
                }
                if ui.selectable_label(current_tool == DrawingTool::Polyline, "⌇ 多段线").clicked() {
                    self.ui_state.set_tool(DrawingTool::Polyline);
                }
                if ui.selectable_label(current_tool == DrawingTool::Text, "A 文本").clicked() {
                    self.ui_state.set_tool(DrawingTool::Text);
                }
                ui.separator();
                if ui.button("🗑").on_hover_text("删除选中").clicked() {
                    for id in self.ui_state.selected_entities.clone() {
                        self.document.remove_entity(&id);
                    }
                    self.ui_state.clear_selection();
                }
                ui.separator();
                if ui.selectable_label(ortho, "⊥").on_hover_text("正交模式 (F8)").clicked() {
                    self.ui_state.ortho_mode = !self.ui_state.ortho_mode;
                }
                if ui.selectable_label(grid, "#").on_hover_text("网格 (G)").clicked() {
                    self.ui_state.show_grid = !self.ui_state.show_grid;
                }
                if ui.button("⊞").on_hover_text("缩放至全部 (Z)").clicked() {
                    self.zoom_to_fit();
                }
            });
        });

        // ===== 状态栏 =====
        // 捕捉信息快照
        let snap_enabled = self.ui_state.snap_state.enabled;
        let snap_info = self.ui_state.snap_state.current_snap.as_ref().map(|s| {
            (s.snap_type.name().to_string(), s.point)
        });
        let effective_pos = self.ui_state.effective_point();

        egui::TopBottomPanel::bottom("status").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(&status);
                
                // 捕捉状态显示
                if let Some((snap_name, _)) = &snap_info {
                    ui.separator();
                    ui.colored_label(egui::Color32::YELLOW, format!("⊕ {}", snap_name));
                }
                
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(format!("X:{:>8.2} Y:{:>8.2}", effective_pos.x, effective_pos.y));
                    ui.separator();
                    ui.label(format!("实体: {}", entity_count));
                    if selected_count > 0 {
                        ui.separator();
                        ui.label(format!("选中: {}", selected_count));
                    }
                    ui.separator();
                    // 捕捉开关
                    let snap_text = if snap_enabled { "🔗 捕捉" } else { "🔗" };
                    if ui.selectable_label(snap_enabled, snap_text).on_hover_text("对象捕捉 (F3)").clicked() {
                        self.ui_state.snap_state.enabled = !self.ui_state.snap_state.enabled;
                    }
                });
            });
        });

        // ===== 右侧面板 - 图层 =====
        egui::SidePanel::right("layers").default_width(150.0).show(ctx, |ui| {
            ui.heading("图层");
            ui.separator();
            for (name, r, g, b, is_current) in &layers_info {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().rect_filled(rect, 1.0, egui::Color32::from_rgb(*r, *g, *b));
                    let txt = if *is_current { egui::RichText::new(name).strong() } else { egui::RichText::new(name) };
                    ui.label(txt);
                });
            }
        });

        // ===== 左侧面板 - 属性 =====
        egui::SidePanel::left("props").default_width(170.0).show(ctx, |ui| {
            ui.heading("属性");
            ui.separator();
            if let Some((type_name, props)) = &selected_info {
                ui.label(format!("类型: {}", type_name));
                ui.separator();
                for p in props { ui.label(p); }
            } else if selected_count > 1 {
                ui.label(format!("{} 个对象", selected_count));
            } else {
                ui.label(format!("工具: {}", current_tool.name()));
            }
            ui.separator();
            ui.label(format!("X: {:.4}", mouse_world.x));
            ui.label(format!("Y: {:.4}", mouse_world.y));
        });

        // ===== 文本输入对话框 =====
        let mut text_action: Option<bool> = None; // Some(true) = 确认, Some(false) = 取消
        let text_input_data = if let EditState::TextInput { position, content, height } = &self.ui_state.edit_state {
            Some((*position, content.clone(), *height))
        } else {
            None
        };
        
        if let Some((pos, mut content, mut height)) = text_input_data {
            egui::Window::new("输入文本")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("内容:");
                        let response = ui.text_edit_singleline(&mut content);
                        // 自动聚焦到输入框
                        response.request_focus();
                    });
                    ui.horizontal(|ui| {
                        ui.label("高度:");
                        ui.add(egui::DragValue::new(&mut height)
                            .speed(0.5)
                            .range(1.0..=1000.0));
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("  确定  ").clicked() {
                            text_action = Some(true);
                        }
                        if ui.button("  取消  ").clicked() {
                            text_action = Some(false);
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(format!("位置: ({:.2}, {:.2})", pos.x, pos.y));
                    ui.label("提示: 点击确定或取消按钮");
                });
            
            // 更新编辑状态中的内容
            self.ui_state.edit_state = EditState::TextInput {
                position: pos,
                content,
                height,
            };
        }
        
        // 处理文本确认/取消
        match text_action {
            Some(true) => {
                if let EditState::TextInput { position, content, height } = &self.ui_state.edit_state {
                    if !content.is_empty() {
                        let text = Text::new(*position, content.clone(), *height);
                        let entity = Entity::new(Geometry::Text(text));
                        self.document.add_entity(entity);
                        self.ui_state.status_message = "文本已创建".to_string();
                    }
                }
                self.ui_state.edit_state = EditState::Idle;
            }
            Some(false) => {
                self.ui_state.edit_state = EditState::Idle;
                self.ui_state.status_message = "取消".to_string();
            }
            None => {}
        }

        // ===== 文本编辑对话框（编辑现有文本）=====
        let mut text_edit_action: Option<bool> = None;
        let text_edit_data = if let EditState::TextEdit { entity_id, position, content, height } = &self.ui_state.edit_state {
            Some((*entity_id, *position, content.clone(), *height))
        } else {
            None
        };
        
        if let Some((entity_id, pos, mut content, mut height)) = text_edit_data {
            egui::Window::new("编辑文本")
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("内容:");
                        let response = ui.text_edit_singleline(&mut content);
                        response.request_focus();
                    });
                    ui.horizontal(|ui| {
                        ui.label("高度:");
                        ui.add(egui::DragValue::new(&mut height)
                            .speed(0.5)
                            .range(1.0..=1000.0));
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("  保存  ").clicked() {
                            text_edit_action = Some(true);
                        }
                        if ui.button("  取消  ").clicked() {
                            text_edit_action = Some(false);
                        }
                    });
                    ui.add_space(4.0);
                    ui.label(format!("位置: ({:.2}, {:.2})", pos.x, pos.y));
                });
            
            // 更新编辑状态中的内容
            self.ui_state.edit_state = EditState::TextEdit {
                entity_id,
                position: pos,
                content,
                height,
            };
        }
        
        // 处理文本编辑确认/取消
        match text_edit_action {
            Some(true) => {
                if let EditState::TextEdit { entity_id, position, content, height } = &self.ui_state.edit_state {
                    if !content.is_empty() {
                        let text = Text::new(*position, content.clone(), *height);
                        if let Some(entity) = self.document.get_entity(entity_id) {
                            let mut new_entity = entity.clone();
                            new_entity.geometry = Geometry::Text(text);
                            self.document.update_entity(entity_id, new_entity);
                            self.ui_state.status_message = "文本已更新".to_string();
                        }
                    }
                }
                self.ui_state.edit_state = EditState::Idle;
            }
            Some(false) => {
                self.ui_state.edit_state = EditState::Idle;
                self.ui_state.status_message = "取消".to_string();
            }
            None => {}
        }

        // ===== 中央绘图区域 =====
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE.fill(egui::Color32::from_rgb(30, 30, 46)))
            .show(ctx, |ui| {
                let available_rect = ui.available_rect_before_wrap();
                self.viewport_size = (available_rect.width(), available_rect.height());
                
                let (response, painter) = ui.allocate_painter(available_rect.size(), egui::Sense::click_and_drag());
                let rect = response.rect;

                // 处理鼠标位置
                if let Some(hover_pos) = response.hover_pos() {
                    self.ui_state.mouse_world_pos = self.screen_to_world(hover_pos, &rect);
                    // 更新捕捉点
                    self.update_snap();
                }

                // 处理滚轮缩放
                let scroll_delta = ui.input(|i| i.raw_scroll_delta);
                if scroll_delta.y.abs() > 0.0 && response.hovered() {
                    let zoom_factor = if scroll_delta.y > 0.0 { 1.1 } else { 0.9 };
                    
                    // 缩放时保持鼠标位置不变
                    if let Some(hover_pos) = response.hover_pos() {
                        let world_before = self.screen_to_world(hover_pos, &rect);
                        self.camera_zoom *= zoom_factor;
                        self.camera_zoom = self.camera_zoom.clamp(0.01, 100.0);
                        let world_after = self.screen_to_world(hover_pos, &rect);
                        self.camera_center.x += world_before.x - world_after.x;
                        self.camera_center.y += world_before.y - world_after.y;
                    }
                }

                // 处理中键平移
                if response.dragged_by(egui::PointerButton::Middle) {
                    let delta = response.drag_delta();
                    self.camera_center.x -= (delta.x as f64) / self.camera_zoom;
                    self.camera_center.y += (delta.y as f64) / self.camera_zoom;
                }

                // 处理双击（编辑文本）
                if response.double_clicked_by(egui::PointerButton::Primary) {
                    self.handle_double_click();
                }
                // 处理左键点击
                else if response.clicked_by(egui::PointerButton::Primary) {
                    self.handle_left_click();
                }

                // 处理拖拽移动
                if response.dragged_by(egui::PointerButton::Primary) {
                    self.handle_drag(response.drag_delta());
                }
                if response.drag_stopped_by(egui::PointerButton::Primary) {
                    self.handle_drag_end();
                }

                // 处理右键（结束多段线或取消）
                if response.clicked_by(egui::PointerButton::Secondary) {
                    self.handle_right_click();
                }

                // 处理键盘快捷键（仅在非文本输入状态下）
                let is_text_input = matches!(self.ui_state.edit_state, EditState::TextInput { .. } | EditState::TextEdit { .. });
                if !is_text_input {
                    ui.input(|i| {
                        // 文件操作
                        if i.modifiers.command && i.key_pressed(egui::Key::N) {
                            self.document = Document::new();
                            self.ui_state.clear_selection();
                            self.ui_state.status_message = "新文档".to_string();
                        }
                        if i.modifiers.command && i.key_pressed(egui::Key::O) {
                            self.show_open_dialog();
                        }
                        if i.modifiers.command && i.key_pressed(egui::Key::S) {
                            if i.modifiers.shift {
                                self.show_save_dialog();
                            } else {
                                self.quick_save();
                            }
                        }
                        
                        // 编辑操作
                        if i.key_pressed(egui::Key::Escape) {
                            self.ui_state.cancel();
                        }
                        if i.key_pressed(egui::Key::Delete) || i.key_pressed(egui::Key::Backspace) {
                            for id in self.ui_state.selected_entities.clone() {
                                self.document.remove_entity(&id);
                            }
                            self.ui_state.clear_selection();
                        }
                        // 复制 Ctrl+C
                        if i.modifiers.command && i.key_pressed(egui::Key::C) {
                            self.copy_selected();
                        }
                        // 粘贴 Ctrl+V
                        if i.modifiers.command && i.key_pressed(egui::Key::V) {
                            self.paste_entities();
                        }
                        // 移动命令 M
                        if i.key_pressed(egui::Key::M) && !self.ui_state.selected_entities.is_empty() {
                            let world_pos = self.ui_state.mouse_world_pos;
                            self.ui_state.edit_state = EditState::MovingEntities {
                                start_pos: world_pos,
                                entity_ids: self.ui_state.selected_entities.clone(),
                            };
                            self.ui_state.status_message = "移动: 指定目标点或拖动鼠标:".to_string();
                        }
                        
                        // 绘图工具
                        if i.key_pressed(egui::Key::L) {
                            self.ui_state.set_tool(DrawingTool::Line);
                        }
                        if i.key_pressed(egui::Key::C) {
                            self.ui_state.set_tool(DrawingTool::Circle);
                        }
                        if i.key_pressed(egui::Key::R) {
                            self.ui_state.set_tool(DrawingTool::Rectangle);
                        }
                        if i.key_pressed(egui::Key::Space) {
                            self.ui_state.set_tool(DrawingTool::Select);
                        }
                        
                        // 视图操作
                        if i.key_pressed(egui::Key::Z) {
                            self.zoom_to_fit();
                        }
                        if i.key_pressed(egui::Key::G) {
                            self.ui_state.show_grid = !self.ui_state.show_grid;
                        }
                        if i.key_pressed(egui::Key::F3) {
                            self.ui_state.snap_state.enabled = !self.ui_state.snap_state.enabled;
                            let status = if self.ui_state.snap_state.enabled { "捕捉已启用" } else { "捕捉已禁用" };
                            self.ui_state.status_message = status.to_string();
                        }
                        if i.key_pressed(egui::Key::F8) {
                            self.ui_state.ortho_mode = !self.ui_state.ortho_mode;
                            let status = if self.ui_state.ortho_mode { "正交模式已启用" } else { "正交模式已禁用" };
                            self.ui_state.status_message = status.to_string();
                        }
                        // 圆弧快捷键
                        if i.key_pressed(egui::Key::A) {
                            self.ui_state.set_tool(DrawingTool::Arc);
                        }
                        // 多段线快捷键
                        if i.key_pressed(egui::Key::P) {
                            self.ui_state.set_tool(DrawingTool::Polyline);
                        }
                        // 文本快捷键
                        if i.key_pressed(egui::Key::T) {
                            self.ui_state.set_tool(DrawingTool::Text);
                        }
                    });
                }

                // ===== 绘制 =====
                // 绘制网格
                self.draw_grid(&painter, &rect);

                // 绘制所有实体
                for entity in self.document.all_entities() {
                    let color = if self.ui_state.selected_entities.contains(&entity.id) {
                        Color::from_hex(0x00FF00)
                    } else if entity.properties.color.is_by_layer() {
                        self.document.layers.get_layer_by_id(entity.layer_id)
                            .map(|l| l.color).unwrap_or(Color::WHITE)
                    } else {
                        entity.properties.color
                    };
                    self.draw_geometry(&painter, &rect, &entity.geometry, color);
                }

                // 绘制预览
                self.draw_preview(&painter, &rect);

                // 绘制正交辅助线
                if self.ui_state.ortho_mode {
                    if let EditState::Drawing { points, .. } = &self.ui_state.edit_state {
                        if let Some(&reference) = points.last() {
                            self.draw_ortho_guides(&painter, &rect, reference);
                        }
                    }
                }

                // 绘制捕捉标记
                if let Some(ref snap) = self.ui_state.snap_state.current_snap {
                    if self.ui_state.snap_state.enabled {
                        self.draw_snap_marker(&painter, &rect, snap.snap_type, snap.point);
                    }
                }

                // 绘制十字光标（使用捕捉点如果有的话）
                if response.hovered() {
                    let cursor_pos = self.ui_state.effective_point();
                    self.draw_crosshair(&painter, &rect, cursor_pos);
                }
            });

        // 请求持续重绘（实现动画效果）
        ctx.request_repaint();
    }
}

/// 设置中文字体支持
fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试从系统加载中文字体
    let font_paths = [
        // macOS
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        // Windows
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ];
    
    for path in font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "chinese".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(font_data)),
            );
            
            // 将中文字体添加到字体族（放在最前面以优先使用）
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese".to_owned());
            fonts.families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "chinese".to_owned());
            
            info!("Loaded Chinese font from: {}", path);
            break;
        }
    }
    
    ctx.set_fonts(fonts);
}

fn main() -> Result<()> {
    // 初始化日志
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder().with_max_level(Level::INFO).finish()
    )?;
    
    info!("Starting ZCAD...");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("ZCAD"),
        ..Default::default()
    };

    eframe::run_native(
        "ZCAD",
        native_options,
        Box::new(|cc| {
            // 加载中文字体
            setup_chinese_fonts(&cc.egui_ctx);
            Ok(Box::new(ZcadApp::default()))
        }),
    ).map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

    Ok(())
}