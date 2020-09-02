//! Collections config.

use std::{io, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    collection::{CollectionName, Collections},
    config::load::{from_path, save_to_path, LoadError},
};

/// Collections config.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct CollectionsConfig {
    /// Default collection.
    ///
    /// Note that this could be non-existent collection name.
    /// If so, it should be treated in the same way as `None` (absense).
    // See <https://github.com/serde-rs/serde/issues/642> for the reason
    // the validation is not performed on this.
    default_collection: Option<CollectionName>,
    /// Collections.
    #[serde(rename = "collection")]
    #[serde(default)]
    #[serde(skip_serializing_if = "Collections::is_empty")]
    collections: Collections,
}

impl CollectionsConfig {
    /// Loads a config from a file at the given path.
    #[inline]
    pub(crate) fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LoadError> {
        from_path(path.as_ref())
    }

    /// Saves the config to the given path.
    pub(crate) fn save_to_path(&self, path: &Path) -> io::Result<()> {
        save_to_path(self, path)
    }

    /// Returns a reference to the collections.
    #[inline]
    #[must_use]
    pub fn collections(&self) -> &Collections {
        &self.collections
    }

    /// Returns a mutable reference to the collections.
    #[inline]
    #[must_use]
    pub fn collections_mut(&mut self) -> &mut Collections {
        &mut self.collections
    }

    /// Returns a default collection.
    #[inline]
    #[must_use]
    pub(super) fn default_collection(&self) -> Option<&CollectionName> {
        self.default_collection.as_ref()
    }

    /// Sets default collection to the given name.
    #[inline]
    pub(super) fn set_default_collection(&mut self, name: Option<CollectionName>) {
        self.default_collection = name;
    }
}
