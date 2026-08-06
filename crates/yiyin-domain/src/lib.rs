#![forbid(unsafe_code)]

//! Pure product rules and render planning for Yiyin.

mod config;
mod error;
mod metadata;
mod metadata_display;
mod render;
mod resource;
mod task;
mod template;

pub use config::*;
pub use error::DomainError;
pub use metadata::*;
pub use metadata_display::*;
pub use render::*;
pub use resource::*;
pub use task::*;
pub use template::*;
