#[path = "support/faulty_filesystem.rs"]
mod faulty_filesystem;

use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use faulty_filesystem::{FaultPoint, FaultyFileSystem};
use yiyin_application::{ErrorCode, IdGenerator, ResourceRepository};
use yiyin_domain::{ResourceId, ResourceKind};
use yiyin_infrastructure::ResourceRegistry;

fn fixtures() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/input")
}

struct Harness {
    temp: tempfile::TempDir,
    inputs: PathBuf,
    registry: ResourceRegistry,
}

impl Harness {
    fn new() -> Self {
        let temp = tempfile::tempdir().expect("tempdir");
        let inputs = temp.path().join("inputs");
        let owned = temp.path().join("owned");
        fs::create_dir_all(&inputs).expect("input root");
        let registry = ResourceRegistry::new(owned.clone()).expect("resource registry");
        Self {
            temp,
            inputs,
            registry,
        }
    }

    fn copy_fixture(&self, fixture: &str, name: &str) -> PathBuf {
        let target = self.inputs.join(name);
        fs::copy(fixtures().join(fixture), &target).expect("copy fixture");
        target
    }
}

#[test]
fn jpeg_png_and_webp_are_registered_from_signatures() {
    let harness = Harness::new();
    let cases = [
        ("landscape-default.jpg", "photo.jpg", "image/jpeg"),
        ("portrait-default.png", "photo.png", "image/png"),
        ("webp-default.webp", "photo.webp", "image/webp"),
    ];

    for (fixture, name, mime) in cases {
        let source = harness.copy_fixture(fixture, name);
        let record = harness
            .registry
            .register_input(&source)
            .expect("register input");

        assert_eq!(record.kind(), ResourceKind::Input);
        assert_eq!(record.mime_type(), mime);
        assert!(record.dimensions().is_some());
        if fixture == "webp-default.webp" {
            assert_eq!(record.density(), None);
        } else {
            assert_eq!(
                record.density().map(yiyin_domain::ImageDensity::get),
                Some(300)
            );
        }
        assert_eq!(
            harness
                .registry
                .resolve(record.id())
                .expect("resolve input")
                .mime_type(),
            mime
        );
    }
}

#[test]
fn extension_mismatch_and_unsupported_content_are_rejected() {
    let harness = Harness::new();
    let mismatch = harness.copy_fixture("landscape-default.jpg", "photo.png");
    let unsupported = harness.inputs.join("not-an-image.jpg");
    fs::write(&unsupported, b"not an image").expect("unsupported fixture");

    assert_eq!(
        harness
            .registry
            .register_input(&mismatch)
            .unwrap_err()
            .code(),
        ErrorCode::FileInvalid
    );
    assert_eq!(
        harness
            .registry
            .register_input(&unsupported)
            .unwrap_err()
            .code(),
        ErrorCode::FileInvalid
    );
}

#[test]
fn owned_font_and_overlay_survive_source_deletion() {
    let harness = Harness::new();
    let font = harness.copy_fixture("千图小兔体.ttf", "font.ttf");
    let overlay = harness.copy_fixture("portrait-default.png", "overlay.png");

    let font_record = harness
        .registry
        .register_owned(ResourceKind::Font, &font, "Body")
        .expect("register font");
    let overlay_record = harness
        .registry
        .register_owned(ResourceKind::Overlay, &overlay, "Frame")
        .expect("register overlay");
    fs::remove_file(font).expect("remove font source");
    fs::remove_file(overlay).expect("remove overlay source");

    let resolved_font = harness
        .registry
        .resolve(font_record.id())
        .expect("resolve font");
    let resolved_overlay = harness
        .registry
        .resolve(overlay_record.id())
        .expect("resolve overlay");
    assert_eq!(resolved_font.mime_type(), "font/ttf");
    assert_eq!(resolved_overlay.mime_type(), "image/png");
    assert!(
        resolved_font
            .source()
            .starts_with(harness.registry.owned_root())
    );
    assert!(
        resolved_overlay
            .source()
            .starts_with(harness.registry.owned_root())
    );
    assert!(resolved_font.source().exists());
    assert!(resolved_overlay.source().exists());
}

