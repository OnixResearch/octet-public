//! Verified proof-rail checks.

use serde::Deserialize;
use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::EnforcementMode;
use crate::model::FamilySelection;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::model::ProofRail;
use crate::model::ReceiptProvenance;
use crate::model::ReceiptVerifier;
use crate::source::SourceFile;

/// Structured Verus proof package lock schema version.
pub const VERUS_PROOF_PACKAGE_LOCK_SCHEMA_VERSION: &str = "octet-verus-proof-package-lock/v1";
/// Structured Verus proof receipt schema version.
pub const VERUS_PROOF_RECEIPT_SCHEMA_VERSION: &str = "octet-verus-proof-receipt/v1";
/// Repository path for the generated Verus proof receipt Nickel contract.
pub const VERUS_PROOF_RECEIPT_CONTRACT_PATH: &str = "standards/proof-rails/contracts/verus-proof-receipt.generated.ncl";
/// Fixture-only verifier mode label.
pub const VERIFIER_MODE_FIXTURE: &str = "fixture";
/// Production verifier mode label.
pub const VERIFIER_MODE_PRODUCTION: &str = "production";
/// Passing proof receipt status.
pub const PROOF_STATUS_PASSED: &str = "passed";
/// Failed proof receipt status.
pub const PROOF_STATUS_FAILED: &str = "failed";

const VERUS_BACKEND: &str = "verus";
const VERIFIED_PATH_COMPONENT: &str = "/verified/";
const VERIFIED_PATH_PREFIX: &str = "verified/";
const FORMAL_PROOF_CLAIM_MARKER: &str = "octet-formal-proof-claim:";
const VERIFIED_MODULE_COVERAGE_POLICY_ID: &str = "octet.verus_proof_rails.verified_module_coverage";
const VERIFIED_MODULE_COVERAGE_SOURCE: &str = "cairn/specs/verus-proof-rails/spec.md#verified-module-proof-coverage";
const VERIFIED_PROOF_REMEDIATION: &str =
    "rerun the configured proof command and save a passing proof receipt under the declared proof path";
const VERIFIED_MODULE_REMEDIATION: &str =
    "declare a matching Verus proof rail and persist a validated receipt, or record manual_review_required provenance";
const FIELD_SOURCE_PATHS: &str = "source_paths";
const FIELD_PROOF_PATHS: &str = "proof_paths";
const FIELD_RECEIPT_PATHS: &str = "receipt_paths";
const FIELD_PASS_SIGNAL: &str = "pass_signal";
const FIELD_RECEIPT_BINDING: &str = "receipt_binding";
const FIELD_RECEIPT_PROVENANCE: &str = "receipt_provenance";
const FIELD_VERIFIER_IDENTITY: &str = "verifier_identity";
const FIELD_PROOF_ARTIFACT_IDENTITY: &str = "proof_artifact_identity";
const FIELD_SCHEMA_VERSION: &str = "schema_version";
const FIELD_POLICY_ID: &str = "policy_id";
const FIELD_RAIL_INDEX: &str = "rail_index";
const FIELD_BACKEND: &str = "backend";
const FIELD_PRODUCER_ID: &str = "producer_id";
const FIELD_CHECKER_ID: &str = "checker_id";
const FIELD_PACKAGE_LOCK_BLAKE3: &str = "package_lock_blake3";
const FIELD_COMMAND_BLAKE3: &str = "command_blake3";
const FIELD_SOURCE_BINDINGS: &str = "source_bindings";
const FIELD_PROOF_BINDINGS: &str = "proof_bindings";
const FIELD_TRUST_BOUNDARIES: &str = "trust_boundaries";
const FIELD_CHECKED_SCOPE: &str = "checked_scope";
const FIELD_STATUS: &str = "status";
const FIELD_VERIFIER_MODE: &str = "verifier_mode";
const FIELD_SOLVER_IDENTITY: &str = "solver_identity";
const FIELD_TOOLCHAIN_IDENTITY: &str = "toolchain_identity";
const FIELD_STDOUT_ARTIFACT: &str = "stdout_artifact";
const FIELD_STDERR_ARTIFACT: &str = "stderr_artifact";
const FIXTURE_ONLY_SCOPE_MARKER: &str = "fixture-only";
const PRODUCTION_SCOPE_MARKER: &str = "production";
const NON_CLAIM_WHOLE_PROGRAM: &str = "not whole-program correctness";
const NON_CLAIM_CERTIFICATION: &str = "not certification";
const NON_CLAIM_BUSINESS_DOMAIN: &str = "not business-domain validation";

/// Deterministic lock record for the proof material a Verus rail intends to check.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VerusProofPackageLock {
    /// Lock schema version.
    pub schema_version: String,
    /// BLAKE3 identity over the declared rail manifest material.
    pub rail_manifest_blake3: String,
    /// Owning policy ID.
    pub policy_id: String,
    /// Zero-based proof rail index within the owning policy.
    pub rail_index: usize,
    /// Proof backend identifier.
    pub backend: String,
    /// Verifier mode, such as `fixture` or `production`.
    pub verifier_mode: String,
    /// Producer expected to emit proof-result material.
    pub producer_id: String,
    /// Small deterministic checker expected to accept the lock/receipt contract.
    pub checker_id: String,
    /// Shell-free verifier command model.
    pub command_argv: Vec<String>,
    /// Environment variable names allowed during proof execution.
    pub environment_allowlist: Vec<String>,
    /// Configured pass signal.
    pub pass_signal: String,
    /// Scoped trust-boundary statements.
    pub trust_boundaries: Vec<String>,
    /// Source file bindings covered by the proof rail.
    pub source_bindings: Vec<VerusProofBinding>,
    /// Proof file bindings covered by the proof rail.
    pub proof_bindings: Vec<VerusProofBinding>,
    /// Verifier binary and closure identity.
    pub verifier_identity: VerusVerifierIdentity,
    /// Rust/Verus toolchain identity.
    pub toolchain_identity: VerusToolchainIdentity,
    /// Solver identity for production proof receipts.
    pub solver_identity: VerusSolverIdentity,
    /// Non-claim statements preventing overbroad proof interpretation.
    pub non_claims: Vec<String>,
}

/// Structured Verus proof receipt emitted by `cargo octet prove`.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VerusProofReceipt {
    /// Receipt schema version.
    pub schema_version: String,
    /// Owning policy ID.
    pub policy_id: String,
    /// Zero-based proof rail index within the owning policy.
    pub rail_index: usize,
    /// Proof backend identifier.
    pub backend: String,
    /// Verifier mode, such as `fixture` or `production`.
    pub verifier_mode: String,
    /// Producer that emitted the proof-result material consumed by the checker.
    pub producer_id: String,
    /// Small deterministic checker that accepted this receipt contract.
    pub checker_id: String,
    /// BLAKE3 identity of the proof package lock that this receipt claims to execute.
    pub package_lock_blake3: String,
    /// Verifier binary and closure identity.
    pub verifier_identity: VerusVerifierIdentity,
    /// Rust/Verus toolchain identity.
    pub toolchain_identity: VerusToolchainIdentity,
    /// Solver identity for production proof receipts.
    pub solver_identity: VerusSolverIdentity,
    /// BLAKE3 hash over deterministic command argv identity.
    pub command_blake3: String,
    /// Source file bindings covered by the proof rail.
    pub source_bindings: Vec<VerusProofBinding>,
    /// Proof file bindings covered by the proof rail.
    pub proof_bindings: Vec<VerusProofBinding>,
    /// Scoped trust-boundary statements.
    pub trust_boundaries: Vec<String>,
    /// Explicit checked scope for provenance.
    pub checked_scope: String,
    /// Proof command status.
    pub status: String,
    /// Configured pass signal.
    pub pass_signal: String,
    /// Persisted proof artifact locator.
    pub proof_artifact_identity: String,
    /// Bounded stdout artifact identity.
    pub stdout_artifact: VerusLogArtifact,
    /// Bounded stderr artifact identity.
    pub stderr_artifact: VerusLogArtifact,
    /// Non-claim statements preventing overbroad proof interpretation.
    pub non_claims: Vec<String>,
}

/// Verus verifier binary and closure identity fields.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct VerusVerifierIdentity {
    /// Verifier label.
    pub verifier_id: String,
    /// Optional verifier identity digest.
    pub verifier_digest: String,
    /// Verifier binary path or label.
    pub verifier_binary_path: String,
    /// Verifier binary BLAKE3 digest.
    pub verifier_binary_digest: String,
    /// Optional Nix recursive closure label.
    pub verifier_nix_closure: String,
    /// Optional Nix recursive closure digest.
    pub verifier_nix_closure_digest: String,
    /// Optional Nix closure graph digest.
    pub verifier_nix_closure_graph_digest: String,
}

/// Rust/Verus toolchain identity fields.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct VerusToolchainIdentity {
    /// Toolchain label.
    pub toolchain_id: String,
    /// Toolchain BLAKE3 digest.
    pub toolchain_digest: String,
}

/// SMT solver identity fields.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct VerusSolverIdentity {
    /// Solver label.
    pub solver_id: String,
    /// Solver BLAKE3 digest.
    pub solver_digest: String,
}

/// One source or proof file binding in a Verus receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VerusProofBinding {
    /// Normalized repo-relative path.
    pub path: String,
    /// BLAKE3 digest over file contents.
    pub blake3: String,
}

/// Identity for bounded verifier log output.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct VerusLogArtifact {
    /// Log artifact locator.
    pub locator: String,
    /// BLAKE3 digest over captured bounded log text.
    pub blake3: String,
    /// Original unbounded log byte count.
    pub bytes: usize,
    /// True when capture was truncated to the configured bound.
    pub truncated: bool,
}

/// Pure input used to construct a Verus proof package lock.
pub struct VerusProofPackageLockInput<'a> {
    /// Owning policy ID.
    pub policy_id: &'a str,
    /// Zero-based proof rail index.
    pub rail_index: usize,
    /// Declared proof rail.
    pub rail: &'a ProofRail,
    /// In-memory source files.
    pub sources: &'a [SourceFile],
    /// In-memory proof files.
    pub proof_sources: &'a [SourceFile],
}

/// Pure input used to construct a structured Verus proof receipt.
pub struct VerusProofReceiptInput<'a> {
    /// Owning policy ID.
    pub policy_id: &'a str,
    /// Zero-based proof rail index.
    pub rail_index: usize,
    /// Declared proof rail.
    pub rail: &'a ProofRail,
    /// In-memory source files.
    pub sources: &'a [SourceFile],
    /// In-memory proof files.
    pub proof_sources: &'a [SourceFile],
    /// Explicit checked-scope label.
    pub checked_scope: &'a str,
    /// Proof command status.
    pub status: &'a str,
    /// BLAKE3 hash over deterministic command argv identity.
    pub command_blake3: &'a str,
    /// Persisted proof artifact locator.
    pub proof_artifact_identity: &'a str,
    /// Bounded stdout artifact identity.
    pub stdout_artifact: VerusLogArtifact,
    /// Bounded stderr artifact identity.
    pub stderr_artifact: VerusLogArtifact,
}

