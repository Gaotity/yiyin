#![forbid(unsafe_code)]

//! Tauri inbound adapter and application composition root.

pub mod app;
pub mod commands;
pub mod dto;
#[cfg(feature = "e2e-fixture")]
mod e2e;
pub mod error;
pub mod events;
pub mod native;
pub mod protocol;
pub mod state;

pub use app::builder;
