//! Collection.
//!
//! Collection is conceptually a set of repositories, and is actually a directory.
//! Operations on repositories and policies for them can be applied separately
//! for each collection.

pub use self::name::{CollectionName, CollectionNameError};

mod name;