/// Checks artifact-backed proof-rail evidence for verified-mode policies.
#[must_use]
pub fn check_verified_proof_rails(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<Diagnostic> {
    if !selection.includes("verification") {
        return Vec::new();
    }

    let mut diagnostics = registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .filter(|policy| policy_is_verified(policy))
        .flat_map(|policy| check_policy_proof_rails(registry, policy, sources))
        .collect::<Vec<_>>();
    diagnostics.extend(check_verified_module_coverage(registry, sources, selection));
    diagnostics
}

/// Validates one persisted proof receipt against a declared rail and in-memory source set.
#[must_use]
pub fn validate_proof_receipt(rail: &ProofRail, sources: &[SourceFile], proof_source: &SourceFile) -> Vec<String> {
    validate_proof_receipt_with_identity(rail, None, None, sources, proof_source)
}

/// Validates one persisted proof receipt with policy identity checks.
#[must_use]
pub fn validate_proof_receipt_with_identity(
    rail: &ProofRail,
    policy_id: Option<&str>,
    rail_index: Option<usize>,
    sources: &[SourceFile],
    proof_source: &SourceFile,
) -> Vec<String> {
    if rail_requires_structured_verus_receipt(rail) {
        validate_verus_proof_receipt(rail, policy_id, rail_index, sources, proof_source)
    } else {
        validate_legacy_proof_receipt(rail, sources, proof_source)
    }
}

/// Builds a deterministic Verus proof package lock from in-memory rail material.
///
/// # Errors
///
/// Returns sorted diagnostics when local material is missing, malformed, or incomplete.
pub fn build_verus_proof_package_lock(
    input: &VerusProofPackageLockInput<'_>,
) -> Result<VerusProofPackageLock, Vec<String>> {
    let mut diagnostics = Vec::new();
    let source_bindings = match expected_path_bindings(&input.rail.source_paths, input.sources, FIELD_SOURCE_PATHS) {
        Ok(bindings) => bindings,
        Err(errors) => {
            diagnostics.extend(errors);
            Vec::new()
        }
    };
    let proof_bindings = match expected_path_bindings(&input.rail.proof_paths, input.proof_sources, FIELD_PROOF_PATHS) {
        Ok(bindings) => bindings,
        Err(errors) => {
            diagnostics.extend(errors);
            Vec::new()
        }
    };
    diagnostics.extend(verifier_identity_completeness_diagnostics(input.rail));
    diagnostics.extend(checker_identity_completeness_diagnostics(input.rail));
    diagnostics.extend(command_model_diagnostics(input.rail));
    diagnostics.extend(trust_boundary_diagnostics(input.rail));
    let verifier_identity =
        resolved_verifier_identity(input.rail, &[input.sources, input.proof_sources].concat(), &mut diagnostics);
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }

    let verifier = &input.rail.receipt_verifier;
    Ok(VerusProofPackageLock {
        schema_version: VERUS_PROOF_PACKAGE_LOCK_SCHEMA_VERSION.to_owned(),
        rail_manifest_blake3: rail_manifest_blake3(input.rail),
        policy_id: input.policy_id.to_owned(),
        rail_index: input.rail_index,
        backend: input.rail.backend.clone(),
        verifier_mode: verifier_mode(input.rail).to_owned(),
        producer_id: expected_producer_id(input.rail).to_owned(),
        checker_id: expected_checker_id(input.rail).to_owned(),
        command_argv: input.rail.command_argv.clone(),
        environment_allowlist: input.rail.environment_allowlist.clone(),
        pass_signal: input.rail.pass_signal.clone(),
        trust_boundaries: input.rail.trust_boundaries.clone(),
        source_bindings,
        proof_bindings,
        verifier_identity,
        toolchain_identity: VerusToolchainIdentity {
            toolchain_id: verifier.toolchain_id.clone(),
            toolchain_digest: verifier.toolchain_digest.clone(),
        },
        solver_identity: VerusSolverIdentity {
            solver_id: verifier.solver_id.clone(),
            solver_digest: verifier.solver_digest.clone(),
        },
        non_claims: default_non_claims(),
    })
}

/// Returns the BLAKE3 identity of a rendered proof package lock.
#[must_use]
pub fn proof_package_lock_blake3(lock: &VerusProofPackageLock) -> String {
    let rendered = serde_json::to_string(lock).unwrap_or_else(|error| format!("serialization-error:{error}"));
    crate::proof_artifact::blake3_hex_text(&rendered)
}

/// Validates a Verus proof package lock against local rail material.
#[must_use]
pub fn validate_verus_proof_package_lock(
    input: &VerusProofPackageLockInput<'_>,
    lock: &VerusProofPackageLock,
) -> Vec<String> {
    let expected = match build_verus_proof_package_lock(input) {
        Ok(expected) => expected,
        Err(errors) => return errors,
    };
    let mut diagnostics = lock_shape_diagnostics(lock);
    push_mismatch(
        &mut diagnostics,
        "proof_package_lock.schema_version",
        VERUS_PROOF_PACKAGE_LOCK_SCHEMA_VERSION,
        &lock.schema_version,
    );
    push_mismatch(
        &mut diagnostics,
        "proof_package_lock.rail_manifest_blake3",
        &expected.rail_manifest_blake3,
        &lock.rail_manifest_blake3,
    );
    push_mismatch(&mut diagnostics, "proof_package_lock.policy_id", &expected.policy_id, &lock.policy_id);
    if lock.rail_index != expected.rail_index {
        diagnostics.push(format!(
            "proof package lock field `rail_index` expected `{}` but got `{}`",
            expected.rail_index, lock.rail_index
        ));
    }
    push_mismatch(&mut diagnostics, "proof_package_lock.backend", &expected.backend, &lock.backend);
    push_mismatch(&mut diagnostics, "proof_package_lock.verifier_mode", &expected.verifier_mode, &lock.verifier_mode);
    push_mismatch(&mut diagnostics, "proof_package_lock.producer_id", &expected.producer_id, &lock.producer_id);
    push_mismatch(&mut diagnostics, "proof_package_lock.checker_id", &expected.checker_id, &lock.checker_id);
    push_vec_mismatch(&mut diagnostics, "proof_package_lock.command_argv", &expected.command_argv, &lock.command_argv);
    push_vec_mismatch(
        &mut diagnostics,
        "proof_package_lock.environment_allowlist",
        &expected.environment_allowlist,
        &lock.environment_allowlist,
    );
    push_mismatch(&mut diagnostics, "proof_package_lock.pass_signal", &expected.pass_signal, &lock.pass_signal);
    push_vec_mismatch(
        &mut diagnostics,
        "proof_package_lock.trust_boundaries",
        &expected.trust_boundaries,
        &lock.trust_boundaries,
    );
    push_bindings_mismatch(
        &mut diagnostics,
        "proof_package_lock.source_bindings",
        &expected.source_bindings,
        &lock.source_bindings,
    );
    push_bindings_mismatch(
        &mut diagnostics,
        "proof_package_lock.proof_bindings",
        &expected.proof_bindings,
        &lock.proof_bindings,
    );
    push_mismatch(
        &mut diagnostics,
        "proof_package_lock.verifier_identity.verifier_id",
        &expected.verifier_identity.verifier_id,
        &lock.verifier_identity.verifier_id,
    );
    push_optional_mismatch(
        &mut diagnostics,
        "proof_package_lock.verifier_identity.verifier_digest",
        &expected.verifier_identity.verifier_digest,
        &lock.verifier_identity.verifier_digest,
    );
    push_mismatch(
        &mut diagnostics,
        "proof_package_lock.verifier_identity.verifier_binary_digest",
        &expected.verifier_identity.verifier_binary_digest,
        &lock.verifier_identity.verifier_binary_digest,
    );
    push_optional_mismatch(
        &mut diagnostics,
        "proof_package_lock.verifier_identity.verifier_nix_closure_graph_digest",
        &expected.verifier_identity.verifier_nix_closure_graph_digest,
        &lock.verifier_identity.verifier_nix_closure_graph_digest,
    );
    push_mismatch(
        &mut diagnostics,
        "proof_package_lock.toolchain_identity.toolchain_id",
        &expected.toolchain_identity.toolchain_id,
        &lock.toolchain_identity.toolchain_id,
    );
    push_optional_mismatch(
        &mut diagnostics,
        "proof_package_lock.solver_identity.solver_id",
        &expected.solver_identity.solver_id,
        &lock.solver_identity.solver_id,
    );
    push_vec_mismatch(&mut diagnostics, "proof_package_lock.non_claims", &expected.non_claims, &lock.non_claims);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

/// Builds a deterministic structured Verus receipt from in-memory rail and command data.
///
/// # Errors
///
/// Returns sorted diagnostics when declared source/proof path bindings are missing
/// or verifier identity fields required for structured receipts are incomplete.
pub fn build_verus_proof_receipt(input: &VerusProofReceiptInput<'_>) -> Result<VerusProofReceipt, Vec<String>> {
    let mut diagnostics = Vec::new();
    let source_bindings = match expected_path_bindings(&input.rail.source_paths, input.sources, FIELD_SOURCE_PATHS) {
        Ok(bindings) => bindings,
        Err(errors) => {
            diagnostics.extend(errors);
            Vec::new()
        }
    };
    let proof_bindings = match expected_path_bindings(&input.rail.proof_paths, input.proof_sources, FIELD_PROOF_PATHS) {
        Ok(bindings) => bindings,
        Err(errors) => {
            diagnostics.extend(errors);
            Vec::new()
        }
    };
    diagnostics.extend(verifier_identity_completeness_diagnostics(input.rail));
    diagnostics.extend(checker_identity_completeness_diagnostics(input.rail));
    let package_lock = match build_verus_proof_package_lock(&VerusProofPackageLockInput {
        policy_id: input.policy_id,
        rail_index: input.rail_index,
        rail: input.rail,
        sources: input.sources,
        proof_sources: input.proof_sources,
    }) {
        Ok(lock) => Some(lock),
        Err(errors) => {
            diagnostics.extend(errors);
            None
        }
    };
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }

    let verifier = &input.rail.receipt_verifier;
    let Some(package_lock) = package_lock else {
        return Err(vec!["package lock missing after diagnostic check".to_owned()]);
    };
    Ok(VerusProofReceipt {
        schema_version: VERUS_PROOF_RECEIPT_SCHEMA_VERSION.to_owned(),
        policy_id: input.policy_id.to_owned(),
        rail_index: input.rail_index,
        backend: input.rail.backend.clone(),
        verifier_mode: verifier_mode(input.rail).to_owned(),
        producer_id: expected_producer_id(input.rail).to_owned(),
        checker_id: expected_checker_id(input.rail).to_owned(),
        package_lock_blake3: proof_package_lock_blake3(&package_lock),
        verifier_identity: VerusVerifierIdentity {
            verifier_id: verifier.verifier_id.clone(),
            verifier_digest: verifier.verifier_digest.clone(),
            verifier_binary_path: verifier.verifier_binary_path.clone(),
            verifier_binary_digest: verifier.verifier_binary_digest.clone(),
            verifier_nix_closure: verifier.verifier_nix_closure.clone(),
            verifier_nix_closure_digest: verifier.verifier_nix_closure_digest.clone(),
            verifier_nix_closure_graph_digest: verifier.verifier_nix_closure_graph_digest.clone(),
        },
        toolchain_identity: VerusToolchainIdentity {
            toolchain_id: verifier.toolchain_id.clone(),
            toolchain_digest: verifier.toolchain_digest.clone(),
        },
        solver_identity: VerusSolverIdentity {
            solver_id: verifier.solver_id.clone(),
            solver_digest: verifier.solver_digest.clone(),
        },
        command_blake3: input.command_blake3.to_owned(),
        source_bindings,
        proof_bindings,
        trust_boundaries: input.rail.trust_boundaries.clone(),
        checked_scope: input.checked_scope.to_owned(),
        status: input.status.to_owned(),
        pass_signal: input.rail.pass_signal.clone(),
        proof_artifact_identity: input.proof_artifact_identity.to_owned(),
        stdout_artifact: input.stdout_artifact.clone(),
        stderr_artifact: input.stderr_artifact.clone(),
        non_claims: default_non_claims(),
    })
}

