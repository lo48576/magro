//! Git functionalities.

use std::{borrow::Cow, iter, path::Path};

use git2::{Error as GitError, Repository, RepositoryOpenFlags};

/// Returns the working directory for the given repository if available.
///
/// Note that `.git` directory should be passed for normal repsoitory as `repo` parameter.
pub(super) fn workdir(repo_path: &Path) -> Result<Option<Cow<'_, Path>>, GitError> {
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
