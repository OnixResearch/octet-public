use std::{
    error::Error,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use cairn_core::{
    blake3_hex, count_tasks, evaluate_gate_snapshot, parse_acceptance_ids,
    synthetic_acceptance_spec, tasks_with_requirement_links, validation_issue_code,
    workflow_markdown_is_structurally_present, ChangeMetadata, ChangeStatus, GateArtifact,
    GateSnapshot, GateStage, LifecycleLayout, Policy, WorkflowArtifactFile,
    WorkflowArtifactObservation, WorkflowArtifactPolicy, WorkflowSchemaPolicy,
    WORKFLOW_ADMISSION_ACCEPTANCE, WORKFLOW_ADMISSION_DELTA_SPECS, WORKFLOW_ADMISSION_DESIGN,
    WORKFLOW_ADMISSION_PROPOSAL, WORKFLOW_ADMISSION_TASKS, WORKFLOW_PATH_KIND_FILE,
    WORKFLOW_PATH_KIND_SPEC_DIRECTORY, WORKFLOW_SPEC_EFFECT_NONE,
};

use crate::support::{path_string, CapabilityRoot, CapabilityRootKind};

const ISSUE_PATH_TYPE: &str = "cairn.workflow.observation.path_type";
const ISSUE_READ_BOUND: &str = "cairn.workflow.observation.read_bound";
const ISSUE_DIRECTORY_BOUND: &str = "cairn.workflow.observation.directory_bound";
const ISSUE_COUNT_BOUND: &str = "cairn.workflow.observation.issue_bound";
const ISSUE_NON_UTF8: &str = "cairn.workflow.observation.non_utf8";
const ISSUE_EMPTY_MARKDOWN: &str = "cairn.workflow.observation.empty_markdown";
const SPEC_FILE_NAME: &str = "spec.md";

#[derive(Debug, Clone)]
struct BoundedArtifact {
    structural_admission: String,
    observation: WorkflowArtifactObservation,
    markdown: Vec<GateArtifact>,
}

// r[impl cairn.artifact_workflow_contract.functional_boundary.shell]
pub(crate) fn observe_workflow_artifacts(
    root: &Path,
    layout: LifecycleLayout,
    change: &str,
    schema: &WorkflowSchemaPolicy,
    policy: &Policy,
) -> Result<Vec<WorkflowArtifactObservation>, Box<dyn Error>> {
    let capability = CapabilityRoot::open(CapabilityRootKind::Scan, root)?;
    let change_relative = PathBuf::from(cairn_core::changes_dir(layout)).join(change);
    require_change_directory(&capability, &change_relative, change)?;
    let mut bounded = Vec::with_capacity(schema.artifacts.len());
    for artifact in &schema.artifacts {
        bounded.push(observe_artifact(
            &capability,
            &change_relative,
            artifact,
            schema,
        )?);
    }
    apply_structural_admission(change, &change_relative, &mut bounded, policy, schema);
    Ok(bounded
        .into_iter()
        .map(|artifact| artifact.observation)
        .collect())
}

fn require_change_directory(
    capability: &CapabilityRoot,
    change_relative: &Path,
    change: &str,
) -> Result<(), Box<dyn Error>> {
    let metadata = capability
        .symlink_metadata(change_relative)
        .map_err(|error| match error.kind() {
            ErrorKind::NotFound => format!("change not found: {change}"),
            _ => format!("failed to inspect change {change}: {error}"),
        })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!("change path is not a safe directory: {change}").into());
    }
    Ok(())
}

fn observe_artifact(
    capability: &CapabilityRoot,
    change_relative: &Path,
    artifact: &WorkflowArtifactPolicy,
    schema: &WorkflowSchemaPolicy,
) -> Result<BoundedArtifact, Box<dyn Error>> {
    let relative = change_relative.join(&artifact.output_path);
    match artifact.path_kind.as_str() {
        WORKFLOW_PATH_KIND_FILE => observe_file(capability, &relative, artifact, schema),
        WORKFLOW_PATH_KIND_SPEC_DIRECTORY => {
            observe_spec_directory(capability, &relative, artifact, schema)
        }
        _ => Err(format!("unsupported workflow path kind: {}", artifact.path_kind).into()),
    }
}

