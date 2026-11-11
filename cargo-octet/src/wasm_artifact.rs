use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use wasmparser::CanonicalFunction;
use wasmparser::Encoding;
use wasmparser::Operator;
use wasmparser::Parser;
use wasmparser::Payload;
use wasmparser::TypeRef;
use wasmparser::ValType;
use wasmparser::Validator;
use wasmparser::WasmFeatures;

pub(crate) const REGISTRY_SCHEMA_VERSION: &str = "octet-wasm-artifact-profiles/v1";
pub(crate) const ENVELOPE_SCHEMA_VERSION: &str = "octet-wasm-artifact-analysis/v1";
pub(crate) const INPUT_ARTIFACT_NAME: &str = "wasm-artifact.bin";
pub(crate) const RECORDED_ONLY_ROLE: &str = "recorded_only";
pub(crate) const REQUIRED_NON_CLAIM: &str =
    "Wasm artifact evidence records bounded static facts and does not prove execution correctness";

const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const GIT_REVISION_HEX_LENGTH: usize = 40;
const COHORT_IDENTITY_DOMAIN: &[u8] = b"octet-wasm-tools-cohort/v1\0";
/// wasm-tools cohort members this build links, pinned with `=` in `cargo-octet/Cargo.toml`:
/// `wasmparser` validates and extracts artifact facts; `wasm-encoder` builds the conformance
/// fixtures. A registry cohort must claim exactly these versions.
/// `linked_cohort_versions_match_cargo_lock` keeps them in sync.
const LINKED_COHORT_MEMBERS: [(&str, &str); 2] = [("wasmparser", "0.253.0"), ("wasm-encoder", "0.253.0")];
const INTERFACE_IDENTITY_DOMAIN: &[u8] = b"octet-wasm-interface/v2\0";
const RUNTIME_COHORT_IDENTITY_DOMAIN: &[u8] = b"octet-wasm-runtime-cohort/v1\0";
const MALFORMED_ISSUE: &str = "malformed-artifact";
const DISALLOWED_KIND_ISSUE: &str = "disallowed-artifact-kind";
const DISALLOWED_PROPOSAL_ISSUE: &str = "disallowed-proposal";
const DISALLOWED_IMPORT_ISSUE: &str = "disallowed-import";
const DISALLOWED_EXPORT_ISSUE: &str = "disallowed-export";
const DISALLOWED_WORLD_ISSUE: &str = "disallowed-world";
const PRODUCER_METADATA_ISSUE: &str = "producer-metadata";
const ABI_MISMATCH_ISSUE: &str = "abi-mismatch";
const MEMORY_LIMIT_ISSUE: &str = "memory-limit";
const TABLE_LIMIT_ISSUE: &str = "table-limit";
const STRUCTURAL_LIMIT_ISSUE: &str = "structural-limit";
const EXTERNAL_EVIDENCE_ISSUE: &str = "external-evidence-mismatch";
const RESOURCE_LIMIT_ISSUE: &str = "resource-limit";
const MEMORY_GROWTH_ISSUE: &str = "memory-growth";
const START_FUNCTION_ISSUE: &str = "start-function";
const DUPLICATE_METADATA_ISSUE: &str = "duplicate-custom-section";
const ERROR_SEVERITY: &str = "error";
const MODULE_KIND: &str = "module";
const COMPONENT_KIND: &str = "component";
const UNKNOWN_KIND: &str = "unknown";
const WASM1_PROPOSAL: &str = "wasm1";
const WASM2_PROPOSAL: &str = "wasm2";
const WASM3_PROPOSAL: &str = "wasm3";
const COMPONENT_MODEL_PROPOSAL: &str = "component-model";
const BULK_MEMORY_PROPOSAL: &str = "bulk-memory";
const CALL_INDIRECT_OVERLONG_PROPOSAL: &str = "call-indirect-overlong";
const MUTABLE_GLOBALS_PROPOSAL: &str = "mutable-globals";
const PRODUCERS_SECTION: &str = "producers";
const COMPONENT_WORLD_SEPARATOR: char = '/';
const STANDALONE_HANDOFF: &str = "standalone";
const MANTLE_HANDOFF: &str = "mantle-exact-byte";
const EXTERNAL_HANDOFF: &str = "external-orchestrator";
const MANTLE_PARENT_PREFIX: &str = "mantle:";
const CHAOSCONTROL_PARENT_PREFIX: &str = "chaoscontrol:";
const MOLTEN_PARENT_PREFIX: &str = "molten:";
const SPACEWASM_RUNTIME_ID: &str = "nasa-spacewasm";
const SPACEWASM_SUPPORT_SCHEMA: &str = "octet-spacewasm-support-projection/v1";
const EXTERNAL_RUNTIME_SCHEMA: &str = "octet-spacewasm-external-runtime-report/v1";
const EXTERNAL_RUNTIME_NON_CLAIM: &str = "Runtime observations do not change Octet static profile conformance";
const WIDE_VALUE_WORDS: u64 = 2;
const SIMD_VALUE_WORDS: u64 = 4;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WasmProfileRegistry {
    pub(crate) schema_version: String,
    pub(crate) cohort: WasmToolsCohort,
    pub(crate) profiles: Vec<WasmArtifactProfile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WasmToolsCohort {
    pub(crate) cohort_id: String,
    pub(crate) identity: String,
    pub(crate) members: Vec<WasmToolsCohortMember>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WasmToolsCohortMember {
    pub(crate) crate_name: String,
    pub(crate) version: String,
    pub(crate) role: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WasmArtifactProfile {
    pub(crate) profile_id: String,
    pub(crate) summary: String,
    pub(crate) allowed_artifact_kinds: Vec<String>,
    pub(crate) allowed_proposals: Vec<String>,
    pub(crate) allowed_worlds: Vec<String>,
    pub(crate) allowed_imports: Vec<String>,
    pub(crate) allowed_exports: Vec<String>,
    pub(crate) required_exports: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) required_imports: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) component_interface: Option<String>,
    pub(crate) allow_any_exports: bool,
    pub(crate) resource_limits: WasmResourceLimits,
    pub(crate) allow_memory_grow: bool,
    pub(crate) allow_start: bool,
    pub(crate) duplicate_custom_sections: String,
    pub(crate) producer_metadata: String,
    pub(crate) severity: WasmFindingSeverities,
    pub(crate) verification_role: String,
    pub(crate) non_claims: Vec<String>,
    #[serde(default)]
    pub(crate) target_runtime: Option<TargetRuntimeCohort>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TargetRuntimeCohort {
    pub(crate) cohort_identity: String,
    pub(crate) support_projection: RuntimeSupportProjection,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RuntimeSupportProjection {
    pub(crate) schema_version: String,
    pub(crate) runtime_id: String,
    pub(crate) source_repository: String,
    pub(crate) source_revision: String,
    pub(crate) source_lock_identity: String,
    pub(crate) reviewed_paths: Vec<String>,
    pub(crate) supported_proposals: Vec<String>,
    pub(crate) denied_or_unimplemented_proposals: Vec<String>,
    pub(crate) module_kind: String,
    pub(crate) default_allow_memory_grow: bool,
    pub(crate) maximum_linear_memory_pages: u64,
    pub(crate) maximum_function_parameter_words: u64,
    pub(crate) maximum_local_words_per_function: u64,
    pub(crate) maximum_function_index: u64,
    pub(crate) external_runtime_configuration: Vec<String>,
    pub(crate) observed_discrepancies: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WasmResourceLimits {
    #[serde(rename = "max_artifact_bytes")]
    pub(crate) artifact_bytes: u64,
    #[serde(rename = "max_imports")]
    pub(crate) imports: u64,
    #[serde(rename = "max_exports")]
    pub(crate) exports: u64,
    #[serde(rename = "max_memories")]
    pub(crate) memories: u64,
    #[serde(rename = "max_tables")]
    pub(crate) tables: u64,
    #[serde(rename = "max_functions")]
    pub(crate) functions: u64,
    #[serde(rename = "max_memory_initial_pages")]
    pub(crate) memory_initial_pages: u64,
    #[serde(rename = "max_memory_maximum_pages")]
    pub(crate) memory_maximum_pages: u64,
    #[serde(rename = "max_table_initial_elements")]
    pub(crate) table_initial_elements: u64,
    #[serde(rename = "max_table_maximum_elements")]
    pub(crate) table_maximum_elements: u64,
    #[serde(rename = "max_function_parameter_words")]
    pub(crate) function_parameter_words: u64,
    #[serde(rename = "max_local_words_per_function")]
    pub(crate) local_words_per_function: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WasmFindingSeverities {
    pub(crate) malformed: String,
    pub(crate) disallowed_kind: String,
    pub(crate) disallowed_import: String,
    pub(crate) disallowed_world: String,
    pub(crate) resource_limit: String,
    pub(crate) growth: String,
    pub(crate) start: String,
    pub(crate) duplicate_metadata: String,
    pub(crate) abi_mismatch: String,
    pub(crate) external_evidence: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(crate) struct WasmInterfaceItem {
    pub(crate) namespace: String,
    pub(crate) name: String,
    pub(crate) kind: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(crate) struct WasmMemoryDeclaration {
    pub(crate) initial_pages: u64,
    pub(crate) maximum_pages: Option<u64>,
    pub(crate) memory64: bool,
    pub(crate) shared: bool,
    pub(crate) page_size_log2: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(crate) struct WasmTableDeclaration {
    pub(crate) initial_elements: u64,
    pub(crate) maximum_elements: Option<u64>,
    pub(crate) table64: bool,
    pub(crate) shared: bool,
    pub(crate) element_type: String,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactFacts {
    pub(crate) artifact_kind: String,
    pub(crate) proposals: Vec<String>,
    pub(crate) imports: Vec<WasmInterfaceItem>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) bound_imports: Vec<WasmInterfaceItem>,
    pub(crate) exports: Vec<WasmInterfaceItem>,
    pub(crate) worlds: Vec<String>,
    pub(crate) memory_count: u64,
    pub(crate) table_count: u64,
    pub(crate) function_count: u64,
    pub(crate) memories: Vec<WasmMemoryDeclaration>,
    pub(crate) tables: Vec<WasmTableDeclaration>,
    pub(crate) max_function_parameter_words: u64,
    pub(crate) max_local_words_per_function: u64,
    pub(crate) memory_grow_count: u64,
    pub(crate) has_start: bool,
    pub(crate) custom_sections: Vec<String>,
    pub(crate) producer_sections: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) function_abis: Vec<crate::spacewasm::FunctionAbi>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactFinding {
    pub(crate) issue_code: String,
    pub(crate) severity: String,
    pub(crate) message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) byte_offset: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactReceiptDetails {
    pub(crate) profile_id: String,
    pub(crate) profile_identity: String,
    pub(crate) registry_identity: String,
    pub(crate) cohort_id: String,
    pub(crate) cohort_identity: String,
    pub(crate) exact_artifact_identity: String,
    pub(crate) interface_identity: String,
    pub(crate) exact_identity_role: String,
    pub(crate) interface_identity_role: String,
    pub(crate) input_artifact: String,
    pub(crate) handoff_role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) handoff_parent_identities: Vec<String>,
    pub(crate) verification_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) target_runtime: Option<TargetRuntimeEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub(crate) external_evidence: Vec<ExternalRuntimeReport>,
    pub(crate) facts: WasmArtifactFacts,
    pub(crate) findings: Vec<WasmArtifactFinding>,
    pub(crate) sarif: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct TargetRuntimeEvidence {
    pub(crate) runtime_id: String,
    pub(crate) source_repository: String,
    pub(crate) source_revision: String,
    pub(crate) source_lock_identity: String,
    pub(crate) cohort_identity: String,
    pub(crate) support_projection_identity: String,
    pub(crate) verification_role: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ExternalRuntimeReport {
    pub(crate) schema_version: String,
    pub(crate) producer: String,
    pub(crate) artifact_identity: String,
    pub(crate) runtime_cohort_identity: String,
    pub(crate) profile_identity: String,
    pub(crate) result: String,
    pub(crate) verification_role: String,
    pub(crate) non_claims: Vec<String>,
    pub(crate) report_identity: String,
}

#[derive(Serialize)]
struct ExternalRuntimeReportIdentity<'a> {
    schema_version: &'a str,
    producer: &'a str,
    artifact_identity: &'a str,
    runtime_cohort_identity: &'a str,
    profile_identity: &'a str,
    result: &'a str,
    verification_role: &'a str,
    non_claims: &'a [String],
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactEnvelope {
    pub(crate) schema_version: String,
    pub(crate) details: WasmArtifactReceiptDetails,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WasmEvaluationStatus {
    Passed,
    ManualReview,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WasmEnvelopeEvaluation {
    pub(crate) details: Option<WasmArtifactReceiptDetails>,
    pub(crate) non_claims: Vec<String>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) status: WasmEvaluationStatus,
}

#[derive(Default)]
struct FactExtraction {
    facts: WasmArtifactFacts,
    parse_diagnostics: Vec<ParseDiagnostic>,
    imported_functions: u64,
    imported_memories: usize,
    imported_tables: usize,
}

// A binary scope is not an instance. Keep its declarations until all local
// instantiations are known, then count definitions per instance, not aliases.
#[derive(Default)]
struct BinaryScope {
    extraction: FactExtraction,
    modules: Vec<Option<usize>>,
    components: Vec<Option<usize>>,
    instances: u64,
    escapes: bool,
    parent: Option<usize>,
}

struct ParseDiagnostic {
    issue_code: &'static str,
    message: String,
    byte_offset: Option<u64>,
}

pub(crate) fn parse_registry(text: &str) -> Result<WasmProfileRegistry, String> {
    serde_json::from_str(text).map_err(|error| format!("parse Wasm artifact profile registry: {error}"))
}

pub(crate) fn validate_registry(registry: &WasmProfileRegistry) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if registry.schema_version != REGISTRY_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported Wasm profile registry schema `{}`; expected `{REGISTRY_SCHEMA_VERSION}`",
            registry.schema_version
        ));
    }
    let computed_cohort = cohort_identity(&registry.cohort.members);
    if registry.cohort.identity != computed_cohort {
        diagnostics.push(format!(
            "stale wasm-tools cohort identity; expected `{}`, observed `{computed_cohort}`",
            registry.cohort.identity
        ));
    }
    if registry.cohort.cohort_id.trim().is_empty() {
        diagnostics.push("wasm-tools cohort_id must not be empty".to_owned());
    }
    validate_cohort_members(&registry.cohort.members, &mut diagnostics);
    validate_linked_cohort_members(&registry.cohort.members, &mut diagnostics);
    validate_profiles(&registry.profiles, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn cohort_identity(members: &[WasmToolsCohortMember]) -> String {
    let mut sorted = members.to_vec();
    sorted.sort();
    let mut hasher = blake3::Hasher::new();
    hasher.update(COHORT_IDENTITY_DOMAIN);
    for member in sorted {
        update_hash_field(&mut hasher, member.crate_name.as_bytes());
        update_hash_field(&mut hasher, member.version.as_bytes());
        update_hash_field(&mut hasher, member.role.as_bytes());
    }
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

pub(crate) fn registry_identity(bytes: &[u8]) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
}

fn support_projection_identity(projection: &RuntimeSupportProjection) -> Result<String, Vec<String>> {
    let bytes = serde_json::to_vec(projection)
        .map_err(|error| vec![format!("serialize runtime support projection identity: {error}")])?;
    Ok(identity(&bytes))
}

fn runtime_cohort_identity(projection: &RuntimeSupportProjection) -> Result<String, Vec<String>> {
    let support_identity = support_projection_identity(projection)?;
    let mut hasher = blake3::Hasher::new();
    hasher.update(RUNTIME_COHORT_IDENTITY_DOMAIN);
    update_hash_field(&mut hasher, projection.runtime_id.as_bytes());
    update_hash_field(&mut hasher, projection.source_repository.as_bytes());
    update_hash_field(&mut hasher, projection.source_revision.as_bytes());
    update_hash_field(&mut hasher, support_identity.as_bytes());
    Ok(format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex()))
}

fn target_runtime_evidence(target: &TargetRuntimeCohort) -> Result<TargetRuntimeEvidence, Vec<String>> {
    Ok(TargetRuntimeEvidence {
        runtime_id: target.support_projection.runtime_id.clone(),
        source_repository: target.support_projection.source_repository.clone(),
        source_revision: target.support_projection.source_revision.clone(),
        source_lock_identity: target.support_projection.source_lock_identity.clone(),
        cohort_identity: target.cohort_identity.clone(),
        support_projection_identity: support_projection_identity(&target.support_projection)?,
        verification_role: "recorded_only".to_owned(),
    })
}

pub(crate) fn analyze_bytes(
    bytes: &[u8],
    registry: &WasmProfileRegistry,
    profile_id: &str,
    registry_identity: String,
    parent_artifacts: &[String],
) -> Result<WasmArtifactEnvelope, Vec<String>> {
    analyze_bytes_with_external(bytes, registry, profile_id, registry_identity, parent_artifacts, &[])
}

pub(crate) fn analyze_bytes_with_external(
    bytes: &[u8],
    registry: &WasmProfileRegistry,
    profile_id: &str,
    registry_identity: String,
    parent_artifacts: &[String],
    external_report_texts: &[String],
) -> Result<WasmArtifactEnvelope, Vec<String>> {
    let diagnostics = validate_registry(registry);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let profile = registry
        .profiles
        .iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| vec![format!("unknown Wasm artifact profile `{profile_id}`")])?;
    let exact_artifact_identity = identity(bytes);
    let profile_identity = profile_identity(profile)?;
    let target_runtime = profile.target_runtime.as_ref().map(target_runtime_evidence).transpose()?;
    let extraction = extract_facts(bytes, profile);
    let interface_identity = interface_identity(&extraction.facts);
    let mut findings = extraction
        .parse_diagnostics
        .into_iter()
        .map(|diagnostic| WasmArtifactFinding {
            issue_code: diagnostic.issue_code.to_owned(),
            severity: parse_issue_severity(&profile.severity, diagnostic.issue_code).to_owned(),
            message: diagnostic.message,
            byte_offset: diagnostic.byte_offset,
        })
        .collect::<Vec<_>>();
    findings.extend(evaluate_policy(bytes, &extraction.facts, profile));
    let (external_evidence, external_findings) = evaluate_external_reports(
        external_report_texts,
        &exact_artifact_identity,
        &profile_identity,
        target_runtime.as_ref(),
        &profile.severity.external_evidence,
    );
    findings.extend(external_findings);
    findings.sort_by(|left, right| {
        left.issue_code
            .cmp(&right.issue_code)
            .then(left.byte_offset.cmp(&right.byte_offset))
            .then(left.message.cmp(&right.message))
    });
    let sarif = render_sarif(&findings);
    Ok(WasmArtifactEnvelope {
        schema_version: ENVELOPE_SCHEMA_VERSION.to_owned(),
        details: WasmArtifactReceiptDetails {
            profile_id: profile.profile_id.clone(),
            profile_identity,
            registry_identity,
            cohort_id: registry.cohort.cohort_id.clone(),
            cohort_identity: registry.cohort.identity.clone(),
            exact_artifact_identity,
            interface_identity,
            exact_identity_role: "exact-bytes".to_owned(),
            interface_identity_role: "normalized-interface-non-equivalence".to_owned(),
            input_artifact: INPUT_ARTIFACT_NAME.to_owned(),
            handoff_role: handoff_role(parent_artifacts).to_owned(),
            handoff_parent_identities: parent_artifacts
                .iter()
                .filter(|parent| is_external_handoff_identity(parent))
                .cloned()
                .collect(),
            verification_role: profile.verification_role.clone(),
            target_runtime,
            external_evidence,
            facts: extraction.facts,
            findings,
            sarif,
        },
        non_claims: profile.non_claims.clone(),
    })
}

// Parse-stage diagnostics share issue codes with policy findings, so each takes
// the severity the profile declares for its issue class.
fn parse_issue_severity<'a>(severities: &'a WasmFindingSeverities, issue_code: &str) -> &'a str {
    match issue_code {
        DISALLOWED_PROPOSAL_ISSUE => &severities.disallowed_kind,
        RESOURCE_LIMIT_ISSUE => &severities.resource_limit,
        ABI_MISMATCH_ISSUE => &severities.abi_mismatch,
        _ => &severities.malformed,
    }
}

pub(crate) fn render_envelope(envelope: &WasmArtifactEnvelope) -> Result<String, String> {
    serde_json::to_string(envelope).map_err(|error| format!("serialize Wasm artifact envelope: {error}"))
}

pub(crate) fn evaluate_envelope(text: &str) -> WasmEnvelopeEvaluation {
    let envelope: WasmArtifactEnvelope = match serde_json::from_str(text) {
        Ok(envelope) => envelope,
        Err(error) => {
            return WasmEnvelopeEvaluation {
                details: None,
                non_claims: Vec::new(),
                diagnostics: vec![format!("parse Wasm artifact analysis envelope: {error}")],
                status: WasmEvaluationStatus::Failed,
            };
        }
    };
    let mut diagnostics = Vec::new();
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported Wasm artifact envelope schema `{}`; expected `{ENVELOPE_SCHEMA_VERSION}`",
            envelope.schema_version
        ));
    }
    validate_receipt_details(&envelope.details, &envelope.non_claims, &mut diagnostics);
    let status = evaluation_status(!diagnostics.is_empty(), &envelope.details.findings);
    diagnostics.sort();
    diagnostics.dedup();
    WasmEnvelopeEvaluation {
        details: Some(envelope.details),
        non_claims: envelope.non_claims,
        diagnostics,
        status,
    }
}

