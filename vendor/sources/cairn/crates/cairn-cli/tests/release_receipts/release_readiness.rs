use super::support::*;
use serde_json::{json, Value};
use std::{error::Error, fs, path::PathBuf};

#[test]
fn release_readiness_lists_replay_oracle_preflight() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.release_readiness]
    // r[verify cairn.supersede.release_readiness.determinism_replay]
    // r[verify cairn.supersede.determinism_coverage_release_evidence.release]
    // r[verify cairn.supersede.determinism_coverage_release_evidence.gates]
    let value = run_json(&["release-readiness", "--root", "."])?;
    assert_eq!(value["determinism_replay"]["required"], true);
    assert_eq!(value["coverage_audit_valid"], true);
    assert_hash_field(&value, "coverage_audit_receipt_hash")?;
    assert_eq!(value["determinism_replay"]["included"], false);
    assert_eq!(
        value["determinism_replay"]["command"],
        "determinism replay --root <root>"
    );
    Ok(())
}

#[test]
fn release_readiness_bundle_emits_operator_evidence() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.traceability_ownership.validation.gates]
    // r[verify cairn.supersede.release_readiness_bundle.schema]
    // r[verify cairn.supersede.release_readiness_bundle.local]
    // r[verify cairn.supersede.release_readiness_bundle.evidence]
    // r[verify cairn.supersede.release_readiness_bundle.docs]
    // r[verify cairn.supersede.release_readiness_bundle.verification]
    // r[verify cairn.supersede.determinism_entropy_inventory.verification]
    // r[verify cairn.supersede.determinism_entropy_inventory.archive]
    // r[verify cairn.supersede.replay_environment_attestation.verification]
    // r[verify cairn.supersede.replay_environment_attestation.archive]
    // r[verify cairn.supersede.replay_receipt_verification.verification]
    // r[verify cairn.supersede.replay_receipt_verification.archive]
    // r[verify cairn.verification_role.release_readiness]
    // r[verify cairn.verification_role.release_readiness.boundary]
    let receipt = run_json(&["release-readiness", "--root", "."])?;
    assert_object_keys(
        &receipt,
        &[
            "command",
            "root",
            "valid",
            "verdict",
            "onix_release_ready",
            "checks",
            "cairn_validation_hash",
            "traceability_coverage_hash",
            "mcp_smoke_receipt_hash",
            "coverage_audit_valid",
            "coverage_audit_receipt_hash",
            "cross_repo_dependencies",
            "cross_repo_evidence_issues",
            "release_evidence_index_check_hash",
            "preserves_corpus_freshness_hash",
            "semantic_review_ledger_hash",
            "stack_provenance_hash",
            "stack_provenance",
            "function_address_hash",
            "function_address",
            "stack_provenance_authentication",
            "function_address_authentication",
            "authenticated_release_gate",
            "workspace_contract_matrix_hash",
            "workspace_contract_matrix",
            "repository_gate_summary",
            "determinism_replay",
            "mcp",
            "preserves_content_hash",
            "verification_role_boundary",
            "receipt_hash",
        ],
    )?;
    assert_eq!(receipt["command"], "release-readiness");
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "pass");
    assert_eq!(receipt["onix_release_ready"], false);
    assert_hash_field(&receipt, "receipt_hash")?;
    assert_hash_field(&receipt, "cairn_validation_hash")?;
    assert_hash_field(&receipt, "traceability_coverage_hash")?;
    assert_hash_field(&receipt, "mcp_smoke_receipt_hash")?;
    assert_hash_field(&receipt, "coverage_audit_receipt_hash")?;
    assert_hash_field(&receipt, "release_evidence_index_check_hash")?;
    assert_hash_field(&receipt, "preserves_corpus_freshness_hash")?;
    assert_hash_field(&receipt, "semantic_review_ledger_hash")?;
    assert_hash_field(&receipt, "stack_provenance_hash")?;
    assert_hash_field(&receipt, "function_address_hash")?;
    assert_hash_field(&receipt, "workspace_contract_matrix_hash")?;
    assert_eq!(receipt["coverage_audit_valid"], true);
    let check_names = receipt["checks"]
        .as_array()
        .ok_or("checks array")?
        .iter()
        .map(|check| check["name"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert!(check_names.contains(&"cairn_validate"));
    assert!(check_names.contains(&"traceability_coverage"));
    assert!(check_names.contains(&"mcp_agent_smoke"));
    assert!(check_names.contains(&"determinism_coverage_audit"));
    assert!(check_names.contains(&"release_evidence_index_freshness"));
    assert!(check_names.contains(&"preserves_corpus_freshness"));
    assert!(check_names.contains(&"semantic_review_ledger"));
    assert!(check_names.contains(&"stack_provenance"));
    assert!(check_names.contains(&"function_address"));
    assert!(check_names.contains(&"stack-provenance_authentication"));
    assert!(check_names.contains(&"function-address_authentication"));
    assert!(check_names.contains(&"authenticated_evidence_release_profile"));
    assert!(check_names.contains(&"workspace_contract_matrix"));
    assert!(check_names.contains(&"repository_gate_policy_listed"));
    assert_eq!(receipt["mcp"]["read_only"], true);
    assert_eq!(receipt["mcp"]["destructive"], false);
    assert_eq!(receipt["stack_provenance"]["mode"], "optional");
    assert_eq!(receipt["stack_provenance"]["disposition"], "absent");
    assert_eq!(receipt["stack_provenance"]["passed"], true);
    assert_eq!(receipt["stack_provenance"]["skipped"], true);
    assert_eq!(receipt["function_address"]["mode"], "optional");
    assert_eq!(receipt["function_address"]["disposition"], "absent");
    assert_eq!(receipt["function_address"]["passed"], true);
    assert_eq!(receipt["function_address"]["skipped"], true);
    assert!(receipt["function_address"]["boundary"]
        .as_str()
        .ok_or("function-address boundary text")?
        .contains("did not prove Rust semantic correctness"));
    assert_eq!(receipt["workspace_contract_matrix"]["mode"], "disabled");
    assert_eq!(
        receipt["workspace_contract_matrix"]["disposition"],
        "disabled"
    );
    assert_eq!(receipt["workspace_contract_matrix"]["passed"], true);
    assert_eq!(receipt["workspace_contract_matrix"]["skipped"], true);
    assert!(receipt["verification_role_boundary"]
        .as_str()
        .ok_or("verification_role_boundary text")?
        .contains("counts only property artifacts as verified"));
    Ok(())
}

#[test]
fn release_readiness_bundle_is_reproducible() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.release_readiness_bundle.schema]
    // r[verify cairn.supersede.release_readiness_bundle.evidence]
    // r[verify cairn.supersede.release_readiness_bundle.verification]
    let receipt = assert_reproducible_receipt(
        &["release-readiness", "--root", "."],
        &[
            "receipt_hash",
            "cairn_validation_hash",
            "traceability_coverage_hash",
            "mcp_smoke_receipt_hash",
            "coverage_audit_receipt_hash",
            "release_evidence_index_check_hash",
            "preserves_corpus_freshness_hash",
            "semantic_review_ledger_hash",
            "stack_provenance_hash",
            "function_address_hash",
        ],
    )?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "pass");
    Ok(())
}

