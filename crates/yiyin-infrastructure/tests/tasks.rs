use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use yiyin_application::{
    ApplicationError, CancellationProbe, ErrorCode, ImageRenderer, RegisteredTask, RenderResult,
    ResourceRecord, ResourceSnapshot, TaskEventSink, TaskQueue,
};
use yiyin_domain::{
    CancellationReason, Config, ImageDimensions, Metadata, RenderRequest, RenderStage, ResourceId,
    ResourceKind, TaskId, TaskState, TaskStatus,
};
use yiyin_infrastructure::TokioTaskQueue;

const WAIT: Duration = Duration::from_secs(3);

struct RenderActivity<'a>(&'a ControlledRenderer);

impl Drop for RenderActivity<'_> {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::AcqRel);
        self.0.changed.notify_all();
    }
}

struct ControlledRenderer {
    released: Mutex<bool>,
    changed: Condvar,
    active: AtomicUsize,
    maximum: AtomicUsize,
    started: AtomicUsize,
    honor_cancellation: bool,
}

impl ControlledRenderer {
    fn new(honor_cancellation: bool) -> Self {
        Self {
            released: Mutex::new(false),
            changed: Condvar::new(),
            active: AtomicUsize::new(0),
            maximum: AtomicUsize::new(0),
            started: AtomicUsize::new(0),
            honor_cancellation,
        }
    }

    fn wait_for_started(&self, expected: usize) {
        let deadline = Instant::now() + WAIT;
        let mut released = self.released.lock().expect("renderer lock");
        while self.started.load(Ordering::Acquire) < expected {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(
                !remaining.is_zero(),
                "renderer did not start {expected} jobs"
            );
            let (next, timeout) = self
                .changed
                .wait_timeout(released, remaining)
                .expect("renderer wait");
            released = next;
            assert!(
                !timeout.timed_out() || self.started.load(Ordering::Acquire) >= expected,
                "renderer did not start {expected} jobs"
            );
        }
    }

    fn release_all(&self) {
        *self.released.lock().expect("renderer lock") = true;
        self.changed.notify_all();
    }

    fn maximum_concurrency(&self) -> usize {
        self.maximum.load(Ordering::Acquire)
    }

    fn started(&self) -> usize {
        self.started.load(Ordering::Acquire)
    }
}

impl ImageRenderer for ControlledRenderer {
    fn render(
        &self,
        request: &RenderRequest,
        cancellation: &dyn CancellationProbe,
        progress: &mut dyn FnMut(RenderStage),
    ) -> Result<RenderResult, ApplicationError> {
        let active = self.active.fetch_add(1, Ordering::AcqRel) + 1;
        self.maximum.fetch_max(active, Ordering::AcqRel);
        self.started.fetch_add(1, Ordering::AcqRel);
        self.changed.notify_all();
        let _activity = RenderActivity(self);

        let mut released = self.released.lock().expect("renderer lock");
        while !*released {
            if self.honor_cancellation && cancellation.is_cancelled() {
                return Err(ApplicationError::cancelled());
            }
            let (next, _) = self
                .changed
                .wait_timeout(released, Duration::from_millis(5))
                .expect("renderer wait");
            released = next;
        }
        drop(released);
        if self.honor_cancellation && cancellation.is_cancelled() {
            return Err(ApplicationError::cancelled());
        }
        for stage in [
            RenderStage::Initializing,
            RenderStage::ReadingMetadata,
            RenderStage::PlanningBackground,
            RenderStage::PlanningText,
            RenderStage::PreparingMainImage,
            RenderStage::PlanningLayout,
            RenderStage::RenderingBackground,
            RenderStage::RenderingMask,
            RenderStage::Completed,
        ] {
            progress(stage);
        }
        let kind = if request.is_preview() {
            ResourceKind::Preview
        } else {
            ResourceKind::Output
        };
        let record = ResourceRecord::new(
            ResourceId::try_from(format!("result-{}", request.task_id().as_str()))
                .expect("resource id"),
            kind,
            request.output_name(),
            PathBuf::from(format!("/private/result/{}", request.output_name())),
        );
        Ok(RenderResult::new(
            request.task_id().clone(),
            ResourceSnapshot::from_record(&record),
            ImageDimensions::new(100, 100).expect("dimensions"),
            None,
            if request.is_preview() { 70 } else { 100 },
        ))
    }
}

#[derive(Default)]
struct RecordingEvents {
    statuses: Mutex<Vec<TaskStatus>>,
    changed: Condvar,
}

impl RecordingEvents {
    fn for_task(&self, id: &TaskId) -> Vec<TaskStatus> {
        self.statuses
            .lock()
            .expect("events lock")
            .iter()
            .filter(|status| status.task_id() == id)
            .cloned()
            .collect()
    }

