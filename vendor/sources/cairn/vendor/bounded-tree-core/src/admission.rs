//! Deterministic tree admission and typed blockers.

const BLOCKERS_PER_OBSERVATION: usize = 8;
const GLOBAL_BLOCKER_COUNT: usize = 2;

/// Typed deterministic plan blockers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum BlockerKind {
    /// The observed entry count exceeds the bound.
    EntryLimitExceeded,
    /// A path has no components.
    EmptyPath,
    /// A path component is empty.
    EmptyComponent,
    /// A path contains `.`.
    CurrentDirectoryComponent,
    /// A path contains `..`.
    ParentDirectoryComponent,
    /// A path contains NUL.
    PathContainsNul,
    /// A path component contains `/`.
    PathContainsForwardSlash,
    /// A path component contains `\\`.
    PathContainsBackslash,
    /// Path accounting overflowed.
    PathAccountingOverflow,
    /// A path exceeds the depth bound.
    PathDepthExceeded,
    /// A path exceeds the byte bound.
    PathBytesExceeded,
    /// A file exceeds the per-file bound.
    FileBytesExceeded,
    /// Files exceed the total byte bound.
    TotalBytesExceeded,
    /// Total file-byte accounting overflowed.
    TotalBytesOverflow,
    /// A symlink target exceeds its byte bound.
    SymlinkTargetBytesExceeded,
    /// An unsupported entry class was supplied.
    UnsupportedKind,
    /// Directory facts contain file or link data.
    InvalidDirectoryFacts,
    /// File facts omit content identity or contain link data.
    InvalidFileFacts,
    /// Symlink facts omit a target or contain file data.
    InvalidSymlinkFacts,
    /// Two observations have the same path.
    DuplicatePath,
    /// An entry has no admitted parent.
    MissingParent,
    /// An entry parent is not a directory.
    ParentNotDirectory,
    /// Policy rejects symbolic links.
    SymlinkRejected,
    /// A symlink target is empty or absolute.
    SymlinkTargetNotRelative,
    /// A symlink target contains NUL.
    SymlinkTargetContainsNul,
    /// A symlink target contains `\\`.
    SymlinkTargetContainsBackslash,
    /// A symlink target escapes the source root.
    SymlinkTargetEscapes,
    /// A symlink target cannot form an admitted path.
    SymlinkTargetMalformed,
    /// A symlink target is not an admitted member.
    SymlinkTargetMissing,
    /// A symlink resolves to itself.
    SymlinkTargetSelf,
}

/// One deterministic blocker with an optional exact path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Blocker {
    pub(crate) kind: BlockerKind,
    pub(crate) path_components: Option<alloc::vec::Vec<alloc::vec::Vec<u8>>>,
}

impl Blocker {
    /// Returns the blocker category.
    #[must_use]
    pub const fn kind(&self) -> BlockerKind {
        self.kind
    }

    /// Returns the blocked relative path, when one exists.
    #[must_use]
    pub fn path_components(&self) -> Option<&[alloc::vec::Vec<u8>]> {
        self.path_components.as_deref()
    }
}

/// Complete caller request to the pure planning core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeRequest {
    /// Owned observations.
    pub observations: alloc::vec::Vec<crate::RawObservation>,
    /// Explicit positive limits.
    pub limits: crate::TreeLimits,
    /// Explicit symlink policy.
    pub symlink_policy: crate::SymlinkPolicy,
}

/// Complete deterministic plan rejection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlanError {
    pub(crate) blockers: alloc::vec::Vec<Blocker>,
}

impl PlanError {
    /// Returns sorted typed blockers.
    #[must_use]
    pub fn blockers(&self) -> &[Blocker] {
        &self.blockers
    }

    pub(crate) fn global(kind: BlockerKind) -> Self {
        Self {
            blockers: alloc::vec![Blocker {
                kind,
                path_components: None,
            }],
        }
    }

