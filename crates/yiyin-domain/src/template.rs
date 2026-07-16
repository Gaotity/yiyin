use crate::{BuiltInField, DomainError, Metadata, ResourceId};

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FieldKey(String);

impl FieldKey {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn built_in(field: BuiltInField) -> Self {
        Self(field.key().to_owned())
    }
}

impl TryFrom<&str> for FieldKey {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            Err(DomainError::Empty("field_key"))
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CaseConversion {
    #[default]
    Default,
    Lowercase,
    Uppercase,
}

#[derive(Clone, Debug, PartialEq)]
pub struct FontSpec {
    family: String,
    size: f64,
    bold: bool,
    italic: bool,
    case_conversion: CaseConversion,
    color: String,
}

impl FontSpec {
    /// Creates validated template typography.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::OutOfRange`] when `size` is not finite and positive.
    pub fn new(
        family: impl Into<String>,
        size: f64,
        bold: bool,
        italic: bool,
        case_conversion: CaseConversion,
        color: impl Into<String>,
    ) -> Result<Self, DomainError> {
        if !size.is_finite() || size <= 0.0 {
            return Err(DomainError::OutOfRange("font_size"));
        }

        Ok(Self {
            family: family.into(),
            size,
            bold,
            italic,
            case_conversion,
            color: color.into(),
        })
    }

    fn established(size: f64, bold: bool) -> Self {
        Self {
            family: String::new(),
            size,
            bold,
            italic: false,
            case_conversion: CaseConversion::Default,
            color: String::new(),
        }
    }

    #[must_use]
    pub fn family(&self) -> &str {
        &self.family
    }

    #[must_use]
    pub const fn size(&self) -> f64 {
        self.size
    }

    #[must_use]
    pub const fn bold(&self) -> bool {
        self.bold
    }

    #[must_use]
    pub const fn italic(&self) -> bool {
        self.italic
    }

    #[must_use]
    pub const fn case_conversion(&self) -> CaseConversion {
        self.case_conversion
    }

    #[must_use]
    pub fn color(&self) -> &str {
        &self.color
    }

