//! Repository.

use std::{convert::TryFrom, str};

use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

/// VCS parse error.
#[derive(Debug, Clone, PartialEq, Eq, ThisError)]
#[error("Failed to parse VCS name")]
pub struct VcsParseError(());

impl VcsParseError {
    /// Creates a new `VcsParseError`.
    #[inline]
    #[must_use]
    fn new() -> Self {
        Self(())
    }
}

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
    ///
    /// # Examples
    ///
    /// ```
    /// # use magro::repository::Vcs;
    /// assert_eq!(Vcs::Git.name_lower(), "git");
    /// ```
    pub fn name_lower(&self) -> &'static str {
        match self {
            Self::Git => "git",
        }
    }

    /// Parses the VCS name in lower case.
    ///
    /// # Examples
    ///
    /// ```
    /// # use magro::repository::Vcs;
    /// assert_eq!(Vcs::try_from_name_lower("git"), Ok(Vcs::Git));
    ///
    /// assert!(Vcs::try_from_name_lower("Git").is_err());
    /// assert!(Vcs::try_from_name_lower("no-such-vcs").is_err());
    /// ```
    pub fn try_from_name_lower(s: &str) -> Result<Self, VcsParseError> {
        match s {
            "git" => Ok(Self::Git),
            _ => Err(VcsParseError::new()),
        }
    }
}

impl str::FromStr for Vcs {
    type Err = VcsParseError;

    /// Parses the VCS name in lower case.
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_from_name_lower(s)
    }
}

impl TryFrom<&str> for Vcs {
    type Error = VcsParseError;

    /// Parses the VCS name in lower case.
    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Self::try_from_name_lower(s)
    }
}
