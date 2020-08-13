//! Collection.
//!
//! Collection is conceptually a set of repositories, and is actually a directory.
//! Operations on repositories and policies for them can be applied separately
//! for each collection.

pub use self::name::{CollectionName, CollectionNameError};

mod name;

/// Repositories collection.
#[derive(Debug, Clone)]
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
