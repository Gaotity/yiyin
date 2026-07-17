use std::{
    collections::HashMap,
    sync::{
        Arc, Condvar, Mutex, RwLock,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use tokio::{runtime::Runtime, sync::Semaphore};
use yiyin_application::{
    ApplicationError, CancellationProbe, ErrorCode, ImageRenderer, RegisteredTask,
    ResourceSnapshot, TaskEventSink, TaskQueue, TaskSnapshot,
};
use yiyin_domain::{CancellationReason, RenderRequest, RenderStage, TaskId, TaskState, TaskStatus};

pub const SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

pub struct AtomicCancellation(AtomicBool);

impl AtomicCancellation {
    #[must_use]
    pub const fn new() -> Self {
        Self(AtomicBool::new(false))
    }

    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}

impl Default for AtomicCancellation {
    fn default() -> Self {
        Self::new()
    }
}

impl CancellationProbe for AtomicCancellation {
    fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
}

struct TaskRecord {
    registered: RegisteredTask,
    status: TaskStatus,
    execution: u64,
    cancellation: Option<Arc<AtomicCancellation>>,
    resource: Option<ResourceSnapshot>,
}

impl TaskRecord {
    fn new(registered: RegisteredTask) -> Self {
        Self {
            status: TaskStatus::registered(registered.id().clone()),
            registered,
            execution: 0,
            cancellation: None,
            resource: None,
        }
    }

    fn snapshot(&self) -> TaskSnapshot {
        let progress = self.status.stage().map_or(0, RenderStage::percent);
        let snapshot = TaskSnapshot::new(
            self.registered.id().clone(),
            self.registered.display_name(),
            self.status.state(),
            progress,
            self.status.is_preview(),
        );
        self.resource.as_ref().map_or(snapshot.clone(), |resource| {
            snapshot.with_resource(resource.clone())
        })
    }
}

struct PreviewSlot {
    task_id: TaskId,
    execution: u64,
    cancellation: Arc<AtomicCancellation>,
}

#[derive(Default)]
struct Activity {
    count: Mutex<usize>,
    changed: Condvar,
}

impl Activity {
    fn start(self: &Arc<Self>) -> ActivityGuard {
        *self.count.lock().expect("activity lock poisoned") += 1;
        ActivityGuard(Arc::clone(self))
    }

    fn wait_until_idle(&self, deadline: Instant) {
        let mut count = self.count.lock().expect("activity lock poisoned");
        while *count != 0 {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                break;
            }
            let (next, timeout) = self
                .changed
                .wait_timeout(count, remaining)
                .expect("activity wait poisoned");
            count = next;
            if timeout.timed_out() {
                break;
            }
        }
    }
}

struct ActivityGuard(Arc<Activity>);

impl Drop for ActivityGuard {
    fn drop(&mut self) {
        let mut count = self.0.count.lock().expect("activity lock poisoned");
        *count = count.saturating_sub(1);
        self.0.changed.notify_all();
    }
}

struct Shared {
    renderer: Arc<dyn ImageRenderer>,
    events: Arc<dyn TaskEventSink>,
    semaphore: Arc<Semaphore>,
    records: RwLock<HashMap<TaskId, TaskRecord>>,
    preview: Mutex<Option<PreviewSlot>>,
    activity: Arc<Activity>,
    shutting_down: AtomicBool,
}

impl Shared {
    fn publish(&self, status: TaskStatus) {
        self.events.publish(status);
    }

    fn is_current_preview(&self, id: &TaskId, execution: u64, preview: bool) -> bool {
        if !preview {
            return true;
        }
        self.preview.lock().is_ok_and(|slot| {
            slot.as_ref()
                .is_some_and(|slot| &slot.task_id == id && slot.execution == execution)
        })
    }

    fn transition_running(&self, id: &TaskId, execution: u64, preview: bool) -> bool {
        let status = {
            let Ok(mut records) = self.records.write() else {
                return false;
            };
            let Some(record) = records.get_mut(id) else {
                return false;
            };
            if record.execution != execution
                || record.status.state() != TaskState::Queued
                || record
                    .cancellation
                    .as_ref()
                    .is_some_and(|cancellation| cancellation.is_cancelled())
                || !self.is_current_preview(id, execution, preview)
            {
                return false;
            }
            if record.status.transition_to(TaskState::Running).is_err() {
                return false;
            }
            record.status.clone()
        };
        self.publish(status);
        true
    }

    fn update_progress(
        &self,
        id: &TaskId,
        execution: u64,
        preview: bool,
        cancellation: &AtomicCancellation,
        stage: RenderStage,
    ) {
        if stage == RenderStage::Completed
            || cancellation.is_cancelled()
            || !self.is_current_preview(id, execution, preview)
        {
            return;
        }
        let status = {
            let Ok(mut records) = self.records.write() else {
                return;
            };
            let Some(record) = records.get_mut(id) else {
                return;
            };
            if record.execution != execution || record.status.state() != TaskState::Running {
                return;
            }
            record.status.advance(stage);
            record.status.clone()
        };
        self.publish(status);
    }

    fn complete(
        &self,
        id: &TaskId,
        execution: u64,
        preview: bool,
        cancellation: &AtomicCancellation,
        resource: ResourceSnapshot,
    ) {
        if cancellation.is_cancelled() || !self.is_current_preview(id, execution, preview) {
            return;
        }
        let status = {
            let Ok(mut records) = self.records.write() else {
                return;
            };
            let Some(record) = records.get_mut(id) else {
                return;
            };
            if record.execution != execution || record.status.state() != TaskState::Running {
                return;
            }
            if record.status.transition_to(TaskState::Completed).is_err() {
                return;
            }
            record.status.advance(RenderStage::Completed);
            record.resource = Some(resource);
            record.status.clone()
        };
        self.publish(status);
    }

    fn fail(&self, id: &TaskId, execution: u64, preview: bool) {
        if !self.is_current_preview(id, execution, preview) {
            return;
        }
        let status = {
            let Ok(mut records) = self.records.write() else {
                return;
            };
            let Some(record) = records.get_mut(id) else {
                return;
            };
            if record.execution != execution || record.status.state() != TaskState::Running {
                return;
            }
            if record.status.transition_to(TaskState::Failed).is_err() {
                return;
            }
            record.status.clone()
        };
        self.publish(status);
    }

    fn cancel_if_needed(&self, id: &TaskId, execution: u64, reason: CancellationReason) {
        let status = {
            let Ok(mut records) = self.records.write() else {
                return;
            };
            let Some(record) = records.get_mut(id) else {
                return;
            };
            if record.execution != execution
                || !matches!(
                    record.status.state(),
                    TaskState::Queued | TaskState::Running
                )
            {
                return;
            }
            if record.status.cancel(reason).is_err() {
                return;
            }
            record.status.clone()
        };
        self.publish(status);
    }
}

pub struct TokioTaskQueue {
    shared: Arc<Shared>,
    runtime: Mutex<Option<Runtime>>,
}

impl TokioTaskQueue {
    /// Creates a queue with a dedicated Tokio scheduling runtime and two image workers.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when the Tokio runtime cannot be created.
    pub fn new(
        renderer: Arc<dyn ImageRenderer>,
        events: Arc<dyn TaskEventSink>,
    ) -> Result<Self, ApplicationError> {
        let runtime = Runtime::new().map_err(internal_io)?;
        Ok(Self {
            shared: Arc::new(Shared {
                renderer,
                events,
                semaphore: Arc::new(Semaphore::new(2)),
                records: RwLock::new(HashMap::new()),
                preview: Mutex::new(None),
                activity: Arc::new(Activity::default()),
                shutting_down: AtomicBool::new(false),
            }),
            runtime: Mutex::new(Some(runtime)),
        })
    }

    fn prepare(
        &self,
        request: &RenderRequest,
    ) -> Result<(u64, Arc<AtomicCancellation>, Vec<TaskStatus>), ApplicationError> {
        let id = request.task_id();
        let preview_request = request.is_preview();
        let mut records = self
            .shared
            .records
            .write()
            .map_err(|_| ApplicationError::internal("task records lock poisoned"))?;
        let mut preview = self
            .shared
            .preview
            .lock()
            .map_err(|_| ApplicationError::internal("preview slot lock poisoned"))?;
        let Some(target) = records.get(id) else {
            return Err(ApplicationError::task_not_found());
        };
        let target_is_current_preview = preview
            .as_ref()
            .is_some_and(|slot| slot.task_id == *id && slot.execution == target.execution);
        if matches!(
            target.status.state(),
            TaskState::Queued | TaskState::Running
        ) && !(preview_request && target_is_current_preview)
        {
            return Err(ApplicationError::invalid_request(
                "The task is already running.",
            ));
        }

        let mut events = Vec::new();
        if preview_request && let Some(stale) = preview.take() {
            stale.cancellation.cancel();
            if let Some(record) = records.get_mut(&stale.task_id)
                && record.execution == stale.execution
                && matches!(
                    record.status.state(),
                    TaskState::Queued | TaskState::Running
                )
            {
                record
                    .status
                    .cancel(CancellationReason::PreviewSuperseded)
                    .map_err(|error| ApplicationError::internal(error.to_string()))?;
                events.push(record.status.clone());
            }
        }

        let record = records
            .get_mut(id)
            .ok_or_else(ApplicationError::task_not_found)?;
        record.execution = record
            .execution
            .checked_add(1)
            .ok_or_else(|| ApplicationError::internal("task execution counter overflow"))?;
        record.status = if preview_request {
            TaskStatus::preview(id.clone())
        } else {
            TaskStatus::registered(id.clone())
        };
        record
            .status
            .transition_to(TaskState::Queued)
            .map_err(|error| ApplicationError::internal(error.to_string()))?;
        record.resource = None;
        let cancellation = Arc::new(AtomicCancellation::new());
        record.cancellation = Some(Arc::clone(&cancellation));
        let execution = record.execution;
        if preview_request {
            *preview = Some(PreviewSlot {
                task_id: id.clone(),
                execution,
                cancellation: Arc::clone(&cancellation),
            });
        }
        events.push(record.status.clone());
        Ok((execution, cancellation, events))
    }

    fn schedule(&self, request: RenderRequest) -> Result<(), ApplicationError> {
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| ApplicationError::internal("task runtime lock poisoned"))?;
        if self.shared.shutting_down.load(Ordering::Acquire) || runtime.is_none() {
            return Err(ApplicationError::invalid_request(
                "The task queue is shutting down.",
            ));
        }
        let (execution, cancellation, events) = self.prepare(&request)?;
        for event in events {
            self.shared.publish(event);
        }
        let shared = Arc::clone(&self.shared);
        let activity = shared.activity.start();
        runtime
            .as_ref()
            .expect("runtime checked above")
            .spawn(async move {
                let _activity = activity;
                run_task(shared, request, execution, cancellation).await;
            });
        Ok(())
    }

    fn cancel_all(&self, reason: CancellationReason, clear: bool) -> Result<(), ApplicationError> {
        let events = {
            let mut records = self
                .shared
                .records
                .write()
                .map_err(|_| ApplicationError::internal("task records lock poisoned"))?;
            let mut events = Vec::new();
            for record in records.values_mut() {
                if matches!(
                    record.status.state(),
                    TaskState::Queued | TaskState::Running
                ) {
                    if let Some(cancellation) = &record.cancellation {
                        cancellation.cancel();
                    }
                    record
                        .status
                        .cancel(reason)
                        .map_err(|error| ApplicationError::internal(error.to_string()))?;
                    events.push(record.status.clone());
                }
            }
            self.shared
                .preview
                .lock()
                .map_err(|_| ApplicationError::internal("preview slot lock poisoned"))?
                .take();
            if clear {
                records.clear();
            }
            events
        };
        for event in events {
            self.shared.publish(event);
        }
        Ok(())
    }
}

