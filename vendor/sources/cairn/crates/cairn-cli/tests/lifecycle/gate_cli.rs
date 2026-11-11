use std::{error::Error, fs};

use super::support::{
    assert_hash_field, assert_reproducible_receipt, cairn, run_json, temp_fixture,
};

#[test]
fn receipt_hashes_are_reproducible() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.blake3_receipts.gate]
    // r[verify cairn.supersede.blake3_receipts.lifecycle_plans]
    // r[verify cairn.supersede.blake3_receipts.migration]
    // r[verify cairn.supersede.blake3_receipts.canonical]
    // r[verify cairn.supersede.deterministic_gates.stage]
    let gate_hashes = ["policy_hash", "input_hash", "receipt_hash"];
    assert_reproducible_receipt(
        &["gate", "proposal", "demo", "--root", "fixtures/basic"],
        &gate_hashes,
    )?;
    assert_reproducible_receipt(
        &["gate", "design", "demo", "--root", "fixtures/basic"],
        &gate_hashes,
    )?;
    assert_reproducible_receipt(
        &["gate", "tasks", "demo", "--root", "fixtures/basic"],
        &gate_hashes,
    )?;

    let lifecycle_hashes = ["policy_hash", "input_hash", "plan_hash", "receipt_hash"];
    assert_reproducible_receipt(
        &["sync", "demo", "--root", "fixtures/basic"],
        &lifecycle_hashes,
    )?;
    assert_reproducible_receipt(
        &["archive", "ready", "--root", "fixtures/archive-ready"],
        &lifecycle_hashes,
    )?;

    let migration = assert_reproducible_receipt(
        &["migrate", "openspec", "--root", "fixtures/basic"],
        &[
            "policy_hash",
            "source_manifest_hash",
            "destination_manifest_hash",
            "manifest_hash",
            "receipt_hash",
        ],
    )?;
    let actions = migration["actions"].as_array().ok_or("migration actions")?;
    assert!(!actions.is_empty());
    for action in actions {
        assert_hash_field(action, "source_hash")?;
        assert_hash_field(action, "destination_hash")?;
    }
    Ok(())
}

#[test]
fn help_lists_commands() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.hermes_skill] r[verify cairn.supersede.hermes_skill.discover]
    // r[verify cairn.supersede.hermes_skill.contracts] r[verify cairn.supersede.hermes_skill.audit]
    // r[verify cairn.supersede.hermes_skill.canonical_cairn]
    // r[verify cairn.supersede.hermes_skill.canonical_cairn.path]
    // r[verify cairn.supersede.hermes_skill.canonical_cairn.live]
    // r[verify cairn.supersede.hermes_skill.redirect]
    // r[verify cairn.supersede.hermes_skill.redirect.local]
    let output = cairn().arg("--help").output()?;
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("usage: cairn <command> [args]"), "{stdout}");
    assert!(stdout.contains("init [--root PATH]"), "{stdout}");
    assert!(
        stdout.contains("change create <name> [--profile PROFILE] [--root PATH] [--policy PATH]"),
        "{stdout}"
    );
    assert!(stdout.contains("change list [--root PATH]"), "{stdout}");
    assert!(stdout.contains("change graph [--root PATH]"), "{stdout}");
    assert!(
        stdout.contains("change show <name> [--root PATH]"),
        "{stdout}"
    );
    assert!(
        stdout.contains("spec list [--root PATH | --lifecycle-store ID]"),
        "{stdout}"
    );
    assert!(
        stdout.contains("spec show <id> [--root PATH | --lifecycle-store ID]"),
        "{stdout}"
    );
    assert!(
        stdout.contains("context [--root PATH | --lifecycle-store ID]"),
        "{stdout}"
    );
    assert!(
        stdout.contains(
            "migrate <cairn|openspec|ironspec> [--root PATH] [--policy PATH] [--execute --plan-hash HASH]"
        ),
        "{stdout}"
    );
    assert!(
        stdout.contains("gate <proposal|design|tasks> <change> [--root PATH] [--policy PATH]"),
        "{stdout}"
    );
    assert!(
        stdout.contains("sync <change> [--root PATH] [--policy PATH] [--cross-repo-evidence FILE|DIR ...] [--execute]"),
        "{stdout}"
    );
    assert!(
        stdout.contains("archive <change> [--root PATH] [--policy PATH] [--cross-repo-evidence FILE|DIR ...] [--execute]"),
        "{stdout}"
    );
    Ok(())
}

