#![allow(
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    reason = "Tauri commands use framework-owned extractors and map application errors"
)]

use tauri::{WebviewWindow, Wry};

use crate::error::CommandResult;

#[tauri::command]
pub fn minimize_window(window: WebviewWindow<Wry>) -> CommandResult<()> {
    window
        .minimize()
        .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn close_window(window: WebviewWindow<Wry>) -> CommandResult<()> {
    window
        .close()
        .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
    Ok(())
}
