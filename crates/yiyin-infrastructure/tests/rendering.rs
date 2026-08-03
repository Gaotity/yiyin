#[path = "support/faulty_filesystem.rs"]
mod faulty_filesystem;

use std::{
    fs,
    path::PathBuf,
    sync::{Arc, RwLock},
};

use faulty_filesystem::{FaultPoint, FaultyFileSystem};
use yiyin_application::{
    CancellationProbe, ErrorCode, ImageRenderer, MetadataReader, ResourceRecord, ResourceRepository,
};
use yiyin_domain::{Config, ImageDimensions, RenderRequest, RenderStage, ResourceKind, TaskId};
use yiyin_infrastructure::{ExifMetadataReader, FileSystem, ResourceRegistry, RustImageRenderer};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/input")
}

struct NeverCancelled;

impl CancellationProbe for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

struct AlwaysCancelled;

impl CancellationProbe for AlwaysCancelled {
    fn is_cancelled(&self) -> bool {
        true
    }
}

/// Cancels once the watched path exists, modelling a task superseded at a
/// specific point of the publish sequence (after the write for the
/// temporary, after the rename for the destination).
struct CancelOnceExists {
    path: PathBuf,
}

impl CancellationProbe for CancelOnceExists {
    fn is_cancelled(&self) -> bool {
        self.path.exists()
    }
}

struct Harness {
    temp: tempfile::TempDir,
    registry: Arc<ResourceRegistry>,
    renderer: RustImageRenderer,
}

impl Harness {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let registry = Arc::new(
            ResourceRegistry::new(temp.path().join("resources")).expect("resource registry"),
        );
        let renderer = RustImageRenderer::new(
            Arc::clone(&registry),
            temp.path().join("output"),
            temp.path().join("preview"),
            fixtures().join("千图小兔体.ttf"),
        )
        .expect("renderer");
        Self {
            temp,
            registry,
            renderer,
        }
    }

    fn with_filesystem(filesystem: Arc<dyn FileSystem>) -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let registry = Arc::new(
            ResourceRegistry::new(temp.path().join("resources")).expect("resource registry"),
        );
        let renderer = RustImageRenderer::with_filesystem_and_bundled_fonts(
            Arc::clone(&registry),
            Arc::new(RwLock::new(temp.path().join("output"))),
            temp.path().join("preview"),
            &[fixtures().join("千图小兔体.ttf")],
            filesystem,
        )
        .expect("renderer");
        Self {
            temp,
            registry,
            renderer,
        }
    }

    fn request(&self, fixture: &str, preview: bool) -> RenderRequest {
        self.request_with(fixture, preview, |config| {
            for template in &mut config.templates {
                template.set_enabled(false);
            }
        })
    }

    fn request_with(
        &self,
        fixture: &str,
        preview: bool,
        configure: impl FnOnce(&mut Config),
    ) -> RenderRequest {
        let source = fixtures().join(fixture);
        let resource = self
            .registry
            .register_input(&source)
            .expect("register fixture");
        let metadata = ExifMetadataReader
            .read(&source)
            .expect("read metadata")
            .unwrap_or_default();
        let mut config = Config::default();
        configure(&mut config);
        let request = RenderRequest::freeze(
            TaskId::try_from(fixture).expect("task id"),
            resource.id().clone(),
            format!("{fixture}.jpg"),
            resource.dimensions().expect("image dimensions"),
            config,
            metadata,
        );
        let request = if let Some(density) = resource.density() {
            request.with_density(density)
        } else {
            request
        };
        if preview {
            request.as_preview()
        } else {
            request
        }
    }

    fn register_logo(&self) -> ResourceRecord {
        let source = self.temp.path().join("sony-w.png");
        fs::copy(fixtures().join("sony-w.png"), &source).expect("copy logo");
        self.registry
            .register_owned(ResourceKind::Overlay, &source, "Sony light")
            .expect("register logo")
    }

    fn logo_request(&self, logo: &ResourceRecord) -> RenderRequest {
        self.request_with("landscape-default.jpg", false, |config| {
            config
                .temp_fields
                .iter_mut()
                .find(|field| field.key().as_str() == "Make")
                .expect("Make field")
                .set_image_variants(Some(logo.id().clone()), Some(logo.id().clone()));
        })
    }
}

