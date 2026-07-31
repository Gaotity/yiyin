mod background;
mod composite;
mod text;

use std::{
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use image::{DynamicImage, ImageEncoder, RgbaImage, codecs::jpeg::JpegEncoder};
use yiyin_application::{
    ApplicationError, CancellationProbe, ImageRenderer, RenderResult, ResourceRepository,
    ResourceSnapshot,
};
use yiyin_domain::{
    BackgroundKind, BuiltInField, FieldValues, ImageDensity, ImageOrientation, RenderPlan,
    RenderRequest, RenderStage, ResourceKind, TemplateField, plan_rows,
};

use crate::{
    FileSystem, ResourceRegistry, StdFileSystem, normalize_make, normalize_model_for_templates,
};

const BUILT_IN_LOGO_VENDORS: [&str; 13] = [
    "canon",
    "dji",
    "fujifilm",
    "hasselblad",
    "leica",
    "nikon",
    "olympus",
    "panasonic",
    "pentax",
    "ricoh",
    "sigma",
    "songdian",
    "sony",
];

pub struct RustImageRenderer {
    resources: Arc<ResourceRegistry>,
    output_root: Arc<RwLock<PathBuf>>,
    preview_root: PathBuf,
    bundled_fonts: Vec<Vec<u8>>,
    filesystem: Arc<dyn FileSystem>,
}

impl RustImageRenderer {
    /// Creates a Rust-only renderer with explicit export and preview roots.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when its roots or deterministic font cannot be read.
    pub fn new(
        resources: Arc<ResourceRegistry>,
        output_root: PathBuf,
        preview_root: PathBuf,
        bundled_font: impl AsRef<Path>,
    ) -> Result<Self, ApplicationError> {
        let bundled_fonts = [bundled_font.as_ref().to_path_buf()];
        Self::with_filesystem_and_bundled_fonts(
            resources,
            Arc::new(RwLock::new(output_root)),
            preview_root,
            &bundled_fonts,
            Arc::new(StdFileSystem),
        )
    }

    /// Creates a renderer whose export root can be changed by the native adapter.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when its roots or deterministic font cannot be read.
    pub fn with_shared_output_root(
        resources: Arc<ResourceRegistry>,
        output_root: Arc<RwLock<PathBuf>>,
        preview_root: PathBuf,
        bundled_font: impl AsRef<Path>,
    ) -> Result<Self, ApplicationError> {
        let bundled_fonts = [bundled_font.as_ref().to_path_buf()];
        Self::with_filesystem_and_bundled_fonts(
            resources,
            output_root,
            preview_root,
            &bundled_fonts,
            Arc::new(StdFileSystem),
        )
    }

    /// Creates a renderer with every bundled product font and a shared export root.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when its roots or bundled fonts cannot be read.
    pub fn with_shared_output_root_and_bundled_fonts(
        resources: Arc<ResourceRegistry>,
        output_root: Arc<RwLock<PathBuf>>,
        preview_root: PathBuf,
        bundled_fonts: &[PathBuf],
    ) -> Result<Self, ApplicationError> {
        Self::with_filesystem_and_bundled_fonts(
            resources,
            output_root,
            preview_root,
            bundled_fonts,
            Arc::new(StdFileSystem),
        )
    }

    /// Creates a renderer with an injected filesystem adapter.
    ///
    /// # Errors
    ///
    /// Returns `INTERNAL` when its roots or deterministic font cannot be read.
    pub fn with_filesystem(
        resources: Arc<ResourceRegistry>,
        output_root: Arc<RwLock<PathBuf>>,
        preview_root: PathBuf,
        bundled_font: &Path,
        filesystem: Arc<dyn FileSystem>,
    ) -> Result<Self, ApplicationError> {
        Self::with_filesystem_and_bundled_fonts(
            resources,
            output_root,
            preview_root,
            &[bundled_font.to_path_buf()],
            filesystem,
        )
    }

    fn with_filesystem_and_bundled_fonts(
        resources: Arc<ResourceRegistry>,
        output_root: Arc<RwLock<PathBuf>>,
        preview_root: PathBuf,
        bundled_fonts: &[PathBuf],
        filesystem: Arc<dyn FileSystem>,
    ) -> Result<Self, ApplicationError> {
        if bundled_fonts.is_empty() {
            return Err(ApplicationError::internal(
                "at least one bundled font is required",
            ));
        }
        let initial_output_root = output_root
            .read()
            .map_err(|_| ApplicationError::internal("output root lock poisoned"))?
            .clone();
        filesystem
            .create_dir_all(&initial_output_root)
            .map_err(internal_io)?;
        filesystem
            .create_dir_all(&preview_root)
            .map_err(internal_io)?;
        let bundled_fonts = bundled_fonts
            .iter()
            .map(|font| filesystem.read(font).map_err(internal_io))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            resources,
            output_root,
            preview_root,
            bundled_fonts,
            filesystem,
        })
    }

    fn render_inner(
        &self,
        request: &RenderRequest,
        cancellation: &dyn CancellationProbe,
        progress: &mut dyn FnMut(RenderStage),
    ) -> Result<RenderResult, ApplicationError> {
        advance(cancellation, progress, RenderStage::Initializing)?;
        let input = self.resources.resolve(request.input())?;
        let bytes = self.filesystem.read(input.source()).map_err(internal_io)?;
        let decoded =
            image::load_from_memory(&bytes).map_err(|_| ApplicationError::file_invalid())?;
        let main = apply_orientation(decoded, request.metadata().orientation()).to_rgba8();
        if main.width() != request.input_dimensions().width
            || main.height() != request.input_dimensions().height
        {
            return Err(ApplicationError::file_invalid());
        }

        advance(cancellation, progress, RenderStage::ReadingMetadata)?;
        let mut display_metadata = request.metadata().clone();
        let raw_make = display_metadata
            .value(BuiltInField::Make)
            .unwrap_or_default()
            .to_owned();
        let raw_model = display_metadata
            .value(BuiltInField::Model)
            .unwrap_or_default()
            .to_owned();
        display_metadata.set(BuiltInField::Make, normalize_make(&raw_make));
        display_metadata.set(
            BuiltInField::Model,
            normalize_model_for_templates(&raw_make, &raw_model),
        );

        advance(cancellation, progress, RenderStage::PlanningBackground)?;
        let initial_plan = RenderPlan::build(&request.clone().with_text_rows(Vec::new()))
            .map_err(|error| ApplicationError::internal(error.to_string()))?;
        let background_kind = if request.options().solid_background {
            BackgroundKind::Light
        } else {
            BackgroundKind::Dark
        };

        advance(cancellation, progress, RenderStage::PlanningText)?;
        let mut fields = request.config().temp_fields.clone();
        fields.extend(request.config().custom_temp_fields.clone());
        apply_builtin_logo(&mut fields, request.metadata(), self.resources.as_ref());
        let rows = plan_rows(
            &request.config().templates,
            &FieldValues::new(display_metadata, fields),
            background_kind,
        );
        let text_context = text::RasterContext {
            background: background_kind,
            background_height: initial_plan.canvas.height,
            text_margin_percent: request.options().text_margin.get(),
            default_family: request.options().font.as_str(),
            bundled_fonts: &self.bundled_fonts,
            resources: self.resources.as_ref(),
        };
        let rendered_rows = text::rasterize_rows(&rows, &text_context, request.text_rows())?;
        let measurements = rendered_rows
            .iter()
            .map(|row| row.measurement)
            .collect::<Vec<_>>();

        advance(cancellation, progress, RenderStage::PreparingMainImage)?;
        let planned_request = request.clone().with_text_rows(measurements);
        advance(cancellation, progress, RenderStage::PlanningLayout)?;
        let plan = RenderPlan::build(&planned_request)
            .map_err(|error| ApplicationError::internal(error.to_string()))?;

        advance(cancellation, progress, RenderStage::RenderingBackground)?;
        let mut canvas = background::render_background(&main, &plan, request.options());
        composite::composite_main(&mut canvas, &main, &plan);

        advance(cancellation, progress, RenderStage::RenderingMask)?;
        for (row, rect) in rendered_rows.iter().zip(&plan.text_rows) {
            composite::overlay_rgba(
                &mut canvas,
                &row.image,
                i64::from(rect.x),
                i64::from(rect.y),
            );
        }
        ensure_active(cancellation)?;

        let result = self.publish(request, &canvas, &plan, cancellation)?;
        progress(RenderStage::Completed);
        Ok(result)
    }

    fn publish(
        &self,
        request: &RenderRequest,
        canvas: &RgbaImage,
        plan: &RenderPlan,
        cancellation: &dyn CancellationProbe,
    ) -> Result<RenderResult, ApplicationError> {
        let quality = if request.is_preview() {
            70
        } else {
            request.options().quality.get()
        };
        let density = request.density();
        let encoded = encode_jpeg(canvas, quality, density)?;
        let (kind, destination) = if request.is_preview() {
            (
                ResourceKind::Preview,
                self.preview_root
                    .join(format!("{}.jpg", request.task_id().as_str())),
            )
        } else {
            {
                let output_root = self
                    .output_root
                    .read()
                    .map_err(|_| ApplicationError::internal("output root lock poisoned"))?
                    .clone();
                self.filesystem
                    .create_dir_all(&output_root)
                    .map_err(internal_io)?;
                (
                    ResourceKind::Output,
                    output_root.join(request.output_name()),
                )
            }
        };
        if !request.is_preview() && self.filesystem.exists(&destination) {
            return Err(ApplicationError::invalid_request(
                "The output file already exists.",
            ));
        }
        crate::durable::durable_publish(
            self.filesystem.as_ref(),
            &destination,
            &encoded,
            None,
            Some(cancellation),
        )?;
        if request.is_preview()
            && let Err(error) = ensure_active(cancellation)
        {
            // A superseded preview must not register a record, or it could
            // evict the current preview's record; the deterministic file is
            // left in place for whichever preview publishes next.
            return Err(error);
        }
        let record =
            match self
                .resources
                .register_generated(kind, &destination, request.output_name())
            {
                Ok(record) => record,
                Err(error) => {
                    let _ =
                        crate::durable::remove_if_present(self.filesystem.as_ref(), &destination);
                    return Err(error);
                }
            };
        Ok(RenderResult::new(
            request.task_id().clone(),
            ResourceSnapshot::from_record(&record),
            plan.canvas,
            density,
            quality,
        ))
    }
}

