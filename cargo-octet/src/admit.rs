use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use artifact_auth_core::ALGORITHM_BLAKE3;
use artifact_auth_core::ArtifactRef as AuthArtifactRef;
use artifact_auth_core::ArtifactStatement;
use artifact_auth_core::AuthenticationPolicy;
use artifact_auth_core::AuthenticationScope;
use artifact_auth_core::KeyCurrentness;
use artifact_auth_core::POLICY_SCHEMA_V1;
use artifact_auth_core::TrustedKeyObservation;
use artifact_auth_ed25519::Ed25519Evidence;
use artifact_auth_ed25519::public_key_identity;
use artifact_auth_ed25519::verify_and_evaluate;
use serde::Deserialize;
use serde::Serialize;

use crate::artifact;
use crate::cap_fs::ArtifactRoot;
use crate::cap_fs::RootRelativePath;
use crate::cap_fs::SourceRoot;
use crate::cli::AdmissionArgs;
use crate::cli::AdmissionMode;
use crate::cli::CheckArgs;
use crate::cli::OutputFormat;
use crate::effect_closure;
use crate::effect_closure::AdmissionStatus;
use crate::effect_closure::ClosureInput;
use crate::effect_closure::EffectClosureReceipt;
use crate::effect_closure::FunctionAddressRecord;
use crate::effect_closure::ProviderIdentity;
use crate::effect_closure::ProviderKind;
use crate::effect_closure::SourceEnvelope;
use crate::effect_closure::WorkspacePolicy;
use crate::evidence::EvidenceTier;
use crate::evidence::ProvenanceRecord;
use crate::function_address::FunctionAddressCheck;
use crate::function_address::FunctionAddressMode;
use crate::kamacite_effect_closure;
use crate::kamacite_effect_closure::KamaciteEffectComparison;
use crate::kamacite_effect_closure::KamaciteEffectHandoff;
use crate::kamacite_effect_closure::KamaciteEffectProfile;
use crate::kamacite_effect_closure::KamaciteEffectSummaryProjection;
use crate::kamacite_effect_closure::KamaciteIdentity;

const EXIT_ADMITTED: i32 = 0;
const EXIT_DENIED: i32 = 1;
const ADMISSION_STATUS_SCHEMA: &str = "octet-effect-closure-status/v1";
const ADMISSION_VERIFICATION_SCHEMA: &str = "octet-effect-closure-verification/v1";
const DEPENDENCY_GRAPH_SCHEMA: &str = "octet-effect-closure-dependency-graph/v1";
const STATUS_ADMITTED: &str = "admitted";
const STATUS_DENIED: &str = "denied";
const STATUS_VALID: &str = "valid";
const STATUS_INVALID: &str = "invalid";
const PHASE_DIRECT_CHECK: &str = "direct-check";
const PHASE_CLOSURE: &str = "effect-closure";
const PHASE_KAMACITE: &str = "kamacite-effect-comparison";
const CHECK_DIR: &str = "check";
const COMMAND_FILE: &str = "command.txt";
const STATUS_FILE: &str = "status.json";
const SUMMARY_FILE: &str = "summary.txt";
const PROVENANCE_FILE: &str = "provenance.jsonl";
const WORKSPACE_POLICY_ARTIFACT: &str = "workspace-policy.json";
const DEPENDENCY_GRAPH_ARTIFACT: &str = "dependency-graph.json";
const KAMACITE_PROFILE_ARTIFACT: &str = "kamacite-effect-profile.json";
const KAMACITE_SUMMARY_ARTIFACT: &str = "kamacite-effect-summary.json";
const KAMACITE_COMPARISON_ARTIFACT: &str = "kamacite-effect-comparison.json";
const KAMACITE_HANDOFF_ARTIFACT: &str = "kamacite-effect-handoff.json";
const CHECK_STATUS_ARTIFACT: &str = "check/status.json";
const CHECK_PROVENANCE_ARTIFACT: &str = "check/provenance.jsonl";
const CHECK_FUNCTION_ADDRESS_ARTIFACT: &str = "check/function-addresses.json";
const FUNCTION_ADDRESS_ROOT_KIND: &str = "function-address-record-root";
const FUNCTION_ADDRESS_MODE_DENY: &str = "deny";
const CARGO_COMMAND: &str = "cargo";
const CARGO_METADATA_COMMAND: &str = "metadata";
const CARGO_METADATA_FORMAT_VERSION: &str = "1";
const TOOLCHAIN_ENV: &str = "RUSTUP_TOOLCHAIN";
const CHECKED_SCOPE_PREFIX: &str = "effect-closure:";
const PROVENANCE_SOURCE_ID: &str = "octet.runner.effect-closure";
const CHECKER_ID: &str = "cargo-octet";
const PROVIDER_SOURCE_TOOLCHAIN: &str = "rustc-toolchain";
const PROVIDER_SOURCE_WORKSPACE: &str = "workspace";
const PROVIDER_SOURCE_PATH_PREFIX: &str = "path-manifest:";
const TOOLCHAIN_PROVIDER_NAMES: &[&str] = &["alloc", "core", "rustc", "std"];
const TOP_LEVEL_ARTIFACTS: &[&str] = &[
    COMMAND_FILE,
    STATUS_FILE,
    SUMMARY_FILE,
    PROVENANCE_FILE,
    effect_closure::RECEIPT_FILE,
    WORKSPACE_POLICY_ARTIFACT,
    DEPENDENCY_GRAPH_ARTIFACT,
    KAMACITE_PROFILE_ARTIFACT,
    KAMACITE_SUMMARY_ARTIFACT,
    KAMACITE_COMPARISON_ARTIFACT,
    KAMACITE_HANDOFF_ARTIFACT,
];
const REQUIRED_PROVENANCE_ARTIFACTS: &[&str] = &[
    COMMAND_FILE,
    STATUS_FILE,
    SUMMARY_FILE,
    effect_closure::RECEIPT_FILE,
    WORKSPACE_POLICY_ARTIFACT,
    DEPENDENCY_GRAPH_ARTIFACT,
    CHECK_STATUS_ARTIFACT,
    CHECK_FUNCTION_ADDRESS_ARTIFACT,
];
pub(crate) const INTEGRATED_CLOSURE_RECEIPT_ARTIFACT_NAME: &str = "effect-closure.json";
pub(crate) const INTEGRATED_CLOSURE_POLICY_ARTIFACT_NAME: &str = "effect-closure-policy.json";
pub(crate) const INTEGRATED_CLOSURE_GRAPH_ARTIFACT_NAME: &str = "effect-closure-dependency-graph.json";
pub(crate) const INTEGRATED_CLOSURE_PARENT_STATUS_ARTIFACT_NAME: &str = "effect-closure-parent-status.json";
pub(crate) const INTEGRATED_CLOSURE_PROVENANCE_SOURCE_ID: &str = "octet.runner.effect-closure";
const NON_CLAIM_INTEGRITY: &str = "Admission verification checks stored BLAKE3 identity and linkage only; it does not rerun Cargo, source inference, dependency review, or compiler analysis, and it provides no signer authenticity, authorization, or defense against coherent replacement of every trust root.";
const ADMISSION_AUTH_POLICY_SCHEMA: &str = "octet-admission-auth-policy/v1";
const ADMISSION_AUTH_EVIDENCE_SCHEMA: &str = "octet-admission-auth-evidence/v1";
const ADMISSION_AUTH_DECISION_SCHEMA: &str = "octet-admission-auth-decision/v1";
const ADMISSION_AUTH_REPORT_SCHEMA: &str = "octet-admission-auth-verification/v1";
const ADMISSION_AUTH_BUNDLE_PROFILE: &str = "octet-effect-closure-bundle.v1";
const ADMISSION_AUTH_RECEIPT_PROFILE: &str = "octet-effect-closure-receipt.v1";
const ADMISSION_AUTH_COMMAND_PROFILE: &str = "octet-effect-closure-command.v1";
const ADMISSION_AUTH_CHECK_STATUS_PROFILE: &str = "octet-effect-closure-check-status.v1";
const ADMISSION_AUTH_FUNCTION_ADDRESS_PROFILE: &str = "octet-effect-closure-function-addresses.v1";
const ADMISSION_AUTH_WORKSPACE_POLICY_PROFILE: &str = "octet-effect-closure-workspace-policy.v1";
const ADMISSION_AUTH_CARGO_GRAPH_PROFILE: &str = "octet-effect-closure-cargo-graph.v1";
const ADMISSION_AUTH_CARGO_LOCK_PROFILE: &str = "octet-effect-closure-cargo-lock.v1";
const ADMISSION_AUTH_POLICY_IDENTITY_PROFILE: &str = "octet-admission-auth-policy-identity.v1";
const ADMISSION_AUTH_SCOPE_IDENTITY_PROFILE: &str = "octet-admission-auth-scope-identity.v1";
const ADMISSION_AUTH_DECISION_DOMAIN: &str = "octet.admission-auth.decision.v1";
const ADMISSION_AUTH_BUNDLE_DOMAIN: &str = "octet.admission-auth.bundle.v1";
const AUTH_BLAKE3_PREFIX: &str = "b3:";
const AUTH_HEX_CHARS: usize = 64;
const AUTH_PUBLIC_KEY_BYTES: usize = 32;
const AUTH_SIGNATURE_BYTES: usize = 64;
const NON_CLAIM_AUTH: &str = "Admission authentication verifies detached Ed25519 signatures over consumer-selected statements and checks consumer-owned producer authorization, key trust, and currentness observations only; it does not prove build correctness, source inference, provenance truth, release eligibility, or deployment safety, and it cannot detect a consumer policy that authorizes the wrong artifact.";
const AUTH_SIGNATURE_UNSUPPORTED_CODES: &[&str] = &[
    "auth.signature_algorithm_unsupported",
    "auth.statement_schema_unsupported",
    "admission_auth.evidence_schema_unsupported",
    "admission_auth.signature_malformed",
];
const AUTH_SIGNATURE_UNVERIFIED_CODES: &[&str] = &[
    "auth.signature_unverified",
    "auth.cryptographic_failure_present",
    "auth.cryptographic_key_identity_mismatch",
    "auth.signature_evidence_missing",
    "admission_auth.evidence_key_untrusted",
];
const AUTH_TRUST_CODES: &[&str] = &[
    "auth.key_untrusted",
    "auth.key_generation_stale",
    "auth.key_not_current",
    "auth.distinct_key_threshold_unsatisfied",
    "auth.threshold_zero",
    "auth.threshold_exceeds_key_count",
    "auth.trusted_key_count_invalid",
    "auth.trusted_key_identity_duplicate",
    "auth.key_generation_zero",
    "auth.allowed_purpose_missing",
    "auth.allowed_purpose_duplicate",
    "admission_auth.currentness_unacceptable",
    "admission_auth.evidence_key_untrusted",
    "admission_auth.evidence_key_duplicate",
];
const AUTH_AUTHORIZATION_CODES: &[&str] = &[
    "auth.purpose_unauthorized",
    "auth.policy_profile_mismatch",
    "auth.statement_scope_mismatch",
    "admission_auth.producer_unauthorized",
    "admission_auth.subject_mismatch",
    "admission_auth.receipt_identity_mismatch",
    "admission_auth.provenance_missing",
    "admission_auth.provenance_malformed",
];

#[derive(Clone, Debug, Deserialize)]
struct CheckStatus {
    status: String,
    exit_code: i32,
    metadata: CheckMetadata,
    error_findings: usize,
}

#[derive(Clone, Debug, Deserialize)]
struct CheckMetadata {
    rustc_version: String,
    toolchain: String,
    profile_name: String,
    profile_hash: String,
    config_hash: String,
}

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct FunctionAddressReport {
    mode: String,
    source_envelopes: Vec<FunctionAddressSourceEnvelope>,
    records: Vec<FunctionAddressRecord>,
    evidence_chain_handoff: FunctionAddressHandoff,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd)]
struct FunctionAddressSourceEnvelope {
    path: String,
    source_identity: String,
}

#[derive(Clone, Debug, Deserialize)]
struct FunctionAddressHandoff {
    canonical_roots: Vec<CanonicalRoot>,
}

