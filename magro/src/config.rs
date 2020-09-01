//! Magro config.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::collection::{CollectionName, Collections};

pub use self::{collection::CollectionsConfig, load::LoadError};

mod collection;
mod load;

/// Magro config.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub struct Config {
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

impl Config {
    /// Loads a config from a file at the given path.
    #[inline]
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self, LoadError> {
        load::from_path(path.as_ref())
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
    pub fn default_collection(&self) -> Option<&CollectionName> {
        self.default_collection.as_ref()
    }

    /// Sets default collection to the given name.
    #[inline]
    pub fn set_default_collection(&mut self, name: Option<CollectionName>) {
        self.default_collection = name;
    }
}