/// Renders deterministic receipt metadata lines that the legacy proof checker validates.
#[must_use]
pub fn proof_receipt_metadata(rail: &ProofRail, sources: &[SourceFile]) -> String {
    let mut lines = expected_receipt_provenance_lines(rail);
    lines.extend(expected_receipt_verifier_lines(rail, sources));
    if receipt_binding_enabled(rail) {
        lines.extend(expected_receipt_binding_lines(rail, sources));
    }
    receipt_lines(&lines)
}

/// Returns normalized, sorted matches for exact/glob rail path patterns.
#[must_use]
pub fn matching_sources<'a>(patterns: &[String], sources: &'a [SourceFile]) -> Vec<&'a SourceFile> {
    crate::proof_artifact::matching_sources(patterns, sources)
}

/// Returns deterministic pattern diagnostics for exact/glob rail path patterns.
#[must_use]
pub fn path_pattern_diagnostics(patterns: &[String], sources: &[SourceFile], field_name: &str) -> Vec<String> {
    crate::proof_artifact::path_pattern_diagnostics(patterns, sources, field_name)
}

/// Renders the generated Nickel contract for structured Verus proof receipts.
#[must_use]
pub fn render_verus_proof_receipt_contract() -> String {
    let mut rendered = String::new();
    rendered.push_str("# @generated by crates/octet-standards/src/proof.rs; do not edit by hand.\n");
    rendered.push_str(
        "let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in\n",
    );
    rendered.push_str("let SchemaVersion = std.contract.from_predicate (fun value => value == \"");
    rendered.push_str(VERUS_PROOF_RECEIPT_SCHEMA_VERSION);
    rendered.push_str("\") in\n");
    rendered.push_str(
        "let Status = std.contract.from_predicate (fun value => std.is_string value && std.array.elem value [\n",
    );
    rendered.push_str("  \"passed\",\n  \"failed\",\n]) in\n");
    rendered.push_str(
        "let VerifierMode = std.contract.from_predicate (fun value => std.is_string value && std.array.elem value [\n",
    );
    rendered.push_str("  \"fixture\",\n  \"production\",\n]) in\n");
    rendered.push_str("let Binding = { path | NonEmptyString, blake3 | NonEmptyString } in\n");
    rendered.push_str(
        "let LogArtifact = { locator | NonEmptyString, blake3 | NonEmptyString, bytes | Number, truncated | Bool } in\n",
    );
    rendered.push_str("let VerifierIdentity = {\n");
    rendered.push_str("  verifier_id | NonEmptyString,\n");
    rendered.push_str("  verifier_digest | String | default = \"\",\n");
    rendered.push_str("  verifier_binary_path | NonEmptyString,\n");
    rendered.push_str("  verifier_binary_digest | NonEmptyString,\n");
    rendered.push_str("  verifier_nix_closure | String | default = \"\",\n");
    rendered.push_str("  verifier_nix_closure_digest | String | default = \"\",\n");
    rendered.push_str("  verifier_nix_closure_graph_digest | String | default = \"\",\n");
    rendered.push_str("} in\n");
    rendered
        .push_str("let ToolchainIdentity = { toolchain_id | NonEmptyString, toolchain_digest | NonEmptyString } in\n");
    rendered.push_str(
        "let SolverIdentity = { solver_id | String | default = \"\", solver_digest | String | default = \"\" } in\n",
    );
    rendered.push_str("{\n");
    rendered.push_str("  VerusProofPackageLockV1 = {\n");
    rendered.push_str("    schema_version | std.contract.from_predicate (fun value => value == \"");
    rendered.push_str(VERUS_PROOF_PACKAGE_LOCK_SCHEMA_VERSION);
    rendered.push_str("\"),\n");
    rendered.push_str("    rail_manifest_blake3 | NonEmptyString,\n");
    rendered.push_str("    policy_id | NonEmptyString,\n");
    rendered.push_str("    rail_index | Number,\n");
    rendered.push_str("    backend | std.contract.from_predicate (fun value => value == \"verus\"),\n");
    rendered.push_str("    verifier_mode | VerifierMode,\n");
    rendered.push_str("    producer_id | NonEmptyString,\n");
    rendered.push_str("    checker_id | NonEmptyString,\n");
    rendered.push_str("    command_argv | Array NonEmptyString,\n");
    rendered.push_str("    environment_allowlist | Array NonEmptyString,\n");
    rendered.push_str("    pass_signal | NonEmptyString,\n");
    rendered.push_str("    trust_boundaries | Array NonEmptyString,\n");
    rendered.push_str("    source_bindings | Array Binding,\n");
    rendered.push_str("    proof_bindings | Array Binding,\n");
    rendered.push_str("    verifier_identity | VerifierIdentity,\n");
    rendered.push_str("    toolchain_identity | ToolchainIdentity,\n");
    rendered.push_str("    solver_identity | SolverIdentity,\n");
    rendered.push_str("    non_claims | Array NonEmptyString,\n");
    rendered.push_str("  },\n");
    rendered.push_str("  VerusProofReceiptV1 = {\n");
    rendered.push_str("    schema_version | SchemaVersion,\n");
    rendered.push_str("    policy_id | NonEmptyString,\n");
    rendered.push_str("    rail_index | Number,\n");
    rendered.push_str("    backend | std.contract.from_predicate (fun value => value == \"verus\"),\n");
    rendered.push_str("    verifier_mode | VerifierMode,\n");
    rendered.push_str("    producer_id | NonEmptyString,\n");
    rendered.push_str("    checker_id | NonEmptyString,\n");
    rendered.push_str("    package_lock_blake3 | NonEmptyString,\n");
    rendered.push_str("    verifier_identity | VerifierIdentity,\n");
    rendered.push_str("    toolchain_identity | ToolchainIdentity,\n");
    rendered.push_str("    solver_identity | SolverIdentity,\n");
    rendered.push_str("    command_blake3 | NonEmptyString,\n");
    rendered.push_str("    source_bindings | Array Binding,\n");
    rendered.push_str("    proof_bindings | Array Binding,\n");
    rendered.push_str("    trust_boundaries | Array NonEmptyString,\n");
    rendered.push_str("    checked_scope | NonEmptyString,\n");
    rendered.push_str("    status | Status,\n");
    rendered.push_str("    pass_signal | NonEmptyString,\n");
    rendered.push_str("    proof_artifact_identity | NonEmptyString,\n");
    rendered.push_str("    stdout_artifact | LogArtifact,\n");
    rendered.push_str("    stderr_artifact | LogArtifact,\n");
    rendered.push_str("    non_claims | Array NonEmptyString,\n");
    rendered.push_str("  },\n");
    rendered.push_str("}\n");
    rendered
}

fn validate_verus_proof_receipt(
    rail: &ProofRail,
    policy_id: Option<&str>,
    rail_index: Option<usize>,
    sources: &[SourceFile],
    proof_source: &SourceFile,
) -> Vec<String> {
    let receipt = match serde_json::from_str::<VerusProofReceipt>(&proof_source.text) {
        Ok(receipt) => receipt,
        Err(error) => return vec![format!("proof receipt field `record` malformed JSON: {error}")],
    };

    let mut diagnostics = Vec::new();
    push_mismatch(&mut diagnostics, FIELD_SCHEMA_VERSION, VERUS_PROOF_RECEIPT_SCHEMA_VERSION, &receipt.schema_version);
    if let Some(expected_policy_id) = policy_id {
        push_mismatch(&mut diagnostics, FIELD_POLICY_ID, expected_policy_id, &receipt.policy_id);
    }
    if let Some(expected_rail_index) = rail_index
        && receipt.rail_index != expected_rail_index
    {
        diagnostics.push(format!(
            "proof receipt field `{FIELD_RAIL_INDEX}` expected `{expected_rail_index}` but got `{}`",
            receipt.rail_index
        ));
    }
    push_mismatch(&mut diagnostics, FIELD_BACKEND, &rail.backend, &receipt.backend);
    push_mismatch(&mut diagnostics, FIELD_PASS_SIGNAL, &rail.pass_signal, &receipt.pass_signal);
    push_mismatch(&mut diagnostics, FIELD_VERIFIER_MODE, verifier_mode(rail), &receipt.verifier_mode);
    diagnostics.extend(validate_producer_checker_identity(rail, &receipt));
    diagnostics.extend(validate_receipt_package_lock(rail, policy_id, rail_index, sources, &receipt));
    if !matches!(receipt.status.as_str(), PROOF_STATUS_PASSED | PROOF_STATUS_FAILED) {
        diagnostics.push(format!("proof receipt field `{FIELD_STATUS}` has unsupported status `{}`", receipt.status));
    }
    if receipt.status != PROOF_STATUS_PASSED {
        diagnostics.push(format!("proof receipt field `{FIELD_STATUS}` is not `{PROOF_STATUS_PASSED}`"));
    }
    if receipt.status == PROOF_STATUS_PASSED && receipt.pass_signal != rail.pass_signal {
        diagnostics
            .push(format!("proof receipt field `{FIELD_PASS_SIGNAL}` missing pass signal `{}`", rail.pass_signal));
    }
    if receipt.command_blake3.trim().is_empty() {
        diagnostics.push(format!("proof receipt field `{FIELD_COMMAND_BLAKE3}` is empty"));
    }
    if receipt.proof_artifact_identity.trim().is_empty() {
        diagnostics.push(format!("proof receipt field `{FIELD_PROOF_ARTIFACT_IDENTITY}` is empty"));
    }
    diagnostics.extend(validate_log_artifact(FIELD_STDOUT_ARTIFACT, &receipt.stdout_artifact));
    diagnostics.extend(validate_log_artifact(FIELD_STDERR_ARTIFACT, &receipt.stderr_artifact));
    diagnostics.extend(validate_verifier_identity(rail, &receipt));
    diagnostics.extend(validate_expected_bindings(
        FIELD_SOURCE_BINDINGS,
        &receipt.source_bindings,
        &rail.source_paths,
        sources,
    ));
    diagnostics.extend(validate_expected_bindings(
        FIELD_PROOF_BINDINGS,
        &receipt.proof_bindings,
        &rail.proof_paths,
        sources,
    ));
    diagnostics.extend(validate_trust_boundaries(rail, &receipt));
    diagnostics.extend(validate_checked_scope(&receipt));
    diagnostics.extend(validate_non_claims(&receipt));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_legacy_proof_receipt(rail: &ProofRail, sources: &[SourceFile], proof_source: &SourceFile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if !proof_source.text.contains(&rail.pass_signal) {
        diagnostics
            .push(format!("proof receipt field `{FIELD_PASS_SIGNAL}` missing pass signal `{}`", rail.pass_signal));
    }
    if receipt_binding_enabled(rail) && !receipt_binding_matches(rail, sources, proof_source) {
        diagnostics.push(format!(
            "mismatched receipt binding `{}` in proof receipt field `{FIELD_RECEIPT_BINDING}`",
            rail.receipt_binding
        ));
    }
    if receipt_provenance_enabled(rail) && !receipt_provenance_matches(rail, proof_source) {
        diagnostics.push(format!(
            "mismatched receipt provenance in proof receipt field `{FIELD_RECEIPT_PROVENANCE}` required lines"
        ));
    }
    if receipt_verifier_enabled(rail) && !receipt_verifier_matches(rail, sources, proof_source) {
        diagnostics
            .push(format!("mismatched receipt verifier metadata in proof receipt field `{FIELD_VERIFIER_IDENTITY}`"));
    }
    diagnostics
}

fn validate_receipt_package_lock(
    rail: &ProofRail,
    policy_id: Option<&str>,
    rail_index: Option<usize>,
    sources: &[SourceFile],
    receipt: &VerusProofReceipt,
) -> Vec<String> {
    let lock_policy_id = policy_id.unwrap_or(&receipt.policy_id);
    let lock_rail_index = rail_index.unwrap_or(receipt.rail_index);
    let lock = match build_verus_proof_package_lock(&VerusProofPackageLockInput {
        policy_id: lock_policy_id,
        rail_index: lock_rail_index,
        rail,
        sources,
        proof_sources: sources,
    }) {
        Ok(lock) => lock,
        Err(errors) => return errors,
    };
    let expected = proof_package_lock_blake3(&lock);
    if receipt.package_lock_blake3 == expected {
        Vec::new()
    } else {
        vec![format!(
            "proof receipt field `{FIELD_PACKAGE_LOCK_BLAKE3}` expected `{expected}` but got `{}`",
            receipt.package_lock_blake3
        )]
    }
}

fn validate_producer_checker_identity(rail: &ProofRail, receipt: &VerusProofReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_mismatch(&mut diagnostics, FIELD_PRODUCER_ID, expected_producer_id(rail), &receipt.producer_id);
    push_mismatch(&mut diagnostics, FIELD_CHECKER_ID, expected_checker_id(rail), &receipt.checker_id);
    if !receipt.producer_id.trim().is_empty() && receipt.producer_id == receipt.checker_id {
        diagnostics.push(format!(
            "proof receipt field `{FIELD_CHECKER_ID}` must name an independent checker instead of self-asserted producer `{}`",
            receipt.producer_id
        ));
    }
    diagnostics
}

fn validate_log_artifact(field_name: &str, artifact: &VerusLogArtifact) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if artifact.locator.trim().is_empty() {
        diagnostics.push(format!("proof receipt field `{field_name}.locator` is empty"));
    }
    if artifact.blake3.trim().is_empty() {
        diagnostics.push(format!("proof receipt field `{field_name}.blake3` is empty"));
    }
    diagnostics
}

