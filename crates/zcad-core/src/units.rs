//! 单位系统
//!
//! 参考 LibreCAD 的 RS_Units 实现，提供完整的单位转换和格式化功能。
//!
//! # AutoCAD 性能问题规避
//!
//! AutoCAD 历史上存在一些设计导致的性能问题，我们需要提前规避：
//!
//! ## 1. 单位转换实时计算
//! AutoCAD 在每次显示/编辑时都进行单位转换计算，当图纸很大时会很慢。
//! **规避方案**：内部统一使用毫米存储，只在显示和导入导出时转换。
//!
//! ## 2. 标注实时重新计算
//! AutoCAD 的标注在每次缩放/平移时都重新计算文本位置和箭头。
//! **规避方案**：缓存标注渲染结果，只在标注数据改变时重新计算。
//!
//! ## 3. 对象选择遍历所有实体
//! AutoCAD 早期版本在选择时遍历所有实体检查是否命中。
//! **规避方案**：使用 R-Tree 空间索引加速选择。
//!
//! ## 4. 撤销/重做存储完整快照
//! AutoCAD 早期版本每次操作存储完整图纸快照。
//! **规避方案**：使用增量操作记录（Operation-based），只存储变更。
//!
//! ## 5. 图层过滤重复遍历
//! AutoCAD 在渲染时对每个图层过滤实体列表。
//! **规避方案**：使用图层索引，按图层分组存储实体引用。

use serde::{Deserialize, Serialize};

/// 绘图单位
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum Unit {
    /// 无单位（使用父级单位）
    None,
    /// 英寸
    Inch,
    /// 英尺 (12 英寸)
    Foot,
    /// 英里 (1760 码)
    Mile,
    /// 毫米 (默认)
    #[default]
    Millimeter,
    /// 厘米
    Centimeter,
    /// 米
    Meter,
    /// 千米
    Kilometer,
    /// 微英寸
    Microinch,
    /// 密尔 (0.001 英寸)
    Mil,
    /// 码 (3 英尺)
    Yard,
    /// 埃 (10^-10 米)
    Angstrom,
    /// 纳米
    Nanometer,
    /// 微米
    Micron,
    /// 分米
    Decimeter,
    /// 十米
    Decameter,
    /// 百米
    Hectometer,
}

impl Unit {
    /// 获取单位到毫米的转换因子
    pub fn to_mm_factor(&self) -> f64 {
        match self {
            Unit::None => 1.0,
            Unit::Inch => 25.4,
            Unit::Foot => 304.8,
            Unit::Mile => 1_609_344.0,
            Unit::Millimeter => 1.0,
            Unit::Centimeter => 10.0,
            Unit::Meter => 1000.0,
            Unit::Kilometer => 1_000_000.0,
            Unit::Microinch => 0.0000254,
            Unit::Mil => 0.0254,
            Unit::Yard => 914.4,
            Unit::Angstrom => 1e-7,
            Unit::Nanometer => 1e-6,
            Unit::Micron => 0.001,
            Unit::Decimeter => 100.0,
            Unit::Decameter => 10_000.0,
            Unit::Hectometer => 100_000.0,
        }
    }

    /// 是否是公制单位
    pub fn is_metric(&self) -> bool {
        matches!(
            self,
            Unit::Millimeter
                | Unit::Centimeter
                | Unit::Meter
                | Unit::Kilometer
                | Unit::Angstrom
                | Unit::Nanometer
                | Unit::Micron
                | Unit::Decimeter
                | Unit::Decameter
                | Unit::Hectometer
        )
    }

    /// 获取单位符号
    pub fn symbol(&self) -> &'static str {
        match self {
            Unit::None => "",
            Unit::Inch => "\"",
            Unit::Foot => "'",
            Unit::Mile => "mi",
            Unit::Millimeter => "mm",
            Unit::Centimeter => "cm",
            Unit::Meter => "m",
            Unit::Kilometer => "km",
            Unit::Microinch => "µin",
            Unit::Mil => "mil",
            Unit::Yard => "yd",
            Unit::Angstrom => "Å",
            Unit::Nanometer => "nm",
            Unit::Micron => "µm",
            Unit::Decimeter => "dm",
            Unit::Decameter => "dam",
            Unit::Hectometer => "hm",
        }
    }

    /// 获取单位名称
    pub fn name(&self) -> &'static str {
        match self {
            Unit::None => "None",
            Unit::Inch => "Inch",
            Unit::Foot => "Foot",
            Unit::Mile => "Mile",
            Unit::Millimeter => "Millimeter",
            Unit::Centimeter => "Centimeter",
            Unit::Meter => "Meter",
            Unit::Kilometer => "Kilometer",
            Unit::Microinch => "Microinch",
            Unit::Mil => "Mil",
            Unit::Yard => "Yard",
            Unit::Angstrom => "Angstrom",
            Unit::Nanometer => "Nanometer",
            Unit::Micron => "Micron",
            Unit::Decimeter => "Decimeter",
            Unit::Decameter => "Decameter",
            Unit::Hectometer => "Hectometer",
        }
    }

    /// 从字符串解析单位
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "mm" | "millimeter" | "millimeters" => Some(Unit::Millimeter),
            "cm" | "centimeter" | "centimeters" => Some(Unit::Centimeter),
            "m" | "meter" | "meters" => Some(Unit::Meter),
            "km" | "kilometer" | "kilometers" => Some(Unit::Kilometer),
            "in" | "inch" | "inches" | "\"" => Some(Unit::Inch),
            "ft" | "foot" | "feet" | "'" => Some(Unit::Foot),
            "mi" | "mile" | "miles" => Some(Unit::Mile),
            "yd" | "yard" | "yards" => Some(Unit::Yard),
            "mil" => Some(Unit::Mil),
            "µm" | "um" | "micron" => Some(Unit::Micron),
            "nm" | "nanometer" => Some(Unit::Nanometer),
            _ => None,
        }
    }
}