/// The status an analysis earns: diagnostics or any error finding fail it,
/// other findings need review, and no findings pass. Receipt verification
/// recomputes it from the recorded findings.
pub(crate) fn evaluation_status(has_diagnostics: bool, findings: &[WasmArtifactFinding]) -> WasmEvaluationStatus {
    if has_diagnostics || findings.iter().any(|finding| finding.severity == ERROR_SEVERITY) {
        WasmEvaluationStatus::Failed
    } else if findings.is_empty() {
        WasmEvaluationStatus::Passed
    } else {
        WasmEvaluationStatus::ManualReview
    }
}

fn external_report_identity(report: &ExternalRuntimeReport) -> Result<String, String> {
    let material = ExternalRuntimeReportIdentity {
        schema_version: &report.schema_version,
        producer: &report.producer,
        artifact_identity: &report.artifact_identity,
        runtime_cohort_identity: &report.runtime_cohort_identity,
        profile_identity: &report.profile_identity,
        result: &report.result,
        verification_role: &report.verification_role,
        non_claims: &report.non_claims,
    };
    serde_json::to_vec(&material)
        .map(|bytes| identity(&bytes))
        .map_err(|error| format!("serialize external runtime report identity: {error}"))
}

fn evaluate_external_reports(
    report_texts: &[String],
    exact_artifact_identity: &str,
    profile_identity: &str,
    target_runtime: Option<&TargetRuntimeEvidence>,
    severity: &str,
) -> (Vec<ExternalRuntimeReport>, Vec<WasmArtifactFinding>) {
    let mut reports = Vec::new();
    let mut findings = Vec::new();
    // Rejected reports are not recorded, so each finding names the report it
    // came from: its 1-based `--external-report` position and, once parsed,
    // its claimed producer and report identity.
    for (position, text) in (1_usize..).zip(report_texts) {
        let report: ExternalRuntimeReport = match serde_json::from_str(text) {
            Ok(report) => report,
            Err(error) => {
                findings.push(finding(
                    EXTERNAL_EVIDENCE_ISSUE,
                    severity,
                    format!("external runtime report #{position}: parse external runtime report: {error}"),
                ));
                continue;
            }
        };
        let diagnostics =
            external_report_diagnostics(&report, exact_artifact_identity, profile_identity, target_runtime);
        let source = format!(
            "external runtime report #{position} (producer `{}`, report_identity `{}`)",
            report.producer, report.report_identity
        );
        for diagnostic in &diagnostics {
            findings.push(finding(EXTERNAL_EVIDENCE_ISSUE, severity, format!("{source}: {diagnostic}")));
        }
        // Only verified reports are recorded evidence. A rejected report stays a
        // finding at the declared severity; recording it would make receipt
        // validation re-reject it and fail the receipt regardless of severity.
        if diagnostics.is_empty() {
            reports.push(report);
        }
    }
    reports.sort_by(|left, right| left.report_identity.cmp(&right.report_identity));
    (reports, findings)
}

fn external_report_diagnostics(
    report: &ExternalRuntimeReport,
    exact_artifact_identity: &str,
    profile_identity: &str,
    target_runtime: Option<&TargetRuntimeEvidence>,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if report.schema_version != EXTERNAL_RUNTIME_SCHEMA {
        diagnostics.push(format!("unsupported external runtime report schema `{}`", report.schema_version));
    }
    if !matches!(report.producer.as_str(), "mantle" | "chaoscontrol" | "molten") {
        diagnostics.push(format!("unsupported external runtime report producer `{}`", report.producer));
    }
    if report.artifact_identity != exact_artifact_identity {
        diagnostics.push("external runtime report artifact identity does not match exact bytes".to_owned());
    }
    if report.profile_identity != profile_identity {
        diagnostics.push("external runtime report profile identity does not match selected profile".to_owned());
    }
    match target_runtime {
        Some(target) if report.runtime_cohort_identity != target.cohort_identity => {
            diagnostics.push("external runtime report cohort identity does not match target runtime".to_owned());
        }
        None => diagnostics.push("external runtime report requires a target runtime profile".to_owned()),
        _ => {}
    }
    if report.result.trim().is_empty() {
        diagnostics.push("external runtime report result must not be empty".to_owned());
    }
    if report.verification_role != RECORDED_ONLY_ROLE {
        diagnostics.push("external runtime report verification role must remain recorded_only".to_owned());
    }
    if !report.non_claims.iter().any(|claim| claim == EXTERNAL_RUNTIME_NON_CLAIM) {
        diagnostics.push("external runtime report is missing the no-claim-transfer boundary".to_owned());
    }
    match external_report_identity(report) {
        Ok(computed) if computed != report.report_identity => {
            diagnostics.push("external runtime report identity does not match its canonical content".to_owned());
        }
        Err(error) => diagnostics.push(error),
        _ => {}
    }
    diagnostics
}

pub(crate) fn validate_receipt_details(
    details: &WasmArtifactReceiptDetails,
    non_claims: &[String],
    diagnostics: &mut Vec<String>,
) {
    for (field, value) in [
        ("profile_identity", details.profile_identity.as_str()),
        ("registry_identity", details.registry_identity.as_str()),
        ("cohort_identity", details.cohort_identity.as_str()),
        ("exact_artifact_identity", details.exact_artifact_identity.as_str()),
        ("interface_identity", details.interface_identity.as_str()),
    ] {
        if !is_blake3_identity(value) {
            diagnostics.push(format!("Wasm artifact {field} must be a BLAKE3 identity"));
        }
    }
    if is_blake3_identity(&details.interface_identity)
        && details.interface_identity != interface_identity(&details.facts)
    {
        diagnostics.push("Wasm artifact interface_identity does not match its recorded interface facts".to_owned());
    }
    if details.input_artifact != INPUT_ARTIFACT_NAME {
        diagnostics.push(format!("Wasm artifact input must be `{INPUT_ARTIFACT_NAME}`"));
    }
    if details.exact_identity_role != "exact-bytes" {
        diagnostics.push("Wasm exact identity role must remain exact-bytes".to_owned());
    }
    if details.interface_identity_role != "normalized-interface-non-equivalence" {
        diagnostics.push("Wasm interface identity role must retain the non-equivalence boundary".to_owned());
    }
    if !matches!(details.verification_role.as_str(), RECORDED_ONLY_ROLE | "boundary") {
        diagnostics.push("Wasm artifact verification role must remain recorded_only or boundary".to_owned());
    }
    let expected_handoff_role = handoff_role(&details.handoff_parent_identities);
    if !matches!(details.handoff_role.as_str(), STANDALONE_HANDOFF | MANTLE_HANDOFF | EXTERNAL_HANDOFF) {
        diagnostics.push(format!("unsupported Wasm artifact handoff role `{}`", details.handoff_role));
    } else if details.handoff_role != expected_handoff_role {
        diagnostics.push(format!(
            "Wasm artifact handoff role `{}` does not match its parent identities; expected `{expected_handoff_role}`",
            details.handoff_role
        ));
    }
    if details.handoff_parent_identities.iter().any(|identity| !is_external_handoff_identity(identity)) {
        diagnostics.push("Wasm handoff contains an unsupported external parent identity".to_owned());
    }
    if let Some(target) = &details.target_runtime {
        for (field, value) in [
            ("target runtime source_lock_identity", target.source_lock_identity.as_str()),
            ("target runtime cohort_identity", target.cohort_identity.as_str()),
            ("target runtime support_projection_identity", target.support_projection_identity.as_str()),
        ] {
            if !is_blake3_identity(value) {
                diagnostics.push(format!("Wasm artifact {field} must be a BLAKE3 identity"));
            }
        }
        if target.verification_role != RECORDED_ONLY_ROLE {
            diagnostics.push("target runtime evidence must remain recorded_only".to_owned());
        }
    }
    for report in &details.external_evidence {
        diagnostics.extend(external_report_diagnostics(
            report,
            &details.exact_artifact_identity,
            &details.profile_identity,
            details.target_runtime.as_ref(),
        ));
    }
    if !non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
        diagnostics.push("Wasm artifact receipt missing execution non-claim".to_owned());
    }
}

fn extract_facts(bytes: &[u8], profile: &WasmArtifactProfile) -> FactExtraction {
    let mut extraction = FactExtraction::default();
    UNKNOWN_KIND.clone_into(&mut extraction.facts.artifact_kind);
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > profile.resource_limits.artifact_bytes {
        return extraction;
    }
    let mut validator = Validator::new_with_features(validation_features(profile));
    let validated = match validator.validate_all(bytes) {
        Ok(types) => Some(types),
        Err(error) => {
            extraction.parse_diagnostics.push(ParseDiagnostic {
                issue_code: if error.missing_wasm_feature().is_some() {
                    DISALLOWED_PROPOSAL_ISSUE
                } else {
                    MALFORMED_ISSUE
                },
                message: format!("wasmparser validation failed: {error}"),
                byte_offset: u64::try_from(error.offset()).ok(),
            });
            None
        }
    };
    let mut scopes = Vec::<BinaryScope>::new();
    let mut stack = Vec::<usize>::new();
    // Set once an unresolved exported alias is seen: it may refer back to any
    // local definition, so no scope's imports can be proven bound.
    let mut all_escape = false;
    for payload in Parser::new(0).parse_all(bytes) {
        let payload = match payload {
            Ok(payload) => payload,
            Err(error) => {
                push_parse_error(&mut extraction, "wasmparser parse failed", &error);
                break;
            }
        };
        if let Payload::Version { encoding, .. } = &payload {
            let id = scopes.len();
            let parent = stack.last().copied();
            if let Some(parent) = parent {
                match encoding {
                    Encoding::Module => scopes[parent].modules.push(Some(id)),
                    Encoding::Component => scopes[parent].components.push(Some(id)),
                }
            }
            let mut scope = BinaryScope {
                parent,
                ..BinaryScope::default()
            };
            UNKNOWN_KIND.clone_into(&mut scope.extraction.facts.artifact_kind);
            scopes.push(scope);
            stack.push(id);
        }
        let Some(&id) = stack.last() else { continue };
        record_scope_links(&payload, id, &mut scopes, &mut all_escape, &mut extraction);
        let ended = matches!(payload, Payload::End(_));
        extract_payload(payload, &mut scopes[id].extraction);
        if ended {
            stack.pop();
        }
    }
    if scopes.is_empty() {
        return extraction;
    }
    extraction.facts.artifact_kind.clone_from(&scopes[0].extraction.facts.artifact_kind);
    let mut multipliers = vec![1_u64; scopes.len()];
    for id in 0..scopes.len() {
        scopes[id].escapes |= all_escape;
        if let Some(parent) = scopes[id].parent {
            multipliers[id] = multipliers[parent].saturating_mul(scopes[id].instances.max(1));
            let parent_escapes = scopes[parent].escapes;
            scopes[id].escapes |= parent_escapes;
        }
    }
    for (id, scope) in scopes.into_iter().enumerate() {
        let bound = id != 0 && scope.instances > 0 && !scope.escapes && validated.is_some();
        merge_scope(&mut extraction, scope.extraction, id, multipliers[id], bound);
    }
    if profile.component_interface.as_deref() == Some("structural-v1")
        && extraction.facts.artifact_kind == COMPONENT_KIND
        && let Some(types) = validated.as_ref()
    {
        canonicalize_component_boundary(&mut extraction.facts, types);
    }
    normalize_facts(&mut extraction.facts);
    if profile.target_runtime.is_some() && extraction.facts.artifact_kind == MODULE_KIND {
        match crate::spacewasm::function_abis(bytes) {
            Ok(abis) => extraction.facts.function_abis = abis,
            Err(message) => extraction.parse_diagnostics.push(ParseDiagnostic {
                issue_code: ABI_MISMATCH_ISSUE,
                message,
                byte_offset: None,
            }),
        }
    }
    extraction
}

fn record_scope_links(
    payload: &Payload<'_>,
    id: usize,
    scopes: &mut [BinaryScope],
    all_escape: &mut bool,
    extraction: &mut FactExtraction,
) {
    use wasmparser::ComponentAlias;
    use wasmparser::ComponentExternalKind;
    use wasmparser::ComponentOuterAliasKind;
    use wasmparser::ComponentTypeRef;
    match payload {
        Payload::ComponentImportSection(reader) => {
            for import in reader.clone().into_iter().flatten() {
                match import.ty {
                    ComponentTypeRef::Module(_) => scopes[id].modules.push(None),
                    ComponentTypeRef::Component(_) => scopes[id].components.push(None),
                    _ => {}
                }
            }
        }
        Payload::ComponentAliasSection(reader) => {
            for alias in reader.clone().into_iter().flatten() {
                match alias {
                    ComponentAlias::Outer {
                        kind: ComponentOuterAliasKind::CoreModule,
                        ..
                    }
                    | ComponentAlias::InstanceExport {
                        kind: ComponentExternalKind::Module,
                        ..
                    } => {
                        scopes[id].modules.push(None);
                    }
                    ComponentAlias::Outer {
                        kind: ComponentOuterAliasKind::Component,
                        ..
                    }
                    | ComponentAlias::InstanceExport {
                        kind: ComponentExternalKind::Component,
                        ..
                    } => {
                        scopes[id].components.push(None);
                    }
                    _ => {}
                }
            }
        }
        Payload::InstanceSection(reader) => {
            for instance in reader.clone().into_iter().flatten() {
                if let wasmparser::Instance::Instantiate { module_index, .. } = instance {
                    let child = scopes[id].modules.get(module_index as usize).copied().flatten();
                    record_instantiation(child, scopes, extraction);
                }
            }
        }
        Payload::ComponentInstanceSection(reader) => {
            for instance in reader.clone().into_iter().flatten() {
                match instance {
                    wasmparser::ComponentInstance::Instantiate { component_index, .. } => {
                        let child = scopes[id].components.get(component_index as usize).copied().flatten();
                        record_instantiation(child, scopes, extraction);
                    }
                    wasmparser::ComponentInstance::FromExports(exports) => {
                        for export in &exports {
                            record_scope_export(export, id, scopes, all_escape);
                        }
                    }
                }
            }
        }
        Payload::ComponentExportSection(reader) => {
            for export in reader.clone().into_iter().flatten() {
                record_scope_export(&export, id, scopes, all_escape);
                // Component exports append aliases to their entity index
                // spaces, unlike exports used only to construct an instance.
                // Keep the local definition identity so later numeric indices
                // and repeated instantiations refer to the actual supplier.
                let entities = match export.kind {
                    ComponentExternalKind::Module => &mut scopes[id].modules,
                    ComponentExternalKind::Component => &mut scopes[id].components,
                    _ => continue,
                };
                let child = entities.get(export.index as usize).copied().flatten();
                entities.push(child);
            }
        }
        _ => {}
    }
}