#[test]
fn owned_font_and_overlay_survive_registry_restart() {
    let temp = tempfile::tempdir().expect("tempdir");
    let inputs = temp.path().join("inputs");
    let owned = temp.path().join("owned");
    fs::create_dir_all(&inputs).expect("input root");
    let font = inputs.join("font.ttf");
    let overlay = inputs.join("overlay.png");
    fs::copy(fixtures().join("千图小兔体.ttf"), &font).expect("font fixture");
    fs::copy(fixtures().join("portrait-default.png"), &overlay).expect("overlay fixture");

    let (font_id, overlay_id) = {
        let registry = ResourceRegistry::new(&owned).expect("resource registry");
        let font = registry
            .register_owned(ResourceKind::Font, &font, "Body")
            .expect("register font");
        let overlay = registry
            .register_owned(ResourceKind::Overlay, &overlay, "Frame")
            .expect("register overlay");
        (font.id().clone(), overlay.id().clone())
    };

    let restarted = ResourceRegistry::new(&owned).expect("restart registry");
    let font = restarted.resolve(&font_id).expect("reload font");
    let overlay = restarted.resolve(&overlay_id).expect("reload overlay");
    assert_eq!(font.display_name(), "Body");
    assert_eq!(font.kind(), ResourceKind::Font);
    assert_eq!(overlay.display_name(), "Frame");
    assert_eq!(overlay.kind(), ResourceKind::Overlay);
}

#[test]
fn persisted_resource_manifest_rejects_parent_traversal() {
    let temp = tempfile::tempdir().expect("tempdir");
    let owned = temp.path().join("owned");
    fs::create_dir_all(&owned).expect("owned root");
    fs::write(
        owned.join("resources.json"),
        r#"{
  "version": 1,
  "resources": [{
    "id": "escaped",
    "kind": "font",
    "display_name": "Escaped",
    "relative_path": "../outside.ttf"
  }]
}"#,
    )
    .expect("malicious manifest");

    assert_eq!(
        ResourceRegistry::new(&owned)
            .err()
            .expect("parent traversal is rejected")
            .code(),
        ErrorCode::Forbidden
    );
}

#[test]
fn bundled_assets_must_live_under_rust_owned_storage() {
    let harness = Harness::new();
    let bundled_root = harness.registry.owned_root().join("bundled");
    fs::create_dir_all(&bundled_root).expect("bundled root");
    let bundled = bundled_root.join("donation.jpg");
    fs::copy(fixtures().join("landscape-default.jpg"), &bundled).expect("bundled fixture");
    let external = harness.copy_fixture("landscape-default.jpg", "external.jpg");

    let record = harness
        .registry
        .register_bundled(&bundled, "zs-wx.jpg")
        .expect("register bundled asset");
    assert_eq!(record.kind(), ResourceKind::BundledAsset);
    assert_eq!(record.display_name(), "zs-wx.jpg");
    assert_eq!(
        harness
            .registry
            .register_bundled(&external, "external.jpg")
            .unwrap_err()
            .code(),
        ErrorCode::Forbidden
    );
}

#[test]
fn deleted_sources_and_unknown_ids_return_stable_errors() {
    let harness = Harness::new();
    let source = harness.copy_fixture("landscape-default.jpg", "photo.jpg");
    let record = harness
        .registry
        .register_input(&source)
        .expect("register input");
    fs::remove_file(source).expect("delete input");

    assert_eq!(
        harness.registry.resolve(record.id()).unwrap_err().code(),
        ErrorCode::FileNotFound
    );
    assert_eq!(
        harness
            .registry
            .resolve(&ResourceId::try_from("unknown").expect("resource id"))
            .unwrap_err()
            .code(),
        ErrorCode::ResourceNotFound
    );
}

#[cfg(unix)]
#[test]
fn resolve_rejects_a_registered_file_replaced_by_an_external_symlink() {
    use std::os::unix::fs::symlink;

    let harness = Harness::new();
    let source = harness.copy_fixture("landscape-default.jpg", "photo.jpg");
    let external = harness.temp.path().join("external.jpg");
    fs::copy(fixtures().join("landscape-default.jpg"), &external).expect("external image");
    let record = harness
        .registry
        .register_input(&source)
        .expect("register input");
    fs::remove_file(&source).expect("remove registered file");
    symlink(&external, &source).expect("replace with symlink");

    assert_eq!(
        harness.registry.resolve(record.id()).unwrap_err().code(),
        ErrorCode::Forbidden
    );
}

