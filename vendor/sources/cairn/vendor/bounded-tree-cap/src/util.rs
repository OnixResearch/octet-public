//! Capability-relative Unix helpers.

use std::io::{Read, Write};
use std::os::unix::ffi::OsStringExt;

use cap_fs_ext::OpenOptionsFollowExt;
use cap_std::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

use crate::prelude::{
    Blake3Digest, Dir, EntryKind, FileMode, FollowSymlinks, Metadata, OpenOptions, OperationKind,
    OsString, Path, Permissions, RelativePath, ShellError, ShellErrorKind, SourceChangeKind,
};

const STREAM_BUFFER_BYTES: usize = 8_192;
const UNIX_PERMISSION_BITS_MASK: u32 = 0o7_777;
const STAGING_FILE_MODE: u32 = 0o600;
const STAGING_DIRECTORY_MODE: u32 = 0o700;

pub fn metadata_kind(metadata: &Metadata) -> EntryKind {
    if metadata.file_type().is_symlink() {
        return EntryKind::Symlink;
    }
    if metadata.is_dir() {
        return EntryKind::Directory;
    }
    if metadata.is_file() {
        return EntryKind::File;
    }
    EntryKind::Unsupported
}

pub fn metadata_mode(metadata: &Metadata) -> FileMode {
    FileMode::new(metadata.mode())
}

pub fn open_parent(
    root: &Dir,
    path: &RelativePath,
    operation: OperationKind,
) -> Result<(Dir, OsString), ShellError> {
    let components = path.components();
    let name_bytes = components
        .last()
        .ok_or_else(|| ShellError::new(ShellErrorKind::UnsupportedPlatformPath, Vec::new()))?;
    let mut current = root
        .try_clone()
        .map_err(|error| ShellError::io(OperationKind::CloneCapability, &[], error))?;
    for component in &components[..components.len().saturating_sub(1)] {
        current =
            cap_fs_ext::DirExt::open_dir_nofollow(&current, OsString::from_vec(component.clone()))
                .map_err(|error| ShellError::io(operation, components, error))?;
    }
    Ok((current, OsString::from_vec(name_bytes.clone())))
}

pub fn open_file_read(
    parent: &Dir,
    name: impl AsRef<Path>,
    path: &[Vec<u8>],
) -> Result<cap_std::fs::File, ShellError> {
    let mut options = OpenOptions::new();
    options.read(true).follow(FollowSymlinks::No);
    parent
        .open_with(name, &options)
        .map_err(|error| ShellError::io(OperationKind::OpenFile, path, error))
}

pub fn open_file_create(
    parent: &Dir,
    name: impl AsRef<Path>,
    path: &[Vec<u8>],
) -> Result<cap_std::fs::File, ShellError> {
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create_new(true)
        .mode(STAGING_FILE_MODE)
        .follow(FollowSymlinks::No);
    parent
        .open_with(name, &options)
        .map_err(|error| ShellError::io(OperationKind::CreateDestination, path, error))
}

pub fn validate_metadata(
    metadata: &Metadata,
    expected_kind: EntryKind,
    expected_mode: FileMode,
    expected_bytes: u64,
    path: &[Vec<u8>],
) -> Result<(), ShellError> {
    validate_kind(metadata, expected_kind, path)?;
    validate_mode(metadata, expected_mode, path)?;
    validate_length(metadata, expected_kind, expected_bytes, path)
}

pub fn validate_kind(
    metadata: &Metadata,
    expected: EntryKind,
    path: &[Vec<u8>],
) -> Result<(), ShellError> {
    if metadata_kind(metadata) == expected {
        return Ok(());
    }
    Err(source_change_error(SourceChangeKind::Kind, path))
}

pub fn validate_mode(
    metadata: &Metadata,
    expected: FileMode,
    path: &[Vec<u8>],
) -> Result<(), ShellError> {
    if metadata_mode(metadata) == expected {
        return Ok(());
    }
    Err(source_change_error(SourceChangeKind::Mode, path))
}

pub fn validate_length(
    metadata: &Metadata,
    expected_kind: EntryKind,
    expected_bytes: u64,
    path: &[Vec<u8>],
) -> Result<(), ShellError> {
    if expected_kind != EntryKind::File || metadata.len() == expected_bytes {
        return Ok(());
    }
    Err(source_change_error(SourceChangeKind::Length, path))
}

pub fn source_change_error(kind: SourceChangeKind, path: &[Vec<u8>]) -> ShellError {
    ShellError::new(ShellErrorKind::SourceChanged(kind), path.to_vec())
}

