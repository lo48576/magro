//! CLI options.

use std::{convert::TryFrom, str};

use magro::{
    collection::{CollectionName, CollectionNameError},
    vcs::{Vcs, VcsParseError},
    Context,
};
use structopt::StructOpt;

use crate::{collection::CollectionOpt, refresh::RefreshOpt};

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
            Subcommand::Refresh(opt) => opt.run(context),
        }
    }
}

/// Subcommand.
#[derive(Debug, Clone, StructOpt)]
pub enum Subcommand {
    /// Modify collections.
    Collection(CollectionOpt),
    /// Refresh collections.
    Refresh(RefreshOpt),
}

/// Space- or comma-separated collection names.
#[derive(Debug, Clone)]
pub(crate) struct CollectionNameList(Vec<CollectionName>);

impl AsRef<[CollectionName]> for CollectionNameList {
    #[inline]
    fn as_ref(&self) -> &[CollectionName] {
        &self.0
    }
}

impl str::FromStr for CollectionNameList {
    type Err = CollectionNameError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(|c: char| c.is_ascii_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .map(CollectionName::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

impl<'a> IntoIterator for &'a CollectionNameList {
    type IntoIter = CollectionNameListIter<'a>;
    type Item = &'a CollectionName;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        CollectionNameListIter(self.0.iter())
    }
}

/// Iterator over `CollectionNameList`.
#[derive(Debug)]
pub(crate) struct CollectionNameListIter<'a>(std::slice::Iter<'a, CollectionName>);

impl<'a> Iterator for CollectionNameListIter<'a> {
    type Item = &'a CollectionName;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

/// Space- or comma-separated VCS types.
#[derive(Debug, Clone)]
pub(crate) struct VcsList(Vec<Vcs>);

impl AsRef<[Vcs]> for VcsList {
    #[inline]
    fn as_ref(&self) -> &[Vcs] {
        &self.0
    }
}

impl str::FromStr for VcsList {
    type Err = VcsParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.split(|c: char| c.is_ascii_whitespace() || c == ',')
            .filter(|s| !s.is_empty())
            .map(Vcs::try_from)
            .collect::<Result<Vec<_>, _>>()
            .map(Self)
    }
}

impl<'a> IntoIterator for &'a VcsList {
    type IntoIter = VcsListIter<'a>;
    type Item = Vcs;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        VcsListIter(self.0.iter())
    }
}

/// Iterator over `VcsList`.
#[derive(Debug)]
pub(crate) struct VcsListIter<'a>(std::slice::Iter<'a, Vcs>);

impl<'a> Iterator for VcsListIter<'a> {
    type Item = Vcs;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied()
    }
}
