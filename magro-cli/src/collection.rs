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
                name,
                allow_remove_nothing,
            } => {
                log::trace!(
                    "collection del name={:?}, allow_remove_nothing={}",
                    name,
                    allow_remove_nothing
                );
                unregister_collection(context, name, *allow_remove_nothing)
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
                    let mut targets = names
                        .iter()
                        .map(|name| collections.get(name.as_str()).ok_or(name));
                    show_collections(context, &mut targets, *verbose)
                }
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
        /// Collection name.
        // Use permissive types. Any invalid collection names won't break consistency of the config.
        name: String,
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
        bail!("Collection `{}` already exists", name.as_str());
    }

    // Save the config.
    context.save_config(&newconf).with_context(|| {
        anyhow!(
            "Failed to save config file {}",
            context.config_path().display()
        )
    })?;
    log::debug!("Added the collection `{}`", name.as_str());

    Ok(())
}

/// Unregister the collection.
///
/// This operation is idempotent when `allow_remove_nothing` is `true`.
fn unregister_collection(
    context: &Context,
    name: &str,
    allow_remove_nothing: bool,
) -> anyhow::Result<()> {
    // Create a new modified config.
    let mut newconf = context.config().clone();
    let is_removed = newconf.collections_mut().remove(name).is_some();
    if !is_removed {
        if allow_remove_nothing {
            log::debug!("Collection named {:?} does not exist", name);
        } else {
            bail!("Collection named {:?} does not exist", name);
        }
    }

    // Save the config.
    context.save_config(&newconf).with_context(|| {
        anyhow!(
            "Failed to save config file {}",
            context.config_path().display()
        )
    })?;
    if is_removed {
        log::info!("Unregistered the collection {:?}", name);
    }

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
            let collection =
                collection.map_err(|name| anyhow!("No such collection `{}`", name.as_str()))?;

            writeln!(handle, "collection: {}", collection.name().as_str())?;
            writeln!(
                handle,
                "    path: {}",
                collection.abspath(context).display()
            )?;
        }
    } else {
        for collection in collections {
            let collection = collection
                .map_err(|name| anyhow!("Collection named `{}` does not exist", name.as_str()))?;

            writeln!(handle, "{}", collection.name().as_str())?;
        }
    }

    Ok(())
}

/// Shows the path to the collection directory.
fn get_path(context: &Context, name: &CollectionName) -> anyhow::Result<()> {
    let path = context
        .config()
        .collections()
        .get(name.as_str())
        .ok_or_else(|| anyhow!("Collection named `{}` does not exist", name.as_str()))?
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
        .get_mut(name.as_str())
        .ok_or_else(|| anyhow!("Collection named `{}` does not exist", name.as_str()))?
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
