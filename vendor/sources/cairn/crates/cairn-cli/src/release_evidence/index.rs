// r[impl release_evidence.cli_source_split.index_module]
// r[impl release_evidence.current_head_snapshot.generate]
// r[impl release_evidence.current_head_snapshot.docs_boundary]
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::canonical_json_hash;
use serde_json::{json, Value};

use crate::agent_receipt::{receipt_stated_or_computed_hash, AGENT_RECEIPT_MAX_BYTES};

use crate::evidence::{
    evidence_export_receipt, evidence_import_receipt, evidence_index_receipt,
    evidence_index_verify_receipt, evidence_republish_receipt, evidence_verify_receipt,
    EvidenceExportOptions, EvidenceImportOptions, EvidenceIndexOptions, EvidenceIndexVerifyOptions,
    EvidenceRepublishOptions, EvidenceVerifyOptions,
};
use crate::support::{
    default_policy_path, path_string, print_json, read_option, read_string_option,
    with_portable_content_hash_receipt, CapabilityRoot, CapabilityRootKind,
};

use super::common::{
    collect_invalid_receipt, git_head_state, release_evidence_issue, write_json_artifact,
    write_markdown_artifact,
};

pub(super) fn run_release_evidence_index(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = release_evidence_index_options(&args)?;
    let receipt = release_evidence_index_receipt(&options);
    let valid = receipt
        .get("valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    print_json(receipt)?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

struct ReleaseEvidenceIndexOptions {
    root: PathBuf,
    out: PathBuf,
    artifact_dir: PathBuf,
}

fn release_evidence_index_options(
    args: &[String],
) -> Result<ReleaseEvidenceIndexOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let Some(out) = read_string_option(args, "--out").map(PathBuf::from) else {
        return Err("release-evidence index requires --out <file>".into());
    };
    let artifact_dir = read_string_option(args, "--artifact-dir")
        .or_else(|| read_string_option(args, "--work-dir"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("target/release-evidence-index"));
    Ok(ReleaseEvidenceIndexOptions {
        root,
        out,
        artifact_dir,
    })
}

fn release_evidence_index_receipt(options: &ReleaseEvidenceIndexOptions) -> Value {
    let root = options.root.clone();
    let mut issues = Vec::new();
    let git_state = git_head_state(&root, &mut issues);
    if !issues.is_empty() {
        return release_evidence_index_final_receipt(
            &root,
            &options.out,
            &options.artifact_dir,
            issues,
            git_state,
            Vec::new(),
            Vec::new(),
        );
    }

    let artifact_root =
        match CapabilityRoot::open_or_create(CapabilityRootKind::Release, &options.artifact_dir) {
            Ok(capability) => capability,
            Err(_) => {
                issues.push(release_evidence_issue(
                    "release_evidence.artifact_dir_create_failed",
                    "artifact_dir",
                    "writable release evidence artifact directory",
                    "failed",
                ));
                return release_evidence_index_final_receipt(
                    &root,
                    &options.out,
                    &options.artifact_dir,
                    issues,
                    git_state,
                    Vec::new(),
                    Vec::new(),
                );
            }
        };

    let export_dir = options.artifact_dir.join("exported");
    let imported_dir = options.artifact_dir.join("imported");
    let republished_dir = options.artifact_dir.join("republished");
    let index_path = options.artifact_dir.join("release-evidence-index.json");
    let verify_path = options.artifact_dir.join("evidence-verify-receipt.json");
    let index_verify_path = options
        .artifact_dir
        .join("evidence-index-verification-receipt.json");

    let export = evidence_export_receipt(&EvidenceExportOptions {
        root: root.clone(),
        out: Some(export_dir.clone()),
        agent_receipt: None,
        runtime_chain: None,
        witness_checkpoint: None,
    });
    collect_invalid_receipt("evidence_export", &export, &mut issues);

    let verify = evidence_verify_receipt(&EvidenceVerifyOptions {
        root: root.clone(),
        policy: default_policy_path().into(),
        bundle: Some(export_dir.clone()),
        stack_provenance: None,
        stack_provenance_graph: None,
    });
    write_json_artifact(
        &artifact_root,
        &verify_path,
        &verify,
        &mut issues,
        "evidence_verify",
    );
    collect_invalid_receipt("evidence_verify", &verify, &mut issues);

    let import = evidence_import_receipt(&EvidenceImportOptions {
        root: root.clone(),
        bundle: Some(export_dir.clone()),
        into: Some(imported_dir.clone()),
    });
    collect_invalid_receipt("evidence_import", &import, &mut issues);

    let republish = evidence_republish_receipt(&EvidenceRepublishOptions {
        root: root.clone(),
        imported: Some(imported_dir.clone()),
        out: Some(republished_dir.clone()),
    });
    collect_invalid_receipt("evidence_republish", &republish, &mut issues);

    let index = evidence_index_receipt(&EvidenceIndexOptions {
        root: root.clone(),
        bundle: Some(republished_dir.clone()),
        out: Some(index_path.clone()),
    });
    collect_invalid_receipt("evidence_index", &index, &mut issues);

    let index_verify = evidence_index_verify_receipt(&EvidenceIndexVerifyOptions {
        root: root.clone(),
        index: Some(index_path.clone()),
    });
    write_json_artifact(
        &artifact_root,
        &index_verify_path,
        &index_verify,
        &mut issues,
        "evidence_index_verify",
    );
    collect_invalid_receipt("evidence_index_verify", &index_verify, &mut issues);

    let receipt_table = release_evidence_receipt_table(
        &root,
        &artifact_root,
        &export_dir,
        &[
            ("evidence_export", &export),
            ("evidence_verify", &verify),
            ("evidence_import", &import),
            ("evidence_republish", &republish),
            ("evidence_index", &index),
            ("evidence_index_verify", &index_verify),
        ],
    );
    let artifacts = release_evidence_artifact_table(&export);
    let markdown = render_release_evidence_index(&git_state, &receipt_table, &artifacts);
    if issues.is_empty() {
        write_markdown_artifact(&options.out, &markdown, &mut issues);
    }

    release_evidence_index_final_receipt(
        &root,
        &options.out,
        &options.artifact_dir,
        issues,
        git_state,
        receipt_table,
        artifacts,
    )
}
fn release_evidence_receipt_table(
    root: &Path,
    artifact_root: &CapabilityRoot,
    export_dir: &Path,
    receipts: &[(&str, &Value)],
) -> Vec<Value> {
    let mut table = Vec::with_capacity(receipts.len() + 1);
    let release_path = export_dir.join("release-readiness.json");
    if let Ok(bytes) = artifact_root.read_bounded_path(&release_path, AGENT_RECEIPT_MAX_BYTES) {
        if let Ok(release) = serde_json::from_slice(&bytes) {
            table.push(receipt_row(root, "release_readiness", &release));
        }
    }
    for (role, receipt) in receipts {
        table.push(receipt_row(root, role, receipt));
    }
    table.sort_by_key(canonical_json_hash);
    table
}

fn receipt_row(root: &Path, role: &str, receipt: &Value) -> Value {
    json!({
        "role": role,
        "command": receipt.get("command").and_then(Value::as_str).unwrap_or("<missing>"),
        "verdict": receipt.get("verdict").and_then(Value::as_str).unwrap_or("<missing>"),
        "receipt_hash": receipt_stated_or_computed_hash(root, receipt),
    })
}

fn release_evidence_artifact_table(export_receipt: &Value) -> Vec<Value> {
    let mut artifacts = export_receipt
        .get("artifacts")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    artifacts.sort_by_key(canonical_json_hash);
    artifacts
}

pub(super) fn render_release_evidence_index(
    git: &Value,
    receipts: &[Value],
    artifacts: &[Value],
) -> String {
    let mut out = String::new();
    out.push_str("# Current-HEAD Release Evidence Index\n\n");
    out.push_str("## Payload\n\n");
    out.push_str(&format!(
        "- Payload commit: `{}`\n- Branch: `{}`\n- Worktree clean: `{}`\n- Upstream: `{}`\n- Ahead/behind: `{}`\n\n",
        git["payload_commit"].as_str().unwrap_or(""),
        git["branch"].as_str().unwrap_or(""),
        git["clean"].as_bool().unwrap_or(false),
        git["upstream"].as_str().unwrap_or(""),
        git["ahead_behind"].as_str().unwrap_or(""),
    ));
    out.push_str("## Commands\n\n");
    for command in [
        "release-readiness --root <root>",
        "evidence export --root <root> --out <artifact-dir>/exported",
        "evidence verify --root <root> --bundle <artifact-dir>/exported",
        "evidence import --root <root> --bundle <artifact-dir>/exported --into <artifact-dir>/imported",
        "evidence republish --root <root> --imported <artifact-dir>/imported --out <artifact-dir>/republished",
        "evidence index --root <root> --bundle <artifact-dir>/republished --out <artifact-dir>/release-evidence-index.json",
        "evidence index verify --root <root> --index <artifact-dir>/release-evidence-index.json",
    ] {
        out.push_str(&format!("- `{command}`\n"));
    }
    out.push_str("\n## Receipt Hashes\n\n| Role | Verdict | Receipt hash |\n| --- | --- | --- |\n");
    for receipt in receipts {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            receipt["role"].as_str().unwrap_or(""),
            receipt["verdict"].as_str().unwrap_or(""),
            receipt["receipt_hash"].as_str().unwrap_or(""),
        ));
    }
    out.push_str("\n## Bundle Artifacts\n\n| Role | Path | BLAKE3 |\n| --- | --- | --- |\n");
    for artifact in artifacts {
        out.push_str(&format!(
            "| `{}` | `{}` | `{}` |\n",
            artifact["role"].as_str().unwrap_or(""),
            artifact["path"].as_str().unwrap_or(""),
            artifact["hash"].as_str().unwrap_or(""),
        ));
    }
    out.push_str("\n## Non-claims\n\n");
    out.push_str("- Local trusted evidence only; no remote mirror trust claim.\n");
    out.push_str(
        "- No credential, transport, or public unattended production-readiness validation.\n",
    );
    out.push_str("- No tag creation or tag movement.\n");
    out.push_str("- The payload commit is the evaluated clean HEAD, not a later documentation commit carrying this index.\n");
    out
}

fn release_evidence_index_final_receipt(
    root: &Path,
    out: &Path,
    artifact_dir: &Path,
    mut issues: Vec<Value>,
    git_state: Value,
    receipt_hashes: Vec<Value>,
    artifacts: Vec<Value>,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        root,
        json!({
            "command": "release-evidence index",
            "root": path_string(root),
            "out": path_string(out),
            "artifact_dir": path_string(artifact_dir),
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "git": git_state,
            "receipt_hashes": receipt_hashes,
            "artifacts": artifacts,
            "issues": issues,
        }),
    )
}
