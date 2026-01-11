//! 几何图元定义
//!
//! 支持的基本图元：
//! - 点 (Point)
//! - 线段 (Line)
//! - 圆 (Circle)
//! - 圆弧 (Arc)
//! - 多段线 (Polyline)
//! - 文本 (Text)
//! - 椭圆 (Ellipse)
//! - 样条曲线 (Spline)
//! - 填充 (Hatch)
//! - 引线 (Leader)

use crate::math::{BoundingBox2, Point2, Vector2, EPSILON};
use serde::{Deserialize, Serialize};

/// 几何类型枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Geometry {
    Point(Point),
    Line(Line),
    Circle(Circle),
    Arc(Arc),
    Polyline(Polyline),
    Text(Text),
    Dimension(Dimension),
    Ellipse(Ellipse),
    Spline(Spline),
    Hatch(Hatch),
    Leader(Leader),
}

impl Geometry {
    /// 获取几何的包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        match self {
            Geometry::Point(p) => p.bounding_box(),
            Geometry::Line(l) => l.bounding_box(),
            Geometry::Circle(c) => c.bounding_box(),
            Geometry::Arc(a) => a.bounding_box(),
            Geometry::Polyline(pl) => pl.bounding_box(),
            Geometry::Text(t) => t.bounding_box(),
            Geometry::Dimension(d) => d.bounding_box(),
            Geometry::Ellipse(e) => e.bounding_box(),
            Geometry::Spline(s) => s.bounding_box(),
            Geometry::Hatch(h) => h.bounding_box(),
            Geometry::Leader(l) => l.bounding_box(),
        }
    }

    /// 获取几何的类型名称
    pub fn type_name(&self) -> &'static str {
        match self {
            Geometry::Point(_) => "Point",
            Geometry::Line(_) => "Line",
            Geometry::Circle(_) => "Circle",
            Geometry::Arc(_) => "Arc",
            Geometry::Polyline(_) => "Polyline",
            Geometry::Text(_) => "Text",
            Geometry::Dimension(_) => "Dimension",
            Geometry::Ellipse(_) => "Ellipse",
            Geometry::Spline(_) => "Spline",
            Geometry::Hatch(_) => "Hatch",
            Geometry::Leader(_) => "Leader",
        }
    }

    /// 检查点是否在几何上（考虑容差）
    pub fn contains_point(&self, point: &Point2, tolerance: f64) -> bool {
        match self {
            Geometry::Point(p) => (p.position - point).norm() <= tolerance,
            Geometry::Line(l) => l.distance_to_point(point) <= tolerance,
            Geometry::Circle(c) => (c.distance_to_point(point)).abs() <= tolerance,
            Geometry::Arc(a) => a.distance_to_point(point) <= tolerance,
            Geometry::Polyline(pl) => pl.distance_to_point(point) <= tolerance,
            Geometry::Text(t) => t.contains_point(point, tolerance),
            Geometry::Dimension(d) => d.contains_point(point, tolerance),
            Geometry::Ellipse(e) => e.distance_to_point(point) <= tolerance,
            Geometry::Spline(s) => s.distance_to_point(point) <= tolerance,
            Geometry::Hatch(h) => h.contains_point(point, tolerance),
            Geometry::Leader(l) => l.distance_to_point(point) <= tolerance,
        }
    }
}

/// 点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Point {
    pub position: Point2,
}

impl Point {
    pub fn new(x: f64, y: f64) -> Self {
        Self {
            position: Point2::new(x, y),
        }
    }

    pub fn from_point2(position: Point2) -> Self {
        Self { position }
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::new(self.position, self.position)
    }
}

/// 线段
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Line {
    pub start: Point2,
    pub end: Point2,
}

impl Line {
    pub fn new(start: Point2, end: Point2) -> Self {
        Self { start, end }
    }

    /// 计算线段长度
    pub fn length(&self) -> f64 {
        (self.end - self.start).norm()
    }

    /// 计算线段方向向量（单位向量）
    pub fn direction(&self) -> Vector2 {
        (self.end - self.start).normalize()
    }

    /// 计算线段中点
    pub fn midpoint(&self) -> Point2 {
        Point2::new(
            (self.start.x + self.end.x) / 2.0,
            (self.start.y + self.end.y) / 2.0,
        )
    }

    /// 计算点到线段的距离
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        let v = self.end - self.start;
        let w = point - self.start;

        let c1 = w.dot(&v);
        if c1 <= 0.0 {
            return (point - self.start).norm();
        }

        let c2 = v.dot(&v);
        if c2 <= c1 {
            return (point - self.end).norm();
        }

        let b = c1 / c2;
        let pb = self.start + v * b;
        (point - pb).norm()
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::from_points([self.start, self.end])
    }
}

/// 圆
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Circle {
    pub center: Point2,
    pub radius: f64,
}

impl Circle {
    pub fn new(center: Point2, radius: f64) -> Self {
        Self { center, radius }
    }

    /// 计算周长
    pub fn circumference(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }

    /// 计算面积
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius * self.radius
    }

    /// 计算点到圆的距离（负值表示在圆内）
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        (point - self.center).norm() - self.radius
    }

    /// 获取圆上指定角度的点
    pub fn point_at_angle(&self, angle: f64) -> Point2 {
        Point2::new(
            self.center.x + self.radius * angle.cos(),
            self.center.y + self.radius * angle.sin(),
        )
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::new(
            Point2::new(self.center.x - self.radius, self.center.y - self.radius),
            Point2::new(self.center.x + self.radius, self.center.y + self.radius),
        )
    }
}

