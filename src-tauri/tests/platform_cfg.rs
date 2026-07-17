#[test]
fn legacy_base_is_only_compiled_on_supported_import_platforms() {
    let source = include_str!("../src/state.rs");

    assert!(source.contains(
        r#"#[cfg(any(target_os = "macos", target_os = "windows"))]
        let legacy_base = app_data.parent().unwrap_or(&app_data);"#
    ));
}
