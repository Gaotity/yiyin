use serde::{Deserialize, Serialize};
use yiyin_application::{ApplicationError, BootstrapSnapshot};
use yiyin_domain::{
    BackgroundBlur, BackgroundRatio, CaseConversion, Config, FieldContentKind, FontFamily,
    FontSpec, MainImageWidth, MiniTopBottomMargin, Quality, Radius, RenderOptions, ResourceId,
    Shadow, SolidColor, Template, TemplateField, TemplateKind, TextMargin, VerticalAlign,
    default_template_fields, default_templates,
};

use super::{ResourceDescriptorDto, TaskDescriptorDto};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub enum CaseConversionDto {
    Default,
    Lowercase,
    Uppercase,
}

impl From<CaseConversion> for CaseConversionDto {
    fn from(value: CaseConversion) -> Self {
        match value {
            CaseConversion::Default => Self::Default,
            CaseConversion::Lowercase => Self::Lowercase,
            CaseConversion::Uppercase => Self::Uppercase,
        }
    }
}

impl From<CaseConversionDto> for CaseConversion {
    fn from(value: CaseConversionDto) -> Self {
        match value {
            CaseConversionDto::Default => Self::Default,
            CaseConversionDto::Lowercase => Self::Lowercase,
            CaseConversionDto::Uppercase => Self::Uppercase,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct FontSpecDto {
    pub family: String,
    pub size: f64,
    pub bold: bool,
    pub italic: bool,
    pub case_conversion: CaseConversionDto,
    pub color: String,
}

impl From<&FontSpec> for FontSpecDto {
    fn from(value: &FontSpec) -> Self {
        Self {
            family: value.family().to_owned(),
            size: value.size(),
            bold: value.bold(),
            italic: value.italic(),
            case_conversion: value.case_conversion().into(),
            color: value.color().to_owned(),
        }
    }
}

impl TryFrom<FontSpecDto> for FontSpec {
    type Error = ApplicationError;

    fn try_from(value: FontSpecDto) -> Result<Self, Self::Error> {
        FontSpec::new(
            value.family,
            value.size,
            value.bold,
            value.italic,
            value.case_conversion.into(),
            value.color,
        )
        .map_err(|_| ApplicationError::config_invalid())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub enum FieldContentKindDto {
    Text,
    Image,
}

impl From<FieldContentKind> for FieldContentKindDto {
    fn from(value: FieldContentKind) -> Self {
        match value {
            FieldContentKind::Text => Self::Text,
            FieldContentKind::Image => Self::Image,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct TemplateFieldDto {
    pub key: String,
    pub name: String,
    pub visible: bool,
    pub use_custom_value: bool,
    pub force_custom_value: bool,
    pub custom_value: String,
    pub content_kind: FieldContentKindDto,
    pub dark_image_id: Option<String>,
    pub light_image_id: Option<String>,
    pub font_override: Option<FontSpecDto>,
}

impl From<&TemplateField> for TemplateFieldDto {
    fn from(value: &TemplateField) -> Self {
        Self {
            key: value.key().as_str().to_owned(),
            name: value.name().to_owned(),
            visible: value.visible(),
            use_custom_value: value.uses_custom_value(),
            force_custom_value: value.forces_custom_value(),
            custom_value: value.custom_value().to_owned(),
            content_kind: value.content_kind().into(),
            dark_image_id: value.dark_image().map(|id| id.as_str().to_owned()),
            light_image_id: value.light_image().map(|id| id.as_str().to_owned()),
            font_override: value.font_override().map(Into::into),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub enum TemplateKindDto {
    System,
    Custom,
}

impl From<TemplateKind> for TemplateKindDto {
    fn from(value: TemplateKind) -> Self {
        match value {
            TemplateKind::System => Self::System,
            TemplateKind::Custom => Self::Custom,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub enum VerticalAlignDto {
    Center,
    Baseline,
}

impl From<VerticalAlign> for VerticalAlignDto {
    fn from(value: VerticalAlign) -> Self {
        match value {
            VerticalAlign::Center => Self::Center,
            VerticalAlign::Baseline => Self::Baseline,
        }
    }
}

impl From<VerticalAlignDto> for VerticalAlign {
    fn from(value: VerticalAlignDto) -> Self {
        match value {
            VerticalAlignDto::Center => Self::Center,
            VerticalAlignDto::Baseline => Self::Baseline,
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct TemplateDto {
    pub key: String,
    pub name: String,
    pub format: String,
    pub enabled: bool,
    pub kind: TemplateKindDto,
    pub height: Option<f64>,
    pub font: FontSpecDto,
    pub vertical_align: VerticalAlignDto,
}

impl From<&Template> for TemplateDto {
    fn from(value: &Template) -> Self {
        Self {
            key: value.key().to_owned(),
            name: value.name().to_owned(),
            format: value.format().to_owned(),
            enabled: value.enabled(),
            kind: value.kind().into(),
            height: value.height(),
            font: value.font().into(),
            vertical_align: value.vertical_align().into(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct BackgroundRatioDto {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
#[allow(
    clippy::struct_excessive_bools,
    reason = "the DTO mirrors product options"
)]
pub struct RenderOptionsDto {
    pub quick_output: bool,
    pub landscape: bool,
    pub solid_background: bool,
    pub solid_color: String,
    pub radius: f64,
    pub radius_visible: bool,
    pub shadow: f64,
    pub shadow_visible: bool,
    pub background_ratio_visible: bool,
    pub background_ratio: BackgroundRatioDto,
    pub font: String,
    pub main_image_width: u8,
    pub text_margin: f64,
    pub quality: u8,
    pub mini_top_bottom_margin: f64,
    pub background_blur: u8,
    pub preview_visible: bool,
}

impl From<&RenderOptions> for RenderOptionsDto {
    fn from(value: &RenderOptions) -> Self {
        Self {
            quick_output: value.iot,
            landscape: value.landscape,
            solid_background: value.solid_background,
            solid_color: value.solid_color.as_str().to_owned(),
            radius: value.radius.get(),
            radius_visible: value.radius_visible,
            shadow: value.shadow.get(),
            shadow_visible: value.shadow_visible,
            background_ratio_visible: value.background_ratio_visible,
            background_ratio: BackgroundRatioDto {
                width: value.background_ratio.width(),
                height: value.background_ratio.height(),
            },
            font: value.font.as_str().to_owned(),
            main_image_width: value.main_image_width.get(),
            text_margin: value.text_margin.get(),
            quality: value.quality.get(),
            mini_top_bottom_margin: value.mini_top_bottom_margin.get(),
            background_blur: value.background_blur.get(),
            preview_visible: value.preview_visible,
        }
    }
}

impl RenderOptionsDto {
    /// Converts presentation-owned options back to the domain, preserving the
    /// Rust-owned `origin_wh_output` invariant from the current configuration.
    fn apply_to(self, current: &RenderOptions) -> Result<RenderOptions, ApplicationError> {
        let invalid = |_| ApplicationError::config_invalid();
        Ok(RenderOptions {
            iot: self.quick_output,
            landscape: self.landscape,
            solid_background: self.solid_background,
            solid_color: SolidColor::try_from(self.solid_color.as_str()).map_err(invalid)?,
            origin_wh_output: current.origin_wh_output,
            radius: Radius::try_from(self.radius).map_err(invalid)?,
            radius_visible: self.radius_visible,
            shadow: Shadow::try_from(self.shadow).map_err(invalid)?,
            shadow_visible: self.shadow_visible,
            background_ratio_visible: self.background_ratio_visible,
            background_ratio: BackgroundRatio::try_from((
                self.background_ratio.width,
                self.background_ratio.height,
            ))
            .map_err(invalid)?,
            font: FontFamily::try_from(self.font.as_str()).map_err(invalid)?,
            main_image_width: MainImageWidth::try_from(self.main_image_width).map_err(invalid)?,
            text_margin: TextMargin::try_from(self.text_margin).map_err(invalid)?,
            quality: Quality::try_from(self.quality).map_err(invalid)?,
            mini_top_bottom_margin: MiniTopBottomMargin::try_from(self.mini_top_bottom_margin)
                .map_err(invalid)?,
            background_blur: BackgroundBlur::try_from(self.background_blur).map_err(invalid)?,
            preview_visible: self.preview_visible,
        })
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct PublicConfigDto {
    pub version: String,
    pub options: RenderOptionsDto,
    pub template_fields: Vec<TemplateFieldDto>,
    pub custom_template_fields: Vec<TemplateFieldDto>,
    pub templates: Vec<TemplateDto>,
}

impl From<&Config> for PublicConfigDto {
    fn from(value: &Config) -> Self {
        Self {
            version: value.version.clone(),
            options: (&value.options).into(),
            template_fields: value.temp_fields.iter().map(Into::into).collect(),
            custom_template_fields: value.custom_temp_fields.iter().map(Into::into).collect(),
            templates: value.templates.iter().map(Into::into).collect(),
        }
    }
}

impl PublicConfigDto {
    /// Applies a presentation-safe configuration without accepting an output path.
    ///
    /// # Errors
    ///
    /// Returns `CONFIG_INVALID` when values or required system keys are invalid.
    pub fn apply_to(self, current: &Config) -> Result<Config, ApplicationError> {
        if self.version != current.version {
            return Err(ApplicationError::config_invalid());
        }
        Ok(Config {
            version: current.version.clone(),
            output: current.output.clone(),
            options: self.options.apply_to(&current.options)?,
            temp_fields: convert_fields(self.template_fields, true)?,
            custom_temp_fields: convert_fields(self.custom_template_fields, false)?,
            templates: convert_templates(self.templates)?,
        })
    }
}

fn convert_fields(
    values: Vec<TemplateFieldDto>,
    built_in: bool,
) -> Result<Vec<TemplateField>, ApplicationError> {
    let defaults = default_template_fields();
    let fields = values
        .into_iter()
        .map(|value| {
            let mut field = if built_in {
                defaults
                    .iter()
                    .find(|candidate| candidate.key().as_str() == value.key)
                    .cloned()
                    .ok_or_else(ApplicationError::config_invalid)?
            } else {
                TemplateField::custom(&value.key, value.name)
                    .map_err(|_| ApplicationError::config_invalid())?
            };
            field.set_visible(value.visible);
            match value.content_kind {
                FieldContentKindDto::Text => field.set_custom_text_state(
                    value.custom_value,
                    value.use_custom_value,
                    value.force_custom_value,
                ),
                FieldContentKindDto::Image => field.set_image_variants(
                    optional_resource_id(value.dark_image_id)?,
                    optional_resource_id(value.light_image_id)?,
                ),
            }
            field.set_font_override(value.font_override.map(TryInto::try_into).transpose()?);
            Ok(field)
        })
        .collect::<Result<Vec<_>, _>>()?;
    if built_in {
        let expected = defaults
            .iter()
            .map(|field| field.key().as_str())
            .collect::<std::collections::BTreeSet<_>>();
        let actual = fields
            .iter()
            .map(|field| field.key().as_str())
            .collect::<std::collections::BTreeSet<_>>();
        if actual != expected || fields.len() != defaults.len() {
            return Err(ApplicationError::config_invalid());
        }
    }
    Ok(fields)
}

fn optional_resource_id(value: Option<String>) -> Result<Option<ResourceId>, ApplicationError> {
    value
        .map(ResourceId::try_from)
        .transpose()
        .map_err(|_| ApplicationError::config_invalid())
}

fn convert_templates(values: Vec<TemplateDto>) -> Result<Vec<Template>, ApplicationError> {
    let defaults = default_templates();
    let templates = values
        .into_iter()
        .map(|value| {
            let font: FontSpec = value.font.try_into()?;
            let mut template = match value.kind {
                TemplateKindDto::System => defaults
                    .iter()
                    .find(|candidate| candidate.key() == value.key)
                    .cloned()
                    .ok_or_else(ApplicationError::config_invalid)?,
                TemplateKindDto::Custom => Template::custom(
                    &value.key,
                    value.name.clone(),
                    value.format.clone(),
                    value.enabled,
                    font.clone(),
                )
                .map_err(|_| ApplicationError::config_invalid())?,
            };
            template.set_name(value.name);
            template.set_format(value.format);
            template.set_enabled(value.enabled);
            template
                .set_height(value.height)
                .map_err(|_| ApplicationError::config_invalid())?;
            template.set_font(font);
            template.set_vertical_align(value.vertical_align.into());
            Ok(template)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let expected = defaults
        .iter()
        .map(Template::key)
        .collect::<std::collections::BTreeSet<_>>();
    let actual = templates
        .iter()
        .filter(|template| template.kind() == TemplateKind::System)
        .map(Template::key)
        .collect::<std::collections::BTreeSet<_>>();
    if actual != expected {
        return Err(ApplicationError::config_invalid());
    }
    Ok(templates)
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct UpdateConfigRequestDto {
    pub config: PublicConfigDto,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[cfg_attr(test, derive(ts_rs::TS))]
#[serde(rename_all = "camelCase")]
#[cfg_attr(test, ts(rename_all = "camelCase"))]
pub struct BootstrapDto {
    pub config: PublicConfigDto,
    pub resources: Vec<ResourceDescriptorDto>,
    pub tasks: Vec<TaskDescriptorDto>,
    pub warnings: Vec<String>,
}

impl From<&BootstrapSnapshot> for BootstrapDto {
    fn from(value: &BootstrapSnapshot) -> Self {
        Self {
            config: value.config().into(),
            resources: value.resources().iter().map(Into::into).collect(),
            tasks: value.tasks().iter().map(Into::into).collect(),
            warnings: value.warnings().to_vec(),
        }
    }
}