#[test]
fn jpeg_png_and_webp_decode_and_export_atomically() {
    let harness = Harness::new();
    for fixture in [
        "landscape-default.jpg",
        "portrait-default.png",
        "webp-default.webp",
    ] {
        let request = harness.request(fixture, false);
        let mut stages = Vec::new();
        let result = harness
            .renderer
            .render(&request, &NeverCancelled, &mut |stage| stages.push(stage))
            .expect("render fixture");

        assert_eq!(result.resource().kind(), ResourceKind::Output);
        assert_eq!(result.encoder_quality(), 100);
        assert_eq!(
            result.dimensions().width,
            image::open(
                harness
                    .registry
                    .resolve(result.resource().id())
                    .expect("resolve output")
                    .source()
            )
            .expect("decode output")
            .width()
        );
        assert_eq!(stages.first(), Some(&RenderStage::Initializing));
        assert_eq!(stages.last(), Some(&RenderStage::Completed));
        assert!(
            !harness
                .temp
                .path()
                .join("output")
                .join(format!("{fixture}.jpg.tmp"))
                .exists()
        );
    }
}

#[test]
fn preview_uses_quality_seventy_without_publishing_an_export() {
    let harness = Harness::new();
    let request = harness.request("landscape-default.jpg", true);

    let result = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .expect("render preview");

    assert_eq!(result.encoder_quality(), 70);
    assert_eq!(result.resource().kind(), ResourceKind::Preview);
    assert!(
        !harness
            .temp
            .path()
            .join("output")
            .join(request.output_name())
            .exists()
    );
    assert!(
        harness
            .registry
            .resolve(result.resource().id())
            .expect("resolve preview")
            .source()
            .exists()
    );
    let preview = harness
        .registry
        .resolve(result.resource().id())
        .expect("resolve preview")
        .source()
        .to_path_buf();
    harness
        .registry
        .remove(result.resource().id())
        .expect("remove preview");
    assert!(!preview.exists());
}

#[test]
fn a_new_preview_replaces_the_previous_preview_record() {
    let harness = Harness::new();
    let request = harness.request("landscape-default.jpg", true);

    let first = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .expect("first preview");
    let second = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .expect("second preview");

    let previews = harness
        .registry
        .snapshot()
        .into_iter()
        .filter(|record| record.kind() == ResourceKind::Preview)
        .collect::<Vec<_>>();
    assert_eq!(previews.len(), 1);
    assert_eq!(previews[0].id(), second.resource().id());
    assert_eq!(
        harness
            .registry
            .resolve(first.resource().id())
            .unwrap_err()
            .code(),
        ErrorCode::ResourceNotFound
    );
}

#[test]
fn a_superseded_preview_never_registers_a_record() {
    let harness = Harness::new();
    let request = harness.request("landscape-default.jpg", true);
    let destination = harness
        .temp
        .path()
        .join("preview")
        .join(format!("{}.jpg", request.task_id().as_str()));
    let probe = CancelOnceExists {
        path: destination.clone(),
    };

    let error = harness
        .renderer
        .render(&request, &probe, &mut |_| {})
        .expect_err("superseded preview must not complete");

    assert_eq!(error.code(), ErrorCode::Cancelled);
    assert!(
        harness
            .registry
            .snapshot()
            .iter()
            .all(|record| record.kind() != ResourceKind::Preview),
        "a superseded preview must not register (and evict) a preview record"
    );
    // The deterministic file is left in place for the current preview.
    assert!(destination.exists());

    let current = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .expect("current preview");
    assert!(
        harness.registry.resolve(current.resource().id()).is_ok(),
        "the current preview's record must resolve"
    );
}

