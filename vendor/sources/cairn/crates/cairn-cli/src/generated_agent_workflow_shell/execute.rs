use std::{
    error::Error,
    io,
    path::{Path, PathBuf},
};

use cairn_core::{
    blake3_hex, validate_generated_workflow_manifest, GeneratedWorkflowFreshness,
    GeneratedWorkflowInstallPlan, GeneratedWorkflowManifest,
};

use crate::support::CapabilityRoot;

const LOCK_PATH: &str = ".cairn/generated-agent-workflows.lock";
const LOCK_CONTENT: &[u8] = b"cairn generated agent workflows lock\n";
const TEMP_ID_HEX_BYTES: usize = 32;

pub(super) struct AgentWorkflowLock<'a> {
    root: &'a CapabilityRoot,
}

impl<'a> AgentWorkflowLock<'a> {
    pub(super) fn acquire(root: &'a CapabilityRoot) -> Result<Self, Box<dyn Error>> {
        let cairn = root.symlink_metadata(Path::new(".cairn"))?;
        if cairn.file_type().is_symlink() || !cairn.is_dir() {
            return Err(".cairn must be a regular project-local directory".into());
        }
        root.write_new(Path::new(LOCK_PATH), LOCK_CONTENT)
            .map_err(|error| format!("generated workflow lock is unavailable: {error}"))?;
        Ok(Self { root })
    }
}

impl Drop for AgentWorkflowLock<'_> {
    fn drop(&mut self) {
        let _ = self.root.remove_file(Path::new(LOCK_PATH));
    }
}

// r[impl cairn.generated_agent_workflows.install]
// r[impl cairn.generated_agent_workflows.install.fresh]
// r[impl cairn.generated_agent_workflows.install.conflict]
// r[impl cairn.generated_agent_workflows.functional_boundary.shell]
pub(super) fn execute_install_plan(
    root: &CapabilityRoot,
    plan: &GeneratedWorkflowInstallPlan,
) -> Result<bool, Box<dyn Error>> {
    if plan.blocked() {
        return Err("generated workflow installation plan is blocked".into());
    }
    verify_expected_prior_state(root, plan)?;
    let writes = planned_install_writes(plan);
    if writes.is_empty() {
        return Ok(false);
    }
    let mut prepared = Vec::with_capacity(writes.len());
    for write in &writes {
        if let Some(parent) = write.target.parent() {
            if let Err(error) = root.create_dir_all(parent) {
                cleanup_prepared_writes(root, &prepared);
                return Err(error.into());
            }
        }
        let temporary = transaction_path("tmp", &write.target, &write.content_blake3);
        let backup = transaction_path("backup", &write.target, &write.content_blake3);
        if let Err(error) = require_missing(root, &backup) {
            cleanup_prepared_writes(root, &prepared);
            return Err(error);
        }
        if let Err(error) = root.write_new(&temporary, &write.contents) {
            cleanup_prepared_writes(root, &prepared);
            return Err(error.into());
        }
        prepared.push(PreparedWrite {
            target: write.target.clone(),
            temporary,
            backup,
            target_existed: write.target_existed,
        });
    }
    if let Err(error) = verify_expected_prior_state(root, plan) {
        cleanup_prepared_writes(root, &prepared);
        return Err(error);
    }
    match commit_prepared_writes(root, &prepared) {
        Ok(committed) => {
            if let Err(error) = verify_installed_writes(root, &writes) {
                rollback_installed(root, &committed.installed, &committed.backups);
                cleanup_prepared_writes(root, &prepared);
                return Err(error);
            }
            for (_, backup) in committed.backups {
                root.remove_file(&backup)?;
            }
            Ok(true)
        }
        Err(error) => {
            cleanup_prepared_writes(root, &prepared);
            Err(error)
        }
    }
}

struct InstallWrite {
    target: PathBuf,
    contents: Vec<u8>,
    content_blake3: String,
    target_existed: bool,
}

struct PreparedWrite {
    target: PathBuf,
    temporary: PathBuf,
    backup: PathBuf,
    target_existed: bool,
}

struct CommittedWrites {
    installed: Vec<PathBuf>,
    backups: Vec<(PathBuf, PathBuf)>,
}

