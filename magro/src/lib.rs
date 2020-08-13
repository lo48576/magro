//! magro: Manage git repositories.
#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(clippy::missing_docs_in_private_items)]

pub use context::Context;

pub mod collection;
pub mod context;
