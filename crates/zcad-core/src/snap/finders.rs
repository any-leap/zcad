//! 几何捕捉点收集器
//!
//! 各种几何类型的捕捉点查找逻辑

use crate::entity::EntityId;
use crate::geometry::{Arc, Circle, Ellipse, Leader, Line, Polyline, Spline};
use crate::math::{Point2, EPSILON};

use super::{SnapConfig, SnapMask, SnapPoint, SnapType};

/// 捕捉点收集器 trait
pub trait SnapFinder {
    fn collect_snap_points(
        &self,
        entity_id: EntityId,
        mouse: Point2,
        tolerance: f64,
        config: &SnapConfig,
        reference_point: Option<Point2>,
        candidates: &mut Vec<SnapPoint>,
        helpers: &SnapHelpers,
    );
}

/// 几何计算辅助方法
#[derive(Debug, Clone, Copy)]
pub struct SnapHelpers;

impl SnapHelpers {
    /// 计算点到线段的最近点
    pub fn nearest_point_on_line(&self, line: &Line, point: Point2) -> Point2 {
        let v = line.end - line.start;
        let w = point - line.start;

        let c1 = w.dot(&v);
        if c1 <= 0.0 {
            return line.start;
        }

        let c2 = v.dot(&v);
        if c2 <= c1 {
            return line.end;
        }

        let b = c1 / c2;
        line.start + v * b
    }

    /// 计算从参考点到线段的垂足
    pub fn perpendicular_to_line(&self, line: &Line, ref_point: Point2) -> Option<Point2> {
        let v = line.end - line.start;
        let w = ref_point - line.start;

        let c1 = w.dot(&v);
        let c2 = v.dot(&v);

        if c2 < EPSILON {
            return None;
        }

        let b = c1 / c2;
        
        // 垂足必须在线段上
        if b >= 0.0 && b <= 1.0 {
            Some(line.start + v * b)
        } else {
            None
        }
    }

    /// 计算从点到圆的切点
    pub fn tangent_points_to_circle(&self, circle: &Circle, point: Point2) -> Vec<Point2> {
        let d = (point - circle.center).norm();
        
        // 点在圆内，没有切点
        if d <= circle.radius {
            return vec![];
        }

        // 从圆心到点的方向
        let dir = (point - circle.center) / d;
        
        // 切点角度
        let angle = (circle.radius / d).asin();
        
        // 计算两个切点
        let base_angle = dir.y.atan2(dir.x);
        
        vec![
            Point2::new(
                circle.center.x + circle.radius * (base_angle + angle).cos(),
                circle.center.y + circle.radius * (base_angle + angle).sin(),
            ),
            Point2::new(
                circle.center.x + circle.radius * (base_angle - angle).cos(),
                circle.center.y + circle.radius * (base_angle - angle).sin(),
            ),
        ]
    }
}

/// 线段捕捉点收集
pub fn collect_line_snap_points(
    line: &Line,
    entity_id: EntityId,
    mouse: Point2,
    tolerance: f64,
    enabled: &SnapMask,
    reference_point: Option<Point2>,
    candidates: &mut Vec<SnapPoint>,
    helpers: &SnapHelpers,
) {
    // 端点
    if enabled.is_enabled(SnapType::Endpoint) {
        let dist_start = (line.start - mouse).norm();
        if dist_start <= tolerance {
            candidates.push(SnapPoint::new(
                line.start,
                SnapType::Endpoint,
                Some(entity_id),
                dist_start,
            ));
        }

        let dist_end = (line.end - mouse).norm();
        if dist_end <= tolerance {
            candidates.push(SnapPoint::new(
                line.end,
                SnapType::Endpoint,
                Some(entity_id),
                dist_end,
            ));
        }
    }

    // 中点
    if enabled.is_enabled(SnapType::Midpoint) {
        let midpoint = line.midpoint();
        let dist = (midpoint - mouse).norm();
        if dist <= tolerance {
            candidates.push(SnapPoint::new(
                midpoint,
                SnapType::Midpoint,
                Some(entity_id),
                dist,
            ));
        }
    }

    // 垂足
    if enabled.is_enabled(SnapType::Perpendicular) {
        if let Some(ref_point) = reference_point {
            if let Some(perp) = helpers.perpendicular_to_line(line, ref_point) {
                let dist = (perp - mouse).norm();
                if dist <= tolerance {
                    candidates.push(SnapPoint::new(
                        perp,
                        SnapType::Perpendicular,
                        Some(entity_id),
                        dist,
                    ));
                }
            }
        }
    }

    // 最近点
    if enabled.is_enabled(SnapType::Nearest) {
        let nearest = helpers.nearest_point_on_line(line, mouse);
        let dist = (nearest - mouse).norm();
        if dist <= tolerance {
            candidates.push(SnapPoint::new(
                nearest,
                SnapType::Nearest,
                Some(entity_id),
                dist,
            ));
        }
    }
}

