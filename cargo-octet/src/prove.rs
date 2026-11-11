use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fmt::Write as _;
#[cfg(test)]
use std::fs;
use std::path::Path;
use std::path::PathBuf;

use octet_standards::aeneas_observation::MeasuredLeanFile;
use octet_standards::aeneas_observation::observe_aeneas_lean_trust_closure;
use octet_standards::aeneas_observation::observe_charon_extraction_policy;
use octet_standards::aeneas_observation::validate_observed_aeneas_translation;
use octet_standards::artifacts::POLICIES_JSON_FILE;
use octet_standards::formal_proof::AENEAS_LEAN_PROVIDER_ID;
use octet_standards::formal_proof::AENEAS_NAMED_THEOREM_CLAIM_CLASS;
use octet_standards::formal_proof::AENEAS_NON_CLAIM_TRANSLATION_SOUNDNESS;
use octet_standards::formal_proof::AENEAS_NON_CLAIM_VERIFIER_SOUNDNESS;
use octet_standards::formal_proof::AENEAS_NON_CLAIM_WHOLE_PROGRAM;
use octet_standards::formal_proof::AENEAS_STAGE_CHAIN_SCHEMA_VERSION;
use octet_standards::formal_proof::AeneasAssumptionPolicy;
use octet_standards::formal_proof::AeneasLeanCheckResult;
use octet_standards::formal_proof::AeneasLeanCheckStatus;
use octet_standards::formal_proof::AeneasLeanProofPayload;
use octet_standards::formal_proof::AeneasPreflight;
use octet_standards::formal_proof::AeneasPreflightDisposition;
use octet_standards::formal_proof::AeneasStage;
use octet_standards::formal_proof::AeneasStageArtifactIdentity;
use octet_standards::formal_proof::AeneasStageArtifactRole;
use octet_standards::formal_proof::AeneasStageChain;
use octet_standards::formal_proof::AeneasStageKind;
use octet_standards::formal_proof::AeneasTheoremAdmission;
use octet_standards::formal_proof::AeneasTheoremSourceDisposition;
use octet_standards::formal_proof::FormalProofArtifactIdentity;
use octet_standards::formal_proof::FormalProofClaim;
use octet_standards::formal_proof::FormalProofEnvelope;
use octet_standards::formal_proof::FormalProofEvidenceTier;
use octet_standards::formal_proof::FormalProofProvider;
use octet_standards::formal_proof::FormalProofStatus;
use octet_standards::formal_proof::SemanticReference;
use octet_standards::formal_proof::assess_aeneas_theorem_source;
use octet_standards::formal_proof::validate_aeneas_theorem_statement_approval;
use octet_standards::formal_proof::validate_formal_proof_envelope;
use octet_standards::model::FamilySelection;
use octet_standards::model::PolicyEntry;
use octet_standards::model::PolicyRegistry;
use octet_standards::model::ProofRail;
use octet_standards::proof::DIAGNOSTIC_UNPINNED_VERIFIER;
use octet_standards::proof::PROOF_STATUS_FAILED;
use octet_standards::proof::PROOF_STATUS_PASSED;
use octet_standards::proof::VERIFIER_IDENTITY_FLAG;
use octet_standards::proof::VERIFIER_MODE_FIXTURE;
use octet_standards::proof::VERIFIER_MODE_PRODUCTION;
use octet_standards::proof::VerusLogArtifact;
use octet_standards::proof::VerusProofReceipt;
use octet_standards::proof::VerusProofReceiptInput;
use octet_standards::proof::build_verus_proof_receipt;
use octet_standards::proof::rail_requires_pinned_verifier_identity;
use octet_standards::proof::validate_proof_receipt_with_identity;
use octet_standards::proof::verifier_identity_report_diagnostics;
use octet_standards::proof_artifact::normalize_path;
use octet_standards::proof_artifact::path_matches_pattern;
use octet_standards::proof_artifact::path_pattern_diagnostics;
use octet_standards::proof_artifact::receipt_paths as proof_artifact_receipt_paths;
use octet_standards::source::SourceFile;
use octet_standards::vermilion_canary::CANARY_SOURCE;
use octet_standards::vermilion_canary::CANARY_SOURCE_PATH;
use octet_standards::vermilion_canary::VermilionCanaryReceipt;
use octet_standards::vermilion_canary::validate as validate_vermilion_canary;
use serde::Deserialize;
use serde::Serialize;

use crate::aeneas_execution::AENEAS_LEAN_BACKEND;
use crate::aeneas_execution::AeneasCommandSummary;
use crate::aeneas_execution::execute_aeneas_from_environment;
use crate::cap_fs::ArtifactRoot;
use crate::cap_fs::ProofRoot;
use crate::cap_fs::RootFile;
use crate::cap_fs::RootRelativePath;
use crate::cap_fs::SourceRoot;
use crate::cap_fs::WorkspaceRoot;
use crate::cli::OutputFormat;
use crate::cli::ProveArgs;
use crate::effect_ports::ProcessCommandPlanInput;
use crate::effect_ports::build_process_command_plan;
use crate::evidence::EvidenceTier;
use crate::evidence::ProvenanceRecord;
use crate::imported_proof::EVIDENCE_ORIGIN_EXTERNAL_TRELLIS;
use crate::imported_proof::HASH_ALGORITHM_BLAKE3;
use crate::imported_proof::ImportedProofBundle;
use crate::imported_proof::ImportedProofBundleManifest;
use crate::imported_proof::bundle_artifacts;
use crate::imported_proof::validate_imported_proof_manifest_shape;
use crate::proof_goal_state::GoalArtifactRequirement;
use crate::proof_goal_state::GoalEvidence;
use crate::proof_goal_state::ProofGoalInput;
use crate::proof_goal_state::ProofGoalState;
use crate::proof_goal_state::build_proof_goal_state;
use crate::prove_core;

const REPORT_SCHEMA_VERSION: u32 = 1;
const EXIT_PROOF_VALID: i32 = 0;
const EXIT_PROOF_INVALID: i32 = 1;
const COMMAND_ARTIFACT_NAME: &str = "command.txt";
const STATUS_ARTIFACT_NAME: &str = "status.json";
const SUMMARY_ARTIFACT_NAME: &str = "summary.txt";
const PROVENANCE_ARTIFACT_NAME: &str = "provenance.jsonl";
const PROOF_CARDS_ARTIFACT_NAME: &str = "proof-cards.json";
const PROOF_ARTIFACT_DIR_NAME: &str = "proofs";
/// Every top-level entry a prove bundle can hold; receipts, logs, and Aeneas stage artifacts all
/// live under `proofs/`. Reusing a bundle removes only these, so entries `prove` never writes (a
/// `proof-ref.json`, an operator's notes) survive.
const PROVE_BUNDLE_ENTRY_NAMES: &[&str] = &[
    COMMAND_ARTIFACT_NAME,
    STATUS_ARTIFACT_NAME,
    SUMMARY_ARTIFACT_NAME,
    PROVENANCE_ARTIFACT_NAME,
    PROOF_CARDS_ARTIFACT_NAME,
    crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT,
    PROOF_ARTIFACT_DIR_NAME,
];
const AENEAS_ARTIFACT_DIR_SUFFIX: &str = "aeneas";
const AENEAS_LLBC_ARTIFACT: &str = "llbc.llbc";
const AENEAS_TRANSLATION_ARTIFACT: &str = "translation.json";
const AENEAS_GENERATED_LEAN_ARTIFACT: &str = "Llbc.lean";
/// Axioms Lean reports for the approved theorem; the receipt's trust closure is derived from it.
const AENEAS_LEAN_AXIOM_REPORT_ARTIFACT: &str = "lean-axioms.json";
const STATUS_PASSED: &str = "passed";
const STATUS_FAILED: &str = "failed";
const STATUS_MANUAL_REVIEW_REQUIRED: &str = "manual_review_required";
const STATUS_SCHEMA_VERSION: &str = "octet-prove-status/v1";
const PROOF_CARDS_SCHEMA_VERSION: &str = "octet-proof-cards/v1";
const BUNDLE_KIND_PROVE: &str = "prove";
const PROVE_PROFILE_NAME: &str = "prove-verification";
const PROOF_SOURCE_ID: &str = "octet.standards.formal-proof";
const RUNNER_SOURCE_ID: &str = "octet.runner.artifact-bundle";
const CHECKER_OCTET_STANDARDS: &str = "octet-standards";
const CHECKER_CARGO_OCTET: &str = "cargo-octet";
const MANUAL_ACTION: &str = "review cargo octet prove command, status, and scoped proof receipt before release";
const PROOF_CARD_MANUAL_ACTION: &str =
    "review proof cards with matching receipt, provenance, and cargo octet artifact verify output before release";
const ARTIFACT_VERIFY_COMMAND: &str = "cargo octet artifact verify --artifact-dir <artifact-dir>";
const WITHHELD_FORMAL_PROOF_ACTION: &str = "formal_proof provenance withheld because the proof goal is blocked; resolve these blockers and rerun cargo octet prove";
const WITHHELD_FORMAL_PROOF_DIAGNOSTIC: &str = "proof goal blocked; formal_proof provenance withheld";
const TRELLIS_IMPORT_PROVENANCE_ARTIFACT: &str = "standards/proof-imports/generated/trellis-proof-imports.json";
const TRELLIS_IMPORTS_JSON: &str = include_str!("../../standards/proof-imports/generated/trellis-proof-imports.json");
const AENEAS_CONSUMER_POLICY_ID: &str = "octet.aeneas_consumer_contract.external-fixture";
const EVIDENCE_ORIGIN_OCTET_RERUN: &str = "octet_rerun";
const EVIDENCE_ORIGIN_VERMILION_CANARY: &str = "recorded-vermilion-canary";
const VERMILION_CANARY_BACKEND: &str = "vermilion-canary";
const VERMILION_CANARY_SOURCE_ARTIFACT: &str = "proofs/vermilion-canary/checked.rs";
const VERIFIER_MODE_EXTERNAL: &str = "external";
/// Aeneas card non-claim recorded beside the shared `octet_standards` Aeneas non-claims.
pub(crate) const AENEAS_NON_CLAIM_SEMANTIC_EQUIVALENCE: &str =
    "does not establish semantic equivalence or release eligibility";
const NON_CLAIM_WHOLE_PROGRAM: &str = "not whole-program correctness";
const NON_CLAIM_CERTIFICATION: &str = "not certification";
const NON_CLAIM_BUSINESS_DOMAIN: &str = "not business-domain validation";
const NON_CLAIM_SEMANTIC_EQUIVALENCE: &str = "not semantic equivalence";
const NON_CLAIM_FULL_SAFETY: &str = "not full safety";
const NON_CLAIM_BLAKE3_IDENTITY: &str =
    "BLAKE3 identifies exact artifact bytes and does not by itself prove proof correctness";
const FIELD_PROOF_OR_PROPERTY_ID: &str = "proof_or_property_id";
const FIELD_SOURCE_REQUIREMENT_OR_POLICY_ID: &str = "source_requirement_or_policy_id";
const FIELD_CHECKED_RUST_SOURCE_SCOPE: &str = "checked_rust_source_scope";
const FIELD_SOURCE_PATHS: &str = "source_paths";
const FIELD_PROOF_TARGET_PATHS: &str = "proof_target_paths";
const FIELD_VERIFIER_MODE: &str = "verifier_mode";
const FIELD_VERIFIER_IDENTITY_SUMMARY: &str = "verifier_identity_summary";
const FIELD_SOLVER_IDENTITY_SUMMARY: &str = "solver_identity_summary";
const FIELD_TOOLCHAIN_IDENTITY_SUMMARY: &str = "toolchain_identity_summary";
const FIELD_RECEIPT_PATH: &str = "receipt_path";
const FIELD_ARTIFACT_BLAKE3_IDENTITIES: &str = "artifact_blake3_identities";
const FIELD_EVIDENCE_TIER: &str = "evidence_tier";
const FIELD_EVIDENCE_ORIGIN: &str = "evidence_origin";
const FIELD_STATUS: &str = "status";
const FIELD_TRUST_BOUNDARIES: &str = "trust_boundaries";
const FIELD_NON_CLAIMS: &str = "non_claims";
const FIELD_ARTIFACT_VERIFIER_COMMAND: &str = "artifact_verifier_command";
const FIELD_PROVENANCE_ARTIFACT: &str = "provenance_artifact";
const FIELD_EXTERNAL_REFERENCE: &str = "external_reference";
const SAFE_SEPARATOR: char = '-';
const CARGO_MANIFEST_FILE: &str = "Cargo.toml";
const DEFAULT_PROOF_TIMEOUT_MS: u64 = 30_000;
const DEFAULT_LOG_MAX_BYTES: usize = 4_096;
const PROCESS_POLL_INTERVAL_MS: u64 = 10;
const PROCESS_TEARDOWN_TIMEOUT_MS: u64 = 5_000;
const PROCESS_STDIN_MAX_BYTES: usize = 1;
const VERIFIER_IDENTITY_MAX_BYTES: usize = 4_096;
const PROCESS_SUCCESS_EXIT_CODE: i32 = 0;
#[cfg(unix)]
const EXECUTABLE_MODE_BITS: u32 = 0o111;
const PATH_ENVIRONMENT_NAME: &str = "PATH";
const COMMAND_STDOUT_SUFFIX: &str = "stdout";
const COMMAND_STDERR_SUFFIX: &str = "stderr";
const COMMAND_ARGV_FIELD: &str = "command_argv";
const DEFAULT_ENV_ALLOWLIST: &[&str] = &["PATH", "TMPDIR"];
const DIAGNOSTIC_VERIFIER_FAILURE: &str = "verifier-failure";
const EVIDENCE_LOG_CAPTURE_STDOUT: &str = "octet.prove.stdout";
const EVIDENCE_LOG_CAPTURE_STDERR: &str = "octet.prove.stderr";
const EVIDENCE_LOG_CLAIM_BOUNDARY: &str =
    "scoped proof rail output only; not whole-program correctness, certification, semantic equivalence, or full safety";

