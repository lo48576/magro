//! Collections state caches.

use std::{
    cmp,
    collections::{BTreeMap, BTreeSet},
    fs, io, iter,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::{collection::CollectionName, discovery::RepoEntry, vcs::Vcs};

/// Global cache data.
///
/// This type corresponds to data in a cache file.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Cache {
    /// Repositories for collections.
    // Use `BTreeMap` here to keep things sorted.
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    collections: BTreeMap<String, CollectionReposCache>,
}

impl Cache {
    /// Loads a cache from the given path.
    #[inline]
    pub(crate) fn from_path<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        Self::from_path_impl(path.as_ref())
    }

    /// Monomorphized internal implementation of `from_path()`.
    #[inline]
    fn from_path_impl(path: &Path) -> io::Result<Self> {
        let content = match fs::read_to_string(path) {
            Ok(v) => v,
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => return Ok(Self::default()),
                _ => return Err(e),
            },
        };
        match toml::from_str(&content) {
            Ok(v) => Ok(v),
            Err(e) => {
                log::error!("Cache will be reset due to invalid data: {}", e);
                Ok(Self::default())
            }
        }
    }

    /// Returns the collection cache.
    #[inline]
    #[must_use]
    pub fn collection_repos(&self, name: &CollectionName) -> Option<&CollectionReposCache> {
        self.collections.get(name.as_str())
    }

    /// Sets the given collection cache.
    #[inline]
    pub fn cache_collection_repos(
        &mut self,
        name: CollectionName,
        coll_cache: CollectionReposCache,
    ) -> Option<CollectionReposCache> {
        self.collections.insert(name.into(), coll_cache)
    }

    /// Removes the collection cache.
    #[inline]
    pub fn remove_collection_repos_cache(&mut self, name: &str) -> Option<CollectionReposCache> {
        self.collections.remove(name)
    }
}

/// Cache of repositories in a collection.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CollectionReposCache {
    /// Repository (more precisely, git directory) paths.
    // Use `BTreeSet` here to keep things sorted.
    #[serde(default)]
    #[serde(skip_serializing_if = "BTreeSet::is_empty")]
    repos: BTreeSet<RepoCacheEntryWrapper>,
}

impl Extend<RepoCacheEntry> for CollectionReposCache {
    #[inline]
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = RepoCacheEntry>,
    {
        self.repos
            .extend(iter.into_iter().map(RepoCacheEntryWrapper))
    }
}

impl CollectionReposCache {
    /// Returns a sorted iterator of repository cache entries.
    #[inline]
    #[must_use]
    pub fn repositories(&self) -> CollectionRepoCacheIter<'_> {
        CollectionRepoCacheIter::new(self)
    }
}

/// A sorted iterator of repository cache entries.
#[derive(Debug, Clone)]
pub struct CollectionRepoCacheIter<'a> {
    /// Inner iterator.
    inner: std::collections::btree_set::Iter<'a, RepoCacheEntryWrapper>,
}

impl<'a> CollectionRepoCacheIter<'a> {
    /// Creates a new iterator.
    #[inline]
    #[must_use]
    fn new(cache: &'a CollectionReposCache) -> Self {
        Self {
            inner: cache.repos.iter(),
        }
    }
}

impl<'a> Iterator for CollectionRepoCacheIter<'a> {
    type Item = &'a RepoCacheEntry;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|wrapper| &wrapper.0)
    }
}

impl iter::FusedIterator for CollectionRepoCacheIter<'_> {}

/// A wrapper to compare `RepoCacheEntry` using only path.
#[derive(Debug, Clone, Eq, Serialize, Deserialize)]
#[serde(transparent)]
struct RepoCacheEntryWrapper(RepoCacheEntry);

impl PartialEq for RepoCacheEntryWrapper {
    #[inline]
    fn eq(&self, other: &RepoCacheEntryWrapper) -> bool {
        self.0.path == other.0.path
    }
}

impl PartialOrd for RepoCacheEntryWrapper {
    #[inline]
    fn partial_cmp(&self, other: &RepoCacheEntryWrapper) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RepoCacheEntryWrapper {
    #[inline]
    fn cmp(&self, other: &RepoCacheEntryWrapper) -> cmp::Ordering {
        self.0.path.cmp(&other.0.path)
    }
}

/// A cache entry for a repository.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepoCacheEntry {
    /// Path.
    ///
    /// For git, `.git` directory or `*.git` directory.
    path: PathBuf,
    /// VCS type.
    vcs: Vcs,
}

impl RepoCacheEntry {
    /// Creates a new `RepoCacheEntry`.
    #[inline]
    #[must_use]
    pub fn new<P: Into<PathBuf>>(vcs: Vcs, path: P) -> Self {
        Self {
            vcs,
            path: path.into(),
        }
    }

    /// Returns the VCS type.
    #[inline]
    #[must_use]
    pub fn vcs(&self) -> Vcs {
        self.vcs
    }

    /// Returns the repository path.
    #[inline]
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Returns the owned repository path.
    #[inline]
    #[must_use]
    pub fn into_path(self) -> PathBuf {
        self.path
    }

    /// Applies the given function to the path.
    #[inline]
    pub fn try_map_ref_path<F, E>(&self, f: F) -> Result<Self, E>
    where
        for<'a> F: FnOnce(&'a Path) -> Result<PathBuf, E>,
    {
        let path = f(&self.path)?;
        Ok(Self {
            vcs: self.vcs,
            path,
        })
    }
}

impl From<RepoEntry> for RepoCacheEntry {
    #[inline]
    fn from(v: RepoEntry) -> Self {
        let vcs = v.vcs();
        let path = v.into_path();

        Self { vcs, path }
    }
}
