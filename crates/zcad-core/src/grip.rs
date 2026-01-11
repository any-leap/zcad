//! 夹点编辑模块
//! 
//! 提供几何图形的夹点（Grip）定义和编辑功能。
//! 夹点是几何图形上的特殊控制点，用户可以通过拖动夹点来直接编辑图形。

use crate::geometry::Geometry;
use crate::math::Point2;
use serde::{Deserialize, Serialize};

/// 夹点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GripType {
    /// 端点夹点 - 用于移动端点
    Endpoint,
    /// 中点夹点 - 用于移动整体或调整曲线
    Midpoint,
    /// 中心点夹点 - 用于移动圆心
    Center,
    /// 象限点夹点 - 用于调整圆/弧的半径
    Quadrant,
    /// 控制点夹点 - 用于样条曲线
    ControlPoint,
    /// 基点夹点 - 用于块参照
    BasePoint,
    /// 旋转夹点 - 用于旋转操作
    Rotation,
    /// 缩放夹点 - 用于缩放操作
    Scale,
}

/// 夹点定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grip {
    /// 夹点类型
    pub grip_type: GripType,
    /// 夹点位置（世界坐标）
    pub position: Point2,
    /// 夹点索引（用于标识同类型的多个夹点）
    pub index: usize,
    /// 额外数据（用于特殊用途）
    pub data: Option<GripData>,
}

/// 夹点额外数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GripData {
    /// 角度值（用于旋转夹点）
    Angle(f64),
    /// 缩放因子（用于缩放夹点）
    Scale(f64),
    /// 方向向量（用于定向夹点）
    Direction(Point2),
}

impl Grip {
    /// 创建新的夹点
    pub fn new(grip_type: GripType, position: Point2, index: usize) -> Self {
        Self {
            grip_type,
            position,
            index,
            data: None,
        }
    }
    
    /// 创建带额外数据的夹点
    #[allow(dead_code)]
    pub fn with_data(grip_type: GripType, position: Point2, index: usize, data: GripData) -> Self {
        Self {
            grip_type,
            position,
            index,
            data: Some(data),
        }
    }
    
    /// 检查点是否在夹点范围内
    pub fn contains_point(&self, point: Point2, tolerance: f64) -> bool {
        (self.position - point).norm() <= tolerance
    }
}

/// 为几何体获取夹点列表
pub fn get_grips_for_geometry(geometry: &Geometry) -> Vec<Grip> {
    use crate::geometry::*;
    
    match geometry {
        Geometry::Line(line) => get_line_grips(line),
        Geometry::Circle(circle) => get_circle_grips(circle),
        Geometry::Arc(arc) => get_arc_grips(arc),
        Geometry::Point(point) => get_point_grips(point),
        Geometry::Polyline(polyline) => get_polyline_grips(polyline),
        Geometry::Ellipse(ellipse) => get_ellipse_grips(ellipse),
        Geometry::Spline(spline) => get_spline_grips(spline),
        Geometry::Text(_) => vec![], // 文本使用单独的编辑方式
        Geometry::Dimension(_) => vec![], // 标注使用单独的编辑方式
        Geometry::Hatch(_) => vec![], // 填充使用边界编辑
        Geometry::Leader(leader) => get_leader_grips(leader),
    }
}

/// 获取直线的夹点
fn get_line_grips(line: &crate::geometry::Line) -> Vec<Grip> {
    let mid = line.midpoint();
    vec![
        Grip::new(GripType::Endpoint, line.start, 0),
        Grip::new(GripType::Midpoint, mid, 0),
        Grip::new(GripType::Endpoint, line.end, 1),
    ]
}

/// 获取圆的夹点
fn get_circle_grips(circle: &crate::geometry::Circle) -> Vec<Grip> {
    let r = circle.radius;
    let c = circle.center;
    
    vec![
        Grip::new(GripType::Center, c, 0),
        Grip::new(GripType::Quadrant, Point2::new(c.x + r, c.y), 0),
        Grip::new(GripType::Quadrant, Point2::new(c.x, c.y + r), 1),
        Grip::new(GripType::Quadrant, Point2::new(c.x - r, c.y), 2),
        Grip::new(GripType::Quadrant, Point2::new(c.x, c.y - r), 3),
    ]
}

/// 计算圆弧上指定角度的点
fn arc_point_at_angle(arc: &crate::geometry::Arc, angle: f64) -> Point2 {
    Point2::new(
        arc.center.x + arc.radius * angle.cos(),
        arc.center.y + arc.radius * angle.sin(),
    )
}

