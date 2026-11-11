// r[impl cli_maintainability.main_compat_release_cli.focused_modules]
// r[impl cli_maintainability.main_compat_release_cli.focused_modules.visible_boundaries]
// r[impl cli_maintainability.main_compat_release_cli.focused_modules.narrow_bridges]
// r[impl cli_maintainability.main_compat_release_cli.behavior_preserved]
// r[verify cli_maintainability.main_compat_release_cli.behavior_preserved.focused_checks]
// r[verify cli_maintainability.main_compat_release_cli.behavior_preserved.repository_checks]
// r[impl release_evidence.current_head_evidence_index.focused_modules]
// r[impl release_evidence.current_head_evidence_index.focused_modules.visible_boundaries]
// r[impl release_evidence.current_head_evidence_index.focused_modules.narrow_bridges]
// r[verify release_evidence.current_head_evidence_index.behavior_preserved]
// r[verify release_evidence.current_head_evidence_index.behavior_preserved.focused_checks]
// r[verify release_evidence.current_head_evidence_index.behavior_preserved.repository_checks]
use std::{error::Error, fs, path::PathBuf};

use cairn_core::verified::verification_role::VERIFICATION_ROLE_BOUNDARY;
use cairn_core::{
    authenticated_evidence_policy_from_value, canonical_json_hash,
    evaluate_authenticated_release_gate, evaluate_workspace_contract_matrix_readiness,
    workspace_contract_readiness_issues_json, workspace_contract_readiness_policy_from_value,
    AuthenticatedReleaseGateInput,
};
use serde_json::{json, Value};

use crate::authenticated_inputs::{
    authentication_component, AuthenticationArtifact, AuthenticationComponent,
};
use crate::determinism::determinism_coverage_audit_receipt;
use crate::function_address::{function_address_hash, function_address_receipt_value};
use crate::mcp::mcp_agent_smoke_receipt;
use crate::native::cairn_validation_receipt_with_cross_repo_evidence;
use crate::preserves_corpus::preserves_corpus_freshness_receipt;
use crate::release_evidence::release_evidence_check_index_receipt_for_path;
use crate::stack_provenance::{stack_provenance_hash, stack_provenance_receipt_value};
use crate::support::{
    default_policy_path, load_policy_file, path_string, print_json, read_option,
    read_repeated_path_options, read_string_option, safe_relative_path,
    with_portable_content_hash_receipt,
};
use crate::tracey::{repository_gate_summary, traceability_coverage_receipt};

pub(crate) fn run_release_readiness(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.clone(), "--root", ".");
    let policy_path = read_option(args.clone(), "--policy", default_policy_path());
    let review_ledger = read_string_option(&args, "--review-ledger").map(PathBuf::from);
    let stack_provenance = read_string_option(&args, "--stack-provenance").map(PathBuf::from);
    let stack_provenance_graph =
        read_string_option(&args, "--stack-provenance-graph").map(PathBuf::from);
    let stack_provenance_authentication =
        read_string_option(&args, "--stack-provenance-authentication").map(PathBuf::from);
    let function_address_valence =
        read_string_option(&args, "--function-address-valence").map(PathBuf::from);
    let function_address_mantle =
        read_string_option(&args, "--function-address-mantle").map(PathBuf::from);
    let function_address_kamacite =
        read_string_option(&args, "--function-address-kamacite").map(PathBuf::from);
    let function_address_graph =
        read_string_option(&args, "--function-address-graph").map(PathBuf::from);
    let function_address_authentication =
        read_string_option(&args, "--function-address-authentication").map(PathBuf::from);
    let workspace_contract_matrix_receipt =
        read_string_option(&args, "--workspace-contract-matrix-receipt").map(PathBuf::from);
    let cross_repo_evidence = read_repeated_path_options(&args, "--cross-repo-evidence");
    print_json(release_readiness_receipt_with_inputs(
        ReleaseReadinessInputs {
            root,
            policy_path,
            review_ledger,
            stack_provenance,
            stack_provenance_graph,
            stack_provenance_authentication,
            function_address_valence,
            function_address_mantle,
            function_address_kamacite,
            function_address_graph,
            function_address_authentication,
            workspace_contract_matrix_receipt,
            cross_repo_evidence,
        },
    ))
}

