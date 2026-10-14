use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::cap_fs::ArtifactRoot;
use crate::cap_fs::RootRelativePath;
use crate::cli::ArtifactArgs;
use crate::cli::OutputFormat;
use crate::evidence::EvidenceTier;
use crate::evidence::ProvenanceRecord;

const REPORT_SCHEMA_VERSION: u32 = 1;
const EXIT_VALID: i32 = 0;
const EXIT_INVALID: i32 = 1;
const COMMAND_ARTIFACT_NAME: &str = "command.txt";
const STATUS_ARTIFACT_NAME: &str = "status.json";
const SUMMARY_ARTIFACT_NAME: &str = "summary.txt";
const JSON_RESULTS_ARTIFACT_NAME: &str = "results.jsonl";
const SARIF_RESULTS_ARTIFACT_NAME: &str = "results.sarif";
const PROVENANCE_ARTIFACT_NAME: &str = "provenance.jsonl";
const PROOF_CARDS_ARTIFACT_NAME: &str = "proof-cards.json";
const FUNCTION_ADDRESS_ARTIFACT_NAME: &str = "function-addresses.json";
const ARCHITECTURE_IR_ARTIFACT_NAME: &str = "compiler-architecture-ir.json";
const ARCHITECTURE_COVERAGE_ARTIFACT_NAME: &str = "compiler-architecture-coverage.json";
const ARCHITECTURE_ISSUES_ARTIFACT_NAME: &str = "compiler-architecture-issues.json";
const ARCHITECTURE_RECEIPT_ARTIFACT_NAME: &str = "compiler-architecture-receipt.json";
const PROJECT_ARCHITECTURE_POLICY_SOURCE_ARTIFACT_NAME: &str = "project-architecture-policy-source.ncl";
const PROJECT_ARCHITECTURE_POLICY_ARTIFACT_NAME: &str = "project-architecture-policy.json";
const PROJECT_ARCHITECTURE_CARGO_GRAPH_ARTIFACT_NAME: &str = "project-architecture-cargo-graph.json";
const PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME: &str = "project-architecture-receipt.json";
const EFFECT_CLOSURE_RECEIPT_ARTIFACT_NAME: &str = "effect-closure.json";
const EFFECT_CLOSURE_POLICY_ARTIFACT_NAME: &str = "effect-closure-policy.json";
const EFFECT_CLOSURE_GRAPH_ARTIFACT_NAME: &str = "effect-closure-dependency-graph.json";
const EFFECT_CLOSURE_PARENT_STATUS_ARTIFACT_NAME: &str = "effect-closure-parent-status.json";
const EFFECT_CLOSURE_ARTIFACT_NAMES: &[&str] = &[
    EFFECT_CLOSURE_RECEIPT_ARTIFACT_NAME,
    EFFECT_CLOSURE_POLICY_ARTIFACT_NAME,
    EFFECT_CLOSURE_GRAPH_ARTIFACT_NAME,
    EFFECT_CLOSURE_PARENT_STATUS_ARTIFACT_NAME,
];
const PROJECT_ARCHITECTURE_ARTIFACT_NAMES: &[&str] = &[
    PROJECT_ARCHITECTURE_POLICY_SOURCE_ARTIFACT_NAME,
    PROJECT_ARCHITECTURE_POLICY_ARTIFACT_NAME,
    PROJECT_ARCHITECTURE_CARGO_GRAPH_ARTIFACT_NAME,
    PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME,
];
const ARCHITECTURE_ARTIFACT_NAMES: &[&str] = &[
    ARCHITECTURE_IR_ARTIFACT_NAME,
    ARCHITECTURE_COVERAGE_ARTIFACT_NAME,
    ARCHITECTURE_ISSUES_ARTIFACT_NAME,
    ARCHITECTURE_RECEIPT_ARTIFACT_NAME,
];
const RUNNER_PROVENANCE_SOURCE_ID: &str = "octet.runner.artifact-bundle";
const RUNNER_STATIC_FINDINGS_SOURCE_ID: &str = "octet.runner.static-findings";
const FUNCTION_ADDRESS_SOURCE_ID: &str = "octet.function_address.receipt";
const ARCHITECTURE_IR_SOURCE_ID: &str = "octet.compiler-architecture-ir.receipt";
const PROJECT_ARCHITECTURE_SOURCE_ID: &str = "octet.project-architecture-gate.receipt";
const EFFECT_CLOSURE_SOURCE_ID: &str = "octet.runner.effect-closure";
const RUNNER_CHECKER_ID: &str = "cargo-octet";
const STATUS_VALID: &str = "valid";
const STATUS_INVALID: &str = "invalid";
const CHECKED_SCOPE_PREFIX: &str = "profile:";
const FIELD_CHECKED_SCOPE: &str = "checked_scope";
const FIELD_EVIDENCE_TIER: &str = "evidence_tier";
const FIELD_RECORD: &str = "record";
const STATUS_BUNDLE_KIND_PROVE: &str = "prove";
const PROOF_ARTIFACT_PREFIX: &str = "proofs/";
const PROOF_SOURCE_ID: &str = "octet.standards.formal-proof";
const CHECKER_OCTET_STANDARDS: &str = "octet-standards";
const EVIDENCE_LOG_OUTPUT_FORMAT: &str = "evidence-log";
const EVIDENCE_RAIL_OUTPUT_FORMAT: &str = "evidence-rail";
const EVIDENCE_LOG_SOURCE_ID: &str = "octet.evidence_log.manifest";
const EVIDENCE_LOG_MANUAL_ACTION: &str =
    "review the in-tree summary, exact replay command, checked scope, and claim boundary before relying on the raw log";
const VALENCE_EXPORT_SCHEMA_VERSION: &str = "octet-valence-provenance-export/v1";
const VALENCE_EXPORT_RECEIPT_SCHEMA_VERSION: &str = "octet-valence-replayable-receipt/v1";
const VALENCE_EXPORT_NON_CLAIM_CORRECTNESS: &str = "Octet export preserves artifact identity, provenance linkage, and evidence tiers; it does not prove whole-program correctness";
const VALENCE_EXPORT_NON_CLAIM_SEMANTICS: &str =
    "Valence consumes this export for stack traceability; Mantle bundles only the final sidecar opaquely";
const VALENCE_EXPORT_NON_CLAIM_RELEASE: &str = "Octet Valence exports are adapter inputs, not release eligibility, Mantle bundle inclusion, or verifier soundness claims";
const FORMAL_PROOF_TIER: &str = "formal_proof";
const STATIC_LINT_TIER: &str = "static_lint";
const TRANSCRIPT_ASSERTION_TIER: &str = "transcript_assertion";
const HEGEL_BOUNDARY_MARKER: &str = "bounded generated inputs";
#[cfg(test)]
const STATUS_PASSED: &str = "passed";
const STATUS_FAILED: &str = "failed";
const STATUS_MANUAL_REVIEW_REQUIRED: &str = "manual_review_required";
const VALENCE_ADAPTER_TARGET: &str = "valence.stack-provenance.adapter-input";
const TOOL_IDENTITY_NAME: &str = "cargo-octet";
const FAILURE_REASON_SEPARATOR: &str = "; ";
// r[impl octet.evidence_provenance.valence_export.typed_edges]
const EDGE_CHECKS_SOURCE_SHAPE: &str = "checks_source_shape";
const EDGE_RECORDS_LINT_RECEIPT: &str = "records_lint_receipt";
const EDGE_RECORDS_POLICY_RECEIPT: &str = "records_policy_receipt";
const EDGE_RECORDS_FORMAL_PROOF_RECEIPT: &str = "records_formal_proof_receipt";
const EDGE_IMPORTS_TRELLIS_REFERENCE: &str = "imports_trellis_reference";
const EDGE_DERIVES_FROM_PARENT_ARTIFACT: &str = "derives_from_parent_artifact";
const EDGE_FEEDS_VALENCE_ADAPTER: &str = "feeds_valence_adapter";
const SUPPORTED_VALENCE_EDGE_KINDS: &[&str] = &[
    EDGE_CHECKS_SOURCE_SHAPE,
    EDGE_RECORDS_LINT_RECEIPT,
    EDGE_RECORDS_POLICY_RECEIPT,
    EDGE_RECORDS_FORMAL_PROOF_RECEIPT,
    EDGE_IMPORTS_TRELLIS_REFERENCE,
    EDGE_DERIVES_FROM_PARENT_ARTIFACT,
    EDGE_FEEDS_VALENCE_ADAPTER,
];

