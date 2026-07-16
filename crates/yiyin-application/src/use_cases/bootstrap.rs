use std::sync::Arc;

use crate::{
    ApplicationError, BootstrapSnapshot, ConfigRepository, ResourceRepository, ResourceSnapshot,
    TaskQueue,
};

pub struct Bootstrap {
    config: Arc<dyn ConfigRepository>,
    resources: Arc<dyn ResourceRepository>,
    tasks: Arc<dyn TaskQueue>,
}

impl Bootstrap {
    #[must_use]
    pub const fn new(
        config: Arc<dyn ConfigRepository>,
        resources: Arc<dyn ResourceRepository>,
        tasks: Arc<dyn TaskQueue>,
    ) -> Self {
        Self {
            config,
            resources,
            tasks,
        }
    }

    /// Imports legacy state once and returns a presentation-safe snapshot.
    ///
    /// # Errors
    ///
    /// Returns the first repository error without exposing its internal source.
    pub fn execute(&self) -> Result<BootstrapSnapshot, ApplicationError> {
        let import = self.config.import_legacy_if_needed()?;
        let config = self.config.load()?;
        let resources = self
            .resources
            .snapshot()
            .iter()
            .map(ResourceSnapshot::from_record)
            .collect();
        Ok(BootstrapSnapshot::new(
            config,
            resources,
            self.tasks.snapshot(),
            import.warnings().to_vec(),
        ))
    }
}
