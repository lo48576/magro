//! `collection` subcommand.

use std::path::{Path, PathBuf};

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
            Subcommand::Del { name } => {
                log::trace!("collection del name={:?}", name);
                unregister_collection(context, name)
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
/// This operation is idempotent.
fn unregister_collection(context: &Context, name: &str) -> anyhow::Result<()> {
    // Create a new modified config.
    let mut newconf = context.config().clone();
    let is_removed = newconf.collections_mut().remove(name).is_some();
    if !is_removed {
        // This is not critical. Just warn.
        log::warn!("Collection named {:?} does not exist", name);
    }

    // Save the config.
    context.save_config(&newconf).with_context(|| {
        anyhow!(
            "Failed to save config file {}",
            context.config_path().display()
        )
    })?;
    log::info!("Unregistered the collection {:?}", name);

    Ok(())
}