/// 圆捕捉点收集
pub fn collect_circle_snap_points(
    circle: &Circle,
    entity_id: EntityId,
    mouse: Point2,
    tolerance: f64,
    enabled: &SnapMask,
    reference_point: Option<Point2>,
    candidates: &mut Vec<SnapPoint>,
    helpers: &SnapHelpers,
) {
    // 圆心
    if enabled.is_enabled(SnapType::Center) {
        let dist = (circle.center - mouse).norm();
        if dist <= tolerance {
            candidates.push(SnapPoint::new(
                circle.center,
                SnapType::Center,
                Some(entity_id),
                dist,
            ));
        }
    }

    // 象限点
    if enabled.is_enabled(SnapType::Quadrant) {
        let quadrant_angles = [0.0, std::f64::consts::FRAC_PI_2, std::f64::consts::PI, 3.0 * std::f64::consts::FRAC_PI_2];
        for angle in quadrant_angles {
            let point = circle.point_at_angle(angle);
            let dist = (point - mouse).norm();
            if dist <= tolerance {
                candidates.push(SnapPoint::new(
                    point,
                    SnapType::Quadrant,
                    Some(entity_id),
                    dist,
                ));
            }
        }
    }

    // 切点
    if enabled.is_enabled(SnapType::Tangent) {
        if let Some(ref_point) = reference_point {
            for tangent in helpers.tangent_points_to_circle(circle, ref_point) {
                let dist = (tangent - mouse).norm();
                if dist <= tolerance {
                    candidates.push(SnapPoint::new(
                        tangent,
                        SnapType::Tangent,
                        Some(entity_id),
                        dist,
                    ));
                }
            }
        }
    }

    // 最近点（圆上）
    if enabled.is_enabled(SnapType::Nearest) {
        let dir = (mouse - circle.center).normalize();
        let nearest = circle.center + dir * circle.radius;
        let dist = (nearest - mouse).norm();
        if dist <= tolerance {
            candidates.push(SnapPoint::new(
                nearest,
                SnapType::Nearest,
                Some(entity_id),
                dist,
            ));
        }
    }
}

/// 圆弧捕捉点收集
pub fn collect_arc_snap_points(
    arc: &Arc,
    entity_id: EntityId,
    mouse: Point2,
    tolerance: f64,
    enabled: &SnapMask,
    candidates: &mut Vec<SnapPoint>,
) {
    // 端点
    if enabled.is_enabled(SnapType::Endpoint) {
        let start = arc.start_point();
        let dist_start = (start - mouse).norm();
        if dist_start <= tolerance {
            candidates.push(SnapPoint::new(
                start,
                SnapType::Endpoint,
                Some(entity_id),
                dist_start,
            ));
        }

        let end = arc.end_point();
        let dist_end = (end - mouse).norm();
        if dist_end <= tolerance {
            candidates.push(SnapPoint::new(
                end,
                SnapType::Endpoint,
                Some(entity_id),
                dist_end,
            ));
        }
    }

    // 圆心
    if enabled.is_enabled(SnapType::Center) {
        let dist = (arc.center - mouse).norm();
        if dist <= tolerance {
            candidates.push(SnapPoint::new(
                arc.center,
                SnapType::Center,
                Some(entity_id),
                dist,
            ));
        }
    }

    // 中点（弧的中点）
    if enabled.is_enabled(SnapType::Midpoint) {
        let mid_angle = arc.start_angle + arc.sweep_angle() / 2.0;
        let midpoint = Point2::new(
            arc.center.x + arc.radius * mid_angle.cos(),
            arc.center.y + arc.radius * mid_angle.sin(),
        );
        let dist = (midpoint - mouse).norm();
        if dist <= tolerance {
            candidates.push(SnapPoint::new(
                midpoint,
                SnapType::Midpoint,
                Some(entity_id),
                dist,
            ));
        }
    }
}

