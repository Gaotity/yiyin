#![allow(
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    reason = "Tauri commands use framework-owned extractors and map application errors"
)]

use std::{collections::BTreeSet, path::PathBuf};

use tauri::{AppHandle, State, Wry};
use tauri_plugin_dialog::DialogExt;
use yiyin_domain::{ResourceId, ResourceKind, TaskId};

use crate::{
    dto::{
        MetadataDto, RegisterFontRequestDto, ResourceDescriptorDto, ResourceIdRequestDto,
        TaskDescriptorDto, TaskIdRequestDto,
    },
    error::CommandResult,
    state::AppState,
};

use super::run_blocking;

#[tauri::command]
pub async fn choose_images(
    app: AppHandle<Wry>,
    state: State<'_, AppState>,
) -> CommandResult<Vec<TaskDescriptorDto>> {
    let selected = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Images", &["jpg", "jpeg", "png", "webp"])
            .blocking_pick_files()
    })
    .await
    .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?;
    let Some(selected) = selected else {
        return Ok(task_dtos(&state));
    };
    let paths = selected
        .into_iter()
        .map(|file| {
            file.into_path()
                .map_err(|_| yiyin_application::ApplicationError::file_invalid())
        })
        .collect::<Result<Vec<_>, _>>()?;
    register_image_paths(&state, &paths)
}

pub(crate) fn register_image_paths(
    state: &AppState,
    paths: &[PathBuf],
) -> CommandResult<Vec<TaskDescriptorDto>> {
    let existing = state
        .tasks
        .snapshot()
        .into_iter()
        .map(|task| task.id().as_str().to_owned())
        .collect::<BTreeSet<_>>();
    let snapshots = state.register_images.execute(paths)?;
    let new_ids = snapshots
        .iter()
        .filter(|task| !existing.contains(task.id().as_str()))
        .map(|task| TaskId::try_from(task.id().as_str()))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| {
            yiyin_application::ApplicationError::internal("registered task id became invalid")
        })?;
    state.start_tasks.execute_quick_output(&new_ids)?;
    Ok(task_dtos(state))
}

#[tauri::command]
pub async fn register_font(
    app: AppHandle<Wry>,
    state: State<'_, AppState>,
    request: RegisterFontRequestDto,
) -> CommandResult<ResourceDescriptorDto> {
    let selected = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Fonts", &["ttf", "otf"])
            .blocking_pick_file()
    })
    .await
    .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?
    .ok_or_else(yiyin_application::ApplicationError::cancelled)?
    .into_path()
    .map_err(|_| yiyin_application::ApplicationError::file_invalid())?;
    state
        .register_font
        .execute(&request.name, &selected)
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
pub fn remove_font(
    state: State<'_, AppState>,
    request: ResourceIdRequestDto,
) -> CommandResult<Vec<ResourceDescriptorDto>> {
    let id = ResourceId::try_from(request.id).map_err(|_| {
        yiyin_application::ApplicationError::invalid_request("The resource ID is invalid.")
    })?;
    state.remove_font.execute(&id)?;
    Ok(state
        .resources
        .snapshot()
        .iter()
        .filter(|record| record.kind() == ResourceKind::Font)
        .map(yiyin_application::ResourceSnapshot::from_record)
        .map(Into::into)
        .collect())
}

#[tauri::command]
pub async fn register_overlay(
    app: AppHandle<Wry>,
    state: State<'_, AppState>,
) -> CommandResult<ResourceDescriptorDto> {
    let selected = tauri::async_runtime::spawn_blocking(move || {
        app.dialog()
            .file()
            .add_filter("Images", &["jpg", "jpeg", "png", "webp"])
            .blocking_pick_file()
    })
    .await
    .map_err(|error| yiyin_application::ApplicationError::internal(error.to_string()))?
    .ok_or_else(yiyin_application::ApplicationError::cancelled)?
    .into_path()
    .map_err(|_| yiyin_application::ApplicationError::file_invalid())?;
    state
        .register_overlay
        .execute(&selected)
        .map(Into::into)
        .map_err(Into::into)
}

#[tauri::command]
pub async fn read_task_exif(
    app: AppHandle<Wry>,
    request: TaskIdRequestDto,
) -> CommandResult<Option<MetadataDto>> {
    run_blocking(app, move |state| {
        let id = TaskId::try_from(request.id).map_err(|_| {
            yiyin_application::ApplicationError::invalid_request("The task ID is invalid.")
        })?;
        state
            .read_task_exif
            .execute(&id)
            .map(|metadata| metadata.as_ref().map(Into::into))
            .map_err(Into::into)
    })
    .await
}

fn task_dtos(state: &AppState) -> Vec<TaskDescriptorDto> {
    state.tasks.snapshot().iter().map(Into::into).collect()
}