fn validate_verifier_identity(rail: &ProofRail, receipt: &VerusProofReceipt) -> Vec<String> {
    let verifier = &rail.receipt_verifier;
    let actual = &receipt.verifier_identity;
    let mut diagnostics = Vec::new();
    push_mismatch(&mut diagnostics, "verifier_identity.verifier_id", &verifier.verifier_id, &actual.verifier_id);
    push_optional_mismatch(
        &mut diagnostics,
        "verifier_identity.verifier_digest",
        &verifier.verifier_digest,
        &actual.verifier_digest,
    );
    push_mismatch(
        &mut diagnostics,
        "verifier_identity.verifier_binary_path",
        &verifier.verifier_binary_path,
        &actual.verifier_binary_path,
    );
    push_mismatch(
        &mut diagnostics,
        "verifier_identity.verifier_binary_digest",
        &verifier.verifier_binary_digest,
        &actual.verifier_binary_digest,
    );
    push_optional_mismatch(
        &mut diagnostics,
        "verifier_identity.verifier_nix_closure",
        &verifier.verifier_nix_closure,
        &actual.verifier_nix_closure,
    );
    push_optional_mismatch(
        &mut diagnostics,
        "verifier_identity.verifier_nix_closure_digest",
        &verifier.verifier_nix_closure_digest,
        &actual.verifier_nix_closure_digest,
    );
    push_optional_mismatch(
        &mut diagnostics,
        "verifier_identity.verifier_nix_closure_graph_digest",
        &verifier.verifier_nix_closure_graph_digest,
        &actual.verifier_nix_closure_graph_digest,
    );
    push_mismatch(
        &mut diagnostics,
        "toolchain_identity.toolchain_id",
        &verifier.toolchain_id,
        &receipt.toolchain_identity.toolchain_id,
    );
    push_mismatch(
        &mut diagnostics,
        "toolchain_identity.toolchain_digest",
        &verifier.toolchain_digest,
        &receipt.toolchain_identity.toolchain_digest,
    );
    push_optional_mismatch(
        &mut diagnostics,
        "solver_identity.solver_id",
        &verifier.solver_id,
        &receipt.solver_identity.solver_id,
    );
    push_optional_mismatch(
        &mut diagnostics,
        "solver_identity.solver_digest",
        &verifier.solver_digest,
        &receipt.solver_identity.solver_digest,
    );
    diagnostics.extend(verifier_identity_completeness_diagnostics(rail));
    diagnostics
}

fn validate_expected_bindings(
    field_name: &str,
    actual: &[VerusProofBinding],
    patterns: &[String],
    sources: &[SourceFile],
) -> Vec<String> {
    let expected = match expected_path_bindings(patterns, sources, field_name) {
        Ok(bindings) => bindings,
        Err(errors) => return errors,
    };
    if expected == actual {
        Vec::new()
    } else {
        vec![format!(
            "proof receipt field `{field_name}` has stale or mismatched path bindings"
        )]
    }
}

fn validate_trust_boundaries(rail: &ProofRail, receipt: &VerusProofReceipt) -> Vec<String> {
    if receipt.trust_boundaries == rail.trust_boundaries && !receipt.trust_boundaries.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "proof receipt field `{FIELD_TRUST_BOUNDARIES}` is missing configured trust boundaries"
        )]
    }
}

fn validate_checked_scope(receipt: &VerusProofReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.checked_scope.trim().is_empty() {
        diagnostics.push(format!("proof receipt field `{FIELD_CHECKED_SCOPE}` is empty"));
    }
    match receipt.verifier_mode.as_str() {
        VERIFIER_MODE_FIXTURE if !receipt.checked_scope.contains(FIXTURE_ONLY_SCOPE_MARKER) => diagnostics.push(format!(
            "fixture proof receipt field `{FIELD_CHECKED_SCOPE}` must explicitly include `{FIXTURE_ONLY_SCOPE_MARKER}`"
        )),
        VERIFIER_MODE_PRODUCTION if !receipt.checked_scope.contains(PRODUCTION_SCOPE_MARKER) => diagnostics.push(format!(
            "production proof receipt field `{FIELD_CHECKED_SCOPE}` must explicitly include `{PRODUCTION_SCOPE_MARKER}`"
        )),
        _ => {}
    }
    diagnostics
}

fn validate_non_claims(receipt: &VerusProofReceipt) -> Vec<String> {
    let required = default_non_claims();
    let missing = required
        .iter()
        .filter(|claim| !receipt.non_claims.iter().any(|actual| actual == *claim))
        .cloned()
        .collect::<Vec<_>>();
    if missing.is_empty() {
        Vec::new()
    } else {
        vec![format!(
            "proof receipt field `non_claims` missing scoped non-claims: {}",
            missing.join(", ")
        )]
    }
}

fn lock_shape_diagnostics(lock: &VerusProofPackageLock) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if lock.command_argv.is_empty() {
        diagnostics.push("proof package lock field `command_argv` is empty".to_owned());
    }
    for argument in &lock.command_argv {
        if argument.trim().is_empty() {
            diagnostics.push("proof package lock field `command_argv` contains an empty argument".to_owned());
        }
    }
    for variable in &lock.environment_allowlist {
        if variable.trim().is_empty() {
            diagnostics.push("proof package lock field `environment_allowlist` contains an empty variable".to_owned());
        }
    }
    if lock.trust_boundaries.is_empty() {
        diagnostics.push("proof package lock field `trust_boundaries` is empty".to_owned());
    }
    if lock.non_claims.is_empty() {
        diagnostics.push("proof package lock field `non_claims` is empty".to_owned());
    }
    diagnostics
}

fn command_model_diagnostics(rail: &ProofRail) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if rail.command_argv.is_empty() {
        diagnostics.push("proof rail is missing `command_argv`".to_owned());
    }
    for argument in &rail.command_argv {
        if argument.trim().is_empty() {
            diagnostics.push("proof rail field `command_argv` contains an empty argument".to_owned());
        }
    }
    for variable in &rail.environment_allowlist {
        if variable.trim().is_empty() {
            diagnostics.push("proof rail field `environment_allowlist` contains an empty variable".to_owned());
        }
    }
    if rail.pass_signal.trim().is_empty() {
        diagnostics.push("proof rail is missing `pass_signal`".to_owned());
    }
    diagnostics
}

fn trust_boundary_diagnostics(rail: &ProofRail) -> Vec<String> {
    if rail.trust_boundaries.is_empty() {
        vec!["proof rail is missing `trust_boundaries`".to_owned()]
    } else {
        Vec::new()
    }
}

fn resolved_verifier_identity(
    rail: &ProofRail,
    sources: &[SourceFile],
    diagnostics: &mut Vec<String>,
) -> VerusVerifierIdentity {
    let verifier = &rail.receipt_verifier;
    let verifier_nix_closure_graph_digest = if verifier.verifier_nix_closure_graph_paths.is_empty() {
        verifier.verifier_nix_closure_graph_digest.clone()
    } else {
        match crate::proof_artifact::nix_closure_graph_digest(&verifier.verifier_nix_closure_graph_paths, sources) {
            Ok(Some(digest)) => digest,
            Ok(None) => verifier.verifier_nix_closure_graph_digest.clone(),
            Err(error) => {
                diagnostics.push(format!(
                    "proof package lock field `verifier_identity.verifier_nix_closure_graph_digest` rejected malformed closure graph: {error}"
                ));
                verifier.verifier_nix_closure_graph_digest.clone()
            }
        }
    };
    VerusVerifierIdentity {
        verifier_id: verifier.verifier_id.clone(),
        verifier_digest: verifier.verifier_digest.clone(),
        verifier_binary_path: verifier.verifier_binary_path.clone(),
        verifier_binary_digest: verifier.verifier_binary_digest.clone(),
        verifier_nix_closure: verifier.verifier_nix_closure.clone(),
        verifier_nix_closure_digest: verifier.verifier_nix_closure_digest.clone(),
        verifier_nix_closure_graph_digest,
    }
}

