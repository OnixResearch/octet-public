//! Reviewed requirement-to-evidence contracts and required-evidence freshness queries.
//!
//! The contract is human-authored Nickel exported to deterministic JSON. Evaluation binds each
//! clause to an explicit evidence disposition, retained artifacts, source identities, and, where
//! required, attributable negative controls. The emitted decision is a Rust-owned DTO that can be
//! recomputed from the same contract and evidence without an AI model or hosted provider.
//!
//! Freshness queries consume the same contract as the expected-obligation inventory. They are
//! read-only: they never execute providers and never infer causal dependency from a shared label.

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::cli::OutputFormat;
use crate::cli::RequirementsArgs;
use crate::cli::RequirementsCheckArgs;
use crate::cli::RequirementsFreshnessArgs;
use crate::cli::RequirementsMode;
use crate::cli::RequirementsVerifyArgs;
use crate::evidence_multihop::EvidenceSnapshot;
use crate::evidence_multihop::QueryLimits;
use crate::evidence_multihop::ReachabilityRequest;
use crate::evidence_multihop::RelationKind;
use crate::evidence_multihop::SnapshotCompleteness;
use crate::evidence_multihop::evaluate_typed_reference;
use crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex;
use crate::valence_core_shim::evidence_codebase_index::EvidenceIndexIssueSeverity;
use crate::valence_core_shim::evidence_codebase_index::EvidenceIndexRecord;

const CONTRACT_SCHEMA_VERSION: &str = "octet-requirement-contract/v1";
const EVIDENCE_SCHEMA_VERSION: &str = "octet-requirement-evidence/v1";
const DECISION_SCHEMA_VERSION: &str = "octet-requirement-decision/v1";
const FRESHNESS_SCHEMA_VERSION: &str = "octet-requirement-freshness/v1";
const PROOF_CARDS_SCHEMA_VERSION: &str = "octet-proof-cards/v1";
const FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION: &str = "octet-formal-proof-envelope/v1";
const NEGATIVE_CHECK_SCHEMA_VERSION_V2: &str = "octet-proof-negative-check/v2";
const NEGATIVE_STATUS_DEMONSTRATED: &str = "property_demonstrated";
const REQUIRED_NON_CLAIM: &str = "evaluates supplied reviewed contracts and evidence only; does not authenticate reviewers or producers, prove verifier soundness, or establish whole-program correctness";
const MAX_ARTIFACT_BYTES: u64 = 16 * 1024 * 1024;
const MAX_RELATED_RECORDS: usize = 12;
const MAX_FRESHNESS_DEPTH: usize = 3;
const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;
const DISPOSITION_FORMAL_PROOF: &str = "formal_proof";
const DISPOSITION_EMPIRICAL_CHECK: &str = "empirical_check";
const DISPOSITION_ACCEPTED_ASSUMPTION: &str = "accepted_assumption";
const DISPOSITION_UNRESOLVED: &str = "unresolved";
const DISPOSITION_EXCLUDED: &str = "excluded";
const STATUS_PASSED: &str = "passed";
const STATUS_FAILED: &str = "failed";
const STATUS_MISSING: &str = "missing";
const STATUS_STALE_BINDING: &str = "stale_binding";
const STATUS_UNSUPPORTED: &str = "unsupported";
const STATUS_INCOMPLETE: &str = "incomplete";
const STATUS_EXCLUDED: &str = "excluded";
const STATUS_UNRESOLVED: &str = "unresolved";
const STATUS_ASSUMPTION_ACCEPTED: &str = "assumption_accepted";
const STATUS_CURRENT: &str = "current";
const CHECK_STATUS_PASSED: &str = "passed";
const ASSUMPTION_STATUS_ACCEPTED: &str = "accepted";
const BLAKE3_HEX_LENGTH: usize = 64;

/// Runs the requirements command.
pub(crate) fn run(args: &RequirementsArgs) -> Result<i32, String> {
    match &args.mode {
        RequirementsMode::Check(check) => run_check(check),
        RequirementsMode::Verify(verify) => run_verify(verify),
        RequirementsMode::Freshness(freshness) => run_freshness(freshness),
    }
}

fn run_check(args: &RequirementsCheckArgs) -> Result<i32, String> {
    if let Some(output) = &args.output
        && (output == &args.contract || output == &args.evidence)
    {
        return Err("requirements check output must not overwrite the contract or evidence input".to_owned());
    }
    let contract_bytes = read_bounded(&args.contract)?;
    let evidence_bytes = read_bounded(&args.evidence)?;
    let contract: RequirementContract = parse_json(&contract_bytes, &args.contract)?;
    let evidence: RequirementEvidence = parse_json(&evidence_bytes, &args.evidence)?;
    let decision = evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, &args.source_root)?;
    write_decision(&decision, args.output.as_deref(), args.output_format)?;
    Ok(exit_for_aggregate(&decision.aggregate_result))
}

fn run_verify(args: &RequirementsVerifyArgs) -> Result<i32, String> {
    let contract_bytes = read_bounded(&args.contract)?;
    let evidence_bytes = read_bounded(&args.evidence)?;
    let contract: RequirementContract = parse_json(&contract_bytes, &args.contract)?;
    let evidence: RequirementEvidence = parse_json(&evidence_bytes, &args.evidence)?;
    let recorded_bytes = read_bounded(&args.decision)?;
    let recorded: RequirementDecision = parse_json(&recorded_bytes, &args.decision)?;
    let recomputed = evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, &args.source_root)?;
    let mut diagnostics = Vec::new();
    compare_decisions(&recorded, &recomputed, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    let status = if diagnostics.is_empty() {
        STATUS_PASSED
    } else {
        STATUS_FAILED
    };
    let report = DecisionVerificationReport {
        schema_version: DECISION_SCHEMA_VERSION.to_owned(),
        status: status.to_owned(),
        contract_id: recomputed.contract_id.clone(),
        recorded_decision_blake3: recorded.decision_blake3.clone(),
        recomputed_decision_blake3: recomputed.decision_blake3.clone(),
        aggregate_result: recomputed.aggregate_result.clone(),
        diagnostics,
        non_claims: non_claims(&contract),
    };
    match args.output_format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("serialize decision verification: {error}"))?
        ),
        OutputFormat::Human | OutputFormat::Sarif => {
            println!(
                "octet requirements verify contract={} status={} aggregate={} recorded={} recomputed={} diagnostics={}",
                report.contract_id,
                report.status,
                report.aggregate_result,
                report.recorded_decision_blake3,
                report.recomputed_decision_blake3,
                report.diagnostics.len()
            );
            for diagnostic in &report.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            for non_claim in &report.non_claims {
                println!("non-claim: {non_claim}");
            }
        }
    }
    Ok(exit_for_aggregate(status))
}

fn run_freshness(args: &RequirementsFreshnessArgs) -> Result<i32, String> {
    let contract_bytes = read_bounded(&args.contract)?;
    let contract: RequirementContract = parse_json(&contract_bytes, &args.contract)?;
    let index_bytes = read_bounded(&args.index)?;
    let index: EvidenceCodebaseIndex = parse_json(&index_bytes, &args.index)?;
    let report = evaluate_freshness(&contract, &index, &contract_bytes)?;
    match args.output_format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| format!("serialize freshness report: {error}"))?
        ),
        OutputFormat::Human | OutputFormat::Sarif => print_freshness_human(&report),
    }
    Ok(exit_for_freshness(&report.aggregate_result))
}

fn exit_for_aggregate(result: &str) -> i32 {
    if result == STATUS_PASSED || result == STATUS_EXCLUDED {
        EXIT_SUCCESS
    } else {
        EXIT_FAILURE
    }
}

fn exit_for_freshness(result: &str) -> i32 {
    if result == STATUS_CURRENT {
        EXIT_SUCCESS
    } else {
        EXIT_FAILURE
    }
}

/// Requirement contract exported from Nickel.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RequirementContract {
    pub(crate) schema_version: String,
    pub(crate) contract_id: String,
    pub(crate) reviewed_specification: ReviewedSpecification,
    #[serde(default)]
    pub(crate) source_root: Option<String>,
    pub(crate) clauses: Vec<RequirementClause>,
    #[serde(default)]
    pub(crate) assumptions: Vec<AcceptedAssumption>,
    #[serde(default)]
    pub(crate) exclusions: Vec<ExcludedBehavior>,
    #[serde(default)]
    pub(crate) non_claims: Vec<String>,
}

/// Reviewed specification identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ReviewedSpecification {
    pub(crate) path: String,
    pub(crate) blake3: String,
}

/// One reviewed clause.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RequirementClause {
    pub(crate) clause_id: String,
    pub(crate) statement: String,
    #[serde(default = "default_true")]
    pub(crate) required: bool,
    pub(crate) disposition: String,
    #[serde(default)]
    pub(crate) checked_scope: Vec<String>,
    #[serde(default)]
    pub(crate) expected_evidence_tier: String,
    #[serde(default)]
    pub(crate) allowed_evidence_ids: Vec<String>,
    #[serde(default)]
    pub(crate) allowed_source_requirement_ids: Vec<String>,
    #[serde(default)]
    pub(crate) required_evidence_classes: Vec<String>,
    #[serde(default)]
    pub(crate) required_negative_controls: Vec<String>,
    #[serde(default)]
    pub(crate) required_bindings: RequirementBindings,
    #[serde(default)]
    pub(crate) source_bindings: Vec<SourceBinding>,
    #[serde(default)]
    pub(crate) assumption_id: Option<String>,
    #[serde(default)]
    pub(crate) exclusion_reason: Option<String>,
    #[serde(default)]
    pub(crate) required_provider_id: Option<String>,
}

/// Explicit bindings a freshness record must satisfy.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RequirementBindings {
    #[serde(default)]
    pub(crate) source_ids: Vec<String>,
    #[serde(default)]
    pub(crate) policy_ids: Vec<String>,
    #[serde(default)]
    pub(crate) tool_identity_hashes: Vec<String>,
    #[serde(default)]
    pub(crate) artifact_hashes: Vec<String>,
    #[serde(default)]
    pub(crate) checked_scope: Vec<String>,
    #[serde(default)]
    pub(crate) evidence_tier: Option<String>,
}

/// One expected source identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct SourceBinding {
    pub(crate) locator: String,
    pub(crate) blake3: String,
}

/// One accepted assumption.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AcceptedAssumption {
    pub(crate) assumption_id: String,
    pub(crate) statement: String,
    pub(crate) identity: String,
    pub(crate) trusted_boundary_identity: String,
    #[serde(default)]
    pub(crate) affected_clause_ids: Vec<String>,
    #[serde(default)]
    pub(crate) required_empirical_evidence: Vec<String>,
}