#[derive(Clone, Debug, Deserialize)]
struct CanonicalRoot {
    kind: String,
    digest: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoMetadata {
    packages: Vec<CargoPackage>,
    workspace_members: Vec<String>,
    resolve: Option<CargoResolve>,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoPackage {
    id: String,
    name: String,
    version: String,
    source: Option<String>,
    manifest_path: String,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoResolve {
    nodes: Vec<CargoNode>,
}

#[derive(Clone, Debug, Deserialize)]
struct CargoNode {
    id: String,
    features: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct DependencyGraphArtifact {
    pub schema_version: String,
    pub target: String,
    pub toolchain: String,
    pub rustc_version: String,
    pub cargo_lock_blake3: String,
    pub providers: Vec<ProviderIdentity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct AdmissionStatusArtifact {
    schema_version: String,
    status: String,
    exit_code: i32,
    phase: String,
    profile_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    receipt_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kamacite: Option<KamaciteStatusArtifact>,
    diagnostics: Vec<String>,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct KamaciteStatusArtifact {
    schema_version: String,
    status: String,
    comparison_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    handoff_identity: Option<String>,
    effect_summary_identity: KamaciteIdentity,
}

#[derive(Clone)]
struct KamaciteBundle {
    profile_bytes: Vec<u8>,
    projection_bytes: Vec<u8>,
    comparison: KamaciteEffectComparison,
    comparison_bytes: Vec<u8>,
    handoff: Option<KamaciteEffectHandoff>,
    handoff_bytes: Option<Vec<u8>>,
}

struct PreparedKamacite {
    profile: KamaciteEffectProfile,
    profile_bytes: Vec<u8>,
    projection: KamaciteEffectSummaryProjection,
    projection_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct AdmissionVerificationReport {
    schema_version: String,
    artifact_dir: String,
    status: String,
    receipt_identity: Option<String>,
    checked_artifacts: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth: Option<AdmissionAuthReport>,
    diagnostics: Vec<String>,
    non_claims: Vec<String>,
}

/// Consumer-owned admission authentication policy.
///
/// This file is authored as Nickel by the consumer and exported deterministically to the JSON
/// shape below. It is supplied outside the candidate artifact directory and is the only source of
/// trust roots, expected identity, authorized scope, and currentness rules. A candidate bundle is
/// never consulted for policy, key bytes, or currentness observations. A policy is authored
/// after deciding which producer keys are trusted, for example:
///
/// ```json
/// {
///   "schema_version": "octet-admission-auth-policy/v1",
///   "policy_id": "release-admission",
///   "profile_id": "onix-artifact-auth.v1",
///   "domain": "octet.effect-closure.admission.v1",
///   "purpose": "admission-verification",
///   "scope_id": "effect-closure-admission",
///   "verifier_context": {
///     "profile": "octet-verifier-context.v1",
///     "algorithm": "blake3",
///     "digest_hex": "<consumer-owned verifier context digest>"
///   },
///   "expected_subject": {
///     "profile": "octet-effect-closure-bundle.v1",
///     "algorithm": "blake3",
///     "digest_hex": "<candidate bundle identity from a prior verification receipt>"
///   },
///   "expected_receipt_identity": "b3:<expected effect-closure receipt identity>",
///   "authorized_producers": [{
///     "producer_id": "release",
///     "key_ids": ["release-key"],
///     "purposes": ["admission-verification"],
///     "scope_ids": ["effect-closure-admission"]
///   }],
///   "threshold": 1,
///   "trusted_keys": [{
///     "producer_id": "release",
///     "key_id": "release-key",
///     "public_key_hex": "<64 lowercase hex characters of the consumer-owned Ed25519 public key>",
///     "generation": 1,
///     "allowed_purposes": ["admission-verification"],
///     "currentness": "current",
///     "currentness_ref": {
///       "profile": "octet-key-currentness-observation.v1",
///       "algorithm": "blake3",
///       "digest_hex": "<consumer-owned currentness observation digest>"
///     },
///     "observed_at_unix": 1760000000
///   }],
///   "currentness_rules": {
///     "accepted": ["current", "verification_overlap"],
///     "observation_profile": "octet-key-currentness-observation.v1",
///     "maximum_observation_age_seconds": 86400,
///     "evaluated_at_unix": 1760000000
///   },
///   "required_provenance": [
///     "octet-effect-closure-receipt.v1",
///     "octet-effect-closure-command.v1",
///     "octet-effect-closure-check-status.v1"
///   ]
/// }
/// ```
///
/// The verifier constructs the canonical statement scope from this policy and the candidate:
/// `domain`, `purpose`, `profile_id`, and `verifier_context` come from the policy; `subject` is
/// the BLAKE3 identity of every retained bundle file plus the receipt identity; parents are the
/// `policy_id` identity, the `scope_id` identity, and the receipt/command/check-status/
/// function-address/workspace-policy/cargo-graph/cargo-lock execution provenance identities.
/// Producers sign that exact scope; the candidate never supplies these values.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AdmissionAuthPolicyInput {
    schema_version: String,
    policy_id: String,
    profile_id: String,
    domain: String,
    purpose: String,
    scope_id: String,
    verifier_context: AuthArtifactRef,
    expected_subject: AuthArtifactRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    expected_receipt_identity: Option<String>,
    authorized_producers: Vec<AuthorizedProducerInput>,
    threshold: u16,
    trusted_keys: Vec<TrustedKeyInput>,
    currentness_rules: CurrentnessRulesInput,
    required_provenance: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AuthorizedProducerInput {
    producer_id: String,
    key_ids: Vec<String>,
    purposes: Vec<String>,
    scope_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TrustedKeyInput {
    producer_id: String,
    key_id: String,
    public_key_hex: String,
    generation: u64,
    allowed_purposes: Vec<String>,
    currentness: KeyCurrentness,
    currentness_ref: AuthArtifactRef,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    observed_at_unix: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct CurrentnessRulesInput {
    accepted: Vec<KeyCurrentness>,
    observation_profile: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    maximum_observation_age_seconds: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    evaluated_at_unix: Option<u64>,
}

/// Consumer-owned detached statements and raw Ed25519 signatures over those exact statements.
///
/// Evidence carries no cryptographic observations and no public keys: key bytes, trust,
/// currentness, and authorization come only from [`AdmissionAuthPolicyInput`].
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct AdmissionAuthEvidenceInput {
    schema_version: String,
    evidence: Vec<SignatureEvidenceInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct SignatureEvidenceInput {
    /// Exact canonical statement signed by the trusted key.
    statement: ArtifactStatement,
    /// Key generation claimed by the signature carrier; it must equal the policy generation.
    generation: u64,
    /// Raw 64-byte Ed25519 detached signature as lowercase hexadecimal.
    signature_hex: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
struct ArtifactAuthIssueV1 {
    code: String,
    field: String,
}

/// Versioned, recomputable admission authentication decision.
///
/// `decision_receipt_blake3` is BLAKE3 over the length-prefixed encoding in
/// [`auth_decision_bytes`]: the decision domain, every field in declaration order, the
/// `passed`/`denied` label, then the issue and non-claim counts and values. The whole decision is
/// embedded in the verification report so consumers can recompute it without the candidate bundle.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct ArtifactAuthDecisionV1 {
    schema_version: String,
    policy_id: String,
    policy_profile_id: String,
    subject_blake3: String,
    receipt_identity: String,
    policy_blake3: String,
    scope_blake3: String,
    checker_blake3: String,
    verified_key_blake3: Vec<String>,
    passed: bool,
    issues: Vec<ArtifactAuthIssueV1>,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
struct AdmissionAuthReport {
    schema_version: String,
    policy_id: String,
    integrity: String,
    signature_authenticity: String,
    authorization: String,
    trust_currentness: String,
    checker: String,
    passed: bool,
    decision_receipt_blake3: String,
    decision: ArtifactAuthDecisionV1,
    diagnostics: Vec<String>,
    non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct AdmissionVerificationInputs<'a> {
    pub auth_policy: Option<&'a Path>,
    pub auth_evidence: Option<&'a Path>,
}

struct TrustedKeyMaterial {
    observation: TrustedKeyObservation,
    public_key_bytes: [u8; AUTH_PUBLIC_KEY_BYTES],
    observed_at_unix: Option<u64>,
}

pub(crate) struct PreparedIntegratedClosure {
    policy: WorkspacePolicy,
    policy_bytes: Vec<u8>,
    pub baseline_path: PathBuf,
}

#[derive(Clone)]
pub(crate) struct IntegratedClosureArtifacts {
    pub parent: octet_standards::project_architecture_gate::EffectClosureParent,
    pub receipt_json: String,
    pub policy_json: String,
    pub dependency_graph_json: String,
    pub parent_status_json: String,
}

// r[impl octet.effect_closure.cli]
// r[impl octet.effect_closure.boundaries.default]
pub fn run(args: &AdmissionArgs) -> Result<i32, String> {
    match &args.mode {
        AdmissionMode::Run {
            profile,
            closure,
            policy,
            kamacite_effect_profile,
            scope_args,
            cargo_check_args,
        } => run_admission(
            args,
            profile,
            *closure,
            policy,
            kamacite_effect_profile.as_deref(),
            scope_args,
            cargo_check_args.as_deref(),
        ),
        AdmissionMode::Verify {
            auth_policy,
            auth_evidence,
        } => run_verification_with(args, AdmissionVerificationInputs {
            auth_policy: auth_policy.as_deref(),
            auth_evidence: auth_evidence.as_deref(),
        }),
    }
}

// r[impl octet.effect_closure.cli.admitted]
// r[impl octet.effect_closure.cli.check_failure]
fn run_admission(
    args: &AdmissionArgs,
    profile: &str,
    closure: bool,
    policy_path: &Path,
    kamacite_effect_profile: Option<&Path>,
    scope_args: &[String],
    cargo_check_args: Option<&[String]>,
) -> Result<i32, String> {
    if profile != effect_closure::DETERMINISTIC_CORE_PROFILE || !closure {
        return Err(format!(
            "effect closure v1 requires `--profile {} --closure`",
            effect_closure::DETERMINISTIC_CORE_PROFILE
        ));
    }
    crate::engine::reject_process_dylint_toml_override()?;

    let cwd = env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
    let workspace_config = crate::config::load(&cwd)?;
    let workspace_root = workspace_config.workspace_root;
    let selected_profile = crate::strict_profile::prepare_selected_profile(Some(profile), &workspace_root)?
        .ok_or_else(|| "effect closure requires a selected strict profile".to_owned())?;
    let prepared = prepare_inputs(&workspace_root, policy_path)?;
    let artifact_dir = absolute_from_workspace(&workspace_root, &args.artifact_dir);
    let check_dir = artifact_dir.join(CHECK_DIR);
    let root = prepare_artifact_root(&artifact_dir)?;
    let command_bytes = normalized_text_bytes(&args.command_line);

    let check = CheckArgs {
        scope_args: scope_args.to_vec(),
        cargo_check_args: cargo_check_args.map(<[String]>::to_vec),
        profile: Some(profile.to_owned()),
        output_format: OutputFormat::Human,
        artifact_dir: Some(check_dir.clone()),
        baseline: None,
        write_baseline: None,
        cache: false,
        function_addresses: FunctionAddressCheck {
            mode: FunctionAddressMode::Off,
            paths: Vec::new(),
            baseline: Some(prepared.baseline_path.clone()),
        },
        command_line: args.command_line.clone(),
    };
    let check_exit = crate::engine::run(&check)?;
    if check_exit != EXIT_ADMITTED {
        write_blocked_status(&root, profile, check_exit, &command_bytes)?;
        return Ok(check_exit);
    }
    verify_check_bundle(&check_dir)?;

    let check_status_bytes = read_required(&check_dir.join(STATUS_FILE))?;
    let function_address_bytes =
        read_required(&check_dir.join(crate::function_address::FUNCTION_ADDRESS_ARTIFACT_NAME))?;
    let check_status = parse_check_status(&check_status_bytes, profile)?;
    let function_report = parse_function_report(&function_address_bytes)?;
    let target = selected_target(cargo_check_args.unwrap_or_default())?;
    let (dependency_graph, dependency_graph_bytes) =
        dependency_graph(&workspace_root, cargo_check_args.unwrap_or_default(), &target, &check_status.metadata)?;
    let sources = read_selected_sources(&workspace_root, &selected_profile.core_source_scopes)?;
    validate_function_source_envelopes(&sources, &function_report)?;
    let record_root = function_address_record_root(&function_report)?;
    let catalog = effect_closure::embedded_catalog()?;

    let receipt = effect_closure::compute_closure(ClosureInput {
        profile_id: profile.to_owned(),
        profile_hash: check_status.metadata.profile_hash.clone(),
        config_hash: check_status.metadata.config_hash.clone(),
        command_blake3: crate::evidence::artifact_blake3_hex(&command_bytes),
        check_status_blake3: crate::evidence::artifact_blake3_hex(&check_status_bytes),
        function_address_artifact_blake3: crate::evidence::artifact_blake3_hex(&function_address_bytes),
        function_address_record_root: record_root,
        workspace_policy_blake3: crate::evidence::artifact_blake3_hex(&prepared.policy_bytes),
        cargo_graph_blake3: crate::evidence::artifact_blake3_hex(&dependency_graph_bytes),
        cargo_lock_blake3: dependency_graph.cargo_lock_blake3.clone(),
        target,
        toolchain: check_status.metadata.toolchain.clone(),
        sources,
        function_addresses: function_report.records,
        resolved_providers: dependency_graph.providers.clone(),
        catalog,
        policy: prepared.policy,
    });

    let kamacite = match kamacite_effect_profile {
        Some(path) if receipt.status == AdmissionStatus::Admitted => {
            Some(prepare_kamacite_bundle(&workspace_root, path, &receipt)?)
        }
        _ => None,
    };

    write_closure_bundle(
        &root,
        &artifact_dir,
        &command_bytes,
        &prepared.policy_bytes,
        &dependency_graph_bytes,
        &check_status_bytes,
        &function_address_bytes,
        &receipt,
        kamacite.as_ref(),
    )?;
    render_admission_output(args.output_format, &artifact_dir, &receipt, kamacite.as_ref())?;
    let admitted = receipt.status == AdmissionStatus::Admitted
        && kamacite.as_ref().is_none_or(|bundle| bundle.comparison.status == AdmissionStatus::Admitted);
    Ok(if admitted { EXIT_ADMITTED } else { EXIT_DENIED })
}

pub(crate) fn run_verification_with(
    args: &AdmissionArgs,
    inputs: AdmissionVerificationInputs<'_>,
) -> Result<i32, String> {
    let cwd = env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
    let artifact_dir = verification_artifact_dir(&cwd, &args.artifact_dir)?;
    let report = match (inputs.auth_policy, inputs.auth_evidence) {
        (None, None) => verify_bundle(&artifact_dir)?,
        (Some(policy), Some(evidence)) => verify_bundle_authorized(
            &artifact_dir,
            &consumer_input_path(&cwd, policy),
            &consumer_input_path(&cwd, evidence),
        )?,
        _ => {
            return Err("admission authentication requires both `--auth-policy` and `--auth-evidence`".to_owned());
        }
    };
    match args.output_format {
        OutputFormat::Human => {
            eprintln!("Effect closure admission verification: {}", report.status);
            eprintln!("Artifact directory: {}", report.artifact_dir);
            if let Some(identity) = &report.receipt_identity {
                eprintln!("Receipt identity: {identity}");
            }
            if let Some(auth) = &report.auth {
                eprintln!("Admission authentication: {}", if auth.passed { "passed" } else { "denied" });
                eprintln!(
                    "  integrity={} signature_authenticity={} authorization={} trust_currentness={} checker={}",
                    auth.integrity,
                    auth.signature_authenticity,
                    auth.authorization,
                    auth.trust_currentness,
                    auth.checker
                );
                eprintln!("  decision receipt: {}", auth.decision_receipt_blake3);
            }
            for diagnostic in &report.diagnostics {
                eprintln!("- {diagnostic}");
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("serialize admission verification report: {error}"))?
        ),
        OutputFormat::Sarif => unreachable!("SARIF is rejected by admission argument parsing"),
    }
    Ok(if report.diagnostics.is_empty() {
        EXIT_ADMITTED
    } else {
        EXIT_DENIED
    })
}

fn consumer_input_path(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        cwd.join(path)
    }
}

/// Resolves `admit verify --artifact-dir` the same way `admit` does: an absolute path is used as
/// given and a relative path is relative to the workspace root, not the current directory.
fn verification_artifact_dir(cwd: &Path, artifact_dir: &Path) -> Result<PathBuf, String> {
    if artifact_dir.is_absolute() {
        return Ok(artifact_dir.to_path_buf());
    }
    let workspace_root = crate::config::load(cwd)?.workspace_root;
    Ok(absolute_from_workspace(&workspace_root, artifact_dir))
}

pub(crate) fn prepare_integrated_closure(
    workspace_root: &Path,
    binding: &octet_standards::project_architecture_policy::ArchitectureAdmissionBinding,
) -> Result<PreparedIntegratedClosure, String> {
    prepare_inputs(workspace_root, Path::new(&binding.effect_closure_policy))
}

fn prepare_inputs(workspace_root: &Path, policy_path: &Path) -> Result<PreparedIntegratedClosure, String> {
    let policy_path = checked_workspace_path(workspace_root, policy_path, "effect-closure policy")?;
    let policy_bytes = read_required(&policy_path)?;
    let policy = serde_json::from_slice::<WorkspacePolicy>(&policy_bytes)
        .map_err(|error| format!("parse {}: {error}", policy_path.display()))?;
    let diagnostics = effect_closure::validate_workspace_policy(&policy);
    if !diagnostics.is_empty() {
        return Err(format!("invalid effect-closure workspace policy:\n- {}", diagnostics.join("\n- ")));
    }
    let baseline_path = checked_workspace_path(
        workspace_root,
        Path::new(&policy.function_address_baseline),
        "function-address baseline",
    )?;
    if !baseline_path.is_file() {
        return Err(format!("function-address baseline does not exist: {}", baseline_path.display()));
    }
    Ok(PreparedIntegratedClosure {
        policy,
        policy_bytes,
        baseline_path,
    })
}

// r[impl octet.effect_closure.kamacite_cli]
fn prepare_kamacite_bundle(
    workspace_root: &Path,
    profile_path: &Path,
    receipt: &EffectClosureReceipt,
) -> Result<KamaciteBundle, String> {
    let prepared = prepare_kamacite_inputs(workspace_root, profile_path)?;
    let comparison = kamacite_effect_closure::compare(receipt, &prepared.profile, &prepared.projection);
    let comparison_bytes = kamacite_effect_closure::canonical_json(&comparison)?.into_bytes();
    let (handoff, handoff_bytes) = if comparison.status == AdmissionStatus::Admitted {
        let handoff = kamacite_effect_closure::build_handoff(&comparison)?;
        let diagnostics = kamacite_effect_closure::validate_handoff(&handoff);
        if !diagnostics.is_empty() {
            return Err(format!(
                "new Kamacite effect handoff failed self-verification:\n- {}",
                render_kamacite_issues(&diagnostics)
            ));
        }
        let bytes = kamacite_effect_closure::canonical_json(&handoff)?.into_bytes();
        (Some(handoff), Some(bytes))
    } else {
        (None, None)
    };
    Ok(KamaciteBundle {
        profile_bytes: prepared.profile_bytes,
        projection_bytes: prepared.projection_bytes,
        comparison,
        comparison_bytes,
        handoff,
        handoff_bytes,
    })
}

fn prepare_kamacite_inputs(workspace_root: &Path, profile_path: &Path) -> Result<PreparedKamacite, String> {
    let profile_path = checked_workspace_path(workspace_root, profile_path, "Kamacite effect profile")?;
    let profile_bytes = read_required(&profile_path)?;
    let profile = serde_json::from_slice::<KamaciteEffectProfile>(&profile_bytes)
        .map_err(|error| format!("parse {}: {error}", profile_path.display()))?;
    let diagnostics = kamacite_effect_closure::validate_profile(&profile);
    if !diagnostics.is_empty() {
        return Err(format!(
            "invalid Kamacite effect profile {}:\n- {}",
            profile_path.display(),
            render_kamacite_issues(&diagnostics)
        ));
    }
    let projection_path = checked_workspace_path(
        workspace_root,
        Path::new(&profile.kamacite.projection),
        "Kamacite effect summary projection",
    )?;
    let projection_bytes = read_required(&projection_path)?;
    let observed = format!("b3:{}", crate::evidence::artifact_blake3_hex(&projection_bytes));
    if observed != profile.kamacite.projection_blake3 {
        return Err(format!(
            "stale Kamacite effect summary projection identity for {}: profile binds `{}`, file is `{observed}`",
            projection_path.display(),
            profile.kamacite.projection_blake3
        ));
    }
    let projection = serde_json::from_slice::<KamaciteEffectSummaryProjection>(&projection_bytes)
        .map_err(|error| format!("parse {}: {error}", projection_path.display()))?;
    let diagnostics = kamacite_effect_closure::validate_projection(&projection);
    if !diagnostics.is_empty() {
        return Err(format!(
            "invalid Kamacite effect summary projection {}:\n- {}",
            projection_path.display(),
            render_kamacite_issues(&diagnostics)
        ));
    }
    Ok(PreparedKamacite {
        profile,
        profile_bytes,
        projection,
        projection_bytes,
    })
}

fn render_kamacite_issues(issues: &[kamacite_effect_closure::ValidationIssue]) -> String {
    issues
        .iter()
        .map(|issue| format!("{}: {}", issue.code, issue.message))
        .collect::<Vec<_>>()
        .join("\n- ")
}

/// Opens the admission artifact directory and removes the previous bundle's top-level artifacts. A
/// non-empty directory that is not an existing admission bundle is refused before anything is
/// deleted, so `--artifact-dir` cannot remove unrelated files.
fn prepare_artifact_root(path: &Path) -> Result<ArtifactRoot, String> {
    let root = ArtifactRoot::open_or_create(path)?;
    if !root.read_entry_names()?.is_empty() && !is_existing_admission_bundle(&root, path)? {
        return Err(format!(
            "refusing to use artifact directory {}: it is not empty and is not an existing Octet admission bundle \
             (no admission `{STATUS_FILE}` or strict-profile `{CHECK_DIR}/` bundle); choose a new or empty directory",
            path.display()
        ));
    }
    for artifact in TOP_LEVEL_ARTIFACTS {
        root.remove_file_or_dir_all(&relative(artifact)?)?;
    }
    Ok(root)
}

/// Every completed or blocked admission run writes an admission-schema `status.json`; a run that
/// stopped after its direct check still leaves the strict-profile `check/` bundle markers.
fn is_existing_admission_bundle(root: &ArtifactRoot, path: &Path) -> Result<bool, String> {
    if root.is_file(&relative(STATUS_FILE)?)? {
        let status = serde_json::from_slice::<AdmissionStatusArtifact>(&read_required(&path.join(STATUS_FILE))?);
        if status.is_ok_and(|status| status.schema_version == ADMISSION_STATUS_SCHEMA) {
            return Ok(true);
        }
    }
    for marker in [CHECK_STATUS_ARTIFACT, CHECK_PROVENANCE_ARTIFACT] {
        if !root.is_file(&relative(marker)?)? {
            return Ok(false);
        }
    }
    Ok(true)
}

fn write_blocked_status(
    root: &ArtifactRoot,
    profile: &str,
    check_exit: i32,
    command_bytes: &[u8],
) -> Result<(), String> {
    let diagnostic =
        format!("direct strict-profile check failed with exit code {check_exit}; closure was not attempted");
    let status = AdmissionStatusArtifact {
        schema_version: ADMISSION_STATUS_SCHEMA.to_owned(),
        status: STATUS_DENIED.to_owned(),
        exit_code: check_exit,
        phase: PHASE_DIRECT_CHECK.to_owned(),
        profile_id: profile.to_owned(),
        receipt_identity: None,
        kamacite: None,
        diagnostics: vec![diagnostic.clone()],
        non_claims: vec![NON_CLAIM_INTEGRITY.to_owned()],
    };
    let status_bytes = pretty_json_bytes(&status)?;
    let summary = format!(
        "Octet effect closure admission\nStatus: denied\nPhase: direct-check\nProfile: {profile}\nDiagnostic: {diagnostic}\n"
    );
    root.write(&relative(COMMAND_FILE)?, command_bytes)?;
    root.write(&relative(STATUS_FILE)?, &status_bytes)?;
    root.write(&relative(SUMMARY_FILE)?, summary.as_bytes())
}

fn verify_check_bundle(check_dir: &Path) -> Result<(), String> {
    let root = ArtifactRoot::open_existing(check_dir)?;
    let files = root
        .read_files_recursive()?
        .into_iter()
        .map(|file| artifact::BundleFile {
            name: file.locator,
            contents: file.contents,
        })
        .collect::<Vec<_>>();
    let report = artifact::verify_bundle(artifact::ArtifactBundleInput {
        artifact_dir: check_dir.display().to_string(),
        files,
    });
    if report.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "strict-profile parent artifact verification failed:\n- {}",
            report.diagnostics.join("\n- ")
        ))
    }
}

fn parse_check_status(bytes: &[u8], expected_profile: &str) -> Result<CheckStatus, String> {
    let status =
        serde_json::from_slice::<CheckStatus>(bytes).map_err(|error| format!("parse check/status.json: {error}"))?;
    if status.exit_code != EXIT_ADMITTED
        || status.error_findings != 0
        || status.metadata.profile_name != expected_profile
    {
        return Err(format!(
            "direct check status is not eligible for closure: status={}, exit_code={}, errors={}, profile={}",
            status.status, status.exit_code, status.error_findings, status.metadata.profile_name
        ));
    }
    Ok(status)
}

fn parse_function_report(bytes: &[u8]) -> Result<FunctionAddressReport, String> {
    let report = serde_json::from_slice::<FunctionAddressReport>(bytes)
        .map_err(|error| format!("parse check/function-addresses.json: {error}"))?;
    if report.mode != FUNCTION_ADDRESS_MODE_DENY {
        return Err(format!("effect closure requires deny-mode function addresses, got `{}`", report.mode));
    }
    if report.records.is_empty() {
        return Err("effect closure requires at least one function-address record".to_owned());
    }
    Ok(report)
}

pub(crate) fn run_integrated_closure(
    workspace_root: &Path,
    binding: &octet_standards::project_architecture_policy::ArchitectureAdmissionBinding,
    selected_profile: &crate::strict_profile::SelectedStrictProfile,
    cargo_check_args: &[String],
    metadata: &crate::determinism::RunMetadata,
    command_line: &str,
    status_json: &str,
    function_address_json: &str,
    prepared: PreparedIntegratedClosure,
) -> Result<IntegratedClosureArtifacts, String> {
    if selected_profile.policy.profile_id != binding.strict_profile {
        return Err("architecture admission strict-profile binding does not match selected profile".to_owned());
    }
    let function_report = parse_function_report(function_address_json.as_bytes())?;
    let target = selected_target(cargo_check_args)?;
    let (dependency_graph, dependency_graph_bytes) =
        dependency_graph(workspace_root, cargo_check_args, &target, &CheckMetadata {
            rustc_version: metadata.rustc_version.clone(),
            toolchain: metadata.toolchain.clone(),
            profile_name: metadata.profile_name.clone(),
            profile_hash: metadata.profile_hash.clone(),
            config_hash: metadata.config_hash.clone(),
        })?;
    let sources = read_selected_sources(workspace_root, &selected_profile.core_source_scopes)?;
    validate_function_source_envelopes(&sources, &function_report)?;
    let record_root = function_address_record_root(&function_report)?;
    let command_bytes = normalized_text_bytes(command_line);
    let receipt = effect_closure::compute_closure(ClosureInput {
        profile_id: binding.strict_profile.clone(),
        profile_hash: metadata.profile_hash.clone(),
        config_hash: metadata.config_hash.clone(),
        command_blake3: crate::evidence::artifact_blake3_hex(&command_bytes),
        check_status_blake3: crate::evidence::artifact_blake3_hex(status_json.as_bytes()),
        function_address_artifact_blake3: crate::evidence::artifact_blake3_hex(function_address_json.as_bytes()),
        function_address_record_root: record_root,
        workspace_policy_blake3: crate::evidence::artifact_blake3_hex(&prepared.policy_bytes),
        cargo_graph_blake3: crate::evidence::artifact_blake3_hex(&dependency_graph_bytes),
        cargo_lock_blake3: dependency_graph.cargo_lock_blake3.clone(),
        target,
        toolchain: metadata.toolchain.clone(),
        sources,
        function_addresses: function_report.records,
        resolved_providers: dependency_graph.providers.clone(),
        catalog: effect_closure::embedded_catalog()?,
        policy: prepared.policy,
    });
    let diagnostics = effect_closure::verify_receipt_identity(&receipt);
    if !diagnostics.is_empty() {
        return Err(format!(
            "new integrated effect-closure receipt failed self-verification: {}",
            diagnostics.join("; ")
        ));
    }
    let parent = octet_standards::project_architecture_gate::EffectClosureParent {
        receipt_id: receipt.receipt_identity.clone(),
        admitted: receipt.status == AdmissionStatus::Admitted,
        evidence_tier: EvidenceTier::PolicyCheck.as_str().to_owned(),
        non_claims: receipt.non_claims.clone(),
    };
    Ok(IntegratedClosureArtifacts {
        parent,
        receipt_json: pretty_json_string(&receipt)?,
        policy_json: String::from_utf8(prepared.policy_bytes)
            .map_err(|error| format!("effect-closure policy is not UTF-8: {error}"))?,
        dependency_graph_json: String::from_utf8(dependency_graph_bytes)
            .map_err(|error| format!("effect-closure dependency graph is not UTF-8: {error}"))?,
        parent_status_json: status_json.to_owned(),
    })
}

fn function_address_record_root(report: &FunctionAddressReport) -> Result<String, String> {
    report
        .evidence_chain_handoff
        .canonical_roots
        .iter()
        .find(|root| root.kind == FUNCTION_ADDRESS_ROOT_KIND)
        .map(|root| root.digest.clone())
        .ok_or_else(|| "function-address receipt is missing its canonical record root".to_owned())
}

fn read_selected_sources(workspace_root: &Path, scopes: &[PathBuf]) -> Result<Vec<SourceEnvelope>, String> {
    let source_root = SourceRoot::open_existing(workspace_root)?;
    let mut selected = BTreeMap::<String, Vec<u8>>::new();
    for scope in scopes {
        let relative_scope = RootRelativePath::new(scope)?;
        if source_root.is_dir(&relative_scope)? {
            for file in source_root.read_files_recursive_from(&relative_scope)? {
                if Path::new(&file.locator).extension().is_some_and(|extension| extension == "rs") {
                    selected.insert(file.locator, file.contents);
                }
            }
        } else {
            if scope.extension().is_none_or(|extension| extension != "rs") {
                return Err(format!("selected core source file is not Rust: {}", scope.display()));
            }
            let source = source_root.read_to_string(&relative_scope)?;
            selected.insert(relative_scope.display(), source.into_bytes());
        }
    }
    if selected.is_empty() {
        return Err("selected core source scopes contain no Rust files".to_owned());
    }
    selected
        .into_iter()
        .map(|(path, bytes)| {
            let source = String::from_utf8(bytes)
                .map_err(|error| format!("selected Rust source `{path}` is not UTF-8: {error}"))?;
            let source_identity = crate::determinism::full_hash(&source);
            Ok(SourceEnvelope {
                path,
                source,
                source_identity,
            })
        })
        .collect()
}

fn validate_function_source_envelopes(
    sources: &[SourceEnvelope],
    report: &FunctionAddressReport,
) -> Result<(), String> {
    let selected = sources
        .iter()
        .map(|source| FunctionAddressSourceEnvelope {
            path: source.path.clone(),
            source_identity: source.source_identity.clone(),
        })
        .collect::<BTreeSet<_>>();
    let reported = report.source_envelopes.iter().cloned().collect::<BTreeSet<_>>();
    if selected == reported {
        Ok(())
    } else {
        Err("selected source identities do not exactly match deny-mode function-address envelopes".to_owned())
    }
}

fn dependency_graph(
    workspace_root: &Path,
    cargo_check_args: &[String],
    target: &str,
    check_metadata: &CheckMetadata,
) -> Result<(DependencyGraphArtifact, Vec<u8>), String> {
    let cargo_lock = read_required(&workspace_root.join("Cargo.lock"))?;
    let cargo_lock_blake3 = crate::evidence::artifact_blake3_hex(&cargo_lock);
    let mut command = Command::new(CARGO_COMMAND);
    command
        .arg(CARGO_METADATA_COMMAND)
        .arg("--format-version")
        .arg(CARGO_METADATA_FORMAT_VERSION)
        .arg("--locked")
        .arg("--filter-platform")
        .arg(target)
        .args(metadata_cargo_args(cargo_check_args)?)
        .current_dir(workspace_root)
        .env(TOOLCHAIN_ENV, &check_metadata.toolchain);
    let output = command.output().map_err(|error| format!("run cargo metadata: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed with status {:?}: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let metadata = serde_json::from_slice::<CargoMetadata>(&output.stdout)
        .map_err(|error| format!("parse cargo metadata JSON: {error}"))?;
    let workspace_members = metadata.workspace_members.into_iter().collect::<BTreeSet<_>>();
    let feature_map = metadata
        .resolve
        .map(|resolve| {
            resolve
                .nodes
                .into_iter()
                .map(|node| {
                    let mut features = node.features;
                    features.sort();
                    features.dedup();
                    (node.id, features)
                })
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut providers = Vec::new();
    for package in metadata.packages {
        let Some(features) = feature_map.get(&package.id).cloned() else {
            continue;
        };
        let workspace_package = workspace_members.contains(&package.id);
        let source = package_source(&package, workspace_package);
        providers.push(ProviderIdentity {
            kind: if workspace_package {
                ProviderKind::Workspace
            } else {
                ProviderKind::CargoPackage
            },
            name: package.name,
            version: package.version,
            source,
            target: target.to_owned(),
            features,
        });
    }
    for name in TOOLCHAIN_PROVIDER_NAMES {
        providers.push(ProviderIdentity {
            kind: ProviderKind::Toolchain,
            name: (*name).to_owned(),
            version: check_metadata.rustc_version.clone(),
            source: PROVIDER_SOURCE_TOOLCHAIN.to_owned(),
            target: target.to_owned(),
            features: Vec::new(),
        });
    }
    providers.sort();
    providers.dedup();
    let graph = DependencyGraphArtifact {
        schema_version: DEPENDENCY_GRAPH_SCHEMA.to_owned(),
        target: target.to_owned(),
        toolchain: check_metadata.toolchain.clone(),
        rustc_version: check_metadata.rustc_version.clone(),
        cargo_lock_blake3,
        providers,
    };
    let bytes = pretty_json_bytes(&graph)?;
    Ok((graph, bytes))
}

fn package_source(package: &CargoPackage, workspace_package: bool) -> String {
    if workspace_package {
        return PROVIDER_SOURCE_WORKSPACE.to_owned();
    }
    package.source.clone().unwrap_or_else(|| {
        let manifest_hash = fs::read(&package.manifest_path).map_or_else(
            |_| crate::determinism::full_hash(&package.manifest_path),
            |bytes| crate::evidence::artifact_blake3_hex(&bytes),
        );
        format!("{PROVIDER_SOURCE_PATH_PREFIX}{manifest_hash}")
    })
}

/// Selects the direct Cargo arguments that change which manifest, configuration, or feature set
/// `cargo metadata` resolves, so the recorded provider graph describes the checked build.
fn metadata_cargo_args(cargo_check_args: &[String]) -> Result<Vec<String>, String> {
    let mut args = Vec::new();
    let mut cargo_check_args = cargo_check_args.iter();
    while let Some(arg) = cargo_check_args.next() {
        if ["--manifest-path", "--config", "--features", "-F"].contains(&arg.as_str()) {
            let value = cargo_check_args.next().ok_or_else(|| format!("Cargo option `{arg}` is missing its value"))?;
            args.push(arg.clone());
            args.push(value.clone());
        } else if ["--offline", "--frozen", "--all-features", "--no-default-features"].contains(&arg.as_str())
            || arg.starts_with("--manifest-path=")
            || arg.starts_with("--config=")
            || arg.starts_with("--features=")
            || arg.starts_with("-F")
        {
            args.push(arg.clone());
        }
    }
    Ok(args)
}

fn selected_target(cargo_check_args: &[String]) -> Result<String, String> {
    let mut explicit = None;
    let mut index = 0;
    while index < cargo_check_args.len() {
        let arg = &cargo_check_args[index];
        if arg == "--target" {
            let value = cargo_check_args.get(index + 1).ok_or_else(|| "missing target after `--target`".to_owned())?;
            if explicit.replace(value.clone()).is_some() {
                return Err("cargo check arguments contain more than one target".to_owned());
            }
            index += 2;
            continue;
        }
        if let Some(value) = arg.strip_prefix("--target=") {
            if value.is_empty() {
                return Err("missing target after `--target=`".to_owned());
            }
            if explicit.replace(value.to_owned()).is_some() {
                return Err("cargo check arguments contain more than one target".to_owned());
            }
        }
        index += 1;
    }
    explicit.ok_or_else(|| {
        "effect closure v1 requires an explicit Cargo `--target TARGET` argument after `--`; ambient Cargo target configuration is not admissible evidence"
            .to_owned()
    })
}

// r[impl octet.effect_closure.evidence.visible]
// r[impl octet.effect_closure.kamacite_handoff]
fn write_closure_bundle(
    root: &ArtifactRoot,
    artifact_dir: &Path,
    command_bytes: &[u8],
    policy_bytes: &[u8],
    dependency_graph_bytes: &[u8],
    check_status_bytes: &[u8],
    function_address_bytes: &[u8],
    receipt: &EffectClosureReceipt,
    kamacite: Option<&KamaciteBundle>,
) -> Result<(), String> {
    let receipt_bytes = pretty_json_bytes(receipt)?;
    let status = status_from_receipt(
        receipt,
        kamacite.map(|bundle| &bundle.comparison),
        kamacite.and_then(|bundle| bundle.handoff.as_ref()),
    );
    let status_bytes = pretty_json_bytes(&status)?;
    let summary_bytes = render_summary(receipt, kamacite).into_bytes();
    root.write(&relative(COMMAND_FILE)?, command_bytes)?;
    root.write(&relative(effect_closure::RECEIPT_FILE)?, &receipt_bytes)?;
    root.write(&relative(STATUS_FILE)?, &status_bytes)?;
    root.write(&relative(SUMMARY_FILE)?, &summary_bytes)?;
    root.write(&relative(WORKSPACE_POLICY_ARTIFACT)?, policy_bytes)?;
    root.write(&relative(DEPENDENCY_GRAPH_ARTIFACT)?, dependency_graph_bytes)?;

    let mut artifacts = BTreeMap::from([
        (COMMAND_FILE.to_owned(), command_bytes.to_vec()),
        (effect_closure::RECEIPT_FILE.to_owned(), receipt_bytes),
        (STATUS_FILE.to_owned(), status_bytes),
        (SUMMARY_FILE.to_owned(), summary_bytes),
        (WORKSPACE_POLICY_ARTIFACT.to_owned(), policy_bytes.to_vec()),
        (DEPENDENCY_GRAPH_ARTIFACT.to_owned(), dependency_graph_bytes.to_vec()),
        (CHECK_STATUS_ARTIFACT.to_owned(), check_status_bytes.to_vec()),
        (CHECK_FUNCTION_ADDRESS_ARTIFACT.to_owned(), function_address_bytes.to_vec()),
    ]);
    if let Some(kamacite) = kamacite {
        root.write(&relative(KAMACITE_PROFILE_ARTIFACT)?, &kamacite.profile_bytes)?;
        root.write(&relative(KAMACITE_SUMMARY_ARTIFACT)?, &kamacite.projection_bytes)?;
        root.write(&relative(KAMACITE_COMPARISON_ARTIFACT)?, &kamacite.comparison_bytes)?;
        artifacts.insert(KAMACITE_PROFILE_ARTIFACT.to_owned(), kamacite.profile_bytes.clone());
        artifacts.insert(KAMACITE_SUMMARY_ARTIFACT.to_owned(), kamacite.projection_bytes.clone());
        artifacts.insert(KAMACITE_COMPARISON_ARTIFACT.to_owned(), kamacite.comparison_bytes.clone());
        if let (Some(handoff), Some(handoff_bytes)) = (&kamacite.handoff, &kamacite.handoff_bytes) {
            root.write(&relative(KAMACITE_HANDOFF_ARTIFACT)?, handoff_bytes)?;
            artifacts.insert(KAMACITE_HANDOFF_ARTIFACT.to_owned(), handoff_bytes.clone());
            let _ = handoff;
        }
    }
    let provenance = render_provenance(&artifacts, &receipt.inputs.profile_id)?;
    root.write(&relative(PROVENANCE_FILE)?, provenance.as_bytes())?;
    artifacts.insert(PROVENANCE_FILE.to_owned(), provenance.into_bytes());

    let verification = verify_bundle(artifact_dir)?;
    if verification.diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "new effect-closure artifact bundle failed self-verification:\n- {}",
            verification.diagnostics.join("\n- ")
        ))
    }
}

fn status_from_receipt(
    receipt: &EffectClosureReceipt,
    comparison: Option<&KamaciteEffectComparison>,
    handoff: Option<&KamaciteEffectHandoff>,
) -> AdmissionStatusArtifact {
    let kamacite_denied = comparison.is_some_and(|comparison| comparison.status != AdmissionStatus::Admitted);
    let status = if receipt.status == AdmissionStatus::Admitted && !kamacite_denied {
        AdmissionStatus::Admitted
    } else {
        AdmissionStatus::Denied
    };
    let kamacite = comparison.map(|comparison| KamaciteStatusArtifact {
        schema_version: kamacite_effect_closure::COMPARISON_SCHEMA_VERSION.to_owned(),
        status: match comparison.status {
            AdmissionStatus::Admitted => STATUS_ADMITTED,
            AdmissionStatus::Denied => STATUS_DENIED,
        }
        .to_owned(),
        comparison_identity: comparison.comparison_identity.clone(),
        handoff_identity: handoff.map(|handoff| handoff.handoff_identity.clone()),
        effect_summary_identity: comparison.effect_summary_identity.clone(),
    });
    let diagnostics = receipt
        .diagnostics
        .iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
        .chain(kamacite_diagnostics(kamacite.as_ref()))
        .collect();
    AdmissionStatusArtifact {
        schema_version: ADMISSION_STATUS_SCHEMA.to_owned(),
        status: match status {
            AdmissionStatus::Admitted => STATUS_ADMITTED,
            AdmissionStatus::Denied => STATUS_DENIED,
        }
        .to_owned(),
        exit_code: match status {
            AdmissionStatus::Admitted => EXIT_ADMITTED,
            AdmissionStatus::Denied => EXIT_DENIED,
        },
        phase: if kamacite.is_some() {
            PHASE_KAMACITE
        } else {
            PHASE_CLOSURE
        }
        .to_owned(),
        profile_id: receipt.inputs.profile_id.clone(),
        receipt_identity: Some(receipt.receipt_identity.clone()),
        kamacite,
        diagnostics,
        non_claims: receipt.non_claims.clone(),
    }
}

fn kamacite_diagnostics(status: Option<&KamaciteStatusArtifact>) -> impl Iterator<Item = String> + '_ {
    status
        .filter(|status| status.status == STATUS_DENIED)
        .into_iter()
        .map(|status| format!("kamacite_effect_comparison: comparison `{}` is denied", status.comparison_identity))
}

fn render_summary(receipt: &EffectClosureReceipt, kamacite: Option<&KamaciteBundle>) -> String {
    let mut summary = String::new();
    let status = status_from_receipt(
        receipt,
        kamacite.map(|bundle| &bundle.comparison),
        kamacite.and_then(|bundle| bundle.handoff.as_ref()),
    );
    let _ = writeln!(&mut summary, "Octet effect closure admission");
    let _ = writeln!(&mut summary, "Status: {}", status.status);
    let _ = writeln!(&mut summary, "Claim scope: {}", receipt.claim_scope);
    let _ = writeln!(&mut summary, "Profile: {}", receipt.inputs.profile_id);
    let _ = writeln!(&mut summary, "Profile hash: {}", receipt.inputs.profile_hash);
    let _ = writeln!(&mut summary, "Config hash: {}", receipt.inputs.config_hash);
    let _ = writeln!(&mut summary, "Receipt identity: {}", receipt.receipt_identity);
    let _ = writeln!(&mut summary, "Selected sources: {}", receipt.sources.len());
    let _ = writeln!(&mut summary, "Functions: {}", receipt.nodes.len());
    let _ = writeln!(&mut summary, "Reviewed providers: {}", receipt.providers.len());
    let _ = writeln!(&mut summary, "Diagnostics: {}", receipt.diagnostics.len());
    for diagnostic in &receipt.diagnostics {
        let call_site = diagnostic.call_site_id.as_deref().unwrap_or("-");
        let _ = writeln!(&mut summary, "- {} [{}] {}", diagnostic.code, call_site, diagnostic.message);
    }
    if let Some(kamacite) = kamacite {
        let comparison = &kamacite.comparison;
        let _ = writeln!(&mut summary, "Kamacite comparison: {}", status_string(comparison.status));
        let _ = writeln!(&mut summary, "Kamacite comparison identity: {}", comparison.comparison_identity);
        let _ = writeln!(&mut summary, "Kamacite mapping profile identity: {}", comparison.mapping_profile_identity);
        let _ = writeln!(
            &mut summary,
            "Kamacite effect summary: {}:{}",
            comparison.effect_summary_identity.domain, comparison.effect_summary_identity.hex
        );
        if let Some(handoff) = &kamacite.handoff {
            let _ = writeln!(&mut summary, "Kamacite handoff identity: {}", handoff.handoff_identity);
        }
        let _ = writeln!(&mut summary, "Kamacite comparison rows: {}", comparison.rows.len());
        let _ = writeln!(&mut summary, "Kamacite diagnostics: {}", comparison.diagnostics.len());
        for diagnostic in &comparison.diagnostics {
            let _ =
                writeln!(&mut summary, "- {} [{}] {}", diagnostic.code, diagnostic.class.as_str(), diagnostic.message);
        }
    }
    let _ = writeln!(&mut summary, "Non-claims:");
    for non_claim in &receipt.non_claims {
        let _ = writeln!(&mut summary, "- {non_claim}");
    }
    summary
}

fn status_string(status: AdmissionStatus) -> &'static str {
    match status {
        AdmissionStatus::Admitted => STATUS_ADMITTED,
        AdmissionStatus::Denied => STATUS_DENIED,
    }
}

fn render_admission_output(
    output_format: OutputFormat,
    artifact_dir: &Path,
    receipt: &EffectClosureReceipt,
    kamacite: Option<&KamaciteBundle>,
) -> Result<(), String> {
    match output_format {
        OutputFormat::Human => eprint!("{}", render_summary(receipt, kamacite)),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&status_from_receipt(
                receipt,
                kamacite.map(|bundle| &bundle.comparison),
                kamacite.and_then(|bundle| bundle.handoff.as_ref()),
            ))
            .map_err(|error| format!("serialize admission status output: {error}"))?
        ),
        OutputFormat::Sarif => unreachable!("SARIF is rejected by admission argument parsing"),
    }
    eprintln!("Admission artifacts: {}", artifact_dir.display());
    Ok(())
}

fn render_provenance(artifacts: &BTreeMap<String, Vec<u8>>, profile_id: &str) -> Result<String, String> {
    let checked_scope = format!("{CHECKED_SCOPE_PREFIX}{profile_id}");
    let mut names = REQUIRED_PROVENANCE_ARTIFACTS.to_vec();
    for artifact in [
        KAMACITE_PROFILE_ARTIFACT,
        KAMACITE_SUMMARY_ARTIFACT,
        KAMACITE_COMPARISON_ARTIFACT,
        KAMACITE_HANDOFF_ARTIFACT,
    ] {
        if artifacts.contains_key(artifact) {
            names.push(artifact);
        }
    }
    let records = names
        .iter()
        .map(|artifact| {
            let bytes = artifacts
                .get(*artifact)
                .ok_or_else(|| format!("missing artifact `{artifact}` while rendering admission provenance"))?;
            let parent_artifacts = match *artifact {
                effect_closure::RECEIPT_FILE => vec![
                    COMMAND_FILE.to_owned(),
                    WORKSPACE_POLICY_ARTIFACT.to_owned(),
                    DEPENDENCY_GRAPH_ARTIFACT.to_owned(),
                    CHECK_STATUS_ARTIFACT.to_owned(),
                    CHECK_FUNCTION_ADDRESS_ARTIFACT.to_owned(),
                ],
                STATUS_FILE | SUMMARY_FILE => vec![effect_closure::RECEIPT_FILE.to_owned()],
                KAMACITE_COMPARISON_ARTIFACT => vec![
                    effect_closure::RECEIPT_FILE.to_owned(),
                    KAMACITE_PROFILE_ARTIFACT.to_owned(),
                    KAMACITE_SUMMARY_ARTIFACT.to_owned(),
                ],
                KAMACITE_HANDOFF_ARTIFACT => vec![
                    KAMACITE_COMPARISON_ARTIFACT.to_owned(),
                    effect_closure::RECEIPT_FILE.to_owned(),
                    KAMACITE_PROFILE_ARTIFACT.to_owned(),
                    KAMACITE_SUMMARY_ARTIFACT.to_owned(),
                ],
                _ => Vec::new(),
            };
            Ok(ProvenanceRecord {
                schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
                source_id: PROVENANCE_SOURCE_ID.to_owned(),
                checker_id: CHECKER_ID.to_owned(),
                evidence_tier: EvidenceTier::PolicyCheck,
                artifact_locator: (*artifact).to_owned(),
                blake3: crate::evidence::artifact_blake3_hex(bytes),
                checked_scope: checked_scope.clone(),
                verifier_identity: None,
                proof_artifact_identity: None,
                human_action: None,
                parent_artifacts,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    let mut output = String::new();
    for record in records {
        output.push_str(
            &serde_json::to_string(&record).map_err(|error| format!("serialize admission provenance: {error}"))?,
        );
        output.push('\n');
    }
    Ok(output)
}

// r[impl octet.effect_closure.evidence.tamper]
fn verify_bundle(artifact_dir: &Path) -> Result<AdmissionVerificationReport, String> {
    verify_bundle_inner(artifact_dir, true)
}

/// Run the unsigned integrity verification first, then evaluate consumer-owned detached
/// signatures and policy over the exact candidate bytes. The unsigned mode and its non-claim are
/// unchanged when no authentication inputs are supplied.
fn verify_bundle_authorized(
    artifact_dir: &Path,
    policy_path: &Path,
    evidence_path: &Path,
) -> Result<AdmissionVerificationReport, String> {
    verify_bundle_authorized_inner(artifact_dir, true, policy_path, evidence_path)
}

fn verify_bundle_authorized_inner(
    artifact_dir: &Path,
    verify_strict_profile_parent: bool,
    policy_path: &Path,
    evidence_path: &Path,
) -> Result<AdmissionVerificationReport, String> {
    reject_candidate_supplied_input(artifact_dir, policy_path, "admission auth policy")?;
    reject_candidate_supplied_input(artifact_dir, evidence_path, "admission auth evidence")?;
    let policy = load_admission_auth_policy(policy_path)?;
    let evidence = load_admission_auth_evidence(evidence_path)?;
    let mut report = verify_bundle_inner(artifact_dir, verify_strict_profile_parent)?;
    let root = ArtifactRoot::open_existing(artifact_dir)?;
    let files = root
        .read_files_recursive()?
        .into_iter()
        .map(|file| (file.locator, file.contents))
        .collect::<BTreeMap<_, _>>();
    let receipt = files
        .get(effect_closure::RECEIPT_FILE)
        .and_then(|bytes| serde_json::from_slice::<EffectClosureReceipt>(bytes).ok());
    let auth = evaluate_admission_auth(&report, &files, receipt.as_ref(), &policy, &evidence)?;
    for issue in &auth.decision.issues {
        report.diagnostics.push(format!("admission auth {}: {}", issue.code, issue.field));
    }
    report.diagnostics.sort();
    report.diagnostics.dedup();
    if !auth.passed {
        report.status = STATUS_INVALID.to_owned();
    }
    report.auth = Some(auth);
    report.non_claims = [NON_CLAIM_INTEGRITY, NON_CLAIM_AUTH]
        .into_iter()
        .map(str::to_owned)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(report)
}

/// Trust roots and evidence must be consumer-owned inputs outside the candidate bundle. A bundle
/// can still contain arbitrary files, but none of them can become a trust root or observation.
fn reject_candidate_supplied_input(artifact_dir: &Path, input_path: &Path, label: &str) -> Result<(), String> {
    let artifact_dir = fs::canonicalize(artifact_dir)
        .map_err(|error| format!("open artifact dir {}: {error}", artifact_dir.display()))?;
    let input_path =
        fs::canonicalize(input_path).map_err(|error| format!("read {label} {}: {error}", input_path.display()))?;
    if input_path.starts_with(&artifact_dir) {
        return Err(format!(
            "{label} must be consumer-owned and outside the candidate artifact directory: {} is inside {}",
            input_path.display(),
            artifact_dir.display()
        ));
    }
    Ok(())
}

fn load_admission_auth_policy(path: &Path) -> Result<AdmissionAuthPolicyInput, String> {
    let bytes = fs::read(path).map_err(|error| format!("read admission auth policy {}: {error}", path.display()))?;
    let policy = serde_json::from_slice::<AdmissionAuthPolicyInput>(&bytes)
        .map_err(|error| format!("parse admission auth policy {}: {error}", path.display()))?;
    validate_admission_auth_policy(&policy)?;
    Ok(policy)
}

fn load_admission_auth_evidence(path: &Path) -> Result<AdmissionAuthEvidenceInput, String> {
    let bytes = fs::read(path).map_err(|error| format!("read admission auth evidence {}: {error}", path.display()))?;
    serde_json::from_slice::<AdmissionAuthEvidenceInput>(&bytes)
        .map_err(|error| format!("parse admission auth evidence {}: {error}", path.display()))
}

fn auth_issue(code: &str, field: &str) -> ArtifactAuthIssueV1 {
    ArtifactAuthIssueV1 {
        code: code.to_owned(),
        field: field.to_owned(),
    }
}

fn validate_auth_text(value: &str, field: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > artifact_auth_core::MAX_TEXT_BYTES || value.chars().any(char::is_control) {
        return Err(format!("invalid admission auth policy text field `{field}`"));
    }
    Ok(())
}

fn validate_auth_ref(value: &AuthArtifactRef, field: &str) -> Result<(), String> {
    validate_auth_text(&value.profile, &format!("{field}.profile"))?;
    if !matches!(value.algorithm.as_str(), "blake3" | "sha256") {
        return Err(format!("admission auth policy `{field}.algorithm` must be `blake3` or `sha256`"));
    }
    if value.digest_hex.len() != AUTH_HEX_CHARS || decode_lower_hex(&value.digest_hex, AUTH_HEX_CHARS / 2).is_none() {
        return Err(format!("admission auth policy `{field}.digest_hex` must be 64 lowercase hex characters"));
    }
    Ok(())
}

fn validate_admission_auth_policy(policy: &AdmissionAuthPolicyInput) -> Result<(), String> {
    if policy.schema_version != ADMISSION_AUTH_POLICY_SCHEMA {
        return Err(format!(
            "unsupported admission auth policy schema `{}`; expected `{ADMISSION_AUTH_POLICY_SCHEMA}`",
            policy.schema_version
        ));
    }
    for (value, field) in [
        (&policy.policy_id, "policy_id"),
        (&policy.profile_id, "profile_id"),
        (&policy.domain, "domain"),
        (&policy.purpose, "purpose"),
        (&policy.scope_id, "scope_id"),
    ] {
        validate_auth_text(value, field)?;
    }
    validate_auth_ref(&policy.verifier_context, "verifier_context")?;
    validate_auth_ref(&policy.expected_subject, "expected_subject")?;
    if policy.expected_subject.profile != ADMISSION_AUTH_BUNDLE_PROFILE
        || policy.expected_subject.algorithm != ALGORITHM_BLAKE3
    {
        return Err(format!(
            "admission auth policy `expected_subject` must use profile `{ADMISSION_AUTH_BUNDLE_PROFILE}` and algorithm `blake3`"
        ));
    }
    if let Some(receipt_identity) = &policy.expected_receipt_identity
        && normalize_auth_digest(receipt_identity, "expected_receipt_identity").is_err()
    {
        return Err("admission auth policy `expected_receipt_identity` must be a blake3 identity".to_owned());
    }
    if policy.threshold == 0 {
        return Err("admission auth policy `threshold` must be at least 1".to_owned());
    }
    if policy.trusted_keys.is_empty() || policy.trusted_keys.len() > artifact_auth_core::MAX_TRUSTED_KEYS {
        return Err("admission auth policy must supply between 1 and 64 trusted keys".to_owned());
    }
    if usize::from(policy.threshold) > policy.trusted_keys.len() {
        return Err("admission auth policy `threshold` exceeds the trusted-key count".to_owned());
    }
    let mut key_identities = BTreeSet::new();
    let mut key_labels = BTreeSet::new();
    for (index, key) in policy.trusted_keys.iter().enumerate() {
        let field = format!("trusted_keys[{index}]");
        validate_auth_text(&key.producer_id, &format!("{field}.producer_id"))?;
        validate_auth_text(&key.key_id, &format!("{field}.key_id"))?;
        if !key_labels.insert((key.producer_id.as_str(), key.key_id.as_str())) {
            return Err(format!("admission auth policy has duplicate trusted key label `{field}`"));
        }
        if decode_lower_hex(&key.public_key_hex, AUTH_PUBLIC_KEY_BYTES).is_none() {
            return Err(format!("admission auth policy `{field}.public_key_hex` must be 64 lowercase hex characters"));
        }
        if key.generation == 0 {
            return Err(format!("admission auth policy `{field}.generation` must be at least 1"));
        }
        validate_auth_purposes(&key.allowed_purposes, &format!("{field}.allowed_purposes"))?;
        validate_auth_ref(&key.currentness_ref, &format!("{field}.currentness_ref"))?;
        let identity = decode_lower_hex(&key.public_key_hex, AUTH_PUBLIC_KEY_BYTES)
            .map(|bytes| public_key_identity(&bytes))
            .ok_or_else(|| format!("admission auth policy `{field}.public_key_hex` is invalid"))?;
        if !key_identities.insert(identity.digest_hex.clone()) {
            return Err(format!("admission auth policy has duplicate trusted key `{field}`"));
        }
    }
    if policy.currentness_rules.accepted.is_empty() {
        return Err("admission auth policy `currentness_rules.accepted` must not be empty".to_owned());
    }
    validate_auth_text(&policy.currentness_rules.observation_profile, "currentness_rules.observation_profile")?;
    if policy.currentness_rules.maximum_observation_age_seconds.is_some()
        && policy.currentness_rules.evaluated_at_unix.is_none()
    {
        return Err(
            "admission auth policy `currentness_rules.evaluated_at_unix` is required when a maximum observation age is set"
                .to_owned(),
        );
    }
    if policy.authorized_producers.is_empty() {
        return Err("admission auth policy `authorized_producers` must not be empty".to_owned());
    }
    let mut producers = BTreeSet::new();
    for (index, producer) in policy.authorized_producers.iter().enumerate() {
        let field = format!("authorized_producers[{index}]");
        validate_auth_text(&producer.producer_id, &format!("{field}.producer_id"))?;
        if !producers.insert(producer.producer_id.as_str()) {
            return Err(format!("admission auth policy has duplicate authorized producer `{field}`"));
        }
        if producer.key_ids.is_empty() {
            return Err(format!("admission auth policy `{field}.key_ids` must not be empty"));
        }
        validate_auth_text_list(&producer.key_ids, &format!("{field}.key_ids"))?;
        validate_auth_purposes(&producer.purposes, &format!("{field}.purposes"))?;
        validate_auth_text_list(&producer.scope_ids, &format!("{field}.scope_ids"))?;
    }
    if policy.required_provenance.is_empty() {
        return Err("admission auth policy `required_provenance` must not be empty".to_owned());
    }
    validate_auth_text_list(&policy.required_provenance, "required_provenance")?;
    Ok(())
}

fn validate_auth_text_list(values: &[String], field: &str) -> Result<(), String> {
    if values.is_empty() {
        return Err(format!("admission auth policy `{field}` must not be empty"));
    }
    let mut seen = BTreeSet::new();
    for (index, value) in values.iter().enumerate() {
        validate_auth_text(value, &format!("{field}[{index}]"))?;
        if !seen.insert(value.as_str()) {
            return Err(format!("admission auth policy `{field}` has a duplicate value"));
        }
    }
    Ok(())
}

fn validate_auth_purposes(values: &[String], field: &str) -> Result<(), String> {
    validate_auth_text_list(values, field)
}

fn decode_lower_hex(value: &str, expected_bytes: usize) -> Option<Vec<u8>> {
    if value.len() != expected_bytes * 2 {
        return None;
    }
    let mut output = Vec::with_capacity(expected_bytes);
    let bytes = value.as_bytes();
    for pair in bytes.chunks_exact(2) {
        let high = lower_hex_digit(pair[0])?;
        let low = lower_hex_digit(pair[1])?;
        output.push((high << 4) | low);
    }
    Some(output)
}

fn lower_hex_digit(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        _ => None,
    }
}

#[cfg(test)]
fn encode_lower_hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(output, "{byte:02x}");
    }
    output
}

fn normalize_auth_digest(value: &str, field: &str) -> Result<String, String> {
    let digest = value.strip_prefix(AUTH_BLAKE3_PREFIX).unwrap_or(value);
    if digest.len() != AUTH_HEX_CHARS || decode_lower_hex(digest, AUTH_HEX_CHARS / 2).is_none() {
        return Err(format!("admission auth `{field}` must be a 64-character lowercase blake3 identity"));
    }
    Ok(digest.to_owned())
}

fn blake3_auth_bytes(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

fn push_auth_text(output: &mut Vec<u8>, value: &str) {
    let length = u32::try_from(value.len()).unwrap_or(u32::MAX);
    output.extend_from_slice(&length.to_be_bytes());
    output.extend_from_slice(value.as_bytes());
}

fn push_auth_count(output: &mut Vec<u8>, count: usize) {
    let length = u32::try_from(count).unwrap_or(u32::MAX);
    output.extend_from_slice(&length.to_be_bytes());
}

/// Canonical candidate bundle identity over every retained file and the receipt identity.
fn admission_bundle_identity(
    files: &BTreeMap<String, Vec<u8>>,
    receipt: Option<&EffectClosureReceipt>,
) -> AuthArtifactRef {
    let mut bytes = Vec::new();
    push_auth_text(&mut bytes, ADMISSION_AUTH_BUNDLE_DOMAIN);
    push_auth_text(&mut bytes, receipt.map(|receipt| receipt.receipt_identity.as_str()).unwrap_or_default());
    push_auth_count(&mut bytes, files.len());
    for (locator, contents) in files {
        push_auth_text(&mut bytes, locator);
        push_auth_text(&mut bytes, &blake3_auth_bytes(contents));
    }
    AuthArtifactRef {
        profile: ADMISSION_AUTH_BUNDLE_PROFILE.to_owned(),
        algorithm: ALGORITHM_BLAKE3.to_owned(),
        digest_hex: blake3_auth_bytes(&bytes),
    }
}

fn admission_provenance_refs(receipt: &EffectClosureReceipt) -> Result<Vec<AuthArtifactRef>, String> {
    let mut parents = Vec::new();
    for (profile, value, field) in [
        (ADMISSION_AUTH_RECEIPT_PROFILE, receipt.receipt_identity.as_str(), "receipt_identity"),
        (ADMISSION_AUTH_COMMAND_PROFILE, receipt.inputs.command_blake3.as_str(), "command_blake3"),
        (
            ADMISSION_AUTH_CHECK_STATUS_PROFILE,
            receipt.inputs.check_status_blake3.as_str(),
            "check_status_blake3",
        ),
        (
            ADMISSION_AUTH_FUNCTION_ADDRESS_PROFILE,
            receipt.inputs.function_address_artifact_blake3.as_str(),
            "function_address_artifact_blake3",
        ),
        (
            ADMISSION_AUTH_WORKSPACE_POLICY_PROFILE,
            receipt.inputs.workspace_policy_blake3.as_str(),
            "workspace_policy_blake3",
        ),
        (ADMISSION_AUTH_CARGO_GRAPH_PROFILE, receipt.inputs.cargo_graph_blake3.as_str(), "cargo_graph_blake3"),
        (ADMISSION_AUTH_CARGO_LOCK_PROFILE, receipt.inputs.cargo_lock_blake3.as_str(), "cargo_lock_blake3"),
    ] {
        parents.push(AuthArtifactRef {
            profile: profile.to_owned(),
            algorithm: ALGORITHM_BLAKE3.to_owned(),
            digest_hex: normalize_auth_digest(value, field)?,
        });
    }
    parents.sort();
    Ok(parents)
}

struct AdmissionAuthScope {
    scope: AuthenticationScope,
    subject: AuthArtifactRef,
    issues: Vec<ArtifactAuthIssueV1>,
}

fn build_admission_auth_scope(
    policy: &AdmissionAuthPolicyInput,
    files: &BTreeMap<String, Vec<u8>>,
    receipt: Option<&EffectClosureReceipt>,
) -> AdmissionAuthScope {
    let subject = admission_bundle_identity(files, receipt);
    let mut issues = Vec::new();
    if subject != policy.expected_subject {
        issues.push(auth_issue("admission_auth.subject_mismatch", "policy.expected_subject"));
    }
    let actual_receipt_identity = receipt.map(|receipt| receipt.receipt_identity.clone()).unwrap_or_default();
    if receipt.is_none() {
        issues.push(auth_issue("admission_auth.receipt_missing", "bundle.effect-closure.json"));
    }
    if let Some(expected) = &policy.expected_receipt_identity
        && normalize_auth_digest(expected, "expected_receipt_identity").ok().as_deref()
            != normalize_auth_digest(&actual_receipt_identity, "receipt_identity").ok().as_deref()
    {
        issues.push(auth_issue("admission_auth.receipt_identity_mismatch", "policy.expected_receipt_identity"));
    }
    let mut parents = vec![
        AuthArtifactRef {
            profile: ADMISSION_AUTH_POLICY_IDENTITY_PROFILE.to_owned(),
            algorithm: ALGORITHM_BLAKE3.to_owned(),
            digest_hex: blake3_auth_bytes(policy.policy_id.as_bytes()),
        },
        AuthArtifactRef {
            profile: ADMISSION_AUTH_SCOPE_IDENTITY_PROFILE.to_owned(),
            algorithm: ALGORITHM_BLAKE3.to_owned(),
            digest_hex: blake3_auth_bytes(policy.scope_id.as_bytes()),
        },
    ];
    if let Some(receipt) = receipt {
        match admission_provenance_refs(receipt) {
            Ok(refs) => {
                let present = refs.iter().map(|parent| parent.profile.as_str()).collect::<BTreeSet<_>>();
                for (index, required) in policy.required_provenance.iter().enumerate() {
                    if !present.contains(required.as_str()) {
                        issues.push(auth_issue(
                            "admission_auth.provenance_missing",
                            &format!("policy.required_provenance[{index}]"),
                        ));
                    }
                }
                parents.extend(refs);
            }
            Err(_) => {
                issues.push(auth_issue("admission_auth.provenance_malformed", "bundle.effect-closure.json"));
            }
        }
    }
    parents.sort();
    AdmissionAuthScope {
        scope: AuthenticationScope {
            domain: policy.domain.clone(),
            purpose: policy.purpose.clone(),
            profile_id: policy.profile_id.clone(),
            subject: subject.clone(),
            parents,
            verifier_context: policy.verifier_context.clone(),
        },
        subject,
        issues,
    }
}

fn build_authentication_policy(
    policy: &AdmissionAuthPolicyInput,
) -> Result<(AuthenticationPolicy, Vec<TrustedKeyMaterial>), String> {
    let mut materials = Vec::with_capacity(policy.trusted_keys.len());
    for (index, key) in policy.trusted_keys.iter().enumerate() {
        let bytes = decode_lower_hex(&key.public_key_hex, AUTH_PUBLIC_KEY_BYTES)
            .ok_or_else(|| format!("admission auth policy `trusted_keys[{index}].public_key_hex` is invalid"))?;
        let public_key_bytes: [u8; AUTH_PUBLIC_KEY_BYTES] = bytes
            .try_into()
            .map_err(|_| format!("admission auth policy `trusted_keys[{index}].public_key_hex` is invalid"))?;
        let key_identity = public_key_identity(&public_key_bytes);
        materials.push(TrustedKeyMaterial {
            observation: TrustedKeyObservation {
                producer_id: key.producer_id.clone(),
                key_id: key.key_id.clone(),
                key_identity,
                allowed_purposes: key.allowed_purposes.clone(),
                generation: key.generation,
                currentness: key.currentness,
                currentness_ref: key.currentness_ref.clone(),
            },
            public_key_bytes,
            observed_at_unix: key.observed_at_unix,
        });
    }
    let trusted_keys = materials.iter().map(|material| material.observation.clone()).collect();
    Ok((
        AuthenticationPolicy {
            schema: POLICY_SCHEMA_V1.to_owned(),
            profile_id: policy.profile_id.clone(),
            threshold: policy.threshold,
            trusted_keys,
        },
        materials,
    ))
}

fn producer_authorized(policy: &AdmissionAuthPolicyInput, statement: &ArtifactStatement) -> bool {
    policy.authorized_producers.iter().any(|producer| {
        producer.producer_id == statement.producer_id
            && producer.key_ids.contains(&statement.key_id)
            && producer.purposes.contains(&policy.purpose)
            && producer.scope_ids.contains(&policy.scope_id)
    })
}

fn currentness_observation_issue(
    policy: &AdmissionAuthPolicyInput,
    material: &TrustedKeyMaterial,
    field: &str,
) -> Option<ArtifactAuthIssueV1> {
    let rules = &policy.currentness_rules;
    let key = &material.observation;
    if !rules.accepted.contains(&key.currentness) {
        return Some(auth_issue("admission_auth.currentness_unacceptable", &format!("{field}.currentness")));
    }
    if key.currentness_ref.profile != rules.observation_profile {
        return Some(auth_issue(
            "admission_auth.currentness_unacceptable",
            &format!("{field}.currentness_ref.profile"),
        ));
    }
    if let Some(maximum_age) = rules.maximum_observation_age_seconds {
        let Some(evaluated_at) = rules.evaluated_at_unix else {
            return Some(auth_issue(
                "admission_auth.currentness_unacceptable",
                "policy.currentness_rules.evaluated_at_unix",
            ));
        };
        let Some(observed_at) = material.observed_at_unix else {
            return Some(auth_issue("admission_auth.currentness_unacceptable", &format!("{field}.observed_at_unix")));
        };
        let age = evaluated_at.saturating_sub(observed_at);
        if observed_at > evaluated_at || age > maximum_age {
            return Some(auth_issue("admission_auth.currentness_unacceptable", &format!("{field}.observed_at_unix")));
        }
    } else if let (Some(evaluated_at), Some(observed_at)) = (rules.evaluated_at_unix, material.observed_at_unix)
        && observed_at > evaluated_at
    {
        return Some(auth_issue("admission_auth.currentness_unacceptable", &format!("{field}.observed_at_unix")));
    }
    None
}

fn auth_code_present(issues: &[ArtifactAuthIssueV1], codes: &[&str]) -> bool {
    issues.iter().any(|issue| codes.contains(&issue.code.as_str()))
}

fn auth_decision_bytes(decision: &ArtifactAuthDecisionV1) -> Vec<u8> {
    let mut bytes = Vec::new();
    push_auth_text(&mut bytes, ADMISSION_AUTH_DECISION_DOMAIN);
    push_auth_text(&mut bytes, &decision.schema_version);
    push_auth_text(&mut bytes, &decision.policy_id);
    push_auth_text(&mut bytes, &decision.policy_profile_id);
    push_auth_text(&mut bytes, &decision.subject_blake3);
    push_auth_text(&mut bytes, &decision.receipt_identity);
    push_auth_text(&mut bytes, &decision.policy_blake3);
    push_auth_text(&mut bytes, &decision.scope_blake3);
    push_auth_text(&mut bytes, &decision.checker_blake3);
    push_auth_count(&mut bytes, decision.verified_key_blake3.len());
    for key in &decision.verified_key_blake3 {
        push_auth_text(&mut bytes, key);
    }
    push_auth_text(&mut bytes, if decision.passed { "passed" } else { "denied" });
    push_auth_count(&mut bytes, decision.issues.len());
    for issue in &decision.issues {
        push_auth_text(&mut bytes, &issue.code);
        push_auth_text(&mut bytes, &issue.field);
    }
    push_auth_count(&mut bytes, decision.non_claims.len());
    for claim in &decision.non_claims {
        push_auth_text(&mut bytes, claim);
    }
    bytes
}

fn auth_decision_receipt_blake3(decision: &ArtifactAuthDecisionV1) -> String {
    blake3_auth_bytes(&auth_decision_bytes(decision))
}

fn evaluate_admission_auth(
    report: &AdmissionVerificationReport,
    files: &BTreeMap<String, Vec<u8>>,
    receipt: Option<&EffectClosureReceipt>,
    policy: &AdmissionAuthPolicyInput,
    evidence: &AdmissionAuthEvidenceInput,
) -> Result<AdmissionAuthReport, String> {
    let (core_policy, materials) = build_authentication_policy(policy)?;
    let scope_result = build_admission_auth_scope(policy, files, receipt);
    let mut issues = scope_result.issues;
    if !report.diagnostics.is_empty() {
        issues.push(auth_issue("admission_auth.candidate_integrity_invalid", "bundle"));
    }
    if evidence.schema_version != ADMISSION_AUTH_EVIDENCE_SCHEMA {
        issues.push(auth_issue("admission_auth.evidence_schema_unsupported", "evidence.schema_version"));
    }
    let mut matched = Vec::new();
    let mut signatures = Vec::new();
    let mut seen_keys = BTreeSet::new();
    if evidence.schema_version == ADMISSION_AUTH_EVIDENCE_SCHEMA {
        for (index, item) in evidence.evidence.iter().enumerate() {
            let field = format!("evidence[{index}]");
            if !producer_authorized(policy, &item.statement) {
                issues.push(auth_issue(
                    "admission_auth.producer_unauthorized",
                    &format!("{field}.statement.producer_id"),
                ));
            }
            let Some((material_index, material)) = materials.iter().enumerate().find(|(_, material)| {
                material.observation.producer_id == item.statement.producer_id
                    && material.observation.key_id == item.statement.key_id
            }) else {
                issues.push(auth_issue(
                    "admission_auth.evidence_key_untrusted",
                    &format!("{field}.statement.key_identity"),
                ));
                continue;
            };
            let key_marker = (item.statement.producer_id.clone(), item.statement.key_id.clone());
            if !seen_keys.insert(key_marker) {
                issues.push(auth_issue("admission_auth.evidence_key_duplicate", &format!("{field}.statement.key_id")));
                continue;
            }
            if let Some(issue) = currentness_observation_issue(policy, material, &field) {
                issues.push(issue);
            }
            let Some(signature) = decode_lower_hex(&item.signature_hex, AUTH_SIGNATURE_BYTES) else {
                issues.push(auth_issue("admission_auth.signature_malformed", &format!("{field}.signature_hex")));
                continue;
            };
            matched.push((&item.statement, item.generation, material_index));
            signatures.push(signature);
        }
    }
    let ed25519_evidence = matched
        .iter()
        .zip(signatures.iter())
        .map(|((statement, generation, material_index), signature)| Ed25519Evidence {
            statement,
            generation: *generation,
            public_key_bytes: &materials[*material_index].public_key_bytes,
            signature_bytes: signature,
        })
        .collect::<Vec<_>>();
    let core_decision = verify_and_evaluate(&core_policy, &scope_result.scope, &ed25519_evidence);
    issues.extend(core_decision.issues.iter().map(|issue| ArtifactAuthIssueV1 {
        code: issue.code.clone(),
        field: issue.field.clone(),
    }));
    issues.sort();
    issues.dedup();
    let integrity = if report.diagnostics.is_empty() {
        STATUS_VALID
    } else {
        STATUS_INVALID
    };
    let signature_authenticity = if auth_code_present(&issues, AUTH_SIGNATURE_UNSUPPORTED_CODES) {
        "unsupported"
    } else if auth_code_present(&issues, AUTH_SIGNATURE_UNVERIFIED_CODES) || ed25519_evidence.is_empty() {
        "unverified"
    } else {
        "verified"
    };
    let authorization = if auth_code_present(&issues, AUTH_AUTHORIZATION_CODES) {
        "unsatisfied"
    } else {
        "satisfied"
    };
    let trust_currentness = if auth_code_present(&issues, AUTH_TRUST_CODES) {
        "unsatisfied"
    } else {
        "satisfied"
    };
    let checker = if core_decision.passed { "passed" } else { "denied" };
    let mut non_claims = core_decision.non_claims.clone();
    non_claims.push(NON_CLAIM_AUTH.to_owned());
    non_claims.sort();
    non_claims.dedup();
    let decision = ArtifactAuthDecisionV1 {
        schema_version: ADMISSION_AUTH_DECISION_SCHEMA.to_owned(),
        policy_id: policy.policy_id.clone(),
        policy_profile_id: policy.profile_id.clone(),
        subject_blake3: scope_result.subject.digest_hex.clone(),
        receipt_identity: receipt.map(|receipt| receipt.receipt_identity.clone()).unwrap_or_default(),
        policy_blake3: core_decision.policy_blake3,
        scope_blake3: core_decision.scope_blake3,
        checker_blake3: core_decision.decision_blake3,
        verified_key_blake3: core_decision.verified_key_blake3,
        passed: core_decision.passed && issues.is_empty() && integrity == STATUS_VALID,
        issues: issues.clone(),
        non_claims: non_claims.clone(),
    };
    let decision_receipt_blake3 = auth_decision_receipt_blake3(&decision);
    let diagnostics = issues.iter().map(|issue| format!("{}: {}", issue.code, issue.field)).collect::<Vec<_>>();
    Ok(AdmissionAuthReport {
        schema_version: ADMISSION_AUTH_REPORT_SCHEMA.to_owned(),
        policy_id: policy.policy_id.clone(),
        integrity: integrity.to_owned(),
        signature_authenticity: signature_authenticity.to_owned(),
        authorization: authorization.to_owned(),
        trust_currentness: trust_currentness.to_owned(),
        checker: checker.to_owned(),
        passed: decision.passed,
        decision_receipt_blake3,
        decision,
        diagnostics,
        non_claims,
    })
}

// Bundle verification checks every bound artifact against one parsed receipt; splitting it would
// scatter the cross-artifact identity checks.
#[allow(clippy::too_many_lines)]
fn verify_bundle_inner(
    artifact_dir: &Path,
    verify_strict_profile_parent: bool,
) -> Result<AdmissionVerificationReport, String> {
    let root = ArtifactRoot::open_existing(artifact_dir)?;
    let files = root
        .read_files_recursive()?
        .into_iter()
        .map(|file| (file.locator, file.contents))
        .collect::<BTreeMap<_, _>>();
    let mut diagnostics = Vec::new();
    if verify_strict_profile_parent {
        match verify_check_bundle(&artifact_dir.join(CHECK_DIR)) {
            Ok(()) => {}
            Err(error) => diagnostics.push(error),
        }
    }
    let unsupported_schema = files
        .get(effect_closure::RECEIPT_FILE)
        .and_then(|bytes| effect_closure::unsupported_receipt_schema(bytes));
    let receipt = match unsupported_schema {
        Some(diagnostic) => {
            diagnostics.push(diagnostic);
            None
        }
        None => parse_required::<EffectClosureReceipt>(&files, effect_closure::RECEIPT_FILE, &mut diagnostics),
    };
    let status = parse_required::<AdmissionStatusArtifact>(&files, STATUS_FILE, &mut diagnostics);
    let policy = parse_required::<WorkspacePolicy>(&files, WORKSPACE_POLICY_ARTIFACT, &mut diagnostics);
    let graph = parse_required::<DependencyGraphArtifact>(&files, DEPENDENCY_GRAPH_ARTIFACT, &mut diagnostics);
    let check_status = parse_required::<CheckStatus>(&files, CHECK_STATUS_ARTIFACT, &mut diagnostics);
    let function_report =
        parse_required::<FunctionAddressReport>(&files, CHECK_FUNCTION_ADDRESS_ARTIFACT, &mut diagnostics);
    let kamacite = verify_kamacite_bundle(&files, receipt.as_ref(), status.as_ref(), &mut diagnostics);

    if let Some(receipt) = &receipt {
        diagnostics.extend(effect_closure::verify_receipt_identity(receipt));
        verify_bound_artifact_hash(receipt.inputs.command_blake3.as_str(), &files, COMMAND_FILE, &mut diagnostics);
        verify_bound_artifact_hash(
            receipt.inputs.check_status_blake3.as_str(),
            &files,
            CHECK_STATUS_ARTIFACT,
            &mut diagnostics,
        );
        verify_bound_artifact_hash(
            receipt.inputs.function_address_artifact_blake3.as_str(),
            &files,
            CHECK_FUNCTION_ADDRESS_ARTIFACT,
            &mut diagnostics,
        );
        verify_bound_artifact_hash(
            receipt.inputs.workspace_policy_blake3.as_str(),
            &files,
            WORKSPACE_POLICY_ARTIFACT,
            &mut diagnostics,
        );
        verify_bound_artifact_hash(
            receipt.inputs.cargo_graph_blake3.as_str(),
            &files,
            DEPENDENCY_GRAPH_ARTIFACT,
            &mut diagnostics,
        );
        if let Some(graph) = &graph {
            if graph.schema_version != DEPENDENCY_GRAPH_SCHEMA
                || graph.cargo_lock_blake3 != receipt.inputs.cargo_lock_blake3
                || graph.target != receipt.inputs.target
                || graph.toolchain != receipt.inputs.toolchain
            {
                diagnostics.push("dependency-graph identity does not match the effect-closure receipt".to_owned());
            }
            let providers = graph.providers.iter().collect::<BTreeSet<_>>();
            if receipt.providers.iter().any(|provider| !providers.contains(provider)) {
                diagnostics.push("receipt uses a provider absent from dependency-graph.json".to_owned());
            }
        }
        if let Some(check_status) = &check_status {
            let parent_matches = check_status.exit_code == EXIT_ADMITTED
                && check_status.error_findings == 0
                && check_status.metadata.profile_name == receipt.inputs.profile_id
                && check_status.metadata.profile_hash == receipt.inputs.profile_hash
                && check_status.metadata.config_hash == receipt.inputs.config_hash
                && check_status.metadata.toolchain == receipt.inputs.toolchain;
            if !parent_matches {
                diagnostics.push("direct check status identity does not match the effect-closure receipt".to_owned());
            }
        }
        diagnostics.extend(verify_bound_closure_inputs(receipt, function_report.as_ref(), policy.as_ref()));
        if let Some(status) = &status {
            let expected = status_from_receipt(
                receipt,
                kamacite.as_ref().map(|bundle| &bundle.comparison),
                kamacite.as_ref().and_then(|bundle| bundle.handoff.as_ref()),
            );
            let status_matches = status == &expected;
            if !status_matches {
                diagnostics.push("status.json is stale or does not match effect-closure.json".to_owned());
            }
        }
        if let Some(summary_bytes) = files.get(SUMMARY_FILE) {
            if summary_bytes.as_slice() != render_summary(receipt, kamacite.as_ref()).as_bytes() {
                diagnostics.push("summary.txt is stale or does not match effect-closure.json".to_owned());
            }
        } else {
            diagnostics.push(format!("missing admission artifact `{SUMMARY_FILE}`"));
        }
        match render_provenance(&files, &receipt.inputs.profile_id) {
            Ok(expected) => match files.get(PROVENANCE_FILE) {
                Some(actual) if actual.as_slice() == expected.as_bytes() => {}
                Some(_) => {
                    diagnostics.push("provenance.jsonl is stale or contains an artifact hash mismatch".to_owned());
                }
                None => diagnostics.push(format!("missing admission artifact `{PROVENANCE_FILE}`")),
            },
            Err(error) => diagnostics.push(error),
        }
    }

    diagnostics.sort();
    diagnostics.dedup();
    let mut checked_artifacts = REQUIRED_PROVENANCE_ARTIFACTS.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>();
    for artifact in [
        KAMACITE_PROFILE_ARTIFACT,
        KAMACITE_SUMMARY_ARTIFACT,
        KAMACITE_COMPARISON_ARTIFACT,
        KAMACITE_HANDOFF_ARTIFACT,
    ] {
        if files.contains_key(artifact) {
            checked_artifacts.push(artifact.to_owned());
        }
    }
    Ok(AdmissionVerificationReport {
        schema_version: ADMISSION_VERIFICATION_SCHEMA.to_owned(),
        artifact_dir: artifact_dir.display().to_string(),
        status: if diagnostics.is_empty() {
            STATUS_VALID
        } else {
            STATUS_INVALID
        }
        .to_owned(),
        receipt_identity: receipt.map(|receipt| receipt.receipt_identity),
        checked_artifacts,
        auth: None,
        diagnostics,
        non_claims: vec![NON_CLAIM_INTEGRITY.to_owned()],
    })
}

#[allow(
    clippy::too_many_lines,
    reason = "the bundle verifier keeps the ordered identity, parent, and artifact checks together"
)]
fn verify_kamacite_bundle(
    files: &BTreeMap<String, Vec<u8>>,
    receipt: Option<&EffectClosureReceipt>,
    status: Option<&AdmissionStatusArtifact>,
    diagnostics: &mut Vec<String>,
) -> Option<KamaciteBundle> {
    let required = [
        KAMACITE_PROFILE_ARTIFACT,
        KAMACITE_SUMMARY_ARTIFACT,
        KAMACITE_COMPARISON_ARTIFACT,
    ];
    let present = required
        .iter()
        .chain(std::iter::once(&KAMACITE_HANDOFF_ARTIFACT))
        .filter(|name| files.contains_key(**name))
        .count();
    let declared = status.is_some_and(|status| status.kamacite.is_some());
    if present == 0 {
        if declared {
            diagnostics
                .push("status.json declares a Kamacite comparison but the Kamacite artifacts are missing".to_owned());
        }
        return None;
    }
    if !declared {
        diagnostics.push("Kamacite artifacts are present but status.json does not declare a comparison".to_owned());
    }
    let mut complete = true;
    for artifact in required {
        if !files.contains_key(artifact) {
            diagnostics.push(format!("missing admission artifact `{artifact}`"));
            complete = false;
        }
    }
    if !complete {
        return None;
    }
    let profile_bytes = files.get(KAMACITE_PROFILE_ARTIFACT)?.clone();
    let projection_bytes = files.get(KAMACITE_SUMMARY_ARTIFACT)?.clone();
    let comparison_bytes = files.get(KAMACITE_COMPARISON_ARTIFACT)?.clone();
    let profile = match serde_json::from_slice::<KamaciteEffectProfile>(&profile_bytes) {
        Ok(profile) => profile,
        Err(error) => {
            diagnostics.push(format!("malformed admission artifact `{KAMACITE_PROFILE_ARTIFACT}`: {error}"));
            return None;
        }
    };
    let projection = match serde_json::from_slice::<KamaciteEffectSummaryProjection>(&projection_bytes) {
        Ok(projection) => projection,
        Err(error) => {
            diagnostics.push(format!("malformed admission artifact `{KAMACITE_SUMMARY_ARTIFACT}`: {error}"));
            return None;
        }
    };
    let stored_comparison = match serde_json::from_slice::<KamaciteEffectComparison>(&comparison_bytes) {
        Ok(comparison) => comparison,
        Err(error) => {
            diagnostics.push(format!("malformed admission artifact `{KAMACITE_COMPARISON_ARTIFACT}`: {error}"));
            return None;
        }
    };
    for issue in kamacite_effect_closure::validate_profile(&profile) {
        diagnostics.push(format!("{}: {}", issue.code, issue.message));
    }
    for issue in kamacite_effect_closure::validate_projection(&projection) {
        diagnostics.push(format!("{}: {}", issue.code, issue.message));
    }
    let observed = format!("b3:{}", crate::evidence::artifact_blake3_hex(&projection_bytes));
    if observed != profile.kamacite.projection_blake3 {
        diagnostics
            .push("stored Kamacite effect summary does not match the mapping profile projection identity".to_owned());
    }
    let receipt = receipt?;
    let expected_comparison = kamacite_effect_closure::compare(receipt, &profile, &projection);
    match (
        kamacite_effect_closure::canonical_json(&stored_comparison),
        kamacite_effect_closure::canonical_json(&expected_comparison),
    ) {
        (Ok(stored), Ok(expected)) if stored == expected => {}
        (Ok(_), Ok(_)) => diagnostics.push(
            "Kamacite comparison artifact is stale or does not match its closure, profile, and projection parents"
                .to_owned(),
        ),
        (Err(error), _) | (_, Err(error)) => diagnostics.push(error),
    }
    let handoff_bytes = files.get(KAMACITE_HANDOFF_ARTIFACT).cloned();
    let expected_handoff = if expected_comparison.status == AdmissionStatus::Admitted {
        match kamacite_effect_closure::build_handoff(&expected_comparison) {
            Ok(handoff) => Some(handoff),
            Err(error) => {
                diagnostics.push(error);
                None
            }
        }
    } else {
        None
    };
    match (&handoff_bytes, &expected_handoff) {
        (Some(bytes), Some(expected)) => match serde_json::from_slice::<KamaciteEffectHandoff>(bytes) {
            Ok(stored) => {
                diagnostics.extend(
                    kamacite_effect_closure::verify_handoff_against(&stored, &expected_comparison)
                        .into_iter()
                        .map(|issue| format!("{}: {}", issue.code, issue.message)),
                );
                match (
                    kamacite_effect_closure::canonical_json(&stored),
                    kamacite_effect_closure::canonical_json(expected),
                ) {
                    (Ok(stored), Ok(expected)) if stored == expected => {}
                    (Ok(_), Ok(_)) => {
                        diagnostics.push("Kamacite handoff artifact is stale or does not match its parents".to_owned());
                    }
                    (Err(error), _) | (_, Err(error)) => diagnostics.push(error),
                }
            }
            Err(error) => {
                diagnostics.push(format!("malformed admission artifact `{KAMACITE_HANDOFF_ARTIFACT}`: {error}"));
            }
        },
        (Some(_), None) => {
            diagnostics.push("denied Kamacite comparison must not emit a handoff".to_owned());
        }
        (None, Some(_)) => {
            diagnostics.push("admitted Kamacite comparison is missing its handoff artifact".to_owned());
        }
        (None, None) => {}
    }
    Some(KamaciteBundle {
        profile_bytes,
        projection_bytes,
        comparison: expected_comparison,
        comparison_bytes,
        handoff: expected_handoff,
        handoff_bytes,
    })
}

/// Re-derives what a closure receipt asserts about its bound function-address report and workspace
/// policy: a valid policy, a deny-mode report with the receipt's record root and source envelopes,
/// and, for an `admitted` receipt, nodes, roles, and reviewed summaries that match both. `admit
/// verify` and `artifact verify` share this so an edited, re-identified bundle fails either way.
pub(crate) fn verify_bound_closure_inputs(
    receipt: &EffectClosureReceipt,
    function_report: Option<&FunctionAddressReport>,
    policy: Option<&WorkspacePolicy>,
) -> Vec<String> {
    let mut diagnostics = policy.map(effect_closure::validate_workspace_policy).unwrap_or_default();
    if let Some(function_report) = function_report {
        if function_report.mode != FUNCTION_ADDRESS_MODE_DENY {
            diagnostics.push("function-address parent is not deny mode".to_owned());
        }
        match function_address_record_root(function_report) {
            Ok(root) if root == receipt.inputs.function_address_record_root => {}
            Ok(_) => diagnostics.push("function-address record root does not match the receipt".to_owned()),
            Err(error) => diagnostics.push(error),
        }
        let receipt_sources = receipt
            .sources
            .iter()
            .map(|source| FunctionAddressSourceEnvelope {
                path: source.path.clone(),
                source_identity: source.source_identity.clone(),
            })
            .collect::<BTreeSet<_>>();
        let report_sources = function_report.source_envelopes.iter().cloned().collect::<BTreeSet<_>>();
        if receipt_sources != report_sources {
            diagnostics.push("receipt sources do not match function-address source envelopes".to_owned());
        }
    }
    if let (Some(function_report), Some(policy)) = (function_report, policy) {
        diagnostics.extend(effect_closure::validate_admitted_bindings(receipt, &function_report.records, policy));
    }
    diagnostics
}

fn verify_bound_artifact_hash(
    expected: &str,
    files: &BTreeMap<String, Vec<u8>>,
    artifact: &str,
    diagnostics: &mut Vec<String>,
) {
    match files.get(artifact) {
        Some(bytes) if crate::evidence::artifact_blake3_hex(bytes) == expected => {}
        Some(_) => diagnostics.push(format!("artifact `{artifact}` BLAKE3 identity does not match the receipt")),
        None => diagnostics.push(format!("missing admission artifact `{artifact}`")),
    }
}

fn parse_required<T: for<'de> Deserialize<'de>>(
    files: &BTreeMap<String, Vec<u8>>,
    artifact: &str,
    diagnostics: &mut Vec<String>,
) -> Option<T> {
    let Some(bytes) = files.get(artifact) else {
        diagnostics.push(format!("missing admission artifact `{artifact}`"));
        return None;
    };
    match serde_json::from_slice(bytes) {
        Ok(value) => Some(value),
        Err(error) => {
            diagnostics.push(format!("malformed admission artifact `{artifact}`: {error}"));
            None
        }
    }
}

fn checked_workspace_path(workspace_root: &Path, path: &Path, kind: &str) -> Result<PathBuf, String> {
    let relative = if path.is_absolute() {
        path.strip_prefix(workspace_root)
            .map_err(|_| format!("{kind} path must stay within workspace root: {}", path.display()))?
            .to_path_buf()
    } else {
        path.to_path_buf()
    };
    if relative.as_os_str().is_empty()
        || relative
            .components()
            .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
    {
        return Err(format!("{kind} path must be a non-empty workspace-relative path: {}", path.display()));
    }
    Ok(workspace_root.join(relative))
}

fn absolute_from_workspace(workspace_root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    }
}

fn relative(path: &str) -> Result<RootRelativePath, String> {
    RootRelativePath::new(path)
}

fn read_required(path: &Path) -> Result<Vec<u8>, String> {
    fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))
}

fn normalized_text_bytes(text: &str) -> Vec<u8> {
    let mut bytes = text.as_bytes().to_vec();
    bytes.push(b'\n');
    bytes
}

fn pretty_json_string<T: Serialize>(value: &T) -> Result<String, String> {
    let mut rendered = serde_json::to_string_pretty(value).map_err(|error| format!("serialize JSON: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

fn pretty_json_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    let mut bytes =
        serde_json::to_vec_pretty(value).map_err(|error| format!("serialize admission artifact: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use artifact_auth_core::STATEMENT_SCHEMA_V1;
    use ed25519_dalek::Signer;
    use ed25519_dalek::SigningKey;

    use super::*;

    const TEST_PROFILE: &str = "deterministic-core";
    const TEST_HASH: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    // r[verify octet.effect_closure.dependency_summaries.target]
    #[test]
    fn metadata_feature_selection_is_narrow_and_deterministic() {
        let args = vec![
            "--lib".to_owned(),
            "--features".to_owned(),
            "serde,proof".to_owned(),
            "--all-targets".to_owned(),
            "--no-default-features".to_owned(),
        ];
        assert_eq!(metadata_cargo_args(&args).expect("metadata args"), vec![
            "--features",
            "serde,proof",
            "--no-default-features"
        ]);
        assert!(selected_target(&args).is_err());
        assert_eq!(
            selected_target(&["--target=x86_64-unknown-linux-gnu".to_owned()]).expect("explicit target"),
            "x86_64-unknown-linux-gnu"
        );
        assert!(selected_target(&["--target".to_owned(), "a".to_owned(), "--target=b".to_owned()]).is_err());
    }

    #[test]
    fn metadata_args_forward_manifest_config_and_short_features() {
        let args = [
            "--lib",
            "-F",
            "extra",
            "-Fother",
            "--config",
            "net.offline=true",
            "--config=build.rustflags=[]",
            "--manifest-path",
            "p/Cargo.toml",
            "--manifest-path=q/Cargo.toml",
            "--target",
            "x86_64-unknown-linux-gnu",
            "--release",
            "--frozen",
        ]
        .map(str::to_owned);
        assert_eq!(metadata_cargo_args(&args).expect("metadata args"), vec![
            "-F",
            "extra",
            "-Fother",
            "--config",
            "net.offline=true",
            "--config=build.rustflags=[]",
            "--manifest-path",
            "p/Cargo.toml",
            "--manifest-path=q/Cargo.toml",
            "--frozen",
        ]);
        for flag in ["--manifest-path", "--config", "--features", "-F"] {
            assert_eq!(
                metadata_cargo_args(&["--lib".to_owned(), flag.to_owned()]),
                Err(format!("Cargo option `{flag}` is missing its value"))
            );
        }
    }

    #[test]
    fn workspace_path_validation_rejects_escape_and_accepts_relative() {
        let root = Path::new("/workspace");
        assert_eq!(
            checked_workspace_path(root, Path::new("policy.json"), "policy").expect("relative policy"),
            PathBuf::from("/workspace/policy.json")
        );
        assert!(checked_workspace_path(root, Path::new("../policy.json"), "policy").is_err());
        assert!(checked_workspace_path(root, Path::new("/outside/policy.json"), "policy").is_err());
    }

    const TEST_SOURCE_PATH: &str = "src/lib.rs";
    const TEST_TARGET: &str = "test-target";
    const TEST_TOOLCHAIN: &str = "test-toolchain";
    const TEST_RUSTC_VERSION: &str = "rustc-test";

    struct BundleFixture {
        command_bytes: Vec<u8>,
        check_status_bytes: Vec<u8>,
        function_address_bytes: Vec<u8>,
        policy_bytes: Vec<u8>,
        graph_bytes: Vec<u8>,
        input: ClosureInput,
    }

    fn test_function_address(fill: char) -> String {
        format!("b3:{}", fill.to_string().repeat(64))
    }

    fn bundle_fixture(
        source: &str,
        records: &[(&str, char)],
        call_summaries: Vec<effect_closure::CallSummary>,
        providers: Vec<ProviderIdentity>,
    ) -> BundleFixture {
        let command_bytes = normalized_text_bytes("cargo octet admit --profile deterministic-core --closure");
        let source_identity = crate::determinism::full_hash(source);
        let check_status_bytes = pretty_json_bytes(&serde_json::json!({
            "status": "clean",
            "exit_code": EXIT_ADMITTED,
            "metadata": {
                "rustc_version": TEST_RUSTC_VERSION,
                "toolchain": TEST_TOOLCHAIN,
                "profile_name": TEST_PROFILE,
                "profile_hash": TEST_HASH,
                "config_hash": TEST_HASH
            },
            "error_findings": 0
        }))
        .expect("check status JSON");
        let records = records
            .iter()
            .map(|(name, fill)| FunctionAddressRecord {
                path: TEST_SOURCE_PATH.to_owned(),
                name: (*name).to_owned(),
                function_address: test_function_address(*fill),
                source_identity: source_identity.clone(),
            })
            .collect::<Vec<_>>();
        let function_address_bytes = pretty_json_bytes(&serde_json::json!({
            "mode": FUNCTION_ADDRESS_MODE_DENY,
            "source_envelopes": [{
                "path": TEST_SOURCE_PATH,
                "source_identity": source_identity
            }],
            "records": records,
            "evidence_chain_handoff": {
                "canonical_roots": [{
                    "kind": FUNCTION_ADDRESS_ROOT_KIND,
                    "digest": TEST_HASH
                }]
            }
        }))
        .expect("function-address JSON");
        let policy = WorkspacePolicy {
            schema_version: effect_closure::WORKSPACE_POLICY_SCHEMA_VERSION.to_owned(),
            profile_id: TEST_PROFILE.to_owned(),
            function_address_baseline: "function-addresses.json".to_owned(),
            module_roles: vec![effect_closure::ModuleRoleAssignment {
                path: TEST_SOURCE_PATH.to_owned(),
                role: effect_closure::ModuleRole::Core,
            }],
            call_summaries,
            non_claims: vec!["bounded test policy".to_owned()],
        };
        let policy_bytes = pretty_json_bytes(&policy).expect("policy JSON");
        let graph = DependencyGraphArtifact {
            schema_version: DEPENDENCY_GRAPH_SCHEMA.to_owned(),
            target: TEST_TARGET.to_owned(),
            toolchain: TEST_TOOLCHAIN.to_owned(),
            rustc_version: TEST_RUSTC_VERSION.to_owned(),
            cargo_lock_blake3: TEST_HASH.to_owned(),
            providers: providers.clone(),
        };
        let graph_bytes = pretty_json_bytes(&graph).expect("graph JSON");
        let input = ClosureInput {
            profile_id: TEST_PROFILE.to_owned(),
            profile_hash: TEST_HASH.to_owned(),
            config_hash: TEST_HASH.to_owned(),
            command_blake3: crate::evidence::artifact_blake3_hex(&command_bytes),
            check_status_blake3: crate::evidence::artifact_blake3_hex(&check_status_bytes),
            function_address_artifact_blake3: crate::evidence::artifact_blake3_hex(&function_address_bytes),
            function_address_record_root: TEST_HASH.to_owned(),
            workspace_policy_blake3: crate::evidence::artifact_blake3_hex(&policy_bytes),
            cargo_graph_blake3: crate::evidence::artifact_blake3_hex(&graph_bytes),
            cargo_lock_blake3: TEST_HASH.to_owned(),
            target: TEST_TARGET.to_owned(),
            toolchain: TEST_TOOLCHAIN.to_owned(),
            sources: vec![SourceEnvelope {
                path: TEST_SOURCE_PATH.to_owned(),
                source: source.to_owned(),
                source_identity,
            }],
            function_addresses: records,
            resolved_providers: providers,
            catalog: effect_closure::embedded_catalog().expect("catalog"),
            policy,
        };
        BundleFixture {
            command_bytes,
            check_status_bytes,
            function_address_bytes,
            policy_bytes,
            graph_bytes,
            input,
        }
    }

    fn write_bundle(
        dir: &Path,
        fixture: &BundleFixture,
        receipt: &EffectClosureReceipt,
        kamacite: Option<&KamaciteBundle>,
    ) {
        let mut artifacts = BTreeMap::from([
            (COMMAND_FILE.to_owned(), fixture.command_bytes.clone()),
            (
                STATUS_FILE.to_owned(),
                pretty_json_bytes(&status_from_receipt(
                    receipt,
                    kamacite.map(|bundle| &bundle.comparison),
                    kamacite.and_then(|bundle| bundle.handoff.as_ref()),
                ))
                .expect("status JSON"),
            ),
            (SUMMARY_FILE.to_owned(), render_summary(receipt, kamacite).into_bytes()),
            (effect_closure::RECEIPT_FILE.to_owned(), pretty_json_bytes(receipt).expect("receipt JSON")),
            (WORKSPACE_POLICY_ARTIFACT.to_owned(), fixture.policy_bytes.clone()),
            (DEPENDENCY_GRAPH_ARTIFACT.to_owned(), fixture.graph_bytes.clone()),
            (CHECK_STATUS_ARTIFACT.to_owned(), fixture.check_status_bytes.clone()),
            (CHECK_FUNCTION_ADDRESS_ARTIFACT.to_owned(), fixture.function_address_bytes.clone()),
        ]);
        if let Some(kamacite) = kamacite {
            artifacts.insert(KAMACITE_PROFILE_ARTIFACT.to_owned(), kamacite.profile_bytes.clone());
            artifacts.insert(KAMACITE_SUMMARY_ARTIFACT.to_owned(), kamacite.projection_bytes.clone());
            artifacts.insert(KAMACITE_COMPARISON_ARTIFACT.to_owned(), kamacite.comparison_bytes.clone());
            if let Some(handoff_bytes) = &kamacite.handoff_bytes {
                artifacts.insert(KAMACITE_HANDOFF_ARTIFACT.to_owned(), handoff_bytes.clone());
            }
        }
        let provenance = render_provenance(&artifacts, TEST_PROFILE).expect("provenance");
        artifacts.insert(PROVENANCE_FILE.to_owned(), provenance.into_bytes());
        for (locator, bytes) in &artifacts {
            let path = dir.join(locator);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create artifact parent");
            }
            fs::write(path, bytes).expect("write artifact");
        }
    }

    fn shipped_kamacite_bundle(receipt: &EffectClosureReceipt) -> KamaciteBundle {
        let profile: KamaciteEffectProfile =
            serde_json::from_str(include_str!("../../standards/effect-closure/generated/kamacite-effect-profile.json"))
                .expect("shipped profile");
        let projection: KamaciteEffectSummaryProjection =
            serde_json::from_str(include_str!("../../standards/effect-closure/fixtures/kamacite/effect-summary.json"))
                .expect("frozen projection");
        let comparison = kamacite_effect_closure::compare(receipt, &profile, &projection);
        let comparison_bytes =
            kamacite_effect_closure::canonical_json(&comparison).expect("comparison JSON").into_bytes();
        let (handoff, handoff_bytes) = if comparison.status == AdmissionStatus::Admitted {
            let handoff = kamacite_effect_closure::build_handoff(&comparison).expect("handoff");
            let bytes = kamacite_effect_closure::canonical_json(&handoff).expect("handoff JSON").into_bytes();
            (Some(handoff), Some(bytes))
        } else {
            (None, None)
        };
        KamaciteBundle {
            profile_bytes: include_bytes!("../../standards/effect-closure/generated/kamacite-effect-profile.json")
                .to_vec(),
            projection_bytes: include_bytes!("../../standards/effect-closure/fixtures/kamacite/effect-summary.json")
                .to_vec(),
            comparison,
            comparison_bytes,
            handoff,
            handoff_bytes,
        }
    }

    fn tamper_json(bytes: &[u8], field: &str, value: &str) -> Vec<u8> {
        let mut json: serde_json::Value = serde_json::from_slice(bytes).expect("artifact JSON");
        json[field] = serde_json::Value::String(value.to_owned());
        serde_json::to_vec_pretty(&json).expect("tampered JSON")
    }

    #[test]
    fn kamacite_inputs_fail_closed_on_stale_projection_and_invalid_profile() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let projection_path = root.join("standards/effect-closure/fixtures/kamacite/effect-summary.json");
        fs::create_dir_all(projection_path.parent().expect("projection parent")).expect("create projection dir");
        fs::write(
            &projection_path,
            include_bytes!("../../standards/effect-closure/fixtures/kamacite/effect-summary.json"),
        )
        .expect("write projection");
        fs::write(
            root.join("kamacite.json"),
            include_bytes!("../../standards/effect-closure/generated/kamacite-effect-profile.json"),
        )
        .expect("write profile");

        let prepared = prepare_kamacite_inputs(root, Path::new("kamacite.json")).expect("valid Kamacite inputs");
        assert_eq!(prepared.profile.mapping_id, "octet-kamacite-effect-mapping-v1");

        fs::write(&projection_path, b"{}\n").expect("tamper projection");
        let error =
            prepare_kamacite_inputs(root, Path::new("kamacite.json")).err().expect("stale projection must fail");
        assert!(error.contains("stale Kamacite effect summary projection identity"), "{error}");

        fs::write(
            &projection_path,
            include_bytes!("../../standards/effect-closure/fixtures/kamacite/effect-summary.json"),
        )
        .expect("restore projection");
        fs::write(
            root.join("kamacite.json"),
            tamper_json(
                include_bytes!("../../standards/effect-closure/generated/kamacite-effect-profile.json"),
                "profile_id",
                "other-profile",
            ),
        )
        .expect("write invalid profile");
        let error = prepare_kamacite_inputs(root, Path::new("kamacite.json")).err().expect("invalid profile must fail");
        assert!(error.contains("invalid Kamacite effect profile"), "{error}");

        let error = prepare_kamacite_inputs(root, Path::new("missing.json")).err().expect("missing profile must fail");
        assert!(error.contains("read"), "{error}");
    }

    // r[verify octet.effect_closure.kamacite_fixtures]
    #[test]
    #[allow(
        clippy::too_many_lines,
        reason = "the fixture exercises the complete positive and tamper matrix in one deterministic test"
    )]
    fn kamacite_handoff_bundle_verifies_and_tampering_fails_closed() {
        let temp = tempfile::tempdir().expect("tempdir");
        let fixture = bundle_fixture("fn root() {}\n", &[("root", 'a')], Vec::new(), Vec::new());
        let receipt = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(receipt.status, AdmissionStatus::Admitted);
        let bundle = shipped_kamacite_bundle(&receipt);
        assert_eq!(bundle.comparison.status, AdmissionStatus::Admitted, "{:?}", bundle.comparison.diagnostics);
        write_bundle(temp.path(), &fixture, &receipt, Some(&bundle));
        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);

        let mut tampered_handoff = bundle.clone();
        tampered_handoff.handoff_bytes =
            Some(tamper_json(bundle.handoff_bytes.as_ref().expect("handoff bytes"), "mapping_id", "forged"));
        write_bundle(temp.path(), &fixture, &receipt, Some(&tampered_handoff));
        let invalid = verify_bundle_inner(temp.path(), false).expect("invalid verification report");
        assert_eq!(invalid.status, STATUS_INVALID);
        assert!(
            invalid
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("kamacite_handoff_identity_mismatch")),
            "{:?}",
            invalid.diagnostics
        );

