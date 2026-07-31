use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};

use yiyin_domain::{
    Config, ImageDensity, ImageDimensions, Metadata, OutputDirectory, RenderRequest, RenderStage,
    ResourceId, ResourceKind, TaskId, TaskStatus,
};

use crate::{ApplicationError, ImportOutcome, RegisteredTask, RenderResult, TaskSnapshot};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResourceRecord {
    id: ResourceId,
    kind: ResourceKind,
    display_name: String,
    source: PathBuf,
    mime_type: String,
    allowed_root: PathBuf,
    dimensions: Option<ImageDimensions>,
    density: Option<ImageDensity>,
}

impl ResourceRecord {
    #[must_use]
    pub fn new(
        id: ResourceId,
        kind: ResourceKind,
        display_name: impl Into<String>,
        source: PathBuf,
    ) -> Self {
        let allowed_root = source.parent().map_or_else(PathBuf::new, Path::to_path_buf);
        Self {
            id,
            kind,
            display_name: display_name.into(),
            source,
            mime_type: String::new(),
            allowed_root,
            dimensions: None,
            density: None,
        }
    }

    #[must_use]
    pub fn with_security_context(
        mut self,
        mime_type: impl Into<String>,
        allowed_root: PathBuf,
    ) -> Self {
        self.mime_type = mime_type.into();
        self.allowed_root = allowed_root;
        self
    }

    #[must_use]
    pub const fn with_image_info(
        mut self,
        dimensions: ImageDimensions,
        density: Option<ImageDensity>,
    ) -> Self {
        self.dimensions = Some(dimensions);
        self.density = density;
        self
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

    #[must_use]
    pub fn source(&self) -> &Path {
        &self.source
    }

    #[must_use]
    pub fn mime_type(&self) -> &str {
        &self.mime_type
    }

    #[must_use]
    pub fn allowed_root(&self) -> &Path {
        &self.allowed_root
    }

    #[must_use]
    pub const fn dimensions(&self) -> Option<ImageDimensions> {
        self.dimensions
    }

    #[must_use]
    pub const fn density(&self) -> Option<ImageDensity> {
        self.density
    }
}

#[allow(
    clippy::missing_errors_doc,
    reason = "adapter implementations define the concrete repository failures"
)]
pub trait ConfigRepository: Send + Sync {
    fn load(&self) -> Result<Config, ApplicationError>;
    fn store(&self, config: &Config) -> Result<(), ApplicationError>;
    fn import_legacy_if_needed(&self) -> Result<ImportOutcome, ApplicationError>;
}

#[allow(
    clippy::missing_errors_doc,
    reason = "adapter implementations define the concrete resource failures"
)]
pub trait ResourceRepository: Send + Sync {
    fn register_input(&self, source: &Path) -> Result<ResourceRecord, ApplicationError>;
    fn register_owned(
        &self,
        kind: ResourceKind,
        source: &Path,
        display_name: &str,
    ) -> Result<ResourceRecord, ApplicationError>;
    fn remove(&self, id: &ResourceId) -> Result<(), ApplicationError>;
    fn resolve(&self, id: &ResourceId) -> Result<ResourceRecord, ApplicationError>;
    fn snapshot(&self) -> Vec<ResourceRecord>;
}

#[allow(
    clippy::missing_errors_doc,
    reason = "adapter implementations define the concrete metadata failures"
)]
pub trait MetadataReader: Send + Sync {
    fn read(&self, source: &Path) -> Result<Option<Metadata>, ApplicationError>;
}

#[allow(
    clippy::missing_errors_doc,
    reason = "adapter implementations define the concrete output failures"
)]
pub trait OutputDirectoryGateway: Send + Sync {
    fn existing_names(&self) -> Result<BTreeSet<String>, ApplicationError>;
    fn reserve(&self, file_name: &str) -> Result<(), ApplicationError>;
    fn release(&self, file_name: &str) -> Result<(), ApplicationError>;
    /// Probes the configured root, creating it when missing, without
    /// switching the live root to it.
    fn ensure_root(&self, root: &OutputDirectory) -> Result<(), ApplicationError>;
    /// Switches the live root and drops every reservation: a new root is a
    /// new naming namespace, so names reserved for the previous root are void.
    fn change_root(&self, root: &OutputDirectory) -> Result<(), ApplicationError>;
}

pub trait IdGenerator: Send + Sync {
    fn next_resource_id(&self) -> ResourceId;
    fn next_task_id(&self) -> TaskId;
}

#[allow(
    clippy::missing_errors_doc,
    reason = "queue implementations define the concrete scheduling failures"
)]
pub trait TaskQueue: Send + Sync {
    fn register(&self, task: RegisteredTask) -> Result<(), ApplicationError>;
    fn registered(&self, id: &TaskId) -> Option<RegisteredTask>;
    fn enqueue(&self, request: RenderRequest) -> Result<(), ApplicationError>;
    fn preview(&self, request: RenderRequest) -> Result<(), ApplicationError>;
    fn cancel(&self, id: &TaskId) -> Result<(), ApplicationError>;
    fn clear_output_name(&self, id: &TaskId) -> Result<(), ApplicationError>;
    fn clear(&self) -> Result<(), ApplicationError>;
    fn shutdown(&self) -> Result<(), ApplicationError>;
    fn snapshot(&self) -> Vec<TaskSnapshot>;
}

pub trait TaskEventSink: Send + Sync {
    fn publish(&self, status: TaskStatus);
}

pub trait CancellationProbe: Send + Sync {
    fn is_cancelled(&self) -> bool;
}

#[allow(
    clippy::missing_errors_doc,
    reason = "renderer adapters define the concrete rendering failures"
)]
pub trait ImageRenderer: Send + Sync {
    fn render(
        &self,
        request: &RenderRequest,
        cancellation: &dyn CancellationProbe,
        progress: &mut dyn FnMut(RenderStage),
    ) -> Result<RenderResult, ApplicationError>;
}