    fn convert_case(&self, value: &str) -> String {
        match self.case_conversion {
            CaseConversion::Default => value.to_owned(),
            CaseConversion::Lowercase => value.to_lowercase(),
            CaseConversion::Uppercase => value.to_uppercase(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BackgroundKind {
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TemplateKind {
    System,
    Custom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerticalAlign {
    Center,
    Baseline,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FieldContentKind {
    Text,
    Image,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TemplateField {
    key: FieldKey,
    name: String,
    visible: bool,
    use_custom_value: bool,
    force_custom_value: bool,
    custom_value: String,
    content_kind: FieldContentKind,
    dark_image: Option<ResourceId>,
    light_image: Option<ResourceId>,
    font_override: Option<FontSpec>,
}

impl TemplateField {
    fn built_in(field: BuiltInField) -> Self {
        Self {
            key: FieldKey::built_in(field),
            name: field.name().to_owned(),
            visible: true,
            use_custom_value: false,
            force_custom_value: false,
            custom_value: String::new(),
            content_kind: FieldContentKind::Text,
            dark_image: None,
            light_image: None,
            font_override: None,
        }
    }

    /// Creates a user-defined template field with a stable key.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Empty`] when `key` is blank.
    pub fn custom(key: &str, name: impl Into<String>) -> Result<Self, DomainError> {
        Ok(Self {
            key: FieldKey::try_from(key)?,
            name: name.into(),
            visible: true,
            use_custom_value: false,
            force_custom_value: false,
            custom_value: String::new(),
            content_kind: FieldContentKind::Text,
            dark_image: None,
            light_image: None,
            font_override: None,
        })
    }

    #[must_use]
    pub const fn key(&self) -> &FieldKey {
        &self.key
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub const fn visible(&self) -> bool {
        self.visible
    }

    #[must_use]
    pub const fn uses_custom_value(&self) -> bool {
        self.use_custom_value
    }

    #[must_use]
    pub const fn forces_custom_value(&self) -> bool {
        self.force_custom_value
    }

    #[must_use]
    pub fn custom_value(&self) -> &str {
        &self.custom_value
    }

    #[must_use]
    pub const fn content_kind(&self) -> FieldContentKind {
        self.content_kind
    }

    #[must_use]
    pub const fn dark_image(&self) -> Option<&ResourceId> {
        self.dark_image.as_ref()
    }

    #[must_use]
    pub const fn light_image(&self) -> Option<&ResourceId> {
        self.light_image.as_ref()
    }

    #[must_use]
    pub const fn font_override(&self) -> Option<&FontSpec> {
        self.font_override.as_ref()
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn set_custom_text(&mut self, value: impl Into<String>, force: bool) {
        self.set_custom_text_state(value, true, force);
    }

    pub fn set_custom_text_state(&mut self, value: impl Into<String>, enabled: bool, force: bool) {
        self.use_custom_value = enabled;
        self.force_custom_value = force;
        self.custom_value = value.into();
        self.content_kind = FieldContentKind::Text;
    }

    pub fn set_image_variants(
        &mut self,
        dark_image: Option<ResourceId>,
        light_image: Option<ResourceId>,
    ) {
        self.content_kind = FieldContentKind::Image;
        self.dark_image = dark_image;
        self.light_image = light_image;
    }

    pub fn set_font_override(&mut self, font: Option<FontSpec>) {
        self.font_override = font;
    }

    fn resolve(
        &self,
        metadata: &Metadata,
        background: BackgroundKind,
        template_font: &FontSpec,
    ) -> Option<RowSlot> {
        if !self.visible {
            return None;
        }

        let font = self.font_override.as_ref().unwrap_or(template_font).clone();
        if self.content_kind == FieldContentKind::Image {
            let resource = match background {
                BackgroundKind::Light => self.dark_image.as_ref(),
                BackgroundKind::Dark => self.light_image.as_ref(),
            }?;
            return Some(RowSlot::Image {
                resource: resource.clone(),
                font,
            });
        }

        let metadata_value = BuiltInField::from_key(self.key.as_str())
            .and_then(|field| metadata.value(field))
            .unwrap_or_default();
        let value = if self.use_custom_value
            && (self.force_custom_value || metadata_value.trim().is_empty())
        {
            self.custom_value.as_str()
        } else {
            metadata_value
        };
        let value = value.trim();
        if value.is_empty() {
            None
        } else {
            Some(RowSlot::Text {
                value: font.convert_case(value),
                font,
            })
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    key: String,
    name: String,
    format: String,
    enabled: bool,
    kind: TemplateKind,
    height: Option<f64>,
    font: FontSpec,
    vertical_align: VerticalAlign,
}

impl Template {
    /// Creates a user-defined text template.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Empty`] when `key` is blank.
    pub fn custom(
        key: &str,
        name: impl Into<String>,
        format: impl Into<String>,
        enabled: bool,
        font: FontSpec,
    ) -> Result<Self, DomainError> {
        if key.trim().is_empty() {
            return Err(DomainError::Empty("template_key"));
        }

        Ok(Self {
            key: key.to_owned(),
            name: name.into(),
            format: format.into(),
            enabled,
            kind: TemplateKind::Custom,
            height: None,
            font,
            vertical_align: VerticalAlign::Baseline,
        })
    }

    fn established(key: &str, name: &str, format: &str, enabled: bool, font: FontSpec) -> Self {
        Self {
            key: key.to_owned(),
            name: name.to_owned(),
            format: format.to_owned(),
            enabled,
            kind: TemplateKind::System,
            height: None,
            font,
            vertical_align: VerticalAlign::Baseline,
        }
    }

    #[must_use]
    pub fn key(&self) -> &str {
        &self.key
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn format(&self) -> &str {
        &self.format
    }

    #[must_use]
    pub const fn enabled(&self) -> bool {
        self.enabled
    }

    #[must_use]
    pub const fn kind(&self) -> TemplateKind {
        self.kind
    }

    #[must_use]
    pub const fn height(&self) -> Option<f64> {
        self.height
    }

    #[must_use]
    pub const fn font(&self) -> &FontSpec {
        &self.font
    }

    #[must_use]
    pub const fn vertical_align(&self) -> VerticalAlign {
        self.vertical_align
    }

    pub fn set_name(&mut self, name: impl Into<String>) {
        self.name = name.into();
    }

    pub fn set_format(&mut self, format: impl Into<String>) {
        self.format = format.into();
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Sets an optional positive template height.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::OutOfRange`] for non-positive or non-finite values.
    pub fn set_height(&mut self, height: Option<f64>) -> Result<(), DomainError> {
        if height.is_some_and(|value| !value.is_finite() || value <= 0.0) {
            return Err(DomainError::OutOfRange("template_height"));
        }
        self.height = height;
        Ok(())
    }

    pub fn set_font(&mut self, font: FontSpec) {
        self.font = font;
    }

    pub fn set_vertical_align(&mut self, vertical_align: VerticalAlign) {
        self.vertical_align = vertical_align;
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct FieldValues {
    metadata: Metadata,
    fields: Vec<TemplateField>,
}

impl FieldValues {
    #[must_use]
    pub const fn new(metadata: Metadata, fields: Vec<TemplateField>) -> Self {
        Self { metadata, fields }
    }

    #[must_use]
    pub fn field_mut(&mut self, key: &str) -> Option<&mut TemplateField> {
        self.fields
            .iter_mut()
            .find(|field| field.key.as_str() == key)
    }

    fn field(&self, key: &str) -> Option<&TemplateField> {
        self.fields.iter().find(|field| field.key.as_str() == key)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum RowSlot {
    Text {
        value: String,
        font: FontSpec,
    },
    Image {
        resource: ResourceId,
        font: FontSpec,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextRowPlan {
    template_key: String,
    slots: Vec<RowSlot>,
    height: Option<f64>,
    vertical_align: VerticalAlign,
    font: FontSpec,
}

impl TextRowPlan {
    #[must_use]
    pub fn template_key(&self) -> &str {
        &self.template_key
    }

    #[must_use]
    pub fn slots(&self) -> &[RowSlot] {
        &self.slots
    }

    #[must_use]
    pub fn plain_text(&self) -> String {
        self.slots
            .iter()
            .filter_map(|slot| match slot {
                RowSlot::Text { value, .. } => Some(value.as_str()),
                RowSlot::Image { .. } => None,
            })
            .collect::<String>()
            .trim()
            .to_owned()
    }

    #[must_use]
    pub const fn height(&self) -> Option<f64> {
        self.height
    }

    #[must_use]
    pub const fn vertical_align(&self) -> VerticalAlign {
        self.vertical_align
    }

    #[must_use]
    pub const fn font(&self) -> &FontSpec {
        &self.font
    }
}

#[must_use]
pub fn default_template_fields() -> Vec<TemplateField> {
    BuiltInField::ALL
        .into_iter()
        .map(TemplateField::built_in)
        .collect()
}

#[must_use]
pub fn default_templates() -> Vec<Template> {
    vec![
        Template::established(
            "make-model",
            "Logo型号模版",
            "{Make} {Model}",
            true,
            FontSpec::established(3.0, true),
        ),
        Template::established(
            "exif-params",
            "参数模版 - 等效焦距",
            "{FocalLengthIn35mmFormat}mm f/{FNumber} {ExposureTime}s ISO{ISO}",
            true,
            FontSpec::established(2.2, true),
        ),
        Template::established(
            "exif-params-1",
            "参数模版 - 原始焦距",
            "{FocalLength}mm f/{FNumber} {ExposureTime}s ISO{ISO}",
            false,
            FontSpec::established(2.2, true),
        ),
    ]
}

#[must_use]
pub fn plan_rows(
    templates: &[Template],
    values: &FieldValues,
    background: BackgroundKind,
) -> Vec<TextRowPlan> {
    templates
        .iter()
        .filter(|template| template.enabled)
        .filter_map(|template| plan_row(template, values, background))
        .collect()
}

fn plan_row(
    template: &Template,
    values: &FieldValues,
    background: BackgroundKind,
) -> Option<TextRowPlan> {
    let mut slots = Vec::new();
    let mut remaining = template.format.trim();
    let mut placeholder_count = 0;
    let mut resolved_count = 0;

    while let Some(open) = remaining.find('{') {
        push_text(&mut slots, &remaining[..open], &template.font);
        let after_open = &remaining[open + 1..];
        let Some(close) = after_open.find('}') else {
            push_text(&mut slots, &remaining[open..], &template.font);
            remaining = "";
            break;
        };

        placeholder_count += 1;
        let key = &after_open[..close];
        if let Some(slot) = values
            .field(key)
            .and_then(|field| field.resolve(&values.metadata, background, &template.font))
        {
            slots.push(slot);
            resolved_count += 1;
        }
        remaining = &after_open[close + 1..];
    }
    push_text(&mut slots, remaining, &template.font);

    trim_edge_text(&mut slots);
    if (placeholder_count > 0 && resolved_count == 0) || slots.is_empty() {
        return None;
    }

    Some(TextRowPlan {
        template_key: template.key.clone(),
        slots,
        height: template.height,
        vertical_align: template.vertical_align,
        font: template.font.clone(),
    })
}

fn push_text(slots: &mut Vec<RowSlot>, value: &str, font: &FontSpec) {
    if !value.is_empty() {
        slots.push(RowSlot::Text {
            value: font.convert_case(value),
            font: font.clone(),
        });
    }
}

fn trim_edge_text(slots: &mut Vec<RowSlot>) {
    if let Some(RowSlot::Text { value, .. }) = slots.first_mut() {
        *value = value.trim_start().to_owned();
    }
    if let Some(RowSlot::Text { value, .. }) = slots.last_mut() {
        *value = value.trim_end().to_owned();
    }
    slots.retain(|slot| !matches!(slot, RowSlot::Text { value, .. } if value.is_empty()));
}

#[cfg(test)]
mod tests {
    use super::{
        BackgroundKind, BuiltInField, CaseConversion, FieldValues, FontSpec, RowSlot, Template,
        default_template_fields, default_templates, plan_rows,
    };
    use crate::{Config, DomainError, Metadata, ResourceId};

    fn values_with_model(model: &str) -> FieldValues {
        let mut metadata = Metadata::default();
        metadata.set(BuiltInField::Model, model);
        FieldValues::new(metadata, default_template_fields())
    }

    #[test]
    fn built_in_fields_preserve_all_established_keys_and_names() {
        let fields = default_template_fields();
        let actual = fields
            .iter()
            .map(|field| (field.key().as_str(), field.name()))
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                ("PersonalSign", "个性签名"),
                ("Make", "Logo"),
                ("Model", "型号"),
                ("LensMake", "镜头Logo"),
                ("LensModel", "镜头型号"),
                ("ExposureTime", "快门"),
                ("FNumber", "光圈"),
                ("ISO", "ISO"),
                ("FocalLength", "焦距"),
                ("FocalLengthIn35mmFormat", "等效焦距"),
                ("ExposureProgram", "档位"),
                ("DateTimeOriginal", "拍摄日期"),
                ("ExposureCompensation", "曝光补偿"),
                ("MeteringMode", "测光模式"),
                ("WhiteBalance", "白平衡"),
            ]
        );
    }

    #[test]
    fn default_templates_match_v1_6() {
        let templates = default_templates();
        let actual = templates
            .iter()
            .map(|template| {
                (
                    template.key(),
                    template.name(),
                    template.format(),
                    template.enabled(),
                    template.font().size(),
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            actual,
            vec![
                ("make-model", "Logo型号模版", "{Make} {Model}", true, 3.0),
                (
                    "exif-params",
                    "参数模版 - 等效焦距",
                    "{FocalLengthIn35mmFormat}mm f/{FNumber} {ExposureTime}s ISO{ISO}",
                    true,
                    2.2,
                ),
                (
                    "exif-params-1",
                    "参数模版 - 原始焦距",
                    "{FocalLength}mm f/{FNumber} {ExposureTime}s ISO{ISO}",
                    false,
                    2.2,
                ),
            ]
        );
    }

    #[test]
    fn hidden_or_empty_fields_collapse_without_literal_placeholders() {
        let mut metadata = Metadata::default();
        metadata.set(BuiltInField::Make, "Nikon");
        metadata.set(BuiltInField::Model, "z8");
        let mut fields = default_template_fields();
        fields
            .iter_mut()
            .find(|field| field.key().as_str() == "Make")
            .expect("built-in Make field")
            .set_visible(false);

        let rows = plan_rows(
            &default_templates(),
            &FieldValues::new(metadata, fields),
            BackgroundKind::Dark,
        );

        assert_eq!(rows[0].plain_text(), "z8");
        assert!(!rows[0].plain_text().contains('{'));
    }

    #[test]
    fn forced_custom_value_overrides_metadata_but_optional_value_does_not() {
        let mut metadata = Metadata::default();
        metadata.set(BuiltInField::Model, "z8");
        let mut fields = default_template_fields();
        let model_index = fields
            .iter()
            .position(|field| field.key().as_str() == "Model")
            .expect("built-in Model field");
        fields[model_index].set_custom_text("custom", false);

        let optional = plan_rows(
            &default_templates()[..1],
            &FieldValues::new(metadata.clone(), fields.clone()),
            BackgroundKind::Dark,
        );
        assert_eq!(optional[0].plain_text(), "z8");

        fields[model_index].set_custom_text("custom", true);
        let forced = plan_rows(
            &default_templates()[..1],
            &FieldValues::new(metadata, fields),
            BackgroundKind::Dark,
        );
        assert_eq!(forced[0].plain_text(), "custom");
    }

    #[test]
    fn absent_metadata_omits_empty_rows() {
        let rows = plan_rows(
            &default_templates(),
            &FieldValues::new(Metadata::default(), default_template_fields()),
            BackgroundKind::Dark,
        );

        assert!(rows.is_empty());
    }

    #[test]
    fn image_fields_choose_the_variant_for_the_background() {
        let mut fields = default_template_fields();
        fields
            .iter_mut()
            .find(|field| field.key().as_str() == "Make")
            .expect("built-in Make field")
            .set_image_variants(
                Some(ResourceId::try_from("dark-logo").expect("valid id")),
                Some(ResourceId::try_from("light-logo").expect("valid id")),
            );
        let mut metadata = Metadata::default();
        metadata.set(BuiltInField::Make, "Nikon");
        let values = FieldValues::new(metadata, fields);

        let light_rows = plan_rows(&default_templates()[..1], &values, BackgroundKind::Light);
        let dark_rows = plan_rows(&default_templates()[..1], &values, BackgroundKind::Dark);

        assert!(matches!(
            &light_rows[0].slots()[0],
            RowSlot::Image { resource, .. } if resource.as_str() == "dark-logo"
        ));
        assert!(matches!(
            &dark_rows[0].slots()[0],
            RowSlot::Image { resource, .. } if resource.as_str() == "light-logo"
        ));
    }

    #[test]
    fn field_font_case_conversion_overrides_template_case_conversion() {
        let template_font = FontSpec::new("", 2.2, false, false, CaseConversion::Uppercase, "")
            .expect("valid template font");
        let template = Template::custom("case", "case", "shot {Model}", true, template_font)
            .expect("valid template");
        let field_font = FontSpec::new("", 2.2, false, false, CaseConversion::Lowercase, "")
            .expect("valid field font");
        let mut values = values_with_model("Z8");
        values
            .field_mut("Model")
            .expect("built-in Model field")
            .set_font_override(Some(field_font));

        let rows = plan_rows(&[template], &values, BackgroundKind::Dark);

        assert_eq!(rows[0].plain_text(), "SHOT z8");
    }

    #[test]
    fn system_templates_cannot_be_deleted() {
        let mut config = Config::default();

        assert_eq!(
            config.remove_template("make-model"),
            Err(DomainError::ProtectedSystemTemplate)
        );
        assert_eq!(config.templates.len(), 3);
    }
}
