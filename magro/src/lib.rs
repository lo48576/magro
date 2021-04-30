//! magro: Manage git repositories.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(clippy::missing_docs_in_private_items)]

pub use self::{config::Config, context::Context};

pub mod cache;
pub mod collection;
pub mod config;
pub mod context;
pub mod discovery;
mod lock_fs;
pub mod vcs;
