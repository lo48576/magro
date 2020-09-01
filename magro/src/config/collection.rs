//! Collections config.

use std::{fs, io, path::Path};

use serde::{Deserialize, Serialize};

use crate::{
    collection::{CollectionName, Collections},
    config::load::LoadError,
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
        let content = fs::read_to_string(path)?;
        toml::from_str(&content).map_err(LoadError::from_decode)
    }

    /// Saves the config to the given path.
    pub(crate) fn save_to_path(&self, path: &Path) -> io::Result<()> {
        let content = {
            let mut content = String::new();
            let mut ser = toml::Serializer::new(&mut content);
            ser.pretty_array(true);
            // This is expected to always success, because the config is valid and
            // the serialization itself does not perform I/O.
            self.serialize(&mut ser)
                .expect("Default config data should be serializable");
            content
        };
        fs::write(path, &content)
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
