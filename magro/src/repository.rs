//! Repository.

use serde::{Deserialize, Serialize};

/// VCS type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(rename_all = "kebab-case")]
pub enum Vcs {
    /// Git.
    Git,
}

impl Vcs {
    /// Returns the VCS name in lower case.
    pub fn name_lower(&self) -> &'static str {
        match self {
            Self::Git => "git",
        }
    }
}
