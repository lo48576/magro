//! `list` subcommand.

use std::{
    borrow::Cow,
    collections::HashSet,
    fmt,
    io::{self, Write},
    path::Path,
    str,
};

use anyhow::{anyhow, Context as _};
use magro::{
    collection::{Collection, CollectionName},
    vcs::Vcs,
    Context,
};
use structopt::StructOpt;

use crate::cli_opt::{CollectionNameList, VcsList};

/// Path base.
#[derive(Debug, Clone, Copy)]
enum PathBase {
    /// Filesystem's oot directory.
    Root,
    /// Collection directory.
    Collection,
    /// Home directory.
    Home,
}

impl PathBase {
    /// Returns a list of possible options.
    #[inline]
    #[must_use]
    fn possible_opt_values() -> &'static [&'static str] {
        &["root", "collection", "home"]
    }

    /// Returns the option value.
    #[inline]
    #[must_use]
    fn as_opt_value(&self) -> &'static str {
        match self {
            Self::Root => "root",
            Self::Collection => "collection",
            Self::Home => "home",
        }
    }

    /// Parses the option value.
    #[inline]
    #[must_use]
    fn from_opt_value(s: &str) -> Option<Self> {
        match s {
            "root" => Some(Self::Root),
            "collection" => Some(Self::Collection),
            "home" => Some(Self::Home),
            _ => None,
        }
    }
}

impl Default for PathBase {
    #[inline]
    fn default() -> Self {
        PathBase::Root
    }
}

impl str::FromStr for PathBase {
    type Err = anyhow::Error;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::from_opt_value(s).ok_or_else(|| anyhow!("Unsupported path style {:?}", s))
    }
}

impl fmt::Display for PathBase {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(self.as_opt_value())
    }
}

/// Options for `refresh` subcommand.
#[derive(Debug, Clone, StructOpt)]
#[non_exhaustive]
pub struct ListOpt {
    /// Separates lines by NUL characters.
    #[structopt(long, short = "z")]
    null_data: bool,
    /// Prints relativized paths using the specified base directory.
    ///
    /// Note that relativization can fail for some paths. In such case, `root`
    /// is used as fallback.
    #[structopt(
        long,
        possible_values = PathBase::possible_opt_values(),
        default_value = "root"
    )]
    path_base: PathBase,
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
            "list vcs={:?} collections={:?} null_data={} path_base={} workdir={}",
            self.vcs,
            self.collections,
            self.null_data,
            self.path_base,
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
                self.path_base,
            )
        } else {
            list_repos(
                context,
                &mut targets,
                target_vcs.as_ref(),
                self.workdir,
                self.null_data,
                self.path_base,
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
    path_base: PathBase,
) -> anyhow::Result<()> {
    let cache = context
        .get_or_load_cache()
        .context("Failed to load cache file")?
        .clone();

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let newline = if null_data { b"\0" } else { b"\n" };
    let home_dir = context.home_dir();

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
                debug_assert!(path_to_show.is_absolute());
                let path_to_show: &Path = match path_base {
                    PathBase::Root => &path_to_show,
                    PathBase::Collection => try_relativize(&path_to_show, &coll_base_path),
                    PathBase::Home => try_relativize(&path_to_show, home_dir),
                };

                print_raw_path(&mut handle, &path_to_show)?;
                handle.write_all(newline)?;
            }
        }
    }

    Ok(())
}

/// Returns relativized path if succeeded, or returns the raw input if failed.
fn try_relativize<'a>(path: &'a Path, base: &Path) -> &'a Path {
    debug_assert!(path.is_absolute());
    debug_assert!(base.is_absolute());

    if let Ok(relative) = path.strip_prefix(base) {
        return relative;
    }

    // Note that the working directory of a repository
    // could be outside of the collection directory.
    log::debug!(
        "Directory {:?} might not be a descendant of {:?}",
        path,
        base
    );
    // Use absolute path.
    path
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn path_style_consistent_string_conversion() {
        for &opt in PathBase::possible_opt_values() {
            assert_eq!(opt, opt.parse::<PathBase>().unwrap().to_string())
        }
    }
}
