#[path = "support/legacy_manifest.rs"]
mod legacy_manifest;

use std::{collections::BTreeMap, fs};

use yiyin_application::{ErrorCode, MetadataReader};
use yiyin_domain::{BuiltInField, ImageOrientation, Metadata};
use yiyin_infrastructure::{
    ExifMetadataReader, format_shutter, normalize_nikon_model, normalize_sony_model,
};

#[test]
fn vendor_model_formatting_matches_the_legacy_template_formatter() {
    assert_eq!(normalize_nikon_model("NIKON", "NIKON Z 7_2"), " ℤ 7 Ⅱ");
    assert_eq!(normalize_sony_model("ILCE-7RM5"), "α7rm5");
}

#[test]
fn shutter_formatting_preserves_fractional_and_whole_seconds() {
    assert_eq!(format_shutter(1, 125), "1/125");
    assert_eq!(format_shutter(2, 1), "2");
    assert_eq!(format_shutter(3, 2), "1.5");
    assert_eq!(format_shutter(0, 1), "");
    assert_eq!(format_shutter(1, 0), "");
}

#[test]
fn every_manifest_scenario_metadata_matches_the_frozen_capture() {
    let manifest = legacy_manifest::load_manifest();
    for scenario in &manifest.scenarios {
        let metadata = read_scenario_metadata(&scenario.id);
        assert_matches_frozen_capture(&scenario.id, &metadata);
    }
}

#[test]
fn generic_exif_fields_are_normalized_from_real_fixture() {
    let metadata = read_scenario_metadata("landscape-default");
    assert_matches_frozen_capture("landscape-default", &metadata);

    assert_eq!(metadata.orientation(), Some(ImageOrientation::Normal));
    assert_eq!(
        metadata.density().map(yiyin_domain::ImageDensity::get),
        Some(300)
    );
}

#[test]
fn nikon_and_sony_fixture_values_remain_copy_exif_compatible() {
    let nikon = read_scenario_metadata("built-in-equivalent-focal");
    assert_matches_frozen_capture("built-in-equivalent-focal", &nikon);

    let sony = read_scenario_metadata("logo-light");
    assert_matches_frozen_capture("logo-light", &sony);
}

#[test]
fn webp_exif_chunk_uses_the_same_normalization_pipeline() {
    let metadata = read_scenario_metadata("webp-default");
    assert_matches_frozen_capture("webp-default", &metadata);

    assert_eq!(
        metadata.density().map(yiyin_domain::ImageDensity::get),
        Some(300)
    );
}

#[test]
fn orientation_is_extracted_without_crossing_the_template_field_boundary() {
    let metadata = read_scenario_metadata("exif-orientation-6");
    assert_matches_frozen_capture("exif-orientation-6", &metadata);

    assert_eq!(metadata.orientation(), Some(ImageOrientation::Rotate90));
}

/// Reads the manifest scenario's input image through the EXIF pipeline.
fn read_scenario_metadata(id: &str) -> Metadata {
    let manifest = legacy_manifest::load_manifest();
    let scenario = manifest
        .scenarios
        .iter()
        .find(|scenario| scenario.id == id)
        .unwrap_or_else(|| panic!("manifest scenario {id}"));
    ExifMetadataReader
        .read(&legacy_manifest::legacy_root().join(&scenario.input))
        .expect("read fixture EXIF")
        .expect("fixture metadata")
}

/// Diffs the reader output against the frozen normalized-EXIF capture in both
/// directions: every captured field must survive the pipeline unchanged, and
/// the pipeline must not produce fields the capture did not record.
fn assert_matches_frozen_capture(id: &str, metadata: &Metadata) {
    let manifest = legacy_manifest::load_manifest();
    let scenario = manifest
        .scenarios
        .iter()
        .find(|scenario| scenario.id == id)
        .unwrap_or_else(|| panic!("manifest scenario {id}"));
    let expected: BTreeMap<String, String> = legacy_manifest::load_expected_metadata(scenario);

    for (key, value) in &expected {
        let field = BuiltInField::from_key(key)
            .unwrap_or_else(|| panic!("{id}: frozen key {key} is not a built-in field"));
        assert_eq!(
            metadata.value(field),
            Some(value.as_str()),
            "{id} field {key}"
        );
    }
    for field in BuiltInField::ALL {
        assert_eq!(
            metadata.value(field),
            expected.get(field.key()).map(String::as_str),
            "{id} field {}",
            field.key()
        );
    }
}

#[test]
fn image_without_supported_exif_returns_none() {
    let temp = tempfile::tempdir().expect("tempdir");
    let path = temp.path().join("empty.png");
    image::RgbaImage::from_pixel(2, 2, image::Rgba([1, 2, 3, 255]))
        .save(&path)
        .expect("write PNG");

    assert_eq!(ExifMetadataReader.read(&path).expect("read PNG"), None);
}

#[test]
fn missing_and_malformed_files_map_to_stable_errors() {
    let temp = tempfile::tempdir().expect("tempdir");
    let missing = temp.path().join("missing.jpg");
    assert_eq!(
        ExifMetadataReader.read(&missing).unwrap_err().code(),
        ErrorCode::FileNotFound
    );

    let malformed = temp.path().join("malformed.jpg");
    fs::write(&malformed, b"not an image").expect("write malformed fixture");
    assert_eq!(
        ExifMetadataReader.read(&malformed).unwrap_err().code(),
        ErrorCode::FileInvalid
    );
}