fn apply_builtin_logo(
    fields: &mut [TemplateField],
    metadata: &yiyin_domain::Metadata,
    resources: &ResourceRegistry,
) {
    let Some(make) = metadata.value(BuiltInField::Make) else {
        return;
    };
    let lookup = make.replace("CORPORATION", "").trim().to_ascii_lowercase();
    if !BUILT_IN_LOGO_VENDORS.contains(&lookup.as_str()) {
        return;
    }
    let records = resources.snapshot();
    let id_for = |suffix: &str| {
        let name = format!("{lookup}-{suffix}.png");
        records
            .iter()
            .find(|record| {
                record.kind() == ResourceKind::BundledAsset && record.display_name() == name
            })
            .map(|record| record.id().clone())
    };
    let (Some(dark), Some(light)) = (id_for("b"), id_for("w")) else {
        return;
    };
    let Some(field) = fields
        .iter_mut()
        .find(|field| field.key().as_str() == BuiltInField::Make.key())
    else {
        return;
    };
    if field.content_kind() == yiyin_domain::FieldContentKind::Image
        || (field.uses_custom_value() && field.forces_custom_value())
    {
        return;
    }
    field.set_image_variants(Some(dark), Some(light));
}

impl ImageRenderer for RustImageRenderer {
    fn render(
        &self,
        request: &RenderRequest,
        cancellation: &dyn CancellationProbe,
        progress: &mut dyn FnMut(RenderStage),
    ) -> Result<RenderResult, ApplicationError> {
        self.render_inner(request, cancellation, progress)
    }
}