/// 获取圆弧的夹点
fn get_arc_grips(arc: &crate::geometry::Arc) -> Vec<Grip> {
    let start_point = arc_point_at_angle(arc, arc.start_angle);
    let end_point = arc_point_at_angle(arc, arc.end_angle);
    let mid_angle = (arc.start_angle + arc.end_angle) / 2.0;
    let mid_point = arc_point_at_angle(arc, mid_angle);
    
    vec![
        Grip::new(GripType::Center, arc.center, 0),
        Grip::new(GripType::Endpoint, start_point, 0),
        Grip::new(GripType::Midpoint, mid_point, 0),
        Grip::new(GripType::Endpoint, end_point, 1),
    ]
}

/// 获取点的夹点
fn get_point_grips(point: &crate::geometry::Point) -> Vec<Grip> {
    vec![Grip::new(GripType::Center, point.position, 0)]
}

/// 获取多段线的夹点
fn get_polyline_grips(polyline: &crate::geometry::Polyline) -> Vec<Grip> {
    let mut grips = Vec::new();
    
    // 添加顶点夹点
    for (i, vertex) in polyline.vertices.iter().enumerate() {
        grips.push(Grip::new(GripType::Endpoint, vertex.point, i));
    }
    
    // 添加线段中点
    for i in 0..polyline.vertices.len().saturating_sub(1) {
        let mid = Point2::new(
            (polyline.vertices[i].point.x + polyline.vertices[i + 1].point.x) / 2.0,
            (polyline.vertices[i].point.y + polyline.vertices[i + 1].point.y) / 2.0,
        );
        grips.push(Grip::new(GripType::Midpoint, mid, i));
    }
    
    // 如果闭合，添加最后一条边的中点
    if polyline.closed && polyline.vertices.len() >= 2 {
        let last = polyline.vertices.len() - 1;
        let mid = Point2::new(
            (polyline.vertices[last].point.x + polyline.vertices[0].point.x) / 2.0,
            (polyline.vertices[last].point.y + polyline.vertices[0].point.y) / 2.0,
        );
        grips.push(Grip::new(GripType::Midpoint, mid, last));
    }
    
    grips
}

/// 获取椭圆的夹点
fn get_ellipse_grips(ellipse: &crate::geometry::Ellipse) -> Vec<Grip> {
    let c = ellipse.center;
    let major = ellipse.major_axis;
    let minor_dir = crate::math::Vector2::new(-major.y, major.x).normalize();
    let minor_len = major.norm() * ellipse.ratio;
    
    vec![
        Grip::new(GripType::Center, c, 0),
        Grip::new(GripType::Endpoint, c + major, 0),  // 长轴端点
        Grip::new(GripType::Endpoint, c - major, 1),  // 长轴另一端
        Grip::new(GripType::Endpoint, c + minor_dir * minor_len, 2),  // 短轴端点
        Grip::new(GripType::Endpoint, c - minor_dir * minor_len, 3),  // 短轴另一端
    ]
}

/// 获取样条曲线的夹点
fn get_spline_grips(spline: &crate::geometry::Spline) -> Vec<Grip> {
    spline.control_points
        .iter()
        .enumerate()
        .map(|(i, pt)| Grip::new(GripType::ControlPoint, *pt, i))
        .collect()
}

/// 获取引线的夹点
fn get_leader_grips(leader: &crate::geometry::Leader) -> Vec<Grip> {
    leader.vertices
        .iter()
        .enumerate()
        .map(|(i, pt)| Grip::new(GripType::Endpoint, *pt, i))
        .collect()
}

/// 通过移动夹点来更新几何体
/// 
/// 返回更新后的几何体副本，如果更新失败则返回 None
pub fn update_geometry_by_grip(
    geometry: &Geometry,
    grip: &Grip,
    new_position: Point2,
) -> Option<Geometry> {
    use crate::geometry::*;
    
    match geometry {
        Geometry::Line(line) => update_line_by_grip(line, grip, new_position),
        Geometry::Circle(circle) => update_circle_by_grip(circle, grip, new_position),
        Geometry::Arc(arc) => update_arc_by_grip(arc, grip, new_position),
        Geometry::Point(point) => update_point_by_grip(point, grip, new_position),
        Geometry::Polyline(polyline) => update_polyline_by_grip(polyline, grip, new_position),
        Geometry::Ellipse(ellipse) => update_ellipse_by_grip(ellipse, grip, new_position),
        Geometry::Spline(spline) => update_spline_by_grip(spline, grip, new_position),
        Geometry::Leader(leader) => update_leader_by_grip(leader, grip, new_position),
        _ => None,
    }
}