/// 单位转换
pub fn convert(value: f64, from: Unit, to: Unit) -> f64 {
    let mm_value = value * from.to_mm_factor();
    mm_value / to.to_mm_factor()
}

/// 线性格式（用于显示长度）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LinearFormat {
    /// 科学计数法 (e.g. 2.5E+05)
    Scientific,
    /// 十进制 (e.g. 9.5)
    #[default]
    Decimal,
    /// 工程格式 (e.g. 7' 11.5")
    Engineering,
    /// 建筑格式 (e.g. 7'-9 1/8")
    Architectural,
    /// 分数格式 (e.g. 7 9/16)
    Fractional,
    /// 公制建筑格式 (DIN 406)
    ArchitecturalMetric,
}

/// 角度单位
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AngleUnit {
    /// 度
    #[default]
    Degrees,
    /// 弧度
    Radians,
    /// 百分度
    Gradians,
}

impl AngleUnit {
    /// 从弧度转换
    pub fn from_radians(&self, radians: f64) -> f64 {
        match self {
            AngleUnit::Degrees => radians.to_degrees(),
            AngleUnit::Radians => radians,
            AngleUnit::Gradians => radians * 200.0 / std::f64::consts::PI,
        }
    }

    /// 转换为弧度
    pub fn to_radians(&self, value: f64) -> f64 {
        match self {
            AngleUnit::Degrees => value.to_radians(),
            AngleUnit::Radians => value,
            AngleUnit::Gradians => value * std::f64::consts::PI / 200.0,
        }
    }

    /// 获取单位符号
    pub fn symbol(&self) -> &'static str {
        match self {
            AngleUnit::Degrees => "°",
            AngleUnit::Radians => "rad",
            AngleUnit::Gradians => "gon",
        }
    }
}

/// 角度显示格式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum AngleFormat {
    /// 十进制度 (e.g. 24.5°)
    #[default]
    DegreesDecimal,
    /// 度分秒 (e.g. 24°30'5")
    DegreesMinutesSeconds,
    /// 百分度 (e.g. 390.5 gon)
    Gradians,
    /// 弧度 (e.g. 1.57 rad)
    Radians,
    /// 测量员格式 (e.g. N 45° E)
    Surveyors,
}

/// 格式化长度值
pub fn format_linear(value: f64, unit: Unit, format: LinearFormat, precision: u8, show_unit: bool) -> String {
    let formatted = match format {
        LinearFormat::Scientific => format!("{:.prec$E}", value, prec = precision as usize),
        LinearFormat::Decimal => format!("{:.prec$}", value, prec = precision as usize),
        LinearFormat::Engineering => format_engineering(value, unit, precision),
        LinearFormat::Architectural => format_architectural(value, unit, precision),
        LinearFormat::Fractional => format_fractional(value, precision),
        LinearFormat::ArchitecturalMetric => format_architectural_metric(value, precision),
    };

    if show_unit {
        format!("{}{}", formatted, unit.symbol())
    } else {
        formatted
    }
}

/// 工程格式
fn format_engineering(value: f64, unit: Unit, precision: u8) -> String {
    // 转换为英尺和英寸
    let inches = if unit == Unit::Inch {
        value
    } else {
        convert(value, unit, Unit::Inch)
    };

    let feet = (inches / 12.0).floor();
    let remaining_inches = inches - feet * 12.0;

    if feet > 0.0 {
        format!("{}'-{:.prec$}\"", feet as i64, remaining_inches, prec = precision as usize)
    } else {
        format!("{:.prec$}\"", remaining_inches, prec = precision as usize)
    }
}

/// 建筑格式
fn format_architectural(value: f64, unit: Unit, precision: u8) -> String {
    let inches = if unit == Unit::Inch {
        value
    } else {
        convert(value, unit, Unit::Inch)
    };

    let feet = (inches / 12.0).floor();
    let remaining_inches = inches - feet * 12.0;
    let whole_inches = remaining_inches.floor();
    let fractional = remaining_inches - whole_inches;

    // 简化分数
    let (num, denom) = approximate_fraction(fractional, 1 << precision);

    if feet > 0.0 {
        if num > 0 {
            format!("{}'-{} {}/{}\"", feet as i64, whole_inches as i64, num, denom)
        } else if whole_inches > 0.0 {
            format!("{}'-{}\"", feet as i64, whole_inches as i64)
        } else {
            format!("{}'", feet as i64)
        }
    } else if whole_inches > 0.0 || num > 0 {
        if num > 0 {
            format!("{} {}/{}\"", whole_inches as i64, num, denom)
        } else {
            format!("{}\"", whole_inches as i64)
        }
    } else {
        "0\"".to_string()
    }
}

