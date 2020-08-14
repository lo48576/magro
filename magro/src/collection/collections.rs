//! Set of collections.

use std::{collections::BTreeMap, fmt, iter};

use serde::{Deserialize, Serialize};

use crate::collection::Collection;

/// Set of collections.
// Note that this is serialized / deserialized as an array, rather than a map.
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

    /// Removes a collection with the given name and returns it, if exists.
    #[inline]
    pub fn remove(&mut self, name: &str) -> Option<Collection> {
        self.collections.remove(name)
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

    /// Returns an iterator of the collections.
    #[inline]
    #[must_use]
    pub fn iter(&self) -> Iter<'_> {
        self.into_iter()
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

impl<'de> Deserialize<'de> for Collections {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        /// Visitor type for `Collections`.
        #[derive(Default)]
        struct CollectionsVisitor(BTreeMap<String, Collection>);

        impl<'de> serde::de::Visitor<'de> for CollectionsVisitor {
            type Value = Collections;

            #[inline]
            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("array of collections")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                use serde::de;

                let mut map = BTreeMap::new();
                while let Some(collection) = seq.next_element::<Collection>()? {
                    let name = collection.name().as_str().to_owned();
                    let dup = map.insert(name, collection);
                    if let Some(dup) = dup {
                        return Err(de::Error::custom(format!(
                            "collections with duplicate name {:?}",
                            dup.name.as_str()
                        )));
                    }
                }

                Ok(Collections { collections: map })
            }
        }

        deserializer.deserialize_seq(CollectionsVisitor::default())
    }
}

impl Serialize for Collections {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeSeq;

        let mut seq = serializer.serialize_seq(Some(self.collections.len()))?;
        for collection in self.collections.values() {
            seq.serialize_element(collection)?;
        }
        seq.end()
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
