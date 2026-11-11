use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use cairn_core::cross_repo_dependency_contract;
use serde_json::{json, Value};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(root());
    command
}

fn run_json(command: &mut Command) -> Result<Value, Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn copy_dir(source: &Path, destination: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)?.collect::<Result<Vec<_>, _>>()? {
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            copy_dir(&entry.path(), &target)?;
        } else {
            fs::copy(entry.path(), target)?;
        }
    }
    Ok(())
}

fn fixture(name: &str) -> Result<(PathBuf, PathBuf), Box<dyn Error>> {
    let fixture =
        std::env::temp_dir().join(format!("cairn-cross-repo-{name}-{}", std::process::id()));
    if fixture.exists() {
        fs::remove_dir_all(&fixture)?;
    }
    copy_dir(&root().join("fixtures/archive-ready"), &fixture)?;
    let matrix: Value = serde_json::from_slice(&fs::read(
        root().join("fixtures/cross-repo-dependencies/matrix.json"),
    )?)?;
    let row = &matrix["positive"][0];
    fs::write(
        fixture.join(".cairn/changes/ready/metadata.json"),
        serde_json::to_vec_pretty(&json!({
            "cross_repo_dependencies": [row["dependency"].clone()]
        }))?,
    )?;
    let evidence = fixture.join("upstream.json");
    fs::write(&evidence, serde_json::to_vec_pretty(&row["evidence"])?)?;
    Ok((fixture, evidence))
}

#[test]
// r[verify cairn.cross_repo_change_dependencies.contract]
// r[verify cairn.cross_repo_change_dependencies.contract.identity]
// r[verify cairn.cross_repo_change_dependencies.contract.invalid]
// r[verify cairn.cross_repo_change_dependencies.contract.local_boundary]
// r[verify cairn.cross_repo_change_dependencies.docs]
// r[verify cairn.cross_repo_change_dependencies.docs.metadata]
// r[verify cairn.cross_repo_change_dependencies.docs.non_claims]
// r[verify cairn.cross_repo_change_dependencies.docs.local_boundary]
fn generated_policy_matches_the_core_contract() -> Result<(), Box<dyn Error>> {
    let policy: Value = serde_json::from_slice(&fs::read(
        root().join("cairn-policy/generated/cairn-policy.json"),
    )?)?;
    let core = cross_repo_dependency_contract();
    let generated = &policy["cross_repo_dependency_policy"];
    for field in [
        "schema",
        "supported_receipt_kinds",
        "passing_status",
        "non_passing_statuses",
        "required_non_claims",
    ] {
        assert_eq!(generated[field], core[field], "policy drift in {field}");
    }
    Ok(())
}

#[test]
// r[verify cairn.cross_repo_change_dependencies.shell.explicit]
// r[verify cairn.cross_repo_change_dependencies.blockers]
// r[verify cairn.cross_repo_change_dependencies.blockers.validation]
// r[verify cairn.cross_repo_change_dependencies.final_validation.positive]
// r[verify cairn.cross_repo_change_dependencies.status_feedback.pass]
fn explicit_matching_evidence_satisfies_cli_and_mcp_validation() -> Result<(), Box<dyn Error>> {
    let (fixture, evidence) = fixture("positive")?;
    let receipt = run_json(cairn().args([
        "validate",
        "--root",
        fixture.to_str().ok_or("fixture path")?,
        "--cross-repo-evidence",
        evidence.to_str().ok_or("evidence path")?,
    ]))?;
    assert_eq!(receipt["valid"], true, "{receipt:#}");
    assert_eq!(
        receipt["cross_repo_dependencies"][0]["report"]["valid"],
        true
    );
    let readiness = run_json(cairn().args([
        "release-readiness",
        "--root",
        fixture.to_str().ok_or("fixture path")?,
        "--cross-repo-evidence",
        evidence.to_str().ok_or("evidence path")?,
    ]))?;
    assert_eq!(
        readiness["cross_repo_dependencies"][0]["report"]["valid"], true,
        "{readiness:#}"
    );

    let input = json!({
        "root": fixture,
        "cross_repo_evidence": [evidence]
    })
    .to_string();
    let mcp = run_json(cairn().args(["mcp", "--call", "cairn_validate", &input]))?;
    assert_eq!(mcp["structuredContent"]["valid"], true, "{mcp:#}");
    let mutation_input = json!({
        "root": fixture,
        "change": "ready",
        "cross_repo_evidence": [evidence]
    })
    .to_string();
    let plan = run_json(cairn().args(["mcp", "--call", "cairn_sync_plan", &mutation_input]))?;
    assert_eq!(
        plan["structuredContent"]["cross_repo_dependencies"]["valid"], true,
        "{plan:#}"
    );
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
// r[verify cairn.cross_repo_change_dependencies.shell.no_fetch]
// r[verify cairn.cross_repo_change_dependencies.blockers.planning]
// r[verify cairn.cross_repo_change_dependencies.final_validation]
// r[verify cairn.cross_repo_change_dependencies.final_validation.negative]
// r[verify cairn.cross_repo_change_dependencies.final_validation.gates]
// r[verify cairn.cross_repo_change_dependencies.status_feedback.diagnostic]
// r[verify cairn.cross_repo_change_dependencies.validation.mismatch]
fn absent_or_unsafe_evidence_blocks_without_mutation() -> Result<(), Box<dyn Error>> {
    let (fixture, evidence) = fixture("negative")?;
    let accepted = fixture.join(".cairn/specs/demo/spec.md");
    let before = fs::read(&accepted)?;
    let root_text = fixture.to_str().ok_or("fixture path")?;

    let validation = run_json(cairn().args(["validate", "--root", root_text]))?;
    assert_eq!(validation["valid"], false);
    assert_eq!(
        validation["cross_repo_dependencies"][0]["report"]["rows"][0]["status"],
        "missing"
    );

    let blocked = run_json(cairn().args(["sync", "ready", "--root", root_text, "--execute"]))?;
    assert_eq!(blocked["blocked"], true);
    assert_eq!(blocked["mutated"], false);
    assert_eq!(fs::read(&accepted)?, before);
    let mutation_input = json!({"root": fixture, "change": "ready"}).to_string();
    let mcp_blocked = run_json(cairn().args([
        "mcp",
        "--allow-mutations",
        "--call",
        "cairn_sync_execute",
        &mutation_input,
    ]))?;
    assert_eq!(mcp_blocked["structuredContent"]["blocked"], true);
    assert_eq!(mcp_blocked["structuredContent"]["mutated"], false);
    assert_eq!(fs::read(&accepted)?, before);

    let mut unsafe_summary: Value = serde_json::from_slice(&fs::read(&evidence)?)?;
    unsafe_summary["diagnostic_summary"] = json!("token=raw-secret");
    fs::write(&evidence, serde_json::to_vec_pretty(&unsafe_summary)?)?;
    let unsafe_receipt = run_json(cairn().args([
        "validate",
        "--root",
        root_text,
        "--cross-repo-evidence",
        evidence.to_str().ok_or("evidence path")?,
    ]))?;
    assert_eq!(unsafe_receipt["valid"], false);
    let rendered = serde_json::to_string(&unsafe_receipt["cross_repo_dependencies"])?;
    assert!(rendered.contains("cross_repo_dependency.diagnostic_unsafe"));
    assert!(!rendered.contains("raw-secret"));
    assert_eq!(fs::read(&accepted)?, before);
    fs::remove_dir_all(fixture)?;
    Ok(())
}
