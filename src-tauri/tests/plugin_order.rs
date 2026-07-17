use yiyin_desktop::app::PLUGIN_ORDER;

#[test]
fn single_instance_is_first_and_restores_the_existing_main_window() {
    assert_eq!(
        PLUGIN_ORDER,
        [
            "single-instance",
            "log",
            "dialog",
            "opener",
            "yiyin-protocol"
        ]
    );
    let source = include_str!("../src/app.rs");
    let single_instance = source
        .find("single_instance_plugin()")
        .expect("single instance");
    let log = source.find("tauri_plugin_log").expect("log plugin");
    let dialog = source.find("tauri_plugin_dialog").expect("dialog plugin");
    let opener = source.find("tauri_plugin_opener").expect("opener plugin");
    assert!(single_instance < log && log < dialog && dialog < opener);
    for focus_step in [
        "get_webview_window(\"main\")",
        ".unminimize()",
        ".show()",
        ".set_focus()",
    ] {
        assert!(source.contains(focus_step), "missing {focus_step}");
    }
}
