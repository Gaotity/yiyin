use std::collections::BTreeMap;

use yiyin_domain::{
    Config, ImageDensity, ImageDimensions, NumericConstraint, NumericOption, ResourceId,
    ResourceKind, TaskId, TaskState,
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
    resource: Option<ResourceSnapshot>,
    output_name: Option<String>,
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
            resource: None,
            output_name: None,
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
            resource: None,
            output_name: None,
        }
    }

    #[must_use]
    pub fn with_resource(mut self, resource: ResourceSnapshot) -> Self {
        self.resource = Some(resource);
        self
    }

    #[must_use]
    pub fn with_output_name(mut self, output_name: impl Into<String>) -> Self {
        self.output_name = Some(output_name.into());
        self
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

    #[must_use]
    pub const fn resource(&self) -> Option<&ResourceSnapshot> {
        self.resource.as_ref()
    }

    #[must_use]
    pub fn output_name(&self) -> Option<&str> {
        self.output_name.as_deref()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RenderResult {
    task_id: TaskId,
    resource: ResourceSnapshot,
    dimensions: ImageDimensions,
    density: Option<ImageDensity>,
    encoder_quality: u8,
}

impl RenderResult {
    #[must_use]
    pub const fn new(
        task_id: TaskId,
        resource: ResourceSnapshot,
        dimensions: ImageDimensions,
        density: Option<ImageDensity>,
        encoder_quality: u8,
    ) -> Self {
        Self {
            task_id,
            resource,
            dimensions,
            density,
            encoder_quality,
        }
    }

    #[must_use]
    pub const fn task_id(&self) -> &TaskId {
        &self.task_id
    }

    #[must_use]
    pub const fn resource(&self) -> &ResourceSnapshot {
        &self.resource
    }

    #[must_use]
    pub const fn dimensions(&self) -> ImageDimensions {
        self.dimensions
    }

    #[must_use]
    pub const fn density(&self) -> Option<ImageDensity> {
        self.density
    }

    #[must_use]
    pub const fn encoder_quality(&self) -> u8 {
        self.encoder_quality
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BootstrapSnapshot {
    config: Config,
    constraints: BTreeMap<NumericOption, NumericConstraint>,
    resources: Vec<ResourceSnapshot>,
    tasks: Vec<TaskSnapshot>,
    warnings: Vec<String>,
}

impl BootstrapSnapshot {
    #[must_use]
    pub fn new(
        config: Config,
        resources: Vec<ResourceSnapshot>,
        tasks: Vec<TaskSnapshot>,
        warnings: Vec<String>,
    ) -> Self {
        Self {
            config,
            constraints: NumericOption::ALL
                .into_iter()
                .map(|option| (option, option.constraint()))
                .collect(),
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
    pub fn constraints(&self) -> &BTreeMap<NumericOption, NumericConstraint> {
        &self.constraints
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