/// 多段线捕捉点收集
pub fn collect_polyline_snap_points(
    polyline: &Polyline,
    entity_id: EntityId,
    mouse: Point2,
    tolerance: f64,
    enabled: &SnapMask,
    reference_point: Option<Point2>,
    candidates: &mut Vec<SnapPoint>,
    helpers: &SnapHelpers,
) {
    // 顶点（端点）
    if enabled.is_enabled(SnapType::Endpoint) {
        for vertex in &polyline.vertices {
            let dist = (vertex.point - mouse).norm();
            if dist <= tolerance {
                candidates.push(SnapPoint::new(
                    vertex.point,
                    SnapType::Endpoint,
                    Some(entity_id),
                    dist,
                ));
            }
        }
    }

    // 线段中点
    if enabled.is_enabled(SnapType::Midpoint) {
        for i in 0..polyline.segment_count() {
            let v1 = &polyline.vertices[i];
            let v2 = &polyline.vertices[(i + 1) % polyline.vertices.len()];

            // 只处理直线段的中点
            if v1.bulge.abs() < EPSILON {
                let midpoint = Point2::new(
                    (v1.point.x + v2.point.x) / 2.0,
                    (v1.point.y + v2.point.y) / 2.0,
                );
                let dist = (midpoint - mouse).norm();
                if dist <= tolerance {
                    candidates.push(SnapPoint::new(
                        midpoint,
                        SnapType::Midpoint,
                        Some(entity_id),
                        dist,
                    ));
                }
            }
        }
    }

    // 最近点和垂足需要遍历所有线段
    if enabled.is_enabled(SnapType::Nearest) || enabled.is_enabled(SnapType::Perpendicular) {
        for i in 0..polyline.segment_count() {
            let v1 = &polyline.vertices[i];
            let v2 = &polyline.vertices[(i + 1) % polyline.vertices.len()];

            // 只处理直线段
            if v1.bulge.abs() < EPSILON {
                let line = Line::new(v1.point, v2.point);

                if enabled.is_enabled(SnapType::Nearest) {
                    let nearest = helpers.nearest_point_on_line(&line, mouse);
                    let dist = (nearest - mouse).norm();
                    if dist <= tolerance {
                        candidates.push(SnapPoint::new(
                            nearest,
                            SnapType::Nearest,
                            Some(entity_id),
                            dist,
                        ));
                    }
                }

                if enabled.is_enabled(SnapType::Perpendicular) {
                    if let Some(ref_point) = reference_point {
                        if let Some(perp) = helpers.perpendicular_to_line(&line, ref_point) {
                            let dist = (perp - mouse).norm();
                            if dist <= tolerance {
                                candidates.push(SnapPoint::new(
                                    perp,
                                    SnapType::Perpendicular,
                                    Some(entity_id),
                                    dist,
                                ));
                            }
                        }
                    }
                }
            }
        }
    }
}

