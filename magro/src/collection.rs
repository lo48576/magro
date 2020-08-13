//! Collection.
//!
//! Collection is conceptually a set of repositories, and is actually a directory.
//! Operations on repositories and policies for them can be applied separately
//! for each collection.

use serde::{Deserialize, Serialize};

pub use self::{
    collections::Collections,
    name::{CollectionName, CollectionNameError},
};

pub mod collections;
mod name;

/// Repositories collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Collection {
    /// Collection name.
    name: CollectionName,
}

impl Collection {
    /// Returns the collection name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &CollectionName {
        &self.name
    }
}
