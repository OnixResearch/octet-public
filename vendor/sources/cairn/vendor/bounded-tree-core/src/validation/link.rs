//! Pure internal symbolic-link admission.

const PATH_SEPARATOR: u8 = b'/';
const CURRENT_COMPONENT: &[u8] = b".";
const PARENT_COMPONENT: &[u8] = b"..";

pub(super) fn check_symlinks(
    candidates: &mut [crate::admission::Candidate],
    limits: crate::TreeLimits,
    policy: crate::SymlinkPolicy,
    blockers: &mut crate::admission::BlockerSink,
) {
    debug_assert!(limits.symlink_target_bytes() > 0);
    debug_assert!(candidates.len() <= blockers.observation_count());
    let kinds = candidates
        .iter()
        .map(|candidate| (candidate.path.clone(), candidate.raw.kind))
        .collect::<alloc::collections::BTreeMap<_, _>>();
    for candidate in candidates {
        if candidate.raw.kind != crate::EntryKind::Symlink {
            continue;
        }
        if policy == crate::SymlinkPolicy::Reject {
            blockers.push(crate::admission::path_blocker(
                crate::BlockerKind::SymlinkRejected,
                &candidate.path,
            ));
            continue;
        }
        let Some(target) = candidate.raw.symlink_target.as_deref() else {
            continue;
        };
        let resolved = match resolve_target(&candidate.path, target, limits) {
            Ok(path) => path,
            Err(kind) => {
                blockers.push(crate::admission::path_blocker(kind, &candidate.path));
                continue;
            }
        };
        validate_resolved_target(candidate, resolved, &kinds, blockers);
    }
}

pub(super) fn validate_resolved_target(
    candidate: &mut crate::admission::Candidate,
    resolved: crate::RelativePath,
    kinds: &alloc::collections::BTreeMap<crate::RelativePath, crate::EntryKind>,
    blockers: &mut crate::admission::BlockerSink,
) {
    if resolved == candidate.path {
        blockers.push(crate::admission::path_blocker(
            crate::BlockerKind::SymlinkTargetSelf,
            &candidate.path,
        ));
        return;
    }
    if !kinds.contains_key(&resolved) {
        blockers.push(crate::admission::path_blocker(
            crate::BlockerKind::SymlinkTargetMissing,
            &candidate.path,
        ));
        return;
    }
    candidate.resolved_symlink_target = Some(resolved);
}

fn resolve_target(
    link: &crate::RelativePath,
    target: &[u8],
    limits: crate::TreeLimits,
) -> Result<crate::RelativePath, crate::BlockerKind> {
    debug_assert!(!link.components().is_empty());
    debug_assert!(limits.depth() > 0);
    validate_target_bytes(target, limits)?;
    let component_count = link
        .components()
        .len()
        .checked_add(target.len())
        .ok_or(crate::BlockerKind::PathAccountingOverflow)?;
    let parent_count = link.components().len().saturating_sub(1);
    let mut resolved = alloc::vec::Vec::with_capacity(component_count);
    resolved.extend_from_slice(&link.components()[..parent_count]);
    for component in target.split(|byte| *byte == PATH_SEPARATOR) {
        if component.is_empty() || component == CURRENT_COMPONENT {
            continue;
        }
        if component == PARENT_COMPONENT {
            resolved
                .pop()
                .ok_or(crate::BlockerKind::SymlinkTargetEscapes)?;
            continue;
        }
        resolved.push(component.to_vec());
    }
    admit_target(resolved, limits)
}

fn validate_target_bytes(
    target: &[u8],
    limits: crate::TreeLimits,
) -> Result<(), crate::BlockerKind> {
    let target_bytes =
        u64::try_from(target.len()).map_err(|_| crate::BlockerKind::SymlinkTargetBytesExceeded)?;
    if target_bytes > limits.symlink_target_bytes() {
        return Err(crate::BlockerKind::SymlinkTargetBytesExceeded);
    }
    if target.is_empty() || target[0] == PATH_SEPARATOR {
        return Err(crate::BlockerKind::SymlinkTargetNotRelative);
    }
    if target.contains(&0) {
        return Err(crate::BlockerKind::SymlinkTargetContainsNul);
    }
    if target.contains(&b'\\') {
        return Err(crate::BlockerKind::SymlinkTargetContainsBackslash);
    }
    Ok(())
}

fn admit_target(
    components: alloc::vec::Vec<alloc::vec::Vec<u8>>,
    limits: crate::TreeLimits,
) -> Result<crate::RelativePath, crate::BlockerKind> {
    let path = crate::RelativePath::new(components)
        .map_err(|_| crate::BlockerKind::SymlinkTargetMalformed)?;
    if path.encoded_bytes() > limits.path_bytes() {
        return Err(crate::BlockerKind::PathBytesExceeded);
    }
    let depth = path
        .depth()
        .map_err(|_| crate::BlockerKind::PathAccountingOverflow)?;
    if depth > limits.depth() {
        return Err(crate::BlockerKind::PathDepthExceeded);
    }
    Ok(path)
}
