#[path = "support/legacy_manifest.rs"]
mod legacy_manifest;
#[path = "support/perceptual.rs"]
mod perceptual;

use std::{fs, path::PathBuf, sync::Arc};

use legacy_manifest::Scenario;
use yiyin_application::{CancellationProbe, ImageRenderer, MetadataReader, ResourceRepository};
use yiyin_domain::{
    BackgroundBlur, BackgroundRatio, CaseConversion, Config, FontSpec, ImageDensity,
    ImageDimensions, Quality, Radius, RenderRequest, ResourceKind, Shadow, TaskId, Template,
    TextMeasurement, VerticalAlign,
};
use yiyin_infrastructure::{ExifMetadataReader, ResourceRegistry, RustImageRenderer};

struct NeverCancelled;

impl CancellationProbe for NeverCancelled {
    fn is_cancelled(&self) -> bool {
        false
    }
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "the golden test keeps one readable end-to-end compatibility flow"
)]
fn rust_renderer_matches_every_captured_legacy_scenario() {
    let root = legacy_manifest::fixture_root();
    let manifest = legacy_manifest::load_manifest();
    let temp = tempfile::tempdir().expect("tempdir");
    let registry =
        Arc::new(ResourceRegistry::new(temp.path().join("resources")).expect("resource registry"));
    let renderer = RustImageRenderer::new(
        Arc::clone(&registry),
        temp.path().join("output"),
        temp.path().join("preview"),
        root.join("input/千图小兔体.ttf"),
    )
    .expect("renderer");
    let mut failures = Vec::new();

    for scenario in &manifest.scenarios {
        let source = root.join("legacy").join(&scenario.input);
        let resource = registry.register_input(&source).expect("register input");
        let metadata = ExifMetadataReader
            .read(&source)
            .expect("read fixture metadata")
            .unwrap_or_default();
        let mut config = scenario_config(scenario);
        if matches!(scenario.id.as_str(), "logo-light" | "logo-dark") {
            let dark = registry
                .register_owned(
                    ResourceKind::Overlay,
                    &root.join("input/sony-b.png"),
                    "Sony dark",
                )
                .expect("register dark Sony logo");
            let light = registry
                .register_owned(
                    ResourceKind::Overlay,
                    &root.join("input/sony-w.png"),
                    "Sony light",
                )
                .expect("register light Sony logo");
            config
                .temp_fields
                .iter_mut()
                .find(|field| field.key().as_str() == "Make")
                .expect("Make field")
                .set_image_variants(Some(dark.id().clone()), Some(light.id().clone()));
        }
        config.options.quality = Quality::try_from(100).expect("quality");
        let request = RenderRequest::freeze(
            TaskId::try_from(scenario.id.as_str()).expect("task id"),
            resource.id().clone(),
            format!("{}.jpg", scenario.id),
            resource.dimensions().expect("input dimensions"),
            config,
            metadata,
        )
        .with_text_rows(text_measurements(scenario));
        let request = if let Some(density) = resource.density() {
            request.with_density(density)
        } else {
            request
        };
        let result = renderer
            .render(&request, &NeverCancelled, &mut |_| {})
            .unwrap_or_else(|error| panic!("render {}: {error:?}", scenario.id));
        assert_eq!(
            result.dimensions(),
            ImageDimensions::new(scenario.output.width, scenario.output.height)
                .expect("expected dimensions"),
            "{} geometry",
            scenario.id
        );
        assert_eq!(
            result.density().map(ImageDensity::get),
            scenario.output.density,
            "{} density",
            scenario.id
        );

        let actual_record = registry
            .resolve(result.resource().id())
            .expect("resolve actual output");
        let actual = image::open(actual_record.source())
            .expect("decode actual")
            .to_rgb8();
        let expected = image::open(root.join("legacy").join(&scenario.expected_output))
            .expect("decode expected")
            .to_rgb8();
        assert_eq!(
            actual.dimensions(),
            expected.dimensions(),
            "{} pixels",
            scenario.id
        );
        let metrics = perceptual::compare(&actual, &expected);
        let passed = metrics.ssim >= scenario.perceptual_threshold.min_ssim
            && metrics.changed_pixel_ratio <= scenario.perceptual_threshold.max_changed_pixel_ratio;
        let artifact_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/golden-diffs")
            .join(&scenario.id);
        if passed {
            if artifact_root.exists() {
                fs::remove_dir_all(&artifact_root).expect("remove stale golden diff");
            }
        } else {
            perceptual::write_failure_artifacts(&artifact_root, &actual, &expected)
                .expect("write golden diff");
            failures.push(format!(
                "{}: SSIM {:.6} (min {:.2}), changed {:.6} (max {:.2})",
                scenario.id,
                metrics.ssim,
                scenario.perceptual_threshold.min_ssim,
                metrics.changed_pixel_ratio,
                scenario.perceptual_threshold.max_changed_pixel_ratio,
            ));
        }
    }

    assert!(failures.is_empty(), "\n{}", failures.join("\n"));
}