pub(crate) fn release_readiness_receipt(root: PathBuf, policy_path: PathBuf) -> Value {
    release_readiness_receipt_with_review_ledger(root, policy_path, None)
}

// r[verify cairn.determinism.preserves_content_hash_binding_readiness.validation]
pub(crate) fn release_readiness_receipt_with_review_ledger(
    root: PathBuf,
    policy_path: PathBuf,
    review_ledger: Option<PathBuf>,
) -> Value {
    release_readiness_receipt_with_inputs(ReleaseReadinessInputs {
        root,
        policy_path,
        review_ledger,
        stack_provenance: None,
        stack_provenance_graph: None,
        stack_provenance_authentication: None,
        function_address_valence: None,
        function_address_mantle: None,
        function_address_kamacite: None,
        function_address_graph: None,
        function_address_authentication: None,
        workspace_contract_matrix_receipt: None,
        cross_repo_evidence: Vec::new(),
    })
}

struct ReleaseReadinessInputs {
    root: PathBuf,
    policy_path: PathBuf,
    review_ledger: Option<PathBuf>,
    stack_provenance: Option<PathBuf>,
    stack_provenance_graph: Option<PathBuf>,
    stack_provenance_authentication: Option<PathBuf>,
    function_address_valence: Option<PathBuf>,
    function_address_mantle: Option<PathBuf>,
    function_address_kamacite: Option<PathBuf>,
    function_address_graph: Option<PathBuf>,
    function_address_authentication: Option<PathBuf>,
    workspace_contract_matrix_receipt: Option<PathBuf>,
    cross_repo_evidence: Vec<PathBuf>,
}

