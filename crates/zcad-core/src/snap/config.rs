//! 捕捉配置

use serde::{Deserialize, Serialize};
use super::SnapType;

/// 捕捉配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapConfig {
    /// 捕捉容差（屏幕像素）
    pub tolerance: f64,
    /// 启用的捕捉类型
    pub enabled_types: SnapMask,
    /// 网格间距
    pub grid_spacing: f64,
    /// 是否显示捕捉标记
    pub show_markers: bool,
    /// 是否显示捕捉提示
    pub show_tooltips: bool,
    
    // ========== 极轴追踪 (Polar Tracking) ==========
    /// 是否启用极轴追踪
    pub polar_tracking: bool,
    /// 极轴角度列表（弧度），如 0°, 15°, 30°, 45°, 60°, 75°, 90°
    pub polar_angles: Vec<f64>,
    /// 极轴追踪的容差（弧度）
    pub polar_tolerance: f64,
    
    // ========== 延长线捕捉 (Extension Snap) ==========
    /// 是否启用延长线捕捉
    pub extension_snap: bool,
    
    // ========== 距离捕捉 (Distance Snap) ==========
    /// 是否启用从端点的距离捕捉
    pub distance_snap: bool,
    /// 距离捕捉的距离值
    pub snap_distance: f64,
    /// 中点分段数（用于等分点捕捉）
    pub middle_points: usize,
}

impl Default for SnapConfig {
    fn default() -> Self {
        Self {
            tolerance: 10.0, // 10像素
            enabled_types: SnapMask::default(),
            grid_spacing: 10.0,
            show_markers: true,
            show_tooltips: true,
            // 极轴追踪默认配置
            polar_tracking: false,
            polar_angles: vec![
                0.0,
                std::f64::consts::PI / 12.0,  // 15°
                std::f64::consts::PI / 6.0,   // 30°
                std::f64::consts::PI / 4.0,   // 45°
                std::f64::consts::PI / 3.0,   // 60°
                5.0 * std::f64::consts::PI / 12.0, // 75°
                std::f64::consts::PI / 2.0,   // 90°
            ],
            polar_tolerance: std::f64::consts::PI / 180.0 * 5.0, // 5度容差
            // 延长线捕捉
            extension_snap: false,
            // 距离捕捉
            distance_snap: false,
            snap_distance: 10.0,
            middle_points: 1, // 默认只有一个中点
        }
    }
}

/// 捕捉掩码（位域，用于快速启用/禁用捕捉类型）
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SnapMask {
    bits: u16,
}

impl SnapMask {
    pub const ENDPOINT: u16 = 1 << 0;
    pub const MIDPOINT: u16 = 1 << 1;
    pub const CENTER: u16 = 1 << 2;
    pub const INTERSECTION: u16 = 1 << 3;
    pub const PERPENDICULAR: u16 = 1 << 4;
    pub const TANGENT: u16 = 1 << 5;
    pub const NEAREST: u16 = 1 << 6;
    pub const GRID: u16 = 1 << 7;
    pub const QUADRANT: u16 = 1 << 8;

    pub const NONE: SnapMask = SnapMask { bits: 0 };
    pub const ALL: SnapMask = SnapMask { bits: 0xFFFF };

    pub fn new(bits: u16) -> Self {
        Self { bits }
    }

    pub fn is_enabled(&self, snap_type: SnapType) -> bool {
        let bit = match snap_type {
            SnapType::Endpoint => Self::ENDPOINT,
            SnapType::Midpoint => Self::MIDPOINT,
            SnapType::Center => Self::CENTER,
            SnapType::Intersection => Self::INTERSECTION,
            SnapType::Perpendicular => Self::PERPENDICULAR,
            SnapType::Tangent => Self::TANGENT,
            SnapType::Nearest => Self::NEAREST,
            SnapType::Grid => Self::GRID,
            SnapType::Quadrant => Self::QUADRANT,
        };
        self.bits & bit != 0
    }

    pub fn set(&mut self, snap_type: SnapType, enabled: bool) {
        let bit = match snap_type {
            SnapType::Endpoint => Self::ENDPOINT,
            SnapType::Midpoint => Self::MIDPOINT,
            SnapType::Center => Self::CENTER,
            SnapType::Intersection => Self::INTERSECTION,
            SnapType::Perpendicular => Self::PERPENDICULAR,
            SnapType::Tangent => Self::TANGENT,
            SnapType::Nearest => Self::NEAREST,
            SnapType::Grid => Self::GRID,
            SnapType::Quadrant => Self::QUADRANT,
        };
        if enabled {
            self.bits |= bit;
        } else {
            self.bits &= !bit;
        }
    }

    pub fn toggle(&mut self, snap_type: SnapType) {
        let enabled = self.is_enabled(snap_type);
        self.set(snap_type, !enabled);
    }
}

impl Default for SnapMask {
    fn default() -> Self {
        // 默认启用常用的捕捉类型
        Self {
            bits: Self::ENDPOINT | Self::MIDPOINT | Self::CENTER | Self::INTERSECTION,
        }
    }
}
