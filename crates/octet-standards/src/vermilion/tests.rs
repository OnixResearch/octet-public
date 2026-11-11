// r[verify octet.vermilion_proof_rail.negative_controls]
// r[verify octet.vermilion_proof_rail.comparison]
use pretty_assertions::assert_eq;

use super::comparison::ComparisonCategory;
use super::comparison::RouteClaim;
use super::comparison::RouteEvidence;
use super::comparison::RouteOutcome;
use super::comparison::compare;
use super::*;

const RECEIPT_FIXTURE: &str =
    include_str!("../../../../standards/proof-rails/vermilion-lean/fixtures/valid/complete-receipt.json");
const NEGATIVE_MATRIX: &str =
    include_str!("../../../../standards/proof-rails/vermilion-lean/fixtures/negative-matrix.json");
const OTHER_B3: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

fn fixture_receipt() -> VermilionReceipt {
    let mut receipt: VermilionReceipt = serde_json::from_str(RECEIPT_FIXTURE).expect("parse receipt fixture");
    if receipt.receipt_identity.is_empty() {
        receipt.receipt_identity = receipt_identity(&receipt).expect("seal fixture receipt");
    }
    receipt
}

#[derive(serde::Deserialize)]
struct NegativeMatrix {
    cases: Vec<NegativeCase>,
}

#[derive(serde::Deserialize)]
struct NegativeCase {
    case_id: String,
    mutation: String,
    expected_outcome: String,
    expected_diagnostic: String,
}

#[test]
fn checked_profile_and_contract_are_exact_and_complete() {
    let profile = selected_profile().expect("checked provider profile");
    assert_eq!(validate_profile(&profile), Vec::new());
    assert_eq!(profile.profile_id, EXPECTED_PROFILE_ID);
    assert_eq!(profile.provider_id, PROVIDER_ID);
    assert_eq!(profile.verus_fork.revision, "48600836424a79420589c56d86e720236a029811");
    assert_eq!(profile.lean.toolchain, "leanprover/lean4:v4.33.0");
    assert_eq!(profile.checker.checker_id, "vrml_check");
    assert_eq!(profile.limits.timeout_ms, 900_000);

    let contract = selected_contract().expect("checked pilot contract");
    assert_eq!(validate_contract(&contract), Vec::new());
    assert_eq!(contract.contract_id, CONTRACT_ID);
    assert_eq!(contract.width_bits, 64);
    assert_eq!(contract.overflow_behavior, "checked");
    assert_eq!(contract.precondition, "value < u64::MAX");
    assert_eq!(contract.routes.len(), 3);
}

#[test]
fn contract_binds_the_actual_pilot_bytes_and_statement() {
    let contract = selected_contract().expect("checked pilot contract");
    assert_eq!(contract.subject_blake3, prefixed_identity(PILOT_SOURCE.as_bytes()));
    assert_eq!(contract.statement_blake3, prefixed_identity(PILOT_SPEC_TEXT.as_bytes()));
    assert_eq!(contract.subject_path, "standards/proof-rails/vermilion-lean/pilot/src/lib.rs");
    assert!(
        contract
            .routes
            .iter()
            .any(|route| route.backend == "verus" && route.refinement_bridge.contains("plain-Rust"))
    );
    assert!(contract.routes.iter().any(|route| route.backend == "aeneas-lean"));
}

#[test]
fn complete_contract_fixture_is_supported_but_recorded_only() {
    let receipt = fixture_receipt();
    assert_eq!(validate_receipt(&receipt), Vec::new());
    let evaluation = evaluate(&receipt, ToolchainAvailability::Available);
    assert_eq!(evaluation.outcome, VermilionOutcome::Supported);
    assert_eq!(evaluation.diagnostic, DiagnosticClass::None);
    assert_eq!(evaluation.evidence_disposition, EvidenceDisposition::RecordedOnly);
    assert!(!evaluation.property_evidence_admitted);
    assert_eq!(evaluation.open_obligations, vec!["pilot.increment.refinement-to-spec".to_owned()]);
    assert_eq!(receipt.origin, ReceiptOrigin::SyntheticContractFixture);
}