fn release_readiness_receipt_with_inputs(inputs: ReleaseReadinessInputs) -> Value {
    let ReleaseReadinessInputs {
        root,
        policy_path,
        review_ledger,
        stack_provenance,
        stack_provenance_graph,
        stack_provenance_authentication,
        function_address_valence,
        function_address_mantle,
        function_address_kamacite,
        function_address_graph,
        function_address_authentication,
        workspace_contract_matrix_receipt,
        cross_repo_evidence,
    } = inputs;
    // r[impl cairn.supersede.release_readiness_bundle]
    // r[impl cairn.supersede.release_readiness_bundle.schema]
    // r[impl cairn.supersede.release_readiness_bundle.local]
    // r[impl cairn.supersede.release_readiness_bundle.evidence]
    // r[impl cairn.supersede.release_readiness_bundle.failure]
    let root_text = path_string(root.clone());
    let validation = readiness_component("cairn_validate", || {
        cairn_validation_receipt_with_cross_repo_evidence(
            root.clone(),
            policy_path.clone(),
            &cross_repo_evidence,
        )
    });
    let traceability = readiness_component("traceability_coverage", || {
        traceability_coverage_receipt(root.clone(), policy_path.clone())
    });
    let mcp_smoke =
        readiness_component("mcp_agent_smoke", || mcp_agent_smoke_receipt(root.clone()));
    let coverage_audit = readiness_component("determinism_coverage_audit", || {
        let policy = load_policy_file(policy_path.clone())?;
        Ok(determinism_coverage_audit_receipt(&root, &policy))
    });
    let release_evidence_index = conventional_release_evidence_index_component(root.clone());
    let preserves_corpus = preserves_corpus_component(root.clone());
    let semantic_review_ledger = semantic_review_ledger_component(root.clone(), review_ledger);
    let policy = load_policy_file(policy_path.clone()).ok();
    let stack_provenance_component_value = stack_provenance_component(
        policy_path.clone(),
        stack_provenance.clone(),
        stack_provenance_graph.clone(),
    );
    let function_address_component_value = function_address_component(
        policy_path.clone(),
        function_address_valence.clone(),
        function_address_mantle.clone(),
        function_address_kamacite.clone(),
        function_address_graph.clone(),
    );
    let stack_provenance_authentication = authentication_component(
        &root,
        policy.as_ref().map(|value| &value.raw),
        "stack-provenance",
        stack_provenance_authentication,
        vec![
            AuthenticationArtifact {
                role: "receipt",
                path: stack_provenance,
            },
            AuthenticationArtifact {
                role: "graph",
                path: stack_provenance_graph,
            },
        ],
        "receipt",
    );
    let function_address_authentication = authentication_component(
        &root,
        policy.as_ref().map(|value| &value.raw),
        "function-address",
        function_address_authentication,
        vec![
            AuthenticationArtifact {
                role: "valence_receipt",
                path: function_address_valence,
            },
            AuthenticationArtifact {
                role: "mantle_binding",
                path: function_address_mantle,
            },
            AuthenticationArtifact {
                role: "kamacite_receipt",
                path: function_address_kamacite,
            },
            AuthenticationArtifact {
                role: "graph",
                path: function_address_graph,
            },
        ],
        "valence_receipt",
    );
    let authenticated_release = authenticated_release_component(
        policy.as_ref().map(|value| &value.raw),
        stack_provenance_component_value.value["mode"]
            .as_str()
            .unwrap_or(""),
        function_address_component_value.value["mode"]
            .as_str()
            .unwrap_or(""),
        &stack_provenance_authentication,
        &function_address_authentication,
    );
    let workspace_contract_matrix = workspace_contract_matrix_component(
        root.clone(),
        policy.as_ref().map(|value| &value.raw),
        workspace_contract_matrix_receipt,
    );
    let repository_gate_summary = repository_gate_summary();
    let replay_preflight = json!({
        "command": "determinism replay --root <root>",
        "included": false,
        "required": true,
        "reason": "workspace replay may copy fixtures and is an explicit operator/CI preflight",
    });
    let coverage_audit_valid = coverage_audit.check["passed"].clone();
    let cross_repo_dependencies = validation.value["cross_repo_dependencies"].clone();
    let cross_repo_evidence_issues = validation.value["cross_repo_evidence_issues"].clone();
    let mut checks = vec![
        validation.check,
        traceability.check,
        mcp_smoke.check,
        coverage_audit.check,
        release_evidence_index.check,
        preserves_corpus.check,
        semantic_review_ledger.check,
        stack_provenance_component_value.check,
        function_address_component_value.check,
        stack_provenance_authentication.check,
        function_address_authentication.check,
        authenticated_release.check,
        workspace_contract_matrix.check,
    ];
    checks.push(json!({
        "name": "repository_gate_policy_listed",
        "passed": true,
        "source": "maintained local policy",
        "required_external_commands": repository_gate_summary["required_commands"].clone(),
        "evidence_hash": canonical_json_hash(&repository_gate_summary)
    }));
    let is_valid = checks
        .iter()
        .all(|check| check["passed"].as_bool().unwrap_or(false));
    let onix_release_ready = is_valid
        && authenticated_release.value["required"].as_bool() == Some(true)
        && authenticated_release.value["passed"].as_bool() == Some(true);
    with_portable_content_hash_receipt(
        &root,
        json!({
            "command": "release-readiness",
            "root": root_text,
            "valid": is_valid,
            "verdict": if is_valid { "pass" } else { "fail" },
            "onix_release_ready": onix_release_ready,
            "checks": checks,
            "cairn_validation_hash": validation.receipt_hash,
            "cross_repo_dependencies": cross_repo_dependencies,
            "cross_repo_evidence_issues": cross_repo_evidence_issues,
            "traceability_coverage_hash": traceability.receipt_hash,
            "mcp_smoke_receipt_hash": mcp_smoke.receipt_hash,
            "coverage_audit_valid": coverage_audit_valid,
            "coverage_audit_receipt_hash": coverage_audit.receipt_hash,
            "release_evidence_index_check_hash": release_evidence_index.receipt_hash,
            "preserves_corpus_freshness_hash": preserves_corpus.receipt_hash,
            "semantic_review_ledger_hash": semantic_review_ledger.receipt_hash,
            "stack_provenance_hash": stack_provenance_component_value.receipt_hash,
            "stack_provenance": stack_provenance_component_value.value,
            "function_address_hash": function_address_component_value.receipt_hash,
            "function_address": function_address_component_value.value,
            "stack_provenance_authentication": stack_provenance_authentication.value,
            "function_address_authentication": function_address_authentication.value,
            "authenticated_release_gate": authenticated_release.value,
            "workspace_contract_matrix_hash": workspace_contract_matrix.receipt_hash,
            "workspace_contract_matrix": workspace_contract_matrix.value,
            "repository_gate_summary": repository_gate_summary,
            "determinism_replay": replay_preflight,
            "mcp": {"read_only": true, "destructive": false},
            "verification_role_boundary": VERIFICATION_ROLE_BOUNDARY,
        }),
    )
}

