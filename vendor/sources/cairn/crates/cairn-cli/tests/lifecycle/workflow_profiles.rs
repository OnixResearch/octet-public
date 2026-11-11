use std::{error::Error, fs, path::Path};

use serde_json::json;

use super::support::{cairn, run_json, run_json_command, temp_project};

const VALID_PROPOSAL: &str = "## Why\n\nA bounded workflow change is required.\n\n## What Changes\n\n- Add deterministic profile coverage.\n\n## Impact\n\n- Tests and lifecycle evidence change.\n";
const VALID_DESIGN: &str = "# Design\n\n## Context\n\nThe workflow needs deterministic profile behavior.\n\n## Decisions\n\n### Decision: Use checked policy\n\n**Choice:** Use one checked profile projection.\n\n**Rationale:** This keeps lifecycle authority explicit.\n";
const VALID_SPEC: &str = "# Profile test delta\n\n## ADDED Requirements\n\n### Requirement: Keep the profile deterministic\n\nr[test.profile] Cairn MUST keep this profile deterministic.\n\n#### Scenario: Deterministic profile\n\nr[test.profile.deterministic]\n- GIVEN a valid profile\n- WHEN Cairn evaluates it\n- THEN the result MUST be deterministic.\n";
const VALID_SPEC_TASKS: &str = "# Tasks\n\n- [x] [serial] Implement the profile behavior. r[test.profile]\n- [x] [serial] Verify positive and negative behavior. r[test.profile.deterministic]\n";
const VALID_ACCEPTANCE: &str = "# Change-local acceptance\n\na[test.local-first] The workflow reports the selected profile.\na[test.local-second] Sync records an explicit no-op.\n";
const VALID_ACCEPTANCE_TASKS: &str = "# Tasks\n\n- [x] [serial] Report the selected profile. a[test.local-first]\n- [x] [serial] Verify the no-op sync. a[test.local-second]\n";