fn stack_policy_path(name: &str, mode: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = temp_project(name)?.join("policy.json");
    let mut policy = policy_json()?;
    policy["stack_provenance_gate"]["mode"] = json!(mode);
    fs::write(&path, serde_json::to_vec_pretty(&policy)?)?;
    Ok(path)
}

fn stack_fixture(name: &str) -> String {
    repo_root()
        .join("fixtures/stack-provenance")
        .join(name)
        .display()
        .to_string()
}

fn stack_issue_codes(receipt: &Value) -> Vec<String> {
    receipt["stack_provenance"]["issues"]
        .as_array()
        .map(|issues| {
            issues
                .iter()
                .filter_map(|issue| issue["code"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn function_address_policy_path(name: &str, mode: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = temp_project(name)?.join("policy.json");
    let mut policy = policy_json()?;
    policy["function_address_gate"]["mode"] = json!(mode);
    fs::write(&path, serde_json::to_vec_pretty(&policy)?)?;
    Ok(path)
}

fn function_address_fixture_dir(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let dir = temp_project(name)?;
    let source_count = 1_u64;
    let dependency_edge_count = 1_u64;
    let function_count = 2_u64;
    let recorded_only_count = 2_u64;
    let boundary = "Cairn consumed Valence, Mantle, and Kamacite function-address evidence for lifecycle/release policy conformance only; it did not prove Rust semantic correctness, compiler correctness, whole-program safety, verifier soundness, or release eligibility";
    fs::write(
        dir.join("valence.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": "valence.function-address-evidence.v1",
            "valid": true,
            "verdict": "PASS",
            "receipt_hash": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "valence_policy_hash": "62c88a0263720dea4c37a48ef46921c46e5dd3e4924fff096b84a65d5738a412",
            "claim_scope": "function_address.v1.lifecycle_policy_conformance",
            "kamacite_receipt_hash": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "roles": [
                "function-address-evidence-sidecar",
                "valence-function-address-evidence-profile",
                "kamacite-function-address-receipt"
            ],
            "schemas": [
                "valence.function-address-evidence.v1",
                "function-address-evidence-v1",
                "kamacite.function-address-receipt.v1"
            ],
            "verification_role_summary": {
                "property_count": 0,
                "recorded_only_count": recorded_only_count,
                "verified_count": 0
            },
            "non_claim_boundary": boundary
        }))?,
    )?;
    fs::write(
        dir.join("mantle.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": "mantle.function-address-binding.v1",
            "disposition": "present",
            "valid": true,
            "receipt_hash": "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
            "claim_scope": "function_address.v1.lifecycle_policy_conformance",
            "sidecar_digest": "eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
            "valence_receipt_digest": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "source_archive_digest": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "release_binary_digest": "1111111111111111111111111111111111111111111111111111111111111111",
            "non_claim_boundary": boundary
        }))?,
    )?;
    fs::write(
        dir.join("kamacite.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": "kamacite.function-address-receipt.v1",
            "valid": true,
            "receipt_hash": "4444444444444444444444444444444444444444444444444444444444444444",
            "canonical_preserves_receipt_hash": "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "evidence_role": "kamacite-function-address-receipt",
            "claim_scope": "function_address.v1.lifecycle_policy_conformance",
            "projection_authority": "non-authoritative",
            "non_claims": [
                "evidence-chain readiness does not prove evidence truth or release eligibility"
            ]
        }))?,
    )?;
    fs::write(
        dir.join("graph.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": "valence.function-address-graph.v1",
            "function_count": function_count,
            "source_count": source_count,
            "dependency_edge_count": dependency_edge_count,
            "recorded_only_count": recorded_only_count,
            "property_count": 0,
            "failed_link_count": 0,
            "report_hash": "2222222222222222222222222222222222222222222222222222222222222222"
        }))?,
    )?;
    Ok(dir)
}