fn update_line_by_grip(line: &crate::geometry::Line, grip: &Grip, new_pos: Point2) -> Option<Geometry> {
    let mut new_line = line.clone();
    match grip.grip_type {
        GripType::Endpoint => {
            if grip.index == 0 {
                new_line.start = new_pos;
            } else {
                new_line.end = new_pos;
            }
        }
        GripType::Midpoint => {
            // 移动中点 = 整体平移
            let offset = new_pos - grip.position;
            new_line.start += offset;
            new_line.end += offset;
        }
        _ => return None,
    }
    Some(Geometry::Line(new_line))
}

fn update_circle_by_grip(circle: &crate::geometry::Circle, grip: &Grip, new_pos: Point2) -> Option<Geometry> {
    let mut new_circle = circle.clone();
    match grip.grip_type {
        GripType::Center => {
            new_circle.center = new_pos;
        }
        GripType::Quadrant => {
            // 调整半径
            new_circle.radius = (new_pos - circle.center).norm();
        }
        _ => return None,
    }
    Some(Geometry::Circle(new_circle))
}

fn update_arc_by_grip(arc: &crate::geometry::Arc, grip: &Grip, new_pos: Point2) -> Option<Geometry> {
    let mut new_arc = arc.clone();
    match grip.grip_type {
        GripType::Center => {
            new_arc.center = new_pos;
        }
        GripType::Endpoint => {
            // 调整起点或终点角度
            let angle = (new_pos - arc.center).y.atan2((new_pos - arc.center).x);
            if grip.index == 0 {
                new_arc.start_angle = angle;
            } else {
                new_arc.end_angle = angle;
            }
            // 同时调整半径
            new_arc.radius = (new_pos - arc.center).norm();
        }
        GripType::Midpoint => {
            // 调整半径
            new_arc.radius = (new_pos - arc.center).norm();
        }
        _ => return None,
    }
    Some(Geometry::Arc(new_arc))
}

fn update_point_by_grip(point: &crate::geometry::Point, grip: &Grip, new_pos: Point2) -> Option<Geometry> {
    if grip.grip_type == GripType::Center {
        Some(Geometry::Point(crate::geometry::Point { position: new_pos }))
    } else {
        None
    }
}

fn update_polyline_by_grip(polyline: &crate::geometry::Polyline, grip: &Grip, new_pos: Point2) -> Option<Geometry> {
    let mut new_polyline = polyline.clone();
    match grip.grip_type {
        GripType::Endpoint => {
            if grip.index < new_polyline.vertices.len() {
                new_polyline.vertices[grip.index].point = new_pos;
            }
        }
        GripType::Midpoint => {
            // 移动线段中点 = 移动相邻两个顶点
            let offset = new_pos - grip.position;
            if grip.index < new_polyline.vertices.len() {
                new_polyline.vertices[grip.index].point += offset;
            }
            let next = (grip.index + 1) % new_polyline.vertices.len();
            if next < new_polyline.vertices.len() {
                new_polyline.vertices[next].point += offset;
            }
        }
        _ => return None,
    }
    Some(Geometry::Polyline(new_polyline))
}

fn update_ellipse_by_grip(ellipse: &crate::geometry::Ellipse, grip: &Grip, new_pos: Point2) -> Option<Geometry> {
    let mut new_ellipse = ellipse.clone();
    match grip.grip_type {
        GripType::Center => {
            new_ellipse.center = new_pos;
        }
        GripType::Endpoint => {
            match grip.index {
                0 | 1 => {
                    // 长轴端点
                    new_ellipse.major_axis = new_pos - ellipse.center;
                    if grip.index == 1 {
                        new_ellipse.major_axis = -new_ellipse.major_axis;
                    }
                }
                2 | 3 => {
                    // 短轴端点 - 调整 ratio
                    let minor_len = (new_pos - ellipse.center).norm();
                    let major_len = ellipse.major_axis.norm();
                    if major_len > 0.0 {
                        new_ellipse.ratio = minor_len / major_len;
                    }
                }
                _ => return None,
            }
        }
        _ => return None,
    }
    Some(Geometry::Ellipse(new_ellipse))
}

fn update_spline_by_grip(spline: &crate::geometry::Spline, grip: &Grip, new_pos: Point2) -> Option<Geometry> {
    if grip.grip_type == GripType::ControlPoint && grip.index < spline.control_points.len() {
        let mut new_spline = spline.clone();
        new_spline.control_points[grip.index] = new_pos;
        Some(Geometry::Spline(new_spline))
    } else {
        None
    }
}

fn update_leader_by_grip(leader: &crate::geometry::Leader, grip: &Grip, new_pos: Point2) -> Option<Geometry> {
    if grip.grip_type == GripType::Endpoint && grip.index < leader.vertices.len() {
        let mut new_leader = leader.clone();
        new_leader.vertices[grip.index] = new_pos;
        Some(Geometry::Leader(new_leader))
    } else {
        None
    }
}
