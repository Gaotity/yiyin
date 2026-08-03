# Task status events flow through a single seam

A freshly registered task's initial status reached the frontend twice: `TokioTaskQueue::register` published it through the `TaskEventSink` port, and the drag-and-drop handler emitted the same `task-status` events directly — which forced both the handler and `register_image_paths` to snapshot existing task ids before registration and diff the full queue snapshot afterwards just to learn which tasks were new. We decided the use case already knows the answer: `RegisterImages::execute` mints the task ids itself, so it now returns exactly the tasks it registered, in registration order, and no layer computes a newness diff. Task-status notifications flow only through `TaskEventSink`; the drop handler keeps only its error path (`drop-error`), because errors are not task statuses.

## Consequences

Rerouting the drop emit through the sink instead of deleting it was rejected: `register` already publishes the initial status, so a second publish would double-emit. The `From<&TaskDescriptorDto> for TaskStatusEventDto` conversion that fed the bypass lost its only caller and was removed, so a future adapter-side status emit has no off-seam conversion to reach for. Registration order, not queue order, now defines which tasks quick-output starts — identical today, but no longer dependent on snapshot timing.
