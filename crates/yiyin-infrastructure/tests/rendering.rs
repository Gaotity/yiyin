use std::{fs, path::PathBuf, sync::Arc};

use yiyin_application::{
    CancellationProbe, ErrorCode, ImageRenderer, MetadataReader, ResourceRepository,
};
use yiyin_domain::{Config, ImageDimensions, RenderRequest, RenderStage, ResourceKind, TaskId};
use yiyin_infrastructure::{ExifMetadataReader, ResourceRegistry, RustImageRenderer};

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

    fn request(&self, fixture: &str, preview: bool) -> RenderRequest {
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
        for template in &mut config.templates {
            template.set_enabled(false);
        }
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
