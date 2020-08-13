//! Magro context.

use std::{
    borrow::Cow,
    fs, io,
    path::{Path, PathBuf},
};

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

/// Creates a `ProjectDirs` with the default parameters.
fn get_project_dirs() -> anyhow::Result<ProjectDirs> {
    ProjectDirs::from("org", "loliconduct", "magro")
        .context("Failed to get project directory")
        .map_err(Into::into)
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
        let project_dirs = get_project_dirs().map_err(Error::new)?;
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

    /// Returns the home directory.
    #[inline]
    #[must_use]
    pub(crate) fn home_dir(&self) -> &Path {
        self.user_dirs.home_dir()
    }
}

/// Saves a config to the given path.
fn save_config(path: &Path, conf: &Config) -> io::Result<()> {
    use serde::Serialize;

    let content = {
        let mut content = String::new();
        let mut ser = toml::Serializer::new(&mut content);
        ser.pretty_array(true);
        // This is expected to always success, because the config is valid and
        // the serialization does not perform I/O.
        conf.serialize(&mut ser)
            .expect("Failed to serialize the default config");
        content
    };
    fs::write(path, &content)
}

/// Creates a new default config file if not exist.
pub fn create_default_config_file_if_missing() -> Result<PathBuf, Error> {
    let project_dirs = get_project_dirs().map_err(Error::new)?;
    let conf_dir = project_dirs.config_dir();

    let config_path = conf_dir.join(DEFAULT_CONFIG_RELPATH);
    log::trace!("Default config file path is {:?}", config_path);

    if config_path.exists() {
        // A file already exists. Do nothing.
        // Note that it might not be a normal file: it can be a directory or
        // something else.
        log::trace!(
            "File {:?} already exists. Not creating the default config",
            config_path
        );
        return Ok(config_path);
    }

    if conf_dir.is_dir() {
        log::trace!("Creating app config directory {:?}", conf_dir);
        fs::DirBuilder::new()
            .recursive(true)
            .create(conf_dir)
            .with_context(|| anyhow!("Failed to create directory {}", conf_dir.display()))
            .map_err(Error::new)?;
    }

    let config = Config::default();
    save_config(&config_path, &config)
        .with_context(|| {
            anyhow!(
                "Failed to save the config file to {}",
                config_path.display()
            )
        })
        .map_err(Error::new)?;
    log::debug!("Created default config file to {}", config_path.display());

    Ok(config_path)
}