fn observe_file(
    capability: &CapabilityRoot,
    relative: &Path,
    artifact: &WorkflowArtifactPolicy,
    schema: &WorkflowSchemaPolicy,
) -> Result<BoundedArtifact, Box<dyn Error>> {
    let metadata = match capability.symlink_metadata(relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(absent_artifact(artifact)),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Ok(invalid_without_files(artifact, ISSUE_PATH_TYPE));
    }
    let max_bytes = u64::try_from(schema.bounds.max_artifact_bytes)?;
    let bytes = match capability.read_bounded(relative, max_bytes) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::InvalidData => {
            return Ok(invalid_without_files(artifact, ISSUE_READ_BOUND));
        }
        Err(error) => return Err(error.into()),
    };
    let path = path_string(relative);
    let file = WorkflowArtifactFile {
        path: path.clone(),
        blake3: blake3_hex(&bytes),
    };
    match String::from_utf8(bytes) {
        Ok(markdown) => Ok(BoundedArtifact {
            structural_admission: artifact.structural_admission.clone(),
            observation: WorkflowArtifactObservation {
                artifact_id: artifact.id.clone(),
                present: true,
                structurally_valid: false,
                files: vec![file],
                issue_codes: Vec::new(),
            },
            markdown: vec![GateArtifact { path, markdown }],
        }),
        Err(_) => Ok(BoundedArtifact {
            structural_admission: artifact.structural_admission.clone(),
            observation: WorkflowArtifactObservation {
                artifact_id: artifact.id.clone(),
                present: true,
                structurally_valid: false,
                files: vec![file],
                issue_codes: vec![format!("{ISSUE_NON_UTF8}:{}", artifact.id)],
            },
            markdown: Vec::new(),
        }),
    }
}

fn observe_spec_directory(
    capability: &CapabilityRoot,
    relative: &Path,
    artifact: &WorkflowArtifactPolicy,
    schema: &WorkflowSchemaPolicy,
) -> Result<BoundedArtifact, Box<dyn Error>> {
    let metadata = match capability.symlink_metadata(relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(absent_artifact(artifact)),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Ok(invalid_without_files(artifact, ISSUE_PATH_TYPE));
    }
    let mut files = Vec::new();
    let mut markdown = Vec::new();
    let mut issues = Vec::new();
    collect_spec_directory(
        capability,
        relative,
        schema,
        &mut files,
        &mut markdown,
        &mut issues,
    )?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    markdown.sort_by(|left, right| left.path.cmp(&right.path));
    bound_issue_codes(&mut issues, schema.bounds.max_issues);
    let present = !files.is_empty() || !issues.is_empty();
    Ok(BoundedArtifact {
        structural_admission: artifact.structural_admission.clone(),
        observation: WorkflowArtifactObservation {
            artifact_id: artifact.id.clone(),
            present,
            structurally_valid: false,
            files,
            issue_codes: issues,
        },
        markdown,
    })
}