#[test]
fn exif_orientation_is_applied_before_geometry() {
    let harness = Harness::new();
    let request = harness.request("exif-orientation-6.jpg", false);
    assert_eq!(
        request.input_dimensions(),
        ImageDimensions::new(1436, 2188).expect("oriented dimensions")
    );

    let result = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .expect("render oriented image");

    assert!(result.dimensions().height > result.dimensions().width);
}

#[test]
fn density_is_preserved_in_the_published_jpeg() {
    let harness = Harness::new();
    let request = harness.request("landscape-default.jpg", false);

    let result = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .expect("render with density");
    let output = harness
        .registry
        .resolve(result.resource().id())
        .expect("resolve output");
    let metadata = ExifMetadataReader
        .read(output.source())
        .expect("read output metadata")
        .expect("output metadata");

    assert_eq!(
        result.density().map(yiyin_domain::ImageDensity::get),
        Some(300)
    );
    assert_eq!(
        metadata.density().map(yiyin_domain::ImageDensity::get),
        Some(300)
    );
}

#[test]
fn cancellation_and_existing_outputs_never_publish_partial_bytes() {
    let harness = Harness::new();
    let request = harness.request("landscape-default.jpg", false);
    let output = harness
        .temp
        .path()
        .join("output")
        .join(request.output_name());

    let error = harness
        .renderer
        .render(&request, &AlwaysCancelled, &mut |_| {})
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::Cancelled);
    assert!(!output.exists());
    assert!(!output.with_extension("jpg.tmp").exists());

    fs::create_dir_all(output.parent().expect("output parent")).expect("output directory");
    fs::write(&output, b"successful prior output").expect("prior output");
    let error = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidRequest);
    assert_eq!(
        fs::read(output).expect("prior output remains"),
        b"successful prior output"
    );
}

#[test]
fn cancellation_between_the_write_and_the_rename_publishes_nothing() {
    let harness = Harness::new();
    let request = harness.request("landscape-default.jpg", false);
    let output = harness
        .temp
        .path()
        .join("output")
        .join(request.output_name());
    // The temporary appears right after write_and_sync and disappears into
    // the rename, so tripping on its existence cancels inside that window.
    let temporary = PathBuf::from(format!("{}.tmp", output.display()));
    let probe = CancelOnceExists {
        path: temporary.clone(),
    };

    let error = harness
        .renderer
        .render(&request, &probe, &mut |_| {})
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::Cancelled);
    assert!(!output.exists());
    assert!(!temporary.exists());
}

#[test]
fn a_font_read_failure_surfaces_as_internal() {
    let filesystem = FaultyFileSystem::default();
    let harness = Harness::with_filesystem(Arc::new(filesystem.clone()));
    let source = harness.temp.path().join("custom-font.ttf");
    fs::copy(fixtures().join("千图小兔体.ttf"), &source).expect("copy font");
    let font = harness
        .registry
        .register_owned(ResourceKind::Font, &source, "Custom font")
        .expect("register font");
    filesystem.fail_once(FaultPoint::Read(font.source().to_path_buf()));

    let request = harness.request("landscape-default.jpg", false);
    let error = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::Internal);
}

#[test]
fn a_logo_read_failure_surfaces_as_file_invalid() {
    let filesystem = FaultyFileSystem::default();
    let harness = Harness::with_filesystem(Arc::new(filesystem.clone()));
    let logo = harness.register_logo();
    filesystem.fail_once(FaultPoint::Read(logo.source().to_path_buf()));

    let request = harness.logo_request(&logo);
    let error = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::FileInvalid);
}

#[test]
fn undecodable_logo_bytes_surface_as_file_invalid() {
    let harness = Harness::new();
    let logo = harness.register_logo();
    fs::write(logo.source(), b"these are not image bytes").expect("corrupt logo");

    let request = harness.logo_request(&logo);
    let error = harness
        .renderer
        .render(&request, &NeverCancelled, &mut |_| {})
        .unwrap_err();

    assert_eq!(error.code(), ErrorCode::FileInvalid);
}
