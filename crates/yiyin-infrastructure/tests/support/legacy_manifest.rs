//! Loader for the frozen legacy compatibility benchmark captured in
//! `tests/fixtures/legacy/manifest.json`. Golden tests must consume these
//! values instead of re-hardcoding them so a bad capture or manifest edit
//! fails loudly instead of diverging silently.

#![allow(
    dead_code,
    reason = "each test binary consumes only the manifest fields it asserts on"
)]

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer};

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
pub struct Manifest {
    pub scenarios: Vec<Scenario>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scenario {
    pub id: String,
    pub input: String,
    pub options: serde_json::Value,
    pub template_keys: Vec<String>,
    pub expected_output: String,
    pub expected_metadata: String,
    pub output: ExpectedOutput,
    pub exact_geometry: ExactGeometry,
    pub perceptual_threshold: Threshold,
}

#[derive(Deserialize)]
pub struct ExpectedOutput {
    pub width: u32,
    pub height: u32,
    pub density: Option<u32>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Threshold {
    #[serde(deserialize_with = "exact_f64")]
    pub min_ssim: f64,
    #[serde(deserialize_with = "exact_f64")]
    pub max_changed_pixel_ratio: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExactGeometry {
    pub canvas: Canvas,
    pub main_rect: RectGeometry,
    pub text_rows: Vec<TextRow>,
    pub mask_surface: MaskSurfaceGeometry,
    #[serde(deserialize_with = "exact_f64")]
    pub shadow_blur: f64,
    #[serde(deserialize_with = "exact_f64")]
    pub corner_radius: f64,
}

#[derive(Deserialize)]
pub struct Canvas {
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize)]
pub struct RectGeometry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize)]
pub struct TextRow {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    #[serde(deserialize_with = "exact_f64")]
    pub height: f64,
}

#[derive(Deserialize)]
pub struct MaskSurfaceGeometry {
    pub width: u32,
    pub height: u32,
    #[serde(deserialize_with = "exact_f64")]
    pub scale: f64,
}

pub fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
}

pub fn legacy_root() -> PathBuf {
    fixture_root().join("legacy")
}

pub fn load_manifest() -> Manifest {
    serde_json::from_slice(&fs::read(legacy_root().join("manifest.json")).expect("read manifest"))
        .expect("parse manifest")
}

/// Loads the frozen normalized-EXIF capture a scenario points at via its
/// `expectedMetadata` path.
pub fn load_expected_metadata(scenario: &Scenario) -> BTreeMap<String, String> {
    load_expected_metadata_at(&legacy_root().join(&scenario.expected_metadata))
}

pub fn load_expected_metadata_at(path: &Path) -> BTreeMap<String, String> {
    serde_json::from_slice(&fs::read(path).expect("read expected metadata"))
        .expect("parse expected metadata")
}
