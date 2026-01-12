//! 样条曲线
//! 
//! 支持 B-样条和 NURBS 曲线，使用 De Boor 算法求值

use crate::math::{BoundingBox2, Point2, EPSILON};
use serde::{Deserialize, Serialize};

use super::Line;

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
