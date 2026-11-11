use std::{
    io,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::support::CapabilityRoot;

pub(super) const COMPACTION_JSON_MAX_BYTES: u64 = 16_777_216;

pub(super) fn read_json<T>(
    repository: &CapabilityRoot,
    relative: &Path,
) -> Result<T, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    let bytes = repository.read_bounded(relative, COMPACTION_JSON_MAX_BYTES)?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(super) fn read_json_if_exists<T>(
    repository: &CapabilityRoot,
    relative: &Path,
) -> Result<Option<T>, Box<dyn std::error::Error>>
where
    T: serde::de::DeserializeOwned,
{
    match repository.read_bounded(relative, COMPACTION_JSON_MAX_BYTES) {
        Ok(bytes) => Ok(Some(serde_json::from_slice(&bytes)?)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn write_json_atomic(
    repository: &CapabilityRoot,
    relative: &Path,
    value: &Value,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = serde_json::to_vec_pretty(value)?;
    bytes.push(b'\n');
    write_bytes_atomic(repository, relative, &bytes)
}

pub(super) fn write_bytes_atomic(
    repository: &CapabilityRoot,
    relative: &Path,
    bytes: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let parent = relative
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    repository.create_dir_all(parent)?;
    reject_symlink_destination(repository, relative)?;
    let temporary = temporary_path(relative)?;
    remove_if_present(repository, &temporary)?;
    repository.write(&temporary, bytes)?;
    repository.rename(&temporary, repository, relative)?;
    Ok(())
}

pub(super) fn path_exists(repository: &CapabilityRoot, relative: &Path) -> Result<bool, io::Error> {
    match repository.symlink_metadata(relative) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

pub(super) fn remove_if_present(
    repository: &CapabilityRoot,
    relative: &Path,
) -> Result<(), io::Error> {
    match repository.remove_file(relative) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(super) fn reject_symlink_destination(
    repository: &CapabilityRoot,
    relative: &Path,
) -> Result<(), io::Error> {
    match repository.symlink_metadata(relative) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "archive compaction refuses symlink destinations",
        )),
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn temporary_path(relative: &Path) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let file_name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("archive compaction output path requires a UTF-8 file name")?;
    Ok(relative.with_file_name(format!("{file_name}.cairn-tmp")))
}
