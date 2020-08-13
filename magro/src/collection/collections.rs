//! Set of collections.

use std::{collections::BTreeMap, iter};

use crate::collection::Collection;

/// Set of collections.
#[derive(Default, Debug, Clone)]
pub struct Collections {
    /// Collections.
    // Use `String` as keys to make it easier to query by any string (even if
    // the query string is invalid name).
    collections: BTreeMap<String, Collection>,
}

impl Collections {
    /// Returns the collection with the given name, if available.
    #[inline]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&Collection> {
        self.collections.get(name)
    }

    /// Adds the given collection to this `Collections`, and returns the old entry if exists.
    #[inline]
    pub fn insert(&mut self, collection: Collection) -> Option<Collection> {
        let name = collection.name().as_str().to_owned();
        self.collections.insert(name, collection)
    }

    /// Returns the number of the collections.
    #[inline]
    #[must_use]
    pub fn len(&self) -> usize {
        self.collections.len()
    }

    /// Returns `true` if there are no collections.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.collections.is_empty()
    }
}

impl<'a> IntoIterator for &'a Collections {
    type IntoIter = Iter<'a>;
    type Item = &'a Collection;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            inner: self.collections.values(),
        }
    }
}

/// An iterator over the collections.
#[derive(Debug, Clone)]
pub struct Iter<'a> {
    /// Inner iterator.
    // NOTE: If you want to use another type as an inner iterator,
    // check whether `{DoubleEnded,ExactSize,Fused}Iterator` is implemented.
    inner: std::collections::btree_map::Values<'a, String, Collection>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Collection;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a> DoubleEndedIterator for Iter<'a> {
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

impl<'a> ExactSizeIterator for Iter<'a> {
    #[inline]
    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<'a> iter::FusedIterator for Iter<'a> {}
