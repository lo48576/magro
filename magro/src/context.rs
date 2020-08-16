//! Magro context.

use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context as _};
use directories::{ProjectDirs, UserDirs};
use once_cell::sync::OnceCell;
use thiserror::Error as ThisError;

use crate::{cache::Cache, Config};

/// Default config file path relative to the config directory.
const DEFAULT_CONFIG_RELPATH: &str = "config.toml";

/// Default cache file path relative to the cache directory.
const DEFAULT_CACHE_RELPATH: &str = "cache.toml";

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
#[derive(Debug)]
pub struct Context {
    /// User directories.
    user_dirs: UserDirs,
    /// Project directories.
    project_dirs: ProjectDirs,
    /// Config file path.
    config_path: PathBuf,
    /// Cache file path.
    cache_path: PathBuf,
    /// Config.
    config: Config,
    /// Lazily loaded cache.
    cache: OnceCell<Cache>,
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
        let config_path =
            config_path.map_or_else(|| conf_dir.join(DEFAULT_CONFIG_RELPATH), ToOwned::to_owned);
        let config = Config::from_path(&config_path)
            .with_context(|| anyhow!("Failed to load the config file {}", config_path.display()))
            .map_err(Error::new)?;
        log::debug!(
            "Loaded config file {:?}",
            AsRef::<Path>::as_ref(&config_path)
        );

        let cache_dir = project_dirs.cache_dir();
        // TODO: How to decide cache file path corresponding to config path?
        let cache_path = cache_dir.join(DEFAULT_CACHE_RELPATH);

        Ok(Self {
            user_dirs,
            config_path,
            cache_path,
            project_dirs,
            config,
            cache: OnceCell::new(),
        })
    }

    /// Returns the home directory.
    #[inline]
    #[must_use]
    pub fn home_dir(&self) -> &Path {
        self.user_dirs.home_dir()
    }

    /// Returns the currently used config path.
    #[inline]
    #[must_use]
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Returns the currently used cache file path.
    #[inline]
    #[must_use]
    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }

    /// Returns the config.
    #[inline]
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Saves the given config.
    #[inline]
    pub fn save_config(&self, config: &Config) -> io::Result<()> {
        save_config(&self.config_path, config)
    }

    /// Loads the cache if necessary, and returns the cache.
    #[inline]
    pub fn get_or_load_cache(&self) -> io::Result<&Cache> {
        let cache_path = self.cache_path();
        self.cache.get_or_try_init(|| Cache::from_path(cache_path))
    }

    /// Returns the cache if it is already loaded.
    #[inline]
    pub fn get_cache(&self) -> Option<&Cache> {
        self.cache.get()
    }

    /// Saves the given cache.
    #[inline]
    pub fn save_cache(&self, cache: &Cache) -> io::Result<()> {
        save_cache(&self.cache_path, cache)
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
            .expect("Default config data should be serializable");
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

/// Saves a cache to the given path.
fn save_cache(path: &Path, cache: &Cache) -> io::Result<()> {
    use serde::Serialize;

    let content = {
        let mut content = String::new();
        let mut ser = toml::Serializer::new(&mut content);
        //ser.pretty_array(true);
        // This is expected to always success, because the config is valid and
        // the serialization does not perform I/O.
        cache
            .serialize(&mut ser)
            .expect("Default cache data should be serializable");
        content
    };
    let cache_dir = path.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::Other,
            format!("Attempt to save cache file to invalid path {:?}", path),
        )
    })?;
    if !cache_dir.is_dir() {
        log::trace!(
            "Creating a directory {:?} for to save cache file",
            cache_dir
        );
        fs::DirBuilder::new().recursive(true).create(cache_dir)?;
    }
    fs::write(path, &content)
}
