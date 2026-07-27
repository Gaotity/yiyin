use std::{fs, path::PathBuf};

#[test]
fn main_capability_and_production_window_expose_no_generic_native_api() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let capability: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join("capabilities/main.json")).expect("read capability"),
    )
    .expect("parse capability");
    let permissions = capability["permissions"]
        .as_array()
        .expect("capability permissions");
    assert_eq!(
        permissions,
        &[
            serde_json::Value::String("core:event:allow-listen".to_owned()),
            serde_json::Value::String("core:event:allow-unlisten".to_owned()),
            serde_json::Value::String("core:window:allow-start-dragging".to_owned()),
        ]
    );
    for forbidden in ["dialog:", "opener:", "fs:", "shell:", "http:"] {
        assert!(
            !permissions
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|permission| permission.starts_with(forbidden)),
            "forbidden permission {forbidden}"
        );
    }

    let config: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("tauri.conf.json")).expect("read Tauri config"))
            .expect("parse Tauri config");
    assert_eq!(config["identifier"], "io.github.gaotity.yiyin");
    assert_eq!(config["productName"], "壹印");
    assert_eq!(config["version"], "1.6.0");
    assert_eq!(config["app"]["withGlobalTauri"], false);
    let window = &config["app"]["windows"][0];
    assert_eq!(window["width"], 900);
    assert_eq!(window["height"], 730);
    assert_eq!(window["resizable"], false);
    assert_eq!(window["devtools"], false);
    let csp = config["app"]["security"]["csp"]
        .as_str()
        .expect("CSP string");
    assert!(csp.contains("default-src 'self'"));
    assert!(!csp.contains("'unsafe-eval'"));
    assert!(!csp.contains("https://*"));
}