fn apply_orientation(image: DynamicImage, orientation: Option<ImageOrientation>) -> DynamicImage {
    match orientation.unwrap_or(ImageOrientation::Normal) {
        ImageOrientation::Normal => image,
        ImageOrientation::MirrorHorizontal => image.fliph(),
        ImageOrientation::Rotate180 => image.rotate180(),
        ImageOrientation::MirrorVertical => image.flipv(),
        ImageOrientation::MirrorHorizontalRotate270 => image.fliph().rotate270(),
        ImageOrientation::Rotate90 => image.rotate90(),
        ImageOrientation::MirrorHorizontalRotate90 => image.fliph().rotate90(),
        ImageOrientation::Rotate270 => image.rotate270(),
    }
}

fn encode_jpeg(
    canvas: &RgbaImage,
    quality: u8,
    density: Option<ImageDensity>,
) -> Result<Vec<u8>, ApplicationError> {
    let rgb = DynamicImage::ImageRgba8(canvas.clone()).to_rgb8();
    let mut encoded = Vec::new();
    JpegEncoder::new_with_quality(&mut encoded, quality)
        .write_image(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|error| ApplicationError::internal(error.to_string()))?;
    if let Some(density) = density {
        insert_density_exif(&mut encoded, density.get())?;
    }
    Ok(encoded)
}