#[test]
fn receipt_fixture_is_sealed_and_round_trips() {
    let receipt = fixture_receipt();
    let bytes = serde_json::to_vec_pretty(&receipt).expect("serialize receipt");
    let reparsed: VermilionReceipt = serde_json::from_slice(&bytes).expect("parse serialized receipt");
    assert_eq!(reparsed, receipt);
    assert_eq!(receipt_identity(&reparsed).expect("receipt identity"), receipt.receipt_identity);
}

#[test]
fn absent_toolchain_is_blocked_not_supported() {
    let receipt = fixture_receipt();
    let evaluation = evaluate(&receipt, ToolchainAvailability::Unavailable);
    assert_eq!(evaluation.outcome, VermilionOutcome::Blocked);
    assert_eq!(evaluation.diagnostic, DiagnosticClass::MissingChecker);
    assert_eq!(evaluation.evidence_disposition, EvidenceDisposition::RecordedOnly);
    assert!(!evaluation.property_evidence_admitted);
}

#[test]
fn negative_controls_fail_at_their_declared_gate() {
    let matrix: NegativeMatrix = serde_json::from_str(NEGATIVE_MATRIX).expect("parse negative matrix");
    assert_eq!(matrix.cases.len(), 20);
    for case in &matrix.cases {
        let receipt = mutation(&case.mutation);
        let evaluation = if case.mutation == "toolchain-unavailable" {
            evaluate(&receipt, ToolchainAvailability::Unavailable)
        } else {
            evaluate(&receipt, ToolchainAvailability::Available)
        };
        assert_eq!(
            evaluation.outcome.as_str(),
            case.expected_outcome,
            "case {} outcome\ndiagnostics: {:?}",
            case.case_id,
            evaluation.diagnostics
        );
        assert_eq!(
            evaluation.diagnostic.as_str(),
            case.expected_diagnostic,
            "case {} diagnostic\ndiagnostics: {:?}",
            case.case_id,
            evaluation.diagnostics
        );
        assert!(!evaluation.property_evidence_admitted, "case {} admitted property evidence", case.case_id);
    }
}

fn mutation(name: &str) -> VermilionReceipt {
    let mut receipt = fixture_receipt();
    match name {
        "replace-source" => {
            receipt.source_tree[0].path = "src/other_increment.rs".to_owned();
            receipt.source_tree[0].blake3 = OTHER_B3.to_owned();
        }
        "replace-spec-revision" => receipt.semantic_spec.revision = "2099-01-01".to_owned(),
        "weaken-precondition" => receipt.semantic_spec.precondition = "value <= u64::MAX".to_owned(),
        "wrapping-overflow" => receipt.semantic_spec.overflow_behavior = "wrapping".to_owned(),
        "nat-width" => receipt.semantic_spec.width_bits = 32,
        "aeneas-provider" => {
            receipt.provider.provider_id = "aeneas-lean".to_owned();
            receipt.toolchain.backend = "aeneas-lean".to_owned();
        }
        "stale-artifact" => {
            receipt.generated_artifacts[0].blake3 = OTHER_B3.to_owned();
        }
        "stale-twin" => {
            receipt.generated_artifacts[2].blake3 = OTHER_B3.to_owned();
        }
        "remove-obligation" => {
            receipt.obligations.remove(0);
            receipt.proof_inventory.theorems.remove(0);
            receipt
                .generated_artifacts
                .retain(|artifact| artifact.role != "proof-twin:pilot.increment.overflow-check");
            receipt.proof_inventory.axioms = vec!["propext".to_owned()];
            receipt.checker.required_obligations = 1;
            receipt.checker.checked_obligations = 1;
        }
        "add-obligation" => {
            let mut extra = receipt.obligations[1].clone();
            extra.obligation_id = "pilot.increment.extra".to_owned();
            extra.proof_twin_role = "proof-twin:pilot.increment.extra".to_owned();
            extra.proof_twin_blake3 = "b3:5555555555555555555555555555555555555555555555555555555555555555".to_owned();
            extra.theorem = "pilot.increment_extra".to_owned();
            receipt.obligations.push(extra.clone());
            receipt.generated_artifacts.push(ProofArtifact {
                role: extra.proof_twin_role.clone(),
                path: "standards/proof-rails/vermilion-lean/fixtures/valid/proofs/extra.lean".to_owned(),
                blake3: extra.proof_twin_blake3.clone(),
            });
            receipt.proof_inventory.theorems.push(TheoremRecord {
                obligation_id: extra.obligation_id,
                theorem: extra.theorem,
                assumptions: extra.assumptions,
            });
            receipt.checker.required_obligations = 3;
            receipt.checker.checked_obligations = 3;
        }
        "add-axiom" => {
            receipt.obligations[0].assumptions.push("Classical.choiceX".to_owned());
            receipt.proof_inventory.theorems[0].assumptions.push("Classical.choiceX".to_owned());
            receipt.proof_inventory.axioms.push("Classical.choiceX".to_owned());
        }
        "sorry-assumption" => {
            receipt.obligations[0].assumptions.push("sorryAx".to_owned());
            receipt.proof_inventory.theorems[0].assumptions.push("sorryAx".to_owned());
            receipt.proof_inventory.axioms.push("sorryAx".to_owned());
        }
        "interrupt-checker" => {
            receipt.checker.status = CheckerStatus::Interrupted;
            receipt.checker.exit_code = None;
            receipt.checker.complete = false;
        }
        "empty-output" => receipt.checker.output_empty = true,
        "unavailable-checker" => {
            receipt.checker.status = CheckerStatus::Unavailable;
            receipt.checker.exit_code = None;
            receipt.checker.complete = false;
            receipt.checker.diagnostic_identity = Some(OTHER_B3.to_owned());
        }
        "stale-lake-manifest" => receipt.toolchain.lake_manifest_blake3 = OTHER_B3.to_owned(),
        "tamper-receipt" => receipt.receipt_identity = OTHER_B3.to_owned(),
        "drop-non-claim" => {
            receipt.non_claims.pop();
        }
        "synthetic-property" => receipt.evidence_disposition = EvidenceDisposition::Property,
        "toolchain-unavailable" => {}
        other => panic!("unknown negative mutation `{other}`"),
    }
    if name != "tamper-receipt" {
        receipt.receipt_identity = receipt_identity(&receipt).expect("reseal mutated receipt");
    }
    receipt
}

