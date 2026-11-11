use std::{
    error::Error,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use cairn_core::{
    build_workflow_instructions, canonical_json_hash, changes_dir, derive_workflow_status,
    evaluate_workflow_profile, select_workflow_profile, validate_change_name,
    validate_review_receipt, workflow_change_profile_metadata_from_value, AgentGuidanceRequest,
    GateSnapshot, GateStage, LifecycleLayout, Policy, WorkflowArtifactObservation,
    WorkflowArtifactState, WorkflowChangeProfileMetadata, WorkflowProfileArtifactHash,
    WorkflowProfileEvaluation, WorkflowProfileEvidence, WorkflowProfileRef,
    WorkflowProfileSelection, WorkflowReviewEvidence, WorkflowReviewInputEvidence,
    WorkflowSchemaPolicy, WorkflowStatusReport, AGENT_GUIDANCE_OPERATION_WORKFLOW_INSTRUCTIONS,
    MAX_WORKFLOW_ARTIFACT_BYTES_LIMIT, WORKFLOW_ADMISSION_ACCEPTANCE,
    WORKFLOW_ADMISSION_DELTA_SPECS, WORKFLOW_ADMISSION_TASKS, WORKFLOW_PROFILE_SOURCE_CREATE,
};
use serde_json::{json, Value};

use crate::support::{detect_layout, load_policy_file, path_string, print_json};

pub(crate) mod observe;
mod options;
mod receipts;

use observe::observe_workflow_artifacts;
use options::{parse_workflow_request, WorkflowRequestKind};
use receipts::{instructions_receipt, status_receipt, workflow_input_hash, WorkflowReceiptContext};

// r[impl cairn.artifact_workflow_contract.cli]
pub(crate) fn run_workflow(
    args: Vec<String>,
    root_local: Option<&crate::lifecycle_store_shell::RootLocalInvocation>,
) -> Result<(), Box<dyn Error>> {
    let request = parse_workflow_request(&args)?;
    let value = match &request.kind {
        WorkflowRequestKind::Status { change } => workflow_status_value(
            &request.root,
            change,
            request.schema.as_deref(),
            &request.policy,
        )?,
        WorkflowRequestKind::Instructions { change, artifact } => {
            workflow_instructions_value_with_project_root(
                &request.root,
                root_local
                    .and_then(crate::lifecycle_store_shell::RootLocalInvocation::project_root)
                    .unwrap_or(&request.root),
                change,
                artifact,
                request.schema.as_deref(),
                &request.policy,
            )?
        }
    };
    print_json(value)
}

pub(crate) fn workflow_status_value(
    root: &Path,
    change: &str,
    schema_selector: Option<&str>,
    policy_path: &Path,
) -> Result<Value, Box<dyn Error>> {
    let context = workflow_context(root, change, schema_selector, policy_path)?;
    status_receipt(&context.receipt_context(root, change), &context.report)
}

pub(crate) fn workflow_instructions_value(
    root: &Path,
    change: &str,
    artifact: &str,
    schema_selector: Option<&str>,
    policy_path: &Path,
) -> Result<Value, Box<dyn Error>> {
    workflow_instructions_value_with_project_root(
        root,
        root,
        change,
        artifact,
        schema_selector,
        policy_path,
    )
}

// r[impl cairn.agent_guidance_context.integration.instructions]
// r[impl cairn.agent_guidance_context.authority.advisory]
fn workflow_instructions_value_with_project_root(
    root: &Path,
    project_root: &Path,
    change: &str,
    artifact: &str,
    schema_selector: Option<&str>,
    policy_path: &Path,
) -> Result<Value, Box<dyn Error>> {
    let mut context = workflow_context(root, change, schema_selector, policy_path)?;
    let instructions = build_workflow_instructions(
        &context.schema,
        &context.report,
        artifact,
        &context.change_path,
    )?;
    let policy = load_policy_file(policy_path.to_path_buf())?;
    let guidance = crate::agent_guidance_shell::load_agent_guidance(
        project_root,
        root,
        AgentGuidanceRequest {
            profile: Some(context.selection.profile_id.clone()),
            artifact: Some(artifact.to_string()),
            operation: Some(AGENT_GUIDANCE_OPERATION_WORKFLOW_INSTRUCTIONS.to_string()),
        },
        &instructions.constraints,
        &policy,
    )?;
    if let Some(guidance) = &guidance {
        context.input_hash = canonical_json_hash(&json!({
            "workflow_input_hash": context.input_hash,
            "guidance_portable_identity": guidance.portable_identity,
        }));
    }
    instructions_receipt(
        &context.receipt_context(root, change),
        &instructions,
        guidance.as_ref(),
    )
}

#[derive(Debug, Clone)]
pub(crate) struct WorkflowProfileContext {
    pub(crate) layout: LifecycleLayout,
    pub(crate) policy_name: String,
    pub(crate) policy_hash: String,
    pub(crate) schema: WorkflowSchemaPolicy,
    pub(crate) selection: WorkflowProfileSelection,
    pub(crate) evaluation: WorkflowProfileEvaluation,
    pub(crate) report: WorkflowStatusReport,
    pub(crate) input_hash: String,
    pub(crate) profile_identity_hash: String,
    pub(crate) change_path: String,
}

impl WorkflowProfileContext {
    fn receipt_context<'a>(
        &'a self,
        root: &'a Path,
        change: &'a str,
    ) -> WorkflowReceiptContext<'a> {
        WorkflowReceiptContext {
            root,
            layout: self.layout,
            change,
            policy_name: &self.policy_name,
            policy_hash: &self.policy_hash,
            input_hash: &self.input_hash,
            schema: &self.schema,
            selection: &self.selection,
            evaluation: &self.evaluation,
            profile_identity_hash: &self.profile_identity_hash,
        }
    }

    pub(crate) fn completion_blockers(&self) -> Vec<String> {
        let mut blockers = self
            .evaluation
            .issues
            .iter()
            .map(|issue| format!("{}: {}", issue.code, issue.message))
            .collect::<Vec<_>>();
        blockers.extend(self.report.artifacts.iter().filter_map(|artifact| {
            if matches!(
                artifact.state,
                WorkflowArtifactState::Done | WorkflowArtifactState::Skipped
            ) {
                None
            } else {
                Some(format!(
                    "cairn.workflow.profile.artifact: {} is {}",
                    artifact.id,
                    artifact.state.as_str()
                ))
            }
        }));
        blockers.sort();
        blockers.dedup();
        blockers
    }

    pub(crate) fn gate_report(
        &self,
        snapshot: &GateSnapshot,
        stage: GateStage,
        policy: &Policy,
    ) -> cairn_core::GateReport {
        if self.selection.compatibility_fallback {
            return cairn_core::evaluate_gate_snapshot(snapshot, stage, policy);
        }
        cairn_core::evaluate_profile_gate_snapshot(
            snapshot,
            stage,
            policy,
            &self.schema,
            &self.evaluation,
            &self.report,
        )
    }

    pub(crate) fn receipt_fields(&self) -> Value {
        json!({
            "workflow_profile": self.selection.profile_id,
            "workflow_profile_version": self.selection.profile_version,
            "workflow_profile_source": self.selection.source,
            "workflow_profile_compatibility_fallback": self.selection.compatibility_fallback,
            "spec_effect": self.evaluation.spec_effect,
            "acceptance_ids": self.evaluation.acceptance_ids,
            "review_receipt_hashes": self.evaluation.review_receipt_hashes,
            "profile_identity_hash": self.profile_identity_hash,
            "non_claims": self.evaluation.non_claims,
        })
    }
}

