use std::{
    ffi::OsString,
    io,
    path::{Path, PathBuf},
};

use yiyin_application::{ApplicationError, CancellationProbe};

use crate::FileSystem;

/// Publishes `contents` to `destination` durably: stale-temporary cleanup,
/// temporary write + sync, an optional validity-checked `.bak` backup of the
/// prior file, an optional cancellation checkpoint, an atomic rename, and a
/// parent directory sync.
///
/// Rollback is unified for every caller: any failure before the rename
/// removes the temporary and leaves a prior destination untouched; a
/// `sync_parent` failure after the rename restores the backup when one was
/// made and otherwise removes the destination, so a failed publish never
/// leaves new bytes behind.
///
/// # Errors
///
/// Returns `INTERNAL` on filesystem failures, `FORBIDDEN` when the temporary
/// sibling would escape the destination's parent directory, and `CANCELLED`
/// when the optional probe reports cancellation before the rename.
#[allow(
    clippy::type_complexity,
    reason = "the optional validity callback keeps the shared signature explicit at every call site"
)]
pub fn durable_publish(
    filesystem: &dyn FileSystem,
    destination: &Path,
    contents: &[u8],
    prior_is_valid: Option<&dyn Fn(&[u8]) -> bool>,
    cancellation: Option<&dyn CancellationProbe>,
) -> Result<(), ApplicationError> {
    let parent = destination
        .parent()
        .ok_or_else(|| ApplicationError::internal("publish destination has no parent"))?;
    filesystem.create_dir_all(parent).map_err(internal_io)?;

    let temporary = with_suffix(destination, ".tmp");
    if temporary.parent() != Some(parent) {
        return Err(ApplicationError::forbidden());
    }
    remove_if_present(filesystem, &temporary)?;

    if let Err(error) = filesystem.write_and_sync(&temporary, contents) {
        let _ = remove_if_present(filesystem, &temporary);
        return Err(internal_io(error));
    }

    let backup = with_suffix(destination, ".bak");
    let prior_valid = prior_is_valid.and_then(|prior_is_valid| {
        filesystem
            .read(destination)
            .ok()
            .filter(|bytes| prior_is_valid(bytes))
    });
    if prior_valid.is_some()
        && let Err(error) = filesystem.copy(destination, &backup)
    {
        let _ = remove_if_present(filesystem, &temporary);
        return Err(internal_io(error));
    }

    if let Some(cancellation) = cancellation
        && cancellation.is_cancelled()
    {
        let _ = remove_if_present(filesystem, &temporary);
        return Err(ApplicationError::cancelled());
    }

    if let Err(error) = filesystem.rename(&temporary, destination) {
        let _ = remove_if_present(filesystem, &temporary);
        return Err(internal_io(error));
    }

    if let Err(error) = filesystem.sync_parent(destination) {
        if prior_valid.is_some() && filesystem.exists(&backup) {
            let _ = filesystem.copy(&backup, destination);
        } else {
            let _ = remove_if_present(filesystem, destination);
        }
        return Err(internal_io(error));
    }
    Ok(())
}

pub(crate) fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path
        .file_name()
        .map_or_else(OsString::new, std::ffi::OsStr::to_os_string);
    name.push(suffix);
    path.with_file_name(name)
}

pub(crate) fn remove_if_present(
    filesystem: &dyn FileSystem,
    path: &Path,
) -> Result<(), ApplicationError> {
    if filesystem.exists(path) {
        filesystem.remove_file(path).map_err(internal_io)?;
    }
    Ok(())
}

#[allow(
    clippy::needless_pass_by_value,
    reason = "the function is passed directly to Result::map_err"
)]
fn internal_io(error: io::Error) -> ApplicationError {
    ApplicationError::internal(error.to_string())
}