fn supported_claim(property: &str) -> RouteClaim {
    RouteClaim {
        property_id: property.to_owned(),
        statement_blake3: "b3:af259da355a01524f6030f09ee196f20cb675991e4421204b63093fd017d38c4".to_owned(),
        width_bits: 64,
        overflow_behavior: "checked".to_owned(),
        precondition: "value < u64::MAX".to_owned(),
        result_type: "Option<u64>".to_owned(),
    }
}

fn route(backend: &str, claim: RouteClaim, outcome: RouteOutcome, bridge_checked: bool) -> RouteEvidence {
    RouteEvidence {
        backend: backend.to_owned(),
        scope: format!("formal-proof:{backend}:pilot#rail-0"),
        claim,
        outcome,
        refinement_bridge_id: format!("{backend}.pilot.refinement"),
        refinement_bridge_checked: bridge_checked,
    }
}

#[test]
fn nat_lemma_route_is_incomparable_with_the_u64_contract() {
    let routes = vec![
        route(
            "vermilion-lean",
            supported_claim("octet.bounded_increment.checked/v1"),
            RouteOutcome::Supported,
            true,
        ),
        route("verus", supported_claim("verus.nat.monotonic-add"), RouteOutcome::Supported, false),
    ];
    let report = compare(&routes);
    assert_eq!(report.category, ComparisonCategory::IncomparableClaims);
    assert_eq!(report.agreed_property_id, None);
    assert!(!report.evidence_merge);
    assert!(report.open_obligations.contains(&"verus.pilot.refinement".to_owned()));
    assert_eq!(report.aeneas_outcome, None);
    assert_eq!(report.vermilion_outcome, None);
}

#[test]
fn u64_route_agreement_does_not_merge_authority() {
    let routes = vec![
        route(
            "vermilion-lean",
            supported_claim("octet.bounded_increment.checked/v1"),
            RouteOutcome::Supported,
            true,
        ),
        route("aeneas-lean", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Supported, true),
    ];
    let report = compare(&routes);
    assert_eq!(report.category, ComparisonCategory::ComparableAgreement);
    assert_eq!(report.agreed_property_id.as_deref(), Some("octet.bounded_increment.checked/v1"));
    assert!(!report.evidence_merge);
    assert!(report.rows.iter().all(|row| row.comparable));
    assert_eq!(report.aeneas_outcome, None);
    assert_eq!(report.vermilion_outcome, None);
}

