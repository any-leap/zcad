//! ZCAD 主应用程序入口
//! 使用 eframe 作为应用框架，提供完整的 egui + wgpu 集成

mod app;
mod camera;
mod file_ops;
mod fonts;
mod history_ops;
mod input;
mod rendering;
mod theme;
mod ui;
mod ui_state;

use anyhow::Result;
use eframe::egui;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use app::ZcadApp;
use fonts::setup_chinese_fonts;

fn main() -> Result<()> {
    // 初始化日志
    tracing::subscriber::set_global_default(
        FmtSubscriber::builder().with_max_level(Level::INFO).finish()
    )?;
    
    info!("Starting ZCAD...");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])
            .with_title("ZCAD"),
        ..Default::default()
    };

    eframe::run_native(
        "ZCAD",
        native_options,
        Box::new(|cc| {
            // 加载中文字体
            setup_chinese_fonts(&cc.egui_ctx);
            Ok(Box::new(ZcadApp::default()))
        }),
    ).map_err(|e| anyhow::anyhow!("eframe error: {}", e))?;

    Ok(())
}
