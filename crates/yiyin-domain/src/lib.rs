#![forbid(unsafe_code)]

//! Pure product rules and render planning for Yiyin.

mod config;
mod error;
mod metadata;
mod resource;
mod template;

pub use config::*;
pub use error::DomainError;
pub use metadata::*;
pub use resource::*;
pub use template::*;