/// One explicit exclusion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ExcludedBehavior {
    pub(crate) exclusion_id: String,
    pub(crate) clause_id: String,
    pub(crate) reason: String,
    #[serde(default)]
    pub(crate) reviewed_specification_blake3: String,
}

/// Evidence bundle consumed by contract evaluation.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RequirementEvidence {
    pub(crate) schema_version: String,
    pub(crate) contract_id: String,
    #[serde(default)]
    pub(crate) contract_blake3: Option<String>,
    #[serde(default)]
    pub(crate) source_snapshot: Vec<SourceBinding>,
    #[serde(default)]
    pub(crate) proof_decks: Vec<ProofDeckEvidence>,
    #[serde(default)]
    pub(crate) empirical_checks: Vec<EmpiricalCheckEvidence>,
    #[serde(default)]
    pub(crate) assumption_records: Vec<AssumptionEvidence>,
    #[serde(default)]
    pub(crate) non_claims: Vec<String>,
}

/// One retained proof-card deck.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProofDeckEvidence {
    pub(crate) evidence_id: String,
    pub(crate) deck: String,
    #[serde(default = "default_dot")]
    pub(crate) artifact_root: String,
}

/// One empirical check record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct EmpiricalCheckEvidence {
    pub(crate) evidence_id: String,
    pub(crate) evidence_class: String,
    pub(crate) status: String,
    pub(crate) checker_id: String,
    #[serde(default)]
    pub(crate) command_argv: Vec<String>,
    #[serde(default)]
    pub(crate) checked_scope: Vec<String>,
    #[serde(default)]
    pub(crate) source_bindings: Vec<SourceBinding>,
    #[serde(default)]
    pub(crate) artifact: Option<String>,
    #[serde(default)]
    pub(crate) artifact_blake3: Option<String>,
    #[serde(default)]
    pub(crate) diagnostics: Vec<String>,
    #[serde(default)]
    pub(crate) non_claims: Vec<String>,
}

/// One assumption evidence record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AssumptionEvidence {
    pub(crate) assumption_id: String,
    pub(crate) identity: String,
    pub(crate) trusted_boundary_identity: String,
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) supporting_evidence_ids: Vec<String>,
    #[serde(default)]
    pub(crate) diagnostics: Vec<String>,
}

/// Rust-owned contract decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct RequirementDecision {
    pub(crate) schema_version: String,
    pub(crate) contract_id: String,
    pub(crate) contract_blake3: String,
    pub(crate) evidence_blake3: String,
    pub(crate) source_snapshot_blake3: String,
    pub(crate) reviewed_specification: ReviewedSpecification,
    pub(crate) aggregate_result: String,
    pub(crate) clauses: Vec<ClauseDecision>,
    pub(crate) assumptions: Vec<AssumptionDecision>,
    pub(crate) exclusions: Vec<ExcludedBehavior>,
    pub(crate) limitations: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) decision_blake3: String,
}

/// One clause decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ClauseDecision {
    pub(crate) clause_id: String,
    pub(crate) required: bool,
    pub(crate) disposition: String,
    pub(crate) status: String,
    pub(crate) evidence_ids: Vec<String>,
    pub(crate) source_bindings: Vec<SourceBinding>,
    pub(crate) diagnostics: Vec<String>,
}

/// One assumption decision.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AssumptionDecision {
    pub(crate) assumption_id: String,
    pub(crate) status: String,
    pub(crate) identity: String,
    pub(crate) trusted_boundary_identity: String,
    pub(crate) supporting_evidence_ids: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
}

/// Verification report for a stored decision.
#[derive(Clone, Debug, Serialize)]
struct DecisionVerificationReport {
    schema_version: String,
    status: String,
    contract_id: String,
    recorded_decision_blake3: String,
    recomputed_decision_blake3: String,
    aggregate_result: String,
    diagnostics: Vec<String>,
    non_claims: Vec<String>,
}

/// Freshness report over one evidence index.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct FreshnessReport {
    pub(crate) schema_version: String,
    pub(crate) contract_id: String,
    pub(crate) contract_blake3: String,
    pub(crate) index_hash: String,
    pub(crate) index_schema_version: String,
    pub(crate) snapshot_id: String,
    pub(crate) completeness: String,
    pub(crate) aggregate_result: String,
    pub(crate) clauses: Vec<ClauseFreshness>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

/// One clause freshness outcome.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ClauseFreshness {
    pub(crate) clause_id: String,
    pub(crate) required: bool,
    pub(crate) status: String,
    pub(crate) expected_bindings: RequirementBindings,
    pub(crate) matched_record_ids: Vec<String>,
    pub(crate) actual_bindings: Vec<ActualBindings>,
    pub(crate) related_records: Vec<RelatedRecord>,
    pub(crate) diagnostics: Vec<String>,
}

/// Binding facts observed on one related record.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct ActualBindings {
    pub(crate) record_id: String,
    pub(crate) stable_id: String,
    pub(crate) evidence_tier: String,
    pub(crate) source_ids: Vec<String>,
    pub(crate) policy_ids: Vec<String>,
    pub(crate) artifact_hashes: Vec<String>,
    pub(crate) requirement_ids: Vec<String>,
}

/// One bounded related-record explanation.
#[derive(Clone, Debug, Serialize)]
pub(crate) struct RelatedRecord {
    pub(crate) record_id: String,
    pub(crate) stable_id: String,
    pub(crate) depth: usize,
    pub(crate) relation: String,
    pub(crate) semantics: String,
}

fn default_true() -> bool {
    true
}

fn default_dot() -> String {
    ".".to_owned()
}

fn parse_json<T: for<'de> Deserialize<'de>>(bytes: &[u8], path: &Path) -> Result<T, String> {
    serde_json::from_slice(bytes).map_err(|error| format!("parse {} as JSON: {error}", path.display()))
}

fn read_bounded(path: &Path) -> Result<Vec<u8>, String> {
    let metadata = fs::metadata(path).map_err(|error| format!("read metadata {}: {error}", path.display()))?;
    if metadata.len() > MAX_ARTIFACT_BYTES {
        return Err(format!("{} exceeds the maximum retained artifact size", path.display()));
    }
    fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn read_json_bounded<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T, String> {
    let bytes = read_bounded(path)?;
    parse_json(&bytes, path)
}

fn non_claims(contract: &RequirementContract) -> Vec<String> {
    let mut claims = contract.non_claims.clone();
    if !claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
        claims.push(REQUIRED_NON_CLAIM.to_owned());
    }
    claims.sort();
    claims.dedup();
    claims
}

