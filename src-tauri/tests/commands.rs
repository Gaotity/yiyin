#[test]
fn composition_root_registers_the_complete_dedicated_command_surface() {
    let source = include_str!("../src/app.rs");
    for command in [
        "commands::bootstrap::bootstrap",
        "commands::config::update_config",
        "commands::config::reset_config",
        "commands::native::choose_output_directory",
        "commands::native::open_output_directory",
        "commands::resources::choose_images",
        "commands::resources::register_font",
        "commands::resources::remove_font",
        "commands::resources::register_overlay",
        "commands::resources::read_task_exif",
        "commands::tasks::start_tasks",
        "commands::tasks::preview_task",
        "commands::tasks::cancel_task",
        "commands::tasks::clear_tasks",
        "commands::window::minimize_window",
        "commands::window::close_window",
        "commands::native::open_external_url",
    ] {
        assert!(source.contains(command), "missing command {command}");
    }
}
