//! `refresh` subcommand.

use anyhow::{bail, Context as _};
use magro::{
    cache::{CollectionReposCache, RepoCacheEntry},
    collection::{Collection, CollectionName},
    Context,
};
use structopt::StructOpt;

use crate::cli_opt::CollectionNameList;

/// Options for `refresh` subcommand.
#[derive(Debug, Clone, StructOpt)]
#[non_exhaustive]
pub struct RefreshOpt {
    /// Show verbose log.
    #[structopt(long, short, multiple = true)]
    verbose: bool,
    /// Do not show log.
    ///
    /// This is default, and only useful when disabling previously specified
    /// `--verbose` option.
    // This field only exists to make it possible to disable `--verbose`, and
    // is not intended for use by any other code than structopt.
    // `verbose` field should always be used to get loglevel.
    #[structopt(
        long = "quiet",
        short = "q",
        overrides_with = "verbose",
        multiple = true
    )]
    _quiet: bool,
    /// Runs the operation as possible even when errors are detected.
    ///
    /// Note that errors are ignored during the operation, but the program will
    /// exit with failure (i.e. errors won't be completely ignored).
    #[structopt(long)]
    keep_going: bool,
    /// Collections to refresh.
    ///
    /// If no collections are specified, it behaves as all collections are given.
    #[structopt(long, short, parse(try_from_str), multiple = true)]
    collections: Vec<CollectionNameList>,
}

impl RefreshOpt {
    /// Runs the actual operation.
    pub fn run(&self, context: &Context) -> anyhow::Result<()> {
        log::trace!(
            "refresh collections={:?}, verbose={}",
            self.collections,
            self.verbose
        );

        let collections = context.collections_config().collections();
        let mut targets = self
            .collections
            .iter()
            .flatten()
            .map(|name| collections.get(name).ok_or(name))
            .peekable();

        if targets.peek().is_none() {
            refresh_collections(
                context,
                &mut collections.iter().map(Ok),
                self.verbose,
                self.keep_going,
            )
        } else {
            refresh_collections(context, &mut targets, self.verbose, self.keep_going)
        }
    }
}

/// Refreshes the collections.
// Using `dyn Iterator` won't be problem, because the number of collections is
// expected to be small (for usual usage).
fn refresh_collections(
    context: &Context,
    collections: &mut dyn Iterator<Item = Result<&Collection, &CollectionName>>,
    verbose: bool,
    keep_going: bool,
) -> anyhow::Result<()> {
    use std::fmt::Write;

    let mut cache = context
        .get_or_load_cache()
        .context("Failed to load cache file")?
        .clone();

    let mut error_collections: Vec<&CollectionName> = Vec::new();

    for collection in collections {
        let collection = match collection {
            Ok(v) => v,
            Err(name) => {
                if keep_going {
                    log::error!("Collection named `{}` does not exist", name);
                    continue;
                } else {
                    bail!("Collection named `{}` does not exist", name);
                }
            }
        };
        log::debug!("Refreshing collection `{}`", collection.name());

        // `?` can be used here, because `generate_collection_repos_cache()`
        // could return `Err(_)` only when `keep_going` is false.
        let collection_cache: Option<_> =
            generate_collection_repos_cache(context, collection, verbose, keep_going)?;
        if collection_cache.is_none() {
            error_collections.push(collection.name());
        }
        let collection_cache = collection_cache.unwrap_or_default();

        cache.cache_collection_repos(collection.name().clone(), collection_cache);
    }

    // Save the cache file.
    context
        .save_cache(&cache)
        .context("Failed to save cache file")?;

    if !error_collections.is_empty() {
        assert!(keep_going);

        // Report the error.
        let failed_names = {
            let mut iter = error_collections.into_iter();
            let mut names = iter
                .next()
                .expect("`error_collections` is not empty")
                .to_string();
            let _ = iter.try_for_each(|name| write!(names, ", {}", name));
            names
        };
        log::warn!("Refresh failed for these collection(s): {}", failed_names);
    }

    Ok(())
}

/// Generates a `CollectionReposCache` for the given collection.
///
/// This always returns `Ok(_)` when `keep_going` is `true`.
/// `Ok(None)` will be returned when `keep_going` is `true` and failed to
/// discover repositories.
pub(crate) fn generate_collection_repos_cache(
    context: &Context,
    collection: &Collection,
    verbose: bool,
    keep_going: bool,
) -> anyhow::Result<Option<CollectionReposCache>> {
    log::debug!(
        "Generating cache for the collection `{}`",
        collection.name()
    );

    let repos = match discover_repositories(context, collection, verbose, keep_going) {
        Ok(v) => v,
        Err(e) => {
            if !keep_going {
                return Err(e);
            }
            log::error!(
                "Error happened during refreshing collection `{}`: {}",
                collection.name(),
                e
            );
            // The entire collection is unavailable.
            return Ok(None);
        }
    };

    // Create the new collection cache.
    let mut collection_cache = CollectionReposCache::default();
    collection_cache.extend(repos);

    Ok(Some(collection_cache))
}

/// Discovers the git directories.
///
/// If the collection directory does not exist, this returns `Ok(_)`.
/// If the collection directory is symlink and the directory pointed to
/// does not exist, this returns `Err(_)`.
/// (In other words, this function will treat broken symlink as an error.)
///
/// If `keep_going` is `true`, errors during directory traversal is not treated
/// as error, and the traversal will be continued.
///
/// However, errors during setup before directory traversal (i.e.
/// collection-wide error) will be still treated as an error.
/// For example, if the collection directory itself is unreadable, this
/// function returns `Err(_)` even when `keep_going` is `true`.
///
/// Returned `RepoCacheEntry`s will have relative path to the repositories,
/// and their base path is the collection directory.
fn discover_repositories(
    context: &Context,
    collection: &Collection,
    verbose: bool,
    keep_going: bool,
) -> anyhow::Result<Vec<RepoCacheEntry>> {
    let root_dir = collection.abspath(context);
    let repos = match magro::discovery::RepoSeeker::new(&root_dir) {
        Ok(Some(repos)) => {
            let mut result: Vec<RepoCacheEntry> = Vec::new();

            for entry in repos {
                let repo = match entry {
                    Ok(v) => v,
                    Err(e) => {
                        if keep_going {
                            log::error!("Error during directory traversal: {}", e);
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                };

                log::info!(
                    "Found {} repository {:?}",
                    repo.vcs().name_lower(),
                    repo.path()
                );
                if verbose {
                    println!(
                        "Found {} repository {:?}",
                        repo.vcs().name_lower(),
                        repo.path()
                    );
                }

                // Relativize.
                let repo = RepoCacheEntry::from(repo)
                    .try_map_ref_path(|path| path.strip_prefix(&root_dir).map(Into::into))
                    .expect("The repository path must be prefixed by `root_dir`");
                result.push(repo);
            }
            result
        }
        Ok(None) => Vec::new(),
        Err(e) => return Err(e).context(format!("Cannot traverse the directory {:?}", root_dir)),
    };

    Ok(repos)
}
