//! 捕捉引擎

use crate::entity::Entity;
use crate::geometry::Geometry;
use crate::math::Point2;

use super::config::{SnapConfig, SnapMask};
use super::finders::{
    self, SnapHelpers,
};
use super::intersection::IntersectionFinder;
use super::polar::PolarSnap;
use super::types::{SnapPoint, SnapType};

/// 捕捉引擎
///
/// 负责计算和管理对象捕捉
#[derive(Debug, Clone)]
pub struct SnapEngine {
    config: SnapConfig,
    /// 缓存的候选捕捉点
    candidates: Vec<SnapPoint>,
    /// 辅助方法
    helpers: SnapHelpers,
}

impl SnapEngine {
    pub fn new(config: SnapConfig) -> Self {
        Self {
            config,
            candidates: Vec::with_capacity(64),
            helpers: SnapHelpers,
        }
    }

    /// 获取配置
    pub fn config(&self) -> &SnapConfig {
        &self.config
    }

    /// 获取配置（可变）
    pub fn config_mut(&mut self) -> &mut SnapConfig {
        &mut self.config
    }

    /// 寻找最佳捕捉点
    ///
    /// # 参数
    /// - `mouse_world`: 鼠标的世界坐标
    /// - `entities`: 要搜索的实体列表
    /// - `zoom`: 当前缩放级别（用于计算屏幕距离）
    /// - `reference_point`: 参考点（用于垂足、切点等计算）
    pub fn find_snap_point(
        &mut self,
        mouse_world: Point2,
        entities: &[&Entity],
        zoom: f64,
        reference_point: Option<Point2>,
    ) -> Option<SnapPoint> {
        self.candidates.clear();

        // 世界坐标容差
        let world_tolerance = self.config.tolerance / zoom;

        // 1. 网格捕捉
        if self.config.enabled_types.is_enabled(SnapType::Grid) {
            if let Some(snap) = self.snap_to_grid(mouse_world, world_tolerance) {
                self.candidates.push(snap);
            }
        }

        // 2. 收集所有实体的捕捉点
        for entity in entities {
            self.collect_entity_snap_points(
                entity,
                mouse_world,
                world_tolerance,
                reference_point,
            );
        }

        // 3. 交点捕捉（需要成对的实体）
        if self.config.enabled_types.is_enabled(SnapType::Intersection) {
            self.collect_intersection_points(entities, mouse_world, world_tolerance);
        }

        // 4. 找到最近的捕捉点
        self.candidates
            .iter()
            .filter(|p| p.distance <= world_tolerance)
            .min_by(|a, b| a.distance.partial_cmp(&b.distance).unwrap_or(std::cmp::Ordering::Equal))
            .cloned()
    }

