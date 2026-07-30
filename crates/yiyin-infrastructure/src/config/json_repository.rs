use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use yiyin_application::{ApplicationError, ConfigRepository, ImportOutcome};
use yiyin_domain::{
    BackgroundBlur, BackgroundRatio, CURRENT_VERSION, CaseConversion, Config, FieldContentKind,
    FontFamily, FontSpec, MainImageWidth, MiniTopBottomMargin, Quality, Radius, RenderOptions,
    ResourceId, Shadow, SolidColor, Template, TemplateField, TemplateKind, TextMargin,
    VerticalAlign, default_template_fields, default_templates,
};

use crate::{FileSystem, StdFileSystem};

use super::LegacyImportOptions;

const CURRENT_CONFIG_VERSION: u32 = 1;

pub struct JsonConfigRepository {
    path: PathBuf,
    filesystem: Arc<dyn FileSystem>,
    legacy: Option<LegacyImportOptions>,
}

impl JsonConfigRepository {
    #[must_use]
    pub fn new(path: PathBuf) -> Self {
        Self::with_filesystem(path, Arc::new(StdFileSystem))
    }

    #[must_use]
    pub fn with_filesystem(path: PathBuf, filesystem: Arc<dyn FileSystem>) -> Self {
        Self {
            path,
            filesystem,
            legacy: None,
        }
    }

    #[must_use]
    pub fn with_legacy_import(mut self, options: LegacyImportOptions) -> Self {
        self.legacy = Some(options);
        self
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn filesystem(&self) -> &dyn FileSystem {
        self.filesystem.as_ref()
    }

    pub(crate) fn legacy_options(&self) -> Option<&LegacyImportOptions> {
        self.legacy.as_ref()
    }

    fn recover_invalid(&self, invalid_contents: &[u8]) -> Result<Config, ApplicationError> {
        let invalid_backup = sibling_with_suffix(&self.path, ".invalid.bak");
        self.filesystem
            .write_and_sync(&invalid_backup, invalid_contents)
            .map_err(internal_io)?;

        let backup = sibling_with_suffix(&self.path, ".bak");
        let recovered = if self.filesystem.exists(&backup) {
            self.filesystem
                .read(&backup)
                .ok()
                .and_then(|bytes| decode_config(&bytes).ok())
                .unwrap_or_default()
        } else {
            Config::default()
        };
        self.store(&recovered)?;
        Ok(recovered)
    }
}

impl ConfigRepository for JsonConfigRepository {
    fn load(&self) -> Result<Config, ApplicationError> {
        if !self.filesystem.exists(&self.path) {
            let config = Config::default();
            self.store(&config)?;
            return Ok(config);
        }

        let bytes = self.filesystem.read(&self.path).map_err(internal_io)?;
        match decode_config(&bytes) {
            Ok(config) => Ok(config),
            Err(DecodeFailure::FutureVersion) => Err(ApplicationError::config_invalid()),
            Err(DecodeFailure::Invalid) => self.recover_invalid(&bytes),
        }
    }

    fn store(&self, config: &Config) -> Result<(), ApplicationError> {
        let bytes = encode_config(config)?;
        atomic_write(self.filesystem.as_ref(), &self.path, &bytes, |contents| {
            decode_config(contents).is_ok()
        })
    }

