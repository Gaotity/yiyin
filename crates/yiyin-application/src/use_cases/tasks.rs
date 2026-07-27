use std::{collections::BTreeSet, sync::Arc};

use yiyin_domain::{Config, OutputNameResolver, Quality, RenderRequest, TaskId, TaskState};

use crate::{
    ApplicationError, ConfigRepository, MetadataReader, OutputDirectoryGateway, RegisteredTask,
    ResourceRepository, TaskQueue, TaskSnapshot,
};

pub struct StartTasks {
    config: Arc<dyn ConfigRepository>,
    resources: Arc<dyn ResourceRepository>,
    metadata: Arc<dyn MetadataReader>,
    output: Arc<dyn OutputDirectoryGateway>,
    tasks: Arc<dyn TaskQueue>,
}

impl StartTasks {
    #[must_use]
    pub const fn new(
        config: Arc<dyn ConfigRepository>,
        resources: Arc<dyn ResourceRepository>,
        metadata: Arc<dyn MetadataReader>,
        output: Arc<dyn OutputDirectoryGateway>,
        tasks: Arc<dyn TaskQueue>,
    ) -> Self {
        Self {
            config,
            resources,
            metadata,
            output,
            tasks,
        }
    }

    /// Freezes and enqueues the requested tasks regardless of quick-output mode.
    ///
    /// # Errors
    ///
    /// Returns `INVALID_REQUEST` for no tasks, `TASK_NOT_FOUND` for an unknown
    /// task, or forwards configuration, output, and queue errors.
    pub fn execute(&self, ids: &[TaskId]) -> Result<Vec<TaskSnapshot>, ApplicationError> {
        let registered = self.require_registered(ids)?;
        let config = self.config.load()?;
        self.enqueue(registered, &config)
    }

    /// Enqueues newly registered tasks only when quick output is enabled.
    ///
    /// This entry point lets the Rust command adapter preserve automatic output
    /// without making React inspect configuration or call `start_tasks` itself.
    ///
    /// # Errors
    ///
    /// Returns `INVALID_REQUEST` for no tasks, `TASK_NOT_FOUND` for an unknown
    /// task, or forwards configuration, output, and queue errors.
    pub fn execute_quick_output(
        &self,
        ids: &[TaskId],
    ) -> Result<Vec<TaskSnapshot>, ApplicationError> {
        let registered = self.require_registered(ids)?;
        let config = self.config.load()?;
        if !config.options.iot {
            return Ok(self.tasks.snapshot());
        }
        self.enqueue(registered, &config)
    }

    fn require_registered(&self, ids: &[TaskId]) -> Result<Vec<RegisteredTask>, ApplicationError> {
        if ids.is_empty() {
            return Err(ApplicationError::invalid_request("No tasks were selected."));
        }
        ids.iter()
            .map(|id| {
                self.tasks
                    .registered(id)
                    .ok_or_else(ApplicationError::task_not_found)
            })
            .collect()
    }

    fn enqueue(
        &self,
        registered: Vec<RegisteredTask>,
        config: &Config,
    ) -> Result<Vec<TaskSnapshot>, ApplicationError> {
        self.release_terminal_reservations()?;
        let mut existing_names = self.output.existing_names()?;
        let mut requests = Vec::with_capacity(registered.len());
        for task in registered {
            let output_name = resolve_output_name(task.display_name(), &existing_names)?;
            existing_names.insert(output_name.clone());
            let request = build_request(
                &task,
                output_name,
                config,
                self.resources.as_ref(),
                self.metadata.as_ref(),
            )?;
            requests.push(request);
        }
        for request in requests {
            self.output.reserve(request.output_name())?;
            self.tasks.enqueue(request)?;
        }
        Ok(self.tasks.snapshot())
    }

    /// Releases reservations held by tasks that failed or were cancelled so a
    /// later export can reuse the original output name.
    fn release_terminal_reservations(&self) -> Result<(), ApplicationError> {
        release_terminal_reservations(self.tasks.as_ref(), self.output.as_ref())
    }
}

/// Releases reservations held by tasks that failed or were cancelled so a
/// later export can reuse the original output name. Each reservation is
/// released exactly once: the snapshot's output name is cleared after the
/// release so a later start cannot free a name that another task has since
/// reserved.
fn release_terminal_reservations(
    tasks: &dyn TaskQueue,
    output: &dyn OutputDirectoryGateway,
) -> Result<(), ApplicationError> {
    for task in tasks.snapshot() {
        if matches!(task.state(), TaskState::Failed | TaskState::Cancelled)
            && let Some(output_name) = task.output_name()
        {
            output.release(output_name)?;
            tasks.clear_output_name(task.id())?;
        }
    }
    Ok(())
}

