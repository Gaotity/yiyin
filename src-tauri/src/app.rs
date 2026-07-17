use tauri::{DragDropEvent, Manager, RunEvent, Wry};

use crate::{commands, protocol::ResourceProtocol, state::AppState};

pub const PLUGIN_ORDER: [&str; 5] = [
    "single-instance",
    "log",
    "dialog",
    "opener",
    "yiyin-protocol",
];

fn single_instance_plugin() -> tauri::plugin::TauriPlugin<Wry> {
    tauri_plugin_single_instance::init(|app, _arguments, _working_directory| {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.unminimize();
            let _ = window.show();
            let _ = window.set_focus();
        }
    })
}

/// Creates the desktop builder with native plugins in the security-reviewed order.
#[must_use]
pub fn builder() -> tauri::Builder<Wry> {
    tauri::Builder::default()
        .plugin(single_instance_plugin())
        .plugin(tauri_plugin_log::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_opener::Builder::new()
                .open_js_links_on_click(false)
                .build(),
        )
        .register_uri_scheme_protocol("yiyin", |context, request| {
            let state = context.app_handle().state::<AppState>();
            ResourceProtocol::new(state.resources.clone())
                .respond(&request.uri().to_string())
                .into_http()
        })
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap::bootstrap,
            commands::config::update_config,
            commands::config::reset_config,
            commands::native::choose_output_directory,
            commands::native::open_output_directory,
            commands::resources::choose_images,
            commands::resources::register_font,
            commands::resources::remove_font,
            commands::resources::register_overlay,
            commands::resources::read_task_exif,
            commands::tasks::start_tasks,
            commands::tasks::preview_task,
            commands::tasks::cancel_task,
            commands::tasks::clear_tasks,
            commands::window::minimize_window,
            commands::window::close_window,
            commands::native::open_external_url,
        ])
        .setup(|app| {
            let state = AppState::compose(app.handle())?;
            app.manage(state);
            if let Some(window) = app.get_webview_window("main") {
                let handle = app.handle().clone();
                window.on_window_event(move |event| {
                    if let tauri::WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = event {
                        let handle = handle.clone();
                        let paths = paths.clone();
                        tauri::async_runtime::spawn_blocking(move || {
                            let state = handle.state::<AppState>();
                            if let Err(error) =
                                commands::resources::register_image_paths(&state, &paths)
                            {
                                log::warn!("drag-and-drop registration failed: {}", error.message);
                            }
                        });
                    }
                });
            }
            Ok(())
        })
}

/// Builds and runs the desktop application.
///
/// # Panics
///
/// Panics when the reviewed Tauri configuration cannot be initialized.
pub fn run() {
    builder()
        .build(tauri::generate_context!())
        .expect("failed to build Yiyin")
        .run(|app, event| {
            if matches!(event, RunEvent::Exit | RunEvent::ExitRequested { .. })
                && let Err(error) = app.state::<AppState>().tasks.shutdown()
            {
                log::error!("task shutdown failed: {error}");
            }
        });
}
