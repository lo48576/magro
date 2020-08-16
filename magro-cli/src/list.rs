//! `list` subcommand.

use std::{
    borrow::Cow,
    collections::HashSet,
    io::{self, Write},
    path::Path,
};

use anyhow::{anyhow, Context as _};
use magro::{
    collection::{Collection, CollectionName},
    vcs::Vcs,
    Context,
};
use structopt::StructOpt;

use crate::cli_opt::{CollectionNameList, VcsList};

/// Options for `refresh` subcommand.
#[derive(Debug, Clone, StructOpt)]
#[non_exhaustive]
pub struct ListOpt {
    /// Separates lines by NUL characters.
    #[structopt(long, short = "z")]
    null_data: bool,
    /// Prints relative path to the collection directory.
    // TODO: Better (and hopefully shorter) name.
    // TODO: Make it enum, not boolean.
    #[structopt(long)]
    relative_to_collection: bool,
    /// Prints working directory
    #[structopt(long)]
    workdir: bool,
    /// Prints only repositories of specified VCS's.
    // Not using `-v` for this, as it can be confused with `--verbose`.
    #[structopt(long, parse(try_from_str), multiple = true)]
    vcs: Vec<VcsList>,
    /// Prints only repositories of the specified collections.
    #[structopt(long, short, parse(try_from_str))]
    collections: Vec<CollectionNameList>,
}

impl ListOpt {
    /// Runs the actual operation.
    pub fn run(&self, context: &Context) -> anyhow::Result<()> {
        log::trace!(
            "list vcs={:?} collections={:?} null_data={} workdir={}",
            self.vcs,
            self.collections,
            self.null_data,
            self.workdir
        );

        let target_vcs: Option<HashSet<Vcs>> = match self.vcs.as_slice() {
            [] => None,
            vcs => Some(vcs.iter().flatten().collect()),
        };
        let collections = context.config().collections();
        let mut targets = self
            .collections
            .iter()
            .flatten()
            .map(|name| collections.get(name).ok_or(name))
            .peekable();

        if targets.peek().is_none() {
            list_repos(
                context,
                &mut collections.iter().map(Ok),
                target_vcs.as_ref(),
                self.workdir,
                self.null_data,
                self.relative_to_collection,
            )
        } else {
            list_repos(
                context,
                &mut targets,
                target_vcs.as_ref(),
                self.workdir,
                self.null_data,
                self.relative_to_collection,
            )
        }
    }
}

/// List repositories.
// Using `dyn Iterator` won't be problem, because the number of collections is
// expected to be small (for usual usage).
fn list_repos(
    context: &Context,
    collections: &mut dyn Iterator<Item = Result<&Collection, &CollectionName>>,
    target_vcs: Option<&HashSet<Vcs>>,
    show_workdir: bool,
    null_data: bool,
    relative_to_collection: bool,
) -> anyhow::Result<()> {
    let cache = context
        .get_or_load_cache()
        .context("Failed to load cache file")?
        .clone();

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let newline = if null_data { b"\0" } else { b"\n" };
    for collection in collections {
        let collection =
            collection.map_err(|name| anyhow!("Collection named `{}` does not exist", name))?;

        let coll_name = collection.name();
        let coll_base_path = collection.abspath(context);

        log::trace!("Listing repositories in the collection `{}`", coll_name);

        let coll_cache = match cache.collection_repos(coll_name) {
            Some(v) => v,
            None => {
                log::info!("No cache found for collection `{}`", coll_name);
                continue;
            }
        };

        for repo in coll_cache.repositories() {
            let vcs = repo.vcs();
            let abspath = coll_base_path.join(repo.path());

            if target_vcs.map_or(true, |targets| targets.contains(&vcs)) {
                let path_to_show = if show_workdir {
                    // FIXME: Is it ok to return immediately if it returned error?
                    let workdir = vcs.workdir(&abspath).with_context(|| {
                        anyhow!(
                            "Failed to get working directory for {} repository {:?}",
                            vcs.name_lower(),
                            abspath
                        )
                    })?;
                    match workdir {
                        Some(v) => v,
                        None => {
                            log::debug!(
                                "No working directory for {} repository {:?}",
                                vcs.name_lower(),
                                abspath
                            );
                            continue;
                        }
                    }
                } else {
                    Cow::Borrowed(abspath.as_ref())
                };
                let path_to_show: &Path = if relative_to_collection {
                    match path_to_show.strip_prefix(&coll_base_path) {
                        Ok(v) => v,
                        Err(_) => {
                            // Note that the working directory of a repository
                            // could be outside of the collection directory.
                            log::debug!(
                                "Directory {:?} might not descendant of {:?}",
                                path_to_show,
                                coll_base_path
                            );
                            // Use absolute path.
                            &path_to_show
                        }
                    }
                } else {
                    &path_to_show
                };

                print_raw_path(&mut handle, &path_to_show)?;
                handle.write_all(newline)?;
            }
        }
    }

    Ok(())
}

/// Attempts to print the raw path, even when it is invalid UTF-8 sequence.
///
/// This does not write the trailing newline.
#[cfg(unix)]
#[inline]
fn print_raw_path<W: io::Write>(writer: &mut W, path: &Path) -> io::Result<()> {
    use std::os::unix::ffi::OsStrExt;

    writer.write_all(path.as_os_str().as_bytes())
}

/// Attempts to print the raw path, even when it is invalid UTF-8 sequence.
///
/// This does not write the trailing newline.
#[cfg(not(unix))]
#[inline]
fn print_raw_path<W: io::Write>(writer: &mut W, path: &Path) -> io::Result<()> {
    write!(writer, "{}", path.display())
}
