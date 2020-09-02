//! Main config.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::config::load::{from_path, LoadError};

/// Main config.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct MainConfig {}

impl MainConfig {
    /// Loads a config from a file at the given path.
    #[inline]
    pub(crate) fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LoadError> {
        from_path(path.as_ref())
    }
}
