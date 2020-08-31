//! `clone` subcommand.

use std::{borrow::Cow, iter, path::Path};

use anyhow::{bail, Context as _};
use magro::{cache::RepoCacheEntry, collection::CollectionName, vcs::Vcs, Context};
use structopt::StructOpt;

use crate::cli_opt::OptionBool;

/// Options for `clone` subcommand.
#[derive(Debug, Clone, StructOpt)]
#[non_exhaustive]
pub struct CloneOpt {
    /// URI of the reposiotry.
    uri: String,
    /// Collection to put the cloned repository.
    #[structopt(long, short)]
    collection: Option<CollectionName>,
    /// VCS to use.
    ///
    /// If not specified, the program attempt to detect VCS automatically.
    #[structopt(
        long,
        possible_values = &Vcs::variants().map(|v| v.name_lower()).collect::<Vec<_>>(),
    )]
    vcs: Option<Vcs>,
    /// Whether to clone bare repository.
    #[structopt(
        long,
        possible_values = OptionBool::possible_opt_values(),
        default_value = "auto",
    )]
    bare: OptionBool,
}

impl CloneOpt {
    /// Runs the actual operation.
    pub fn run(&self, context: &Context) -> anyhow::Result<()> {
        log::trace!(
            "clone uri={:?}, collection={:?}, vcs={:?}, bare={}",
            self.uri,
            self.collection,
            self.vcs,
            self.bare
        );

        clone_repo(
            context,
            &self.uri,
            self.collection.as_ref(),
            self.vcs,
            self.bare,
        )
    }
}

/// Clones the repository.
fn clone_repo(
    context: &Context,
    uri: &str,
    collection_name: Option<&CollectionName>,
    vcs_opt: Option<Vcs>,
    bare: OptionBool,
) -> anyhow::Result<()> {
    let collection = if let Some(name) = collection_name {
        context
            .config()
            .collections()
            .get(name)
            .with_context(|| format!("Collection `{}` not found", name))?
    } else if let Some(name) = context.config().default_collection() {
        context
            .config()
            .collections()
            .get(name)
            .with_context(|| format!("Default collection `{}` not found", name))?
    } else {
        bail!("No target collection specified");
    };

    let vcs = vcs_opt
        .or_else(|| suppose_vcs_from_uri(uri))
        .with_context(|| format!("Failed to get VCS type for URI {:?}", uri))?;
    log::debug!("Assumed VCS is {}", vcs.name_lower());

    let bare = bare == OptionBool::Yes;

    let reldest = match vcs {
        Vcs::Git => {
            git_dest_relpath(uri, bare).context("Failed to determine clone destination path")?
        }
        vcs => {
            // This should not happen because `magro-cli` implementation is
            // devloped at the same time with `magro` backend.
            unreachable!("Got unknown VCS {}", vcs.name_lower());
        }
    };
    assert!(reldest.is_relative());

    let absdest = collection.abspath(context).join(&reldest);
    log::debug!("Destination directory is {:?}", absdest);

    vcs.clone(uri, &absdest, bare)
        .with_context(|| format!("Failed to clone repository {:?} into {:?}", uri, absdest))?;

    // Update cache.
    let mut newcache = context
        .get_or_load_cache()
        .context("Failed to load cache file")?
        .clone();
    if let Some(mut repos) = newcache.remove_collection_repos_cache(collection.name()) {
        let entry = RepoCacheEntry::new(vcs, reldest);
        // Use `extend_one` once stabilized.
        // See <https://github.com/rust-lang/rust/issues/72631>.
        repos.extend(iter::once(entry));
        newcache.cache_collection_repos(collection.name().to_owned(), repos);
    }

    // Save the cache file.
    context
        .save_cache(&newcache)
        .context("Failed to save cache file")?;

    Ok(())
}

/// Tries to suppose VCS type for the given URI.
// TODO: Write unit tests.
fn suppose_vcs_from_uri(uri: &str) -> Option<Vcs> {
    if uri.ends_with(".git") {
        return Some(Vcs::Git);
    }
    if uri.starts_with("git://") {
        return Some(Vcs::Git);
    }
    if let Some(authority_start) = uri.find("://").map(|v| v + 3) {
        if let Some(first_slash) = uri[authority_start..]
            .find('/')
            .map(|v| v + authority_start)
        {
            let hostname = {
                // authority: `[ user [ ':' pass ] '@' ] hostname [':' port ]`
                let authority = &uri[authority_start..first_slash];
                let hostname_start = authority.rfind('@').map_or(0, |v| v + 1);
                let hostname_end = authority
                    .rfind(':')
                    .filter(|&v| v > hostname_start)
                    .unwrap_or_else(|| authority.len());
                &authority[hostname_start..hostname_end]
            };
            log::trace!("Hostname of {:?} is {:?}", uri, hostname);

            if hostname.starts_with("git") {
                return Some(Vcs::Git);
            }
        }
    }

    None
}

