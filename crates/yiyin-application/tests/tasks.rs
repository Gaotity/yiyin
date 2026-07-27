use std::{
    collections::BTreeSet,
    path::PathBuf,
    sync::{Arc, Mutex},
};

use yiyin_application::{
    ApplicationError, CancelTask, ClearTasks, ConfigRepository, ErrorCode, ImportOutcome,
    MetadataReader, OutputDirectoryGateway, PreviewTask, RegisteredTask, ResourceRecord,
    ResourceRepository, ResourceSnapshot, StartTasks, TaskQueue, TaskSnapshot,
};
use yiyin_domain::{
    BuiltInField, Config, ImageDimensions, Metadata, OutputNameResolver, Quality, RenderRequest,
    ResourceId, ResourceKind, TaskId, TaskState,
};

#[derive(Clone)]
struct FakeConfig(Arc<Mutex<Config>>);

impl FakeConfig {
    fn with_quality(quality: u8) -> Self {
        let mut config = Config::default();
        config.options.quality = Quality::try_from(quality).expect("valid quality");
        Self(Arc::new(Mutex::new(config)))
    }

    fn set_quality(&self, quality: u8) {
        self.0.lock().expect("config lock").options.quality =
            Quality::try_from(quality).expect("valid quality");
    }

    fn set_quick_output(&self, enabled: bool) {
        self.0.lock().expect("config lock").options.iot = enabled;
    }

    fn set_output(&self, output: &str) {
        output.clone_into(&mut self.0.lock().expect("config lock").output);
    }
}

#[derive(Clone, Default)]
struct FakeResources(Arc<Mutex<Vec<ResourceRecord>>>);

impl FakeResources {
    fn push(&self, record: ResourceRecord) {
        self.0.lock().expect("resources lock").push(record);
    }
}

impl ResourceRepository for FakeResources {
    fn register_input(
        &self,
        _source: &std::path::Path,
    ) -> Result<ResourceRecord, ApplicationError> {
        Err(ApplicationError::invalid_request("Not used by this test."))
    }

    fn register_owned(
        &self,
        _kind: ResourceKind,
        _source: &std::path::Path,
        _display_name: &str,
    ) -> Result<ResourceRecord, ApplicationError> {
        Err(ApplicationError::invalid_request("Not used by this test."))
    }

    fn remove(&self, _id: &ResourceId) -> Result<(), ApplicationError> {
        Err(ApplicationError::invalid_request("Not used by this test."))
    }

    fn resolve(&self, id: &ResourceId) -> Result<ResourceRecord, ApplicationError> {
        self.0
            .lock()
            .expect("resources lock")
            .iter()
            .find(|record| record.id() == id)
            .cloned()
            .ok_or_else(ApplicationError::resource_not_found)
    }

    fn snapshot(&self) -> Vec<ResourceRecord> {
        self.0.lock().expect("resources lock").clone()
    }
}

#[derive(Clone, Default)]
struct FakeMetadata(Arc<Mutex<Metadata>>);

impl FakeMetadata {
    fn set_model(&self, model: &str) {
        self.0
            .lock()
            .expect("metadata lock")
            .set(BuiltInField::Model, model);
    }
}

impl MetadataReader for FakeMetadata {
    fn read(&self, _source: &std::path::Path) -> Result<Option<Metadata>, ApplicationError> {
        Ok(Some(self.0.lock().expect("metadata lock").clone()))
    }
}

impl ConfigRepository for FakeConfig {
    fn load(&self) -> Result<Config, ApplicationError> {
        Ok(self.0.lock().expect("config lock").clone())
    }

    fn store(&self, config: &Config) -> Result<(), ApplicationError> {
        *self.0.lock().expect("config lock") = config.clone();
        Ok(())
    }

    fn import_legacy_if_needed(&self) -> Result<ImportOutcome, ApplicationError> {
        Ok(ImportOutcome::clean())
    }
}

#[derive(Clone, Default)]
struct FakeOutput {
    names: Arc<Mutex<BTreeSet<String>>>,
    reservations: Arc<Mutex<Vec<String>>>,
    releases: Arc<Mutex<Vec<String>>>,
}

impl FakeOutput {
    fn with_names(names: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            names: Arc::new(Mutex::new(names.into_iter().map(str::to_owned).collect())),
            reservations: Arc::default(),
            releases: Arc::default(),
        }
    }

    fn reservations(&self) -> Vec<String> {
        self.reservations.lock().expect("reservations lock").clone()
    }

    fn releases(&self) -> Vec<String> {
        self.releases.lock().expect("releases lock").clone()
    }
}