fn function_address_issue_codes(receipt: &Value) -> Vec<String> {
    receipt["function_address"]["issues"]
        .as_array()
        .map(|issues| {
            issues
                .iter()
                .filter_map(|issue| issue["code"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn prefix_function_address_digest(path: PathBuf, field: &str) -> Result<(), Box<dyn Error>> {
    let mut value: Value = serde_json::from_slice(&fs::read(&path)?)?;
    let digest = value[field]
        .as_str()
        .ok_or("function-address digest string")?
        .to_string();
    value[field] = json!(format!("b3:{digest}"));
    fs::write(path, serde_json::to_vec_pretty(&value)?)?;
    Ok(())
}

#[test]
fn onix_release_profile_fails_without_authenticated_evidence() -> Result<(), Box<dyn Error>> {
    let path = temp_project("onix-release-authentication-required")?.join("policy.json");
    let mut policy = policy_json()?;
    policy["authenticated_evidence_policy"]["release_profile"] = json!("onix-release");
    policy["stack_provenance_gate"]["mode"] = json!("required");
    policy["function_address_gate"]["mode"] = json!("required");
    fs::write(&path, serde_json::to_vec_pretty(&policy)?)?;
    let receipt = run_json_strings(&[
        "release-readiness".into(),
        "--root".into(),
        ".".into(),
        "--policy".into(),
        path.display().to_string(),
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["authenticated_release_gate"]["required"], true);
    assert_eq!(receipt["authenticated_release_gate"]["passed"], false);
    assert_eq!(receipt["onix_release_ready"], false);
    let issue_codes = receipt["authenticated_release_gate"]["issues"]
        .as_array()
        .ok_or("authenticated release issues")?
        .iter()
        .filter_map(|issue| issue["code"].as_str())
        .collect::<Vec<_>>();
    assert!(issue_codes.contains(&"authenticated_inputs.release_gate_missing"));
    Ok(())
}

#[test]
fn release_readiness_accepts_required_stack_provenance_fixture() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.stack_provenance_gates.policy]
    // r[verify cairn.stack_provenance_gates.receipt_validation.complete]
    // r[verify cairn.stack_provenance_gates.graph_summary.included]
    let policy = stack_policy_path("stack-provenance-required", "required")?;
    let receipt = run_json_strings(&[
        "release-readiness".into(),
        "--root".into(),
        ".".into(),
        "--policy".into(),
        policy.display().to_string(),
        "--stack-provenance".into(),
        stack_fixture("positive/required-complete.json"),
        "--stack-provenance-graph".into(),
        stack_fixture("positive/graph-summary.json"),
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["stack_provenance"]["passed"], true);
    assert_eq!(receipt["stack_provenance"]["disposition"], "present");
    assert_hash_field(&receipt, "stack_provenance_hash")?;
    assert_hash_field(&receipt["stack_provenance"], "valence_receipt_hash")?;
    assert_hash_field(&receipt["stack_provenance"]["graph_summary"], "report_hash")?;
    Ok(())
}

#[test]
fn release_readiness_fails_required_missing_stack_provenance() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.stack_provenance_gates.policy.required_missing]
    let policy = stack_policy_path("stack-provenance-required-missing", "required")?;
    let receipt = run_json_strings(&[
        "release-readiness".into(),
        "--root".into(),
        ".".into(),
        "--policy".into(),
        policy.display().to_string(),
    ])?;
    assert_eq!(receipt["valid"], false);
    assert!(stack_issue_codes(&receipt)
        .contains(&"stack_provenance.valence_receipt_missing".to_string()));
    Ok(())
}

#[test]
fn release_readiness_accepts_required_function_address_fixture() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.stack_provenance_gates.function_address_input_schema.final_validation]
    // r[verify cairn.function_address_gates.receipt_validation.complete]
    // r[verify cairn.function_address_gates.graph_summary.included]
    // r[verify cairn.function_address_gates.docs]
    // r[verify cairn.function_address_gates.docs.boundary]
    // r[verify cairn.function_address_gates.final_validation]
    // r[verify cairn.function_address_gates.final_validation.reviewable]
    let policy = function_address_policy_path("function-address-required", "required")?;
    let fixture = function_address_fixture_dir("function-address-fixtures")?;
    let receipt = run_json_strings(&[
        "release-readiness".into(),
        "--root".into(),
        ".".into(),
        "--policy".into(),
        policy.display().to_string(),
        "--function-address-valence".into(),
        fixture.join("valence.json").display().to_string(),
        "--function-address-mantle".into(),
        fixture.join("mantle.json").display().to_string(),
        "--function-address-kamacite".into(),
        fixture.join("kamacite.json").display().to_string(),
        "--function-address-graph".into(),
        fixture.join("graph.json").display().to_string(),
    ])?;
    assert_eq!(
        receipt["valid"], true,
        "{:?}",
        receipt["function_address"]["issues"]
    );
    assert_eq!(
        receipt["function_address"]["passed"], true,
        "{:?}",
        receipt["function_address"]["issues"]
    );
    assert_eq!(
        receipt["function_address"]["disposition"], "present",
        "{receipt:#}"
    );
    assert_hash_field(&receipt, "function_address_hash")?;
    assert_hash_field(&receipt["function_address"], "valence_receipt_hash")?;
    assert_hash_field(&receipt["function_address"], "mantle_binding_hash")?;
    assert_hash_field(&receipt["function_address"], "kamacite_receipt_hash")?;
    assert_hash_field(
        &receipt["function_address"],
        "canonical_preserves_receipt_hash",
    )?;
    assert_hash_field(&receipt["function_address"], "evidence_chain_hash")?;
    assert_eq!(
        receipt["function_address"]["json_projection_authority"],
        "non-authoritative"
    );
    assert_hash_field(&receipt["function_address"]["graph_summary"], "report_hash")?;
    assert!(receipt["function_address"]["boundary"]
        .as_str()
        .ok_or("function-address boundary text")?
        .contains("did not prove Rust semantic correctness"));
    Ok(())
}

#[test]
fn release_readiness_accepts_function_address_b3_digest_spellings() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.stack_provenance_gates.function_address_digest_normalization.shell]
    // r[verify cairn.stack_provenance_gates.function_address_digest_normalization.positive]
    // r[verify cairn.stack_provenance_gates.function_address_digest_normalization.validation]
    let policy = function_address_policy_path("function-address-b3-required", "required")?;
    let fixture = function_address_fixture_dir("function-address-b3-fixtures")?;
    prefix_function_address_digest(fixture.join("valence.json"), "receipt_hash")?;
    prefix_function_address_digest(fixture.join("mantle.json"), "valence_receipt_digest")?;
    prefix_function_address_digest(fixture.join("mantle.json"), "sidecar_digest")?;
    prefix_function_address_digest(fixture.join("kamacite.json"), "receipt_hash")?;
    prefix_function_address_digest(
        fixture.join("kamacite.json"),
        "canonical_preserves_receipt_hash",
    )?;
    prefix_function_address_digest(fixture.join("graph.json"), "report_hash")?;

    let receipt = run_json_strings(&[
        "release-readiness".into(),
        "--root".into(),
        ".".into(),
        "--policy".into(),
        policy.display().to_string(),
        "--function-address-valence".into(),
        fixture.join("valence.json").display().to_string(),
        "--function-address-mantle".into(),
        fixture.join("mantle.json").display().to_string(),
        "--function-address-kamacite".into(),
        fixture.join("kamacite.json").display().to_string(),
        "--function-address-graph".into(),
        fixture.join("graph.json").display().to_string(),
    ])?;

    assert_eq!(
        receipt["valid"], true,
        "{:?}",
        receipt["function_address"]["issues"]
    );
    assert_eq!(
        receipt["function_address"]["passed"], true,
        "{:?}",
        receipt["function_address"]["issues"]
    );
    assert_eq!(
        receipt["function_address"]["valence_receipt_hash"],
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"
    );
    assert_hash_field(&receipt["function_address"], "mantle_binding_hash")?;
    assert_hash_field(&receipt["function_address"]["graph_summary"], "report_hash")?;
    Ok(())
}

