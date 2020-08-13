//! Magro context.

use anyhow::Context as _;
use directories::{ProjectDirs, UserDirs};
use thiserror::Error as ThisError;

/// Context error.
#[derive(Debug, ThisError)]
#[error(transparent)]
pub struct Error(anyhow::Error);

impl Error {
    /// Wraps the given error.
    #[inline]
    #[must_use]
    fn new(e: impl Into<anyhow::Error>) -> Self {
        Self(e.into())
    }
}

/// Magro context.
///
/// Context is a bundle of config and cached information.
#[derive(Debug, Clone)]
pub struct Context {
    /// User directories.
    user_dirs: UserDirs,
    /// Project directories.
    project_dirs: ProjectDirs,
}

impl Context {
    /// Creates a new context.
    #[inline]
    pub fn new() -> Result<Self, Error> {
        let user_dirs = UserDirs::new()
            .context("Failed to get user directory")
            .map_err(Error::new)?;
        log::debug!("Home directory: {:?}", user_dirs.home_dir());
        let project_dirs = ProjectDirs::from("org", "loliconduct", "magro")
            .context("Failed to get project directory")
            .map_err(Error::new)?;
        log::debug!("Config directory: {:?}", project_dirs.config_dir());

        Ok(Self {
            user_dirs,
            project_dirs,
        })
    }
}
