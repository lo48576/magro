//! CLI options.

use magro::Context;
use structopt::StructOpt;

use crate::collection::CollectionOpt;

/// CLI options.
#[derive(Debug, Clone, StructOpt)]
#[non_exhaustive]
pub struct Opt {
    /// Subcommand.
    #[structopt(subcommand)]
    subcommand: Subcommand,
}

impl Opt {
    /// Runs the actual operation.
    pub fn run(&self, context: &Context) -> anyhow::Result<()> {
        match &self.subcommand {
            Subcommand::Collection(opt) => opt.run(context),
        }
    }
}

/// Subcommand.
#[derive(Debug, Clone, StructOpt)]
pub enum Subcommand {
    /// Modify collections.
    Collection(CollectionOpt),
}
