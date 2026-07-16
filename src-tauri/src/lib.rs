#![forbid(unsafe_code)]

//! Tauri inbound adapter and application composition root.

/// Creates the Tauri application builder used by the desktop entry point.
#[must_use]
pub fn builder() -> tauri::Builder<tauri::Wry> {
    tauri::Builder::default()
}