#[test]
fn release_readiness_rejects_authoritative_function_address_json_projection(
) -> Result<(), Box<dyn Error>> {
    // r[verify cairn.stack_provenance_gates.function_address_preserves_readiness.negative]
    // r[verify cairn.stack_provenance_gates.function_address_preserves_readiness.json_projection]
    let policy = function_address_policy_path("function-address-projection-negative", "required")?;
    let fixture = function_address_fixture_dir("function-address-projection-negative-fixtures")?;
    let kamacite_path = fixture.join("kamacite.json");
    let mut kamacite: Value = serde_json::from_slice(&fs::read(&kamacite_path)?)?;
    kamacite["projection_authority"] = json!("authoritative");
    fs::write(&kamacite_path, serde_json::to_vec_pretty(&kamacite)?)?;

    let receipt = run_json_strings(&[
        "release-readiness".into(),
        "--root".into(),
        ".".into(),
        "--policy".into(),
        policy.display().to_string(),
        "--function-address-valence".into(),
        fixture.join("valence.json").display().to_string(),
        "--function-address-mantle".into(),
        fixture.join("mantle.json").display().to_string(),
        "--function-address-kamacite".into(),
        kamacite_path.display().to_string(),
        "--function-address-graph".into(),
        fixture.join("graph.json").display().to_string(),
    ])?;
    assert_eq!(receipt["valid"], false);
    assert!(function_address_issue_codes(&receipt)
        .contains(&"function_address_preserves.json_projection_authoritative".to_string()));
    Ok(())
}

