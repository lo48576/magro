//! Collection.
//!
//! Collection is conceptually a set of repositories, and is actually a directory.
//! Operations on repositories and policies for them can be applied separately
//! for each collection.

use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::Context;

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
    /// Path to the collection directory.
    ///
    /// If the path is relative, it is relative to the base directory.
    /// (Base directory is currently home directory.)
    ///
    /// If the path is absolute, use it as is.
    path: PathBuf,
}

impl Collection {
    /// Creates a new collection.
    #[inline]
    #[must_use]
    pub fn new(name: CollectionName, path: PathBuf) -> Self {
        Self { name, path }
    }

    /// Returns the collection name.
    #[inline]
    #[must_use]
    pub fn name(&self) -> &CollectionName {
        &self.name
    }

    /// Returns the absolute path of the collection.
    #[inline]
    #[must_use]
    pub fn abspath(&self, context: &Context) -> Cow<'_, Path> {
        if self.path.is_absolute() {
            return Cow::Borrowed(&self.path);
        }
        let base = context.home_dir();
        Cow::Owned(base.join(&self.path))
    }
}
