//! Filesystem ops.

use std::fs::File;
use std::io::{Read, Result, Write};
use std::path::Path;

use fd_lock::FdLock;

/// Opens a file and returns it in a lockable form.
#[inline]
pub(crate) fn open<P: AsRef<Path>>(path: P) -> Result<FdLock<File>> {
    File::open(path).map(FdLock::new)
}

/// Opens or creates a file and returns it in a lockable form.
#[inline]
pub(crate) fn create<P: AsRef<Path>>(path: P) -> Result<FdLock<File>> {
    File::create(path).map(FdLock::new)
}

/// Reads the file exclusively from the given flie into a string.
///
/// During the read, the file is locked.
pub(crate) fn read_to_string_from_lockable_file(
    path: &Path,
    file: &mut FdLock<File>,
) -> Result<String> {
    let mut content = String::new();
    {
        log::trace!("Locking file {} for read", path.display());
        let mut lock = file.lock()?;
        log::trace!("Successfully locked file {} for read", path.display());
        lock.read_to_string(&mut content)?;
    }
    log::trace!("Unlocked file {}", path.display());

    Ok(content)
}

/// Reads the file exclusively from the flie at the given path into a string.
///
/// During the read, the file is locked.
#[inline]
pub(crate) fn read_to_string<P: AsRef<Path>>(path: P) -> Result<String> {
    read_to_string_impl(path.as_ref())
}

/// Reads the file exclusively from the flie at the given path into a string.
///
/// During the read, the file is locked.
fn read_to_string_impl(path: &Path) -> Result<String> {
    let mut file = open(path)?;
    read_to_string_from_lockable_file(path, &mut file)
}

/// Writes the given content exclusively to the file at the given path.
///
/// During the write, the file is locked.
#[inline]
pub(crate) fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) -> Result<()> {
    write_impl(path.as_ref(), contents.as_ref())
}

/// Writes the given content exclusively to the file at the given path.
///
/// During the write, the file is locked.
fn write_impl(path: &Path, contents: &[u8]) -> Result<()> {
    let mut file = create(path)?;
    log::trace!("Locking file {} for write", path.display());
    {
        let mut lock = file.lock()?;
        log::trace!("Successfully locked file {} for write", path.display());
        lock.write_all(contents)?;
    }
    log::trace!("Unlocked file {}", path.display());

    Ok(())
}
