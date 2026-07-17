#![allow(
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    reason = "Tauri commands use framework-owned extractors and map application errors"
)]

use tauri::{AppHandle, State, Wry};
use yiyin_domain::{Config, FieldContentKind, ResourceKind};

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
    validate_resource_references(&next, &state)?;
    state
        .update_config
        .execute(next)
        .map(|config| (&config).into())
        .map_err(Into::into)
}

#[tauri::command]
pub fn reset_config(
    app: AppHandle<Wry>,
    state: State<'_, AppState>,
) -> CommandResult<PublicConfigDto> {
    let config = state
        .reset_config
        .execute()
        .map_err(crate::dto::CommandErrorDto::from)?;
    crate::commands::native::reset_output_root(&app, &state, &config.output)?;
    Ok((&config).into())
}

fn validate_resource_references(config: &Config, state: &AppState) -> CommandResult<()> {
    for id in config
        .temp_fields
        .iter()
        .chain(&config.custom_temp_fields)
        .filter(|field| field.content_kind() == FieldContentKind::Image)
        .flat_map(|field| [field.dark_image(), field.light_image()])
        .flatten()
    {
        let valid = state.resources.resolve(id).is_ok_and(|record| {
            matches!(
                record.kind(),
                ResourceKind::BundledAsset | ResourceKind::Overlay
            )
        });
        if !valid {
            return Err(yiyin_application::ApplicationError::config_invalid().into());
        }
    }
    Ok(())
}
