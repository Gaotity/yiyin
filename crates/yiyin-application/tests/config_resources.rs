use std::{
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use yiyin_application::{
    ApplicationError, Bootstrap, CancellationProbe, Clock, ConfigRepository, ErrorCode,
    IdGenerator, ImageRenderer, ImportOutcome, MetadataReader, OutputDirectoryGateway,
    ReadTaskExif, RegisterFont, RegisterImages, RegisterOverlay, RegisteredTask, RemoveFont,
    RenderResult, ResetConfig, ResourceRecord, ResourceRepository, TaskEventSink, TaskQueue,
    TaskSnapshot, UpdateConfig,
};
use yiyin_domain::{
    BuiltInField, Config, ImageDimensions, Metadata, RenderRequest, RenderStage, ResourceId,
    ResourceKind, TaskId, TaskStatus,
};

#[derive(Clone)]
struct FakeConfig {
    value: Arc<Mutex<Config>>,
    writes: Arc<Mutex<usize>>,
}

impl Default for FakeConfig {
    fn default() -> Self {
        Self {
            value: Arc::new(Mutex::new(Config::default())),
            writes: Arc::new(Mutex::new(0)),
        }
    }
}

impl FakeConfig {
    fn write_count(&self) -> usize {
        *self.writes.lock().expect("writes lock")
    }
}

impl ConfigRepository for FakeConfig {
    fn load(&self) -> Result<Config, ApplicationError> {
        Ok(self.value.lock().expect("config lock").clone())
    }

    fn store(&self, config: &Config) -> Result<(), ApplicationError> {
        *self.value.lock().expect("config lock") = config.clone();
        *self.writes.lock().expect("writes lock") += 1;
        Ok(())
    }

    fn import_legacy_if_needed(&self) -> Result<ImportOutcome, ApplicationError> {
        Ok(ImportOutcome::with_warnings(vec![
            "legacy warning".to_owned(),
        ]))
    }
}

#[derive(Clone, Default)]
struct FakeResources {
    records: Arc<Mutex<Vec<ResourceRecord>>>,
}

impl FakeResources {
    fn push(&self, record: ResourceRecord) {
        self.records.lock().expect("records lock").push(record);
    }
}

impl ResourceRepository for FakeResources {
    fn register_input(&self, source: &Path) -> Result<ResourceRecord, ApplicationError> {
        if source == Path::new("missing.jpg") {
            return Err(ApplicationError::file_not_found());
        }
        let name = source
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("image.jpg");
        let record = ResourceRecord::new(
            ResourceId::try_from(format!("input-{name}")).expect("resource id"),
            ResourceKind::Input,
            name,
            source.to_path_buf(),
        )
        .with_image_info(ImageDimensions::new(100, 80).expect("dimensions"), None);
        self.push(record.clone());
        Ok(record)
    }

    fn register_owned(
        &self,
        kind: ResourceKind,
        source: &Path,
        display_name: &str,
    ) -> Result<ResourceRecord, ApplicationError> {
        if source == Path::new("missing.ttf") {
            return Err(ApplicationError::file_not_found());
        }
        let record = ResourceRecord::new(
            ResourceId::try_from(format!("owned-{display_name}")).expect("resource id"),
            kind,
            display_name,
            source.to_path_buf(),
        );
        self.push(record.clone());
        Ok(record)
    }

    fn remove(&self, id: &ResourceId) -> Result<(), ApplicationError> {
        self.records
            .lock()
            .expect("records lock")
            .retain(|record| record.id() != id);
        Ok(())
    }

    fn resolve(&self, id: &ResourceId) -> Result<ResourceRecord, ApplicationError> {
        self.records
            .lock()
            .expect("records lock")
            .iter()
            .find(|record| record.id() == id)
            .cloned()
            .ok_or_else(ApplicationError::resource_not_found)
    }

    fn snapshot(&self) -> Vec<ResourceRecord> {
        self.records.lock().expect("records lock").clone()
    }
}

#[derive(Clone, Default)]
struct FakeQueue {
    registered: Arc<Mutex<Vec<RegisteredTask>>>,
}

impl TaskQueue for FakeQueue {
    fn register(&self, task: RegisteredTask) -> Result<(), ApplicationError> {
        self.registered.lock().expect("tasks lock").push(task);
        Ok(())
    }

    fn registered(&self, id: &TaskId) -> Option<RegisteredTask> {
        self.registered
            .lock()
            .expect("tasks lock")
            .iter()
            .find(|task| task.id() == id)
            .cloned()
    }

    fn enqueue(&self, _request: RenderRequest) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn preview(&self, _request: RenderRequest) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn cancel(&self, _id: &TaskId) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn clear(&self) -> Result<(), ApplicationError> {
        self.registered.lock().expect("tasks lock").clear();
        Ok(())
    }

    fn shutdown(&self) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn snapshot(&self) -> Vec<TaskSnapshot> {
        self.registered
            .lock()
            .expect("tasks lock")
            .iter()
            .map(TaskSnapshot::from_registered)
            .collect()
    }
}

#[derive(Clone, Default)]
struct FakeIds(Arc<Mutex<u64>>);

impl IdGenerator for FakeIds {
    fn next_resource_id(&self) -> ResourceId {
        ResourceId::try_from("resource-id").expect("resource id")
    }

    fn next_task_id(&self) -> TaskId {
        let mut value = self.0.lock().expect("id lock");
        *value += 1;
        TaskId::try_from(format!("task-{value}")).expect("task id")
    }
}

struct FakeMetadata;

impl MetadataReader for FakeMetadata {
    fn read(&self, _source: &Path) -> Result<Option<Metadata>, ApplicationError> {
        let mut metadata = Metadata::default();
        metadata.set(BuiltInField::Model, "z8");
        Ok(Some(metadata))
    }
}

#[test]
fn bootstrap_imports_before_returning_safe_snapshots() {
    let config = Arc::new(FakeConfig::default());
    let resources = Arc::new(FakeResources::default());
    resources.push(ResourceRecord::new(
        ResourceId::try_from("font-1").expect("resource id"),
        ResourceKind::Font,
        "Body",
        PathBuf::from("/private/font.ttf"),
    ));
    let snapshot = Bootstrap::new(config, resources, Arc::new(FakeQueue::default()))
        .execute()
        .expect("bootstrap");

    assert_eq!(snapshot.warnings(), &["legacy warning"]);
    assert_eq!(snapshot.resources()[0].display_name(), "Body");
    assert!(!format!("{snapshot:?}").contains("/private/font.ttf"));
}

#[test]
fn update_config_rejects_invalid_values_before_persisting() {
    let repository = Arc::new(FakeConfig::default());
    let mut invalid = Config::default();
    invalid.output.clear();

    let error = UpdateConfig::new(repository.clone())
        .execute(invalid)
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::ConfigInvalid);
    assert_eq!(repository.write_count(), 0);
}