#[derive(Debug, Serialize)]
struct ProveReport {
    schema_version: u32,
    status: &'static str,
    artifact_dir: String,
    proofs: Vec<ProofReportRow>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct ProofReportRow {
    policy_id: String,
    rail_index: usize,
    backend: String,
    verifier_mode: String,
    checked_scope: String,
    receipt: String,
    status: &'static str,
    diagnostics: Vec<String>,
}

#[derive(Debug, Serialize)]
struct RailListReport {
    schema_version: u32,
    selected_rails: Vec<RailListRow>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProofCardDeck {
    schema_version: String,
    cards: Vec<ProofCard>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProofCardArtifactIdentity {
    artifact: String,
    blake3: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProofCardExternalReference {
    repository_label: String,
    repository_revision: String,
    hash_algorithm: String,
    promotion_command_argv: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct ProofCard {
    proof_or_property_id: String,
    source_requirement_or_policy_id: String,
    policy_id: String,
    rail_index: usize,
    checked_rust_source_scope: String,
    source_paths: Vec<String>,
    proof_target_paths: Vec<String>,
    verifier_mode: String,
    verifier_identity_summary: String,
    solver_identity_summary: String,
    toolchain_identity_summary: String,
    receipt_path: String,
    artifact_blake3_identities: Vec<ProofCardArtifactIdentity>,
    evidence_tier: String,
    evidence_origin: String,
    status: String,
    trust_boundaries: Vec<String>,
    non_claims: Vec<String>,
    artifact_verifier_command: String,
    provenance_artifact: String,
    external_reference: Option<ProofCardExternalReference>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    formal_proof_envelope: Option<FormalProofEnvelope>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    negative_check: Option<crate::proof_ref::ProofCardNegativeCheck>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
struct RailListRow {
    policy_id: String,
    rail_index: usize,
    backend: String,
    source_paths: Vec<String>,
    proof_paths: Vec<String>,
    receipt_paths: Vec<String>,
    verifier_label: String,
    verifier_mode: String,
    checked_scope: String,
    non_claim: String,
}

#[derive(Debug, Serialize)]
struct StatusArtifact {
    schema_version: &'static str,
    bundle_kind: &'static str,
    status: &'static str,
    output_format: &'static str,
    metadata: StatusMetadata,
    proofs: Vec<ProofReportRow>,
}

#[derive(Debug, Serialize)]
struct StatusMetadata {
    profile_name: &'static str,
}

struct SelectedRail<'a> {
    policy: &'a PolicyEntry,
    rail_index: usize,
    rail: &'a ProofRail,
}

struct ExecutedProof {
    row: ProofReportRow,
    receipt_text: String,
    /// Receipt provenance for a rail whose verifier and receipt checks passed: `formal_proof`, or
    /// `manual_review_required` once a blocked goal state withholds the formal claim.
    formal_provenance: Option<ProvenanceRecord>,
    logs: Vec<CapturedProofLog>,
    extra_artifacts: Vec<GeneratedProofArtifact>,
}

#[derive(Clone, Debug)]
struct GeneratedProofArtifact {
    locator: String,
    contents: Vec<u8>,
}

struct CapturedProofLog {
    locator: String,
    text: String,
    checked_scope: String,
    capture_source: &'static str,
}

struct CommandSpec {
    executable: String,
    args: Vec<String>,
    argv: Vec<String>,
    timeout_ms: u64,
    environment_allowlist: Vec<String>,
}

#[allow(clippy::struct_excessive_bools)]
struct CommandOutcome {
    command_succeeded: bool,
    timed_out: bool,
    stdout_text: String,
    stdout_original_bytes: usize,
    stdout_truncated: bool,
    stderr_text: String,
    stderr_original_bytes: usize,
    stderr_truncated: bool,
    command_blake3: String,
    diagnostics: Vec<String>,
    aeneas_summary: Option<AeneasCommandSummary>,
    canary_receipt: Option<String>,
}

pub fn run(args: &ProveArgs) -> Result<i32, String> {
    if let Some(negative_check) = &args.negative_check {
        return crate::proof_candidates::run_record_negative_check_with_evidence(
            negative_check,
            negative_check.verifier_evidence.as_deref(),
            args.output_format,
        );
    }
    if args.candidates {
        return crate::proof_candidates::run(args);
    }
    let registry = load_registry(args)?;
    let working_dir = proof_root(&args.source)?;
    let sources = load_sources(&args.source, &working_dir)?;
    let selection = prove_selection(&args.family);
    let rail_selection = prove_core::selected_rail_indices(&registry, &sources, &selection, &args.rail);
    let selected_rails = selected_rails(&registry, &rail_selection.selected);
    if args.list {
        let selector_failed = !rail_selection.diagnostics.is_empty();
        let mut diagnostics = rail_selection.diagnostics;
        diagnostics.extend(list_diagnostics(&selected_rails, &sources));
        print_rail_list(args.output_format, &rail_list_report(&selected_rails, diagnostics))?;
        return Ok(if selector_failed {
            EXIT_PROOF_INVALID
        } else {
            EXIT_PROOF_VALID
        });
    }

    ensure_artifact_dir_reusable(&args.artifact_dir, &working_dir)?;
    let mut diagnostics = rail_selection.diagnostics;
    if selected_rails.is_empty() {
        diagnostics.push("no selected proof rails matched the provided sources".to_owned());
    }
    let mut executions = if diagnostics.is_empty() {
        selected_rails
            .iter()
            .map(|selected| execute_proof(selected, &sources, &working_dir, args.explain))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };
    let goal_states = settle_proof_goals(args, &mut executions);
    let report = prove_report(args, diagnostics, &executions);
    write_artifacts(args, &report, &executions, &goal_states)?;
    print_report(args.output_format, &report)?;

    Ok(if report.diagnostics.is_empty() {
        EXIT_PROOF_VALID
    } else {
        EXIT_PROOF_INVALID
    })
}

fn prove_report(args: &ProveArgs, mut diagnostics: Vec<String>, executions: &[ExecutedProof]) -> ProveReport {
    diagnostics.extend(executions.iter().flat_map(|execution| execution.row.diagnostics.clone()));
    ProveReport {
        schema_version: REPORT_SCHEMA_VERSION,
        status: if diagnostics.is_empty() {
            STATUS_PASSED
        } else {
            STATUS_FAILED
        },
        artifact_dir: args.artifact_dir.display().to_string(),
        proofs: executions.iter().map(|execution| execution.row.clone()).collect(),
        diagnostics,
    }
}

/// Computes each rail's goal state and withholds `formal_proof` provenance from blocked rails, so
/// a blocked claim never reaches the bundle as `formal_proof`.
fn settle_proof_goals(args: &ProveArgs, executions: &mut [ExecutedProof]) -> Vec<ProofGoalState> {
    executions
        .iter_mut()
        .filter(|execution| execution.row.backend != VERMILION_CANARY_BACKEND)
        .map(|execution| {
            let goal_state = proof_goal_state_for_execution(args, execution);
            if !goal_state.emits_formal_proof {
                withhold_formal_proof(execution, &goal_state.blockers);
            }
            goal_state
        })
        .collect()
}

/// Downgrades a blocked rail's receipt provenance to `manual_review_required`, still binding the
/// receipt bytes the run writes, and marks the rail for manual review with the blockers as
/// diagnostics. Rails that never earned provenance (verifier or receipt failures) are unchanged.
fn withhold_formal_proof(execution: &mut ExecutedProof, blockers: &[String]) {
    let Some(record) = execution.formal_provenance.as_mut() else {
        return;
    };
    record.evidence_tier = EvidenceTier::ManualReviewRequired;
    record.blake3 = crate::evidence::artifact_blake3_hex(execution.receipt_text.as_bytes());
    record.human_action = Some(format!("{WITHHELD_FORMAL_PROOF_ACTION}: {}", blockers.join("; ")));
    execution.row.status = STATUS_MANUAL_REVIEW_REQUIRED;
    execution
        .row
        .diagnostics
        .extend(blockers.iter().map(|blocker| format!("{WITHHELD_FORMAL_PROOF_DIAGNOSTIC}: {blocker}")));
    execution.row.diagnostics.sort();
    execution.row.diagnostics.dedup();
}

fn load_registry(args: &ProveArgs) -> Result<PolicyRegistry, String> {
    let registry_path = if let Some(registry) = &args.registry {
        registry.clone()
    } else {
        args.policy_artifacts_dir
            .as_ref()
            .expect("parser requires a registry source")
            .join(POLICIES_JSON_FILE)
    };
    let text = read_workspace_file(&registry_path)?;
    serde_json::from_str(&text)
        .map_err(|error| format!("parse {} as policy registry JSON: {error}", registry_path.display()))
}

/// Loads every `--source` path under `root`. Overlapping arguments, such as a directory and a file
/// inside it, name the same locator more than once; each locator is kept once so rail patterns
/// match the file, not its duplicates.
fn load_sources(paths: &[PathBuf], root: &Path) -> Result<Vec<SourceFile>, String> {
    let source_root = SourceRoot::open_existing(root)?;
    let mut sources = Vec::new();
    for path in paths {
        append_sources(path, root, &source_root, &mut sources)?;
    }
    let mut seen = BTreeSet::new();
    sources.retain(|source| seen.insert(source.path.clone()));
    Ok(sources)
}

fn append_sources(
    path: &Path,
    root: &Path,
    source_root: &SourceRoot,
    sources: &mut Vec<SourceFile>,
) -> Result<(), String> {
    let relative = source_relative_path(path, root)?;
    if source_root.is_dir(&relative)? {
        append_directory_sources(source_root, &relative, sources)
    } else {
        append_file_source(source_root, &relative, sources)
    }
}

fn append_directory_sources(
    source_root: &SourceRoot,
    relative: &RootRelativePath,
    sources: &mut Vec<SourceFile>,
) -> Result<(), String> {
    for file in source_root.read_files_recursive_from(relative)? {
        sources.push(source_file_from_root_file(file)?);
    }
    Ok(())
}

fn append_file_source(
    source_root: &SourceRoot,
    relative: &RootRelativePath,
    sources: &mut Vec<SourceFile>,
) -> Result<(), String> {
    let text = source_root.read_to_string(relative)?;
    sources.push(SourceFile::new(relative.display(), text));
    Ok(())
}

fn source_file_from_root_file(file: RootFile) -> Result<SourceFile, String> {
    let text = String::from_utf8(file.contents)
        .map_err(|error| format!("read source path `{}` as UTF-8: {error}", file.locator))?;
    Ok(SourceFile::new(file.locator, text))
}

/// Locates a `--source` argument under the proof root. The root itself is a valid source; a path
/// outside it is rejected instead of being reinterpreted relative to the root.
fn source_relative_path(path: &Path, root: &Path) -> Result<RootRelativePath, String> {
    let canonical = path.canonicalize().map_err(|error| format!("resolve proof source {}: {error}", path.display()))?;
    let relative = canonical
        .strip_prefix(root)
        .map_err(|_| format!("proof source {} is outside the proof root {}", path.display(), root.display()))?;
    if relative.as_os_str().is_empty() {
        Ok(RootRelativePath::current())
    } else {
        RootRelativePath::new(relative)
    }
}

fn read_workspace_file(path: &Path) -> Result<String, String> {
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let file_name = path.file_name().ok_or_else(|| format!("read {}: path must name a file", path.display()))?;
    let root = WorkspaceRoot::open_existing(parent)?;
    root.read_to_string(&RootRelativePath::new(Path::new(file_name))?)
}

fn prove_selection(families: &[String]) -> FamilySelection {
    if families.is_empty() {
        FamilySelection::from_families(["verification"])
    } else {
        FamilySelection::from_families(families.iter().cloned())
    }
}

fn selected_rails<'a>(
    registry: &'a PolicyRegistry,
    selected: &[prove_core::SelectedRailIndex],
) -> Vec<SelectedRail<'a>> {
    selected
        .iter()
        .map(|selected| {
            let policy = &registry.policies[selected.policy_index];
            SelectedRail {
                policy,
                rail_index: selected.rail_index,
                rail: &policy.proof_rails[selected.rail_index],
            }
        })
        .collect()
}

// Proof execution loads sources, runs the command, and binds logs and receipt into one
// ExecutedProof in sequence.
#[allow(clippy::too_many_lines)]
fn execute_proof(
    selected: &SelectedRail<'_>,
    sources: &[SourceFile],
    working_dir: &Path,
    explain: bool,
) -> ExecutedProof {
    let receipt_locator = proof_artifact_locator(selected.policy, selected.rail_index);
    let stdout_locator = proof_log_artifact_locator(selected.policy, selected.rail_index, COMMAND_STDOUT_SUFFIX);
    let stderr_locator = proof_log_artifact_locator(selected.policy, selected.rail_index, COMMAND_STDERR_SUFFIX);
    let checked_scope = checked_scope(selected.policy, selected.rail_index, selected.rail);
    let mut diagnostics = path_pattern_diagnostics(&selected.rail.source_paths, sources, "source_paths");
    let (proof_sources, proof_source_diagnostics) = load_proof_sources(selected.rail, working_dir);
    diagnostics.extend(proof_source_diagnostics);
    let outcome = run_proof_command(selected.rail, working_dir);
    diagnostics.extend(outcome.diagnostics.clone());
    diagnostics.extend(proof_failure_diagnostics(selected.rail, &outcome));

    let stdout_artifact =
        log_artifact(&stdout_locator, &outcome.stdout_text, outcome.stdout_original_bytes, outcome.stdout_truncated);
    let stderr_artifact =
        log_artifact(&stderr_locator, &outcome.stderr_text, outcome.stderr_original_bytes, outcome.stderr_truncated);
    let logs = vec![
        captured_proof_log(&stdout_locator, &outcome.stdout_text, &checked_scope, EVIDENCE_LOG_CAPTURE_STDOUT),
        captured_proof_log(&stderr_locator, &outcome.stderr_text, &checked_scope, EVIDENCE_LOG_CAPTURE_STDERR),
    ];
    let mut extra_artifacts = Vec::new();
    if selected.rail.backend == VERMILION_CANARY_BACKEND && diagnostics.is_empty() {
        extra_artifacts.push(GeneratedProofArtifact {
            locator: VERMILION_CANARY_SOURCE_ARTIFACT.to_owned(),
            contents: CANARY_SOURCE.as_bytes().to_vec(),
        });
    }
    let receipt_text = if selected.rail.backend == VERMILION_CANARY_BACKEND {
        outcome
            .canary_receipt
            .clone()
            .unwrap_or_else(|| failed_receipt_text(&selected.policy.policy_id, selected.rail_index, &diagnostics))
    } else if selected.rail.backend == AENEAS_LEAN_BACKEND {
        match outcome.aeneas_summary.as_ref().and_then(
            |summary| {
                if diagnostics.is_empty() { Some(summary) } else { None }
            },
        ) {
            Some(summary) => match build_aeneas_receipt(selected, sources, &proof_sources, &checked_scope, summary) {
                Ok((receipt, artifacts)) => {
                    extra_artifacts = artifacts;
                    receipt
                }
                Err(error) => {
                    diagnostics.push(error);
                    failed_receipt_text(&selected.policy.policy_id, selected.rail_index, &diagnostics)
                }
            },
            None => failed_receipt_text(&selected.policy.policy_id, selected.rail_index, &diagnostics),
        }
    } else {
        let command_status = if diagnostics.is_empty() {
            PROOF_STATUS_PASSED
        } else {
            PROOF_STATUS_FAILED
        };
        match build_verus_proof_receipt(&VerusProofReceiptInput {
            policy_id: &selected.policy.policy_id,
            rail_index: selected.rail_index,
            rail: selected.rail,
            sources,
            proof_sources: &proof_sources,
            checked_scope: &checked_scope,
            status: command_status,
            command_blake3: &outcome.command_blake3,
            proof_artifact_identity: &receipt_locator,
            stdout_artifact,
            stderr_artifact,
        }) {
            Ok(receipt) => serde_json::to_string_pretty(&receipt).map_or_else(
                |error| format!("{{\"schema_version\":\"receipt-serialization-error\",\"error\":{error:?}}}\n"),
                |text| format!("{text}\n"),
            ),
            Err(errors) => {
                diagnostics.extend(errors);
                failed_receipt_text(&selected.policy.policy_id, selected.rail_index, &diagnostics)
            }
        }
    };

    if selected.rail.backend == AENEAS_LEAN_BACKEND {
        match serde_json::from_str::<FormalProofEnvelope>(&receipt_text) {
            Ok(envelope) => diagnostics.extend(validate_formal_proof_envelope(&envelope)),
            Err(_) if diagnostics.is_empty() => {
                diagnostics.push("Aeneas receipt did not contain a formal-proof envelope".to_owned());
            }
            Err(_) => {}
        }
    } else if selected.rail.backend != VERMILION_CANARY_BACKEND {
        let proof_source = SourceFile::new(receipt_locator.clone(), receipt_text.clone());
        let mut validation_sources = sources.to_vec();
        // A source scope that contains the proof files (for example the workspace root) already
        // loaded them; adding them again would make each proof path match twice.
        validation_sources
            .extend(proof_sources.into_iter().filter(|proof| !sources.iter().any(|source| source.path == proof.path)));
        diagnostics.extend(validate_proof_receipt_with_identity(
            selected.rail,
            Some(&selected.policy.policy_id),
            Some(selected.rail_index),
            &validation_sources,
            &proof_source,
        ));
    }
    if explain {
        diagnostics = diagnostics.into_iter().map(|diagnostic| format!("explain: {diagnostic}")).collect();
    }
    diagnostics.sort();
    diagnostics.dedup();

    let status = if diagnostics.is_empty() {
        STATUS_PASSED
    } else {
        STATUS_FAILED
    };
    if status != STATUS_PASSED {
        extra_artifacts.clear();
    }
    let formal_provenance = diagnostics.is_empty().then(|| {
        let mut record = proof_provenance_record(
            &receipt_locator,
            receipt_text.as_bytes(),
            &checked_scope,
            &selected.rail.receipt_verifier.verifier_id,
        );
        if selected.rail.backend == VERMILION_CANARY_BACKEND {
            record.evidence_tier = EvidenceTier::ManualReviewRequired;
            record.human_action = Some(
                "recorded-only measured Vermilion canary; review before any release use; not a formal proof".to_owned(),
            );
        }
        record
    });
    ExecutedProof {
        row: ProofReportRow {
            policy_id: selected.policy.policy_id.clone(),
            rail_index: selected.rail_index,
            backend: selected.rail.backend.clone(),
            verifier_mode: verifier_mode(selected.rail).to_owned(),
            checked_scope,
            receipt: receipt_locator,
            status,
            diagnostics,
        },
        receipt_text,
        formal_provenance,
        logs,
        extra_artifacts,
    }
}

#[derive(Debug, Deserialize)]
struct AeneasReceiptProfile {
    aeneas_release: String,
    charon_revision: String,
    ocaml_toolchain: String,
    lean_toolchain: String,
    aeneas_lean_library: String,
    checked_source_surface: Vec<String>,
    assumption_policy: AeneasAssumptionPolicy,
    semantic_reference: AeneasReceiptSemanticReference,
}

#[derive(Debug, Deserialize)]
struct AeneasReceiptSemanticReference {
    revision: String,
    license: String,
    source_sentinel: String,
    recorded_only: bool,
}

struct AeneasArtifactCollector {
    artifacts: Vec<GeneratedProofArtifact>,
}

impl AeneasArtifactCollector {
    fn new() -> Self {
        Self { artifacts: Vec::new() }
    }

    fn add(&mut self, locator: &str, contents: Vec<u8>) -> AeneasStageArtifactIdentity {
        let blake3 = crate::evidence::artifact_blake3_hex(&contents);
        self.artifacts.push(GeneratedProofArtifact {
            locator: locator.to_owned(),
            contents,
        });
        AeneasStageArtifactIdentity {
            role: AeneasStageArtifactRole::ProviderProfile,
            locator: locator.to_owned(),
            blake3,
        }
    }

    fn add_with_role(
        &mut self,
        role: AeneasStageArtifactRole,
        locator: &str,
        contents: Vec<u8>,
    ) -> AeneasStageArtifactIdentity {
        let mut identity = self.add(locator, contents);
        identity.role = role;
        identity
    }
}

// The Aeneas receipt binds provider, source, theorem, and artifact identities into one envelope
// from the same parsed profile and summary.
#[allow(clippy::too_many_lines)]
fn build_aeneas_receipt(
    selected: &SelectedRail<'_>,
    sources: &[SourceFile],
    proof_sources: &[SourceFile],
    checked_scope: &str,
    summary: &AeneasCommandSummary,
) -> Result<(String, Vec<GeneratedProofArtifact>), String> {
    let profile = serde_json::from_slice::<AeneasReceiptProfile>(&summary.provider_profile)
        .map_err(|error| format!("parse embedded Aeneas receipt profile: {error}"))?;
    let effective_policy = summary
        .effective_policy
        .clone()
        .ok_or_else(|| "Aeneas execution did not return its enforced execution policy".to_owned())?;
    let llbc_text = observed_aeneas_text(summary, AENEAS_LLBC_ARTIFACT)?;
    let translation_text = observed_aeneas_text(summary, AENEAS_TRANSLATION_ARTIFACT)?;
    let generated_lean_text = observed_aeneas_text(summary, AENEAS_GENERATED_LEAN_ARTIFACT)?;
    let extraction_policy = observe_charon_extraction_policy(&llbc_text)
        .map_err(|diagnostics| format!("observe Charon extraction policy: {}", diagnostics.join("; ")))?;
    let translation_diagnostics =
        validate_observed_aeneas_translation(&llbc_text, &translation_text, &[MeasuredLeanFile {
            locator: AENEAS_GENERATED_LEAN_ARTIFACT,
            contents: generated_lean_text.as_bytes(),
        }]);
    if !translation_diagnostics.is_empty() {
        return Err(format!("Aeneas translation manifest validation failed: {}", translation_diagnostics.join("; ")));
    }
    let source_path = selected
        .rail
        .source_paths
        .first()
        .ok_or_else(|| "Aeneas rail must select one Rust source path".to_owned())?;
    let source = sources
        .iter()
        .find(|source| source.path == *source_path)
        .ok_or_else(|| format!("Aeneas source binding `{source_path}` was not loaded"))?;
    let theorem_source =
        proof_sources.first().ok_or_else(|| "Aeneas rail must select one theorem source".to_owned())?;
    let theorem_id = if selected.rail.theorem_id.trim().is_empty() {
        return Err("Aeneas rail must declare theorem_id".to_owned());
    } else {
        selected.rail.theorem_id.clone()
    };
    let theorem_name = extract_theorem_name(&theorem_source.text)
        .ok_or_else(|| "Aeneas theorem source does not contain a named theorem".to_owned())?;
    let mut collector = AeneasArtifactCollector::new();
    let locator = |name: &str| aeneas_artifact_locator(selected.policy, selected.rail_index, name);
    let profile_identity = collector.add_with_role(
        AeneasStageArtifactRole::ProviderProfile,
        &locator("provider-profile.json"),
        summary.provider_profile.clone(),
    );
    let package_lock_contents = observed_aeneas_artifact(summary, "lake-manifest.json")?;
    let package_lock_identity = collector.add_with_role(
        AeneasStageArtifactRole::PackageLock,
        &locator("package-lock.json"),
        package_lock_contents,
    );
    let source_identity = collector.add_with_role(
        AeneasStageArtifactRole::RustSource,
        &locator(&format!("source/{source_path}")),
        source.text.as_bytes().to_vec(),
    );
    let charon_options = json_bytes(&extraction_policy)?;
    let charon_options_identity = collector.add_with_role(
        AeneasStageArtifactRole::CharonOptions,
        &locator("charon-options.json"),
        charon_options,
    );
    let charon_command_identity = collector.add_with_role(
        AeneasStageArtifactRole::CharonCommand,
        &locator("charon-command.json"),
        json_bytes(summary.stage_argvs.first().ok_or_else(|| "Aeneas receipt is missing Charon argv".to_owned())?)?,
    );
    let llbc_identity = collector.add_with_role(
        AeneasStageArtifactRole::Llbc,
        &locator(AENEAS_LLBC_ARTIFACT),
        observed_aeneas_artifact(summary, "llbc.llbc")?,
    );
    let charon_outputs = vec![charon_options_identity, charon_command_identity, llbc_identity];
    let aeneas_command_identity = collector.add_with_role(
        AeneasStageArtifactRole::AeneasCommand,
        &locator("aeneas-command.json"),
        json_bytes(summary.stage_argvs.get(1).ok_or_else(|| "Aeneas receipt is missing Aeneas argv".to_owned())?)?,
    );
    let translation_identity = collector.add_with_role(
        AeneasStageArtifactRole::TranslationManifest,
        &locator(AENEAS_TRANSLATION_ARTIFACT),
        observed_aeneas_artifact(summary, "translation.json")?,
    );
    let aeneas_outputs = vec![aeneas_command_identity, translation_identity];
    let generated_lean_identity = collector.add_with_role(
        AeneasStageArtifactRole::GeneratedLean,
        &locator(AENEAS_GENERATED_LEAN_ARTIFACT),
        observed_aeneas_artifact(summary, "Llbc.lean")?,
    );
    let external_models_identity = collector.add_with_role(
        AeneasStageArtifactRole::ExternalModelInventory,
        &locator("external-models.json"),
        json_bytes(&profile.assumption_policy.external_models)?,
    );
    let generated_outputs = vec![generated_lean_identity, external_models_identity];
    let theorem_source_identity = collector.add_with_role(
        AeneasStageArtifactRole::TheoremSource,
        &locator("theorem.lean"),
        theorem_source.text.as_bytes().to_vec(),
    );
    let theorem_statement_identity = collector.add_with_role(
        AeneasStageArtifactRole::TheoremStatement,
        &locator("theorem-statement.lean"),
        theorem_source.text.as_bytes().to_vec(),
    );
    let approved_statement_blake3 = validate_aeneas_theorem_statement_approval(
        &selected.rail.approved_theorem_statement_blake3,
        &theorem_statement_identity.blake3,
    )?;
    let theorem_adequacy = assess_aeneas_theorem_source(&theorem_source.text);
    let theorem_adequacy_identity = collector.add_with_role(
        AeneasStageArtifactRole::TheoremAdequacy,
        &locator("theorem-adequacy.json"),
        json_bytes(&theorem_adequacy)?,
    );
    let lean_axiom_report = observed_aeneas_text(summary, AENEAS_LEAN_AXIOM_REPORT_ARTIFACT)?;
    let trust_closure = observe_aeneas_lean_trust_closure(
        &theorem_source.text,
        &generated_lean_text,
        &theorem_name,
        &lean_axiom_report,
        &profile.assumption_policy,
    )?;
    let trust_closure_identity = collector.add_with_role(
        AeneasStageArtifactRole::TrustClosure,
        &locator("trust-closure.json"),
        json_bytes(&trust_closure)?,
    );
    let theorem_outputs = vec![
        theorem_source_identity.clone(),
        theorem_statement_identity.clone(),
        theorem_adequacy_identity,
        trust_closure_identity,
    ];
    let lean_toolchain_identity = collector.add_with_role(
        AeneasStageArtifactRole::LeanToolchain,
        &locator("lean-toolchain"),
        observed_aeneas_artifact(summary, "lean-toolchain")?,
    );
    let library_identity = collector.add_with_role(
        AeneasStageArtifactRole::AeneasLeanLibrary,
        &locator("aeneas-lean-library.json"),
        json_bytes(&profile.aeneas_lean_library)?,
    );
    let lean_command_identity = collector.add_with_role(
        AeneasStageArtifactRole::LeanCommand,
        &locator("lean-command.json"),
        json_bytes(summary.stage_argvs.get(2).ok_or_else(|| "Aeneas receipt is missing Lean argv".to_owned())?)?,
    );
    let checker_result = AeneasLeanCheckResult {
        theorem_id: theorem_id.clone(),
        status: AeneasLeanCheckStatus::Passed,
    };
    let checker_identity = collector.add_with_role(
        AeneasStageArtifactRole::CheckerResult,
        &locator("checker-result.json"),
        json_bytes(&checker_result)?,
    );
    let lean_outputs = vec![
        lean_toolchain_identity,
        library_identity,
        lean_command_identity,
        checker_identity,
    ];
    let stage_chain = AeneasStageChain {
        schema_version: AENEAS_STAGE_CHAIN_SCHEMA_VERSION.to_owned(),
        provider_profile: profile_identity,
        package_lock: package_lock_identity,
        stages: vec![
            AeneasStage {
                kind: AeneasStageKind::RustSource,
                input_artifacts: Vec::new(),
                output_artifacts: vec![source_identity.clone()],
            },
            AeneasStage {
                kind: AeneasStageKind::CharonExtraction,
                input_artifacts: vec![source_identity.clone()],
                output_artifacts: charon_outputs.clone(),
            },
            AeneasStage {
                kind: AeneasStageKind::AeneasTranslation,
                input_artifacts: charon_outputs.clone(),
                output_artifacts: aeneas_outputs.clone(),
            },
            AeneasStage {
                kind: AeneasStageKind::GeneratedLean,
                input_artifacts: aeneas_outputs.clone(),
                output_artifacts: generated_outputs.clone(),
            },
            AeneasStage {
                kind: AeneasStageKind::Theorem,
                input_artifacts: generated_outputs.clone(),
                output_artifacts: theorem_outputs.clone(),
            },
            AeneasStage {
                kind: AeneasStageKind::LeanCheck,
                input_artifacts: theorem_outputs.clone(),
                output_artifacts: lean_outputs,
            },
        ],
    };
    let consumer = build_aeneas_consumer_binding(selected, source, theorem_source, &collector)?;
    let requirement_id = consumer.as_ref().map_or_else(
        || "octet.aeneas_lean_proof_rail.theorem_adequacy".to_owned(),
        |binding| binding.theorem.requirement_id.clone(),
    );
    let claim_class = consumer
        .as_ref()
        .map_or_else(|| AENEAS_NAMED_THEOREM_CLAIM_CLASS.to_owned(), |binding| binding.theorem.claim_class.clone());
    let admission = AeneasTheoremAdmission {
        theorem_id: theorem_id.clone(),
        requirement_id: requirement_id.clone(),
        claim_class: claim_class.clone(),
        source_disposition: AeneasTheoremSourceDisposition::Maintained,
        approved_statement_blake3,
        named_theorem_present: !theorem_name.is_empty(),
        statement_approved: true,
        adequacy: theorem_adequacy.clone(),
        trust_closure: trust_closure.clone(),
        lean_check: checker_result,
    };
    let payload = AeneasLeanProofPayload {
        aeneas_release: profile.aeneas_release,
        charon_revision: profile.charon_revision,
        ocaml_toolchain: profile.ocaml_toolchain,
        lean_toolchain: profile.lean_toolchain,
        aeneas_lean_library: profile.aeneas_lean_library,
        checked_source_surface: profile.checked_source_surface,
        preflight: AeneasPreflight {
            unsupported_inputs: Vec::new(),
            disposition: AeneasPreflightDisposition::Ready,
        },
        command_argv: summary.stage_argvs.iter().flatten().cloned().collect(),
        limits: effective_policy.limits,
        environment_allowlist: effective_policy.environment_allowlist,
        extraction_policy,
        assumption_policy: profile.assumption_policy,
        theorem_admission: Some(admission),
        stage_chain,
        semantic_reference: SemanticReference {
            revision: profile.semantic_reference.revision,
            license: profile.semantic_reference.license,
            source_sentinel: profile.semantic_reference.source_sentinel,
            recorded_only: profile.semantic_reference.recorded_only,
        },
    };
    let mut non_claims = vec![
        AENEAS_NON_CLAIM_TRANSLATION_SOUNDNESS.to_owned(),
        AENEAS_NON_CLAIM_VERIFIER_SOUNDNESS.to_owned(),
        AENEAS_NON_CLAIM_WHOLE_PROGRAM.to_owned(),
        AENEAS_NON_CLAIM_SEMANTIC_EQUIVALENCE.to_owned(),
    ];
    if let Some(binding) = consumer.as_ref() {
        non_claims.extend(binding.non_claims.clone());
    }
    non_claims.sort();
    non_claims.dedup();
    let envelope = FormalProofEnvelope {
        schema_version: octet_standards::formal_proof::FORMAL_PROOF_ENVELOPE_SCHEMA_VERSION.to_owned(),
        provider_id: AENEAS_LEAN_PROVIDER_ID.to_owned(),
        policy_id: selected.policy.policy_id.clone(),
        rail_id: format!("{}#rail-{}", selected.policy.policy_id, selected.rail_index),
        checked_scope: checked_scope.to_owned(),
        status: FormalProofStatus::Passed,
        evidence_tier: FormalProofEvidenceTier::FormalProof,
        claim: FormalProofClaim {
            theorem_or_property_id: theorem_id,
            requirement_id,
            claim_class,
        },
        package_lock_identity: stage_chain_package_lock(&payload.stage_chain).blake3.clone(),
        source_bindings: vec![
            FormalProofArtifactIdentity {
                locator: source_identity.locator.clone(),
                blake3: source_identity.blake3.clone(),
            },
            FormalProofArtifactIdentity {
                locator: theorem_source_identity.locator.clone(),
                blake3: theorem_source_identity.blake3.clone(),
            },
        ],
        artifact_identities: collector
            .artifacts
            .iter()
            .map(|artifact| FormalProofArtifactIdentity {
                locator: artifact.locator.clone(),
                blake3: crate::evidence::artifact_blake3_hex(&artifact.contents),
            })
            .collect(),
        trust_boundaries: selected.rail.trust_boundaries.clone(),
        non_claims,
        consumer,
        provider: FormalProofProvider::AeneasLean(payload),
    };
    let diagnostics = validate_formal_proof_envelope(&envelope);
    if !diagnostics.is_empty() {
        return Err(format!("Aeneas formal receipt validation failed: {}", diagnostics.join("; ")));
    }
    let text =
        serde_json::to_string_pretty(&envelope).map_err(|error| format!("serialize Aeneas formal receipt: {error}"))?;
    Ok((format!("{text}\n"), collector.artifacts))
}

// r[impl octet.aeneas_consumer_contract.execution]
// r[impl octet.aeneas_consumer_contract.source_selection]
fn build_aeneas_consumer_binding(
    selected: &SelectedRail<'_>,
    source: &SourceFile,
    theorem_source: &SourceFile,
    collector: &AeneasArtifactCollector,
) -> Result<Option<octet_standards::aeneas_consumer::ConsumerReceiptBinding>, String> {
    let profile = octet_standards::aeneas_consumer::selected_profile()?;
    let theorem = profile
        .theorems
        .first()
        .ok_or_else(|| "Aeneas consumer profile has no theorem mapping".to_owned())?;
    if selected.policy.policy_id != AENEAS_CONSUMER_POLICY_ID || selected.rail.theorem_id != theorem.theorem_id {
        return Ok(None);
    }
    let source_paths = profile.source.paths.iter().map(|binding| binding.path.as_str()).collect::<Vec<_>>();
    let expected_statement = theorem.approved_statement_blake3.strip_prefix("b3:").unwrap_or_default();
    if selected.rail.backend != AENEAS_LEAN_BACKEND
        || selected.rail.source_paths.iter().map(String::as_str).collect::<Vec<_>>() != source_paths
        || selected.rail.proof_paths != [theorem.path.as_str()]
        || selected.rail.approved_theorem_statement_blake3 != expected_statement
        || selected.rail.timeout_ms != profile.resource_policy.timeout_ms
        || selected.rail.stdout_max_bytes != profile.resource_policy.stdout_max_bytes
        || selected.rail.stderr_max_bytes != profile.resource_policy.stderr_max_bytes
        || selected.rail.environment_allowlist != profile.resource_policy.environment_allowlist
    {
        return Err("selected Aeneas consumer rail does not match the checked consumer profile".to_owned());
    }
    let item = profile
        .source
        .selected_items
        .first()
        .ok_or_else(|| "Aeneas consumer profile has no selected item".to_owned())?;
    let locator = |name: &str| aeneas_artifact_locator(selected.policy, selected.rail_index, name);
    let llbc = generated_aeneas_artifact_text(collector, &locator(AENEAS_LLBC_ARTIFACT))?;
    let translation = generated_aeneas_artifact_text(collector, &locator(AENEAS_TRANSLATION_ARTIFACT))?;
    let generated = generated_aeneas_artifact_text(collector, &locator(AENEAS_GENERATED_LEAN_ARTIFACT))?;
    let input = octet_standards::aeneas_consumer::ObservationInput {
        source_path: &source.path,
        source_text: &source.text,
        selected_item: &item.qualified_name,
        theorem_path: &theorem_source.path,
        theorem_text: &theorem_source.text,
        llbc_text: llbc,
        translation_text: translation,
        generated_lean_text: generated,
    };
    let binding = octet_standards::aeneas_consumer::build_binding(
        &profile,
        octet_standards::aeneas_consumer::PROFILE_IDENTITY,
        &input,
        octet_standards::aeneas_consumer::ConsumerOutcome::Supported,
    )
    .map_err(|diagnostics| format!("Aeneas consumer binding failed: {}", diagnostics.join("; ")))?;
    let diagnostics = octet_standards::aeneas_consumer::validate_receipt(&binding);
    if diagnostics.is_empty() {
        Ok(Some(binding))
    } else {
        Err(format!("Aeneas consumer receipt validation failed: {}", diagnostics.join("; ")))
    }
}

fn generated_aeneas_artifact_text<'a>(
    collector: &'a AeneasArtifactCollector,
    locator: &str,
) -> Result<&'a str, String> {
    let artifact = collector
        .artifacts
        .iter()
        .find(|artifact| artifact.locator == locator)
        .ok_or_else(|| format!("Aeneas consumer artifact `{locator}` is missing"))?;
    std::str::from_utf8(&artifact.contents)
        .map_err(|error| format!("Aeneas consumer artifact `{locator}` is not UTF-8: {error}"))
}

fn stage_chain_package_lock(chain: &AeneasStageChain) -> &AeneasStageArtifactIdentity {
    &chain.package_lock
}

fn observed_aeneas_artifact(summary: &AeneasCommandSummary, locator: &str) -> Result<Vec<u8>, String> {
    summary
        .artifacts
        .iter()
        .find(|artifact| artifact.locator == locator)
        .map(|artifact| artifact.contents.clone())
        .ok_or_else(|| format!("Aeneas execution did not produce `{locator}`"))
}

fn observed_aeneas_text(summary: &AeneasCommandSummary, locator: &str) -> Result<String, String> {
    String::from_utf8(observed_aeneas_artifact(summary, locator)?)
        .map_err(|error| format!("Aeneas artifact `{locator}` is not UTF-8: {error}"))
}

fn json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(value)
        .map(|mut bytes| {
            bytes.push(b'\n');
            bytes
        })
        .map_err(|error| format!("serialize Aeneas receipt artifact: {error}"))
}