#[test]
fn profile_scaffolds_are_exact_and_unknown_profiles_do_not_mutate() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_workflow_profiles.selection.create]
    // r[verify cairn.lifecycle_workflow_profiles.selection.invalid]
    // r[verify cairn.lifecycle_workflow_profiles.verification]
    // r[verify cairn.lifecycle_workflow_profiles.verification.suites]
    let root = temp_project("workflow-profile-scaffolds")?;
    let root_text = root.to_str().ok_or("utf8 root")?;
    run_json(&["init", "--root", root_text])?;

    let assurance = run_json(&[
        "change",
        "create",
        "assurance-demo",
        "--profile",
        "assurance",
        "--root",
        root_text,
    ])?;
    assert_eq!(assurance["workflow_profile"], "assurance");
    let assurance_root = root.join(".cairn/changes/assurance-demo");
    for path in [
        "proposal.md",
        "design.md",
        "risk.md",
        "tasks.md",
        "verification.md",
        "reviews/assurance.json",
        "metadata.json",
    ] {
        assert!(assurance_root.join(path).is_file(), "missing {path}");
    }
    assert!(assurance_root.join("specs").is_dir());
    assert!(!assurance_root.join("acceptance.md").exists());

    let no_spec = run_json(&[
        "change",
        "create",
        "no-spec-demo",
        "--profile",
        "no-spec-delta",
        "--root",
        root_text,
    ])?;
    assert_eq!(no_spec["workflow_profile"], "no-spec-delta");
    let no_spec_root = root.join(".cairn/changes/no-spec-demo");
    assert!(no_spec_root.join("acceptance.md").is_file());
    assert!(no_spec_root
        .join("reviews/no-spec-classification.json")
        .is_file());
    assert!(!no_spec_root.join("specs").exists());
    let metadata: serde_json::Value =
        serde_json::from_slice(&fs::read(no_spec_root.join("metadata.json"))?)?;
    assert_eq!(metadata["workflow_profile"]["id"], "no-spec-delta");
    assert_eq!(metadata["workflow_profile"]["version"], 1);
    assert_eq!(metadata["spec_effect"], "none");

    let output = cairn()
        .args([
            "change",
            "create",
            "unknown-demo",
            "--profile",
            "unknown",
            "--root",
            root_text,
        ])
        .output()?;
    assert!(!output.status.success());
    assert!(!root.join(".cairn/changes/unknown-demo").exists());

    let ambient = cairn()
        .env("CAIRN_WORKFLOW_PROFILE", "no-spec-delta")
        .args(["change", "create", "ambient-demo", "--root", root_text])
        .output()?;
    assert!(ambient.status.success());
    let ambient_metadata: serde_json::Value = serde_json::from_slice(&fs::read(
        root.join(".cairn/changes/ambient-demo/metadata.json"),
    )?)?;
    assert_eq!(ambient_metadata["workflow_profile"]["id"], "spec-driven");

    fs::write(
        assurance_root.join("proposal.md"),
        "## Why\n\nThis profile selection is now substantive.\n",
    )?;
    let drift = cairn()
        .args([
            "change",
            "create",
            "assurance-demo",
            "--profile",
            "no-spec-delta",
            "--root",
            root_text,
        ])
        .output()?;
    assert!(!drift.status.success());
    assert!(!assurance_root.join("acceptance.md").exists());
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn assurance_profile_requires_current_reviewed_obligations() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_workflow_profiles.assurance.complete]
    // r[verify cairn.lifecycle_workflow_profiles.assurance.blocked]
    let root = temp_project("workflow-profile-assurance")?;
    let root_text = root.to_str().ok_or("utf8 root")?;
    run_json(&["init", "--root", root_text])?;
    run_json(&[
        "change",
        "create",
        "assurance-demo",
        "--profile",
        "assurance",
        "--root",
        root_text,
    ])?;
    let change = root.join(".cairn/changes/assurance-demo");
    write_assurance_artifacts(&change)?;
    write_review(
        &change,
        "reviews/assurance.json",
        &[
            "proposal.md",
            "specs/demo/spec.md",
            "design.md",
            "risk.md",
            "tasks.md",
            "verification.md",
        ],
    )?;

    let status = run_json(&["workflow", "status", "assurance-demo", "--root", root_text])?;
    assert_eq!(status["valid"], true);
    assert_eq!(status["workflow_profile"], "assurance");
    assert!(status["artifacts"]
        .as_array()
        .ok_or("artifacts")?
        .iter()
        .all(|artifact| artifact["state"] == "done"));
    let gate = run_json(&["gate", "tasks", "assurance-demo", "--root", root_text])?;
    assert_eq!(gate["verdict"], "PASS");

    fs::write(
        change.join("risk.md"),
        "# Changed risk\n\nThe review is now stale.\n",
    )?;
    let stale = run_json(&["gate", "tasks", "assurance-demo", "--root", root_text])?;
    assert_eq!(stale["verdict"], "FAIL");
    assert!(stale["issues"]
        .to_string()
        .contains("cairn.workflow.profile.review"));
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn no_spec_profile_syncs_as_no_op_and_rechecks_before_archive() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_workflow_profiles.no_spec.valid]
    // r[verify cairn.lifecycle_workflow_profiles.no_spec_sync.preview]
    // r[verify cairn.lifecycle_workflow_profiles.no_spec_sync.archive]
    // r[verify cairn.lifecycle_workflow_profiles.functional_boundary.shell]
    let root = temp_project("workflow-profile-no-spec")?;
    let root_text = root.to_str().ok_or("utf8 root")?;
    run_json(&["init", "--root", root_text])?;
    run_json(&[
        "change",
        "create",
        "no-spec-demo",
        "--profile",
        "no-spec-delta",
        "--root",
        root_text,
    ])?;
    let change = root.join(".cairn/changes/no-spec-demo");
    write_no_spec_artifacts(&change)?;
    write_review(
        &change,
        "reviews/no-spec-classification.json",
        &["proposal.md", "design.md", "acceptance.md", "tasks.md"],
    )?;

    let gate = run_json(&["gate", "tasks", "no-spec-demo", "--root", root_text])?;
    assert_eq!(gate["verdict"], "PASS");
    let sync = run_json(&["sync", "no-spec-demo", "--root", root_text])?;
    assert_eq!(sync["blocked"], false);
    assert_eq!(sync["actions"], json!([]));
    assert_eq!(sync["spec_effect"], "none");
    assert_eq!(
        sync["acceptance_ids"],
        json!(["test.local-first", "test.local-second"])
    );
    let executed = run_json(&["sync", "no-spec-demo", "--root", root_text, "--execute"])?;
    assert_eq!(executed["mutated"], false);
    assert!(fs::read_dir(root.join(".cairn/specs"))?.next().is_none());

    fs::write(
        change.join("acceptance.md"),
        format!("{VALID_ACCEPTANCE}\na[test.changed] The review is stale.\n"),
    )?;
    let stale = run_json(&["archive", "no-spec-demo", "--root", root_text])?;
    assert_eq!(stale["blocked"], true);
    assert!(change.exists());

    fs::write(change.join("acceptance.md"), VALID_ACCEPTANCE)?;
    write_review(
        &change,
        "reviews/no-spec-classification.json",
        &["proposal.md", "design.md", "acceptance.md", "tasks.md"],
    )?;
    let archived = run_json(&["archive", "no-spec-demo", "--root", root_text, "--execute"])?;
    assert_eq!(archived["blocked"], false);
    assert_eq!(archived["mutated"], true);
    assert!(!change.exists());
    assert!(
        fs::read_dir(root.join(".cairn/archive"))?.any(|entry| entry.ok().is_some_and(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .ends_with("no-spec-demo")
        }))
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn mcp_create_and_workflow_status_match_the_cli_profile() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_workflow_profiles.integration]
    // r[verify cairn.lifecycle_workflow_profiles.verification.suites]
    let root = temp_project("workflow-profile-mcp")?;
    let root_text = root.to_str().ok_or("utf8 root")?;
    run_json(&["init", "--root", root_text])?;
    let create_input = json!({
        "root": root_text,
        "change": "mcp-profile-demo",
        "profile": "assurance",
    })
    .to_string();
    let created = run_json_command(cairn().args([
        "mcp",
        "--allow-mutations",
        "--call",
        "cairn_change_create",
        &create_input,
    ]))?;
    assert_eq!(
        created["structuredContent"]["workflow_profile"],
        "assurance"
    );

    let cli = run_json(&[
        "workflow",
        "status",
        "mcp-profile-demo",
        "--root",
        root_text,
    ])?;
    let status_input = json!({"root": root_text, "change": "mcp-profile-demo"}).to_string();
    let mcp =
        run_json_command(cairn().args(["mcp", "--call", "cairn_workflow_status", &status_input]))?;
    let structured = &mcp["structuredContent"];
    assert_eq!(structured["workflow_profile"], cli["workflow_profile"]);
    assert_eq!(structured["spec_effect"], cli["spec_effect"]);
    assert_eq!(
        structured["profile_identity_hash"],
        cli["profile_identity_hash"]
    );
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn workflow_profile_guide_names_authority_and_non_claims() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_workflow_profiles.docs]
    // r[verify cairn.lifecycle_workflow_profiles.docs.selection]
    // r[verify cairn.lifecycle_workflow_profiles.boundary.nonclaim]
    let guide = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/workflow-profiles.md"),
    )?;
    assert!(guide.contains("spec-driven"));
    assert!(guide.contains("assurance"));
    assert!(guide.contains("no-spec-delta"));
    assert!(guide
        .contains("user-global files, environment variables, agent prompts, or remote packages"));
    assert!(guide.contains("does not prove zero behavior effect"));
    assert!(guide.contains("release eligibility"));
    Ok(())
}

