#![forbid(unsafe_code)]

//! Outbound adapters for persistence, resources, metadata, rendering, and tasks.

mod config;
mod filesystem;

pub use config::*;
pub use filesystem::*;
pub use yiyin_application as application;
pub use yiyin_domain as domain;
