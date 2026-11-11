//! Prepared-source revalidation before destination mutation.

use crate::prelude::{
    Dir, EntryKind, ErrorKind, MemberFact, Metadata, OperationKind, OsString, PreparedTree,
    RelativePath, ShellError, ShellErrorKind, SourceChangeKind, hash_open_file, open_file_read,
    open_parent, validate_metadata,
};

pub(super) fn all(prepared: &PreparedTree) -> Result<(), ShellError> {
    for member in prepared.plan.member_facts() {
        member_fact(&prepared.source, member)?;
    }
    Ok(())
}

fn member_fact(source: &Dir, member: &MemberFact) -> Result<(), ShellError> {
    let path = member.path();
    let (parent, name) = open_parent(source, path, OperationKind::OpenDirectory)?;
    let metadata = source_metadata(&parent, &name, path)?;
    validate_metadata(
        &metadata,
        member.kind(),
        member.mode(),
        member.file_bytes(),
        path.components(),
    )?;
    member_content(&parent, &name, member)
}

pub(super) fn member_content(
    parent: &Dir,
    name: &OsString,
    member: &MemberFact,
) -> Result<(), ShellError> {
    if member.kind() == EntryKind::Directory {
        return Ok(());
    }
    if member.kind() == EntryKind::File {
        return file(parent, name, member);
    }
    if member.kind() == EntryKind::Symlink {
        return symlink(parent, name, member);
    }
    Err(ShellError::new(
        ShellErrorKind::SourceChanged(SourceChangeKind::Kind),
        member.path().components().to_vec(),
    ))
}

pub(super) fn source_metadata(
    parent: &Dir,
    name: &OsString,
    path: &RelativePath,
) -> Result<Metadata, ShellError> {
    match parent.symlink_metadata(name) {
        Ok(metadata) => Ok(metadata),
        Err(error) if error.kind() == ErrorKind::NotFound => Err(ShellError::new(
            ShellErrorKind::SourceChanged(SourceChangeKind::Missing),
            path.components().to_vec(),
        )),
        Err(error) => Err(ShellError::io(
            OperationKind::ReadMetadata,
            path.components(),
            error,
        )),
    }
}

fn file(parent: &Dir, name: &OsString, member: &MemberFact) -> Result<(), ShellError> {
    let path = member.path();
    debug_assert!(!path.components().is_empty());
    debug_assert!(member.file_content().is_some());
    let mut opened_file = open_file_read(parent, name, path.components())?;
    let metadata = opened_file
        .metadata()
        .map_err(|error| ShellError::io(OperationKind::ReadMetadata, path.components(), error))?;
    validate_metadata(
        &metadata,
        EntryKind::File,
        member.mode(),
        member.file_bytes(),
        path.components(),
    )?;
    let digest = hash_open_file(&mut opened_file, member.file_bytes(), path.components())?;
    if member.file_content() != Some(digest) {
        return Err(ShellError::new(
            ShellErrorKind::SourceChanged(SourceChangeKind::Content),
            path.components().to_vec(),
        ));
    }
    Ok(())
}

fn symlink(parent: &Dir, name: &OsString, member: &MemberFact) -> Result<(), ShellError> {
    use std::os::unix::ffi::OsStrExt;

    let path = member.path();
    let target = parent
        .read_link_contents(name)
        .map_err(|error| ShellError::io(OperationKind::ReadSource, path.components(), error))?;
    if member.symlink_target() != Some(target.as_os_str().as_bytes()) {
        return Err(ShellError::new(
            ShellErrorKind::SourceChanged(SourceChangeKind::SymlinkTarget),
            path.components().to_vec(),
        ));
    }
    Ok(())
}