fn extract_theorem_name(contents: &str) -> Option<String> {
    octet_standards::formal_proof::aeneas_theorem_name(contents).map(str::to_owned)
}

fn proof_failure_diagnostics(rail: &ProofRail, outcome: &CommandOutcome) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if outcome.timed_out {
        diagnostics.push(format!("{DIAGNOSTIC_VERIFIER_FAILURE}: proof command `{}` timed out", command_display(rail)));
    }
    if !outcome.command_succeeded {
        diagnostics.push(format!(
            "{DIAGNOSTIC_VERIFIER_FAILURE}: proof command `{}` exited without success",
            command_display(rail)
        ));
    }
    for (stream, truncated, configured_limit) in [
        (COMMAND_STDOUT_SUFFIX, outcome.stdout_truncated, rail.stdout_max_bytes),
        (COMMAND_STDERR_SUFFIX, outcome.stderr_truncated, rail.stderr_max_bytes),
    ] {
        if truncated {
            diagnostics.push(format!(
                "{DIAGNOSTIC_VERIFIER_FAILURE}: proof command `{}` {stream} exceeded a bounded log limit of {} bytes",
                command_display(rail),
                configured_log_limit(configured_limit)
            ));
        }
    }
    if !outcome.stdout_text.contains(&rail.pass_signal) {
        diagnostics.push(format!("proof command stdout missing pass signal `{}`", rail.pass_signal));
    }
    diagnostics
}

fn load_proof_sources(rail: &ProofRail, working_dir: &Path) -> (Vec<SourceFile>, Vec<String>) {
    let mut sources = Vec::new();
    let mut diagnostics = Vec::new();
    let proof_root = match ProofRoot::open_existing(working_dir) {
        Ok(root) => root,
        Err(error) => return (sources, vec![error]),
    };
    for proof_path in &rail.proof_paths {
        if proof_path.contains('*') || proof_path.contains('?') {
            diagnostics.push(format!(
                "proof rail field `proof_paths` pattern `{proof_path}` must be passed as an explicit proof source for receipt construction"
            ));
            continue;
        }
        let relative = match RootRelativePath::new(proof_path.as_str()) {
            Ok(relative) => relative,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        match proof_root.read_to_string(&relative) {
            Ok(text) => sources.push(SourceFile::new(proof_path.clone(), text)),
            Err(error) => diagnostics.push(error),
        }
    }
    (sources, diagnostics)
}

/// Resolves the one directory that source locators, proof paths, and proof commands are relative
/// to. Each `--source` resolves to its Cargo workspace root (see `cargo_workspace_root`), and every
/// source must resolve to the same root, so argument order cannot change the result. Without
/// sources the current directory is resolved the same way.
fn proof_root(source_paths: &[PathBuf]) -> Result<PathBuf, String> {
    let current_directory = [PathBuf::from(".")];
    let starts = if source_paths.is_empty() {
        &current_directory[..]
    } else {
        source_paths
    };
    let mut roots = BTreeMap::<PathBuf, Vec<String>>::new();
    for path in starts {
        let canonical =
            path.canonicalize().map_err(|error| format!("resolve proof source {}: {error}", path.display()))?;
        let start = if canonical.is_dir() {
            canonical.as_path()
        } else {
            canonical.parent().unwrap_or(&canonical)
        };
        roots.entry(cargo_workspace_root(start)?).or_default().push(path.display().to_string());
    }
    let mut resolved = roots.keys();
    if let (Some(root), None) = (resolved.next(), resolved.next()) {
        return Ok(root.clone());
    }
    let found = roots
        .iter()
        .map(|(root, sources)| format!("`{}` for {}", root.display(), sources.join(", ")))
        .collect::<Vec<_>>();
    Err(format!("proof sources must share one Cargo workspace root; found {}", found.join("; ")))
}

/// Finds the Cargo workspace root for `start` the way Cargo does: the nearest package manifest
/// belongs to the nearest enclosing `[workspace]` that lists it as a member and does not exclude
/// it; a package outside every such workspace is its own root. Without any `Cargo.toml`, `start`
/// is the root.
fn cargo_workspace_root(start: &Path) -> Result<PathBuf, String> {
    let Some(package_dir) = start.ancestors().find(|directory| directory.join(CARGO_MANIFEST_FILE).is_file()) else {
        return Ok(start.to_path_buf());
    };
    for directory in package_dir.ancestors() {
        if !directory.join(CARGO_MANIFEST_FILE).is_file() {
            continue;
        }
        let manifest = WorkspaceRoot::open_existing(directory)?
            .read_to_string(&RootRelativePath::new(CARGO_MANIFEST_FILE)?)?
            .parse::<toml::Table>()
            .map_err(|error| format!("parse {}: {error}", directory.join(CARGO_MANIFEST_FILE).display()))?;
        let Some(workspace) = manifest.get("workspace").and_then(toml::Value::as_table) else {
            continue;
        };
        let is_member = package_dir
            .strip_prefix(directory)
            .is_ok_and(|package| workspace_lists_member(workspace, &package.to_string_lossy()));
        return Ok(if is_member { directory } else { package_dir }.to_path_buf());
    }
    Ok(package_dir.to_path_buf())
}

fn workspace_lists_member(workspace: &toml::Table, package: &str) -> bool {
    let package = normalize_path(package);
    if package.is_empty() {
        return true;
    }
    let entries = |key: &str| {
        workspace
            .get(key)
            .and_then(toml::Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(toml::Value::as_str)
    };
    let excluded = entries("exclude").map(normalize_path).any(|excluded| {
        package == excluded || package.strip_prefix(&excluded).is_some_and(|rest| rest.starts_with('/'))
    });
    !excluded && entries("members").any(|member| path_matches_pattern(member, &package))
}

fn run_proof_command(rail: &ProofRail, working_dir: &Path) -> CommandOutcome {
    if rail.backend == VERMILION_CANARY_BACKEND {
        return run_vermilion_canary(rail, working_dir);
    }
    if rail.backend == AENEAS_LEAN_BACKEND {
        let summary = execute_aeneas_from_environment(rail, working_dir);
        let summary_for_outcome = summary.clone();
        return CommandOutcome {
            command_succeeded: summary.command_succeeded,
            timed_out: summary.timed_out,
            stdout_text: summary.stdout_text,
            stdout_original_bytes: summary.stdout_original_bytes,
            stdout_truncated: summary.stdout_truncated,
            stderr_text: summary.stderr_text,
            stderr_original_bytes: summary.stderr_original_bytes,
            stderr_truncated: summary.stderr_truncated,
            command_blake3: summary.command_blake3,
            diagnostics: summary.diagnostics,
            aeneas_summary: Some(summary_for_outcome),
            canary_receipt: None,
        };
    }
    let spec = match proof_command_spec(rail) {
        Ok(spec) => spec,
        Err(diagnostic) => {
            return CommandOutcome {
                command_succeeded: false,
                timed_out: false,
                stdout_text: String::new(),
                stdout_original_bytes: 0,
                stdout_truncated: false,
                stderr_text: diagnostic.clone(),
                stderr_original_bytes: diagnostic.len(),
                stderr_truncated: false,
                command_blake3: command_blake3(&rail.command_argv),
                diagnostics: vec![diagnostic],
                aeneas_summary: None,
                canary_receipt: None,
            };
        }
    };
    let stdout_limit = configured_log_limit(rail.stdout_max_bytes);
    let stderr_limit = configured_log_limit(rail.stderr_max_bytes);
    let command_blake3 = command_blake3(&spec.argv);
    let output = resolve_command(&spec, working_dir).map_err(|error| vec![error]).and_then(|command| {
        if rail_requires_pinned_verifier_identity(rail) {
            let diagnostics = pinned_verifier_diagnostics(rail, &spec, &command);
            if !diagnostics.is_empty() {
                return Err(diagnostics);
            }
        }
        run_resolved(&command, &spec.argv.join(" "), &spec.args, spec.timeout_ms, stdout_limit, stderr_limit)
            .map_err(|error| vec![error])
    });
    match output {
        Ok(output) => {
            let bounded_stdout = bounded_utf8(output.stdout);
            let bounded_stderr = bounded_utf8(output.stderr);
            CommandOutcome {
                command_succeeded: output.disposition == bounded_exec::Disposition::Succeeded,
                timed_out: output.completion == bounded_exec::Completion::TimedOut,
                stdout_text: bounded_stdout.text,
                stdout_original_bytes: bounded_stdout.original_bytes,
                stdout_truncated: bounded_stdout.truncated,
                stderr_text: bounded_stderr.text,
                stderr_original_bytes: bounded_stderr.original_bytes,
                stderr_truncated: bounded_stderr.truncated,
                command_blake3,
                diagnostics: Vec::new(),
                aeneas_summary: None,
                canary_receipt: None,
            }
        }
        Err(diagnostics) => {
            let stderr_text = diagnostics.join("\n");
            CommandOutcome {
                command_succeeded: false,
                timed_out: false,
                stdout_text: String::new(),
                stdout_original_bytes: 0,
                stdout_truncated: false,
                stderr_original_bytes: stderr_text.len(),
                stderr_text,
                stderr_truncated: false,
                command_blake3,
                diagnostics,
                aeneas_summary: None,
                canary_receipt: None,
            }
        }
    }
}

fn run_vermilion_canary(rail: &ProofRail, working_dir: &Path) -> CommandOutcome {
    let Some(receipt_path) = rail.receipt_paths.first() else {
        let diagnostic = "vermilion canary rail has no receipt path".to_owned();
        return canary_failure(diagnostic, None);
    };
    let text = match read_workspace_file(&working_dir.join(receipt_path)) {
        Ok(text) => text,
        Err(error) => return canary_failure(error, None),
    };
    let receipt = match serde_json::from_str::<VermilionCanaryReceipt>(&text) {
        Ok(receipt) => receipt,
        Err(error) => return canary_failure(format!("parse vermilion canary receipt: {error}"), Some(text)),
    };
    let diagnostics = validate_vermilion_canary(&receipt);
    let command_blake3 = crate::evidence::artifact_blake3_hex(format!("vermilion-canary/v1\n{text}").as_bytes());
    if diagnostics.is_empty() {
        let stdout_text = format!("{}\n", rail.pass_signal);
        CommandOutcome {
            command_succeeded: true,
            timed_out: false,
            stdout_original_bytes: stdout_text.len(),
            stdout_text,
            stdout_truncated: false,
            stderr_original_bytes: 0,
            stderr_text: String::new(),
            stderr_truncated: false,
            command_blake3,
            diagnostics,
            aeneas_summary: None,
            canary_receipt: Some(text),
        }
    } else {
        let stderr_text = diagnostics.join("\n");
        CommandOutcome {
            command_succeeded: false,
            timed_out: false,
            stdout_original_bytes: 0,
            stdout_text: String::new(),
            stdout_truncated: false,
            stderr_original_bytes: stderr_text.len(),
            stderr_text,
            stderr_truncated: false,
            command_blake3,
            diagnostics,
            aeneas_summary: None,
            canary_receipt: Some(text),
        }
    }
}

fn canary_failure(diagnostic: String, canary_receipt: Option<String>) -> CommandOutcome {
    CommandOutcome {
        command_succeeded: false,
        timed_out: false,
        stdout_text: String::new(),
        stdout_original_bytes: 0,
        stdout_truncated: false,
        stderr_text: diagnostic.clone(),
        stderr_original_bytes: diagnostic.len(),
        stderr_truncated: false,
        command_blake3: crate::evidence::artifact_blake3_hex(b"vermilion-canary/v1/failure"),
        diagnostics: vec![diagnostic],
        aeneas_summary: None,
        canary_receipt,
    }
}

fn proof_command_spec(rail: &ProofRail) -> Result<CommandSpec, String> {
    let plan = build_process_command_plan(ProcessCommandPlanInput {
        command_argv: rail.command_argv.clone(),
        timeout_ms: configured_timeout_ms(rail),
        environment_allowlist: proof_environment_allowlist(rail),
    })
    .map_err(|diagnostics| {
        if diagnostics.iter().any(|diagnostic| diagnostic.contains(COMMAND_ARGV_FIELD)) {
            format!("proof rail field `{COMMAND_ARGV_FIELD}` is required; refusing to execute shell command string")
        } else {
            diagnostics.join("; ")
        }
    })?;
    Ok(CommandSpec {
        executable: plan.executable,
        args: plan.args,
        argv: plan.argv,
        timeout_ms: plan.timeout_ms,
        environment_allowlist: plan.environment_allowlist,
    })
}

/// A proof command's program, resolved once so the identity probe and the proof run execute the
/// same file.
struct ResolvedCommand {
    program: PathBuf,
    current_dir: PathBuf,
    environment: Vec<(OsString, OsString)>,
}

fn resolve_command(spec: &CommandSpec, working_dir: &Path) -> Result<ResolvedCommand, String> {
    let command_line = spec.argv.join(" ");
    let current_dir = std::path::absolute(working_dir).map_err(|error| {
        format!("resolve proof command `{command_line}` working directory {}: {error}", working_dir.display())
    })?;
    let environment = allowed_environment(&spec.environment_allowlist)
        .into_iter()
        .map(|(name, value)| (OsString::from(name), value))
        .collect::<Vec<_>>();
    let search_path = environment.iter().find(|(name, _)| name == PATH_ENVIRONMENT_NAME).map(|(_, value)| value);
    let program = resolve_program(&spec.executable, &current_dir, search_path).ok_or_else(|| {
        format!("spawn proof command `{command_line}`: executable `{}` was not found", spec.executable)
    })?;
    Ok(ResolvedCommand {
        program,
        current_dir,
        environment,
    })
}

/// Runs the resolved verifier with `--identity` under the rail's timeout and requires the
/// reported verifier binary, solver, and toolchain digests to equal the rail's pins, so a program
/// that merely shares the verifier's name cannot produce production proof evidence.
fn pinned_verifier_diagnostics(rail: &ProofRail, spec: &CommandSpec, command: &ResolvedCommand) -> Vec<String> {
    let probe_line = format!("{} {VERIFIER_IDENTITY_FLAG}", spec.executable);
    let output = match run_resolved(
        command,
        &probe_line,
        &[VERIFIER_IDENTITY_FLAG.to_owned()],
        spec.timeout_ms,
        VERIFIER_IDENTITY_MAX_BYTES,
        VERIFIER_IDENTITY_MAX_BYTES,
    ) {
        Ok(output) => output,
        Err(error) => return vec![format!("{DIAGNOSTIC_UNPINNED_VERIFIER}: {error}")],
    };
    if output.disposition != bounded_exec::Disposition::Succeeded {
        return vec![format!(
            "{DIAGNOSTIC_UNPINNED_VERIFIER}: verifier identity probe `{probe_line}` did not succeed: {:?}",
            output.disposition
        )];
    }
    if output.stdout.truncated {
        return vec![format!(
            "{DIAGNOSTIC_UNPINNED_VERIFIER}: verifier identity probe `{probe_line}` exceeded {VERIFIER_IDENTITY_MAX_BYTES} bytes"
        )];
    }
    verifier_identity_report_diagnostics(rail, &String::from_utf8_lossy(&output.stdout.bytes))
}

/// Runs the resolved command with both output pipes drained concurrently into bounded buffers, so
/// verbose verifiers cannot stall on a full pipe. The command runs in its own process group, which
/// is killed on timeout and after exit, so descendants that inherited the pipes cannot keep
/// teardown waiting.
fn run_resolved(
    command: &ResolvedCommand,
    command_line: &str,
    args: &[String],
    timeout_ms: u64,
    stdout_max_bytes: usize,
    stderr_max_bytes: usize,
) -> Result<bounded_exec::ExecutionOutput, String> {
    let outcome_policy = bounded_exec::OutcomePolicy::new(vec![PROCESS_SUCCESS_EXIT_CODE], false, false)
        .map_err(|error| format!("construct proof command outcome policy: {error:?}"))?;
    bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: command.program.clone(),
            args: args.iter().map(OsString::from).collect(),
            current_dir: command.current_dir.clone(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: command.environment.clone(),
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms,
            stdin_max_bytes: PROCESS_STDIN_MAX_BYTES,
            stdout_max_bytes,
            stderr_max_bytes,
            poll_interval_ms: PROCESS_POLL_INTERVAL_MS.min(timeout_ms),
            teardown_timeout_ms: PROCESS_TEARDOWN_TIMEOUT_MS,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy,
    })
    .map_err(|error| match error {
        bounded_exec::RunError::Io {
            operation: bounded_exec::Operation::Spawn,
            error,
        } => {
            format!("spawn proof command `{command_line}`: {error}")
        }
        error => format!("run proof command `{command_line}`: {error}"),
    })
}

/// Resolves the executable the way `execvp` would for the child: paths containing `/` are relative
/// to the child's working directory, bare names are searched in the child's `PATH`.
fn resolve_program(executable: &str, current_dir: &Path, search_path: Option<&OsString>) -> Option<PathBuf> {
    if executable.contains('/') {
        return Some(current_dir.join(executable));
    }
    std::env::split_paths(search_path?)
        .map(|directory| current_dir.join(directory).join(executable))
        .find(|candidate| is_executable_file(candidate))
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;

    std::fs::metadata(path)
        .is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & EXECUTABLE_MODE_BITS != 0)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

fn proof_environment_allowlist(rail: &ProofRail) -> Vec<String> {
    let mut names = DEFAULT_ENV_ALLOWLIST.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>();
    names.extend(rail.environment_allowlist.iter().cloned());
    names.sort();
    names.dedup();
    names
}

fn allowed_environment(names: &[String]) -> Vec<(String, OsString)> {
    names.iter().filter_map(|name| std::env::var_os(name).map(|value| (name.clone(), value))).collect()
}

struct BoundedText {
    text: String,
    original_bytes: usize,
    truncated: bool,
}

fn bounded_utf8(captured: bounded_exec::CapturedOutput) -> BoundedText {
    let text = String::from_utf8(captured.bytes)
        .unwrap_or_else(|error| String::from_utf8_lossy(error.as_bytes()).into_owned());
    BoundedText {
        text,
        original_bytes: captured.observed_bytes,
        truncated: captured.truncated,
    }
}

fn configured_timeout_ms(rail: &ProofRail) -> u64 {
    if rail.timeout_ms == 0 {
        DEFAULT_PROOF_TIMEOUT_MS
    } else {
        rail.timeout_ms
    }
}

fn configured_log_limit(limit: usize) -> usize {
    if limit == 0 { DEFAULT_LOG_MAX_BYTES } else { limit }
}

fn command_blake3(argv: &[String]) -> String {
    let text = serde_json::to_string(argv).expect("command argv should serialize");
    crate::evidence::artifact_blake3_hex(text.as_bytes())
}

fn log_artifact(locator: &str, text: &str, original_bytes: usize, truncated: bool) -> VerusLogArtifact {
    VerusLogArtifact {
        locator: locator.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(text.as_bytes()),
        bytes: original_bytes,
        truncated,
    }
}

fn captured_proof_log(
    locator: &str,
    text: &str,
    checked_scope: &str,
    capture_source: &'static str,
) -> CapturedProofLog {
    CapturedProofLog {
        locator: locator.to_owned(),
        text: text.to_owned(),
        checked_scope: checked_scope.to_owned(),
        capture_source,
    }
}

fn checked_scope(policy: &PolicyEntry, rail_index: usize, rail: &ProofRail) -> String {
    if !rail.checked_scope.trim().is_empty() {
        rail.checked_scope.clone()
    } else if verifier_mode(rail) == VERIFIER_MODE_PRODUCTION {
        format!("formal-proof:production:{}#rail-{rail_index}", policy.policy_id)
    } else {
        format!("formal-proof:fixture-only:{}#rail-{rail_index}", policy.policy_id)
    }
}

fn verifier_mode(rail: &ProofRail) -> &str {
    if rail.verifier_mode.trim().is_empty() {
        VERIFIER_MODE_FIXTURE
    } else {
        rail.verifier_mode.as_str()
    }
}

fn proof_artifact_locator(policy: &PolicyEntry, rail_index: usize) -> String {
    format!("{PROOF_ARTIFACT_DIR_NAME}/{}-rail-{rail_index}.receipt", safe_artifact_stem(&policy.policy_id))
}

fn proof_log_artifact_locator(policy: &PolicyEntry, rail_index: usize, suffix: &str) -> String {
    format!("{PROOF_ARTIFACT_DIR_NAME}/{}-rail-{rail_index}.{suffix}", safe_artifact_stem(&policy.policy_id))
}

/// Aeneas stage artifacts for one rail live in `proofs/<policy>-rail-<N>.aeneas/`, next to that
/// rail's receipt and logs, so rails in one bundle never overwrite each other's stage artifacts.
fn aeneas_artifact_locator(policy: &PolicyEntry, rail_index: usize, name: &str) -> String {
    format!("{}/{name}", proof_log_artifact_locator(policy, rail_index, AENEAS_ARTIFACT_DIR_SUFFIX))
}

fn safe_artifact_stem(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else {
                SAFE_SEPARATOR
            }
        })
        .collect()
}

fn proof_provenance_record(
    artifact_locator: &str,
    contents: &[u8],
    checked_scope: &str,
    verifier_identity: &str,
) -> ProvenanceRecord {
    ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: PROOF_SOURCE_ID.to_owned(),
        checker_id: CHECKER_OCTET_STANDARDS.to_owned(),
        evidence_tier: EvidenceTier::FormalProof,
        artifact_locator: artifact_locator.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(contents),
        checked_scope: checked_scope.to_owned(),
        verifier_identity: Some(verifier_identity.to_owned()),
        proof_artifact_identity: Some(artifact_locator.to_owned()),
        human_action: None,
        parent_artifacts: vec![COMMAND_ARTIFACT_NAME.to_owned(), STATUS_ARTIFACT_NAME.to_owned()],
    }
}

fn failed_receipt_text(policy_id: &str, rail_index: usize, diagnostics: &[String]) -> String {
    let value = serde_json::json!({
        "schema_version": "octet-verus-proof-receipt/v1",
        "policy_id": policy_id,
        "rail_index": rail_index,
        "status": PROOF_STATUS_FAILED,
        "diagnostics": diagnostics,
    });
    format!("{}\n", serde_json::to_string_pretty(&value).expect("failed receipt json"))
}

/// Refuses an artifact directory `prepare_artifact_dir` would reject, without creating or pruning
/// it, so a foreign directory fails before any verifier runs rather than after every selected rail.
/// The directory must also not be the proof workspace root or one of its ancestors: a workspace
/// holding bundle markers would otherwise lose its own `proofs/` tree to pruning.
fn ensure_artifact_dir_reusable(artifact_dir: &Path, workspace_root: &Path) -> Result<(), String> {
    crate::cap_fs::reject_artifact_dir_enclosing_workspace(artifact_dir, workspace_root, "prove")?;
    let exists = artifact_dir
        .try_exists()
        .map_err(|error| format!("inspect artifact directory {}: {error}", artifact_dir.display()))?;
    if exists {
        reusable_artifact_entries(&ArtifactRoot::open_existing(artifact_dir)?, artifact_dir)?;
    }
    Ok(())
}

/// Opens the artifact directory the way `cargo octet check` does: only an empty directory or an
/// earlier prove bundle may be reused. An earlier bundle's outputs are cleared first, so receipts,
/// logs, Aeneas stage artifacts, or an evidence-log manifest from a previous run cannot survive
/// into this one; entries outside `PROVE_BUNDLE_ENTRY_NAMES` are left in place.
fn prepare_artifact_dir(artifact_dir: &Path) -> Result<ArtifactRoot, String> {
    let artifact_root = ArtifactRoot::open_or_create(artifact_dir)?;
    for entry in reusable_artifact_entries(&artifact_root, artifact_dir)? {
        if PROVE_BUNDLE_ENTRY_NAMES.contains(&entry.as_str()) {
            artifact_root.remove_file_or_dir_all(&RootRelativePath::new(entry.as_str())?)?;
        }
    }
    Ok(artifact_root)
}

/// The entries of an empty directory or an earlier prove bundle; any other directory is refused.
fn reusable_artifact_entries(artifact_root: &ArtifactRoot, artifact_dir: &Path) -> Result<Vec<String>, String> {
    let entries = artifact_root.read_entry_names()?;
    if entries.is_empty() || is_existing_prove_bundle(artifact_root)? {
        return Ok(entries);
    }
    Err(format!(
        "refusing to use artifact directory {}: it is not empty and is not an existing `cargo octet prove` bundle \
         (missing `{PROVENANCE_ARTIFACT_NAME}` or a `{STATUS_ARTIFACT_NAME}` with bundle_kind `{BUNDLE_KIND_PROVE}`); \
         choose a new or empty directory",
        artifact_dir.display()
    ))
}

fn is_existing_prove_bundle(artifact_root: &ArtifactRoot) -> Result<bool, String> {
    let status = RootRelativePath::new(STATUS_ARTIFACT_NAME)?;
    if !artifact_root.is_file(&RootRelativePath::new(PROVENANCE_ARTIFACT_NAME)?)? || !artifact_root.is_file(&status)? {
        return Ok(false);
    }
    let status_text = artifact_root.read_to_string(&status)?;
    Ok(serde_json::from_str::<serde_json::Value>(&status_text).is_ok_and(|status_json| {
        status_json.get("bundle_kind").and_then(serde_json::Value::as_str) == Some(BUNDLE_KIND_PROVE)
    }))
}

