//! Config load.

use std::{fs, io, path::Path};

use thiserror::Error as ThisError;

use crate::config::Config;

/// Config load error.
#[derive(Debug, ThisError)]
#[error("{}: {}", kind.as_str(), source)]
pub struct LoadError {
    /// Error kind.
    kind: LoadErrorKind,
    /// Error source.
    #[source]
    source: anyhow::Error,
}

impl LoadError {
    /// Creates a new decode error.
    #[inline]
    fn from_decode(e: impl Into<anyhow::Error>) -> Self {
        Self {
            kind: LoadErrorKind::Decode,
            source: e.into(),
        }
    }
}

impl From<io::Error> for LoadError {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self {
            kind: LoadErrorKind::Io,
            source: e.into(),
        }
    }
}

/// Error kind for `LoadError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
enum LoadErrorKind {
    /// Decode error.
    ///
    /// This may be caused by syntax error and semantic error.
    Decode,
    /// I/O error.
    Io,
}

impl LoadErrorKind {
    /// Returns a generic error message for the error kind.
    #[inline]
    #[must_use]
    fn as_str(&self) -> &'static str {
        match *self {
            Self::Decode => "Decode error",
            Self::Io => "I/O error",
        }
    }
}

/// Loads a config from a file at the given path.
pub(super) fn from_path(path: &Path) -> Result<Config, LoadError> {
    let content = fs::read_to_string(path)?;
    from_toml_str(&content)
}

/// Loads a config from the given toml string.
fn from_toml_str(content: &str) -> Result<Config, LoadError> {
    toml::from_str(content).map_err(LoadError::from_decode)
}
