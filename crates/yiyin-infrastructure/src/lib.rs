#![forbid(unsafe_code)]

//! Outbound adapters for persistence, resources, metadata, rendering, and tasks.

mod config;
mod filesystem;
mod metadata;
mod resources;

pub use config::*;
pub use filesystem::*;
pub use metadata::*;
pub use resources::*;
pub use yiyin_application as application;
pub use yiyin_domain as domain;