/// 圆弧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arc {
    pub center: Point2,
    pub radius: f64,
    /// 起始角度（弧度）
    pub start_angle: f64,
    /// 终止角度（弧度）
    pub end_angle: f64,
}

impl Arc {
    pub fn new(center: Point2, radius: f64, start_angle: f64, end_angle: f64) -> Self {
        Self {
            center,
            radius,
            start_angle,
            end_angle,
        }
    }

    /// 从三点创建圆弧
    pub fn from_three_points(p1: Point2, p2: Point2, p3: Point2) -> Option<Self> {
        // 计算圆心
        let d = 2.0
            * (p1.x * (p2.y - p3.y) + p2.x * (p3.y - p1.y) + p3.x * (p1.y - p2.y));

        if d.abs() < EPSILON {
            return None; // 三点共线
        }

        let ux = ((p1.x * p1.x + p1.y * p1.y) * (p2.y - p3.y)
            + (p2.x * p2.x + p2.y * p2.y) * (p3.y - p1.y)
            + (p3.x * p3.x + p3.y * p3.y) * (p1.y - p2.y))
            / d;
        let uy = ((p1.x * p1.x + p1.y * p1.y) * (p3.x - p2.x)
            + (p2.x * p2.x + p2.y * p2.y) * (p1.x - p3.x)
            + (p3.x * p3.x + p3.y * p3.y) * (p2.x - p1.x))
            / d;

        let center = Point2::new(ux, uy);
        let radius = (p1 - center).norm();

        let start_angle = (p1.y - center.y).atan2(p1.x - center.x);
        let end_angle = (p3.y - center.y).atan2(p3.x - center.x);

        Some(Self::new(center, radius, start_angle, end_angle))
    }

    /// 计算弧长
    pub fn length(&self) -> f64 {
        self.sweep_angle().abs() * self.radius
    }

    /// 计算扫过的角度
    pub fn sweep_angle(&self) -> f64 {
        let mut sweep = self.end_angle - self.start_angle;
        while sweep < 0.0 {
            sweep += 2.0 * std::f64::consts::PI;
        }
        while sweep > 2.0 * std::f64::consts::PI {
            sweep -= 2.0 * std::f64::consts::PI;
        }
        sweep
    }

    /// 获取起点
    pub fn start_point(&self) -> Point2 {
        Point2::new(
            self.center.x + self.radius * self.start_angle.cos(),
            self.center.y + self.radius * self.start_angle.sin(),
        )
    }

    /// 获取终点
    pub fn end_point(&self) -> Point2 {
        Point2::new(
            self.center.x + self.radius * self.end_angle.cos(),
            self.center.y + self.radius * self.end_angle.sin(),
        )
    }

    /// 计算点到圆弧的距离
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        let angle = (point.y - self.center.y).atan2(point.x - self.center.x);

        // 检查角度是否在弧的范围内
        if self.contains_angle(angle) {
            ((point - self.center).norm() - self.radius).abs()
        } else {
            // 返回到端点的最小距离
            let d1 = (point - self.start_point()).norm();
            let d2 = (point - self.end_point()).norm();
            d1.min(d2)
        }
    }

    /// 检查角度是否在弧的范围内
    fn contains_angle(&self, angle: f64) -> bool {
        let mut a = angle;
        let mut start = self.start_angle;
        let mut end = self.end_angle;

        // 归一化到 [0, 2π)
        while a < 0.0 {
            a += 2.0 * std::f64::consts::PI;
        }
        while start < 0.0 {
            start += 2.0 * std::f64::consts::PI;
        }
        while end < 0.0 {
            end += 2.0 * std::f64::consts::PI;
        }

        if start <= end {
            a >= start && a <= end
        } else {
            a >= start || a <= end
        }
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        let mut bbox = BoundingBox2::from_points([self.start_point(), self.end_point()]);

        // 检查象限点
        let pi = std::f64::consts::PI;
        for angle in [0.0, pi / 2.0, pi, 3.0 * pi / 2.0] {
            if self.contains_angle(angle) {
                bbox.expand_to_include(&Point2::new(
                    self.center.x + self.radius * angle.cos(),
                    self.center.y + self.radius * angle.sin(),
                ));
            }
        }

        bbox
    }
}

/// 多段线顶点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolylineVertex {
    pub point: Point2,
    /// 凸度（bulge）- 用于弧线段，0表示直线
    pub bulge: f64,
}

impl PolylineVertex {
    pub fn new(point: Point2) -> Self {
        Self { point, bulge: 0.0 }
    }

    pub fn with_bulge(point: Point2, bulge: f64) -> Self {
        Self { point, bulge }
    }
}

/// 多段线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Polyline {
    pub vertices: Vec<PolylineVertex>,
    /// 是否闭合
    pub closed: bool,
}

impl Polyline {
    pub fn new(vertices: Vec<PolylineVertex>, closed: bool) -> Self {
        Self { vertices, closed }
    }

    /// 从点列表创建（所有顶点都是直线连接）
    pub fn from_points(points: impl IntoIterator<Item = Point2>, closed: bool) -> Self {
        Self {
            vertices: points
                .into_iter()
                .map(PolylineVertex::new)
                .collect(),
            closed,
        }
    }

    /// 顶点数量
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// 线段数量
    pub fn segment_count(&self) -> usize {
        if self.vertices.len() < 2 {
            return 0;
        }
        if self.closed {
            self.vertices.len()
        } else {
            self.vertices.len() - 1
        }
    }