fn rail_manifest_blake3(rail: &ProofRail) -> String {
    let rendered = serde_json::to_string(rail).unwrap_or_else(|error| format!("serialization-error:{error}"));
    crate::proof_artifact::blake3_hex_text(&rendered)
}

fn push_vec_mismatch(diagnostics: &mut Vec<String>, field_name: &str, expected: &[String], actual: &[String]) {
    if expected != actual {
        diagnostics.push(format!("proof package lock field `{field_name}` does not match declared rail material"));
    }
}

fn push_bindings_mismatch(
    diagnostics: &mut Vec<String>,
    field_name: &str,
    expected: &[VerusProofBinding],
    actual: &[VerusProofBinding],
) {
    if expected != actual {
        diagnostics.push(format!("proof package lock field `{field_name}` has stale or mismatched path bindings"));
    }
}

fn push_mismatch(diagnostics: &mut Vec<String>, field_name: &str, expected: &str, actual: &str) {
    if expected.trim().is_empty() {
        diagnostics.push(format!("proof receipt expected field `{field_name}` is not configured"));
    } else if expected != actual {
        diagnostics.push(format!("proof receipt field `{field_name}` expected `{expected}` but got `{actual}`"));
    }
}

fn push_optional_mismatch(diagnostics: &mut Vec<String>, field_name: &str, expected: &str, actual: &str) {
    if !expected.trim().is_empty() && expected != actual {
        diagnostics.push(format!("proof receipt field `{field_name}` expected `{expected}` but got `{actual}`"));
    }
}

fn checker_identity_completeness_diagnostics(rail: &ProofRail) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_required_config(&mut diagnostics, FIELD_RECEIPT_PROVENANCE, "runner", expected_checker_id(rail));
    if expected_producer_id(rail) == expected_checker_id(rail) && !expected_producer_id(rail).trim().is_empty() {
        diagnostics.push(format!(
            "proof rail configured `{FIELD_RECEIPT_PROVENANCE}.runner` must differ from producer `{}`",
            expected_producer_id(rail)
        ));
    }
    diagnostics
}

fn verifier_identity_completeness_diagnostics(rail: &ProofRail) -> Vec<String> {
    let verifier = &rail.receipt_verifier;
    let mut diagnostics = Vec::new();
    if verifier_mode(rail) == VERIFIER_MODE_PRODUCTION {
        push_required_config(&mut diagnostics, FIELD_VERIFIER_IDENTITY, "verifier_id", &verifier.verifier_id);
        push_required_config(
            &mut diagnostics,
            FIELD_VERIFIER_IDENTITY,
            "verifier_binary_digest",
            &verifier.verifier_binary_digest,
        );
        push_required_config(&mut diagnostics, FIELD_SOLVER_IDENTITY, "solver_id", &verifier.solver_id);
        push_required_config(&mut diagnostics, FIELD_SOLVER_IDENTITY, "solver_digest", &verifier.solver_digest);
        push_required_config(&mut diagnostics, FIELD_TOOLCHAIN_IDENTITY, "toolchain_id", &verifier.toolchain_id);
        push_required_config(
            &mut diagnostics,
            FIELD_TOOLCHAIN_IDENTITY,
            "toolchain_digest",
            &verifier.toolchain_digest,
        );
        if verifier.verifier_nix_closure_digest.trim().is_empty()
            && verifier.verifier_nix_closure_graph_digest.trim().is_empty()
        {
            diagnostics.push(
                "production verifier identity requires `verifier_nix_closure_digest` or `verifier_nix_closure_graph_digest`"
                    .to_owned(),
            );
        }
    } else {
        push_required_config(&mut diagnostics, FIELD_VERIFIER_IDENTITY, "verifier_id", &verifier.verifier_id);
        push_required_config(
            &mut diagnostics,
            FIELD_VERIFIER_IDENTITY,
            "verifier_binary_digest",
            &verifier.verifier_binary_digest,
        );
        push_required_config(&mut diagnostics, FIELD_TOOLCHAIN_IDENTITY, "toolchain_id", &verifier.toolchain_id);
        push_required_config(
            &mut diagnostics,
            FIELD_TOOLCHAIN_IDENTITY,
            "toolchain_digest",
            &verifier.toolchain_digest,
        );
    }
    diagnostics
}

fn push_required_config(diagnostics: &mut Vec<String>, parent_field: &str, field_name: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("proof rail is missing `{parent_field}.{field_name}`"));
    }
}

fn expected_producer_id(rail: &ProofRail) -> &str {
    rail.receipt_verifier.verifier_id.as_str()
}

fn expected_checker_id(rail: &ProofRail) -> &str {
    rail.receipt_provenance.runner.as_str()
}

fn default_non_claims() -> Vec<String> {
    [
        NON_CLAIM_WHOLE_PROGRAM,
        NON_CLAIM_CERTIFICATION,
        NON_CLAIM_BUSINESS_DOMAIN,
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

fn policy_is_verified(policy: &PolicyEntry) -> bool {
    policy.current_enforcement_mode == EnforcementMode::Verified
        || policy.target_enforcement_mode == EnforcementMode::Verified
}

fn check_policy_proof_rails(
    registry: &PolicyRegistry,
    policy: &PolicyEntry,
    sources: &[SourceFile],
) -> Vec<Diagnostic> {
    if policy.proof_rails.is_empty() {
        return vec![proof_diagnostic(
            policy,
            "policy",
            None,
            "verified policy is missing proof rail metadata".to_owned(),
        )];
    }

    policy
        .proof_rails
        .iter()
        .enumerate()
        .flat_map(|(rail_index, rail)| check_proof_rail(registry, policy, rail_index, rail, sources))
        .collect()
}

fn check_proof_rail(
    registry: &PolicyRegistry,
    policy: &PolicyEntry,
    rail_index: usize,
    rail: &ProofRail,
    sources: &[SourceFile],
) -> Vec<Diagnostic> {
    if !rail_is_relevant(rail, sources) {
        return Vec::new();
    }

    let mut diagnostics = checker_registry_diagnostics(registry, rail)
        .into_iter()
        .map(|message| proof_diagnostic(policy, "policy", None, message))
        .collect::<Vec<_>>();
    diagnostics.extend(
        path_pattern_diagnostics_inner(&rail.source_paths, sources, FIELD_SOURCE_PATHS)
            .into_iter()
            .map(|message| proof_diagnostic(policy, first_path_or_policy(&rail.source_paths), None, message)),
    );

    if matching_sources(&rail.source_paths, sources).is_empty() {
        diagnostics.push(proof_diagnostic(
            policy,
            first_path_or_policy(&rail.source_paths),
            None,
            format!(
                "verified proof rail `{}` has no checked source matching [{}]",
                rail.backend,
                rail.source_paths.join(", ")
            ),
        ));
    }

    let receipt_patterns = receipt_paths(rail);
    diagnostics.extend(
        path_pattern_diagnostics_inner(&receipt_patterns, sources, FIELD_RECEIPT_PATHS)
            .into_iter()
            .map(|message| proof_diagnostic(policy, first_path_or_policy(&receipt_patterns), None, message)),
    );
    let proof_sources = matching_sources(&receipt_patterns, sources);
    if proof_sources.is_empty() {
        diagnostics.push(proof_diagnostic(
            policy,
            first_path_or_policy(&receipt_patterns),
            None,
            format!(
                "verified proof rail `{}` has no checked proof receipt matching [{}]",
                rail.backend,
                receipt_patterns.join(", ")
            ),
        ));
        return diagnostics;
    }

    if proof_sources.iter().any(|proof_source| {
        validate_proof_receipt_with_identity(rail, Some(&policy.policy_id), Some(rail_index), sources, proof_source)
            .is_empty()
    }) {
        return diagnostics;
    }

    let first_failure = validate_proof_receipt_with_identity(
        rail,
        Some(&policy.policy_id),
        Some(rail_index),
        sources,
        proof_sources[0],
    );
    diagnostics.push(proof_diagnostic(
        policy,
        &proof_sources[0].path,
        None,
        format!("verified proof rail `{}` failed receipt validation: {}", rail.backend, first_failure.join("; ")),
    ));
    diagnostics
}

fn checker_registry_diagnostics(registry: &PolicyRegistry, rail: &ProofRail) -> Vec<String> {
    if !rail_requires_structured_verus_receipt(rail) {
        return Vec::new();
    }
    let checker_id = expected_checker_id(rail);
    if checker_id.trim().is_empty() {
        return vec![format!(
            "proof rail is missing `{FIELD_RECEIPT_PROVENANCE}.runner` checker identity"
        )];
    }
    if registry.has_checker_id(checker_id) {
        Vec::new()
    } else {
        vec![format!(
            "proof rail checker `{checker_id}` is not registered in checker_ids"
        )]
    }
}

fn check_verified_module_coverage(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<Diagnostic> {
    sources
        .iter()
        .filter(|source| source_has_verified_claim(source))
        .filter(|source| !matching_verus_rail_exists(registry, selection, source))
        .map(verified_module_coverage_diagnostic)
        .collect()
}

fn source_has_verified_claim(source: &SourceFile) -> bool {
    let normalized_path = normalize_path(&source.path);
    normalized_path.starts_with(VERIFIED_PATH_PREFIX)
        || normalized_path.contains(VERIFIED_PATH_COMPONENT)
        || source.text.contains(FORMAL_PROOF_CLAIM_MARKER)
}

fn matching_verus_rail_exists(registry: &PolicyRegistry, selection: &FamilySelection, source: &SourceFile) -> bool {
    registry
        .policies
        .iter()
        .filter(|policy| selection.includes(&policy.pillar))
        .filter(|policy| policy_is_verified(policy))
        .flat_map(|policy| policy.proof_rails.iter())
        .filter(|rail| rail.backend == VERUS_BACKEND)
        .any(|rail| rail.source_paths.iter().any(|pattern| path_matches_pattern(pattern, &source.path)))
}

fn verified_module_coverage_diagnostic(source: &SourceFile) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: VERIFIED_MODULE_COVERAGE_POLICY_ID.to_owned(),
        source_section: VERIFIED_MODULE_COVERAGE_SOURCE.to_owned(),
        path: source.path.clone(),
        line: None,
        severity: crate::model::Severity::Error,
        message: "manual_review_required: verified module or formal-proof claim has no matching Verus proof rail"
            .to_owned(),
        remediation: VERIFIED_MODULE_REMEDIATION.to_owned(),
    })
}

fn rail_is_relevant(rail: &ProofRail, sources: &[SourceFile]) -> bool {
    crate::proof_artifact::proof_rail_is_relevant(rail, sources)
}

fn receipt_paths(rail: &ProofRail) -> Vec<String> {
    crate::proof_artifact::receipt_paths(rail)
}

fn first_path_or_policy(paths: &[String]) -> &str {
    paths.first().map_or("policy", String::as_str)
}

fn rail_requires_structured_verus_receipt(rail: &ProofRail) -> bool {
    rail.backend == VERUS_BACKEND || rail.receipt_provenance.schema_version == VERUS_PROOF_RECEIPT_SCHEMA_VERSION
}

