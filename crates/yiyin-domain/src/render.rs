use std::collections::BTreeSet;

use crate::{Config, DomainError, Metadata, RenderOptions, ResourceId, TaskId};

const SHADOW_SURFACE_WIDTH_CAP: u32 = 10_240;
const TEXT_BOTTOM_OFFSET_RATE: f64 = 0.027;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

impl ImageDimensions {
    /// Creates non-zero image dimensions.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::OutOfRange`] when either dimension is zero.
    pub const fn new(width: u32, height: u32) -> Result<Self, DomainError> {
        if width == 0 || height == 0 {
            Err(DomainError::OutOfRange("image_dimensions"))
        } else {
            Ok(Self { width, height })
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageDensity(u32);

impl ImageDensity {
    /// Creates a positive pixels-per-inch value.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::OutOfRange`] when `pixels_per_inch` is zero.
    pub const fn new(pixels_per_inch: u32) -> Result<Self, DomainError> {
        if pixels_per_inch == 0 {
            Err(DomainError::OutOfRange("image_density"))
        } else {
            Ok(Self(pixels_per_inch))
        }
    }

    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextMeasurement {
    width: u32,
    height: f64,
}

impl TextMeasurement {
    /// Creates a positive text bitmap measurement.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::OutOfRange`] for zero width or a non-positive,
    /// non-finite height.
    pub fn new(width: u32, height: f64) -> Result<Self, DomainError> {
        if width == 0 || !height.is_finite() || height <= 0.0 {
            Err(DomainError::OutOfRange("text_measurement"))
        } else {
            Ok(Self { width, height })
        }
    }

    #[must_use]
    pub const fn width(self) -> u32 {
        self.width
    }

    #[must_use]
    pub const fn height(self) -> f64 {
        self.height
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderRequest {
    task_id: TaskId,
    input: ResourceId,
    output_name: String,
    input_dimensions: ImageDimensions,
    density: Option<ImageDensity>,
    config: Config,
    metadata: Metadata,
    text_rows: Vec<TextMeasurement>,
    preview: bool,
}

impl RenderRequest {
    #[must_use]
    pub fn new(
        task_id: TaskId,
        input: ResourceId,
        output_name: impl Into<String>,
        input_dimensions: ImageDimensions,
        options: RenderOptions,
    ) -> Self {
        let config = Config {
            options,
            ..Config::default()
        };
        Self::freeze(
            task_id,
            input,
            output_name,
            input_dimensions,
            config,
            Metadata::default(),
        )
    }

    #[must_use]
    pub fn freeze(
        task_id: TaskId,
        input: ResourceId,
        output_name: impl Into<String>,
        input_dimensions: ImageDimensions,
        config: Config,
        metadata: Metadata,
    ) -> Self {
        Self {
            task_id,
            input,
            output_name: output_name.into(),
            input_dimensions,
            density: None,
            config,
            metadata,
            text_rows: Vec::new(),
            preview: false,
        }
    }

    #[must_use]
    pub fn with_text_rows(mut self, text_rows: Vec<TextMeasurement>) -> Self {
        self.text_rows = text_rows;
        self
    }

    #[must_use]
    pub const fn with_density(mut self, density: ImageDensity) -> Self {
        self.density = Some(density);
        self
    }

    #[must_use]
    pub const fn as_preview(mut self) -> Self {
        self.preview = true;
        self
    }

    #[must_use]
    pub const fn task_id(&self) -> &TaskId {
        &self.task_id
    }

    #[must_use]
    pub const fn input(&self) -> &ResourceId {
        &self.input
    }

    #[must_use]
    pub fn output_name(&self) -> &str {
        &self.output_name
    }

    #[must_use]
    pub const fn input_dimensions(&self) -> ImageDimensions {
        self.input_dimensions
    }

    #[must_use]
    pub const fn density(&self) -> Option<ImageDensity> {
        self.density
    }

    #[must_use]
    pub const fn options(&self) -> &RenderOptions {
        &self.config.options
    }

    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub const fn metadata(&self) -> &Metadata {
        &self.metadata
    }

    #[must_use]
    pub fn text_rows(&self) -> &[TextMeasurement] {
        &self.text_rows
    }

    #[must_use]
    pub const fn is_preview(&self) -> bool {
        self.preview
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TextRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: f64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MaskSurface {
    pub width: u32,
    pub height: u32,
    pub scale: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RenderPlan {
    pub canvas: ImageDimensions,
    pub main_rect: Rect,
    pub text_rows: Vec<TextRect>,
    pub mask_surface: MaskSurface,
    pub shadow_blur: f64,
    pub corner_radius: f64,
}

impl RenderPlan {
    /// Builds renderer-independent geometry in the legacy calculation order.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::OutOfRange`] when configured ratios produce a
    /// non-finite or larger-than-`u32` surface.
    pub fn build(request: &RenderRequest) -> Result<Self, DomainError> {
        let input = request.input_dimensions;
        let adjusted = ratio_adjusted_dimensions(input, request.options())?;
        let reset = apply_landscape(adjusted, request.options());
        let initial_background = expand_for_main_width(
            input.height,
            reset,
            input.width,
            request.options().main_image_width.get(),
        )?;
        let bottom_offset = f64::from(initial_background.height) * TEXT_BOTTOM_OFFSET_RATE;
        let content = vertical_spacing(
            initial_background.height,
            input.height,
            &request.text_rows,
            request.options(),
        )?;
        let canvas = expand_for_main_width(
            content.height,
            reset,
            input.width,
            request.options().main_image_width.get(),
        )?;
        let main_rect = recenter_content(canvas, input, content.height, content.top)?;
        let text_rows = place_text_rows(canvas, &request.text_rows, bottom_offset)?;
        let mask_surface = shadow_surface(canvas)?;
        let scaled_main_height = ceil_u32(f64::from(input.height) * mask_surface.scale)?;
        let shadow_blur = if request.options().shadow_visible {
            f64::from(scaled_main_height) * (request.options().shadow.get() / 100.0)
                / mask_surface.scale
        } else {
            0.0
        };
        let corner_radius = if request.options().radius_visible {
            f64::from(scaled_main_height) * (request.options().radius.get() / 100.0)
                / mask_surface.scale
        } else {
            0.0
        };

        Ok(Self {
            canvas,
            main_rect,
            text_rows,
            mask_surface,
            shadow_blur,
            corner_radius,
        })
    }
}

fn ratio_adjusted_dimensions(
    input: ImageDimensions,
    options: &RenderOptions,
) -> Result<ImageDimensions, DomainError> {
    if !options.background_ratio_visible || !options.background_ratio.is_active() {
        return Ok(input);
    }

    let ratio = options.background_ratio.width() / options.background_ratio.height();
    if input.width >= input.height {
        ImageDimensions::new(input.width, round_u32(f64::from(input.width) / ratio)?)
    } else {
        ImageDimensions::new(round_u32(f64::from(input.height) * ratio)?, input.height)
    }
}

fn apply_landscape(reset: ImageDimensions, options: &RenderOptions) -> ImageDimensions {
    if options.landscape && reset.width < reset.height {
        ImageDimensions {
            width: reset.height,
            height: reset.width,
        }
    } else {
        reset
    }
}

fn expand_for_main_width(
    requested_height: u32,
    reset: ImageDimensions,
    main_width: u32,
    width_percent: u8,
) -> Result<ImageDimensions, DomainError> {
    let ratio = f64::from(reset.width) / f64::from(reset.height);
    let mut height = requested_height;
    let mut width = ceil_u32(f64::from(height) * ratio)?;
    let maximum_main_ratio = f64::from(width_percent) / 100.0;

    if f64::from(main_width) / f64::from(width) > maximum_main_ratio {
        width = ceil_u32(f64::from(main_width) / maximum_main_ratio)?;
        height = ceil_u32(f64::from(width) / ratio)?;
    }

    ImageDimensions::new(width, height)
}

struct ContentSpacing {
    height: u32,
    top: u32,
}

fn vertical_spacing(
    background_height: u32,
    main_height: u32,
    text_rows: &[TextMeasurement],
    options: &RenderOptions,
) -> Result<ContentSpacing, DomainError> {
    let minimum_top =
        ceil_u32(f64::from(background_height) * options.mini_top_bottom_margin.get() / 100.0)?;
    let shadow_top = if options.shadow_visible {
        ceil_u32(f64::from(main_height) * options.shadow.get() / 100.0)?
    } else {
        0
    };
    let top = minimum_top.max(shadow_top);
    let mut main_offset = f64::from(top * 2);
    if !text_rows.is_empty() {
        main_offset *= 0.75;
        main_offset += f64::from(background_height) * TEXT_BOTTOM_OFFSET_RATE;
    }
    let text_height = text_rows.iter().map(|row| row.height).sum::<f64>();
    let height = ceil_u32(text_height + f64::from(main_height) + main_offset)?;

    Ok(ContentSpacing { height, top })
}

fn recenter_content(
    canvas: ImageDimensions,
    main: ImageDimensions,
    content_height: u32,
    top: u32,
) -> Result<Rect, DomainError> {
    let x = round_u32(f64::from(canvas.width - main.width) / 2.0)?;
    let vertical_center = round_u32(f64::from(canvas.height - content_height) / 2.0)?;
    Ok(Rect {
        x,
        y: top + vertical_center,
        width: main.width,
        height: main.height,
    })
}

fn place_text_rows(
    canvas: ImageDimensions,
    measurements: &[TextMeasurement],
    bottom_offset: f64,
) -> Result<Vec<TextRect>, DomainError> {
    let mut reversed: Vec<TextRect> = Vec::with_capacity(measurements.len());
    for (reverse_index, measurement) in measurements.iter().rev().enumerate() {
        let height = if reverse_index == 0 {
            measurement.height + bottom_offset
        } else {
            measurement.height
        };
        let y = if let Some(previous) = reversed.last() {
            round_u32(f64::from(previous.y) - height)?
        } else {
            round_u32(f64::from(canvas.height) - height)?
        };
        reversed.push(TextRect {
            x: round_u32(f64::from(canvas.width - measurement.width) / 2.0)?,
            y,
            width: measurement.width,
            height,
        });
    }
    reversed.reverse();
    Ok(reversed)
}

fn shadow_surface(canvas: ImageDimensions) -> Result<MaskSurface, DomainError> {
    if canvas.width <= SHADOW_SURFACE_WIDTH_CAP {
        return Ok(MaskSurface {
            width: canvas.width,
            height: canvas.height,
            scale: 1.0,
        });
    }

    let scale = f64::from(SHADOW_SURFACE_WIDTH_CAP) / f64::from(canvas.width);
    Ok(MaskSurface {
        width: SHADOW_SURFACE_WIDTH_CAP,
        height: floor_u32(f64::from(canvas.height) * scale)?,
        scale,
    })
}

fn ceil_u32(value: f64) -> Result<u32, DomainError> {
    checked_u32(value.ceil())
}

fn floor_u32(value: f64) -> Result<u32, DomainError> {
    checked_u32(value.floor())
}

fn round_u32(value: f64) -> Result<u32, DomainError> {
    checked_u32(value.round())
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value is integral and bounded to the non-negative u32 range above"
)]
fn checked_u32(value: f64) -> Result<u32, DomainError> {
    if !value.is_finite() || value < 0.0 || value > f64::from(u32::MAX) {
        Err(DomainError::OutOfRange("render_geometry"))
    } else {
        Ok(value as u32)
    }
}

pub struct OutputNameResolver;

impl OutputNameResolver {
    /// Resolves the next JPEG file name using the v1.6 collision algorithm.
    ///
    /// # Errors
    ///
    /// Returns [`DomainError::Empty`] when the source has no usable stem and
    /// [`DomainError::OutOfRange`] if the greatest suffix cannot be incremented.
    pub fn resolve(
        source_name: &str,
        existing_names: &BTreeSet<String>,
    ) -> Result<String, DomainError> {
        let stem = file_stem(source_name).ok_or(DomainError::Empty("source_name"))?;
        let base_output = format!("{stem}.jpg");
        if !existing_names.contains(&base_output) {
            return Ok(base_output);
        }

        let prefix = format!("{stem}-");
        let greatest = existing_names
            .iter()
            .filter_map(|name| file_stem(name))
            .filter_map(|existing_stem| existing_stem.strip_prefix(&prefix))
            .filter(|suffix| !suffix.contains('-'))
            .filter_map(|suffix| suffix.parse::<u64>().ok())
            .max()
            .unwrap_or(0);
        let next = greatest
            .checked_add(1)
            .ok_or(DomainError::OutOfRange("output_name"))?;
        Ok(format!("{stem}-{next}.jpg"))
    }
}

fn file_stem(name: &str) -> Option<&str> {
    let file_name = name.rsplit(['/', '\\']).next()?;
    let stem = file_name
        .rsplit_once('.')
        .map_or(file_name, |(stem, _)| stem);
    (!stem.is_empty()).then_some(stem)
}

#[cfg(test)]
mod output_name_tests {
    use std::collections::BTreeSet;

    use super::OutputNameResolver;

    #[test]
    fn output_names_preserve_the_legacy_collision_algorithm() {
        assert_eq!(
            OutputNameResolver::resolve("photo.png", &BTreeSet::new()).unwrap(),
            "photo.jpg"
        );
        assert_eq!(
            OutputNameResolver::resolve("photo.png", &BTreeSet::from(["photo.jpg".to_owned()]))
                .unwrap(),
            "photo-1.jpg",
        );
        assert_eq!(
            OutputNameResolver::resolve(
                "photo.webp",
                &BTreeSet::from([
                    "photo.jpg".to_owned(),
                    "photo-1.jpg".to_owned(),
                    "photo-4.jpg".to_owned(),
                ]),
            )
            .unwrap(),
            "photo-5.jpg",
        );
        assert_eq!(
            OutputNameResolver::resolve(
                "photo.jpeg",
                &BTreeSet::from(["photo.jpg".to_owned(), "photo-final.jpg".to_owned()]),
            )
            .unwrap(),
            "photo-1.jpg",
        );
    }
}