pub fn hash_open_file(
    file: &mut cap_std::fs::File,
    expected_bytes: u64,
    path: &[Vec<u8>],
) -> Result<Blake3Digest, ShellError> {
    let mut hasher = blake3::Hasher::new();
    stream_exact(file, None, &mut hasher, expected_bytes, path)?;
    Ok(Blake3Digest::from_bytes(*hasher.finalize().as_bytes()))
}

pub fn copy_and_hash_open_file(
    source: &mut cap_std::fs::File,
    destination: &mut cap_std::fs::File,
    expected_bytes: u64,
    path: &[Vec<u8>],
) -> Result<Blake3Digest, ShellError> {
    let mut hasher = blake3::Hasher::new();
    stream_exact(source, Some(destination), &mut hasher, expected_bytes, path)?;
    destination
        .flush()
        .map_err(|error| ShellError::io(OperationKind::WriteDestination, path, error))?;
    Ok(Blake3Digest::from_bytes(*hasher.finalize().as_bytes()))
}

fn stream_exact(
    source: &mut cap_std::fs::File,
    mut destination: Option<&mut cap_std::fs::File>,
    hasher: &mut blake3::Hasher,
    expected_bytes: u64,
    path: &[Vec<u8>],
) -> Result<(), ShellError> {
    debug_assert!(!path.is_empty());
    let buffer_bytes = u64::try_from(STREAM_BUFFER_BYTES).map_err(|_| {
        ShellError::new(
            ShellErrorKind::LimitExceeded(bounded_tree_core::LimitKind::FileBytes),
            path.to_vec(),
        )
    })?;
    let mut buffer = [0_u8; STREAM_BUFFER_BYTES];
    let mut observed_bytes = 0_u64;
    debug_assert_eq!(observed_bytes, 0);
    while observed_bytes < expected_bytes {
        let remaining = expected_bytes.checked_sub(observed_bytes).ok_or_else(|| {
            ShellError::new(
                ShellErrorKind::SourceChanged(SourceChangeKind::Length),
                path.to_vec(),
            )
        })?;
        let read_byte_count = if remaining >= buffer_bytes {
            STREAM_BUFFER_BYTES
        } else {
            usize::try_from(remaining).map_err(|_| {
                ShellError::new(
                    ShellErrorKind::LimitExceeded(bounded_tree_core::LimitKind::FileBytes),
                    path.to_vec(),
                )
            })?
        };
        let count = source
            .read(&mut buffer[..read_byte_count])
            .map_err(|error| ShellError::io(OperationKind::ReadSource, path, error))?;
        if count == 0 {
            return Err(ShellError::new(
                ShellErrorKind::SourceChanged(SourceChangeKind::Length),
                path.to_vec(),
            ));
        }
        hasher.update(&buffer[..count]);
        if let Some(writer) = destination.as_deref_mut() {
            writer
                .write_all(&buffer[..count])
                .map_err(|error| ShellError::io(OperationKind::WriteDestination, path, error))?;
        }
        let count = u64::try_from(count).map_err(|_| {
            ShellError::new(
                ShellErrorKind::LimitExceeded(bounded_tree_core::LimitKind::FileBytes),
                path.to_vec(),
            )
        })?;
        observed_bytes = observed_bytes.checked_add(count).ok_or_else(|| {
            ShellError::new(
                ShellErrorKind::LimitExceeded(bounded_tree_core::LimitKind::FileBytes),
                path.to_vec(),
            )
        })?;
    }
    let extra_count = source
        .read(&mut buffer[..1])
        .map_err(|error| ShellError::io(OperationKind::ReadSource, path, error))?;
    if extra_count != 0 {
        return Err(ShellError::new(
            ShellErrorKind::SourceChanged(SourceChangeKind::Length),
            path.to_vec(),
        ));
    }
    Ok(())
}

pub fn set_file_mode(
    file: &cap_std::fs::File,
    mode: FileMode,
    path: &[Vec<u8>],
) -> Result<(), ShellError> {
    file.set_permissions(Permissions::from_mode(
        mode.bits() & UNIX_PERMISSION_BITS_MASK,
    ))
    .map_err(|error| ShellError::io(OperationKind::SetMode, path, error))
}

pub fn set_directory_mode(dir: &Dir, mode: FileMode, path: &[Vec<u8>]) -> Result<(), ShellError> {
    dir.set_permissions(
        ".",
        Permissions::from_mode(mode.bits() & UNIX_PERMISSION_BITS_MASK),
    )
    .map_err(|error| ShellError::io(OperationKind::SetMode, path, error))
}

pub const fn staging_directory_mode() -> FileMode {
    FileMode::new(STAGING_DIRECTORY_MODE)
}