#[test]
fn release_readiness_clears_function_address_chain_hash_after_shell_failure(
) -> Result<(), Box<dyn Error>> {
    // r[verify cairn.stack_provenance_gates.function_address_preserves_readiness.negative]
    let policy = function_address_policy_path("function-address-shell-negative", "required")?;
    let fixture = function_address_fixture_dir("function-address-shell-negative-fixtures")?;
    let receipt = run_json_strings(&[
        "release-readiness".into(),
        "--root".into(),
        ".".into(),
        "--policy".into(),
        policy.display().to_string(),
        "--function-address-valence".into(),
        fixture.join("valence.json").display().to_string(),
        "--function-address-mantle".into(),
        fixture.join("mantle.json").display().to_string(),
        "--function-address-kamacite".into(),
        fixture.join("kamacite.json").display().to_string(),
        "--function-address-graph".into(),
        fixture.join("missing-graph.json").display().to_string(),
    ])?;
    assert_eq!(receipt["valid"], false);
    assert!(function_address_issue_codes(&receipt)
        .contains(&"function_address.input_unreadable".to_string()));
    assert_eq!(
        receipt["function_address"]["evidence_chain_hash"],
        Value::Null
    );
    Ok(())
}

#[test]
fn release_readiness_fails_required_missing_function_address() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.function_address_gates.policy.required_missing]
    // r[verify cairn.function_address_gates.fixtures.negative.required]
    let policy = function_address_policy_path("function-address-required-missing", "required")?;
    let receipt = run_json_strings(&[
        "release-readiness".into(),
        "--root".into(),
        ".".into(),
        "--policy".into(),
        policy.display().to_string(),
    ])?;
    assert_eq!(receipt["valid"], false);
    assert!(function_address_issue_codes(&receipt)
        .contains(&"function_address.valence_receipt_missing".to_string()));
    Ok(())
}

