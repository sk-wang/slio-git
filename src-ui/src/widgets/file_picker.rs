//! File picker widget using rfd crate
//!
//! Provides native file dialog functionality

use rfd::FileDialog;
use std::path::PathBuf;

/// Open a folder selection dialog
pub fn pick_folder() -> Option<PathBuf> {
    FileDialog::new()
        .set_title("选择一个 Git 仓库")
        .set_directory(
            std::env::current_dir()
                .ok()
                .or_else(dirs::home_dir)
                .unwrap_or_else(|| PathBuf::from("/")),
        )
        .pick_folder()
}

/// Open a file selection dialog
pub fn pick_file() -> Option<PathBuf> {
    FileDialog::new().set_title("选择文件").pick_file()
}

/// Save file dialog
pub fn save_file(default_name: &str) -> Option<PathBuf> {
    FileDialog::new()
        .set_title("保存文件")
        .set_file_name(default_name)
        .save_file()
}