fn record_scope_export(
    export: &wasmparser::ComponentExport<'_>,
    id: usize,
    scopes: &mut [BinaryScope],
    all_escape: &mut bool,
) {
    let child = match export.kind {
        wasmparser::ComponentExternalKind::Module => scopes[id].modules.get(export.index as usize),
        wasmparser::ComponentExternalKind::Component => scopes[id].components.get(export.index as usize),
        _ => return,
    }
    .copied()
    .flatten();
    if let Some(child) = child {
        scopes[child].escapes = true;
    } else {
        // An exported generative alias may refer back to any local definition.
        // Without an exact link, retain every potentially unresolved import.
        // Set one flag instead of marking every scope per export, which is
        // quadratic for crafted inputs with many scopes and such exports.
        *all_escape = true;
    }
}

fn record_instantiation(child: Option<usize>, scopes: &mut [BinaryScope], extraction: &mut FactExtraction) {
    if let Some(child) = child {
        scopes[child].instances = scopes[child].instances.saturating_add(1);
    } else {
        // Imported/aliased generative entities have no locally bounded resource
        // implementation. Never silently assign them zero resources.
        extraction.parse_diagnostics.push(ParseDiagnostic {
            issue_code: RESOURCE_LIMIT_ISSUE,
            message: "cannot bound resources of an imported or aliased module/component instantiation".to_owned(),
            byte_offset: None,
        });
    }
}

fn merge_scope(target: &mut FactExtraction, scope: FactExtraction, id: usize, instances: u64, bound: bool) {
    let mut facts = scope.facts;
    if bound {
        // Valid component instantiation supplies these imports. Their resource
        // identities belong to the supplying instance, not this import alias.
        facts.function_count = facts.function_count.saturating_sub(scope.imported_functions);
        facts.memories.drain(..scope.imported_memories);
        facts.tables.drain(..scope.imported_tables);
        facts.memory_count = facts.memory_count.saturating_sub(scope.imported_memories as u64);
        facts.table_count = facts.table_count.saturating_sub(scope.imported_tables as u64);
        for import in &mut facts.imports {
            import.namespace = format!("scope[{id}]::{}", import.namespace);
        }
        target.facts.bound_imports.append(&mut facts.imports);
    } else {
        // Nested component imports are not the root boundary; retain unresolved
        // authority without letting a root allowlist accidentally authorize it.
        if id != 0 && facts.artifact_kind == COMPONENT_KIND {
            for import in &mut facts.imports {
                import.namespace = format!("component@{id}");
            }
        }
        target.facts.imports.append(&mut facts.imports);
    }
    if id == 0 {
        target.facts.exports.append(&mut facts.exports);
    }
    target.facts.memory_count = target.facts.memory_count.saturating_add(facts.memory_count.saturating_mul(instances));
    target.facts.table_count = target.facts.table_count.saturating_add(facts.table_count.saturating_mul(instances));
    target.facts.function_count =
        target.facts.function_count.saturating_add(facts.function_count.saturating_mul(instances));
    target.facts.memories.append(&mut facts.memories);
    target.facts.tables.append(&mut facts.tables);
    target.facts.max_function_parameter_words =
        target.facts.max_function_parameter_words.max(facts.max_function_parameter_words);
    target.facts.max_local_words_per_function =
        target.facts.max_local_words_per_function.max(facts.max_local_words_per_function);
    target.facts.memory_grow_count =
        target.facts.memory_grow_count.saturating_add(facts.memory_grow_count.saturating_mul(instances));
    target.facts.has_start |= facts.has_start;
    target.facts.producer_sections = target.facts.producer_sections.saturating_add(facts.producer_sections);
    target
        .facts
        .custom_sections
        .extend(facts.custom_sections.into_iter().map(|name| format!("scope[{id}]::{name}")));
    target.parse_diagnostics.extend(scope.parse_diagnostics);
}

fn canonicalize_component_boundary(facts: &mut WasmArtifactFacts, types: &wasmparser::types::Types) {
    for (items, imports) in [(&mut facts.imports, true), (&mut facts.exports, false)] {
        for item in items {
            if item.namespace != COMPONENT_KIND {
                continue;
            }
            let entity = if imports {
                types.as_ref().component_item_for_import(&item.name)
            } else {
                types.as_ref().component_item_for_export(&item.name)
            };
            if let Some(entity) = entity {
                item.kind = structural_component_kind(entity.ty, types);
            }
        }
    }
}

fn structural_component_kind(
    entity: wasmparser::component_types::ComponentEntityType,
    types: &wasmparser::types::Types,
) -> String {
    use wasmparser::component_types::ComponentAnyTypeId;
    use wasmparser::component_types::ComponentEntityType;
    match entity {
        ComponentEntityType::Func(id) => {
            let function = &types[id];
            let params =
                function.params.iter().map(|(_, ty)| structural_value(*ty, types, 0)).collect::<Vec<_>>().join(",");
            let result = function.result.map_or_else(|| "unit".to_owned(), |ty| structural_value(ty, types, 0));
            format!("{}Func({params})->{result}", if function.async_ { "Async" } else { "" })
        }
        ComponentEntityType::Type {
            referenced: ComponentAnyTypeId::Defined(id),
            ..
        } => {
            format!("Type({})", structural_value(wasmparser::component_types::ComponentValType::Type(id), types, 0))
        }
        _ => format!("{entity:?}"),
    }
}

fn structural_value(
    ty: wasmparser::component_types::ComponentValType,
    types: &wasmparser::types::Types,
    depth: usize,
) -> String {
    use wasmparser::PrimitiveValType;
    use wasmparser::component_types::ComponentDefinedType;
    use wasmparser::component_types::ComponentValType;
    // Only the reviewed structural vocabulary is admitted. Other types remain
    // distinguishable as unsupported, never rendered as the admitted signature.
    if depth > 16 {
        return "unsupported".to_owned();
    }
    match ty {
        ComponentValType::Primitive(PrimitiveValType::U8) => "u8".to_owned(),
        ComponentValType::Primitive(PrimitiveValType::String) => "string".to_owned(),
        ComponentValType::Type(id) => match &types[id] {
            ComponentDefinedType::Primitive(primitive) => {
                structural_value(ComponentValType::Primitive(*primitive), types, depth + 1)
            }
            ComponentDefinedType::List(element) => format!("list<{}>", structural_value(*element, types, depth + 1)),
            ComponentDefinedType::Result { ok, err } => format!(
                "result<{},{}>",
                ok.map_or_else(|| "unit".to_owned(), |ty| structural_value(ty, types, depth + 1)),
                err.map_or_else(|| "unit".to_owned(), |ty| structural_value(ty, types, depth + 1))
            ),
            _ => "unsupported".to_owned(),
        },
        ComponentValType::Primitive(_) => "unsupported".to_owned(),
    }
}

fn validation_features(profile: &WasmArtifactProfile) -> WasmFeatures {
    let mut features = WasmFeatures::empty();
    for proposal in &profile.allowed_proposals {
        features |= match proposal.as_str() {
            WASM1_PROPOSAL => WasmFeatures::WASM1,
            WASM2_PROPOSAL => WasmFeatures::WASM2,
            WASM3_PROPOSAL => WasmFeatures::WASM3,
            COMPONENT_MODEL_PROPOSAL => WasmFeatures::COMPONENT_MODEL,
            BULK_MEMORY_PROPOSAL => WasmFeatures::BULK_MEMORY,
            CALL_INDIRECT_OVERLONG_PROPOSAL => WasmFeatures::CALL_INDIRECT_OVERLONG,
            MUTABLE_GLOBALS_PROPOSAL => WasmFeatures::MUTABLE_GLOBAL,
            _ => WasmFeatures::empty(),
        };
    }
    features
}

fn push_parse_error(extraction: &mut FactExtraction, context: &str, error: &wasmparser::BinaryReaderError) {
    extraction.parse_diagnostics.push(ParseDiagnostic {
        issue_code: MALFORMED_ISSUE,
        message: format!("{context}: {error}"),
        byte_offset: u64::try_from(error.offset()).ok(),
    });
}

fn extract_payload(payload: Payload<'_>, extraction: &mut FactExtraction) {
    match payload {
        Payload::Version { encoding, .. } if extraction.facts.artifact_kind == UNKNOWN_KIND => {
            match encoding {
                Encoding::Module => MODULE_KIND,
                Encoding::Component => COMPONENT_KIND,
            }
            .clone_into(&mut extraction.facts.artifact_kind);
        }
        Payload::ImportSection(reader) => extract_core_imports(reader, extraction),
        Payload::ExportSection(reader) => {
            for export in reader {
                match export {
                    Ok(export) => extraction.facts.exports.push(WasmInterfaceItem {
                        namespace: MODULE_KIND.to_owned(),
                        name: export.name.to_owned(),
                        kind: format!("{:?}", export.kind),
                    }),
                    Err(error) => push_parse_error(extraction, "parse core export", &error),
                }
            }
        }
        Payload::ComponentImportSection(reader) => {
            for import in reader {
                match import {
                    Ok(import) => extraction.facts.imports.push(WasmInterfaceItem {
                        namespace: COMPONENT_KIND.to_owned(),
                        name: import.name.name.to_owned(),
                        kind: format!("{:?}", import.ty),
                    }),
                    Err(error) => push_parse_error(extraction, "parse component import", &error),
                }
            }
        }
        Payload::ComponentExportSection(reader) => {
            for export in reader {
                match export {
                    Ok(export) => extraction.facts.exports.push(WasmInterfaceItem {
                        namespace: COMPONENT_KIND.to_owned(),
                        name: export.name.name.to_owned(),
                        kind: format!("{:?}", export.kind),
                    }),
                    Err(error) => push_parse_error(extraction, "parse component export", &error),
                }
            }
        }
        Payload::TypeSection(reader) => {
            for function_type in reader.into_iter_err_on_gc_types() {
                match function_type {
                    Ok(function_type) => {
                        let words = function_type.params().iter().copied().map(value_type_words).sum::<u64>();
                        extraction.facts.max_function_parameter_words =
                            extraction.facts.max_function_parameter_words.max(words);
                    }
                    Err(error) => push_parse_error(extraction, "parse core function type", &error),
                }
            }
        }
        Payload::FunctionSection(reader) => {
            extraction.facts.function_count += u64::from(reader.count());
        }
        Payload::ComponentCanonicalSection(reader) => extract_canonical_functions(reader, extraction),
        Payload::MemorySection(reader) => {
            extraction.facts.memory_count += u64::from(reader.count());
            for memory in reader {
                match memory {
                    Ok(memory) => extraction.facts.memories.push(memory_declaration(memory)),
                    Err(error) => push_parse_error(extraction, "parse core memory", &error),
                }
            }
        }
        Payload::TableSection(reader) => {
            extraction.facts.table_count += u64::from(reader.count());
            for table in reader {
                match table {
                    Ok(table) => extraction.facts.tables.push(table_declaration(table.ty)),
                    Err(error) => push_parse_error(extraction, "parse core table", &error),
                }
            }
        }
        Payload::StartSection { .. } | Payload::ComponentStartSection { .. } => {
            extraction.facts.has_start = true;
        }
        Payload::CodeSectionEntry(body) => extract_code_entry(&body, extraction),
        Payload::CustomSection(section) => {
            let name = section.name().to_owned();
            if name == PRODUCERS_SECTION {
                extraction.facts.producer_sections += 1;
            }
            extraction.facts.custom_sections.push(name);
        }
        _ => {}
    }
}

fn extract_core_imports(reader: wasmparser::ImportSectionReader<'_>, extraction: &mut FactExtraction) {
    for import in reader.into_imports() {
        match import {
            Ok(import) => {
                match import.ty {
                    TypeRef::Func(_) => {
                        extraction.facts.function_count += 1;
                        extraction.imported_functions += 1;
                    }
                    TypeRef::Memory(memory) => {
                        extraction.imported_memories += 1;
                        extraction.facts.memory_count += 1;
                        extraction.facts.memories.push(memory_declaration(memory));
                    }
                    TypeRef::Table(table) => {
                        extraction.imported_tables += 1;
                        extraction.facts.table_count += 1;
                        extraction.facts.tables.push(table_declaration(table));
                    }
                    _ => {}
                }
                extraction.facts.imports.push(WasmInterfaceItem {
                    namespace: import.module.to_owned(),
                    name: import.name.to_owned(),
                    kind: format!("{:?}", import.ty),
                });
            }
            Err(error) => push_parse_error(extraction, "parse core import", &error),
        }
    }
}

fn extract_canonical_functions(
    reader: wasmparser::ComponentCanonicalSectionReader<'_>,
    extraction: &mut FactExtraction,
) {
    for function in reader {
        match function {
            // Lifts expose an existing core function; every other
            // canonical operation in pinned wasmparser 0.253.0 defines
            // one core function in this binary scope. Scope merging
            // applies enclosing instance multiplicity exactly once.
            Ok(CanonicalFunction::Lift { .. }) => {}
            Ok(
                CanonicalFunction::Lower { .. }
                | CanonicalFunction::ResourceNew { .. }
                | CanonicalFunction::ResourceDrop { .. }
                | CanonicalFunction::ResourceRep { .. }
                | CanonicalFunction::ThreadSpawnRef { .. }
                | CanonicalFunction::ThreadSpawnIndirect { .. }
                | CanonicalFunction::ThreadAvailableParallelism
                | CanonicalFunction::BackpressureInc
                | CanonicalFunction::BackpressureDec
                | CanonicalFunction::TaskReturn { .. }
                | CanonicalFunction::TaskCancel
                | CanonicalFunction::ContextGet { .. }
                | CanonicalFunction::ContextSet { .. }
                | CanonicalFunction::ThreadYield { .. }
                | CanonicalFunction::SubtaskDrop
                | CanonicalFunction::SubtaskCancel { .. }
                | CanonicalFunction::StreamNew { .. }
                | CanonicalFunction::StreamRead { .. }
                | CanonicalFunction::StreamWrite { .. }
                | CanonicalFunction::StreamCancelRead { .. }
                | CanonicalFunction::StreamCancelWrite { .. }
                | CanonicalFunction::StreamDropReadable { .. }
                | CanonicalFunction::StreamDropWritable { .. }
                | CanonicalFunction::FutureNew { .. }
                | CanonicalFunction::FutureRead { .. }
                | CanonicalFunction::FutureWrite { .. }
                | CanonicalFunction::FutureCancelRead { .. }
                | CanonicalFunction::FutureCancelWrite { .. }
                | CanonicalFunction::FutureDropReadable { .. }
                | CanonicalFunction::FutureDropWritable { .. }
                | CanonicalFunction::ErrorContextNew { .. }
                | CanonicalFunction::ErrorContextDebugMessage { .. }
                | CanonicalFunction::ErrorContextDrop
                | CanonicalFunction::WaitableSetNew
                | CanonicalFunction::WaitableSetWait { .. }
                | CanonicalFunction::WaitableSetPoll { .. }
                | CanonicalFunction::WaitableSetDrop
                | CanonicalFunction::WaitableJoin
                | CanonicalFunction::ThreadIndex
                | CanonicalFunction::ThreadNewIndirect { .. }
                | CanonicalFunction::ThreadResumeLater
                | CanonicalFunction::ThreadSuspend { .. }
                | CanonicalFunction::ThreadSuspendThenResume { .. }
                | CanonicalFunction::ThreadYieldThenResume { .. }
                | CanonicalFunction::ThreadSuspendThenPromote { .. }
                | CanonicalFunction::ThreadYieldThenPromote { .. },
            ) => extraction.facts.function_count = extraction.facts.function_count.saturating_add(1),
            Err(error) => push_parse_error(extraction, "parse canonical function", &error),
        }
    }
}

fn extract_code_entry(body: &wasmparser::FunctionBody<'_>, extraction: &mut FactExtraction) {
    match body.get_locals_reader() {
        Ok(locals) => {
            let mut local_words = 0_u64;
            for local in locals {
                match local {
                    Ok((count, value_type)) => {
                        local_words =
                            local_words.saturating_add(u64::from(count).saturating_mul(value_type_words(value_type)));
                    }
                    Err(error) => push_parse_error(extraction, "parse Wasm locals", &error),
                }
            }
            extraction.facts.max_local_words_per_function =
                extraction.facts.max_local_words_per_function.max(local_words);
        }
        Err(error) => push_parse_error(extraction, "read Wasm locals", &error),
    }
    match body.get_operators_reader() {
        Ok(mut operators) => {
            while !operators.eof() {
                match operators.read() {
                    Ok(Operator::MemoryGrow { .. }) => extraction.facts.memory_grow_count += 1,
                    Ok(_) => {}
                    Err(error) => {
                        push_parse_error(extraction, "parse Wasm operator", &error);
                        break;
                    }
                }
            }
        }
        Err(error) => push_parse_error(extraction, "read Wasm operators", &error),
    }
}

fn memory_declaration(memory: wasmparser::MemoryType) -> WasmMemoryDeclaration {
    WasmMemoryDeclaration {
        initial_pages: memory.initial,
        maximum_pages: memory.maximum,
        memory64: memory.memory64,
        shared: memory.shared,
        page_size_log2: memory.page_size_log2(),
    }
}

fn table_declaration(table: wasmparser::TableType) -> WasmTableDeclaration {
    WasmTableDeclaration {
        initial_elements: table.initial,
        maximum_elements: table.maximum,
        table64: table.table64,
        shared: table.shared,
        element_type: format!("{:?}", table.element_type),
    }
}

fn value_type_words(value_type: ValType) -> u64 {
    match value_type {
        ValType::I64 | ValType::F64 => WIDE_VALUE_WORDS,
        ValType::V128 => SIMD_VALUE_WORDS,
        _ => 1,
    }
}

