//! `collection` subcommand.

use std::{
    io::{self, Write},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, bail, Context as _};
use magro::{
    collection::{Collection, CollectionName},
    Context,
};
use structopt::StructOpt;

use crate::refresh::generate_collection_repos_cache;

/// Options for `collection` subcommand.
#[derive(Debug, Clone, StructOpt)]
#[non_exhaustive]
pub struct CollectionOpt {
    /// Subcommand.
    #[structopt(subcommand)]
    subcommand: Subcommand,
}

impl CollectionOpt {
    /// Runs the actual operation.
    pub fn run(&self, context: &mut Context) -> anyhow::Result<()> {
        match &self.subcommand {
            Subcommand::SetDefault {
                name,
                unset: _unset,
            } => {
                debug_assert_eq!(
                    name.is_none(),
                    *_unset,
                    "Either of `<name>` or `--unset` should be specified"
                );
                log::trace!("collection set-default name={:?}", name);
                set_default(context, name.as_ref())
            }
            Subcommand::Add {
                name,
                path,
                refresh,
            } => {
                log::trace!("collection add name={:?} path={:?}", name, path);
                add_collection(context, name, path, *refresh)
            }
            Subcommand::Del {
                names,
                allow_remove_nothing,
            } => {
                log::trace!(
                    "collection del name={:?}, allow_remove_nothing={}",
                    names,
                    allow_remove_nothing
                );
                unregister_collection(context, names, *allow_remove_nothing)
            }
            Subcommand::Show {
                collections: names,
                verbose,
            } => {
                log::trace!(
                    "collection show collections={:?}, verbose={}",
                    names,
                    verbose
                );
                let collections = context.config().collections();
                if names.is_empty() {
                    show_collections(context, &mut collections.iter().map(Ok), *verbose)
                } else {
                    let mut targets = names.iter().map(|name| collections.get(name).ok_or(name));
                    show_collections(context, &mut targets, *verbose)
                }
            }
            Subcommand::Rename { old_name, new_name } => {
                log::trace!(
                    "collection rename old_name={:?}, new_name={:?}",
                    old_name,
                    new_name
                );
                rename_collection(context, old_name, new_name)
            }
            Subcommand::GetPath { name } => {
                log::trace!("collection get-path name={:?}", name);
                get_path(context, name)
            }
            Subcommand::SetPath { name, path } => {
                log::trace!("collection set-path name={:?}, path={:?}", name, path);
                set_path(context, name, path)
            }
        }
    }
}

/// Subcommand of `collection`.
#[derive(Debug, Clone, StructOpt)]
pub enum Subcommand {
    /// Sets the default collection for some operations.
    SetDefault {
        /// Collection name.
        #[structopt(parse(try_from_str), required_unless = "unset")]
        name: Option<CollectionName>,
        /// Unsets the default collection.
        #[structopt(long, conflicts_with_all = &["name"])]
        unset: bool,
    },
    /// Adds a new collection.
    Add {
        /// Collection name.
        #[structopt(parse(try_from_str))]
        name: CollectionName,
        /// Path to the collection directory.
        ///
        /// If the path is relative, it is resolved using home directory as the base.
        /// If the path is absolute, it is used as is.
        #[structopt(parse(from_os_str))]
        path: PathBuf,
        /// Runs refresh operation for the newly added collection.
        ///
        /// Specifying this option updates a cache for the collection in the
        /// same way as `refresh --collections={new_collection} --keep-going`.
        #[structopt(long)]
        refresh: bool,
    },
    /// Unregisters a new collection.
    ///
    /// This just make magro forget about the collection, and never removes files from storage.
    Del {
        /// Collection names.
        // Use permissive types. Any invalid collection names won't break consistency of the config.
        #[structopt(required = true, min_values = 1)]
        names: Vec<String>,
        /// Do not emit an error if the collection does not exist.
        #[structopt(long = "allow-remove-nothing")]
        allow_remove_nothing: bool,
    },
    /// Shows the collections.
    Show {
        /// Collection name.
        ///
        /// If not specified, it is treated as all collections are specified.
        collections: Vec<CollectionName>,
        /// Shows verbose information.
        #[structopt(long = "verbose", short = "v")]
        verbose: bool,
    },
    /// Renames the collection.
    Rename {
        /// Old name.
        old_name: CollectionName,
        /// New name.
        new_name: CollectionName,
    },
    /// Shows the path to the collection directory.
    GetPath {
        /// Collection name.
        name: CollectionName,
    },
    /// Sets the path to the collection directory.
    SetPath {
        /// Collection name.
        name: CollectionName,
        /// Path to the collection directory.
        ///
        /// If the path is relative, it is resolved using home directory as the base.
        /// If the path is absolute, it is used as is.
        #[structopt(parse(from_os_str))]
        path: PathBuf,
    },
}

/// Sets the default collection.
fn set_default(context: &mut Context, name: Option<&CollectionName>) -> anyhow::Result<()> {
    if let Some(name) = name {
        if context.config().collections().get(name).is_none() {
            bail!("Collection named `{}` not found", name);
        }
    }
    context.config_mut().set_default_collection(name.cloned());
    context
        .save_config_if_dirty()
        .context("Failed to save config")?;

    match name {
        Some(name) => log::trace!("Set default collection to `{}`", name),
        None => log::trace!("Unset default colleciton"),
    }

    Ok(())
}

