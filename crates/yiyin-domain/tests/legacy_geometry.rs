#![allow(
    clippy::float_cmp,
    clippy::unreadable_literal,
    reason = "constants and equality intentionally mirror the committed legacy manifest"
)]

use yiyin_domain::{
    BackgroundRatio, ImageDimensions, MaskSurface, Rect, RenderOptions, RenderPlan, RenderRequest,
    ResourceId, TaskId, TextMeasurement, TextRect,
};

struct Fixture {
    id: &'static str,
    input: (u32, u32),
    text: &'static [(u32, f64)],
    expected_canvas: (u32, u32),
    expected_main: (u32, u32, u32, u32),
    expected_text: &'static [(u32, u32, u32, f64)],
    shadow_blur: f64,
    corner_radius: f64,
}

const DEFAULT_LANDSCAPE_TEXT: &[(u32, f64)] = &[(390, 51.0), (465, 47.0)];
const DEFAULT_LANDSCAPE_ROWS: &[(u32, u32, u32, f64)] =
    &[(1107, 1567, 390, 51.0), (1069, 1618, 465, 90.119)];

const FIXTURES: &[Fixture] = &[
    Fixture {
        id: "portrait-default",
        input: (980, 1468),
        text: &[(398, 52.0), (474, 48.0)],
        expected_canvas: (1166, 1746),
        expected_main: (93, 89, 980, 1468),
        expected_text: &[(384, 1602, 398, 52.0), (346, 1654, 474, 92.064)],
        shadow_blur: 88.08,
        corner_radius: 30.828000000000003,
    },
    Fixture {
        id: "landscape-default",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
        expected_canvas: (2603, 1708),
        expected_main: (208, 87, 2188, 1436),
        expected_text: DEFAULT_LANDSCAPE_ROWS,
        shadow_blur: 86.16,
        corner_radius: 30.156000000000002,
    },
    Fixture {
        id: "webp-default",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
        expected_canvas: (2603, 1708),
        expected_main: (208, 87, 2188, 1436),
        expected_text: DEFAULT_LANDSCAPE_ROWS,
        shadow_blur: 86.16,
        corner_radius: 30.156000000000002,
    },
    Fixture {
        id: "exif-orientation-6",
        input: (1436, 2188),
        text: &[(563, 77.0), (677, 71.0)],
        expected_canvas: (1707, 2600),
        expected_main: (136, 132, 1436, 2188),
        expected_text: &[(572, 2386, 563, 77.0), (515, 2463, 677, 136.664)],
        shadow_blur: 131.28,
        corner_radius: 45.948,
    },
    Fixture {
        id: "explicit-ratio-3x2",
        input: (980, 1468),
        text: &[(364, 47.0), (432, 43.0)],
        expected_canvas: (2598, 1732),
        expected_main: (809, 89, 980, 1468),
        expected_text: &[(1117, 1602, 364, 47.0), (1083, 1649, 432, 82.636)],
        shadow_blur: 88.08,
        corner_radius: 30.828000000000003,
    },
    Fixture {
        id: "portrait-to-landscape",
        input: (5568, 3712),
        text: &[(914, 130.0), (1108, 121.0)],
        expected_canvas: (6614, 4409),
        expected_main: (523, 223, 5568, 3712),
        expected_text: &[(2850, 4047, 914, 130.0), (2753, 4177, 1108, 232.375)],
        shadow_blur: 222.72,
        corner_radius: 77.952,
    },
    Fixture {
        id: "solid-white-no-shadow",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
        expected_canvas: (2432, 1597),
        expected_main: (122, 10, 2188, 1436),
        expected_text: &[(1021, 1456, 390, 51.0), (984, 1507, 465, 90.119)],
        shadow_blur: 0.0,
        corner_radius: 0.0,
    },
    Fixture {
        id: "blurred-shadow-radius",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
        expected_canvas: (2603, 1708),
        expected_main: (208, 87, 2188, 1436),
        expected_text: DEFAULT_LANDSCAPE_ROWS,
        shadow_blur: 86.16,
        corner_radius: 30.156000000000002,
    },
    Fixture {
        id: "built-in-equivalent-focal",
        input: (2188, 1436),
        text: &[(320, 62.0), (465, 47.0)],
        expected_canvas: (2620, 1719),
        expected_main: (216, 87, 2188, 1436),
        expected_text: &[(1150, 1567, 320, 62.0), (1078, 1629, 465, 90.119)],
        shadow_blur: 86.16,
        corner_radius: 30.156000000000002,
    },
    Fixture {
        id: "built-in-original-focal",
        input: (2188, 1436),
        text: &[(320, 62.0), (465, 47.0)],
        expected_canvas: (2620, 1719),
        expected_main: (216, 87, 2188, 1436),
        expected_text: &[(1150, 1567, 320, 62.0), (1078, 1629, 465, 90.119)],
        shadow_blur: 86.16,
        corner_radius: 30.156000000000002,
    },
    Fixture {
        id: "logo-light",
        input: (2188, 1436),
        text: &[(414, 62.0)],
        expected_canvas: (2548, 1672),
        expected_main: (180, 87, 2188, 1436),
        expected_text: &[(1067, 1567, 414, 105.119)],
        shadow_blur: 86.16,
        corner_radius: 30.156000000000002,
    },
    Fixture {
        id: "logo-dark",
        input: (2188, 1436),
        text: &[(414, 62.0)],
        expected_canvas: (2548, 1672),
        expected_main: (180, 87, 2188, 1436),
        expected_text: &[(1067, 1567, 414, 105.119)],
        shadow_blur: 86.16,
        corner_radius: 30.156000000000002,
    },
    Fixture {
        id: "custom-text-forced",
        input: (980, 1468),
        text: &[(312, 46.0)],
        expected_canvas: (1130, 1692),
        expected_main: (75, 89, 980, 1468),
        expected_text: &[(409, 1602, 312, 90.064)],
        shadow_blur: 88.08,
        corner_radius: 30.828000000000003,
    },
    Fixture {
        id: "bundled-custom-font",
        input: (2188, 1436),
        text: DEFAULT_LANDSCAPE_TEXT,
        expected_canvas: (2603, 1708),
        expected_main: (208, 87, 2188, 1436),
        expected_text: DEFAULT_LANDSCAPE_ROWS,
        shadow_blur: 86.16,
        corner_radius: 30.156000000000002,
    },
];