#[test]
fn reset_config_persists_the_complete_default_model() {
    let repository = Arc::new(FakeConfig::default());
    let config = ResetConfig::new(repository.clone())
        .execute()
        .expect("reset config");

    assert_eq!(config, Config::default());
    assert_eq!(repository.write_count(), 1);
}

#[test]
fn registering_images_creates_safe_registered_tasks() {
    let queue = Arc::new(FakeQueue::default());
    let snapshots = RegisterImages::new(
        Arc::new(FakeResources::default()),
        Arc::new(FakeIds::default()),
        queue,
    )
    .execute(&[PathBuf::from("photo.jpg")])
    .expect("register image");

    assert_eq!(snapshots.len(), 1);
    assert_eq!(snapshots[0].display_name(), "photo.jpg");
    assert!(!format!("{:?}", snapshots[0]).contains('/'));
}

#[test]
fn font_registration_reports_duplicate_and_missing_source_explicitly() {
    let resources = Arc::new(FakeResources::default());
    resources.push(ResourceRecord::new(
        ResourceId::try_from("font-existing").expect("resource id"),
        ResourceKind::Font,
        "Body",
        PathBuf::from("body.ttf"),
    ));
    let register = RegisterFont::new(resources.clone());

    let duplicate = register.execute("Body", Path::new("body.ttf")).unwrap_err();
    let missing = register
        .execute("Title", Path::new("missing.ttf"))
        .unwrap_err();

    assert_eq!(duplicate.code(), ErrorCode::InvalidRequest);
    assert_eq!(missing.code(), ErrorCode::FileNotFound);
}