fn normalize_facts(facts: &mut WasmArtifactFacts) {
    facts.imports.sort();
    facts.bound_imports.sort();
    facts.exports.sort();
    facts.custom_sections.sort();
    facts.memories.sort();
    facts.tables.sort();
    facts.worlds = facts
        .imports
        .iter()
        .chain(&facts.exports)
        .filter(|item| item.name.contains(COMPONENT_WORLD_SEPARATOR))
        .map(|item| item.name.clone())
        .collect();
    facts.worlds.sort();
    facts.worlds.dedup();
    facts.proposals = match facts.artifact_kind.as_str() {
        MODULE_KIND => vec![WASM1_PROPOSAL.to_owned()],
        COMPONENT_KIND => vec![COMPONENT_MODEL_PROPOSAL.to_owned()],
        _ => Vec::new(),
    };
}

fn evaluate_policy(bytes: &[u8], facts: &WasmArtifactFacts, profile: &WasmArtifactProfile) -> Vec<WasmArtifactFinding> {
    let mut findings = Vec::new();
    if !profile.allowed_artifact_kinds.iter().any(|kind| kind == &facts.artifact_kind) {
        findings.push(finding(
            DISALLOWED_KIND_ISSUE,
            &profile.severity.disallowed_kind,
            format!("artifact kind `{}` is not allowed by profile `{}`", facts.artifact_kind, profile.profile_id),
        ));
    }
    for proposal in &facts.proposals {
        if !profile.allowed_proposals.iter().any(|allowed| allowed == proposal) {
            findings.push(finding(
                DISALLOWED_PROPOSAL_ISSUE,
                &profile.severity.disallowed_kind,
                format!("proposal `{proposal}` is not allowed by profile `{}`", profile.profile_id),
            ));
        }
    }
    push_interface_findings(facts, profile, &mut findings);
    push_spacewasm_abi_findings(bytes, facts, profile, &mut findings);
    if !profile.allowed_worlds.is_empty() {
        for world in &facts.worlds {
            if !profile.allowed_worlds.iter().any(|allowed| allowed == world) {
                findings.push(finding(
                    DISALLOWED_WORLD_ISSUE,
                    &profile.severity.disallowed_world,
                    format!("component world `{world}` is not allowed by profile `{}`", profile.profile_id),
                ));
            }
        }
    }
    push_resource_findings(bytes, facts, profile, &mut findings);
    push_metadata_findings(facts, profile, &mut findings);
    findings
}

fn push_interface_findings(
    facts: &WasmArtifactFacts,
    profile: &WasmArtifactProfile,
    findings: &mut Vec<WasmArtifactFinding>,
) {
    // SpaceWasm pilot profiles admit function imports by resolved signature
    // (`push_spacewasm_abi_findings`), so the core type index is immaterial.
    let signature_checked_imports =
        matches!(profile.profile_id.as_str(), "spacewasm-mvp-host-pilot" | "spacewasm-molten-abi-pilot");
    for import in &facts.imports {
        let identity = if signature_checked_imports && import.kind.starts_with("Func(") {
            format!("{}::{}::Func", import.namespace, import.name)
        } else {
            import_identity(import)
        };
        if !profile.allowed_imports.iter().any(|allowed| allowed == &identity) {
            findings.push(finding(
                DISALLOWED_IMPORT_ISSUE,
                &profile.severity.disallowed_import,
                format!("import `{identity}` is not allowed by profile `{}`", profile.profile_id),
            ));
        }
    }
    if !profile.allow_any_exports {
        for export in &facts.exports {
            let export_identity = import_identity(export);
            if !profile.allowed_exports.iter().any(|allowed| allowed == &export_identity) {
                findings.push(finding(
                    DISALLOWED_EXPORT_ISSUE,
                    &profile.severity.abi_mismatch,
                    format!("export `{export_identity}` is not allowed by profile `{}`", profile.profile_id),
                ));
            }
        }
    }
    for required_import in &profile.required_imports {
        if !facts.imports.iter().any(|import| import_identity(import) == *required_import) {
            findings.push(finding(
                ABI_MISMATCH_ISSUE,
                &profile.severity.abi_mismatch,
                format!("required import `{required_import}` is missing for profile `{}`", profile.profile_id),
            ));
        }
    }
    for required_export in &profile.required_exports {
        if !facts.exports.iter().any(|export| import_identity(export) == *required_export) {
            findings.push(finding(
                ABI_MISMATCH_ISSUE,
                &profile.severity.abi_mismatch,
                format!("required export `{required_export}` is missing for profile `{}`", profile.profile_id),
            ));
        }
    }
}

fn push_spacewasm_abi_findings(
    bytes: &[u8],
    facts: &WasmArtifactFacts,
    profile: &WasmArtifactProfile,
    findings: &mut Vec<WasmArtifactFinding>,
) {
    if profile.profile_id == "spacewasm-mvp" {
        let run = facts.function_abis.iter().find(|abi| abi.direction == "export" && abi.name == "run");
        if !run.is_some_and(|abi| abi.parameters.is_empty() && abi.results.is_empty()) {
            findings.push(finding(
                ABI_MISMATCH_ISSUE,
                &profile.severity.abi_mismatch,
                "SpaceWasm hostless run ABI requires () -> ()".to_owned(),
            ));
        }
    }
    if profile.profile_id == "spacewasm-mvp-host-pilot" {
        for message in crate::spacewasm::host_abi_errors(bytes, &facts.function_abis) {
            findings.push(finding(ABI_MISMATCH_ISSUE, &profile.severity.abi_mismatch, message));
        }
    }
    if profile.profile_id == "spacewasm-molten-abi-pilot" {
        for message in crate::spacewasm::molten_abi_errors(&facts.function_abis, &facts.memories) {
            findings.push(finding(ABI_MISMATCH_ISSUE, &profile.severity.abi_mismatch, message));
        }
    }
}

fn push_resource_findings(
    bytes: &[u8],
    facts: &WasmArtifactFacts,
    profile: &WasmArtifactProfile,
    findings: &mut Vec<WasmArtifactFinding>,
) {
    let artifact_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    push_limit_finding(
        findings,
        "artifact bytes",
        artifact_bytes,
        profile.resource_limits.artifact_bytes,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        findings,
        "imports",
        count(&facts.imports),
        profile.resource_limits.imports,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        findings,
        "exports",
        count(&facts.exports),
        profile.resource_limits.exports,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        findings,
        "memories",
        facts.memory_count,
        profile.resource_limits.memories,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        findings,
        "tables",
        facts.table_count,
        profile.resource_limits.tables,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        findings,
        "functions",
        facts.function_count,
        profile.resource_limits.functions,
        &profile.severity.resource_limit,
    );
    for memory in &facts.memories {
        push_typed_limit_finding(
            findings,
            MEMORY_LIMIT_ISSUE,
            "memory initial pages",
            memory.initial_pages,
            profile.resource_limits.memory_initial_pages,
            &profile.severity.resource_limit,
        );
        if let Some(maximum) = memory.maximum_pages {
            push_typed_limit_finding(
                findings,
                MEMORY_LIMIT_ISSUE,
                "memory maximum pages",
                maximum,
                profile.resource_limits.memory_maximum_pages,
                &profile.severity.resource_limit,
            );
        }
    }
    for table in &facts.tables {
        push_typed_limit_finding(
            findings,
            TABLE_LIMIT_ISSUE,
            "table initial elements",
            table.initial_elements,
            profile.resource_limits.table_initial_elements,
            &profile.severity.resource_limit,
        );
        if let Some(maximum) = table.maximum_elements {
            push_typed_limit_finding(
                findings,
                TABLE_LIMIT_ISSUE,
                "table maximum elements",
                maximum,
                profile.resource_limits.table_maximum_elements,
                &profile.severity.resource_limit,
            );
        }
    }
    push_typed_limit_finding(
        findings,
        STRUCTURAL_LIMIT_ISSUE,
        "function parameter words",
        facts.max_function_parameter_words,
        profile.resource_limits.function_parameter_words,
        &profile.severity.resource_limit,
    );
    push_typed_limit_finding(
        findings,
        STRUCTURAL_LIMIT_ISSUE,
        "local words per function",
        facts.max_local_words_per_function,
        profile.resource_limits.local_words_per_function,
        &profile.severity.resource_limit,
    );
}

fn push_metadata_findings(
    facts: &WasmArtifactFacts,
    profile: &WasmArtifactProfile,
    findings: &mut Vec<WasmArtifactFinding>,
) {
    if !profile.allow_memory_grow && facts.memory_grow_count > 0 {
        findings.push(finding(
            MEMORY_GROWTH_ISSUE,
            &profile.severity.growth,
            format!("profile `{}` forbids memory.grow", profile.profile_id),
        ));
    }
    if !profile.allow_start && facts.has_start {
        findings.push(finding(
            START_FUNCTION_ISSUE,
            &profile.severity.start,
            format!("profile `{}` forbids start functions", profile.profile_id),
        ));
    }
    if profile.producer_metadata == "required" && facts.producer_sections == 0 {
        findings.push(finding(
            PRODUCER_METADATA_ISSUE,
            &profile.severity.duplicate_metadata,
            format!("profile `{}` requires producers metadata", profile.profile_id),
        ));
    }
    if profile.producer_metadata == "reject" && facts.producer_sections > 0 {
        findings.push(finding(
            PRODUCER_METADATA_ISSUE,
            &profile.severity.duplicate_metadata,
            format!("profile `{}` forbids producers metadata", profile.profile_id),
        ));
    }
    if profile.duplicate_custom_sections == "reject" {
        for section in duplicate_names(&facts.custom_sections) {
            findings.push(finding(
                DUPLICATE_METADATA_ISSUE,
                &profile.severity.duplicate_metadata,
                format!("duplicate custom section `{section}` is rejected"),
            ));
        }
    }
}

fn validate_cohort_members(members: &[WasmToolsCohortMember], diagnostics: &mut Vec<String>) {
    if members.is_empty() {
        diagnostics.push("wasm-tools cohort must contain members".to_owned());
        return;
    }
    let mut names = BTreeSet::new();
    for member in members {
        if member.crate_name.trim().is_empty() || member.version.trim().is_empty() || member.role.trim().is_empty() {
            diagnostics.push("wasm-tools cohort members require crate_name, version, and role".to_owned());
        }
        if !names.insert(member.crate_name.as_str()) {
            diagnostics.push(format!("duplicate wasm-tools cohort member `{}`", member.crate_name));
        }
    }
}

fn validate_linked_cohort_members(members: &[WasmToolsCohortMember], diagnostics: &mut Vec<String>) {
    for (crate_name, linked_version) in LINKED_COHORT_MEMBERS {
        match members.iter().find(|member| member.crate_name == crate_name) {
            Some(member) if member.version == linked_version => {}
            Some(member) => diagnostics.push(format!(
                "wasm-tools cohort member `{crate_name}` claims version `{}`; this build links `{linked_version}`",
                member.version
            )),
            None => diagnostics
                .push(format!("wasm-tools cohort must pin linked member `{crate_name}` at `{linked_version}`")),
        }
    }
}

fn validate_profiles(profiles: &[WasmArtifactProfile], diagnostics: &mut Vec<String>) {
    if profiles.is_empty() {
        diagnostics.push("Wasm artifact registry must contain profiles".to_owned());
        return;
    }
    let mut ids = BTreeSet::new();
    for profile in profiles {
        if !ids.insert(profile.profile_id.as_str()) {
            diagnostics.push(format!("duplicate Wasm artifact profile `{}`", profile.profile_id));
        }
        if profile.allowed_artifact_kinds.is_empty() || profile.allowed_proposals.is_empty() {
            diagnostics
                .push(format!("Wasm artifact profile `{}` has empty kind or proposal policy", profile.profile_id));
        }
        for proposal in &profile.allowed_proposals {
            if !matches!(
                proposal.as_str(),
                WASM1_PROPOSAL
                    | WASM2_PROPOSAL
                    | WASM3_PROPOSAL
                    | COMPONENT_MODEL_PROPOSAL
                    | BULK_MEMORY_PROPOSAL
                    | CALL_INDIRECT_OVERLONG_PROPOSAL
                    | MUTABLE_GLOBALS_PROPOSAL
            ) {
                diagnostics.push(format!(
                    "Wasm artifact profile `{}` names unknown proposal `{proposal}`",
                    profile.profile_id
                ));
            }
        }
        if !profile.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
            diagnostics
                .push(format!("Wasm artifact profile `{}` is missing the execution non-claim", profile.profile_id));
        }
        if !matches!(profile.producer_metadata.as_str(), "ignore" | "recorded-only" | "required" | "reject") {
            diagnostics.push(format!(
                "Wasm artifact profile `{}` has unsupported producer metadata posture `{}`",
                profile.profile_id, profile.producer_metadata
            ));
        }
        if !matches!(profile.duplicate_custom_sections.as_str(), "allow" | "reject") {
            diagnostics.push(format!(
                "Wasm artifact profile `{}` has unsupported duplicate custom section posture `{}`",
                profile.profile_id, profile.duplicate_custom_sections
            ));
        }
        validate_severities(profile, diagnostics);
        if profile.component_interface.as_deref().is_some_and(|format| format != "structural-v1") {
            diagnostics
                .push(format!("Wasm artifact profile `{}` has unknown component interface format", profile.profile_id));
        }
        for required_import in &profile.required_imports {
            if !profile.allowed_imports.contains(required_import) {
                diagnostics.push(format!(
                    "Wasm artifact profile `{}` requires an import without allowing it",
                    profile.profile_id
                ));
            }
        }
        for required_export in &profile.required_exports {
            if profile.allow_any_exports {
                continue;
            }
            if !profile.allowed_exports.iter().any(|allowed| allowed == required_export) {
                diagnostics.push(format!(
                    "Wasm artifact profile `{}` requires export `{required_export}` without allowing it",
                    profile.profile_id
                ));
            }
        }
        if let Some(target) = &profile.target_runtime {
            validate_target_runtime(profile, target, diagnostics);
        } else if profile.profile_id == "spacewasm-mvp" {
            diagnostics.push("spacewasm-mvp profile requires a target runtime cohort".to_owned());
        }
        if !matches!(profile.verification_role.as_str(), RECORDED_ONLY_ROLE | "boundary") {
            diagnostics.push(format!(
                "Wasm artifact profile `{}` requests an unsupported verification role",
                profile.profile_id
            ));
        }
        validate_profile_rules(profile, diagnostics);
    }
}

fn validate_profile_rules(profile: &WasmArtifactProfile, diagnostics: &mut Vec<String>) {
    for (field, entries) in [
        ("allowed_artifact_kinds", &profile.allowed_artifact_kinds),
        ("allowed_proposals", &profile.allowed_proposals),
        ("allowed_worlds", &profile.allowed_worlds),
        ("allowed_imports", &profile.allowed_imports),
        ("allowed_exports", &profile.allowed_exports),
        ("required_imports", &profile.required_imports),
        ("required_exports", &profile.required_exports),
    ] {
        let mut seen = BTreeSet::new();
        for entry in entries {
            if !seen.insert(entry.as_str()) {
                diagnostics
                    .push(format!("Wasm artifact profile `{}` repeats {field} entry `{entry}`", profile.profile_id));
            }
        }
    }
    for world in &profile.allowed_worlds {
        if !is_wit_world_name(world) {
            diagnostics.push(format!("Wasm artifact profile `{}` names invalid world `{world}`", profile.profile_id));
        }
    }
}

// A world is `namespace:package/name` with an optional `@semver` suffix; each
// part is a component-model kebab-case label.
fn is_wit_world_name(value: &str) -> bool {
    let (name, version) = match value.split_once('@') {
        Some((name, version)) => (name, Some(version)),
        None => (value, None),
    };
    let Some((package, world)) = name.split_once(COMPONENT_WORLD_SEPARATOR) else {
        return false;
    };
    let Some((namespace, package)) = package.split_once(':') else {
        return false;
    };
    [namespace, package, world].into_iter().all(is_wit_label) && version.is_none_or(is_semver)
}

fn is_wit_label(label: &str) -> bool {
    !label.is_empty()
        && label.split('-').all(|fragment| {
            let mut bytes = fragment.bytes();
            match bytes.next() {
                Some(first) if first.is_ascii_lowercase() => {
                    bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
                }
                Some(first) if first.is_ascii_uppercase() => {
                    bytes.all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
                }
                _ => false,
            }
        })
}

fn is_semver(version: &str) -> bool {
    let (version, build) = match version.split_once('+') {
        Some((version, build)) => (version, Some(build)),
        None => (version, None),
    };
    let (core, prerelease) = match version.split_once('-') {
        Some((core, prerelease)) => (core, Some(prerelease)),
        None => (version, None),
    };
    let numeric = |part: &str| {
        !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()) && (part == "0" || !part.starts_with('0'))
    };
    let identifiers = |text: &str| {
        text.split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'-'))
    };
    let mut parts = core.split('.');
    parts.by_ref().take(3).filter(|part| numeric(part)).count() == 3
        && parts.next().is_none()
        && prerelease.is_none_or(identifiers)
        && build.is_none_or(identifiers)
}

fn validate_severities(profile: &WasmArtifactProfile, diagnostics: &mut Vec<String>) {
    let severity = &profile.severity;
    for (field, value) in [
        ("malformed", &severity.malformed),
        ("disallowed_kind", &severity.disallowed_kind),
        ("disallowed_import", &severity.disallowed_import),
        ("disallowed_world", &severity.disallowed_world),
        ("resource_limit", &severity.resource_limit),
        ("growth", &severity.growth),
        ("start", &severity.start),
        ("duplicate_metadata", &severity.duplicate_metadata),
        ("abi_mismatch", &severity.abi_mismatch),
        ("external_evidence", &severity.external_evidence),
    ] {
        if !matches!(value.as_str(), "advisory" | "warning" | ERROR_SEVERITY) {
            diagnostics.push(format!(
                "Wasm artifact profile `{}` has unsupported {field} severity `{value}`",
                profile.profile_id
            ));
        }
    }
}