    /// 计算总长度
    pub fn length(&self) -> f64 {
        if self.vertices.len() < 2 {
            return 0.0;
        }

        let mut total = 0.0;
        for i in 0..self.segment_count() {
            let v1 = &self.vertices[i];
            let v2 = &self.vertices[(i + 1) % self.vertices.len()];

            if v1.bulge.abs() < EPSILON {
                // 直线段
                total += (v2.point - v1.point).norm();
            } else {
                // 弧线段
                total += self.arc_segment_length(v1, v2);
            }
        }
        total
    }

    /// 计算弧线段长度
    fn arc_segment_length(&self, v1: &PolylineVertex, v2: &PolylineVertex) -> f64 {
        let chord = (v2.point - v1.point).norm();
        let s = chord / 2.0;
        let bulge = v1.bulge.abs();
        let radius = s * (1.0 + bulge * bulge) / (2.0 * bulge);
        let angle = 4.0 * bulge.atan();
        radius * angle.abs()
    }

    /// 计算点到多段线的距离
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        if self.vertices.is_empty() {
            return f64::MAX;
        }
        if self.vertices.len() == 1 {
            return (point - self.vertices[0].point).norm();
        }

        let mut min_dist = f64::MAX;
        for i in 0..self.segment_count() {
            let v1 = &self.vertices[i];
            let v2 = &self.vertices[(i + 1) % self.vertices.len()];

            let dist = if v1.bulge.abs() < EPSILON {
                // 直线段
                let line = Line::new(v1.point, v2.point);
                line.distance_to_point(point)
            } else {
                // 弧线段 - 简化处理，使用直线近似
                let line = Line::new(v1.point, v2.point);
                line.distance_to_point(point)
            };

            min_dist = min_dist.min(dist);
        }
        min_dist
    }

    pub fn bounding_box(&self) -> BoundingBox2 {
        if self.vertices.is_empty() {
            return BoundingBox2::empty();
        }
        BoundingBox2::from_points(self.vertices.iter().map(|v| v.point))
    }
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TextAlignment {
    /// 左对齐（默认）
    #[default]
    Left,
    /// 居中对齐
    Center,
    /// 右对齐
    Right,
}

/// 文本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    /// 插入点
    pub position: Point2,
    /// 文本内容
    pub content: String,
    /// 文本高度
    pub height: f64,
    /// 旋转角度（弧度）
    pub rotation: f64,
    /// 对齐方式
    pub alignment: TextAlignment,
}

impl Text {
    /// 创建新的文本对象
    pub fn new(position: Point2, content: impl Into<String>, height: f64) -> Self {
        Self {
            position,
            content: content.into(),
            height,
            rotation: 0.0,
            alignment: TextAlignment::Left,
        }
    }

    /// 设置旋转角度
    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置对齐方式
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// 估算文本宽度（简化计算，假设每个字符宽度约为高度的0.6倍）
    pub fn estimated_width(&self) -> f64 {
        // 对于中文字符，宽度接近高度；对于英文，约为高度的0.6倍
        // 这里采用简化的混合估算
        let char_count = self.content.chars().count();
        let cjk_count = self.content.chars().filter(|c| Self::is_cjk(*c)).count();
        let ascii_count = char_count - cjk_count;
        
        (cjk_count as f64 * self.height) + (ascii_count as f64 * self.height * 0.6)
    }

    /// 检查是否是CJK字符
    fn is_cjk(c: char) -> bool {
        matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{F900}'..='\u{FAFF}')
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        let width = self.estimated_width();
        let height = self.height;
        
        // 根据对齐方式计算基准点
        let base_x = match self.alignment {
            TextAlignment::Left => self.position.x,
            TextAlignment::Center => self.position.x - width / 2.0,
            TextAlignment::Right => self.position.x - width,
        };
        
        // 简化处理：忽略旋转的包围盒计算
        if self.rotation.abs() < EPSILON {
            BoundingBox2::new(
                Point2::new(base_x, self.position.y),
                Point2::new(base_x + width, self.position.y + height),
            )
        } else {
            // 带旋转的包围盒：计算四个角点
            let corners = [
                Point2::new(0.0, 0.0),
                Point2::new(width, 0.0),
                Point2::new(width, height),
                Point2::new(0.0, height),
            ];
            
            let cos_r = self.rotation.cos();
            let sin_r = self.rotation.sin();
            
            let rotated: Vec<Point2> = corners.iter().map(|p| {
                let rx = p.x * cos_r - p.y * sin_r + base_x;
                let ry = p.x * sin_r + p.y * cos_r + self.position.y;
                Point2::new(rx, ry)
            }).collect();
            
            BoundingBox2::from_points(rotated)
        }
    }

    /// 检查点是否在文本包围盒内
    pub fn contains_point(&self, point: &Point2, tolerance: f64) -> bool {
        let bbox = self.bounding_box();
        let expanded = BoundingBox2::new(
            Point2::new(bbox.min.x - tolerance, bbox.min.y - tolerance),
            Point2::new(bbox.max.x + tolerance, bbox.max.y + tolerance),
        );
        expanded.contains(point)
    }
}

/// 标注类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DimensionType {
    /// 对齐标注 (Aligned) - 默认
    #[default]
    Aligned,
    /// 线性标注 (Linear) - 水平或垂直
    Linear,
    /// 半径标注
    Radius,
    /// 直径标注
    Diameter,
    /// 角度标注
    Angular,
    /// 弧长标注
    ArcLength,
    /// 坐标标注
    Ordinate,
}

/// 尺寸标注
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    /// 第一个测量点
    pub definition_point1: Point2,
    /// 第二个测量点
    pub definition_point2: Point2,
    /// 标注线位置点 (决定标注线的高度/距离)
    pub line_location: Point2,
    /// 标注类型
    pub dim_type: DimensionType,
    /// 覆盖文本 (如果为空则显示测量值)
    pub text_override: Option<String>,
    /// 文本高度
    pub text_height: f64,
    /// 文本位置 (如果为None，则自动计算默认位置)
    pub text_position: Option<Point2>,
}

