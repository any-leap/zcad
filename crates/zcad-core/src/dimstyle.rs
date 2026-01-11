//! 标注样式（Dimension Style）
//!
//! 参考 LibreCAD 的 LC_DimStyle 实现，提供完整的标注样式定义。
//! 
//! 标注样式定义了标注的外观和行为，包括：
//! - 文本格式和大小
//! - 箭头类型和大小
//! - 尺寸线和延伸线样式
//! - 单位和精度设置

use crate::units::{AngleFormat, LinearFormat, Unit};
use serde::{Deserialize, Serialize};

/// 箭头类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ArrowType {
    /// 实心闭合箭头
    #[default]
    ClosedFilled,
    /// 空心闭合箭头
    ClosedBlank,
    /// 开放箭头
    Open,
    /// 点
    Dot,
    /// 小点
    DotSmall,
    /// 空心点
    DotBlank,
    /// 原点指示器
    Origin,
    /// 直角
    RightAngle,
    /// 斜线
    Oblique,
    /// 无箭头
    None,
    /// 建筑标记（斜线）
    ArchitecturalTick,
    /// 积分符号
    Integral,
}

/// 文本对齐方式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DimTextAlignment {
    /// 居中
    #[default]
    Center,
    /// 左对齐
    Left,
    /// 右对齐
    Right,
    /// 标注线上方
    Above,
    /// 标注线外部
    Outside,
}

/// 文本垂直位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DimTextVertical {
    /// 居中
    #[default]
    Centered,
    /// 上方
    Above,
    /// 外部
    Outside,
    /// JIS 标准
    JIS,
}

/// 标注样式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimStyle {
    /// 样式名称
    pub name: String,
    
    // ===== 线条设置 =====
    /// 延伸线超出量（超出标注线的长度）
    pub extension_line_extension: f64,
    /// 延伸线偏移（与定义点的距离）
    pub extension_line_offset: f64,
    /// 抑制第一条延伸线
    pub suppress_extension_line1: bool,
    /// 抑制第二条延伸线
    pub suppress_extension_line2: bool,
    
    // ===== 箭头设置 =====
    /// 第一个箭头类型
    pub arrow_type1: ArrowType,
    /// 第二个箭头类型
    pub arrow_type2: ArrowType,
    /// 引线箭头类型
    pub leader_arrow_type: ArrowType,
    /// 箭头大小
    pub arrow_size: f64,
    
    // ===== 文本设置 =====
    /// 文本高度
    pub text_height: f64,
    /// 文本与标注线的间距
    pub text_gap: f64,
    /// 文本水平对齐
    pub text_horizontal: DimTextAlignment,
    /// 文本垂直位置
    pub text_vertical: DimTextVertical,
    /// 文本方向与标注线对齐
    pub text_aligned: bool,
    /// 文本颜色（None = 跟随层）
    pub text_color: Option<(u8, u8, u8)>,
    
    // ===== 单位设置 =====
    /// 长度单位
    pub linear_unit: Unit,
    /// 长度格式
    pub linear_format: LinearFormat,
    /// 长度精度（小数位数）
    pub linear_precision: u8,
    /// 长度比例因子（应用于测量值的乘数）
    pub linear_scale_factor: f64,
    /// 前缀
    pub prefix: String,
    /// 后缀
    pub suffix: String,
    /// 显示单位符号
    pub show_unit: bool,
    
    // ===== 角度单位设置 =====
    /// 角度格式
    pub angle_format: AngleFormat,
    /// 角度精度
    pub angle_precision: u8,
    /// 零度抑制（不显示末尾的零）
    pub zero_suppression: bool,
    
    // ===== 公差设置 =====
    /// 显示公差
    pub show_tolerance: bool,
    /// 上公差
    pub tolerance_upper: f64,
    /// 下公差
    pub tolerance_lower: f64,
    /// 公差精度
    pub tolerance_precision: u8,
    
    // ===== 替代单位 =====
    /// 显示替代单位
    pub show_alternate_units: bool,
    /// 替代单位
    pub alternate_unit: Unit,
    /// 替代单位精度
    pub alternate_precision: u8,
    /// 替代单位比例因子
    pub alternate_scale_factor: f64,
}

impl Default for DimStyle {
    fn default() -> Self {
        Self {
            name: "Standard".to_string(),
            
            // 线条设置
            extension_line_extension: 1.25,  // mm
            extension_line_offset: 0.625,    // mm
            suppress_extension_line1: false,
            suppress_extension_line2: false,
            
            // 箭头设置
            arrow_type1: ArrowType::ClosedFilled,
            arrow_type2: ArrowType::ClosedFilled,
            leader_arrow_type: ArrowType::ClosedFilled,
            arrow_size: 2.5,  // mm
            
            // 文本设置
            text_height: 2.5,  // mm
            text_gap: 0.625,   // mm
            text_horizontal: DimTextAlignment::Center,
            text_vertical: DimTextVertical::Above,
            text_aligned: true,
            text_color: None,
            
            // 单位设置
            linear_unit: Unit::Millimeter,
            linear_format: LinearFormat::Decimal,
            linear_precision: 2,
            linear_scale_factor: 1.0,
            prefix: String::new(),
            suffix: String::new(),
            show_unit: false,
            
            // 角度设置
            angle_format: AngleFormat::DegreesDecimal,
            angle_precision: 1,
            zero_suppression: false,
            
            // 公差设置
            show_tolerance: false,
            tolerance_upper: 0.0,
            tolerance_lower: 0.0,
            tolerance_precision: 2,
            
            // 替代单位
            show_alternate_units: false,
            alternate_unit: Unit::Inch,
            alternate_precision: 2,
            alternate_scale_factor: 1.0,
        }
    }
}