impl TaskQueue for TokioTaskQueue {
    fn register(&self, task: RegisteredTask) -> Result<(), ApplicationError> {
        if self.shared.shutting_down.load(Ordering::Acquire) {
            return Err(ApplicationError::invalid_request(
                "The task queue is shutting down.",
            ));
        }
        let status = TaskStatus::registered(task.id().clone());
        let mut records = self
            .shared
            .records
            .write()
            .map_err(|_| ApplicationError::internal("task records lock poisoned"))?;
        if records.contains_key(task.id()) {
            return Err(ApplicationError::invalid_request(
                "The task is already registered.",
            ));
        }
        records.insert(task.id().clone(), TaskRecord::new(task));
        drop(records);
        self.shared.publish(status);
        Ok(())
    }

    fn registered(&self, id: &TaskId) -> Option<RegisteredTask> {
        self.shared
            .records
            .read()
            .ok()
            .and_then(|records| records.get(id).map(|record| record.registered.clone()))
    }

    fn enqueue(&self, request: RenderRequest) -> Result<(), ApplicationError> {
        if request.is_preview() {
            return Err(ApplicationError::invalid_request(
                "Preview requests must use the preview queue.",
            ));
        }
        self.schedule(request)
    }

    fn preview(&self, request: RenderRequest) -> Result<(), ApplicationError> {
        if !request.is_preview() {
            return Err(ApplicationError::invalid_request(
                "The preview request is invalid.",
            ));
        }
        self.schedule(request)
    }

