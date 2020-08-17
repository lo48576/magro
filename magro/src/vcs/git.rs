//! Git functionalities.

use std::{borrow::Cow, fs, io, iter, path::Path};

use anyhow::{anyhow, Context as _};
use git2::{
    build::RepoBuilder, Cred, CredentialType, FetchOptions, RemoteCallbacks, Repository,
    RepositoryOpenFlags,
};
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

impl From<io::Error> for Error {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self::new(e)
    }
}

impl From<anyhow::Error> for Error {
    #[inline]
    fn from(e: anyhow::Error) -> Self {
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

/// Clones the repository at `uri` as a local directory `dest`.
pub(super) fn clone(uri: &str, dest: &Path, bare: bool) -> Result<(), Error> {
    log::trace!("Cloning {:?} into {:?}", uri, dest);

    match dest.metadata() {
        Ok(meta) => {
            // Git accepts symlink to a directory as a destination.
            if !meta.is_dir() {
                return Err(anyhow!("Destination path {:?} is not a directory", dest).into());
            }
        }
        Err(e) => {
            if e.kind() != io::ErrorKind::NotFound {
                return Err(e.into());
            }
            // Create the destination directory.
            fs::DirBuilder::new()
                .recursive(true)
                .create(dest)
                .with_context(|| format!("Failed to create destination directory {:?}", dest))?;
        }
    }

    let mut builder: RepoBuilder<'_> = {
        let mut callbacks = RemoteCallbacks::new();
        callbacks.credentials(|_url, username_from_url, allowed_types| {
            let user = username_from_url.unwrap_or("git");
            if allowed_types.contains(CredentialType::USERNAME) {
                // See <https://github.com/rust-lang/git2-rs/issues/329#issuecomment-403318088>.
                return Cred::username(user);
            }
            if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                return Cred::ssh_key_from_agent(user);
            }
            Cred::default()
        });
        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);
        let mut builder = RepoBuilder::new();
        builder.fetch_options(fetch_opts);
        builder
    };

    builder.bare(bare);

    builder.clone(uri, dest)?;
    log::trace!("Successfully cloned {:?} into {:?}", uri, dest);

    Ok(())
}
