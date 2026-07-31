use std::{collections::BTreeSet, sync::Arc};

use yiyin_domain::Config;

use crate::{ApplicationError, ConfigRepository};

pub struct UpdateConfig {
    repository: Arc<dyn ConfigRepository>,
}

impl UpdateConfig {
    #[must_use]
    pub const fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }

    /// Validates and persists a complete configuration transaction.
    ///
    /// # Errors
    ///
    /// Returns `CONFIG_INVALID` without writing when required values or stable
    /// keys are invalid, otherwise forwards repository errors. The output
    /// directory needs no check here: its value object is always non-empty.
    pub fn execute(&self, config: Config) -> Result<Config, ApplicationError> {
        validate(&config)?;
        self.repository.store(&config)?;
        Ok(config)
    }
}

pub struct ResetConfig {
    repository: Arc<dyn ConfigRepository>,
}

impl ResetConfig {
    #[must_use]
    pub const fn new(repository: Arc<dyn ConfigRepository>) -> Self {
        Self { repository }
    }

    /// Restores and persists the complete v1.6 default model.
    ///
    /// # Errors
    ///
    /// Forwards repository persistence errors.
    pub fn execute(&self) -> Result<Config, ApplicationError> {
        let config = Config::default();
        self.repository.store(&config)?;
        Ok(config)
    }
}

fn validate(config: &Config) -> Result<(), ApplicationError> {
    if config.version.trim().is_empty() {
        return Err(ApplicationError::config_invalid());
    }

    let mut field_keys = BTreeSet::new();
    if config
        .temp_fields
        .iter()
        .chain(&config.custom_temp_fields)
        .any(|field| !field_keys.insert(field.key().as_str()))
    {
        return Err(ApplicationError::config_invalid());
    }

    let mut template_keys = BTreeSet::new();
    if config
        .templates
        .iter()
        .any(|template| !template_keys.insert(template.key()))
    {
        return Err(ApplicationError::config_invalid());
    }

    Ok(())
}
