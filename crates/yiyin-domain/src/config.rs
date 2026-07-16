use crate::{
    DomainError, Template, TemplateField, TemplateKind, default_template_fields, default_templates,
};

macro_rules! bounded_integer {
    ($name:ident, $primitive:ty, $field:literal, $minimum:expr, $maximum:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub struct $name($primitive);

        impl $name {
            #[must_use]
            pub const fn get(self) -> $primitive {
                self.0
            }
        }

        impl TryFrom<$primitive> for $name {
            type Error = DomainError;

            fn try_from(value: $primitive) -> Result<Self, Self::Error> {
                if ($minimum..=$maximum).contains(&value) {
                    Ok(Self(value))
                } else {
                    Err(DomainError::OutOfRange($field))
                }
            }
        }
    };
}

macro_rules! bounded_decimal {
    ($name:ident, $field:literal, $minimum:expr, $maximum:expr, $scale:expr) => {
        #[derive(Clone, Copy, Debug, PartialEq)]
        pub struct $name(f64);

        impl $name {
            #[must_use]
            pub const fn get(self) -> f64 {
                self.0
            }
        }

        impl TryFrom<f64> for $name {
            type Error = DomainError;

            fn try_from(value: f64) -> Result<Self, Self::Error> {
                if !value.is_finite() || !($minimum..=$maximum).contains(&value) {
                    return Err(DomainError::OutOfRange($field));
                }

                let scaled = value * $scale;
                if (scaled - scaled.round()).abs() > 1e-9 {
                    return Err(DomainError::InvalidValue($field));
                }

                Ok(Self(value))
            }
        }
    };
}

bounded_integer!(MainImageWidth, u8, "main_img_w_rate", 1, 100);
bounded_integer!(Quality, u8, "quality", 1, 100);
bounded_integer!(BackgroundBlur, u8, "bg_blur", 0, 100);
bounded_decimal!(Radius, "radius", 0.0, 50.0, 10.0);
bounded_decimal!(Shadow, "shadow", 0.0, 50.0, 10.0);
bounded_decimal!(TextMargin, "text_margin", 0.0, 10_000.0, 100.0);
bounded_decimal!(
    MiniTopBottomMargin,
    "mini_top_bottom_margin",
    0.0,
    100.0,
    100.0
);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BackgroundRatio {
    width: f64,
    height: f64,
}

impl BackgroundRatio {
    #[must_use]
    pub const fn width(self) -> f64 {
        self.width
    }

    #[must_use]
    pub const fn height(self) -> f64 {
        self.height
    }

    #[must_use]
    pub fn is_active(self) -> bool {
        self.width > 0.0 && self.height > 0.0
    }
}

impl TryFrom<(f64, f64)> for BackgroundRatio {
    type Error = DomainError;

    fn try_from((width, height): (f64, f64)) -> Result<Self, Self::Error> {
        if !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
            return Err(DomainError::OutOfRange("bg_rate"));
        }