#[derive(Debug, Clone)]
pub struct BundleFile {
    pub name: String,
    pub contents: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct ArtifactBundleInput {
    pub artifact_dir: String,
    pub files: Vec<BundleFile>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct VerifiedArtifactRow {
    pub artifact: String,
    pub source_id: String,
    pub checker_id: String,
    pub evidence_tier: String,
    pub blake3: String,
    pub checked_scope: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manual_action: Option<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct ArtifactVerificationReport {
    pub schema_version: u32,
    pub artifact_dir: String,
    pub output_format: String,
    pub status: &'static str,
    pub artifacts: Vec<VerifiedArtifactRow>,
    pub diagnostics: Vec<String>,
}

// r[impl octet.evidence_provenance.valence_export.replayable_receipts]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValenceProvenanceExport {
    pub schema_version: String,
    pub artifact_dir: String,
    pub verification_status: String,
    pub artifacts: Vec<ValenceArtifactIdentity>,
    pub provenance_records: Vec<ValenceProvenanceRecord>,
    pub typed_edges: Vec<ValenceProvenanceEdge>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_cards: Vec<ValenceProofCard>,
    pub diagnostics: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValenceArtifactIdentity {
    pub artifact: String,
    pub blake3: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValenceProvenanceEdge {
    pub edge_kind: String,
    pub source_artifact: String,
    pub target_artifact: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValenceReceiptEnvelope {
    pub schema_version: String,
    pub producer_id: String,
    pub checker_id: String,
    pub tool_identity: String,
    pub policy_or_checker_version: String,
    pub normalized_invocation: Vec<String>,
    pub input_artifacts: Vec<ValenceArtifactIdentity>,
    pub output_artifacts: Vec<ValenceArtifactIdentity>,
    pub parent_artifacts: Vec<String>,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub failure_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ast_grep: Option<crate::ast_grep_rail::AstGrepReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm_artifact: Option<crate::wasm_artifact::WasmArtifactReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub charon_semantic: Option<crate::charon_semantic::CharonSemanticReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hegel_property: Option<crate::evidence_rails::HegelPropertyReceiptDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_fuzz: Option<crate::cargo::fuzz::ReceiptDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sarif_import: Option<crate::external::sarif::ReceiptDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub coverage_baseline: Option<crate::coverage::ReceiptDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cargo_mutants: Option<crate::mutation::ReceiptDetails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub benchmark_evidence: Option<crate::benchmark::ReceiptDetails>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValenceProvenanceRecord {
    pub source_id: String,
    pub checker_id: String,
    pub evidence_tier: String,
    pub artifact: String,
    pub blake3: String,
    pub checked_scope: String,
    pub receipt: ValenceReceiptEnvelope,
    pub typed_edges: Vec<ValenceProvenanceEdge>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_artifact_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_action: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_artifacts: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ValenceProofCard {
    pub proof_or_property_id: String,
    pub source_requirement_or_policy_id: String,
    pub policy_id: String,
    pub rail_index: u32,
    pub receipt_path: String,
    pub artifact_blake3_identities: Vec<ValenceArtifactIdentity>,
    pub evidence_tier: String,
    pub evidence_origin: String,
    pub status: String,
    pub non_claims: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub external_reference: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal_proof_envelope: Option<octet_standards::formal_proof::FormalProofEnvelope>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct RawProofCardDeck {
    cards: Vec<ValenceProofCard>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct BundleStatus {
    output_format: Option<OutputFormat>,
    bundle_kind: BundleKind,
    checked_scope: Option<String>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BundleKind {
    Check,
    Prove,
}

#[derive(Clone, Copy)]
struct ExpectedRecordShape {
    source_id: &'static str,
    checker_id: &'static str,
    evidence_tier: EvidenceTier,
}

pub fn run(args: &ArtifactArgs) -> Result<i32, String> {
    if args.output_format == OutputFormat::Sarif {
        return Err("artifact verification does not support SARIF output; use human or json".to_owned());
    }

    let files = read_bundle_files(&args.artifact_dir)?;
    let input = ArtifactBundleInput {
        artifact_dir: args.artifact_dir.display().to_string(),
        files,
    };
    let report = verify_bundle(input.clone());
    if let Some(output_path) = &args.emit_valence_provenance {
        write_valence_provenance_export(output_path, &report, input.clone())?;
    }

    match args.output_format {
        OutputFormat::Human => print_human_report(&report),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report)
                .map_err(|error| format!("serialize artifact verification report: {error}"))?
        ),
        OutputFormat::Sarif => unreachable!("SARIF rejected before rendering"),
    }

    Ok(if report.diagnostics.is_empty() {
        EXIT_VALID
    } else {
        EXIT_INVALID
    })
}

// r[impl octet.evidence_provenance.valence_export.cli]
// r[impl octet.evidence_provenance.valence_export.after_verify]
fn write_valence_provenance_export(
    output_path: &Path,
    report: &ArtifactVerificationReport,
    input: ArtifactBundleInput,
) -> Result<(), String> {
    if !report.diagnostics.is_empty() {
        return Ok(());
    }
    let export = build_valence_provenance_export(input)
        .map_err(|diagnostics| format!("refusing Valence provenance export:\n- {}", diagnostics.join("\n- ")))?;
    let bytes =
        serde_json::to_vec_pretty(&export).map_err(|error| format!("serialize Valence provenance export: {error}"))?;
    write_valence_provenance_export_bytes(output_path, &bytes)
}

// r[impl octet.evidence_provenance.valence_export]
pub fn build_valence_provenance_export(input: ArtifactBundleInput) -> Result<ValenceProvenanceExport, Vec<String>> {
    let report = verify_bundle(input.clone());
    if !report.diagnostics.is_empty() {
        return Err(report.diagnostics);
    }
    let file_map = collect_file_map(input.files);
    let mut diagnostics = Vec::new();
    let evidence_rail_receipt = parse_optional_evidence_rail_receipt(&file_map, &mut diagnostics);
    let provenance_records = parse_provenance_records(&file_map, &mut diagnostics)
        .into_iter()
        .map(|record| valence_provenance_record(record, &file_map, evidence_rail_receipt.as_ref()))
        .collect::<Vec<_>>();
    let proof_cards = parse_valence_proof_cards(&file_map, &mut diagnostics);
    diagnostics.extend(validate_valence_proof_cards(&proof_cards, &file_map));
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    let mut export = ValenceProvenanceExport {
        schema_version: VALENCE_EXPORT_SCHEMA_VERSION.to_owned(),
        artifact_dir: input.artifact_dir,
        verification_status: report.status.to_owned(),
        artifacts: report
            .artifacts
            .into_iter()
            .map(|artifact| ValenceArtifactIdentity {
                artifact: artifact.artifact,
                blake3: artifact.blake3,
            })
            .collect(),
        typed_edges: Vec::new(),
        provenance_records,
        proof_cards,
        diagnostics: Vec::new(),
        non_claims: vec![
            VALENCE_EXPORT_NON_CLAIM_CORRECTNESS.to_owned(),
            VALENCE_EXPORT_NON_CLAIM_SEMANTICS.to_owned(),
            VALENCE_EXPORT_NON_CLAIM_RELEASE.to_owned(),
        ],
    };
    export.typed_edges = valence_export_edges(&export.provenance_records, &export.proof_cards);
    let export_diagnostics = validate_valence_provenance_export(&export);
    if !export_diagnostics.is_empty() {
        return Err(export_diagnostics);
    }
    Ok(export)
}

// r[impl octet.evidence_provenance.valence_export.fail_closed]
// r[impl octet.evidence_provenance.valence_export.replayable_receipts.blocked]
// r[impl octet.evidence_provenance.valence_export.typed_edges.unknown]
pub fn validate_valence_provenance_export(export: &ValenceProvenanceExport) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if export.non_claims.is_empty() {
        diagnostics.push("Valence provenance export must carry at least one explicit non-claim".to_owned());
    }
    for record in &export.provenance_records {
        validate_valence_receipt(record, &mut diagnostics);
        validate_valence_edges(&record.typed_edges, &mut diagnostics);
        validate_edge_tier_boundaries(record, &mut diagnostics);
    }
    validate_valence_edges(&export.typed_edges, &mut diagnostics);
    for proof_card in &export.proof_cards {
        if proof_card.non_claims.is_empty() {
            diagnostics.push(format!(
                "proof card `{}` must carry at least one explicit non-claim",
                proof_card.proof_or_property_id
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn valence_provenance_record(
    record: ProvenanceRecord,
    file_map: &BTreeMap<String, Vec<u8>>,
    evidence_rail_receipt: Option<&crate::evidence_rails::EvidenceRailReceipt>,
) -> ValenceProvenanceRecord {
    let receipt = valence_receipt_envelope(&record, file_map, evidence_rail_receipt);
    let typed_edges = valence_record_edges(&record);
    ValenceProvenanceRecord {
        source_id: record.source_id,
        checker_id: record.checker_id,
        evidence_tier: record.evidence_tier.as_str().to_owned(),
        artifact: record.artifact_locator,
        blake3: record.blake3,
        checked_scope: record.checked_scope,
        receipt,
        typed_edges,
        verifier_identity: record.verifier_identity,
        proof_artifact_identity: record.proof_artifact_identity,
        human_action: record.human_action,
        parent_artifacts: record.parent_artifacts,
    }
}

fn parse_optional_evidence_rail_receipt(
    file_map: &BTreeMap<String, Vec<u8>>,
    diagnostics: &mut Vec<String>,
) -> Option<crate::evidence_rails::EvidenceRailReceipt> {
    let bytes = file_map.get(crate::evidence_rails::RECEIPT_ARTIFACT_NAME)?;
    match serde_json::from_slice(bytes) {
        Ok(receipt) => Some(receipt),
        Err(error) => {
            diagnostics.push(format!("malformed {}: {error}", crate::evidence_rails::RECEIPT_ARTIFACT_NAME));
            None
        }
    }
}

// r[impl octet.evidence_provenance.valence_export.hegel_receipts]
// r[impl octet.evidence_provenance.valence_export.hegel_receipts.export]
fn valence_receipt_envelope(
    record: &ProvenanceRecord,
    file_map: &BTreeMap<String, Vec<u8>>,
    evidence_rail_receipt: Option<&crate::evidence_rails::EvidenceRailReceipt>,
) -> ValenceReceiptEnvelope {
    let input_artifacts = record
        .parent_artifacts
        .iter()
        .filter_map(|artifact| valence_artifact_identity(file_map, artifact))
        .collect::<Vec<_>>();
    let output_artifacts =
        valence_artifact_identity(file_map, &record.artifact_locator).into_iter().collect::<Vec<_>>();
    let matching_evidence_rail_receipt =
        evidence_rail_receipt.filter(|_| record.artifact_locator == crate::evidence_rails::RECEIPT_ARTIFACT_NAME);
    let status = matching_evidence_rail_receipt
        .map(|receipt| receipt.status.clone())
        .unwrap_or_else(|| status_for_provenance_record(record));
    ValenceReceiptEnvelope {
        schema_version: VALENCE_EXPORT_RECEIPT_SCHEMA_VERSION.to_owned(),
        producer_id: record.source_id.clone(),
        checker_id: record.checker_id.clone(),
        tool_identity: tool_identity_for_record(record),
        policy_or_checker_version: policy_or_checker_version(record),
        normalized_invocation: normalized_invocation(record, file_map, matching_evidence_rail_receipt),
        input_artifacts,
        output_artifacts,
        parent_artifacts: record.parent_artifacts.clone(),
        status: status.clone(),
        failure_reason: failure_reason_for_status(&status, matching_evidence_rail_receipt),
        ast_grep: matching_evidence_rail_receipt.and_then(|receipt| receipt.ast_grep.clone()),
        wasm_artifact: matching_evidence_rail_receipt.and_then(|receipt| receipt.wasm_artifact.clone()),
        charon_semantic: matching_evidence_rail_receipt.and_then(|receipt| receipt.charon_semantic.clone()),
        hegel_property: matching_evidence_rail_receipt.and_then(|receipt| receipt.hegel_property.clone()),
        cargo_fuzz: matching_evidence_rail_receipt.and_then(|receipt| receipt.cargo_fuzz.clone()),
        sarif_import: matching_evidence_rail_receipt.and_then(|receipt| receipt.sarif_import.clone()),
        coverage_baseline: matching_evidence_rail_receipt.and_then(|receipt| receipt.coverage_baseline.clone()),
        cargo_mutants: matching_evidence_rail_receipt.and_then(|receipt| receipt.cargo_mutants.clone()),
        benchmark_evidence: matching_evidence_rail_receipt.and_then(|receipt| receipt.benchmark_evidence.clone()),
        non_claims: non_claims_for_record(matching_evidence_rail_receipt),
    }
}

fn valence_artifact_identity(file_map: &BTreeMap<String, Vec<u8>>, artifact: &str) -> Option<ValenceArtifactIdentity> {
    file_map.get(artifact).map(|bytes| ValenceArtifactIdentity {
        artifact: artifact.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(bytes),
    })
}

fn status_for_provenance_record(record: &ProvenanceRecord) -> String {
    if record.evidence_tier == EvidenceTier::ManualReviewRequired {
        STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
    } else {
        STATUS_VALID.to_owned()
    }
}

fn tool_identity_for_record(record: &ProvenanceRecord) -> String {
    if record.checker_id == RUNNER_CHECKER_ID {
        format!("{TOOL_IDENTITY_NAME}@{}", env!("CARGO_PKG_VERSION"))
    } else {
        record.checker_id.clone()
    }
}

fn policy_or_checker_version(record: &ProvenanceRecord) -> String {
    format!("{}@{}", record.checker_id, env!("CARGO_PKG_VERSION"))
}

fn normalized_invocation(
    record: &ProvenanceRecord,
    file_map: &BTreeMap<String, Vec<u8>>,
    evidence_rail_receipt: Option<&crate::evidence_rails::EvidenceRailReceipt>,
) -> Vec<String> {
    if let Some(receipt) = evidence_rail_receipt {
        return receipt.command_argv.clone();
    }
    file_map
        .get(COMMAND_ARTIFACT_NAME)
        .map(|bytes| String::from_utf8_lossy(bytes).split_whitespace().map(str::to_owned).collect::<Vec<_>>())
        .filter(|argv| !argv.is_empty())
        .unwrap_or_else(|| vec![record.checker_id.clone()])
}

fn failure_reason_for_status(
    status: &str,
    evidence_rail_receipt: Option<&crate::evidence_rails::EvidenceRailReceipt>,
) -> Option<String> {
    if !status_requires_failure_reason(status) {
        return None;
    }
    let Some(receipt) = evidence_rail_receipt else {
        return Some(format!("receipt status `{status}`"));
    };
    if !receipt.diagnostics.is_empty() {
        return Some(receipt.diagnostics.join(FAILURE_REASON_SEPARATOR));
    }
    if !receipt.findings.is_empty() {
        return Some(format!("receipt recorded {} finding(s)", receipt.findings.len()));
    }
    Some(format!("receipt status `{status}`"))
}

fn status_requires_failure_reason(status: &str) -> bool {
    status == STATUS_FAILED || status == STATUS_INVALID
}

fn non_claims_for_record(evidence_rail_receipt: Option<&crate::evidence_rails::EvidenceRailReceipt>) -> Vec<String> {
    evidence_rail_receipt.map_or_else(
        || {
            vec![
                VALENCE_EXPORT_NON_CLAIM_CORRECTNESS.to_owned(),
                VALENCE_EXPORT_NON_CLAIM_SEMANTICS.to_owned(),
                VALENCE_EXPORT_NON_CLAIM_RELEASE.to_owned(),
            ]
        },
        |receipt| receipt.non_claims.clone(),
    )
}

fn valence_record_edges(record: &ProvenanceRecord) -> Vec<ValenceProvenanceEdge> {
    let mut edges = Vec::new();
    if record.evidence_tier == EvidenceTier::StaticLint {
        edges.push(edge(EDGE_CHECKS_SOURCE_SHAPE, &record.artifact_locator, &record.checked_scope));
        edges.push(edge(EDGE_RECORDS_LINT_RECEIPT, &record.artifact_locator, &record.source_id));
    } else if record.evidence_tier == EvidenceTier::FormalProof {
        edges.push(edge(EDGE_RECORDS_FORMAL_PROOF_RECEIPT, &record.artifact_locator, &record.source_id));
    } else if record.evidence_tier == EvidenceTier::PolicyCheck {
        edges.push(edge(EDGE_RECORDS_POLICY_RECEIPT, &record.artifact_locator, &record.source_id));
    }
    for parent in &record.parent_artifacts {
        edges.push(edge(EDGE_DERIVES_FROM_PARENT_ARTIFACT, &record.artifact_locator, parent));
    }
    edges.push(edge(EDGE_FEEDS_VALENCE_ADAPTER, &record.artifact_locator, VALENCE_ADAPTER_TARGET));
    edges
}

fn valence_export_edges(
    records: &[ValenceProvenanceRecord],
    proof_cards: &[ValenceProofCard],
) -> Vec<ValenceProvenanceEdge> {
    let mut edges = records.iter().flat_map(|record| record.typed_edges.iter().cloned()).collect::<Vec<_>>();
    edges.extend(proof_cards.iter().filter_map(trellis_reference_edge));
    edges.sort_by(|left, right| {
        left.edge_kind
            .cmp(&right.edge_kind)
            .then(left.source_artifact.cmp(&right.source_artifact))
            .then(left.target_artifact.cmp(&right.target_artifact))
    });
    edges.dedup();
    edges
}

fn trellis_reference_edge(card: &ValenceProofCard) -> Option<ValenceProvenanceEdge> {
    (card.evidence_origin == crate::imported_proof::EVIDENCE_ORIGIN_EXTERNAL_TRELLIS).then(|| {
        edge(
            EDGE_IMPORTS_TRELLIS_REFERENCE,
            &card.receipt_path,
            card.external_reference
                .as_ref()
                .and_then(|value| value.get("source"))
                .and_then(Value::as_str)
                .unwrap_or("trellis.external-reference"),
        )
    })
}

fn edge(edge_kind: &str, source_artifact: &str, target_artifact: &str) -> ValenceProvenanceEdge {
    ValenceProvenanceEdge {
        edge_kind: edge_kind.to_owned(),
        source_artifact: source_artifact.to_owned(),
        target_artifact: target_artifact.to_owned(),
    }
}

fn validate_valence_receipt(record: &ValenceProvenanceRecord, diagnostics: &mut Vec<String>) {
    let receipt = &record.receipt;
    if receipt.schema_version != VALENCE_EXPORT_RECEIPT_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported Valence receipt schema_version `{}` for artifact `{}`",
            receipt.schema_version, record.artifact
        ));
    }
    push_missing_if_blank(diagnostics, &record.artifact, "receipt.producer_id", &receipt.producer_id);
    push_missing_if_blank(diagnostics, &record.artifact, "receipt.checker_id", &receipt.checker_id);
    push_missing_if_blank(diagnostics, &record.artifact, "receipt.tool_identity", &receipt.tool_identity);
    push_missing_if_blank(
        diagnostics,
        &record.artifact,
        "receipt.policy_or_checker_version",
        &receipt.policy_or_checker_version,
    );
    if receipt.normalized_invocation.is_empty() {
        diagnostics.push(format!("artifact `{}` receipt missing normalized_invocation", record.artifact));
    }
    if receipt.output_artifacts.is_empty() {
        diagnostics.push(format!("artifact `{}` receipt missing output_artifacts", record.artifact));
    }
    if !receipt
        .output_artifacts
        .iter()
        .any(|artifact| artifact.artifact == record.artifact && artifact.blake3 == record.blake3)
    {
        diagnostics.push(format!(
            "artifact `{}` receipt output_artifacts do not bind exported BLAKE3 identity",
            record.artifact
        ));
    }
    if status_requires_failure_reason(&receipt.status) && receipt.failure_reason.as_deref().is_none_or(str::is_empty) {
        diagnostics.push(format!("artifact `{}` failed receipt missing failure_reason", record.artifact));
    }
    if receipt.non_claims.is_empty() {
        diagnostics.push(format!("artifact `{}` receipt must carry at least one explicit non-claim", record.artifact));
    }
    if record.checker_id == crate::evidence::CHECKER_AST_GREP {
        validate_ast_grep_valence_receipt(record, diagnostics);
    }
    if record.source_id == "octet.artifact.wasm-tools" {
        validate_wasm_valence_receipt(record, diagnostics);
    }
    if record.checker_id == crate::evidence::CHECKER_HEGEL_RS {
        validate_hegel_valence_receipt(record, diagnostics);
    }
}

fn validate_ast_grep_valence_receipt(record: &ValenceProvenanceRecord, diagnostics: &mut Vec<String>) {
    if record.evidence_tier != STATIC_LINT_TIER {
        diagnostics.push(format!(
            "ast-grep Valence export must remain `{STATIC_LINT_TIER}` evidence, got `{}`",
            record.evidence_tier
        ));
    }
    match record.receipt.ast_grep.as_ref() {
        Some(details) if details.verification_role == crate::ast_grep_rail::RECORDED_ONLY_ROLE => {}
        Some(_) => diagnostics.push("ast-grep Valence export must remain recorded_only".to_owned()),
        None => diagnostics.push("ast-grep Valence export missing ast_grep receipt details".to_owned()),
    }
    if !record.receipt.non_claims.iter().any(|claim| claim == crate::ast_grep_rail::REQUIRED_NON_CLAIM) {
        diagnostics.push("ast-grep Valence export missing syntax-only non-claim".to_owned());
    }
}

fn validate_wasm_valence_receipt(record: &ValenceProvenanceRecord, diagnostics: &mut Vec<String>) {
    if record.evidence_tier != EvidenceTier::PolicyCheck.as_str() {
        diagnostics
            .push(format!("Wasm Valence export must remain `policy_check` evidence, got `{}`", record.evidence_tier));
    }
    match record.receipt.wasm_artifact.as_ref() {
        Some(details) if matches!(details.verification_role.as_str(), "recorded_only" | "boundary") => {}
        Some(_) => diagnostics.push("Wasm Valence export requests an unsupported verification role".to_owned()),
        None => diagnostics.push("Wasm Valence export missing wasm_artifact receipt details".to_owned()),
    }
    if !record.receipt.non_claims.iter().any(|claim| claim == crate::wasm_artifact::REQUIRED_NON_CLAIM) {
        diagnostics.push("Wasm Valence export missing execution non-claim".to_owned());
    }
}

// r[impl octet.evidence_provenance.valence_export.hegel_receipts.no_promotion]
// r[impl octet.evidence_provenance.valence_export.hegel_receipts.blocked]
fn validate_hegel_valence_receipt(record: &ValenceProvenanceRecord, diagnostics: &mut Vec<String>) {
    if record.evidence_tier == FORMAL_PROOF_TIER {
        diagnostics.push("Hegel Valence export must not use formal_proof tier".to_owned());
    }
    if record.evidence_tier != TRANSCRIPT_ASSERTION_TIER {
        diagnostics.push(format!(
            "Hegel Valence export must remain `{TRANSCRIPT_ASSERTION_TIER}` evidence, got `{}`",
            record.evidence_tier
        ));
    }
    if record.receipt.hegel_property.is_none() {
        diagnostics.push("Hegel Valence export missing hegel_property receipt details".to_owned());
    }
    if !record.receipt.non_claims.iter().any(|claim| claim.contains(HEGEL_BOUNDARY_MARKER)) {
        diagnostics.push("Hegel Valence export missing bounded generated-testing non-claim".to_owned());
    }
}

fn push_missing_if_blank(diagnostics: &mut Vec<String>, artifact: &str, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("artifact `{artifact}` receipt missing {field}"));
    }
}

fn validate_valence_edges(edges: &[ValenceProvenanceEdge], diagnostics: &mut Vec<String>) {
    for edge in edges {
        if !SUPPORTED_VALENCE_EDGE_KINDS.contains(&edge.edge_kind.as_str()) {
            diagnostics.push(format!("unsupported Valence provenance edge kind `{}`", edge.edge_kind));
        }
        if edge.source_artifact.trim().is_empty() || edge.target_artifact.trim().is_empty() {
            diagnostics.push(format!("Valence provenance edge `{}` has an empty endpoint", edge.edge_kind));
        }
    }
}

// r[impl octet.evidence_provenance.valence_export.typed_edges.no_promotion]
fn validate_edge_tier_boundaries(record: &ValenceProvenanceRecord, diagnostics: &mut Vec<String>) {
    if record.evidence_tier == FORMAL_PROOF_TIER {
        return;
    }
    if record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_RECORDS_FORMAL_PROOF_RECEIPT) {
        diagnostics.push(format!(
            "artifact `{}` has formal-proof edge with non-formal evidence tier `{}`",
            record.artifact, record.evidence_tier
        ));
    }
}

fn parse_valence_proof_cards(
    file_map: &BTreeMap<String, Vec<u8>>,
    diagnostics: &mut Vec<String>,
) -> Vec<ValenceProofCard> {
    let Some(bytes) = file_map.get(PROOF_CARDS_ARTIFACT_NAME) else {
        return Vec::new();
    };
    match serde_json::from_slice::<RawProofCardDeck>(bytes) {
        Ok(deck) => deck.cards,
        Err(error) => {
            diagnostics.push(format!("malformed {PROOF_CARDS_ARTIFACT_NAME}: {error}"));
            Vec::new()
        }
    }
}

// r[impl octet.evidence_provenance.valence_export.tier_preservation]
// r[impl octet.evidence_provenance.valence_export.tier_preservation_hardened]
fn validate_aeneas_proof_card_envelope(card: &ValenceProofCard, file_map: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
    let Some(envelope) = &card.formal_proof_envelope else {
        return Vec::new();
    };
    let mut diagnostics = octet_standards::formal_proof::validate_formal_proof_envelope(envelope)
        .into_iter()
        .map(|diagnostic| format!("Aeneas proof card `{}`: {diagnostic}", card.proof_or_property_id))
        .collect::<Vec<_>>();
    if card.evidence_tier == FORMAL_PROOF_TIER
        && envelope.evidence_tier != octet_standards::formal_proof::FormalProofEvidenceTier::FormalProof
    {
        diagnostics.push(format!(
            "Aeneas proof card `{}` formal_proof tier does not match its envelope tier",
            card.proof_or_property_id
        ));
    }
    let mut identities = envelope
        .source_bindings
        .iter()
        .chain(envelope.artifact_identities.iter())
        .map(|identity| (&identity.locator, &identity.blake3))
        .collect::<Vec<_>>();
    if let octet_standards::formal_proof::FormalProofProvider::AeneasLean(payload) = &envelope.provider {
        identities.push((&payload.stage_chain.provider_profile.locator, &payload.stage_chain.provider_profile.blake3));
        identities.push((&payload.stage_chain.package_lock.locator, &payload.stage_chain.package_lock.blake3));
        for stage in &payload.stage_chain.stages {
            identities.extend(stage.input_artifacts.iter().map(|identity| (&identity.locator, &identity.blake3)));
            identities.extend(stage.output_artifacts.iter().map(|identity| (&identity.locator, &identity.blake3)));
        }
    }
    for (locator, expected) in identities {
        if let Some(bytes) = file_map.get(locator) {
            let observed = crate::evidence::artifact_blake3_hex(bytes);
            if observed != *expected {
                diagnostics.push(format!(
                    "Aeneas proof card `{}` artifact `{locator}` BLAKE3 mismatch",
                    card.proof_or_property_id
                ));
            }
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_valence_proof_cards(proof_cards: &[ValenceProofCard], file_map: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for card in proof_cards {
        if !card.diagnostics.is_empty() {
            diagnostics.push(format!(
                "proof card `{}` has diagnostics: {}",
                card.proof_or_property_id,
                card.diagnostics.join("; ")
            ));
        }
        if card.non_claims.is_empty() {
            diagnostics
                .push(format!("proof card `{}` must carry at least one explicit non-claim", card.proof_or_property_id));
        }
        diagnostics.extend(validate_aeneas_proof_card_envelope(card, file_map));
        if card.evidence_origin == crate::imported_proof::EVIDENCE_ORIGIN_EXTERNAL_TRELLIS
            && card.evidence_tier == FORMAL_PROOF_TIER
        {
            diagnostics.push(format!(
                "proof card `{}` promotes imported Trellis evidence to formal_proof",
                card.proof_or_property_id
            ));
        }
        if card.receipt_path.trim().is_empty() {
            diagnostics.push(format!("proof card `{}` is missing receipt_path", card.proof_or_property_id));
            continue;
        }
        if card.evidence_origin != crate::imported_proof::EVIDENCE_ORIGIN_EXTERNAL_TRELLIS
            && !file_map.contains_key(&card.receipt_path)
        {
            diagnostics.push(format!(
                "proof card `{}` references missing proof receipt `{}`",
                card.proof_or_property_id, card.receipt_path
            ));
        }
    }
    diagnostics
}

pub fn verify_bundle(input: ArtifactBundleInput) -> ArtifactVerificationReport {
    let file_map = collect_file_map(input.files);
    if !file_map.contains_key(STATUS_ARTIFACT_NAME)
        && file_map.contains_key(crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT)
    {
        return verify_evidence_log_bundle(input.artifact_dir, &file_map);
    }
    if !file_map.contains_key(STATUS_ARTIFACT_NAME)
        && file_map.contains_key(crate::evidence_rails::RECEIPT_ARTIFACT_NAME)
    {
        return verify_evidence_rail_bundle(input.artifact_dir, &file_map);
    }
    let status = inspect_status(&file_map);
    let output_format = status.output_format.unwrap_or_else(|| infer_output_format(&file_map));
    let base_expected_files = expected_artifact_files(output_format, status.bundle_kind);
    let mut diagnostics = status.diagnostics;

    let provenance_records = parse_provenance_records(&file_map, &mut diagnostics);
    let provenance_expected_files =
        expected_files_with_proofs(&base_expected_files, &provenance_records, status.bundle_kind);
    let expected_non_provenance = provenance_expected_files
        .iter()
        .map(String::as_str)
        .filter(|name| *name != PROVENANCE_ARTIFACT_NAME)
        .collect::<Vec<_>>();
    let mut expected_files = provenance_expected_files.clone();
    extend_expected_evidence_log_files(&file_map, &mut expected_files, &mut diagnostics);

    validate_file_set(&file_map, &expected_files, &mut diagnostics);

    let expected_hashes = expected_artifact_hashes(&file_map, &expected_non_provenance);
    validate_provenance_records(
        &provenance_records,
        &expected_hashes,
        &expected_non_provenance,
        status.checked_scope.as_deref(),
        output_format,
        status.bundle_kind,
        &mut diagnostics,
    );
    validate_project_architecture_bundle(&file_map, &mut diagnostics);
    validate_integrated_effect_closure_bundle(&file_map, &mut diagnostics);

    let artifacts = verified_artifact_rows(&provenance_records, &expected_non_provenance);
    diagnostics.sort();
    diagnostics.dedup();

    ArtifactVerificationReport {
        schema_version: REPORT_SCHEMA_VERSION,
        artifact_dir: input.artifact_dir,
        output_format: output_format.as_str().to_owned(),
        status: if diagnostics.is_empty() {
            STATUS_VALID
        } else {
            STATUS_INVALID
        },
        artifacts,
        diagnostics,
    }
}

fn validate_project_architecture_bundle(file_map: &BTreeMap<String, Vec<u8>>, diagnostics: &mut Vec<String>) {
    let present = PROJECT_ARCHITECTURE_ARTIFACT_NAMES.iter().filter(|name| file_map.contains_key(**name)).count();
    if present == 0 {
        return;
    }
    if present != PROJECT_ARCHITECTURE_ARTIFACT_NAMES.len() {
        diagnostics.push("project architecture artifact set is incomplete".to_owned());
        return;
    }
    let receipt = parse_project_architecture_artifact::<
        octet_standards::project_architecture_gate::ProjectArchitectureReceipt,
    >(file_map, PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME, diagnostics);
    let graph = parse_project_architecture_artifact::<octet_standards::project_architecture_gate::ArchitectureCargoGraph>(
        file_map,
        PROJECT_ARCHITECTURE_CARGO_GRAPH_ARTIFACT_NAME,
        diagnostics,
    );
    let policy = parse_project_architecture_artifact::<
        octet_standards::project_architecture_policy::ProjectArchitecturePolicy,
    >(file_map, PROJECT_ARCHITECTURE_POLICY_ARTIFACT_NAME, diagnostics);
    let ir = parse_project_architecture_artifact::<octet_architecture_ir::CompilerArchitectureIr>(
        file_map,
        ARCHITECTURE_IR_ARTIFACT_NAME,
        diagnostics,
    );
    let coverage = parse_project_architecture_artifact::<octet_architecture_ir::CompilerArchitectureCoverage>(
        file_map,
        ARCHITECTURE_COVERAGE_ARTIFACT_NAME,
        diagnostics,
    );
    let compiler_receipt = parse_project_architecture_artifact::<octet_architecture_ir::CompilerArchitectureReceipt>(
        file_map,
        ARCHITECTURE_RECEIPT_ARTIFACT_NAME,
        diagnostics,
    );
    let command = file_map.get(COMMAND_ARTIFACT_NAME);
    let (Some(receipt), Some(graph), Some(policy), Some(ir), Some(coverage), Some(compiler_receipt), Some(command)) =
        (receipt, graph, policy, ir, coverage, compiler_receipt, command)
    else {
        return;
    };
    diagnostics.extend(octet_standards::project_architecture_gate::verify_project_architecture_receipt(&receipt));
    if graph.cargo_graph_id != octet_standards::project_architecture_gate::architecture_cargo_graph_identity(&graph) {
        diagnostics.push("project architecture Cargo graph identity is stale or malformed".to_owned());
    }
    if ir.architecture_ir_id != octet_architecture_ir::ir_identity(&ir) {
        diagnostics.push("compiler Architecture IR identity is stale or malformed".to_owned());
    }
    if coverage.coverage_id != octet_architecture_ir::coverage_identity(&coverage) {
        diagnostics.push("compiler architecture coverage identity is stale or malformed".to_owned());
    }
    if compiler_receipt.receipt_id != octet_architecture_ir::receipt_identity(&compiler_receipt) {
        diagnostics.push("compiler architecture receipt identity is stale or malformed".to_owned());
    }
    if receipt.policy_source_blake3
        != file_map
            .get(PROJECT_ARCHITECTURE_POLICY_SOURCE_ARTIFACT_NAME)
            .map(|bytes| crate::evidence::artifact_blake3_hex(bytes))
            .unwrap_or_default()
    {
        diagnostics.push("project architecture policy source identity does not match stored Nickel bytes".to_owned());
    }
    if receipt.policy_blake3
        != file_map
            .get(PROJECT_ARCHITECTURE_POLICY_ARTIFACT_NAME)
            .map(|bytes| crate::evidence::artifact_blake3_hex(bytes))
            .unwrap_or_default()
    {
        diagnostics.push("project architecture policy identity does not match stored policy bytes".to_owned());
    }
    if receipt.command_blake3 != crate::evidence::artifact_blake3_hex(command) {
        diagnostics.push("project architecture command identity does not match command.txt".to_owned());
    }
    if receipt.project_id != policy.project_id
        || receipt.mode != policy.mode
        || receipt.cargo_graph_id != graph.cargo_graph_id
        || receipt.architecture_ir_id != ir.architecture_ir_id
        || receipt.coverage_id != coverage.coverage_id
        || receipt.compiler_receipt_id != compiler_receipt.receipt_id
    {
        diagnostics.push(
            "project architecture receipt links do not match stored policy, graph, or compiler artifacts".to_owned(),
        );
    }
    let observed_policy_source = file_map
        .get(PROJECT_ARCHITECTURE_POLICY_SOURCE_ARTIFACT_NAME)
        .map(|bytes| crate::evidence::artifact_blake3_hex(bytes))
        .unwrap_or_default();
    let observed_policy = file_map
        .get(PROJECT_ARCHITECTURE_POLICY_ARTIFACT_NAME)
        .map(|bytes| crate::evidence::artifact_blake3_hex(bytes))
        .unwrap_or_default();
    let observed_command = crate::evidence::artifact_blake3_hex(command);
    let regenerated = octet_standards::project_architecture_gate::evaluate_project_architecture(
        &octet_standards::project_architecture_gate::ProjectArchitectureGateInput {
            policy: &policy,
            policy_source_blake3: &observed_policy_source,
            policy_blake3: &observed_policy,
            lint_phase_status: &receipt.lint_phase_status,
            command_blake3: &observed_command,
            cargo_graph: &graph,
            architecture_ir: &ir,
            coverage: &coverage,
            compiler_receipt: &compiler_receipt,
            effect_closure: receipt.effect_closure.as_ref(),
        },
    );
    if regenerated != receipt {
        diagnostics
            .push("project architecture receipt does not match pure reevaluation of stored artifacts".to_owned());
    }
    validate_project_architecture_status(&receipt, file_map, diagnostics);
    let closure_present = EFFECT_CLOSURE_ARTIFACT_NAMES.iter().any(|name| file_map.contains_key(*name));
    if receipt.effect_closure.is_some() && !closure_present {
        diagnostics.push("project architecture receipt links missing integrated effect-closure artifacts".to_owned());
    }
}

fn validate_project_architecture_status(
    receipt: &octet_standards::project_architecture_gate::ProjectArchitectureReceipt,
    file_map: &BTreeMap<String, Vec<u8>>,
    diagnostics: &mut Vec<String>,
) {
    let status = file_map
        .get(STATUS_ARTIFACT_NAME)
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(bytes).ok());
    let status_matches = status.as_ref().is_some_and(|value| {
        value.pointer("/phases/lint/status").and_then(serde_json::Value::as_str)
            == Some(receipt.lint_phase_status.as_str())
            && value.pointer("/phases/architecture/status").and_then(serde_json::Value::as_str)
                == Some(receipt.status.as_str())
            && value.pointer("/phases/architecture/receipt_id").and_then(serde_json::Value::as_str)
                == Some(receipt.receipt_id.as_str())
    });
    if !status_matches {
        diagnostics.push("project architecture receipt phase status does not match status.json".to_owned());
    }
}

fn validate_integrated_effect_closure_bundle(file_map: &BTreeMap<String, Vec<u8>>, diagnostics: &mut Vec<String>) {
    let present = EFFECT_CLOSURE_ARTIFACT_NAMES.iter().filter(|name| file_map.contains_key(**name)).count();
    if present == 0 {
        return;
    }
    if present != EFFECT_CLOSURE_ARTIFACT_NAMES.len() {
        diagnostics.push("integrated effect-closure artifact set is incomplete".to_owned());
        return;
    }
    let receipt = parse_project_architecture_artifact::<crate::effect_closure::EffectClosureReceipt>(
        file_map,
        EFFECT_CLOSURE_RECEIPT_ARTIFACT_NAME,
        diagnostics,
    );
    let policy = parse_project_architecture_artifact::<crate::effect_closure::WorkspacePolicy>(
        file_map,
        EFFECT_CLOSURE_POLICY_ARTIFACT_NAME,
        diagnostics,
    );
    let graph = parse_project_architecture_artifact::<crate::admit::DependencyGraphArtifact>(
        file_map,
        EFFECT_CLOSURE_GRAPH_ARTIFACT_NAME,
        diagnostics,
    );
    let parent = parse_project_architecture_artifact::<
        octet_standards::project_architecture_gate::ProjectArchitectureReceipt,
    >(file_map, PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME, diagnostics);
    let (Some(receipt), Some(policy), Some(graph), Some(parent)) = (receipt, policy, graph, parent) else {
        return;
    };
    diagnostics.extend(crate::effect_closure::verify_receipt_identity(&receipt));
    if receipt.inputs.workspace_policy_blake3
        != file_map
            .get(EFFECT_CLOSURE_POLICY_ARTIFACT_NAME)
            .map(|bytes| crate::evidence::artifact_blake3_hex(bytes))
            .unwrap_or_default()
    {
        diagnostics.push("effect-closure policy identity does not match stored policy bytes".to_owned());
    }
    if receipt.inputs.cargo_graph_blake3
        != file_map
            .get(EFFECT_CLOSURE_GRAPH_ARTIFACT_NAME)
            .map(|bytes| crate::evidence::artifact_blake3_hex(bytes))
            .unwrap_or_default()
    {
        diagnostics.push("effect-closure Cargo graph identity does not match stored graph bytes".to_owned());
    }
    if receipt.inputs.check_status_blake3
        != file_map
            .get(EFFECT_CLOSURE_PARENT_STATUS_ARTIFACT_NAME)
            .map(|bytes| crate::evidence::artifact_blake3_hex(bytes))
            .unwrap_or_default()
    {
        diagnostics.push("effect-closure parent status identity does not match stored status bytes".to_owned());
    }
    if receipt.inputs.command_blake3
        != file_map
            .get(COMMAND_ARTIFACT_NAME)
            .map(|bytes| crate::evidence::artifact_blake3_hex(bytes))
            .unwrap_or_default()
    {
        diagnostics.push("effect-closure command identity does not match command.txt".to_owned());
    }
    if receipt.inputs.function_address_artifact_blake3
        != file_map
            .get(FUNCTION_ADDRESS_ARTIFACT_NAME)
            .map(|bytes| crate::evidence::artifact_blake3_hex(bytes))
            .unwrap_or_default()
    {
        diagnostics.push("effect-closure function-address identity does not match stored evidence".to_owned());
    }
    if receipt.inputs.profile_id != policy.profile_id {
        diagnostics.push("effect-closure receipt profile does not match its stored policy".to_owned());
    }
    if receipt.inputs.target != graph.target
        || receipt.inputs.toolchain != graph.toolchain
        || receipt.inputs.cargo_lock_blake3 != graph.cargo_lock_blake3
    {
        diagnostics.push("effect-closure receipt inputs do not match the stored dependency graph".to_owned());
    }
    let graph_providers = graph.providers.iter().collect::<std::collections::BTreeSet<_>>();
    if receipt.providers.iter().any(|provider| !graph_providers.contains(provider)) {
        diagnostics.push("effect-closure receipt uses a provider absent from the stored dependency graph".to_owned());
    }
    let linked = parent.effect_closure.as_ref().is_some_and(|closure| {
        closure.receipt_id == receipt.receipt_identity
            && closure.admitted == (receipt.status == crate::effect_closure::AdmissionStatus::Admitted)
            && closure.evidence_tier == EvidenceTier::PolicyCheck.as_str()
            && closure.non_claims == receipt.non_claims
    });
    if !linked {
        diagnostics.push(
            "project architecture receipt does not exactly link the integrated effect-closure receipt".to_owned(),
        );
    }
}

fn parse_project_architecture_artifact<T: serde::de::DeserializeOwned>(
    file_map: &BTreeMap<String, Vec<u8>>,
    name: &str,
    diagnostics: &mut Vec<String>,
) -> Option<T> {
    let Some(bytes) = file_map.get(name) else {
        diagnostics.push(format!("missing expected artifact `{name}`"));
        return None;
    };
    match serde_json::from_slice(bytes) {
        Ok(value) => Some(value),
        Err(error) => {
            diagnostics.push(format!("malformed {name}: {error}"));
            None
        }
    }
}

fn read_bundle_files(artifact_dir: &Path) -> Result<Vec<BundleFile>, String> {
    let root = ArtifactRoot::open_existing(artifact_dir)?;
    root.read_files_recursive().map(|files| {
        files
            .into_iter()
            .map(|file| BundleFile {
                name: file.locator,
                contents: file.contents,
            })
            .collect()
    })
}

fn write_valence_provenance_export_bytes(output_path: &Path, bytes: &[u8]) -> Result<(), String> {
    let parent = output_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let file_name = output_path
        .file_name()
        .ok_or_else(|| format!("write {}: output path must name a file", output_path.display()))?;
    let root = ArtifactRoot::open_or_create(parent)?;
    let relative = RootRelativePath::new(Path::new(file_name))?;
    root.write(&relative, bytes)
}

fn print_human_report(report: &ArtifactVerificationReport) {
    println!(
        "octet artifact verification artifact_dir={} output_format={} status={}",
        report.artifact_dir, report.output_format, report.status
    );
    for artifact in &report.artifacts {
        let manual_action = artifact.manual_action.as_deref().unwrap_or("");
        println!(
            "artifact={} source_id={} checker_id={} evidence_tier={} blake3={} checked_scope={} manual_action={}",
            artifact.artifact,
            artifact.source_id,
            artifact.checker_id,
            artifact.evidence_tier,
            artifact.blake3,
            artifact.checked_scope,
            manual_action
        );
    }
    for diagnostic in &report.diagnostics {
        println!("diagnostic: {diagnostic}");
    }
}

fn collect_file_map(files: Vec<BundleFile>) -> BTreeMap<String, Vec<u8>> {
    let mut file_map = BTreeMap::new();
    for file in files {
        file_map.insert(file.name, file.contents);
    }
    file_map
}

fn verify_evidence_log_bundle(
    artifact_dir: String,
    file_map: &BTreeMap<String, Vec<u8>>,
) -> ArtifactVerificationReport {
    let mut diagnostics = Vec::new();
    let manifest = parse_evidence_log_manifest(file_map, &mut diagnostics);
    let mut expected_files = vec![crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT.to_owned()];
    let artifacts = if let Some(manifest) = manifest {
        diagnostics.extend(crate::evidence_log::validate_evidence_log_manifest(&manifest, file_map));
        expected_files.extend(crate::evidence_log::manifest_artifact_locators(&manifest));
        evidence_log_artifact_rows(&manifest)
    } else {
        Vec::new()
    };
    expected_files.sort();
    expected_files.dedup();
    validate_file_set(file_map, &expected_files, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();

    ArtifactVerificationReport {
        schema_version: REPORT_SCHEMA_VERSION,
        artifact_dir,
        output_format: EVIDENCE_LOG_OUTPUT_FORMAT.to_owned(),
        status: if diagnostics.is_empty() {
            STATUS_VALID
        } else {
            STATUS_INVALID
        },
        artifacts,
        diagnostics,
    }
}

fn evidence_log_artifact_rows(manifest: &crate::evidence_log::EvidenceLogManifest) -> Vec<VerifiedArtifactRow> {
    manifest
        .logs
        .iter()
        .map(|entry| VerifiedArtifactRow {
            artifact: entry.artifact_locator.clone(),
            source_id: EVIDENCE_LOG_SOURCE_ID.to_owned(),
            checker_id: RUNNER_CHECKER_ID.to_owned(),
            evidence_tier: EvidenceTier::TranscriptAssertion.as_str().to_owned(),
            blake3: entry.blake3.clone(),
            checked_scope: entry.checked_scope.clone(),
            manual_action: Some(EVIDENCE_LOG_MANUAL_ACTION.to_owned()),
        })
        .collect()
}

fn verify_evidence_rail_bundle(
    artifact_dir: String,
    file_map: &BTreeMap<String, Vec<u8>>,
) -> ArtifactVerificationReport {
    let mut diagnostics = Vec::new();
    let receipt = parse_evidence_rail_receipt(file_map, &mut diagnostics);
    let expected_files = crate::evidence_rails::expected_bundle_files_for_receipt(receipt.as_ref());
    validate_file_set(file_map, &expected_files, &mut diagnostics);
    let expected_non_provenance = expected_files
        .iter()
        .map(String::as_str)
        .filter(|name| *name != crate::evidence_rails::PROVENANCE_ARTIFACT_NAME)
        .collect::<Vec<_>>();
    let expected_hashes = expected_artifact_hashes(file_map, &expected_non_provenance);
    let provenance_records = parse_provenance_records(file_map, &mut diagnostics);
    diagnostics.extend(crate::evidence::validate_provenance_chain(&provenance_records, &expected_hashes));
    validate_evidence_rail_record_shapes(receipt.as_ref(), &provenance_records, &mut diagnostics);
    validate_wasm_artifact_binding(receipt.as_ref(), &provenance_records, &mut diagnostics);
    validate_charon_semantic_artifact_binding(receipt.as_ref(), &provenance_records, &mut diagnostics);
    validate_exact_provenance_coverage(&provenance_records, &expected_non_provenance, &mut diagnostics);
    let artifacts = verified_artifact_rows(&provenance_records, &expected_non_provenance);
    diagnostics.sort();
    diagnostics.dedup();

    ArtifactVerificationReport {
        schema_version: REPORT_SCHEMA_VERSION,
        artifact_dir,
        output_format: EVIDENCE_RAIL_OUTPUT_FORMAT.to_owned(),
        status: if diagnostics.is_empty() {
            STATUS_VALID
        } else {
            STATUS_INVALID
        },
        artifacts,
        diagnostics,
    }
}

fn parse_evidence_rail_receipt(
    file_map: &BTreeMap<String, Vec<u8>>,
    diagnostics: &mut Vec<String>,
) -> Option<crate::evidence_rails::EvidenceRailReceipt> {
    let Some(bytes) = file_map.get(crate::evidence_rails::RECEIPT_ARTIFACT_NAME) else {
        diagnostics.push(format!("missing expected artifact `{}`", crate::evidence_rails::RECEIPT_ARTIFACT_NAME));
        return None;
    };
    match serde_json::from_slice::<crate::evidence_rails::EvidenceRailReceipt>(bytes) {
        Ok(receipt) => {
            diagnostics.extend(crate::evidence_rails::validate_receipt(&receipt));
            Some(receipt)
        }
        Err(error) => {
            diagnostics.push(format!("malformed {}: {error}", crate::evidence_rails::RECEIPT_ARTIFACT_NAME));
            None
        }
    }
}

fn validate_evidence_rail_record_shapes(
    receipt: Option<&crate::evidence_rails::EvidenceRailReceipt>,
    records: &[ProvenanceRecord],
    diagnostics: &mut Vec<String>,
) {
    let Some(receipt) = receipt else {
        return;
    };
    for record in records {
        if record.artifact_locator == crate::evidence_rails::RECEIPT_ARTIFACT_NAME {
            if record.checker_id != receipt.checker_id {
                diagnostics.push(format!(
                    "unexpected checker_id for evidence rail receipt; expected `{}`, got `{}`",
                    receipt.checker_id, record.checker_id
                ));
            }
            let expected_tier = crate::evidence_rails::receipt_tier_from_rail_id(&receipt.rail_id);
            if record.evidence_tier != expected_tier {
                diagnostics.push(format!(
                    "unexpected evidence_tier for evidence rail receipt; expected `{}`, got `{}`",
                    expected_tier.as_str(),
                    record.evidence_tier.as_str()
                ));
            }
        }
        if record.artifact_locator == crate::evidence_rails::SUMMARY_ARTIFACT_NAME {
            if record.checker_id != RUNNER_CHECKER_ID {
                diagnostics.push(format!(
                    "unexpected checker_id for evidence rail summary; expected `{RUNNER_CHECKER_ID}`, got `{}`",
                    record.checker_id
                ));
            }
            if record.evidence_tier != EvidenceTier::ManualReviewRequired {
                diagnostics.push(format!(
                    "unexpected evidence_tier for evidence rail summary; expected `manual_review_required`, got `{}`",
                    record.evidence_tier.as_str()
                ));
            }
        }
    }
}

fn validate_wasm_artifact_binding(
    receipt: Option<&crate::evidence_rails::EvidenceRailReceipt>,
    records: &[ProvenanceRecord],
    diagnostics: &mut Vec<String>,
) {
    let Some(details) = receipt.and_then(|receipt| receipt.wasm_artifact.as_ref()) else {
        return;
    };
    let Some(input_record) =
        records.iter().find(|record| record.artifact_locator == crate::wasm_artifact::INPUT_ARTIFACT_NAME)
    else {
        diagnostics.push("Wasm artifact bundle missing exact-byte provenance record".to_owned());
        return;
    };
    let observed_identity = format!("b3:{}", input_record.blake3);
    if details.exact_artifact_identity != observed_identity {
        diagnostics.push(format!(
            "Wasm exact-byte identity mismatch; receipt `{}`, artifact `{observed_identity}`",
            details.exact_artifact_identity
        ));
    }
    let receipt_record = records
        .iter()
        .find(|record| record.artifact_locator == crate::evidence_rails::RECEIPT_ARTIFACT_NAME);
    if receipt_record.is_none_or(|record| {
        !record.parent_artifacts.iter().any(|parent| parent == crate::wasm_artifact::INPUT_ARTIFACT_NAME)
    }) {
        diagnostics.push("Wasm artifact receipt provenance must link the exact input bytes".to_owned());
    }
}

fn validate_charon_semantic_artifact_binding(
    receipt: Option<&crate::evidence_rails::EvidenceRailReceipt>,
    records: &[ProvenanceRecord],
    diagnostics: &mut Vec<String>,
) {
    let Some(details) = receipt.and_then(|receipt| receipt.charon_semantic.as_ref()) else {
        return;
    };
    let receipt_record = records
        .iter()
        .find(|record| record.artifact_locator == crate::evidence_rails::RECEIPT_ARTIFACT_NAME);
    for artifact in &details.artifacts {
        let Some(record) = records.iter().find(|record| record.artifact_locator == artifact.name) else {
            diagnostics.push(format!("Charon semantic artifact `{}` missing provenance record", artifact.name));
            continue;
        };
        let observed_identity = format!("b3:{}", record.blake3);
        if artifact.identity != observed_identity {
            diagnostics.push(format!(
                "Charon semantic artifact `{}` identity mismatch; receipt `{}`, artifact `{observed_identity}`",
                artifact.name, artifact.identity
            ));
        }
        if record.evidence_tier != EvidenceTier::TranscriptAssertion {
            diagnostics.push(format!(
                "Charon semantic artifact `{}` must remain transcript-assertion evidence",
                artifact.name
            ));
        }
        if record.checker_id != crate::evidence::CHECKER_CARGO_OCTET {
            diagnostics.push(format!("Charon semantic artifact `{}` has unexpected checker", artifact.name));
        }
        if receipt_record
            .is_none_or(|receipt_record| !receipt_record.parent_artifacts.iter().any(|parent| parent == &artifact.name))
        {
            diagnostics.push(format!("Charon semantic receipt provenance must link artifact `{}`", artifact.name));
        }
    }
}

fn inspect_status(file_map: &BTreeMap<String, Vec<u8>>) -> BundleStatus {
    let Some(status_bytes) = file_map.get(STATUS_ARTIFACT_NAME) else {
        return BundleStatus {
            output_format: None,
            bundle_kind: BundleKind::Check,
            checked_scope: None,
            diagnostics: vec![format!("missing expected artifact `{STATUS_ARTIFACT_NAME}`")],
        };
    };

    let status_json: serde_json::Value = match serde_json::from_slice(status_bytes) {
        Ok(value) => value,
        Err(error) => {
            return BundleStatus {
                output_format: None,
                bundle_kind: BundleKind::Check,
                checked_scope: None,
                diagnostics: vec![format!("malformed {STATUS_ARTIFACT_NAME}: {error}")],
            };
        }
    };

    let output_format = match status_json.get("output_format").and_then(serde_json::Value::as_str) {
        Some("human") => Some(OutputFormat::Human),
        Some("json") => Some(OutputFormat::Json),
        Some("sarif") => Some(OutputFormat::Sarif),
        Some(other) => {
            return BundleStatus {
                output_format: None,
                bundle_kind: BundleKind::Check,
                checked_scope: None,
                diagnostics: vec![format!("unsupported {STATUS_ARTIFACT_NAME} output_format `{other}`")],
            };
        }
        None => {
            return BundleStatus {
                output_format: None,
                bundle_kind: BundleKind::Check,
                checked_scope: None,
                diagnostics: vec![format!("missing {STATUS_ARTIFACT_NAME} output_format")],
            };
        }
    };

    let bundle_kind = match status_json.get("bundle_kind").and_then(serde_json::Value::as_str) {
        Some(STATUS_BUNDLE_KIND_PROVE) => BundleKind::Prove,
        Some(_) | None => BundleKind::Check,
    };

    let checked_scope = status_json
        .get("metadata")
        .and_then(|metadata| metadata.get("profile_name"))
        .and_then(serde_json::Value::as_str)
        .filter(|profile| !profile.trim().is_empty())
        .map(|profile| format!("{CHECKED_SCOPE_PREFIX}{profile}"));

    let diagnostics = if checked_scope.is_some() {
        Vec::new()
    } else {
        vec![format!("missing {STATUS_ARTIFACT_NAME} metadata.profile_name")]
    };

    BundleStatus {
        output_format,
        bundle_kind,
        checked_scope,
        diagnostics,
    }
}

fn infer_output_format(file_map: &BTreeMap<String, Vec<u8>>) -> OutputFormat {
    if file_map.contains_key(SARIF_RESULTS_ARTIFACT_NAME) {
        OutputFormat::Sarif
    } else if file_map.contains_key(JSON_RESULTS_ARTIFACT_NAME) {
        OutputFormat::Json
    } else {
        OutputFormat::Human
    }
}

fn expected_artifact_files(output_format: OutputFormat, bundle_kind: BundleKind) -> Vec<String> {
    let mut files = vec![
        COMMAND_ARTIFACT_NAME.to_owned(),
        PROVENANCE_ARTIFACT_NAME.to_owned(),
        STATUS_ARTIFACT_NAME.to_owned(),
        SUMMARY_ARTIFACT_NAME.to_owned(),
    ];
    if bundle_kind == BundleKind::Check {
        match output_format {
            OutputFormat::Human => {}
            OutputFormat::Json => files.push(JSON_RESULTS_ARTIFACT_NAME.to_owned()),
            OutputFormat::Sarif => files.push(SARIF_RESULTS_ARTIFACT_NAME.to_owned()),
        }
    } else {
        files.push(PROOF_CARDS_ARTIFACT_NAME.to_owned());
    }
    files.sort();
    files
}

fn expected_files_with_proofs(
    base_expected_files: &[String],
    records: &[ProvenanceRecord],
    bundle_kind: BundleKind,
) -> Vec<String> {
    let mut files = base_expected_files.to_vec();
    if bundle_kind == BundleKind::Prove {
        files.extend(records.iter().filter_map(expected_proof_artifact));
    }
    if bundle_kind == BundleKind::Check {
        files.extend(records.iter().filter_map(expected_check_extra_artifact));
    }
    files.sort();
    files.dedup();
    files
}

fn expected_check_extra_artifact(record: &ProvenanceRecord) -> Option<String> {
    let is_function_address =
        record.source_id == FUNCTION_ADDRESS_SOURCE_ID && record.artifact_locator == FUNCTION_ADDRESS_ARTIFACT_NAME;
    let is_architecture = record.source_id == ARCHITECTURE_IR_SOURCE_ID
        && ARCHITECTURE_ARTIFACT_NAMES.contains(&record.artifact_locator.as_str());
    let is_project_architecture = record.source_id == PROJECT_ARCHITECTURE_SOURCE_ID
        && PROJECT_ARCHITECTURE_ARTIFACT_NAMES.contains(&record.artifact_locator.as_str());
    let is_effect_closure = record.source_id == EFFECT_CLOSURE_SOURCE_ID
        && EFFECT_CLOSURE_ARTIFACT_NAMES.contains(&record.artifact_locator.as_str());
    (is_function_address || is_architecture || is_project_architecture || is_effect_closure)
        .then(|| record.artifact_locator.clone())
}

fn expected_proof_artifact(record: &ProvenanceRecord) -> Option<String> {
    (record.evidence_tier == EvidenceTier::FormalProof && record.artifact_locator.starts_with(PROOF_ARTIFACT_PREFIX))
        .then(|| record.artifact_locator.clone())
}

fn validate_file_set(file_map: &BTreeMap<String, Vec<u8>>, expected_files: &[String], diagnostics: &mut Vec<String>) {
    for expected in expected_files {
        if !file_map.contains_key(expected) {
            diagnostics.push(format!("missing expected artifact `{expected}`"));
        }
    }

    for actual in file_map.keys() {
        if !expected_files.iter().any(|expected| expected == actual) {
            diagnostics.push(format!("unexpected artifact `{actual}`"));
        }
    }
}

fn extend_expected_evidence_log_files(
    file_map: &BTreeMap<String, Vec<u8>>,
    expected_files: &mut Vec<String>,
    diagnostics: &mut Vec<String>,
) {
    if !file_map.contains_key(crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT) {
        return;
    }
    expected_files.push(crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT.to_owned());
    let Some(manifest) = parse_evidence_log_manifest(file_map, diagnostics) else {
        return;
    };
    diagnostics.extend(crate::evidence_log::validate_evidence_log_manifest(&manifest, file_map));
    expected_files.extend(crate::evidence_log::manifest_artifact_locators(&manifest));
    expected_files.sort();
    expected_files.dedup();
}

fn parse_evidence_log_manifest(
    file_map: &BTreeMap<String, Vec<u8>>,
    diagnostics: &mut Vec<String>,
) -> Option<crate::evidence_log::EvidenceLogManifest> {
    let bytes = file_map.get(crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT)?;
    match serde_json::from_slice(bytes) {
        Ok(manifest) => Some(manifest),
        Err(error) => {
            diagnostics.push(format!("malformed {}: {error}", crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT));
            None
        }
    }
}

fn parse_provenance_records(
    file_map: &BTreeMap<String, Vec<u8>>,
    diagnostics: &mut Vec<String>,
) -> Vec<ProvenanceRecord> {
    let Some(provenance_bytes) = file_map.get(PROVENANCE_ARTIFACT_NAME) else {
        return Vec::new();
    };
    let provenance_text = String::from_utf8_lossy(provenance_bytes);
    let mut records = Vec::new();
    for (offset, line) in provenance_text.lines().enumerate() {
        let line_number = offset + 1;
        match serde_json::from_str::<ProvenanceRecord>(line) {
            Ok(record) => records.push(record),
            Err(error) => diagnostics.push(format!(
                "malformed provenance line {line_number} field `{}`: {error}",
                malformed_provenance_field(line)
            )),
        }
    }
    records
}

fn malformed_provenance_field(line: &str) -> &'static str {
    if line.contains(FIELD_EVIDENCE_TIER) {
        FIELD_EVIDENCE_TIER
    } else {
        FIELD_RECORD
    }
}

fn expected_artifact_hashes(
    file_map: &BTreeMap<String, Vec<u8>>,
    expected_non_provenance: &[&str],
) -> BTreeMap<String, String> {
    let mut hashes = BTreeMap::new();
    for artifact in expected_non_provenance {
        if let Some(bytes) = file_map.get(*artifact) {
            hashes.insert((*artifact).to_owned(), crate::evidence::artifact_blake3_hex(bytes));
        }
    }
    hashes
}

fn validate_provenance_records(
    records: &[ProvenanceRecord],
    expected_hashes: &BTreeMap<String, String>,
    expected_non_provenance: &[&str],
    expected_checked_scope: Option<&str>,
    output_format: OutputFormat,
    bundle_kind: BundleKind,
    diagnostics: &mut Vec<String>,
) {
    diagnostics.extend(crate::evidence::validate_provenance_chain(records, expected_hashes));
    for record in records {
        validate_expected_record_shape(record, output_format, bundle_kind, expected_checked_scope, diagnostics);
    }
    validate_exact_provenance_coverage(records, expected_non_provenance, diagnostics);
}

fn validate_expected_record_shape(
    record: &ProvenanceRecord,
    output_format: OutputFormat,
    bundle_kind: BundleKind,
    expected_checked_scope: Option<&str>,
    diagnostics: &mut Vec<String>,
) {
    let Some(expected) = expected_shape_for_artifact(&record.artifact_locator, output_format, bundle_kind) else {
        diagnostics.push(format!("unmatched provenance record for artifact `{}`", record.artifact_locator));
        return;
    };

    if record.source_id != expected.source_id {
        diagnostics.push(format!(
            "unexpected source_id for artifact `{}`; expected `{}`, got `{}`",
            record.artifact_locator, expected.source_id, record.source_id
        ));
    }
    if record.checker_id != expected.checker_id {
        diagnostics.push(format!(
            "unexpected checker_id for artifact `{}`; expected `{}`, got `{}`",
            record.artifact_locator, expected.checker_id, record.checker_id
        ));
    }
    if record.evidence_tier != expected.evidence_tier {
        diagnostics.push(format!(
            "unexpected evidence_tier for artifact `{}`; expected `{}`, got `{}`",
            record.artifact_locator,
            expected.evidence_tier.as_str(),
            record.evidence_tier.as_str()
        ));
    }

    if bundle_kind == BundleKind::Prove && record.evidence_tier == EvidenceTier::FormalProof {
        if record.checked_scope.trim().is_empty() {
            diagnostics.push(format!("missing required provenance field `{FIELD_CHECKED_SCOPE}`"));
        }
        return;
    }

    match expected_checked_scope {
        Some(scope) if record.checked_scope == scope => {}
        Some(scope) => diagnostics.push(format!(
            "unexpected checked_scope for artifact `{}`; expected `{scope}`, got `{}`",
            record.artifact_locator, record.checked_scope
        )),
        None if !record.checked_scope.trim().is_empty() => {}
        None => diagnostics.push(format!("missing required provenance field `{FIELD_CHECKED_SCOPE}`")),
    }
}

fn expected_shape_for_artifact(
    artifact: &str,
    output_format: OutputFormat,
    bundle_kind: BundleKind,
) -> Option<ExpectedRecordShape> {
    let manual_review_shape = ExpectedRecordShape {
        source_id: RUNNER_PROVENANCE_SOURCE_ID,
        checker_id: RUNNER_CHECKER_ID,
        evidence_tier: EvidenceTier::ManualReviewRequired,
    };
    match artifact {
        COMMAND_ARTIFACT_NAME | STATUS_ARTIFACT_NAME | SUMMARY_ARTIFACT_NAME => Some(manual_review_shape),
        PROOF_CARDS_ARTIFACT_NAME if bundle_kind == BundleKind::Prove => Some(manual_review_shape),
        JSON_RESULTS_ARTIFACT_NAME if output_format == OutputFormat::Json => Some(ExpectedRecordShape {
            source_id: RUNNER_STATIC_FINDINGS_SOURCE_ID,
            checker_id: RUNNER_CHECKER_ID,
            evidence_tier: EvidenceTier::StaticLint,
        }),
        SARIF_RESULTS_ARTIFACT_NAME if output_format == OutputFormat::Sarif => Some(ExpectedRecordShape {
            source_id: RUNNER_STATIC_FINDINGS_SOURCE_ID,
            checker_id: RUNNER_CHECKER_ID,
            evidence_tier: EvidenceTier::StaticLint,
        }),
        FUNCTION_ADDRESS_ARTIFACT_NAME if bundle_kind == BundleKind::Check => Some(ExpectedRecordShape {
            source_id: FUNCTION_ADDRESS_SOURCE_ID,
            checker_id: RUNNER_CHECKER_ID,
            evidence_tier: EvidenceTier::StaticLint,
        }),
        _ if bundle_kind == BundleKind::Check && ARCHITECTURE_ARTIFACT_NAMES.contains(&artifact) => {
            Some(ExpectedRecordShape {
                source_id: ARCHITECTURE_IR_SOURCE_ID,
                checker_id: RUNNER_CHECKER_ID,
                evidence_tier: EvidenceTier::StaticLint,
            })
        }
        _ if bundle_kind == BundleKind::Check && PROJECT_ARCHITECTURE_ARTIFACT_NAMES.contains(&artifact) => {
            Some(ExpectedRecordShape {
                source_id: PROJECT_ARCHITECTURE_SOURCE_ID,
                checker_id: RUNNER_CHECKER_ID,
                evidence_tier: EvidenceTier::PolicyCheck,
            })
        }
        _ if bundle_kind == BundleKind::Check && EFFECT_CLOSURE_ARTIFACT_NAMES.contains(&artifact) => {
            Some(ExpectedRecordShape {
                source_id: EFFECT_CLOSURE_SOURCE_ID,
                checker_id: RUNNER_CHECKER_ID,
                evidence_tier: EvidenceTier::PolicyCheck,
            })
        }
        _ if bundle_kind == BundleKind::Prove && artifact.starts_with(PROOF_ARTIFACT_PREFIX) => {
            Some(ExpectedRecordShape {
                source_id: PROOF_SOURCE_ID,
                checker_id: CHECKER_OCTET_STANDARDS,
                evidence_tier: EvidenceTier::FormalProof,
            })
        }
        _ => None,
    }
}

fn validate_exact_provenance_coverage(
    records: &[ProvenanceRecord],
    expected_non_provenance: &[&str],
    diagnostics: &mut Vec<String>,
) {
    let mut counts = BTreeMap::<String, usize>::new();
    for record in records {
        *counts.entry(record.artifact_locator.clone()).or_default() += 1;
    }

    for artifact in expected_non_provenance {
        match counts.get(*artifact).copied().unwrap_or_default() {
            0 => diagnostics.push(format!("missing provenance record for artifact `{artifact}`")),
            1 => {}
            count => diagnostics.push(format!("duplicate provenance records for artifact `{artifact}`; found {count}")),
        }
    }
}

fn verified_artifact_rows(records: &[ProvenanceRecord], expected_non_provenance: &[&str]) -> Vec<VerifiedArtifactRow> {
    let mut rows = Vec::new();
    for artifact in expected_non_provenance {
        if let Some(record) = records.iter().find(|record| record.artifact_locator == *artifact) {
            rows.push(VerifiedArtifactRow {
                artifact: (*artifact).to_owned(),
                source_id: record.source_id.clone(),
                checker_id: record.checker_id.clone(),
                evidence_tier: record.evidence_tier.as_str().to_owned(),
                blake3: record.blake3.clone(),
                checked_scope: record.checked_scope.clone(),
                manual_action: record.human_action.clone(),
            });
        }
    }
    rows
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTIFACT_DIR: &str = "artifacts/octet";
    const STATUS_HUMAN: &str = "human";
    const STATUS_JSON: &str = "json";
    const STATUS_SARIF: &str = "sarif";
    const CHECKED_SCOPE: &str = "profile:workspace-metadata";
    const COMMAND_CONTENTS: &[u8] = b"cargo octet check\n";
    const STATUS_CONTENTS: &[u8] = br#"{
  "output_format": "human",
  "metadata": { "profile_name": "workspace-metadata" }
}
"#;
    const SUMMARY_CONTENTS: &[u8] = b"--- octet summary ---\nStatus: clean\n---\n";
    const JSON_CONTENTS: &[u8] = b"{\"lint\":\"no_unwrap\"}\n";
    const SARIF_CONTENTS: &[u8] = br#"{"version":"2.1.0"}
"#;
    const FUNCTION_ADDRESS_CONTENTS: &[u8] = br#"{"schema":"octet.function-address.v1"}
"#;
    const ARCHITECTURE_IR_CONTENTS: &[u8] = br#"{"schema_version":"octet-compiler-architecture-ir/v1"}
"#;
    const ARCHITECTURE_COVERAGE_CONTENTS: &[u8] = br#"{"schema_version":"octet-compiler-architecture-coverage/v1"}
"#;
    const ARCHITECTURE_ISSUES_CONTENTS: &[u8] = b"[]\n";
    const ARCHITECTURE_RECEIPT_CONTENTS: &[u8] = br#"{"schema_version":"octet-compiler-architecture-receipt/v1"}
"#;
    const HUMAN_ACTION: &str = "review runner status before release";
    const EVIDENCE_LOG_ARTIFACT_NAME: &str = "proofs/sample-rail-0.stdout";
    const EVIDENCE_LOG_CONTENTS: &[u8] = b"proof-result: pass sample\n";
    const EVIDENCE_LOG_REPLAY_COMMAND: &str = "cargo octet prove --registry policies.json --source src/lib.rs";
    const EVIDENCE_LOG_CAPTURE_SOURCE: &str = "octet.prove.stdout";
    const PROOF_RECEIPT_ARTIFACT_NAME: &str = "proofs/sample.receipt";
    const PROOF_RECEIPT_CONTENTS: &[u8] = b"proof-result: pass sample\n";
    const PROOF_VERIFIER_IDENTITY: &str = "verus@fixture";
    const COMMAND_RECORD_INDEX: usize = 0;
    const STATUS_RECORD_INDEX: usize = 1;
    const SUMMARY_RECORD_INDEX: usize = 2;

    #[test]
    fn complete_human_json_and_sarif_bundles_pass() {
        for output_format in [OutputFormat::Human, OutputFormat::Json, OutputFormat::Sarif] {
            let report = verify_bundle(valid_input(output_format));

            assert_eq!(report.status, STATUS_VALID, "unexpected diagnostics: {:?}", report.diagnostics);
            assert!(report.diagnostics.is_empty());
            assert_eq!(report.output_format, output_format.as_str());
        }
    }

    #[test]
    fn function_address_sidecar_bundle_passes_verification() {
        let mut input = valid_input(OutputFormat::Human);
        input.files.push(BundleFile {
            name: FUNCTION_ADDRESS_ARTIFACT_NAME.to_owned(),
            contents: FUNCTION_ADDRESS_CONTENTS.to_vec(),
        });
        let mut records = valid_records(OutputFormat::Human);
        records.push(function_address_provenance_record());
        replace_file(&mut input, PROVENANCE_ARTIFACT_NAME, render_records(&records).into_bytes());

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_VALID, "unexpected diagnostics: {:?}", report.diagnostics);
        assert!(report.artifacts.iter().any(|artifact| {
            artifact.artifact == FUNCTION_ADDRESS_ARTIFACT_NAME
                && artifact.source_id == FUNCTION_ADDRESS_SOURCE_ID
                && artifact.evidence_tier == EvidenceTier::StaticLint.as_str()
        }));
    }

    #[test]
    fn compiler_architecture_sidecar_bundle_passes_and_rejects_wrong_source() {
        let mut input = valid_input(OutputFormat::Human);
        let architecture_files = [
            (ARCHITECTURE_IR_ARTIFACT_NAME, ARCHITECTURE_IR_CONTENTS),
            (ARCHITECTURE_COVERAGE_ARTIFACT_NAME, ARCHITECTURE_COVERAGE_CONTENTS),
            (ARCHITECTURE_ISSUES_ARTIFACT_NAME, ARCHITECTURE_ISSUES_CONTENTS),
            (ARCHITECTURE_RECEIPT_ARTIFACT_NAME, ARCHITECTURE_RECEIPT_CONTENTS),
        ];
        for (name, contents) in architecture_files {
            input.files.push(BundleFile {
                name: name.to_owned(),
                contents: contents.to_vec(),
            });
        }
        let mut records = valid_records(OutputFormat::Human);
        records.extend(architecture_files.into_iter().map(|(name, contents)| {
            let mut record = record(name, contents, EvidenceTier::StaticLint);
            record.source_id = ARCHITECTURE_IR_SOURCE_ID.to_owned();
            record
        }));
        replace_file(&mut input, PROVENANCE_ARTIFACT_NAME, render_records(&records).into_bytes());

        let report = verify_bundle(input.clone());
        assert_eq!(report.status, STATUS_VALID, "unexpected diagnostics: {:?}", report.diagnostics);

        let mut wrong_records = records;
        let architecture_record = wrong_records
            .iter_mut()
            .find(|record| record.artifact_locator == ARCHITECTURE_RECEIPT_ARTIFACT_NAME)
            .expect("architecture receipt record");
        architecture_record.source_id = RUNNER_STATIC_FINDINGS_SOURCE_ID.to_owned();
        replace_file(&mut input, PROVENANCE_ARTIFACT_NAME, render_records(&wrong_records).into_bytes());
        let wrong_report = verify_bundle(input);
        assert_eq!(wrong_report.status, STATUS_INVALID);
        assert!(wrong_report.diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("unexpected source_id") && diagnostic.contains(ARCHITECTURE_RECEIPT_ARTIFACT_NAME)
        }));
    }

    #[test]
    fn valence_export_preserves_static_lint_and_manual_review_tiers() {
        let export = build_valence_provenance_export(valid_input(OutputFormat::Json)).expect("valid export");

        assert_eq!(export.schema_version, VALENCE_EXPORT_SCHEMA_VERSION);
        assert_eq!(export.verification_status, STATUS_VALID);
        assert!(export.non_claims.iter().any(|claim| claim.contains("does not prove whole-program correctness")));
        assert!(export.artifacts.iter().any(|artifact| artifact.artifact == JSON_RESULTS_ARTIFACT_NAME));
        assert!(export.provenance_records.iter().any(|record| {
            record.artifact == JSON_RESULTS_ARTIFACT_NAME && record.evidence_tier == EvidenceTier::StaticLint.as_str()
        }));
        assert!(export.provenance_records.iter().any(|record| {
            record.artifact == SUMMARY_ARTIFACT_NAME
                && record.evidence_tier == EvidenceTier::ManualReviewRequired.as_str()
                && record.human_action.as_deref() == Some(HUMAN_ACTION)
        }));
    }

    // r[verify octet.evidence_provenance.valence_export.cli.write]
    // r[verify octet.evidence_provenance.valence_export.replayable_receipts.valid]
    // r[verify octet.evidence_provenance.valence_export.typed_edges.known]
    #[test]
    fn valence_export_emits_replayable_receipts_and_typed_edges() {
        let export = build_valence_provenance_export(valid_input(OutputFormat::Json)).expect("valid export");
        let lint_record = export
            .provenance_records
            .iter()
            .find(|record| record.artifact == JSON_RESULTS_ARTIFACT_NAME)
            .expect("lint record exported");

        assert_eq!(lint_record.receipt.schema_version, VALENCE_EXPORT_RECEIPT_SCHEMA_VERSION);
        assert_eq!(lint_record.receipt.producer_id, RUNNER_STATIC_FINDINGS_SOURCE_ID);
        assert_eq!(lint_record.receipt.status, STATUS_VALID);
        assert!(lint_record.receipt.failure_reason.is_none());
        assert!(lint_record.receipt.normalized_invocation.iter().any(|arg| arg == "cargo"));
        assert!(lint_record.receipt.output_artifacts.iter().any(|artifact| {
            artifact.artifact == JSON_RESULTS_ARTIFACT_NAME && artifact.blake3 == lint_record.blake3
        }));
        assert!(lint_record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_CHECKS_SOURCE_SHAPE));
        assert!(lint_record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_RECORDS_LINT_RECEIPT));
        assert!(export.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_FEEDS_VALENCE_ADAPTER));
    }

    // r[verify octet.evidence_provenance.valence_export.tier_preservation.formal_proof]
    // r[verify octet.evidence_provenance.valence_export.negative_fixtures]
    // r[verify octet.evidence_provenance.valence_export.negative_fixtures.positive]
    // r[verify octet.evidence_provenance.valence_export.tier_preservation_hardened.octet_receipt]
    #[test]
    fn valence_export_positive_fixtures_cover_lint_policy_and_formal_proof() {
        let lint_export = build_valence_provenance_export(valid_input(OutputFormat::Json)).expect("valid lint export");
        assert!(lint_export.provenance_records.iter().any(|record| {
            record.artifact == JSON_RESULTS_ARTIFACT_NAME
                && record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_RECORDS_LINT_RECEIPT)
        }));

        let policy_export =
            build_valence_provenance_export(valid_evidence_rail_input(Vec::new())).expect("valid policy export");
        assert!(policy_export.provenance_records.iter().any(|record| {
            record.artifact == crate::evidence_rails::RECEIPT_ARTIFACT_NAME
                && record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_RECORDS_POLICY_RECEIPT)
        }));

