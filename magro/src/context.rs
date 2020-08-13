//! Magro context.

use std::{borrow::Cow, path::Path};

use anyhow::{anyhow, Context as _};
use directories::{ProjectDirs, UserDirs};
use thiserror::Error as ThisError;

use crate::Config;

/// Default config file path relative to the config directory.
const DEFAULT_CONFIG_RELPATH: &str = "config.toml";

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
    /// Config.
    config: Config,
}

impl Context {
    /// Creates a new context with default config path.
    #[inline]
    pub fn new(config_path: Option<&Path>) -> Result<Self, Error> {
        let user_dirs = UserDirs::new()
            .context("Failed to get user directory")
            .map_err(Error::new)?;
        log::debug!("Home directory: {:?}", user_dirs.home_dir());
        let project_dirs = ProjectDirs::from("org", "loliconduct", "magro")
            .context("Failed to get project directory")
            .map_err(Error::new)?;
        log::debug!("Config directory: {:?}", project_dirs.config_dir());

        let conf_dir = project_dirs.config_dir();
        let config_path = config_path.map_or_else(
            || Cow::Owned(conf_dir.join(DEFAULT_CONFIG_RELPATH)),
            Cow::Borrowed,
        );
        let config = Config::from_path(&config_path)
            .with_context(|| anyhow!("Failed to load the config file {}", config_path.display()))
            .map_err(Error::new)?;
        log::debug!(
            "Loaded config file {:?}",
            AsRef::<Path>::as_ref(&config_path)
        );

        Ok(Self {
            user_dirs,
            project_dirs,
            config,
        })
    }
}
