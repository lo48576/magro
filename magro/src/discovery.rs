//! Repositories discovery.

use std::{
    fs, io, iter,
    path::{Path, PathBuf},
};

use anyhow::anyhow;
use git2::{Repository, RepositoryOpenFlags};
use thiserror::Error as ThisError;

use crate::vcs::Vcs;

/// Repository discovery error.
#[derive(Debug, ThisError)]
#[error(transparent)]
pub struct Error {
    /// Source error.
    source: anyhow::Error,
}

impl Error {
    /// Creates a new error.
    #[inline]
    #[must_use]
    fn new(e: impl Into<anyhow::Error>) -> Self {
        Self { source: e.into() }
    }

    /// Creates a new error with the given context.
    #[inline]
    #[must_use]
    fn context<C>(e: impl Into<anyhow::Error>, context: C) -> Self
    where
        C: std::fmt::Display + Send + Sync + 'static,
    {
        Self {
            source: e.into().context(context),
        }
    }
}

/// A repository entry.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RepoEntry {
    /// VCS type.
    vcs: Vcs,
    /// Path.
    ///
    /// For git, `.git` directory or `*.git` directory.
    path: PathBuf,
}

impl RepoEntry {
    /// Creates a new `RepoCacheEntry`.
    #[inline]
    #[must_use]
    fn new<P: Into<PathBuf>>(vcs: Vcs, path: P) -> Self {
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
}

/// Repositories seeker, an iterator of repositories under a directory.
#[derive(Debug)]
pub struct RepoSeeker {
    /// Walkdir iterator.
    dir_walker: walkdir::IntoIter,
}

impl RepoSeeker {
    /// Creates a new `RepoSeeker`.
    ///
    /// * Returns `Ok(Some(_))` if the path is accessible as a directory.
    /// * Returns `Ok(None)` if the path does not exist.
    /// * Returns `Err(_)` if the path is broken symlink or I/O error happened.
    #[inline]
    pub fn new<P: AsRef<Path>>(root_dir: P) -> Result<Option<Self>, Error> {
        Self::new_impl(root_dir.as_ref())
    }

    /// Monomorphized internal implementation for `new()`.
    fn new_impl(root_dir: &Path) -> Result<Option<Self>, Error> {
        if !root_dir.exists() {
            // Check if the directory is symlink.
            return match fs::symlink_metadata(&root_dir) {
                Ok(meta) => {
                    assert!(
                        meta.file_type().is_symlink(),
                        "The file should be a broken symlink, if `exists()` \
                         returns false but `symlink_metadata()` succeeds"
                    );

                    // Broken symlink.
                    Err(Error::new(anyhow!(
                        "Collection directory {} is a broken symlink",
                        root_dir.display()
                    )))
                }
                Err(e) => match e.kind() {
                    io::ErrorKind::NotFound => Ok(None),
                    _ => Err(Error::new(anyhow!(
                        "Failed to access the collection directory {}",
                        root_dir.display()
                    ))),
                },
            };
        }

        let mut dir_walker = walkdir::WalkDir::new(root_dir).into_iter();
        // Skip the root directory itself.
        match dir_walker.next() {
            None => unreachable!("The first direntry should be the collection dierctory itself"),
            Some(Ok(entry)) => {
                debug_assert_eq!(entry.path(), root_dir);
            }
            Some(Err(e)) => {
                return Err(Error::context(
                    e,
                    format!("Failed to traverse the directory {:?}", root_dir),
                ));
            }
        }

        Ok(Some(Self { dir_walker }))
    }

    /// Seeks the next repository, and returns it if found.
    fn seek_next(&mut self) -> Result<Option<RepoEntry>, Error> {
        loop {
            let entry = match self.dir_walker.next() {
                None => return Ok(None),
                Some(Ok(v)) => v,
                Some(Err(e)) => return Err(Error::new(e)),
            };

            if !entry.file_type().is_dir() {
                // Not a directory.
                continue;
            }
            let path = entry.path();
            let filename = entry.path().file_name().expect(
                "The DirEntry points to a descendant of the target directory, \
                 and it should have a filename",
            );

            // Check if the directory is a `.git` directory or a bare repository.
            if filename == ".git" || path.extension().map_or(false, |ext| ext == ".git") {
                match test_git_directory(path) {
                    Ok(repo) => {
                        // Get out of `.git` directory.
                        self.dir_walker.skip_current_dir();

                        let workdir = repo.workdir();
                        let parent = path
                            .parent()
                            .expect("`path` has the seek root directory as its ancestor");
                        if workdir == Some(parent) {
                            log::trace!(
                                "Skipping {:?} as it is the working directory of {:?}",
                                parent,
                                path
                            );
                            // Get out of working directory of the repository.
                            self.dir_walker.skip_current_dir();
                        }
                        return Ok(Some(RepoEntry::new(Vcs::Git, entry.into_path())));
                    }
                    Err(e) => {
                        log::debug!(
                            "Directory {:?} is neither a git directory nor a bare repository: {}",
                            path,
                            e
                        );
                    }
                }
            }
        }
    }
}

impl Iterator for RepoSeeker {
    type Item = Result<RepoEntry, Error>;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.seek_next().transpose()
    }
}

/// Tests if the directory is a git directory.
#[inline]
fn test_git_directory(gitdir: &Path) -> Result<Repository, git2::Error> {
    // NO_SEARCH: No need of extra traversal because we already have
    // candidate path of the git directory.
    // NO_DOTGIT: No need of appending `/.git` because we already have
    // `.git` directory path.
    let open_flags = RepositoryOpenFlags::NO_SEARCH | RepositoryOpenFlags::NO_DOTGIT;
    Repository::open_ext(&gitdir, open_flags, iter::empty::<&str>())
}