fn validate_contract(contract: &RequirementContract) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if contract.schema_version != CONTRACT_SCHEMA_VERSION {
        diagnostics.push(format!(
            "requirement contract field `schema_version` expected `{CONTRACT_SCHEMA_VERSION}` but got `{}`",
            contract.schema_version
        ));
    }
    push_required(&mut diagnostics, "contract_id", &contract.contract_id);
    push_required(&mut diagnostics, "reviewed_specification.path", &contract.reviewed_specification.path);
    if !is_blake3_hex(&contract.reviewed_specification.blake3) {
        diagnostics.push("reviewed_specification.blake3 must be a lowercase BLAKE3 hex digest".to_owned());
    }
    let mut clause_ids = BTreeSet::new();
    for clause in &contract.clauses {
        push_required(&mut diagnostics, "clause_id", &clause.clause_id);
        push_required(&mut diagnostics, "statement", &clause.statement);
        if !clause_ids.insert(clause.clause_id.clone()) {
            diagnostics.push(format!("duplicate requirement clause id `{}`", clause.clause_id));
        }
        if !matches!(
            clause.disposition.as_str(),
            DISPOSITION_FORMAL_PROOF
                | DISPOSITION_EMPIRICAL_CHECK
                | DISPOSITION_ACCEPTED_ASSUMPTION
                | DISPOSITION_UNRESOLVED
                | DISPOSITION_EXCLUDED
        ) {
            diagnostics.push(format!(
                "requirement clause `{}` has unsupported disposition `{}`",
                clause.clause_id, clause.disposition
            ));
        }
        if clause.disposition == DISPOSITION_EXCLUDED {
            if clause.exclusion_reason.as_deref().is_none_or(str::is_empty) {
                diagnostics
                    .push(format!("excluded requirement clause `{}` must declare exclusion_reason", clause.clause_id));
            }
        } else if clause.checked_scope.is_empty() {
            diagnostics.push(format!("requirement clause `{}` must declare checked_scope", clause.clause_id));
        }
        if clause.disposition == DISPOSITION_FORMAL_PROOF && clause.expected_evidence_tier != DISPOSITION_FORMAL_PROOF {
            diagnostics
                .push(format!("formal-proof clause `{}` must require evidence tier `formal_proof`", clause.clause_id));
        }
        if clause.disposition == DISPOSITION_ACCEPTED_ASSUMPTION && clause.assumption_id.is_none() {
            diagnostics.push(format!("assumption clause `{}` must name an assumption_id", clause.clause_id));
        }
        if clause.source_bindings.is_empty() && clause.disposition != DISPOSITION_EXCLUDED {
            diagnostics.push(format!(
                "requirement clause `{}` has an empty source scope; an unexamined scope cannot establish coverage",
                clause.clause_id
            ));
        }
        if clause.allowed_evidence_ids.is_empty() && clause.disposition != DISPOSITION_EXCLUDED {
            diagnostics.push(format!("requirement clause `{}` must name allowed_evidence_ids", clause.clause_id));
        }
        for binding in &clause.source_bindings {
            if !is_blake3_hex(&binding.blake3) {
                diagnostics.push(format!(
                    "requirement clause `{}` source binding `{}` must carry a BLAKE3 digest",
                    clause.clause_id, binding.locator
                ));
            }
        }
    }
    if clause_ids.is_empty() {
        diagnostics.push("requirement contract must declare at least one clause".to_owned());
    }
    if !contract.clauses.iter().any(|clause| clause.required) {
        diagnostics.push("requirement contract has an empty required inventory".to_owned());
    }
    let assumption_ids = contract
        .assumptions
        .iter()
        .map(|assumption| assumption.assumption_id.as_str())
        .collect::<BTreeSet<_>>();
    for assumption in &contract.assumptions {
        push_required(&mut diagnostics, "assumption_id", &assumption.assumption_id);
        push_required(&mut diagnostics, "assumption.identity", &assumption.identity);
        push_required(&mut diagnostics, "assumption.trusted_boundary_identity", &assumption.trusted_boundary_identity);
        for clause_id in &assumption.affected_clause_ids {
            if !clause_ids.contains(clause_id) {
                diagnostics.push(format!(
                    "assumption `{}` names unknown affected clause `{clause_id}`",
                    assumption.assumption_id
                ));
            }
        }
        if assumption.required_empirical_evidence.is_empty() {
            diagnostics.push(format!(
                "assumption `{}` must require at least one empirical evidence id",
                assumption.assumption_id
            ));
        }
    }
    for clause in &contract.clauses {
        if clause.disposition == DISPOSITION_ACCEPTED_ASSUMPTION
            && let Some(assumption_id) = &clause.assumption_id
            && !assumption_ids.contains(assumption_id.as_str())
        {
            diagnostics
                .push(format!("requirement clause `{}` names unknown assumption `{assumption_id}`", clause.clause_id));
        }
    }
    let exclusion_clause_ids =
        contract.exclusions.iter().map(|exclusion| exclusion.clause_id.as_str()).collect::<BTreeSet<_>>();
    for exclusion in &contract.exclusions {
        push_required(&mut diagnostics, "exclusion_id", &exclusion.exclusion_id);
        push_required(&mut diagnostics, "exclusion.reason", &exclusion.reason);
        if !clause_ids.contains(&exclusion.clause_id) {
            diagnostics
                .push(format!("exclusion `{}` names unknown clause `{}`", exclusion.exclusion_id, exclusion.clause_id));
        }
    }
    for clause in &contract.clauses {
        if clause.disposition == DISPOSITION_EXCLUDED && !exclusion_clause_ids.contains(clause.clause_id.as_str()) {
            diagnostics
                .push(format!("excluded requirement clause `{}` has no matching exclusion record", clause.clause_id));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn push_required(diagnostics: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("missing required requirement-contract field `{field}`"));
    }
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn hash_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn resolve_path(root: &Path, locator: &str) -> PathBuf {
    let path = Path::new(locator);
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

fn evaluate_contract(
    contract: &RequirementContract,
    evidence: &RequirementEvidence,
    contract_bytes: &[u8],
    evidence_bytes: &[u8],
    source_root: &Path,
) -> Result<RequirementDecision, String> {
    let contract_blake3 = hash_bytes(contract_bytes);
    let evidence_blake3 = hash_bytes(evidence_bytes);
    let mut diagnostics = validate_contract(contract);
    if evidence.schema_version != EVIDENCE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "requirement evidence field `schema_version` expected `{EVIDENCE_SCHEMA_VERSION}` but got `{}`",
            evidence.schema_version
        ));
    }
    if evidence.contract_id != contract.contract_id {
        diagnostics.push(format!(
            "requirement evidence contract_id `{}` does not match contract `{}`",
            evidence.contract_id, contract.contract_id
        ));
    }
    if let Some(declared) = &evidence.contract_blake3
        && declared != &contract_blake3
    {
        diagnostics.push("requirement evidence contract_blake3 does not match the supplied contract bytes".to_owned());
    }
    let spec_path = resolve_path(source_root, &contract.reviewed_specification.path);
    match fs::read(&spec_path) {
        Ok(bytes) => {
            let actual = hash_bytes(&bytes);
            if actual != contract.reviewed_specification.blake3 {
                diagnostics.push(format!(
                    "reviewed specification `{}` changed: expected {}, actual {actual}",
                    contract.reviewed_specification.path, contract.reviewed_specification.blake3
                ));
            }
        }
        Err(error) => diagnostics
            .push(format!("reviewed specification `{}` is missing: {error}", contract.reviewed_specification.path)),
    }
    let source_snapshot_blake3 =
        source_snapshot_hash(contract, &evidence.source_snapshot, source_root, &mut diagnostics);
    let assumptions = evaluate_assumptions(contract, evidence, &mut diagnostics);
    let mut clauses = Vec::new();
    for clause in &contract.clauses {
        clauses.push(evaluate_clause(contract, evidence, clause, source_root, &assumptions, &mut diagnostics));
    }
    if clauses.iter().all(|clause| !clause.required) {
        diagnostics.push("requirement contract has an empty required inventory".to_owned());
    }
    let mut assumptions_out = Vec::new();
    for assumption in &contract.assumptions {
        let decision = assumptions.get(&assumption.assumption_id).cloned().unwrap_or_else(|| AssumptionDecision {
            assumption_id: assumption.assumption_id.clone(),
            status: STATUS_MISSING.to_owned(),
            identity: assumption.identity.clone(),
            trusted_boundary_identity: assumption.trusted_boundary_identity.clone(),
            supporting_evidence_ids: Vec::new(),
            diagnostics: vec![format!(
                "assumption `{}` has no evidence record",
                assumption.assumption_id
            )],
        });
        assumptions_out.push(decision);
    }
    let mut limitations = Vec::new();
    for assumption in &contract.assumptions {
        limitations.push(format!(
            "accepted assumption `{}` is bound to identity {} and trust boundary {}",
            assumption.assumption_id, assumption.identity, assumption.trusted_boundary_identity
        ));
    }
    for exclusion in &contract.exclusions {
        limitations.push(format!(
            "excluded behavior `{}` for clause `{}`: {}",
            exclusion.exclusion_id, exclusion.clause_id, exclusion.reason
        ));
    }
    if !assumptions_out.iter().all(|assumption| assumption.status == STATUS_ASSUMPTION_ACCEPTED) {
        diagnostics.push("one or more accepted assumptions are missing, stale, or unsupported".to_owned());
    }
    let required_failed = clauses.iter().any(|clause| {
        clause.required
            && !matches!(clause.status.as_str(), STATUS_PASSED | STATUS_ASSUMPTION_ACCEPTED | STATUS_EXCLUDED)
    });
    let aggregate_result = if diagnostics.is_empty() && !required_failed {
        STATUS_PASSED
    } else {
        STATUS_FAILED
    }
    .to_owned();
    diagnostics.sort();
    diagnostics.dedup();
    let mut decision = RequirementDecision {
        schema_version: DECISION_SCHEMA_VERSION.to_owned(),
        contract_id: contract.contract_id.clone(),
        contract_blake3,
        evidence_blake3,
        source_snapshot_blake3,
        reviewed_specification: contract.reviewed_specification.clone(),
        aggregate_result,
        clauses,
        assumptions: assumptions_out,
        exclusions: contract.exclusions.clone(),
        limitations,
        diagnostics,
        non_claims: non_claims(contract),
        decision_blake3: String::new(),
    };
    decision.decision_blake3 = decision_hash(&decision)?;
    Ok(decision)
}

fn source_snapshot_hash(
    contract: &RequirementContract,
    declared: &[SourceBinding],
    source_root: &Path,
    diagnostics: &mut Vec<String>,
) -> String {
    let mut bindings = BTreeMap::new();
    for binding in declared {
        bindings.insert(binding.locator.clone(), binding.blake3.clone());
    }
    for clause in &contract.clauses {
        for binding in &clause.source_bindings {
            let path = resolve_path(source_root, &binding.locator);
            match fs::read(&path) {
                Ok(bytes) => {
                    let actual = hash_bytes(&bytes);
                    if actual != binding.blake3 {
                        diagnostics.push(format!(
                            "clause `{}` source `{}` changed: expected {}, actual {actual}",
                            clause.clause_id, binding.locator, binding.blake3
                        ));
                    }
                    if let Some(declared_hash) = bindings.get(&binding.locator) {
                        if declared_hash != &actual {
                            diagnostics.push(format!(
                                "evidence source_snapshot `{}` does not match the checked source bytes",
                                binding.locator
                            ));
                        }
                    } else {
                        bindings.insert(binding.locator.clone(), actual);
                    }
                }
                Err(error) => diagnostics.push(format!(
                    "clause `{}` source `{}` is missing or unreadable: {error}",
                    clause.clause_id, binding.locator
                )),
            }
        }
    }
    let canonical = serde_json::to_vec(&bindings).unwrap_or_default();
    hash_bytes(&canonical)
}

fn evaluate_assumptions(
    contract: &RequirementContract,
    evidence: &RequirementEvidence,
    diagnostics: &mut Vec<String>,
) -> BTreeMap<String, AssumptionDecision> {
    let empirical = evidence
        .empirical_checks
        .iter()
        .map(|check| (check.evidence_id.as_str(), check))
        .collect::<BTreeMap<_, _>>();
    let mut decisions = BTreeMap::new();
    for assumption in &contract.assumptions {
        let record = evidence.assumption_records.iter().find(|record| record.assumption_id == assumption.assumption_id);
        let mut assumption_diagnostics = Vec::new();
        let Some(record) = record else {
            assumption_diagnostics
                .push(format!("assumption `{}` has no retained evidence record", assumption.assumption_id));
            decisions.insert(assumption.assumption_id.clone(), AssumptionDecision {
                assumption_id: assumption.assumption_id.clone(),
                status: STATUS_MISSING.to_owned(),
                identity: assumption.identity.clone(),
                trusted_boundary_identity: assumption.trusted_boundary_identity.clone(),
                supporting_evidence_ids: Vec::new(),
                diagnostics: assumption_diagnostics,
            });
            continue;
        };
        if record.status != ASSUMPTION_STATUS_ACCEPTED {
            assumption_diagnostics
                .push(format!("assumption `{}` evidence status is `{}`", assumption.assumption_id, record.status));
        }
        if record.identity != assumption.identity {
            assumption_diagnostics.push(format!(
                "assumption `{}` identity changed: expected {}, retained {}",
                assumption.assumption_id, assumption.identity, record.identity
            ));
        }
        if record.trusted_boundary_identity != assumption.trusted_boundary_identity {
            assumption_diagnostics.push(format!(
                "assumption `{}` trusted boundary changed: expected {}, retained {}",
                assumption.assumption_id, assumption.trusted_boundary_identity, record.trusted_boundary_identity
            ));
        }
        let mut supporting = Vec::new();
        for required_id in &assumption.required_empirical_evidence {
            let Some(check) = empirical.get(required_id.as_str()) else {
                assumption_diagnostics.push(format!(
                    "assumption `{}` requires missing empirical evidence `{required_id}`",
                    assumption.assumption_id
                ));
                continue;
            };
            if check.status != CHECK_STATUS_PASSED {
                assumption_diagnostics.push(format!(
                    "assumption `{}` required empirical evidence `{required_id}` status is `{}`",
                    assumption.assumption_id, check.status
                ));
            }
            supporting.push(required_id.clone());
        }
        for supplied in &record.supporting_evidence_ids {
            if !supporting.contains(supplied) {
                assumption_diagnostics.push(format!(
                    "assumption `{}` evidence record cites unsupported evidence `{supplied}`",
                    assumption.assumption_id
                ));
            }
        }
        for diagnostic in &record.diagnostics {
            assumption_diagnostics.push(format!("assumption `{}`: {diagnostic}", assumption.assumption_id));
        }
        assumption_diagnostics.sort();
        assumption_diagnostics.dedup();
        diagnostics.extend(
            assumption_diagnostics
                .iter()
                .map(|diagnostic| format!("assumption `{}`: {diagnostic}", assumption.assumption_id)),
        );
        let status = if assumption_diagnostics.is_empty() {
            STATUS_ASSUMPTION_ACCEPTED
        } else {
            STATUS_FAILED
        };
        if status != STATUS_ASSUMPTION_ACCEPTED {
            diagnostics.push(format!("assumption `{}` is not accepted", assumption.assumption_id));
        }
        decisions.insert(assumption.assumption_id.clone(), AssumptionDecision {
            assumption_id: assumption.assumption_id.clone(),
            status: status.to_owned(),
            identity: record.identity.clone(),
            trusted_boundary_identity: record.trusted_boundary_identity.clone(),
            supporting_evidence_ids: supporting,
            diagnostics: assumption_diagnostics,
        });
    }
    decisions
}