#[test]
fn release_readiness_reports_preserves_corpus_freshness() -> Result<(), Box<dyn Error>> {
    // r[verify release_evidence.preserves_readiness_summary.output]
    // r[verify release_evidence.preserves_readiness_summary.tests]
    let receipt = run_json(&["release-readiness", "--root", "."])?;
    let preserves = receipt["checks"]
        .as_array()
        .ok_or("checks array")?
        .iter()
        .find(|check| check["name"] == "preserves_corpus_freshness")
        .ok_or("preserves corpus freshness check")?;
    assert_eq!(preserves["passed"], true);
    assert!(preserves["source"]
        .as_str()
        .unwrap_or_default()
        .contains("preserves-corpus update"));
    assert_hash_field(&receipt, "preserves_corpus_freshness_hash")?;
    Ok(())
}

#[test]
fn release_readiness_fails_closed_when_preserves_corpus_is_absent() -> Result<(), Box<dyn Error>> {
    // r[verify release_evidence.preserves_readiness_summary.fail_closed]
    // r[verify release_evidence.preserves_readiness_summary.validation]
    let fixture = temp_fixture(
        "release-readiness-missing-preserves-corpus",
        "fixtures/basic",
    )?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;
    let receipt = run_json(&["release-readiness", "--root", root])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "fail");
    let preserves = receipt["checks"]
        .as_array()
        .ok_or("checks array")?
        .iter()
        .find(|check| check["name"] == "preserves_corpus_freshness")
        .ok_or("preserves corpus freshness check")?;
    assert_eq!(preserves["passed"], false);
    assert_hash_field(&receipt, "preserves_corpus_freshness_hash")?;
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn release_readiness_skips_absent_semantic_review_ledger() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.readiness_gate.release_readiness.skipped_absent]
    // r[verify cairn.semantic_review.readiness_gate.boundary.no_reviewer_execution]
    let receipt = run_json(&["release-readiness", "--root", "."])?;
    let semantic_review = receipt["checks"]
        .as_array()
        .ok_or("checks array")?
        .iter()
        .find(|check| check["name"] == "semantic_review_ledger")
        .ok_or("semantic review ledger check")?;
    assert_eq!(semantic_review["passed"], true);
    assert_eq!(semantic_review["skipped"], true);
    assert!(semantic_review["source"]
        .as_str()
        .unwrap_or_default()
        .contains("--review-ledger"));
    assert_hash_field(&receipt, "semantic_review_ledger_hash")?;
    Ok(())
}