    fn import_legacy_if_needed(&self) -> Result<ImportOutcome, ApplicationError> {
        super::legacy_import::import_if_needed(self)
    }
}

pub(crate) fn atomic_write(
    filesystem: &dyn FileSystem,
    path: &Path,
    contents: &[u8],
    prior_is_valid: impl Fn(&[u8]) -> bool,
) -> Result<(), ApplicationError> {
    let parent = path
        .parent()
        .ok_or_else(|| ApplicationError::internal("configuration path has no parent"))?;
    filesystem.create_dir_all(parent).map_err(internal_io)?;

    let temporary = sibling_with_suffix(path, ".tmp");
    if temporary.parent() != Some(parent) {
        return Err(ApplicationError::forbidden());
    }
    if filesystem.exists(&temporary) {
        remove_file_if_present(filesystem, &temporary)?;
    }

    if let Err(error) = filesystem.write_and_sync(&temporary, contents) {
        let _ = remove_file_if_present(filesystem, &temporary);
        return Err(internal_io(error));
    }

    let backup = sibling_with_suffix(path, ".bak");
    let prior_valid = filesystem
        .read(path)
        .ok()
        .filter(|bytes| prior_is_valid(bytes));
    if prior_valid.is_some()
        && let Err(error) = filesystem.copy(path, &backup)
    {
        let _ = remove_file_if_present(filesystem, &temporary);
        return Err(internal_io(error));
    }

    if let Err(error) = filesystem.rename(&temporary, path) {
        let _ = remove_file_if_present(filesystem, &temporary);
        return Err(internal_io(error));
    }

    if let Err(error) = filesystem.sync_parent(path) {
        if prior_valid.is_some() && filesystem.exists(&backup) {
            let _ = filesystem.copy(&backup, path);
        } else {
            let _ = remove_file_if_present(filesystem, path);
        }
        return Err(internal_io(error));
    }
    Ok(())
}

fn remove_file_if_present(
    filesystem: &dyn FileSystem,
    path: &Path,
) -> Result<(), ApplicationError> {
    if filesystem.exists(path) {
        filesystem.remove_file(path).map_err(internal_io)?;
    }
    Ok(())
}

fn sibling_with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map_or_else(OsString::new, std::ffi::OsStr::to_os_string);
    name.push(suffix);
    path.with_file_name(name)
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "the function is passed directly to Result::map_err"
)]
fn internal_io(error: io::Error) -> ApplicationError {
    ApplicationError::internal(error.to_string())
}

#[derive(Debug)]
enum DecodeFailure {
    FutureVersion,
    Invalid,
}

fn encode_config(config: &Config) -> Result<Vec<u8>, ApplicationError> {
    serde_json::to_vec_pretty(&StoredConfigEnvelope {
        version: CURRENT_CONFIG_VERSION,
        config: StoredConfigV1::from_domain(config),
    })
    .map_err(|error| ApplicationError::internal(error.to_string()))
}

pub(crate) struct DecodedLegacyConfig {
    pub config: Config,
    pub image_references: Vec<LegacyImageReference>,
    pub warnings: Vec<String>,
}

pub(crate) struct LegacyImageReference {
    pub id: ResourceId,
    pub source: String,
}

pub(crate) fn decode_legacy_config(contents: &[u8]) -> DecodedLegacyConfig {
    let mut warnings = Vec::new();
    let value = serde_json::from_slice::<serde_json::Value>(contents)
        .ok()
        .and_then(|value| value.as_object().cloned());
    let Some(object) = value else {
        warnings.push("Legacy configuration is unreadable; defaults were imported.".to_owned());
        return DecodedLegacyConfig {
            config: Config::default(),
            image_references: Vec::new(),
            warnings,
        };
    };
    let mut known = serde_json::Map::new();
    for key in [
        "version",
        "output",
        "options",
        "tempFields",
        "customTempFields",
        "temps",
    ] {
        if let Some(value) = object.get(key) {
            known.insert(key.to_owned(), value.clone());
        }
    }
    if let Some(serde_json::Value::Array(templates)) = known.get_mut("temps") {
        for template in templates {
            if let serde_json::Value::Object(template) = template {
                // The legacy app persisted this unused layout placeholder on every template.
                template.remove("position");
            }
        }
    }

    let defaults = Config::default();
    // The imported configuration now runs on the new app; stamp it with the
    // current version (the migration marker records the legacy provenance).
    let version = CURRENT_VERSION.to_owned();
    let output = salvage_string(&known, "output", &defaults.output, &mut warnings);
    let options = salvage_options(known.get("options"), &mut warnings);
    let mut image_references = Vec::new();
    let mut temp_fields = salvage_fields(
        known.get("tempFields"),
        &mut image_references,
        &mut warnings,
    );
    fill_default_fields(&mut temp_fields);
    let custom_temp_fields = salvage_fields(
        known.get("customTempFields"),
        &mut image_references,
        &mut warnings,
    );
    let mut templates = salvage_templates(known.get("temps"), &mut warnings);
    fill_default_templates(&mut templates);

    DecodedLegacyConfig {
        config: Config {
            version,
            output,
            options,
            temp_fields,
            custom_temp_fields,
            templates,
        },
        image_references,
        warnings,
    }
}