        let mut tampered_comparison = bundle.clone();
        tampered_comparison.comparison_bytes = tamper_json(&bundle.comparison_bytes, "mapping_id", "forged-comparison");
        write_bundle(temp.path(), &fixture, &receipt, Some(&tampered_comparison));
        let invalid = verify_bundle_inner(temp.path(), false).expect("invalid verification report");
        assert_eq!(invalid.status, STATUS_INVALID);
        assert!(
            invalid
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("Kamacite comparison artifact is stale")),
            "{:?}",
            invalid.diagnostics
        );

        let mut tampered_profile = bundle.clone();
        tampered_profile.profile_bytes = tamper_json(&bundle.profile_bytes, "mapping_id", "forged-profile");
        write_bundle(temp.path(), &fixture, &receipt, Some(&tampered_profile));
        let invalid = verify_bundle_inner(temp.path(), false).expect("invalid verification report");
        assert_eq!(invalid.status, STATUS_INVALID);
        assert!(
            invalid
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("Kamacite comparison artifact is stale")),
            "{:?}",
            invalid.diagnostics
        );

        let mut missing_handoff = bundle.clone();
        missing_handoff.handoff = None;
        missing_handoff.handoff_bytes = None;
        write_bundle(temp.path(), &fixture, &receipt, Some(&missing_handoff));
        fs::remove_file(temp.path().join(KAMACITE_HANDOFF_ARTIFACT)).expect("remove handoff artifact");
        let invalid = verify_bundle_inner(temp.path(), false).expect("invalid verification report");
        assert_eq!(invalid.status, STATUS_INVALID);
        assert!(
            invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("missing its handoff artifact")),
            "{:?}",
            invalid.diagnostics
        );

        let mut denied_profile =
            serde_json::from_slice::<KamaciteEffectProfile>(&bundle.profile_bytes).expect("profile JSON");
        denied_profile.semantic_emissions.push(kamacite_effect_closure::SemanticEmissionRule {
            function_address: "b3:9999999999999999999999999999999999999999999999999999999999999999".to_owned(),
            call_site_id: None,
            provider: ProviderIdentity {
                kind: ProviderKind::Workspace,
                name: "octet".to_owned(),
                version: "0.1.0".to_owned(),
                source: "workspace".to_owned(),
                target: TEST_TARGET.to_owned(),
                features: Vec::new(),
            },
            operation_identity: KamaciteIdentity {
                domain: "effect-operation".to_owned(),
                algorithm: "blake3".to_owned(),
                hex: "7fa2709c205ccfa8c4b8773ab97396193a5130a2e42b2f94916802b04e07f5f8".to_owned(),
            },
            evidence: effect_closure::EvidenceReference {
                kind: "reviewed-semantic-emission-v1".to_owned(),
                locator: "src/lib.rs".to_owned(),
                blake3: TEST_HASH.to_owned(),
            },
            non_claims: vec!["This emission does not prove runtime reachability.".to_owned()],
        });
        let denied_profile_bytes = serde_json::to_vec_pretty(&denied_profile).expect("denied profile JSON");
        let denied_projection = serde_json::from_slice::<KamaciteEffectSummaryProjection>(&bundle.projection_bytes)
            .expect("projection JSON");
        let denied_comparison = kamacite_effect_closure::compare(&receipt, &denied_profile, &denied_projection);
        assert_eq!(denied_comparison.status, AdmissionStatus::Denied);
        let denied_bundle = KamaciteBundle {
            profile_bytes: denied_profile_bytes,
            projection_bytes: bundle.projection_bytes.clone(),
            comparison_bytes: kamacite_effect_closure::canonical_json(&denied_comparison)
                .expect("denied comparison JSON")
                .into_bytes(),
            comparison: denied_comparison,
            handoff: bundle.handoff.clone(),
            handoff_bytes: bundle.handoff_bytes.clone(),
        };
        write_bundle(temp.path(), &fixture, &receipt, Some(&denied_bundle));
        let invalid = verify_bundle_inner(temp.path(), false).expect("invalid verification report");
        assert_eq!(invalid.status, STATUS_INVALID);
        assert!(
            invalid
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("denied Kamacite comparison must not emit a handoff")),
            "{:?}",
            invalid.diagnostics
        );
    }

    fn toolchain_summary(
        caller_fill: char,
        shape: &str,
        effects: Vec<effect_closure::Effect>,
    ) -> (effect_closure::CallSummary, ProviderIdentity) {
        let provider = ProviderIdentity {
            kind: ProviderKind::Toolchain,
            name: "alloc".to_owned(),
            version: TEST_RUSTC_VERSION.to_owned(),
            source: PROVIDER_SOURCE_TOOLCHAIN.to_owned(),
            target: TEST_TARGET.to_owned(),
            features: Vec::new(),
        };
        let scope = effect_closure::prelude_call_scope(TEST_SOURCE_PATH, shape);
        let summary = effect_closure::CallSummary {
            call_site_id: effect_closure::call_site_identity(
                &test_function_address(caller_fill),
                TEST_SOURCE_PATH,
                &scope,
                shape,
                0,
            ),
            review_label: format!("reviewed `{shape}` toolchain call"),
            callee: shape.trim_start_matches("call:").to_owned(),
            provider: provider.clone(),
            effects,
            evidence: effect_closure::EvidenceReference {
                kind: "toolchain-api-review-v1".to_owned(),
                locator: "rustc-version#alloc".to_owned(),
                blake3: effect_closure::provider_identity_blake3(&provider),
            },
        };
        (summary, provider)
    }

    /// Rewrites the stored receipt the way the previous schema wrote it: schema label `v1` and
    /// call edges without the `scope` field that schema v2 requires.
    fn downgrade_receipt_to_v1(receipt: &EffectClosureReceipt) -> Vec<u8> {
        let mut value = serde_json::to_value(receipt).expect("receipt JSON");
        value["schema_version"] = serde_json::Value::from("octet-effect-closure-receipt/v1");
        for node in value["nodes"].as_array_mut().expect("receipt nodes") {
            for call in node["calls"].as_array_mut().expect("node calls") {
                call.as_object_mut().expect("call object").remove("scope");
            }
        }
        serde_json::to_vec_pretty(&value).expect("v1 receipt JSON")
    }

    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn previous_receipt_schema_is_reported_unsupported() {
        let temp = tempfile::tempdir().expect("tempdir");
        let fixture = bundle_fixture("fn root() { mystery(); }\n", &[("root", 'a')], Vec::new(), Vec::new());
        let receipt = effect_closure::compute_closure(fixture.input.clone());
        assert!(!receipt.nodes[0].calls.is_empty());
        write_bundle(temp.path(), &fixture, &receipt, None);
        fs::write(temp.path().join(effect_closure::RECEIPT_FILE), downgrade_receipt_to_v1(&receipt))
            .expect("write v1 receipt");

        let report = verify_bundle_inner(temp.path(), false).expect("verification report");

        assert_eq!(report.status, STATUS_INVALID);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unsupported effect-closure receipt schema")
                    && diagnostic.contains("octet-effect-closure-receipt/v1")),
            "{:?}",
            report.diagnostics
        );
        assert!(
            !report.diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed")),
            "{:?}",
            report.diagnostics
        );
    }

    // r[verify octet.effect_closure.evidence.visible]
    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn complete_admission_bundle_verifies_and_tampering_is_invalid() {
        let temp = tempfile::tempdir().expect("tempdir");
        fs::create_dir_all(temp.path().join(CHECK_DIR)).expect("create check dir");
        let fixture = bundle_fixture("fn root() {}\n", &[("root", 'a')], Vec::new(), Vec::new());
        let receipt = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(receipt.status, AdmissionStatus::Admitted);
        write_bundle(temp.path(), &fixture, &receipt, None);

        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);
        let incomplete_parent = verify_bundle(temp.path()).expect("parent verification report");
        assert_eq!(incomplete_parent.status, STATUS_INVALID);
        assert!(
            incomplete_parent
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("strict-profile parent artifact verification failed"))
        );

        fs::write(temp.path().join(SUMMARY_FILE), b"tampered\n").expect("tamper summary");
        let invalid = verify_bundle_inner(temp.path(), false).expect("invalid verification report");
        assert_eq!(invalid.status, STATUS_INVALID);
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("summary.txt is stale")));
        assert!(invalid.diagnostics.iter().any(|diagnostic| diagnostic.contains("provenance.jsonl is stale")));
    }

    fn forge_admitted(mut receipt: EffectClosureReceipt) -> EffectClosureReceipt {
        receipt.diagnostics.clear();
        receipt.status = AdmissionStatus::Admitted;
        effect_closure::finalize_receipt(receipt)
    }

    fn assert_forgery_rejected(fixture: &BundleFixture, forged: &EffectClosureReceipt, expected: &str) {
        let temp = tempfile::tempdir().expect("tempdir");
        write_bundle(temp.path(), fixture, forged, None);
        let report = verify_bundle_inner(temp.path(), false).expect("verification report");
        assert_eq!(report.status, STATUS_INVALID, "forged admitted receipt verified");
        assert!(
            report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
            "expected `{expected}` in {:?}",
            report.diagnostics
        );
    }

    /// Rebinds the fixture to a workspace policy whose module roles differ from the default
    /// single `core` assignment, keeping every bound identity consistent with the new policy bytes.
    fn with_module_roles(
        mut fixture: BundleFixture,
        module_roles: Vec<effect_closure::ModuleRoleAssignment>,
    ) -> BundleFixture {
        fixture.input.policy.module_roles = module_roles;
        fixture.policy_bytes = pretty_json_bytes(&fixture.input.policy).expect("policy JSON");
        fixture.input.workspace_policy_blake3 = crate::evidence::artifact_blake3_hex(&fixture.policy_bytes);
        fixture
    }

    /// Rewrites the first call of a denied single-node receipt into the reviewed-summary edge the
    /// policy summary would produce, the way an attacker would hand-edit a denied bundle.
    fn forge_reviewed_edge(
        mut receipt: EffectClosureReceipt,
        summary: &effect_closure::CallSummary,
    ) -> EffectClosureReceipt {
        let node = &mut receipt.nodes[0];
        let call = &mut node.calls[0];
        assert_eq!(call.call_site_id, summary.call_site_id);
        call.resolution = effect_closure::CallResolution::ReviewedSummary;
        call.provider = Some(summary.provider.clone());
        call.effects.clone_from(&summary.effects);
        call.review_label = Some(summary.review_label.clone());
        call.evidence = Some(summary.evidence.clone());
        node.direct_effects.clone_from(&summary.effects);
        node.closure_effects.clone_from(&summary.effects);
        receipt.providers = vec![summary.provider.clone()];
        forge_admitted(receipt)
    }

    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admitted_receipt_must_match_bound_function_addresses_and_policy_summaries() {
        let source = "fn root() { let _values: Vec<u8> = Vec::new(); }\n";
        let (summary, provider) = toolchain_summary('b', "call:Vec::new", vec![effect_closure::Effect::Alloc]);
        let fixture = bundle_fixture(source, &[("root", 'b')], vec![summary], vec![provider]);
        let admitted = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(admitted.status, AdmissionStatus::Admitted, "{:?}", admitted.diagnostics);
        let temp = tempfile::tempdir().expect("tempdir");
        write_bundle(temp.path(), &fixture, &admitted, None);
        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);

        let (summary, provider) = toolchain_summary('b', "call:Vec::new", vec![effect_closure::Effect::Panic]);
        let fixture = bundle_fixture(source, &[("root", 'b')], vec![summary], vec![provider]);
        let mut narrowed = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(narrowed.status, AdmissionStatus::Denied);
        narrowed.nodes[0].calls[0].effects.clear();
        narrowed.nodes[0].direct_effects.clear();
        narrowed.nodes[0].closure_effects.clear();
        assert_forgery_rejected(&fixture, &forge_admitted(narrowed), "does not match its workspace-policy summary");

        let fixture = bundle_fixture("fn root() {}\n", &[("ghost", 'c'), ("root", 'd')], Vec::new(), Vec::new());
        let unmatched = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(unmatched.status, AdmissionStatus::Denied);
        assert_forgery_rejected(
            &fixture,
            &forge_admitted(unmatched),
            "do not exactly match the bound function-address",
        );

        let (summary, provider) = toolchain_summary('e', "call:gone", Vec::new());
        let fixture = bundle_fixture("fn root() {}\n", &[("root", 'e')], vec![summary], vec![provider]);
        let stale = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(stale.status, AdmissionStatus::Denied);
        assert_forgery_rejected(&fixture, &forge_admitted(stale), "is not used by the admitted receipt");
    }

    // r[verify octet.effect_closure.roles.invalid]
    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admitted_receipt_must_bind_exactly_one_core_role_per_source() {
        let fixture = bundle_fixture("fn root() {}\n", &[("root", 'f')], Vec::new(), Vec::new());
        let fixture = with_module_roles(fixture, vec![effect_closure::ModuleRoleAssignment {
            path: TEST_SOURCE_PATH.to_owned(),
            role: effect_closure::ModuleRole::Shell,
        }]);
        let shell = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(shell.status, AdmissionStatus::Denied);
        assert_forgery_rejected(&fixture, &forge_admitted(shell.clone()), "exactly one workspace-policy role `core`");
        let mut relabelled_core = shell;
        relabelled_core.nodes[0].role = effect_closure::ModuleRole::Core;
        assert_forgery_rejected(&fixture, &forge_admitted(relabelled_core), "exactly one workspace-policy role `core`");

        let fixture = bundle_fixture("fn root() {}\n", &[("root", 'f')], Vec::new(), Vec::new());
        let fixture = with_module_roles(fixture, vec![effect_closure::ModuleRoleAssignment {
            path: "src/other.rs".to_owned(),
            role: effect_closure::ModuleRole::Core,
        }]);
        let unroled = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(unroled.status, AdmissionStatus::Denied);
        assert_eq!(unroled.nodes[0].role, effect_closure::ModuleRole::Core, "unroled paths default to core");
        assert_forgery_rejected(&fixture, &forge_admitted(unroled), "exactly one workspace-policy role `core`");
    }

    const ALLOCATING_SOURCE: &str = "fn root() { let _values: Vec<u8> = Vec::new(); }\n";

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admitted_toolchain_summary_evidence_must_bind_its_provider_identity() {
        let source = ALLOCATING_SOURCE;
        let (mut summary, provider) = toolchain_summary('9', "call:Vec::new", vec![effect_closure::Effect::Alloc]);
        let fixture = bundle_fixture(source, &[("root", '9')], vec![summary.clone()], vec![provider.clone()]);
        let provider_bound = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(provider_bound.status, AdmissionStatus::Admitted, "{:?}", provider_bound.diagnostics);
        let temp = tempfile::tempdir().expect("tempdir");
        write_bundle(temp.path(), &fixture, &provider_bound, None);
        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);

        summary.evidence.blake3 = crate::determinism::full_hash("a different toolchain provider");
        let fixture = bundle_fixture(source, &[("root", '9')], vec![summary.clone()], vec![provider]);
        let stale_toolchain = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(stale_toolchain.status, AdmissionStatus::Denied);
        assert_forgery_rejected(
            &fixture,
            &forge_reviewed_edge(stale_toolchain, &summary),
            "is not bound to the recorded Cargo.lock or provider identity",
        );
    }

    // r[verify octet.effect_closure.dependency_summaries.invalid]
    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn admitted_cargo_summary_evidence_must_bind_the_recorded_cargo_lock() {
        let source = ALLOCATING_SOURCE;
        let (mut summary, _) = toolchain_summary('8', "call:Vec::new", vec![effect_closure::Effect::Alloc]);
        summary.provider = ProviderIdentity {
            kind: ProviderKind::CargoPackage,
            name: "vecs".to_owned(),
            version: "1.0.0".to_owned(),
            source: "registry+https://github.com/rust-lang/crates.io-index".to_owned(),
            target: TEST_TARGET.to_owned(),
            features: Vec::new(),
        };
        summary.evidence = effect_closure::EvidenceReference {
            kind: "cargo-lock-review-v1".to_owned(),
            locator: "Cargo.lock#vecs".to_owned(),
            blake3: TEST_HASH.to_owned(),
        };
        let fixture = bundle_fixture(source, &[("root", '8')], vec![summary.clone()], vec![summary.provider.clone()]);
        let lock_bound = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(lock_bound.status, AdmissionStatus::Admitted, "{:?}", lock_bound.diagnostics);
        let temp = tempfile::tempdir().expect("tempdir");
        write_bundle(temp.path(), &fixture, &lock_bound, None);
        let valid = verify_bundle_inner(temp.path(), false).expect("valid verification report");
        assert_eq!(valid.status, STATUS_VALID, "{:?}", valid.diagnostics);

        summary.evidence.blake3 = crate::determinism::full_hash("an older Cargo.lock");
        let fixture = bundle_fixture(source, &[("root", '8')], vec![summary.clone()], vec![summary.provider.clone()]);
        let stale_lock = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(stale_lock.status, AdmissionStatus::Denied);
        assert_forgery_rejected(
            &fixture,
            &forge_reviewed_edge(stale_lock, &summary),
            "is not bound to the recorded Cargo.lock or provider identity",
        );
    }

    #[test]
    fn artifact_root_preparation_refuses_foreign_directories_without_deleting() {
        let temp = tempfile::tempdir().expect("tempdir");

        let fresh = temp.path().join("fresh");
        prepare_artifact_root(&fresh).expect("missing artifact directory is created");
        prepare_artifact_root(&fresh).expect("empty artifact directory is accepted");

        let foreign = temp.path().join("foreign");
        fs::create_dir_all(foreign.join(DEPENDENCY_GRAPH_ARTIFACT)).expect("create foreign directory");
        fs::write(foreign.join(DEPENDENCY_GRAPH_ARTIFACT).join("data"), "keep\n").expect("foreign nested file");
        fs::write(foreign.join(COMMAND_FILE), "keep\n").expect("foreign command file");
        fs::write(foreign.join(STATUS_FILE), "{}\n").expect("foreign status file");
        let error = prepare_artifact_root(&foreign).err().expect("foreign directory must be refused");
        assert!(error.contains("not an existing Octet admission bundle"), "{error}");
        assert!(foreign.join(DEPENDENCY_GRAPH_ARTIFACT).join("data").is_file());
        assert!(foreign.join(COMMAND_FILE).is_file());
        assert!(foreign.join(STATUS_FILE).is_file());

        let previous = temp.path().join("previous");
        let root = ArtifactRoot::open_or_create(&previous).expect("previous bundle root");
        write_blocked_status(&root, TEST_PROFILE, EXIT_DENIED, b"cargo octet admit\n").expect("previous bundle");
        fs::write(previous.join(effect_closure::RECEIPT_FILE), "{}\n").expect("stale receipt");
        prepare_artifact_root(&previous).expect("previous admission bundle is accepted");
        assert!(!previous.join(STATUS_FILE).exists());
        assert!(!previous.join(effect_closure::RECEIPT_FILE).exists());

        let interrupted = temp.path().join("interrupted");
        fs::create_dir_all(interrupted.join(CHECK_DIR)).expect("interrupted check dir");
        fs::write(interrupted.join(CHECK_STATUS_ARTIFACT), "{}\n").expect("check status marker");
        fs::write(interrupted.join(CHECK_PROVENANCE_ARTIFACT), "").expect("check provenance marker");
        fs::write(interrupted.join(COMMAND_FILE), "cargo octet admit\n").expect("interrupted command");
        prepare_artifact_root(&interrupted).expect("run stopped after its direct check is accepted");
        assert!(!interrupted.join(COMMAND_FILE).exists());
        assert!(interrupted.join(CHECK_STATUS_ARTIFACT).is_file());
    }

    #[test]
    fn verification_resolves_relative_artifact_dir_against_workspace_root() {
        let temp = tempfile::tempdir().expect("tempdir");
        let workspace = temp.path().canonicalize().expect("canonical workspace");
        fs::write(
            workspace.join("Cargo.toml"),
            "[workspace]\n\n[workspace.metadata.octet]\ndefault_scope = []\ncargo_check_args = []\n",
        )
        .expect("workspace manifest");
        let nested = workspace.join("crates/member");
        fs::create_dir_all(&nested).expect("nested directory");

        let relative = Path::new("target/octet-admission");
        assert_eq!(
            verification_artifact_dir(&nested, relative).expect("relative artifact dir"),
            workspace.join(relative)
        );
        let absolute = workspace.join("elsewhere");
        assert_eq!(verification_artifact_dir(&nested, &absolute).expect("absolute artifact dir"), absolute);
    }

    const AUTH_TEST_SEED_A: u8 = 31;
    const AUTH_TEST_SEED_B: u8 = 37;
    const AUTH_TEST_SEED_C: u8 = 41;
    const AUTH_TEST_CURRENTNESS_PROFILE: &str = "octet-key-currentness-observation.v1";
    const AUTH_TEST_PURPOSE: &str = "admission-verification";
    const AUTH_TEST_SCOPE: &str = "effect-closure-admission";
    const AUTH_TEST_PRODUCER: &str = "release";
    const AUTH_TEST_KEY_A: &str = "release-key-a";
    const AUTH_TEST_KEY_B: &str = "release-key-b";

    struct AuthTestBundle {
        dir: PathBuf,
        files: BTreeMap<String, Vec<u8>>,
        receipt: EffectClosureReceipt,
        subject: AuthArtifactRef,
    }

    fn auth_test_ref(profile: &str, seed: &str) -> AuthArtifactRef {
        AuthArtifactRef {
            profile: profile.to_owned(),
            algorithm: ALGORITHM_BLAKE3.to_owned(),
            digest_hex: blake3::hash(seed.as_bytes()).to_hex().to_string(),
        }
    }

    fn read_bundle_files(dir: &Path) -> BTreeMap<String, Vec<u8>> {
        ArtifactRoot::open_existing(dir)
            .expect("artifact root")
            .read_files_recursive()
            .expect("bundle files")
            .into_iter()
            .map(|file| (file.locator, file.contents))
            .collect()
    }

    fn auth_test_bundle(dir: &Path, source: &str) -> AuthTestBundle {
        let fixture = bundle_fixture(source, &[("root", 'a')], Vec::new(), Vec::new());
        let receipt = effect_closure::compute_closure(fixture.input.clone());
        assert_eq!(receipt.status, AdmissionStatus::Admitted);
        write_bundle(dir, &fixture, &receipt, None);
        let files = read_bundle_files(dir);
        let subject = admission_bundle_identity(&files, Some(&receipt));
        AuthTestBundle {
            dir: dir.to_path_buf(),
            files,
            receipt,
            subject,
        }
    }

    fn auth_test_key(
        seed: u8,
        key_id: &str,
        generation: u64,
        currentness: KeyCurrentness,
    ) -> (TrustedKeyInput, SigningKey) {
        let signing = SigningKey::from_bytes(&[seed; AUTH_PUBLIC_KEY_BYTES]);
        let public_key = signing.verifying_key().to_bytes();
        (
            TrustedKeyInput {
                producer_id: AUTH_TEST_PRODUCER.to_owned(),
                key_id: key_id.to_owned(),
                public_key_hex: encode_lower_hex(&public_key),
                generation,
                allowed_purposes: vec![AUTH_TEST_PURPOSE.to_owned()],
                currentness,
                currentness_ref: auth_test_ref(AUTH_TEST_CURRENTNESS_PROFILE, key_id),
                observed_at_unix: None,
            },
            signing,
        )
    }

    fn auth_test_policy(
        subject: &AuthArtifactRef,
        keys: &[&TrustedKeyInput],
        threshold: u16,
    ) -> AdmissionAuthPolicyInput {
        AdmissionAuthPolicyInput {
            schema_version: ADMISSION_AUTH_POLICY_SCHEMA.to_owned(),
            policy_id: "release-admission".to_owned(),
            profile_id: "onix-artifact-auth.v1".to_owned(),
            domain: "octet.effect-closure.admission.v1".to_owned(),
            purpose: AUTH_TEST_PURPOSE.to_owned(),
            scope_id: AUTH_TEST_SCOPE.to_owned(),
            verifier_context: auth_test_ref("octet-verifier-context.v1", "verifier-context"),
            expected_subject: subject.clone(),
            expected_receipt_identity: None,
            authorized_producers: vec![AuthorizedProducerInput {
                producer_id: AUTH_TEST_PRODUCER.to_owned(),
                key_ids: keys.iter().map(|key| key.key_id.clone()).collect(),
                purposes: vec![AUTH_TEST_PURPOSE.to_owned()],
                scope_ids: vec![AUTH_TEST_SCOPE.to_owned()],
            }],
            threshold,
            trusted_keys: keys.iter().map(|key| (*key).clone()).collect(),
            currentness_rules: CurrentnessRulesInput {
                accepted: vec![KeyCurrentness::Current, KeyCurrentness::VerificationOverlap],
                observation_profile: AUTH_TEST_CURRENTNESS_PROFILE.to_owned(),
                maximum_observation_age_seconds: None,
                evaluated_at_unix: None,
            },
            required_provenance: vec![
                ADMISSION_AUTH_RECEIPT_PROFILE.to_owned(),
                ADMISSION_AUTH_COMMAND_PROFILE.to_owned(),
                ADMISSION_AUTH_CHECK_STATUS_PROFILE.to_owned(),
            ],
        }
    }

    fn auth_test_scope(policy: &AdmissionAuthPolicyInput, bundle: &AuthTestBundle) -> AuthenticationScope {
        build_admission_auth_scope(policy, &bundle.files, Some(&bundle.receipt)).scope
    }

    fn auth_test_signed(
        signing: &SigningKey,
        scope: &AuthenticationScope,
        key_id: &str,
        generation: u64,
    ) -> SignatureEvidenceInput {
        auth_test_signed_for(AUTH_TEST_PRODUCER, signing, scope, key_id, generation)
    }

    fn auth_test_signed_for(
        producer_id: &str,
        signing: &SigningKey,
        scope: &AuthenticationScope,
        key_id: &str,
        generation: u64,
    ) -> SignatureEvidenceInput {
        let public_key = signing.verifying_key().to_bytes();
        let statement = ArtifactStatement {
            schema: STATEMENT_SCHEMA_V1.to_owned(),
            scope: scope.clone(),
            producer_id: producer_id.to_owned(),
            key_id: key_id.to_owned(),
            key_identity: public_key_identity(&public_key),
        };
        let message = artifact_auth_core::canonical_statement_bytes(&statement).expect("canonical statement");
        let signature = signing.sign(&message);
        SignatureEvidenceInput {
            statement,
            generation,
            signature_hex: encode_lower_hex(&signature.to_bytes()),
        }
    }

    fn auth_test_evidence(items: Vec<SignatureEvidenceInput>) -> AdmissionAuthEvidenceInput {
        AdmissionAuthEvidenceInput {
            schema_version: ADMISSION_AUTH_EVIDENCE_SCHEMA.to_owned(),
            evidence: items,
        }
    }

    fn write_auth_inputs(
        dir: &Path,
        policy: &AdmissionAuthPolicyInput,
        evidence: &AdmissionAuthEvidenceInput,
    ) -> (PathBuf, PathBuf) {
        fs::create_dir_all(dir).expect("consumer input dir");
        let policy_path = dir.join("auth-policy.json");
        let evidence_path = dir.join("auth-evidence.json");
        fs::write(&policy_path, serde_json::to_vec_pretty(policy).expect("policy JSON")).expect("write policy");
        fs::write(&evidence_path, serde_json::to_vec_pretty(evidence).expect("evidence JSON")).expect("write evidence");
        (policy_path, evidence_path)
    }

    fn verify_auth(
        bundle: &AuthTestBundle,
        policy: &AdmissionAuthPolicyInput,
        evidence: &AdmissionAuthEvidenceInput,
        inputs_dir: &Path,
    ) -> AdmissionVerificationReport {
        let (policy_path, evidence_path) = write_auth_inputs(inputs_dir, policy, evidence);
        verify_bundle_authorized_inner(&bundle.dir, false, &policy_path, &evidence_path)
            .expect("authorized verification report")
    }

    fn auth_issue_codes(report: &AdmissionVerificationReport) -> Vec<String> {
        report
            .auth
            .as_ref()
            .expect("auth report")
            .decision
            .issues
            .iter()
            .map(|issue| issue.code.clone())
            .collect()
    }

    // r[verify octet.effect_closure.evidence.tamper]
    #[test]
    fn authorized_verification_binds_two_distinct_keys_and_recomputes_the_decision() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let (key_a, signing_a) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let (key_b, signing_b) = auth_test_key(AUTH_TEST_SEED_B, AUTH_TEST_KEY_B, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&bundle.subject, &[&key_a, &key_b], 2);
        let scope = auth_test_scope(&policy, &bundle);
        let evidence = auth_test_evidence(vec![
            auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1),
            auth_test_signed(&signing_b, &scope, AUTH_TEST_KEY_B, 1),
        ]);
        let report = verify_auth(&bundle, &policy, &evidence, &temp.path().join("consumer"));
        assert_eq!(report.status, STATUS_VALID, "{:?}", report.diagnostics);
        let auth = report.auth.as_ref().expect("auth report");
        assert!(auth.passed, "{:?}", auth.decision.issues);
        assert_eq!(auth.integrity, STATUS_VALID);
        assert_eq!(auth.signature_authenticity, "verified");
        assert_eq!(auth.authorization, "satisfied");
        assert_eq!(auth.trust_currentness, "satisfied");
        assert_eq!(auth.checker, "passed");
        assert_eq!(auth.decision.verified_key_blake3.len(), 2);
        assert_eq!(auth.decision_receipt_blake3, auth_decision_receipt_blake3(&auth.decision));
        let serialized = serde_json::to_vec(&auth.decision).expect("decision JSON");
        let reparsed = serde_json::from_slice::<ArtifactAuthDecisionV1>(&serialized).expect("decision round trip");
        assert_eq!(auth.decision_receipt_blake3, auth_decision_receipt_blake3(&reparsed));
        assert!(auth.non_claims.iter().any(|claim| claim.as_str() == NON_CLAIM_AUTH));
        assert!(report.non_claims.iter().any(|claim| claim.as_str() == NON_CLAIM_INTEGRITY));

        let unsigned = verify_bundle_inner(&bundle.dir, false).expect("unsigned report");
        assert!(unsigned.auth.is_none());
        assert_eq!(unsigned.non_claims, vec![NON_CLAIM_INTEGRITY.to_owned()]);
        assert_eq!(unsigned.status, STATUS_VALID, "{:?}", unsigned.diagnostics);
    }

    #[test]
    fn changed_policy_scope_or_subject_is_rejected() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let (key_a, signing_a) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&bundle.subject, &[&key_a], 1);
        let scope = auth_test_scope(&policy, &bundle);
        let evidence = auth_test_evidence(vec![auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1)]);

        let mut changed_subject = policy.clone();
        changed_subject.expected_subject.digest_hex = blake3::hash(b"other-bundle").to_hex().to_string();
        let report = verify_auth(&bundle, &changed_subject, &evidence, &temp.path().join("subject"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"admission_auth.subject_mismatch".to_owned()));
        assert_eq!(report.auth.as_ref().expect("auth").authorization, "unsatisfied");

        let mut changed_purpose = policy.clone();
        changed_purpose.purpose = "release-evidence".to_owned();
        let report = verify_auth(&bundle, &changed_purpose, &evidence, &temp.path().join("purpose"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"auth.statement_scope_mismatch".to_owned()));

        let mut wrong_purpose_scope = scope.clone();
        wrong_purpose_scope.purpose = "release-evidence".to_owned();
        let wrong_purpose_evidence =
            auth_test_evidence(vec![auth_test_signed(&signing_a, &wrong_purpose_scope, AUTH_TEST_KEY_A, 1)]);
        let report = verify_auth(&bundle, &policy, &wrong_purpose_evidence, &temp.path().join("wrong-purpose"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"auth.statement_scope_mismatch".to_owned()));

        let mut changed_profile = policy.clone();
        changed_profile.profile_id = "other-profile".to_owned();
        let report = verify_auth(&bundle, &changed_profile, &evidence, &temp.path().join("profile"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"auth.statement_scope_mismatch".to_owned()));

        let mut changed_scope_id = policy.clone();
        changed_scope_id.scope_id = "other-scope".to_owned();
        let report = verify_auth(&bundle, &changed_scope_id, &evidence, &temp.path().join("scope"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"admission_auth.producer_unauthorized".to_owned()));
        assert!(auth_issue_codes(&report).contains(&"auth.statement_scope_mismatch".to_owned()));

        let mut changed_policy_id = policy;
        changed_policy_id.policy_id = "other-policy".to_owned();
        let report = verify_auth(&bundle, &changed_policy_id, &evidence, &temp.path().join("policy-id"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"auth.statement_scope_mismatch".to_owned()));
    }

    #[test]
    fn producer_authorization_is_required_in_addition_to_signature_validity() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let (key_a, signing_a) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&bundle.subject, &[&key_a], 1);
        let scope = auth_test_scope(&policy, &bundle);
        let evidence = auth_test_evidence(vec![auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1)]);

        let mut unauthorized = policy.clone();
        unauthorized.authorized_producers[0].purposes = vec!["release-evidence".to_owned()];
        let report = verify_auth(&bundle, &unauthorized, &evidence, &temp.path().join("purpose"));
        let auth = report.auth.as_ref().expect("auth");
        assert!(!auth.passed);
        assert_eq!(auth.signature_authenticity, "verified");
        assert_eq!(auth.authorization, "unsatisfied");
        assert!(auth_issue_codes(&report).contains(&"admission_auth.producer_unauthorized".to_owned()));

        let mut unknown_producer = policy.clone();
        unknown_producer.authorized_producers[0].producer_id = "other-producer".to_owned();
        let report = verify_auth(&bundle, &unknown_producer, &evidence, &temp.path().join("producer"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"admission_auth.producer_unauthorized".to_owned()));

        let (_, intruder_signing) = auth_test_key(AUTH_TEST_SEED_C, "intruder-key", 1, KeyCurrentness::Current);
        let intruder = auth_test_evidence(vec![auth_test_signed_for(
            "intruder",
            &intruder_signing,
            &scope,
            "intruder-key",
            1,
        )]);
        let report = verify_auth(&bundle, &policy, &intruder, &temp.path().join("intruder"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"admission_auth.producer_unauthorized".to_owned()));
    }

    #[test]
    fn generation_threshold_and_duplicate_evidence_are_enforced() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let (key_a, signing_a) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&bundle.subject, &[&key_a], 1);
        let scope = auth_test_scope(&policy, &bundle);
        let evidence = auth_test_evidence(vec![auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1)]);

        let mut stale_generation = policy.clone();
        stale_generation.trusted_keys[0].generation = 2;
        let report = verify_auth(&bundle, &stale_generation, &evidence, &temp.path().join("generation"));
        assert!(auth_issue_codes(&report).contains(&"auth.key_generation_stale".to_owned()));
        assert_eq!(report.auth.as_ref().expect("auth").trust_currentness, "unsatisfied");

        let mut two_keys = policy.clone();
        two_keys.threshold = 2;
        let (key_b, _) = auth_test_key(AUTH_TEST_SEED_B, AUTH_TEST_KEY_B, 1, KeyCurrentness::Current);
        two_keys.trusted_keys.push(key_b);
        two_keys.authorized_producers[0].key_ids.push(AUTH_TEST_KEY_B.to_owned());
        let report = verify_auth(&bundle, &two_keys, &evidence, &temp.path().join("threshold"));
        assert!(auth_issue_codes(&report).contains(&"auth.distinct_key_threshold_unsatisfied".to_owned()));
        assert_eq!(report.auth.as_ref().expect("auth").trust_currentness, "unsatisfied");

        let duplicate = auth_test_evidence(vec![
            auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1),
            auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1),
        ]);
        let report = verify_auth(&bundle, &policy, &duplicate, &temp.path().join("duplicate"));
        assert!(auth_issue_codes(&report).contains(&"admission_auth.evidence_key_duplicate".to_owned()));
    }

    #[test]
    fn altered_bytes_and_coherent_whole_bundle_replacement_are_rejected() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let (key_a, signing_a) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&bundle.subject, &[&key_a], 1);
        let scope = auth_test_scope(&policy, &bundle);
        let evidence = auth_test_evidence(vec![auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1)]);
        let (policy_path, evidence_path) = write_auth_inputs(&temp.path().join("consumer"), &policy, &evidence);

        fs::write(bundle.dir.join(SUMMARY_FILE), b"tampered\n").expect("tamper summary");
        let report = verify_bundle_authorized_inner(&bundle.dir, false, &policy_path, &evidence_path)
            .expect("authorized verification report");
        let auth = report.auth.as_ref().expect("auth");
        assert!(!auth.passed);
        assert_eq!(auth.integrity, STATUS_INVALID);
        assert!(auth_issue_codes(&report).contains(&"admission_auth.subject_mismatch".to_owned()));

        let pristine = auth_test_bundle(&temp.path().join("pristine"), "fn root() {}\n");
        let replacement = auth_test_bundle(&temp.path().join("replacement"), "fn root() { let _value = 1; }\n");
        assert_ne!(replacement.subject.digest_hex, pristine.subject.digest_hex);
        for (locator, contents) in &replacement.files {
            let path = pristine.dir.join(locator);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("replacement parent");
            }
            fs::write(path, contents).expect("replace bundle file");
        }
        let report = verify_bundle_authorized_inner(&pristine.dir, false, &policy_path, &evidence_path)
            .expect("authorized verification report");
        let auth = report.auth.as_ref().expect("auth");
        assert!(!auth.passed);
        assert!(auth_issue_codes(&report).contains(&"admission_auth.subject_mismatch".to_owned()));
        assert!(auth_issue_codes(&report).contains(&"auth.statement_scope_mismatch".to_owned()));
    }

    #[test]
    fn malformed_and_unsupported_evidence_is_rejected() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let (key_a, signing_a) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&bundle.subject, &[&key_a], 1);
        let scope = auth_test_scope(&policy, &bundle);
        let valid = auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1);

        let mut malformed = valid.clone();
        malformed.signature_hex = "zz".to_owned();
        let report =
            verify_auth(&bundle, &policy, &auth_test_evidence(vec![malformed]), &temp.path().join("malformed"));
        let auth = report.auth.as_ref().expect("auth");
        assert!(!auth.passed);
        assert_eq!(auth.signature_authenticity, "unsupported");
        assert!(auth_issue_codes(&report).contains(&"admission_auth.signature_malformed".to_owned()));

        let mut wrong_signature = valid.clone();
        wrong_signature.signature_hex = "00".repeat(AUTH_SIGNATURE_BYTES);
        let report = verify_auth(
            &bundle,
            &policy,
            &auth_test_evidence(vec![wrong_signature]),
            &temp.path().join("wrong-signature"),
        );
        let auth = report.auth.as_ref().expect("auth");
        assert!(!auth.passed);
        assert_eq!(auth.signature_authenticity, "unverified");
        assert_eq!(auth.checker, "denied");

        let mut unsupported = auth_test_evidence(vec![valid.clone()]);
        unsupported.schema_version = "octet-admission-auth-evidence/v2".to_owned();
        let report = verify_auth(&bundle, &policy, &unsupported, &temp.path().join("unsupported"));
        let auth = report.auth.as_ref().expect("auth");
        assert!(!auth.passed);
        assert_eq!(auth.signature_authenticity, "unsupported");
        assert!(auth_issue_codes(&report).contains(&"admission_auth.evidence_schema_unsupported".to_owned()));

        let (untrusted_key, untrusted_signing) =
            auth_test_key(AUTH_TEST_SEED_C, "unknown-key", 1, KeyCurrentness::Current);
        let mut permissive = policy.clone();
        permissive.authorized_producers[0].key_ids.push(untrusted_key.key_id.clone());
        let untrusted =
            auth_test_evidence(vec![auth_test_signed(&untrusted_signing, &scope, &untrusted_key.key_id, 1)]);
        let report = verify_auth(&bundle, &permissive, &untrusted, &temp.path().join("untrusted"));
        let auth = report.auth.as_ref().expect("auth");
        assert!(!auth.passed);
        assert_eq!(auth.trust_currentness, "unsatisfied");
        assert!(auth_issue_codes(&report).contains(&"admission_auth.evidence_key_untrusted".to_owned()));
    }

    #[test]
    fn revoked_superseded_unknown_and_stale_currentness_is_rejected() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let (key_a, signing_a) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&bundle.subject, &[&key_a], 1);
        let scope = auth_test_scope(&policy, &bundle);
        let evidence = auth_test_evidence(vec![auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1)]);

        for (index, currentness) in [
            KeyCurrentness::Revoked,
            KeyCurrentness::Superseded,
            KeyCurrentness::Unknown,
        ]
        .into_iter()
        .enumerate()
        {
            let mut rejected = policy.clone();
            rejected.trusted_keys[0].currentness = currentness;
            rejected.currentness_rules.accepted = vec![currentness];
            let report = verify_auth(&bundle, &rejected, &evidence, &temp.path().join(format!("currentness-{index}")));
            let auth = report.auth.as_ref().expect("auth");
            assert!(!auth.passed, "{currentness:?}");
            assert_eq!(auth.trust_currentness, "unsatisfied", "{currentness:?}");
            assert!(
                auth_issue_codes(&report).contains(&"auth.key_not_current".to_owned()),
                "{currentness:?}: {:?}",
                auth_issue_codes(&report)
            );
        }

        let mut unacceptable = policy.clone();
        unacceptable.currentness_rules.accepted = vec![KeyCurrentness::VerificationOverlap];
        let report = verify_auth(&bundle, &unacceptable, &evidence, &temp.path().join("unacceptable"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"admission_auth.currentness_unacceptable".to_owned()));

        let mut stale = policy.clone();
        stale.currentness_rules.maximum_observation_age_seconds = Some(60);
        stale.currentness_rules.evaluated_at_unix = Some(1_000);
        stale.trusted_keys[0].observed_at_unix = Some(900);
        let report = verify_auth(&bundle, &stale, &evidence, &temp.path().join("stale"));
        assert!(!report.auth.as_ref().expect("auth").passed);
        assert!(auth_issue_codes(&report).contains(&"admission_auth.currentness_unacceptable".to_owned()));

        stale.trusted_keys[0].observed_at_unix = Some(950);
        let report = verify_auth(&bundle, &stale, &evidence, &temp.path().join("fresh"));
        assert!(report.auth.as_ref().expect("auth").passed, "{:?}", auth_issue_codes(&report));
    }

    #[test]
    fn candidate_files_cannot_supply_trust_roots_or_observations() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let forged_policy = auth_test_ref("octet-verifier-context.v1", "forged");
        fs::write(
            bundle.dir.join("auth-policy.json"),
            serde_json::to_vec_pretty(&forged_policy).expect("forged policy"),
        )
        .expect("write forged policy into bundle");
        fs::write(bundle.dir.join("auth-evidence.json"), b"{\"verified\":true}\n")
            .expect("write forged evidence into bundle");
        let files = read_bundle_files(&bundle.dir);
        let subject = admission_bundle_identity(&files, Some(&bundle.receipt));
        let (key_a, signing_a) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&subject, &[&key_a], 1);
        let scope = build_admission_auth_scope(&policy, &files, Some(&bundle.receipt)).scope;
        let evidence = auth_test_evidence(vec![auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1)]);
        let consumer = temp.path().join("consumer");
        let (policy_path, evidence_path) = write_auth_inputs(&consumer, &policy, &evidence);

        let report = verify_bundle_authorized_inner(&bundle.dir, false, &policy_path, &evidence_path)
            .expect("external consumer inputs still verify");
        assert!(report.auth.as_ref().expect("auth").passed, "{:?}", report.diagnostics);

        let error =
            verify_bundle_authorized_inner(&bundle.dir, false, &bundle.dir.join("auth-policy.json"), &evidence_path)
                .expect_err("candidate-supplied policy must be rejected");
        assert!(error.contains("outside the candidate artifact directory"), "{error}");

        let error =
            verify_bundle_authorized_inner(&bundle.dir, false, &policy_path, &bundle.dir.join("auth-evidence.json"))
                .expect_err("candidate-supplied evidence must be rejected");
        assert!(error.contains("outside the candidate artifact directory"), "{error}");
    }

    #[test]
    fn receipt_identity_and_required_provenance_are_enforced() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let (key_a, signing_a) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&bundle.subject, &[&key_a], 1);
        let scope = auth_test_scope(&policy, &bundle);
        let evidence = auth_test_evidence(vec![auth_test_signed(&signing_a, &scope, AUTH_TEST_KEY_A, 1)]);

        let mut wrong_receipt = policy.clone();
        wrong_receipt.expected_receipt_identity = Some(format!("{AUTH_BLAKE3_PREFIX}{}", "0".repeat(AUTH_HEX_CHARS)));
        let report = verify_auth(&bundle, &wrong_receipt, &evidence, &temp.path().join("receipt"));
        let auth = report.auth.as_ref().expect("auth");
        assert!(!auth.passed);
        assert_eq!(auth.authorization, "unsatisfied");
        assert!(auth_issue_codes(&report).contains(&"admission_auth.receipt_identity_mismatch".to_owned()));

        let mut missing_provenance = policy.clone();
        missing_provenance.required_provenance.push("octet-effect-closure-proof.v1".to_owned());
        let report = verify_auth(&bundle, &missing_provenance, &evidence, &temp.path().join("provenance"));
        let auth = report.auth.as_ref().expect("auth");
        assert!(!auth.passed);
        assert_eq!(auth.authorization, "unsatisfied");
        assert!(auth_issue_codes(&report).contains(&"admission_auth.provenance_missing".to_owned()));
    }

    #[test]
    fn admission_auth_input_files_reject_unknown_fields_and_invalid_keys() {
        let temp = tempfile::tempdir().expect("tempdir");
        let bundle = auth_test_bundle(&temp.path().join("bundle"), "fn root() {}\n");
        let (key_a, _) = auth_test_key(AUTH_TEST_SEED_A, AUTH_TEST_KEY_A, 1, KeyCurrentness::Current);
        let policy = auth_test_policy(&bundle.subject, &[&key_a], 1);
        let consumer = temp.path().join("consumer");
        fs::create_dir_all(&consumer).expect("consumer dir");
        let mut value = serde_json::to_value(&policy).expect("policy value");
        value["unknown_field"] = serde_json::Value::Bool(true);
        let path = consumer.join("unknown.json");
        fs::write(&path, serde_json::to_vec(&value).expect("policy JSON")).expect("write policy");
        assert!(load_admission_auth_policy(&path).is_err());

        let mut invalid_key = policy.clone();
        invalid_key.trusted_keys[0].public_key_hex = "00".to_owned();
        assert!(validate_admission_auth_policy(&invalid_key).is_err());

        let mut zero_threshold = policy;
        zero_threshold.threshold = 0;
        assert!(validate_admission_auth_policy(&zero_threshold).is_err());
    }
}
