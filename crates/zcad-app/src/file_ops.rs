//! 文件操作模块

use std::path::PathBuf;
use tracing::info;
use zcad_file::Document;

use crate::ui_state::UiStateManager;

/// 文件操作类型
#[derive(Debug, Clone)]
pub enum FileOperation {
    Open(PathBuf),
    Save(PathBuf),
}

/// 文件操作处理器
pub struct FileOperations {
    /// 待处理的文件操作
    pending_op: Option<FileOperation>,
}

impl Default for FileOperations {
    fn default() -> Self {
        Self::new()
    }
}

impl FileOperations {
    pub fn new() -> Self {
        Self { pending_op: None }
    }

    /// 设置待处理的文件操作
    pub fn set_pending(&mut self, op: FileOperation) {
        self.pending_op = Some(op);
    }

    /// 获取并清除待处理的操作
    pub fn take_pending(&mut self) -> Option<FileOperation> {
        self.pending_op.take()
    }

    /// 打开文件对话框 - 打开文件
    pub fn show_open_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("ZCAD Files", &["zcad"])
            .add_filter("DXF Files", &["dxf"])
            .add_filter("All Files", &["*"])
            .set_title("打开文件")
            .pick_file()
        {
            self.pending_op = Some(FileOperation::Open(path));
        }
    }

    /// 打开文件对话框 - 保存文件
    pub fn show_save_dialog(&mut self, current_path: Option<&std::path::Path>) {
        let mut dialog = rfd::FileDialog::new()
            .add_filter("ZCAD Files", &["zcad"])
            .add_filter("DXF Files", &["dxf"])
            .set_title("保存文件");

        // 如果已有文件名，使用它
        if let Some(path) = current_path {
            if let Some(file_name) = path.file_name() {
                dialog = dialog.set_file_name(file_name.to_string_lossy().as_ref());
            }
        }

        if let Some(path) = dialog.save_file() {
            self.pending_op = Some(FileOperation::Save(path));
        }
    }

    /// 处理文件操作
    pub fn process<U: UiStateManager>(
        &mut self,
        document: &mut Document,
        ui_state: &mut U,
        zoom_to_fit: impl FnOnce(),
    ) {
        if let Some(op) = self.pending_op.take() {
            match op {
                FileOperation::Open(path) => {
                    match Document::open(&path) {
                        Ok(doc) => {
                            *document = doc;
                            ui_state.clear_selection();
                            zoom_to_fit();
                            ui_state.set_status_message(format!("已打开: {}", path.display()));
                            info!("Opened file: {}", path.display());
                        }
                        Err(e) => {
                            ui_state.set_status_message(format!("打开失败: {}", e));
                            tracing::error!("Failed to open file: {}", e);
                        }
                    }
                }
                FileOperation::Save(path) => {
                    match document.save_as(&path) {
                        Ok(_) => {
                            ui_state.set_status_message(format!("已保存: {}", path.display()));
                            info!("Saved file: {}", path.display());
                        }
                        Err(e) => {
                            ui_state.set_status_message(format!("保存失败: {}", e));
                            tracing::error!("Failed to save file: {}", e);
                        }
                    }
                }
            }
        }
    }

    /// 快速保存（已有路径）
    pub fn quick_save<U: UiStateManager>(&mut self, document: &mut Document, ui_state: &mut U) {
        if document.file_path().is_some() {
            match document.save() {
                Ok(_) => {
                    ui_state.set_status_message("已保存".to_string());
                    info!("Quick saved file");
                }
                Err(e) => {
                    ui_state.set_status_message(format!("保存失败: {}", e));
                    tracing::error!("Failed to quick save: {}", e);
                }
            }
        } else {
            // 没有路径，显示另存为对话框
            self.show_save_dialog(None);
        }
    }
}