/// Calculate relative destination path for the given repository.
fn git_dest_relpath(uri_orig: &str, bare: bool) -> anyhow::Result<Cow<'_, Path>> {
    // Remove `.git` suffix if necessary.
    let uri = if bare {
        uri_orig
    } else {
        uri_orig.strip_suffix(".git").unwrap_or(uri_orig)
    };

    // Reject local repository.
    let first_colon = match uri.find(':') {
        Some(v) => v,
        None => {
            // Local path.
            bail!(
                "Cannot determine destination path for cloning the local reposiotry {:?}",
                uri_orig
            );
        }
    };
    if uri[..first_colon].find('/').is_some() {
        // Git considers this as local path.
        //
        // > This syntax \[scp-like syntax\] is only recognized if there are
        // > no slashes before the first colon.
        // >
        // > --- <https://mirrors.edge.kernel.org/pub/software/scm/git/docs/git-clone.html#URLS>
        bail!(
            "Cannot determine destination path for cloning the local reposiotry {:?}",
            uri_orig
        );
    }

    // Check if `uri` is alternative scp-like syntax `[user@]host.xz:path/to/repo`.
    // Destination is `[user@]host.xz/path/to/repo`.
    if !uri[(first_colon + 1)..].starts_with("//") {
        // scp-like syntax.
        log::trace!("{:?} is considered as an scp-lie syntax", uri_orig);
        let userhost = &uri[..first_colon];
        // Remove `git@` prefix.
        // `git` is a common user and usually is not useful information.
        let userhost = userhost.strip_prefix("git@").unwrap_or(userhost);

        // Treat absolute path as relative.
        // Destination for `host:/path` should be `host/path`, but
        // `"host".join("/path")` would be `/path`.
        let path = Path::new(uri[(first_colon + 1)..].trim_start_matches('/'));
        assert!(
            path.is_relative(),
            "The path part should be treated as relative"
        );

        return Ok(Cow::Owned(Path::new(userhost).join(path)));
    }

    // `scheme://host[:port]/path/to/repo` syntax.
    log::trace!("{:?} is considered as a normal URI", uri);
    debug_assert!(uri[(first_colon + 1)..].starts_with("//"));
    let host_and_path = &uri[(first_colon + 3)..];
    // Remove `git@` prefix.
    // `git` is a common user and usually is not useful information.
    let host_and_path = host_and_path.strip_prefix("git@").unwrap_or(host_and_path);

    Ok(Cow::Borrowed(Path::new(host_and_path)))
}

#[cfg(test)]
mod tests {
    use super::*;

    mod git_dest_relpath {
        use super::*;

        #[test]
        fn normal_cases() {
            assert_eq!(
                git_dest_relpath("user@example.com:path/to/repo", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("user@example.com/path/to/repo"))
            );
            assert_eq!(
                git_dest_relpath("https://user@example.com/path/to/repo", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("user@example.com/path/to/repo"))
            );
            assert_eq!(
                git_dest_relpath("https://example.com/path/to/repo", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("example.com/path/to/repo"))
            );
            assert_eq!(
                git_dest_relpath("git://user@example.com/path/to/repo", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("user@example.com/path/to/repo"))
            );
        }

        // For non-bare repositories, `.git` suffix should be removed.
        #[test]
        fn omit_dotgit_suffix_for_non_bare() {
            assert_eq!(
                git_dest_relpath("user@example.com:path/to/repo.git", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("user@example.com/path/to/repo"))
            );
            assert_eq!(
                git_dest_relpath("https://example.com/path/to/repo.git", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("example.com/path/to/repo"))
            );

            // `.git` suffix is removed only once.
            assert_eq!(
                git_dest_relpath("https://example.com/path/to/repo.git.git", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("example.com/path/to/repo.git"))
            );
        }

        // `git` user should be treated as not specified when deciding destination path.
        #[test]
        fn omit_git_user() {
            assert_eq!(
                git_dest_relpath("git@example.com:path/to/repo", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("example.com/path/to/repo"))
            );
            assert_eq!(
                git_dest_relpath("https://git@example.com/path/to/repo", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("example.com/path/to/repo"))
            );
        }

        // If the path part is absolute or contains extra slashes at the
        // beginning, the toplevel directory should be host part.
        #[test]
        fn path_relative() {
            assert_eq!(
                git_dest_relpath("example.com:/path/to/repo", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("example.com/path/to/repo"))
            );
            assert_eq!(
                git_dest_relpath("https://example.com//path/to/repo", false)
                    .ok()
                    .as_deref(),
                Some(Path::new("example.com/path/to/repo"))
            );
        }
    }
}
