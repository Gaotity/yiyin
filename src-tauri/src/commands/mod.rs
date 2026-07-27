pub mod bootstrap;
pub mod config;
pub mod native;
pub mod resources;
pub mod tasks;
pub mod window;

use tauri::{AppHandle, Manager, Wry};

use crate::{dto::CommandErrorDto, error::CommandResult, state::AppState};

pub(super) async fn run_blocking<T, F>(app: AppHandle<Wry>, operation: F) -> CommandResult<T>
where
    T: Send + 'static,
    F: FnOnce(&AppState) -> CommandResult<T> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<AppState>();
        operation(&state)
    })
    .await
    .map_err(|error| {
        CommandErrorDto::from(yiyin_application::ApplicationError::internal(
            error.to_string(),
        ))
    })?
}