fn write_assurance_artifacts(change: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(change.join("specs/demo"))?;
    fs::write(change.join("proposal.md"), VALID_PROPOSAL)?;
    fs::write(change.join("design.md"), VALID_DESIGN)?;
    fs::write(change.join("specs/demo/spec.md"), VALID_SPEC)?;
    fs::write(
        change.join("risk.md"),
        "# Risk analysis\n\n## Assets and hazards\n\nThe lifecycle profile is the bounded asset. Stale evidence is the hazard.\n\n## Mitigations and residual risks\n\nExact BLAKE3 links mitigate stale evidence. Reviewers own residual semantic risk.\n",
    )?;
    fs::write(change.join("tasks.md"), VALID_SPEC_TASKS)?;
    fs::write(
        change.join("verification.md"),
        "# Verification plan\n\n## Positive checks\n\nRun focused profile and lifecycle tests.\n\n## Negative checks and gaps\n\nReject unknown profiles and stale review hashes. These checks do not prove test sufficiency.\n",
    )?;
    Ok(())
}

fn write_no_spec_artifacts(change: &Path) -> Result<(), Box<dyn Error>> {
    fs::write(change.join("proposal.md"), VALID_PROPOSAL)?;
    fs::write(change.join("design.md"), VALID_DESIGN)?;
    fs::write(change.join("acceptance.md"), VALID_ACCEPTANCE)?;
    fs::write(change.join("tasks.md"), VALID_ACCEPTANCE_TASKS)?;
    let metadata_path = change.join("metadata.json");
    let mut metadata: serde_json::Value = serde_json::from_slice(&fs::read(&metadata_path)?)?;
    metadata["spec_effect_rationale"] =
        json!("This internal evidence-only fixture does not change accepted requirements.");
    fs::write(metadata_path, serde_json::to_vec_pretty(&metadata)?)?;
    Ok(())
}

fn write_review(
    change: &Path,
    review_path: &str,
    required_paths: &[&str],
) -> Result<(), Box<dyn Error>> {
    let reviewed_inputs = required_paths
        .iter()
        .map(|path| {
            Ok(json!({
                "path": path,
                "hash": cairn_core::blake3_hex(&fs::read(change.join(path))?),
            }))
        })
        .collect::<Result<Vec<_>, Box<dyn Error>>>()?;
    let receipt = json!({
        "schema_version": "cairn.semantic-review.review-receipt.v1",
        "review_kind": "plan",
        "decision": "approved",
        "reviewed_inputs": reviewed_inputs,
        "reviewer": {"source": "manual", "id": "profile-test-reviewer"},
        "findings": [],
    });
    let destination = change.join(review_path);
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(destination, serde_json::to_vec_pretty(&receipt)?)?;
    Ok(())
}