fn workflow_context(
    root: &Path,
    change: &str,
    schema_selector: Option<&str>,
    policy_path: &Path,
) -> Result<WorkflowProfileContext, Box<dyn Error>> {
    validate_change_name(change)?;
    let selected = detect_layout(root);
    if selected.layout != LifecycleLayout::Native {
        return Err(
            "cairn.workflow.layout_unsupported: workflow commands require native cairn layout"
                .into(),
        );
    }
    let policy = load_policy_file(policy_path.to_path_buf())?;
    let context = load_workflow_profile_context(root, selected.layout, change, &policy)?;
    if schema_selector.is_some_and(|selector| selector != context.schema.id) {
        return Err(format!(
            "cairn.workflow.profile.selection: schema assertion does not match immutable change profile: requested {}, selected {}",
            schema_selector.unwrap_or_default(),
            context.schema.id
        )
        .into());
    }
    Ok(context)
}

pub(crate) fn load_workflow_profile_context(
    root: &Path,
    layout: LifecycleLayout,
    change: &str,
    policy: &Policy,
) -> Result<WorkflowProfileContext, Box<dyn Error>> {
    // r[impl cairn.lifecycle_workflow_profiles.integration]
    // r[impl cairn.lifecycle_workflow_profiles.functional_boundary]
    validate_change_name(change)?;
    let change_relative = PathBuf::from(changes_dir(layout)).join(change);
    let metadata_value = read_change_metadata_value(root, &change_relative)?;
    let metadata = workflow_change_profile_metadata_from_value(&metadata_value)?;
    let (selection, schema) = select_workflow_profile(
        &policy.workflow_profile_policy,
        &policy.workflow_schemas,
        &metadata,
    )?;
    let observations = observe_workflow_artifacts(root, layout, change, &schema, policy)?;
    let mut report = derive_workflow_status(&schema, &observations)?;
    let artifact_hashes = profile_artifact_hashes(&change_relative, &observations);
    let acceptance_markdown = read_admission_artifact(
        root,
        &change_relative,
        &schema,
        WORKFLOW_ADMISSION_ACCEPTANCE,
    )?;
    let tasks_markdown =
        read_admission_artifact(root, &change_relative, &schema, WORKFLOW_ADMISSION_TASKS)?;
    let reviews = read_review_evidence(root, &change_relative, &schema, &artifact_hashes)?;
    let delta_spec_paths = observations
        .iter()
        .filter(|observation| {
            schema.artifacts.iter().any(|artifact| {
                artifact.id == observation.artifact_id
                    && artifact.structural_admission == WORKFLOW_ADMISSION_DELTA_SPECS
            })
        })
        .flat_map(|observation| observation.files.iter().map(|file| file.path.clone()))
        .collect::<Vec<_>>();
    let evidence = WorkflowProfileEvidence {
        spec_effect: metadata.spec_effect,
        spec_effect_rationale: metadata.spec_effect_rationale,
        delta_spec_paths,
        acceptance_markdown,
        tasks_markdown,
        artifact_hashes,
        reviews,
    };
    let evaluation = evaluate_workflow_profile(&schema, &evidence);
    apply_profile_evaluation(&mut report, &evaluation);
    let input_hash = workflow_input_hash(change, &schema, &observations)?;
    let policy_hash = canonical_json_hash(&policy.raw);
    let profile_identity_hash = canonical_json_hash(&json!({
        "selection": selection,
        "schema": schema,
        "evaluation": evaluation,
        "observations": observations,
    }));
    Ok(WorkflowProfileContext {
        layout,
        policy_name: policy.name.clone(),
        policy_hash,
        schema,
        selection,
        evaluation,
        report,
        input_hash,
        profile_identity_hash,
        change_path: path_string(change_relative),
    })
}