impl OutputDirectoryGateway for FakeOutput {
    fn existing_names(&self) -> Result<BTreeSet<String>, ApplicationError> {
        Ok(self.names.lock().expect("names lock").clone())
    }

    fn reserve(&self, file_name: &str) -> Result<(), ApplicationError> {
        self.names
            .lock()
            .expect("names lock")
            .insert(file_name.to_owned());
        self.reservations
            .lock()
            .expect("reservations lock")
            .push(file_name.to_owned());
        Ok(())
    }

    fn release(&self, file_name: &str) -> Result<(), ApplicationError> {
        self.names.lock().expect("names lock").remove(file_name);
        self.releases
            .lock()
            .expect("releases lock")
            .push(file_name.to_owned());
        Ok(())
    }
}

#[derive(Default)]
struct QueueState {
    registered: Vec<RegisteredTask>,
    exports: Vec<RenderRequest>,
    preview: Option<RenderRequest>,
    superseded_previews: Vec<TaskId>,
    cancelled: BTreeSet<String>,
    failed: BTreeSet<String>,
    cleared_output: BTreeSet<String>,
    results: Vec<(TaskId, ResourceSnapshot)>,
}

#[derive(Clone, Default)]
struct FakeQueue(Arc<Mutex<QueueState>>);

impl FakeQueue {
    fn request(&self, id: &TaskId) -> Option<RenderRequest> {
        self.0
            .lock()
            .expect("queue lock")
            .exports
            .iter()
            .find(|request| request.task_id() == id)
            .cloned()
    }

    fn preview(&self) -> Option<RenderRequest> {
        self.0.lock().expect("queue lock").preview.clone()
    }

    fn superseded_previews(&self) -> Vec<TaskId> {
        self.0
            .lock()
            .expect("queue lock")
            .superseded_previews
            .clone()
    }

    fn export_names(&self, id: &TaskId) -> Vec<String> {
        self.0
            .lock()
            .expect("queue lock")
            .exports
            .iter()
            .filter(|request| request.task_id() == id)
            .map(|request| request.output_name().to_owned())
            .collect()
    }

    fn fail(&self, id: &TaskId) {
        self.0
            .lock()
            .expect("queue lock")
            .failed
            .insert(id.as_str().to_owned());
    }

    fn complete(&self, id: &TaskId) {
        let record = ResourceRecord::new(
            ResourceId::try_from(format!("output-{}", id.as_str())).expect("resource id"),
            ResourceKind::Output,
            "photo.jpg",
            PathBuf::from("/private/output/photo.jpg"),
        );
        self.0
            .lock()
            .expect("queue lock")
            .results
            .push((id.clone(), ResourceSnapshot::from_record(&record)));
    }
}

impl TaskQueue for FakeQueue {
    fn register(&self, task: RegisteredTask) -> Result<(), ApplicationError> {
        self.0.lock().expect("queue lock").registered.push(task);
        Ok(())
    }

    fn registered(&self, id: &TaskId) -> Option<RegisteredTask> {
        self.0
            .lock()
            .expect("queue lock")
            .registered
            .iter()
            .find(|task| task.id() == id)
            .cloned()
    }

    fn enqueue(&self, request: RenderRequest) -> Result<(), ApplicationError> {
        let mut state = self.0.lock().expect("queue lock");
        state.cleared_output.remove(request.task_id().as_str());
        state.exports.push(request);
        Ok(())
    }

    fn preview(&self, request: RenderRequest) -> Result<(), ApplicationError> {
        let mut state = self.0.lock().expect("queue lock");
        if let Some(previous) = state.preview.replace(request) {
            state.superseded_previews.push(previous.task_id().clone());
        }
        Ok(())
    }

    fn cancel(&self, id: &TaskId) -> Result<(), ApplicationError> {
        self.0
            .lock()
            .expect("queue lock")
            .cancelled
            .insert(id.as_str().to_owned());
        Ok(())
    }

    fn clear_output_name(&self, id: &TaskId) -> Result<(), ApplicationError> {
        self.0
            .lock()
            .expect("queue lock")
            .cleared_output
            .insert(id.as_str().to_owned());
        Ok(())
    }

    fn clear(&self) -> Result<(), ApplicationError> {
        *self.0.lock().expect("queue lock") = QueueState::default();
        Ok(())
    }

    fn shutdown(&self) -> Result<(), ApplicationError> {
        Ok(())
    }

