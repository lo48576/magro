//! Magro context.

use std::{
    borrow::Cow,
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use directories::{ProjectDirs, UserDirs};
use once_cell::sync::OnceCell;
use thiserror::Error as ThisError;

use crate::{cache::Cache, config::Config, lock_fs};

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
    /// Config directory path.
    config_dir: PathBuf,
    /// Config.
    config: Config,
    /// Cache file path.
    cache_path: PathBuf,
    /// Lazily loaded cache.
    cache: OnceCell<Cache>,
}

impl Context {
    /// Creates a new context with default config path.
    #[inline]
    pub fn new() -> Result<Self, Error> {
        let user_dirs = UserDirs::new()
            .context("Failed to get user directory")
            .map_err(Error::new)?;
        log::debug!("Home directory: {:?}", user_dirs.home_dir());
        let project_dirs = get_project_dirs().map_err(Error::new)?;
        log::debug!("Config directory: {:?}", project_dirs.config_dir());

        let config_dir = project_dirs.config_dir().to_owned();
        let config = Config::from_dir_path(&config_dir)
            .context("Failed to load config")
            .map_err(Error::new)?;

        let cache_dir = project_dirs.cache_dir();
        let cache_path = cache_dir.join(DEFAULT_CACHE_RELPATH);

        Ok(Self {
            user_dirs,
            config_dir,
            config,
            cache_path,
            project_dirs,
            cache: OnceCell::new(),
        })
    }

    /// Returns the home directory.
    #[inline]
    #[must_use]
    pub fn home_dir(&self) -> &Path {
        self.user_dirs.home_dir()
    }

    /// Returns a reference to the config.
    #[inline]
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Returns a mutable reference to the config.
    #[inline]
    #[must_use]
    pub fn config_mut(&mut self) -> &mut Config {
        &mut self.config
    }

    /// Saves the config if (possibly) dirty.
    #[inline]
    pub fn save_config_if_dirty(&mut self) -> io::Result<()> {
        self.config.save_if_dirty(&self.config_dir)
    }

    /// Loads the cache if necessary, and returns the cache.
    #[inline]
    pub fn get_or_load_cache(&self) -> io::Result<&Cache> {
        self.cache
            .get_or_try_init(|| Cache::from_path(&self.cache_path))
    }

    /// Loads the cache if necessary, and returns the cache.
    #[inline]
    pub fn get_or_load_cache_mut(&mut self) -> io::Result<&mut Cache> {
        match self
            .cache
            .get_or_try_init(|| Cache::from_path(&self.cache_path))
        {
            Ok(_) => Ok(self
                .cache
                .get_mut()
                .expect("Should never happen because successfully initialized")),
            Err(e) => Err(e),
        }
    }

    /// Returns the cache if it is already loaded.
    #[inline]
    pub fn get_cache(&self) -> Option<&Cache> {
        self.cache.get()
    }

    /// Saves the given cache.
    #[inline]
    pub fn save_cache(&self) -> io::Result<()> {
        let cache = self
            .get_or_load_cache()
            .map_or_else(|_| Cow::Owned(Default::default()), Cow::Borrowed);
        save_cache(&self.cache_path, &cache)
    }
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
    lock_fs::write(path, &content)
}
