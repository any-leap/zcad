//! 中文字体配置模块

use eframe::egui;
use tracing::info;

/// 设置中文字体支持
pub fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    
    // 尝试从系统加载中文字体
    let font_paths = [
        // macOS
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        // Linux
        "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
        "/usr/share/fonts/truetype/droid/DroidSansFallbackFull.ttf",
        // Windows
        "C:\\Windows\\Fonts\\msyh.ttc",
        "C:\\Windows\\Fonts\\simsun.ttc",
    ];
    
    for path in font_paths {
        if let Ok(font_data) = std::fs::read(path) {
            fonts.font_data.insert(
                "chinese".to_owned(),
                std::sync::Arc::new(egui::FontData::from_owned(font_data)),
            );
            
            // 将中文字体添加到字体族（放在最前面以优先使用）
            fonts.families
                .entry(egui::FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese".to_owned());
            fonts.families
                .entry(egui::FontFamily::Monospace)
                .or_default()
                .insert(0, "chinese".to_owned());
            
            info!("Loaded Chinese font from: {}", path);
            break;
        }
    }
    
    ctx.set_fonts(fonts);
}