fn validate_target_runtime(profile: &WasmArtifactProfile, target: &TargetRuntimeCohort, diagnostics: &mut Vec<String>) {
    let support = &target.support_projection;
    if support.schema_version != SPACEWASM_SUPPORT_SCHEMA {
        diagnostics.push(format!("unsupported target runtime support schema `{}`", support.schema_version));
    }
    if support.runtime_id != SPACEWASM_RUNTIME_ID {
        diagnostics.push(format!("unsupported target runtime `{}`", support.runtime_id));
    }
    for proposal in &profile.allowed_proposals {
        if !support.supported_proposals.contains(proposal)
            || support.denied_or_unimplemented_proposals.contains(proposal)
        {
            diagnostics.push(format!("profile proposal `{proposal}` is outside reviewed runtime support"));
        }
    }
    let valid_revision = support.source_revision.len() == GIT_REVISION_HEX_LENGTH
        && support.source_revision.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase());
    if !valid_revision {
        diagnostics.push("target runtime source revision must be a lowercase 40-hex Git revision".to_owned());
    }
    if !is_blake3_identity(&support.source_lock_identity) {
        diagnostics.push("target runtime source lock must carry a BLAKE3 identity".to_owned());
    }
    match runtime_cohort_identity(support) {
        Ok(computed) if computed != target.cohort_identity => diagnostics.push(format!(
            "stale target runtime cohort identity; expected `{}`, observed `{computed}`",
            target.cohort_identity
        )),
        Err(errors) => diagnostics.extend(errors),
        _ => {}
    }
    if support.module_kind != MODULE_KIND {
        diagnostics.push("SpaceWasm support projection must remain core-module-only".to_owned());
    }
    if support.default_allow_memory_grow != profile.allow_memory_grow {
        diagnostics.push("target runtime memory.grow posture differs from profile policy".to_owned());
    }
    for (label, observed, configured) in [
        ("memory pages", support.maximum_linear_memory_pages, profile.resource_limits.memory_maximum_pages),
        (
            "function parameter words",
            support.maximum_function_parameter_words,
            profile.resource_limits.function_parameter_words,
        ),
        (
            "local words per function",
            support.maximum_local_words_per_function,
            profile.resource_limits.local_words_per_function,
        ),
        ("function index", support.maximum_function_index, profile.resource_limits.functions),
    ] {
        if configured > observed {
            diagnostics
                .push(format!("profile {label} limit `{configured}` exceeds target runtime support `{observed}`"));
        }
    }
}

fn profile_identity(profile: &WasmArtifactProfile) -> Result<String, Vec<String>> {
    let bytes =
        serde_json::to_vec(profile).map_err(|error| vec![format!("serialize Wasm profile identity: {error}")])?;
    Ok(identity(&bytes))
}

fn interface_identity(facts: &WasmArtifactFacts) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(INTERFACE_IDENTITY_DOMAIN);
    update_hash_field(&mut hasher, facts.artifact_kind.as_bytes());
    for world in &facts.worlds {
        update_hash_field(&mut hasher, world.as_bytes());
    }
    for item in &facts.imports {
        update_interface_item(&mut hasher, b"import", item);
    }
    for item in &facts.exports {
        update_interface_item(&mut hasher, b"export", item);
    }
    for abi in &facts.function_abis {
        update_hash_field(&mut hasher, abi.direction.as_bytes());
        update_hash_field(&mut hasher, abi.namespace.as_bytes());
        update_hash_field(&mut hasher, abi.name.as_bytes());
        for parameter in &abi.parameters {
            update_hash_field(&mut hasher, parameter.as_bytes());
        }
        update_hash_field(&mut hasher, b"results");
        for result in &abi.results {
            update_hash_field(&mut hasher, result.as_bytes());
        }
    }
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

fn update_interface_item(hasher: &mut blake3::Hasher, direction: &[u8], item: &WasmInterfaceItem) {
    update_hash_field(hasher, direction);
    update_hash_field(hasher, item.namespace.as_bytes());
    update_hash_field(hasher, item.name.as_bytes());
    update_hash_field(hasher, item.kind.as_bytes());
}

fn update_hash_field(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    let length = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
}

fn identity(bytes: &[u8]) -> String {
    format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
}

fn import_identity(item: &WasmInterfaceItem) -> String {
    format!("{}::{}::{}", item.namespace, item.name, item.kind)
}

fn push_typed_limit_finding(
    findings: &mut Vec<WasmArtifactFinding>,
    issue_code: &str,
    resource: &str,
    observed: u64,
    limit: u64,
    severity: &str,
) {
    if observed > limit {
        findings.push(finding(
            issue_code,
            severity,
            format!("{resource} `{observed}` exceeds profile limit `{limit}`"),
        ));
    }
}

fn push_limit_finding(
    findings: &mut Vec<WasmArtifactFinding>,
    resource: &str,
    observed: u64,
    limit: u64,
    severity: &str,
) {
    if observed > limit {
        findings.push(finding(
            RESOURCE_LIMIT_ISSUE,
            severity,
            format!("{resource} count `{observed}` exceeds profile limit `{limit}`"),
        ));
    }
}

fn count<T>(items: &[T]) -> u64 {
    u64::try_from(items.len()).unwrap_or(u64::MAX)
}

fn duplicate_names(names: &[String]) -> Vec<String> {
    let mut duplicates = Vec::new();
    for window in names.windows(2) {
        if window[0] == window[1] {
            duplicates.push(window[0].clone());
        }
    }
    duplicates.sort();
    duplicates.dedup();
    duplicates
}

fn finding(issue_code: &str, severity: &str, message: String) -> WasmArtifactFinding {
    WasmArtifactFinding {
        issue_code: issue_code.to_owned(),
        severity: severity.to_owned(),
        message,
        byte_offset: None,
    }
}

fn render_sarif(findings: &[WasmArtifactFinding]) -> Value {
    let results = findings
        .iter()
        .map(|finding| {
            let mut physical_location = serde_json::json!({"artifactLocation": {"uri": INPUT_ARTIFACT_NAME}});
            if let Some(offset) = finding.byte_offset {
                physical_location["region"] = serde_json::json!({"byteOffset": offset});
            }
            serde_json::json!({
                "ruleId": finding.issue_code,
                "level": sarif_level(&finding.severity),
                "message": {"text": finding.message},
                "locations": [{"physicalLocation": physical_location}]
            })
        })
        .collect::<Vec<_>>();
    serde_json::json!({
        "version": "2.1.0",
        "runs": [{
            "tool": {"driver": {"name": "cargo-octet-wasm-artifact"}},
            "results": results
        }]
    })
}

// SARIF 2.1.0 levels are `none`, `note`, `warning`, and `error`; the profile's
// `advisory` severity is informational.
fn sarif_level(severity: &str) -> &str {
    if severity == "advisory" { "note" } else { severity }
}