fn insert_density_exif(encoded: &mut Vec<u8>, density: u32) -> Result<(), ApplicationError> {
    if !encoded.starts_with(&[0xff, 0xd8]) {
        return Err(ApplicationError::internal("JPEG encoder omitted SOI"));
    }
    let mut tiff = Vec::with_capacity(66);
    tiff.extend_from_slice(b"II");
    tiff.extend_from_slice(&42_u16.to_le_bytes());
    tiff.extend_from_slice(&8_u32.to_le_bytes());
    tiff.extend_from_slice(&3_u16.to_le_bytes());
    append_ifd_entry(&mut tiff, 0x011a, 5, 1, 50);
    append_ifd_entry(&mut tiff, 0x011b, 5, 1, 58);
    append_ifd_entry(&mut tiff, 0x0128, 3, 1, 2);
    tiff.extend_from_slice(&0_u32.to_le_bytes());
    tiff.extend_from_slice(&density.to_le_bytes());
    tiff.extend_from_slice(&1_u32.to_le_bytes());
    tiff.extend_from_slice(&density.to_le_bytes());
    tiff.extend_from_slice(&1_u32.to_le_bytes());

    let mut segment = Vec::with_capacity(76);
    segment.extend_from_slice(&[0xff, 0xe1]);
    let payload_length = 6_usize
        .checked_add(tiff.len())
        .and_then(|length| length.checked_add(2))
        .ok_or_else(|| ApplicationError::internal("EXIF density segment overflow"))?;
    let payload_length = u16::try_from(payload_length)
        .map_err(|_| ApplicationError::internal("EXIF density segment is too large"))?;
    segment.extend_from_slice(&payload_length.to_be_bytes());
    segment.extend_from_slice(b"Exif\0\0");
    segment.extend_from_slice(&tiff);
    encoded.splice(2..2, segment);
    Ok(())
}

fn append_ifd_entry(buffer: &mut Vec<u8>, tag: u16, field_type: u16, count: u32, value: u32) {
    buffer.extend_from_slice(&tag.to_le_bytes());
    buffer.extend_from_slice(&field_type.to_le_bytes());
    buffer.extend_from_slice(&count.to_le_bytes());
    buffer.extend_from_slice(&value.to_le_bytes());
}

fn advance(
    cancellation: &dyn CancellationProbe,
    progress: &mut dyn FnMut(RenderStage),
    stage: RenderStage,
) -> Result<(), ApplicationError> {
    ensure_active(cancellation)?;
    progress(stage);
    Ok(())
}

fn ensure_active(cancellation: &dyn CancellationProbe) -> Result<(), ApplicationError> {
    if cancellation.is_cancelled() {
        Err(ApplicationError::cancelled())
    } else {
        Ok(())
    }
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "the function is passed directly to Result::map_err"
)]
fn internal_io(error: std::io::Error) -> ApplicationError {
    ApplicationError::internal(error.to_string())
}

#[cfg(test)]
mod bundled_logo_tests {
    use std::fs;

    use yiyin_domain::{BuiltInField, FieldContentKind, Metadata, default_template_fields};

    use super::*;

    #[test]
    fn applies_registered_vendor_logo_without_overriding_a_forced_field() {
        let directory = tempfile::tempdir().expect("create resource root");
        let registry = ResourceRegistry::new(directory.path()).expect("create registry");
        let dark_path = directory.path().join("sony-b.png");
        let light_path = directory.path().join("sony-w.png");
        fs::write(
            &dark_path,
            include_bytes!("../../../../assets/logos/sony-b.png"),
        )
        .expect("write dark logo");
        fs::write(
            &light_path,
            include_bytes!("../../../../assets/logos/sony-w.png"),
        )
        .expect("write light logo");
        let dark = registry
            .register_bundled(&dark_path, "sony-b.png")
            .expect("register dark logo");
        let light = registry
            .register_bundled(&light_path, "sony-w.png")
            .expect("register light logo");
        let mut metadata = Metadata::default();
        metadata.set(BuiltInField::Make, "SONY CORPORATION");

        let mut fields = default_template_fields();
        apply_builtin_logo(&mut fields, &metadata, &registry);
        let make = fields
            .iter()
            .find(|field| field.key().as_str() == "Make")
            .expect("Make field");
        assert_eq!(make.content_kind(), FieldContentKind::Image);
        assert_eq!(make.dark_image(), Some(dark.id()));
        assert_eq!(make.light_image(), Some(light.id()));

        let mut forced = default_template_fields();
        forced
            .iter_mut()
            .find(|field| field.key().as_str() == "Make")
            .expect("Make field")
            .set_custom_text("Studio", true);
        apply_builtin_logo(&mut forced, &metadata, &registry);
        let make = forced
            .iter()
            .find(|field| field.key().as_str() == "Make")
            .expect("Make field");
        assert_eq!(make.content_kind(), FieldContentKind::Text);
        assert_eq!(make.custom_value(), "Studio");
    }
}