#[test]
fn remove_font_rejects_non_font_resources() {
    let resources = Arc::new(FakeResources::default());
    let overlay = ResourceRecord::new(
        ResourceId::try_from("overlay-1").expect("resource id"),
        ResourceKind::Overlay,
        "Frame",
        PathBuf::from("frame.png"),
    );
    let overlay_id = overlay.id().clone();
    resources.push(overlay);

    let error = RemoveFont::new(resources).execute(&overlay_id).unwrap_err();

    assert_eq!(error.code(), ErrorCode::Forbidden);
}

#[test]
fn overlay_registration_returns_only_an_opaque_resource() {
    let snapshot = RegisterOverlay::new(Arc::new(FakeResources::default()))
        .execute(Path::new("frame.png"))
        .expect("register overlay");

    assert_eq!(snapshot.kind(), ResourceKind::Overlay);
    assert_eq!(snapshot.display_name(), "frame.png");
    assert!(!format!("{snapshot:?}").contains('/'));
}

#[test]
fn task_exif_lookup_resolves_paths_only_inside_ports() {
    let resources = Arc::new(FakeResources::default());
    let record = resources
        .register_input(Path::new("photo.jpg"))
        .expect("resource");
    let queue = Arc::new(FakeQueue::default());
    let task_id = TaskId::try_from("task-exif").expect("task id");
    queue
        .register(RegisteredTask::new(
            task_id.clone(),
            record.id().clone(),
            "photo.jpg",
            ImageDimensions::new(100, 80).expect("dimensions"),
            None,
        ))
        .expect("register task");

    let metadata = ReadTaskExif::new(queue, resources, Arc::new(FakeMetadata))
        .execute(&task_id)
        .expect("read metadata")
        .expect("metadata exists");

    assert_eq!(metadata.value(BuiltInField::Model), Some("z8"));
}

#[test]
fn internal_errors_never_expose_their_source() {
    let error = ApplicationError::internal("secret /private/path");
    assert_eq!(error.code().as_str(), "INTERNAL");
    assert!(!error.safe_message().contains("secret"));
    assert!(!error.to_string().contains("/private/path"));
}

#[test]
fn stable_error_codes_are_exact() {
    let codes = [
        ErrorCode::Cancelled,
        ErrorCode::ConfigInvalid,
        ErrorCode::FileInvalid,
        ErrorCode::FileNotFound,
        ErrorCode::Forbidden,
        ErrorCode::Internal,
        ErrorCode::InvalidRequest,
        ErrorCode::ResourceNotFound,
        ErrorCode::TaskNotFound,
    ];

    assert_eq!(
        codes.map(ErrorCode::as_str),
        [
            "CANCELLED",
            "CONFIG_INVALID",
            "FILE_INVALID",
            "FILE_NOT_FOUND",
            "FORBIDDEN",
            "INTERNAL",
            "INVALID_REQUEST",
            "RESOURCE_NOT_FOUND",
            "TASK_NOT_FOUND",
        ],
    );
}

// Compile-time coverage for the remaining object-safe ports.
fn _ports_are_object_safe(
    _clock: &dyn Clock,
    _output: &dyn OutputDirectoryGateway,
    _cancellation: &dyn CancellationProbe,
    _renderer: &dyn ImageRenderer,
    _events: &dyn TaskEventSink,
) {
}

fn _renderer_signature(
    renderer: &dyn ImageRenderer,
    request: &RenderRequest,
    cancellation: &dyn CancellationProbe,
) -> Result<RenderResult, ApplicationError> {
    renderer.render(request, cancellation, &mut |_stage: RenderStage| {})
}

fn _event_signature(events: &dyn TaskEventSink, status: TaskStatus) {
    events.publish(status);
}