        Ok(Self { width, height })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SolidColor(String);

impl SolidColor {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for SolidColor {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let digits = value
            .strip_prefix('#')
            .ok_or(DomainError::InvalidValue("solid_color"))?;
        if matches!(digits.len(), 3 | 6) && digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            Ok(Self(value.to_owned()))
        } else {
            Err(DomainError::InvalidValue("solid_color"))
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFamily(String);

impl FontFamily {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for FontFamily {
    type Error = DomainError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        if value.trim().is_empty() {
            Err(DomainError::Empty("font"))
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[allow(
    clippy::struct_excessive_bools,
    reason = "the fields mirror the persisted v1.6 option flags"
)]
pub struct RenderOptions {
    pub iot: bool,
    pub landscape: bool,
    pub solid_background: bool,
    pub solid_color: SolidColor,
    pub origin_wh_output: bool,
    pub radius: Radius,
    pub radius_visible: bool,
    pub shadow: Shadow,
    pub shadow_visible: bool,
    pub background_ratio_visible: bool,
    pub background_ratio: BackgroundRatio,
    pub font: FontFamily,
    pub main_image_width: MainImageWidth,
    pub text_margin: TextMargin,
    pub quality: Quality,
    pub mini_top_bottom_margin: MiniTopBottomMargin,
    pub background_blur: BackgroundBlur,
    pub preview_visible: bool,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            iot: false,
            landscape: false,
            solid_background: false,
            solid_color: SolidColor("#fff".to_owned()),
            origin_wh_output: false,
            radius: Radius(2.1),
            radius_visible: true,
            shadow: Shadow(6.0),
            shadow_visible: true,
            background_ratio_visible: false,
            background_ratio: BackgroundRatio {
                width: 0.0,
                height: 0.0,
            },
            font: FontFamily("PingFang SC".to_owned()),
            main_image_width: MainImageWidth(90),
            text_margin: TextMargin(0.4),
            quality: Quality(100),
            mini_top_bottom_margin: MiniTopBottomMargin(0.0),
            background_blur: BackgroundBlur(100),
            preview_visible: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub version: String,
    pub output: String,
    pub options: RenderOptions,
    pub temp_fields: Vec<TemplateField>,
    pub custom_temp_fields: Vec<TemplateField>,
    pub templates: Vec<Template>,
}

impl Config {
    /// Removes a custom template.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::ProtectedSystemTemplate`] for a system template and
    /// [`DomainError::InvalidValue`] when the key is unknown.
    pub fn remove_template(&mut self, key: &str) -> Result<(), DomainError> {
        let index = self
            .templates
            .iter()
            .position(|template| template.key() == key)
            .ok_or(DomainError::InvalidValue("template"))?;
        if self.templates[index].kind() == TemplateKind::System {
            return Err(DomainError::ProtectedSystemTemplate);
        }
        self.templates.remove(index);
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            version: "1.6.0".to_owned(),
            output: "Pictures/watermark".to_owned(),
            options: RenderOptions::default(),
            temp_fields: default_template_fields(),
            custom_temp_fields: Vec::new(),
            templates: default_templates(),
        }
    }
}

#[cfg(test)]
#[allow(
    clippy::float_cmp,
    reason = "persisted defaults are exact decimal values"
)]
mod tests {
    use super::{
        BackgroundBlur, BackgroundRatio, Config, MainImageWidth, MiniTopBottomMargin, Quality,
        Radius, Shadow, TextMargin,
    };
    use crate::DomainError;

    #[test]
    fn defaults_match_v1_6() {
        let config = Config::default();

        assert_eq!(config.version, "1.6.0");
        assert_eq!(config.output, "Pictures/watermark");
        assert!(!config.options.iot);
        assert!(!config.options.landscape);
        assert!(!config.options.solid_background);
        assert_eq!(config.options.solid_color.as_str(), "#fff");
        assert!(!config.options.origin_wh_output);
        assert_eq!(config.options.radius.get(), 2.1);
        assert!(config.options.radius_visible);
        assert_eq!(config.options.shadow.get(), 6.0);
        assert!(config.options.shadow_visible);
        assert!(!config.options.background_ratio_visible);
        assert_eq!(config.options.background_ratio.width(), 0.0);
        assert_eq!(config.options.background_ratio.height(), 0.0);
        assert_eq!(config.options.font.as_str(), "PingFang SC");
        assert_eq!(config.options.main_image_width.get(), 90);
        assert_eq!(config.options.text_margin.get(), 0.4);
        assert_eq!(config.options.quality.get(), 100);
        assert_eq!(config.options.mini_top_bottom_margin.get(), 0.0);
        assert_eq!(config.options.background_blur.get(), 100);
        assert!(!config.options.preview_visible);
    }

    #[test]
    fn integer_ranges_are_exact() {
        assert_eq!(
            MainImageWidth::try_from(0),
            Err(DomainError::OutOfRange("main_img_w_rate"))
        );
        assert!(MainImageWidth::try_from(1).is_ok());
        assert!(MainImageWidth::try_from(100).is_ok());
        assert_eq!(
            MainImageWidth::try_from(101),
            Err(DomainError::OutOfRange("main_img_w_rate"))
        );

        assert_eq!(
            Quality::try_from(0),
            Err(DomainError::OutOfRange("quality"))
        );
        assert!(Quality::try_from(1).is_ok());
        assert!(Quality::try_from(100).is_ok());
        assert_eq!(
            Quality::try_from(101),
            Err(DomainError::OutOfRange("quality"))
        );

        assert!(BackgroundBlur::try_from(0).is_ok());
        assert!(BackgroundBlur::try_from(100).is_ok());
        assert_eq!(
            BackgroundBlur::try_from(101),
            Err(DomainError::OutOfRange("bg_blur"))
        );
    }

    #[test]
    fn decimal_ranges_and_precision_are_exact() {
        assert!(Radius::try_from(0.0).is_ok());
        assert!(Radius::try_from(50.0).is_ok());
        assert_eq!(
            Radius::try_from(-0.1),
            Err(DomainError::OutOfRange("radius"))
        );
        assert_eq!(
            Radius::try_from(50.1),
            Err(DomainError::OutOfRange("radius"))
        );
        assert_eq!(
            Radius::try_from(2.15),
            Err(DomainError::InvalidValue("radius"))
        );

        assert!(Shadow::try_from(0.0).is_ok());
        assert!(Shadow::try_from(50.0).is_ok());
        assert_eq!(
            Shadow::try_from(6.01),
            Err(DomainError::InvalidValue("shadow"))
        );

        assert!(TextMargin::try_from(0.0).is_ok());
        assert!(TextMargin::try_from(10_000.0).is_ok());
        assert_eq!(
            TextMargin::try_from(10_000.01),
            Err(DomainError::OutOfRange("text_margin"))
        );
        assert_eq!(
            TextMargin::try_from(0.001),
            Err(DomainError::InvalidValue("text_margin"))
        );

        assert!(MiniTopBottomMargin::try_from(0.0).is_ok());
        assert!(MiniTopBottomMargin::try_from(100.0).is_ok());
        assert_eq!(
            MiniTopBottomMargin::try_from(100.01),
            Err(DomainError::OutOfRange("mini_top_bottom_margin"))
        );
        assert_eq!(
            MiniTopBottomMargin::try_from(0.001),
            Err(DomainError::InvalidValue("mini_top_bottom_margin"))
        );

        assert_eq!(
            Radius::try_from(f64::NAN),
            Err(DomainError::OutOfRange("radius"))
        );
        assert_eq!(
            TextMargin::try_from(f64::INFINITY),
            Err(DomainError::OutOfRange("text_margin"))
        );
    }

    #[test]
    fn background_ratio_requires_non_negative_finite_components() {
        let inactive = BackgroundRatio::try_from((0.0, 3.0)).expect("zero is valid but inactive");
        assert!(!inactive.is_active());

        let active = BackgroundRatio::try_from((3.0, 2.0)).expect("positive ratio is active");
        assert!(active.is_active());

        assert_eq!(
            BackgroundRatio::try_from((-1.0, 2.0)),
            Err(DomainError::OutOfRange("bg_rate"))
        );
        assert_eq!(
            BackgroundRatio::try_from((1.0, f64::INFINITY)),
            Err(DomainError::OutOfRange("bg_rate"))
        );
    }
}
