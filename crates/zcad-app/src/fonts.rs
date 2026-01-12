//! 字体配置模块
//! 
//! 提供中英文字体支持和多字号配置

use eframe::egui::{self, FontData, FontDefinitions, FontFamily, FontId, TextStyle};
use std::sync::Arc;
use tracing::info;

/// 设置中文字体支持和字体样式
pub fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = FontDefinitions::default();
    
    // 中文字体路径（按优先级排序）
    let chinese_font_paths = [
        // Windows - 优先使用微软雅黑
        ("C:\\Windows\\Fonts\\msyh.ttc", "Microsoft YaHei"),
        ("C:\\Windows\\Fonts\\msyhbd.ttc", "Microsoft YaHei Bold"),
        ("C:\\Windows\\Fonts\\simsun.ttc", "SimSun"),
        // macOS
        ("/System/Library/Fonts/PingFang.ttc", "PingFang SC"),
        ("/System/Library/Fonts/STHeiti Light.ttc", "STHeiti"),
        ("/System/Library/Fonts/Hiragino Sans GB.ttc", "Hiragino"),
        // Linux
        ("/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc", "Noto Sans CJK"),
        ("/usr/share/fonts/truetype/wqy/wqy-microhei.ttc", "WenQuanYi"),
        ("/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf", "Droid Sans"),
    ];
    
    let mut chinese_font_loaded = false;
    
    for (path, name) in chinese_font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "chinese".to_owned(),
                Arc::new(FontData::from_owned(font_data)),
            );
            
            // 将中文字体添加到字体族
            fonts.families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese".to_owned());
            fonts.families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "chinese".to_owned());
            
            info!("Loaded Chinese font: {} from {}", name, path);
            chinese_font_loaded = true;
            break;
        }
    }
    
    if !chinese_font_loaded {
        info!("Warning: No Chinese font found, UI may not display Chinese characters correctly");
    }
    
    ctx.set_fonts(fonts);
    
    // 设置自定义文本样式
    setup_text_styles(ctx);
}

/// 设置文本样式（字号配置）
fn setup_text_styles(ctx: &egui::Context) {
    use egui::FontFamily::Proportional;
    use egui::FontFamily::Monospace;
    
    let mut style = (*ctx.style()).clone();
    
    // 自定义字号配置 - 更加清晰的层次
    style.text_styles = [
        // 小号文字 - 状态栏、辅助信息
        (TextStyle::Small, FontId::new(11.0, Proportional)),
        // 正文 - 默认大小
        (TextStyle::Body, FontId::new(13.0, Proportional)),
        // 按钮文字
        (TextStyle::Button, FontId::new(12.0, Proportional)),
        // 标题
        (TextStyle::Heading, FontId::new(16.0, Proportional)),
        // 等宽字体 - 坐标、命令行
        (TextStyle::Monospace, FontId::new(12.0, Monospace)),
    ].into();
    
    ctx.set_style(style);
}
