use std::{collections::BTreeSet, sync::Arc};

use yiyin_domain::{Config, FieldContentKind, OutputDirectory, ResourceKind};

use crate::{ApplicationError, ConfigRepository, OutputDirectoryGateway, ResourceRepository};

pub struct UpdateConfig {
    repository: Arc<dyn ConfigRepository>,
    resources: Arc<dyn ResourceRepository>,
}

impl UpdateConfig {
    #[must_use]
    pub const fn new(
        repository: Arc<dyn ConfigRepository>,
        resources: Arc<dyn ResourceRepository>,
    ) -> Self {
        Self {
            repository,
            resources,
        }
    }

    /// Validates and persists a complete configuration transaction.
    ///
    /// # Errors
    ///
    /// Returns `CONFIG_INVALID` without writing when required values, stable
    /// keys, or image resource references are invalid, otherwise forwards
    /// repository errors. The output directory needs no check here: its value
    /// object is always non-empty.
    pub fn execute(&self, config: Config) -> Result<Config, ApplicationError> {
        validate(&config)?;
        self.validate_resource_references(&config)?;
        self.repository.store(&config)?;
        Ok(config)
    }

    /// Image template fields may only reference bundled assets or overlays.
    fn validate_resource_references(&self, config: &Config) -> Result<(), ApplicationError> {
        for id in config
            .temp_fields
            .iter()
            .chain(&config.custom_temp_fields)
            .filter(|field| field.content_kind() == FieldContentKind::Image)
            .flat_map(|field| [field.dark_image(), field.light_image()])
            .flatten()
        {
            let valid = self.resources.resolve(id).is_ok_and(|record| {
                matches!(
                    record.kind(),
                    ResourceKind::BundledAsset | ResourceKind::Overlay
                )
            });
            if !valid {
                return Err(ApplicationError::config_invalid());
            }
        }
        Ok(())
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

pub struct SetOutputDirectory {
    repository: Arc<dyn ConfigRepository>,
    output: Arc<dyn OutputDirectoryGateway>,
}

impl SetOutputDirectory {
    #[must_use]
    pub const fn new(
        repository: Arc<dyn ConfigRepository>,
        output: Arc<dyn OutputDirectoryGateway>,
    ) -> Self {
        Self { repository, output }
    }

    /// Moves exports to a new root: probe, persist, then swap.
    ///
    /// # Errors
    ///
    /// A failed probe changes nothing; a failed persist leaves at most a
    /// created directory with the previous root still live; the swap can only
    /// fail on a poisoned lock. This is the only write path for the output
    /// directory (ADR 0002).
    pub fn execute(&self, output: OutputDirectory) -> Result<Config, ApplicationError> {
        let mut config = self.repository.load()?;
        config.output = output;
        self.output.ensure_root(&config.output)?;
        self.repository.store(&config)?;
        self.output.change_root(&config.output)?;
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
