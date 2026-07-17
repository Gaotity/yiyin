#![allow(
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    reason = "Tauri commands use framework-owned extractors and map application errors"
)]

use tauri::State;

use crate::{dto::BootstrapDto, error::CommandResult, state::AppState};

#[tauri::command]
pub fn bootstrap(state: State<'_, AppState>) -> CommandResult<BootstrapDto> {
    state
        .bootstrap
        .execute()
        .map(|snapshot| BootstrapDto::from(&snapshot))
        .map_err(Into::into)
}