    /// 收集单个实体的捕捉点
    fn collect_entity_snap_points(
        &mut self,
        entity: &Entity,
        mouse: Point2,
        tolerance: f64,
        reference_point: Option<Point2>,
    ) {
        let Some(geometry) = entity.geometry() else { return; };
        let enabled = &self.config.enabled_types;
        
        match geometry {
            Geometry::Point(p) => {
                if enabled.is_enabled(SnapType::Endpoint) {
                    let dist = (p.position - mouse).norm();
                    if dist <= tolerance {
                        self.candidates.push(SnapPoint::new(
                            p.position,
                            SnapType::Endpoint,
                            Some(entity.id),
                            dist,
                        ));
                    }
                }
            }
            Geometry::Line(line) => {
                finders::collect_line_snap_points(
                    line,
                    entity.id,
                    mouse,
                    tolerance,
                    enabled,
                    reference_point,
                    &mut self.candidates,
                    &self.helpers,
                );
            }
            Geometry::Circle(circle) => {
                finders::collect_circle_snap_points(
                    circle,
                    entity.id,
                    mouse,
                    tolerance,
                    enabled,
                    reference_point,
                    &mut self.candidates,
                    &self.helpers,
                );
            }
            Geometry::Arc(arc) => {
                finders::collect_arc_snap_points(
                    arc,
                    entity.id,
                    mouse,
                    tolerance,
                    enabled,
                    &mut self.candidates,
                );
            }
            Geometry::Polyline(polyline) => {
                finders::collect_polyline_snap_points(
                    polyline,
                    entity.id,
                    mouse,
                    tolerance,
                    enabled,
                    reference_point,
                    &mut self.candidates,
                    &self.helpers,
                );
            }
            Geometry::Text(text) => {
                // 文本只捕捉插入点
                if enabled.is_enabled(SnapType::Endpoint) {
                    let dist = (text.position - mouse).norm();
                    if dist <= tolerance {
                        self.candidates.push(SnapPoint::new(
                            text.position,
                            SnapType::Endpoint,
                            Some(entity.id),
                            dist,
                        ));
                    }
                }
            }
            Geometry::Dimension(dim) => {
                // 标注捕捉定义点
                if enabled.is_enabled(SnapType::Endpoint) {
                    for &pt in &[dim.definition_point1, dim.definition_point2] {
                        let dist = (pt - mouse).norm();
                        if dist <= tolerance {
                            self.candidates.push(SnapPoint::new(
                                pt,
                                SnapType::Endpoint,
                                Some(entity.id),
                                dist,
                            ));
                        }
                    }
                }
            }
            Geometry::Ellipse(ellipse) => {
                finders::collect_ellipse_snap_points(
                    ellipse,
                    entity.id,
                    mouse,
                    tolerance,
                    enabled,
                    &mut self.candidates,
                );
            }
            Geometry::Spline(spline) => {
                finders::collect_spline_snap_points(
                    spline,
                    entity.id,
                    mouse,
                    tolerance,
                    enabled,
                    &mut self.candidates,
                );
            }
            Geometry::Hatch(_) => {
                // 填充通常不参与捕捉
            }
            Geometry::Leader(leader) => {
                finders::collect_leader_snap_points(
                    leader,
                    entity.id,
                    mouse,
                    tolerance,
                    enabled,
                    &mut self.candidates,
                );
            }
            Geometry::Table(_) => {
                // 表格通常不参与捕捉（使用单独的网格捕捉）
            }
        }
    }

    /// 收集交点
    fn collect_intersection_points(
        &mut self,
        entities: &[&Entity],
        mouse: Point2,
        tolerance: f64,
    ) {
        // 双重循环检查所有实体对
        for i in 0..entities.len() {
            for j in (i + 1)..entities.len() {
                let (Some(geom_i), Some(geom_j)) = (entities[i].geometry(), entities[j].geometry()) else { continue; };
                let intersections = IntersectionFinder::find_intersections(geom_i, geom_j);
                
                for point in intersections {
                    let dist = (point - mouse).norm();
                    if dist <= tolerance {
                        self.candidates.push(SnapPoint::new(
                            point,
                            SnapType::Intersection,
                            None, // 交点涉及两个实体
                            dist,
                        ));
                    }
                }
            }
        }
    }

    /// 网格捕捉
    fn snap_to_grid(&self, mouse: Point2, tolerance: f64) -> Option<SnapPoint> {
        let spacing = self.config.grid_spacing;
        
        let grid_x = (mouse.x / spacing).round() * spacing;
        let grid_y = (mouse.y / spacing).round() * spacing;
        let grid_point = Point2::new(grid_x, grid_y);
        
        let dist = (grid_point - mouse).norm();
        if dist <= tolerance {
            Some(SnapPoint::new(grid_point, SnapType::Grid, None, dist))
        } else {
            None
        }
    }

    // ========== 极轴追踪和高级功能（委托给 PolarSnap）==========

    /// 极轴追踪
    pub fn snap_to_polar(&self, coord: Point2, base: Point2) -> Option<SnapPoint> {
        PolarSnap::snap_to_polar(coord, base, &self.config)
    }

