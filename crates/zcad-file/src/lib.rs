//! ZCAD 文件格式处理
//!
//! 支持：
//! - `.zcad` 原生格式（基于SQLite）
//! - `.dxf` 导入/导出
//! - `.dwg` 导入（通过 LibreDWG）
//! - SVG/PDF 导出

pub mod document;
pub mod dxf_io;
pub mod dxf_raw;
#[cfg(feature = "dwg")]
pub mod dwg_io;
pub mod error;
pub mod export;
pub mod native;

pub use document::Document;
pub use error::FileError;
pub use export::{ExportFormat, PageSetup, PaperSize, Orientation, SvgExporter, PdfExporter, export_entities};

// 原始 DXF 解析器（用于完整的 Layout/Viewport 支持）
pub use dxf_raw::{DxfRawParser, DxfLayout, DxfViewport, DxfWriter, parse_layouts, parse_viewports};

