use crate::{DomainError, TaskId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RenderStage {
    Initializing,
    ReadingMetadata,
    PlanningBackground,
    PlanningText,
    PreparingMainImage,
    PlanningLayout,
    RenderingBackground,
    RenderingMask,
    Completed,
}

impl RenderStage {
    #[must_use]
    pub const fn percent(self) -> u8 {
        match self {
            Self::Initializing => 1,
            Self::ReadingMetadata => 10,
            Self::PlanningBackground => 20,
            Self::PlanningText => 30,
            Self::PreparingMainImage => 50,
            Self::PlanningLayout => 60,
            Self::RenderingBackground => 70,
            Self::RenderingMask => 90,
            Self::Completed => 100,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    Registered,
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CancellationReason {
    User,
    PreviewSuperseded,
    Cleared,
}

/// Applies a legal task state transition.
///
/// # Errors
///
/// Returns [`DomainError::IllegalTaskTransition`] for every transition not
/// enumerated by the product state machine.
pub const fn transition(from: TaskState, to: TaskState) -> Result<TaskState, DomainError> {
    match (from, to) {
        (TaskState::Registered, TaskState::Queued | TaskState::Running)
        | (TaskState::Queued, TaskState::Running | TaskState::Cancelled)
        | (TaskState::Running, TaskState::Completed | TaskState::Failed | TaskState::Cancelled) => {
            Ok(to)
        }
        _ => Err(DomainError::IllegalTaskTransition),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TaskStatus {
    task_id: TaskId,
    state: TaskState,
    stage: Option<RenderStage>,
    cancellation_reason: Option<CancellationReason>,
    preview: bool,
}

impl TaskStatus {
    #[must_use]
    pub const fn registered(task_id: TaskId) -> Self {
        Self {
            task_id,
            state: TaskState::Registered,
            stage: None,
            cancellation_reason: None,
            preview: false,
        }
    }

    #[must_use]
    pub fn preview(task_id: TaskId) -> Self {
        Self {
            preview: true,
            ..Self::registered(task_id)
        }
    }

    /// Transitions this status to another legal state.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::IllegalTaskTransition`] when the transition is not legal.
    pub fn transition_to(&mut self, state: TaskState) -> Result<(), DomainError> {
        self.state = transition(self.state, state)?;
        Ok(())
    }

    pub fn advance(&mut self, stage: RenderStage) {
        self.stage = Some(stage);
    }

    /// Cancels a queued or running task with an explicit reason.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::IllegalTaskTransition`] when the current state cannot cancel.
    pub fn cancel(&mut self, reason: CancellationReason) -> Result<(), DomainError> {
        self.transition_to(TaskState::Cancelled)?;
        self.cancellation_reason = Some(reason);
        Ok(())
    }

    #[must_use]
    pub const fn task_id(&self) -> &TaskId {
        &self.task_id
    }

    #[must_use]
    pub const fn state(&self) -> TaskState {
        self.state
    }

    #[must_use]
    pub const fn stage(&self) -> Option<RenderStage> {
        self.stage
    }

    #[must_use]
    pub const fn cancellation_reason(&self) -> Option<CancellationReason> {
        self.cancellation_reason
    }

    #[must_use]
    pub const fn is_preview(&self) -> bool {
        self.preview
    }
}

#[cfg(test)]
mod tests {
    use super::{CancellationReason, RenderStage, TaskState, TaskStatus, transition};
    use crate::{DomainError, TaskId};

    #[test]
    fn render_stages_preserve_progress_milestones() {
        let stages = [
            RenderStage::Initializing,
            RenderStage::ReadingMetadata,
            RenderStage::PlanningBackground,
            RenderStage::PlanningText,
            RenderStage::PreparingMainImage,
            RenderStage::PlanningLayout,
            RenderStage::RenderingBackground,
            RenderStage::RenderingMask,
            RenderStage::Completed,
        ];

        assert_eq!(
            stages.map(RenderStage::percent),
            [1, 10, 20, 30, 50, 60, 70, 90, 100]
        );
    }

    #[test]
    fn task_state_transitions_are_closed_and_explicit() {
        assert_eq!(
            transition(TaskState::Registered, TaskState::Queued),
            Ok(TaskState::Queued)
        );
        assert_eq!(
            transition(TaskState::Registered, TaskState::Running),
            Ok(TaskState::Running)
        );
        assert_eq!(
            transition(TaskState::Queued, TaskState::Running),
            Ok(TaskState::Running)
        );
        assert_eq!(
            transition(TaskState::Queued, TaskState::Cancelled),
            Ok(TaskState::Cancelled)
        );
        assert_eq!(
            transition(TaskState::Running, TaskState::Completed),
            Ok(TaskState::Completed)
        );
        assert_eq!(
            transition(TaskState::Running, TaskState::Failed),
            Ok(TaskState::Failed)
        );
        assert_eq!(
            transition(TaskState::Running, TaskState::Cancelled),
            Ok(TaskState::Cancelled)
        );
        assert_eq!(
            transition(TaskState::Completed, TaskState::Running),
            Err(DomainError::IllegalTaskTransition),
        );
    }

    #[test]
    fn preview_supersession_is_an_explicit_cancellation() {
        let id = TaskId::try_from("preview-a").expect("task id");
        let mut status = TaskStatus::preview(id);
        status
            .transition_to(TaskState::Running)
            .expect("preview starts");
        status
            .cancel(CancellationReason::PreviewSuperseded)
            .expect("running preview cancels");

        assert_eq!(status.state(), TaskState::Cancelled);
        assert_eq!(
            status.cancellation_reason(),
            Some(CancellationReason::PreviewSuperseded)
        );
        assert!(status.is_preview());
    }
}