fn is_blake3_identity(value: &str) -> bool {
    value
        .strip_prefix(BLAKE3_PREFIX)
        .is_some_and(|hex| hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

pub(crate) fn is_external_handoff_identity(value: &str) -> bool {
    value.starts_with(MANTLE_PARENT_PREFIX)
        || value.starts_with(CHAOSCONTROL_PARENT_PREFIX)
        || value.starts_with(MOLTEN_PARENT_PREFIX)
}

fn handoff_role(parent_artifacts: &[String]) -> &'static str {
    let mut role = STANDALONE_HANDOFF;
    for parent in parent_artifacts.iter().filter(|parent| is_external_handoff_identity(parent)) {
        if parent.starts_with(MANTLE_PARENT_PREFIX) {
            return MANTLE_HANDOFF;
        }
        role = EXTERNAL_HANDOFF;
    }
    role
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use wasm_encoder::CustomSection;
    use wasm_encoder::Module;

    use super::*;

    const PROFILE_JSON: &str = include_str!("../../standards/wasm-artifact/generated/profiles.json");
    const PROFILE_ID: &str = "portable-component-baseline";
    const SPACEWASM_PROFILE_ID: &str = "spacewasm-mvp";
    const EFFECTS_PROFILE_ID: &str = "molten-component-effects-v1";
    const EFFECTS_SEED: &str = include_str!("../../standards/wasm-artifact/fixtures/positive/component-effects.wat");

    #[test]
    fn custom_metadata_duplicates_are_local_to_each_binary_scope() {
        let mut component = wasm_encoder::Component::new();
        let named = module_with_custom("name", &[]);
        let mut module = Module::new();
        module.section(&CustomSection {
            name: Cow::Borrowed("name"),
            data: Cow::Borrowed(&[]),
        });
        component.section(&wasm_encoder::RawSection { id: 1, data: &named });
        component.section(&wasm_encoder::ModuleSection(&module));
        component.section(&CustomSection {
            name: Cow::Borrowed("name"),
            data: Cow::Borrowed(&[]),
        });
        let mut nested = wasm_encoder::Component::new();
        nested.section(&CustomSection {
            name: Cow::Borrowed("name"),
            data: Cow::Borrowed(&[]),
        });
        component.section(&wasm_encoder::NestedComponentSection(&nested));
        let registry = valid_registry();
        let report = analyze(&component.finish(), &registry, &[]);
        assert!(report.details.findings.is_empty(), "{:?}", report.details.findings);

        let duplicate = module_with_duplicate_custom("name");
        let mut component = wasm_encoder::Component::new();
        component.section(&wasm_encoder::RawSection {
            id: 1,
            data: &duplicate,
        });
        let report = analyze(&component.finish(), &registry, &[]);
        assert!(report.details.findings.iter().any(|finding| finding.issue_code == DUPLICATE_METADATA_ISSUE));
    }

    #[test]
    fn bound_adapters_are_not_external_imports_or_extra_resources() {
        let registry = valid_registry();
        let source = r#"(component
            (core module $provider
                (table (export "table") 1 1 funcref)
                (func (export "f")))
            (core instance $provider-instance (instantiate $provider))
            (core module $adapter
                (import "env" "f" (func))
                (import "env" "table" (table 1 1 funcref)))
            (core instance (instantiate $adapter (with "env" (instance $provider-instance))))
        )"#;
        let bytes = wat::parse_str(source).expect("bound adapter fixture");
        let report = analyze(&bytes, &registry, &[]);
        assert!(report.details.findings.is_empty(), "{:?}", report.details.findings);
        assert_eq!(report.details.facts.table_count, 1);
        assert_eq!(report.details.facts.tables.len(), 1);

        let unresolved =
            source.replace("(core instance (instantiate $adapter (with \"env\" (instance $provider-instance))))", "");
        let report = analyze(&wat::parse_str(unresolved).expect("unresolved adapter"), &registry, &[]);
        assert!(report.details.findings.iter().any(|finding| finding.issue_code == DISALLOWED_IMPORT_ISSUE));

        // Export after its definition: a reusable module still carries ambient imports.
        let escaped = format!(
            "{} (export \"adapter\" (core module $adapter)))",
            &source[..source.rfind(')').expect("component close")]
        );
        let report = analyze(&wat::parse_str(escaped).expect("exported adapter"), &registry, &[]);
        assert!(report.details.findings.iter().any(|finding| finding.issue_code == DISALLOWED_IMPORT_ISSUE));

        let repeated =
            source.replace("(core module $adapter", "(core instance (instantiate $provider)) (core module $adapter");
        let report = analyze(&wat::parse_str(repeated).expect("repeated instantiation"), &registry, &[]);
        assert_eq!(report.details.facts.table_count, 2);
        assert!(report.details.findings.iter().any(|finding| finding.issue_code == RESOURCE_LIMIT_ISSUE));
    }

    #[test]
    fn unresolved_exports_retain_authority_of_every_definition() {
        let registry = valid_registry();
        let adapter = r#"
            (core module $provider
                (table (export "table") 1 1 funcref)
                (func (export "f")))
            (core instance $provider-instance (instantiate $provider))
            (core module $adapter
                (import "env" "f" (func))
                (import "env" "table" (table 1 1 funcref)))
            (core instance (instantiate $adapter (with "env" (instance $provider-instance))))"#;
        let reexport = r#"(export "reexported" (core module $imported))"#;
        let adapter_import_is_unbound = |source: String| {
            let report = analyze(&wat::parse_str(source).expect("re-exported imported module"), &registry, &[]);
            let unbound = report.details.facts.imports.iter().any(|item| item.namespace == "env" && item.name == "f");
            let bound = report.details.facts.bound_imports.iter().any(|item| item.name == "f");
            assert_ne!(unbound, bound, "{:?}", report.details.facts);
            unbound
        };

        let unexported = format!(r#"(component (import "m" (core module $imported)) {adapter})"#);
        assert!(!adapter_import_is_unbound(unexported));
        // An export without a local link may alias any definition, wherever it appears.
        for source in [
            format!(r#"(component (import "m" (core module $imported)) {adapter} {reexport})"#),
            format!(r#"(component (import "m" (core module $imported)) {reexport} {adapter})"#),
        ] {
            assert!(adapter_import_is_unbound(source));
        }
    }

    #[test]
    fn unresolved_exports_mark_escaping_scopes_in_constant_work() {
        const SCOPES: usize = 4096;
        const EXPORTS: u32 = 8192;
        let mut scopes = std::iter::repeat_with(BinaryScope::default).take(SCOPES).collect::<Vec<_>>();
        // Index 0 is an imported module; every other index is out of range.
        scopes[0].modules.push(None);
        let mut all_escape = false;
        for index in 0..EXPORTS {
            let export = wasmparser::ComponentExport {
                name: wasmparser::ComponentExternName {
                    name: "reexported",
                    implements: None,
                    version_suffix: None,
                    external_id: None,
                },
                kind: wasmparser::ComponentExternalKind::Module,
                index: index % 2,
                ty: None,
            };
            record_scope_export(&export, 0, &mut scopes, &mut all_escape);
        }
        // Unresolved exports set one flag instead of writing every scope per
        // export; the multiplier pass applies it once per scope.
        assert!(all_escape);
        assert!(scopes.iter().all(|scope| !scope.escapes));
    }

    #[test]
    fn effects_profile_requires_exact_structural_boundary_and_keeps_baselines_closed() {
        let registry = valid_registry();
        let bytes = wat::parse_str(EFFECTS_SEED).expect("effect seed");
        let report = analyze_profile(&bytes, &registry, EFFECTS_PROFILE_ID, &[]);
        assert!(report.details.findings.is_empty(), "{:?}", report.details.findings);
        assert_eq!(report.details.exact_artifact_identity, identity(&bytes));
        assert_eq!(report.details.verification_role, RECORDED_ONLY_ROLE);
        assert_eq!(report.details.facts.imports.len(), 2);
        let reindexed = EFFECTS_SEED.replace("(type $bytes", "(type (list string)) (type $bytes");
        let reindexed = analyze_profile(
            &wat::parse_str(reindexed).expect("reindexed effect types"),
            &registry,
            EFFECTS_PROFILE_ID,
            &[],
        );
        assert!(reindexed.details.findings.is_empty(), "{:?}", reindexed.details.findings);
        assert_eq!(report.details.interface_identity, reindexed.details.interface_identity);
        assert_ne!(report.details.exact_artifact_identity, reindexed.details.exact_artifact_identity);
        for baseline in [PROFILE_ID, "molten-component-baseline"] {
            let denied = analyze_profile(&bytes, &registry, baseline, &[]);
            assert!(denied.details.findings.iter().any(|finding| finding.issue_code == DISALLOWED_IMPORT_ISSUE));
        }
        let cases = [
            (EFFECTS_SEED.replace("effect-request", "other-product-effect"), DISALLOWED_IMPORT_ISSUE),
            (EFFECTS_SEED.replace("effect-request", "wasi:random/random@0.2.0"), DISALLOWED_IMPORT_ISSUE),
            (
                EFFECTS_SEED.replace("(param \"request\" $payload)", "(param \"request\" string)"),
                DISALLOWED_IMPORT_ISSUE,
            ),
            (EFFECTS_SEED.replace("(list u8)", "(list u16)"), DISALLOWED_IMPORT_ISSUE),
            (EFFECTS_SEED.replace("(result $outcome)", "(result string)"), ABI_MISMATCH_ISSUE),
            (EFFECTS_SEED.replace("\"invoke\"", "\"other\""), ABI_MISMATCH_ISSUE),
            (EFFECTS_SEED.replace("(export \"invoke\" (func $effect))", ""), ABI_MISMATCH_ISSUE),
            (
                EFFECTS_SEED
                    .replace("(export \"invoke\"", "(import \"extra\" (func (type $request))) (export \"invoke\""),
                DISALLOWED_IMPORT_ISSUE,
            ),
        ];
        for (source, issue) in cases {
            let report = analyze_profile(
                &wat::parse_str(source).expect("negative effect surface"),
                &registry,
                EFFECTS_PROFILE_ID,
                &[],
            );
            assert!(
                report.details.findings.iter().any(|finding| finding.issue_code == issue),
                "missing {issue}: {:?}",
                report.details.findings
            );
        }
        let missing = "(component (core module (func)))";
        let report = analyze_profile(
            &wat::parse_str(missing).expect("missing effect boundary"),
            &registry,
            EFFECTS_PROFILE_ID,
            &[],
        );
        assert!(report.details.findings.iter().any(|finding| finding.message.contains("required import")));
    }

    #[test]
    fn effects_admits_overlong_zero_table_encoding_without_reference_types() {
        use wasm_encoder::Encode;
        let registry = valid_registry();
        let module = |table_index: &[u8]| {
            let mut module = Module::new();
            module.section(&wasm_encoder::RawSection {
                id: 1,
                data: &[1, 0x60, 0, 0],
            });
            module.section(&wasm_encoder::RawSection { id: 3, data: &[1, 0] });
            module.section(&wasm_encoder::RawSection {
                id: 4,
                data: &[1, 0x70, 1, 1, 1],
            });
            let mut body = vec![0, 0x41, 0, 0x11, 0];
            body.extend_from_slice(table_index);
            body.push(0x0b);
            let mut code = vec![1, u8::try_from(body.len()).expect("small call-indirect body")];
            code.extend(body);
            module.section(&wasm_encoder::RawSection { id: 10, data: &code });
            module
        };
        for (table_index, denied) in [(&[0][..], false), (&[0x80, 0][..], false), (&[1][..], true)] {
            let mut bytes = wat::parse_str(EFFECTS_SEED).expect("effect surface");
            bytes.push(1);
            wasm_encoder::ModuleSection(&module(table_index)).encode(&mut bytes);
            let report = analyze_profile(&bytes, &registry, EFFECTS_PROFILE_ID, &[]);
            if denied {
                assert!(
                    report.details.findings.iter().any(|finding| matches!(
                        finding.issue_code.as_str(),
                        MALFORMED_ISSUE | DISALLOWED_PROPOSAL_ISSUE
                    )),
                    "{:?}",
                    report.details.findings
                );
            } else {
                assert!(report.details.findings.is_empty(), "{:?}", report.details.findings);
            }
        }
        let portable =
            registry.profiles.iter().find(|profile| profile.profile_id == PROFILE_ID).expect("portable profile");
        assert!(extract_facts(&module(&[0]).finish(), portable).parse_diagnostics.is_empty());
        assert!(
            extract_facts(&module(&[0x80, 0]).finish(), portable)
                .parse_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.issue_code == MALFORMED_ISSUE)
        );
        let effects = registry
            .profiles
            .iter()
            .find(|profile| profile.profile_id == EFFECTS_PROFILE_ID)
            .expect("effect profile");
        assert!(!validation_features(effects).contains(WasmFeatures::REFERENCE_TYPES));
        let source = format!(
            "{} (core module (func (drop (ref.null func)))))",
            &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")]
        );
        let report = analyze_profile(
            &wat::parse_str(source).expect("reference-types negative"),
            &registry,
            EFFECTS_PROFILE_ID,
            &[],
        );
        assert!(report.details.findings.iter().any(|finding| finding.issue_code == DISALLOWED_PROPOSAL_ISSUE));
    }

    #[test]
    fn effects_function_budget_counts_canonical_lower_definitions() {
        let registry = valid_registry();
        for (lowers, denied) in [(127, false), (128, true)] {
            let source = format!(
                r#"{}
                (core module $allocator
                    (memory (export "memory") 1 1)
                    (func (export "realloc") (param i32 i32 i32 i32) (result i32) i32.const 0))
                (core instance $allocator-instance (instantiate $allocator))
                (alias core export $allocator-instance "memory" (core memory $memory))
                (alias core export $allocator-instance "realloc" (core func $realloc))
                {})"#,
                &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")],
                "(core func (canon lower (func $effect) (memory $memory) (realloc $realloc)))".repeat(lowers)
            );
            let bytes = wat::parse_str(source).expect("canonical lower function boundary");
            let report = analyze_profile(&bytes, &registry, EFFECTS_PROFILE_ID, &[]);
            assert_eq!(report.details.facts.function_count, lowers as u64 + 1);
            if denied {
                assert_eq!(report.details.findings.len(), 1, "{:?}", report.details.findings);
                assert_eq!(report.details.findings[0].issue_code, RESOURCE_LIMIT_ISSUE);
            } else {
                assert!(report.details.findings.is_empty(), "{:?}", report.details.findings);
            }
        }
    }

    #[test]
    fn effects_function_budget_counts_resource_intrinsics_but_not_aliases_or_lifts() {
        let registry = valid_registry();
        for operation in ["resource.new", "resource.drop", "resource.rep"] {
            for (definitions, denied) in [(127, false), (128, true)] {
                let source = format!(
                    r#"{}
                    (type $resource (resource (rep i32)))
                    (core module $implementation (func (export "noop")))
                    (core instance $implementation-instance (instantiate $implementation))
                    (alias core export $implementation-instance "noop" (core func $noop))
                    (type $noop-type (func))
                    {} {} {})"#,
                    &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")],
                    format!("(core func (canon {operation} $resource))").repeat(definitions),
                    "(alias core export $implementation-instance \"noop\" (core func))".repeat(128),
                    "(func (type $noop-type) (canon lift (core func $noop)))".repeat(128),
                );
                let report = analyze_profile(
                    &wat::parse_str(source).expect("canonical resource function boundary"),
                    &registry,
                    EFFECTS_PROFILE_ID,
                    &[],
                );
                assert_eq!(report.details.facts.function_count, definitions as u64 + 1, "{operation}");
                if denied {
                    assert_eq!(report.details.findings.len(), 1, "{operation}: {:?}", report.details.findings);
                    assert_eq!(report.details.findings[0].issue_code, RESOURCE_LIMIT_ISSUE);
                } else {
                    assert!(report.details.findings.is_empty(), "{operation}: {:?}", report.details.findings);
                }
            }
        }
    }

    #[test]
    fn effects_function_budget_multiplies_canonical_definitions_by_enclosing_instances() {
        let registry = valid_registry();
        for (definitions, denied) in [(64, false), (65, true)] {
            let source = format!(
                r"{}
                (component $inner
                    (type $resource (resource (rep i32)))
                    {})
                (instance (instantiate $inner))
                (instance (instantiate $inner)))",
                &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")],
                "(core func (canon resource.new $resource))".repeat(definitions),
            );
            let report = analyze_profile(
                &wat::parse_str(source).expect("canonical enclosing instance function boundary"),
                &registry,
                EFFECTS_PROFILE_ID,
                &[],
            );
            assert_eq!(report.details.facts.function_count, definitions as u64 * 2);
            if denied {
                assert_eq!(report.details.findings.len(), 1, "{:?}", report.details.findings);
                assert_eq!(report.details.findings[0].issue_code, RESOURCE_LIMIT_ISSUE);
            } else {
                assert!(report.details.findings.is_empty(), "{:?}", report.details.findings);
            }
        }
    }

    #[test]
    fn effects_resource_limits_preserve_exported_module_and_component_indices() {
        let registry = valid_registry();
        let cases = [
            (
                "(core module (table 1 1 funcref))",
                "(export \"supplier\" (core module 0))",
                "(core module)",
                "(core instance (instantiate 1))",
            ),
            (
                "(component (core module $m (table 1 1 funcref)) (core instance (instantiate $m)))",
                "(export \"supplier\" (component 0))",
                "(component)",
                "(instance (instantiate 1))",
            ),
        ];
        for (supplier, export, empty, instantiate) in cases {
            for (instances, containers, denied) in [(2, 1, false), (3, 1, true), (1, 2, false), (2, 2, true)] {
                let source = format!(
                    "{} (component $container {supplier} {export} {empty} {}) {})",
                    &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")],
                    instantiate.repeat(instances),
                    "(instance (instantiate $container))".repeat(containers),
                );
                let report = analyze_profile(
                    &wat::parse_str(source).expect("exported generative alias resource boundary"),
                    &registry,
                    EFFECTS_PROFILE_ID,
                    &[],
                );
                assert_eq!(report.details.facts.table_count, (instances * containers) as u64, "{export}");
                if denied {
                    assert_eq!(report.details.findings.len(), 1, "{export}: {:?}", report.details.findings);
                    assert_eq!(report.details.findings[0].issue_code, RESOURCE_LIMIT_ISSUE);
                } else {
                    assert!(report.details.findings.is_empty(), "{export}: {:?}", report.details.findings);
                }
            }
        }
    }

    #[test]
    fn effects_instance_from_exports_does_not_append_generative_indices() {
        let registry = valid_registry();
        let cases = [
            (
                "(core module (table 1 1 funcref))",
                "(instance (export \"supplier\" (core module 0)))",
                "(core module)",
                "(core instance (instantiate 1))",
            ),
            (
                "(component (core module $m (table 1 1 funcref)) (core instance (instantiate $m)))",
                "(instance (export \"supplier\" (component 0)))",
                "(component)",
                "(instance (instantiate 1))",
            ),
        ];
        for (supplier, exported_instance, empty, instantiate) in cases {
            let source = format!(
                "{} (component $container {supplier} {exported_instance} {empty} {}) (instance (instantiate $container)))",
                &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")],
                instantiate.repeat(3),
            );
            let report = analyze_profile(
                &wat::parse_str(source).expect("instance export entity index separation"),
                &registry,
                EFFECTS_PROFILE_ID,
                &[],
            );
            // Index 1 is the empty definition, not an alias of the supplier.
            assert_eq!(report.details.facts.table_count, 1, "{exported_instance}");
            assert!(report.details.findings.is_empty(), "{exported_instance}: {:?}", report.details.findings);
        }
    }

    #[test]
    fn effects_resource_limits_count_independent_instantiations_and_check_declarations() {
        let registry = valid_registry();
        for (tables, expected) in [(2, false), (3, true)] {
            let source = format!(
                "{} {})",
                &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")],
                "(core module (table 1 1 funcref))".repeat(tables)
            );
            let report = analyze_profile(
                &wat::parse_str(source).expect("table count boundary"),
                &registry,
                EFFECTS_PROFILE_ID,
                &[],
            );
            assert_eq!(report.details.facts.table_count, tables as u64);
            assert_eq!(
                report.details.findings.iter().any(|finding| finding.issue_code == RESOURCE_LIMIT_ISSUE),
                expected
            );
        }
        for (elements, expected) in [(65535, false), (65536, true)] {
            let source = format!(
                "{} (core module (table {elements} {elements} funcref)))",
                &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")]
            );
            let report = analyze_profile(
                &wat::parse_str(source).expect("table elements boundary"),
                &registry,
                EFFECTS_PROFILE_ID,
                &[],
            );
            assert_eq!(report.details.findings.iter().any(|finding| finding.issue_code == TABLE_LIMIT_ISSUE), expected);
        }
        for (instances, expected) in [(2, false), (3, true)] {
            let source = format!(
                "{} (component $inner (core module $m (table 1 1 funcref)) (core instance (instantiate $m))) {})",
                &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")],
                "(instance (instantiate $inner))".repeat(instances)
            );
            let report = analyze_profile(
                &wat::parse_str(source).expect("enclosing component multiplicity"),
                &registry,
                EFFECTS_PROFILE_ID,
                &[],
            );
            assert_eq!(report.details.facts.table_count, instances as u64);
            assert_eq!(
                report.details.findings.iter().any(|finding| finding.issue_code == RESOURCE_LIMIT_ISSUE),
                expected
            );
        }
        let mut bounded_memory_registry = registry.clone();
        let profile = bounded_memory_registry
            .profiles
            .iter_mut()
            .find(|profile| profile.profile_id == EFFECTS_PROFILE_ID)
            .expect("effects profile");
        profile.resource_limits.memory_initial_pages = 1;
        profile.resource_limits.memory_maximum_pages = 1;
        for (maximum, expected) in [(1, false), (2, true)] {
            let source = format!(
                "{} (core module (memory 1 {maximum})))",
                &EFFECTS_SEED[..EFFECTS_SEED.rfind(')').expect("seed close")]
            );
            let report = analyze_profile(
                &wat::parse_str(source).expect("memory maximum boundary"),
                &bounded_memory_registry,
                EFFECTS_PROFILE_ID,
                &[],
            );
            assert_eq!(
                report.details.findings.iter().any(|finding| finding.issue_code == MEMORY_LIMIT_ISSUE),
                expected
            );
        }
    }
    const MANTLE_PARENT: &str = "mantle:fixture-materialization";

    #[test]
    fn checked_in_registry_and_cohort_identity_are_fresh() {
        let registry = parse_registry(PROFILE_JSON).expect("profile registry");

        let diagnostics = validate_registry(&registry);
        let checked_projection: RuntimeSupportProjection =
            serde_json::from_str(include_str!("../../standards/wasm-artifact/spacewasm/support-projection.json"))
                .expect("checked SpaceWasm support projection");
        let embedded_projection = registry
            .profiles
            .iter()
            .find(|profile| profile.profile_id == SPACEWASM_PROFILE_ID)
            .and_then(|profile| profile.target_runtime.as_ref())
            .map(|target| &target.support_projection)
            .expect("embedded SpaceWasm support projection");

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(embedded_projection, &checked_projection);
    }

    #[test]
    fn valid_module_and_component_produce_execution_free_facts() {
        let registry = valid_registry();
        let module = wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/positive/module.wat"))
            .expect("module fixture");
        let component = wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/positive/component.wat"))
            .expect("component fixture");

        let module_report = analyze(&module, &registry, &[]);
        let component_report = analyze(&component, &registry, &[]);

        assert!(module_report.details.findings.is_empty());
        assert_eq!(module_report.details.facts.artifact_kind, MODULE_KIND);
        assert!(component_report.details.findings.is_empty());
        assert_eq!(component_report.details.facts.artifact_kind, COMPONENT_KIND);
        assert_eq!(module_report.details.verification_role, RECORDED_ONLY_ROLE);
    }

    #[test]
    fn spacewasm_profile_binds_reviewed_cohort_and_static_resources() {
        let registry = valid_registry();
        let module =
            wat::parse_str(include_str!("../../standards/wasm-artifact/spacewasm/fixtures/positive/mvp-module.wat"))
                .expect("SpaceWasm module fixture");

        let report = analyze_profile(&module, &registry, SPACEWASM_PROFILE_ID, &[]);
        let target = report.details.target_runtime.expect("SpaceWasm target evidence");

        assert!(report.details.findings.is_empty(), "{:?}", report.details.findings);
        assert_eq!(report.details.facts.artifact_kind, MODULE_KIND);
        assert_eq!(report.details.facts.memory_count, 1);
        assert_eq!(report.details.facts.table_count, 1);
        assert_eq!(target.runtime_id, SPACEWASM_RUNTIME_ID);
        assert_eq!(target.verification_role, RECORDED_ONLY_ROLE);
        assert!(is_blake3_identity(&target.cohort_identity));
        assert!(is_blake3_identity(&target.support_projection_identity));
    }

    #[test]
    fn spacewasm_profile_rejects_feature_abi_and_resource_boundaries() {
        let registry = valid_registry();
        let cases = [
            (include_str!("../../standards/wasm-artifact/fixtures/positive/component.wat"), DISALLOWED_KIND_ISSUE),
            (
                include_str!("../../standards/wasm-artifact/spacewasm/fixtures/negative/bulk-memory.wat"),
                DISALLOWED_PROPOSAL_ISSUE,
            ),
            (
                include_str!("../../standards/wasm-artifact/spacewasm/fixtures/negative/nontrapping-conversion.wat"),
                DISALLOWED_PROPOSAL_ISSUE,
            ),
            (
                include_str!("../../standards/wasm-artifact/fixtures/negative/sign-extension.wat"),
                DISALLOWED_PROPOSAL_ISSUE,
            ),
            (
                include_str!("../../standards/wasm-artifact/spacewasm/fixtures/negative/wrong-import.wat"),
                DISALLOWED_IMPORT_ISSUE,
            ),
            (
                include_str!("../../standards/wasm-artifact/spacewasm/fixtures/negative/wrong-abi.wat"),
                ABI_MISMATCH_ISSUE,
            ),
            (
                include_str!("../../standards/wasm-artifact/spacewasm/fixtures/negative/memory-limit.wat"),
                MEMORY_LIMIT_ISSUE,
            ),
            (
                include_str!("../../standards/wasm-artifact/spacewasm/fixtures/negative/table-limit.wat"),
                TABLE_LIMIT_ISSUE,
            ),
            (
                include_str!("../../standards/wasm-artifact/spacewasm/fixtures/negative/parameter-limit.wat"),
                STRUCTURAL_LIMIT_ISSUE,
            ),
            (include_str!("../../standards/wasm-artifact/fixtures/negative/memory-grow.wat"), MEMORY_GROWTH_ISSUE),
            (
                include_str!("../../standards/wasm-artifact/fixtures/negative/start-function.wat"),
                START_FUNCTION_ISSUE,
            ),
        ];

        for (wat, expected_issue) in cases {
            let bytes = wat::parse_str(wat).expect("SpaceWasm negative fixture");
            let report = analyze_profile(&bytes, &registry, SPACEWASM_PROFILE_ID, &[]);
            assert!(
                report.details.findings.iter().any(|finding| finding.issue_code == expected_issue),
                "missing {expected_issue}: {:?}",
                report.details.findings
            );
        }
    }

    #[test]
    fn spacewasm_profile_rejects_stale_cohort_and_target_role_promotion() {
        let mut registry = valid_registry();
        let target = registry
            .profiles
            .iter_mut()
            .find(|profile| profile.profile_id == SPACEWASM_PROFILE_ID)
            .and_then(|profile| profile.target_runtime.as_mut())
            .expect("SpaceWasm target profile");
        target.support_projection.source_revision = "ffffffffffffffffffffffffffffffffffffffff".to_owned();

        let diagnostics = validate_registry(&registry);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("stale target runtime cohort identity")));

        let registry = valid_registry();
        let module =
            wat::parse_str(include_str!("../../standards/wasm-artifact/spacewasm/fixtures/positive/mvp-module.wat"))
                .expect("SpaceWasm module fixture");
        let mut report = analyze_profile(&module, &registry, SPACEWASM_PROFILE_ID, &[]);
        report.details.target_runtime.as_mut().expect("SpaceWasm target evidence").verification_role =
            "runtime_correctness".to_owned();
        let evaluation = evaluate_envelope(&render_envelope(&report).expect("render promoted report"));
        assert_eq!(evaluation.status, WasmEvaluationStatus::Failed);
        assert!(evaluation.diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded_only")));
    }

    #[test]
    fn spacewasm_external_reports_are_recorded_only_and_do_not_change_static_policy() {
        let registry = valid_registry();
        let denied_bytes =
            wat::parse_str(include_str!("../../standards/wasm-artifact/spacewasm/fixtures/negative/wrong-abi.wat"))
                .expect("denied SpaceWasm fixture");
        let static_report = analyze_profile(&denied_bytes, &registry, SPACEWASM_PROFILE_ID, &[]);
        let external = external_report_for(&static_report.details, "chaoscontrol", "executed-successfully");
        let external_text = serde_json::to_string(&external).expect("serialize external report");

        let linked =
            analyze_profile_with_external(&denied_bytes, &registry, SPACEWASM_PROFILE_ID, &[], &[external_text]);

        assert!(linked.details.findings.iter().any(|finding| finding.issue_code == ABI_MISMATCH_ISSUE));
        assert!(!linked.details.findings.iter().any(|finding| finding.issue_code == EXTERNAL_EVIDENCE_ISSUE));
        assert_eq!(linked.details.external_evidence.len(), 1);
        assert_eq!(linked.details.external_evidence[0].verification_role, RECORDED_ONLY_ROLE);

        let mut mismatched = external.clone();
        mismatched.artifact_identity = format!("{BLAKE3_PREFIX}{}", "0".repeat(BLAKE3_HEX_LENGTH));
        mismatched.report_identity = external_report_identity(&mismatched).expect("mismatched identity");
        let mismatched_text = serde_json::to_string(&mismatched).expect("serialize mismatched report");
        let mismatch_rejected =
            analyze_profile_with_external(&denied_bytes, &registry, SPACEWASM_PROFILE_ID, &[], &[mismatched_text]);
        assert!(
            mismatch_rejected
                .details
                .findings
                .iter()
                .any(|finding| finding.issue_code == EXTERNAL_EVIDENCE_ISSUE)
        );

        let mut promoted = external;
        promoted.verification_role = "runtime_correctness".to_owned();
        promoted.report_identity = external_report_identity(&promoted).expect("promoted identity");
        let promoted_text = serde_json::to_string(&promoted).expect("serialize promoted report");
        let rejected =
            analyze_profile_with_external(&denied_bytes, &registry, SPACEWASM_PROFILE_ID, &[], &[promoted_text]);
        assert!(rejected.details.findings.iter().any(|finding| finding.issue_code == EXTERNAL_EVIDENCE_ISSUE));
    }

    #[test]
    fn rejected_external_report_findings_identify_the_failing_report() {
        let bytes =
            wat::parse_str(include_str!("../../standards/wasm-artifact/spacewasm/fixtures/positive/mvp-module.wat"))
                .expect("SpaceWasm fixture");
        let mut registry = valid_registry();
        let profile = registry
            .profiles
            .iter_mut()
            .find(|profile| profile.profile_id == SPACEWASM_PROFILE_ID)
            .expect("SpaceWasm profile");
        "advisory".clone_into(&mut profile.severity.external_evidence);
        let static_report = analyze_profile(&bytes, &registry, SPACEWASM_PROFILE_ID, &[]);
        let verified = external_report_for(&static_report.details, "chaoscontrol", "executed-successfully");
        let mut mismatched = external_report_for(&static_report.details, "mantle", "executed-successfully");
        mismatched.artifact_identity = format!("{BLAKE3_PREFIX}{}", "0".repeat(BLAKE3_HEX_LENGTH));
        mismatched.report_identity = external_report_identity(&mismatched).expect("mismatched identity");
        let texts =
            [&verified, &mismatched].map(|report| serde_json::to_string(report).expect("serialize external report"));

        let report = analyze_profile_with_external(&bytes, &registry, SPACEWASM_PROFILE_ID, &[], &texts);

        assert_eq!(report.details.external_evidence, vec![verified.clone()]);
        let findings = &report.details.findings;
        assert!(!findings.is_empty());
        for finding in findings {
            assert_eq!(finding.issue_code, EXTERNAL_EVIDENCE_ISSUE);
            assert!(finding.message.contains(&mismatched.report_identity), "{}", finding.message);
            assert!(finding.message.contains("mantle"), "{}", finding.message);
            assert!(finding.message.contains("#2"), "{}", finding.message);
            assert!(!finding.message.contains(&verified.report_identity), "{}", finding.message);
        }

        let unparsable = analyze_profile_with_external(&bytes, &registry, SPACEWASM_PROFILE_ID, &[], &[
            texts[0].clone(),
            "{".to_owned(),
        ]);
        assert!(unparsable.details.findings.iter().all(|finding| finding.message.contains("#2")));
    }

    #[test]
    fn rejected_external_reports_take_the_declared_external_evidence_severity() {
        let bytes =
            wat::parse_str(include_str!("../../standards/wasm-artifact/spacewasm/fixtures/positive/mvp-module.wat"))
                .expect("SpaceWasm fixture");
        for (severity, expected_status) in [
            ("advisory", WasmEvaluationStatus::ManualReview),
            ("warning", WasmEvaluationStatus::ManualReview),
            (ERROR_SEVERITY, WasmEvaluationStatus::Failed),
        ] {
            let mut registry = valid_registry();
            let profile = registry
                .profiles
                .iter_mut()
                .find(|profile| profile.profile_id == SPACEWASM_PROFILE_ID)
                .expect("SpaceWasm profile");
            severity.clone_into(&mut profile.severity.external_evidence);
            let static_report = analyze_profile(&bytes, &registry, SPACEWASM_PROFILE_ID, &[]);
            assert!(static_report.details.findings.is_empty(), "{:?}", static_report.details.findings);
            let mut mismatched = external_report_for(&static_report.details, "chaoscontrol", "executed-successfully");
            mismatched.artifact_identity = format!("{BLAKE3_PREFIX}{}", "0".repeat(BLAKE3_HEX_LENGTH));
            mismatched.report_identity = external_report_identity(&mismatched).expect("mismatched identity");
            let mismatched_text = serde_json::to_string(&mismatched).expect("serialize mismatched report");

            let report =
                analyze_profile_with_external(&bytes, &registry, SPACEWASM_PROFILE_ID, &[], &[mismatched_text]);

            assert!(report.details.external_evidence.is_empty(), "{severity}: rejected report must not be recorded");
            let findings = &report.details.findings;
            assert!(!findings.is_empty());
            assert!(
                findings
                    .iter()
                    .all(|finding| finding.issue_code == EXTERNAL_EVIDENCE_ISSUE && finding.severity == severity),
                "{findings:?}"
            );
            let evaluation = evaluate_envelope(&render_envelope(&report).expect("render envelope"));
            assert!(evaluation.diagnostics.is_empty(), "{severity}: {:?}", evaluation.diagnostics);
            assert_eq!(evaluation.status, expected_status, "{severity}");
        }
    }

    #[test]
    fn checked_in_spacewasm_external_reports_bind_current_profile_and_reject_overclaims() {
        let registry = valid_registry();
        let bytes =
            wat::parse_str(include_str!("../../standards/wasm-artifact/spacewasm/fixtures/positive/mvp-module.wat"))
                .expect("SpaceWasm fixture");
        let report = analyze_profile(&bytes, &registry, SPACEWASM_PROFILE_ID, &[]);
        let target = report.details.target_runtime.as_ref();
        let recorded: ExternalRuntimeReport = serde_json::from_str(include_str!(
            "../../standards/wasm-artifact/spacewasm/fixtures/external/chaoscontrol-recorded.json"
        ))
        .expect("recorded external report");
        let overclaim: ExternalRuntimeReport = serde_json::from_str(include_str!(
            "../../standards/wasm-artifact/spacewasm/fixtures/external/overclaim.json"
        ))
        .expect("overclaim external report");

        let recorded_diagnostics = external_report_diagnostics(
            &recorded,
            &report.details.exact_artifact_identity,
            &report.details.profile_identity,
            target,
        );
        let overclaim_diagnostics = external_report_diagnostics(
            &overclaim,
            &report.details.exact_artifact_identity,
            &report.details.profile_identity,
            target,
        );

        assert!(recorded_diagnostics.is_empty(), "{recorded_diagnostics:?}");
        assert!(overclaim_diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded_only")));
    }

    #[test]
    fn spacewasm_corpus_replays_exact_bytes_and_expected_issue_classes() {
        let registry = valid_registry();
        let corpus: serde_json::Value =
            serde_json::from_str(include_str!("../../standards/wasm-artifact/spacewasm/corpus.json"))
                .expect("SpaceWasm corpus");
        let cases = corpus["cases"].as_array().expect("corpus cases");
        let positive_bytes = spacewasm_fixture_bytes("fixtures/generated/mvp-module.wasm");
        let positive_report = analyze_profile(positive_bytes, &registry, SPACEWASM_PROFILE_ID, &[]);

        for case in cases {
            let source = case["source"].as_str().expect("case source");
            let expected_digest = case["artifact_blake3"].as_str().expect("artifact digest");
            let expected_issue = case["expected_issue_class"].as_str().expect("expected issue");
            let artifact_kind = case["artifact_kind"].as_str().expect("artifact kind");
            let bytes = spacewasm_fixture_bytes(source);
            assert_eq!(identity(bytes), format!("{BLAKE3_PREFIX}{expected_digest}"), "{source}");

            if artifact_kind == "wasm" {
                let report = analyze_profile(bytes, &registry, SPACEWASM_PROFILE_ID, &[]);
                if expected_issue == "none" {
                    assert!(report.details.findings.is_empty(), "{source}: {:?}", report.details.findings);
                } else {
                    assert!(
                        report.details.findings.iter().any(|finding| finding.issue_code == expected_issue),
                        "{source} missing {expected_issue}: {:?}",
                        report.details.findings
                    );
                }
            } else {
                let external: ExternalRuntimeReport = serde_json::from_slice(bytes).expect("external report fixture");
                let diagnostics = external_report_diagnostics(
                    &external,
                    &positive_report.details.exact_artifact_identity,
                    &positive_report.details.profile_identity,
                    positive_report.details.target_runtime.as_ref(),
                );
                if expected_issue == "none" {
                    assert!(diagnostics.is_empty(), "{source}: {diagnostics:?}");
                } else {
                    assert!(!diagnostics.is_empty(), "{source} unexpectedly accepted");
                }
            }
        }
    }

    #[test]
    fn malformed_import_growth_start_and_resource_cases_fail_closed() {
        let registry = valid_registry();
        let function_limit = registry.profiles[0].resource_limits.functions;
        let over_function_limit = usize::try_from(function_limit.saturating_add(1)).expect("fixture function count");
        let resource_fixture = format!("(module {})", "(func)".repeat(over_function_limit));
        let cases = [
            (vec![0_u8, 1_u8], MALFORMED_ISSUE),
            (
                wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/sign-extension.wat"))
                    .expect("proposal fixture"),
                DISALLOWED_PROPOSAL_ISSUE,
            ),
            (
                wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/undeclared-wasi.wat"))
                    .expect("import fixture"),
                DISALLOWED_IMPORT_ISSUE,
            ),
            (
                wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/memory-grow.wat"))
                    .expect("growth fixture"),
                MEMORY_GROWTH_ISSUE,
            ),
            (
                wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/start-function.wat"))
                    .expect("start fixture"),
                START_FUNCTION_ISSUE,
            ),
            (wat::parse_str(&resource_fixture).expect("resource fixture"), RESOURCE_LIMIT_ISSUE),
        ];

        for (bytes, expected_issue) in cases {
            let report = analyze(&bytes, &registry, &[]);
            assert!(
                report.details.findings.iter().any(|finding| finding.issue_code == expected_issue),
                "missing {expected_issue}: {:?}",
                report.details.findings
            );
        }
    }

    #[test]
    fn report_bytes_and_finding_order_are_deterministic() {
        let registry = valid_registry();
        let bytes = wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/negative/undeclared-wasi.wat"))
            .expect("negative fixture");

        let first = analyze(&bytes, &registry, &[]);
        let second = analyze(&bytes, &registry, &[]);
        let first_json = render_envelope(&first).expect("render first report");
        let second_json = render_envelope(&second).expect("render second report");
        let finding_keys = first
            .details
            .findings
            .iter()
            .map(|finding| (&finding.issue_code, &finding.message, finding.byte_offset))
            .collect::<Vec<_>>();
        let mut sorted_keys = finding_keys.clone();
        sorted_keys.sort();

        assert_eq!(first_json, second_json);
        assert_eq!(finding_keys, sorted_keys);
    }

    #[test]
    fn exact_and_interface_identities_remain_distinct_roles() {
        let registry = valid_registry();
        let first = module_with_custom("metadata", b"first");
        let second = module_with_custom("metadata", b"second");

        let first_report = analyze(&first, &registry, &[]);
        let second_report = analyze(&second, &registry, &[]);

        assert_ne!(first_report.details.exact_artifact_identity, second_report.details.exact_artifact_identity);
        assert_eq!(first_report.details.interface_identity, second_report.details.interface_identity);
        assert_eq!(first_report.details.exact_identity_role, "exact-bytes");
        assert_eq!(first_report.details.interface_identity_role, "normalized-interface-non-equivalence");
    }

    #[test]
    fn duplicate_metadata_and_disallowed_world_policy_are_negative_fixtures() {
        let registry = valid_registry();
        let duplicate = module_with_duplicate_custom("producers");
        let duplicate_report = analyze(&duplicate, &registry, &[]);
        assert!(
            duplicate_report
                .details
                .findings
                .iter()
                .any(|finding| { finding.issue_code == DUPLICATE_METADATA_ISSUE })
        );

        let mut profile = registry.profiles[0].clone();
        profile.allowed_worlds = vec!["example:allowed/world".to_owned()];
        let facts = WasmArtifactFacts {
            artifact_kind: COMPONENT_KIND.to_owned(),
            proposals: vec![COMPONENT_MODEL_PROPOSAL.to_owned()],
            worlds: vec!["example:other/world".to_owned()],
            ..WasmArtifactFacts::default()
        };
        let findings = evaluate_policy(&[], &facts, &profile);
        assert!(findings.iter().any(|finding| finding.issue_code == DISALLOWED_WORLD_ISSUE));

        profile.allow_any_exports = false;
        profile.producer_metadata = "reject".to_owned();
        let restricted_facts = WasmArtifactFacts {
            artifact_kind: MODULE_KIND.to_owned(),
            proposals: vec![WASM1_PROPOSAL.to_owned()],
            exports: vec![WasmInterfaceItem {
                namespace: MODULE_KIND.to_owned(),
                name: "run".to_owned(),
                kind: "Func".to_owned(),
            }],
            producer_sections: 1,
            ..WasmArtifactFacts::default()
        };
        let restricted_findings = evaluate_policy(&[], &restricted_facts, &profile);
        assert!(restricted_findings.iter().any(|finding| finding.issue_code == DISALLOWED_EXPORT_ISSUE));
        assert!(restricted_findings.iter().any(|finding| finding.issue_code == PRODUCER_METADATA_ISSUE));
    }

    #[test]
    fn mantle_handoff_binds_exact_bytes_without_claim_transfer() {
        let registry = valid_registry();
        let module = wat::parse_str("(module)").expect("module fixture");

        let report = analyze(&module, &registry, &[MANTLE_PARENT.to_owned()]);

        assert_eq!(report.details.handoff_role, MANTLE_HANDOFF);
        assert_eq!(report.details.handoff_parent_identities, vec![MANTLE_PARENT.to_owned()]);
        assert_eq!(report.details.input_artifact, INPUT_ARTIFACT_NAME);
        assert!(report.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM));
    }

    #[test]
    fn receipt_validation_rejects_identity_confusion_and_role_promotion() {
        let registry = valid_registry();
        let module = wat::parse_str("(module)").expect("module fixture");
        let mut report = analyze(&module, &registry, &[]);
        report.details.exact_identity_role = "normalized-interface".to_owned();
        report.details.verification_role = "formal_proof".to_owned();

        let evaluation = evaluate_envelope(&render_envelope(&report).expect("render envelope"));

        assert_eq!(evaluation.status, WasmEvaluationStatus::Failed);
        assert!(evaluation.diagnostics.iter().any(|diagnostic| diagnostic.contains("exact-bytes")));
        assert!(evaluation.diagnostics.iter().any(|diagnostic| diagnostic.contains("recorded_only or boundary")));
    }

    #[test]
    fn non_handoff_parent_artifacts_keep_standalone_role() {
        let registry = valid_registry();
        let module = wat::parse_str("(module)").expect("module fixture");
        let parents = [
            "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
            "artifacts/octet/status.json".to_owned(),
        ];

        let report = analyze(&module, &registry, &parents);

        assert_eq!(report.details.handoff_role, STANDALONE_HANDOFF);
        assert!(report.details.handoff_parent_identities.is_empty());
        let evaluation = evaluate_envelope(&render_envelope(&report).expect("render envelope"));
        assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    }

    #[test]
    fn external_orchestrator_handoff_binds_valid_external_identities() {
        let registry = valid_registry();
        let module = wat::parse_str("(module)").expect("module fixture");
        let parents = [
            "chaoscontrol:fixture-run".to_owned(),
            "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        ];

        let report = analyze(&module, &registry, &parents);

        assert_eq!(report.details.handoff_role, EXTERNAL_HANDOFF);
        assert_eq!(report.details.handoff_parent_identities, vec!["chaoscontrol:fixture-run".to_owned()]);
        let evaluation = evaluate_envelope(&render_envelope(&report).expect("render envelope"));
        assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    }

    #[test]
    fn receipt_validation_rejects_handoff_role_identity_mismatches() {
        let registry = valid_registry();
        let module = wat::parse_str("(module)").expect("module fixture");
        let cases: [(&str, &[&str]); 5] = [
            (EXTERNAL_HANDOFF, &[]),
            (EXTERNAL_HANDOFF, &[MANTLE_PARENT]),
            (STANDALONE_HANDOFF, &["molten:fixture-run"]),
            (MANTLE_HANDOFF, &["chaoscontrol:fixture-run"]),
            (MANTLE_HANDOFF, &[]),
        ];
        for (role, identities) in cases {
            let mut report = analyze(&module, &registry, &[]);
            role.clone_into(&mut report.details.handoff_role);
            report.details.handoff_parent_identities = identities.iter().map(|&identity| identity.to_owned()).collect();

            let evaluation = evaluate_envelope(&render_envelope(&report).expect("render envelope"));

            assert_eq!(evaluation.status, WasmEvaluationStatus::Failed, "{role} {identities:?}");
            assert!(
                evaluation
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.contains("does not match its parent identities")),
                "{role} {identities:?}: {:?}",
                evaluation.diagnostics
            );
        }
    }

    #[test]
    fn receipt_validation_recomputes_interface_identity_from_facts() {
        let registry = valid_registry();
        let module = wat::parse_str("(module)").expect("module fixture");
        let mut report = analyze(&module, &registry, &[]);
        report.details.facts.imports.push(WasmInterfaceItem {
            namespace: "env".to_owned(),
            name: "injected".to_owned(),
            kind: "func".to_owned(),
        });

        let evaluation = evaluate_envelope(&render_envelope(&report).expect("render envelope"));

        assert_eq!(evaluation.status, WasmEvaluationStatus::Failed);
        assert!(
            evaluation
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("interface_identity does not match"))
        );

        let module =
            wat::parse_str(include_str!("../../standards/wasm-artifact/spacewasm/fixtures/positive/mvp-module.wat"))
                .expect("SpaceWasm module fixture");
        let mut report = analyze_profile(&module, &registry, SPACEWASM_PROFILE_ID, &[]);
        let untampered = evaluate_envelope(&render_envelope(&report).expect("render envelope"));
        assert!(untampered.diagnostics.is_empty(), "{:?}", untampered.diagnostics);
        report
            .details
            .facts
            .function_abis
            .first_mut()
            .expect("SpaceWasm function ABI")
            .results
            .push("i64".to_owned());

        let evaluation = evaluate_envelope(&render_envelope(&report).expect("render envelope"));

        assert_eq!(evaluation.status, WasmEvaluationStatus::Failed);
        assert!(
            evaluation
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("interface_identity does not match"))
        );
    }

    #[test]
    fn registry_rejects_unknown_severity_and_duplicate_section_postures() {
        let mut registry = valid_registry();
        "Error".clone_into(&mut registry.profiles[0].severity.malformed);
        "fatal".clone_into(&mut registry.profiles[0].severity.external_evidence);
        "Reject".clone_into(&mut registry.profiles[0].duplicate_custom_sections);

        let diagnostics = validate_registry(&registry);

        for expected in [
            "unsupported malformed severity `Error`",
            "unsupported external_evidence severity `fatal`",
            "unsupported duplicate custom section posture `Reject`",
        ] {
            assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)), "{expected}: {diagnostics:?}");
        }
    }

    #[test]
    fn cohort_drift_fails_registry_validation_without_a_severity_knob() {
        let mut value: Value = serde_json::from_str(PROFILE_JSON).expect("profile registry JSON");
        value["profiles"][0]["severity"]["cohort_drift"] = Value::String("advisory".to_owned());
        let error = parse_registry(&value.to_string()).expect_err("cohort_drift is not a severity");
        assert!(error.contains("unknown field `cohort_drift`"), "{error}");

        let mut registry = valid_registry();
        registry.cohort.members[0].version = "0.254.0".to_owned();
        for profile in &mut registry.profiles {
            for severity in [
                &mut profile.severity.malformed,
                &mut profile.severity.disallowed_kind,
                &mut profile.severity.disallowed_import,
                &mut profile.severity.disallowed_world,
                &mut profile.severity.resource_limit,
                &mut profile.severity.growth,
                &mut profile.severity.start,
                &mut profile.severity.duplicate_metadata,
                &mut profile.severity.abi_mismatch,
                &mut profile.severity.external_evidence,
            ] {
                "advisory".clone_into(severity);
            }
        }
        let module = wat::parse_str("(module)").expect("module fixture");
        let diagnostics =
            analyze_bytes(&module, &registry, PROFILE_ID, identity(PROFILE_JSON.as_bytes()), &[]).expect_err("drift");
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("stale wasm-tools cohort identity")));
    }

    #[test]
    fn registry_cohort_must_claim_the_linked_wasm_tools_versions() {
        let module = wat::parse_str("(module)").expect("module fixture");
        let registry = valid_registry();
        assert!(validate_registry(&registry).is_empty());
        analyze_bytes(&module, &registry, PROFILE_ID, identity(PROFILE_JSON.as_bytes()), &[]).expect("linked cohort");

        for (crate_name, _) in LINKED_COHORT_MEMBERS {
            // A self-consistent registry whose identity matches its own claimed members.
            let mut claimed = valid_registry();
            claimed
                .cohort
                .members
                .iter_mut()
                .find(|member| member.crate_name == crate_name)
                .expect("linked cohort member")
                .version = "0.300.0".to_owned();
            claimed.cohort.identity = cohort_identity(&claimed.cohort.members);
            let diagnostics = analyze_bytes(&module, &claimed, PROFILE_ID, identity(PROFILE_JSON.as_bytes()), &[])
                .expect_err("claimed cohort differs from the linked build");
            assert!(
                diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.contains(crate_name) && diagnostic.contains("0.300.0")),
                "{crate_name}: {diagnostics:?}"
            );

            let mut missing = valid_registry();
            missing.cohort.members.retain(|member| member.crate_name != crate_name);
            missing.cohort.identity = cohort_identity(&missing.cohort.members);
            let diagnostics = validate_registry(&missing);
            assert!(
                diagnostics.iter().any(|diagnostic| diagnostic.contains(crate_name)),
                "{crate_name}: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn linked_cohort_versions_match_cargo_lock() {
        let lock: toml::Table = toml::from_str(include_str!("../../Cargo.lock")).expect("parse Cargo.lock");
        let locked_versions = |crate_name: &str| {
            lock["package"]
                .as_array()
                .expect("Cargo.lock packages")
                .iter()
                .filter(|package| package["name"].as_str() == Some(crate_name))
                .map(|package| package["version"].as_str().expect("locked version").to_owned())
                .collect::<Vec<_>>()
        };
        for (crate_name, version) in LINKED_COHORT_MEMBERS {
            assert_eq!(locked_versions(crate_name), [version], "{crate_name}");
        }
        // `wat` builds the text fixtures; its `1.N.P` release ships with wasm-tools `0.N.P`.
        let (_, wasmparser_version) = LINKED_COHORT_MEMBERS
            .into_iter()
            .find(|(crate_name, _)| *crate_name == "wasmparser")
            .expect("wasmparser is linked");
        let release = wasmparser_version.strip_prefix("0.").expect("wasm-tools 0.N.P version");
        assert_eq!(locked_versions("wat"), [format!("1.{release}")]);
    }

    #[test]
    fn registry_rejects_unknown_fields_duplicate_rules_and_invalid_worlds() {
        let checked: Value = serde_json::from_str(PROFILE_JSON).expect("profile registry JSON");
        let spacewasm = valid_registry()
            .profiles
            .iter()
            .position(|profile| profile.profile_id == SPACEWASM_PROFILE_ID)
            .expect("SpaceWasm profile");
        let target = format!("/profiles/{spacewasm}/target_runtime");
        let projection = format!("{target}/support_projection");
        for pointer in [
            "",
            "/cohort",
            "/cohort/members/0",
            "/profiles/0",
            "/profiles/0/resource_limits",
            "/profiles/0/severity",
            &target,
            &projection,
        ] {
            let mut value = checked.clone();
            value
                .pointer_mut(pointer)
                .and_then(Value::as_object_mut)
                .expect("registry object")
                .insert("unreviewed".to_owned(), Value::Bool(true));
            let error = parse_registry(&value.to_string()).expect_err(pointer);
            assert!(error.contains("unknown field `unreviewed`"), "{pointer}: {error}");
        }

        let mut registry = valid_registry();
        let profile = &mut registry.profiles[0];
        profile.allowed_proposals.push(WASM1_PROPOSAL.to_owned());
        profile.allowed_imports = vec!["env::f::Func(0)".to_owned(), "env::f::Func(0)".to_owned()];
        profile.required_imports = vec!["env::f::Func(0)".to_owned(), "env::f::Func(0)".to_owned()];
        profile.required_exports = vec!["module::run::Func".to_owned(), "module::run::Func".to_owned()];
        profile.allowed_worlds = vec![
            "example:allowed/world".to_owned(),
            "wasi:cli/command@0.2.0-rc.1+build.7".to_owned(),
            "example:allowed/world".to_owned(),
            "not a world".to_owned(),
            "Example:mixedCase/world".to_owned(),
            "example:pkg/world@1.0".to_owned(),
            "example:pkg/nested/world".to_owned(),
        ];
        let diagnostics = validate_registry(&registry);
        for expected in [
            "repeats allowed_proposals entry `wasm1`",
            "repeats allowed_imports entry `env::f::Func(0)`",
            "repeats required_imports entry `env::f::Func(0)`",
            "repeats required_exports entry `module::run::Func`",
            "repeats allowed_worlds entry `example:allowed/world`",
            "names invalid world `not a world`",
            "names invalid world `Example:mixedCase/world`",
            "names invalid world `example:pkg/world@1.0`",
            "names invalid world `example:pkg/nested/world`",
        ] {
            assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)), "{expected}: {diagnostics:?}");
        }
        for valid in ["`example:allowed/world`", "`wasi:cli/command@0.2.0-rc.1+build.7`"] {
            assert!(
                !diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.contains("invalid world") && diagnostic.contains(valid)),
                "{valid}: {diagnostics:?}"
            );
        }
    }

    #[test]
    fn parse_stage_findings_use_their_issue_severity() {
        let mut registry = valid_registry();
        for profile in &mut registry.profiles {
            "advisory".clone_into(&mut profile.severity.malformed);
            "warning".clone_into(&mut profile.severity.resource_limit);
            "warning".clone_into(&mut profile.severity.abi_mismatch);
        }
        let severity_of = |report: &WasmArtifactEnvelope, issue_code: &str| {
            report
                .details
                .findings
                .iter()
                .find(|finding| finding.issue_code == issue_code)
                .map(|finding| finding.severity.clone())
        };

        // Instantiating an imported core module cannot be resource-bounded.
        let unbounded = wat::parse_str("(component (import \"m\" (core module $m)) (core instance (instantiate $m)))")
            .expect("unbounded instantiation fixture");
        let report = analyze(&unbounded, &registry, &[]);
        assert_eq!(severity_of(&report, RESOURCE_LIMIT_ISSUE).as_deref(), Some("warning"), "{report:?}");

        // Exporting a missing function fails validation and ABI resolution.
        let mut missing_export = b"\0asm\x01\0\0\0".to_vec();
        missing_export.extend_from_slice(&[7, 7, 1, 3, b'r', b'u', b'n', 0, 0]);
        let report = analyze_profile(&missing_export, &registry, SPACEWASM_PROFILE_ID, &[]);
        assert_eq!(severity_of(&report, ABI_MISMATCH_ISSUE).as_deref(), Some("warning"), "{report:?}");
        assert_eq!(severity_of(&report, MALFORMED_ISSUE).as_deref(), Some("advisory"), "{report:?}");
    }

    #[test]
    fn sarif_projection_uses_sarif_levels_and_omits_unknown_regions() {
        let mut located = finding(MALFORMED_ISSUE, ERROR_SEVERITY, "located".to_owned());
        located.byte_offset = Some(8);
        let findings = [
            finding(MEMORY_GROWTH_ISSUE, "advisory", "advisory".to_owned()),
            finding(START_FUNCTION_ISSUE, "warning", "warning".to_owned()),
            located,
        ];
        let sarif = render_sarif(&findings);
        let results = sarif["runs"][0]["results"].as_array().expect("SARIF results");
        let levels = results.iter().map(|result| result["level"].as_str()).collect::<Vec<_>>();
        assert_eq!(levels, [Some("note"), Some("warning"), Some("error")]);
        let location = |index: usize| &results[index]["locations"][0]["physicalLocation"];
        assert!(location(0).get("region").is_none(), "{sarif}");
        assert_eq!(location(0)["artifactLocation"]["uri"], INPUT_ARTIFACT_NAME);
        assert_eq!(location(2)["region"]["byteOffset"], 8);
    }

    fn analyze(bytes: &[u8], registry: &WasmProfileRegistry, parents: &[String]) -> WasmArtifactEnvelope {
        analyze_profile(bytes, registry, PROFILE_ID, parents)
    }

    fn analyze_profile(
        bytes: &[u8],
        registry: &WasmProfileRegistry,
        profile_id: &str,
        parents: &[String],
    ) -> WasmArtifactEnvelope {
        analyze_bytes(bytes, registry, profile_id, identity(PROFILE_JSON.as_bytes()), parents).expect("analysis")
    }

    fn analyze_profile_with_external(
        bytes: &[u8],
        registry: &WasmProfileRegistry,
        profile_id: &str,
        parents: &[String],
        external_reports: &[String],
    ) -> WasmArtifactEnvelope {
        analyze_bytes_with_external(
            bytes,
            registry,
            profile_id,
            identity(PROFILE_JSON.as_bytes()),
            parents,
            external_reports,
        )
        .expect("analysis with external reports")
    }

    fn external_report_for(
        details: &WasmArtifactReceiptDetails,
        producer: &str,
        result: &str,
    ) -> ExternalRuntimeReport {
        let mut report = ExternalRuntimeReport {
            schema_version: EXTERNAL_RUNTIME_SCHEMA.to_owned(),
            producer: producer.to_owned(),
            artifact_identity: details.exact_artifact_identity.clone(),
            runtime_cohort_identity: details
                .target_runtime
                .as_ref()
                .expect("target runtime evidence")
                .cohort_identity
                .clone(),
            profile_identity: details.profile_identity.clone(),
            result: result.to_owned(),
            verification_role: RECORDED_ONLY_ROLE.to_owned(),
            non_claims: vec![EXTERNAL_RUNTIME_NON_CLAIM.to_owned()],
            report_identity: String::new(),
        };
        report.report_identity = external_report_identity(&report).expect("external report identity");
        report
    }

    fn spacewasm_fixture_bytes(source: &str) -> &'static [u8] {
        match source {
            "fixtures/generated/mvp-module.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/mvp-module.wasm")
            }
            "fixtures/generated/component.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/component.wasm")
            }
            "fixtures/generated/bulk-memory.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/bulk-memory.wasm")
            }
            "fixtures/generated/sign-extension.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/sign-extension.wasm")
            }
            "fixtures/generated/nontrapping-conversion.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/nontrapping-conversion.wasm")
            }
            "fixtures/generated/wrong-import.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/wrong-import.wasm")
            }
            "fixtures/generated/wrong-abi.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/wrong-abi.wasm")
            }
            "fixtures/generated/memory-limit.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/memory-limit.wasm")
            }
            "fixtures/generated/table-limit.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/table-limit.wasm")
            }
            "fixtures/generated/parameter-limit.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/parameter-limit.wasm")
            }
            "fixtures/generated/memory-grow.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/memory-grow.wasm")
            }
            "fixtures/generated/start-function.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/start-function.wasm")
            }
            "fixtures/generated/smith-fast.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/smith-fast.wasm")
            }
            "fixtures/generated/mutated-mvp.wasm" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/generated/mutated-mvp.wasm")
            }
            "fixtures/external/chaoscontrol-recorded.json" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/external/chaoscontrol-recorded.json")
            }
            "fixtures/external/overclaim.json" => {
                include_bytes!("../../standards/wasm-artifact/spacewasm/fixtures/external/overclaim.json")
            }
            _ => panic!("unknown SpaceWasm corpus fixture `{source}`"),
        }
    }

    fn valid_registry() -> WasmProfileRegistry {
        parse_registry(PROFILE_JSON).expect("checked-in profile registry")
    }

    fn module_with_custom(name: &str, data: &[u8]) -> Vec<u8> {
        let mut module = Module::new();
        module.section(&CustomSection {
            name: Cow::Owned(name.to_owned()),
            data: Cow::Owned(data.to_vec()),
        });
        module.finish()
    }

    #[test]
    fn molten_profile_validates_bulk_memory_and_padded_indirect_calls_without_reference_types() {
        let registry = valid_registry();
        assert!(validate_registry(&registry).is_empty());
        let profile = registry
            .profiles
            .iter()
            .find(|profile| profile.profile_id == "molten-component-baseline")
            .expect("molten profile");
        let features = validation_features(profile);
        let bulk_memory = wat::parse_str("(module (memory 1) (func i32.const 0 i32.const 0 i32.const 0 memory.copy))")
            .expect("bulk-memory fixture");
        Validator::new_with_features(features).validate_all(&bulk_memory).expect("declared bulk memory");
        // One () -> () function and funcref table; call_indirect uses a padded
        // LEB-encoded zero table index, as emitted by the reviewed LLVM linker.
        let padded_call = b"\0asm\x01\0\0\0\x01\x04\x01\x60\0\0\x03\x02\x01\0\x04\x04\x01\x70\0\x01\x0a\x0a\x01\x08\0\x41\0\x11\0\x80\0\x0b";
        Validator::new_with_features(features)
            .validate_all(padded_call)
            .expect("declared padded table immediate");
        let without_encoding = features & !WasmFeatures::CALL_INDIRECT_OVERLONG;
        assert!(Validator::new_with_features(without_encoding).validate_all(padded_call).is_err());
        let references =
            wat::parse_str("(module (func (result externref) ref.null extern))").expect("reference fixture");
        assert!(Validator::new_with_features(features).validate_all(&references).is_err());
        let simd = wat::parse_str("(module (func (result v128) v128.const i32x4 0 0 0 0))").expect("SIMD fixture");
        assert!(Validator::new_with_features(features).validate_all(&simd).is_err());
        let baseline = registry.profiles.iter().find(|profile| profile.profile_id == PROFILE_ID).expect("baseline");
        assert!(Validator::new_with_features(validation_features(baseline)).validate_all(&bulk_memory).is_err());
        assert!(Validator::new_with_features(validation_features(baseline)).validate_all(padded_call).is_err());
        let mut unknown = profile.clone();
        unknown.allowed_proposals = vec![String::from("call-indirect-overlong-unknown")];
        let mut diagnostics = Vec::new();
        validate_profiles(&[unknown], &mut diagnostics);
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains("names unknown proposal")),
            "{diagnostics:?}"
        );
    }

    fn module_with_duplicate_custom(name: &str) -> Vec<u8> {
        let mut module = Module::new();
        let first = CustomSection {
            name: Cow::Owned(name.to_owned()),
            data: Cow::Borrowed(b"first"),
        };
        let second = CustomSection {
            name: Cow::Owned(name.to_owned()),
            data: Cow::Borrowed(b"second"),
        };
        module.section(&first);
        module.section(&second);
        module.finish()
    }
}
