//! Bounded observation of one no-follow directory entry.

use std::os::unix::ffi::OsStrExt;

use crate::prelude::{
    Dir, EntryKind, FileMode, LimitKind, Metadata, OperationKind, OsString, RawObservation,
    ShellError, ShellErrorKind, TreeLimits, hash_open_file, metadata_kind, metadata_mode,
    open_file_read, validate_metadata,
};

pub(super) struct ObservationInput<'a> {
    pub parent: &'a Dir,
    pub name: &'a OsString,
    pub path: Vec<Vec<u8>>,
    pub metadata: &'a Metadata,
    pub limits: TreeLimits,
    pub total_file_bytes: &'a mut u64,
}

pub(super) fn observe(context: ObservationInput<'_>) -> Result<RawObservation, ShellError> {
    let kind = metadata_kind(context.metadata);
    let mode = metadata_mode(context.metadata);
    if kind == EntryKind::Directory {
        return Ok(directory(context.path, mode));
    }
    if kind == EntryKind::File {
        return file(context, mode);
    }
    if kind == EntryKind::Symlink {
        return symlink(context, mode);
    }
    Ok(unsupported(context.path, mode))
}

pub(super) fn directory(path_components: Vec<Vec<u8>>, mode: FileMode) -> RawObservation {
    RawObservation {
        path_components,
        kind: EntryKind::Directory,
        mode,
        file_bytes: 0,
        symlink_target: None,
        file_content: None,
    }
}

pub(super) fn unsupported(path_components: Vec<Vec<u8>>, mode: FileMode) -> RawObservation {
    RawObservation {
        path_components,
        kind: EntryKind::Unsupported,
        mode,
        file_bytes: 0,
        symlink_target: None,
        file_content: None,
    }
}

fn file(context: ObservationInput<'_>, mode: FileMode) -> Result<RawObservation, ShellError> {
    debug_assert_eq!(metadata_kind(context.metadata), EntryKind::File);
    debug_assert!(context.limits.file_bytes() > 0);
    let file_bytes = context.metadata.len();
    if file_bytes > context.limits.file_bytes() {
        return Err(limit_error(LimitKind::FileBytes, context.path));
    }
    let next_total = context
        .total_file_bytes
        .checked_add(file_bytes)
        .ok_or_else(|| limit_error(LimitKind::TotalBytes, context.path.clone()))?;
    if next_total > context.limits.total_bytes() {
        return Err(limit_error(LimitKind::TotalBytes, context.path));
    }
    let mut opened_file = open_file_read(context.parent, context.name, &context.path)?;
    let opened_metadata = opened_file
        .metadata()
        .map_err(|error| ShellError::io(OperationKind::ReadMetadata, &context.path, error))?;
    validate_metadata(
        &opened_metadata,
        EntryKind::File,
        mode,
        file_bytes,
        &context.path,
    )?;
    let file_content = hash_open_file(&mut opened_file, file_bytes, &context.path)?;
    *context.total_file_bytes = next_total;
    Ok(RawObservation {
        path_components: context.path,
        kind: EntryKind::File,
        mode,
        file_bytes,
        symlink_target: None,
        file_content: Some(file_content),
    })
}

fn symlink(context: ObservationInput<'_>, mode: FileMode) -> Result<RawObservation, ShellError> {
    debug_assert_eq!(metadata_kind(context.metadata), EntryKind::Symlink);
    debug_assert!(context.limits.symlink_target_bytes() > 0);
    let target = context
        .parent
        .read_link_contents(context.name)
        .map_err(|error| ShellError::io(OperationKind::ReadSource, &context.path, error))?;
    let target = target.as_os_str().as_bytes().to_vec();
    let target_bytes = u64::try_from(target.len())
        .map_err(|_| limit_error(LimitKind::SymlinkTargetBytes, context.path.clone()))?;
    if target_bytes > context.limits.symlink_target_bytes() {
        return Err(limit_error(LimitKind::SymlinkTargetBytes, context.path));
    }
    Ok(RawObservation {
        path_components: context.path,
        kind: EntryKind::Symlink,
        mode,
        file_bytes: 0,
        symlink_target: Some(target),
        file_content: None,
    })
}

fn limit_error(kind: LimitKind, path: Vec<Vec<u8>>) -> ShellError {
    ShellError::new(ShellErrorKind::LimitExceeded(kind), path)
}
