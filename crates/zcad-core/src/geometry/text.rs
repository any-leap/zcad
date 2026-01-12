//! 文本

use crate::math::{BoundingBox2, Point2, EPSILON};
use serde::{Deserialize, Serialize};

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TextAlignment {
    /// 左对齐（默认）
    #[default]
    Left,
    /// 居中对齐
    Center,
    /// 右对齐
    Right,
}

/// 文本
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Text {
    /// 插入点
    pub position: Point2,
    /// 文本内容
    pub content: String,
    /// 文本高度
    pub height: f64,
    /// 旋转角度（弧度）
    pub rotation: f64,
    /// 对齐方式
    pub alignment: TextAlignment,
}

impl Text {
    /// 创建新的文本对象
    pub fn new(position: Point2, content: impl Into<String>, height: f64) -> Self {
        Self {
            position,
            content: content.into(),
            height,
            rotation: 0.0,
            alignment: TextAlignment::Left,
        }
    }

    /// 设置旋转角度
    pub fn with_rotation(mut self, rotation: f64) -> Self {
        self.rotation = rotation;
        self
    }

    /// 设置对齐方式
    pub fn with_alignment(mut self, alignment: TextAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// 估算文本宽度（简化计算，假设每个字符宽度约为高度的0.6倍）
    pub fn estimated_width(&self) -> f64 {
        // 对于中文字符，宽度接近高度；对于英文，约为高度的0.6倍
        // 这里采用简化的混合估算
        let char_count = self.content.chars().count();
        let cjk_count = self.content.chars().filter(|c| Self::is_cjk(*c)).count();
        let ascii_count = char_count - cjk_count;
        
        (cjk_count as f64 * self.height) + (ascii_count as f64 * self.height * 0.6)
    }

    /// 检查是否是CJK字符
    fn is_cjk(c: char) -> bool {
        matches!(c, '\u{4E00}'..='\u{9FFF}' | '\u{3400}'..='\u{4DBF}' | '\u{F900}'..='\u{FAFF}')
    }

    /// 获取包围盒
    pub fn bounding_box(&self) -> BoundingBox2 {
        let width = self.estimated_width();
        let height = self.height;
        
        // 根据对齐方式计算基准点
        let base_x = match self.alignment {
            TextAlignment::Left => self.position.x,
            TextAlignment::Center => self.position.x - width / 2.0,
            TextAlignment::Right => self.position.x - width,
        };
        
        // 简化处理：忽略旋转的包围盒计算
        if self.rotation.abs() < EPSILON {
            BoundingBox2::new(
                Point2::new(base_x, self.position.y),
                Point2::new(base_x + width, self.position.y + height),
            )
        } else {
            // 带旋转的包围盒：计算四个角点
            let corners = [
                Point2::new(0.0, 0.0),
                Point2::new(width, 0.0),
                Point2::new(width, height),
                Point2::new(0.0, height),
            ];
            
            let cos_r = self.rotation.cos();
            let sin_r = self.rotation.sin();
            
            let rotated: Vec<Point2> = corners.iter().map(|p| {
                let rx = p.x * cos_r - p.y * sin_r + base_x;
                let ry = p.x * sin_r + p.y * cos_r + self.position.y;
                Point2::new(rx, ry)
            }).collect();
            
            BoundingBox2::from_points(rotated)
        }
    }

    /// 检查点是否在文本包围盒内
    pub fn contains_point(&self, point: &Point2, tolerance: f64) -> bool {
        let bbox = self.bounding_box();
        let expanded = BoundingBox2::new(
            Point2::new(bbox.min.x - tolerance, bbox.min.y - tolerance),
            Point2::new(bbox.max.x + tolerance, bbox.max.y + tolerance),
        );
        expanded.contains(point)
    }
}
