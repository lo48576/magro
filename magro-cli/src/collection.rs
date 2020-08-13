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
