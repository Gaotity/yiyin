#[test]
fn desktop_logging_excludes_dependency_debug_and_trace_noise() {
    let source = include_str!("../src/app.rs");
    assert!(source.contains(".level(log::LevelFilter::Info)"));
}
