//! ZCAD 文件格式处理
//!
//! 支持：
//! - `.zcad` 原生格式（基于SQLite）
//! - `.dxf` 导入/导出

pub mod document;
pub mod dxf_io;
pub mod error;
pub mod native;

pub use document::Document;
pub use error::FileError;