#[test]
fn semantic_and_tool_outcomes_map_to_existing_vocabulary() {
    let mut u32_claim = supported_claim("octet.bounded_increment.checked/v1");
    u32_claim.width_bits = 32;
    let mismatch = compare(&[
        route(
            "vermilion-lean",
            supported_claim("octet.bounded_increment.checked/v1"),
            RouteOutcome::Supported,
            true,
        ),
        route("aeneas-lean", u32_claim, RouteOutcome::Supported, true),
    ]);
    assert_eq!(mismatch.category, ComparisonCategory::SemanticMismatch);
    assert_eq!(mismatch.aeneas_outcome.as_deref(), Some("failed"));

    let failure = compare(&[
        route("vermilion-lean", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Failed, true),
        route("aeneas-lean", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Supported, true),
    ]);
    assert_eq!(failure.category, ComparisonCategory::ProofFailure);

    let blocked = compare(&[
        route("vermilion-lean", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Blocked, true),
        route("aeneas-lean", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Supported, true),
    ]);
    assert_eq!(blocked.category, ComparisonCategory::UnsupportedOrBlocked);
    assert_eq!(blocked.aeneas_outcome.as_deref(), Some("blocked"));

    let timeout = compare(&[
        route(
            "vermilion-lean",
            supported_claim("octet.bounded_increment.checked/v1"),
            RouteOutcome::TimedOut,
            true,
        ),
        route("aeneas-lean", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Supported, true),
    ]);
    assert_eq!(timeout.category, ComparisonCategory::TimeoutOrExhaustion);
    assert_eq!(timeout.vermilion_outcome.as_deref(), Some("failed"));

    let tool_error = compare(&[
        route(
            "vermilion-lean",
            supported_claim("octet.bounded_increment.checked/v1"),
            RouteOutcome::ToolError,
            true,
        ),
        route("aeneas-lean", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Supported, true),
    ]);
    assert_eq!(tool_error.category, ComparisonCategory::ToolError);
}

#[test]
fn comparison_requires_at_least_two_routes_and_visible_bridges() {
    let single = compare(&[route(
        "vermilion-lean",
        supported_claim("octet.bounded_increment.checked/v1"),
        RouteOutcome::Supported,
        true,
    )]);
    assert_eq!(single.category, ComparisonCategory::IncomparableClaims);

    let open_bridge = compare(&[
        route(
            "vermilion-lean",
            supported_claim("octet.bounded_increment.checked/v1"),
            RouteOutcome::Supported,
            false,
        ),
        route("aeneas-lean", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Supported, true),
    ]);
    assert_eq!(open_bridge.category, ComparisonCategory::IncomparableClaims);
    assert_eq!(open_bridge.open_obligations, vec!["vermilion-lean.pilot.refinement".to_owned()]);
}

#[test]
fn u64_pilot_routes_remain_incomparable_until_executable_bridges_close() {
    let routes = vec![
        route("aeneas-lean", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Supported, true),
        route(
            "vermilion-lean",
            supported_claim("octet.bounded_increment.checked/v1"),
            RouteOutcome::Supported,
            false,
        ),
        route("verus", supported_claim("octet.bounded_increment.checked/v1"), RouteOutcome::Supported, false),
    ];
    let report = compare(&routes);
    assert_eq!(report.category, ComparisonCategory::IncomparableClaims);
    assert_eq!(report.agreed_property_id, None);
    assert_eq!(report.open_obligations, vec![
        "vermilion-lean.pilot.refinement".to_owned(),
        "verus.pilot.refinement".to_owned()
    ]);
    let row = |backend: &str| report.rows.iter().find(|row| row.backend == backend).expect("route row");
    assert!(row("aeneas-lean").comparable);
    assert!(!row("vermilion-lean").comparable);
    assert!(!row("verus").comparable);
    assert_eq!(
        row("vermilion-lean").non_comparable_reason.as_deref(),
        Some("refinement bridge is an open obligation")
    );
}