    fn snapshot(&self) -> Vec<TaskSnapshot> {
        let state = self.0.lock().expect("queue lock");
        state
            .registered
            .iter()
            .map(|task| {
                let snapshot = if let Some((_, resource)) = state
                    .results
                    .iter()
                    .find(|(task_id, _)| task_id == task.id())
                {
                    TaskSnapshot::new(
                        task.id().clone(),
                        task.display_name(),
                        TaskState::Completed,
                        100,
                        false,
                    )
                    .with_resource(resource.clone())
                } else if state.failed.contains(task.id().as_str()) {
                    TaskSnapshot::new(
                        task.id().clone(),
                        task.display_name(),
                        TaskState::Failed,
                        0,
                        false,
                    )
                } else if state.cancelled.contains(task.id().as_str()) {
                    TaskSnapshot::new(
                        task.id().clone(),
                        task.display_name(),
                        TaskState::Cancelled,
                        0,
                        false,
                    )
                } else {
                    let task_state = if state
                        .exports
                        .iter()
                        .any(|request| request.task_id() == task.id())
                    {
                        TaskState::Queued
                    } else {
                        TaskState::Registered
                    };
                    TaskSnapshot::new(task.id().clone(), task.display_name(), task_state, 0, false)
                };
                if state.cleared_output.contains(task.id().as_str()) {
                    snapshot
                } else if let Some(request) = state
                    .exports
                    .iter()
                    .rev()
                    .find(|request| request.task_id() == task.id())
                {
                    snapshot.with_output_name(request.output_name().to_owned())
                } else {
                    snapshot
                }
            })
            .collect()
    }
}

struct Harness {
    config: Arc<FakeConfig>,
    resources: Arc<FakeResources>,
    metadata: Arc<FakeMetadata>,
    output: Arc<FakeOutput>,
    queue: Arc<FakeQueue>,
}

impl Harness {
    fn with_quality(quality: u8) -> Self {
        Self {
            config: Arc::new(FakeConfig::with_quality(quality)),
            resources: Arc::new(FakeResources::default()),
            metadata: Arc::new(FakeMetadata::default()),
            output: Arc::new(FakeOutput::default()),
            queue: Arc::new(FakeQueue::default()),
        }
    }

    fn with_output_names(names: impl IntoIterator<Item = &'static str>) -> Self {
        Self {
            config: Arc::new(FakeConfig::with_quality(100)),
            resources: Arc::new(FakeResources::default()),
            metadata: Arc::new(FakeMetadata::default()),
            output: Arc::new(FakeOutput::with_names(names)),
            queue: Arc::new(FakeQueue::default()),
        }
    }

    fn register(&self, id: &str, name: &str) -> TaskId {
        let task_id = TaskId::try_from(id).expect("task id");
        let resource_id = ResourceId::try_from(format!("input-{id}")).expect("resource id");
        self.resources.push(
            ResourceRecord::new(
                resource_id.clone(),
                ResourceKind::Input,
                name,
                PathBuf::from(format!("/private/input/{name}")),
            )
            .with_image_info(ImageDimensions::new(1200, 800).expect("dimensions"), None),
        );
        self.queue
            .register(RegisteredTask::new(
                task_id.clone(),
                resource_id,
                name,
                ImageDimensions::new(1200, 800).expect("dimensions"),
                None,
            ))
            .expect("register task");
        task_id
    }

    fn start(&self) -> StartTasks {
        StartTasks::new(
            self.config.clone(),
            self.resources.clone(),
            self.metadata.clone(),
            self.output.clone(),
            self.queue.clone(),
        )
    }
}

#[test]
fn configuration_is_frozen_when_task_starts() {
    let harness = Harness::with_quality(80);
    let id = harness.register("task-1", "photo.jpg");
    harness.config.set_output("/before");
    harness.metadata.set_model("before");

    harness.start().execute(std::slice::from_ref(&id)).unwrap();
    harness.config.set_quality(40);
    harness.config.set_output("/after");
    harness.metadata.set_model("after");

    let request = harness.queue.request(&id).expect("queued request");
    assert_eq!(request.options().quality.get(), 80);
    assert_eq!(request.config().output, "/before");
    assert_eq!(
        request.metadata().value(BuiltInField::Model),
        Some("before")
    );
}

#[test]
fn quick_output_enqueues_while_explicit_mode_waits() {
    let harness = Harness::with_quality(100);
    let waiting = harness.register("waiting", "waiting.png");

    harness
        .start()
        .execute_quick_output(std::slice::from_ref(&waiting))
        .unwrap();
    assert!(harness.queue.request(&waiting).is_none());

    harness.config.set_quick_output(true);
    let quick = harness.register("quick", "quick.webp");
    harness
        .start()
        .execute_quick_output(std::slice::from_ref(&quick))
        .unwrap();
    assert!(harness.queue.request(&quick).is_some());

    harness
        .start()
        .execute(std::slice::from_ref(&waiting))
        .unwrap();
    assert!(harness.queue.request(&waiting).is_some());
}

