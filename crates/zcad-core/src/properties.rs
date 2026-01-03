//! 实体属性定义
//!
//! 包含颜色、线型、线宽等视觉属性。

use serde::{Deserialize, Serialize};

/// RGBA颜色
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    /// 从十六进制值创建（如 0xFF0000 表示红色）
    pub const fn from_hex(hex: u32) -> Self {
        Self {
            r: ((hex >> 16) & 0xFF) as u8,
            g: ((hex >> 8) & 0xFF) as u8,
            b: (hex & 0xFF) as u8,
            a: 255,
        }
    }

    /// 转换为 [0.0, 1.0] 范围的浮点数组
    pub fn to_f32_array(&self) -> [f32; 4] {
        [
            self.r as f32 / 255.0,
            self.g as f32 / 255.0,
            self.b as f32 / 255.0,
            self.a as f32 / 255.0,
        ]
    }

    // 预定义颜色（AutoCAD ACI颜色兼容）
    pub const RED: Color = Color::new(255, 0, 0);
    pub const YELLOW: Color = Color::new(255, 255, 0);
    pub const GREEN: Color = Color::new(0, 255, 0);
    pub const CYAN: Color = Color::new(0, 255, 255);
    pub const BLUE: Color = Color::new(0, 0, 255);
    pub const MAGENTA: Color = Color::new(255, 0, 255);
    pub const WHITE: Color = Color::new(255, 255, 255);
    pub const BLACK: Color = Color::new(0, 0, 0);
    pub const GRAY: Color = Color::new(128, 128, 128);

    /// 颜色跟随图层（ByLayer）
    pub const BY_LAYER: Color = Color::with_alpha(0, 0, 0, 0);

    /// 颜色跟随块（ByBlock）
    pub const BY_BLOCK: Color = Color::with_alpha(0, 0, 0, 1);

    pub fn is_by_layer(&self) -> bool {
        self.a == 0
    }

    pub fn is_by_block(&self) -> bool {
        self.a == 1 && self.r == 0 && self.g == 0 && self.b == 0
    }
}

impl Default for Color {
    fn default() -> Self {
        Self::BY_LAYER
    }
}

/// 线型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LineType {
    /// 连续线（实线）
    Continuous,
    /// 虚线
    Dashed,
    /// 点线
    Dotted,
    /// 点划线
    DashDot,
    /// 双点划线
    DashDotDot,
    /// 中心线
    Center,
    /// 隐藏线
    Hidden,
    /// 自定义线型
    Custom {
        name: String,
        /// 线型模式（正数表示画线，负数表示空白）
        pattern: Vec<f64>,
    },
    /// 跟随图层
    ByLayer,
    /// 跟随块
    ByBlock,
}

impl LineType {
    /// 获取线型的模式数据
    pub fn pattern(&self) -> Vec<f64> {
        match self {
            LineType::Continuous => vec![],
            LineType::Dashed => vec![12.0, -6.0],
            LineType::Dotted => vec![0.0, -6.0],
            LineType::DashDot => vec![12.0, -6.0, 0.0, -6.0],
            LineType::DashDotDot => vec![12.0, -6.0, 0.0, -6.0, 0.0, -6.0],
            LineType::Center => vec![32.0, -6.0, 6.0, -6.0],
            LineType::Hidden => vec![6.0, -3.0],
            LineType::Custom { pattern, .. } => pattern.clone(),
            LineType::ByLayer | LineType::ByBlock => vec![],
        }
    }

    /// 计算线型的总长度（一个重复单元）
    pub fn pattern_length(&self) -> f64 {
        self.pattern().iter().map(|x| x.abs()).sum()
    }
}

impl Default for LineType {
    fn default() -> Self {
        LineType::ByLayer
    }
}

/// 线宽（毫米）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum LineWeight {
    /// 默认线宽
    Default,
    /// 跟随图层
    ByLayer,
    /// 跟随块
    ByBlock,
    /// 指定线宽（毫米）
    Width(f64),
}

impl LineWeight {
    /// 获取实际线宽值（像素，假设96dpi）
    pub fn to_pixels(&self, layer_width: f64, default_width: f64) -> f64 {
        match self {
            LineWeight::Default => default_width,
            LineWeight::ByLayer => layer_width,
            LineWeight::ByBlock => default_width, // 简化处理
            LineWeight::Width(w) => *w * 96.0 / 25.4, // mm to pixels at 96dpi
        }
    }
}

impl Default for LineWeight {
    fn default() -> Self {
        LineWeight::ByLayer
    }
}

/// 实体的视觉属性
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Properties {
    /// 颜色
    pub color: Color,
    /// 线型
    pub line_type: LineType,
    /// 线宽
    pub line_weight: LineWeight,
    /// 透明度 (0-100, 0=不透明)
    pub transparency: u8,
}

impl Default for Properties {
    fn default() -> Self {
        Self {
            color: Color::BY_LAYER,
            line_type: LineType::ByLayer,
            line_weight: LineWeight::ByLayer,
            transparency: 0,
        }
    }
}

impl Properties {
    /// 创建带有指定颜色的属性
    pub fn with_color(color: Color) -> Self {
        Self {
            color,
            ..Default::default()
        }
    }

    /// 设置颜色
    pub fn set_color(mut self, color: Color) -> Self {
        self.color = color;
        self
    }

    /// 设置线型
    pub fn set_line_type(mut self, line_type: LineType) -> Self {
        self.line_type = line_type;
        self
    }

    /// 设置线宽
    pub fn set_line_weight(mut self, line_weight: LineWeight) -> Self {
        self.line_weight = line_weight;
        self
    }
}