fn scenario_config(scenario: &Scenario) -> Config {
    let mut config = Config::default();
    for template in &mut config.templates {
        template.set_enabled(
            scenario
                .template_keys
                .iter()
                .any(|key| key == template.key()),
        );
    }
    if scenario
        .template_keys
        .iter()
        .any(|key| key == "fixture-signature")
    {
        let font = FontSpec::new("", 2.2, false, false, CaseConversion::Default, "")
            .expect("fixture font");
        let mut template = Template::custom(
            "fixture-signature",
            "Fixture signature",
            "{PersonalSign}",
            true,
            font,
        )
        .expect("fixture template");
        template.set_vertical_align(VerticalAlign::Center);
        config.templates.push(template);
        let field = config
            .temp_fields
            .iter_mut()
            .find(|field| field.key().as_str() == "PersonalSign")
            .expect("personal sign field");
        field.set_custom_text("YIYIN FIXTURE", true);
        field.set_font_override(Some(
            FontSpec::new("", 2.4, true, false, CaseConversion::Uppercase, "#ffffff")
                .expect("fixture field font"),
        ));
    }
    let options = &scenario.options;
    if options
        .get("landscape")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        config.options.landscape = true;
    }
    if let Some(solid) = options.get("solid_bg").and_then(serde_json::Value::as_bool) {
        config.options.solid_background = solid;
    }
    if let Some(visible) = options
        .get("shadow_show")
        .and_then(serde_json::Value::as_bool)
    {
        config.options.shadow_visible = visible;
    }
    if let Some(visible) = options
        .get("radius_show")
        .and_then(serde_json::Value::as_bool)
    {
        config.options.radius_visible = visible;
    }
    if let Some(value) = options.get("shadow").and_then(serde_json::Value::as_f64) {
        config.options.shadow = Shadow::try_from(value).expect("shadow");
    }
    if let Some(value) = options.get("radius").and_then(serde_json::Value::as_f64) {
        config.options.radius = Radius::try_from(value).expect("radius");
    }
    if let Some(value) = options.get("bg_blur").and_then(serde_json::Value::as_u64) {
        config.options.background_blur =
            BackgroundBlur::try_from(u8::try_from(value).expect("blur u8")).expect("blur");
    }
    if options
        .get("bg_rate_show")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false)
    {
        let ratio = options.get("bg_rate").expect("background ratio");
        config.options.background_ratio_visible = true;
        config.options.background_ratio = BackgroundRatio::try_from((
            ratio
                .get("w")
                .and_then(serde_json::Value::as_f64)
                .expect("ratio width"),
            ratio
                .get("h")
                .and_then(serde_json::Value::as_f64)
                .expect("ratio height"),
        ))
        .expect("ratio");
    }
    config
}

/// Rebuilds the measured text extents the legacy renderer was fed. Row widths
/// come from the frozen manifest's `exactGeometry.textRows`; the measured row
/// heights are renderer inputs the capture did not record, so they stay here.
fn text_measurements(scenario: &Scenario) -> Vec<TextMeasurement> {
    let measured_heights: &[f64] = match scenario.id.as_str() {
        "portrait-default" => &[52.0, 48.0],
        "landscape-default"
        | "webp-default"
        | "blurred-shadow-radius"
        | "solid-white-no-shadow"
        | "bundled-custom-font" => &[51.0, 47.0],
        "exif-orientation-6" => &[77.0, 71.0],
        "explicit-ratio-3x2" => &[47.0, 43.0],
        "portrait-to-landscape" => &[130.0, 121.0],
        "built-in-equivalent-focal" | "built-in-original-focal" => &[62.0, 47.0],
        "logo-light" | "logo-dark" => &[62.0],
        "custom-text-forced" => &[46.0],
        _ => panic!("unknown fixture {}", scenario.id),
    };
    let rows = &scenario.exact_geometry.text_rows;
    assert_eq!(
        rows.len(),
        measured_heights.len(),
        "{} text row count",
        scenario.id
    );
    rows.iter()
        .zip(measured_heights)
        .map(|(row, &height)| TextMeasurement::new(row.width, height).expect("text measurement"))
        .collect()
}
