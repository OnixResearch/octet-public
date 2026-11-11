//! Pure shape and member-fact validation.

mod link;

pub(crate) fn validate_all(
    candidates: &mut [crate::admission::Candidate],
    limits: crate::TreeLimits,
    policy: crate::SymlinkPolicy,
    blockers: &mut crate::admission::BlockerSink,
) {
    validate_duplicates(candidates, blockers);
    validate_parent_shapes(candidates, blockers);
    link::check_symlinks(candidates, limits, policy, blockers);
}

pub(crate) fn validate_observation(
    raw: &crate::RawObservation,
    path: &crate::RelativePath,
    limits: crate::TreeLimits,
    total_file_bytes: &mut u64,
    blockers: &mut crate::admission::BlockerSink,
) {
    validate_path_limits(path, limits, blockers);
    validate_entry_facts(raw, path, limits, total_file_bytes, blockers);
}

pub(crate) fn validate_total_bytes(
    total_file_bytes: u64,
    limits: crate::TreeLimits,
    blockers: &mut crate::admission::BlockerSink,
) {
    if total_file_bytes > limits.total_bytes() {
        blockers.push(crate::admission::Blocker {
            kind: crate::BlockerKind::TotalBytesExceeded,
            path_components: None,
        });
    }
}

pub(crate) fn validate_path_limits(
    path: &crate::RelativePath,
    limits: crate::TreeLimits,
    blockers: &mut crate::admission::BlockerSink,
) {
    if path.encoded_bytes() > limits.path_bytes() {
        blockers.push(crate::admission::path_blocker(
            crate::BlockerKind::PathBytesExceeded,
            path,
        ));
    }
    match path.depth() {
        Ok(depth) if depth > limits.depth() => blockers.push(crate::admission::path_blocker(
            crate::BlockerKind::PathDepthExceeded,
            path,
        )),
        Err(_) => blockers.push(crate::admission::path_blocker(
            crate::BlockerKind::PathAccountingOverflow,
            path,
        )),
        Ok(_) => {}
    }
}

pub(crate) fn validate_entry_facts(
    raw: &crate::RawObservation,
    path: &crate::RelativePath,
    limits: crate::TreeLimits,
    total_file_bytes: &mut u64,
    blockers: &mut crate::admission::BlockerSink,
) {
    if raw.kind == crate::EntryKind::Directory {
        validate_directory_facts(raw, path, blockers);
        return;
    }
    if raw.kind == crate::EntryKind::File {
        validate_file_facts(raw, path, limits, total_file_bytes, blockers);
        return;
    }
    if raw.kind == crate::EntryKind::Symlink {
        validate_symlink_facts(raw, path, blockers);
        return;
    }
    blockers.push(crate::admission::path_blocker(
        crate::BlockerKind::UnsupportedKind,
        path,
    ));
}

pub(crate) fn validate_directory_facts(
    raw: &crate::RawObservation,
    path: &crate::RelativePath,
    blockers: &mut crate::admission::BlockerSink,
) {
    if raw.file_bytes != 0 || raw.file_content.is_some() || raw.symlink_target.is_some() {
        blockers.push(crate::admission::path_blocker(
            crate::BlockerKind::InvalidDirectoryFacts,
            path,
        ));
    }
}

pub(crate) fn validate_file_facts(
    raw: &crate::RawObservation,
    path: &crate::RelativePath,
    limits: crate::TreeLimits,
    total_file_bytes: &mut u64,
    blockers: &mut crate::admission::BlockerSink,
) {
    debug_assert_eq!(raw.kind, crate::EntryKind::File);
    debug_assert!(limits.file_bytes() > 0);
    if raw.file_content.is_none() || raw.symlink_target.is_some() {
        blockers.push(crate::admission::path_blocker(
            crate::BlockerKind::InvalidFileFacts,
            path,
        ));
    }
    if raw.file_bytes > limits.file_bytes() {
        blockers.push(crate::admission::path_blocker(
            crate::BlockerKind::FileBytesExceeded,
            path,
        ));
    }
    match total_file_bytes.checked_add(raw.file_bytes) {
        Some(total) => *total_file_bytes = total,
        None => blockers.push(crate::admission::Blocker {
            kind: crate::BlockerKind::TotalBytesOverflow,
            path_components: None,
        }),
    }
}

pub(crate) fn validate_symlink_facts(
    raw: &crate::RawObservation,
    path: &crate::RelativePath,
    blockers: &mut crate::admission::BlockerSink,
) {
    if raw.file_bytes != 0 || raw.file_content.is_some() || raw.symlink_target.is_none() {
        blockers.push(crate::admission::path_blocker(
            crate::BlockerKind::InvalidSymlinkFacts,
            path,
        ));
    }
}

fn validate_duplicates(
    candidates: &[crate::admission::Candidate],
    blockers: &mut crate::admission::BlockerSink,
) {
    for pair in candidates.windows(2) {
        if pair[0].path == pair[1].path {
            blockers.push(crate::admission::path_blocker(
                crate::BlockerKind::DuplicatePath,
                &pair[0].path,
            ));
        }
    }
}

fn validate_parent_shapes(
    candidates: &[crate::admission::Candidate],
    blockers: &mut crate::admission::BlockerSink,
) {
    debug_assert!(candidates.len() <= blockers.observation_count());
    let kinds = candidates
        .iter()
        .map(|candidate| (candidate.path.clone(), candidate.raw.kind))
        .collect::<alloc::collections::BTreeMap<_, _>>();
    debug_assert!(kinds.len() <= candidates.len());
    for candidate in candidates {
        let parent = match candidate.path.parent() {
            Ok(Some(parent)) => parent,
            Ok(None) => continue,
            Err(_) => {
                blockers.push(crate::admission::path_blocker(
                    crate::BlockerKind::PathAccountingOverflow,
                    &candidate.path,
                ));
                continue;
            }
        };
        match kinds.get(&parent) {
            None => blockers.push(crate::admission::path_blocker(
                crate::BlockerKind::MissingParent,
                &candidate.path,
            )),
            Some(crate::EntryKind::Directory) => {}
            Some(_) => blockers.push(crate::admission::path_blocker(
                crate::BlockerKind::ParentNotDirectory,
                &candidate.path,
            )),
        }
    }
}