impl Dimension {
    pub fn new(p1: Point2, p2: Point2, location: Point2) -> Self {
        Self {
            definition_point1: p1,
            definition_point2: p2,
            line_location: location,
            dim_type: DimensionType::Aligned,
            text_override: None,
            text_height: 10.0, // 默认高度
            text_position: None,
        }
    }

    /// 获取文本的实际显示位置（如果未设置，则计算默认位置）
    pub fn get_text_position(&self) -> Point2 {
        if let Some(pos) = self.text_position {
            return pos;
        }
        self.default_text_position()
    }

    /// 计算默认文本位置
    pub fn default_text_position(&self) -> Point2 {
        match self.dim_type {
            DimensionType::Aligned | DimensionType::Linear => {
                let dir = (self.definition_point2 - self.definition_point1).normalize();
                let perp = Vector2::new(-dir.y, dir.x);
                let v_loc = self.line_location - self.definition_point1;
                let dist = v_loc.dot(&perp);
                
                // 确保signum不为0，如果为0，默认向上
                let sign = if dist.abs() < EPSILON { 1.0 } else { dist.signum() };
                
                // 默认偏移量：dist + 0.8 * text_height * sign
                let total_dist = dist + sign * (self.text_height * 0.8);
                let offset_vec = perp * total_dist;
                
                self.definition_point1 + (self.definition_point2 - self.definition_point1) * 0.5 + offset_vec
            }
            DimensionType::Radius | DimensionType::Diameter => {
                // 默认位置就是 line_location (用户点击的位置)
                self.line_location
            }
            DimensionType::Angular | DimensionType::ArcLength => {
                // 角度标注：文本位于角平分线上
                let v1 = (self.definition_point2 - self.definition_point1).normalize();
                let v2 = (self.line_location - self.definition_point1).normalize();
                let bisector = (v1 + v2).normalize();
                let radius = (self.definition_point2 - self.definition_point1).norm() * 0.6;
                self.definition_point1 + bisector * radius
            }
            DimensionType::Ordinate => {
                // 坐标标注：文本位于 line_location
                self.line_location
            }
        }
    }

    /// 获取文本包围盒
    pub fn text_bounding_box(&self) -> BoundingBox2 {
        let pos = self.get_text_position();
        let content = self.display_text();
        let text = Text::new(pos, content, self.text_height)
            .with_alignment(TextAlignment::Center); // 标注文本通常是居中绘制
        text.bounding_box()
    }

    /// 获取测量值
    pub fn measurement(&self) -> f64 {
        match self.dim_type {
            DimensionType::Aligned => (self.definition_point2 - self.definition_point1).norm(),
            DimensionType::Linear => {
                // 线性标注通常根据line_location的位置决定是水平还是垂直
                // 这里简化处理：根据两点的主要差异方向决定
                let dx = (self.definition_point2.x - self.definition_point1.x).abs();
                let dy = (self.definition_point2.y - self.definition_point1.y).abs();
                if dx >= dy { dx } else { dy }
            }
            DimensionType::Radius => {
                // 对于半径标注，p1是圆心，p2是圆上一点
                (self.definition_point2 - self.definition_point1).norm()
            }
            DimensionType::Diameter => {
                // 对于直径标注，p1是圆心，p2是圆上一点，测量值为半径 * 2
                (self.definition_point2 - self.definition_point1).norm() * 2.0
            }
            DimensionType::Angular => {
                // 角度标注：p1 是顶点，p2 是第一条边上的点，line_location 是第二条边上的点
                let v1 = self.definition_point2 - self.definition_point1;
                let v2 = self.line_location - self.definition_point1;
                let dot = v1.x * v2.x + v1.y * v2.y;
                let cross = v1.x * v2.y - v1.y * v2.x;
                cross.atan2(dot).abs().to_degrees()
            }
            DimensionType::ArcLength => {
                // 弧长标注：p1 是圆心，p2 是起点，line_location 是终点
                let radius = (self.definition_point2 - self.definition_point1).norm();
                let v1 = self.definition_point2 - self.definition_point1;
                let v2 = self.line_location - self.definition_point1;
                let angle = (v1.x * v2.y - v1.y * v2.x).atan2(v1.x * v2.x + v1.y * v2.y).abs();
                radius * angle
            }
            DimensionType::Ordinate => {
                // 坐标标注：显示 x 或 y 坐标
                // 根据 line_location 相对于 definition_point1 的位置决定
                let dx = (self.line_location.x - self.definition_point1.x).abs();
                let dy = (self.line_location.y - self.definition_point1.y).abs();
                if dx > dy {
                    self.definition_point1.x
                } else {
                    self.definition_point1.y
                }
            }
        }
    }

    /// 获取显示的文本
    pub fn display_text(&self) -> String {
        if let Some(text) = &self.text_override {
            text.clone()
        } else {
            let val = self.measurement();
            match self.dim_type {
                DimensionType::Radius => format!("R{:.2}", val),
                DimensionType::Diameter => format!("%%C{:.2}", val), // %%C 是 CAD 中直径符号的转义
                DimensionType::Angular => format!("{:.1}°", val),
                DimensionType::ArcLength => format!("⌒{:.2}", val),
                DimensionType::Ordinate => format!("{:.2}", val),
                _ => format!("{:.2}", val),
            }
        }
    }