#[test]
fn every_legacy_fixture_has_exact_geometry() {
    for fixture in FIXTURES {
        let plan = RenderPlan::build(&request(fixture)).expect("fixture render plan");
        let expected_text = fixture
            .expected_text
            .iter()
            .map(|&(x, y, width, height)| TextRect {
                x,
                y,
                width,
                height,
            })
            .collect::<Vec<_>>();

        assert_eq!(
            plan.canvas,
            ImageDimensions::new(fixture.expected_canvas.0, fixture.expected_canvas.1)
                .expect("fixture canvas dimensions"),
            "{} canvas",
            fixture.id,
        );
        assert_eq!(
            plan.main_rect,
            Rect {
                x: fixture.expected_main.0,
                y: fixture.expected_main.1,
                width: fixture.expected_main.2,
                height: fixture.expected_main.3,
            },
            "{} main rect",
            fixture.id,
        );
        assert_eq!(plan.text_rows, expected_text, "{} text rows", fixture.id);
        assert_eq!(
            plan.mask_surface,
            MaskSurface {
                width: fixture.expected_canvas.0,
                height: fixture.expected_canvas.1,
                scale: 1.0,
            },
            "{} mask surface",
            fixture.id,
        );
        assert_eq!(
            plan.shadow_blur, fixture.shadow_blur,
            "{} shadow",
            fixture.id
        );
        assert_eq!(
            plan.corner_radius, fixture.corner_radius,
            "{} radius",
            fixture.id
        );
    }
}

fn request(fixture: &Fixture) -> RenderRequest {
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
fn shadow_mask_surface_is_capped_at_10240_pixels_wide() {
    let request = RenderRequest::new(
        TaskId::try_from("large").expect("task id"),
        ResourceId::try_from("large").expect("resource id"),
        "large.jpg",
        ImageDimensions::new(12_000, 8_000).expect("large dimensions"),
        RenderOptions::default(),
    );

    let plan = RenderPlan::build(&request).expect("large render plan");

    assert_eq!(plan.mask_surface.width, 10_240);
    assert!(plan.mask_surface.scale < 1.0);
}