        let proof_export = build_valence_provenance_export(valid_prove_input()).expect("valid proof export");
        let proof_record = proof_export
            .provenance_records
            .iter()
            .find(|record| record.artifact == PROOF_RECEIPT_ARTIFACT_NAME)
            .expect("formal proof record exported");
        assert_eq!(proof_record.evidence_tier, EvidenceTier::FormalProof.as_str());
        assert!(proof_record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_RECORDS_FORMAL_PROOF_RECEIPT));
        assert_eq!(proof_record.verifier_identity.as_deref(), Some(PROOF_VERIFIER_IDENTITY));
    }

    // r[verify octet.evidence_provenance.valence_export.replayable_receipts.failed]
    // r[verify octet.evidence_provenance.valence_export.hegel_receipts.export]
    // r[verify octet.evidence_provenance.valence_export.hegel_receipts.no_promotion]
    // r[verify octet.evidence_provenance.valence_export.hegel_receipts.blocked]
    #[test]
    fn valence_export_preserves_hegel_property_receipt_boundary() {
        let export = build_valence_provenance_export(valid_hegel_evidence_rail_input()).expect("valid Hegel export");
        let receipt_record = export
            .provenance_records
            .iter()
            .find(|record| record.artifact == crate::evidence_rails::RECEIPT_ARTIFACT_NAME)
            .expect("Hegel receipt record exported");
        let hegel_details = receipt_record.receipt.hegel_property.as_ref().expect("Hegel details exported");

        assert_eq!(receipt_record.checker_id, crate::evidence::CHECKER_HEGEL_RS);
        assert_eq!(receipt_record.evidence_tier, TRANSCRIPT_ASSERTION_TIER);
        assert_eq!(hegel_details.property_id, "valence.evidence-ir.canonical-identity-order");
        assert!(receipt_record.receipt.non_claims.iter().any(|claim| claim.contains(HEGEL_BOUNDARY_MARKER)));
        assert!(!receipt_record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_RECORDS_FORMAL_PROOF_RECEIPT));

        let blocked = build_valence_provenance_export(hegel_missing_details_input())
            .expect_err("Hegel receipt missing details blocks export");
        assert!(blocked.iter().any(|diagnostic| diagnostic.contains("hegel_property details")));
    }

    #[test]
    fn valence_export_preserves_cargo_fuzz_transcript_boundary() {
        let export =
            build_valence_provenance_export(valid_cargo_fuzz_evidence_rail_input()).expect("valid cargo-fuzz export");
        let receipt_record = export
            .provenance_records
            .iter()
            .find(|record| record.artifact == crate::evidence_rails::RECEIPT_ARTIFACT_NAME)
            .expect("cargo-fuzz receipt record exported");
        let details = receipt_record.receipt.cargo_fuzz.as_ref().expect("cargo-fuzz details exported");

        assert_eq!(receipt_record.checker_id, crate::evidence::CHECKER_OCTET_DYNAMIC_EVIDENCE);
        assert_eq!(receipt_record.evidence_tier, TRANSCRIPT_ASSERTION_TIER);
        assert_eq!(details.profile_id, crate::cargo::fuzz::PROFILE_ID);
        assert!(
            receipt_record
                .receipt
                .non_claims
                .iter()
                .any(|claim| claim == crate::cargo::fuzz::REQUIRED_NON_CLAIM)
        );
        assert!(!receipt_record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_RECORDS_FORMAL_PROOF_RECEIPT));

        let blocked = build_valence_provenance_export(cargo_fuzz_missing_details_input())
            .expect_err("cargo-fuzz receipt missing details blocks export");
        assert!(blocked.iter().any(|diagnostic| diagnostic.contains("cargo_fuzz details")));
    }

    #[test]
    fn valence_export_preserves_ast_grep_candidate_boundary() {
        let export =
            build_valence_provenance_export(valid_ast_grep_evidence_rail_input()).expect("valid ast-grep export");
        let receipt_record = export
            .provenance_records
            .iter()
            .find(|record| record.artifact == crate::evidence_rails::RECEIPT_ARTIFACT_NAME)
            .expect("ast-grep receipt record exported");
        let details = receipt_record.receipt.ast_grep.as_ref().expect("ast-grep details exported");

        assert_eq!(receipt_record.checker_id, crate::evidence::CHECKER_AST_GREP);
        assert_eq!(receipt_record.evidence_tier, STATIC_LINT_TIER);
        assert_eq!(details.verification_role, crate::ast_grep_rail::RECORDED_ONLY_ROLE);
        assert!(receipt_record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_CHECKS_SOURCE_SHAPE));
        assert!(!receipt_record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_RECORDS_FORMAL_PROOF_RECEIPT));

        let blocked = build_valence_provenance_export(ast_grep_promoted_role_input())
            .expect_err("promoted ast-grep role blocks export");
        assert!(blocked.iter().any(|diagnostic| diagnostic.contains("recorded_only")));
    }

    #[test]
    fn wasm_artifact_bundle_verifies_exact_bytes_and_exports_bounded_valence_role() {
        let input = valid_wasm_evidence_rail_input();
        let report = verify_bundle(input.clone());
        assert_eq!(report.status, STATUS_VALID, "unexpected diagnostics: {:?}", report.diagnostics);

        let export = build_valence_provenance_export(input).expect("valid Wasm artifact export");
        let receipt_record = export
            .provenance_records
            .iter()
            .find(|record| record.source_id == "octet.artifact.wasm-tools")
            .expect("Wasm receipt exported");
        let details = receipt_record.receipt.wasm_artifact.as_ref().expect("Wasm details exported");
        assert_eq!(receipt_record.evidence_tier, EvidenceTier::PolicyCheck.as_str());
        assert_eq!(details.verification_role, crate::wasm_artifact::RECORDED_ONLY_ROLE);
        assert_eq!(details.exact_identity_role, "exact-bytes");
        assert!(!receipt_record.typed_edges.iter().any(|edge| edge.edge_kind == EDGE_RECORDS_FORMAL_PROOF_RECEIPT));

        let mut tampered = valid_wasm_evidence_rail_input();
        replace_file(&mut tampered, crate::wasm_artifact::INPUT_ARTIFACT_NAME, b"tampered".to_vec());
        let tampered_report = verify_bundle(tampered);
        assert!(
            tampered_report.diagnostics.iter().any(|diagnostic| diagnostic.contains("hash mismatch")
                || diagnostic.contains("exact-byte identity mismatch"))
        );

        let blocked =
            build_valence_provenance_export(wasm_promoted_role_input()).expect_err("promoted Wasm role blocks export");
        assert!(
            blocked.iter().any(|diagnostic| {
                diagnostic.contains("verification role") || diagnostic.contains("recorded_only or boundary")
            }),
            "{blocked:?}"
        );
    }

    #[test]
    fn valence_export_preserves_failed_receipt_reason_without_passing_claim() {
        let export = build_valence_provenance_export(failed_evidence_rail_input()).expect("failed receipt export");
        let receipt_record = export
            .provenance_records
            .iter()
            .find(|record| record.artifact == crate::evidence_rails::RECEIPT_ARTIFACT_NAME)
            .expect("receipt record exported");

        assert_eq!(receipt_record.receipt.status, STATUS_FAILED);
        assert_ne!(receipt_record.receipt.status, STATUS_PASSED);
        assert!(receipt_record.receipt.failure_reason.as_deref().is_some_and(|reason| reason.contains("deny.toml")));
        assert!(receipt_record.receipt.non_claims.iter().any(|claim| claim.contains("legal advice")));
    }

    // r[verify octet.evidence_provenance.valence_export.cli.diagnostics]
    // r[verify octet.evidence_provenance.valence_export.typed_edges.unknown]
    // r[verify octet.evidence_provenance.valence_export.typed_edges.no_promotion]
    // r[verify octet.evidence_provenance.valence_export.negative_fixtures.negative]
    #[test]
    fn valence_export_validation_rejects_bad_edges_missing_non_claims_and_failed_receipts() {
        let mut unsupported_edge =
            build_valence_provenance_export(valid_input(OutputFormat::Json)).expect("valid export");
        unsupported_edge.provenance_records[COMMAND_RECORD_INDEX].typed_edges.push(edge(
            "unsupported_fixture_edge",
            COMMAND_ARTIFACT_NAME,
            VALENCE_ADAPTER_TARGET,
        ));
        let unsupported_diagnostics = validate_valence_provenance_export(&unsupported_edge);
        assert!(
            unsupported_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unsupported Valence provenance edge kind"))
        );

        let mut missing_non_claim =
            build_valence_provenance_export(valid_input(OutputFormat::Json)).expect("valid export");
        missing_non_claim.non_claims.clear();
        missing_non_claim.provenance_records[COMMAND_RECORD_INDEX].receipt.non_claims.clear();
        let non_claim_diagnostics = validate_valence_provenance_export(&missing_non_claim);
        assert!(non_claim_diagnostics.iter().any(|diagnostic| diagnostic.contains("explicit non-claim")));

        let mut promoted_edge = build_valence_provenance_export(valid_input(OutputFormat::Json)).expect("valid export");
        promoted_edge.provenance_records[COMMAND_RECORD_INDEX].typed_edges.push(edge(
            EDGE_RECORDS_FORMAL_PROOF_RECEIPT,
            COMMAND_ARTIFACT_NAME,
            RUNNER_PROVENANCE_SOURCE_ID,
        ));
        let promoted_diagnostics = validate_valence_provenance_export(&promoted_edge);
        assert!(promoted_diagnostics.iter().any(|diagnostic| diagnostic.contains("formal-proof edge")));

        let mut failed_without_reason =
            build_valence_provenance_export(failed_evidence_rail_input()).expect("failed receipt export");
        let receipt_record = failed_without_reason
            .provenance_records
            .iter_mut()
            .find(|record| record.artifact == crate::evidence_rails::RECEIPT_ARTIFACT_NAME)
            .expect("receipt record exported");
        receipt_record.receipt.failure_reason = None;
        let failed_diagnostics = validate_valence_provenance_export(&failed_without_reason);
        assert!(
            failed_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("failed receipt missing failure_reason"))
        );
    }

    // r[verify octet.evidence_provenance.valence_export.fail_closed.stale_hash]
    // r[verify octet.evidence_provenance.valence_export.fail_closed.missing_parent]
    // r[verify octet.evidence_provenance.valence_export.fail_closed.unknown_checker]
    // r[verify octet.evidence_provenance.valence_export.replayable_receipts.blocked]
    // r[verify octet.evidence_provenance.valence_export.tier_preservation.trellis_import]
    // r[verify octet.evidence_provenance.valence_export.tier_preservation_hardened.trellis_manual]
    // r[verify octet.evidence_provenance.valence_export.tier_preservation_hardened.promotion_failure]
    #[test]
    fn valence_export_rejects_stale_hash_missing_parent_and_tier_promotion() {
        let mut records = valid_records(OutputFormat::Json);
        records[COMMAND_RECORD_INDEX].blake3 = crate::evidence::artifact_blake3_hex(b"different bytes");
        let mut stale_hash = valid_input(OutputFormat::Json);
        replace_file(&mut stale_hash, PROVENANCE_ARTIFACT_NAME, render_records(&records).into_bytes());

        let stale_diagnostics = build_valence_provenance_export(stale_hash).expect_err("stale hash rejected");
        assert!(stale_diagnostics.iter().any(|diagnostic| diagnostic.contains("hash mismatch")));

        let missing_parent =
            build_valence_provenance_export(valid_evidence_rail_input(vec!["artifacts/octet/status.json".to_owned()]))
                .expect_err("missing parent rejected");
        assert!(missing_parent.iter().any(|diagnostic| diagnostic.contains("missing parent artifact")));

        let mut unknown_records = valid_records(OutputFormat::Json);
        unknown_records[STATUS_RECORD_INDEX].checker_id = "mystery-checker".to_owned();
        let mut unknown_checker = valid_input(OutputFormat::Json);
        replace_file(&mut unknown_checker, PROVENANCE_ARTIFACT_NAME, render_records(&unknown_records).into_bytes());
        let unknown_diagnostics =
            build_valence_provenance_export(unknown_checker).expect_err("unknown checker rejected");
        assert!(unknown_diagnostics.iter().any(|diagnostic| diagnostic.contains("unknown checker_id")));

        let trellis_reference = sample_valence_proof_card(
            crate::imported_proof::EVIDENCE_ORIGIN_EXTERNAL_TRELLIS,
            EvidenceTier::ManualReviewRequired.as_str(),
            "trellis-reference.json",
        );
        let trellis_edge = trellis_reference_edge(&trellis_reference).expect("Trellis edge emitted");
        assert_eq!(trellis_edge.edge_kind, EDGE_IMPORTS_TRELLIS_REFERENCE);

        let promoted = sample_valence_proof_card(
            crate::imported_proof::EVIDENCE_ORIGIN_EXTERNAL_TRELLIS,
            EvidenceTier::FormalProof.as_str(),
            "not-rerun-by-octet",
        );
        let promotion_diagnostics = validate_valence_proof_cards(&[promoted], &BTreeMap::new());
        assert!(promotion_diagnostics.iter().any(|diagnostic| diagnostic.contains("promotes imported Trellis")));

        let missing_receipt =
            sample_valence_proof_card("octet_rerun", EvidenceTier::ManualReviewRequired.as_str(), "missing.receipt");
        let receipt_diagnostics = validate_valence_proof_cards(&[missing_receipt], &BTreeMap::new());
        assert!(receipt_diagnostics.iter().any(|diagnostic| diagnostic.contains("missing proof receipt")));
    }

    #[test]
    fn missing_expected_artifact_fails_closed() {
        let mut input = valid_input(OutputFormat::Json);
        input.files.retain(|file| file.name != JSON_RESULTS_ARTIFACT_NAME);

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing expected artifact `results.jsonl`"))
        );
    }

    #[test]
    fn stale_extra_artifact_fails_closed() {
        let mut input = valid_input(OutputFormat::Human);
        input.files.push(BundleFile {
            name: "stale.txt".to_owned(),
            contents: b"stale\n".to_vec(),
        });

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic == "unexpected artifact `stale.txt`"));
    }

    #[test]
    fn malformed_jsonl_fails_closed() {
        let mut input = valid_input(OutputFormat::Human);
        replace_file(&mut input, PROVENANCE_ARTIFACT_NAME, b"not-json\n".to_vec());

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed provenance line")));
    }

    #[test]
    fn evidence_log_manifest_extends_artifact_verification() {
        let mut input = valid_input(OutputFormat::Human);
        append_valid_evidence_log_manifest(&mut input);

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_VALID, "unexpected diagnostics: {:?}", report.diagnostics);
        assert!(report.diagnostics.is_empty());
    }

    #[test]
    fn evidence_log_manifest_tampering_fails_closed() {
        let mut input = valid_input(OutputFormat::Human);
        append_valid_evidence_log_manifest(&mut input);
        replace_file(&mut input, EVIDENCE_LOG_ARTIFACT_NAME, b"tampered proof log\n".to_vec());

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("field `blake3` mismatch")));
    }

    #[test]
    fn evidence_log_only_manifest_directory_passes() {
        let input = valid_evidence_log_only_input();

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_VALID, "unexpected diagnostics: {:?}", report.diagnostics);
        assert_eq!(report.output_format, EVIDENCE_LOG_OUTPUT_FORMAT);
        assert_eq!(report.artifacts.len(), 1);
        assert_eq!(report.artifacts[0].artifact, EVIDENCE_LOG_ARTIFACT_NAME);
        assert_eq!(report.artifacts[0].evidence_tier, EvidenceTier::TranscriptAssertion.as_str());
    }

    #[test]
    fn evidence_log_only_manifest_directory_rejects_stale_sidecar() {
        let mut input = valid_evidence_log_only_input();
        input.files.push(BundleFile {
            name: "stale-sidecar.txt".to_owned(),
            contents: b"stale\n".to_vec(),
        });

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("unexpected artifact")));
    }

    #[test]
    fn evidence_rail_bundle_passes_and_rejects_tampering_missing_parent_and_stale_sidecar() {
        let valid = verify_bundle(valid_evidence_rail_input(Vec::new()));
        assert_eq!(valid.status, STATUS_VALID, "unexpected diagnostics: {:?}", valid.diagnostics);
        assert_eq!(valid.output_format, EVIDENCE_RAIL_OUTPUT_FORMAT);

        let mut tampered = valid_evidence_rail_input(Vec::new());
        replace_file(&mut tampered, crate::evidence_rails::SUMMARY_ARTIFACT_NAME, b"tampered\n".to_vec());
        let tampered_report = verify_bundle(tampered);
        assert_eq!(tampered_report.status, STATUS_INVALID);
        assert!(tampered_report.diagnostics.iter().any(|diagnostic| diagnostic.contains("hash mismatch")));

        let missing_parent = verify_bundle(valid_evidence_rail_input(vec!["artifacts/octet/status.json".to_owned()]));
        assert_eq!(missing_parent.status, STATUS_INVALID);
        assert!(missing_parent.diagnostics.iter().any(|diagnostic| diagnostic.contains("missing parent artifact")));

        let mut stale = valid_evidence_rail_input(Vec::new());
        stale.files.push(BundleFile {
            name: "stale.txt".to_owned(),
            contents: b"stale\n".to_vec(),
        });
        let stale_report = verify_bundle(stale);
        assert_eq!(stale_report.status, STATUS_INVALID);
        assert!(stale_report.diagnostics.iter().any(|diagnostic| diagnostic.contains("unexpected artifact")));
    }

    #[test]
    fn hash_mismatch_unknown_checker_invalid_tier_and_missing_manual_action_fail_closed() {
        let mut records = valid_records(OutputFormat::Json);
        records[COMMAND_RECORD_INDEX].blake3 = crate::evidence::artifact_blake3_hex(b"different bytes");
        records[STATUS_RECORD_INDEX].checker_id = "mystery-checker".to_owned();
        records[STATUS_RECORD_INDEX].human_action = None;
        records[SUMMARY_RECORD_INDEX].evidence_tier = EvidenceTier::FormalProof;
        let mut input = valid_input(OutputFormat::Json);
        replace_file(&mut input, PROVENANCE_ARTIFACT_NAME, render_records(&records).into_bytes());

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("hash mismatch for artifact locator `command.txt`"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unknown checker_id `mystery-checker`"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing required provenance field `human_action`"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unexpected evidence_tier for artifact `summary.txt`"))
        );
    }

    #[test]
    fn exact_coverage_rejects_duplicate_and_unmatched_records() {
        let mut records = valid_records(OutputFormat::Human);
        records.push(records[COMMAND_RECORD_INDEX].clone());
        let mut extra = records[COMMAND_RECORD_INDEX].clone();
        extra.artifact_locator = "ghost.txt".to_owned();
        records.push(extra);
        let mut input = valid_input(OutputFormat::Human);
        replace_file(&mut input, PROVENANCE_ARTIFACT_NAME, render_records(&records).into_bytes());

        let report = verify_bundle(input);

        assert_eq!(report.status, STATUS_INVALID);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("duplicate provenance records for artifact `command.txt`"))
        );
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unmatched provenance record for artifact `ghost.txt`"))
        );
    }

    fn valid_input(output_format: OutputFormat) -> ArtifactBundleInput {
        let mut files = vec![
            BundleFile {
                name: COMMAND_ARTIFACT_NAME.to_owned(),
                contents: COMMAND_CONTENTS.to_vec(),
            },
            BundleFile {
                name: STATUS_ARTIFACT_NAME.to_owned(),
                contents: status_contents(output_format).into_bytes(),
            },
            BundleFile {
                name: SUMMARY_ARTIFACT_NAME.to_owned(),
                contents: SUMMARY_CONTENTS.to_vec(),
            },
        ];
        match output_format {
            OutputFormat::Human => {}
            OutputFormat::Json => files.push(BundleFile {
                name: JSON_RESULTS_ARTIFACT_NAME.to_owned(),
                contents: JSON_CONTENTS.to_vec(),
            }),
            OutputFormat::Sarif => files.push(BundleFile {
                name: SARIF_RESULTS_ARTIFACT_NAME.to_owned(),
                contents: SARIF_CONTENTS.to_vec(),
            }),
        }
        files.push(BundleFile {
            name: PROVENANCE_ARTIFACT_NAME.to_owned(),
            contents: render_records(&valid_records(output_format)).into_bytes(),
        });
        ArtifactBundleInput {
            artifact_dir: ARTIFACT_DIR.to_owned(),
            files,
        }
    }

    fn valid_records(output_format: OutputFormat) -> Vec<ProvenanceRecord> {
        let mut records = vec![
            record(COMMAND_ARTIFACT_NAME, COMMAND_CONTENTS, EvidenceTier::ManualReviewRequired),
            record(STATUS_ARTIFACT_NAME, status_contents(output_format).as_bytes(), EvidenceTier::ManualReviewRequired),
            record(SUMMARY_ARTIFACT_NAME, SUMMARY_CONTENTS, EvidenceTier::ManualReviewRequired),
        ];
        match output_format {
            OutputFormat::Human => {}
            OutputFormat::Json => {
                records.push(record(JSON_RESULTS_ARTIFACT_NAME, JSON_CONTENTS, EvidenceTier::StaticLint));
            }
            OutputFormat::Sarif => {
                records.push(record(SARIF_RESULTS_ARTIFACT_NAME, SARIF_CONTENTS, EvidenceTier::StaticLint));
            }
        }
        records
    }

    fn valid_prove_input() -> ArtifactBundleInput {
        let proof_card =
            sample_valence_proof_card("octet-rerun", EvidenceTier::FormalProof.as_str(), PROOF_RECEIPT_ARTIFACT_NAME);
        let proof_cards = serde_json::json!({
            "schema_version": "octet-proof-cards/v1",
            "cards": [proof_card],
        });
        let proof_cards_bytes = serde_json::to_vec_pretty(&proof_cards).expect("serialize proof cards");
        let mut records = vec![
            record(COMMAND_ARTIFACT_NAME, COMMAND_CONTENTS, EvidenceTier::ManualReviewRequired),
            record(STATUS_ARTIFACT_NAME, prove_status_contents().as_bytes(), EvidenceTier::ManualReviewRequired),
            record(SUMMARY_ARTIFACT_NAME, SUMMARY_CONTENTS, EvidenceTier::ManualReviewRequired),
            record(PROOF_CARDS_ARTIFACT_NAME, &proof_cards_bytes, EvidenceTier::ManualReviewRequired),
            formal_proof_record(),
        ];
        records.sort_by(|left, right| left.artifact_locator.cmp(&right.artifact_locator));
        ArtifactBundleInput {
            artifact_dir: ARTIFACT_DIR.to_owned(),
            files: vec![
                BundleFile {
                    name: COMMAND_ARTIFACT_NAME.to_owned(),
                    contents: COMMAND_CONTENTS.to_vec(),
                },
                BundleFile {
                    name: STATUS_ARTIFACT_NAME.to_owned(),
                    contents: prove_status_contents().into_bytes(),
                },
                BundleFile {
                    name: SUMMARY_ARTIFACT_NAME.to_owned(),
                    contents: SUMMARY_CONTENTS.to_vec(),
                },
                BundleFile {
                    name: PROOF_CARDS_ARTIFACT_NAME.to_owned(),
                    contents: proof_cards_bytes,
                },
                BundleFile {
                    name: PROOF_RECEIPT_ARTIFACT_NAME.to_owned(),
                    contents: PROOF_RECEIPT_CONTENTS.to_vec(),
                },
                BundleFile {
                    name: PROVENANCE_ARTIFACT_NAME.to_owned(),
                    contents: render_records(&records).into_bytes(),
                },
            ],
        }
    }

    fn prove_status_contents() -> String {
        String::from_utf8(STATUS_CONTENTS.to_vec())
            .expect("status fixture utf8")
            .replace("\n  \"metadata\"", "\n  \"bundle_kind\": \"prove\",\n  \"metadata\"")
    }

    fn formal_proof_record() -> ProvenanceRecord {
        ProvenanceRecord {
            schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: PROOF_SOURCE_ID.to_owned(),
            checker_id: CHECKER_OCTET_STANDARDS.to_owned(),
            evidence_tier: EvidenceTier::FormalProof,
            artifact_locator: PROOF_RECEIPT_ARTIFACT_NAME.to_owned(),
            blake3: crate::evidence::artifact_blake3_hex(PROOF_RECEIPT_CONTENTS),
            checked_scope: CHECKED_SCOPE.to_owned(),
            verifier_identity: Some(PROOF_VERIFIER_IDENTITY.to_owned()),
            proof_artifact_identity: Some(crate::evidence::artifact_blake3_hex(PROOF_RECEIPT_CONTENTS)),
            human_action: None,
            parent_artifacts: vec![PROOF_CARDS_ARTIFACT_NAME.to_owned()],
        }
    }

    fn function_address_provenance_record() -> ProvenanceRecord {
        let mut record = record(FUNCTION_ADDRESS_ARTIFACT_NAME, FUNCTION_ADDRESS_CONTENTS, EvidenceTier::StaticLint);
        record.source_id = FUNCTION_ADDRESS_SOURCE_ID.to_owned();
        record
    }

    fn record(artifact: &str, contents: &[u8], evidence_tier: EvidenceTier) -> ProvenanceRecord {
        ProvenanceRecord {
            schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: if evidence_tier == EvidenceTier::StaticLint {
                RUNNER_STATIC_FINDINGS_SOURCE_ID.to_owned()
            } else {
                RUNNER_PROVENANCE_SOURCE_ID.to_owned()
            },
            checker_id: RUNNER_CHECKER_ID.to_owned(),
            evidence_tier,
            artifact_locator: artifact.to_owned(),
            blake3: crate::evidence::artifact_blake3_hex(contents),
            checked_scope: CHECKED_SCOPE.to_owned(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: (evidence_tier == EvidenceTier::ManualReviewRequired).then(|| HUMAN_ACTION.to_owned()),
            parent_artifacts: Vec::new(),
        }
    }

    fn status_contents(output_format: OutputFormat) -> String {
        let name = match output_format {
            OutputFormat::Human => STATUS_HUMAN,
            OutputFormat::Json => STATUS_JSON,
            OutputFormat::Sarif => STATUS_SARIF,
        };
        String::from_utf8(STATUS_CONTENTS.to_vec())
            .expect("status fixture utf8")
            .replace(STATUS_HUMAN, name)
    }

    fn render_records(records: &[ProvenanceRecord]) -> String {
        let mut rendered = String::new();
        for record in records {
            rendered.push_str(&serde_json::to_string(record).expect("serialize provenance fixture"));
            rendered.push('\n');
        }
        rendered
    }

    fn append_valid_evidence_log_manifest(input: &mut ArtifactBundleInput) {
        let manifest = valid_evidence_log_manifest();
        input.files.push(BundleFile {
            name: crate::evidence_log::EVIDENCE_LOG_MANIFEST_ARTIFACT.to_owned(),
            contents: serde_json::to_vec_pretty(&manifest).expect("serialize evidence log manifest"),
        });
        input.files.push(BundleFile {
            name: EVIDENCE_LOG_ARTIFACT_NAME.to_owned(),
            contents: EVIDENCE_LOG_CONTENTS.to_vec(),
        });
    }

    fn valid_evidence_log_only_input() -> ArtifactBundleInput {
        let mut input = ArtifactBundleInput {
            artifact_dir: "archive-evidence/dogfood-run".to_owned(),
            files: Vec::new(),
        };
        append_valid_evidence_log_manifest(&mut input);
        input
    }

    fn valid_evidence_log_manifest() -> crate::evidence_log::EvidenceLogManifest {
        crate::evidence_log::manifest_from_entries(vec![crate::evidence_log::EvidenceLogEntry::from_bytes(
            EVIDENCE_LOG_ARTIFACT_NAME,
            EVIDENCE_LOG_CONTENTS,
            EVIDENCE_LOG_REPLAY_COMMAND,
            CHECKED_SCOPE,
            EVIDENCE_LOG_CAPTURE_SOURCE,
            crate::evidence_log::RETENTION_CONTENT_ADDRESSED_RAW,
        )])
    }

    fn valid_evidence_rail_input(parent_artifacts: Vec<String>) -> ArtifactBundleInput {
        evidence_rail_input(r#"{"diagnostics":[]}"#, Some("deny.toml"), parent_artifacts)
    }

    fn valid_hegel_evidence_rail_input() -> ArtifactBundleInput {
        hegel_evidence_rail_input(|_receipt| {})
    }

    fn valid_cargo_fuzz_evidence_rail_input() -> ArtifactBundleInput {
        cargo_fuzz_evidence_rail_input(|_receipt| {})
    }

    fn cargo_fuzz_missing_details_input() -> ArtifactBundleInput {
        cargo_fuzz_evidence_rail_input(|receipt| {
            receipt.cargo_fuzz = None;
        })
    }

    fn hegel_missing_details_input() -> ArtifactBundleInput {
        hegel_evidence_rail_input(|receipt| {
            receipt.hegel_property = None;
        })
    }

    fn valid_ast_grep_evidence_rail_input() -> ArtifactBundleInput {
        ast_grep_evidence_rail_input(|_receipt| {})
    }

    fn ast_grep_promoted_role_input() -> ArtifactBundleInput {
        ast_grep_evidence_rail_input(|receipt| {
            receipt.ast_grep.as_mut().expect("ast-grep details").verification_role = "formal_proof".to_owned();
        })
    }

    fn valid_wasm_evidence_rail_input() -> ArtifactBundleInput {
        wasm_evidence_rail_input(|_receipt| {})
    }

    fn wasm_promoted_role_input() -> ArtifactBundleInput {
        wasm_evidence_rail_input(|receipt| {
            receipt.wasm_artifact.as_mut().expect("Wasm details").verification_role = "formal_proof".to_owned();
        })
    }

    fn failed_evidence_rail_input() -> ArtifactBundleInput {
        evidence_rail_input(r#"{"diagnostics":[]}"#, None, Vec::new())
    }

    fn evidence_rail_input(
        import_text: &str,
        config_path: Option<&str>,
        parent_artifacts: Vec<String>,
    ) -> ArtifactBundleInput {
        let receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
            rail: crate::evidence_rails::EvidenceRailKind::DependencyAudit,
            mode: "import",
            import_text,
            workspace_scope: CHECKED_SCOPE,
            config_path,
            baseline_identity: None,
            lockfile_identity: None,
            locked: true,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts,
            tool_exit_code: None,
        });
        let receipt_json = crate::evidence_rails::render_receipt_json(&receipt).expect("render evidence rail receipt");
        let summary = crate::evidence_rails::render_summary(&receipt);
        let provenance = crate::evidence_rails::render_provenance_jsonl(&receipt, &receipt_json, &summary)
            .expect("render evidence rail provenance");
        evidence_rail_bundle("artifacts/dependency-audit", receipt_json, summary, provenance)
    }

    fn ast_grep_evidence_rail_input(
        mutate: impl FnOnce(&mut crate::evidence_rails::EvidenceRailReceipt),
    ) -> ArtifactBundleInput {
        let profile =
            crate::ast_grep_rail::parse_profile(include_str!("../../standards/ast-grep/generated/profile.json"))
                .expect("ast-grep profile");
        let envelope = crate::ast_grep_rail::render_scan_envelope(
            profile,
            "0.41.1".to_owned(),
            serde_json::json!([{
                "ruleId": "unsafe-block-review",
                "severity": "warning",
                "message": "review unsafe block",
                "file": "src/lib.rs",
                "range": {"start": {"line": 1, "column": 2}, "end": {"line": 1, "column": 8}},
                "metaVariables": {"multi": {"BODY": [{"text": "call()"}]}}
            }]),
            String::new(),
            0,
        )
        .expect("ast-grep envelope");
        let mut receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
            rail: crate::evidence_rails::EvidenceRailKind::AstGrepStructural,
            mode: "import",
            import_text: &envelope,
            workspace_scope: CHECKED_SCOPE,
            config_path: Some("standards/ast-grep/generated/profile.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: None,
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        mutate(&mut receipt);
        let receipt_json = crate::evidence_rails::render_receipt_json(&receipt).expect("render ast-grep rail receipt");
        let summary = crate::evidence_rails::render_summary(&receipt);
        let provenance = crate::evidence_rails::render_provenance_jsonl(&receipt, &receipt_json, &summary)
            .expect("render ast-grep rail provenance");
        evidence_rail_bundle("artifacts/ast-grep", receipt_json, summary, provenance)
    }

    fn wasm_evidence_rail_input(
        mutate: impl FnOnce(&mut crate::evidence_rails::EvidenceRailReceipt),
    ) -> ArtifactBundleInput {
        let registry_bytes = include_bytes!("../../standards/wasm-artifact/generated/profiles.json");
        let registry =
            crate::wasm_artifact::parse_registry(std::str::from_utf8(registry_bytes).expect("profile registry UTF-8"))
                .expect("profile registry");
        let artifact = wat::parse_str(include_str!("../../standards/wasm-artifact/fixtures/positive/module.wat"))
            .expect("Wasm fixture");
        let envelope = crate::wasm_artifact::analyze_bytes(
            &artifact,
            &registry,
            "portable-component-baseline",
            crate::wasm_artifact::registry_identity(registry_bytes),
            &[],
        )
        .expect("Wasm analysis");
        let envelope_text = crate::wasm_artifact::render_envelope(&envelope).expect("render Wasm envelope");
        let mut receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
            rail: crate::evidence_rails::EvidenceRailKind::WasmArtifact,
            mode: "import",
            import_text: &envelope_text,
            workspace_scope: CHECKED_SCOPE,
            config_path: Some("standards/wasm-artifact/generated/profiles.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("portable-component-baseline"),
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        mutate(&mut receipt);
        let receipt_json = crate::evidence_rails::render_receipt_json(&receipt).expect("render Wasm rail receipt");
        let summary = crate::evidence_rails::render_summary(&receipt);
        let provenance = crate::evidence_rails::render_provenance_jsonl(&receipt, &receipt_json, &summary)
            .expect("render Wasm rail provenance");
        let mut bundle = evidence_rail_bundle("artifacts/wasm-artifact", receipt_json, summary, provenance);
        bundle.files.push(BundleFile {
            name: crate::wasm_artifact::INPUT_ARTIFACT_NAME.to_owned(),
            contents: artifact,
        });
        bundle
    }

    fn cargo_fuzz_evidence_rail_input(
        mutate: impl FnOnce(&mut crate::evidence_rails::EvidenceRailReceipt),
    ) -> ArtifactBundleInput {
        let mut receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
            rail: crate::evidence_rails::EvidenceRailKind::DynamicEvidence,
            mode: "import",
            import_text: include_str!("../tests/fixtures/evidence-rails/cargo-fuzz-clean.json"),
            workspace_scope: CHECKED_SCOPE,
            config_path: Some("dynamic-evidence-profiles.json"),
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some(crate::cargo::fuzz::PROFILE_ID),
            requested_command: None,
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        mutate(&mut receipt);
        let receipt_json = crate::evidence_rails::render_receipt_json(&receipt).expect("render cargo-fuzz receipt");
        let summary = crate::evidence_rails::render_summary(&receipt);
        let provenance = crate::evidence_rails::render_provenance_jsonl(&receipt, &receipt_json, &summary)
            .expect("render cargo-fuzz provenance");
        evidence_rail_bundle("artifacts/cargo-fuzz", receipt_json, summary, provenance)
    }

    fn hegel_evidence_rail_input(
        mutate: impl FnOnce(&mut crate::evidence_rails::EvidenceRailReceipt),
    ) -> ArtifactBundleInput {
        let mut receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
            rail: crate::evidence_rails::EvidenceRailKind::HegelProperty,
            mode: "import",
            import_text: include_str!("../tests/fixtures/evidence-rails/hegel-property-passed.json"),
            workspace_scope: CHECKED_SCOPE,
            config_path: None,
            baseline_identity: None,
            lockfile_identity: None,
            locked: false,
            advisory_updated_by_prior_step: false,
            profile: Some("fast"),
            requested_command: Some("cargo test -p cargo-octet hegel_receipt"),
            parent_artifacts: Vec::new(),
            tool_exit_code: None,
        });
        mutate(&mut receipt);
        let receipt_json = crate::evidence_rails::render_receipt_json(&receipt).expect("render Hegel rail receipt");
        let summary = crate::evidence_rails::render_summary(&receipt);
        let provenance = crate::evidence_rails::render_provenance_jsonl(&receipt, &receipt_json, &summary)
            .expect("render Hegel rail provenance");
        evidence_rail_bundle("artifacts/hegel-property", receipt_json, summary, provenance)
    }

    fn evidence_rail_bundle(
        artifact_dir: &str,
        receipt_json: String,
        summary: String,
        provenance: String,
    ) -> ArtifactBundleInput {
        ArtifactBundleInput {
            artifact_dir: artifact_dir.to_owned(),
            files: vec![
                BundleFile {
                    name: crate::evidence_rails::RECEIPT_ARTIFACT_NAME.to_owned(),
                    contents: receipt_json.into_bytes(),
                },
                BundleFile {
                    name: crate::evidence_rails::SUMMARY_ARTIFACT_NAME.to_owned(),
                    contents: summary.into_bytes(),
                },
                BundleFile {
                    name: crate::evidence_rails::PROVENANCE_ARTIFACT_NAME.to_owned(),
                    contents: provenance.into_bytes(),
                },
            ],
        }
    }

    fn replace_file(input: &mut ArtifactBundleInput, file_name: &str, contents: Vec<u8>) {
        let file = input
            .files
            .iter_mut()
            .find(|file| file.name == file_name)
            .unwrap_or_else(|| panic!("missing fixture file {file_name}"));
        file.contents = contents;
    }

    fn sample_valence_proof_card(evidence_origin: &str, evidence_tier: &str, receipt_path: &str) -> ValenceProofCard {
        ValenceProofCard {
            proof_or_property_id: "proof.fixture".to_owned(),
            source_requirement_or_policy_id: "r[fixture]".to_owned(),
            policy_id: "r[fixture]".to_owned(),
            rail_index: 0,
            receipt_path: receipt_path.to_owned(),
            artifact_blake3_identities: vec![ValenceArtifactIdentity {
                artifact: receipt_path.to_owned(),
                blake3: crate::evidence::artifact_blake3_hex(b"receipt"),
            }],
            evidence_tier: evidence_tier.to_owned(),
            evidence_origin: evidence_origin.to_owned(),
            status: if evidence_tier == FORMAL_PROOF_TIER {
                STATUS_PASSED.to_owned()
            } else {
                STATUS_MANUAL_REVIEW_REQUIRED.to_owned()
            },
            non_claims: vec![VALENCE_EXPORT_NON_CLAIM_CORRECTNESS.to_owned()],
            external_reference: None,
            formal_proof_envelope: None,
            diagnostics: Vec::new(),
        }
    }
}