    /// 计算包围盒 (简化估算)
    pub fn bounding_box(&self) -> BoundingBox2 {
        BoundingBox2::from_points([
            self.definition_point1,
            self.definition_point2,
            self.line_location,
        ])
    }

    /// 检查点是否在标注上 (简化：检查是否在包围盒内)
    pub fn contains_point(&self, point: &Point2, tolerance: f64) -> bool {
        let bbox = self.bounding_box();
        let expanded = BoundingBox2::new(
            Point2::new(bbox.min.x - tolerance, bbox.min.y - tolerance),
            Point2::new(bbox.max.x + tolerance, bbox.max.y + tolerance),
        );
        expanded.contains(point)
    }
}

impl Polyline {
    /// 爆炸为独立的线段/圆弧
    ///
    /// 这是我们要做好的功能 - 智能爆炸，只生成需要的几何体
    pub fn explode(&self) -> Vec<Geometry> {
        if self.vertices.len() < 2 {
            return vec![];
        }

        let mut result = Vec::with_capacity(self.segment_count());

        for i in 0..self.segment_count() {
            let v1 = &self.vertices[i];
            let v2 = &self.vertices[(i + 1) % self.vertices.len()];

            if v1.bulge.abs() < EPSILON {
                // 直线段
                result.push(Geometry::Line(Line::new(v1.point, v2.point)));
            } else {
                // 弧线段
                if let Some(arc) = self.vertex_pair_to_arc(v1, v2) {
                    result.push(Geometry::Arc(arc));
                } else {
                    // 回退到直线
                    result.push(Geometry::Line(Line::new(v1.point, v2.point)));
                }
            }
        }

        result
    }

    /// 将顶点对转换为圆弧
    fn vertex_pair_to_arc(&self, v1: &PolylineVertex, v2: &PolylineVertex) -> Option<Arc> {
        let chord = v2.point - v1.point;
        let chord_len = chord.norm();

        if chord_len < EPSILON {
            return None;
        }

        let bulge = v1.bulge;
        let s = chord_len / 2.0;
        let h = s * bulge; // 弧高

        // 计算圆心
        let mid = Point2::new(
            (v1.point.x + v2.point.x) / 2.0,
            (v1.point.y + v2.point.y) / 2.0,
        );

        let radius = (s * s + h * h) / (2.0 * h.abs());
        let d = radius - h.abs(); // 圆心到弦的距离

        // 弦的垂直方向
        let perp = if bulge > 0.0 {
            Vector2::new(-chord.y, chord.x).normalize()
        } else {
            Vector2::new(chord.y, -chord.x).normalize()
        };

        let center = mid + perp * d;

        let start_angle = (v1.point.y - center.y).atan2(v1.point.x - center.x);
        let end_angle = (v2.point.y - center.y).atan2(v2.point.x - center.x);

        Some(Arc::new(center, radius, start_angle, end_angle))
    }
}

// ========== 椭圆 (Ellipse) ==========

/// 椭圆
/// 
/// 支持完整椭圆和椭圆弧，使用 DXF 兼容的参数化方式：
/// - 中心点 + 长轴端点（相对向量）+ 短轴比例
/// - 起始/终止参数用于椭圆弧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ellipse {
    /// 中心点
    pub center: Point2,
    /// 长轴端点（相对于中心的向量）
    pub major_axis: Vector2,
    /// 短轴与长轴的比例 (0.0 < ratio <= 1.0)
    pub ratio: f64,
    /// 起始参数（弧度，0.0 表示长轴正方向）
    pub start_param: f64,
    /// 终止参数（弧度，2π 表示完整椭圆）
    pub end_param: f64,
}

impl Ellipse {
    /// 创建完整椭圆
    pub fn new(center: Point2, major_axis: Vector2, ratio: f64) -> Self {
        Self {
            center,
            major_axis,
            ratio: ratio.clamp(EPSILON, 1.0),
            start_param: 0.0,
            end_param: 2.0 * std::f64::consts::PI,
        }
    }

    /// 创建椭圆弧
    pub fn arc(center: Point2, major_axis: Vector2, ratio: f64, start_param: f64, end_param: f64) -> Self {
        Self {
            center,
            major_axis,
            ratio: ratio.clamp(EPSILON, 1.0),
            start_param,
            end_param,
        }
    }

    /// 从轴长创建椭圆（水平长轴）
    pub fn from_radii(center: Point2, major_radius: f64, minor_radius: f64) -> Self {
        let ratio = minor_radius / major_radius;
        Self::new(center, Vector2::new(major_radius, 0.0), ratio)
    }

    /// 获取长轴半径
    pub fn major_radius(&self) -> f64 {
        self.major_axis.norm()
    }

    /// 获取短轴半径
    pub fn minor_radius(&self) -> f64 {
        self.major_radius() * self.ratio
    }

    /// 获取长轴旋转角度（相对于X轴）
    pub fn rotation(&self) -> f64 {
        self.major_axis.y.atan2(self.major_axis.x)
    }

    /// 获取短轴方向向量（单位向量）
    pub fn minor_axis_direction(&self) -> Vector2 {
        let rot = self.rotation();
        Vector2::new(-rot.sin(), rot.cos())
    }

    /// 获取短轴端点向量
    pub fn minor_axis(&self) -> Vector2 {
        self.minor_axis_direction() * self.minor_radius()
    }

    /// 是否是完整椭圆
    pub fn is_full(&self) -> bool {
        (self.end_param - self.start_param - 2.0 * std::f64::consts::PI).abs() < EPSILON
    }

