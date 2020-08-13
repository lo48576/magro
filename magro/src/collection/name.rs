//! Collection name.

use std::{borrow, convert::TryFrom, fmt, str};

use thiserror::Error as ThisError;

/// Collection name error.
#[derive(Debug, Clone, ThisError)]
#[error("Invalid collection name: {message}")]
pub struct CollectionNameError {
    /// Message.
    message: String,
}

impl CollectionNameError {
    /// Creates a new error with the given message.
    #[inline]
    #[must_use]
    fn with_message(s: impl fmt::Display) -> Self {
        Self {
            message: s.to_string(),
        }
    }
}

/// Collection name.
///
/// Collection name should satisfy all restrictions below:
///
/// * Should not be empty.
/// * Should consist of ASCII alphanumeric, ASCII hyphen, or ASCII underscore.
/// * Should not start with an ASCII hyphen.
///
/// # Examples
///
/// ```
/// use std::convert::TryFrom;
/// # use magro::collection::CollectionName;
///
/// assert_eq!(CollectionName::try_from("hello").unwrap(), "hello");
/// assert_eq!(
///     CollectionName::try_from("hello-world").unwrap(),
///     "hello-world"
/// );
/// assert_eq!(CollectionName::try_from("_hello").unwrap(), "_hello");
/// assert_eq!(CollectionName::try_from("1234").unwrap(), "1234");
///
/// assert!(CollectionName::try_from("foo bar").is_err());
/// assert!(CollectionName::try_from("-foo").is_err());
/// assert!(CollectionName::try_from("foo/bar").is_err());
/// assert!(CollectionName::try_from("").is_err());
/// // U+03B1: Greek Small Letter Alpha.
/// assert!(CollectionName::try_from("\u{03B1}").is_err());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CollectionName(String);

impl CollectionName {
    /// Returns the string slice for the collection name.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.as_ref()
    }
}

impl CollectionName {
    /// Validates the given string as a collection name.
    fn validate(s: &str) -> Result<(), CollectionNameError> {
        if s.is_empty() {
            return Err(CollectionNameError::with_message("Empty colection name"));
        }

        if s.as_bytes()[0] == b'-' {
            return Err(CollectionNameError::with_message(
                "Collection name starts with '-'",
            ));
        }

        if let Some(c) = s
            .chars()
            .find(|&c| !(c.is_ascii_alphanumeric() || c == '_' || c == '-'))
        {
            return Err(CollectionNameError::with_message(format!(
                "Invalid character {:?}",
                c
            )));
        }

        Ok(())
    }
}

impl PartialEq<&'_ str> for CollectionName {
    #[inline]
    fn eq(&self, other: &&str) -> bool {
        self.0 == *other
    }
}

impl PartialEq<str> for CollectionName {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.0 == other
    }
}

impl AsRef<str> for CollectionName {
    #[inline]
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl borrow::Borrow<str> for CollectionName {
    #[inline]
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&'_ str> for CollectionName {
    type Error = CollectionNameError;

    #[inline]
    fn try_from(s: &'_ str) -> Result<Self, Self::Error> {
        Self::validate(s)?;

        Ok(Self(s.into()))
    }
}

impl TryFrom<String> for CollectionName {
    type Error = CollectionNameError;

    #[inline]
    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::validate(&s)?;

        Ok(Self(s))
    }
}

impl str::FromStr for CollectionName {
    type Err = CollectionNameError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::validate(s)?;

        Ok(Self(s.into()))
    }
}

impl From<CollectionName> for String {
    #[inline]
    fn from(s: CollectionName) -> Self {
        s.0
    }
}
