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
    pub fn run(&self, context: &Context) -> anyhow::Result<()> {
        match &self.subcommand {
            Subcommand::Add { name, path } => {
                log::trace!("collection add name={:?} path={:?}", name, path);
                add_collection(context, name, path)
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

/// Adds the collection.
fn add_collection(context: &Context, name: &CollectionName, path: &Path) -> anyhow::Result<()> {
    // Create a new modified config.
    let mut newconf = context.config().clone();
    let collection = Collection::new(name.clone(), path.to_owned());
    let has_conflict = newconf.collections_mut().insert(collection).is_some();
    if has_conflict {
        bail!("Collection `{}` already exists", name);
    }

    // Save the config.
    context.save_config(&newconf).with_context(|| {
        anyhow!(
            "Failed to save config file {}",
            context.config_path().display()
        )
    })?;
    log::debug!("Added the collection `{}`", name);

    Ok(())
}

/// Unregister the collection.
///
/// This operation is idempotent when `allow_remove_nothing` is `true`.
fn unregister_collection(
    context: &Context,
    names: &[String],
    allow_remove_nothing: bool,
) -> anyhow::Result<()> {
    // Create a new modified config.
    let mut newconf = context.config().clone();
    for name in names {
        let is_removed = newconf.collections_mut().remove(name).is_some();
        if !is_removed {
            if allow_remove_nothing {
                log::debug!("Collection named {:?} does not exist", name);
            } else {
                bail!("Collection named {:?} does not exist", name);
            }
        }
    }

    // Save the config.
    context.save_config(&newconf).with_context(|| {
        anyhow!(
            "Failed to save config file {}",
            context.config_path().display()
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
    context: &Context,
    old_name: &CollectionName,
    new_name: &CollectionName,
) -> anyhow::Result<()> {
    // Create a new modified config.
    let mut newconf = context.config().clone();
    let mut collection = newconf
        .collections_mut()
        .remove(old_name)
        .ok_or_else(|| anyhow!("Collection named `{}` does not exist", old_name))?;
    collection.set_name(new_name.clone());
    let has_conflict = newconf.collections_mut().insert(collection).is_some();
    if has_conflict {
        bail!("Collection `{}` already exists", new_name);
    }

    // Save the config.
    context.save_config(&newconf).with_context(|| {
        anyhow!(
            "Failed to save config file {}",
            context.config_path().display()
        )
    })?;
    log::debug!("Renamed the collection `{}` to `{}`", old_name, new_name);

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
fn set_path(context: &Context, name: &CollectionName, path: &Path) -> anyhow::Result<()> {
    // Create a new modified config.
    let mut newconf = context.config().clone();
    newconf
        .collections_mut()
        .get_mut(name)
        .ok_or_else(|| anyhow!("Collection named `{}` does not exist", name))?
        .set_path(path);

    // Save the config.
    context.save_config(&newconf).with_context(|| {
        anyhow!(
            "Failed to save config file {}",
            context.config_path().display()
        )
    })?;
    log::debug!("Set the path of the collection {:?} to {:?}", name, path);

    Ok(())
}