    /// 正交限制
    pub fn restrict_orthogonal(&self, coord: Point2, base: Point2) -> Point2 {
        PolarSnap::restrict_orthogonal(coord, base)
    }

    /// 水平限制
    pub fn restrict_horizontal(&self, coord: Point2, base: Point2) -> Point2 {
        PolarSnap::restrict_horizontal(coord, base)
    }

    /// 垂直限制
    pub fn restrict_vertical(&self, coord: Point2, base: Point2) -> Point2 {
        PolarSnap::restrict_vertical(coord, base)
    }

    /// 角度限制
    pub fn restrict_angle(&self, base: Point2, coord: Point2, angle: f64) -> Point2 {
        PolarSnap::restrict_angle(base, coord, angle)
    }

    // ========== 配置方法 ==========

    /// 设置极轴追踪角度（度数）
    pub fn set_polar_angles_degrees(&mut self, angles: &[f64]) {
        self.config.polar_angles = angles
            .iter()
            .map(|deg| deg.to_radians())
            .collect();
    }

    /// 获取极轴追踪角度（度数）
    pub fn get_polar_angles_degrees(&self) -> Vec<f64> {
        self.config.polar_angles
            .iter()
            .map(|rad| rad.to_degrees())
            .collect()
    }

    /// 切换极轴追踪
    pub fn toggle_polar_tracking(&mut self) {
        self.config.polar_tracking = !self.config.polar_tracking;
    }

    /// 切换延长线捕捉
    pub fn toggle_extension_snap(&mut self) {
        self.config.extension_snap = !self.config.extension_snap;
    }

    /// 切换距离捕捉
    pub fn toggle_distance_snap(&mut self) {
        self.config.distance_snap = !self.config.distance_snap;
    }

    /// 设置距离捕捉的距离值
    pub fn set_snap_distance(&mut self, distance: f64) {
        self.config.snap_distance = distance;
    }

    /// 设置中点分段数
    pub fn set_middle_points(&mut self, count: usize) {
        self.config.middle_points = count.max(1);
    }
}

impl Default for SnapEngine {
    fn default() -> Self {
        Self::new(SnapConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::Line;
    use crate::math::EPSILON;

    #[test]
    fn test_snap_mask() {
        let mut mask = SnapMask::default();
        assert!(mask.is_enabled(SnapType::Endpoint));
        assert!(mask.is_enabled(SnapType::Midpoint));
        assert!(!mask.is_enabled(SnapType::Nearest));

        mask.set(SnapType::Nearest, true);
        assert!(mask.is_enabled(SnapType::Nearest));

        mask.toggle(SnapType::Endpoint);
        assert!(!mask.is_enabled(SnapType::Endpoint));
    }

    #[test]
    fn test_line_intersection() {
        let l1 = Line::new(Point2::new(0.0, 0.0), Point2::new(10.0, 10.0));
        let l2 = Line::new(Point2::new(0.0, 10.0), Point2::new(10.0, 0.0));

        let intersection = IntersectionFinder::line_line_intersection(&l1, &l2);
        assert!(intersection.is_some());

        let p = intersection.unwrap();
        assert!((p.x - 5.0).abs() < EPSILON);
        assert!((p.y - 5.0).abs() < EPSILON);
    }

    #[test]
    fn test_nearest_point_on_line() {
        let helpers = SnapHelpers;
        let line = Line::new(Point2::new(0.0, 0.0), Point2::new(10.0, 0.0));

        // 中间点
        let nearest = helpers.nearest_point_on_line(&line, Point2::new(5.0, 5.0));
        assert!((nearest.x - 5.0).abs() < EPSILON);
        assert!((nearest.y).abs() < EPSILON);

        // 线段外的点
        let nearest = helpers.nearest_point_on_line(&line, Point2::new(-5.0, 0.0));
        assert!((nearest.x).abs() < EPSILON); // 应该返回起点
    }
}