/// Adds the collection.
fn add_collection(
    context: &mut Context,
    name: &CollectionName,
    path: &Path,
    refresh: bool,
) -> anyhow::Result<()> {
    let collection = Collection::new(name.clone(), path.to_owned());
    let has_conflict = context
        .config_mut()
        .collections_mut()
        .insert(collection)
        .is_some();
    if has_conflict {
        bail!("Collection `{}` already exists", name);
    }

    context
        .save_config_if_dirty()
        .context("Failed to save config")?;

    // Create a new modified cache.
    let collection = context
        .config()
        .collections()
        .get(name)
        .expect("Should never fail: the collection was added just now");
    let mut newcache = context
        .get_or_load_cache()
        .context("Failed to load cache")?
        .clone();
    let coll_cache = if refresh {
        generate_collection_repos_cache(context, collection, false, true)
            .expect("Should not be `Err(_)` when `keep_going` is `true`")
            .unwrap_or_default()
    } else {
        Default::default()
    };
    newcache.cache_collection_repos(name.clone(), coll_cache);

    // Save the cache.
    context.save_cache(&newcache).with_context(|| {
        anyhow!(
            "Failed to save cache file {}",
            context.cache_path().display()
        )
    })?;

    log::debug!("Added the collection `{}`", name);

    Ok(())
}

/// Unregister the collection.
///
/// This operation is idempotent when `allow_remove_nothing` is `true`.
fn unregister_collection(
    context: &mut Context,
    names: &[String],
    allow_remove_nothing: bool,
) -> anyhow::Result<()> {
    // Create a new modified cache.
    let mut newcache = context
        .get_or_load_cache()
        .context("Failed to load cache")?
        .clone();

    for name in names {
        let is_removed = context
            .config_mut()
            .collections_mut()
            .remove(name)
            .is_some();
        if !is_removed {
            if allow_remove_nothing {
                log::debug!("Collection named {:?} does not exist", name);
            } else {
                bail!("Collection named {:?} does not exist", name);
            }
        }

        newcache.remove_collection_repos_cache(name);
    }

    // Save the config.
    context
        .save_config_if_dirty()
        .context("Failed to save config")?;

    // Save the cache.
    context.save_cache(&newcache).with_context(|| {
        anyhow!(
            "Failed to save cache file {}",
            context.cache_path().display()
        )
    })?;

    Ok(())
}

/// Shows the collections.
// Using `dyn Iterator` won't be problem, because the number of collections is
// expected to be small (for usual usage).
fn show_collections(
    context: &Context,
    collections: &mut dyn Iterator<Item = Result<&Collection, &CollectionName>>,
    verbose: bool,
) -> anyhow::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    if verbose {
        for collection in collections {
            let collection = collection.map_err(|name| anyhow!("No such collection `{}`", name))?;

            writeln!(handle, "collection: {}", collection.name())?;
            writeln!(
                handle,
                "    path: {}",
                collection.abspath(context).display()
            )?;
        }
    } else {
        for collection in collections {
            let collection =
                collection.map_err(|name| anyhow!("Collection named `{}` does not exist", name))?;

            writeln!(handle, "{}", collection.name())?;
        }
    }

    Ok(())
}

/// Renames the collection.
fn rename_collection(
    context: &mut Context,
    old_name: &CollectionName,
    new_name: &CollectionName,
) -> anyhow::Result<()> {
    let collections = context.config_mut().collections_mut();
    let mut collection = collections
        .remove(old_name)
        .ok_or_else(|| anyhow!("Collection named `{}` does not exist", old_name))?;
    collection.set_name(new_name.clone());
    let has_conflict = collections.insert(collection).is_some();
    if has_conflict {
        bail!("Collection `{}` already exists", new_name);
    }

    // Save the config.
    context
        .save_config_if_dirty()
        .context("Failed to save config")?;
    log::debug!("Renamed the collection `{}` to `{}`", old_name, new_name);

    // Create a new modified cache.
    let mut newcache = context
        .get_or_load_cache()
        .context("Failed to load cache")?
        .clone();
    let coll_cache = newcache
        .remove_collection_repos_cache(old_name)
        .unwrap_or_default();
    newcache.cache_collection_repos(new_name.clone(), coll_cache);

    // Save the cache.
    context.save_cache(&newcache).with_context(|| {
        anyhow!(
            "Failed to save cache file {}",
            context.cache_path().display()
        )
    })?;

    Ok(())
}

/// Shows the path to the collection directory.
fn get_path(context: &Context, name: &CollectionName) -> anyhow::Result<()> {
    let path = context
        .config()
        .collections()
        .get(name)
        .ok_or_else(|| anyhow!("Collection named `{}` does not exist", name))?
        .abspath(context);
    writeln!(io::stdout(), "{}", path.display())?;

    Ok(())
}

/// Sets the path to the collection directory.
fn set_path(context: &mut Context, name: &CollectionName, path: &Path) -> anyhow::Result<()> {
    context
        .config_mut()
        .collections_mut()
        .get_mut(name)
        .ok_or_else(|| anyhow!("Collection named `{}` does not exist", name))?
        .set_path(path);

    // Save the config.
    context
        .save_config_if_dirty()
        .context("Failed to save config")?;
    log::debug!("Set the path of the collection {:?} to {:?}", name, path);

    Ok(())
}
