#![allow(
    clippy::float_cmp,
    reason = "geometry is compared exactly against the committed legacy manifest"
)]

use std::{fs, path::PathBuf};

use serde::{Deserialize, Deserializer};
use yiyin_domain::{
    BackgroundRatio, ImageDimensions, Rect, RenderOptions, RenderPlan, RenderRequest, ResourceId,
    TaskId, TextMeasurement, TextRect,
};

/// `serde_json`'s default float parser is not correctly rounded for long decimal
/// strings (e.g. `30.156000000000002` loses 1 ulp), so exact comparisons
/// against the frozen manifest re-parse the raw token with `str::parse`.
fn exact_f64<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let raw = <Box<serde_json::value::RawValue>>::deserialize(deserializer)?;
    raw.get().parse::<f64>().map_err(serde::de::Error::custom)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    scenarios: Vec<Scenario>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Scenario {
    id: String,
    exact_geometry: ExactGeometry,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExactGeometry {
    canvas: Canvas,
    main_rect: RectGeometry,
    text_rows: Vec<TextRowGeometry>,
    #[serde(deserialize_with = "exact_f64")]
    shadow_blur: f64,
    #[serde(deserialize_with = "exact_f64")]
    corner_radius: f64,
}

#[derive(Deserialize)]
struct Canvas {
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
struct RectGeometry {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

#[derive(Deserialize)]
struct TextRowGeometry {
    x: u32,
    y: u32,
    width: u32,
    #[serde(deserialize_with = "exact_f64")]
    height: f64,
}

/// Renderer inputs the legacy capture did not freeze: the source image
/// dimensions and the measured text extents fed into the layout engine. The
/// expected geometry they produce lives only in the manifest.
struct FixtureInput {
    id: &'static str,
    input: (u32, u32),
    text: &'static [(u32, f64)],
}

const DEFAULT_LANDSCAPE_TEXT: &[(u32, f64)] = &[(390, 51.0), (465, 47.0)];

const FIXTURE_INPUTS: &[FixtureInput] = &[
    FixtureInput {
        id: "portrait-default",
        input: (980, 1468),
        text: &[(398, 52.0), (474, 48.0)],
    },
    FixtureInput {
        id: "landscape-default",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
    },
    FixtureInput {
        id: "webp-default",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
    },
    FixtureInput {
        id: "exif-orientation-6",
        input: (1436, 2188),
        text: &[(563, 77.0), (677, 71.0)],
    },
    FixtureInput {
        id: "explicit-ratio-3x2",
        input: (980, 1468),
        text: &[(364, 47.0), (432, 43.0)],
    },
    FixtureInput {
        id: "portrait-to-landscape",
        input: (5568, 3712),
        text: &[(914, 130.0), (1108, 121.0)],
    },
    FixtureInput {
        id: "solid-white-no-shadow",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
    },
    FixtureInput {
        id: "blurred-shadow-radius",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
    },
    FixtureInput {
        id: "built-in-equivalent-focal",
        input: (2188, 1436),
        text: &[(320, 62.0), (465, 47.0)],
    },
    FixtureInput {
        id: "built-in-original-focal",
        input: (2188, 1436),
        text: &[(320, 62.0), (465, 47.0)],
    },
    FixtureInput {
        id: "logo-light",
        input: (2188, 1436),
        text: &[(414, 62.0)],
    },
    FixtureInput {
        id: "logo-dark",
        input: (2188, 1436),
        text: &[(414, 62.0)],
    },
    FixtureInput {
        id: "custom-text-forced",
        input: (980, 1468),
        text: &[(312, 46.0)],
    },
    FixtureInput {
        id: "bundled-custom-font",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
    },
];

fn manifest() -> Manifest {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/legacy/manifest.json");
    serde_json::from_slice(&fs::read(path).expect("read manifest")).expect("parse manifest")
}

#[test]
fn every_legacy_fixture_has_exact_geometry() {
    let manifest = manifest();
    assert_eq!(
        manifest.scenarios.len(),
        FIXTURE_INPUTS.len(),
        "every manifest scenario needs renderer inputs and vice versa"
    );

    for fixture in FIXTURE_INPUTS {
        let geometry = &manifest
            .scenarios
            .iter()
            .find(|scenario| scenario.id == fixture.id)
            .unwrap_or_else(|| panic!("manifest scenario {}", fixture.id))
            .exact_geometry;
        let plan = RenderPlan::build(&request(fixture)).expect("fixture render plan");
        let expected_text = geometry
            .text_rows
            .iter()
            .map(|row| TextRect {
                x: row.x,
                y: row.y,
                width: row.width,
                height: row.height,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            plan.canvas,
            ImageDimensions::new(geometry.canvas.width, geometry.canvas.height)
                .expect("fixture canvas dimensions"),
            "{} canvas",
            fixture.id,
        );
        assert_eq!(
            plan.main_rect,
            Rect {
                x: geometry.main_rect.x,
                y: geometry.main_rect.y,
                width: geometry.main_rect.width,
                height: geometry.main_rect.height,
            },
            "{} main rect",
            fixture.id,
        );
        assert_eq!(plan.text_rows, expected_text, "{} text rows", fixture.id);
        assert_eq!(
            plan.shadow_blur, geometry.shadow_blur,
            "{} shadow",
            fixture.id
        );
        assert_eq!(
            plan.corner_radius, geometry.corner_radius,
            "{} radius",
            fixture.id
        );
    }
}

fn request(fixture: &FixtureInput) -> RenderRequest {
    let mut options = RenderOptions::default();
    match fixture.id {
        "explicit-ratio-3x2" => {
            options.background_ratio_visible = true;
            options.background_ratio = BackgroundRatio::try_from((3.0, 2.0)).expect("valid ratio");
        }
        "portrait-to-landscape" => options.landscape = true,
        "solid-white-no-shadow" => {
            options.solid_background = true;
            options.shadow_visible = false;
            options.radius_visible = false;
        }
        "logo-dark" => options.solid_background = true,
        _ => {}
    }

    let text_rows = fixture
        .text
        .iter()
        .map(|&(width, height)| {
            TextMeasurement::new(width, height).expect("fixture text dimensions")
        })
        .collect();

    RenderRequest::new(
        TaskId::try_from(fixture.id).expect("fixture task id"),
        ResourceId::try_from(fixture.id).expect("fixture resource id"),
        "fixture.jpg",
        ImageDimensions::new(fixture.input.0, fixture.input.1).expect("fixture input dimensions"),
        options,
    )
    .with_text_rows(text_rows)
}

#[test]
fn shadow_geometry_uses_the_surface_scale_capped_at_10240_pixels_wide() {
    let request = RenderRequest::new(
        TaskId::try_from("large").expect("task id"),
        ResourceId::try_from("large").expect("resource id"),
        "large.jpg",
        ImageDimensions::new(12_000, 8_000).expect("large dimensions"),
        RenderOptions::default(),
    );

    let plan = RenderPlan::build(&request).expect("large render plan");

    // The 13_440px-wide canvas exceeds the 10_240px surface cap, so the shadow
    // and radius scale by 10_240 / 13_440 instead of 1.0.
    assert_eq!(plan.shadow_blur, 480.06);
    assert_eq!(plan.corner_radius, 168.021_000_000_000_04);
}