impl DimStyle {
    /// 创建新的标注样式
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }
    
    /// 创建 ISO 标准标注样式
    pub fn iso() -> Self {
        Self {
            name: "ISO-25".to_string(),
            text_height: 2.5,
            arrow_size: 2.5,
            extension_line_extension: 1.25,
            extension_line_offset: 0.625,
            text_gap: 0.625,
            linear_unit: Unit::Millimeter,
            linear_precision: 2,
            ..Default::default()
        }
    }
    
    /// 创建建筑标注样式（英制）
    pub fn architectural() -> Self {
        Self {
            name: "Architectural".to_string(),
            text_height: 3.0,
            arrow_size: 3.0,
            arrow_type1: ArrowType::ArchitecturalTick,
            arrow_type2: ArrowType::ArchitecturalTick,
            linear_unit: Unit::Inch,
            linear_format: LinearFormat::Architectural,
            linear_precision: 4,  // 1/16"
            ..Default::default()
        }
    }
    
    /// 创建机械制图标注样式
    pub fn mechanical() -> Self {
        Self {
            name: "Mechanical".to_string(),
            text_height: 3.5,
            arrow_size: 3.0,
            linear_unit: Unit::Millimeter,
            linear_precision: 3,
            show_tolerance: true,
            tolerance_upper: 0.1,
            tolerance_lower: -0.1,
            ..Default::default()
        }
    }
    
    /// 格式化测量值
    pub fn format_measurement(&self, value: f64) -> String {
        use crate::units::format_linear;
        
        let scaled_value = value * self.linear_scale_factor;
        let formatted = format_linear(
            scaled_value,
            self.linear_unit,
            self.linear_format,
            self.linear_precision,
            self.show_unit,
        );
        
        let mut result = String::new();
        
        // 添加前缀
        if !self.prefix.is_empty() {
            result.push_str(&self.prefix);
        }
        
        result.push_str(&formatted);
        
        // 添加公差
        if self.show_tolerance {
            result.push_str(&format!(
                " +{:.prec$}/-{:.prec$}",
                self.tolerance_upper,
                self.tolerance_lower.abs(),
                prec = self.tolerance_precision as usize
            ));
        }
        
        // 添加后缀
        if !self.suffix.is_empty() {
            result.push_str(&self.suffix);
        }
        
        // 添加替代单位
        if self.show_alternate_units {
            let alt_value = crate::units::convert(scaled_value, self.linear_unit, self.alternate_unit);
            let alt_formatted = format_linear(
                alt_value * self.alternate_scale_factor,
                self.alternate_unit,
                LinearFormat::Decimal,
                self.alternate_precision,
                true,
            );
            result.push_str(&format!(" [{}]", alt_formatted));
        }
        
        result
    }
    
    /// 格式化角度
    pub fn format_angle(&self, radians: f64) -> String {
        crate::units::format_angle(radians, self.angle_format, self.angle_precision)
    }
}

/// 标注样式管理器
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DimStyleManager {
    /// 所有标注样式
    styles: Vec<DimStyle>,
    /// 当前活动样式索引
    current_style_index: usize,
}

impl DimStyleManager {
    /// 创建新的样式管理器（带默认样式）
    pub fn new() -> Self {
        Self {
            styles: vec![
                DimStyle::default(),
                DimStyle::iso(),
                DimStyle::architectural(),
                DimStyle::mechanical(),
            ],
            current_style_index: 0,
        }
    }
    
    /// 获取当前样式
    pub fn current_style(&self) -> &DimStyle {
        &self.styles[self.current_style_index]
    }
    
    /// 获取当前样式（可变）
    pub fn current_style_mut(&mut self) -> &mut DimStyle {
        &mut self.styles[self.current_style_index]
    }
    
    /// 设置当前样式
    pub fn set_current_style(&mut self, name: &str) -> bool {
        if let Some(index) = self.styles.iter().position(|s| s.name == name) {
            self.current_style_index = index;
            true
        } else {
            false
        }
    }
    
    /// 添加样式
    pub fn add_style(&mut self, style: DimStyle) {
        self.styles.push(style);
    }
    
    /// 获取所有样式名称
    pub fn style_names(&self) -> Vec<&str> {
        self.styles.iter().map(|s| s.name.as_str()).collect()
    }
    
    /// 按名称获取样式
    pub fn get_style(&self, name: &str) -> Option<&DimStyle> {
        self.styles.iter().find(|s| s.name == name)
    }
    
    /// 按名称获取样式（可变）
    pub fn get_style_mut(&mut self, name: &str) -> Option<&mut DimStyle> {
        self.styles.iter_mut().find(|s| s.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_measurement() {
        let style = DimStyle::default();
        let result = style.format_measurement(25.4);
        assert_eq!(result, "25.40");
    }

    #[test]
    fn test_architectural_format() {
        let style = DimStyle::architectural();
        // 测试建筑格式
        let result = style.format_measurement(95.25);  // 3'-9 1/2"
        println!("Architectural: {}", result);
    }

    #[test]
    fn test_style_manager() {
        let manager = DimStyleManager::new();
        assert_eq!(manager.current_style().name, "Standard");
        assert_eq!(manager.style_names().len(), 4);
    }
}