pub struct PreviewTask {
    config: Arc<dyn ConfigRepository>,
    resources: Arc<dyn ResourceRepository>,
    metadata: Arc<dyn MetadataReader>,
    output: Arc<dyn OutputDirectoryGateway>,
    tasks: Arc<dyn TaskQueue>,
}

impl PreviewTask {
    #[must_use]
    pub const fn new(
        config: Arc<dyn ConfigRepository>,
        resources: Arc<dyn ResourceRepository>,
        metadata: Arc<dyn MetadataReader>,
        output: Arc<dyn OutputDirectoryGateway>,
        tasks: Arc<dyn TaskQueue>,
    ) -> Self {
        Self {
            config,
            resources,
            metadata,
            output,
            tasks,
        }
    }

    /// Replaces the dedicated preview slot with a frozen quality-70 request.
    ///
    /// # Errors
    ///
    /// Returns `TASK_NOT_FOUND` for an unknown task or forwards configuration
    /// and preview-queue errors.
    pub fn execute(&self, id: &TaskId) -> Result<Vec<TaskSnapshot>, ApplicationError> {
        let task = self
            .tasks
            .registered(id)
            .ok_or_else(ApplicationError::task_not_found)?;
        // A preview clears the task's tracked output name, so a failed or
        // cancelled export's reservation must be released first — otherwise
        // the gateway keeps the name and the retry becomes a phantom gap.
        release_terminal_reservations(self.tasks.as_ref(), self.output.as_ref())?;
        let mut config = self.config.load()?;
        config.options.quality = Quality::try_from(70_u8).map_err(|_| {
            ApplicationError::internal("the domain rejected the fixed preview quality")
        })?;
        let output_name = resolve_output_name(task.display_name(), &BTreeSet::new())?;
        let request = build_request(
            &task,
            output_name,
            &config,
            self.resources.as_ref(),
            self.metadata.as_ref(),
        )?;
        self.tasks.preview(request.as_preview())?;
        Ok(self.tasks.snapshot())
    }
}

pub struct CancelTask {
    tasks: Arc<dyn TaskQueue>,
}

impl CancelTask {
    #[must_use]
    pub const fn new(tasks: Arc<dyn TaskQueue>) -> Self {
        Self { tasks }
    }

    /// Cancels one known task and returns the reconciled snapshot.
    ///
    /// # Errors
    ///
    /// Returns `TASK_NOT_FOUND` for an unknown task or forwards queue errors.
    pub fn execute(&self, id: &TaskId) -> Result<Vec<TaskSnapshot>, ApplicationError> {
        self.tasks
            .registered(id)
            .ok_or_else(ApplicationError::task_not_found)?;
        self.tasks.cancel(id)?;
        Ok(self.tasks.snapshot())
    }
}

pub struct ClearTasks {
    tasks: Arc<dyn TaskQueue>,
}

impl ClearTasks {
    #[must_use]
    pub const fn new(tasks: Arc<dyn TaskQueue>) -> Self {
        Self { tasks }
    }

    /// Clears registered and queued task state without deleting exported files.
    ///
    /// # Errors
    ///
    /// Forwards queue cleanup errors.
    pub fn execute(&self) -> Result<Vec<TaskSnapshot>, ApplicationError> {
        self.tasks.clear()?;
        Ok(self.tasks.snapshot())
    }
}

fn resolve_output_name(
    source_name: &str,
    existing_names: &BTreeSet<String>,
) -> Result<String, ApplicationError> {
    OutputNameResolver::resolve(source_name, existing_names)
        .map_err(|_| ApplicationError::file_invalid())
}

fn build_request(
    task: &RegisteredTask,
    output_name: String,
    config: &Config,
    resources: &dyn ResourceRepository,
    metadata: &dyn MetadataReader,
) -> Result<RenderRequest, ApplicationError> {
    let resource = resources.resolve(task.input())?;
    let metadata = metadata.read(resource.source())?.unwrap_or_default();
    let request = RenderRequest::freeze(
        task.id().clone(),
        task.input().clone(),
        output_name,
        task.dimensions(),
        config.clone(),
        metadata,
    );
    Ok(if let Some(density) = task.density() {
        request.with_density(density)
    } else {
        request
    })
}