fn verify_expected_prior_state(
    root: &CapabilityRoot,
    plan: &GeneratedWorkflowInstallPlan,
) -> Result<(), Box<dyn Error>> {
    let max_bytes = u64::try_from(plan.max_file_bytes)?;
    for file in &plan.files {
        let path = Path::new(&file.relative_path);
        require_no_symlink_components(root, path)?;
        match file.freshness {
            GeneratedWorkflowFreshness::Current => {}
            GeneratedWorkflowFreshness::Missing => require_missing(root, path)?,
            GeneratedWorkflowFreshness::StaleSource => {
                let expected = file
                    .expected_prior_blake3
                    .as_deref()
                    .ok_or("stale generated file has no expected prior identity")?;
                let observed = root.read_bounded(path, max_bytes)?;
                if blake3_hex(&observed) != expected {
                    return Err(format!(
                        "generated workflow prior identity changed at {}",
                        path.display()
                    )
                    .into());
                }
            }
            _ => return Err("generated workflow prior state is not replaceable".into()),
        }
    }
    let manifest_path = Path::new(&plan.manifest_path);
    require_no_symlink_components(root, manifest_path)?;
    match plan.manifest_freshness {
        GeneratedWorkflowFreshness::Current => {}
        GeneratedWorkflowFreshness::Missing => require_missing(root, manifest_path)?,
        GeneratedWorkflowFreshness::StaleSource => {
            let expected = plan
                .expected_prior_manifest_identity
                .as_deref()
                .ok_or("stale manifest has no expected prior identity")?;
            let bytes = root.read_bounded(manifest_path, max_bytes)?;
            let manifest: GeneratedWorkflowManifest = serde_json::from_slice(&bytes)?;
            if !validate_generated_workflow_manifest(&manifest)
                || manifest.manifest_identity != expected
            {
                return Err("generated workflow manifest prior identity changed".into());
            }
        }
        _ => return Err("generated workflow manifest state is not replaceable".into()),
    }
    Ok(())
}

fn require_no_symlink_components(root: &CapabilityRoot, path: &Path) -> Result<(), Box<dyn Error>> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match root.symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "generated workflow path contains a symlink: {}",
                    current.display()
                )
                .into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn require_missing(root: &CapabilityRoot, path: &Path) -> Result<(), Box<dyn Error>> {
    match root.symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Ok(_) => Err(format!(
            "generated workflow expected a missing path: {}",
            path.display()
        )
        .into()),
        Err(error) => Err(error.into()),
    }
}

fn planned_install_writes(plan: &GeneratedWorkflowInstallPlan) -> Vec<InstallWrite> {
    let mut writes = plan
        .files
        .iter()
        .filter(|file| file.freshness != GeneratedWorkflowFreshness::Current)
        .map(|file| InstallWrite {
            target: PathBuf::from(&file.relative_path),
            contents: file.contents.as_bytes().to_vec(),
            content_blake3: file.generated_blake3.clone(),
            target_existed: file.freshness == GeneratedWorkflowFreshness::StaleSource,
        })
        .collect::<Vec<_>>();
    if plan.manifest_freshness != GeneratedWorkflowFreshness::Current {
        writes.push(InstallWrite {
            target: PathBuf::from(&plan.manifest_path),
            contents: plan.manifest_bytes.as_bytes().to_vec(),
            content_blake3: blake3_hex(plan.manifest_bytes.as_bytes()),
            target_existed: plan.manifest_freshness == GeneratedWorkflowFreshness::StaleSource,
        });
    }
    writes
}

fn commit_prepared_writes(
    root: &CapabilityRoot,
    prepared: &[PreparedWrite],
) -> Result<CommittedWrites, Box<dyn Error>> {
    let mut backed_up = Vec::new();
    for write in prepared.iter().filter(|write| write.target_existed) {
        if let Err(error) = root.rename(&write.target, root, &write.backup) {
            restore_backups(root, &backed_up);
            return Err(error.into());
        }
        backed_up.push((write.target.clone(), write.backup.clone()));
    }
    let mut installed = Vec::new();
    for write in prepared {
        if let Err(error) = root.rename(&write.temporary, root, &write.target) {
            rollback_installed(root, &installed, &backed_up);
            return Err(error.into());
        }
        installed.push(write.target.clone());
    }
    Ok(CommittedWrites {
        installed,
        backups: backed_up,
    })
}