fn write_artifacts(
    args: &ProveArgs,
    report: &ProveReport,
    executions: &[ExecutedProof],
    goal_states: &[ProofGoalState],
) -> Result<(), String> {
    let artifact_root = prepare_artifact_dir(&args.artifact_dir)?;
    artifact_root.create_dir_all(&RootRelativePath::new(PROOF_ARTIFACT_DIR_NAME)?)?;

    let command_text = format!("{}\n", args.command_line);
    let status_text = status_text(args.output_format, report)?;
    let summary_text = summary_text(report, &args.command_line, goal_states, executions);
    let proof_cards_text = proof_cards_text(executions)?;
    let provenance_text = provenance_text(&command_text, &status_text, &summary_text, &proof_cards_text, executions)?;
    let evidence_log_manifest_text = evidence_log_manifest_text(&args.command_line, executions)?;

    write_artifact_text(&artifact_root, COMMAND_ARTIFACT_NAME, &command_text)?;
    write_artifact_text(&artifact_root, STATUS_ARTIFACT_NAME, &status_text)?;
    write_artifact_text(&artifact_root, SUMMARY_ARTIFACT_NAME, &summary_text)?;
    write_artifact_text(&artifact_root, PROOF_CARDS_ARTIFACT_NAME, &proof_cards_text)?;
    write_artifact_text(&artifact_root, PROVENANCE_ARTIFACT_NAME, &provenance_text)?;
    if let Some(manifest_text) = evidence_log_manifest_text {
        write_artifact_text(&artifact_root, crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT, &manifest_text)?;
    }
    for execution in executions {
        write_artifact_text(&artifact_root, &execution.row.receipt, &execution.receipt_text)?;
        for artifact in &execution.extra_artifacts {
            write_artifact_bytes(&artifact_root, &artifact.locator, &artifact.contents)?;
        }
        for log in &execution.logs {
            write_artifact_text(&artifact_root, &log.locator, &log.text)?;
        }
    }
    Ok(())
}

fn status_text(output_format: OutputFormat, report: &ProveReport) -> Result<String, String> {
    let artifact = StatusArtifact {
        schema_version: STATUS_SCHEMA_VERSION,
        bundle_kind: BUNDLE_KIND_PROVE,
        status: report.status,
        output_format: output_format.as_str(),
        metadata: StatusMetadata {
            profile_name: PROVE_PROFILE_NAME,
        },
        proofs: report.proofs.clone(),
    };
    json_text(&artifact)
}

fn summary_text(
    report: &ProveReport,
    replay_command: &str,
    goal_states: &[ProofGoalState],
    executions: &[ExecutedProof],
) -> String {
    let mut text = format!(
        "octet prove status={} proofs={} diagnostics={}\n",
        report.status,
        report.proofs.len(),
        report.diagnostics.len()
    );
    writeln!(&mut text, "replay_command: {replay_command}").expect("write replay command summary to string");
    writeln!(&mut text, "claim_boundary: {EVIDENCE_LOG_CLAIM_BOUNDARY}")
        .expect("write claim boundary summary to string");
    for proof in &report.proofs {
        writeln!(
            &mut text,
            "proof policy_id={} rail_index={} backend={} verifier_mode={} status={} receipt={} checked_scope={}",
            proof.policy_id,
            proof.rail_index,
            proof.backend,
            proof.verifier_mode,
            proof.status,
            proof.receipt,
            proof.checked_scope
        )
        .expect("write proof summary to string");
    }
    for goal_state in goal_states {
        writeln!(
            &mut text,
            "proof_goal_state schema_version={} goal_id={} claim_id={} satisfied_evidence={} missing_obligations={} blockers={} emits_formal_proof={}",
            goal_state.schema_version,
            goal_state.goal_id,
            goal_state.claim_id,
            goal_state.satisfied_evidence.len(),
            goal_state.missing_obligations.len(),
            goal_state.blockers.len(),
            goal_state.emits_formal_proof
        )
        .expect("write proof goal-state summary to string");
        for missing in &goal_state.missing_obligations {
            writeln!(&mut text, "proof_goal_state missing_obligation: {missing}")
                .expect("write proof goal-state missing obligation to string");
        }
        for blocker in &goal_state.blockers {
            writeln!(&mut text, "proof_goal_state blocker: {blocker}")
                .expect("write proof goal-state blocker to string");
        }
        for remediation in &goal_state.remediation_commands {
            writeln!(&mut text, "proof_goal_state remediation: {remediation}")
                .expect("write proof goal-state remediation to string");
        }
    }
    for entry in evidence_log_entries(replay_command, executions) {
        writeln!(
            &mut text,
            "evidence_log artifact={} blake3={} bytes={} checked_scope={} capture_source={} retention={} claim_boundary={}",
            entry.artifact_locator,
            entry.blake3,
            entry.bytes,
            entry.checked_scope,
            entry.capture_source,
            entry.retention,
            EVIDENCE_LOG_CLAIM_BOUNDARY
        )
        .expect("write evidence log summary to string");
        writeln!(&mut text, "evidence_log replay_command: {}", entry.replay_command)
            .expect("write evidence log replay command to string");
    }
    for diagnostic in &report.diagnostics {
        writeln!(&mut text, "diagnostic: {diagnostic}").expect("write diagnostic summary to string");
    }
    text
}

fn proof_goal_state_for_execution(args: &ProveArgs, execution: &ExecutedProof) -> ProofGoalState {
    let row = &execution.row;
    let goal_id = proof_or_property_id(&row.policy_id, row.rail_index);
    let explain_command = proof_goal_explain_command(args, &goal_id);
    // The required identity comes from the receipt bytes this run writes; the evidence is the
    // digest and locator the formal_proof provenance record attests. A disagreement is stale.
    let expected_receipt_blake3 = crate::evidence::artifact_blake3_hex(execution.receipt_text.as_bytes());
    let evidence = execution
        .formal_provenance
        .as_ref()
        .map(|provenance| {
            vec![GoalEvidence {
                artifact: provenance.artifact_locator.clone(),
                actual_blake3: provenance.blake3.clone(),
                evidence_tier: provenance.evidence_tier,
                status: row.status.to_owned(),
            }]
        })
        .unwrap_or_default();
    let mut diagnostics = row.diagnostics.clone();
    diagnostics.extend(proof_card_for_execution(execution).diagnostics);
    let non_claims = proof_goal_non_claims();
    let boundary_wording = non_claims.join("; ");

    build_proof_goal_state(ProofGoalInput {
        goal_id,
        claim_id: row.policy_id.clone(),
        checked_scope: row.checked_scope.clone(),
        assumptions: proof_goal_assumptions(row),
        required_artifacts: vec![GoalArtifactRequirement {
            artifact: row.receipt.clone(),
            expected_blake3: expected_receipt_blake3,
        }],
        evidence,
        expected_evidence_tier: EvidenceTier::FormalProof,
        diagnostics,
        non_claims,
        boundary_wording,
        remediation_commands: vec![ARTIFACT_VERIFY_COMMAND.to_owned()],
        explain_command,
    })
}

/// Builds a runnable `cargo octet prove --explain` rerun for one goal from the run's registry,
/// source, and family inputs; without the families, rails from other pillars would be filtered
/// out of the rerun. The rerun writes beside the reviewed bundle rather than over it.
fn proof_goal_explain_command(args: &ProveArgs, goal_id: &str) -> String {
    let mut words = vec!["octet".to_owned(), "prove".to_owned()];
    if let Some(registry) = &args.registry {
        words.extend(["--registry".to_owned(), registry.display().to_string()]);
    }
    if let Some(policy_artifacts_dir) = &args.policy_artifacts_dir {
        words.extend([
            "--policy-artifacts-dir".to_owned(),
            policy_artifacts_dir.display().to_string(),
        ]);
    }
    for source in &args.source {
        words.extend(["--source".to_owned(), source.display().to_string()]);
    }
    for family in &args.family {
        words.extend(["--family".to_owned(), family.clone()]);
    }
    words.extend([
        "--rail".to_owned(),
        goal_id.to_owned(),
        "--artifact-dir".to_owned(),
        explain_artifact_dir(&args.artifact_dir).display().to_string(),
        "--explain".to_owned(),
    ]);
    crate::cli::format_command_line("cargo", &words)
}

fn explain_artifact_dir(artifact_dir: &Path) -> PathBuf {
    let mut name = artifact_dir.file_name().unwrap_or(std::ffi::OsStr::new("octet-proofs")).to_os_string();
    name.push("-explain");
    artifact_dir.with_file_name(name)
}

fn proof_goal_assumptions(row: &ProofReportRow) -> Vec<String> {
    vec![
        format!("policy `{}` selected rail `{}`", row.policy_id, row.rail_index),
        format!("checked scope `{}`", row.checked_scope),
        format!("verifier mode `{}`", row.verifier_mode),
    ]
}

fn proof_goal_non_claims() -> Vec<String> {
    let mut non_claims = default_card_non_claims();
    non_claims.extend([
        "not Lean execution".to_owned(),
        "not solver completeness".to_owned(),
        "not verified runtime".to_owned(),
    ]);
    non_claims
}

fn evidence_log_manifest_text(replay_command: &str, executions: &[ExecutedProof]) -> Result<Option<String>, String> {
    let entries = evidence_log_entries(replay_command, executions);
    if entries.is_empty() {
        return Ok(None);
    }
    let manifest = crate::evidence_log::manifest_from_entries(entries);
    json_text(&manifest).map(Some)
}

fn evidence_log_entries(
    replay_command: &str,
    executions: &[ExecutedProof],
) -> Vec<crate::evidence_log::EvidenceLogEntry> {
    executions
        .iter()
        .flat_map(|execution| {
            execution.logs.iter().map(|log| {
                crate::evidence_log::EvidenceLogEntry::from_bytes(
                    &log.locator,
                    log.text.as_bytes(),
                    replay_command,
                    &log.checked_scope,
                    log.capture_source,
                    crate::evidence_log::RETENTION_CONTENT_ADDRESSED_RAW,
                )
            })
        })
        .collect()
}

fn provenance_text(
    command_text: &str,
    status_text: &str,
    summary_text: &str,
    proof_cards_text: &str,
    executions: &[ExecutedProof],
) -> Result<String, String> {
    let mut records = vec![
        runner_provenance_record(COMMAND_ARTIFACT_NAME, command_text.as_bytes()),
        runner_provenance_record(STATUS_ARTIFACT_NAME, status_text.as_bytes()),
        runner_provenance_record(SUMMARY_ARTIFACT_NAME, summary_text.as_bytes()),
        proof_card_provenance_record(proof_cards_text.as_bytes()),
    ];
    records.extend(executions.iter().filter_map(|execution| execution.formal_provenance.clone()));
    records.extend(executions.iter().flat_map(|execution| {
        let withheld = execution
            .formal_provenance
            .as_ref()
            .filter(|receipt_record| receipt_record.evidence_tier != EvidenceTier::FormalProof);
        execution.extra_artifacts.iter().map(move |artifact| {
            let mut record = proof_provenance_record(
                &artifact.locator,
                &artifact.contents,
                &execution.row.checked_scope,
                "aeneas-stage-artifact",
            );
            if let Some(receipt_record) = withheld {
                record.evidence_tier = receipt_record.evidence_tier;
                record.human_action.clone_from(&receipt_record.human_action);
            }
            record
        })
    }));
    let mut text = String::new();
    for record in records {
        text.push_str(
            &serde_json::to_string(&record).map_err(|error| format!("serialize provenance record: {error}"))?,
        );
        text.push('\n');
    }
    Ok(text)
}

fn runner_provenance_record(artifact_locator: &str, contents: &[u8]) -> ProvenanceRecord {
    ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: RUNNER_SOURCE_ID.to_owned(),
        checker_id: CHECKER_CARGO_OCTET.to_owned(),
        evidence_tier: EvidenceTier::ManualReviewRequired,
        artifact_locator: artifact_locator.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(contents),
        checked_scope: format!("profile:{PROVE_PROFILE_NAME}"),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: Some(MANUAL_ACTION.to_owned()),
        parent_artifacts: Vec::new(),
    }
}

fn proof_card_provenance_record(contents: &[u8]) -> ProvenanceRecord {
    ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: RUNNER_SOURCE_ID.to_owned(),
        checker_id: CHECKER_CARGO_OCTET.to_owned(),
        evidence_tier: EvidenceTier::ManualReviewRequired,
        artifact_locator: PROOF_CARDS_ARTIFACT_NAME.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(contents),
        checked_scope: format!("profile:{PROVE_PROFILE_NAME}"),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: Some(PROOF_CARD_MANUAL_ACTION.to_owned()),
        parent_artifacts: Vec::new(),
    }
}

fn proof_cards_text(executions: &[ExecutedProof]) -> Result<String, String> {
    let imported = imported_trellis_manifest()?;
    let mut cards = executions.iter().map(proof_card_for_execution).collect::<Vec<_>>();
    cards.extend(imported.bundles.iter().map(proof_card_for_imported_bundle));
    let deck = ProofCardDeck {
        schema_version: PROOF_CARDS_SCHEMA_VERSION.to_owned(),
        cards,
    };
    json_text(&deck)
}

fn imported_trellis_manifest() -> Result<ImportedProofBundleManifest, String> {
    let manifest = serde_json::from_str::<ImportedProofBundleManifest>(TRELLIS_IMPORTS_JSON)
        .map_err(|error| format!("parse {TRELLIS_IMPORT_PROVENANCE_ARTIFACT}: {error}"))?;
    let diagnostics = validate_imported_proof_manifest_shape(&manifest);
    if diagnostics.is_empty() {
        Ok(manifest)
    } else {
        Err(format!("invalid {TRELLIS_IMPORT_PROVENANCE_ARTIFACT}: {}", diagnostics.join("; ")))
    }
}

fn proof_card_for_execution(execution: &ExecutedProof) -> ProofCard {
    if execution.row.backend == VERMILION_CANARY_BACKEND {
        let mut card = initial_vermilion_canary_proof_card(execution);
        let diagnostics = proof_card_validation_diagnostics(&card, None, None);
        if !diagnostics.is_empty() {
            EvidenceTier::ManualReviewRequired.as_str().clone_into(&mut card.evidence_tier);
            card.diagnostics.extend(diagnostics);
        }
        return card;
    }
    let receipt = parse_verus_receipt(&execution.receipt_text).ok();
    let formal_proof_envelope = parse_formal_proof_envelope(&execution.receipt_text).ok();
    let mut card = formal_proof_envelope.as_ref().map_or_else(
        || initial_proof_card(execution, receipt.as_ref(), execution.formal_provenance.as_ref()),
        |envelope| initial_aeneas_proof_card(execution, envelope, execution.formal_provenance.as_ref()),
    );
    card.formal_proof_envelope.clone_from(&formal_proof_envelope);
    // A receipt record withheld to `manual_review_required` binds the bytes but cannot promote the
    // card.
    let formal_provenance = execution
        .formal_provenance
        .as_ref()
        .filter(|record| record.evidence_tier == EvidenceTier::FormalProof);
    if formal_provenance.is_some() && (receipt.is_some() || formal_proof_envelope.is_some()) {
        card.evidence_tier.clear();
        card.evidence_tier.push_str(EvidenceTier::FormalProof.as_str());
    }
    let diagnostics = formal_proof_envelope.as_ref().map_or_else(
        || proof_card_validation_diagnostics(&card, receipt.as_ref(), formal_provenance),
        |envelope| aeneas_proof_card_linkage_diagnostics(&card, envelope, formal_provenance),
    );
    if diagnostics.is_empty() {
        card
    } else {
        card.evidence_tier.clear();
        card.evidence_tier.push_str(EvidenceTier::ManualReviewRequired.as_str());
        card.diagnostics.extend(diagnostics);
        card
    }
}

fn initial_aeneas_proof_card(
    execution: &ExecutedProof,
    envelope: &FormalProofEnvelope,
    provenance: Option<&ProvenanceRecord>,
) -> ProofCard {
    let row = &execution.row;
    let mut identities = envelope
        .source_bindings
        .iter()
        .chain(envelope.artifact_identities.iter())
        .map(|identity| ProofCardArtifactIdentity {
            artifact: identity.locator.clone(),
            blake3: identity.blake3.clone(),
        })
        .collect::<Vec<_>>();
    if let Some(provenance) = provenance {
        identities.push(ProofCardArtifactIdentity {
            artifact: provenance.artifact_locator.clone(),
            blake3: provenance.blake3.clone(),
        });
    }
    let FormalProofProvider::AeneasLean(payload) = &envelope.provider else {
        unreachable!("Aeneas proof card helper requires the Aeneas provider")
    };
    let mut non_claims = default_card_non_claims();
    non_claims.extend(envelope.non_claims.clone());
    non_claims.sort();
    non_claims.dedup();
    ProofCard {
        proof_or_property_id: envelope.claim.theorem_or_property_id.clone(),
        source_requirement_or_policy_id: envelope.claim.requirement_id.clone(),
        policy_id: row.policy_id.clone(),
        rail_index: row.rail_index,
        checked_rust_source_scope: row.checked_scope.clone(),
        source_paths: envelope.source_bindings.iter().map(|binding| binding.locator.clone()).collect(),
        proof_target_paths: envelope.artifact_identities.iter().map(|identity| identity.locator.clone()).collect(),
        verifier_mode: row.verifier_mode.clone(),
        verifier_identity_summary: format!("aeneas-lean@{}", payload.aeneas_release),
        solver_identity_summary: String::new(),
        toolchain_identity_summary: payload.lean_toolchain.clone(),
        receipt_path: row.receipt.clone(),
        artifact_blake3_identities: identities,
        evidence_tier: envelope.evidence_tier.as_str().to_owned(),
        evidence_origin: EVIDENCE_ORIGIN_OCTET_RERUN.to_owned(),
        status: envelope.status.as_str().to_owned(),
        trust_boundaries: envelope.trust_boundaries.clone(),
        non_claims,
        artifact_verifier_command: ARTIFACT_VERIFY_COMMAND.to_owned(),
        provenance_artifact: PROVENANCE_ARTIFACT_NAME.to_owned(),
        external_reference: None,
        formal_proof_envelope: None,
        negative_check: None,
        diagnostics: row.diagnostics.clone(),
    }
}

