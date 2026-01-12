//! 椭圆
//! 
//! 支持完整椭圆和椭圆弧，使用 DXF 兼容的参数化方式：
//! - 中心点 + 长轴端点（相对向量）+ 短轴比例
//! - 起始/终止参数用于椭圆弧

use crate::math::{BoundingBox2, Point2, Vector2, EPSILON};
use serde::{Deserialize, Serialize};

/// 椭圆
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