fn evaluate_clause(
    contract: &RequirementContract,
    evidence: &RequirementEvidence,
    clause: &RequirementClause,
    source_root: &Path,
    assumptions: &BTreeMap<String, AssumptionDecision>,
    diagnostics: &mut Vec<String>,
) -> ClauseDecision {
    let mut clause_diagnostics = Vec::new();
    if clause.source_bindings.is_empty() && clause.disposition != DISPOSITION_EXCLUDED {
        clause_diagnostics.push("source scope is empty; no evidence can establish coverage".to_owned());
    }
    for binding in &clause.source_bindings {
        let path = resolve_path(source_root, &binding.locator);
        match fs::read(&path) {
            Ok(bytes) => {
                let actual = hash_bytes(&bytes);
                if actual != binding.blake3 {
                    clause_diagnostics.push(format!(
                        "source `{}` changed: expected {}, actual {actual}",
                        binding.locator, binding.blake3
                    ));
                }
            }
            Err(error) => clause_diagnostics.push(format!("source `{}` is unreadable: {error}", binding.locator)),
        }
    }
    let (status, evidence_ids) = match clause.disposition.as_str() {
        DISPOSITION_EXCLUDED => {
            let matching = contract.exclusions.iter().find(|exclusion| exclusion.clause_id == clause.clause_id);
            if matching.is_none() {
                clause_diagnostics.push("excluded clause has no exclusion record".to_owned());
            }
            (STATUS_EXCLUDED.to_owned(), Vec::new())
        }
        DISPOSITION_UNRESOLVED => {
            clause_diagnostics.push("required clause remains unresolved".to_owned());
            (STATUS_UNRESOLVED.to_owned(), Vec::new())
        }
        DISPOSITION_FORMAL_PROOF => evaluate_formal_clause(evidence, clause, source_root, &mut clause_diagnostics),
        DISPOSITION_EMPIRICAL_CHECK => {
            evaluate_empirical_clause(evidence, clause, source_root, &mut clause_diagnostics)
        }
        DISPOSITION_ACCEPTED_ASSUMPTION => match &clause.assumption_id {
            None => {
                clause_diagnostics.push("assumption clause does not name an assumption".to_owned());
                (STATUS_FAILED.to_owned(), Vec::new())
            }
            Some(assumption_id) => match assumptions.get(assumption_id) {
                Some(assumption) if assumption.status == STATUS_ASSUMPTION_ACCEPTED => {
                    (STATUS_ASSUMPTION_ACCEPTED.to_owned(), assumption.supporting_evidence_ids.clone())
                }
                Some(assumption) => {
                    clause_diagnostics
                        .push(format!("assumption `{assumption_id}` is not accepted: {}", assumption.status));
                    (STATUS_FAILED.to_owned(), assumption.supporting_evidence_ids.clone())
                }
                None => {
                    clause_diagnostics.push(format!("assumption `{assumption_id}` has no decision"));
                    (STATUS_FAILED.to_owned(), Vec::new())
                }
            },
        },
        _ => {
            clause_diagnostics.push(format!("unsupported disposition `{}`", clause.disposition));
            (STATUS_UNSUPPORTED.to_owned(), Vec::new())
        }
    };
    if clause.required && status != STATUS_PASSED && status != STATUS_ASSUMPTION_ACCEPTED {
        clause_diagnostics.push(format!("required clause `{}` did not pass", clause.clause_id));
    }
    clause_diagnostics.sort();
    clause_diagnostics.dedup();
    if !clause_diagnostics.is_empty() && status == STATUS_PASSED {
        // A passing status with diagnostics is contradictory; fail closed.
        let status = STATUS_FAILED;
        return ClauseDecision {
            clause_id: clause.clause_id.clone(),
            required: clause.required,
            disposition: clause.disposition.clone(),
            status: status.to_owned(),
            evidence_ids,
            source_bindings: clause.source_bindings.clone(),
            diagnostics: clause_diagnostics,
        };
    }
    if status == STATUS_FAILED && clause_diagnostics.is_empty() {
        clause_diagnostics.push("clause failed without a diagnostic".to_owned());
    }
    if clause.required && status == STATUS_EXCLUDED {
        clause_diagnostics.push("required clause cannot be satisfied by an exclusion".to_owned());
    }
    diagnostics
        .extend(clause_diagnostics.iter().map(|diagnostic| format!("clause `{}`: {diagnostic}", clause.clause_id)));
    ClauseDecision {
        clause_id: clause.clause_id.clone(),
        required: clause.required,
        disposition: clause.disposition.clone(),
        status: status.to_owned(),
        evidence_ids,
        source_bindings: clause.source_bindings.clone(),
        diagnostics: clause_diagnostics,
    }
}

fn evaluate_formal_clause(
    evidence: &RequirementEvidence,
    clause: &RequirementClause,
    source_root: &Path,
    diagnostics: &mut Vec<String>,
) -> (String, Vec<String>) {
    let mut evidence_ids = Vec::new();
    for deck in &evidence.proof_decks {
        if !clause.allowed_evidence_ids.contains(&deck.evidence_id) {
            continue;
        }
        let deck_path = resolve_path(source_root, &deck.deck);
        let deck_json = match read_json_bounded::<serde_json::Value>(&deck_path) {
            Ok(value) => value,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        if deck_json.get("schema_version").and_then(serde_json::Value::as_str) != Some(PROOF_CARDS_SCHEMA_VERSION) {
            diagnostics.push(format!("proof deck `{}` has an unsupported schema", deck_path.display()));
            continue;
        }
        let Some(cards) = deck_json.get("cards").and_then(serde_json::Value::as_array) else {
            diagnostics.push(format!("proof deck `{}` has no cards array", deck_path.display()));
            continue;
        };
        for card in cards {
            let Some(card_id) = card.get("proof_or_property_id").and_then(serde_json::Value::as_str) else {
                continue;
            };
            if !clause.allowed_evidence_ids.contains(&card_id.to_owned()) {
                continue;
            }
            let mut card_diagnostics = Vec::new();
            validate_formal_card(card, clause, &deck.artifact_root, source_root, &mut card_diagnostics);
            if card_diagnostics.is_empty() {
                evidence_ids.push(card_id.to_owned());
            } else {
                diagnostics.extend(card_diagnostics);
            }
        }
    }
    if evidence_ids.is_empty() && diagnostics.is_empty() {
        diagnostics.push(format!(
            "clause `{}` has no retained formal-proof card among allowed evidence ids",
            clause.clause_id
        ));
    }
    let status = if evidence_ids.is_empty() {
        STATUS_MISSING
    } else {
        STATUS_PASSED
    };
    (status.to_owned(), evidence_ids)
}

fn validate_formal_card(
    card: &serde_json::Value,
    clause: &RequirementClause,
    artifact_root: &str,
    source_root: &Path,
    diagnostics: &mut Vec<String>,
) {
    let status = card.get("status").and_then(serde_json::Value::as_str).unwrap_or_default();
    let tier = card.get("evidence_tier").and_then(serde_json::Value::as_str).unwrap_or_default();
    if status != CHECK_STATUS_PASSED {
        diagnostics.push(format!("proof card status `{status}` is not `{CHECK_STATUS_PASSED}`"));
    }
    if tier != DISPOSITION_FORMAL_PROOF {
        diagnostics.push(format!("proof card evidence tier `{tier}` is not `{DISPOSITION_FORMAL_PROOF}`"));
    }
    if let Some(source_requirement) = card.get("source_requirement_or_policy_id").and_then(serde_json::Value::as_str)
        && !clause.allowed_source_requirement_ids.is_empty()
        && !clause.allowed_source_requirement_ids.iter().any(|allowed| allowed == source_requirement)
    {
        diagnostics.push(format!(
            "proof card source requirement `{source_requirement}` is not allowed for clause `{}`",
            clause.clause_id
        ));
    }
    if let Some(scope) = card.get("checked_rust_source_scope").and_then(serde_json::Value::as_str)
        && !clause.checked_scope.iter().any(|allowed| allowed == scope)
    {
        diagnostics
            .push(format!("proof card checked scope `{scope}` is not allowed for clause `{}`", clause.clause_id));
    }
    let card_diagnostics = card.get("diagnostics").and_then(serde_json::Value::as_array);
    if card_diagnostics.is_some_and(|items| !items.is_empty()) {
        diagnostics.push("proof card carries diagnostics".to_owned());
    }
    let non_claims = card.get("non_claims").and_then(serde_json::Value::as_array);
    if non_claims.is_none_or(|items| items.is_empty()) {
        diagnostics.push("proof card must carry explicit non-claims".to_owned());
    }
    if clause.required_negative_controls.is_empty() {
        // no negative control required
    } else {
        validate_negative_control(card, clause, diagnostics);
    }
    let artifact_root = resolve_path(source_root, artifact_root);
    if let Some(identities) = card.get("artifact_blake3_identities").and_then(serde_json::Value::as_array) {
        for identity in identities {
            let Some(locator) = identity.get("artifact").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let Some(expected) = identity.get("blake3").and_then(serde_json::Value::as_str) else {
                continue;
            };
            let path = resolve_path(&artifact_root, locator);
            match fs::read(&path) {
                Ok(bytes) => {
                    let actual = hash_bytes(&bytes);
                    if actual != expected {
                        diagnostics
                            .push(format!("proof artifact `{locator}` changed: expected {expected}, actual {actual}"));
                    }
                }
                Err(error) => diagnostics.push(format!("proof artifact `{locator}` is missing: {error}")),
            }
        }
    } else {
        diagnostics.push("proof card has no retained artifact identities".to_owned());
    }
    if let Some(envelope) = card.get("formal_proof_envelope") {
        let envelope_schema = envelope.get("schema_version").and_then(serde_json::Value::as_str);
        if envelope_schema != Some(FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION) {
            diagnostics.push("formal proof envelope has an unsupported schema".to_owned());
        }
        if envelope.get("status").and_then(serde_json::Value::as_str) != Some(CHECK_STATUS_PASSED) {
            diagnostics.push("formal proof envelope status is not passed".to_owned());
        }
        if let Some(provider) = &clause.required_provider_id {
            let actual = envelope.get("provider_id").and_then(serde_json::Value::as_str);
            if actual != Some(provider.as_str()) {
                diagnostics.push(format!(
                    "formal proof provider `{}` does not match required provider `{provider}`",
                    actual.unwrap_or("missing")
                ));
            }
        }
    } else if clause.required_provider_id.is_some() {
        diagnostics.push("required provider evidence is missing a formal proof envelope".to_owned());
    }
}

fn validate_negative_control(card: &serde_json::Value, clause: &RequirementClause, diagnostics: &mut Vec<String>) {
    let Some(negative) = card.get("negative_check") else {
        diagnostics.push(format!(
            "clause `{}` requires an attributable negative control but the card has none",
            clause.clause_id
        ));
        return;
    };
    if negative.get("schema_version").and_then(serde_json::Value::as_str) != Some(NEGATIVE_CHECK_SCHEMA_VERSION_V2) {
        diagnostics.push("negative control is not a v2 attributable record".to_owned());
        return;
    }
    if negative.get("status").and_then(serde_json::Value::as_str) != Some(NEGATIVE_STATUS_DEMONSTRATED) {
        diagnostics.push("negative control status is not property_demonstrated".to_owned());
    }
    let Some(evidence) = negative.get("evidence").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control has no retained verifier evidence".to_owned());
        return;
    };
    let string = |field: &str| evidence.get(field).and_then(serde_json::Value::as_str).unwrap_or_default();
    if string("verifier").trim().is_empty() {
        diagnostics.push("negative control evidence has no verifier identity".to_owned());
    }
    let card_id = card.get("proof_or_property_id").and_then(serde_json::Value::as_str).unwrap_or_default();
    if string("proof_or_property_id") != card_id {
        diagnostics.push("negative control evidence proof_or_property_id does not match the proof card".to_owned());
    }
    if string("approved_specification_id").trim().is_empty() {
        diagnostics.push("negative control evidence has no approved specification identity".to_owned());
    }
    if string("source_requirement_or_policy_id").trim().is_empty() {
        diagnostics.push("negative control evidence has no source requirement identity".to_owned());
    }
    let Some(baseline) = evidence.get("baseline").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control evidence has no baseline verifier run".to_owned());
        return;
    };
    let Some(mutant) = evidence.get("mutant").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control evidence has no mutant verifier run".to_owned());
        return;
    };
    if baseline.get("outcome").and_then(serde_json::Value::as_str) != Some("accepted") {
        diagnostics.push("negative control baseline verifier run was not accepted".to_owned());
    }
    if baseline
        .get("rejection_attribution")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|value| !value.is_empty())
    {
        diagnostics.push("negative control baseline run unexpectedly attributes a rejection".to_owned());
    }
    if mutant.get("outcome").and_then(serde_json::Value::as_str) != Some("rejected") {
        diagnostics.push("negative control mutant verifier run was not rejected".to_owned());
    }
    let attribution = mutant.get("rejection_attribution").and_then(serde_json::Value::as_str);
    if attribution != Some(card_id) {
        diagnostics.push(format!(
            "negative control rejection is attributed to `{}` instead of the expected obligation `{card_id}`",
            attribution.unwrap_or("missing")
        ));
    }
    let Some(mutation) = evidence.get("mutation").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control evidence has no declared mutation identity".to_owned());
        return;
    };
    if mutation.get("patch_blake3").and_then(serde_json::Value::as_str).is_none_or(str::is_empty) {
        diagnostics.push("negative control mutation has no patch identity".to_owned());
    }
    let baseline_source =
        mutation.get("baseline_source_blake3").and_then(serde_json::Value::as_str).unwrap_or_default();
    let mutant_source = mutation.get("mutant_source_blake3").and_then(serde_json::Value::as_str).unwrap_or_default();
    if baseline_source.is_empty() || mutant_source.is_empty() {
        diagnostics.push("negative control mutation does not retain both source identities".to_owned());
    } else if baseline_source == mutant_source {
        diagnostics.push("negative control mutant source bytes are unchanged".to_owned());
    }
    let Some(measured) = negative.get("measured").and_then(serde_json::Value::as_object) else {
        diagnostics.push("negative control record has no measured identities".to_owned());
        return;
    };
    for field in [
        "baseline_source_blake3",
        "mutant_source_blake3",
        "mutation_id",
        "patch_blake3",
    ] {
        if measured.get(field).and_then(serde_json::Value::as_str).is_none_or(str::is_empty) {
            diagnostics.push(format!("negative control measured identity `{field}` is missing"));
        }
    }
    if measured.get("baseline_source_blake3").and_then(serde_json::Value::as_str) != Some(baseline_source) {
        diagnostics.push("negative control measured baseline source does not match the declared mutation".to_owned());
    }
    if measured.get("mutant_source_blake3").and_then(serde_json::Value::as_str) != Some(mutant_source) {
        diagnostics.push("negative control measured mutant source does not match the declared mutation".to_owned());
    }
}