fn aeneas_proof_card_linkage_diagnostics(
    card: &ProofCard,
    envelope: &FormalProofEnvelope,
    provenance: Option<&ProvenanceRecord>,
) -> Vec<String> {
    let mut diagnostics = validate_formal_proof_envelope(envelope);
    let Some(provenance) = provenance else {
        diagnostics.push("Aeneas formal proof card missing matching formal_proof provenance".to_owned());
        return diagnostics;
    };
    if card.receipt_path != provenance.artifact_locator {
        diagnostics.push("Aeneas proof card receipt path does not match provenance".to_owned());
    }
    if !card
        .artifact_blake3_identities
        .iter()
        .any(|identity| identity.artifact == provenance.artifact_locator && identity.blake3 == provenance.blake3)
    {
        diagnostics.push("Aeneas proof card is missing the receipt BLAKE3 identity".to_owned());
    }
    if card.status != STATUS_PASSED || envelope.status != FormalProofStatus::Passed {
        diagnostics.push("Aeneas formal proof card requires passed status".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn initial_proof_card(
    execution: &ExecutedProof,
    receipt: Option<&VerusProofReceipt>,
    provenance: Option<&ProvenanceRecord>,
) -> ProofCard {
    let row = &execution.row;
    let source_paths = receipt
        .map(|receipt| receipt.source_bindings.iter().map(|binding| binding.path.clone()).collect())
        .unwrap_or_default();
    let proof_target_paths = receipt
        .map(|receipt| receipt.proof_bindings.iter().map(|binding| binding.path.clone()).collect())
        .unwrap_or_default();
    let trust_boundaries = receipt.map(|receipt| receipt.trust_boundaries.clone()).unwrap_or_default();
    let artifact_blake3_identities =
        receipt.map(|receipt| receipt_artifact_identities(receipt, provenance)).unwrap_or_default();
    let non_claims = receipt.map_or_else(default_card_non_claims, |receipt| card_non_claims(&receipt.non_claims));
    ProofCard {
        proof_or_property_id: proof_or_property_id(&row.policy_id, row.rail_index),
        source_requirement_or_policy_id: row.policy_id.clone(),
        policy_id: row.policy_id.clone(),
        rail_index: row.rail_index,
        checked_rust_source_scope: row.checked_scope.clone(),
        source_paths,
        proof_target_paths,
        verifier_mode: row.verifier_mode.clone(),
        verifier_identity_summary: receipt
            .map(|receipt| {
                identity_summary(&receipt.verifier_identity.verifier_id, &receipt.verifier_identity.verifier_digest)
            })
            .unwrap_or_default(),
        solver_identity_summary: receipt
            .map(|receipt| identity_summary(&receipt.solver_identity.solver_id, &receipt.solver_identity.solver_digest))
            .unwrap_or_default(),
        toolchain_identity_summary: receipt
            .map(|receipt| {
                identity_summary(&receipt.toolchain_identity.toolchain_id, &receipt.toolchain_identity.toolchain_digest)
            })
            .unwrap_or_default(),
        receipt_path: row.receipt.clone(),
        artifact_blake3_identities,
        evidence_tier: EvidenceTier::ManualReviewRequired.as_str().to_owned(),
        evidence_origin: EVIDENCE_ORIGIN_OCTET_RERUN.to_owned(),
        status: row.status.to_owned(),
        trust_boundaries,
        non_claims,
        artifact_verifier_command: ARTIFACT_VERIFY_COMMAND.to_owned(),
        provenance_artifact: PROVENANCE_ARTIFACT_NAME.to_owned(),
        external_reference: None,
        formal_proof_envelope: None,
        negative_check: None,
        diagnostics: row.diagnostics.clone(),
    }
}

fn initial_vermilion_canary_proof_card(execution: &ExecutedProof) -> ProofCard {
    let row = &execution.row;
    let receipt_identity = crate::evidence::artifact_blake3_hex(execution.receipt_text.as_bytes());
    let source_identity = crate::evidence::artifact_blake3_hex(CANARY_SOURCE.as_bytes());
    let mut non_claims = default_card_non_claims();
    non_claims
        .push("recorded-only measured Vermilion canary; not a formal proof or fresh provider execution".to_owned());
    let mut source_paths = vec![CANARY_SOURCE_PATH.to_owned()];
    let mut verifier_identity_summary = String::from("vermilion-lean canary");
    let mut solver_identity_summary = String::new();
    let mut toolchain_identity_summary = String::new();
    let mut trust_boundaries = vec![
        "recorded-only measured Vermilion canary; the canary is not a formal proof or a fresh provider execution"
            .to_owned(),
    ];
    match serde_json::from_str::<VermilionCanaryReceipt>(&execution.receipt_text) {
        Ok(canary) => {
            source_paths = vec![canary.source_path.clone()];
            verifier_identity_summary = format!(
                "vermilion@{}; verus@{}; lean@{}",
                canary.vermilion_revision, canary.verus_fork_revision, canary.lean_toolchain
            );
            solver_identity_summary = format!("cvc5@{}", canary.cvc5_revision);
            toolchain_identity_summary = format!("{}; mathlib@{}", canary.lean_toolchain, canary.mathlib_revision);
            non_claims.extend(canary.non_claims.clone());
            trust_boundaries.extend(canary.non_claims.clone());
        }
        Err(error) => {
            non_claims.push(format!("recorded-only canary receipt could not be parsed: {error}"));
        }
    }
    non_claims.sort();
    non_claims.dedup();
    trust_boundaries.sort();
    trust_boundaries.dedup();
    ProofCard {
        proof_or_property_id: format!("vermilion-canary:{}", row.policy_id),
        source_requirement_or_policy_id: row.policy_id.clone(),
        policy_id: row.policy_id.clone(),
        rail_index: row.rail_index,
        checked_rust_source_scope: row.checked_scope.clone(),
        source_paths,
        proof_target_paths: vec![row.receipt.clone()],
        verifier_mode: "recorded".to_owned(),
        verifier_identity_summary,
        solver_identity_summary,
        toolchain_identity_summary,
        receipt_path: row.receipt.clone(),
        artifact_blake3_identities: vec![
            ProofCardArtifactIdentity {
                artifact: row.receipt.clone(),
                blake3: receipt_identity,
            },
            ProofCardArtifactIdentity {
                artifact: VERMILION_CANARY_SOURCE_ARTIFACT.to_owned(),
                blake3: source_identity,
            },
        ],
        evidence_tier: EvidenceTier::ManualReviewRequired.as_str().to_owned(),
        evidence_origin: EVIDENCE_ORIGIN_VERMILION_CANARY.to_owned(),
        status: STATUS_MANUAL_REVIEW_REQUIRED.to_owned(),
        trust_boundaries,
        non_claims,
        artifact_verifier_command: ARTIFACT_VERIFY_COMMAND.to_owned(),
        provenance_artifact: PROVENANCE_ARTIFACT_NAME.to_owned(),
        external_reference: None,
        formal_proof_envelope: None,
        negative_check: None,
        diagnostics: row.diagnostics.clone(),
    }
}

fn proof_card_for_imported_bundle(bundle: &ImportedProofBundle) -> ProofCard {
    let manifest = ImportedProofBundleManifest {
        schema_version: crate::imported_proof::IMPORTED_PROOF_BUNDLE_SCHEMA_VERSION.to_owned(),
        generated_from: TRELLIS_IMPORT_PROVENANCE_ARTIFACT.to_owned(),
        bundles: vec![bundle.clone()],
    };
    let diagnostics = validate_imported_proof_manifest_shape(&manifest);
    let artifact_blake3_identities = bundle_artifacts(bundle)
        .into_iter()
        .map(|artifact| ProofCardArtifactIdentity {
            artifact: artifact.path.clone(),
            blake3: artifact.blake3.clone(),
        })
        .collect::<Vec<_>>();
    let mut non_claims = default_card_non_claims();
    non_claims.push(NON_CLAIM_BLAKE3_IDENTITY.to_owned());
    let mut card = ProofCard {
        proof_or_property_id: bundle.bundle_id.clone(),
        source_requirement_or_policy_id: "octet.verus_proof_rails.trellis_blake3_import".to_owned(),
        policy_id: "octet.verus_proof_rails.trellis_blake3_import".to_owned(),
        rail_index: 0,
        checked_rust_source_scope: format!("imported-proof:external-trellis:{}", bundle.bundle_id),
        source_paths: vec![bundle.source_path.clone()],
        proof_target_paths: vec![bundle.proof_path.clone()],
        verifier_mode: VERIFIER_MODE_EXTERNAL.to_owned(),
        verifier_identity_summary: bundle.verifier_identity.clone(),
        solver_identity_summary: String::new(),
        toolchain_identity_summary: bundle.verifier_command_argv.join(" "),
        receipt_path: bundle
            .receipt_artifacts
            .first()
            .map_or_else(|| "not-rerun-by-octet".to_owned(), |artifact| artifact.path.clone()),
        artifact_blake3_identities,
        evidence_tier: bundle.intended_evidence_tier.clone(),
        evidence_origin: EVIDENCE_ORIGIN_EXTERNAL_TRELLIS.to_owned(),
        status: STATUS_MANUAL_REVIEW_REQUIRED.to_owned(),
        trust_boundaries: bundle.trust_boundary_notes.clone(),
        non_claims,
        artifact_verifier_command: ARTIFACT_VERIFY_COMMAND.to_owned(),
        provenance_artifact: TRELLIS_IMPORT_PROVENANCE_ARTIFACT.to_owned(),
        external_reference: Some(ProofCardExternalReference {
            repository_label: bundle.repository_label.clone(),
            repository_revision: bundle.repository_revision.clone(),
            hash_algorithm: bundle.hash_algorithm.clone(),
            promotion_command_argv: bundle.promotion_command_argv.clone(),
        }),
        formal_proof_envelope: None,
        negative_check: None,
        diagnostics,
    };
    let card_diagnostics = proof_card_validation_diagnostics(&card, None, None);
    card.diagnostics.extend(card_diagnostics);
    card.diagnostics.sort();
    card.diagnostics.dedup();
    card
}

fn receipt_artifact_identities(
    receipt: &VerusProofReceipt,
    provenance: Option<&ProvenanceRecord>,
) -> Vec<ProofCardArtifactIdentity> {
    let mut identities = Vec::new();
    identities.extend(receipt.source_bindings.iter().map(binding_identity));
    identities.extend(receipt.proof_bindings.iter().map(binding_identity));
    identities.push(log_identity(&receipt.stdout_artifact));
    identities.push(log_identity(&receipt.stderr_artifact));
    if let Some(provenance) = provenance {
        identities.push(ProofCardArtifactIdentity {
            artifact: receipt.proof_artifact_identity.clone(),
            blake3: provenance.blake3.clone(),
        });
    }
    identities
}

fn binding_identity(binding: &octet_standards::proof::VerusProofBinding) -> ProofCardArtifactIdentity {
    ProofCardArtifactIdentity {
        artifact: binding.path.clone(),
        blake3: binding.blake3.clone(),
    }
}

fn log_identity(artifact: &VerusLogArtifact) -> ProofCardArtifactIdentity {
    ProofCardArtifactIdentity {
        artifact: artifact.locator.clone(),
        blake3: artifact.blake3.clone(),
    }
}

fn proof_card_validation_diagnostics(
    card: &ProofCard,
    receipt: Option<&VerusProofReceipt>,
    provenance: Option<&ProvenanceRecord>,
) -> Vec<String> {
    let mut diagnostics = proof_card_required_field_diagnostics(card);
    diagnostics.extend(proof_card_mode_diagnostics(card));
    diagnostics.extend(proof_card_non_claim_diagnostics(card));
    diagnostics.extend(proof_card_imported_reference_diagnostics(card));
    diagnostics.extend(proof_card_linkage_diagnostics(card, receipt, provenance));
    diagnostics.extend(formal_proof_envelope_diagnostics(card));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn proof_card_required_field_diagnostics(card: &ProofCard) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_missing_field(&mut diagnostics, FIELD_PROOF_OR_PROPERTY_ID, &card.proof_or_property_id);
    push_missing_field(&mut diagnostics, FIELD_SOURCE_REQUIREMENT_OR_POLICY_ID, &card.source_requirement_or_policy_id);
    push_missing_field(&mut diagnostics, FIELD_CHECKED_RUST_SOURCE_SCOPE, &card.checked_rust_source_scope);
    push_missing_collection(&mut diagnostics, FIELD_SOURCE_PATHS, &card.source_paths);
    push_missing_collection(&mut diagnostics, FIELD_PROOF_TARGET_PATHS, &card.proof_target_paths);
    push_missing_field(&mut diagnostics, FIELD_VERIFIER_MODE, &card.verifier_mode);
    push_missing_field(&mut diagnostics, FIELD_VERIFIER_IDENTITY_SUMMARY, &card.verifier_identity_summary);
    if card.verifier_mode == VERIFIER_MODE_PRODUCTION {
        push_missing_field(&mut diagnostics, FIELD_SOLVER_IDENTITY_SUMMARY, &card.solver_identity_summary);
    }
    push_missing_field(&mut diagnostics, FIELD_TOOLCHAIN_IDENTITY_SUMMARY, &card.toolchain_identity_summary);
    push_missing_field(&mut diagnostics, FIELD_RECEIPT_PATH, &card.receipt_path);
    push_missing_identity_collection(
        &mut diagnostics,
        FIELD_ARTIFACT_BLAKE3_IDENTITIES,
        &card.artifact_blake3_identities,
    );
    push_missing_field(&mut diagnostics, FIELD_EVIDENCE_TIER, &card.evidence_tier);
    push_missing_field(&mut diagnostics, FIELD_EVIDENCE_ORIGIN, &card.evidence_origin);
    push_missing_field(&mut diagnostics, FIELD_STATUS, &card.status);
    push_missing_collection(&mut diagnostics, FIELD_TRUST_BOUNDARIES, &card.trust_boundaries);
    push_missing_collection(&mut diagnostics, FIELD_NON_CLAIMS, &card.non_claims);
    push_missing_field(&mut diagnostics, FIELD_ARTIFACT_VERIFIER_COMMAND, &card.artifact_verifier_command);
    push_missing_field(&mut diagnostics, FIELD_PROVENANCE_ARTIFACT, &card.provenance_artifact);
    diagnostics
}

fn proof_card_mode_diagnostics(card: &ProofCard) -> Vec<String> {
    if card.verifier_mode == VERIFIER_MODE_FIXTURE && !card.checked_rust_source_scope.contains("fixture-only") {
        return vec!["proof card fixture verifier mode missing fixture-only checked scope".to_owned()];
    }
    if card.verifier_mode == VERIFIER_MODE_PRODUCTION && !card.checked_rust_source_scope.contains("production") {
        return vec!["proof card production verifier mode missing production checked scope".to_owned()];
    }
    if card.verifier_mode == VERIFIER_MODE_EXTERNAL && !card.checked_rust_source_scope.contains("external-trellis") {
        return vec!["proof card external verifier mode missing external-trellis checked scope".to_owned()];
    }
    Vec::new()
}

fn proof_card_non_claim_diagnostics(card: &ProofCard) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for required in default_card_non_claims() {
        if !card.non_claims.iter().any(|claim| claim == &required) {
            diagnostics.push(format!("proof card field `{FIELD_NON_CLAIMS}` missing `{required}`"));
        }
    }
    let joined = card.non_claims.join(" ").to_lowercase();
    push_overclaim(&mut diagnostics, &joined, "proves whole-program correctness", "whole-program correctness");
    push_overclaim(&mut diagnostics, &joined, "provides certification", "certification");
    push_overclaim(&mut diagnostics, &joined, "provides business-domain validation", "business-domain validation");
    push_overclaim(&mut diagnostics, &joined, "proves semantic equivalence", "semantic equivalence");
    push_overclaim(&mut diagnostics, &joined, "guarantees full safety", "full safety");
    diagnostics
}

fn proof_card_imported_reference_diagnostics(card: &ProofCard) -> Vec<String> {
    if card.evidence_origin != EVIDENCE_ORIGIN_EXTERNAL_TRELLIS {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    if card.evidence_tier == EvidenceTier::FormalProof.as_str() {
        diagnostics.push(
            "imported Trellis proof card must not report formal_proof without an Octet-owned rerun receipt".to_owned(),
        );
    }
    let Some(reference) = card.external_reference.as_ref() else {
        diagnostics.push(format!("proof card missing required field `{FIELD_EXTERNAL_REFERENCE}`"));
        return diagnostics;
    };
    push_missing_field(&mut diagnostics, "external_reference.repository_label", &reference.repository_label);
    push_missing_field(&mut diagnostics, "external_reference.repository_revision", &reference.repository_revision);
    push_missing_field(&mut diagnostics, "external_reference.hash_algorithm", &reference.hash_algorithm);
    push_missing_collection(
        &mut diagnostics,
        "external_reference.promotion_command_argv",
        &reference.promotion_command_argv,
    );
    if reference.hash_algorithm != HASH_ALGORITHM_BLAKE3 {
        diagnostics.push(format!(
            "proof card external_reference.hash_algorithm must be `{HASH_ALGORITHM_BLAKE3}`, got `{}`",
            reference.hash_algorithm
        ));
    }
    if !card.non_claims.iter().any(|claim| claim == NON_CLAIM_BLAKE3_IDENTITY) {
        diagnostics.push(format!("proof card field `{FIELD_NON_CLAIMS}` missing `{NON_CLAIM_BLAKE3_IDENTITY}`"));
    }
    diagnostics
}

fn proof_card_linkage_diagnostics(
    card: &ProofCard,
    receipt: Option<&VerusProofReceipt>,
    provenance: Option<&ProvenanceRecord>,
) -> Vec<String> {
    if card.evidence_tier != EvidenceTier::FormalProof.as_str() {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    let Some(receipt) = receipt else {
        return vec!["formal proof card missing parsed structured receipt".to_owned()];
    };
    let Some(provenance) = provenance else {
        return vec!["formal proof card missing matching formal_proof provenance".to_owned()];
    };

    if card.receipt_path != receipt.proof_artifact_identity {
        diagnostics.push(format!(
            "proof card receipt_path `{}` does not match receipt proof_artifact_identity `{}`",
            card.receipt_path, receipt.proof_artifact_identity
        ));
    }
    if card.receipt_path != provenance.artifact_locator {
        diagnostics.push(format!(
            "proof card receipt_path `{}` does not match provenance artifact `{}`",
            card.receipt_path, provenance.artifact_locator
        ));
    }
    if !card_has_identity(card, &provenance.artifact_locator, &provenance.blake3) {
        diagnostics.push("proof card formal_proof status missing receipt BLAKE3 artifact identity".to_owned());
    }
    for binding in &receipt.source_bindings {
        if !card_has_identity(card, &binding.path, &binding.blake3) {
            diagnostics.push(format!("proof card missing source BLAKE3 artifact identity `{}`", binding.path));
        }
    }
    for binding in &receipt.proof_bindings {
        if !card_has_identity(card, &binding.path, &binding.blake3) {
            diagnostics.push(format!("proof card missing proof BLAKE3 artifact identity `{}`", binding.path));
        }
    }
    if provenance.evidence_tier != EvidenceTier::FormalProof {
        diagnostics.push("proof card formal_proof status missing formal_proof provenance tier".to_owned());
    }
    if provenance.proof_artifact_identity.as_deref() != Some(card.receipt_path.as_str()) {
        diagnostics
            .push("proof card formal_proof status missing matching provenance proof artifact identity".to_owned());
    }
    if provenance.verifier_identity.as_deref() != Some(receipt.verifier_identity.verifier_id.as_str()) {
        diagnostics.push("proof card formal_proof status missing matching provenance verifier identity".to_owned());
    }
    if card.source_paths != receipt_paths(&receipt.source_bindings) {
        diagnostics.push("proof card source_paths do not match receipt source bindings".to_owned());
    }
    if card.proof_target_paths != receipt_paths(&receipt.proof_bindings) {
        diagnostics.push("proof card proof_target_paths do not match receipt proof bindings".to_owned());
    }
    if card.policy_id != receipt.policy_id || card.rail_index != receipt.rail_index {
        diagnostics.push("proof card policy identity does not match receipt identity".to_owned());
    }
    if card.checked_rust_source_scope != receipt.checked_scope
        || card.checked_rust_source_scope != provenance.checked_scope
    {
        diagnostics.push("proof card checked scope does not match receipt and provenance".to_owned());
    }
    if card.status != PROOF_STATUS_PASSED || receipt.status != PROOF_STATUS_PASSED {
        diagnostics.push("proof card formal_proof status requires passed receipt status".to_owned());
    }
    diagnostics
}

fn formal_proof_envelope_diagnostics(card: &ProofCard) -> Vec<String> {
    let Some(envelope) = &card.formal_proof_envelope else {
        return Vec::new();
    };
    let mut diagnostics = validate_formal_proof_envelope(envelope);
    if envelope.provider_id == octet_standards::formal_proof::AENEAS_LEAN_PROVIDER_ID
        && card.evidence_tier == EvidenceTier::FormalProof.as_str()
        && envelope.evidence_tier != octet_standards::formal_proof::FormalProofEvidenceTier::FormalProof
    {
        diagnostics.push("Aeneas formal proof card tier does not match its envelope tier".to_owned());
    }
    diagnostics
}

fn parse_formal_proof_envelope(text: &str) -> Result<FormalProofEnvelope, String> {
    serde_json::from_str(text).map_err(|error| format!("parse formal proof envelope for card: {error}"))
}

fn card_has_identity(card: &ProofCard, artifact: &str, blake3: &str) -> bool {
    card.artifact_blake3_identities
        .iter()
        .any(|identity| identity.artifact == artifact && identity.blake3 == blake3)
}

fn push_missing_field(diagnostics: &mut Vec<String>, field_name: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("proof card missing required field `{field_name}`"));
    }
}

fn push_missing_collection(diagnostics: &mut Vec<String>, field_name: &str, values: &[String]) {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        diagnostics.push(format!("proof card missing required field `{field_name}`"));
    }
}

fn push_missing_identity_collection(
    diagnostics: &mut Vec<String>,
    field_name: &str,
    values: &[ProofCardArtifactIdentity],
) {
    if values.is_empty()
        || values.iter().any(|value| value.artifact.trim().is_empty() || value.blake3.trim().is_empty())
    {
        diagnostics.push(format!("proof card missing required field `{field_name}`"));
    }
}

fn push_overclaim(diagnostics: &mut Vec<String>, text: &str, needle: &str, claim: &str) {
    if text.contains(needle) {
        diagnostics.push(format!("proof card contains unscoped {claim} claim"));
    }
}

fn proof_or_property_id(policy_id: &str, rail_index: usize) -> String {
    format!("{policy_id}#rail-{rail_index}")
}

fn identity_summary(label: &str, digest: &str) -> String {
    if digest.trim().is_empty() {
        label.to_owned()
    } else {
        format!("{label} ({digest})")
    }
}

fn card_non_claims(receipt_claims: &[String]) -> Vec<String> {
    let mut claims = receipt_claims.to_vec();
    for claim in default_card_non_claims() {
        if !claims.iter().any(|existing| existing == &claim) {
            claims.push(claim);
        }
    }
    claims
}

fn default_card_non_claims() -> Vec<String> {
    [
        NON_CLAIM_WHOLE_PROGRAM,
        NON_CLAIM_CERTIFICATION,
        NON_CLAIM_BUSINESS_DOMAIN,
        NON_CLAIM_SEMANTIC_EQUIVALENCE,
        NON_CLAIM_FULL_SAFETY,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn receipt_paths(bindings: &[octet_standards::proof::VerusProofBinding]) -> Vec<String> {
    bindings.iter().map(|binding| binding.path.clone()).collect()
}

fn parse_verus_receipt(text: &str) -> Result<VerusProofReceipt, String> {
    serde_json::from_str(text).map_err(|error| format!("parse proof receipt for card: {error}"))
}

fn write_artifact_text(root: &ArtifactRoot, locator: &str, text: &str) -> Result<(), String> {
    write_artifact_bytes(root, locator, text.as_bytes())
}

fn write_artifact_bytes(root: &ArtifactRoot, locator: &str, bytes: &[u8]) -> Result<(), String> {
    let relative = RootRelativePath::new(locator)?;
    debug_assert!(
        relative
            .as_path()
            .components()
            .next()
            .is_some_and(|top| PROVE_BUNDLE_ENTRY_NAMES.iter().any(|name| top.as_os_str() == *name)),
        "every prove output must sit under PROVE_BUNDLE_ENTRY_NAMES so a later run can prune it: {locator}"
    );
    root.write(&relative, bytes)
}

fn json_text<T: Serialize>(value: &T) -> Result<String, String> {
    let text = serde_json::to_string_pretty(value).map_err(|error| format!("serialize JSON: {error}"))?;
    Ok(format!("{text}\n"))
}

fn print_report(output_format: OutputFormat, report: &ProveReport) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet prove artifact_dir={} status={} proofs={} diagnostics={}",
                report.artifact_dir,
                report.status,
                report.proofs.len(),
                report.diagnostics.len()
            );
            for proof in &report.proofs {
                println!(
                    "proof policy_id={} rail_index={} backend={} verifier_mode={} status={} receipt={} checked_scope={}",
                    proof.policy_id,
                    proof.rail_index,
                    proof.backend,
                    proof.verifier_mode,
                    proof.status,
                    proof.receipt,
                    proof.checked_scope
                );
            }
            for diagnostic in &report.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            Ok(())
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report).map_err(|error| format!("serialize report: {error}"))?);
            Ok(())
        }
        OutputFormat::Sarif => Err("prove does not support SARIF output; use human or json".to_owned()),
    }
}

fn rail_list_report(selected_rails: &[SelectedRail<'_>], diagnostics: Vec<String>) -> RailListReport {
    RailListReport {
        schema_version: REPORT_SCHEMA_VERSION,
        selected_rails: selected_rails.iter().map(rail_list_row).collect(),
        diagnostics,
    }
}

fn rail_list_row(selected: &SelectedRail<'_>) -> RailListRow {
    RailListRow {
        policy_id: selected.policy.policy_id.clone(),
        rail_index: selected.rail_index,
        backend: selected.rail.backend.clone(),
        source_paths: selected.rail.source_paths.clone(),
        proof_paths: selected.rail.proof_paths.clone(),
        receipt_paths: proof_artifact_receipt_paths(selected.rail),
        verifier_label: selected.rail.receipt_verifier.verifier_id.clone(),
        verifier_mode: verifier_mode(selected.rail).to_owned(),
        checked_scope: checked_scope(selected.policy, selected.rail_index, selected.rail),
        non_claim: "scoped proof evidence only; not whole-program correctness, certification, semantic equivalence, or full safety".to_owned(),
    }
}

fn list_diagnostics(selected_rails: &[SelectedRail<'_>], sources: &[SourceFile]) -> Vec<String> {
    selected_rails
        .iter()
        .flat_map(|selected| path_pattern_diagnostics(&selected.rail.source_paths, sources, "source_paths"))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn print_rail_list(output_format: OutputFormat, report: &RailListReport) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => {
            println!(
                "octet prove rails selected={} diagnostics={}",
                report.selected_rails.len(),
                report.diagnostics.len()
            );
            for rail in &report.selected_rails {
                println!(
                    "rail policy_id={} rail_index={} backend={} verifier={} mode={} scope={} sources={} proofs={}",
                    rail.policy_id,
                    rail.rail_index,
                    rail.backend,
                    rail.verifier_label,
                    rail.verifier_mode,
                    rail.checked_scope,
                    rail.source_paths.join(","),
                    rail.proof_paths.join(",")
                );
                println!("non-claim: {}", rail.non_claim);
            }
            for diagnostic in &report.diagnostics {
                println!("diagnostic: {diagnostic}");
            }
            Ok(())
        }
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(report).map_err(|error| format!("serialize report: {error}"))?);
            Ok(())
        }
        OutputFormat::Sarif => Err("prove does not support SARIF output; use human or json".to_owned()),
    }
}

fn command_display(rail: &ProofRail) -> String {
    if rail.command_argv.is_empty() {
        rail.command.clone()
    } else {
        rail.command_argv.join(" ")
    }
}

#[cfg(test)]
mod tests {
    use octet_standards::model::Classification;
    use octet_standards::model::EnforcementLevel;
    use octet_standards::model::EnforcementMode;
    use octet_standards::model::FixtureCoverage;
    use octet_standards::model::FixtureStatus;
    use octet_standards::model::ReceiptProvenance;
    use octet_standards::model::ReceiptVerifier;
    use octet_standards::model::Severity;
    use octet_standards::model::SuppressionRequirements;

    use super::*;
    use crate::artifact::ArtifactBundleInput;
    use crate::artifact::BundleFile;
    use crate::artifact::verify_bundle;

    const POLICY_ID: &str = "octet.verus_proof_rails.fixture";
    const PRODUCTION_NEGATIVE_POLICY_ID: &str = "octet.verus_proof_rails.fixture-production-negative";
    const PRODUCTION_VERIFIER_ID: &str = "verus@production-fixture";
    const SOURCE_RELATIVE_PATH: &str = "src/verified/addition.rs";
    const PROOF_RELATIVE_PATH: &str = "verus/proofs/addition_bound.rs";
    const PASS_SIGNAL: &str = "proof-result: pass fixture-verus";
    const WRONG_PASS_SIGNAL: &str = "proof-result: pass never-printed";
    const VERIFIER_ID: &str = "octet-verus@fixture-v1";
    const TRUST_BOUNDARY: &str = "fixture verifies the proof plumbing only";
    const SHORT_TIMEOUT_MS: u64 = 1;
    const TEST_TIMEOUT_MS: u64 = 1_000;
    const TEST_LOG_LIMIT_BYTES: usize = 128;
    const PIPE_OVERFLOW_BYTES: usize = 200_000;
    const DESCENDANT_SLEEP_SECS: u64 = 20;
    const DESCENDANT_TEARDOWN_BOUND_SECS: u64 = 10;
    const AENEAS_FIXTURE_TIMEOUT_MS: u64 = 2_700_000;
    const AENEAS_FIXTURE_LOG_LIMIT_BYTES: usize = 1_048_576;
    const AENEAS_CONSUMER_TRUST_BOUNDARY: &str =
        "the external consumer proves only its named theorem over exact bound inputs";
    const VERUS_V1_RECEIPT_GOLDEN_BLAKE3: &str = "adc4998ee90cf9a1165c456dd2c0696f447e9c743c97bb27bbac4582cfda5cf8";
    const VERUS_V1_PROOF_CARDS_GOLDEN_BLAKE3: &str = "572f78530d72c67d628c6af965f2caa750af76bb11ab2a41854a4f9b9b30baac";