fn verifier_mode(rail: &ProofRail) -> &str {
    if rail.verifier_mode.trim().is_empty() {
        VERIFIER_MODE_FIXTURE
    } else {
        rail.verifier_mode.as_str()
    }
}

fn expected_path_bindings(
    patterns: &[String],
    sources: &[SourceFile],
    field_name: &str,
) -> Result<Vec<VerusProofBinding>, Vec<String>> {
    let identities = crate::proof_artifact::source_identities_for_patterns(patterns, sources, field_name)?;
    Ok(identities
        .into_iter()
        .map(|identity| VerusProofBinding {
            path: identity.path,
            blake3: identity.blake3,
        })
        .collect())
}

fn path_pattern_diagnostics_inner(patterns: &[String], sources: &[SourceFile], field_name: &str) -> Vec<String> {
    crate::proof_artifact::path_pattern_diagnostics(patterns, sources, field_name)
}

fn path_matches_pattern(pattern: &str, path: &str) -> bool {
    crate::proof_artifact::path_matches_pattern(pattern, path)
}

fn normalize_path(path: &str) -> String {
    crate::proof_artifact::normalize_path(path)
}

fn receipt_binding_enabled(rail: &ProofRail) -> bool {
    rail.receipt_binding == crate::proof_artifact::RECEIPT_BINDING_BLAKE3_COMMAND_AND_SOURCE
}

fn receipt_binding_matches(rail: &ProofRail, sources: &[SourceFile], proof_source: &SourceFile) -> bool {
    expected_receipt_binding_lines(rail, sources)
        .iter()
        .all(|line| proof_source.text.lines().any(|receipt_line| receipt_line == line))
}

fn receipt_provenance_enabled(rail: &ProofRail) -> bool {
    let provenance = &rail.receipt_provenance;
    !provenance.schema_version.is_empty()
        || !provenance.runner.is_empty()
        || !provenance.environment.is_empty()
        || provenance.requires_backend
        || provenance.requires_trust_boundary
}

fn receipt_provenance_matches(rail: &ProofRail, proof_source: &SourceFile) -> bool {
    expected_receipt_provenance_lines(rail)
        .iter()
        .all(|line| proof_source.text.lines().any(|receipt_line| receipt_line == line))
}

fn receipt_verifier_enabled(rail: &ProofRail) -> bool {
    let verifier = &rail.receipt_verifier;
    !verifier.verifier_id.is_empty()
        || !verifier.verifier_digest.is_empty()
        || !verifier.verifier_binary_path.is_empty()
        || !verifier.verifier_binary_digest.is_empty()
        || !verifier.verifier_build_closure.is_empty()
        || !verifier.verifier_build_closure_digest.is_empty()
        || !verifier.verifier_build_closure_paths.is_empty()
        || !verifier.verifier_nix_derivation.is_empty()
        || !verifier.verifier_nix_derivation_digest.is_empty()
        || !verifier.verifier_nix_closure.is_empty()
        || !verifier.verifier_nix_closure_digest.is_empty()
        || !verifier.verifier_nix_closure_paths.is_empty()
        || !verifier.verifier_nix_closure_graph_digest.is_empty()
        || !verifier.verifier_nix_closure_graph_paths.is_empty()
        || !verifier.toolchain_id.is_empty()
        || !verifier.toolchain_digest.is_empty()
        || !verifier.solver_id.is_empty()
        || !verifier.solver_digest.is_empty()
}

fn receipt_verifier_matches(rail: &ProofRail, sources: &[SourceFile], proof_source: &SourceFile) -> bool {
    expected_receipt_verifier_lines(rail, sources)
        .iter()
        .all(|line| proof_source.text.lines().any(|receipt_line| receipt_line == line))
}

fn expected_receipt_verifier_lines(rail: &ProofRail, sources: &[SourceFile]) -> Vec<String> {
    let ReceiptVerifier {
        verifier_id,
        verifier_digest,
        verifier_binary_path,
        verifier_binary_digest,
        verifier_build_closure,
        verifier_build_closure_digest,
        verifier_build_closure_paths,
        verifier_nix_derivation,
        verifier_nix_derivation_digest,
        verifier_nix_closure,
        verifier_nix_closure_digest,
        verifier_nix_closure_paths,
        verifier_nix_closure_graph_digest,
        verifier_nix_closure_graph_paths,
        toolchain_id,
        toolchain_digest,
        solver_id,
        solver_digest,
    } = &rail.receipt_verifier;
    let mut lines = Vec::new();
    if !verifier_id.is_empty() {
        lines.push(format!("proof-verifier: {verifier_id}"));
    }
    if !verifier_digest.is_empty() {
        lines.push(format!("proof-verifier-blake3: {verifier_digest}"));
    }
    if !verifier_binary_path.is_empty() {
        lines.push(format!("proof-verifier-binary: {verifier_binary_path}"));
    }
    if !verifier_binary_digest.is_empty() {
        lines.push(format!("proof-verifier-binary-blake3: {verifier_binary_digest}"));
    }
    if !verifier_build_closure.is_empty() {
        lines.push(format!("proof-verifier-build-closure: {verifier_build_closure}"));
    }
    if !verifier_build_closure_digest.is_empty() {
        let digest = build_closure_digest(verifier_build_closure_paths, sources)
            .unwrap_or_else(|| verifier_build_closure_digest.clone());
        lines.push(format!("proof-verifier-build-closure-blake3: {digest}"));
    }
    if !verifier_nix_derivation.is_empty() {
        lines.push(format!("proof-verifier-nix-derivation: {verifier_nix_derivation}"));
    }
    if !verifier_nix_derivation_digest.is_empty() {
        lines.push(format!("proof-verifier-nix-derivation-blake3: {verifier_nix_derivation_digest}"));
    }
    if !verifier_nix_closure.is_empty() {
        lines.push(format!("proof-verifier-nix-closure: {verifier_nix_closure}"));
    }
    if !verifier_nix_closure_digest.is_empty() {
        let digest = build_closure_digest(verifier_nix_closure_paths, sources)
            .unwrap_or_else(|| verifier_nix_closure_digest.clone());
        lines.push(format!("proof-verifier-nix-closure-blake3: {digest}"));
    }
    if !verifier_nix_closure_graph_digest.is_empty() {
        let digest = nix_closure_graph_digest(verifier_nix_closure_graph_paths, sources)
            .unwrap_or_else(|| verifier_nix_closure_graph_digest.clone());
        lines.push(format!("proof-verifier-nix-closure-graph-blake3: {digest}"));
    }
    if !toolchain_id.is_empty() {
        lines.push(format!("proof-toolchain: {toolchain_id}"));
    }
    if !toolchain_digest.is_empty() {
        lines.push(format!("proof-toolchain-blake3: {toolchain_digest}"));
    }
    if !solver_id.is_empty() {
        lines.push(format!("proof-solver: {solver_id}"));
    }
    if !solver_digest.is_empty() {
        lines.push(format!("proof-solver-blake3: {solver_digest}"));
    }
    lines
}

fn build_closure_digest(paths: &[String], sources: &[SourceFile]) -> Option<String> {
    crate::proof_artifact::build_closure_digest(paths, sources)
}

fn nix_closure_graph_digest(paths: &[String], sources: &[SourceFile]) -> Option<String> {
    crate::proof_artifact::nix_closure_graph_digest(paths, sources).ok().flatten()
}

fn expected_receipt_provenance_lines(rail: &ProofRail) -> Vec<String> {
    let ReceiptProvenance {
        schema_version,
        runner,
        environment,
        requires_backend,
        requires_trust_boundary,
    } = &rail.receipt_provenance;
    let mut lines = Vec::new();
    if !schema_version.is_empty() {
        lines.push(format!("proof-receipt-schema: {schema_version}"));
    }
    if *requires_backend {
        lines.push(format!("proof-backend: {}", rail.backend));
    }
    if !runner.is_empty() {
        lines.push(format!("proof-runner: {runner}"));
    }
    if !environment.is_empty() {
        lines.push(format!("proof-environment: {environment}"));
    }
    if *requires_trust_boundary {
        lines.extend(rail.trust_boundaries.iter().map(|boundary| format!("trust-boundary: {boundary}")));
    }
    lines
}

fn expected_receipt_binding_lines(rail: &ProofRail, sources: &[SourceFile]) -> Vec<String> {
    crate::proof_artifact::receipt_binding_lines(rail, sources)
}

