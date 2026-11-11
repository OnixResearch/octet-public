use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
};

use cairn_core::{
    plan_change_collaboration_checkout, ChangeCheckoutDestination, ChangeCollaborationCheckoutPlan,
    ChangeCollaborationState,
};

use crate::{
    radicle_change::status_checkout::{
        path_exists, read_local_change_snapshot, safe_existing_directory,
    },
    support::{CapabilityRoot, CapabilityRootKind},
};

const CHANGES_DIRECTORY: &str = ".cairn/changes";
const PREPARED_DIRECTORY: &str = ".cairn/.collaboration-checkout-prepared";
const REPOSITORY_CHECKOUT_LOCK_DIRECTORY: &str = ".cairn/.collaboration-locks/checkout";
const BLOCKER_CHANGES_DIRECTORY: &str =
    "native .cairn/changes directory is missing, unsafe, or not a directory";
const BLOCKER_PREPARED_EXISTS: &str =
    "prepared collaboration checkout already exists; inspect interrupted local state before recovery";
const BLOCKER_DESTINATION_APPEARED: &str =
    "active-change destination appeared before atomic checkout exposure";
const BLOCKER_POST_VERIFICATION: &str =
    "materialized active change did not match portable collaboration state after checkout";
const BLOCKER_POST_VERIFICATION_UNAVAILABLE: &str =
    "materialized active change could not be inspected after checkout";

pub(crate) struct RepositoryCheckoutLock {
    repository: CapabilityRoot,
    relative: PathBuf,
    active: bool,
}

impl RepositoryCheckoutLock {
    pub(crate) fn acquire(root: &Path, change: &str) -> Result<Self, Box<dyn Error>> {
        let repository = CapabilityRoot::open(CapabilityRootKind::Repository, root)?;
        let parent = PathBuf::from(REPOSITORY_CHECKOUT_LOCK_DIRECTORY);
        repository.create_dir_all(&parent)?;
        let relative = parent.join(cairn_core::blake3_hex(change.as_bytes()));
        match repository.create_dir(&relative) {
            Ok(()) => Ok(Self {
                repository,
                relative,
                active: true,
            }),
            Err(error) if error.kind() == io::ErrorKind::AlreadyExists => Err(format!(
                "another collaboration operation is active for change {change}; after confirming no checkout or update is running, remove stale repository lock {}",
                relative.display()
            )
            .into()),
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) fn release(mut self) -> Result<(), Box<dyn Error>> {
        self.release_inner()?;
        self.active = false;
        Ok(())
    }

    fn release_inner(&self) -> Result<(), Box<dyn Error>> {
        self.repository.remove_dir_all(&self.relative)?;
        Ok(())
    }
}

impl Drop for RepositoryCheckoutLock {
    fn drop(&mut self) {
        if self.active {
            let _ = self.release_inner();
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MaterializationOutcome {
    pub mutated: bool,
    pub final_destination: Option<ChangeCheckoutDestination>,
    pub blockers: Vec<String>,
}

pub(super) fn materialize_collaboration_checkout(
    root: &Path,
    state: &ChangeCollaborationState,
    plan: &ChangeCollaborationCheckoutPlan,
) -> Result<MaterializationOutcome, Box<dyn Error>> {
    debug_assert!(plan.checkout_ready());
    debug_assert_eq!(plan.destination, ChangeCheckoutDestination::Absent);
    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, root)?;
    if !safe_existing_directory(&repository, Path::new(CHANGES_DIRECTORY))? {
        return Ok(blocked(BLOCKER_CHANGES_DIRECTORY));
    }

    let workspace_key = cairn_core::blake3_hex(plan.workspace_id.as_bytes());
    let prepared_parent = PathBuf::from(PREPARED_DIRECTORY).join(workspace_key);
    let prepared = prepared_parent.join(format!("{}-{}", plan.change_id, plan.state_hash));
    if path_exists(&repository, &prepared)? {
        return Ok(blocked(BLOCKER_PREPARED_EXISTS));
    }
    repository.create_dir_all(&prepared_parent)?;
    match repository.create_dir(&prepared) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            return Ok(blocked(BLOCKER_PREPARED_EXISTS));
        }
        Err(error) => return Err(error.into()),
    }

    if let Err(error) = write_and_verify(&repository, &prepared, plan) {
        return Err(cleanup_after_prepare_error(&repository, &prepared, error));
    }

    let destination = PathBuf::from(CHANGES_DIRECTORY).join(&plan.change_id);
    if path_exists(&repository, &destination)? {
        repository.remove_dir_all(&prepared)?;
        return Ok(blocked(BLOCKER_DESTINATION_APPEARED));
    }
    repository.rename(&prepared, &repository, &destination)?;

    let post_plan = read_local_change_snapshot(root, &plan.change_id)
        .and_then(|local| plan_change_collaboration_checkout(state, local).map_err(Into::into));
    match post_plan {
        Ok(post_plan) => {
            let blockers = if post_plan.already_current() {
                Vec::new()
            } else {
                vec![BLOCKER_POST_VERIFICATION.to_string()]
            };
            Ok(MaterializationOutcome {
                mutated: true,
                final_destination: Some(post_plan.destination),
                blockers,
            })
        }
        Err(_) => Ok(MaterializationOutcome {
            mutated: true,
            final_destination: None,
            blockers: vec![BLOCKER_POST_VERIFICATION_UNAVAILABLE.to_string()],
        }),
    }
}

fn write_and_verify(
    repository: &CapabilityRoot,
    prepared: &Path,
    plan: &ChangeCollaborationCheckoutPlan,
) -> Result<(), Box<dyn Error>> {
    for file in &plan.files {
        let output = prepared.join(&file.path);
        let parent = output.parent().unwrap_or(prepared);
        repository.create_dir_all(parent)?;
        repository.write(&output, file.content.as_bytes())?;
        let max_bytes = u64::try_from(file.byte_length)?;
        let restored = repository.read_bounded(&output, max_bytes)?;
        if restored.len() != file.byte_length || cairn_core::blake3_hex(&restored) != file.blake3 {
            return Err(format!(
                "prepared collaboration checkout failed verification: {}",
                file.path
            )
            .into());
        }
    }
    Ok(())
}

fn cleanup_after_prepare_error(
    repository: &CapabilityRoot,
    prepared: &Path,
    error: Box<dyn Error>,
) -> Box<dyn Error> {
    match repository.remove_dir_all(prepared) {
        Ok(()) => error,
        Err(cleanup) => format!(
            "collaboration checkout preparation failed ({error}) and cleanup failed ({cleanup})"
        )
        .into(),
    }
}

fn blocked(blocker: &str) -> MaterializationOutcome {
    MaterializationOutcome {
        mutated: false,
        final_destination: None,
        blockers: vec![blocker.to_string()],
    }
}