pub(crate) fn select_workflow_profile_for_create(
    policy: &Policy,
    profile_id: Option<&str>,
) -> Result<(WorkflowProfileSelection, WorkflowSchemaPolicy), Box<dyn Error>> {
    let selected_id = profile_id.unwrap_or(&policy.workflow_profile_policy.default_profile);
    let selected_schema = policy
        .workflow_schemas
        .iter()
        .find(|schema| schema.id == selected_id)
        .ok_or_else(|| {
            format!("cairn.workflow.profile.selection: unknown profile: {selected_id}")
        })?;
    let metadata = WorkflowChangeProfileMetadata {
        workflow_profile: Some(WorkflowProfileRef {
            id: selected_id.to_string(),
            version: selected_schema.profile_version,
            source: WORKFLOW_PROFILE_SOURCE_CREATE.to_string(),
        }),
        spec_effect: None,
        spec_effect_rationale: None,
    };
    select_workflow_profile(
        &policy.workflow_profile_policy,
        &policy.workflow_schemas,
        &metadata,
    )
    .map_err(Into::into)
}

fn read_change_metadata_value(
    root: &Path,
    change_relative: &Path,
) -> Result<Value, Box<dyn Error>> {
    let metadata_path = change_relative.join("metadata.json");
    let Some(text) = read_bounded_text(root, &metadata_path, MAX_WORKFLOW_ARTIFACT_BYTES_LIMIT)?
    else {
        return Ok(json!({}));
    };
    serde_json::from_str(&text).map_err(|error| {
        format!("cairn.workflow.profile.selection: metadata.json is invalid: {error}").into()
    })
}