    pub(crate) fn at_path(kind: BlockerKind, path: &crate::RelativePath) -> Self {
        Self {
            blockers: alloc::vec![path_blocker(kind, path)],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Candidate {
    pub path: crate::RelativePath,
    pub raw: crate::RawObservation,
    pub resolved_symlink_target: Option<crate::RelativePath>,
}

pub(crate) struct BlockerSink {
    blockers: alloc::vec::Vec<Blocker>,
    maximum: usize,
    observation_count: usize,
}

impl BlockerSink {
    fn new(observation_count: usize) -> Result<Self, PlanError> {
        let maximum = observation_count
            .checked_mul(BLOCKERS_PER_OBSERVATION)
            .and_then(|count| count.checked_add(GLOBAL_BLOCKER_COUNT))
            .ok_or_else(|| PlanError::global(BlockerKind::PathAccountingOverflow))?;
        Ok(Self {
            blockers: alloc::vec::Vec::with_capacity(maximum),
            maximum,
            observation_count,
        })
    }

    pub(crate) fn push(&mut self, blocker: Blocker) {
        debug_assert!(self.blockers.len() < self.maximum);
        debug_assert!(self.blockers.capacity() >= self.maximum);
        self.blockers.push(blocker);
    }

    pub(crate) const fn observation_count(&self) -> usize {
        self.observation_count
    }

    fn finish(mut self) -> Result<(), PlanError> {
        self.blockers.sort();
        if self.blockers.is_empty() {
            return Ok(());
        }
        Err(PlanError {
            blockers: self.blockers,
        })
    }
}

/// Admits supplied facts and returns one complete deterministic plan.
///
/// # Errors
///
/// Returns every deterministic blocker and no plan when any supplied fact fails.
// r[impl bounded_tree.mechanism.core_boundary]
// r[impl bounded_tree.mechanism.explicit_limits]
// r[impl bounded_tree.mechanism.tree_shape]
// r[impl bounded_tree.mechanism.symlink_policy]
// r[impl bounded_tree.mechanism.deterministic_plan]
// r[impl bounded_tree.identity.member_facts]
pub fn plan_tree(request: TreeRequest) -> Result<crate::TreePlan, PlanError> {
    admit_observation_count(&request.observations, request.limits)?;
    let mut blockers = BlockerSink::new(request.observations.len())?;
    let mut candidates = admit_observations(request.observations, request.limits, &mut blockers);
    candidates.sort_by(|left, right| left.path.cmp(&right.path));
    crate::validation::validate_all(
        &mut candidates,
        request.limits,
        request.symlink_policy,
        &mut blockers,
    );
    blockers.finish()?;
    crate::assembly::assemble(&candidates)
}

fn admit_observation_count(
    observations: &[crate::RawObservation],
    limits: crate::TreeLimits,
) -> Result<(), PlanError> {
    let count = u64::try_from(observations.len())
        .map_err(|_| PlanError::global(BlockerKind::EntryLimitExceeded))?;
    if count > limits.entries() {
        return Err(PlanError::global(BlockerKind::EntryLimitExceeded));
    }
    Ok(())
}

fn admit_observations(
    observations: alloc::vec::Vec<crate::RawObservation>,
    limits: crate::TreeLimits,
    blockers: &mut BlockerSink,
) -> alloc::vec::Vec<Candidate> {
    debug_assert!(observations.len() <= blockers.observation_count());
    debug_assert!(limits.entries() > 0);
    let mut candidates = alloc::vec::Vec::with_capacity(observations.len());
    let mut total_file_bytes = 0_u64;
    for raw in observations {
        let path = match crate::RelativePath::new(raw.path_components.clone()) {
            Ok(path) => path,
            Err(error) => {
                blockers.push(Blocker {
                    kind: path_error_kind(error),
                    path_components: Some(raw.path_components),
                });
                continue;
            }
        };
        crate::validation::validate_observation(
            &raw,
            &path,
            limits,
            &mut total_file_bytes,
            blockers,
        );
        candidates.push(Candidate {
            path,
            raw,
            resolved_symlink_target: None,
        });
    }
    crate::validation::validate_total_bytes(total_file_bytes, limits, blockers);
    candidates
}

pub(crate) fn path_error_kind(error: crate::PathError) -> BlockerKind {
    if error == crate::PathError::EmptyPath {
        return BlockerKind::EmptyPath;
    }
    if error == crate::PathError::EmptyComponent {
        return BlockerKind::EmptyComponent;
    }
    if error == crate::PathError::CurrentDirectory {
        return BlockerKind::CurrentDirectoryComponent;
    }
    if error == crate::PathError::ParentDirectory {
        return BlockerKind::ParentDirectoryComponent;
    }
    path_byte_error_kind(error)
}

pub(crate) fn path_byte_error_kind(error: crate::PathError) -> BlockerKind {
    if error == crate::PathError::Nul {
        return BlockerKind::PathContainsNul;
    }
    if error == crate::PathError::ForwardSlash {
        return BlockerKind::PathContainsForwardSlash;
    }
    if error == crate::PathError::Backslash {
        return BlockerKind::PathContainsBackslash;
    }
    BlockerKind::PathAccountingOverflow
}

pub(crate) fn path_blocker(kind: BlockerKind, path: &crate::RelativePath) -> Blocker {
    Blocker {
        kind,
        path_components: Some(path.components().to_vec()),
    }
}
