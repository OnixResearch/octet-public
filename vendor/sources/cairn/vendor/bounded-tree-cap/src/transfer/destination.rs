//! Empty-destination creation and final mode application.

use std::os::unix::ffi::OsStringExt;

use crate::prelude::{
    Blake3Digest, Dir, EntryKind, ErrorKind, FileMode, OperationKind, OsString, PreparedTree,
    RelativePath, ShellError, ShellErrorKind, SourceChangeKind, copy_and_hash_open_file,
    open_file_create, open_file_read, open_parent, set_directory_mode, set_file_mode,
    staging_directory_mode, validate_metadata,
};

pub(super) struct FileCopy<'a> {
    pub source: &'a Dir,
    pub destination: &'a Dir,
    pub path: &'a RelativePath,
    pub mode: FileMode,
    pub file_bytes: u64,
    pub expected_digest: Blake3Digest,
}

pub(super) fn require_empty(destination: &Dir) -> Result<(), ShellError> {
    let mut entries = destination
        .entries()
        .map_err(|error| ShellError::io(OperationKind::ReadDirectory, &[], error))?;
    match entries.next() {
        None => Ok(()),
        Some(Ok(_)) => Err(ShellError::new(
            ShellErrorKind::DestinationNotEmpty,
            Vec::new(),
        )),
        Some(Err(error)) => Err(ShellError::io(OperationKind::ReadDirectory, &[], error)),
    }
}

pub(super) fn create_directory(destination: &Dir, path: &RelativePath) -> Result<(), ShellError> {
    let (parent, name) = open_parent(destination, path, OperationKind::OpenDirectory)?;
    require_absent(&parent, &name, path)?;
    parent.create_dir(&name).map_err(|error| {
        ShellError::io(OperationKind::CreateDestination, path.components(), error)
    })?;
    let created = cap_fs_ext::DirExt::open_dir_nofollow(&parent, &name)
        .map_err(|error| ShellError::io(OperationKind::OpenDirectory, path.components(), error))?;
    set_directory_mode(&created, staging_directory_mode(), path.components())
}

pub(super) fn copy_file(request: &FileCopy<'_>) -> Result<u64, ShellError> {
    debug_assert!(!request.path.components().is_empty());
    debug_assert!(request.mode.bits() > 0);
    let mut source_file = open_validated_source(request)?;
    let (destination_parent, destination_name) = open_parent(
        request.destination,
        request.path,
        OperationKind::OpenDirectory,
    )?;
    require_absent(&destination_parent, &destination_name, request.path)?;
    let mut destination_file = open_file_create(
        &destination_parent,
        &destination_name,
        request.path.components(),
    )?;
    let observed_digest = copy_and_hash_open_file(
        &mut source_file,
        &mut destination_file,
        request.file_bytes,
        request.path.components(),
    )?;
    verify_digest(observed_digest, request)?;
    set_file_mode(&destination_file, request.mode, request.path.components())?;
    Ok(request.file_bytes)
}

fn open_validated_source(request: &FileCopy<'_>) -> Result<cap_std::fs::File, ShellError> {
    let (parent, name) = open_parent(request.source, request.path, OperationKind::OpenDirectory)?;
    let file = open_file_read(&parent, &name, request.path.components())?;
    let metadata = file.metadata().map_err(|error| {
        ShellError::io(
            OperationKind::ReadMetadata,
            request.path.components(),
            error,
        )
    })?;
    validate_metadata(
        &metadata,
        EntryKind::File,
        request.mode,
        request.file_bytes,
        request.path.components(),
    )?;
    Ok(file)
}

pub(super) fn verify_digest(
    observed: Blake3Digest,
    request: &FileCopy<'_>,
) -> Result<(), ShellError> {
    if observed == request.expected_digest {
        return Ok(());
    }
    Err(ShellError::new(
        ShellErrorKind::SourceChanged(SourceChangeKind::Content),
        request.path.components().to_vec(),
    ))
}

pub(super) fn create_symlink(
    destination: &Dir,
    path: &RelativePath,
    target: &[u8],
) -> Result<(), ShellError> {
    let (parent, name) = open_parent(destination, path, OperationKind::OpenDirectory)?;
    require_absent(&parent, &name, path)?;
    cap_fs_ext::DirExt::symlink(&parent, OsString::from_vec(target.to_vec()), &name)
        .map_err(|error| ShellError::io(OperationKind::CreateDestination, path.components(), error))
}

pub(super) fn require_absent(
    parent: &Dir,
    name: &OsString,
    path: &RelativePath,
) -> Result<(), ShellError> {
    match parent.symlink_metadata(name) {
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(()),
        Err(error) => Err(ShellError::io(
            OperationKind::ReadMetadata,
            path.components(),
            error,
        )),
        Ok(_) => Err(ShellError::new(
            ShellErrorKind::DestinationNotEmpty,
            path.components().to_vec(),
        )),
    }
}

pub(super) fn finalize_directory_modes(
    prepared: &PreparedTree,
    destination: &Dir,
) -> Result<(), ShellError> {
    debug_assert_eq!(
        prepared.plan.member_facts().len(),
        prepared.plan.operations().len()
    );
    let mut directories = prepared
        .plan
        .member_facts()
        .iter()
        .filter(|member| member.kind() == EntryKind::Directory)
        .collect::<Vec<_>>();
    debug_assert!(directories.len() <= prepared.plan.member_facts().len());
    directories.sort_by(|left, right| right.path().cmp(left.path()));
    for member in directories {
        let (parent, name) = open_parent(destination, member.path(), OperationKind::OpenDirectory)?;
        let dir = cap_fs_ext::DirExt::open_dir_nofollow(&parent, name).map_err(|error| {
            ShellError::io(
                OperationKind::OpenDirectory,
                member.path().components(),
                error,
            )
        })?;
        set_directory_mode(&dir, member.mode(), member.path().components())?;
    }
    Ok(())
}
