//! 极轴追踪和高级捕捉功能

use crate::entity::EntityId;
use crate::geometry::Line;
use crate::math::{Point2, EPSILON};

use super::{SnapConfig, SnapPoint, SnapType};

/// 极轴和高级捕捉功能
pub struct PolarSnap;

impl PolarSnap {
    /// 极轴追踪 - 捕捉到特定角度的线
    ///
    /// # 参数
    /// - `coord`: 当前鼠标坐标
    /// - `base`: 参考点（上一个点）
    /// - `config`: 捕捉配置
    ///
    /// # 返回
    /// 如果当前点接近某个极轴角度，返回调整后的点
    pub fn snap_to_polar(coord: Point2, base: Point2, config: &SnapConfig) -> Option<SnapPoint> {
        if !config.polar_tracking {
            return None;
        }

        let delta = coord - base;
        let dist = delta.norm();
        
        if dist < EPSILON {
            return None;
        }

        let current_angle = delta.y.atan2(delta.x);
        
        // 检查每个极轴角度（考虑4个象限）
        for &polar_angle in &config.polar_angles {
            for quadrant in 0..4 {
                let check_angle = polar_angle + (quadrant as f64) * std::f64::consts::FRAC_PI_2;
                let normalized_check = normalize_angle(check_angle);
                let normalized_current = normalize_angle(current_angle);
                
                let diff = (normalized_check - normalized_current).abs();
                let diff = diff.min(2.0 * std::f64::consts::PI - diff);
                
                if diff <= config.polar_tolerance {
                    // 捕捉到这个角度
                    let snapped_point = Point2::new(
                        base.x + dist * normalized_check.cos(),
                        base.y + dist * normalized_check.sin(),
                    );
                    let snap_dist = (snapped_point - coord).norm();
                    
                    return Some(SnapPoint::new(
                        snapped_point,
                        SnapType::Grid, // 用 Grid 类型表示极轴捕捉
                        None,
                        snap_dist,
                    ));
                }
            }
        }
        
        None
    }

    /// 正交限制 - 限制为水平或垂直方向
    pub fn restrict_orthogonal(coord: Point2, base: Point2) -> Point2 {
        let dx = (coord.x - base.x).abs();
        let dy = (coord.y - base.y).abs();
        
        if dx > dy {
            Point2::new(coord.x, base.y)
        } else {
            Point2::new(base.x, coord.y)
        }
    }

    /// 水平限制
    pub fn restrict_horizontal(coord: Point2, base: Point2) -> Point2 {
        Point2::new(coord.x, base.y)
    }

    /// 垂直限制
    pub fn restrict_vertical(coord: Point2, base: Point2) -> Point2 {
        Point2::new(base.x, coord.y)
    }

    /// 角度限制 - 限制到指定角度
    pub fn restrict_angle(base: Point2, coord: Point2, angle: f64) -> Point2 {
        let dist = (coord - base).norm();
        Point2::new(
            base.x + dist * angle.cos(),
            base.y + dist * angle.sin(),
        )
    }

    /// 延长线捕捉 - 捕捉到线段延长线上的点
    pub fn snap_to_extension(
        mouse: Point2,
        line: &Line,
        tolerance: f64,
        config: &SnapConfig,
    ) -> Option<SnapPoint> {
        if !config.extension_snap {
            return None;
        }

        let dir = line.direction();
        
        // 检查延长线起点方向
        let t_start = (mouse - line.start).dot(&dir);
        if t_start < 0.0 {
            let projected = line.start + dir * t_start;
            let dist = (projected - mouse).norm();
            if dist <= tolerance {
                return Some(SnapPoint::new(
                    projected,
                    SnapType::Nearest,
                    None,
                    dist,
                ));
            }
        }
        
        // 检查延长线终点方向
        let line_len = line.length();
        let t_end = (mouse - line.start).dot(&dir);
        if t_end > line_len {
            let projected = line.start + dir * t_end;
            let dist = (projected - mouse).norm();
            if dist <= tolerance {
                return Some(SnapPoint::new(
                    projected,
                    SnapType::Nearest,
                    None,
                    dist,
                ));
            }
        }
        
        None
    }

    /// 从端点的距离捕捉
    pub fn snap_to_distance_from_endpoint(
        mouse: Point2,
        line: &Line,
        entity_id: EntityId,
        tolerance: f64,
        config: &SnapConfig,
    ) -> Vec<SnapPoint> {
        if !config.distance_snap || config.snap_distance <= EPSILON {
            return vec![];
        }

        let mut snaps = Vec::new();
        let distance = config.snap_distance;
        let dir = line.direction();
        let line_len = line.length();

        // 从起点的距离点
        if distance < line_len {
            let point = line.start + dir * distance;
            let dist = (point - mouse).norm();
            if dist <= tolerance {
                snaps.push(SnapPoint::new(
                    point,
                    SnapType::Endpoint,
                    Some(entity_id),
                    dist,
                ));
            }
        }

        // 从终点的距离点
        if distance < line_len {
            let point = line.end - dir * distance;
            let dist = (point - mouse).norm();
            if dist <= tolerance {
                snaps.push(SnapPoint::new(
                    point,
                    SnapType::Endpoint,
                    Some(entity_id),
                    dist,
                ));
            }
        }

        snaps
    }

    /// 等分点捕捉（扩展的中点捕捉）
    pub fn snap_to_division_points(
        mouse: Point2,
        line: &Line,
        entity_id: EntityId,
        tolerance: f64,
        config: &SnapConfig,
    ) -> Vec<SnapPoint> {
        let mut snaps = Vec::new();
        let divisions = config.middle_points + 1; // 分段数 = 中点数 + 1
        
        if divisions < 2 {
            return snaps;
        }

        let dir = line.end - line.start;
        
        for i in 1..divisions {
            let t = i as f64 / divisions as f64;
            let point = line.start + dir * t;
            let dist = (point - mouse).norm();
            
            if dist <= tolerance {
                snaps.push(SnapPoint::new(
                    point,
                    SnapType::Midpoint,
                    Some(entity_id),
                    dist,
                ));
            }
        }

        snaps
    }
}

/// 归一化角度到 [0, 2π)
pub fn normalize_angle(angle: f64) -> f64 {
    let two_pi = 2.0 * std::f64::consts::PI;
    let mut a = angle % two_pi;
    if a < 0.0 {
        a += two_pi;
    }
    a
}
