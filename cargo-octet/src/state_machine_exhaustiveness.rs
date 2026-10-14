use serde::Deserialize;
use serde::Serialize;

const SCHEMA_VERSION: &str = "octet.state-machine-exhaustiveness.v1";
const CHECKER_ID: &str = "octet.state_machine_exhaustiveness";
const EVIDENCE_TIER: &str = "source_shape";
const SUPPORTED_CLAIM: &str = "marked transition core uses explicit state/event source-shape handling";
const REQUIRED_NON_CLAIM: &str = "state-machine exhaustiveness evidence is bounded source-shape evidence only; it does not prove whole-program correctness, liveness, protocol completeness outside the declared domain, verifier soundness, release eligibility, exhaustive Rust coverage, or formal proof obligations";
const STATUS_PASS: &str = "pass";
const STATUS_FAIL: &str = "fail";
const STATUS_UNSUPPORTED: &str = "unsupported";
const DIAGNOSTIC_OVERCLAIM: &str = "state_machine_receipt_overclaim";
const DIAGNOSTIC_MISSING_NON_CLAIM: &str = "state_machine_receipt_missing_non_claim";
const DIAGNOSTIC_INVALID_STATUS: &str = "state_machine_receipt_invalid_status";
const DIAGNOSTIC_STALE_COVERAGE: &str = "state_machine_receipt_stale_coverage";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct SourceSpanReceipt {
    file: String,
    start_line: u32,
    end_line: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct StateMachineReceiptInput {
    transition_id: String,
    source_span: SourceSpanReceipt,
    state_type: String,
    event_type: String,
    state_variants: Vec<String>,
    event_variants: Vec<String>,
    status: String,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct StateMachineReceipt {
    schema_version: String,
    checker_id: String,
    transition_id: String,
    source_span: SourceSpanReceipt,
    state_type: String,
    event_type: String,
    state_variants: Vec<String>,
    event_variants: Vec<String>,
    variant_inventory_digest: String,
    coverage_summary_digest: String,
    denial_coverage_digest: String,
    status: String,
    diagnostics: Vec<String>,
    supported_claim: String,
    non_claims: Vec<String>,
    artifact_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct ValenceStateMachineExportRow {
    evidence_id: String,
    evidence_tier: String,
    checker_id: String,
    transition_core_id: String,
    variant_inventory_digest: String,
    coverage_summary_digest: String,
    denial_coverage_digest: String,
    artifact_blake3: String,
    parent_export_identity: String,
    supported_claim: String,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReceiptDiagnostic {
    code: String,
    field: String,
}

fn build_state_machine_receipt(input: StateMachineReceiptInput) -> StateMachineReceipt {
    let variant_inventory_digest = digest_text(&variant_inventory_material(&input));
    let coverage_summary_digest = digest_text(&coverage_summary_material(&input, &variant_inventory_digest));
    let denial_coverage_digest = digest_text(&denial_coverage_material(&input, &coverage_summary_digest));
    let artifact_material =
        artifact_material(&input, &variant_inventory_digest, &coverage_summary_digest, &denial_coverage_digest);
    StateMachineReceipt {
        schema_version: SCHEMA_VERSION.to_owned(),
        checker_id: CHECKER_ID.to_owned(),
        transition_id: input.transition_id,
        source_span: input.source_span,
        state_type: input.state_type,
        event_type: input.event_type,
        state_variants: input.state_variants,
        event_variants: input.event_variants,
        variant_inventory_digest,
        coverage_summary_digest,
        denial_coverage_digest,
        status: input.status,
        diagnostics: input.diagnostics,
        supported_claim: SUPPORTED_CLAIM.to_owned(),
        non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        artifact_blake3: digest_text(&artifact_material),
    }
}

fn export_valence_state_machine_row(receipt: &StateMachineReceipt) -> ValenceStateMachineExportRow {
    ValenceStateMachineExportRow {
        evidence_id: format!("{}:{}", CHECKER_ID, receipt.transition_id),
        evidence_tier: EVIDENCE_TIER.to_owned(),
        checker_id: receipt.checker_id.clone(),
        transition_core_id: receipt.transition_id.clone(),
        variant_inventory_digest: receipt.variant_inventory_digest.clone(),
        coverage_summary_digest: receipt.coverage_summary_digest.clone(),
        denial_coverage_digest: receipt.denial_coverage_digest.clone(),
        artifact_blake3: receipt.artifact_blake3.clone(),
        parent_export_identity: digest_text(&format!("parent-export:{}", receipt.artifact_blake3)),
        supported_claim: receipt.supported_claim.clone(),
        non_claims: receipt.non_claims.clone(),
    }
}

fn validate_state_machine_receipt(receipt: &StateMachineReceipt) -> Vec<ReceiptDiagnostic> {
    let mut diagnostics = Vec::new();
    if !matches!(receipt.status.as_str(), STATUS_PASS | STATUS_FAIL | STATUS_UNSUPPORTED) {
        push_diagnostic(&mut diagnostics, DIAGNOSTIC_INVALID_STATUS, "status");
    }
    if !receipt.non_claims.iter().any(|claim| claim.contains("does not prove")) {
        push_diagnostic(&mut diagnostics, DIAGNOSTIC_MISSING_NON_CLAIM, "non_claims");
    }
    let input = receipt_input_from_receipt(receipt);
    let expected_inventory = digest_text(&variant_inventory_material(&input));
    let expected_coverage = digest_text(&coverage_summary_material(&input, &expected_inventory));
    let expected_denial = digest_text(&denial_coverage_material(&input, &expected_coverage));
    if receipt.variant_inventory_digest != expected_inventory || receipt.coverage_summary_digest != expected_coverage {
        push_diagnostic(&mut diagnostics, DIAGNOSTIC_STALE_COVERAGE, "coverage_summary_digest");
    }
    if receipt.denial_coverage_digest != expected_denial {
        push_diagnostic(&mut diagnostics, DIAGNOSTIC_STALE_COVERAGE, "denial_coverage_digest");
    }
    if overclaims(&receipt.supported_claim) || receipt.non_claims.iter().any(|claim| overclaims_without_negation(claim))
    {
        push_diagnostic(&mut diagnostics, DIAGNOSTIC_OVERCLAIM, "supported_claim");
    }
    diagnostics
}

fn overclaims(text: &str) -> bool {
    let normalized = text.to_ascii_lowercase();
    let has_negation = normalized.contains("does not prove") || normalized.contains("not prove");
    !has_negation
        && [
            "whole-program correctness",
            "liveness",
            "protocol completeness",
            "verifier soundness",
            "release eligibility",
            "exhaustive rust coverage",
            "formal proof",
        ]
        .iter()
        .any(|pattern| normalized.contains(pattern))
}

fn overclaims_without_negation(text: &str) -> bool {
    overclaims(text)
}

fn push_diagnostic(diagnostics: &mut Vec<ReceiptDiagnostic>, code: &str, field: &str) {
    diagnostics.push(ReceiptDiagnostic {
        code: code.to_owned(),
        field: field.to_owned(),
    });
}

fn digest_text(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

fn variant_inventory_material(input: &StateMachineReceiptInput) -> String {
    format!(
        "state_type={};event_type={};state_variants={};event_variants={}",
        input.state_type,
        input.event_type,
        input.state_variants.join(","),
        input.event_variants.join(",")
    )
}

fn coverage_summary_material(input: &StateMachineReceiptInput, variant_inventory_digest: &str) -> String {
    format!(
        "inventory={variant_inventory_digest};status={};diagnostics={}",
        input.status,
        input.diagnostics.join(",")
    )
}

fn denial_coverage_material(input: &StateMachineReceiptInput, coverage_summary_digest: &str) -> String {
    format!(
        "coverage={coverage_summary_digest};invalid_transitions=explicit;status={};diagnostics={}",
        input.status,
        input.diagnostics.join(",")
    )
}

fn artifact_material(
    input: &StateMachineReceiptInput,
    variant_inventory_digest: &str,
    coverage_summary_digest: &str,
    denial_coverage_digest: &str,
) -> String {
    format!(
        "schema={SCHEMA_VERSION};checker={CHECKER_ID};transition={};span={}:{}-{};inventory={variant_inventory_digest};coverage={coverage_summary_digest};denial_coverage={denial_coverage_digest};status={};diagnostics={}",
        input.transition_id,
        input.source_span.file,
        input.source_span.start_line,
        input.source_span.end_line,
        input.status,
        input.diagnostics.join(",")
    )
}

fn receipt_input_from_receipt(receipt: &StateMachineReceipt) -> StateMachineReceiptInput {
    StateMachineReceiptInput {
        transition_id: receipt.transition_id.clone(),
        source_span: receipt.source_span.clone(),
        state_type: receipt.state_type.clone(),
        event_type: receipt.event_type.clone(),
        state_variants: receipt.state_variants.clone(),
        event_variants: receipt.event_variants.clone(),
        status: receipt.status.clone(),
        diagnostics: receipt.diagnostics.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE_START_LINE: u32 = 10;
    const FIXTURE_END_LINE: u32 = 40;

    fn valid_input() -> StateMachineReceiptInput {
        StateMachineReceiptInput {
            transition_id: "aspen::hlc::transition".to_owned(),
            source_span: SourceSpanReceipt {
                file: "src/hlc.rs".to_owned(),
                start_line: FIXTURE_START_LINE,
                end_line: FIXTURE_END_LINE,
            },
            state_type: "ClockState".to_owned(),
            event_type: "ClockEvent".to_owned(),
            state_variants: vec!["Idle".to_owned(), "Running".to_owned()],
            event_variants: vec!["Start".to_owned(), "Stop".to_owned()],
            status: STATUS_PASS.to_owned(),
            diagnostics: Vec::new(),
        }
    }

    #[test]
    fn state_machine_receipt_binds_identity_and_exports_valence_row() {
        let receipt = build_state_machine_receipt(valid_input());
        let row = export_valence_state_machine_row(&receipt);

        assert_eq!(receipt.schema_version, SCHEMA_VERSION);
        assert_eq!(receipt.checker_id, CHECKER_ID);
        assert!(receipt.variant_inventory_digest.chars().all(|candidate| candidate.is_ascii_hexdigit()));
        assert!(receipt.coverage_summary_digest.chars().all(|candidate| candidate.is_ascii_hexdigit()));
        assert!(receipt.denial_coverage_digest.chars().all(|candidate| candidate.is_ascii_hexdigit()));
        assert!(receipt.artifact_blake3.chars().all(|candidate| candidate.is_ascii_hexdigit()));
        assert_eq!(row.evidence_tier, EVIDENCE_TIER);
        assert_eq!(row.transition_core_id, receipt.transition_id);
        assert_eq!(row.coverage_summary_digest, receipt.coverage_summary_digest);
        assert_eq!(row.denial_coverage_digest, receipt.denial_coverage_digest);
        assert_eq!(row.artifact_blake3, receipt.artifact_blake3);
        assert_eq!(validate_state_machine_receipt(&receipt), Vec::new());
    }

    #[test]
    fn state_machine_receipt_rejects_overclaims_missing_non_claims_and_bad_status() {
        let mut receipt = build_state_machine_receipt(valid_input());
        receipt.status = "proved".to_owned();
        receipt.supported_claim = "formal proof of whole-program correctness".to_owned();
        receipt.non_claims.clear();

        let diagnostics = validate_state_machine_receipt(&receipt);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_INVALID_STATUS));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_MISSING_NON_CLAIM));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_OVERCLAIM));
    }

    #[test]
    fn state_machine_receipt_rejects_stale_coverage_identity() {
        let mut receipt = build_state_machine_receipt(valid_input());
        receipt.coverage_summary_digest = "stale".to_owned();

        let diagnostics = validate_state_machine_receipt(&receipt);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_STALE_COVERAGE));
    }

    #[test]
    fn state_machine_receipt_rejects_stale_denial_coverage_identity() {
        let mut receipt = build_state_machine_receipt(valid_input());
        receipt.denial_coverage_digest = "stale".to_owned();

        let diagnostics = validate_state_machine_receipt(&receipt);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.field == "denial_coverage_digest"));
    }
}