fn evaluate_empirical_clause(
    evidence: &RequirementEvidence,
    clause: &RequirementClause,
    source_root: &Path,
    diagnostics: &mut Vec<String>,
) -> (String, Vec<String>) {
    let mut evidence_ids = Vec::new();
    for check in &evidence.empirical_checks {
        if !clause.allowed_evidence_ids.contains(&check.evidence_id) {
            continue;
        }
        let mut check_diagnostics = Vec::new();
        if check.evidence_class != DISPOSITION_EMPIRICAL_CHECK {
            check_diagnostics.push(format!(
                "evidence `{}` class `{}` is not an empirical check",
                check.evidence_id, check.evidence_class
            ));
        }
        if check.status != CHECK_STATUS_PASSED {
            check_diagnostics.push(format!("empirical evidence `{}` status is `{}`", check.evidence_id, check.status));
        }
        if check.command_argv.is_empty() {
            check_diagnostics.push(format!("empirical evidence `{}` has no command argv", check.evidence_id));
        }
        if check.checked_scope.is_empty() {
            check_diagnostics.push(format!("empirical evidence `{}` has no checked scope", check.evidence_id));
        }
        for binding in &check.source_bindings {
            match fs::read(resolve_path(source_root, &binding.locator)) {
                Ok(bytes) => {
                    let actual = hash_bytes(&bytes);
                    if actual != binding.blake3 {
                        check_diagnostics.push(format!(
                            "empirical evidence `{}` source `{}` changed",
                            check.evidence_id, binding.locator
                        ));
                    }
                }
                Err(error) => check_diagnostics.push(format!(
                    "empirical evidence `{}` source `{}` is unreadable: {error}",
                    check.evidence_id, binding.locator
                )),
            }
        }
        if let (Some(artifact), Some(expected)) = (&check.artifact, &check.artifact_blake3) {
            match fs::read(resolve_path(source_root, artifact)) {
                Ok(bytes) => {
                    let actual = hash_bytes(&bytes);
                    if &actual != expected {
                        check_diagnostics
                            .push(format!("empirical evidence `{}` artifact `{artifact}` changed", check.evidence_id));
                    }
                }
                Err(error) => check_diagnostics.push(format!(
                    "empirical evidence `{}` artifact `{artifact}` is unreadable: {error}",
                    check.evidence_id
                )),
            }
        } else {
            check_diagnostics.push(format!("empirical evidence `{}` has no retained artifact", check.evidence_id));
        }
        if check_diagnostics.is_empty() {
            evidence_ids.push(check.evidence_id.clone());
        } else {
            diagnostics.extend(check_diagnostics);
        }
    }
    if evidence_ids.is_empty() && diagnostics.is_empty() {
        diagnostics.push(format!(
            "clause `{}` has no retained empirical evidence among allowed evidence ids",
            clause.clause_id
        ));
    }
    let status = if evidence_ids.is_empty() {
        STATUS_MISSING
    } else if clause.required_evidence_classes.iter().any(|class| class == "formal_proof") {
        diagnostics.push(format!(
            "empirical evidence cannot satisfy the formal-proof requirement for clause `{}`",
            clause.clause_id
        ));
        STATUS_FAILED
    } else {
        STATUS_PASSED
    };
    (status.to_owned(), evidence_ids)
}

fn decision_hash(decision: &RequirementDecision) -> Result<String, String> {
    let mut body = decision.clone();
    body.decision_blake3 = String::new();
    let bytes = serde_json::to_vec(&body).map_err(|error| format!("serialize requirement decision: {error}"))?;
    Ok(hash_bytes(&bytes))
}

fn write_decision(decision: &RequirementDecision, output: Option<&Path>, format: OutputFormat) -> Result<(), String> {
    let text =
        serde_json::to_string_pretty(decision).map_err(|error| format!("serialize requirement decision: {error}"))?;
    if let Some(output) = output {
        fs::write(output, format!("{text}\n"))
            .map_err(|error| format!("write requirement decision {}: {error}", output.display()))?;
    }
    match format {
        OutputFormat::Json => println!("{text}"),
        OutputFormat::Human | OutputFormat::Sarif => {
            println!(
                "octet requirements check contract={} status={} clauses={} assumptions={} exclusions={} diagnostics={}",
                decision.contract_id,
                decision.aggregate_result,
                decision.clauses.len(),
                decision.assumptions.len(),
                decision.exclusions.len(),
                decision.diagnostics.len()
            );
            for clause in &decision.clauses {
                println!(
                    "clause={} disposition={} required={} status={} evidence=[{}]",
                    clause.clause_id,
                    clause.disposition,
                    clause.required,
                    clause.status,
                    clause.evidence_ids.join(",")
                );
                for diagnostic in &clause.diagnostics {
                    println!("  clause-diagnostic: {diagnostic}");
                }
            }
            for assumption in &decision.assumptions {
                println!(
                    "assumption={} status={} identity={} supporting=[{}]",
                    assumption.assumption_id,
                    assumption.status,
                    assumption.identity,
                    assumption.supporting_evidence_ids.join(",")
                );
            }
            for exclusion in &decision.exclusions {
                println!(
                    "excluded={} clause={} reason={}",
                    exclusion.exclusion_id, exclusion.clause_id, exclusion.reason
                );
            }
            for limitation in &decision.limitations {
                println!("limitation: {limitation}");
            }
            for diagnostic in &decision.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            for non_claim in &decision.non_claims {
                println!("non-claim: {non_claim}");
            }
        }
    }
    Ok(())
}

