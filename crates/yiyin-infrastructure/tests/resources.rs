use std::{
    fs,
    path::{Path, PathBuf},
};

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