    fn cancel(&self, id: &TaskId) -> Result<(), ApplicationError> {
        let status = {
            let mut records = self
                .shared
                .records
                .write()
                .map_err(|_| ApplicationError::internal("task records lock poisoned"))?;
            let record = records
                .get_mut(id)
                .ok_or_else(ApplicationError::task_not_found)?;
            if !matches!(
                record.status.state(),
                TaskState::Queued | TaskState::Running
            ) {
                return Err(ApplicationError::invalid_request(
                    "The task cannot be cancelled.",
                ));
            }
            if let Some(cancellation) = &record.cancellation {
                cancellation.cancel();
            }
            record
                .status
                .cancel(CancellationReason::User)
                .map_err(|error| ApplicationError::internal(error.to_string()))?;
            record.status.clone()
        };
        self.shared.publish(status);
        Ok(())
    }

    fn clear(&self) -> Result<(), ApplicationError> {
        self.cancel_all(CancellationReason::Cleared, true)
    }

    fn shutdown(&self) -> Result<(), ApplicationError> {
        if !self.shared.shutting_down.swap(true, Ordering::AcqRel) {
            self.cancel_all(CancellationReason::Shutdown, false)?;
        }
        let deadline = Instant::now() + SHUTDOWN_GRACE;
        self.shared.activity.wait_until_idle(deadline);
        let runtime = self
            .runtime
            .lock()
            .map_err(|_| ApplicationError::internal("task runtime lock poisoned"))?
            .take();
        if let Some(runtime) = runtime {
            runtime.shutdown_timeout(deadline.saturating_duration_since(Instant::now()));
        }
        Ok(())
    }