fn compare_decisions(recorded: &RequirementDecision, recomputed: &RequirementDecision, diagnostics: &mut Vec<String>) {
    if recorded.decision_blake3 != recomputed.decision_blake3 {
        diagnostics.push(format!(
            "recorded decision identity {} does not match the recomputed identity {}",
            recorded.decision_blake3, recomputed.decision_blake3
        ));
    }
    for (field, recorded_value, recomputed_value) in [
        ("contract_blake3", recorded.contract_blake3.as_str(), recomputed.contract_blake3.as_str()),
        ("evidence_blake3", recorded.evidence_blake3.as_str(), recomputed.evidence_blake3.as_str()),
        (
            "source_snapshot_blake3",
            recorded.source_snapshot_blake3.as_str(),
            recomputed.source_snapshot_blake3.as_str(),
        ),
        ("aggregate_result", recorded.aggregate_result.as_str(), recomputed.aggregate_result.as_str()),
    ] {
        if recorded_value != recomputed_value {
            diagnostics.push(format!(
                "recorded decision {field} `{recorded_value}` does not match recomputed `{recomputed_value}`"
            ));
        }
    }
    if recorded.clauses != recomputed.clauses {
        diagnostics.push("recorded clause decisions do not match the recomputed decisions".to_owned());
    }
    if recorded.assumptions != recomputed.assumptions {
        diagnostics.push("recorded assumption decisions do not match the recomputed decisions".to_owned());
    }
    if recorded.exclusions != recomputed.exclusions {
        diagnostics.push("recorded exclusions do not match the recomputed exclusions".to_owned());
    }
    if recorded.reviewed_specification != recomputed.reviewed_specification {
        diagnostics.push("recorded reviewed specification identity does not match the contract".to_owned());
    }
    let declared = decision_hash(recorded).unwrap_or_default();
    if declared != recorded.decision_blake3 {
        diagnostics.push("recorded decision bytes are stale or tampered".to_owned());
    }
}

fn evaluate_freshness(
    contract: &RequirementContract,
    index: &EvidenceCodebaseIndex,
    contract_bytes: &[u8],
) -> Result<FreshnessReport, String> {
    let mut diagnostics = validate_contract(contract);
    let contract_blake3 = hash_bytes(contract_bytes);
    let index_has_errors = index.issues.iter().any(|issue| issue.severity == EvidenceIndexIssueSeverity::Error);
    let index_has_caveats = index.issues.iter().any(|issue| issue.severity == EvidenceIndexIssueSeverity::Caveat);
    let completeness = if index.index_hash.is_empty() {
        SnapshotCompleteness::Undeclared
    } else if index_has_errors || index_has_caveats {
        SnapshotCompleteness::DeclaredPartial {
            reason: "evidence index carries issues or caveats".to_owned(),
        }
    } else {
        SnapshotCompleteness::DeclaredComplete
    };
    let completeness_label = match &completeness {
        SnapshotCompleteness::DeclaredComplete => "declared_complete",
        SnapshotCompleteness::DeclaredPartial { .. } => "declared_partial",
        SnapshotCompleteness::Undeclared => "undeclared",
    }
    .to_owned();
    let snapshot = EvidenceSnapshot::from_index_declared(index, completeness);
    let mut clauses = Vec::new();
    for clause in &contract.clauses {
        if clause.disposition == DISPOSITION_EXCLUDED {
            clauses.push(ClauseFreshness {
                clause_id: clause.clause_id.clone(),
                required: clause.required,
                status: STATUS_EXCLUDED.to_owned(),
                expected_bindings: clause.required_bindings.clone(),
                matched_record_ids: Vec::new(),
                actual_bindings: Vec::new(),
                related_records: Vec::new(),
                diagnostics: vec!["clause is explicitly excluded from this contract".to_owned()],
            });
            continue;
        }
        let related = related_records(&snapshot, &clause.clause_id);
        if related.truncated {
            diagnostics.push(format!(
                "clause `{}` related-record traversal was truncated; coverage is incomplete",
                clause.clause_id
            ));
        }
        let candidates =
            index.records.iter().filter(|record| record_matches_clause(record, clause)).collect::<Vec<_>>();
        let mut actual = candidates.iter().map(|record| actual_bindings(record)).collect::<Vec<_>>();
        actual.sort_by(|left, right| left.record_id.cmp(&right.record_id));
        let matched = candidates
            .iter()
            .filter(|record| record_satisfies_bindings(record, clause))
            .map(|record| record.record_id.clone())
            .collect::<Vec<_>>();
        let mut clause_diagnostics = Vec::new();
        let mut status = if matched.is_empty() {
            if candidates.is_empty() {
                if matches!(snapshot.completeness, SnapshotCompleteness::DeclaredComplete) {
                    clause_diagnostics.push(format!(
                        "clause `{}` has no indexed evidence and the inventory is declared complete",
                        clause.clause_id
                    ));
                    STATUS_MISSING
                } else {
                    clause_diagnostics.push(format!(
                        "clause `{}` has no indexed evidence and the inventory is incomplete",
                        clause.clause_id
                    ));
                    STATUS_INCOMPLETE
                }
            } else {
                clause_diagnostics.push(format!(
                    "clause `{}` has evidence records but none satisfy the declared source/policy/tool bindings",
                    clause.clause_id
                ));
                STATUS_STALE_BINDING
            }
        } else if matched.iter().any(|record_id| {
            index.records.iter().any(|record| record.record_id == *record_id && record_has_failure(record))
        }) {
            clause_diagnostics.push(format!("clause `{}` has a failed indexed evidence record", clause.clause_id));
            STATUS_FAILED
        } else if matched.iter().any(|record_id| {
            index.records.iter().any(|record| {
                record.record_id == *record_id
                    && (record.caveats.iter().any(|caveat| caveat.contains("unsupported"))
                        || record.artifact_family.contains("unsupported"))
            })
        }) {
            clause_diagnostics.push(format!("clause `{}` has unsupported indexed evidence", clause.clause_id));
            STATUS_UNSUPPORTED
        } else {
            STATUS_CURRENT
        };
        if related.truncated || matches!(snapshot.completeness, SnapshotCompleteness::Undeclared) {
            status = STATUS_INCOMPLETE;
            clause_diagnostics
                .push("freshness coverage is incomplete because the snapshot or traversal is partial".to_owned());
        }
        diagnostics
            .extend(clause_diagnostics.iter().map(|diagnostic| format!("clause `{}`: {diagnostic}", clause.clause_id)));
        clauses.push(ClauseFreshness {
            clause_id: clause.clause_id.clone(),
            required: clause.required,
            status: status.to_owned(),
            expected_bindings: clause.required_bindings.clone(),
            matched_record_ids: matched,
            actual_bindings: actual,
            related_records: related.records,
            diagnostics: clause_diagnostics,
        });
    }
    let required_clauses = clauses.iter().filter(|clause| clause.required).collect::<Vec<_>>();
    let aggregate_result = if required_clauses.is_empty() {
        diagnostics.push("required inventory is empty".to_owned());
        STATUS_FAILED
    } else if required_clauses.iter().any(|clause| clause.status != STATUS_CURRENT) {
        if required_clauses.iter().any(|clause| clause.status == STATUS_INCOMPLETE) {
            STATUS_INCOMPLETE
        } else {
            STATUS_FAILED
        }
    } else {
        STATUS_CURRENT
    }
    .to_owned();
    diagnostics.sort();
    diagnostics.dedup();
    Ok(FreshnessReport {
        schema_version: FRESHNESS_SCHEMA_VERSION.to_owned(),
        contract_id: contract.contract_id.clone(),
        contract_blake3,
        index_hash: index.index_hash.clone(),
        index_schema_version: index.schema_version.clone(),
        snapshot_id: snapshot.snapshot_id.clone(),
        completeness: completeness_label,
        aggregate_result,
        clauses,
        diagnostics,
        non_claims: non_claims(contract),
    })
}

fn record_matches_clause(record: &EvidenceIndexRecord, clause: &RequirementClause) -> bool {
    record.requirement_ids.iter().any(|id| normalized_requirement_id(id) == clause.clause_id)
        || clause
            .allowed_evidence_ids
            .iter()
            .any(|id| record.stable_id == *id || record.record_id == *id || record.summary.contains(id))
}

fn normalized_requirement_id(value: &str) -> &str {
    value.strip_prefix("r[").and_then(|value| value.strip_suffix(']')).unwrap_or(value)
}

fn record_satisfies_bindings(record: &EvidenceIndexRecord, clause: &RequirementClause) -> bool {
    let bindings = &clause.required_bindings;
    let source_ok = bindings.source_ids.iter().all(|expected| record.source_ids.contains(expected));
    let policy_ok = bindings.policy_ids.iter().all(|expected| record.policy_ids.contains(expected));
    let tool_ok = bindings
        .tool_identity_hashes
        .iter()
        .all(|expected| record.artifact_hashes.contains(expected) || record.source_ids.contains(expected));
    let artifact_ok = bindings.artifact_hashes.iter().all(|expected| record.artifact_hashes.contains(expected));
    let tier_ok = bindings.evidence_tier.as_ref().is_none_or(|tier| &record.evidence_tier == tier);
    let scope_ok =
        bindings.checked_scope.is_empty() || bindings.checked_scope.iter().any(|scope| record.summary.contains(scope));
    source_ok && policy_ok && tool_ok && artifact_ok && tier_ok && scope_ok
}

fn record_has_failure(record: &EvidenceIndexRecord) -> bool {
    let text = format!("{} {}", record.summary, record.caveats.join(" ")).to_ascii_lowercase();
    text.contains("status=failed")
        || text.contains("status: failed")
        || text.contains("status=fail")
        || text.contains("status: fail")
        || text.contains("rejected")
        || text.contains("invalid")
        || text.contains("missing required")
}

fn actual_bindings(record: &EvidenceIndexRecord) -> ActualBindings {
    ActualBindings {
        record_id: record.record_id.clone(),
        stable_id: record.stable_id.clone(),
        evidence_tier: record.evidence_tier.clone(),
        source_ids: record.source_ids.clone(),
        policy_ids: record.policy_ids.clone(),
        artifact_hashes: record.artifact_hashes.clone(),
        requirement_ids: record.requirement_ids.clone(),
    }
}

struct RelatedRecords {
    records: Vec<RelatedRecord>,
    truncated: bool,
}