#[test]
fn release_readiness_accepts_supplied_semantic_review_ledger() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.readiness_gate.release_readiness]
    // r[verify cairn.semantic_review.readiness_gate.release_readiness.supplied_valid]
    let ledger = temp_project("release-readiness-valid-semantic-ledger")?.join("ledger.json");
    fs::write(
        &ledger,
        r#"{"command":"review ledger","valid":true,"verdict":"PASS","receipt_hash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"}"#,
    )?;
    let receipt = run_json_strings(&[
        "release-readiness".to_string(),
        "--root".to_string(),
        ".".to_string(),
        "--review-ledger".to_string(),
        ledger.display().to_string(),
    ])?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(
        receipt["semantic_review_ledger_hash"],
        "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    );
    let semantic_review = receipt["checks"]
        .as_array()
        .ok_or("checks array")?
        .iter()
        .find(|check| check["name"] == "semantic_review_ledger")
        .ok_or("semantic review ledger check")?;
    assert_eq!(semantic_review["passed"], true);
    assert_eq!(semantic_review["verdict"], "PASS");
    fs::remove_file(ledger)?;
    Ok(())
}

#[test]
fn release_readiness_fails_supplied_semantic_review_ledger() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.semantic_review.readiness_gate.release_readiness.supplied_invalid]
    let ledger = temp_project("release-readiness-invalid-semantic-ledger")?.join("ledger.json");
    fs::write(
        &ledger,
        r#"{"command":"review ledger","valid":false,"verdict":"FAIL","receipt_hash":"abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789"}"#,
    )?;
    let receipt = run_json(&[
        "release-readiness",
        "--root",
        ".",
        "--review-ledger",
        ledger.to_str().ok_or("utf8 ledger path")?,
    ])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "fail");
    let semantic_review = receipt["checks"]
        .as_array()
        .ok_or("checks array")?
        .iter()
        .find(|check| check["name"] == "semantic_review_ledger")
        .ok_or("semantic review ledger check")?;
    assert_eq!(semantic_review["passed"], false);
    assert_eq!(semantic_review["verdict"], "FAIL");
    fs::remove_file(ledger)?;
    Ok(())
}

#[test]
fn release_readiness_bundle_reports_missing_evidence() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.release_readiness_bundle.failure]
    let fixture = temp_fixture("release-readiness-missing-evidence", "fixtures/basic")?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;
    let receipt = run_json(&["release-readiness", "--root", root])?;
    assert_eq!(receipt["valid"], false);
    assert_eq!(receipt["verdict"], "fail");
    assert_hash_field(&receipt, "receipt_hash")?;
    let failed = receipt["checks"]
        .as_array()
        .ok_or("checks array")?
        .iter()
        .filter(|check| check["passed"] == false)
        .collect::<Vec<_>>();
    let has_traceability_failure = failed
        .iter()
        .any(|check| check["name"] == "traceability_coverage");
    let has_mcp_smoke_failure = failed
        .iter()
        .any(|check| check["name"] == "mcp_agent_smoke");
    let is_missing_evidence_failure = has_traceability_failure || has_mcp_smoke_failure;
    assert!(
        is_missing_evidence_failure,
        "expected missing readiness evidence failure: {receipt}"
    );
    fs::remove_dir_all(fixture)?;
    Ok(())
}