#[test]
fn generated_ids_are_opaque_uuid_capabilities() {
    let harness = Harness::new();
    let resource_id = harness.registry.next_resource_id();
    let task_id = harness.registry.next_task_id();

    assert!(uuid::Uuid::parse_str(resource_id.as_str()).is_ok());
    assert!(uuid::Uuid::parse_str(task_id.as_str()).is_ok());
    assert!(
        !resource_id
            .as_str()
            .contains(Path::new("/").to_string_lossy().as_ref())
    );
}

#[test]
fn registration_reports_dimensions_after_exif_orientation() {
    let harness = Harness::new();
    let source = harness.copy_fixture("exif-orientation-6.jpg", "oriented.jpg");

    let record = harness
        .registry
        .register_input(&source)
        .expect("register oriented image");

    assert_eq!(
        record.dimensions(),
        Some(yiyin_domain::ImageDimensions::new(1436, 2188).expect("dimensions"))
    );
}

#[test]
fn removing_generated_resources_deletes_only_preview_cache_files() {
    let harness = Harness::new();
    let preview = harness.copy_fixture("landscape-default.jpg", "preview.jpg");
    let output = harness.copy_fixture("landscape-default.jpg", "output.jpg");
    let preview_record = harness
        .registry
        .register_generated(ResourceKind::Preview, &preview, "Preview")
        .expect("register preview");
    let output_record = harness
        .registry
        .register_generated(ResourceKind::Output, &output, "Output")
        .expect("register output");

    harness
        .registry
        .remove(preview_record.id())
        .expect("remove preview");
    harness
        .registry
        .remove(output_record.id())
        .expect("remove output record");

    assert!(!preview.exists());
    assert!(output.exists());
}

#[test]
fn a_failed_owned_publish_never_leaves_the_destination_behind() {
    let temp = tempfile::tempdir().expect("tempdir");
    let inputs = temp.path().join("inputs");
    fs::create_dir_all(&inputs).expect("input root");
    let font = inputs.join("font.ttf");
    fs::copy(fixtures().join("千图小兔体.ttf"), &font).expect("font fixture");
    let filesystem = FaultyFileSystem::default();
    let registry =
        ResourceRegistry::with_filesystem(&temp.path().join("owned"), Arc::new(filesystem.clone()))
            .expect("resource registry");
    filesystem.fail_once(FaultPoint::SyncParent);

    let error = registry
        .register_owned(ResourceKind::Font, &font, "Body")
        .expect_err("the publish must surface the sync_parent failure");

    assert_eq!(error.code(), ErrorCode::Internal);
    let fonts = registry.owned_root().join("fonts");
    assert!(
        fs::read_dir(&fonts)
            .expect("fonts directory")
            .next()
            .is_none(),
        "neither the destination nor its temporary may survive a failed publish"
    );
}

#[test]
fn registration_reads_dimensions_from_headers_without_full_pixel_decode() {
    let harness = Harness::new();
    let intact = harness.copy_fixture("portrait-default.png", "intact.png");
    let expected = harness
        .registry
        .register_input(&intact)
        .expect("register intact image")
        .dimensions()
        .expect("intact dimensions");

    // Truncate at the first IDAT data byte: the header is complete, but the
    // pixel data is gone. (JPEG cannot serve here — its decoder gray-fills
    // truncated scans and still returns Ok, so it cannot discriminate.)
    let bytes = fs::read(&intact).expect("read intact fixture");
    let mut cursor = 8usize; // skip the PNG signature
    let idat_data_start = loop {
        let chunk_length =
            u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().expect("chunk length"))
                as usize;
        if &bytes[cursor + 4..cursor + 8] == b"IDAT" {
            break cursor + 8;
        }
        cursor += 8 + chunk_length + 4;
    };
    let truncated = harness.inputs.join("truncated.png");
    fs::write(&truncated, &bytes[..idat_data_start]).expect("write truncated fixture");

    assert!(
        image::load_from_memory_with_format(&bytes[..idat_data_start], image::ImageFormat::Png)
            .is_err(),
        "the fixture must fail a full pixel decode to keep this test discriminating"
    );

    let record = harness
        .registry
        .register_input(&truncated)
        .expect("registration must not require a full pixel decode");

    assert_eq!(record.dimensions(), Some(expected));
}
