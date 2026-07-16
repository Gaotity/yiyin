use yiyin_domain::{
    Config, ImageDensity, ImageDimensions, ResourceId, ResourceKind, TaskId, TaskState,
};

use crate::ResourceRecord;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ImportOutcome {
    warnings: Vec<String>,
}

impl ImportOutcome {
    #[must_use]
    pub const fn clean() -> Self {
        Self {
            warnings: Vec::new(),
        }
    }

    #[must_use]
    pub const fn with_warnings(warnings: Vec<String>) -> Self {
        Self { warnings }
    }

    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceSnapshot {
    id: ResourceId,
    kind: ResourceKind,
    display_name: String,
}

impl ResourceSnapshot {
    #[must_use]
    pub fn from_record(record: &ResourceRecord) -> Self {
        Self {
            id: record.id().clone(),
            kind: record.kind(),
            display_name: record.display_name().to_owned(),
        }
    }

    #[must_use]
    pub const fn id(&self) -> &ResourceId {
        &self.id
    }

    #[must_use]
    pub const fn kind(&self) -> ResourceKind {
        self.kind
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegisteredTask {
    id: TaskId,
    input: ResourceId,
    display_name: String,
    dimensions: ImageDimensions,
    density: Option<ImageDensity>,
}

impl RegisteredTask {
    #[must_use]
    pub fn new(
        id: TaskId,
        input: ResourceId,
        display_name: impl Into<String>,
        dimensions: ImageDimensions,
        density: Option<ImageDensity>,
    ) -> Self {
        Self {
            id,
            input,
            display_name: display_name.into(),
            dimensions,
            density,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &TaskId {
        &self.id
    }

    #[must_use]
    pub const fn input(&self) -> &ResourceId {
        &self.input
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    #[must_use]
    pub const fn dimensions(&self) -> ImageDimensions {
        self.dimensions
    }

    #[must_use]
    pub const fn density(&self) -> Option<ImageDensity> {
        self.density
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskSnapshot {
    id: TaskId,
    display_name: String,
    state: TaskState,
    progress: u8,
    preview: bool,
}

impl TaskSnapshot {
    #[must_use]
    pub fn from_registered(task: &RegisteredTask) -> Self {
        Self {
            id: task.id.clone(),
            display_name: task.display_name.clone(),
            state: TaskState::Registered,
            progress: 0,
            preview: false,
        }
    }

    #[must_use]
    pub fn new(
        id: TaskId,
        display_name: impl Into<String>,
        state: TaskState,
        progress: u8,
        preview: bool,
    ) -> Self {
        Self {
            id,
            display_name: display_name.into(),
            state,
            progress,
            preview,
        }
    }

    #[must_use]
    pub const fn id(&self) -> &TaskId {
        &self.id
    }

    #[must_use]
    pub fn display_name(&self) -> &str {
        &self.display_name
    }

    #[must_use]
    pub const fn state(&self) -> TaskState {
        self.state
    }

    #[must_use]
    pub const fn progress(&self) -> u8 {
        self.progress
    }

    #[must_use]
    pub const fn is_preview(&self) -> bool {
        self.preview
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderResult {
    task_id: TaskId,
    resource: ResourceSnapshot,
}

impl RenderResult {
    #[must_use]
    pub const fn new(task_id: TaskId, resource: ResourceSnapshot) -> Self {
        Self { task_id, resource }
    }

    #[must_use]
    pub const fn task_id(&self) -> &TaskId {
        &self.task_id
    }

    #[must_use]
    pub const fn resource(&self) -> &ResourceSnapshot {
        &self.resource
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BootstrapSnapshot {
    config: Config,
    resources: Vec<ResourceSnapshot>,
    tasks: Vec<TaskSnapshot>,
    warnings: Vec<String>,
}

impl BootstrapSnapshot {
    #[must_use]
    pub const fn new(
        config: Config,
        resources: Vec<ResourceSnapshot>,
        tasks: Vec<TaskSnapshot>,
        warnings: Vec<String>,
    ) -> Self {
        Self {
            config,
            resources,
            tasks,
            warnings,
        }
    }

    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub fn resources(&self) -> &[ResourceSnapshot] {
        &self.resources
    }

    #[must_use]
    pub fn tasks(&self) -> &[TaskSnapshot] {
        &self.tasks
    }

    #[must_use]
    pub fn warnings(&self) -> &[String] {
        &self.warnings
    }
}
