//! Magro config.

use std::{io, path::Path};

pub use self::{collection::CollectionsConfig, load::LoadError, main::MainConfig};
use crate::collection::CollectionName;

mod collection;
mod load;
mod main;

/// Default config file path relative to the config directory.
const DEFAULT_MAIN_CONFIG_RELPATH: &str = "config.toml";

/// Default collections config file path relative to the config directory.
const DEFAULT_COLLECTIONS_CONFIG_RELPATH: &str = "collections.toml";

/// Magro config.
#[derive(Debug, Clone)]
pub struct Config {
    /// Main config.
    main: MainConfig,
    /// Collections.
    collections: CollectionsConfig,
    /// Whether the collections config is (possibly) modified.
    collections_is_dirty: bool,
}

impl Config {
    /// Loads config from the given directory.
    pub(super) fn from_dir_path(conf_dir: &Path) -> Result<Self, LoadError> {
        let main = {
            let path = conf_dir.join(DEFAULT_MAIN_CONFIG_RELPATH);
            if path.is_file() {
                let conf = MainConfig::from_path(&path).map_err(|e| e.and_path(path.clone()))?;
                log::debug!("Loaded main config file {:?}", path);
                conf
            } else {
                log::debug!("Main config not found. Using default data");
                MainConfig::default()
            }
        };
        let (collections, collections_is_dirty) = {
            let path = conf_dir.join(DEFAULT_COLLECTIONS_CONFIG_RELPATH);
            if path.is_file() {
                let conf =
                    CollectionsConfig::from_path(&path).map_err(|e| e.and_path(path.clone()))?;
                log::debug!("Loaded collections config file {:?}", path);
                (conf, false)
            } else {
                log::debug!("Collections config not found. Using default data");
                (CollectionsConfig::default(), true)
            }
        };

        Ok(Self {
            main,
            collections,
            collections_is_dirty,
        })
    }

    /// Saves the configs if possibly modified.
    pub(super) fn save_if_dirty(&self, conf_dir: &Path) -> io::Result<()> {
        if self.collections_is_dirty {
            self.collections.save_to_path(conf_dir)?;
        }

        Ok(())
    }

    /// Saves the collections config.
    // TODO: This is temporary. Remove later.
    pub(super) fn save_collections_config(
        &self,
        conf_dir: &Path,
        conf: &CollectionsConfig,
    ) -> io::Result<()> {
        let newconf = Self {
            main: self.main.clone(),
            collections: conf.clone(),
            collections_is_dirty: true,
        };
        newconf.save_if_dirty(conf_dir)
    }

    /// Returns the main config.
    #[inline]
    pub(super) fn main(&self) -> &MainConfig {
        &self.main
    }

    /// Returns the collections config.
    #[inline]
    pub(super) fn collections(&self) -> &CollectionsConfig {
        &self.collections
    }

    /// Returns a default collection.
    #[inline]
    #[must_use]
    pub fn default_collection(&self) -> Option<&CollectionName> {
        self.collections.default_collection()
    }

    /// Sets default collection to the given name.
    #[inline]
    pub fn set_default_collection(&mut self, name: Option<CollectionName>) {
        self.collections_is_dirty = true;
        self.collections.set_default_collection(name);
    }
}