#[test]
fn unknown_command_returns_usage_error() -> Result<(), Box<dyn Error>> {
    let output = cairn().arg("bogus").output()?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown command: bogus"), "{stderr}");
    assert!(stderr.contains("usage: cairn <command> [args]"), "{stderr}");
    Ok(())
}

#[test]
fn gate_context_reports_named_change() -> Result<(), Box<dyn Error>> {
    let value = run_json(&["gate-context", "demo", "--root", "fixtures/basic"])?;
    assert_eq!(value["name"], "demo");
    assert_eq!(value["path"], "fixtures/basic/.cairn/changes/demo");
    assert_eq!(
        value["delta_specs"].as_array().ok_or("delta specs")?.len(),
        1
    );
    Ok(())
}

#[test]
fn gate_reports_stage_verdicts() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.gate_policy]
    let pass = run_json(&["gate", "tasks", "demo", "--root", "fixtures/basic"])?;
    assert_eq!(pass["change"], "demo");
    assert_eq!(pass["stage"], "tasks");
    assert_eq!(pass["verdict"], "PASS");
    assert_eq!(pass["valid"], true);
    assert_eq!(pass["policy"], "cairn-default");
    assert_eq!(pass["substance"].as_array().ok_or("substance")?.len(), 4);
    assert_eq!(pass["evidence_summary"]["mode"], "advisory");
    assert!(pass["evidence_summary"].get("docs_only").is_some());

    let fail = run_json(&[
        "gate",
        "design",
        "missing",
        "--root",
        "fixtures/missing-artifacts",
    ])?;
    assert_eq!(fail["verdict"], "FAIL");
    let messages = fail["issues"]
        .as_array()
        .ok_or("issues")?
        .iter()
        .map(|issue| issue["message"].as_str().ok_or("message"))
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(
        messages,
        vec!["missing: missing proposal.md", "missing: missing design.md"]
    );
    assert_eq!(fail["issues"][0]["code"], "cairn.gate.validation");
    assert_eq!(fail["issues"][0]["scope"], "tasks");
    assert_eq!(fail["issues"][0]["promoted"], "no");
    Ok(())
}

#[test]
fn gate_checks_references_task_links_and_exemptions() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.gate_policy.references]
    // r[verify cairn.supersede.gate_policy.task_links]
    // r[verify cairn.supersede.gate_policy.exemptions]
    let fixture = temp_fixture("gate-reference-checks", "fixtures/basic")?;
    let change_dir = fixture.join(".cairn/changes/demo");
    fs::write(
        change_dir.join("proposal.md"),
        "## Why\n\nr[demo.unknown] gate-exempt[reason=test owner=agent]\n",
    )?;
    fs::write(
        change_dir.join("tasks.md"),
        "## Phase 1\n\n- [ ] [serial] unlinked task\n- [ ] [parallel] r[demo.behavior] linked task\n",
    )?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;

    let value = run_json(&["gate", "tasks", "demo", "--root", root])?;
    assert_eq!(value["verdict"], "FAIL");
    let messages = value["issues"]
        .as_array()
        .ok_or("issues")?
        .iter()
        .map(|issue| issue["message"].as_str().ok_or("message"))
        .collect::<Result<Vec<_>, _>>()?;
    assert!(messages
        .iter()
        .any(|message| message.contains("unknown requirement reference r[demo.unknown]")));
    assert!(messages
        .iter()
        .any(|message| message.contains("malformed gate exemption")));
    assert!(messages
        .iter()
        .any(|message| message.contains("task missing known requirement reference")));
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn gate_accepts_bounded_exemptions_and_dependency_tasks() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.gate_policy.references]
    // r[verify cairn.supersede.gate_policy.task_links]
    // r[verify cairn.supersede.gate_policy.exemptions]
    let fixture = temp_fixture("gate-bounded-exemptions", "fixtures/basic")?;
    let change_dir = fixture.join(".cairn/changes/demo");
    fs::write(
        change_dir.join("proposal.md"),
        "## Why\n\nThis proposal has reviewed content. gate-exempt[reason=test owner=agent until=2026-12-31]\n",
    )?;
    fs::write(
        change_dir.join("tasks.md"),
        "## Phase 1\n\n- [ ] [serial] r[demo.behavior] linked task\n- [ ] [depends:demo.behavior] dependency task\n",
    )?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;

    let value = run_json(&["gate", "tasks", "demo", "--root", root])?;
    assert_eq!(value["verdict"], "PASS");
    assert_eq!(value["valid"], true);
    fs::remove_dir_all(fixture)?;
    Ok(())
}