fn fill_default_fields(fields: &mut Vec<TemplateField>) {
    for field in default_template_fields() {
        if !fields.iter().any(|existing| existing.key() == field.key()) {
            fields.push(field);
        }
    }
}

fn fill_default_templates(templates: &mut Vec<Template>) {
    for template in default_templates() {
        if !templates
            .iter()
            .any(|existing| existing.key() == template.key())
        {
            templates.push(template);
        }
    }
}

fn legacy_element_label(element: &serde_json::Value) -> String {
    element
        .get("key")
        .or_else(|| element.get("name"))
        .and_then(serde_json::Value::as_str)
        .unwrap_or("<unnamed>")
        .to_owned()
}

fn salvage_string(
    known: &serde_json::Map<String, serde_json::Value>,
    key: &str,
    default: &str,
    warnings: &mut Vec<String>,
) -> String {
    let Some(value) = known.get(key) else {
        return default.to_owned();
    };
    if let Some(text) = value.as_str().filter(|text| !text.trim().is_empty()) {
        return text.to_owned();
    }
    warnings.push(format!("Skipped invalid legacy value: {key}"));
    default.to_owned()
}

fn salvage_options(value: Option<&serde_json::Value>, warnings: &mut Vec<String>) -> RenderOptions {
    let default_options = RenderOptions::default();
    let Some(value) = value else {
        return default_options;
    };
    let Some(options) = value.as_object() else {
        warnings.push("Skipped invalid legacy options block.".to_owned());
        return default_options;
    };
    let mut salvaged = default_options;
    for (key, field_value) in options {
        let mut candidate = serde_json::to_value(StoredOptions::from_domain(&salvaged))
            .expect("salvaged options serialize");
        candidate
            .as_object_mut()
            .expect("options object")
            .insert(key.clone(), field_value.clone());
        let next = serde_json::from_value::<StoredOptions>(candidate)
            .ok()
            .and_then(|stored| stored.into_domain().ok());
        match next {
            Some(options) => salvaged = options,
            None => warnings.push(format!("Skipped invalid legacy option: {key}")),
        }
    }
    salvaged
}

fn salvage_fields(
    value: Option<&serde_json::Value>,
    image_references: &mut Vec<LegacyImageReference>,
    warnings: &mut Vec<String>,
) -> Vec<TemplateField> {
    let Some(elements) = value.and_then(serde_json::Value::as_array) else {
        if value.is_some() {
            warnings.push("Skipped invalid legacy fields block.".to_owned());
        }
        return Vec::new();
    };
    let mut fields = Vec::new();
    for element in elements {
        let label = legacy_element_label(element);
        let Ok(mut stored) = serde_json::from_value::<StoredField>(element.clone()) else {
            warnings.push(format!("Skipped invalid legacy field: {label}"));
            continue;
        };
        let mut field_references = Vec::new();
        if sanitize_legacy_image_reference(&mut stored.dark_image, &mut field_references).is_err()
            || sanitize_legacy_image_reference(&mut stored.light_image, &mut field_references)
                .is_err()
        {
            warnings.push(format!("Skipped invalid legacy field: {label}"));
            continue;
        }
        match stored.into_domain() {
            Ok(field) => {
                image_references.append(&mut field_references);
                fields.push(field);
            }
            Err(_) => warnings.push(format!("Skipped invalid legacy field: {label}")),
        }
    }
    fields
}

