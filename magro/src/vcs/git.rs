//! Git functionalities.

use std::{borrow::Cow, iter, path::Path};

use git2::{Repository, RepositoryOpenFlags};
use thiserror::Error as ThisError;

/// Error for git-related operations.
#[derive(Debug, ThisError)]
#[error(transparent)]
pub(super) struct Error(anyhow::Error);

impl Error {
    /// Creates a new error.
    #[inline]
    #[must_use]
    fn new(e: impl Into<anyhow::Error>) -> Self {
        Self(e.into())
    }
}

impl From<git2::Error> for Error {
    #[inline]
    fn from(e: git2::Error) -> Self {
        Self::new(e)
    }
}

/// Returns the working directory for the given repository if available.
///
/// Note that `.git` directory should be passed for normal repsoitory as `repo` parameter.
pub(super) fn workdir(repo_path: &Path) -> Result<Option<Cow<'_, Path>>, Error> {
    // NO_SEARCH: No need of extra traversal because we already have
    // candidate path of the git directory.
    // NO_DOTGIT: No need of appending `/.git` because we already have
    // `.git` directory path.
    //
    // Note that `BARE` should not be specified here, because it makes the
    // working directory ignored.
    let open_flags = RepositoryOpenFlags::NO_SEARCH | RepositoryOpenFlags::NO_DOTGIT;
    let repo = Repository::open_ext(repo_path, open_flags, iter::empty::<&str>())?;

    let workdir = match repo.workdir() {
        Some(v) => v,
        None => return Ok(None),
    };

    if let Some(parent) = repo_path.parent() {
        if parent == workdir {
            // Avoid allocation.
            return Ok(Some(Cow::Borrowed(parent)));
        }
    }

    Ok(Some(Cow::Owned(workdir.to_owned())))
}