fn authenticated_release_component(
    policy_value: Option<&Value>,
    stack_provenance_mode: &str,
    function_address_mode: &str,
    stack_provenance: &AuthenticationComponent,
    function_address: &AuthenticationComponent,
) -> AuthenticatedReleaseComponent {
    let result = policy_value
        .ok_or_else(|| "policy could not be loaded".to_string())
        .and_then(authenticated_evidence_policy_from_value)
        .map(|policy| {
            evaluate_authenticated_release_gate(
                &policy,
                &AuthenticatedReleaseGateInput {
                    stack_provenance_mode,
                    function_address_mode,
                    stack_provenance: stack_provenance.report.as_ref(),
                    function_address: function_address.report.as_ref(),
                },
            )
        });
    match result {
        Ok(report) => {
            let value = serde_json::to_value(&report).unwrap_or_else(|error| {
                json!({
                    "passed": false,
                    "issue": error.to_string(),
                })
            });
            AuthenticatedReleaseComponent {
                check: json!({
                    "name": "authenticated_evidence_release_profile",
                    "passed": report.passed,
                    "required": report.required,
                    "source": "cairn authenticated evidence policy",
                }),
                value,
            }
        }
        Err(error) => AuthenticatedReleaseComponent {
            check: json!({
                "name": "authenticated_evidence_release_profile",
                "passed": false,
                "source": "cairn authenticated evidence policy",
                "issue": error,
            }),
            value: json!({
                "required": false,
                "passed": false,
                "issue": error,
            }),
        },
    }
}

fn workspace_contract_matrix_component(
    root: PathBuf,
    policy: Option<&Value>,
    receipt_path: Option<PathBuf>,
) -> WorkspaceContractMatrixComponent {
    // r[impl cairn.workspace_contract_matrix.release_readiness]
    // r[impl cairn.workspace_contract_matrix.release_readiness.optional]
    // r[impl cairn.workspace_contract_matrix.release_readiness.required]
    let policy_value = policy.unwrap_or(&Value::Null);
    let readiness_policy = workspace_contract_readiness_policy_from_value(policy_value);
    let receipt_value = receipt_path
        .as_ref()
        .and_then(|path| fs::read_to_string(path).ok())
        .and_then(|text| serde_json::from_str::<Value>(&text).ok());
    let readiness =
        evaluate_workspace_contract_matrix_readiness(&readiness_policy, receipt_value.as_ref());
    let value = json!({
        "mode": readiness.mode,
        "required": readiness.required,
        "passed": readiness.passed,
        "skipped": readiness.skipped,
        "disposition": readiness.disposition,
        "receipt_hash": readiness.receipt_hash,
        "boundary": readiness.boundary,
        "issues": workspace_contract_readiness_issues_json(&readiness),
    });
    let receipt_hash = canonical_json_hash(&value);
    let mut check = json!({
        "name": "workspace_contract_matrix",
        "passed": readiness.passed,
        "source": component_source("workspace_contract_matrix"),
        "receipt_hash": receipt_hash,
        "disposition": value["disposition"].clone(),
    });
    if readiness.skipped {
        check["skipped"] = Value::Bool(true);
    }
    if let Some(path) = receipt_path.as_ref() {
        check["path"] = json!(safe_relative_path(&root, path));
    }
    WorkspaceContractMatrixComponent {
        check,
        receipt_hash,
        value,
    }
}

fn stack_provenance_component(
    policy_path: PathBuf,
    receipt_path: Option<PathBuf>,
    graph_path: Option<PathBuf>,
) -> StackProvenanceComponent {
    let value = stack_provenance_receipt_value(policy_path, receipt_path, graph_path);
    let receipt_hash = stack_provenance_hash(&value);
    let mut check = json!({
        "name": "stack_provenance",
        "passed": value["passed"].as_bool().unwrap_or(false),
        "source": component_source("stack_provenance"),
        "receipt_hash": receipt_hash,
        "disposition": value["disposition"].clone(),
    });
    if value["skipped"].as_bool() == Some(true) {
        check["skipped"] = Value::Bool(true);
    }
    StackProvenanceComponent {
        check,
        receipt_hash,
        value,
    }
}

