//! 捕捉类型和捕捉点

use crate::entity::EntityId;
use crate::math::Point2;
use serde::{Deserialize, Serialize};

/// 捕捉类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SnapType {
    /// 端点捕捉
    Endpoint,
    /// 中点捕捉
    Midpoint,
    /// 圆心捕捉
    Center,
    /// 交点捕捉
    Intersection,
    /// 垂足捕捉
    Perpendicular,
    /// 切点捕捉
    Tangent,
    /// 最近点捕捉
    Nearest,
    /// 网格点捕捉
    Grid,
    /// 象限点（圆/弧的0°, 90°, 180°, 270°位置）
    Quadrant,
}

impl SnapType {
    /// 获取捕捉类型的名称
    pub fn name(&self) -> &'static str {
        match self {
            SnapType::Endpoint => "端点",
            SnapType::Midpoint => "中点",
            SnapType::Center => "圆心",
            SnapType::Intersection => "交点",
            SnapType::Perpendicular => "垂足",
            SnapType::Tangent => "切点",
            SnapType::Nearest => "最近点",
            SnapType::Grid => "网格点",
            SnapType::Quadrant => "象限点",
        }
    }

    /// 获取捕捉类型的快捷键
    pub fn shortcut(&self) -> &'static str {
        match self {
            SnapType::Endpoint => "END",
            SnapType::Midpoint => "MID",
            SnapType::Center => "CEN",
            SnapType::Intersection => "INT",
            SnapType::Perpendicular => "PER",
            SnapType::Tangent => "TAN",
            SnapType::Nearest => "NEA",
            SnapType::Grid => "GRI",
            SnapType::Quadrant => "QUA",
        }
    }
}

/// 捕捉点
#[derive(Debug, Clone)]
pub struct SnapPoint {
    /// 捕捉到的世界坐标
    pub point: Point2,
    /// 捕捉类型
    pub snap_type: SnapType,
    /// 关联的实体ID（如果有）
    pub entity_id: Option<EntityId>,
    /// 距离鼠标的屏幕距离（用于排序）
    pub distance: f64,
}

impl SnapPoint {
    pub fn new(point: Point2, snap_type: SnapType, entity_id: Option<EntityId>, distance: f64) -> Self {
        Self {
            point,
            snap_type,
            entity_id,
            distance,
        }
    }
}