    fn wait_for_state(&self, id: &TaskId, state: TaskState) {
        let deadline = Instant::now() + WAIT;
        let mut statuses = self.statuses.lock().expect("events lock");
        while !statuses
            .iter()
            .any(|status| status.task_id() == id && status.state() == state)
        {
            let remaining = deadline.saturating_duration_since(Instant::now());
            assert!(!remaining.is_zero(), "missing {state:?} event for {id:?}");
            let (next, timeout) = self
                .changed
                .wait_timeout(statuses, remaining)
                .expect("events wait");
            statuses = next;
            assert!(
                !timeout.timed_out()
                    || statuses
                        .iter()
                        .any(|status| status.task_id() == id && status.state() == state),
                "missing {state:?} event for {id:?}"
            );
        }
    }
}

impl TaskEventSink for RecordingEvents {
    fn publish(&self, status: TaskStatus) {
        self.statuses.lock().expect("events lock").push(status);
        self.changed.notify_all();
    }
}

struct Harness {
    renderer: Arc<ControlledRenderer>,
    events: Arc<RecordingEvents>,
    queue: Arc<TokioTaskQueue>,
    ids: HashMap<String, TaskId>,
}

impl Harness {
    fn new(honor_cancellation: bool) -> Self {
        let renderer = Arc::new(ControlledRenderer::new(honor_cancellation));
        let events = Arc::new(RecordingEvents::default());
        let queue = Arc::new(
            TokioTaskQueue::new(renderer.clone(), events.clone()).expect("task queue runtime"),
        );
        Self {
            renderer,
            events,
            queue,
            ids: HashMap::new(),
        }
    }

    fn register(&mut self, value: &str) -> TaskId {
        let id = TaskId::try_from(value).expect("task id");
        self.queue
            .register(registered_task(&id))
            .expect("register task");
        self.ids.insert(value.to_owned(), id.clone());
        id
    }

    fn enqueue(&self, id: &TaskId) {
        self.queue
            .enqueue(render_request(id, false))
            .expect("enqueue");
    }
}

fn registered_task(id: &TaskId) -> RegisteredTask {
    RegisteredTask::new(
        id.clone(),
        ResourceId::try_from(format!("input-{}", id.as_str())).expect("resource id"),
        format!("{}.jpg", id.as_str()),
        ImageDimensions::new(100, 100).expect("dimensions"),
        None,
    )
}

fn render_request(id: &TaskId, preview: bool) -> RenderRequest {
    let request = RenderRequest::freeze(
        id.clone(),
        ResourceId::try_from(format!("input-{}", id.as_str())).expect("resource id"),
        format!("{}.jpg", id.as_str()),
        ImageDimensions::new(100, 100).expect("dimensions"),
        Config::default(),
        Metadata::default(),
    );
    if preview {
        request.as_preview()
    } else {
        request
    }
}

#[test]
fn registered_tasks_stay_idle_and_exports_run_with_exactly_two_workers() {
    let mut harness = Harness::new(true);
    let first = harness.register("first");
    let second = harness.register("second");
    let third = harness.register("third");

    std::thread::sleep(Duration::from_millis(30));
    assert_eq!(harness.renderer.started(), 0);
    harness.enqueue(&first);
    harness.enqueue(&second);
    harness.enqueue(&third);
    harness.renderer.wait_for_started(2);
    assert_eq!(harness.renderer.started(), 2);

    harness.renderer.release_all();
    harness.events.wait_for_state(&first, TaskState::Completed);
    harness.events.wait_for_state(&second, TaskState::Completed);
    harness.events.wait_for_state(&third, TaskState::Completed);
    assert_eq!(harness.renderer.maximum_concurrency(), 2);
    harness.queue.shutdown().expect("shutdown");
}

#[test]
fn registration_publishes_the_initial_status_through_the_sink() {
    let mut harness = Harness::new(true);
    let id = harness.register("announced");

    let events = harness.events.for_task(&id);
    assert_eq!(events.len(), 1);
    assert_eq!(
        events.first().expect("registration event").state(),
        TaskState::Registered
    );
    harness.queue.shutdown().expect("shutdown");
}

#[test]
fn progress_is_monotonic_and_completion_contains_the_output_resource() {
    let mut harness = Harness::new(true);
    let id = harness.register("progress");
    harness.enqueue(&id);
    harness.renderer.wait_for_started(1);
    harness.renderer.release_all();
    harness.events.wait_for_state(&id, TaskState::Completed);

    let progress = harness
        .events
        .for_task(&id)
        .iter()
        .filter_map(TaskStatus::stage)
        .map(RenderStage::percent)
        .collect::<Vec<_>>();
    assert_eq!(progress, [1, 10, 20, 30, 50, 60, 70, 90, 100]);
    let snapshot = harness.queue.snapshot();
    assert_eq!(snapshot[0].state(), TaskState::Completed);
    assert_eq!(
        snapshot[0].resource().expect("output resource").kind(),
        ResourceKind::Output
    );
    harness.queue.shutdown().expect("shutdown");
}