fn function_address_component(
    policy_path: PathBuf,
    valence_receipt_path: Option<PathBuf>,
    mantle_binding_path: Option<PathBuf>,
    kamacite_receipt_path: Option<PathBuf>,
    graph_summary_path: Option<PathBuf>,
) -> FunctionAddressComponent {
    // r[impl cairn.function_address_gates.policy]
    // r[impl cairn.function_address_gates.receipt_validation]
    // r[impl cairn.function_address_gates.graph_summary]
    let value = function_address_receipt_value(
        policy_path,
        valence_receipt_path,
        mantle_binding_path,
        kamacite_receipt_path,
        graph_summary_path,
    );
    let receipt_hash = function_address_hash(&value);
    let mut check = json!({
        "name": "function_address",
        "passed": value["passed"].as_bool().unwrap_or(false),
        "source": component_source("function_address"),
        "receipt_hash": receipt_hash,
        "disposition": value["disposition"].clone(),
    });
    if value["skipped"].as_bool() == Some(true) {
        check["skipped"] = Value::Bool(true);
    }
    FunctionAddressComponent {
        check,
        receipt_hash,
        value,
    }
}

fn preserves_corpus_component(root: PathBuf) -> ReadinessComponent {
    // r[impl release_evidence.preserves_readiness_summary.output]
    // r[impl release_evidence.preserves_readiness_summary.fail_closed]
    let path = root.join("fixtures/preserves-canonicalization/corpus.json");
    readiness_component("preserves_corpus_freshness", || {
        Ok(preserves_corpus_freshness_receipt(path))
    })
}

fn semantic_review_ledger_component(
    root: PathBuf,
    explicit_path: Option<PathBuf>,
) -> ReadinessComponent {
    // r[impl cairn.semantic_review.readiness_gate.release_readiness]
    // r[impl cairn.semantic_review.readiness_gate.release_readiness.supplied_valid]
    // r[impl cairn.semantic_review.readiness_gate.release_readiness.supplied_invalid]
    // r[impl cairn.semantic_review.readiness_gate.release_readiness.skipped_absent]
    // r[impl cairn.semantic_review.readiness_gate.boundary]
    // r[impl cairn.semantic_review.readiness_gate.boundary.no_reviewer_execution]
    let path = explicit_path.or_else(|| {
        let conventional = root.join("docs/semantic-review-ledger.json");
        conventional.exists().then_some(conventional)
    });
    let Some(path) = path else {
        let receipt = json!({
            "command": "review ledger",
            "valid": true,
            "verdict": "PASS",
            "skipped": true,
            "reason": "semantic review ledger not supplied and conventional docs/semantic-review-ledger.json is absent; Cairn did not run reviewers or infer semantic correctness",
        });
        let receipt_hash = canonical_json_hash(&receipt);
        return ReadinessComponent {
            check: json!({
                "name": "semantic_review_ledger",
                "passed": true,
                "source": component_source("semantic_review_ledger"),
                "skipped": true,
                "receipt_hash": receipt_hash,
            }),
            receipt_hash,
            value: receipt,
        };
    };

    match fs::read_to_string(&path)
        .map_err(|error| error.to_string())
        .and_then(|text| serde_json::from_str::<Value>(&text).map_err(|error| error.to_string()))
    {
        Ok(receipt) => {
            let is_passed = receipt["valid"].as_bool().unwrap_or(false)
                && matches!(
                    receipt["verdict"].as_str(),
                    Some("pass" | "PASS" | "approved" | "APPROVED")
                );
            let receipt_hash = receipt["receipt_hash"]
                .as_str()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| canonical_json_hash(&receipt));
            ReadinessComponent {
                check: json!({
                    "name": "semantic_review_ledger",
                    "passed": is_passed,
                    "source": component_source("semantic_review_ledger"),
                    "path": safe_relative_path(&root, &path),
                    "receipt_hash": receipt_hash,
                    "verdict": receipt["verdict"].clone(),
                }),
                receipt_hash,
                value: receipt,
            }
        }
        Err(issue) => ReadinessComponent {
            check: json!({
                "name": "semantic_review_ledger",
                "passed": false,
                "source": component_source("semantic_review_ledger"),
                "path": safe_relative_path(&root, &path),
                "issue": issue,
            }),
            receipt_hash: String::new(),
            value: Value::Null,
        },
    }
}