fn receipt_lines(lines: &[String]) -> String {
    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

fn proof_diagnostic(policy: &PolicyEntry, path: &str, line: Option<u32>, message: String) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: path.to_owned(),
        line,
        severity: policy.default_severity,
        message,
        remediation: VERIFIED_PROOF_REMEDIATION.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::model::ProofRail;
    use crate::model::ReceiptProvenance;
    use crate::model::ReceiptVerifier;

    const POLICY_ID: &str = "octet.verus_proof_rails.addition-bound";
    const SECOND_POLICY_ID: &str = "octet.verus_proof_rails.ordering-reflexive-production";
    const SOURCE_PATH: &str = "src/verified/addition.rs";
    const SECOND_SOURCE_PATH: &str = "src/verified/ordering.rs";
    const PROOF_PATH: &str = "verus/proofs/addition_bound.rs";
    const SECOND_PROOF_PATH: &str = "verus/proofs/ordering_reflexive.rs";
    const PASS_SIGNAL: &str = "proof-result: pass octet-addition-bound";
    const COMMAND_BLAKE3: &str = "command-digest";
    const PROOF_ARTIFACT: &str = "proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt";
    const SECOND_PROOF_ARTIFACT: &str = "proofs/octet.verus_proof_rails.ordering-reflexive-production-rail-0.receipt";
    const STDOUT_LOCATOR: &str = "proofs/octet.verus_proof_rails.addition-bound-rail-0.stdout";
    const STDERR_LOCATOR: &str = "proofs/octet.verus_proof_rails.addition-bound-rail-0.stderr";
    const SECOND_STDOUT_LOCATOR: &str = "proofs/octet.verus_proof_rails.ordering-reflexive-production-rail-0.stdout";
    const SECOND_STDERR_LOCATOR: &str = "proofs/octet.verus_proof_rails.ordering-reflexive-production-rail-0.stderr";

    #[test]
    fn structured_verus_receipt_accepts_matching_fields() {
        let rail = verus_rail(VERIFIER_MODE_FIXTURE);
        let sources = source_files();
        let proof_sources = proof_files();
        let receipt = valid_receipt(&rail, &sources, &proof_sources);
        let lock = valid_lock(&rail, &sources, &proof_sources);
        assert_eq!(receipt.producer_id, rail.receipt_verifier.verifier_id);
        assert_eq!(receipt.checker_id, rail.receipt_provenance.runner);
        assert_eq!(receipt.package_lock_blake3, proof_package_lock_blake3(&lock));
        let proof_source = SourceFile::new(PROOF_ARTIFACT, serde_json::to_string(&receipt).expect("receipt json"));

        let diagnostics = validate_proof_receipt_with_identity(
            &rail,
            Some(POLICY_ID),
            Some(0),
            &all_validation_sources(),
            &proof_source,
        );

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn verus_proof_package_lock_validates_exact_material() {
        let rail = verus_rail(VERIFIER_MODE_PRODUCTION);
        let sources = source_files();
        let proof_sources = proof_files();
        let lock = valid_lock(&rail, &sources, &proof_sources);

        let diagnostics = validate_verus_proof_package_lock(&lock_input(&rail, &sources, &proof_sources), &lock);

        assert!(diagnostics.is_empty(), "unexpected lock diagnostics: {diagnostics:?}");
        assert_eq!(lock.schema_version, VERUS_PROOF_PACKAGE_LOCK_SCHEMA_VERSION);
        assert_eq!(lock.producer_id, rail.receipt_verifier.verifier_id);
        assert_eq!(lock.checker_id, rail.receipt_provenance.runner);
        assert_eq!(lock.command_argv, rail.command_argv);
        assert_eq!(lock.environment_allowlist, rail.environment_allowlist);
        assert!(!proof_package_lock_blake3(&lock).is_empty());
    }

    #[test]
    fn verus_proof_package_lock_rejects_stale_material_and_invalid_shape() {
        let rail = verus_rail(VERIFIER_MODE_PRODUCTION);
        let sources = source_files();
        let proof_sources = proof_files();
        let mut lock = valid_lock(&rail, &sources, &proof_sources);
        lock.source_bindings[0].blake3 = "stale-source".to_owned();
        lock.proof_bindings[0].blake3 = "stale-proof".to_owned();
        lock.verifier_identity.verifier_id = "wrong-verifier".to_owned();
        lock.solver_identity.solver_id = "wrong-solver".to_owned();
        lock.command_argv = vec!["octet-verus".to_owned(), "wrong-proof.rs".to_owned()];
        lock.environment_allowlist.push(String::new());
        lock.trust_boundaries.clear();

        let diagnostics = validate_verus_proof_package_lock(&lock_input(&rail, &sources, &proof_sources), &lock);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("source_bindings")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("proof_bindings")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("verifier_identity.verifier_id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("solver_identity.solver_id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("command_argv")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("environment_allowlist")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("trust_boundaries")));
    }

    #[test]
    fn verus_proof_package_lock_rejects_malformed_closure_graph_offline() {
        let mut rail = verus_rail(VERIFIER_MODE_PRODUCTION);
        rail.receipt_verifier.verifier_nix_closure_graph_digest = "configured-graph-digest".to_owned();
        rail.receipt_verifier.verifier_nix_closure_graph_paths = vec!["artifacts/proof/closure.graph".to_owned()];
        let sources = source_files();
        let mut proof_sources = proof_files();
        proof_sources.push(SourceFile::new("artifacts/proof/closure.graph", "/nix/store/verus\nnot-a-count\n"));

        let errors = build_verus_proof_package_lock(&lock_input(&rail, &sources, &proof_sources))
            .expect_err("malformed closure graph should fail before proof execution");

        assert!(errors.iter().any(|error| error.contains("malformed closure graph")));
    }

    #[test]
    fn structured_verus_receipt_rejects_package_lock_hash_drift() {
        let rail = verus_rail(VERIFIER_MODE_FIXTURE);
        let sources = source_files();
        let proof_sources = proof_files();
        let mut receipt = valid_receipt(&rail, &sources, &proof_sources);
        receipt.package_lock_blake3 = "stale-lock".to_owned();
        let proof_source = SourceFile::new(PROOF_ARTIFACT, serde_json::to_string(&receipt).expect("receipt json"));

        let diagnostics = validate_proof_receipt_with_identity(
            &rail,
            Some(POLICY_ID),
            Some(0),
            &all_validation_sources(),
            &proof_source,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_PACKAGE_LOCK_BLAKE3)));
    }

    #[test]
    fn structured_verus_receipt_rejects_malformed_json() {
        let rail = verus_rail(VERIFIER_MODE_FIXTURE);
        let proof_source = SourceFile::new(PROOF_ARTIFACT, "proof-result: pass");

        let diagnostics =
            validate_proof_receipt_with_identity(&rail, Some(POLICY_ID), Some(0), &source_files(), &proof_source);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed JSON")));
    }

    #[test]
    fn structured_verus_receipt_rejects_stale_source_hash() {
        let rail = verus_rail(VERIFIER_MODE_FIXTURE);
        let sources = source_files();
        let proof_sources = proof_files();
        let mut receipt = valid_receipt(&rail, &sources, &proof_sources);
        receipt.source_bindings[0].blake3 = "stale".to_owned();
        let proof_source = SourceFile::new(PROOF_ARTIFACT, serde_json::to_string(&receipt).expect("receipt json"));

        let diagnostics = validate_proof_receipt_with_identity(
            &rail,
            Some(POLICY_ID),
            Some(0),
            &all_validation_sources(),
            &proof_source,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_SOURCE_BINDINGS)));
    }

    #[test]
    fn structured_verus_receipt_rejects_stale_proof_hash() {
        let rail = verus_rail(VERIFIER_MODE_FIXTURE);
        let sources = source_files();
        let proof_sources = proof_files();
        let mut receipt = valid_receipt(&rail, &sources, &proof_sources);
        receipt.proof_bindings[0].blake3 = "stale".to_owned();
        let proof_source = SourceFile::new(PROOF_ARTIFACT, serde_json::to_string(&receipt).expect("receipt json"));

        let diagnostics = validate_proof_receipt_with_identity(
            &rail,
            Some(POLICY_ID),
            Some(0),
            &all_validation_sources(),
            &proof_source,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_PROOF_BINDINGS)));
    }

    #[test]
    fn second_production_pilot_rejects_swapped_proof_binding_without_first_rail_noise() {
        let first_rail = verus_rail(VERIFIER_MODE_PRODUCTION);
        let first_sources = source_files();
        let first_proof_sources = proof_files();
        let first_receipt = valid_receipt_for(&ReceiptFixture {
            policy_id: POLICY_ID,
            rail: &first_rail,
            sources: &first_sources,
            proof_sources: &first_proof_sources,
            checked_scope: "formal-proof:production:octet.verus_proof_rails.addition-bound-production#rail-0",
            proof_artifact: PROOF_ARTIFACT,
            stdout_locator: STDOUT_LOCATOR,
            stderr_locator: STDERR_LOCATOR,
        });
        let first_proof_source =
            SourceFile::new(PROOF_ARTIFACT, serde_json::to_string(&first_receipt).expect("receipt json"));
        let first_diagnostics = validate_proof_receipt_with_identity(
            &first_rail,
            Some(POLICY_ID),
            Some(0),
            &first_validation_sources(),
            &first_proof_source,
        );
        assert!(first_diagnostics.is_empty(), "unexpected first rail diagnostics: {first_diagnostics:?}");

        let second_rail = second_verus_rail();
        let second_sources = second_source_files();
        let second_proof_sources = second_proof_files();
        let mut second_receipt = valid_receipt_for(&ReceiptFixture {
            policy_id: SECOND_POLICY_ID,
            rail: &second_rail,
            sources: &second_sources,
            proof_sources: &second_proof_sources,
            checked_scope: "formal-proof:production:octet.verus_proof_rails.ordering-reflexive-production#rail-0",
            proof_artifact: SECOND_PROOF_ARTIFACT,
            stdout_locator: SECOND_STDOUT_LOCATOR,
            stderr_locator: SECOND_STDERR_LOCATOR,
        });
        second_receipt.proof_bindings = first_receipt.proof_bindings.clone();
        let second_proof_source =
            SourceFile::new(SECOND_PROOF_ARTIFACT, serde_json::to_string(&second_receipt).expect("receipt json"));

        let diagnostics = validate_proof_receipt_with_identity(
            &second_rail,
            Some(SECOND_POLICY_ID),
            Some(0),
            &second_validation_sources(),
            &second_proof_source,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_PROOF_BINDINGS)));
        assert!(diagnostics.iter().all(|diagnostic| !diagnostic.contains(POLICY_ID)));
    }

    #[test]
    fn structured_verus_receipt_rejects_self_asserted_producer_only_promotion() {
        let rail = verus_rail(VERIFIER_MODE_FIXTURE);
        let sources = source_files();
        let proof_sources = proof_files();
        let mut receipt = valid_receipt(&rail, &sources, &proof_sources);
        receipt.checker_id.clone_from(&receipt.producer_id);
        let proof_source = SourceFile::new(PROOF_ARTIFACT, serde_json::to_string(&receipt).expect("receipt json"));

        let diagnostics = validate_proof_receipt_with_identity(
            &rail,
            Some(POLICY_ID),
            Some(0),
            &all_validation_sources(),
            &proof_source,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_CHECKER_ID)));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("self-asserted producer")));
    }

    #[test]
    fn verus_proof_receipt_builder_rejects_missing_checker_identity() {
        let mut rail = verus_rail(VERIFIER_MODE_FIXTURE);
        rail.receipt_provenance.runner.clear();
        let errors = build_verus_proof_receipt(&VerusProofReceiptInput {
            policy_id: POLICY_ID,
            rail_index: 0,
            rail: &rail,
            sources: &source_files(),
            proof_sources: &proof_files(),
            checked_scope: "formal-proof:fixture-only:octet.verus_proof_rails.addition-bound#rail-0",
            status: PROOF_STATUS_PASSED,
            command_blake3: COMMAND_BLAKE3,
            proof_artifact_identity: PROOF_ARTIFACT,
            stdout_artifact: log_artifact(STDOUT_LOCATOR, PASS_SIGNAL),
            stderr_artifact: log_artifact(STDERR_LOCATOR, ""),
        })
        .expect_err("missing checker identity should not build a promotion-grade receipt");

        assert!(errors.iter().any(|error| error.contains("receipt_provenance.runner")));
    }

    #[test]
    fn checker_registry_rejects_unknown_checker_id() {
        let mut rail = verus_rail(VERIFIER_MODE_FIXTURE);
        rail.receipt_provenance.runner = "unregistered-checker".to_owned();
        let registry = PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["octet-standards".to_owned()],
            policies: Vec::new(),
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        };

        let diagnostics = checker_registry_diagnostics(&registry, &rail);

        assert_eq!(diagnostics, vec!["proof rail checker `unregistered-checker` is not registered in checker_ids"]);
    }

    #[test]
    fn structured_verus_receipt_rejects_wrong_verifier_identity() {
        let rail = verus_rail(VERIFIER_MODE_FIXTURE);
        let sources = source_files();
        let proof_sources = proof_files();
        let mut receipt = valid_receipt(&rail, &sources, &proof_sources);
        receipt.verifier_identity.verifier_id = "wrong-verifier".to_owned();
        let proof_source = SourceFile::new(PROOF_ARTIFACT, serde_json::to_string(&receipt).expect("receipt json"));

        let diagnostics = validate_proof_receipt_with_identity(
            &rail,
            Some(POLICY_ID),
            Some(0),
            &all_validation_sources(),
            &proof_source,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("verifier_identity.verifier_id")));
    }

    #[test]
    fn structured_verus_receipt_rejects_missing_proof_artifact_identity() {
        let rail = verus_rail(VERIFIER_MODE_FIXTURE);
        let sources = source_files();
        let proof_sources = proof_files();
        let mut receipt = valid_receipt(&rail, &sources, &proof_sources);
        receipt.proof_artifact_identity.clear();
        let proof_source = SourceFile::new(PROOF_ARTIFACT, serde_json::to_string(&receipt).expect("receipt json"));

        let diagnostics = validate_proof_receipt_with_identity(
            &rail,
            Some(POLICY_ID),
            Some(0),
            &all_validation_sources(),
            &proof_source,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIELD_PROOF_ARTIFACT_IDENTITY)));
    }

    #[test]
    fn structured_verus_receipt_rejects_fixture_promoted_as_production_scope() {
        let rail = verus_rail(VERIFIER_MODE_FIXTURE);
        let sources = source_files();
        let proof_sources = proof_files();
        let mut receipt = valid_receipt(&rail, &sources, &proof_sources);
        receipt.checked_scope = "formal-proof:production:octet.verus_proof_rails.addition-bound#rail-0".to_owned();
        let proof_source = SourceFile::new(PROOF_ARTIFACT, serde_json::to_string(&receipt).expect("receipt json"));

        let diagnostics = validate_proof_receipt_with_identity(
            &rail,
            Some(POLICY_ID),
            Some(0),
            &all_validation_sources(),
            &proof_source,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(FIXTURE_ONLY_SCOPE_MARKER)));
    }

    #[test]
    fn production_verifier_identity_requires_solver_and_closure_identity() {
        let mut rail = verus_rail(VERIFIER_MODE_PRODUCTION);
        rail.receipt_verifier.solver_id.clear();
        rail.receipt_verifier.solver_digest.clear();
        rail.receipt_verifier.verifier_nix_closure_digest.clear();
        rail.receipt_verifier.verifier_nix_closure_graph_digest.clear();

        let diagnostics = verifier_identity_completeness_diagnostics(&rail);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("solver_identity.solver_id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("verifier_nix_closure_digest")));
    }

    #[test]
    fn exact_and_glob_path_matching_are_normalized() {
        let sources = vec![
            SourceFile::new("./src/verified/addition.rs", "source"),
            SourceFile::new("verus/proofs/addition_bound.rs", "proof"),
        ];
        let exact = vec!["src/verified/addition.rs".to_owned()];
        let glob = vec!["verus/proofs/*.rs".to_owned()];

        assert_eq!(matching_sources(&exact, &sources).len(), 1);
        assert_eq!(matching_sources(&glob, &sources).len(), 1);
    }

    #[test]
    fn path_matching_rejects_empty_missing_and_ambiguous_exact_patterns() {
        let sources = vec![SourceFile::new("src/verified/addition.rs", "source")];
        let empty = vec![String::new()];
        let missing = vec!["src/verified/missing.rs".to_owned()];

        assert!(path_pattern_diagnostics(&empty, &sources, FIELD_SOURCE_PATHS)[0].contains("empty"));
        assert!(
            path_pattern_diagnostics(&missing, &sources, FIELD_SOURCE_PATHS)[0].contains("matches no checked path")
        );
    }

    #[test]
    fn substring_only_patterns_do_not_match() {
        let sources = vec![SourceFile::new("src/verified/addition.rs", "source")];
        let substring = vec!["verified".to_owned()];

        assert!(matching_sources(&substring, &sources).is_empty());
        assert!(
            path_pattern_diagnostics(&substring, &sources, FIELD_SOURCE_PATHS)[0].contains("matches no checked path")
        );
    }

    #[test]
    fn verus_proof_receipt_contract_is_fresh() {
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("repo root");
        let path = repo_root.join(VERUS_PROOF_RECEIPT_CONTRACT_PATH);
        let expected = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read generated contract {}: {error}", path.display()));

        assert_eq!(expected, render_verus_proof_receipt_contract());
    }

    struct ReceiptFixture<'a> {
        policy_id: &'a str,
        rail: &'a ProofRail,
        sources: &'a [SourceFile],
        proof_sources: &'a [SourceFile],
        checked_scope: &'a str,
        proof_artifact: &'a str,
        stdout_locator: &'a str,
        stderr_locator: &'a str,
    }

    fn valid_lock(rail: &ProofRail, sources: &[SourceFile], proof_sources: &[SourceFile]) -> VerusProofPackageLock {
        build_verus_proof_package_lock(&lock_input(rail, sources, proof_sources)).expect("valid proof package lock")
    }

    fn lock_input<'a>(
        rail: &'a ProofRail,
        sources: &'a [SourceFile],
        proof_sources: &'a [SourceFile],
    ) -> VerusProofPackageLockInput<'a> {
        VerusProofPackageLockInput {
            policy_id: POLICY_ID,
            rail_index: 0,
            rail,
            sources,
            proof_sources,
        }
    }

    fn valid_receipt(rail: &ProofRail, sources: &[SourceFile], proof_sources: &[SourceFile]) -> VerusProofReceipt {
        valid_receipt_for(&ReceiptFixture {
            policy_id: POLICY_ID,
            rail,
            sources,
            proof_sources,
            checked_scope: "formal-proof:fixture-only:octet.verus_proof_rails.addition-bound#rail-0",
            proof_artifact: PROOF_ARTIFACT,
            stdout_locator: STDOUT_LOCATOR,
            stderr_locator: STDERR_LOCATOR,
        })
    }

    fn valid_receipt_for(fixture: &ReceiptFixture<'_>) -> VerusProofReceipt {
        build_verus_proof_receipt(&VerusProofReceiptInput {
            policy_id: fixture.policy_id,
            rail_index: 0,
            rail: fixture.rail,
            sources: fixture.sources,
            proof_sources: fixture.proof_sources,
            checked_scope: fixture.checked_scope,
            status: PROOF_STATUS_PASSED,
            command_blake3: COMMAND_BLAKE3,
            proof_artifact_identity: fixture.proof_artifact,
            stdout_artifact: log_artifact(fixture.stdout_locator, PASS_SIGNAL),
            stderr_artifact: log_artifact(fixture.stderr_locator, ""),
        })
        .expect("valid receipt")
    }

    fn fixture_solver_id(mode: &str) -> &'static str {
        if mode == VERIFIER_MODE_PRODUCTION {
            "z3-fixture"
        } else {
            ""
        }
    }

    fn fixture_solver_digest(mode: &str) -> &'static str {
        if mode == VERIFIER_MODE_PRODUCTION {
            "z3-fixture-digest"
        } else {
            ""
        }
    }

    fn verus_rail(mode: &str) -> ProofRail {
        ProofRail {
            backend: VERUS_BACKEND.to_owned(),
            command: "octet-verus verus/proofs/addition_bound.rs".to_owned(),
            command_argv: vec!["octet-verus".to_owned(), PROOF_PATH.to_owned()],
            source_paths: vec![SOURCE_PATH.to_owned()],
            proof_paths: vec![PROOF_PATH.to_owned()],
            receipt_paths: vec![PROOF_ARTIFACT.to_owned()],
            pass_signal: PASS_SIGNAL.to_owned(),
            trust_boundaries: vec![
                "fixture-scoped proof rail validates Octet proof plumbing and the declared addition-bound property only; it does not prove whole-program correctness"
                    .to_owned(),
            ],
            receipt_binding: crate::proof_artifact::RECEIPT_BINDING_BLAKE3_COMMAND_AND_SOURCE.to_owned(),
            receipt_provenance: ReceiptProvenance {
                schema_version: VERUS_PROOF_RECEIPT_SCHEMA_VERSION.to_owned(),
                runner: "octet-standards".to_owned(),
                environment: "cargo-octet-prove".to_owned(),
                requires_backend: true,
                requires_trust_boundary: true,
            },
            receipt_verifier: ReceiptVerifier {
                verifier_id: "octet-verus@fixture-v1".to_owned(),
                verifier_digest: "fixture-digest".to_owned(),
                verifier_binary_path: "octet-verus".to_owned(),
                verifier_binary_digest: "fixture-binary-digest".to_owned(),
                verifier_nix_closure: "fixture-closure".to_owned(),
                verifier_nix_closure_digest: "fixture-closure-digest".to_owned(),
                toolchain_id: "verus-fixture-wrapper/v1".to_owned(),
                toolchain_digest: "fixture-toolchain-digest".to_owned(),
                solver_id: fixture_solver_id(mode).to_owned(),
                solver_digest: fixture_solver_digest(mode).to_owned(),
                ..ReceiptVerifier::default()
            },
            verifier_mode: mode.to_owned(),
            checked_scope: format!("formal-proof:{FIXTURE_ONLY_SCOPE_MARKER}:{POLICY_ID}#rail-0"),
            timeout_ms: 1_000,
            stdout_max_bytes: 1_000,
            stderr_max_bytes: 1_000,
            environment_allowlist: vec!["PATH".to_owned()],
        }
    }

    fn second_verus_rail() -> ProofRail {
        let mut rail = verus_rail(VERIFIER_MODE_PRODUCTION);
        rail.command = "octet-production-verus verus/proofs/ordering_reflexive.rs".to_owned();
        rail.command_argv = vec!["octet-production-verus".to_owned(), SECOND_PROOF_PATH.to_owned()];
        rail.source_paths = vec![SECOND_SOURCE_PATH.to_owned()];
        rail.proof_paths = vec![SECOND_PROOF_PATH.to_owned()];
        rail.receipt_paths = vec![SECOND_PROOF_ARTIFACT.to_owned()];
        rail.trust_boundaries = vec![
            "production Verus binary release verifies only the declared reflexive-order proof target; it does not prove whole-program correctness"
                .to_owned(),
        ];
        rail.checked_scope = format!("formal-proof:production:{SECOND_POLICY_ID}#rail-0");
        rail
    }

    fn source_files() -> Vec<SourceFile> {
        vec![SourceFile::new(
            SOURCE_PATH,
            "pub fn checked_add_one(value: u64) -> u64 { value + 1 }\n",
        )]
    }

    fn second_source_files() -> Vec<SourceFile> {
        vec![SourceFile::new(
            SECOND_SOURCE_PATH,
            "pub fn reflexive_order_marker(value: u64) -> u64 { value }\n",
        )]
    }

    fn proof_files() -> Vec<SourceFile> {
        vec![SourceFile::new(
            PROOF_PATH,
            "verus! { proof fn addition_bound() ensures true {} }\n",
        )]
    }

    fn second_proof_files() -> Vec<SourceFile> {
        vec![SourceFile::new(
            SECOND_PROOF_PATH,
            "verus! { proof fn reflexive_order_holds(value: nat) ensures value <= value {} }\n",
        )]
    }

    fn all_validation_sources() -> Vec<SourceFile> {
        first_validation_sources()
    }

    fn first_validation_sources() -> Vec<SourceFile> {
        source_files().into_iter().chain(proof_files()).collect()
    }

    fn second_validation_sources() -> Vec<SourceFile> {
        second_source_files().into_iter().chain(second_proof_files()).collect()
    }

    fn log_artifact(locator: &str, text: &str) -> VerusLogArtifact {
        VerusLogArtifact {
            locator: locator.to_owned(),
            blake3: crate::proof_artifact::blake3_hex_text(text),
            bytes: text.len(),
            truncated: false,
        }
    }
}