fn related_records(snapshot: &EvidenceSnapshot, clause_id: &str) -> RelatedRecords {
    let request = ReachabilityRequest::new(clause_id)
        .with_admitted_relations(BTreeSet::from([
            RelationKind::ParentLink,
            RelationKind::ProvenanceSource,
            RelationKind::SharedRequirement,
            RelationKind::SharedPolicy,
            RelationKind::ContentIdentity,
        ]))
        .with_limits(QueryLimits {
            max_depth: MAX_FRESHNESS_DEPTH,
            max_output: MAX_RELATED_RECORDS,
            ..QueryLimits::default()
        });
    let report = evaluate_typed_reference(snapshot, &request);
    let unknown_start = report
        .gaps
        .iter()
        .any(|gap| matches!(gap, crate::evidence_multihop::SnapshotGap::UnknownStart { .. }));
    let mut records = report
        .reached
        .iter()
        .filter(|reached| reached.stable_id != clause_id)
        .take(MAX_RELATED_RECORDS)
        .map(|reached| {
            let (relation, semantics) = match &reached.reason {
                crate::evidence_multihop::ReachReason::Origin => ("origin".to_owned(), "origin".to_owned()),
                crate::evidence_multihop::ReachReason::Typed {
                    relation, semantics, ..
                } => (format!("{relation:?}").to_ascii_lowercase(), format!("{semantics:?}").to_ascii_lowercase()),
            };
            RelatedRecord {
                record_id: reached.record_id.clone(),
                stable_id: reached.stable_id.clone(),
                depth: reached.depth,
                relation,
                semantics,
            }
        })
        .collect::<Vec<_>>();
    records.sort_by(|left, right| left.depth.cmp(&right.depth).then_with(|| left.stable_id.cmp(&right.stable_id)));
    let truncated = !unknown_start
        && (report.status != crate::evidence_multihop::AnalysisStatus::Complete
            || report.output_truncated
            || report.reached.len() > MAX_RELATED_RECORDS);
    RelatedRecords { records, truncated }
}

