#![allow(
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    reason = "Tauri commands use framework-owned extractors and map application errors"
)]

use tauri::State;

use crate::{
    dto::{PublicConfigDto, UpdateConfigRequestDto},
    error::CommandResult,
    state::AppState,
};

#[tauri::command]
pub fn update_config(
    state: State<'_, AppState>,
    request: UpdateConfigRequestDto,
) -> CommandResult<PublicConfigDto> {
    let current = state.config.load()?;
    let next = request.config.apply_to(&current)?;
    state
        .update_config
        .execute(next)
        .map(|config| (&config).into())
        .map_err(Into::into)
}

#[tauri::command]
pub fn reset_config(state: State<'_, AppState>) -> CommandResult<PublicConfigDto> {
    let config = state
        .reset_config
        .execute()
        .map_err(crate::dto::CommandErrorDto::from)?;
    let config = state
        .set_output_directory
        .execute(config.output.clone())
        .map_err(crate::dto::CommandErrorDto::from)?;
    Ok((&config).into())
}
