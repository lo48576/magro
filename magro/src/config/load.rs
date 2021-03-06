//! Config load.

use std::{
    io,
    path::{Path, PathBuf},
};

use thiserror::Error as ThisError;

use crate::lock_fs;

/// Config load error.
#[derive(Debug, ThisError)]
#[error("{} (at file {:?}): {}", kind.as_str(), path, source)]
pub struct LoadError {
    /// Error kind.
    kind: LoadErrorKind,
    /// Filename.
    path: Option<PathBuf>,
    /// Error source.
    #[source]
    source: anyhow::Error,
}

impl LoadError {
    /// Creates a new decode error.
    #[inline]
    pub(super) fn from_decode(e: impl Into<anyhow::Error>) -> Self {
        Self {
            kind: LoadErrorKind::Decode,
            path: None,
            source: e.into(),
        }
    }

    /// Returns a new error with the given path.
    #[inline]
    pub(super) fn and_path(self, path: impl Into<PathBuf>) -> Self {
        Self {
            path: Some(path.into()),
            ..self
        }
    }
}

impl From<io::Error> for LoadError {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self {
            kind: LoadErrorKind::Io,
            path: None,
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

/// Loads a data from a file at the given path.
pub(super) fn from_path<T>(path: &Path) -> Result<T, LoadError>
where
    for<'a> T: serde::Deserialize<'a>,
{
    let content = lock_fs::read_to_string(path)?;
    toml::from_str::<T>(&content).map_err(LoadError::from_decode)
}

/// Saves the given data to a file at the given path.
pub(super) fn save_to_path<T>(value: T, path: &Path) -> io::Result<()>
where
    T: serde::Serialize,
{
    let content = {
        let mut content = String::new();
        let mut ser = toml::Serializer::new(&mut content);
        ser.pretty_array(true);
        // This is expected to always success, because the config is valid and
        // the serialization itself does not perform I/O.
        value
            .serialize(&mut ser)
            .expect("Valid data should be serializable");
        content
    };
    lock_fs::write(path, &content)
}