    /// 获取椭圆上指定参数的点
    /// 
    /// 参数 t 是椭圆的参数化角度，不是真正的几何角度
    pub fn point_at_param(&self, t: f64) -> Point2 {
        let cos_t = t.cos();
        let sin_t = t.sin();
        let major_dir = self.major_axis / self.major_radius();
        let minor_dir = self.minor_axis_direction();
        
        Point2::new(
            self.center.x + self.major_radius() * cos_t * major_dir.x + self.minor_radius() * sin_t * minor_dir.x,
            self.center.y + self.major_radius() * cos_t * major_dir.y + self.minor_radius() * sin_t * minor_dir.y,
        )
    }

    /// 获取起点
    pub fn start_point(&self) -> Point2 {
        self.point_at_param(self.start_param)
    }

    /// 获取终点
    pub fn end_point(&self) -> Point2 {
        self.point_at_param(self.end_param)
    }

    /// 计算周长（近似值，使用 Ramanujan 公式）
    pub fn circumference(&self) -> f64 {
        let a = self.major_radius();
        let b = self.minor_radius();
        let h = ((a - b) / (a + b)).powi(2);
        std::f64::consts::PI * (a + b) * (1.0 + 3.0 * h / (10.0 + (4.0 - 3.0 * h).sqrt()))
    }

    /// 计算面积
    pub fn area(&self) -> f64 {
        std::f64::consts::PI * self.major_radius() * self.minor_radius()
    }

    /// 计算点到椭圆的距离（近似值）
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        // 将点转换到椭圆的局部坐标系
        let rot = self.rotation();
        let cos_r = rot.cos();
        let sin_r = rot.sin();
        
        let local_x = (point.x - self.center.x) * cos_r + (point.y - self.center.y) * sin_r;
        let local_y = -(point.x - self.center.x) * sin_r + (point.y - self.center.y) * cos_r;
        
        // 使用迭代法找到最近点（Newton-Raphson）
        let a = self.major_radius();
        let b = self.minor_radius();
        
        // 初始猜测：使用角度
        let mut t = local_y.atan2(local_x);
        
        for _ in 0..10 {
            let cos_t = t.cos();
            let sin_t = t.sin();
            
            let ex = a * cos_t;
            let ey = b * sin_t;
            
            let dx = local_x - ex;
            let dy = local_y - ey;
            
            // 切线方向
            let tx = -a * sin_t;
            let ty = b * cos_t;
            
            // 投影
            let dot = dx * tx + dy * ty;
            let len_sq = tx * tx + ty * ty;
            
            if len_sq < EPSILON {
                break;
            }
            
            t += dot / len_sq;
        }
        
        // 检查参数是否在椭圆弧范围内
        if !self.is_full() {
            // 归一化 t 到 [0, 2π)
            let two_pi = 2.0 * std::f64::consts::PI;
            let mut t_norm = t % two_pi;
            if t_norm < 0.0 {
                t_norm += two_pi;
            }
            
            let mut start = self.start_param % two_pi;
            if start < 0.0 {
                start += two_pi;
            }
            let mut end = self.end_param % two_pi;
            if end < 0.0 {
                end += two_pi;
            }
            
            let in_range = if start <= end {
                t_norm >= start && t_norm <= end
            } else {
                t_norm >= start || t_norm <= end
            };
            
            if !in_range {
                // 返回到端点的最小距离
                let d1 = (point - self.start_point()).norm();
                let d2 = (point - self.end_point()).norm();
                return d1.min(d2);
            }
        }
        
        let closest = Point2::new(
            self.center.x + a * t.cos() * cos_r - b * t.sin() * sin_r,
            self.center.y + a * t.cos() * sin_r + b * t.sin() * cos_r,
        );
        
        (point - closest).norm()
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        let rot = self.rotation();
        let cos_r = rot.cos();
        let sin_r = rot.sin();
        let a = self.major_radius();
        let b = self.minor_radius();
        
        // 椭圆在 x, y 方向的极值
        let dx = (a * a * cos_r * cos_r + b * b * sin_r * sin_r).sqrt();
        let dy = (a * a * sin_r * sin_r + b * b * cos_r * cos_r).sqrt();
        
        if self.is_full() {
            BoundingBox2::new(
                Point2::new(self.center.x - dx, self.center.y - dy),
                Point2::new(self.center.x + dx, self.center.y + dy),
            )
        } else {
            // 椭圆弧：采样点计算包围盒
            let mut bbox = BoundingBox2::from_points([self.start_point(), self.end_point()]);
            
            let steps = 32;
            let range = self.end_param - self.start_param;
            for i in 0..=steps {
                let t = self.start_param + range * (i as f64) / (steps as f64);
                bbox.expand_to_include(&self.point_at_param(t));
            }
            
            bbox
        }
    }

    /// 获取椭圆上的采样点（用于渲染）
    pub fn sample_points(&self, segments: usize) -> Vec<Point2> {
        let mut points = Vec::with_capacity(segments + 1);
        let range = self.end_param - self.start_param;
        
        for i in 0..=segments {
            let t = self.start_param + range * (i as f64) / (segments as f64);
            points.push(self.point_at_param(t));
        }
        
        points
    }
}

// ========== 样条曲线 (Spline) ==========

/// 样条曲线类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SplineType {
    /// B-样条 (默认)
    #[default]
    BSpline,
    /// NURBS (有理B样条)
    Nurbs,
    /// 贝塞尔样条
    Bezier,
}

/// 样条曲线
/// 
/// 支持 B-样条和 NURBS 曲线，使用 De Boor 算法求值
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Spline {
    /// 样条类型
    pub spline_type: SplineType,
    /// 曲线阶数（通常为 3 或 4）
    pub degree: u8,
    /// 控制点
    pub control_points: Vec<Point2>,
    /// 节点向量（knot vector）
    pub knots: Vec<f64>,
    /// 权重（用于 NURBS，如果为空则默认全为 1）
    pub weights: Vec<f64>,
    /// 是否闭合
    pub closed: bool,
    /// 拟合点（用于样条拟合）
    pub fit_points: Vec<Point2>,
}