#[test]
fn explicit_cancellation_is_terminal_even_when_the_renderer_returns_success() {
    let mut harness = Harness::new(false);
    let id = harness.register("cancelled");
    harness.enqueue(&id);
    harness.renderer.wait_for_started(1);

    harness.queue.cancel(&id).expect("cancel task");
    harness.renderer.release_all();
    harness.events.wait_for_state(&id, TaskState::Cancelled);
    harness.queue.shutdown().expect("shutdown");

    let events = harness.events.for_task(&id);
    assert_eq!(
        events.last().expect("terminal event").state(),
        TaskState::Cancelled
    );
    assert_eq!(
        events.last().expect("terminal event").cancellation_reason(),
        Some(CancellationReason::User)
    );
    assert!(
        !events
            .iter()
            .any(|status| status.state() == TaskState::Completed)
    );
}

#[test]
fn latest_preview_wins_when_a_stale_renderer_ignores_cancellation() {
    let mut harness = Harness::new(false);
    let stale = harness.register("stale-preview");
    let latest = harness.register("latest-preview");
    harness
        .queue
        .preview(render_request(&stale, true))
        .expect("first preview");
    harness.renderer.wait_for_started(1);
    harness
        .queue
        .preview(render_request(&latest, true))
        .expect("replacement preview");
    harness.renderer.wait_for_started(2);

    harness.renderer.release_all();
    harness.events.wait_for_state(&stale, TaskState::Cancelled);
    harness.events.wait_for_state(&latest, TaskState::Completed);
    harness.queue.shutdown().expect("shutdown");

    let stale_events = harness.events.for_task(&stale);
    assert_eq!(
        stale_events
            .last()
            .expect("stale terminal")
            .cancellation_reason(),
        Some(CancellationReason::PreviewSuperseded)
    );
    assert!(
        !stale_events
            .iter()
            .any(|status| status.state() == TaskState::Completed)
    );
    let latest_snapshot = harness
        .queue
        .snapshot()
        .into_iter()
        .find(|snapshot| snapshot.id() == &latest)
        .expect("latest snapshot");
    assert!(latest_snapshot.is_preview());
    assert_eq!(
        latest_snapshot.resource().expect("preview resource").kind(),
        ResourceKind::Preview
    );
}

#[test]
fn export_supersedes_an_active_preview_for_the_same_task() {
    let mut harness = Harness::new(false);
    let id = harness.register("preview-then-export");
    harness
        .queue
        .preview(render_request(&id, true))
        .expect("preview");
    harness.renderer.wait_for_started(1);

    harness
        .queue
        .enqueue(render_request(&id, false))
        .expect("export supersedes preview");
    harness.renderer.wait_for_started(2);
    harness.renderer.release_all();
    harness.events.wait_for_state(&id, TaskState::Completed);
    harness.queue.shutdown().expect("shutdown");

    let events = harness.events.for_task(&id);
    assert!(events.iter().any(|status| {
        status.state() == TaskState::Cancelled
            && status.cancellation_reason() == Some(CancellationReason::PreviewSuperseded)
    }));
    let snapshot = harness.queue.snapshot();
    assert!(!snapshot[0].is_preview());
    assert_eq!(
        snapshot[0].resource().expect("output resource").kind(),
        ResourceKind::Output
    );
}

#[test]
fn clear_cancels_active_and_pending_work_before_removing_the_snapshot() {
    let mut harness = Harness::new(false);
    let first = harness.register("clear-first");
    let second = harness.register("clear-second");
    let pending = harness.register("clear-pending");
    harness.enqueue(&first);
    harness.enqueue(&second);
    harness.enqueue(&pending);
    harness.renderer.wait_for_started(2);

    harness.queue.clear().expect("clear queue");
    assert!(harness.queue.snapshot().is_empty());
    harness.renderer.release_all();
    harness.queue.shutdown().expect("shutdown");

    for id in [first, second, pending] {
        let events = harness.events.for_task(&id);
        assert_eq!(
            events.last().expect("clear terminal").state(),
            TaskState::Cancelled
        );
        assert_eq!(
            events.last().expect("clear terminal").cancellation_reason(),
            Some(CancellationReason::Cleared)
        );
    }
}

#[test]
fn shutdown_cancels_active_work_and_rejects_new_tasks() {
    let mut harness = Harness::new(true);
    let active = harness.register("shutdown-active");
    harness.enqueue(&active);
    harness.renderer.wait_for_started(1);

    harness.queue.shutdown().expect("shutdown queue");
    let events = harness.events.for_task(&active);
    assert_eq!(
        events.last().expect("shutdown terminal").state(),
        TaskState::Cancelled
    );
    assert_eq!(
        events
            .last()
            .expect("shutdown terminal")
            .cancellation_reason(),
        Some(CancellationReason::Shutdown)
    );
    assert_eq!(
        harness
            .queue
            .register(registered_task(
                &TaskId::try_from("after-shutdown").expect("task id")
            ))
            .unwrap_err()
            .code(),
        ErrorCode::InvalidRequest
    );
}