fn salvage_templates(
    value: Option<&serde_json::Value>,
    warnings: &mut Vec<String>,
) -> Vec<Template> {
    let Some(elements) = value.and_then(serde_json::Value::as_array) else {
        if value.is_some() {
            warnings.push("Skipped invalid legacy templates block.".to_owned());
        }
        return Vec::new();
    };
    let mut templates = Vec::new();
    for element in elements {
        let label = legacy_element_label(element);
        match serde_json::from_value::<StoredTemplate>(element.clone())
            .ok()
            .and_then(|stored| stored.into_domain().ok())
        {
            Some(template) => templates.push(template),
            None => warnings.push(format!("Skipped invalid legacy template: {label}")),
        }
    }
    templates
}

fn sanitize_legacy_image_reference(
    value: &mut String,
    references: &mut Vec<LegacyImageReference>,
) -> Result<(), ApplicationError> {
    if value.trim().is_empty() || value == "false" {
        value.clear();
        return Ok(());
    }
    let source = std::mem::take(value);
    let id =
        ResourceId::try_from(format!("legacy-{}", uuid::Uuid::new_v4())).map_err(invalid_domain)?;
    id.as_str().clone_into(value);
    references.push(LegacyImageReference { id, source });
    Ok(())
}

fn decode_config(contents: &[u8]) -> Result<Config, DecodeFailure> {
    let value: serde_json::Value =
        serde_json::from_slice(contents).map_err(|_| DecodeFailure::Invalid)?;
    let version = value
        .get("version")
        .and_then(serde_json::Value::as_u64)
        .ok_or(DecodeFailure::Invalid)?;
    if version != u64::from(CURRENT_CONFIG_VERSION) {
        return Err(DecodeFailure::FutureVersion);
    }
    let mut config = serde_json::from_value::<StoredConfigEnvelope>(value)
        .map_err(|_| DecodeFailure::Invalid)?
        .config
        .into_domain()
        .map_err(|_| DecodeFailure::Invalid)?;
    // The footer displays this as the running app's version, so an upgraded
    // install always reads the current one.
    CURRENT_VERSION.clone_into(&mut config.version);
    Ok(config)
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoredConfigEnvelope {
    version: u32,
    config: StoredConfigV1,
}

#[derive(Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct StoredConfigV1 {
    version: String,
    output: String,
    options: StoredOptions,
    #[serde(rename = "tempFields")]
    temp_fields: Vec<StoredField>,
    #[serde(rename = "customTempFields")]
    custom_temp_fields: Vec<StoredField>,
    #[serde(rename = "temps")]
    templates: Vec<StoredTemplate>,
}

impl Default for StoredConfigV1 {
    fn default() -> Self {
        Self::from_domain(&Config::default())
    }
}

impl StoredConfigV1 {
    fn from_domain(config: &Config) -> Self {
        Self {
            version: config.version.clone(),
            output: config.output.clone(),
            options: StoredOptions::from_domain(&config.options),
            temp_fields: config
                .temp_fields
                .iter()
                .map(StoredField::from_domain)
                .collect(),
            custom_temp_fields: config
                .custom_temp_fields
                .iter()
                .map(StoredField::from_domain)
                .collect(),
            templates: config
                .templates
                .iter()
                .map(StoredTemplate::from_domain)
                .collect(),
        }
    }

