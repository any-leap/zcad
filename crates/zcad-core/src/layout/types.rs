//! 布局相关类型定义

use serde::{Deserialize, Serialize};

/// 布局 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LayoutId(pub u64);

impl LayoutId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 视口 ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ViewportId(pub u64);

impl ViewportId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

/// 纸张大小（用于布局）
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PaperSize {
    /// A4 (210 x 297 mm)
    A4,
    /// A3 (297 x 420 mm)
    A3,
    /// A2 (420 x 594 mm)
    A2,
    /// A1 (594 x 841 mm)
    A1,
    /// A0 (841 x 1189 mm)
    A0,
    /// Letter (8.5 x 11 in)
    Letter,
    /// Legal (8.5 x 14 in)
    Legal,
    /// Tabloid (11 x 17 in)
    Tabloid,
    /// 自定义尺寸 (宽, 高) mm
    Custom { width: f64, height: f64 },
}

impl Default for PaperSize {
    fn default() -> Self {
        PaperSize::A3
    }
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

    /// 获取纸张名称
    pub fn name(&self) -> &'static str {
        match self {
            PaperSize::A4 => "A4",
            PaperSize::A3 => "A3",
            PaperSize::A2 => "A2",
            PaperSize::A1 => "A1",
            PaperSize::A0 => "A0",
            PaperSize::Letter => "Letter",
            PaperSize::Legal => "Legal",
            PaperSize::Tabloid => "Tabloid",
            PaperSize::Custom { .. } => "Custom",
        }
    }

    /// 所有标准纸张尺寸
    pub fn all_standard() -> &'static [PaperSize] {
        &[
            PaperSize::A4,
            PaperSize::A3,
            PaperSize::A2,
            PaperSize::A1,
            PaperSize::A0,
            PaperSize::Letter,
            PaperSize::Legal,
            PaperSize::Tabloid,
        ]
    }
}

/// 纸张方向
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PaperOrientation {
    /// 纵向（高 > 宽）
    Portrait,
    /// 横向（宽 > 高）
    #[default]
    Landscape,
}

/// 视口状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ViewportStatus {
    /// 激活（可编辑模型空间）
    Active,
    /// 非激活（只显示）
    #[default]
    Inactive,
    /// 锁定（比例锁定）
    Locked,
    /// 隐藏
    Hidden,
}

/// 空间类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum SpaceType {
    /// 模型空间
    #[default]
    Model,
    /// 图纸空间（指定布局）
    Paper(LayoutId),
}

/// 标准比例列表
pub const STANDARD_SCALES: &[(&str, f64)] = &[
    ("1:1", 1.0),
    ("1:2", 2.0),
    ("1:5", 5.0),
    ("1:10", 10.0),
    ("1:20", 20.0),
    ("1:25", 25.0),
    ("1:50", 50.0),
    ("1:100", 100.0),
    ("1:200", 200.0),
    ("1:500", 500.0),
    ("1:1000", 1000.0),
    ("2:1", 0.5),
    ("5:1", 0.2),
    ("10:1", 0.1),
];
