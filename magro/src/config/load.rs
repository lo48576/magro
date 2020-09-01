//! Config load.

use std::{fs, io, path::Path};

use thiserror::Error as ThisError;

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
    pub(super) fn from_decode(e: impl Into<anyhow::Error>) -> Self {
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

/// Loads a data from a file at the given path.
pub(super) fn from_path<T>(path: &Path) -> Result<T, LoadError>
where
    for<'a> T: serde::Deserialize<'a>,
{
    let content = fs::read_to_string(path)?;
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
    fs::write(path, &content)
}
