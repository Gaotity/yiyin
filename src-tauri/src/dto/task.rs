use serde::{Deserialize, Serialize};
use yiyin_application::TaskSnapshot;
use yiyin_domain::{CancellationReason, TaskState, TaskStatus};

use super::ResourceDescriptorDto;

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub enum TaskStateDto {
    Registered,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl From<TaskState> for TaskStateDto {
    fn from(state: TaskState) -> Self {
        match state {
            TaskState::Registered => Self::Registered,
            TaskState::Queued => Self::Queued,
            TaskState::Running => Self::Running,
            TaskState::Completed => Self::Completed,
            TaskState::Failed => Self::Failed,
            TaskState::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub enum CancellationReasonDto {
    User,
    PreviewSuperseded,
    Cleared,
    Shutdown,
}

impl From<CancellationReason> for CancellationReasonDto {
    fn from(reason: CancellationReason) -> Self {
        match reason {
            CancellationReason::User => Self::User,
            CancellationReason::PreviewSuperseded => Self::PreviewSuperseded,
            CancellationReason::Cleared => Self::Cleared,
            CancellationReason::Shutdown => Self::Shutdown,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct TaskDescriptorDto {
    pub id: String,
    pub display_name: String,
    pub state: TaskStateDto,
    pub progress: u8,
    pub preview: bool,
    pub resource: Option<ResourceDescriptorDto>,
}

impl From<&TaskSnapshot> for TaskDescriptorDto {
    fn from(task: &TaskSnapshot) -> Self {
        Self {
            id: task.id().as_str().to_owned(),
            display_name: task.display_name().to_owned(),
            state: task.state().into(),
            progress: task.progress(),
            preview: task.is_preview(),
            resource: task.resource().map(Into::into),
        }
    }
}

impl From<TaskSnapshot> for TaskDescriptorDto {
    fn from(task: TaskSnapshot) -> Self {
        Self::from(&task)
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct TaskStatusEventDto {
    pub task_id: String,
    pub state: TaskStateDto,
    pub progress: u8,
    pub preview: bool,
    pub cancellation_reason: Option<CancellationReasonDto>,
}

impl From<TaskStatus> for TaskStatusEventDto {
    fn from(status: TaskStatus) -> Self {
        Self {
            task_id: status.task_id().as_str().to_owned(),
            state: status.state().into(),
            progress: status.stage().map_or(0, yiyin_domain::RenderStage::percent),
            preview: status.is_preview(),
            cancellation_reason: status.cancellation_reason().map(Into::into),
        }
    }
}

impl From<&TaskDescriptorDto> for TaskStatusEventDto {
    fn from(task: &TaskDescriptorDto) -> Self {
        Self {
            task_id: task.id.clone(),
            state: task.state,
            progress: task.progress,
            preview: task.preview,
            cancellation_reason: None,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct TaskIdRequestDto {
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct TaskIdsRequestDto {
    pub ids: Vec<String>,
}
