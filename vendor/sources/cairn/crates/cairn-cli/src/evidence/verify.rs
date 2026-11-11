use super::*;
use crate::support::{CapabilityRoot, CapabilityRootKind};

pub(crate) struct EvidenceVerifyOptions {
    pub(crate) root: PathBuf,
    pub(crate) policy: PathBuf,
    pub(crate) bundle: Option<PathBuf>,
    pub(crate) stack_provenance: Option<PathBuf>,
    pub(crate) stack_provenance_graph: Option<PathBuf>,
}

struct EvidenceVerifyFinalReceiptParts {
    issues: Vec<Value>,
    artifacts: Vec<Value>,
    export_receipt_hash: Value,
    manifest_hash: Value,
    manifest_verification_receipt_hash: Value,
    stack_provenance: Value,
    stack_provenance_hash: Value,
    verification_role_summary: Value,
}

pub(super) fn run_evidence_verify(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.supersede.evidence_bundle_verify_command.command]
    // r[impl cairn.supersede.evidence_bundle_verify_command.artifacts]
    // r[impl cairn.supersede.evidence_bundle_verify_command.failures]
    let options = evidence_verify_options(&args)?;
    let receipt = evidence_verify_receipt(&options);
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

fn evidence_verify_options(args: &[String]) -> Result<EvidenceVerifyOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let policy = read_option(args.to_vec(), "--policy", default_policy_path());
    let bundle = read_string_option(args, "--bundle").map(PathBuf::from);
    let stack_provenance = read_string_option(args, "--stack-provenance").map(PathBuf::from);
    let stack_provenance_graph =
        read_string_option(args, "--stack-provenance-graph").map(PathBuf::from);
    if bundle.is_none() {
        return Err("evidence verify requires --bundle <dir>".into());
    }
    Ok(EvidenceVerifyOptions {
        root,
        policy,
        bundle,
        stack_provenance,
        stack_provenance_graph,
    })
}