fn conventional_release_evidence_index_component(root: PathBuf) -> ReadinessComponent {
    // r[impl release_evidence.current_head_staleness_guard.release_readiness]
    let index = root.join("docs/release-evidence-current-head.md");
    if index.exists() {
        readiness_component("release_evidence_index_freshness", || {
            Ok(release_evidence_check_index_receipt_for_path(root, index))
        })
    } else {
        let receipt = json!({
            "command": "release-evidence check-index",
            "valid": true,
            "verdict": "pass",
            "skipped": true,
            "reason": "conventional checked-in current-head evidence page is absent; checked-in evidence is treated as immutable audited payload snapshots",
        });
        ReadinessComponent {
            check: json!({
                "name": "release_evidence_index_freshness",
                "passed": true,
                "source": component_source("release_evidence_index_freshness"),
                "skipped": true,
                "receipt_hash": canonical_json_hash(&receipt),
            }),
            receipt_hash: canonical_json_hash(&receipt),
            value: receipt,
        }
    }
}

struct ReadinessComponent {
    check: Value,
    receipt_hash: String,
    value: Value,
}

struct StackProvenanceComponent {
    check: Value,
    receipt_hash: String,
    value: Value,
}

struct FunctionAddressComponent {
    check: Value,
    receipt_hash: String,
    value: Value,
}

struct AuthenticatedReleaseComponent {
    check: Value,
    value: Value,
}

struct WorkspaceContractMatrixComponent {
    check: Value,
    receipt_hash: String,
    value: Value,
}

fn readiness_component<F>(name: &str, operation: F) -> ReadinessComponent
where
    F: FnOnce() -> Result<Value, Box<dyn Error>>,
{
    match operation() {
        Ok(receipt) => {
            let is_passed = receipt["valid"].as_bool().unwrap_or(false)
                || receipt["verdict"].as_str() == Some("pass")
                || receipt["verdict"].as_str() == Some("PASS");
            let receipt_hash = receipt["receipt_hash"]
                .as_str()
                .map(ToOwned::to_owned)
                .unwrap_or_else(|| canonical_json_hash(&receipt));
            ReadinessComponent {
                check: json!({
                    "name": name,
                    "passed": is_passed,
                    "source": component_source(name),
                    "receipt_hash": receipt_hash,
                }),
                receipt_hash,
                value: receipt,
            }
        }
        Err(error) => {
            let issue = error.to_string();
            ReadinessComponent {
                check: json!({
                    "name": name,
                    "passed": false,
                    "source": component_source(name),
                    "issue": issue,
                }),
                receipt_hash: String::new(),
                value: json!({"valid": false, "issues": [issue]}),
            }
        }
    }
}

fn component_source(name: &str) -> &'static str {
    match name {
        "cairn_validate" => "cairn validate --root <root>",
        "traceability_coverage" => "cairn traceability coverage --root <root>",
        "mcp_agent_smoke" => "cairn agent-receipt smoke-mcp --root <root>",
        "determinism_coverage_audit" => "cairn determinism coverage-audit --root <root>",
        "release_evidence_index_freshness" => {
            "cairn release-evidence check-index --root <root> --index docs/release-evidence-current-head.md"
        }
        "preserves_corpus_freshness" => {
            "cairn preserves-corpus update --path fixtures/preserves-canonicalization/corpus.json"
        }
        "semantic_review_ledger" => {
            "cairn release-readiness --root <root> --review-ledger <review-ledger.json>"
        }
        "stack_provenance" => {
            "cairn release-readiness --root <root> --stack-provenance <valence-receipt.json> --stack-provenance-graph <graph-summary.json>"
        }
        "function_address" => {
            "cairn release-readiness --root <root> --function-address-valence <valence-receipt.json> --function-address-mantle <mantle-binding.json> --function-address-kamacite <kamacite-receipt.json> --function-address-graph <graph-summary.json>"
        }
        "workspace_contract_matrix" => {
            "cairn release-readiness --root <root> --workspace-contract-matrix-receipt <receipt.json>"
        }
        _ => "unknown",
    }
}
