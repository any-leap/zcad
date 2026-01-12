//! User-friendly error display
//!
//! Provides separate formatting for end users vs developers.

use crate::code::ErrorCode;
use crate::error::ZcadError;

/// Trait for user-friendly error messages
pub trait UserFriendly {
    /// Get a short message suitable for displaying to end users
    fn user_message(&self) -> String;

    /// Get a suggestion for how to resolve the error (if any)
    fn suggestion(&self) -> Option<String>;

    /// Get detailed technical information (for developers/logs)
    fn technical_details(&self) -> String;

    /// Get a help URL (if available)
    fn help_url(&self) -> Option<String> {
        None
    }
}

impl UserFriendly for ZcadError {
    fn user_message(&self) -> String {
        // Provide localized, user-friendly messages
        match self.code() {
            // File errors
            ErrorCode::FileNotFound => "找不到指定的文件".to_string(),
            ErrorCode::FileReadError => "无法读取文件".to_string(),
            ErrorCode::FileWriteError => "无法保存文件".to_string(),
            ErrorCode::InvalidFileFormat => "文件格式无效".to_string(),
            ErrorCode::UnsupportedVersion => "不支持的文件版本".to_string(),
            ErrorCode::FileCorrupted => "文件已损坏".to_string(),

            // Geometry errors
            ErrorCode::InvalidGeometry => "几何形状无效".to_string(),
            ErrorCode::DegenerateGeometry => "几何形状退化（如零长度线）".to_string(),
            ErrorCode::BooleanFailed => "布尔运算失败".to_string(),
            ErrorCode::ConstraintSolverFailed => "约束求解失败".to_string(),

            // Module errors
            ErrorCode::ModuleNotFound => "模块未找到".to_string(),
            ErrorCode::DependencyNotSatisfied => "模块依赖未满足".to_string(),
            ErrorCode::VersionIncompatible => "模块版本不兼容".to_string(),

            // MCAD errors
            ErrorCode::PartNotFound => "零件未找到".to_string(),
            ErrorCode::FeatureFailed => "特征操作失败".to_string(),
            ErrorCode::SketchError => "草图错误".to_string(),
            ErrorCode::AssemblyConstraintFailed => "装配约束失败".to_string(),

            // BIM errors
            ErrorCode::BimElementNotFound => "BIM 元素未找到".to_string(),
            ErrorCode::ConnectionFailed => "连接创建失败".to_string(),
            ErrorCode::SpatialError => "空间关系错误".to_string(),

            // EDA errors
            ErrorCode::ComponentNotFound => "元器件未找到".to_string(),
            ErrorCode::DrcViolation => "设计规则检查 (DRC) 违规".to_string(),
            ErrorCode::ErcViolation => "电气规则检查 (ERC) 违规".to_string(),
            ErrorCode::NetError => "网络连接错误".to_string(),

            // General errors
            ErrorCode::Cancelled => "操作已取消".to_string(),
            ErrorCode::Timeout => "操作超时".to_string(),
            ErrorCode::InvalidArgument => "参数无效".to_string(),
            ErrorCode::NotSupported => "不支持此操作".to_string(),
            ErrorCode::PermissionDenied => "权限不足".to_string(),

            // Default: use technical message
            _ => self.message().to_string(),
        }
    }

    fn suggestion(&self) -> Option<String> {
        match self.code() {
            // File errors
            ErrorCode::FileNotFound => Some("请检查文件路径是否正确".to_string()),
            ErrorCode::FileReadError => Some("请确保文件未被其他程序占用".to_string()),
            ErrorCode::FileWriteError => Some("请检查磁盘空间和写入权限".to_string()),
            ErrorCode::InvalidFileFormat => Some("请使用正确格式的文件".to_string()),
            ErrorCode::UnsupportedVersion => Some("请使用新版本的 ZCAD 打开此文件".to_string()),
            ErrorCode::FileCorrupted => Some("请尝试从备份恢复文件".to_string()),

            // Geometry errors
            ErrorCode::InvalidGeometry => Some("请检查输入的几何参数".to_string()),
            ErrorCode::BooleanFailed => Some("请尝试简化几何形状或调整公差".to_string()),
            ErrorCode::ConstraintSolverFailed => Some("请减少约束或检查是否存在冲突".to_string()),

            // Module errors
            ErrorCode::ModuleNotFound => Some("请确保模块已正确安装".to_string()),
            ErrorCode::DependencyNotSatisfied => Some("请安装所需的依赖模块".to_string()),

            // MCAD errors
            ErrorCode::FeatureFailed => Some("请检查草图是否闭合，参数是否合理".to_string()),
            ErrorCode::SketchError => Some("请确保草图完全约束且无冲突".to_string()),
            ErrorCode::AssemblyConstraintFailed => Some("请检查约束是否过度定义".to_string()),

            // BIM errors
            ErrorCode::ConnectionFailed => Some("请确保构件正确对齐".to_string()),

            // EDA errors
            ErrorCode::DrcViolation => Some("请调整走线宽度、间距或过孔大小".to_string()),
            ErrorCode::ErcViolation => Some("请检查电路连接是否正确".to_string()),
            ErrorCode::ComponentNotFound => Some("请检查元器件库是否已加载".to_string()),

            _ => None,
        }
    }

    fn technical_details(&self) -> String {
        format!("{:#?}", self)
    }

    fn help_url(&self) -> Option<String> {
        // Could link to documentation based on error code
        let code = self.code().as_u16();
        Some(format!("https://docs.zcad.dev/errors/E{:04}", code))
    }
}

/// Format an error for display in a dialog
pub fn format_error_dialog(error: &ZcadError) -> String {
    let mut output = String::new();

    // Title
    output.push_str(&format!("❌ {}\n\n", error.user_message()));

    // Technical message (if different)
    if error.message() != error.user_message() {
        output.push_str(&format!("详情: {}\n\n", error.message()));
    }

    // Suggestion
    if let Some(suggestion) = error.suggestion() {
        output.push_str(&format!("💡 建议: {}\n\n", suggestion));
    }

    // Error code
    output.push_str(&format!("错误代码: {}", error.code()));

    output
}

/// Format an error for logging
pub fn format_error_log(error: &ZcadError) -> String {
    let mut output = String::new();

    // Error code and message
    output.push_str(&format!("[{}] {}", error.code(), error.message()));

    // Context chain
    for ctx in error.context_chain() {
        output.push_str(&format!("\n  └─ {}", ctx));
    }

    // Location
    if let Some(loc) = error.location() {
        output.push_str(&format!("\n  at {}", loc));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message() {
        let err = ZcadError::new(ErrorCode::FileNotFound, "config.json");
        assert_eq!(err.user_message(), "找不到指定的文件");
    }

    #[test]
    fn test_suggestion() {
        let err = ZcadError::new(ErrorCode::DrcViolation, "Track too narrow");
        let suggestion = err.suggestion();
        assert!(suggestion.is_some());
        assert!(suggestion.unwrap().contains("走线"));
    }

    #[test]
    fn test_format_error_dialog() {
        let err = ZcadError::new(ErrorCode::FileNotFound, "project.zcad");
        let dialog = format_error_dialog(&err);
        assert!(dialog.contains("找不到"));
        assert!(dialog.contains("E2001"));
    }
}