    fn snapshot(&self) -> Vec<TaskSnapshot> {
        let Ok(records) = self.shared.records.read() else {
            return Vec::new();
        };
        let mut snapshots = records
            .values()
            .map(TaskRecord::snapshot)
            .collect::<Vec<_>>();
        snapshots.sort_by(|left, right| left.id().as_str().cmp(right.id().as_str()));
        snapshots
    }
}

impl Drop for TokioTaskQueue {
    fn drop(&mut self) {
        self.shared.shutting_down.store(true, Ordering::Release);
        if let Ok(runtime) = self.runtime.get_mut()
            && let Some(runtime) = runtime.take()
        {
            runtime.shutdown_background();
        }
    }
}

async fn run_task(
    shared: Arc<Shared>,
    request: RenderRequest,
    execution: u64,
    cancellation: Arc<AtomicCancellation>,
) {
    let id = request.task_id().clone();
    let preview = request.is_preview();
    let Ok(_permit) = shared.semaphore.clone().acquire_owned().await else {
        shared.fail(&id, execution, preview);
        return;
    };
    if cancellation.is_cancelled() {
        shared.cancel_if_needed(&id, execution, CancellationReason::User);
        return;
    }
    if !shared.transition_running(&id, execution, preview) {
        return;
    }

    let render_shared = Arc::clone(&shared);
    let render_id = id.clone();
    let render_cancellation = Arc::clone(&cancellation);
    let result = tokio::task::spawn_blocking(move || {
        let mut progress = |stage| {
            render_shared.update_progress(
                &render_id,
                execution,
                preview,
                render_cancellation.as_ref(),
                stage,
            );
        };
        render_shared
            .renderer
            .render(&request, render_cancellation.as_ref(), &mut progress)
    })
    .await;

    match result {
        Ok(Ok(result)) => shared.complete(
            &id,
            execution,
            preview,
            cancellation.as_ref(),
            result.resource().clone(),
        ),
        Ok(Err(error)) if error.code() == ErrorCode::Cancelled => {
            shared.cancel_if_needed(&id, execution, CancellationReason::User);
        }
        Ok(Err(error)) => {
            log::error!("render task failed: {}", error.safe_message());
            shared.fail(&id, execution, preview);
        }
        Err(error) => {
            let error = ApplicationError::internal(error.to_string());
            log::error!("render worker join failed: {}", error.safe_message());
            shared.fail(&id, execution, preview);
        }
    }
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "the function is passed directly to Result::map_err"
)]
fn internal_io(error: std::io::Error) -> ApplicationError {
    ApplicationError::internal(error.to_string())
}