/// 椭圆捕捉点收集
pub fn collect_ellipse_snap_points(
    ellipse: &Ellipse,
    entity_id: EntityId,
    mouse: Point2,
    tolerance: f64,
    enabled: &SnapMask,
    candidates: &mut Vec<SnapPoint>,
) {
    // 圆心
    if enabled.is_enabled(SnapType::Center) {
        let dist = (ellipse.center - mouse).norm();
        if dist <= tolerance {
            candidates.push(SnapPoint::new(
                ellipse.center,
                SnapType::Center,
                Some(entity_id),
                dist,
            ));
        }
    }

    // 端点（椭圆弧的端点）
    if !ellipse.is_full() && enabled.is_enabled(SnapType::Endpoint) {
        let start = ellipse.start_point();
        let dist_start = (start - mouse).norm();
        if dist_start <= tolerance {
            candidates.push(SnapPoint::new(
                start,
                SnapType::Endpoint,
                Some(entity_id),
                dist_start,
            ));
        }

        let end = ellipse.end_point();
        let dist_end = (end - mouse).norm();
        if dist_end <= tolerance {
            candidates.push(SnapPoint::new(
                end,
                SnapType::Endpoint,
                Some(entity_id),
                dist_end,
            ));
        }
    }

    // 象限点（长轴和短轴的端点）
    if enabled.is_enabled(SnapType::Quadrant) {
        let major_end1 = ellipse.center + ellipse.major_axis;
        let major_end2 = ellipse.center - ellipse.major_axis;
        let minor_end1 = ellipse.center + ellipse.minor_axis();
        let minor_end2 = ellipse.center - ellipse.minor_axis();

        for point in [major_end1, major_end2, minor_end1, minor_end2] {
            let dist = (point - mouse).norm();
            if dist <= tolerance {
                candidates.push(SnapPoint::new(
                    point,
                    SnapType::Quadrant,
                    Some(entity_id),
                    dist,
                ));
            }
        }
    }

    // 最近点
    if enabled.is_enabled(SnapType::Nearest) {
        let samples = ellipse.sample_points(64);
        let mut min_dist = f64::MAX;
        let mut nearest = mouse;
        
        for pt in samples {
            let dist = (pt - mouse).norm();
            if dist < min_dist {
                min_dist = dist;
                nearest = pt;
            }
        }
        
        if min_dist <= tolerance {
            candidates.push(SnapPoint::new(
                nearest,
                SnapType::Nearest,
                Some(entity_id),
                min_dist,
            ));
        }
    }
}

/// 样条曲线捕捉点收集
pub fn collect_spline_snap_points(
    spline: &Spline,
    entity_id: EntityId,
    mouse: Point2,
    tolerance: f64,
    enabled: &SnapMask,
    candidates: &mut Vec<SnapPoint>,
) {
    // 控制点（作为端点）
    if enabled.is_enabled(SnapType::Endpoint) {
        for &pt in &spline.control_points {
            let dist = (pt - mouse).norm();
            if dist <= tolerance {
                candidates.push(SnapPoint::new(
                    pt,
                    SnapType::Endpoint,
                    Some(entity_id),
                    dist,
                ));
            }
        }
    }

    // 拟合点
    if enabled.is_enabled(SnapType::Endpoint) {
        for &pt in &spline.fit_points {
            let dist = (pt - mouse).norm();
            if dist <= tolerance {
                candidates.push(SnapPoint::new(
                    pt,
                    SnapType::Endpoint,
                    Some(entity_id),
                    dist,
                ));
            }
        }
    }

    // 最近点
    if enabled.is_enabled(SnapType::Nearest) {
        let samples = spline.sample_points(64);
        let mut min_dist = f64::MAX;
        let mut nearest = mouse;
        
        for pt in samples {
            let dist = (pt - mouse).norm();
            if dist < min_dist {
                min_dist = dist;
                nearest = pt;
            }
        }
        
        if min_dist <= tolerance {
            candidates.push(SnapPoint::new(
                nearest,
                SnapType::Nearest,
                Some(entity_id),
                min_dist,
            ));
        }
    }
}

/// 引线捕捉点收集
pub fn collect_leader_snap_points(
    leader: &Leader,
    entity_id: EntityId,
    mouse: Point2,
    tolerance: f64,
    enabled: &SnapMask,
    candidates: &mut Vec<SnapPoint>,
) {
    // 顶点（端点）
    if enabled.is_enabled(SnapType::Endpoint) {
        for &pt in &leader.vertices {
            let dist = (pt - mouse).norm();
            if dist <= tolerance {
                candidates.push(SnapPoint::new(
                    pt,
                    SnapType::Endpoint,
                    Some(entity_id),
                    dist,
                ));
            }
        }
    }

    // 线段中点
    if enabled.is_enabled(SnapType::Midpoint) {
        for i in 0..leader.vertices.len().saturating_sub(1) {
            let midpoint = Point2::new(
                (leader.vertices[i].x + leader.vertices[i + 1].x) / 2.0,
                (leader.vertices[i].y + leader.vertices[i + 1].y) / 2.0,
            );
            let dist = (midpoint - mouse).norm();
            if dist <= tolerance {
                candidates.push(SnapPoint::new(
                    midpoint,
                    SnapType::Midpoint,
                    Some(entity_id),
                    dist,
                ));
            }
        }
    }
}
