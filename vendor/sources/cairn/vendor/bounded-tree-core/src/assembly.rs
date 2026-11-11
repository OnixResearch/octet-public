//! Deterministic member-fact and operation assembly.

pub(crate) fn assemble(
    candidates: &[crate::admission::Candidate],
) -> Result<crate::TreePlan, crate::PlanError> {
    debug_assert!(
        candidates
            .windows(2)
            .all(|pair| pair[0].path <= pair[1].path)
    );
    debug_assert!(candidates.iter().all(|candidate| {
        candidate.raw.kind != crate::EntryKind::Symlink
            || candidate.resolved_symlink_target.is_some()
    }));
    let member_facts = candidates
        .iter()
        .map(candidate_member_fact)
        .collect::<alloc::vec::Vec<_>>();
    let operations = assemble_operations(&member_facts)?;
    Ok(crate::TreePlan::new(member_facts, operations))
}

fn assemble_operations(
    member_facts: &[crate::MemberFact],
) -> Result<alloc::vec::Vec<crate::CopyOperation>, crate::PlanError> {
    debug_assert!(
        !member_facts
            .iter()
            .any(|member| { member.kind() == crate::EntryKind::Unsupported })
    );
    debug_assert!(
        member_facts
            .windows(2)
            .all(|pair| pair[0].path() <= pair[1].path())
    );
    let operation_count = member_facts.len();
    let mut directories = alloc::vec::Vec::with_capacity(operation_count);
    let mut files = alloc::vec::Vec::with_capacity(operation_count);
    let mut symlinks = alloc::vec::Vec::with_capacity(operation_count);
    for member in member_facts {
        if member.kind() == crate::EntryKind::Directory {
            directories.push(directory_operation(member));
            continue;
        }
        if member.kind() == crate::EntryKind::File {
            files.push(file_operation(member)?);
            continue;
        }
        if member.kind() == crate::EntryKind::Symlink {
            symlinks.push(symlink_operation(member)?);
            continue;
        }
        return Err(invariant_error(
            crate::BlockerKind::UnsupportedKind,
            member.path(),
        ));
    }
    directories.sort_by(|left, right| {
        operation_depth(left)
            .cmp(&operation_depth(right))
            .then_with(|| left.path().cmp(right.path()))
    });
    files.sort_by(|left, right| left.path().cmp(right.path()));
    symlinks.sort_by(|left, right| left.path().cmp(right.path()));
    merge_operations(directories, files, symlinks)
}

fn merge_operations(
    directories: alloc::vec::Vec<crate::CopyOperation>,
    files: alloc::vec::Vec<crate::CopyOperation>,
    symlinks: alloc::vec::Vec<crate::CopyOperation>,
) -> Result<alloc::vec::Vec<crate::CopyOperation>, crate::PlanError> {
    let operation_count = directories
        .len()
        .checked_add(files.len())
        .and_then(|count| count.checked_add(symlinks.len()))
        .ok_or_else(|| crate::PlanError::global(crate::BlockerKind::PathAccountingOverflow))?;
    let mut operations = alloc::vec::Vec::with_capacity(operation_count);
    operations.extend(directories);
    operations.extend(files);
    operations.extend(symlinks);
    Ok(operations)
}

fn candidate_member_fact(candidate: &crate::admission::Candidate) -> crate::MemberFact {
    crate::MemberFact::from_parts(crate::model::MemberFactParts {
        path: candidate.path.clone(),
        kind: candidate.raw.kind,
        mode: candidate.raw.mode,
        file_bytes: candidate.raw.file_bytes,
        symlink_target: candidate.raw.symlink_target.clone(),
        resolved_symlink_target: candidate.resolved_symlink_target.clone(),
        file_content: candidate.raw.file_content,
    })
}

fn directory_operation(member: &crate::MemberFact) -> crate::CopyOperation {
    crate::CopyOperation::CreateDirectory {
        path: member.path().clone(),
        mode: member.mode(),
    }
}

fn file_operation(member: &crate::MemberFact) -> Result<crate::CopyOperation, crate::PlanError> {
    let file_content = member
        .file_content()
        .ok_or_else(|| invariant_error(crate::BlockerKind::InvalidFileFacts, member.path()))?;
    Ok(crate::CopyOperation::CopyFile {
        path: member.path().clone(),
        mode: member.mode(),
        file_bytes: member.file_bytes(),
        file_content,
    })
}

fn symlink_operation(member: &crate::MemberFact) -> Result<crate::CopyOperation, crate::PlanError> {
    let target = member
        .symlink_target()
        .ok_or_else(|| invariant_error(crate::BlockerKind::InvalidSymlinkFacts, member.path()))?
        .to_vec();
    let resolved_target = member
        .resolved_symlink_target()
        .ok_or_else(|| invariant_error(crate::BlockerKind::InvalidSymlinkFacts, member.path()))?
        .clone();
    Ok(crate::CopyOperation::CreateSymlink {
        path: member.path().clone(),
        target,
        resolved_target,
    })
}

fn operation_depth(operation: &crate::CopyOperation) -> usize {
    operation.path().components().len()
}

fn invariant_error(kind: crate::BlockerKind, path: &crate::RelativePath) -> crate::PlanError {
    crate::PlanError::at_path(kind, path)
}