pub(crate) fn evidence_verify_receipt(options: &EvidenceVerifyOptions) -> Value {
    let root = options.root.clone();
    let bundle_dir = options.bundle.clone().unwrap_or_else(|| PathBuf::from("."));
    let mut issues = Vec::new();
    let bundle_root = match CapabilityRoot::open(CapabilityRootKind::EvidenceBundle, &bundle_dir) {
        Ok(capability) => capability,
        Err(_) => {
            issues.push(evidence_verify_issue(
                "evidence_verify.bundle_root_unreadable",
                "bundle",
                "readable evidence bundle root",
                "unreadable",
            ));
            return evidence_verify_final_receipt(&root, empty_evidence_verify_parts(issues));
        }
    };
    let export_receipt =
        match read_json_artifact_from_root(&bundle_root, Path::new("evidence-export-receipt.json"))
        {
            Ok(receipt) => receipt,
            Err(_) => {
                issues.push(evidence_verify_issue(
                    "evidence_verify.export_receipt_missing",
                    "evidence-export-receipt.json",
                    "readable export receipt",
                    "missing",
                ));
                return evidence_verify_final_receipt(&root, empty_evidence_verify_parts(issues));
            }
        };

    if export_receipt.get("command").and_then(Value::as_str) != Some("evidence export") {
        issues.push(evidence_verify_issue(
            "evidence_verify.export_receipt_command_invalid",
            "evidence-export-receipt.command",
            "evidence export",
            "invalid",
        ));
    }
    let stated_export_hash = export_receipt
        .get("receipt_hash")
        .and_then(Value::as_str)
        .unwrap_or("");
    let computed_export_hash = receipt_stated_or_computed_hash(&root, &export_receipt);
    if computed_export_hash.as_str() != Some(stated_export_hash) {
        issues.push(evidence_verify_issue(
            "evidence_verify.export_receipt_hash_mismatch",
            "evidence-export-receipt.receipt_hash",
            "self-consistent receipt hash",
            "mismatch",
        ));
    }

    let mut verified_artifacts = Vec::new();
    let mut seen_roles = std::collections::BTreeSet::new();
    if let Some(artifacts) = export_receipt.get("artifacts").and_then(Value::as_array) {
        for artifact in artifacts {
            let role = artifact
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or("<missing>");
            if !seen_roles.insert(role.to_string()) {
                issues.push(evidence_verify_issue(
                    "evidence_verify.role_duplicate",
                    role,
                    "unique artifact role",
                    "duplicate",
                ));
            }
            let relative = artifact.get("path").and_then(Value::as_str).unwrap_or("");
            if safe_bundle_artifact_path(&bundle_dir, relative).is_none() {
                issues.push(evidence_verify_issue(
                    "evidence_verify.artifact_path_unsafe",
                    role,
                    "safe relative artifact path",
                    "unsafe",
                ));
                continue;
            }
            let declared_hash = artifact.get("hash").and_then(Value::as_str).unwrap_or("");
            let required = artifact
                .get("required")
                .and_then(Value::as_bool)
                .unwrap_or(true);
            let actual_hash = match bundle_root.read(Path::new(relative)) {
                Ok(bytes) => blake3_hex(&bytes),
                Err(_) => {
                    if required {
                        issues.push(evidence_verify_issue(
                            "evidence_verify.artifact_missing",
                            role,
                            "declared artifact file",
                            "missing",
                        ));
                    }
                    String::new()
                }
            };
            if !actual_hash.is_empty() && actual_hash != declared_hash {
                issues.push(evidence_verify_issue(
                    "evidence_verify.artifact_hash_mismatch",
                    role,
                    "declared artifact hash",
                    "mismatch",
                ));
            }
            verified_artifacts.push(json!({
                "role": role,
                "path": relative,
                "hash": if actual_hash.is_empty() { Value::Null } else { json!(actual_hash) },
                "required": required,
                "verification_role": artifact.get("verification_role").and_then(Value::as_str).unwrap_or("unknown"),
            }));
        }
    } else {
        issues.push(evidence_verify_issue(
            "evidence_verify.artifacts_missing",
            "artifacts",
            "artifact array",
            "missing",
        ));
    }

    for role in EVIDENCE_EXPORT_REQUIRED_ROLES {
        if !seen_roles.contains(*role) {
            issues.push(evidence_verify_issue(
                "evidence_verify.required_role_missing",
                role,
                "required artifact role",
                "missing",
            ));
        }
    }

    verify_saved_receipt_valid(
        &bundle_root,
        "coverage_audit",
        "coverage-audit.json",
        "evidence_verify.coverage_audit_invalid",
        &mut issues,
    );
    verify_saved_receipt_valid(
        &bundle_root,
        "replay_verification",
        "replay-verification.json",
        "evidence_verify.replay_verification_invalid",
        &mut issues,
    );
    verify_saved_receipt_valid(
        &bundle_root,
        "release_readiness",
        "release-readiness.json",
        "evidence_verify.release_readiness_invalid",
        &mut issues,
    );
    verify_saved_receipt_valid(
        &bundle_root,
        "agent_evidence_bundle_verification",
        "agent-evidence-bundle-verification.json",
        "evidence_verify.bundle_verification_invalid",
        &mut issues,
    );
    verify_saved_receipt_valid(
        &bundle_root,
        "evidence_manifest_verification",
        "evidence-manifest-verification.json",
        "evidence_verify.manifest_verification_invalid",
        &mut issues,
    );

    let rerun_bundle = agent_evidence_bundle_verification_receipt(
        &root,
        &bundle_dir.join("agent-evidence-bundle.json"),
    );
    if rerun_bundle.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_verify_issue(
            "evidence_verify.bundle_rerun_invalid",
            "agent-evidence-bundle.json",
            "valid rerun bundle verification",
            "invalid",
        ));
    }
    let rerun_manifest =
        evidence_manifest_verification_receipt(&root, &bundle_dir.join("evidence-manifest.json"));
    if rerun_manifest.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_verify_issue(
            "evidence_verify.manifest_rerun_invalid",
            "evidence-manifest.json",
            "valid rerun manifest verification",
            "invalid",
        ));
    }

    let stack_provenance = crate::stack_provenance::stack_provenance_receipt_value(
        options.policy.clone(),
        options.stack_provenance.clone(),
        options.stack_provenance_graph.clone(),
    );
    if stack_provenance.get("passed").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_verify_issue(
            "evidence_verify.stack_provenance_invalid",
            "stack_provenance",
            "passing stack provenance contribution",
            "invalid",
        ));
    }
    let stack_provenance_hash = crate::stack_provenance::stack_provenance_hash(&stack_provenance);
    let verification_role_summary = VerificationRoleSummary::from_artifacts(&verified_artifacts);
    let manifest_hash = export_receipt
        .get("manifest_hash")
        .cloned()
        .unwrap_or(Value::Null);
    let manifest_verification_receipt_hash = export_receipt
        .get("manifest_verification_receipt_hash")
        .cloned()
        .unwrap_or(Value::Null);
    evidence_verify_final_receipt(
        &root,
        EvidenceVerifyFinalReceiptParts {
            issues,
            artifacts: verified_artifacts,
            export_receipt_hash: computed_export_hash,
            manifest_hash,
            manifest_verification_receipt_hash,
            stack_provenance,
            stack_provenance_hash: json!(stack_provenance_hash),
            verification_role_summary: verification_role_summary.to_json(),
        },
    )
}

