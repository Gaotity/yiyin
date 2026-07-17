#![allow(
    clippy::missing_errors_doc,
    clippy::needless_pass_by_value,
    reason = "Tauri commands use framework-owned extractors and map application errors"
)]

use tauri::{AppHandle, State, Wry};
use yiyin_domain::TaskId;

use crate::{
    dto::{TaskDescriptorDto, TaskIdRequestDto, TaskIdsRequestDto},
    error::CommandResult,
    state::AppState,
};

use super::run_blocking;

#[tauri::command]
pub async fn start_tasks(
    app: AppHandle<Wry>,
    request: TaskIdsRequestDto,
) -> CommandResult<Vec<TaskDescriptorDto>> {
    run_blocking(app, move |state| {
        let ids = task_ids(request.ids)?;
        state
            .start_tasks
            .execute(&ids)
            .map(|tasks| tasks.iter().map(Into::into).collect())
            .map_err(Into::into)
    })
    .await
}

#[tauri::command]
pub async fn preview_task(
    app: AppHandle<Wry>,
    request: TaskIdRequestDto,
) -> CommandResult<Vec<TaskDescriptorDto>> {
    run_blocking(app, move |state| {
        let id = task_id(request.id)?;
        state
            .preview_task
            .execute(&id)
            .map(|tasks| tasks.iter().map(Into::into).collect())
            .map_err(Into::into)
    })
    .await
}

#[tauri::command]
pub fn cancel_task(
    state: State<'_, AppState>,
    request: TaskIdRequestDto,
) -> CommandResult<Vec<TaskDescriptorDto>> {
    let id = task_id(request.id)?;
    state
        .cancel_task
        .execute(&id)
        .map(|tasks| tasks.iter().map(Into::into).collect())
        .map_err(Into::into)
}

#[tauri::command]
pub fn clear_tasks(state: State<'_, AppState>) -> CommandResult<Vec<TaskDescriptorDto>> {
    state
        .clear_tasks
        .execute()
        .map(|tasks| tasks.iter().map(Into::into).collect())
        .map_err(Into::into)
}

fn task_ids(values: Vec<String>) -> CommandResult<Vec<TaskId>> {
    values.into_iter().map(task_id).collect()
}

fn task_id(value: String) -> CommandResult<TaskId> {
    TaskId::try_from(value)
        .map_err(|_| {
            yiyin_application::ApplicationError::invalid_request("The task ID is invalid.")
        })
        .map_err(Into::into)
}