impl Spline {
    /// 创建一个空的 B-样条
    pub fn new(degree: u8) -> Self {
        Self {
            spline_type: SplineType::BSpline,
            degree,
            control_points: Vec::new(),
            knots: Vec::new(),
            weights: Vec::new(),
            closed: false,
            fit_points: Vec::new(),
        }
    }

    /// 从控制点创建 B-样条（自动生成均匀节点向量）
    pub fn from_control_points(control_points: Vec<Point2>, degree: u8, closed: bool) -> Self {
        let n = control_points.len();
        let k = degree as usize;
        
        // 生成均匀节点向量
        let num_knots = n + k + 1;
        let mut knots = Vec::with_capacity(num_knots);
        
        for i in 0..num_knots {
            if i < k {
                knots.push(0.0);
            } else if i >= n {
                knots.push((n - k + 1) as f64);
            } else {
                knots.push((i - k + 1) as f64);
            }
        }
        
        Self {
            spline_type: SplineType::BSpline,
            degree,
            control_points,
            knots,
            weights: Vec::new(),
            closed,
            fit_points: Vec::new(),
        }
    }

    /// 使用 De Boor 算法计算样条曲线上的点
    pub fn point_at_param(&self, t: f64) -> Point2 {
        if self.control_points.is_empty() {
            return Point2::origin();
        }
        
        if self.control_points.len() == 1 {
            return self.control_points[0];
        }
        
        let n = self.control_points.len();
        let k = self.degree as usize;
        
        // 找到 t 所在的区间
        let mut span = k;
        while span < n && self.knots.get(span + 1).map_or(false, |&k| k <= t) {
            span += 1;
        }
        span = span.min(n - 1);
        
        // De Boor 算法
        let mut d: Vec<Point2> = (0..=k)
            .filter_map(|i| {
                let idx = span.saturating_sub(k) + i;
                self.control_points.get(idx).copied()
            })
            .collect();
        
        if d.len() <= k {
            return self.control_points.last().copied().unwrap_or(Point2::origin());
        }
        
        for r in 1..=k {
            for j in (r..=k).rev() {
                let idx = span.saturating_sub(k) + j;
                let left = self.knots.get(idx).copied().unwrap_or(0.0);
                let right = self.knots.get(idx + k + 1 - r).copied().unwrap_or(1.0);
                
                let denom = right - left;
                if denom.abs() < EPSILON {
                    continue;
                }
                
                let alpha = (t - left) / denom;
                let j_idx = j;
                let j_prev = j - 1;
                
                if j_idx < d.len() && j_prev < d.len() {
                    d[j_idx] = Point2::new(
                        (1.0 - alpha) * d[j_prev].x + alpha * d[j_idx].x,
                        (1.0 - alpha) * d[j_prev].y + alpha * d[j_idx].y,
                    );
                }
            }
        }
        
        d.get(k).copied().unwrap_or(Point2::origin())
    }

    /// 获取参数范围
    pub fn param_range(&self) -> (f64, f64) {
        let k = self.degree as usize;
        let start = self.knots.get(k).copied().unwrap_or(0.0);
        let end = self.knots.get(self.knots.len().saturating_sub(k + 1)).copied().unwrap_or(1.0);
        (start, end)
    }

    /// 计算点到样条曲线的距离（近似值）
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        let samples = self.sample_points(64);
        
        let mut min_dist = f64::MAX;
        for i in 0..samples.len().saturating_sub(1) {
            let line = Line::new(samples[i], samples[i + 1]);
            min_dist = min_dist.min(line.distance_to_point(point));
        }
        
        min_dist
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        if self.control_points.is_empty() {
            return BoundingBox2::empty();
        }
        
        // 使用控制点的包围盒（保守估计）
        // 更精确的方法需要采样
        let mut bbox = BoundingBox2::from_points(self.control_points.iter().copied());
        
        // 添加采样点以获得更精确的包围盒
        for pt in self.sample_points(32) {
            bbox.expand_to_include(&pt);
        }
        
        bbox
    }

    /// 获取采样点（用于渲染）
    pub fn sample_points(&self, segments: usize) -> Vec<Point2> {
        let mut points = Vec::with_capacity(segments + 1);
        let (start, end) = self.param_range();
        
        for i in 0..=segments {
            let t = start + (end - start) * (i as f64) / (segments as f64);
            points.push(self.point_at_param(t));
        }
        
        points
    }
}

// ========== 填充 (Hatch) ==========

/// 填充边界类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HatchBoundaryElement {
    /// 线段
    Line(Line),
    /// 圆弧
    Arc(Arc),
    /// 椭圆弧
    Ellipse(Ellipse),
    /// 样条
    Spline(Spline),
}

/// 填充边界
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchBoundary {
    /// 边界元素
    pub elements: Vec<HatchBoundaryElement>,
    /// 是否为外边界（false 表示孔洞）
    pub is_outer: bool,
}

impl HatchBoundary {
    pub fn new(elements: Vec<HatchBoundaryElement>, is_outer: bool) -> Self {
        Self { elements, is_outer }
    }

