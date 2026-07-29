fn command_body<'a>(source: &'a str, name: &str) -> &'a str {
    let signature = format!("pub async fn {name}");
    let start = source
        .find(&signature)
        .unwrap_or_else(|| panic!("{name} must remain an async command"));
    let remainder = &source[start..];
    let end = remainder[signature.len()..]
        .find("#[tauri::command]")
        .map_or(remainder.len(), |offset| signature.len() + offset);
    &remainder[..end]
}

#[test]
fn task_render_preparation_runs_outside_the_tauri_main_thread() {
    let source = include_str!("../src/commands/tasks.rs");
    for command in ["start_tasks", "preview_task"] {
        assert!(
            command_body(source, command).contains("run_blocking"),
            "{command} must move blocking resource inspection off the Tauri main thread"
        );
    }
}

#[test]
fn exif_reads_run_outside_the_tauri_main_thread() {
    let source = include_str!("../src/commands/resources.rs");
    assert!(
        command_body(source, "read_task_exif").contains("run_blocking"),
        "read_task_exif must move filesystem and EXIF reads off the Tauri main thread"
    );
}

#[test]
fn blocking_command_helper_uses_the_tauri_blocking_pool() {
    let source = include_str!("../src/commands/mod.rs");
    assert!(source.contains("tauri::async_runtime::spawn_blocking"));
}

#[test]
fn image_registration_runs_outside_the_tauri_main_thread() {
    let source = include_str!("../src/commands/resources.rs");
    assert!(
        command_body(source, "choose_images").contains("run_blocking"),
        "choose_images must move batch image registration off the Tauri main thread"
    );
}
