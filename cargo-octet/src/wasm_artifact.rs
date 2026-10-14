use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
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
const INTERFACE_IDENTITY_DOMAIN: &[u8] = b"octet-wasm-interface/v1\0";
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
pub(crate) struct WasmProfileRegistry {
    pub(crate) schema_version: String,
    pub(crate) cohort: WasmToolsCohort,
    pub(crate) profiles: Vec<WasmArtifactProfile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmToolsCohort {
    pub(crate) cohort_id: String,
    pub(crate) identity: String,
    pub(crate) members: Vec<WasmToolsCohortMember>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(crate) struct WasmToolsCohortMember {
    pub(crate) crate_name: String,
    pub(crate) version: String,
    pub(crate) role: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct WasmArtifactProfile {
    pub(crate) profile_id: String,
    pub(crate) summary: String,
    pub(crate) allowed_artifact_kinds: Vec<String>,
    pub(crate) allowed_proposals: Vec<String>,
    pub(crate) allowed_worlds: Vec<String>,
    pub(crate) allowed_imports: Vec<String>,
    pub(crate) allowed_exports: Vec<String>,
    pub(crate) required_exports: Vec<String>,
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
pub(crate) struct TargetRuntimeCohort {
    pub(crate) cohort_identity: String,
    pub(crate) support_projection: RuntimeSupportProjection,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
pub(crate) struct WasmResourceLimits {
    pub(crate) max_artifact_bytes: u64,
    pub(crate) max_imports: u64,
    pub(crate) max_exports: u64,
    pub(crate) max_memories: u64,
    pub(crate) max_tables: u64,
    pub(crate) max_functions: u64,
    pub(crate) max_memory_initial_pages: u64,
    pub(crate) max_memory_maximum_pages: u64,
    pub(crate) max_table_initial_elements: u64,
    pub(crate) max_table_maximum_elements: u64,
    pub(crate) max_function_parameter_words: u64,
    pub(crate) max_local_words_per_function: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
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
    pub(crate) cohort_drift: String,
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
            severity: if diagnostic.issue_code == DISALLOWED_PROPOSAL_ISSUE {
                profile.severity.disallowed_kind.clone()
            } else {
                profile.severity.malformed.clone()
            },
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
    let status = if !diagnostics.is_empty()
        || envelope.details.findings.iter().any(|finding| finding.severity == ERROR_SEVERITY)
    {
        WasmEvaluationStatus::Failed
    } else if envelope.details.findings.is_empty() {
        WasmEvaluationStatus::Passed
    } else {
        WasmEvaluationStatus::ManualReview
    };
    diagnostics.sort();
    diagnostics.dedup();
    WasmEnvelopeEvaluation {
        details: Some(envelope.details),
        non_claims: envelope.non_claims,
        diagnostics,
        status,
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
    for text in report_texts {
        let report: ExternalRuntimeReport = match serde_json::from_str(text) {
            Ok(report) => report,
            Err(error) => {
                findings.push(finding(
                    EXTERNAL_EVIDENCE_ISSUE,
                    severity,
                    format!("parse external runtime report: {error}"),
                ));
                continue;
            }
        };
        let diagnostics =
            external_report_diagnostics(&report, exact_artifact_identity, profile_identity, target_runtime);
        for diagnostic in diagnostics {
            findings.push(finding(EXTERNAL_EVIDENCE_ISSUE, severity, diagnostic));
        }
        reports.push(report);
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
    if !matches!(details.handoff_role.as_str(), STANDALONE_HANDOFF | MANTLE_HANDOFF | EXTERNAL_HANDOFF) {
        diagnostics.push(format!("unsupported Wasm artifact handoff role `{}`", details.handoff_role));
    }
    if details.handoff_role == STANDALONE_HANDOFF && !details.handoff_parent_identities.is_empty() {
        diagnostics.push("standalone Wasm handoff must not claim external parent identities".to_owned());
    }
    if details.handoff_role == MANTLE_HANDOFF
        && !details.handoff_parent_identities.iter().any(|identity| identity.starts_with(MANTLE_PARENT_PREFIX))
    {
        diagnostics.push("Mantle Wasm handoff requires a mantle: parent identity".to_owned());
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
    extraction.facts.artifact_kind = UNKNOWN_KIND.to_owned();
    let mut validator = Validator::new_with_features(validation_features(profile));
    if let Err(error) = validator.validate_all(bytes) {
        let issue_code = if error.missing_wasm_feature().is_some() {
            DISALLOWED_PROPOSAL_ISSUE
        } else {
            MALFORMED_ISSUE
        };
        extraction.parse_diagnostics.push(ParseDiagnostic {
            issue_code,
            message: format!("wasmparser validation failed: {error}"),
            byte_offset: u64::try_from(error.offset()).ok(),
        });
    }
    for payload in Parser::new(0).parse_all(bytes) {
        let payload = match payload {
            Ok(payload) => payload,
            Err(error) => {
                push_parse_error(&mut extraction, "wasmparser parse failed", &error);
                break;
            }
        };
        extract_payload(payload, &mut extraction);
    }
    normalize_facts(&mut extraction.facts);
    extraction
}

fn validation_features(profile: &WasmArtifactProfile) -> WasmFeatures {
    let mut features = WasmFeatures::empty();
    for proposal in &profile.allowed_proposals {
        features |= match proposal.as_str() {
            WASM1_PROPOSAL => WasmFeatures::WASM1,
            WASM2_PROPOSAL => WasmFeatures::WASM2,
            WASM3_PROPOSAL => WasmFeatures::WASM3,
            COMPONENT_MODEL_PROPOSAL => WasmFeatures::COMPONENT_MODEL,
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
            extraction.facts.artifact_kind = match encoding {
                Encoding::Module => MODULE_KIND,
                Encoding::Component => COMPONENT_KIND,
            }
            .to_owned();
        }
        Payload::ImportSection(reader) => {
            for import in reader.into_imports() {
                match import {
                    Ok(import) => {
                        match import.ty {
                            TypeRef::Func(_) => extraction.facts.function_count += 1,
                            TypeRef::Memory(memory) => {
                                extraction.facts.memory_count += 1;
                                extraction.facts.memories.push(memory_declaration(memory));
                            }
                            TypeRef::Table(table) => {
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
                        let words = function_type.params().iter().map(value_type_words).sum::<u64>();
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
        Payload::CodeSectionEntry(body) => {
            match body.get_locals_reader() {
                Ok(locals) => {
                    let mut local_words = 0_u64;
                    for local in locals {
                        match local {
                            Ok((count, value_type)) => {
                                local_words = local_words
                                    .saturating_add(u64::from(count).saturating_mul(value_type_words(&value_type)));
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

fn value_type_words(value_type: &ValType) -> u64 {
    match value_type {
        ValType::I64 | ValType::F64 => WIDE_VALUE_WORDS,
        ValType::V128 => SIMD_VALUE_WORDS,
        _ => 1,
    }
}

fn normalize_facts(facts: &mut WasmArtifactFacts) {
    facts.imports.sort();
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
    for import in &facts.imports {
        let identity = import_identity(import);
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
    for required_export in &profile.required_exports {
        if !facts.exports.iter().any(|export| import_identity(export) == *required_export) {
            findings.push(finding(
                ABI_MISMATCH_ISSUE,
                &profile.severity.abi_mismatch,
                format!("required export `{required_export}` is missing for profile `{}`", profile.profile_id),
            ));
        }
    }
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
    let artifact_bytes = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    push_limit_finding(
        &mut findings,
        "artifact bytes",
        artifact_bytes,
        profile.resource_limits.max_artifact_bytes,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "imports",
        count(&facts.imports),
        profile.resource_limits.max_imports,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "exports",
        count(&facts.exports),
        profile.resource_limits.max_exports,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "memories",
        facts.memory_count,
        profile.resource_limits.max_memories,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "tables",
        facts.table_count,
        profile.resource_limits.max_tables,
        &profile.severity.resource_limit,
    );
    push_limit_finding(
        &mut findings,
        "functions",
        facts.function_count,
        profile.resource_limits.max_functions,
        &profile.severity.resource_limit,
    );
    for memory in &facts.memories {
        push_typed_limit_finding(
            &mut findings,
            MEMORY_LIMIT_ISSUE,
            "memory initial pages",
            memory.initial_pages,
            profile.resource_limits.max_memory_initial_pages,
            &profile.severity.resource_limit,
        );
        if let Some(maximum) = memory.maximum_pages {
            push_typed_limit_finding(
                &mut findings,
                MEMORY_LIMIT_ISSUE,
                "memory maximum pages",
                maximum,
                profile.resource_limits.max_memory_maximum_pages,
                &profile.severity.resource_limit,
            );
        }
    }
    for table in &facts.tables {
        push_typed_limit_finding(
            &mut findings,
            TABLE_LIMIT_ISSUE,
            "table initial elements",
            table.initial_elements,
            profile.resource_limits.max_table_initial_elements,
            &profile.severity.resource_limit,
        );
        if let Some(maximum) = table.maximum_elements {
            push_typed_limit_finding(
                &mut findings,
                TABLE_LIMIT_ISSUE,
                "table maximum elements",
                maximum,
                profile.resource_limits.max_table_maximum_elements,
                &profile.severity.resource_limit,
            );
        }
    }
    push_typed_limit_finding(
        &mut findings,
        STRUCTURAL_LIMIT_ISSUE,
        "function parameter words",
        facts.max_function_parameter_words,
        profile.resource_limits.max_function_parameter_words,
        &profile.severity.resource_limit,
    );
    push_typed_limit_finding(
        &mut findings,
        STRUCTURAL_LIMIT_ISSUE,
        "local words per function",
        facts.max_local_words_per_function,
        profile.resource_limits.max_local_words_per_function,
        &profile.severity.resource_limit,
    );
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
    findings
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
            if !matches!(proposal.as_str(), WASM1_PROPOSAL | WASM2_PROPOSAL | WASM3_PROPOSAL | COMPONENT_MODEL_PROPOSAL)
            {
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
        (
            "memory pages",
            support.maximum_linear_memory_pages,
            profile.resource_limits.max_memory_maximum_pages,
        ),
        (
            "function parameter words",
            support.maximum_function_parameter_words,
            profile.resource_limits.max_function_parameter_words,
        ),
        (
            "local words per function",
            support.maximum_local_words_per_function,
            profile.resource_limits.max_local_words_per_function,
        ),
        ("function index", support.maximum_function_index, profile.resource_limits.max_functions),
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
            serde_json::json!({
                "ruleId": finding.issue_code,
                "level": finding.severity,
                "message": {"text": finding.message},
                "locations": [{"physicalLocation": {
                    "artifactLocation": {"uri": INPUT_ARTIFACT_NAME},
                    "region": finding.byte_offset.map(|offset| serde_json::json!({"byteOffset": offset})).unwrap_or_else(|| serde_json::json!({}))
                }}]
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
    if parent_artifacts.is_empty() {
        STANDALONE_HANDOFF
    } else if parent_artifacts.iter().any(|parent| parent.starts_with(MANTLE_PARENT_PREFIX)) {
        MANTLE_HANDOFF
    } else {
        EXTERNAL_HANDOFF
    }
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
        let function_limit = registry.profiles[0].resource_limits.max_functions;
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
