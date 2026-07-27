use std::{fs, path::PathBuf, sync::Arc};

use yiyin_application::ResourceRepository;
use yiyin_desktop::protocol::ResourceProtocol;
use yiyin_infrastructure::ResourceRegistry;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/input/landscape-default.jpg")
}

#[test]
fn protocol_serves_only_one_current_opaque_resource_capability() {
    let temp = tempfile::tempdir().expect("tempdir");
    let source = temp.path().join("input.jpg");
    fs::copy(fixture(), &source).expect("copy fixture");
    let registry =
        Arc::new(ResourceRegistry::new(temp.path().join("resources")).expect("resource registry"));
    let record = registry.register_input(&source).expect("register input");
    let protocol = ResourceProtocol::new(registry);
    let valid_url = format!("yiyin://resource/{}", record.id().as_str());

    let valid = protocol.respond(&valid_url);
    assert_eq!(valid.status(), 200);
    assert_eq!(valid.content_type(), Some("image/jpeg"));
    assert_eq!(valid.body(), fs::read(&source).expect("fixture bytes"));
    assert_eq!(valid.header("x-content-type-options"), Some("nosniff"));

    for attack in [
        "yiyin://other/value".to_owned(),
        "yiyin://resource".to_owned(),
        "yiyin://resource/../secret".to_owned(),
        "yiyin://resource/%2Fsecret".to_owned(),
        "yiyin://resource/value%5Csecret".to_owned(),
        format!("{valid_url}/extra"),
        format!("{valid_url}?path=other"),
        "yiyin://resource/unknown-id".to_owned(),
        valid_url.replacen("yiyin:", "https:", 1),
    ] {
        assert_eq!(protocol.respond(&attack).status(), 403, "{attack}");
    }

    fs::remove_file(&source).expect("remove registered source");
    assert_eq!(protocol.respond(&valid_url).status(), 403);
}

#[cfg(unix)]
#[test]
fn protocol_revalidates_containment_and_mime_on_every_request() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().expect("tempdir");
    let source = temp.path().join("input.jpg");
    let external = temp.path().join("external.jpg");
    fs::copy(fixture(), &source).expect("copy fixture");
    fs::copy(fixture(), &external).expect("copy external fixture");
    let registry =
        Arc::new(ResourceRegistry::new(temp.path().join("resources")).expect("resource registry"));
    let record = registry.register_input(&source).expect("register input");
    let protocol = ResourceProtocol::new(registry);
    let url = format!("yiyin://resource/{}", record.id().as_str());

    fs::remove_file(&source).expect("remove source");
    symlink(&external, &source).expect("replace with symlink");
    assert_eq!(protocol.respond(&url).status(), 403);

    fs::remove_file(&source).expect("remove symlink");
    fs::write(&source, b"not an image").expect("replace MIME");
    assert_eq!(protocol.respond(&url).status(), 403);
}
