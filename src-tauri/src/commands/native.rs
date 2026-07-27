#![allow(
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    reason = "Tauri commands use framework-owned extractors and map application errors"
)]

use std::path::{Path, PathBuf};

use tauri::{AppHandle, Manager, State, Wry};
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

use crate::{
    dto::{ExternalDestinationRequestDto, PublicConfigDto},
    error::CommandResult,
    native::external_url,
    state::AppState,
};

#[tauri::command]
pub async fn choose_output_directory(
    app: AppHandle<Wry>,
    state: State<'_, AppState>,
) -> CommandResult<PublicConfigDto> {
    let selected =
        tauri::async_runtime::spawn_blocking(move || app.dialog().file().blocking_pick_folder())
            .await
            .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
    let Some(selected) = selected else {
        let config = state.config.load()?;
        return Ok((&config).into());
    };
    let selected = selected
        .into_path()
        .map_err(|_| yiyin_application::ApplicationError::file_invalid())?;
    state.output.set_root(selected.clone())?;
    let mut config = state.config.load()?;
    config.output = output_string(&selected)?;
    state
        .update_config
        .execute(config)
        .map(|config| (&config).into())
        .map_err(Into::into)
}

#[tauri::command]
pub fn open_output_directory(app: AppHandle<Wry>, state: State<'_, AppState>) -> CommandResult<()> {
    let root = state.output.root()?;
    let root = output_string(&root)?;
    app.opener()
        .open_path(root, None::<&str>)
        .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
    Ok(())
}

#[tauri::command]
pub fn open_external_url(
    app: AppHandle<Wry>,
    request: ExternalDestinationRequestDto,
) -> CommandResult<()> {
    app.opener()
        .open_url(external_url(request.destination), None::<&str>)
        .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
    Ok(())
}

pub(crate) fn reset_output_root(
    app: &AppHandle<Wry>,
    state: &AppState,
    configured: &str,
) -> CommandResult<()> {
    let configured = PathBuf::from(configured);
    let root = if configured.is_absolute() {
        configured
    } else {
        app.path()
            .home_dir()
            .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?
            .join(configured)
    };
    state.output.set_root(root).map_err(Into::into)
}

fn output_string(value: &Path) -> CommandResult<String> {
    value
        .to_str()
        .map(str::to_owned)
        .ok_or_else(|| yiyin_application::ApplicationError::file_invalid().into())
}