#[test]
fn start_reserves_the_legacy_collision_safe_output_name() {
    let harness = Harness::with_output_names(["photo.jpg", "photo-4.jpg"]);
    let id = harness.register("task-1", "photo.png");

    harness.start().execute(std::slice::from_ref(&id)).unwrap();

    assert_eq!(
        harness
            .queue
            .request(&id)
            .expect("queued request")
            .output_name(),
        OutputNameResolver::resolve(
            "photo.png",
            &BTreeSet::from(["photo.jpg".to_owned(), "photo-4.jpg".to_owned()])
        )
        .expect("output name")
    );
    assert_eq!(harness.output.reservations(), ["photo-5.jpg"]);
}

#[test]
fn newer_preview_supersedes_the_previous_preview_without_reserving_output() {
    let harness = Harness::with_quality(100);
    let first = harness.register("preview-a", "a.jpg");
    let second = harness.register("preview-b", "b.jpg");
    let preview = PreviewTask::new(
        harness.config.clone(),
        harness.resources.clone(),
        harness.metadata.clone(),
        harness.queue.clone(),
    );

    preview.execute(&first).unwrap();
    preview.execute(&second).unwrap();

    let request = harness.queue.preview().expect("latest preview");
    assert_eq!(request.task_id(), &second);
    assert!(request.is_preview());
    assert_eq!(request.options().quality.get(), 70);
    assert_eq!(harness.queue.superseded_previews(), [first]);
    assert!(harness.output.reservations().is_empty());
}

#[test]
fn cancel_validates_task_ids_and_returns_the_updated_snapshot() {
    let harness = Harness::with_quality(100);
    let id = harness.register("task-1", "photo.jpg");
    let cancel = CancelTask::new(harness.queue.clone());

    let unknown = TaskId::try_from("unknown").expect("task id");
    assert_eq!(
        cancel.execute(&unknown).unwrap_err().code(),
        ErrorCode::TaskNotFound
    );

    let snapshot = cancel.execute(&id).unwrap();
    assert_eq!(snapshot[0].state(), TaskState::Cancelled);
}

#[test]
fn clear_removes_registered_and_queued_snapshots() {
    let harness = Harness::with_quality(100);
    let id = harness.register("task-1", "photo.jpg");
    harness.start().execute(std::slice::from_ref(&id)).unwrap();

    let snapshot = ClearTasks::new(harness.queue.clone()).execute().unwrap();

    assert!(snapshot.is_empty());
    assert!(harness.queue.snapshot().is_empty());
}

#[test]
fn completed_snapshots_expose_an_opaque_resource_without_a_path() {
    let harness = Harness::with_quality(100);
    let id = harness.register("task-1", "photo.jpg");
    harness.queue.complete(&id);

    let snapshot = harness.queue.snapshot();
    let resource = snapshot[0].resource().expect("completed resource");

    assert_eq!(resource.kind(), ResourceKind::Output);
    assert!(!format!("{snapshot:?}").contains("/private/output"));
}

#[test]
fn cancelling_an_export_releases_its_output_name_for_reuse() {
    let harness = Harness::with_quality(100);
    let id = harness.register("task-1", "photo.png");

    harness.start().execute(std::slice::from_ref(&id)).unwrap();
    CancelTask::new(harness.queue.clone()).execute(&id).unwrap();
    harness.start().execute(std::slice::from_ref(&id)).unwrap();

    assert_eq!(harness.queue.export_names(&id), ["photo.jpg", "photo.jpg"]);
}

#[test]
fn a_failed_export_releases_its_output_name_for_reuse() {
    let harness = Harness::with_quality(100);
    let id = harness.register("task-1", "photo.png");

    harness.start().execute(std::slice::from_ref(&id)).unwrap();
    harness.queue.fail(&id);
    harness.start().execute(std::slice::from_ref(&id)).unwrap();

    assert_eq!(harness.queue.export_names(&id), ["photo.jpg", "photo.jpg"]);
}

#[test]
fn a_released_terminal_reservation_is_not_released_again() {
    let harness = Harness::with_quality(100);
    let failed = harness.register("task-1", "photo.png");
    let replacement = harness.register("task-2", "photo.png");

    harness
        .start()
        .execute(std::slice::from_ref(&failed))
        .unwrap();
    harness.queue.fail(&failed);
    harness
        .start()
        .execute(std::slice::from_ref(&replacement))
        .unwrap();
    harness
        .start()
        .execute(std::slice::from_ref(&replacement))
        .unwrap();

    // The failed task's reservation is released exactly once; releasing it
    // again would free the name the replacement task has since reserved.
    assert_eq!(harness.output.releases(), ["photo.jpg"]);
}