fn read_admission_artifact(
    root: &Path,
    change_relative: &Path,
    schema: &WorkflowSchemaPolicy,
    admission: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    let Some(artifact) = schema
        .artifacts
        .iter()
        .find(|artifact| artifact.structural_admission == admission)
    else {
        return Ok(None);
    };
    read_bounded_text(
        root,
        &change_relative.join(&artifact.output_path),
        schema.bounds.max_artifact_bytes,
    )
}

fn read_review_evidence(
    root: &Path,
    change_relative: &Path,
    schema: &WorkflowSchemaPolicy,
    artifact_hashes: &[WorkflowProfileArtifactHash],
) -> Result<Vec<WorkflowReviewEvidence>, Box<dyn Error>> {
    let observed_hashes = artifact_hashes
        .iter()
        .map(|artifact| (artifact.path.clone(), artifact.blake3.clone()))
        .collect::<Vec<_>>();
    let mut reviews = Vec::new();
    for obligation in &schema.review_obligations {
        let Some(artifact) = schema
            .artifacts
            .iter()
            .find(|artifact| artifact.id == obligation.artifact_id)
        else {
            continue;
        };
        let Some(text) = read_bounded_text(
            root,
            &change_relative.join(&artifact.output_path),
            schema.bounds.max_artifact_bytes,
        )?
        else {
            continue;
        };
        let value = serde_json::from_str::<Value>(&text).unwrap_or(Value::Null);
        let report = validate_review_receipt(&value, &observed_hashes);
        let reviewed_inputs = value
            .get("reviewed_inputs")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|input| {
                Some(WorkflowReviewInputEvidence {
                    path: input.get("path")?.as_str()?.to_string(),
                    hash: input.get("hash")?.as_str()?.to_string(),
                })
            })
            .collect::<Vec<_>>();
        reviews.push(WorkflowReviewEvidence {
            obligation_id: obligation.id.clone(),
            valid: report.valid,
            review_kind: value
                .get("review_kind")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            decision: value
                .get("decision")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
            reviewed_inputs,
            receipt_hash: report.receipt_hash,
        });
    }
    Ok(reviews)
}

fn profile_artifact_hashes(
    change_relative: &Path,
    observations: &[WorkflowArtifactObservation],
) -> Vec<WorkflowProfileArtifactHash> {
    let mut hashes = observations
        .iter()
        .flat_map(|observation| {
            observation.files.iter().map(|file| {
                let path = Path::new(&file.path)
                    .strip_prefix(change_relative)
                    .map(path_string)
                    .unwrap_or_else(|_| file.path.clone());
                WorkflowProfileArtifactHash {
                    artifact_id: observation.artifact_id.clone(),
                    path,
                    blake3: file.blake3.clone(),
                }
            })
        })
        .collect::<Vec<_>>();
    hashes.sort_by(|left, right| {
        left.artifact_id
            .cmp(&right.artifact_id)
            .then(left.path.cmp(&right.path))
    });
    hashes
}

fn apply_profile_evaluation(
    report: &mut WorkflowStatusReport,
    evaluation: &WorkflowProfileEvaluation,
) {
    for issue in &evaluation.issues {
        let blocker = format!("{}:{}", issue.code, issue.message);
        if let Some(artifact_id) = &issue.artifact_id {
            if let Some(status) = report
                .artifacts
                .iter_mut()
                .find(|status| status.id == *artifact_id)
            {
                status.state = WorkflowArtifactState::Blocked;
                status.blockers.push(blocker.clone());
                status.blockers.sort();
                status.blockers.dedup();
            }
        }
        report.issues.push(blocker);
    }
    report.issues.sort();
    report.issues.dedup();
    report.valid = evaluation.issues.is_empty();
}

fn read_bounded_text(
    root: &Path,
    relative: &Path,
    max_bytes: usize,
) -> Result<Option<String>, Box<dyn Error>> {
    let path = root.join(relative);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "workflow profile artifact is not a safe file: {}",
            path.display()
        )
        .into());
    }
    let max_bytes = u64::try_from(max_bytes)?;
    if metadata.len() > max_bytes {
        return Err(format!(
            "workflow profile artifact exceeds its byte bound: {}",
            path.display()
        )
        .into());
    }
    Ok(Some(fs::read_to_string(path)?))
}

