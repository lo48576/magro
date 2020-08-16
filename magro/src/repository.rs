//! Repository.

use std::{convert::TryFrom, iter, mem, str};

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
// NOTE: Update `<VcsVariants as Iterator>::next()` and
// `<VcsVariants as ExactSizeIterator>::len()` when variants are changed.
// NOTE: Variants should be ordered alphabetically.
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

    /// Returns an iterator of VCS types.
    #[inline]
    #[must_use]
    pub fn variants() -> VcsVariants {
        VcsVariants {
            next: Some(Self::Git),
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

/// Iterator of variants of `Vcs` enum type.
#[derive(Debug, Clone)]
pub struct VcsVariants {
    /// Next variant.
    next: Option<Vcs>,
}

impl VcsVariants {
    /// Returns `next()` value without advancing the iterator.
    // No need of `&mut` for current implementation, but it is implementation detail.
    // Keep consistent with `std::iter::Peekable::peek()`.
    #[inline]
    #[must_use]
    pub fn peek(&mut self) -> Option<Vcs> {
        self.next
    }
}

impl Iterator for VcsVariants {
    type Item = Vcs;

    fn next(&mut self) -> Option<Self::Item> {
        let new_next = match self.next? {
            Vcs::Git => None,
        };
        mem::replace(&mut self.next, new_next)
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.len();
        (len, Some(len))
    }
}

impl ExactSizeIterator for VcsVariants {
    #[inline]
    fn len(&self) -> usize {
        1
    }
}

impl iter::FusedIterator for VcsVariants {}