fn collect_spec_directory(
    capability: &CapabilityRoot,
    root: &Path,
    schema: &WorkflowSchemaPolicy,
    files: &mut Vec<WorkflowArtifactFile>,
    markdown: &mut Vec<GateArtifact>,
    issues: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let mut pending = vec![(root.to_path_buf(), 0usize)];
    let mut observed_entries = 0usize;
    while let Some((directory, depth)) = pending.pop() {
        if depth > schema.bounds.max_directory_depth {
            issues.push(ISSUE_DIRECTORY_BOUND.to_string());
            continue;
        }
        let entries =
            match capability.entries_bounded(&directory, schema.bounds.max_directory_entries) {
                Ok(entries) => entries,
                Err(error) if error.kind() == ErrorKind::InvalidData => {
                    issues.push(ISSUE_DIRECTORY_BOUND.to_string());
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
        observed_entries = observed_entries
            .checked_add(entries.len())
            .ok_or("workflow directory entry count overflow")?;
        if observed_entries > schema.bounds.max_directory_entries {
            issues.push(ISSUE_DIRECTORY_BOUND.to_string());
            break;
        }
        for entry in entries {
            let path = directory.join(entry.file_name());
            if entry.file_type().is_dir() {
                pending.push((path, depth + 1));
            } else if entry.file_type().is_file()
                && path.file_name().is_some_and(|name| name == SPEC_FILE_NAME)
            {
                if files.len() >= schema.bounds.max_files_per_artifact {
                    issues.push(ISSUE_DIRECTORY_BOUND.to_string());
                    continue;
                }
                collect_spec_file(capability, &path, schema, files, markdown, issues)?;
            } else if entry.file_type().is_symlink() {
                issues.push(format!("{ISSUE_PATH_TYPE}:{}", path_string(path)));
            }
        }
    }
    Ok(())
}

fn collect_spec_file(
    capability: &CapabilityRoot,
    path: &Path,
    schema: &WorkflowSchemaPolicy,
    files: &mut Vec<WorkflowArtifactFile>,
    markdown: &mut Vec<GateArtifact>,
    issues: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let max_bytes = u64::try_from(schema.bounds.max_artifact_bytes)?;
    let bytes = match capability.read_bounded(path, max_bytes) {
        Ok(bytes) => bytes,
        Err(error) if error.kind() == ErrorKind::InvalidData => {
            issues.push(format!("{ISSUE_READ_BOUND}:{}", path_string(path)));
            return Ok(());
        }
        Err(error) => return Err(error.into()),
    };
    let path = path_string(path);
    files.push(WorkflowArtifactFile {
        path: path.clone(),
        blake3: blake3_hex(&bytes),
    });
    match String::from_utf8(bytes) {
        Ok(value) => markdown.push(GateArtifact {
            path,
            markdown: value,
        }),
        Err(_) => issues.push(format!("{ISSUE_NON_UTF8}:{path}")),
    }
    Ok(())
}

fn apply_structural_admission(
    change: &str,
    change_relative: &Path,
    artifacts: &mut [BoundedArtifact],
    policy: &Policy,
    schema: &WorkflowSchemaPolicy,
) {
    let snapshot = gate_snapshot(change, change_relative, artifacts);
    let snapshot = profile_aware_gate_snapshot(snapshot, artifacts, schema);
    let report = evaluate_gate_snapshot(&snapshot, GateStage::Tasks, policy);
    for artifact in artifacts {
        if !artifact.observation.present {
            continue;
        }
        let structural = structural_issue_codes(change, artifact, &report.issues);
        artifact.observation.issue_codes.extend(structural);
        if artifact.observation.issue_codes.is_empty()
            && artifact
                .markdown
                .iter()
                .all(|file| workflow_markdown_is_structurally_present(file.markdown.as_str()))
            && !artifact.markdown.is_empty()
        {
            artifact.observation.structurally_valid = true;
        } else if artifact.observation.issue_codes.is_empty() {
            artifact.observation.issue_codes.push(format!(
                "{ISSUE_EMPTY_MARKDOWN}:{}",
                artifact.observation.artifact_id
            ));
        }
        bound_issue_codes(
            &mut artifact.observation.issue_codes,
            schema.bounds.max_issues,
        );
    }
}

fn profile_aware_gate_snapshot(
    mut snapshot: GateSnapshot,
    artifacts: &[BoundedArtifact],
    schema: &WorkflowSchemaPolicy,
) -> GateSnapshot {
    if schema.spec_effect != WORKFLOW_SPEC_EFFECT_NONE {
        return snapshot;
    }
    let acceptance_markdown = all_markdown(artifacts, WORKFLOW_ADMISSION_ACCEPTANCE)
        .into_iter()
        .map(|artifact| artifact.markdown)
        .collect::<Vec<_>>()
        .join("\n");
    let (acceptance_ids, _) = parse_acceptance_ids(&acceptance_markdown);
    let synthetic_path = format!("{}/acceptance.md#profile-adapter", snapshot.change.path);
    snapshot.change.delta_specs = vec![synthetic_path.clone()];
    snapshot.delta_specs = vec![GateArtifact {
        path: synthetic_path,
        markdown: synthetic_acceptance_spec(&acceptance_ids),
    }];
    if let Some(tasks) = &mut snapshot.tasks {
        tasks.markdown = tasks_with_requirement_links(&tasks.markdown);
    }
    snapshot
}

fn gate_snapshot(
    change: &str,
    change_relative: &Path,
    artifacts: &[BoundedArtifact],
) -> GateSnapshot {
    let proposal = first_markdown(artifacts, WORKFLOW_ADMISSION_PROPOSAL);
    let design = first_markdown(artifacts, WORKFLOW_ADMISSION_DESIGN);
    let tasks = first_markdown(artifacts, WORKFLOW_ADMISSION_TASKS);
    let delta_specs = all_markdown(artifacts, WORKFLOW_ADMISSION_DELTA_SPECS);
    let task_counts = tasks
        .as_ref()
        .map(|artifact| count_tasks(&artifact.markdown));
    GateSnapshot {
        change: ChangeStatus {
            name: change.to_string(),
            path: path_string(change_relative),
            has_proposal: proposal.is_some(),
            has_design: design.is_some(),
            has_tasks: tasks.is_some(),
            delta_specs: delta_specs.iter().map(|file| file.path.clone()).collect(),
            task_counts,
            metadata: ChangeMetadata::default(),
            metadata_issues: Vec::new(),
        },
        proposal,
        design,
        tasks,
        delta_specs,
    }
}

fn first_markdown(artifacts: &[BoundedArtifact], admission: &str) -> Option<GateArtifact> {
    artifacts
        .iter()
        .find(|artifact| artifact_admission(artifact) == admission)
        .and_then(|artifact| artifact.markdown.first().cloned())
}

fn all_markdown(artifacts: &[BoundedArtifact], admission: &str) -> Vec<GateArtifact> {
    artifacts
        .iter()
        .filter(|artifact| artifact_admission(artifact) == admission)
        .flat_map(|artifact| artifact.markdown.clone())
        .collect()
}

fn artifact_admission(artifact: &BoundedArtifact) -> &str {
    &artifact.structural_admission
}

fn structural_issue_codes(
    change: &str,
    artifact: &BoundedArtifact,
    issues: &[cairn_core::GateIssue],
) -> Vec<String> {
    let task_global_issue = artifact_admission(artifact) == WORKFLOW_ADMISSION_TASKS;
    issues
        .iter()
        .filter(|issue| {
            artifact
                .observation
                .files
                .iter()
                .any(|file| issue.message.starts_with(&file.path))
                || (task_global_issue
                    && issue.message.starts_with(change)
                    && issue.message.contains("task"))
        })
        .map(|issue| {
            let code = validation_issue_code(&issue.message);
            format!("{code}:{}", artifact.observation.artifact_id)
        })
        .collect()
}

fn bound_issue_codes(issues: &mut Vec<String>, max_issues: usize) {
    assert!(max_issues > 0);
    issues.sort();
    issues.dedup();
    if issues.len() <= max_issues {
        return;
    }
    issues.truncate(max_issues);
    let removed = issues.pop();
    assert!(removed.is_some());
    issues.push(ISSUE_COUNT_BOUND.to_string());
    issues.sort();
}

fn absent_artifact(artifact: &WorkflowArtifactPolicy) -> BoundedArtifact {
    BoundedArtifact {
        structural_admission: artifact.structural_admission.clone(),
        observation: WorkflowArtifactObservation::absent(&artifact.id),
        markdown: Vec::new(),
    }
}

fn invalid_without_files(artifact: &WorkflowArtifactPolicy, issue: &str) -> BoundedArtifact {
    BoundedArtifact {
        structural_admission: artifact.structural_admission.clone(),
        observation: WorkflowArtifactObservation {
            artifact_id: artifact.id.clone(),
            present: true,
            structurally_valid: false,
            files: Vec::new(),
            issue_codes: vec![format!("{issue}:{}", artifact.id)],
        },
        markdown: Vec::new(),
    }
}