    // r[verify octet.aeneas_consumer_contract.execution]
    // r[verify octet.aeneas_consumer_contract.valence_handoff]
    #[cfg(unix)]
    #[test]
    #[ignore = "requires the pinned Aeneas/Charon/Lean provider closure"]
    fn live_provider_builds_external_consumer_binding() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../standards/proof-rails/aeneas-lean/fixtures/external-consumer")
            .canonicalize()
            .expect("external consumer fixture root");
        let profile = octet_standards::aeneas_consumer::selected_profile().expect("consumer profile");
        let theorem = profile.theorems.first().expect("consumer theorem");
        let source = SourceFile::new(
            profile.source.paths[0].path.clone(),
            fs::read_to_string(root.join(&profile.source.paths[0].path)).expect("consumer source"),
        );
        let theorem_source = SourceFile::new(
            theorem.path.clone(),
            fs::read_to_string(root.join(&theorem.path)).expect("consumer theorem source"),
        );
        let rail = ProofRail {
            backend: AENEAS_LEAN_BACKEND.to_owned(),
            command: "cargo octet prove --rail aeneas-external-consumer".to_owned(),
            command_argv: vec![
                "cargo".to_owned(),
                "octet".to_owned(),
                "prove".to_owned(),
                "--rail".to_owned(),
                "aeneas-external-consumer".to_owned(),
            ],
            source_paths: vec![source.path.clone()],
            proof_paths: vec![theorem_source.path.clone()],
            theorem_id: theorem.theorem_id.clone(),
            approved_theorem_statement_blake3: theorem
                .approved_statement_blake3
                .strip_prefix("b3:")
                .expect("prefixed theorem identity")
                .to_owned(),
            receipt_paths: vec!["proofs/aeneas-consumer.receipt".to_owned()],
            pass_signal: "aeneas-proof-result: pass".to_owned(),
            trust_boundaries: vec![AENEAS_CONSUMER_TRUST_BOUNDARY.to_owned()],
            verifier_mode: VERIFIER_MODE_FIXTURE.to_owned(),
            checked_scope: format!("formal-proof:external-consumer:{AENEAS_CONSUMER_POLICY_ID}#rail-0"),
            timeout_ms: profile.resource_policy.timeout_ms,
            stdout_max_bytes: profile.resource_policy.stdout_max_bytes,
            stderr_max_bytes: profile.resource_policy.stderr_max_bytes,
            environment_allowlist: profile.resource_policy.environment_allowlist.clone(),
            ..ProofRail::default()
        };
        let mut policy = fixture_policy_stub();
        policy.policy_id = AENEAS_CONSUMER_POLICY_ID.to_owned();
        policy.proof_rails = vec![rail];
        let selected = SelectedRail {
            policy: &policy,
            rail_index: 0,
            rail: &policy.proof_rails[0],
        };
        let summary = execute_aeneas_from_environment(selected.rail, &root);
        assert!(summary.command_succeeded, "external consumer provider failed: {summary:?}");
        let (receipt, _) = build_aeneas_receipt(
            &selected,
            std::slice::from_ref(&source),
            std::slice::from_ref(&theorem_source),
            &selected.rail.checked_scope,
            &summary,
        )
        .expect("external consumer formal receipt");
        let envelope: FormalProofEnvelope = serde_json::from_str(&receipt).expect("typed external consumer receipt");
        let consumer = envelope.consumer.as_ref().expect("consumer binding");
        assert_eq!(consumer.profile_identity, octet_standards::aeneas_consumer::PROFILE_IDENTITY);
        assert_eq!(consumer.outcome, octet_standards::aeneas_consumer::ConsumerOutcome::Supported);
        assert!(validate_formal_proof_envelope(&envelope).is_empty());
    }

    #[test]
    fn aeneas_theorem_name_extraction_accepts_named_theorem() {
        assert_eq!(
            extract_theorem_name("theorem safe_add_matches_model : True := by rfl"),
            Some("safe_add_matches_model".to_owned())
        );
    }

    #[test]
    fn aeneas_theorem_name_and_adequacy_select_the_same_multi_line_theorem() {
        let named = "theorem safe_add_matches_model (x : Nat) :\n    True := by\n  trivial\n";
        let with_helper = format!("{named}\ntheorem helper (x : Nat) : x = x := by rfl\n");
        for source in [named, with_helper.as_str()] {
            assert_eq!(extract_theorem_name(source), Some("safe_add_matches_model".to_owned()), "source: {source}");
            assert_eq!(
                assess_aeneas_theorem_source(source).decision,
                octet_standards::formal_proof::AeneasTheoremAdequacyDecision::Vacuous,
                "source: {source}"
            );
        }
    }

    #[test]
    fn aeneas_theorem_name_extraction_rejects_missing_theorem() {
        assert_eq!(extract_theorem_name("def safe_add := 3"), None);
    }

    // r[verify octet.aeneas_lean_proof_rail.verus_compatibility]
    #[test]
    fn verus_v1_compatibility_golden_preserves_receipt_card_and_artifact_verification() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let artifact_dir = temp.path().join("artifacts");

        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            rail: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Json,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");
        assert_eq!(code, EXIT_PROOF_VALID);
        let receipt_path = artifact_dir
            .join(proof_artifact_locator(&fixture_policy(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL)), 0));
        let receipt = fs::read(receipt_path).expect("read receipt");
        let cards = fs::read(artifact_dir.join(PROOF_CARDS_ARTIFACT_NAME)).expect("read proof cards");
        let report = verify_bundle(bundle_input(&artifact_dir));

        assert_eq!(crate::evidence::artifact_blake3_hex(&receipt), VERUS_V1_RECEIPT_GOLDEN_BLAKE3);
        assert_eq!(crate::evidence::artifact_blake3_hex(&cards), VERUS_V1_PROOF_CARDS_GOLDEN_BLAKE3);
        assert_eq!(report.status, "valid");
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    // The test writes one proof bundle and checks every emitted artifact against it.
    #[allow(clippy::too_many_lines)]
    fn prove_writes_valid_formal_proof_artifact_bundle() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let artifact_dir = temp.path().join("artifacts");

        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            rail: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Json,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_VALID);
        let receipt_text = fs::read_to_string(
            artifact_dir
                .join(proof_artifact_locator(&fixture_policy(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL)), 0)),
        )
        .expect("read receipt");
        assert!(receipt_text.contains("source_bindings"));
        assert!(receipt_text.contains("proof_bindings"));
        assert!(receipt_text.contains("stdout_artifact"));
        let cards_text = fs::read_to_string(artifact_dir.join(PROOF_CARDS_ARTIFACT_NAME)).expect("read proof cards");
        let cards: ProofCardDeck = serde_json::from_str(&cards_text).expect("parse proof cards");
        assert_eq!(cards.schema_version, PROOF_CARDS_SCHEMA_VERSION);
        let card = cards.cards.first().expect("proof card");
        assert_eq!(card.proof_or_property_id, proof_or_property_id(POLICY_ID, 0));
        assert_eq!(card.source_requirement_or_policy_id, POLICY_ID);
        assert_eq!(card.checked_rust_source_scope, format!("formal-proof:fixture-only:{POLICY_ID}#rail-0"));
        assert_eq!(card.source_paths, vec![SOURCE_RELATIVE_PATH.to_owned()]);
        assert_eq!(card.proof_target_paths, vec![PROOF_RELATIVE_PATH.to_owned()]);
        assert_eq!(card.verifier_mode, VERIFIER_MODE_FIXTURE);
        assert!(card.verifier_identity_summary.contains(VERIFIER_ID));
        assert!(card.toolchain_identity_summary.contains("verus-fixture-wrapper/v1"));
        assert_eq!(
            card.receipt_path,
            proof_artifact_locator(&fixture_policy(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL)), 0)
        );
        assert!(card.artifact_blake3_identities.iter().any(|identity| identity.artifact == SOURCE_RELATIVE_PATH));
        assert!(card.artifact_blake3_identities.iter().any(|identity| identity.artifact == PROOF_RELATIVE_PATH));
        assert!(card.artifact_blake3_identities.iter().any(|identity| identity.artifact == card.receipt_path));
        assert_eq!(card.evidence_tier, EvidenceTier::FormalProof.as_str());
        assert_eq!(card.evidence_origin, EVIDENCE_ORIGIN_OCTET_RERUN);
        assert_eq!(card.status, STATUS_PASSED);
        assert!(card.trust_boundaries.iter().any(|boundary| boundary.contains(TRUST_BOUNDARY)));
        assert!(card.non_claims.iter().any(|claim| claim == NON_CLAIM_FULL_SAFETY));
        assert_eq!(card.artifact_verifier_command, ARTIFACT_VERIFY_COMMAND);
        assert_eq!(card.provenance_artifact, PROVENANCE_ARTIFACT_NAME);
        assert!(card.diagnostics.is_empty(), "unexpected proof card diagnostics: {:?}", card.diagnostics);
        let imported_card = cards
            .cards
            .iter()
            .find(|candidate| candidate.proof_or_property_id == "trellis.ucan-proof-reference.digest-admission")
            .expect("imported Trellis proof card");
        assert_eq!(imported_card.evidence_origin, EVIDENCE_ORIGIN_EXTERNAL_TRELLIS);
        assert_eq!(imported_card.evidence_tier, EvidenceTier::ManualReviewRequired.as_str());
        assert_eq!(imported_card.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert_eq!(imported_card.provenance_artifact, TRELLIS_IMPORT_PROVENANCE_ARTIFACT);
        assert!(imported_card.external_reference.is_some());
        assert!(
            imported_card
                .artifact_blake3_identities
                .iter()
                .any(|identity| identity.artifact.contains("ucan_proof_reference_source.rs"))
        );
        assert!(imported_card.non_claims.iter().any(|claim| claim == NON_CLAIM_BLAKE3_IDENTITY));
        assert!(
            imported_card.diagnostics.is_empty(),
            "unexpected imported proof card diagnostics: {:?}",
            imported_card.diagnostics
        );

        let provenance = fs::read_to_string(artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("read provenance");
        let records = provenance_records(&provenance);
        let formal_record = records
            .iter()
            .find(|record| record.evidence_tier == EvidenceTier::FormalProof)
            .expect("formal proof provenance");
        let parsed_receipt = parse_verus_receipt(&receipt_text).expect("parse receipt");
        let card_diagnostics = proof_card_validation_diagnostics(card, Some(&parsed_receipt), Some(formal_record));
        assert!(card_diagnostics.is_empty(), "unexpected card linkage diagnostics: {card_diagnostics:?}");
        assert!(records.iter().any(|record| {
            record.artifact_locator == PROOF_CARDS_ARTIFACT_NAME
                && record.evidence_tier == EvidenceTier::ManualReviewRequired
        }));
        let manifest_text = fs::read_to_string(artifact_dir.join(crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT))
            .expect("read evidence log manifest");
        let manifest: crate::evidence_log::EvidenceLogManifest =
            serde_json::from_str(&manifest_text).expect("parse evidence log manifest");
        assert_eq!(manifest.schema_version, crate::evidence_log::EVIDENCE_LOG_MANIFEST_SCHEMA_VERSION);
        let stdout_entry = manifest
            .logs
            .iter()
            .find(|entry| entry.artifact_locator.ends_with(".stdout"))
            .expect("stdout log manifest entry");
        assert_eq!(stdout_entry.replay_command, "cargo octet prove fixture");
        assert_eq!(stdout_entry.checked_scope, format!("formal-proof:fixture-only:{POLICY_ID}#rail-0"));
        let stdout_bytes =
            fs::read(artifact_dir.join(&stdout_entry.artifact_locator)).expect("read stdout log artifact");
        assert_eq!(stdout_entry.bytes, stdout_bytes.len());
        assert_eq!(stdout_entry.blake3, crate::evidence::artifact_blake3_hex(&stdout_bytes));
        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read prove summary");
        assert!(summary.contains("replay_command: cargo octet prove fixture"));
        assert!(summary.contains(EVIDENCE_LOG_CLAIM_BOUNDARY));
        assert!(summary.contains(&stdout_entry.blake3));

        let report = verify_bundle(bundle_input(&artifact_dir));
        assert_eq!(report.status, "valid", "unexpected diagnostics: {:?}", report.diagnostics);
        assert!(report.artifacts.iter().any(|artifact| artifact.evidence_tier == "formal_proof"));
        assert!(report.artifacts.iter().any(|artifact| artifact.artifact == PROOF_CARDS_ARTIFACT_NAME));
    }

    #[test]
    fn proof_card_validation_rejects_missing_fields_fixture_mode_and_overclaims() {
        let mut card = valid_card_fixture();
        card.evidence_tier = EvidenceTier::FormalProof.as_str().to_owned();
        card.receipt_path.clear();
        card.proof_target_paths.clear();
        card.artifact_blake3_identities.clear();
        card.checked_rust_source_scope = format!("formal-proof:production:{POLICY_ID}#rail-0");
        card.non_claims = vec!["proves whole-program correctness".to_owned()];

        let diagnostics = proof_card_validation_diagnostics(&card, None, None);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_RECEIPT_PATH)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_PROOF_TARGET_PATHS)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_ARTIFACT_BLAKE3_IDENTITIES)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("fixture-only checked scope")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("whole-program correctness")));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("formal proof card missing parsed structured receipt"))
        );
    }

    #[test]
    fn proof_card_linkage_rejects_stale_receipt_and_provenance_links() {
        let rail = fixture_rail(Path::new("."), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let sources = source_files_for_cli_tests();
        let proof_sources = proof_files_for_cli_tests();
        let receipt = build_verus_proof_receipt(&VerusProofReceiptInput {
            policy_id: POLICY_ID,
            rail_index: 0,
            rail: &rail,
            sources: &sources,
            proof_sources: &proof_sources,
            checked_scope: &format!("formal-proof:fixture-only:{POLICY_ID}#rail-0"),
            status: PROOF_STATUS_PASSED,
            command_blake3: "command-digest",
            proof_artifact_identity: &proof_artifact_locator(
                &fixture_policy(Path::new("."), PASS_SIGNAL, command_argv(PASS_SIGNAL)),
                0,
            ),
            stdout_artifact: log_artifact("proofs/test.stdout", PASS_SIGNAL, PASS_SIGNAL.len(), false),
            stderr_artifact: log_artifact("proofs/test.stderr", "", 0, false),
        })
        .expect("valid receipt");
        let receipt_text = serde_json::to_string_pretty(&receipt).expect("receipt text");
        let provenance = proof_provenance_record(
            &receipt.proof_artifact_identity,
            receipt_text.as_bytes(),
            &receipt.checked_scope,
            &receipt.verifier_identity.verifier_id,
        );
        let mut card = valid_card_fixture();
        card.evidence_tier = EvidenceTier::FormalProof.as_str().to_owned();
        card.receipt_path = "proofs/stale.receipt".to_owned();
        card.source_paths = receipt_paths(&receipt.source_bindings);
        card.proof_target_paths = receipt_paths(&receipt.proof_bindings);
        card.checked_rust_source_scope = receipt.checked_scope.clone();
        card.verifier_identity_summary =
            identity_summary(&receipt.verifier_identity.verifier_id, &receipt.verifier_identity.verifier_digest);
        card.toolchain_identity_summary =
            identity_summary(&receipt.toolchain_identity.toolchain_id, &receipt.toolchain_identity.toolchain_digest);
        card.artifact_blake3_identities = receipt_artifact_identities(&receipt, Some(&provenance));

        let diagnostics = proof_card_validation_diagnostics(&card, Some(&receipt), Some(&provenance));

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("does not match receipt proof_artifact_identity"))
        );
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("does not match provenance artifact")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("provenance proof artifact identity")));
    }

    #[test]
    fn proof_card_for_imported_trellis_bundle_stays_manual_review() {
        let bundle = imported_trellis_bundle();

        let card = proof_card_for_imported_bundle(&bundle);

        assert_eq!(card.proof_or_property_id, bundle.bundle_id);
        assert_eq!(card.verifier_mode, VERIFIER_MODE_EXTERNAL);
        assert_eq!(card.evidence_origin, EVIDENCE_ORIGIN_EXTERNAL_TRELLIS);
        assert_eq!(card.evidence_tier, EvidenceTier::ManualReviewRequired.as_str());
        assert_eq!(card.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert!(card.external_reference.is_some());
        assert!(
            card.artifact_blake3_identities
                .iter()
                .any(|identity| identity.artifact == "src/ucan_proof_reference.rs")
        );
        assert!(card.non_claims.iter().any(|claim| claim == NON_CLAIM_BLAKE3_IDENTITY));
        assert!(card.diagnostics.is_empty(), "unexpected imported card diagnostics: {:?}", card.diagnostics);
    }

    #[test]
    fn proof_card_validation_rejects_imported_trellis_formal_proof_overclaim() {
        let bundle = imported_trellis_bundle();
        let mut card = proof_card_for_imported_bundle(&bundle);
        card.evidence_tier = EvidenceTier::FormalProof.as_str().to_owned();

        let diagnostics = proof_card_validation_diagnostics(&card, None, None);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("imported Trellis proof card")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("missing parsed structured receipt")));
    }

    #[test]
    fn prove_fails_without_pass_signal_and_does_not_emit_formal_proof() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), WRONG_PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let artifact_dir = temp.path().join("artifacts");

        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            rail: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: true,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_INVALID);
        let provenance = fs::read_to_string(artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("read provenance");
        assert!(!provenance.contains("formal_proof"));
        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
        assert!(summary.contains("explain: proof command stdout missing pass signal"));
        let report = verify_bundle(bundle_input(&artifact_dir));
        assert_eq!(report.status, "invalid");
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("unexpected artifact")));
    }

    #[cfg(unix)]
    #[test]
    fn prove_production_verifier_failure_is_fail_closed() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let argv = write_fake_production_verifier(temp.path(), &pinned_production_identity(), "", false);
        let command = argv.join(" ");
        let registry = write_production_negative_registry(temp.path(), argv);
        let artifact_dir = temp.path().join("artifacts-production-negative");

        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            rail: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture-production-negative".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_INVALID);
        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
        assert!(summary.contains(&format!("verifier-failure: proof command `{command}` exited without success")));
        let status = fs::read_to_string(artifact_dir.join(STATUS_ARTIFACT_NAME)).expect("read status");
        assert!(status.contains("\"status\": \"failed\""));
        let provenance = fs::read_to_string(artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("read provenance");
        assert!(!provenance.contains("formal_proof"));
        let receipt_path = artifact_dir.join(format!(
            "{PROOF_ARTIFACT_DIR_NAME}/{}-rail-0.receipt",
            safe_artifact_stem(PRODUCTION_NEGATIVE_POLICY_ID)
        ));
        let receipt = fs::read_to_string(receipt_path).expect("read receipt");
        assert!(receipt.contains("\"status\": \"failed\""));
        assert!(receipt.contains("\"stdout_artifact\""));
        assert!(receipt.contains("\"stderr_artifact\""));
    }

    // r[verify octet.verus_proof_rails.safe_execution.failures]
    #[cfg(unix)]
    #[test]
    fn prove_production_rail_refuses_verifier_without_pinned_identity() {
        let pinned = pinned_production_identity();
        let cases = [
            ("no identity", PASS_SIGNAL.to_owned(), "proof-verifier-binary-blake3"),
            ("wrong binary", pinned.replace("production-fixture-binary-digest", "other-binary"), "other-binary"),
            ("wrong solver", pinned.replace("production-fixture-solver-digest", "other-solver"), "other-solver"),
            (
                "wrong toolchain",
                pinned.replace("production-fixture-toolchain-digest", "other-toolchain"),
                "other-toolchain",
            ),
        ];
        for (case, identity, named) in cases {
            let temp = tempfile::tempdir().expect("tempdir");
            let source = write_fixture_source(temp.path());
            write_fixture_proof(temp.path());
            let proof_ran = temp.path().join("proof-ran");
            let argv = write_fake_production_verifier(temp.path(), &identity, &proof_ran.display().to_string(), true);
            let registry = write_production_negative_registry(temp.path(), argv);
            let artifact_dir = temp.path().join("artifacts-unpinned");

            let code = run(&fixture_args(registry, vec![source], artifact_dir.clone())).expect("prove run");

            assert_eq!(code, EXIT_PROOF_INVALID, "{case}");
            assert!(!proof_ran.exists(), "{case}: unpinned verifier must not run the proof");
            let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
            assert!(summary.contains(DIAGNOSTIC_UNPINNED_VERIFIER), "{case}: {summary}");
            assert!(summary.contains(named), "{case}: {summary}");
            let provenance = fs::read_to_string(artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("read provenance");
            assert!(!provenance.contains("formal_proof"), "{case}: {provenance}");
        }
    }

    // r[verify octet.verus_proof_rails.safe_execution.failures]
    #[cfg(unix)]
    #[test]
    fn prove_production_rail_probes_verifier_whatever_its_backend_label() {
        for (backend, schema_version) in [
            ("local-command-receipt", "octet-verus-proof-receipt/v1"),
            ("local-command-receipt", "octet-local-command-receipt/v1"),
        ] {
            let case = format!("{backend} with {schema_version}");
            let temp = tempfile::tempdir().expect("tempdir");
            let source = write_fixture_source(temp.path());
            write_fixture_proof(temp.path());
            let proof_ran = temp.path().join("proof-ran");
            let argv = write_fake_production_verifier(temp.path(), PASS_SIGNAL, &proof_ran.display().to_string(), true);
            let mut registry = production_negative_registry(argv);
            let rail = &mut registry.policies[0].proof_rails[0];
            backend.clone_into(&mut rail.backend);
            schema_version.clone_into(&mut rail.receipt_provenance.schema_version);
            let registry_path = temp.path().join("registry-production-backend.json");
            fs::write(&registry_path, serde_json::to_string_pretty(&registry).expect("registry json"))
                .expect("write registry");
            let artifact_dir = temp.path().join("artifacts-backend");

            let code = run(&fixture_args(registry_path, vec![source], artifact_dir.clone())).expect("prove run");

            assert_eq!(code, EXIT_PROOF_INVALID, "{case}");
            assert!(!proof_ran.exists(), "{case}: unpinned verifier must not run the proof");
            let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
            assert!(summary.contains(DIAGNOSTIC_UNPINNED_VERIFIER), "{case}: {summary}");
            let provenance = fs::read_to_string(artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("read provenance");
            assert!(!provenance.contains("formal_proof"), "{case}: {provenance}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn prove_production_rail_accepts_verifier_with_pinned_identity() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let proof_ran = temp.path().join("proof-ran");
        let argv = write_fake_production_verifier(
            temp.path(),
            &pinned_production_identity(),
            &proof_ran.display().to_string(),
            true,
        );
        let registry = write_production_negative_registry(temp.path(), argv);
        let artifact_dir = temp.path().join("artifacts-pinned");

        let code = run(&fixture_args(registry, vec![source], artifact_dir.clone())).expect("prove run");

        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
        assert_eq!(code, 0, "{summary}");
        assert!(proof_ran.exists());
        let provenance = fs::read_to_string(artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("read provenance");
        assert!(provenance.contains("formal_proof"), "{provenance}");
    }

    #[test]
    fn prove_rejects_missing_command_argv_without_shell_execution() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, Vec::new());
        let artifact_dir = temp.path().join("artifacts");

        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            rail: Vec::new(),
            artifact_dir,
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_INVALID);
    }

    #[test]
    fn prove_reports_spawn_failure_nonzero_exit_and_timeout() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());

        let spawn = run_fixture_with_command(temp.path(), &source, vec!["missing-octet-verifier".to_owned()]);
        assert!(spawn.contains("spawn proof command"));

        let nonzero = run_fixture_with_command(temp.path(), &source, vec!["false".to_owned()]);
        assert!(nonzero.contains("exited without success"));

        let timeout = run_fixture_with_timeout(temp.path(), &source, vec!["sleep".to_owned(), "1".to_owned()]);
        assert!(timeout.contains("timed out"));
    }

    #[test]
    fn blocked_goal_state_explain_remediation_is_a_runnable_prove_command() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let explain_args = |summary: &str| {
            let explain = summary
                .lines()
                .filter_map(|line| line.strip_prefix("proof_goal_state remediation: "))
                .find(|command| command.starts_with("cargo octet prove "))
                .expect("blocked goal state names a prove --explain remediation");
            let shell_words = std::process::Command::new("sh")
                .arg("-c")
                .arg(format!("printf '%s\\0' {explain}"))
                .output()
                .expect("split remediation with sh");
            assert!(shell_words.status.success(), "sh could not split `{explain}`");
            let shell_argv = String::from_utf8(shell_words.stdout).expect("utf-8 shell words");
            let parsed = crate::cli::parse(shell_argv.trim_end_matches('\0').split('\0').map(std::ffi::OsString::from))
                .unwrap_or_else(|error| panic!("remediation `{explain}` must parse: {error}"));
            let crate::cli::CommandArgs::Prove(args) = parsed else {
                panic!("remediation `{explain}` must parse as prove")
            };
            args
        };

        let summary = run_fixture_with_command(temp.path(), &source, vec!["false".to_owned()]);
        let args = explain_args(&summary);
        assert!(args.explain);
        assert_eq!(args.rail, vec![proof_or_property_id(POLICY_ID, 0)]);
        assert_eq!(args.registry, Some(temp.path().join("registry.json")));
        assert_eq!(args.source, vec![source.clone()]);
        assert!(args.family.is_empty(), "the default family needs no flag: {:?}", args.family);
        assert!(
            args.artifact_dir.to_string_lossy().ends_with("-explain"),
            "explain rerun must not overwrite the reviewed bundle: {}",
            args.artifact_dir.display()
        );

        let family = "safety";
        let mut registry = fixture_registry(temp.path(), PASS_SIGNAL, vec!["false".to_owned()]);
        family.clone_into(&mut registry.policies[0].pillar);
        let registry_path = temp.path().join("registry-family.json");
        fs::write(&registry_path, serde_json::to_string_pretty(&registry).expect("registry json"))
            .expect("write registry");
        let artifact_dir = temp.path().join("artifacts-family");
        let code = run(&ProveArgs {
            registry: Some(registry_path),
            policy_artifacts_dir: None,
            source: vec![source],
            family: vec![family.to_owned()],
            rail: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");
        assert_eq!(code, EXIT_PROOF_INVALID);
        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("summary");
        let args = explain_args(&summary);
        assert_eq!(args.family, vec![family.to_owned()], "the rerun must keep the selected family");
        assert!(prove_selection(&args.family).includes(&registry.policies[0].pillar));
    }

    #[test]
    fn goal_state_checks_receipt_bytes_against_formal_provenance() {
        let temp = tempfile::tempdir().expect("tempdir");
        let policy = fixture_policy(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let (args, mut execution) = passing_fixture_execution(temp.path(), &policy);

        let satisfied = proof_goal_state_for_execution(&args, &execution);
        execution.formal_provenance.as_mut().expect("formal provenance").blake3 = "0".repeat(64);
        let stale = proof_goal_state_for_execution(&args, &execution);

        assert!(satisfied.emits_formal_proof, "unexpected blockers: {:?}", satisfied.blockers);
        assert!(!stale.emits_formal_proof);
        assert!(
            stale.missing_obligations.iter().any(|obligation| obligation.contains("stale artifact")),
            "provenance digest that disagrees with receipt bytes must block: {:?}",
            stale.blockers
        );
    }

    #[test]
    fn goal_state_blocks_on_demoted_proof_card_diagnostics() {
        let temp = tempfile::tempdir().expect("tempdir");
        let policy = fixture_policy(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let (args, mut execution) = passing_fixture_execution(temp.path(), &policy);
        VERIFIER_MODE_PRODUCTION.clone_into(&mut execution.row.verifier_mode);

        let card = proof_card_for_execution(&execution);
        let state = proof_goal_state_for_execution(&args, &execution);

        assert!(execution.row.diagnostics.is_empty());
        assert_eq!(card.evidence_tier, EvidenceTier::ManualReviewRequired.as_str());
        assert!(!card.diagnostics.is_empty());
        assert!(!state.emits_formal_proof);
        for diagnostic in &card.diagnostics {
            assert!(
                state.blockers.contains(&format!("diagnostic blocker: {diagnostic}")),
                "card diagnostic `{diagnostic}` missing from goal-state blockers: {:?}",
                state.blockers
            );
        }
    }

    type ExecutionTamper = fn(&mut ExecutedProof);

    #[test]
    fn blocked_goal_withholds_formal_proof_provenance_and_bundle_still_verifies() {
        let cases: [(&str, ExecutionTamper); 3] = [
            ("clean", |_| {}),
            ("demoted card", |execution| {
                VERIFIER_MODE_PRODUCTION.clone_into(&mut execution.row.verifier_mode);
                execution.extra_artifacts.push(GeneratedProofArtifact {
                    locator: format!("{PROOF_ARTIFACT_DIR_NAME}/fixture-stage.bin"),
                    contents: b"stage artifact\n".to_vec(),
                });
            }),
            ("stale provenance digest", |execution| {
                execution.formal_provenance.as_mut().expect("formal provenance").blake3 = "0".repeat(64);
            }),
        ];
        for (case, tamper) in cases {
            let temp = tempfile::tempdir().expect("tempdir");
            let policy = fixture_policy(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
            let (args, mut execution) = passing_fixture_execution(temp.path(), &policy);
            tamper(&mut execution);
            let receipt = execution.row.receipt.clone();
            let mut executions = vec![execution];

            let goal_states = settle_proof_goals(&args, &mut executions);
            let report = prove_report(&args, Vec::new(), &executions);
            write_artifacts(&args, &report, &executions, &goal_states).expect("write prove bundle");

            let provenance = fs::read_to_string(args.artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("provenance");
            let records = provenance
                .lines()
                .map(|line| serde_json::from_str::<ProvenanceRecord>(line).expect("provenance record"))
                .collect::<Vec<_>>();
            let receipt_record =
                records.iter().find(|record| record.artifact_locator == receipt).expect("receipt record");
            let status: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(args.artifact_dir.join(STATUS_ARTIFACT_NAME)).expect("status"),
            )
            .expect("status json");
            let cards: ProofCardDeck = serde_json::from_str(
                &fs::read_to_string(args.artifact_dir.join(PROOF_CARDS_ARTIFACT_NAME)).expect("cards"),
            )
            .expect("cards json");
            let summary = fs::read_to_string(args.artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("summary");
            let verification = verify_bundle(bundle_input(&args.artifact_dir));

            assert_eq!(verification.status, "valid", "{case}: {:?}", verification.diagnostics);
            if case == "clean" {
                assert_eq!(receipt_record.evidence_tier, EvidenceTier::FormalProof, "{case}");
                assert_eq!(report.status, STATUS_PASSED, "{case}");
                assert_eq!(status["proofs"][0]["status"], STATUS_PASSED, "{case}");
                assert_eq!(cards.cards[0].evidence_tier, EvidenceTier::FormalProof.as_str(), "{case}");
                assert!(summary.contains("emits_formal_proof=true"), "{case}: {summary}");
                continue;
            }
            assert!(
                records.iter().all(|record| record.evidence_tier != EvidenceTier::FormalProof),
                "{case}: blocked rail must not emit formal_proof provenance: {records:?}"
            );
            assert_eq!(receipt_record.evidence_tier, EvidenceTier::ManualReviewRequired, "{case}");
            assert!(
                receipt_record
                    .human_action
                    .as_deref()
                    .is_some_and(|action| action.contains(WITHHELD_FORMAL_PROOF_ACTION)),
                "{case}: {receipt_record:?}"
            );
            assert_eq!(report.status, STATUS_FAILED, "{case}");
            assert_eq!(status["proofs"][0]["status"], STATUS_MANUAL_REVIEW_REQUIRED, "{case}");
            assert_eq!(cards.cards[0].evidence_tier, EvidenceTier::ManualReviewRequired.as_str(), "{case}");
            assert!(summary.contains("emits_formal_proof=false"), "{case}: {summary}");
        }
    }

    #[test]
    fn prove_drains_verifier_output_larger_than_pipe_buffer() {
        let temp = tempfile::tempdir().expect("tempdir");
        let script = format!(
            "printf '%s\\n' '{PASS_SIGNAL}'; head -c {PIPE_OVERFLOW_BYTES} /dev/zero; head -c {PIPE_OVERFLOW_BYTES} /dev/zero >&2"
        );
        let rail = fixture_rail(temp.path(), PASS_SIGNAL, vec!["sh".to_owned(), "-c".to_owned(), script]);

        let outcome = run_proof_command(&rail, temp.path());

        assert!(!outcome.timed_out, "large verifier output must not stall until timeout: {:?}", outcome.diagnostics);
        assert!(outcome.command_succeeded, "unexpected diagnostics: {:?}", outcome.diagnostics);
        assert!(outcome.stdout_text.contains(PASS_SIGNAL));
        assert_eq!(outcome.stdout_original_bytes, PASS_SIGNAL.len() + 1 + PIPE_OVERFLOW_BYTES);
        assert!(outcome.stdout_truncated);
        assert_eq!(outcome.stdout_text.len(), TEST_LOG_LIMIT_BYTES);
        assert_eq!(outcome.stderr_original_bytes, PIPE_OVERFLOW_BYTES);
        assert!(outcome.stderr_truncated);
        assert_eq!(outcome.stderr_text.len(), TEST_LOG_LIMIT_BYTES);
    }

    #[test]
    fn prove_teardown_does_not_wait_for_descendants_holding_output_pipes() {
        let temp = tempfile::tempdir().expect("tempdir");
        let exits_with_background_child = format!("sleep {DESCENDANT_SLEEP_SECS} & printf '%s\\n' '{PASS_SIGNAL}'");
        let times_out_waiting_for_child = format!("sleep {DESCENDANT_SLEEP_SECS} & wait");

        for (script, expect_timeout) in [
            (exits_with_background_child, false),
            (times_out_waiting_for_child, true),
        ] {
            let rail = fixture_rail(temp.path(), PASS_SIGNAL, vec!["sh".to_owned(), "-c".to_owned(), script.clone()]);
            let started = std::time::Instant::now();

            let outcome = run_proof_command(&rail, temp.path());

            assert!(
                started.elapsed() < std::time::Duration::from_secs(DESCENDANT_TEARDOWN_BOUND_SECS),
                "`{script}` teardown waited for a descendant holding the output pipes"
            );
            assert_eq!(outcome.timed_out, expect_timeout, "`{script}`: {:?}", outcome.diagnostics);
            assert_eq!(outcome.command_succeeded, !expect_timeout, "`{script}`: {:?}", outcome.diagnostics);
        }
    }

    #[test]
    fn selected_rails_use_shared_receipt_path_relevance() {
        let mut policy = fixture_policy_stub();
        let receipt_path = proof_artifact_locator(&policy, 0);
        policy.proof_rails = vec![ProofRail {
            source_paths: vec!["src/verified/missing.rs".to_owned()],
            proof_paths: vec![receipt_path.clone()],
            receipt_paths: Vec::new(),
            ..ProofRail::default()
        }];
        let registry = PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec![CHECKER_OCTET_STANDARDS.to_owned()],
            policies: vec![policy],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        };
        let sources = vec![SourceFile::new(receipt_path, "proof-result: pass\n")];

        let rail_selection = prove_core::selected_rail_indices(
            &registry,
            &sources,
            &FamilySelection::from_families(["verification"]),
            &[],
        );
        let rails = selected_rails(&registry, &rail_selection.selected);

        assert!(rail_selection.diagnostics.is_empty(), "unexpected diagnostics: {:?}", rail_selection.diagnostics);
        assert_eq!(rails.len(), 1);
        assert_eq!(rail_list_row(&rails[0]).receipt_paths, vec![rails[0].rail.proof_paths[0].clone()]);
    }

    #[test]
    fn prove_rail_selector_runs_matching_policy_rail() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let artifact_dir = temp.path().join("artifacts-rail-selector");

        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            rail: vec![format!("{POLICY_ID}#rail-0")],
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove --rail fixture".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_VALID);
        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
        assert!(summary.contains("status=passed"));
        assert!(summary.contains(&format!("policy_id={POLICY_ID} rail_index=0")));
    }

    #[test]
    fn prove_rail_selector_missing_fails_without_verifier_execution() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, vec!["false".to_owned()]);
        let artifact_dir = temp.path().join("artifacts-missing-rail-selector");

        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            rail: vec!["missing.policy#rail-0".to_owned()],
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove --rail missing".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_INVALID);
        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
        assert!(summary.contains("matched no selected proof rail"));
        assert!(!summary.contains(DIAGNOSTIC_VERIFIER_FAILURE));
    }

    #[test]
    fn prove_rail_selector_partial_miss_fails_before_any_verifier_execution() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, vec!["false".to_owned()]);
        let artifact_dir = temp.path().join("artifacts-partial-rail-selector");

        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            rail: vec![format!("{POLICY_ID}#rail-0"), "missing.policy#rail-0".to_owned()],
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove --rail fixture --rail missing".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");

        assert_eq!(code, EXIT_PROOF_INVALID);
        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
        assert!(summary.contains("proofs=0"));
        assert!(summary.contains("matched no selected proof rail"));
        assert!(!summary.contains(DIAGNOSTIC_VERIFIER_FAILURE));
    }

    #[test]
    fn prove_list_reports_selected_rail_without_artifacts() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let artifact_dir = temp.path().join("artifacts");

        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source],
            family: Vec::new(),
            rail: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Json,
            command_line: "cargo octet prove --list fixture".to_owned(),
            list: true,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove list");

        assert_eq!(code, EXIT_PROOF_VALID);
        assert!(!artifact_dir.exists(), "list mode should not write artifact bundles");
    }

    // r[verify octet.verus_proof_rails.safe_execution.failures]
    #[test]
    fn prove_fails_closed_when_verifier_output_exceeds_log_limit() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let script = format!("printf '%s\\n' '{PASS_SIGNAL}'; head -c {PIPE_OVERFLOW_BYTES} /dev/zero");
        let argv = vec!["sh".to_owned(), "-c".to_owned(), script];
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, argv.clone());
        let artifact_dir = temp.path().join("artifacts-truncated");

        let code = run(&fixture_args(registry, vec![source], artifact_dir.clone())).expect("prove run");

        assert_eq!(code, EXIT_PROOF_INVALID);
        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
        assert!(
            summary.contains(&format!(
                "{DIAGNOSTIC_VERIFIER_FAILURE}: proof command `{}` stdout exceeded a bounded log limit of {TEST_LOG_LIMIT_BYTES} bytes",
                argv.join(" ")
            )),
            "missing truncation diagnostic: {summary}"
        );
        let provenance = fs::read_to_string(artifact_dir.join(PROVENANCE_ARTIFACT_NAME)).expect("read provenance");
        assert!(!provenance.contains("formal_proof"));
    }

    #[test]
    fn prove_list_fails_closed_for_unmatched_or_malformed_rail_selector() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let artifact_dir = temp.path().join("artifacts-list");

        for selector in ["missing.policy#rail-0".to_owned(), format!("{POLICY_ID}#rail-x")] {
            let mut args = fixture_args(registry.clone(), vec![source.clone()], artifact_dir.clone());
            args.list = true;
            args.rail = vec![selector.clone()];

            let code = run(&args).expect("prove list");

            assert_eq!(code, EXIT_PROOF_INVALID, "selector `{selector}` must fail closed");
            assert!(!artifact_dir.exists(), "list mode should not write artifact bundles");
        }
    }

    #[test]
    fn prove_reuses_prove_bundle_directory_without_stale_proof_artifacts() {
        const SECOND_POLICY_ID: &str = "octet.verus_proof_rails.fixture-second";
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let mut registry = fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let mut second = registry.policies[0].clone();
        second.policy_id = SECOND_POLICY_ID.to_owned();
        second.proof_rails[0].receipt_paths = vec![proof_artifact_locator(&second, 0)];
        second.proof_rails[0].checked_scope = format!("formal-proof:fixture-only:{SECOND_POLICY_ID}#rail-0");
        let second_receipt = proof_artifact_locator(&second, 0);
        registry.policies.push(second);
        let registry_path = temp.path().join("registry-two-policies.json");
        fs::write(&registry_path, serde_json::to_string_pretty(&registry).expect("registry json"))
            .expect("write registry");
        let artifact_dir = temp.path().join("artifacts-reused");
        let mut args = fixture_args(registry_path, vec![source], artifact_dir.clone());

        assert_eq!(run(&args).expect("first prove run"), EXIT_PROOF_VALID);
        assert!(artifact_dir.join(&second_receipt).is_file());
        fs::write(artifact_dir.join("proof-ref.json"), "{}\n").expect("write foreign proof ref");
        fs::create_dir_all(artifact_dir.join("notes")).expect("create foreign directory");
        fs::write(artifact_dir.join("notes/review.txt"), "keep\n").expect("write foreign note");
        args.rail = vec![format!("{POLICY_ID}#rail-0")];
        assert_eq!(run(&args).expect("second prove run"), EXIT_PROOF_VALID);

        assert!(!artifact_dir.join(&second_receipt).exists(), "stale receipt from the earlier run must be removed");
        assert!(artifact_dir.join("proof-ref.json").is_file(), "entries prove never writes must survive");
        assert!(artifact_dir.join("notes/review.txt").is_file(), "directories prove never writes must survive");
        fs::remove_file(artifact_dir.join("proof-ref.json")).expect("remove foreign proof ref");
        fs::remove_dir_all(artifact_dir.join("notes")).expect("remove foreign directory");
        let report = verify_bundle(bundle_input(&artifact_dir));
        assert_eq!(report.status, "valid", "unexpected diagnostics: {:?}", report.diagnostics);
    }

    #[test]
    fn prove_refuses_artifact_dir_that_is_or_encloses_the_proof_workspace_root() {
        for nested in [false, true] {
            let temp = tempfile::tempdir().expect("tempdir");
            let workspace = if nested {
                temp.path().join("workspace")
            } else {
                temp.path().to_path_buf()
            };
            fs::create_dir_all(&workspace).expect("create workspace");
            let source = write_fixture_source(&workspace);
            let proof = write_fixture_proof(&workspace);
            let rail_marker = temp.path().join("rail-ran");
            let script = format!(": > '{}'; printf '%s\\n' '{PASS_SIGNAL}'", rail_marker.display());
            let registry =
                write_fixture_registry(&workspace, PASS_SIGNAL, vec!["sh".to_owned(), "-c".to_owned(), script]);
            // Prove-bundle markers make the directory look like an earlier bundle, which is reused.
            fs::write(temp.path().join(STATUS_ARTIFACT_NAME), r#"{"bundle_kind":"prove"}"#).expect("status marker");
            fs::write(temp.path().join(PROVENANCE_ARTIFACT_NAME), "{}\n").expect("provenance marker");

            let result = run(&fixture_args(registry.clone(), vec![source.clone()], temp.path().to_path_buf()));

            for kept in [&source, &proof, &registry] {
                assert!(kept.is_file(), "nested={nested}: {} must survive", kept.display());
            }
            let error = result.expect_err("an artifact directory enclosing the workspace must be refused");
            assert!(error.contains("`--artifact-dir`") && error.contains("workspace root"), "{error}");
            assert!(!rail_marker.exists(), "nested={nested}: proof rails must not run");
        }
    }

    #[test]
    fn prove_refuses_non_empty_artifact_directory_that_is_not_a_prove_bundle_before_running_rails() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let rail_marker = temp.path().join("rail-ran");
        let script = format!(": > '{}'; printf '%s\\n' '{PASS_SIGNAL}'", rail_marker.display());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, vec!["sh".to_owned(), "-c".to_owned(), script]);
        let artifact_dir = temp.path().join("artifacts-foreign");
        fs::create_dir_all(&artifact_dir).expect("create foreign directory");
        fs::write(artifact_dir.join("notes.txt"), "operator notes\n").expect("write foreign file");

        let error = run(&fixture_args(registry.clone(), vec![source.clone()], artifact_dir.clone()))
            .expect_err("foreign artifact directory must be refused");

        assert!(error.contains("refusing to use artifact directory"), "unexpected error: {error}");
        assert!(!rail_marker.exists(), "proof rails must not run before the artifact directory is accepted");
        assert!(artifact_dir.join("notes.txt").is_file(), "foreign files must not be removed");
        assert!(!artifact_dir.join(STATUS_ARTIFACT_NAME).exists());

        let fresh_dir = temp.path().join("artifacts-fresh");
        assert_eq!(run(&fixture_args(registry, vec![source], fresh_dir)).expect("prove run"), EXIT_PROOF_VALID);
        assert!(rail_marker.is_file(), "the fixture rail runs once the artifact directory is acceptable");
    }

    #[test]
    fn prove_deduplicates_overlapping_source_arguments() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));
        let source_dir = source.parent().expect("source directory").to_path_buf();
        let artifact_dir = temp.path().join("artifacts-overlap");

        let code = run(&fixture_args(registry, vec![source_dir, source], artifact_dir.clone())).expect("prove run");

        let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
        assert_eq!(code, EXIT_PROOF_VALID, "overlapping sources must load once: {summary}");
    }

    #[test]
    fn prove_resolves_one_source_root_independent_of_argument_order() {
        let temp = tempfile::tempdir().expect("tempdir");
        let source = write_fixture_source(temp.path());
        write_fixture_proof(temp.path());
        fs::write(
            temp.path().join(CARGO_MANIFEST_FILE),
            "[workspace]\nmembers = [\"crates/*\"]\n\n[package]\nname = \"proof-fixture\"\nversion = \"0.0.0\"\n",
        )
        .expect("write workspace manifest");
        let member = temp.path().join("crates/helper");
        fs::create_dir_all(member.join("src")).expect("create member crate");
        fs::write(member.join(CARGO_MANIFEST_FILE), "[package]\nname = \"helper\"\nversion = \"0.0.0\"\n")
            .expect("write member manifest");
        let member_source = member.join("src/lib.rs");
        fs::write(&member_source, "pub fn helper() {}\n").expect("write member source");
        let registry = write_fixture_registry(temp.path(), PASS_SIGNAL, command_argv(PASS_SIGNAL));

        let mut receipts = Vec::new();
        for (label, sources) in [
            ("member-first", vec![member_source.clone(), source.clone()]),
            ("member-last", vec![source.clone(), member_source.clone()]),
            ("root", vec![temp.path().to_path_buf()]),
        ] {
            let artifact_dir = temp.path().join(format!("artifacts-{label}"));
            let code = run(&fixture_args(registry.clone(), sources, artifact_dir.clone())).expect("prove run");
            let summary = fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("read summary");
            assert_eq!(code, EXIT_PROOF_VALID, "`{label}` sources must resolve to the workspace root: {summary}");
            receipts.push(
                fs::read(artifact_dir.join(proof_artifact_locator(&fixture_policy_stub(), 0))).expect("read receipt"),
            );
            fs::remove_dir_all(&artifact_dir).expect("remove artifact directory before the next source scan");
        }
        assert_eq!(receipts[0], receipts[1], "argument order must not change the proof receipt");

        let outside = tempfile::tempdir().expect("outside tempdir");
        let outside_source = write_fixture_source(outside.path());
        let error = run(&fixture_args(registry, vec![source, outside_source], temp.path().join("artifacts-outside")))
            .expect_err("source outside the proof root must be rejected");
        assert!(error.contains("must share one Cargo workspace root"), "unexpected error: {error}");
    }

    /// The Lean axiom report artifact for `theorem` when its proof uses no axiom.
    fn clean_lean_axiom_report(theorem: &str) -> crate::aeneas_execution::AeneasArtifact {
        crate::aeneas_execution::AeneasArtifact {
            locator: AENEAS_LEAN_AXIOM_REPORT_ARTIFACT.to_owned(),
            contents: serde_json::to_vec(&serde_json::json!({
                "schema_version": octet_standards::aeneas_observation::AENEAS_LEAN_AXIOM_REPORT_SCHEMA_VERSION,
                "theorem": theorem,
                "axioms": [],
            }))
            .expect("Lean axiom report"),
        }
    }

    #[test]
    fn aeneas_receipts_for_different_rails_use_disjoint_artifact_locators() {
        let fixture =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../standards/proof-rails/aeneas-lean/fixtures/valid");
        let bundle = fixture.join("artifacts/proofs/aeneas");
        let read = |name: &str| fs::read(bundle.join(name)).expect("frozen Aeneas fixture artifact");
        let argv = |name: &str| serde_json::from_slice::<Vec<String>>(&read(name)).expect("frozen Aeneas stage argv");
        let summary = AeneasCommandSummary {
            command_succeeded: true,
            timed_out: false,
            stdout_text: String::new(),
            stdout_original_bytes: 0,
            stdout_truncated: false,
            stderr_text: String::new(),
            stderr_original_bytes: 0,
            stderr_truncated: false,
            command_blake3: String::new(),
            diagnostics: Vec::new(),
            stage_argvs: vec![
                argv("charon-command.json"),
                argv("aeneas-command.json"),
                argv("lean-command.json"),
            ],
            artifacts: [
                ("lake-manifest.json", "package-lock.json"),
                ("llbc.llbc", "llbc.llbc"),
                ("translation.json", "translation.json"),
                ("Llbc.lean", "Llbc.lean"),
                ("lean-toolchain", "lean-toolchain"),
            ]
            .into_iter()
            .map(|(observed, frozen)| crate::aeneas_execution::AeneasArtifact {
                locator: observed.to_owned(),
                contents: read(frozen),
            })
            .chain(std::iter::once(clean_lean_axiom_report("safe_add_matches_model")))
            .collect(),
            provider_profile: read("provider-profile.json"),
            effective_policy: Some(frozen_aeneas_effective_policy()),
        };
        let source = SourceFile::new(
            "src/lib.rs",
            fs::read_to_string(fixture.join("real_fixture/src/lib.rs")).expect("Aeneas fixture source"),
        );
        let theorem = SourceFile::new(
            "Theorem.lean",
            fs::read_to_string(fixture.join("real_fixture/Theorem.lean")).expect("Aeneas fixture theorem"),
        );
        let rail = ProofRail {
            backend: AENEAS_LEAN_BACKEND.to_owned(),
            source_paths: vec![source.path.clone()],
            proof_paths: vec![theorem.path.clone()],
            theorem_id: "aeneas.fixture.safe_add".to_owned(),
            approved_theorem_statement_blake3: crate::evidence::artifact_blake3_hex(theorem.text.as_bytes()),
            trust_boundaries: vec!["fixture Aeneas rail proves only its named theorem".to_owned()],
            verifier_mode: VERIFIER_MODE_FIXTURE.to_owned(),
            timeout_ms: AENEAS_FIXTURE_TIMEOUT_MS,
            stdout_max_bytes: AENEAS_FIXTURE_LOG_LIMIT_BYTES,
            stderr_max_bytes: AENEAS_FIXTURE_LOG_LIMIT_BYTES,
            environment_allowlist: [
                "PATH",
                "CHARON_TOOLCHAIN_IS_IN_PATH",
                "LD_LIBRARY_PATH",
                "HOME",
                "MATHLIB_NO_CACHE_ON_UPDATE",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
            ..ProofRail::default()
        };
        let mut policy = fixture_policy_stub();
        policy.proof_rails = vec![rail.clone(), rail];

        let locators = (0..policy.proof_rails.len())
            .map(|rail_index| {
                let selected = SelectedRail {
                    policy: &policy,
                    rail_index,
                    rail: &policy.proof_rails[rail_index],
                };
                let (_, artifacts) = build_aeneas_receipt(
                    &selected,
                    std::slice::from_ref(&source),
                    std::slice::from_ref(&theorem),
                    &checked_scope(&policy, rail_index, selected.rail),
                    &summary,
                )
                .expect("Aeneas formal receipt");
                artifacts.into_iter().map(|artifact| artifact.locator).collect::<BTreeSet<_>>()
            })
            .collect::<Vec<_>>();

        assert!(locators[0].is_disjoint(&locators[1]), "Aeneas rails share artifact locators: {locators:?}");
        for (rail_index, rail_locators) in locators.iter().enumerate() {
            let rail_directory =
                format!("{PROOF_ARTIFACT_DIR_NAME}/{}-rail-{rail_index}.aeneas/", safe_artifact_stem(POLICY_ID));
            assert!(
                rail_locators.iter().all(|locator| locator.starts_with(&rail_directory)),
                "rail {rail_index} artifacts escape `{rail_directory}`: {rail_locators:?}"
            );
        }
    }

    fn frozen_aeneas_effective_policy() -> crate::aeneas_execution::AeneasEffectivePolicy {
        crate::aeneas_execution::AeneasEffectivePolicy {
            limits: octet_standards::formal_proof::AeneasExecutionLimits {
                timeout_ms: AENEAS_FIXTURE_TIMEOUT_MS,
                stdout_max_bytes: AENEAS_FIXTURE_LOG_LIMIT_BYTES,
                stderr_max_bytes: AENEAS_FIXTURE_LOG_LIMIT_BYTES,
            },
            environment_allowlist: [
                "PATH",
                "CHARON_TOOLCHAIN_IS_IN_PATH",
                "LD_LIBRARY_PATH",
                "HOME",
                "MATHLIB_NO_CACHE_ON_UPDATE",
            ]
            .into_iter()
            .map(str::to_owned)
            .collect(),
        }
    }

    #[cfg(unix)]
    fn fake_aeneas_receipt(
        root: &Path,
        rail: ProofRail,
        theorem: &str,
        charon_body: &str,
    ) -> Result<octet_standards::formal_proof::AeneasLeanProofPayload, String> {
        let (source_root, summary) = fake_aeneas_summary(root, &rail, theorem, charon_body)?;
        aeneas_receipt_payload(rail, theorem, &source_root, &summary)
    }

    #[cfg(unix)]
    fn fake_aeneas_summary(
        root: &Path,
        rail: &ProofRail,
        theorem: &str,
        charon_body: &str,
    ) -> Result<(PathBuf, AeneasCommandSummary), String> {
        use crate::aeneas_execution::fake_provider;

        let source_root = fake_provider::source_root(root, &rail.proof_paths[0], theorem);
        let provider = fake_provider::provider(
            &root.join("provider"),
            charon_body,
            &fake_provider::replay_translation(),
            &fake_provider::lean_checks_real_theorem(),
        );
        let summary = crate::aeneas_execution::execute_aeneas_with_provider(rail, &source_root, &provider);
        if !summary.command_succeeded {
            return Err(format!("fake Aeneas rail failed: {:?}", summary.diagnostics));
        }
        Ok((source_root, summary))
    }

    #[cfg(unix)]
    fn aeneas_receipt_payload(
        rail: ProofRail,
        theorem: &str,
        source_root: &Path,
        summary: &AeneasCommandSummary,
    ) -> Result<octet_standards::formal_proof::AeneasLeanProofPayload, String> {
        let source =
            SourceFile::new("src/lib.rs", fs::read_to_string(source_root.join("src/lib.rs")).expect("fixture source"));
        let theorem_source = SourceFile::new(rail.proof_paths[0].clone(), theorem.to_owned());
        let mut policy = fixture_policy_stub();
        policy.proof_rails = vec![rail];
        let selected = SelectedRail {
            policy: &policy,
            rail_index: 0,
            rail: &policy.proof_rails[0],
        };
        let (receipt, _) = build_aeneas_receipt(
            &selected,
            std::slice::from_ref(&source),
            std::slice::from_ref(&theorem_source),
            &checked_scope(&policy, 0, selected.rail),
            summary,
        )?;
        let envelope = serde_json::from_str::<FormalProofEnvelope>(&receipt).expect("Aeneas receipt envelope");
        let FormalProofProvider::AeneasLean(payload) = envelope.provider else {
            panic!("Aeneas receipt must carry the Aeneas provider payload");
        };
        Ok(payload)
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[cfg(unix)]
    #[test]
    fn aeneas_receipt_rejects_trust_inputs_lean_reports() {
        use crate::aeneas_execution::fake_provider;

        let temp = tempfile::tempdir().expect("tempdir");
        let theorem = fake_provider::real_theorem();
        let rail = fake_provider::rail("Theorem.lean", &theorem);
        let (source_root, summary) = fake_aeneas_summary(temp.path(), &rail, &theorem, &fake_provider::replay_llbc())
            .expect("fake Aeneas summary");
        let clean = aeneas_receipt_payload(rail.clone(), &theorem, &source_root, &summary).expect("clean receipt");
        let closure = &clean.theorem_admission.as_ref().expect("theorem admission").trust_closure;
        assert_eq!(closure.declared_admitted_declarations, Vec::<String>::new());
        assert_eq!(closure.undeclared_inputs, Vec::<String>::new());

        let mut admitted = summary.clone();
        let report = admitted
            .artifacts
            .iter_mut()
            .find(|artifact| artifact.locator == AENEAS_LEAN_AXIOM_REPORT_ARTIFACT)
            .expect("Lean axiom report artifact");
        report.contents = String::from_utf8(report.contents.clone())
            .expect("UTF-8 report")
            .replace("\"propext\"", "\"sorryAx\",\"planted\"")
            .into_bytes();
        let rejected = aeneas_receipt_payload(rail.clone(), &theorem, &source_root, &admitted)
            .expect_err("receipt over an admitted proof and an undeclared axiom");
        assert!(rejected.contains("admitted"), "{rejected}");
        assert!(rejected.contains("undeclared"), "{rejected}");

        let mut unreported = summary;
        unreported.artifacts.retain(|artifact| artifact.locator != AENEAS_LEAN_AXIOM_REPORT_ARTIFACT);
        assert!(aeneas_receipt_payload(rail, &theorem, &source_root, &unreported).is_err());
    }

    // r[verify octet.aeneas_lean_proof_rail.charon_contract]
    #[cfg(unix)]
    #[test]
    fn aeneas_receipt_records_charon_options_observed_from_the_produced_llbc() {
        use crate::aeneas_execution::fake_provider;

        let temp = tempfile::tempdir().expect("tempdir");
        let theorem = fake_provider::real_theorem();
        let rail = fake_provider::rail("Theorem.lean", &theorem);
        let payload = fake_aeneas_receipt(temp.path(), rail, &theorem, &fake_provider::replay_llbc())
            .expect("fake Aeneas receipt");
        let llbc =
            fs::read_to_string(Path::new(fake_provider::FROZEN_ARTIFACTS).join("llbc.llbc")).expect("frozen LLBC");

        assert_eq!(
            payload.extraction_policy,
            octet_standards::aeneas_observation::observe_charon_extraction_policy(&llbc).expect("observed policy")
        );
        assert!(payload.extraction_policy.include_rules.is_empty(), "receipt claims include rules Charon never ran");
    }

    // r[verify octet.aeneas_lean_proof_rail.safe_execution]
    #[cfg(unix)]
    #[test]
    fn aeneas_receipt_records_the_effectively_enforced_limits() {
        use crate::aeneas_execution::fake_provider;

        let temp = tempfile::tempdir().expect("tempdir");
        let theorem = fake_provider::real_theorem();
        let mut rail = fake_provider::rail("Theorem.lean", &theorem);
        rail.timeout_ms = 0;
        rail.stdout_max_bytes = AENEAS_FIXTURE_LOG_LIMIT_BYTES;
        let chatty_charon =
            format!("head -c {} /dev/zero\n{}", DEFAULT_LOG_MAX_BYTES * 2, fake_provider::replay_llbc());
        let payload = fake_aeneas_receipt(temp.path(), rail, &theorem, &chatty_charon)
            .expect("output within the rail's declared stdout limit must pass");

        assert_eq!(payload.limits.stdout_max_bytes, AENEAS_FIXTURE_LOG_LIMIT_BYTES);
        assert_eq!(payload.limits.timeout_ms, DEFAULT_PROOF_TIMEOUT_MS);
    }

    fn fixture_args(registry: PathBuf, source: Vec<PathBuf>, artifact_dir: PathBuf) -> ProveArgs {
        ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source,
            family: Vec::new(),
            rail: Vec::new(),
            artifact_dir,
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        }
    }

    fn passing_fixture_execution(root: &Path, policy: &PolicyEntry) -> (ProveArgs, ExecutedProof) {
        let source_paths = vec![write_fixture_source(root)];
        write_fixture_proof(root);
        let working_dir = proof_root(&source_paths).expect("proof root");
        let sources = load_sources(&source_paths, &working_dir).expect("load sources");
        let selected = SelectedRail {
            policy,
            rail_index: 0,
            rail: &policy.proof_rails[0],
        };
        let execution = execute_proof(&selected, &sources, &working_dir, false);
        assert!(execution.row.diagnostics.is_empty(), "fixture proof failed: {:?}", execution.row.diagnostics);
        (fixture_args(root.join("registry.json"), source_paths, root.join("artifacts")), execution)
    }

    fn run_fixture_with_command(root: &Path, source: &Path, argv: Vec<String>) -> String {
        let registry = write_fixture_registry(root, PASS_SIGNAL, argv);
        let artifact_dir =
            root.join(format!("artifacts-{}", safe_artifact_stem(&format!("{:?}", std::time::Instant::now()))));
        let code = run(&ProveArgs {
            registry: Some(registry),
            policy_artifacts_dir: None,
            source: vec![source.to_path_buf()],
            family: Vec::new(),
            rail: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");
        assert_eq!(code, EXIT_PROOF_INVALID);
        fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("summary")
    }

    fn run_fixture_with_timeout(root: &Path, source: &Path, argv: Vec<String>) -> String {
        let registry_path = root.join("registry-timeout.json");
        let mut registry = fixture_registry(root, PASS_SIGNAL, argv);
        registry.policies[0].proof_rails[0].timeout_ms = SHORT_TIMEOUT_MS;
        fs::write(&registry_path, serde_json::to_string_pretty(&registry).expect("registry json"))
            .expect("write registry");
        let artifact_dir = root.join("artifacts-timeout");
        let code = run(&ProveArgs {
            registry: Some(registry_path),
            policy_artifacts_dir: None,
            source: vec![source.to_path_buf()],
            family: Vec::new(),
            rail: Vec::new(),
            artifact_dir: artifact_dir.clone(),
            output_format: OutputFormat::Human,
            command_line: "cargo octet prove fixture".to_owned(),
            list: false,
            explain: false,
            candidates: false,
            proof_cards: None,
            proof_refs: Vec::new(),
            trust_boundary: None,
            negative_check: None,
        })
        .expect("prove run");
        assert_eq!(code, EXIT_PROOF_INVALID);
        fs::read_to_string(artifact_dir.join(SUMMARY_ARTIFACT_NAME)).expect("summary")
    }

    fn command_argv(pass_signal: &str) -> Vec<String> {
        vec!["printf".to_owned(), "%s\\n".to_owned(), pass_signal.to_owned()]
    }

    fn write_fixture_source(root: &Path) -> PathBuf {
        fs::write(root.join(CARGO_MANIFEST_FILE), "[package]\nname = \"proof-fixture\"\nversion = \"0.0.0\"\n")
            .expect("write fixture manifest");
        let path = root.join(SOURCE_RELATIVE_PATH);
        fs::create_dir_all(path.parent().expect("source parent")).expect("create source parent");
        fs::write(&path, "pub fn checked_add_one(value: u64) -> u64 { value + 1 }\n").expect("write source");
        path
    }

    fn write_fixture_proof(root: &Path) -> PathBuf {
        let path = root.join(PROOF_RELATIVE_PATH);
        fs::create_dir_all(path.parent().expect("proof parent")).expect("create proof parent");
        fs::write(&path, "verus! { proof fn addition_bound() ensures true {} }\n").expect("write proof");
        path
    }

    fn write_fixture_registry(root: &Path, pass_signal: &str, argv: Vec<String>) -> PathBuf {
        let path = root.join("registry.json");
        let registry = fixture_registry(root, pass_signal, argv);
        fs::write(&path, serde_json::to_string_pretty(&registry).expect("registry json")).expect("write registry");
        path
    }

    fn write_production_negative_registry(root: &Path, argv: Vec<String>) -> PathBuf {
        let path = root.join("registry-production-negative.json");
        let registry = production_negative_registry(argv);
        fs::write(&path, serde_json::to_string_pretty(&registry).expect("registry json")).expect("write registry");
        path
    }

    /// Writes a stand-in `octet-production-verus` into `root` and returns the rail argv that runs
    /// it. `--identity` prints `identity`; a proof run touches `proof_marker` when it is
    /// non-empty, then prints the pass signal and exits zero when `proves` is set, or exits one
    /// otherwise.
    #[cfg(unix)]
    fn write_fake_production_verifier(root: &Path, identity: &str, proof_marker: &str, proves: bool) -> Vec<String> {
        let name = "octet-production-verus";
        let touch_marker = if proof_marker.is_empty() {
            String::new()
        } else {
            format!(": > '{proof_marker}'\n")
        };
        let proof_result = if proves {
            format!("printf '%s\\n' '{PASS_SIGNAL}'\nexit 0\n")
        } else {
            "exit 1\n".to_owned()
        };
        let script = format!(
            "#!/bin/sh\nif [ \"${{1:-}}\" = --identity ]; then\nprintf '%s' '{identity}'\nexit 0\nfi\n{touch_marker}{proof_result}"
        );
        crate::test_support::write_executable_script(&root.join(name), &script);
        vec![format!("./{name}"), PROOF_RELATIVE_PATH.to_owned()]
    }

    /// The `--identity` report of a verifier built from the pins in `production_negative_rail`.
    #[cfg(unix)]
    fn pinned_production_identity() -> String {
        let verifier = production_negative_rail(Vec::new()).receipt_verifier;
        format!(
            "proof-verifier-mode: production\nproof-verifier-binary-blake3: {}\nproof-toolchain-blake3: {}\nproof-solver-blake3: {}\n",
            verifier.verifier_binary_digest, verifier.toolchain_digest, verifier.solver_digest
        )
    }

    fn fixture_registry(root: &Path, pass_signal: &str, argv: Vec<String>) -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec![CHECKER_OCTET_STANDARDS.to_owned()],
            policies: vec![fixture_policy(root, pass_signal, argv)],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn production_negative_registry(argv: Vec<String>) -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec![CHECKER_OCTET_STANDARDS.to_owned()],
            policies: vec![production_negative_policy(argv)],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn fixture_policy(root: &Path, pass_signal: &str, argv: Vec<String>) -> PolicyEntry {
        PolicyEntry {
            policy_id: POLICY_ID.to_owned(),
            pillar: "verification".to_owned(),
            source_path: "docs/standards/source/verification.adoc".to_owned(),
            source_heading: "Verus proof rails".to_owned(),
            source_anchor: "docs/standards/source/verification.adoc#verus-proof-rails".to_owned(),
            summary: "fixture policy".to_owned(),
            classification: Classification::Hard,
            enforcement_level: EnforcementLevel::Hard,
            current_enforcement_mode: EnforcementMode::Verified,
            target_enforcement_mode: EnforcementMode::Verified,
            escalation_criteria: "fixture".to_owned(),
            mechanisms: Vec::new(),
            deterministic_checker: octet_standards::model::EvidenceCommand::default(),
            proof_rails: vec![fixture_rail(root, pass_signal, argv)],
            checker_owner: "verification".to_owned(),
            checker_id: CHECKER_OCTET_STANDARDS.to_owned(),
            default_severity: Severity::Error,
            suppression_requirements: SuppressionRequirements {
                requires_policy_id: true,
                requires_source_section: true,
                requires_owner: true,
                requires_rationale: true,
                requires_review_trigger: true,
                requires_safer_default: true,
            },
            fixture_coverage: FixtureCoverage {
                status: FixtureStatus::Covered,
                compliant: vec![SOURCE_RELATIVE_PATH.to_owned()],
                noncompliant: vec!["missing-pass-signal".to_owned()],
            },
            verification_evidence_requirements: vec!["fixture".to_owned()],
            classification_rationale: "fixture".to_owned(),
            false_positive_risk: "low".to_owned(),
            existing_enforcement: Vec::new(),
            custom_checker_justification: "fixture".to_owned(),
        }
    }

    fn production_negative_policy(argv: Vec<String>) -> PolicyEntry {
        let mut policy = fixture_policy(Path::new("."), PASS_SIGNAL, argv.clone());
        policy.policy_id = PRODUCTION_NEGATIVE_POLICY_ID.to_owned();
        policy.summary = "production negative fixture policy".to_owned();
        policy.escalation_criteria = "production verifier failures must be fail-closed".to_owned();
        policy.proof_rails = vec![production_negative_rail(argv)];
        policy
    }

    fn production_negative_rail(argv: Vec<String>) -> ProofRail {
        let mut rail = fixture_rail(Path::new("."), PASS_SIGNAL, argv);
        rail.receipt_paths = vec![format!(
            "{PROOF_ARTIFACT_DIR_NAME}/{}-rail-0.receipt",
            safe_artifact_stem(PRODUCTION_NEGATIVE_POLICY_ID)
        )];
        rail.trust_boundaries = vec!["production negative fixture verifies fail-closed behavior only".to_owned()];
        rail.receipt_verifier = ReceiptVerifier {
            verifier_id: PRODUCTION_VERIFIER_ID.to_owned(),
            verifier_digest: "production-fixture-verifier-digest".to_owned(),
            verifier_binary_path: "octet-production-verus".to_owned(),
            verifier_binary_digest: "production-fixture-binary-digest".to_owned(),
            verifier_nix_closure: "production-fixture-closure".to_owned(),
            verifier_nix_closure_graph_digest: "production-fixture-closure-graph-digest".to_owned(),
            toolchain_id: "production-fixture-toolchain".to_owned(),
            toolchain_digest: "production-fixture-toolchain-digest".to_owned(),
            solver_id: "z3@production-fixture".to_owned(),
            solver_digest: "production-fixture-solver-digest".to_owned(),
            ..ReceiptVerifier::default()
        };
        rail.verifier_mode = VERIFIER_MODE_PRODUCTION.to_owned();
        rail.checked_scope = format!("formal-proof:production:{PRODUCTION_NEGATIVE_POLICY_ID}#rail-0");
        rail
    }

    fn fixture_rail(_root: &Path, pass_signal: &str, argv: Vec<String>) -> ProofRail {
        ProofRail {
            backend: "verus".to_owned(),
            command: argv.join(" "),
            command_argv: argv,
            source_paths: vec![SOURCE_RELATIVE_PATH.to_owned()],
            proof_paths: vec![PROOF_RELATIVE_PATH.to_owned()],
            theorem_id: String::new(),
            approved_theorem_statement_blake3: String::new(),
            receipt_paths: vec![proof_artifact_locator(&fixture_policy_stub(), 0)],
            pass_signal: pass_signal.to_owned(),
            trust_boundaries: vec![TRUST_BOUNDARY.to_owned()],
            receipt_binding: "blake3-command-and-source".to_owned(),
            receipt_provenance: ReceiptProvenance {
                schema_version: "octet-verus-proof-receipt/v1".to_owned(),
                runner: CHECKER_OCTET_STANDARDS.to_owned(),
                environment: "cargo-octet-prove-fixture".to_owned(),
                requires_backend: true,
                requires_trust_boundary: true,
            },
            receipt_verifier: ReceiptVerifier {
                verifier_id: VERIFIER_ID.to_owned(),
                verifier_digest: "fixture-digest".to_owned(),
                verifier_binary_path: "octet-verus".to_owned(),
                verifier_binary_digest: "fixture-binary-digest".to_owned(),
                verifier_nix_closure: "fixture-closure".to_owned(),
                verifier_nix_closure_digest: "fixture-closure-digest".to_owned(),
                toolchain_id: "verus-fixture-wrapper/v1".to_owned(),
                toolchain_digest: "fixture-toolchain-digest".to_owned(),
                ..ReceiptVerifier::default()
            },
            verifier_mode: VERIFIER_MODE_FIXTURE.to_owned(),
            checked_scope: format!("formal-proof:fixture-only:{POLICY_ID}#rail-0"),
            timeout_ms: TEST_TIMEOUT_MS,
            stdout_max_bytes: TEST_LOG_LIMIT_BYTES,
            stderr_max_bytes: TEST_LOG_LIMIT_BYTES,
            environment_allowlist: vec!["PATH".to_owned()],
        }
    }

    fn fixture_policy_stub() -> PolicyEntry {
        PolicyEntry {
            policy_id: POLICY_ID.to_owned(),
            pillar: "verification".to_owned(),
            source_path: "fixture".to_owned(),
            source_heading: "fixture".to_owned(),
            source_anchor: "fixture".to_owned(),
            summary: "fixture".to_owned(),
            classification: Classification::Hard,
            enforcement_level: EnforcementLevel::Hard,
            current_enforcement_mode: EnforcementMode::Verified,
            target_enforcement_mode: EnforcementMode::Verified,
            escalation_criteria: "fixture".to_owned(),
            mechanisms: Vec::new(),
            deterministic_checker: octet_standards::model::EvidenceCommand::default(),
            proof_rails: Vec::new(),
            checker_owner: "fixture".to_owned(),
            checker_id: CHECKER_OCTET_STANDARDS.to_owned(),
            default_severity: Severity::Error,
            suppression_requirements: SuppressionRequirements {
                requires_policy_id: true,
                requires_source_section: true,
                requires_owner: true,
                requires_rationale: true,
                requires_review_trigger: true,
                requires_safer_default: true,
            },
            fixture_coverage: FixtureCoverage {
                status: FixtureStatus::Covered,
                compliant: Vec::new(),
                noncompliant: Vec::new(),
            },
            verification_evidence_requirements: vec!["fixture".to_owned()],
            classification_rationale: "fixture".to_owned(),
            false_positive_risk: "low".to_owned(),
            existing_enforcement: Vec::new(),
            custom_checker_justification: "fixture".to_owned(),
        }
    }

    fn source_files_for_cli_tests() -> Vec<SourceFile> {
        vec![SourceFile::new(
            SOURCE_RELATIVE_PATH,
            "pub fn checked_add_one(value: u64) -> u64 { value + 1 }\n",
        )]
    }

    fn proof_files_for_cli_tests() -> Vec<SourceFile> {
        vec![SourceFile::new(
            PROOF_RELATIVE_PATH,
            "verus! { proof fn addition_bound() ensures true {} }\n",
        )]
    }

    fn provenance_records(text: &str) -> Vec<ProvenanceRecord> {
        text.lines()
            .map(|line| serde_json::from_str::<ProvenanceRecord>(line).expect("provenance record"))
            .collect()
    }

    fn valid_card_fixture() -> ProofCard {
        ProofCard {
            proof_or_property_id: proof_or_property_id(POLICY_ID, 0),
            source_requirement_or_policy_id: POLICY_ID.to_owned(),
            policy_id: POLICY_ID.to_owned(),
            rail_index: 0,
            checked_rust_source_scope: format!("formal-proof:fixture-only:{POLICY_ID}#rail-0"),
            source_paths: vec![SOURCE_RELATIVE_PATH.to_owned()],
            proof_target_paths: vec![PROOF_RELATIVE_PATH.to_owned()],
            verifier_mode: VERIFIER_MODE_FIXTURE.to_owned(),
            verifier_identity_summary: VERIFIER_ID.to_owned(),
            solver_identity_summary: String::new(),
            toolchain_identity_summary: "verus-fixture-wrapper/v1".to_owned(),
            receipt_path: proof_artifact_locator(&fixture_policy_stub(), 0),
            artifact_blake3_identities: vec![
                ProofCardArtifactIdentity {
                    artifact: SOURCE_RELATIVE_PATH.to_owned(),
                    blake3: crate::evidence::artifact_blake3_hex(b"source"),
                },
                ProofCardArtifactIdentity {
                    artifact: PROOF_RELATIVE_PATH.to_owned(),
                    blake3: crate::evidence::artifact_blake3_hex(b"proof"),
                },
            ],
            evidence_tier: EvidenceTier::ManualReviewRequired.as_str().to_owned(),
            evidence_origin: EVIDENCE_ORIGIN_OCTET_RERUN.to_owned(),
            status: STATUS_PASSED.to_owned(),
            trust_boundaries: vec![TRUST_BOUNDARY.to_owned()],
            non_claims: default_card_non_claims(),
            artifact_verifier_command: ARTIFACT_VERIFY_COMMAND.to_owned(),
            provenance_artifact: PROVENANCE_ARTIFACT_NAME.to_owned(),
            external_reference: None,
            formal_proof_envelope: None,
            negative_check: None,
            diagnostics: Vec::new(),
        }
    }

    fn imported_trellis_bundle() -> crate::imported_proof::ImportedProofBundle {
        crate::imported_proof::ImportedProofBundle {
            bundle_id: "trellis.ucan-proof-reference.digest-admission".to_owned(),
            repository_label: "trellis".to_owned(),
            repository_revision: "fb232e0e213d3e96878bbf86a10b26e53245a72c".to_owned(),
            source_path: "src/ucan_proof_reference.rs".to_owned(),
            proof_path: "src/ucan_proof_reference.rs".to_owned(),
            verifier_command_argv: vec!["./scripts/verus.sh".to_owned(), "src/ucan_proof_reference.rs".to_owned()],
            verifier_identity: "trellis scripts/verus.sh pinned Verus wrapper".to_owned(),
            hash_algorithm: HASH_ALGORITHM_BLAKE3.to_owned(),
            source_artifact: imported_artifact_identity(
                "src/ucan_proof_reference.rs",
                "018f0f79e0ef7253a153db0b3fa780beb5df1926e0140144580cc9330e87d72f",
                3805,
            ),
            proof_artifact: imported_artifact_identity(
                "src/ucan_proof_reference.rs",
                "018f0f79e0ef7253a153db0b3fa780beb5df1926e0140144580cc9330e87d72f",
                3805,
            ),
            stdout_artifacts: Vec::new(),
            stderr_artifacts: Vec::new(),
            receipt_artifacts: Vec::new(),
            evidence_origin: EVIDENCE_ORIGIN_EXTERNAL_TRELLIS.to_owned(),
            intended_evidence_tier: EvidenceTier::ManualReviewRequired.as_str().to_owned(),
            trust_boundary_notes: vec![
                "BLAKE3 identifies exact artifact bytes and does not by itself prove proof correctness, solver soundness, model adequacy, or Octet receipt validity".to_owned(),
                "Octet formal_proof requires rerunning with an Octet pinned verifier and validating an Octet-owned receipt".to_owned(),
            ],
            promotion_command_argv: vec!["cargo".to_owned(), "octet".to_owned(), "prove".to_owned()],
        }
    }

    fn imported_artifact_identity(
        path: &str,
        blake3: &str,
        bytes: u64,
    ) -> crate::imported_proof::ImportedArtifactIdentity {
        crate::imported_proof::ImportedArtifactIdentity {
            path: path.to_owned(),
            blake3: blake3.to_owned(),
            bytes,
        }
    }

    fn bundle_input(artifact_dir: &Path) -> ArtifactBundleInput {
        ArtifactBundleInput {
            artifact_dir: artifact_dir.display().to_string(),
            files: collect_bundle_files(artifact_dir, artifact_dir),
        }
    }

    fn collect_bundle_files(root: &Path, directory: &Path) -> Vec<BundleFile> {
        let mut entries = fs::read_dir(directory)
            .expect("read artifact directory")
            .collect::<Result<Vec<_>, _>>()
            .expect("read artifact entries");
        entries.sort_by_key(std::fs::DirEntry::path);
        let mut files = Vec::new();
        for entry in entries {
            let path = entry.path();
            if path.is_dir() {
                files.extend(collect_bundle_files(root, &path));
                continue;
            }
            let relative = path.strip_prefix(root).expect("relative artifact").display().to_string();
            files.push(BundleFile {
                name: relative,
                contents: fs::read(&path).expect("read artifact"),
            });
        }
        files
    }
    const CANARY_RECEIPT: &str =
        include_str!("../../standards/proof-rails/vermilion-lean/evidence/2026-11-09-u64-pilot/receipt.json");

    // r[verify octet.vermilion_canary_rail.recorded_only]
    #[test]
    fn canary_rail_validates_measured_receipt_and_emits_manual_review_card() {
        let directory = std::env::temp_dir().join(format!("octet-prove-canary-{}", std::process::id()));
        fs::create_dir_all(&directory).expect("canary test directory");
        fs::write(directory.join("receipt.json"), CANARY_RECEIPT).expect("canary receipt");
        let rail = ProofRail {
            backend: VERMILION_CANARY_BACKEND.to_owned(),
            receipt_paths: vec!["receipt.json".to_owned()],
            pass_signal: "vermilion-canary-result: pass".to_owned(),
            ..ProofRail::default()
        };
        let outcome = run_proof_command(&rail, &directory);
        assert!(outcome.command_succeeded, "{:?}", outcome.diagnostics);
        assert_eq!(outcome.canary_receipt.as_deref(), Some(CANARY_RECEIPT));
        assert!(outcome.stdout_text.contains("vermilion-canary-result: pass"));
        let execution = ExecutedProof {
            row: ProofReportRow {
                policy_id: "octet.vermilion_canary_rail.recorded".to_owned(),
                rail_index: 0,
                backend: VERMILION_CANARY_BACKEND.to_owned(),
                verifier_mode: "recorded".to_owned(),
                checked_scope: "recorded-only:vermilion-canary:test".to_owned(),
                receipt: "proofs/test.receipt".to_owned(),
                status: STATUS_PASSED,
                diagnostics: Vec::new(),
            },
            receipt_text: CANARY_RECEIPT.to_owned(),
            formal_provenance: None,
            logs: Vec::new(),
            extra_artifacts: Vec::new(),
        };
        let card = proof_card_for_execution(&execution);
        assert_eq!(card.evidence_tier, EvidenceTier::ManualReviewRequired.as_str());
        assert_eq!(card.status, STATUS_MANUAL_REVIEW_REQUIRED);
        assert!(card.formal_proof_envelope.is_none());
        assert_eq!(card.evidence_origin, EVIDENCE_ORIGIN_VERMILION_CANARY);
        assert_eq!(card.receipt_path, "proofs/test.receipt");
        let _ = fs::remove_dir_all(directory);
    }
}
