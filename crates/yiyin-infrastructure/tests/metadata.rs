use std::{fs, path::PathBuf};

use yiyin_application::{ErrorCode, MetadataReader};
use yiyin_domain::{BuiltInField, ImageOrientation};
use yiyin_infrastructure::{
    ExifMetadataReader, format_shutter, normalize_nikon_model, normalize_sony_model,
};

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/input")
}

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
fn generic_exif_fields_are_normalized_from_real_fixture() {
    let metadata = ExifMetadataReader
        .read(&fixtures().join("landscape-default.jpg"))
        .expect("read EXIF")
        .expect("metadata");

    assert_eq!(metadata.value(BuiltInField::Make), Some("ACME CORPORATION"));
    assert_eq!(metadata.value(BuiltInField::Model), Some("Camera One"));
    assert_eq!(metadata.value(BuiltInField::LensMake), Some("ACME Optics"));
    assert_eq!(metadata.value(BuiltInField::LensModel), Some("Prime 35"));
    assert_eq!(metadata.value(BuiltInField::FNumber), Some("2.8"));
    assert_eq!(metadata.value(BuiltInField::Iso), Some("200"));
    assert_eq!(metadata.value(BuiltInField::FocalLength), Some("35"));
    assert_eq!(
        metadata.value(BuiltInField::FocalLengthIn35mmFormat),
        Some("52")
    );
    assert_eq!(metadata.value(BuiltInField::ExposureTime), Some("1/125"));
    assert_eq!(
        metadata.value(BuiltInField::DateTimeOriginal),
        Some("2026/01/02 03:04:05")
    );
    assert_eq!(
        metadata.value(BuiltInField::ExposureCompensation),
        Some("+0.3")
    );
    assert_eq!(metadata.value(BuiltInField::WhiteBalance), Some("Auto"));
    assert_eq!(metadata.value(BuiltInField::ExposureProgram), Some("A"));
    assert_eq!(metadata.value(BuiltInField::MeteringMode), Some("评价测光"));
    assert_eq!(metadata.orientation(), Some(ImageOrientation::Normal));
    assert_eq!(
        metadata.density().map(yiyin_domain::ImageDensity::get),
        Some(300)
    );
}

#[test]
fn nikon_and_sony_fixture_values_remain_copy_exif_compatible() {
    let nikon = ExifMetadataReader
        .read(&fixtures().join("built-in-equivalent-focal.jpg"))
        .expect("read Nikon EXIF")
        .expect("Nikon metadata");
    assert_eq!(nikon.value(BuiltInField::Make), Some("NIKON CORPORATION"));
    assert_eq!(nikon.value(BuiltInField::Model), Some("NIKON Z 7_2"));

    let sony = ExifMetadataReader
        .read(&fixtures().join("logo-light.jpg"))
        .expect("read Sony EXIF")
        .expect("Sony metadata");
    assert_eq!(sony.value(BuiltInField::Make), Some("SONY"));
    assert_eq!(sony.value(BuiltInField::Model), Some("ILCE-7RM5"));
}

#[test]
fn webp_exif_chunk_uses_the_same_normalization_pipeline() {
    let metadata = ExifMetadataReader
        .read(&fixtures().join("webp-default.webp"))
        .expect("read WebP EXIF")
        .expect("WebP metadata");

    assert_eq!(metadata.value(BuiltInField::Make), Some("ACME CORPORATION"));
    assert_eq!(metadata.value(BuiltInField::Model), Some("Camera One"));
    assert_eq!(metadata.value(BuiltInField::ExposureTime), Some("1/125"));
    assert_eq!(
        metadata.density().map(yiyin_domain::ImageDensity::get),
        Some(300)
    );
}

#[test]
fn orientation_is_extracted_without_crossing_the_template_field_boundary() {
    let metadata = ExifMetadataReader
        .read(&fixtures().join("exif-orientation-6.jpg"))
        .expect("read oriented EXIF")
        .expect("oriented metadata");

    assert_eq!(metadata.orientation(), Some(ImageOrientation::Rotate90));
    assert_eq!(metadata.value(BuiltInField::Make), Some("ACME CORPORATION"));
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
