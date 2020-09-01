//! Magro config.

use std::path::Path;

use serde::{Deserialize, Serialize};

pub use self::{collection::CollectionsConfig, load::LoadError};

mod collection;
mod load;

/// Magro config.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Config {}

impl Config {
    /// Loads a config from a file at the given path.
    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LoadError> {
        load::from_path(path.as_ref())
    }
}
