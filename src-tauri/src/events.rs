use tauri::{AppHandle, Emitter, Wry};
use yiyin_application::TaskEventSink;
use yiyin_domain::TaskStatus;

use crate::dto::TaskStatusEventDto;

pub const TASK_STATUS_EVENT: &str = "task-status";

pub struct TauriTaskEventSink {
    app: AppHandle<Wry>,
}

impl TauriTaskEventSink {
    #[must_use]
    pub const fn new(app: AppHandle<Wry>) -> Self {
        Self { app }
    }
}

impl TaskEventSink for TauriTaskEventSink {
    fn publish(&self, status: TaskStatus) {
        if let Err(error) = self
            .app
            .emit(TASK_STATUS_EVENT, TaskStatusEventDto::from(status))
        {
            log::error!("failed to emit task status: {error}");
        }
    }
}
