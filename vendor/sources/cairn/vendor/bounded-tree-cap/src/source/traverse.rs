//! Iterative no-follow directory traversal under explicit limits.

use std::os::unix::ffi::OsStrExt;

use crate::prelude::{
    Dir, EntryKind, LimitKind, OperationKind, OsString, RawObservation, RelativePath, ShellError,
    ShellErrorKind, TreeLimits,
};

struct PendingDirectory {
    path_components: Vec<Vec<u8>>,
    dir: Dir,
}

struct ObservationState {
    pending: Vec<PendingDirectory>,
    observations: Vec<RawObservation>,
    total_file_bytes: u64,
}

impl ObservationState {
    fn new(root: Dir) -> Self {
        Self {
            pending: vec![PendingDirectory {
                path_components: Vec::new(),
                dir: root,
            }],
            observations: Vec::new(),
            total_file_bytes: 0,
        }
    }

    fn push_pending(&mut self, directory: PendingDirectory) {
        self.pending.push(directory);
    }

    fn push_observation(&mut self, observation: RawObservation) {
        self.observations.push(observation);
    }
}

pub(super) fn observe_tree(
    source: &Dir,
    limits: TreeLimits,
) -> Result<Vec<RawObservation>, ShellError> {
    let root = source
        .try_clone()
        .map_err(|error| ShellError::io(OperationKind::CloneCapability, &[], error))?;
    let mut state = ObservationState::new(root);
    while let Some(directory) = state.pending.pop() {
        observe_directory(&directory, limits, &mut state)?;
    }
    Ok(state.observations)
}

fn observe_directory(
    directory: &PendingDirectory,
    limits: TreeLimits,
    state: &mut ObservationState,
) -> Result<(), ShellError> {
    debug_assert!(limits.entries() > 0);
    debug_assert!(limits.depth() > 0);
    let mut children = collect_children(directory, limits, state.observations.len())?;
    children.reverse();
    for child_name in children {
        enforce_entry_limit(state.observations.len(), limits, &directory.path_components)?;
        let child_path = child_path(directory, &child_name, limits)?;
        let metadata = directory
            .dir
            .symlink_metadata(&child_name)
            .map_err(|error| ShellError::io(OperationKind::ReadMetadata, &child_path, error))?;
        let observation = super::entry::observe(super::entry::ObservationInput {
            parent: &directory.dir,
            name: &child_name,
            path: child_path.clone(),
            metadata: &metadata,
            limits,
            total_file_bytes: &mut state.total_file_bytes,
        })?;
        if observation.kind == EntryKind::Directory {
            let dir = cap_fs_ext::DirExt::open_dir_nofollow(&directory.dir, &child_name).map_err(
                |error| ShellError::io(OperationKind::OpenDirectory, &child_path, error),
            )?;
            state.push_pending(PendingDirectory {
                path_components: child_path,
                dir,
            });
        }
        state.push_observation(observation);
    }
    Ok(())
}

fn child_path(
    directory: &PendingDirectory,
    name: &OsString,
    limits: TreeLimits,
) -> Result<Vec<Vec<u8>>, ShellError> {
    let mut path = directory.path_components.clone();
    path.push(name.as_bytes().to_vec());
    enforce_path_limits(&path, limits)?;
    Ok(path)
}

fn collect_children(
    directory: &PendingDirectory,
    limits: TreeLimits,
    observed_count: usize,
) -> Result<Vec<OsString>, ShellError> {
    let remaining = remaining_entries(observed_count, limits, &directory.path_components)?;
    debug_assert!(remaining <= limits.entries());
    let probe_count = probe_entry_count(remaining, &directory.path_components)?;
    debug_assert!(probe_count > 0);
    let entries = directory
        .dir
        .entries()
        .map_err(|error| {
            ShellError::io(
                OperationKind::ReadDirectory,
                &directory.path_components,
                error,
            )
        })?
        .take(probe_count)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| {
            ShellError::io(
                OperationKind::ReadDirectory,
                &directory.path_components,
                error,
            )
        })?;
    enforce_probe_result(entries.len(), remaining, &directory.path_components)?;
    let mut children = entries
        .into_iter()
        .map(|entry| entry.file_name())
        .collect::<Vec<_>>();
    children.sort_by(|left, right| left.as_bytes().cmp(right.as_bytes()));
    Ok(children)
}

fn remaining_entries(
    observed_count: usize,
    limits: TreeLimits,
    path: &[Vec<u8>],
) -> Result<u64, ShellError> {
    let observed_count = u64::try_from(observed_count).map_err(|_| entry_limit_error(path))?;
    limits
        .entries()
        .checked_sub(observed_count)
        .ok_or_else(|| entry_limit_error(path))
}

fn probe_entry_count(remaining: u64, path: &[Vec<u8>]) -> Result<usize, ShellError> {
    let probe_count = remaining
        .checked_add(1)
        .ok_or_else(|| entry_limit_error(path))?;
    usize::try_from(probe_count).map_err(|_| entry_limit_error(path))
}

fn enforce_probe_result(
    observed_count: usize,
    remaining: u64,
    path: &[Vec<u8>],
) -> Result<(), ShellError> {
    let observed_count = u64::try_from(observed_count).map_err(|_| entry_limit_error(path))?;
    if observed_count > remaining {
        return Err(entry_limit_error(path));
    }
    Ok(())
}

fn enforce_entry_limit(
    observed_count: usize,
    limits: TreeLimits,
    path: &[Vec<u8>],
) -> Result<(), ShellError> {
    let observed_count = u64::try_from(observed_count).map_err(|_| entry_limit_error(path))?;
    if observed_count >= limits.entries() {
        return Err(entry_limit_error(path));
    }
    Ok(())
}

fn entry_limit_error(path: &[Vec<u8>]) -> ShellError {
    ShellError::new(
        ShellErrorKind::LimitExceeded(LimitKind::Entries),
        path.to_vec(),
    )
}

fn enforce_path_limits(path: &[Vec<u8>], limits: TreeLimits) -> Result<(), ShellError> {
    let relative = RelativePath::new(path.to_vec())
        .map_err(|_| ShellError::new(ShellErrorKind::UnsupportedPlatformPath, path.to_vec()))?;
    if relative.encoded_bytes() > limits.path_bytes() {
        return Err(ShellError::new(
            ShellErrorKind::LimitExceeded(LimitKind::PathBytes),
            path.to_vec(),
        ));
    }
    enforce_depth(&relative, limits, path)
}

fn enforce_depth(
    relative: &RelativePath,
    limits: TreeLimits,
    path: &[Vec<u8>],
) -> Result<(), ShellError> {
    let depth = relative.depth().map_err(|_| {
        ShellError::new(
            ShellErrorKind::LimitExceeded(LimitKind::Depth),
            path.to_vec(),
        )
    })?;
    if depth > limits.depth() {
        return Err(ShellError::new(
            ShellErrorKind::LimitExceeded(LimitKind::Depth),
            path.to_vec(),
        ));
    }
    Ok(())
}
