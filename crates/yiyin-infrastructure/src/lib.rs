#![forbid(unsafe_code)]

//! Outbound adapters for persistence, resources, metadata, rendering, and tasks.

mod config;
#[doc(hidden)]
pub mod durable;
mod filesystem;
mod metadata;
mod rendering;
mod resources;
mod tasks;

pub use config::*;
pub use filesystem::*;
pub use metadata::*;
pub use rendering::*;
pub use resources::*;
pub use tasks::*;
pub use yiyin_application as application;
pub use yiyin_domain as domain;