fn print_freshness_human(report: &FreshnessReport) {
    println!(
        "octet requirements freshness contract={} status={} clauses={} snapshot={} completeness={} diagnostics={}",
        report.contract_id,
        report.aggregate_result,
        report.clauses.len(),
        report.snapshot_id,
        report.completeness,
        report.diagnostics.len()
    );
    for clause in &report.clauses {
        println!(
            "clause={} required={} status={} matched=[{}] related={}",
            clause.clause_id,
            clause.required,
            clause.status,
            clause.matched_record_ids.join(","),
            clause.related_records.len()
        );
        for diagnostic in &clause.diagnostics {
            println!("  clause-diagnostic: {diagnostic}");
        }
        for related in &clause.related_records {
            println!(
                "  related={} stable_id={} depth={} relation={} semantics={}",
                related.record_id, related.stable_id, related.depth, related.relation, related.semantics
            );
        }
    }
    for diagnostic in &report.diagnostics {
        println!("diagnostic: {diagnostic}");
    }
    for non_claim in &report.non_claims {
        println!("non-claim: {non_claim}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SPEC_PATH: &str = "spec.md";
    const SOURCE_PATH: &str = "src/lib.rs";
    const PROOF_DECK: &str = "proof-cards.json";
    const PROOF_CARD_ID: &str = "clause.add";

    fn fixture_root() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }

    fn write_file(root: &Path, locator: &str, bytes: &[u8]) -> SourceBinding {
        let path = root.join(locator);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent");
        }
        fs::write(&path, bytes).expect("write fixture file");
        SourceBinding {
            locator: locator.to_owned(),
            blake3: hash_bytes(bytes),
        }
    }

    fn contract(source_binding: SourceBinding, spec_hash: String, disposition: &str) -> RequirementContract {
        RequirementContract {
            schema_version: CONTRACT_SCHEMA_VERSION.to_owned(),
            contract_id: "octet.requirements.test".to_owned(),
            reviewed_specification: ReviewedSpecification {
                path: SPEC_PATH.to_owned(),
                blake3: spec_hash,
            },
            source_root: None,
            clauses: vec![RequirementClause {
                clause_id: "clause.add".to_owned(),
                statement: "addition is checked".to_owned(),
                required: true,
                disposition: disposition.to_owned(),
                checked_scope: vec!["formal-proof:fixture".to_owned()],
                expected_evidence_tier: DISPOSITION_FORMAL_PROOF.to_owned(),
                allowed_evidence_ids: vec![PROOF_CARD_ID.to_owned()],
                allowed_source_requirement_ids: vec!["clause.add".to_owned()],
                required_evidence_classes: vec!["proof_card".to_owned()],
                required_negative_controls: Vec::new(),
                required_bindings: RequirementBindings::default(),
                source_bindings: vec![source_binding],
                assumption_id: None,
                exclusion_reason: None,
                required_provider_id: None,
            }],
            assumptions: Vec::new(),
            exclusions: Vec::new(),
            non_claims: Vec::new(),
        }
    }

    fn proof_deck(root: &Path, status: &str) -> ProofDeckEvidence {
        let artifact = "proof.receipt";
        let artifact_bytes = b"{\"status\":\"passed\"}";
        let artifact_hash = hash_bytes(artifact_bytes);
        fs::write(root.join(artifact), artifact_bytes).expect("write receipt");
        let deck = serde_json::json!({
            "schema_version": PROOF_CARDS_SCHEMA_VERSION,
            "cards": [{
                "proof_or_property_id": PROOF_CARD_ID,
                "source_requirement_or_policy_id": "clause.add",
                "checked_rust_source_scope": "formal-proof:fixture",
                "evidence_tier": "formal_proof",
                "status": status,
                "non_claims": ["not whole-program correctness"],
                "diagnostics": [],
                "artifact_blake3_identities": [{"artifact": artifact, "blake3": artifact_hash}],
            }],
        });
        fs::write(root.join(PROOF_DECK), serde_json::to_vec_pretty(&deck).expect("serialize deck"))
            .expect("write deck");
        ProofDeckEvidence {
            evidence_id: PROOF_CARD_ID.to_owned(),
            deck: PROOF_DECK.to_owned(),
            artifact_root: ".".to_owned(),
        }
    }

    fn evidence(contract_blake3: String, source_binding: SourceBinding) -> RequirementEvidence {
        RequirementEvidence {
            schema_version: EVIDENCE_SCHEMA_VERSION.to_owned(),
            contract_id: "octet.requirements.test".to_owned(),
            contract_blake3: Some(contract_blake3),
            source_snapshot: vec![source_binding],
            proof_decks: Vec::new(),
            empirical_checks: Vec::new(),
            assumption_records: Vec::new(),
            non_claims: Vec::new(),
        }
    }

    #[test]
    fn valid_formal_clause_passes_and_recomputes() {
        let root = fixture_root();
        let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
        let source = write_file(root.path(), SOURCE_PATH, b"pub fn add(a: u8, b: u8) -> u8 { a + b }");
        let mut contract = contract(source.clone(), spec.blake3.clone(), DISPOSITION_FORMAL_PROOF);
        contract.clauses[0].allowed_source_requirement_ids = vec!["clause.add".to_owned()];
        let mut evidence = evidence(String::new(), source.clone());
        evidence.proof_decks = vec![proof_deck(root.path(), "passed")];
        let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
        evidence.contract_blake3 = Some(hash_bytes(&contract_bytes));
        let decision = evaluate_contract(
            &contract,
            &evidence,
            &contract_bytes,
            &serde_json::to_vec(&evidence).expect("serialize evidence"),
            root.path(),
        )
        .expect("evaluate");
        assert_eq!(decision.aggregate_result, STATUS_PASSED, "{:?}", decision.diagnostics);
        assert_eq!(decision.clauses[0].status, STATUS_PASSED);
        let mut recorded = decision.clone();
        recorded.decision_blake3 = String::new();
        recorded.decision_blake3 = decision_hash(&recorded).expect("hash decision");
        let mut diagnostics = Vec::new();
        compare_decisions(&recorded, &decision, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn missing_evidence_fails_closed() {
        let root = fixture_root();
        let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
        let source = write_file(root.path(), SOURCE_PATH, b"source");
        let contract = contract(source.clone(), spec.blake3.clone(), DISPOSITION_FORMAL_PROOF);
        let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
        let mut evidence = evidence(hash_bytes(&contract_bytes), source);
        evidence.proof_decks = vec![proof_deck(root.path(), "failed")];
        let decision =
            evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
        assert_eq!(decision.aggregate_result, STATUS_FAILED);
        assert_eq!(decision.clauses[0].status, STATUS_MISSING);
    }

    #[test]
    fn changed_source_and_spec_fail_closed() {
        let root = fixture_root();
        let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
        let source = write_file(root.path(), SOURCE_PATH, b"source");
        let contract = contract(source, spec.blake3, DISPOSITION_FORMAL_PROOF);
        let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
        fs::write(root.path().join(SPEC_PATH), b"changed spec").expect("change spec");
        fs::write(root.path().join(SOURCE_PATH), b"changed source").expect("change source");
        let evidence = evidence(hash_bytes(&contract_bytes), SourceBinding {
            locator: SOURCE_PATH.to_owned(),
            blake3: hash_bytes(b"source"),
        });
        let decision =
            evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
        assert_eq!(decision.aggregate_result, STATUS_FAILED);
        assert!(decision.diagnostics.iter().any(|diagnostic| diagnostic.contains("reviewed specification")));
        assert!(decision.diagnostics.iter().any(|diagnostic| diagnostic.contains("source")));
    }

    #[test]
    fn empirical_evidence_cannot_satisfy_formal_clause() {
        let root = fixture_root();
        let spec = write_file(root.path(), SPEC_PATH, b"spec");
        let source = write_file(root.path(), SOURCE_PATH, b"source");
        let mut contract = contract(source, spec.blake3, DISPOSITION_EMPIRICAL_CHECK);
        contract.clauses[0].required_evidence_classes = vec!["formal_proof".to_owned()];
        let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
        let mut evidence = evidence(hash_bytes(&contract_bytes), SourceBinding {
            locator: SOURCE_PATH.to_owned(),
            blake3: hash_bytes(b"source"),
        });
        evidence.empirical_checks.push(EmpiricalCheckEvidence {
            evidence_id: PROOF_CARD_ID.to_owned(),
            evidence_class: DISPOSITION_EMPIRICAL_CHECK.to_owned(),
            status: CHECK_STATUS_PASSED.to_owned(),
            checker_id: "test".to_owned(),
            command_argv: vec!["cargo".to_owned(), "test".to_owned()],
            checked_scope: vec!["test".to_owned()],
            source_bindings: vec![SourceBinding {
                locator: SOURCE_PATH.to_owned(),
                blake3: hash_bytes(b"source"),
            }],
            artifact: Some("check.txt".to_owned()),
            artifact_blake3: Some(hash_bytes(b"check output")),
            diagnostics: Vec::new(),
            non_claims: Vec::new(),
        });
        fs::write(root.path().join("check.txt"), b"check output").expect("write check artifact");
        let decision =
            evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
        assert_eq!(decision.aggregate_result, STATUS_FAILED, "{:?}", decision.diagnostics);
        assert!(decision.diagnostics.iter().any(|diagnostic| diagnostic.contains("formal-proof")));
    }

    #[test]
    fn duplicate_clause_ids_and_empty_required_inventory_fail() {
        let mut contract = contract(
            SourceBinding {
                locator: SOURCE_PATH.to_owned(),
                blake3: "0".repeat(64),
            },
            "0".repeat(64),
            DISPOSITION_FORMAL_PROOF,
        );
        contract.clauses.push(contract.clauses[0].clone());
        assert!(validate_contract(&contract).iter().any(|diagnostic| diagnostic.contains("duplicate")));
        contract.clauses.truncate(1);
        contract.clauses[0].required = false;
        let diagnostics = validate_contract(&contract);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("empty required inventory")));
    }

    #[test]
    fn stale_binding_and_missing_freshness_are_distinct() {
        let index = EvidenceCodebaseIndex {
            schema_version: "octet-evidence-codebase-index/v1".to_owned(),
            normalization_version: "v1".to_owned(),
            index_hash: "index-hash".to_owned(),
            boundary: "identity and linkage only".to_owned(),
            source_artifact_count: 1,
            record_count: 1,
            records: vec![EvidenceIndexRecord {
                record_id: "record".to_owned(),
                artifact_family: "octet-proof-card".to_owned(),
                stable_id: "clause.add".to_owned(),
                artifact_locator: "proof-cards.json".to_owned(),
                artifact_blake3: "hash".to_owned(),
                evidence_tier: "formal_proof".to_owned(),
                source_ids: vec!["old-source".to_owned()],
                requirement_ids: vec!["clause.add".to_owned()],
                lint_ids: Vec::new(),
                policy_ids: vec!["old-policy".to_owned()],
                fingerprints: Vec::new(),
                artifact_hashes: Vec::new(),
                provenance_source_ids: Vec::new(),
                replay_commands: Vec::new(),
                parent_links: Vec::new(),
                caveats: Vec::new(),
                summary: "passed".to_owned(),
            }],
            issues: Vec::new(),
        };
        let mut contract = contract(
            SourceBinding {
                locator: SOURCE_PATH.to_owned(),
                blake3: "0".repeat(64),
            },
            "0".repeat(64),
            DISPOSITION_FORMAL_PROOF,
        );
        contract.clauses[0].required_bindings = RequirementBindings {
            source_ids: vec!["new-source".to_owned()],
            policy_ids: vec!["new-policy".to_owned()],
            ..RequirementBindings::default()
        };
        let report = evaluate_freshness(&contract, &index, b"contract").expect("freshness");
        assert_eq!(report.clauses[0].status, STATUS_STALE_BINDING);
        let mut index = index;
        index.records.clear();
        let report = evaluate_freshness(&contract, &index, b"contract").expect("freshness");
        assert_eq!(report.clauses[0].status, STATUS_MISSING);
    }

    #[test]
    fn checked_example_contract_passes_and_is_recomputable() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root");
        let contract_path = root.join("standards/requirements/example-contract.json");
        let evidence_path = root.join("standards/requirements/fixtures/example/evidence.json");
        let contract: RequirementContract = read_json_bounded(&contract_path).expect("read example contract");
        let evidence: RequirementEvidence = read_json_bounded(&evidence_path).expect("read example evidence");
        let contract_bytes = fs::read(&contract_path).expect("read contract bytes");
        let evidence_bytes = fs::read(&evidence_path).expect("read evidence bytes");
        let decision = evaluate_contract(&contract, &evidence, &contract_bytes, &evidence_bytes, root)
            .expect("evaluate example contract");
        assert_eq!(decision.aggregate_result, STATUS_PASSED, "{:?}", decision.diagnostics);
        assert_eq!(decision.clauses[0].status, STATUS_PASSED);
        assert_eq!(decision.clauses[1].status, STATUS_ASSUMPTION_ACCEPTED);
        assert_eq!(decision.clauses[2].status, STATUS_EXCLUDED);
        assert!(decision.assumptions.iter().all(|assumption| assumption.status == STATUS_ASSUMPTION_ACCEPTED));
        let mut recorded = decision.clone();
        recorded.decision_blake3 = String::new();
        recorded.decision_blake3 = decision_hash(&recorded).expect("hash decision");
        let mut diagnostics = Vec::new();
        compare_decisions(&recorded, &decision, &mut diagnostics);
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
    }

    #[test]
    fn checked_example_contract_freshness_reports_current_obligations() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root");
        let contract_path = root.join("standards/requirements/example-contract.json");
        let contract: RequirementContract = read_json_bounded(&contract_path).expect("read example contract");
        let contract_bytes = fs::read(&contract_path).expect("read contract bytes");
        let proof_cards = root.join("standards/requirements/fixtures/example/proof-cards.json");
        let finding = root.join("standards/requirements/fixtures/example/index/check-finding.json");
        let inputs = vec![
            crate::valence_core_shim::evidence_codebase_index::EvidenceArtifactInput {
                locator: "standards/requirements/fixtures/example/proof-cards.json".to_owned(),
                bytes: fs::read(&proof_cards).expect("read proof cards"),
            },
            crate::valence_core_shim::evidence_codebase_index::EvidenceArtifactInput {
                locator: "standards/requirements/fixtures/example/index/check-finding.json".to_owned(),
                bytes: fs::read(&finding).expect("read finding"),
            },
        ];
        let index = crate::valence_core_shim::evidence_codebase_index::build_evidence_codebase_index(
            &inputs,
            crate::valence_core_shim::evidence_codebase_index::EvidenceIndexBuildMode::Strict,
        );
        assert!(index.issues.is_empty(), "{:?}", index.issues);
        let report = evaluate_freshness(&contract, &index, &contract_bytes).expect("freshness");
        assert_eq!(report.aggregate_result, STATUS_CURRENT, "{:?}", report.diagnostics);
        assert_eq!(report.clauses[0].status, STATUS_CURRENT);
        assert_eq!(report.clauses[1].status, STATUS_CURRENT);
        assert_eq!(report.clauses[2].status, STATUS_EXCLUDED);
        assert!(report.clauses[0].actual_bindings.iter().any(|binding| binding.stable_id.contains("safe_add")));
        assert!(report.clauses[1].actual_bindings.iter().any(|binding| binding.stable_id.contains("lean_toolchain")));
    }

    #[test]
    fn tampered_decision_bytes_are_rejected() {
        let root = fixture_root();
        let spec = write_file(root.path(), SPEC_PATH, b"reviewed specification");
        let source = write_file(root.path(), SOURCE_PATH, b"source");
        let contract = contract(source.clone(), spec.blake3, DISPOSITION_FORMAL_PROOF);
        let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
        let mut evidence = evidence(hash_bytes(&contract_bytes), source);
        evidence.proof_decks = vec![proof_deck(root.path(), "passed")];
        let decision =
            evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
        let mut tampered = decision.clone();
        tampered.aggregate_result = STATUS_FAILED.to_owned();
        let mut diagnostics = Vec::new();
        compare_decisions(&tampered, &decision, &mut diagnostics);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded decision bytes are stale")));
        let mut hash_tampered = decision.clone();
        hash_tampered.decision_blake3 = "0".repeat(64);
        let mut diagnostics = Vec::new();
        compare_decisions(&hash_tampered, &decision, &mut diagnostics);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("does not match the recomputed")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded decision bytes are stale")));
    }

    #[test]
    fn changed_assumption_identity_cannot_inherit_acceptance() {
        let root = fixture_root();
        let spec = write_file(root.path(), SPEC_PATH, b"spec");
        let source = write_file(root.path(), SOURCE_PATH, b"source");
        let mut contract = contract(source.clone(), spec.blake3.clone(), DISPOSITION_ACCEPTED_ASSUMPTION);
        contract.clauses[0].assumption_id = Some("assumption.test".to_owned());
        contract.assumptions.push(AcceptedAssumption {
            assumption_id: "assumption.test".to_owned(),
            statement: "fixture assumption".to_owned(),
            identity: "identity-a".to_owned(),
            trusted_boundary_identity: "boundary-a".to_owned(),
            affected_clause_ids: vec![contract.clauses[0].clause_id.clone()],
            required_empirical_evidence: vec!["check.test".to_owned()],
        });
        let contract_bytes = serde_json::to_vec(&contract).expect("serialize contract");
        let mut evidence = evidence(hash_bytes(&contract_bytes), source.clone());
        evidence.empirical_checks.push(EmpiricalCheckEvidence {
            evidence_id: "check.test".to_owned(),
            evidence_class: DISPOSITION_EMPIRICAL_CHECK.to_owned(),
            status: CHECK_STATUS_PASSED.to_owned(),
            checker_id: "test".to_owned(),
            command_argv: vec!["cargo".to_owned(), "test".to_owned()],
            checked_scope: vec!["test".to_owned()],
            source_bindings: vec![source],
            artifact: Some("check.txt".to_owned()),
            artifact_blake3: Some(hash_bytes(b"check")),
            diagnostics: Vec::new(),
            non_claims: Vec::new(),
        });
        fs::write(root.path().join("check.txt"), b"check").expect("write artifact");
        evidence.assumption_records.push(AssumptionEvidence {
            assumption_id: "assumption.test".to_owned(),
            identity: "identity-b".to_owned(),
            trusted_boundary_identity: "boundary-a".to_owned(),
            status: ASSUMPTION_STATUS_ACCEPTED.to_owned(),
            supporting_evidence_ids: vec!["check.test".to_owned()],
            diagnostics: Vec::new(),
        });
        let decision =
            evaluate_contract(&contract, &evidence, &contract_bytes, b"evidence", root.path()).expect("evaluate");
        assert_eq!(decision.aggregate_result, STATUS_FAILED);
        assert!(decision.diagnostics.iter().any(|diagnostic| diagnostic.contains("identity changed")));
    }

    #[test]
    fn required_exclusion_does_not_pass() {
        let mut contract = contract(
            SourceBinding {
                locator: SOURCE_PATH.to_owned(),
                blake3: "0".repeat(64),
            },
            "0".repeat(64),
            DISPOSITION_EXCLUDED,
        );
        contract.clauses[0].exclusion_reason = Some("outside scope".to_owned());
        contract.exclusions.push(ExcludedBehavior {
            exclusion_id: "exclusion.test".to_owned(),
            clause_id: contract.clauses[0].clause_id.clone(),
            reason: "outside scope".to_owned(),
            reviewed_specification_blake3: String::new(),
        });
        let mut diagnostics = Vec::new();
        let decision = evaluate_clause(
            &contract,
            &RequirementEvidence {
                schema_version: EVIDENCE_SCHEMA_VERSION.to_owned(),
                contract_id: contract.contract_id.clone(),
                contract_blake3: None,
                source_snapshot: Vec::new(),
                proof_decks: Vec::new(),
                empirical_checks: Vec::new(),
                assumption_records: Vec::new(),
                non_claims: Vec::new(),
            },
            &contract.clauses[0],
            Path::new("."),
            &BTreeMap::new(),
            &mut diagnostics,
        );
        assert_eq!(decision.status, STATUS_EXCLUDED);
        assert!(!decision.diagnostics.is_empty());
    }
}