    /// 获取边界的包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        let mut bbox = BoundingBox2::empty();
        for elem in &self.elements {
            let elem_bbox = match elem {
                HatchBoundaryElement::Line(l) => l.bounding_box(),
                HatchBoundaryElement::Arc(a) => a.bounding_box(),
                HatchBoundaryElement::Ellipse(e) => e.bounding_box(),
                HatchBoundaryElement::Spline(s) => s.bounding_box(),
            };
            bbox = bbox.union(&elem_bbox);
        }
        bbox
    }
}

/// 填充图案类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HatchPatternType {
    /// 实心填充
    Solid,
    /// 预定义图案
    Predefined(String),
    /// 用户自定义图案
    Custom {
        /// 图案线定义
        lines: Vec<HatchPatternLine>,
    },
}

/// 填充图案线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HatchPatternLine {
    /// 角度（弧度）
    pub angle: f64,
    /// 起点
    pub base_point: Point2,
    /// 偏移（用于平行线）
    pub offset: Vector2,
    /// 虚线模式（正数表示实线，负数表示间隙）
    pub dash_pattern: Vec<f64>,
}

/// 填充
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hatch {
    /// 边界
    pub boundaries: Vec<HatchBoundary>,
    /// 图案类型
    pub pattern_type: HatchPatternType,
    /// 图案角度（弧度）
    pub angle: f64,
    /// 图案比例
    pub scale: f64,
}

impl Hatch {
    /// 创建实心填充
    pub fn solid(boundaries: Vec<HatchBoundary>) -> Self {
        Self {
            boundaries,
            pattern_type: HatchPatternType::Solid,
            angle: 0.0,
            scale: 1.0,
        }
    }

    /// 创建图案填充
    pub fn pattern(boundaries: Vec<HatchBoundary>, pattern_name: &str, angle: f64, scale: f64) -> Self {
        Self {
            boundaries,
            pattern_type: HatchPatternType::Predefined(pattern_name.to_string()),
            angle,
            scale,
        }
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        let mut bbox = BoundingBox2::empty();
        for boundary in &self.boundaries {
            bbox = bbox.union(&boundary.bounding_box());
        }
        bbox
    }

    /// 检查点是否在填充区域内
    pub fn contains_point(&self, _point: &Point2, _tolerance: f64) -> bool {
        // TODO: 实现点在多边形内的判断（射线法）
        false
    }
}

// ========== 引线 (Leader) ==========

/// 箭头类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArrowType {
    /// 无箭头
    None,
    /// 闭合填充箭头（默认）
    #[default]
    ClosedFilled,
    /// 闭合空心箭头
    ClosedBlank,
    /// 开口箭头
    Open,
    /// 点
    Dot,
    /// 圆
    Circle,
}

/// 引线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Leader {
    /// 顶点列表（从箭头到文本）
    pub vertices: Vec<Point2>,
    /// 箭头类型
    pub arrow_type: ArrowType,
    /// 箭头大小
    pub arrow_size: f64,
    /// 关联的文本
    pub text: Option<String>,
    /// 文本高度
    pub text_height: f64,
}

impl Leader {
    /// 创建新的引线
    pub fn new(vertices: Vec<Point2>) -> Self {
        Self {
            vertices,
            arrow_type: ArrowType::ClosedFilled,
            arrow_size: 3.0,
            text: None,
            text_height: 2.5,
        }
    }

    /// 设置箭头类型
    pub fn with_arrow(mut self, arrow_type: ArrowType, size: f64) -> Self {
        self.arrow_type = arrow_type;
        self.arrow_size = size;
        self
    }

    /// 设置文本
    pub fn with_text(mut self, text: impl Into<String>, height: f64) -> Self {
        self.text = Some(text.into());
        self.text_height = height;
        self
    }

    /// 获取箭头位置（第一个顶点）
    pub fn arrow_point(&self) -> Option<Point2> {
        self.vertices.first().copied()
    }

    /// 获取箭头方向
    pub fn arrow_direction(&self) -> Option<Vector2> {
        if self.vertices.len() >= 2 {
            Some((self.vertices[0] - self.vertices[1]).normalize())
        } else {
            None
        }
    }

    /// 获取文本位置（最后一个顶点）
    pub fn text_position(&self) -> Option<Point2> {
        self.vertices.last().copied()
    }

    /// 计算总长度
    pub fn length(&self) -> f64 {
        let mut total = 0.0;
        for i in 0..self.vertices.len().saturating_sub(1) {
            total += (self.vertices[i + 1] - self.vertices[i]).norm();
        }
        total
    }

    /// 计算点到引线的距离
    pub fn distance_to_point(&self, point: &Point2) -> f64 {
        let mut min_dist = f64::MAX;
        for i in 0..self.vertices.len().saturating_sub(1) {
            let line = Line::new(self.vertices[i], self.vertices[i + 1]);
            min_dist = min_dist.min(line.distance_to_point(point));
        }
        min_dist
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        if self.vertices.is_empty() {
            return BoundingBox2::empty();
        }
        BoundingBox2::from_points(self.vertices.iter().copied())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_line_length() {
        let line = Line::new(Point2::new(0.0, 0.0), Point2::new(3.0, 4.0));
        assert!((line.length() - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_circle_area() {
        let circle = Circle::new(Point2::origin(), 1.0);
        assert!((circle.area() - std::f64::consts::PI).abs() < EPSILON);
    }

    #[test]
    fn test_polyline_explode() {
        let pl = Polyline::from_points(
            [
                Point2::new(0.0, 0.0),
                Point2::new(10.0, 0.0),
                Point2::new(10.0, 10.0),
            ],
            false,
        );

        let exploded = pl.explode();
        assert_eq!(exploded.len(), 2);
        assert!(matches!(exploded[0], Geometry::Line(_)));
        assert!(matches!(exploded[1], Geometry::Line(_)));
    }
}