pub(crate) fn add_mcp_transport(mut value: Value) -> Value {
    if let Some(object) = value.as_object_mut() {
        object.remove("receipt_hash");
        object.remove("portable_receipt_hash");
        object.insert(
            "mcp".to_string(),
            serde_json::json!({"read_only": true, "destructive": false}),
        );
    }
    value
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    const TEST_DIRECTORY_PREFIX: &str = "cairn-workflow-shell";
    const VALID_SPEC: &str = "# Demo Delta\n\n## ADDED Requirements\n\n### Requirement: Demo\n\nr[demo.workflow] Cairn MUST keep this fixture deterministic.\n\n#### Scenario: Demo passes\n\nr[demo.workflow.pass]\n- GIVEN a valid fixture\n- WHEN validation runs\n- THEN the fixture MUST pass.\n";
    const VALID_PROPOSAL: &str = "## Why\n\nA deterministic test is required.\n\n## What Changes\n\n- Add a test.\n\n## Impact\n\n- Tests change.\n";
    const VALID_DESIGN: &str = "# Design\n\n## Decision\n\nUse a deterministic fixture.\n";
    const VALID_TASKS: &str = "# Tasks\n\n- [ ] [serial] Add the fixture. r[demo.workflow]\n";

    fn test_root(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{TEST_DIRECTORY_PREFIX}-{name}-{}",
            std::process::id()
        ))
    }

    fn policy_path() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../cairn-policy/generated/cairn-policy.json")
    }

    fn write_fixture(root: &Path) -> Result<(), Box<dyn Error>> {
        let change = root.join(".cairn/changes/demo");
        fs::create_dir_all(change.join("specs/demo"))?;
        fs::create_dir_all(root.join(".cairn/specs"))?;
        fs::create_dir_all(root.join(".cairn/archive"))?;
        fs::write(change.join("proposal.md"), VALID_PROPOSAL)?;
        fs::write(change.join("design.md"), VALID_DESIGN)?;
        fs::write(change.join("tasks.md"), VALID_TASKS)?;
        fs::write(change.join("specs/demo/spec.md"), VALID_SPEC)?;
        Ok(())
    }

    fn remove_fixture(root: &Path) -> Result<(), Box<dyn Error>> {
        match fs::remove_dir_all(root) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    #[test]
    fn status_and_instructions_are_deterministic_and_read_only() -> Result<(), Box<dyn Error>> {
        // r[verify cairn.artifact_workflow_contract.cli.inspect]
        // r[verify cairn.artifact_workflow_contract.instructions.ready]
        let root = test_root("positive");
        remove_fixture(&root)?;
        write_fixture(&root)?;
        let tasks_before = fs::read(root.join(".cairn/changes/demo/tasks.md"))?;
        let first = workflow_status_value(&root, "demo", None, &policy_path())?;
        let second = workflow_status_value(&root, "demo", None, &policy_path())?;
        assert_eq!(first, second);
        assert_eq!(first["valid"], true);
        assert_eq!(first["artifacts"][0]["state"], "done");
        let instructions =
            workflow_instructions_value(&root, "demo", "tasks", None, &policy_path())?;
        assert_eq!(instructions["artifact"]["artifact_id"], "tasks");
        assert_eq!(instructions["read_only"], true);
        assert_eq!(
            fs::read(root.join(".cairn/changes/demo/tasks.md"))?,
            tasks_before
        );
        remove_fixture(&root)?;
        Ok(())
    }

    #[test]
    fn cli_and_mcp_return_equivalent_facts_and_reject_unknown_fields() -> Result<(), Box<dyn Error>>
    {
        // r[verify cairn.artifact_workflow_contract.mcp.parity]
        // r[verify cairn.artifact_workflow_contract.mcp.invalid]
        let root = test_root("mcp");
        remove_fixture(&root)?;
        write_fixture(&root)?;
        let policy = policy_path();
        let direct = workflow_status_value(&root, "demo", None, &policy)?;
        let mcp = crate::mcp::mcp_call_tool(
            "cairn_workflow_status",
            &serde_json::json!({
                "root": root,
                "change": "demo",
                "policy": policy,
            }),
            false,
        )?;
        let structured = &mcp["structuredContent"];
        assert_eq!(structured["artifacts"], direct["artifacts"]);
        assert_eq!(structured["input_hash"], direct["input_hash"]);
        assert_eq!(structured["mcp"]["read_only"], true);
        assert!(crate::mcp::mcp_call_tool(
            "cairn_workflow_status",
            &serde_json::json!({
                "root": root,
                "change": "demo",
                "unknown": "rejected",
            }),
            false,
        )
        .is_err());
        remove_fixture(&root)?;
        Ok(())
    }

    #[test]
    fn existing_lifecycle_receipt_fields_remain_available() -> Result<(), Box<dyn Error>> {
        // r[verify cairn.artifact_workflow_contract.compatibility]
        // r[verify cairn.artifact_workflow_contract.compatibility.existing]
        let root = test_root("compatibility");
        remove_fixture(&root)?;
        write_fixture(&root)?;
        let policy = policy_path();
        let cases = [
            (
                "cairn_status",
                serde_json::json!({"root": root}),
                &["root", "layout", "specs", "changes", "receipt_hash"][..],
            ),
            (
                "cairn_gate",
                serde_json::json!({
                    "root": root,
                    "change": "demo",
                    "stage": "tasks",
                    "policy": policy,
                }),
                &[
                    "change",
                    "stage",
                    "verdict",
                    "issues",
                    "policy_hash",
                    "input_hash",
                    "receipt_hash",
                ][..],
            ),
            (
                "cairn_sync_plan",
                serde_json::json!({
                    "root": root,
                    "change": "demo",
                    "policy": policy,
                }),
                &[
                    "actions",
                    "blocked",
                    "change",
                    "dry_run",
                    "input_hash",
                    "plan_hash",
                    "receipt_hash",
                ][..],
            ),
            (
                "cairn_archive_plan",
                serde_json::json!({
                    "root": root,
                    "change": "demo",
                    "policy": policy,
                }),
                &[
                    "actions",
                    "blocked",
                    "change",
                    "dry_run",
                    "input_hash",
                    "plan_hash",
                    "receipt_hash",
                ][..],
            ),
        ];
        for (tool, input, required_fields) in cases {
            let output = crate::mcp::mcp_call_tool(tool, &input, false)?;
            let structured = &output["structuredContent"];
            for field in required_fields {
                assert!(structured.get(field).is_some(), "{tool} lost {field}");
            }
            assert!(structured.get("workflow_schema").is_none());
        }
        remove_fixture(&root)?;
        Ok(())
    }

    #[test]
    fn workflow_guide_states_commands_and_authority_boundaries() -> Result<(), Box<dyn Error>> {
        // r[verify cairn.artifact_workflow_contract.docs]
        // r[verify cairn.artifact_workflow_contract.docs.boundary]
        // r[verify cairn.artifact_workflow_contract.boundary.nonclaim]
        let guide = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docs/artifact-workflow.md"),
        )?;
        assert!(guide.contains("cairn workflow status CHANGE --root PATH"));
        assert!(guide.contains("cairn workflow instructions ARTIFACT --change CHANGE --root PATH"));
        assert!(guide.contains("Workflow output proves only the selected policy graph"));
        assert!(guide.contains("release eligibility"));
        assert!(guide.contains("proposal, design, and tasks gates as separate authority checks"));
        Ok(())
    }

    #[test]
    fn malformed_artifact_and_unknown_selectors_fail_closed() -> Result<(), Box<dyn Error>> {
        // r[verify cairn.artifact_workflow_contract.cli.invalid]
        // r[verify cairn.artifact_workflow_contract.status.invalid]
        let root = test_root("negative");
        remove_fixture(&root)?;
        write_fixture(&root)?;
        fs::write(
            root.join(".cairn/changes/demo/specs/demo/spec.md"),
            "CAIRN_PLACEHOLDER",
        )?;
        let status = workflow_status_value(&root, "demo", None, &policy_path())?;
        assert_eq!(status["artifacts"][1]["state"], "blocked");
        assert!(workflow_status_value(&root, "demo", Some("unknown"), &policy_path()).is_err());
        let error = workflow_instructions_value(&root, "demo", "unknown", None, &policy_path())
            .expect_err("unknown artifact fails");
        assert!(error
            .to_string()
            .contains(cairn_core::WORKFLOW_ISSUE_ARTIFACT_UNKNOWN));
        remove_fixture(&root)?;
        Ok(())
    }
}