    fn into_domain(self) -> Result<Config, ApplicationError> {
        let mut temp_fields = self
            .temp_fields
            .into_iter()
            .map(StoredField::into_domain)
            .collect::<Result<Vec<_>, _>>()?;
        fill_default_fields(&mut temp_fields);

        let custom_temp_fields = self
            .custom_temp_fields
            .into_iter()
            .map(StoredField::into_domain)
            .collect::<Result<Vec<_>, _>>()?;
        let mut templates = self
            .templates
            .into_iter()
            .map(StoredTemplate::into_domain)
            .collect::<Result<Vec<_>, _>>()?;
        fill_default_templates(&mut templates);

        Ok(Config {
            version: self.version,
            output: self.output,
            options: self.options.into_domain()?,
            temp_fields,
            custom_temp_fields,
            templates,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "the fields mirror the established persisted option flags"
)]
struct StoredOptions {
    iot: bool,
    landscape: bool,
    #[serde(rename = "solid_bg")]
    solid_background: bool,
    solid_color: String,
    origin_wh_output: bool,
    radius: f64,
    #[serde(rename = "radius_show")]
    radius_visible: bool,
    shadow: f64,
    #[serde(rename = "shadow_show")]
    shadow_visible: bool,
    #[serde(rename = "bg_rate_show")]
    background_ratio_visible: bool,
    #[serde(rename = "bg_rate")]
    background_ratio: StoredRatio,
    font: String,
    #[serde(rename = "main_img_w_rate")]
    main_image_width: u8,
    text_margin: f64,
    quality: u8,
    mini_top_bottom_margin: f64,
    #[serde(rename = "bg_blur")]
    background_blur: u8,
    #[serde(rename = "preview_show")]
    preview_visible: bool,
}

impl Default for StoredOptions {
    fn default() -> Self {
        Self::from_domain(&RenderOptions::default())
    }
}

impl StoredOptions {
    fn from_domain(options: &RenderOptions) -> Self {
        Self {
            iot: options.iot,
            landscape: options.landscape,
            solid_background: options.solid_background,
            solid_color: options.solid_color.as_str().to_owned(),
            origin_wh_output: options.origin_wh_output,
            radius: options.radius.get(),
            radius_visible: options.radius_visible,
            shadow: options.shadow.get(),
            shadow_visible: options.shadow_visible,
            background_ratio_visible: options.background_ratio_visible,
            background_ratio: StoredRatio {
                width: options.background_ratio.width(),
                height: options.background_ratio.height(),
            },
            font: options.font.as_str().to_owned(),
            main_image_width: options.main_image_width.get(),
            text_margin: options.text_margin.get(),
            quality: options.quality.get(),
            mini_top_bottom_margin: options.mini_top_bottom_margin.get(),
            background_blur: options.background_blur.get(),
            preview_visible: options.preview_visible,
        }
    }

    fn into_domain(self) -> Result<RenderOptions, ApplicationError> {
        Ok(RenderOptions {
            iot: self.iot,
            landscape: self.landscape,
            solid_background: self.solid_background,
            solid_color: SolidColor::try_from(self.solid_color.as_str()).map_err(invalid_domain)?,
            origin_wh_output: self.origin_wh_output,
            radius: Radius::try_from(self.radius).map_err(invalid_domain)?,
            radius_visible: self.radius_visible,
            shadow: Shadow::try_from(self.shadow).map_err(invalid_domain)?,
            shadow_visible: self.shadow_visible,
            background_ratio_visible: self.background_ratio_visible,
            background_ratio: BackgroundRatio::try_from((
                self.background_ratio.width,
                self.background_ratio.height,
            ))
            .map_err(invalid_domain)?,
            font: FontFamily::try_from(self.font.as_str()).map_err(invalid_domain)?,
            main_image_width: MainImageWidth::try_from(self.main_image_width)
                .map_err(invalid_domain)?,
            text_margin: TextMargin::try_from(self.text_margin).map_err(invalid_domain)?,
            quality: Quality::try_from(self.quality).map_err(invalid_domain)?,
            mini_top_bottom_margin: MiniTopBottomMargin::try_from(self.mini_top_bottom_margin)
                .map_err(invalid_domain)?,
            background_blur: BackgroundBlur::try_from(self.background_blur)
                .map_err(invalid_domain)?,
            preview_visible: self.preview_visible,
        })
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct StoredRatio {
    #[serde(rename = "w")]
    width: f64,
    #[serde(rename = "h")]
    height: f64,
}

impl Default for StoredRatio {
    fn default() -> Self {
        Self {
            width: 0.0,
            height: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct StoredField {
    #[serde(rename = "use")]
    use_custom_value: bool,
    #[serde(rename = "forceUse")]
    force_custom_value: bool,
    #[serde(rename = "show")]
    visible: bool,
    key: String,
    name: String,
    value: String,
    #[serde(rename = "bImg")]
    dark_image: String,
    #[serde(rename = "wImg")]
    light_image: String,
    #[serde(rename = "type")]
    content_type: StoredFieldType,
    font: StoredFieldFont,
}

impl Default for StoredField {
    fn default() -> Self {
        Self {
            use_custom_value: false,
            force_custom_value: false,
            visible: true,
            key: String::new(),
            name: String::new(),
            value: String::new(),
            dark_image: String::new(),
            light_image: String::new(),
            content_type: StoredFieldType::Text,
            font: StoredFieldFont::default(),
        }
    }
}

impl StoredField {
    fn from_domain(field: &TemplateField) -> Self {
        Self {
            use_custom_value: field.uses_custom_value(),
            force_custom_value: field.forces_custom_value(),
            visible: field.visible(),
            key: field.key().as_str().to_owned(),
            name: field.name().to_owned(),
            value: field.custom_value().to_owned(),
            dark_image: field
                .dark_image()
                .map_or_else(String::new, |id| id.as_str().to_owned()),
            light_image: field
                .light_image()
                .map_or_else(String::new, |id| id.as_str().to_owned()),
            content_type: match field.content_kind() {
                FieldContentKind::Text => StoredFieldType::Text,
                FieldContentKind::Image => StoredFieldType::Image,
            },
            font: StoredFieldFont::from_domain(field.font_override()),
        }
    }

    fn into_domain(self) -> Result<TemplateField, ApplicationError> {
        let mut field = TemplateField::custom(&self.key, self.name).map_err(invalid_domain)?;
        field.set_visible(self.visible);
        field.set_custom_text_state(self.value, self.use_custom_value, self.force_custom_value);
        if self.content_type == StoredFieldType::Image {
            field.set_image_variants(
                optional_resource_id(&self.dark_image)?,
                optional_resource_id(&self.light_image)?,
            );
        }
        field.set_font_override(self.font.into_domain()?);
        Ok(field)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum StoredFieldType {
    #[default]
    Text,
    #[serde(rename = "img")]
    Image,
}

#[derive(Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct StoredFieldFont {
    #[serde(rename = "use")]
    enabled: bool,
    bold: bool,
    italic: bool,
    size: f64,
    #[serde(rename = "font")]
    family: String,
    #[serde(rename = "caseType")]
    case_conversion: StoredCaseConversion,
    color: String,
}

impl Default for StoredFieldFont {
    fn default() -> Self {
        Self {
            enabled: false,
            bold: false,
            italic: false,
            size: 0.0,
            family: String::new(),
            case_conversion: StoredCaseConversion::Default,
            color: String::new(),
        }
    }
}

impl StoredFieldFont {
    fn from_domain(font: Option<&FontSpec>) -> Self {
        font.map_or_else(Self::default, |font| Self {
            enabled: true,
            bold: font.bold(),
            italic: font.italic(),
            size: font.size(),
            family: font.family().to_owned(),
            case_conversion: StoredCaseConversion::from_domain(font.case_conversion()),
            color: font.color().to_owned(),
        })
    }

    fn into_domain(self) -> Result<Option<FontSpec>, ApplicationError> {
        if !self.enabled {
            return Ok(None);
        }
        Ok(Some(
            FontSpec::new(
                self.family,
                self.size,
                self.bold,
                self.italic,
                self.case_conversion.into_domain(),
                self.color,
            )
            .map_err(invalid_domain)?,
        ))
    }
}

#[derive(Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct StoredTemplate {
    key: String,
    name: String,
    #[serde(rename = "temp")]
    format: String,
    #[serde(rename = "use")]
    enabled: bool,
    #[serde(rename = "type")]
    kind: StoredTemplateKind,
    height: Option<f64>,
    font: StoredTemplateFont,
    #[serde(rename = "verticalAlign")]
    vertical_align: StoredVerticalAlign,
}

impl Default for StoredTemplate {
    fn default() -> Self {
        Self {
            key: String::new(),
            name: String::new(),
            format: String::new(),
            enabled: false,
            kind: StoredTemplateKind::Custom,
            height: None,
            font: StoredTemplateFont::default(),
            vertical_align: StoredVerticalAlign::Baseline,
        }
    }
}

impl StoredTemplate {
    fn from_domain(template: &Template) -> Self {
        Self {
            key: template.key().to_owned(),
            name: template.name().to_owned(),
            format: template.format().to_owned(),
            enabled: template.enabled(),
            kind: match template.kind() {
                TemplateKind::System => StoredTemplateKind::System,
                TemplateKind::Custom => StoredTemplateKind::Custom,
            },
            height: template.height(),
            font: StoredTemplateFont::from_domain(template.font()),
            vertical_align: StoredVerticalAlign::from_domain(template.vertical_align()),
        }
    }

    fn into_domain(self) -> Result<Template, ApplicationError> {
        let font = self.font.into_domain()?;
        let mut template = match self.kind {
            StoredTemplateKind::System => default_templates()
                .into_iter()
                .find(|template| template.key() == self.key)
                .ok_or_else(ApplicationError::config_invalid)?,
            StoredTemplateKind::Custom => Template::custom(
                &self.key,
                self.name.clone(),
                self.format.clone(),
                self.enabled,
                font.clone(),
            )
            .map_err(invalid_domain)?,
        };
        template.set_name(self.name);
        template.set_format(self.format);
        template.set_enabled(self.enabled);
        template.set_height(self.height).map_err(invalid_domain)?;
        template.set_font(font);
        template.set_vertical_align(self.vertical_align.into_domain());
        Ok(template)
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum StoredTemplateKind {
    System,
    #[default]
    Custom,
}

#[derive(Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct StoredTemplateFont {
    size: f64,
    #[serde(rename = "font")]
    family: String,
    bold: bool,
    italic: bool,
    color: String,
    #[serde(rename = "caseType")]
    case_conversion: StoredCaseConversion,
}

impl Default for StoredTemplateFont {
    fn default() -> Self {
        Self {
            size: 2.2,
            family: String::new(),
            bold: false,
            italic: false,
            color: String::new(),
            case_conversion: StoredCaseConversion::Default,
        }
    }
}

impl StoredTemplateFont {
    fn from_domain(font: &FontSpec) -> Self {
        Self {
            size: font.size(),
            family: font.family().to_owned(),
            bold: font.bold(),
            italic: font.italic(),
            color: font.color().to_owned(),
            case_conversion: StoredCaseConversion::from_domain(font.case_conversion()),
        }
    }

    fn into_domain(self) -> Result<FontSpec, ApplicationError> {
        FontSpec::new(
            self.family,
            self.size,
            self.bold,
            self.italic,
            self.case_conversion.into_domain(),
            self.color,
        )
        .map_err(invalid_domain)
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
enum StoredCaseConversion {
    #[default]
    #[serde(rename = "default")]
    Default,
    #[serde(rename = "lowcase")]
    Lowercase,
    #[serde(rename = "upcase")]
    Uppercase,
}

impl StoredCaseConversion {
    const fn from_domain(value: CaseConversion) -> Self {
        match value {
            CaseConversion::Default => Self::Default,
            CaseConversion::Lowercase => Self::Lowercase,
            CaseConversion::Uppercase => Self::Uppercase,
        }
    }

    const fn into_domain(self) -> CaseConversion {
        match self {
            Self::Default => CaseConversion::Default,
            Self::Lowercase => CaseConversion::Lowercase,
            Self::Uppercase => CaseConversion::Uppercase,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum StoredVerticalAlign {
    Center,
    #[default]
    Baseline,
}

impl StoredVerticalAlign {
    const fn from_domain(value: VerticalAlign) -> Self {
        match value {
            VerticalAlign::Center => Self::Center,
            VerticalAlign::Baseline => Self::Baseline,
        }
    }

    const fn into_domain(self) -> VerticalAlign {
        match self {
            Self::Center => VerticalAlign::Center,
            Self::Baseline => VerticalAlign::Baseline,
        }
    }
}

fn optional_resource_id(value: &str) -> Result<Option<ResourceId>, ApplicationError> {
    if value.trim().is_empty() {
        Ok(None)
    } else {
        ResourceId::try_from(value)
            .map(Some)
            .map_err(invalid_domain)
    }
}

fn invalid_domain(_: yiyin_domain::DomainError) -> ApplicationError {
    ApplicationError::config_invalid()
}