fn verify_installed_writes(
    root: &CapabilityRoot,
    writes: &[InstallWrite],
) -> Result<(), Box<dyn Error>> {
    for write in writes {
        let max_bytes = u64::try_from(write.contents.len())?;
        let observed = root.read_bounded(&write.target, max_bytes)?;
        let observed_blake3 = blake3_hex(&observed);
        if observed_blake3 != write.content_blake3 {
            return Err(format!(
                "generated workflow post-write identity mismatch at {}",
                write.target.display()
            )
            .into());
        }
    }
    Ok(())
}

fn rollback_installed(
    root: &CapabilityRoot,
    installed: &[PathBuf],
    backups: &[(PathBuf, PathBuf)],
) {
    for target in installed.iter().rev() {
        let _ = root.remove_file(target);
    }
    restore_backups(root, backups);
}

fn restore_backups(root: &CapabilityRoot, backups: &[(PathBuf, PathBuf)]) {
    for (target, backup) in backups.iter().rev() {
        let _ = root.rename(backup, root, target);
    }
}

fn cleanup_prepared_writes(root: &CapabilityRoot, prepared: &[PreparedWrite]) {
    for write in prepared {
        match root.remove_file(&write.temporary) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(_) => {}
        }
    }
}

fn transaction_path(kind: &str, target: &Path, identity: &str) -> PathBuf {
    let input = format!("{kind}:{}:{identity}", target.display());
    let digest = blake3_hex(input.as_bytes());
    PathBuf::from(format!(
        ".cairn/generated-agent-workflow-{}-{}.pending",
        kind,
        &digest[..TEMP_ID_HEX_BYTES]
    ))
}

#[cfg(test)]
mod tests {
    use std::{fs, io};

    use cairn_core::{
        admit_generated_agent_workflow_policy, canonical_json_hash, parse_policy_json,
        plan_generated_agent_workflow_install, GeneratedWorkflowDestinationSnapshot,
        GeneratedWorkflowManifestObservation,
    };

    use super::*;
    use crate::support::CapabilityRootKind;

    const BLOCKING_DIRECTORY: &str = "blocking-directory";

    #[test]
    fn changed_target_preflight_leaves_no_partial_outputs() -> Result<(), Box<dyn Error>> {
        let directory = std::env::temp_dir().join(format!(
            "cairn-generated-workflow-rollback-{}",
            std::process::id()
        ));
        match fs::remove_dir_all(&directory) {
            Ok(()) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        fs::create_dir_all(directory.join(".cairn"))?;
        fs::create_dir(directory.join(BLOCKING_DIRECTORY))?;
        let root = CapabilityRoot::open(CapabilityRootKind::Repository, &directory)?;
        let policy = parse_policy_json(include_str!(
            "../../../../cairn-policy/generated/cairn-policy.json"
        ))?;
        let catalog =
            admit_generated_agent_workflow_policy(&policy.generated_agent_workflow_policy)
                .map_err(|issues| format!("invalid policy: {issues:?}"))?;
        let snapshot = GeneratedWorkflowDestinationSnapshot {
            files: Vec::new(),
            manifest: GeneratedWorkflowManifestObservation::Missing,
        };
        let mut plan = plan_generated_agent_workflow_install(
            &catalog,
            "claude",
            ".claude/commands",
            &canonical_json_hash(&policy.raw),
            &snapshot,
        )
        .map_err(|issues| format!("invalid plan: {issues:?}"))?;
        let original_paths = plan
            .files
            .iter()
            .map(|file| file.relative_path.clone())
            .collect::<Vec<_>>();
        plan.files.last_mut().ok_or("planned file")?.relative_path = BLOCKING_DIRECTORY.to_string();

        let error = execute_install_plan(&root, &plan).expect_err("preflight must fail");
        assert!(!error.to_string().is_empty());
        assert!(directory.join(BLOCKING_DIRECTORY).is_dir());
        assert!(original_paths
            .iter()
            .all(|path| !directory.join(path).exists()));
        assert!(!directory.join(&plan.manifest_path).exists());
        assert!(fs::read_dir(directory.join(".cairn"))?.next().is_none());
        fs::remove_dir_all(directory)?;
        Ok(())
    }
}
