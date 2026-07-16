use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use yiyin_domain::{Metadata, ResourceId, ResourceKind, TaskId};

use crate::{
    ApplicationError, IdGenerator, MetadataReader, RegisteredTask, ResourceRepository,
    ResourceSnapshot, TaskQueue, TaskSnapshot,
};

pub struct RegisterImages {
    resources: Arc<dyn ResourceRepository>,
    ids: Arc<dyn IdGenerator>,
    tasks: Arc<dyn TaskQueue>,
}

impl RegisterImages {
    #[must_use]
    pub const fn new(
        resources: Arc<dyn ResourceRepository>,
        ids: Arc<dyn IdGenerator>,
        tasks: Arc<dyn TaskQueue>,
    ) -> Self {
        Self {
            resources,
            ids,
            tasks,
        }
    }

    /// Registers verified inputs as product tasks.
    ///
    /// # Errors
    ///
    /// Returns `INVALID_REQUEST` for an empty selection, `FILE_INVALID` when a
    /// registered input has no dimensions, or forwards port errors.
    pub fn execute(&self, sources: &[PathBuf]) -> Result<Vec<TaskSnapshot>, ApplicationError> {
        if sources.is_empty() {
            return Err(ApplicationError::invalid_request(
                "No images were selected.",
            ));
        }

        for source in sources {
            let record = self.resources.register_input(source)?;
            if record.kind() != ResourceKind::Input {
                return Err(ApplicationError::file_invalid());
            }
            let dimensions = record
                .dimensions()
                .ok_or_else(ApplicationError::file_invalid)?;
            self.tasks.register(RegisteredTask::new(
                self.ids.next_task_id(),
                record.id().clone(),
                record.display_name(),
                dimensions,
                record.density(),
            ))?;
        }

        Ok(self.tasks.snapshot())
    }
}

pub struct RegisterFont {
    resources: Arc<dyn ResourceRepository>,
}

impl RegisterFont {
    #[must_use]
    pub const fn new(resources: Arc<dyn ResourceRepository>) -> Self {
        Self { resources }
    }

    /// Registers a named font in Rust-owned storage.
    ///
    /// # Errors
    ///
    /// Returns `INVALID_REQUEST` for an empty or duplicate name and forwards
    /// file/repository errors.
    pub fn execute(&self, name: &str, source: &Path) -> Result<ResourceSnapshot, ApplicationError> {
        if name.trim().is_empty() {
            return Err(ApplicationError::invalid_request(
                "The font name is required.",
            ));
        }
        if self
            .resources
            .snapshot()
            .iter()
            .any(|record| record.kind() == ResourceKind::Font && record.display_name() == name)
        {
            return Err(ApplicationError::invalid_request(
                "A font with this name already exists.",
            ));
        }
        let record = self
            .resources
            .register_owned(ResourceKind::Font, source, name)?;
        Ok(ResourceSnapshot::from_record(&record))
    }
}

pub struct RemoveFont {
    resources: Arc<dyn ResourceRepository>,
}

impl RemoveFont {
    #[must_use]
    pub const fn new(resources: Arc<dyn ResourceRepository>) -> Self {
        Self { resources }
    }

    /// Removes a registered font capability.
    ///
    /// # Errors
    ///
    /// Returns `FORBIDDEN` for non-font resources and forwards repository errors.
    pub fn execute(&self, id: &ResourceId) -> Result<(), ApplicationError> {
        let record = self.resources.resolve(id)?;
        if record.kind() != ResourceKind::Font {
            return Err(ApplicationError::forbidden());
        }
        self.resources.remove(id)
    }
}

pub struct RegisterOverlay {
    resources: Arc<dyn ResourceRepository>,
}

impl RegisterOverlay {
    #[must_use]
    pub const fn new(resources: Arc<dyn ResourceRepository>) -> Self {
        Self { resources }
    }

    /// Copies an overlay into Rust-owned storage and returns an opaque snapshot.
    ///
    /// # Errors
    ///
    /// Returns `INVALID_REQUEST` when no safe display name can be derived and
    /// forwards repository errors.
    pub fn execute(&self, source: &Path) -> Result<ResourceSnapshot, ApplicationError> {
        let display_name = source
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| ApplicationError::invalid_request("The overlay name is invalid."))?;
        let record = self
            .resources
            .register_owned(ResourceKind::Overlay, source, display_name)?;
        Ok(ResourceSnapshot::from_record(&record))
    }
}

pub struct ReadTaskExif {
    tasks: Arc<dyn TaskQueue>,
    resources: Arc<dyn ResourceRepository>,
    metadata: Arc<dyn MetadataReader>,
}

impl ReadTaskExif {
    #[must_use]
    pub const fn new(
        tasks: Arc<dyn TaskQueue>,
        resources: Arc<dyn ResourceRepository>,
        metadata: Arc<dyn MetadataReader>,
    ) -> Self {
        Self {
            tasks,
            resources,
            metadata,
        }
    }

    /// Reads normalized metadata for a registered task without returning its path.
    ///
    /// # Errors
    ///
    /// Returns `TASK_NOT_FOUND` for an unknown task and forwards resource or
    /// metadata adapter errors.
    pub fn execute(&self, id: &TaskId) -> Result<Option<Metadata>, ApplicationError> {
        let task = self
            .tasks
            .registered(id)
            .ok_or_else(ApplicationError::task_not_found)?;
        let resource = self.resources.resolve(task.input())?;
        Ok(self
            .metadata
            .read(resource.source())?
            .filter(|metadata| !metadata.is_empty()))
    }
}