/// 分数格式
fn format_fractional(value: f64, precision: u8) -> String {
    let whole = value.floor();
    let fractional = value - whole;

    let (num, denom) = approximate_fraction(fractional, 1 << precision);

    if num > 0 {
        if whole > 0.0 {
            format!("{} {}/{}", whole as i64, num, denom)
        } else {
            format!("{}/{}", num, denom)
        }
    } else {
        format!("{}", whole as i64)
    }
}

/// 公制建筑格式
fn format_architectural_metric(value: f64, precision: u8) -> String {
    // DIN 406 格式：米.厘米⁵ (上标毫米)
    let meters = (value / 1000.0).floor();
    let remaining = value - meters * 1000.0;
    let centimeters = (remaining / 10.0).floor();
    let millimeters = remaining - centimeters * 10.0;

    if meters > 0.0 {
        format!(
            "{}.{:02}{}",
            meters as i64,
            centimeters as i64,
            superscript_number(millimeters as i64)
        )
    } else if centimeters > 0.0 {
        format!("{}{}", centimeters as i64, superscript_number(millimeters as i64))
    } else {
        format!("{:.prec$}", millimeters, prec = precision as usize)
    }
}

/// 将数字转换为上标
fn superscript_number(n: i64) -> String {
    if n == 0 {
        return String::new();
    }

    let superscripts = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let mut result = String::new();
    let mut num = n.abs();

    if num == 0 {
        return "⁰".to_string();
    }

    while num > 0 {
        result.insert(0, superscripts[(num % 10) as usize]);
        num /= 10;
    }

    result
}

/// 近似分数
fn approximate_fraction(value: f64, max_denom: i64) -> (i64, i64) {
    if value.abs() < 1e-10 {
        return (0, 1);
    }

    let mut best_num = 0i64;
    let mut best_denom = 1i64;
    let mut best_error = value.abs();

    for denom in 1..=max_denom {
        let num = (value * denom as f64).round() as i64;
        let error = (value - num as f64 / denom as f64).abs();

        if error < best_error {
            best_error = error;
            best_num = num;
            best_denom = denom;
        }

        if error < 1e-10 {
            break;
        }
    }

    // 简化分数
    let gcd = gcd(best_num.abs(), best_denom);
    (best_num / gcd, best_denom / gcd)
}

/// 最大公约数
fn gcd(a: i64, b: i64) -> i64 {
    if b == 0 {
        a
    } else {
        gcd(b, a % b)
    }
}

/// 格式化角度
pub fn format_angle(value: f64, format: AngleFormat, precision: u8) -> String {
    match format {
        AngleFormat::DegreesDecimal => {
            format!("{:.prec$}°", value.to_degrees(), prec = precision as usize)
        }
        AngleFormat::DegreesMinutesSeconds => {
            let degrees = value.to_degrees();
            let d = degrees.floor();
            let remaining = (degrees - d) * 60.0;
            let m = remaining.floor();
            let s = (remaining - m) * 60.0;
            format!("{}°{}'{}\"", d as i64, m as i64, s as i64)
        }
        AngleFormat::Gradians => {
            let gradians = value * 200.0 / std::f64::consts::PI;
            format!("{:.prec$}gon", gradians, prec = precision as usize)
        }
        AngleFormat::Radians => {
            format!("{:.prec$}rad", value, prec = precision as usize)
        }
        AngleFormat::Surveyors => {
            // 简化的测量员格式
            let degrees = value.to_degrees();
            let degrees = degrees.rem_euclid(360.0);

            let (ns, ew, angle) = if degrees <= 90.0 {
                ("N", "E", degrees)
            } else if degrees <= 180.0 {
                ("S", "E", 180.0 - degrees)
            } else if degrees <= 270.0 {
                ("S", "W", degrees - 180.0)
            } else {
                ("N", "W", 360.0 - degrees)
            };

            format!("{} {:.prec$}° {}", ns, angle, ew, prec = precision as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unit_conversion() {
        assert!((convert(1.0, Unit::Inch, Unit::Millimeter) - 25.4).abs() < 0.001);
        assert!((convert(1000.0, Unit::Millimeter, Unit::Meter) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_format_linear() {
        let result = format_linear(25.4, Unit::Millimeter, LinearFormat::Decimal, 2, true);
        assert_eq!(result, "25.40mm");
    }

    #[test]
    fn test_format_angle() {
        let result = format_angle(std::f64::consts::FRAC_PI_4, AngleFormat::DegreesDecimal, 1);
        assert_eq!(result, "45.0°");
    }
}