const EVIDENCE_EXPORT_REQUIRED_ROLES: &[&str] = &[
    "agent_evidence_bundle",
    "agent_evidence_bundle_verification",
    "agent_receipt",
    "coverage_audit",
    "evidence_manifest",
    "evidence_manifest_verification",
    "release_readiness",
    "replay_receipt",
    "replay_verification",
];

fn read_json_artifact_from_root(
    bundle_root: &CapabilityRoot,
    relative: &Path,
) -> Result<Value, Box<dyn Error>> {
    let bytes = bundle_root.read(relative)?;
    let byte_count = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if byte_count > AGENT_RECEIPT_MAX_BYTES {
        return Err("JSON artifact exceeds the byte limit".into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}

fn verify_saved_receipt_valid(
    bundle_root: &CapabilityRoot,
    role: &str,
    relative: &str,
    code: &str,
    issues: &mut Vec<Value>,
) {
    match read_json_artifact_from_root(bundle_root, Path::new(relative)) {
        Ok(receipt) => {
            if receipt.get("valid").and_then(Value::as_bool) != Some(true) {
                issues.push(evidence_verify_issue(
                    code,
                    role,
                    "persisted valid receipt",
                    "invalid",
                ));
            }
        }
        Err(_) => issues.push(evidence_verify_issue(
            "evidence_verify.receipt_unreadable",
            role,
            "readable JSON receipt",
            "unreadable",
        )),
    }
}

fn empty_evidence_verify_parts(issues: Vec<Value>) -> EvidenceVerifyFinalReceiptParts {
    EvidenceVerifyFinalReceiptParts {
        issues,
        artifacts: Vec::new(),
        export_receipt_hash: Value::Null,
        manifest_hash: Value::Null,
        manifest_verification_receipt_hash: Value::Null,
        stack_provenance: serde_json::json!({
            "name": "stack_provenance",
            "mode": "unknown",
            "required": false,
            "passed": true,
            "skipped": true,
            "disposition": "not-evaluated",
            "issues": [],
            "valence_receipt_hash": null,
            "claim_scope": null,
            "graph_summary": null,
            "boundary": cairn_core::verified::stack_provenance::STACK_PROVENANCE_BOUNDARY,
        }),
        stack_provenance_hash: Value::Null,
        verification_role_summary: serde_json::json!({
            "property_count": 0,
            "recorded_only_count": 0,
            "boundary_count": 0,
            "unknown_count": 0,
            "total": 0,
            "verified_count": 0,
            "boundary": cairn_core::verified::verification_role::VERIFICATION_ROLE_BOUNDARY,
        }),
    }
}

fn evidence_verify_issue(code: &str, field_path: &str, expected: &str, actual: &str) -> Value {
    json!({
        "code": code,
        "field_path": field_path,
        "expected": expected,
        "actual": actual,
    })
}

fn evidence_verify_final_receipt(root: &Path, mut parts: EvidenceVerifyFinalReceiptParts) -> Value {
    parts.issues.sort_by_key(canonical_json_hash);
    parts.artifacts.sort_by_key(|artifact| {
        artifact
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string()
    });
    let valid = parts.issues.is_empty();
    with_portable_content_hash_receipt(
        root,
        json!({
            "command": "evidence verify",
            "root": path_string(root),
            "bundle": "<bundle>",
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": parts.issues,
            "artifacts": parts.artifacts,
            "export_receipt_hash": parts.export_receipt_hash,
            "manifest_hash": parts.manifest_hash,
            "manifest_verification_receipt_hash": parts.manifest_verification_receipt_hash,
            "stack_provenance": parts.stack_provenance,
            "stack_provenance_hash": parts.stack_provenance_hash,
            "verification_role_summary": parts.verification_role_summary,
        }),
    )
}
