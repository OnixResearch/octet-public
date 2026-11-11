use std::collections::BTreeSet;
use std::path::Path;

use super::BridgeStatus;
use super::CONTRACT_ID;
use super::CONTRACT_SCHEMA;
use super::CheckerStatus;
use super::Diagnostic;
use super::DiagnosticClass;
use super::EvidenceDisposition;
use super::ObligationStatus;
use super::PILOT_SOURCE;
use super::PILOT_SPEC_TEXT;
use super::PROFILE_SCHEMA;
use super::PROVIDER_ID;
use super::RECEIPT_SCHEMA;
use super::REQUIRED_NON_CLAIMS;
use super::ReceiptOrigin;
use super::diagnostic;
use super::is_b3;
use super::model::ProviderProfile;
use super::model::SemanticContract;
use super::model::VermilionReceipt;
use super::prefixed_identity;

const PROFILE_TIMEOUT_MAX_MS: u64 = 900_000;
const PROFILE_LOG_MAX_BYTES: usize = 1_048_576;
const VERMILION_ROUTE: &str = "vermilion-lean";
const BACKEND: &str = "lean4";
const GENERATED_LEAN_ROLE: &str = "generated-lean";
const OBLIGATION_MANIFEST_ROLE: &str = "obligation-manifest";
const PROFILE_NON_CLAIMS: [&str; 6] = [
    "does not prove Vermilion, Verus, or Lean soundness",
    "does not prove the Verus front end, Vermilion lowering, or Lean generator correct",
    "does not prove whole-program correctness",
    "does not prove semantic equivalence",
    "does not prove unsafe Rust or concurrent Rust",
    "does not establish release eligibility",
];

pub(super) fn profile(profile: &ProviderProfile) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if profile.schema_version != PROFILE_SCHEMA
        || profile.provider_id != PROVIDER_ID
        || profile.profile_id.trim().is_empty()
        || profile.claim_scope != "per-obligation-lean-kernel-theorem-over-pinned-verus-frontend"
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::WrongProvider,
            "profile",
            "Vermilion provider identity or claim scope drifted",
        ));
    }
    validate_verus_fork(profile, &mut diagnostics);
    validate_lean(profile, &mut diagnostics);
    validate_checker(profile, &mut diagnostics);
    validate_environment(profile, &mut diagnostics);
    validate_non_claims(&profile.non_claims, &PROFILE_NON_CLAIMS, "profile", &mut diagnostics);
    diagnostics.sort_by(|left, right| left.class.as_str().cmp(right.class.as_str()));
    diagnostics.dedup();
    diagnostics
}

fn validate_verus_fork(profile: &ProviderProfile, diagnostics: &mut Vec<Diagnostic>) {
    let fork = &profile.verus_fork;
    if fork.repository != "https://github.com/ilyasergey/verus"
        || fork.branch != "dev"
        || !is_git_revision(&fork.revision)
        || fork.vstd_revision != fork.revision
        || !is_git_revision(&profile.vermilion_revision)
        || profile.translator.revision != profile.vermilion_revision
        || profile.translator.argv.is_empty()
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::ToolchainDrift,
            "verus_fork",
            "Vermilion/Verus fork, vstd, or translator pin is incomplete",
        ));
    }
}

fn validate_lean(profile: &ProviderProfile, diagnostics: &mut Vec<Diagnostic>) {
    let lean = &profile.lean;
    if lean.toolchain != "leanprover/lean4:v4.33.0"
        || !is_git_revision(&lean.mathlib_revision)
        || !is_b3(&lean.lake_manifest_blake3)
        || lean.packages.is_empty()
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::ToolchainDrift,
            "lean",
            "Lean toolchain, Mathlib, or dependency audit pin is incomplete",
        ));
    }
    let mut names = BTreeSet::new();
    for package in &lean.packages {
        if package.name.trim().is_empty() || !is_git_revision(&package.revision) || !names.insert(&package.name) {
            diagnostics.push(diagnostic(
                DiagnosticClass::ToolchainDrift,
                "lean.packages",
                "Lean dependency pin is malformed or duplicated",
            ));
        }
    }
    if !lean
        .packages
        .iter()
        .any(|package| package.name == "mathlib" && package.revision == lean.mathlib_revision)
        || !lean.packages.iter().any(|package| package.name == "cvc5")
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::MissingDependencyAudit,
            "lean.packages",
            "Mathlib or cvc5 dependency pin is missing from the manifest",
        ));
    }
}

fn validate_checker(profile: &ProviderProfile, diagnostics: &mut Vec<Diagnostic>) {
    let checker = &profile.checker;
    if checker.checker_id != "vrml_check"
        || !is_git_revision(&checker.revision)
        || checker.argv.is_empty()
        || !checker.requires_twin_check
        || !profile.assumption_policy.requires_complete_declaration
        || !profile.assumption_policy.require_twin_check
        || profile.assumption_policy.allowed_axioms != ["propext", "Classical.choice", "Quot.sound"]
        || !profile.assumption_policy.proof_holes.iter().any(|hole| hole == "sorry")
        || !profile.assumption_policy.proof_holes.iter().any(|hole| hole == "admit")
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::ToolchainDrift,
            "checker",
            "checker identity, twin requirement, or assumption policy drifted",
        ));
    }
}

fn validate_environment(profile: &ProviderProfile, diagnostics: &mut Vec<Diagnostic>) {
    let expected_environment = [
        "PATH",
        "HOME",
        "CARGO_HOME",
        "RUSTUP_HOME",
        "LEAN_PATH",
        "VERUS_VIR_EXPORT",
        "MATHLIB_NO_CACHE_ON_UPDATE",
    ];
    let environment = profile.environment_allowlist.iter().map(String::as_str).collect::<Vec<_>>();
    let limits = &profile.limits;
    if environment.as_slice() != expected_environment
        || limits.timeout_ms == 0
        || limits.timeout_ms > PROFILE_TIMEOUT_MAX_MS
        || limits.stdout_max_bytes == 0
        || limits.stdout_max_bytes > PROFILE_LOG_MAX_BYTES
        || limits.stderr_max_bytes == 0
        || limits.stderr_max_bytes > PROFILE_LOG_MAX_BYTES
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::CommandDrift,
            "environment_allowlist",
            "provider environment or resource bounds exceed the reviewed policy",
        ));
    }
}

pub(super) fn contract(contract: &SemanticContract) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if contract.schema_version != CONTRACT_SCHEMA
        || contract.contract_id != CONTRACT_ID
        || contract.revision.trim().is_empty()
        || contract.width_bits != 64
        || contract.overflow_behavior != "checked"
        || contract.precondition != "value < u64::MAX"
        || contract.postcondition != "result == Some(value + 1)"
        || contract.result_type != "Option<u64>"
        || contract.termination_scope != "single-call, no loops"
        || contract.observable_failures.is_empty()
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::WeakenedPrecondition,
            "contract",
            "pilot contract width, overflow, precondition, or termination scope drifted",
        ));
    }
    if unsafe_path(&contract.subject_path) || !is_b3(&contract.subject_blake3) {
        diagnostics.push(diagnostic(
            DiagnosticClass::SourceSubstitution,
            "contract.subject",
            "pilot subject path or identity is unsafe",
        ));
    } else if contract.subject_blake3 != prefixed_identity(PILOT_SOURCE.as_bytes()) {
        diagnostics.push(diagnostic(
            DiagnosticClass::SourceSubstitution,
            "contract.subject_blake3",
            "pilot subject bytes do not match the contract binding",
        ));
    }
    if !is_b3(&contract.statement_blake3) || contract.statement_blake3 != prefixed_identity(PILOT_SPEC_TEXT.as_bytes())
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::SpecSubstitution,
            "contract.statement_blake3",
            "pilot statement bytes do not match the contract binding",
        ));
    }
    let backends = contract.routes.iter().map(|route| route.backend.as_str()).collect::<BTreeSet<_>>();
    if backends != BTreeSet::from(["aeneas-lean", "vermilion-lean", "verus"]) {
        diagnostics.push(diagnostic(
            DiagnosticClass::SpecSubstitution,
            "contract.routes",
            "pilot contract must declare exactly the Verus, Vermilion, and Aeneas routes",
        ));
    }
    for route in &contract.routes {
        let ids = route.required_obligations.iter().map(|item| item.obligation_id.as_str()).collect::<BTreeSet<_>>();
        if route.required_obligations.is_empty()
            || ids.len() != route.required_obligations.len()
            || route.refinement_bridge_id.trim().is_empty()
            || route.refinement_bridge.trim().is_empty()
            || route.claim_scope.trim().is_empty()
        {
            diagnostics.push(diagnostic(
                DiagnosticClass::MissingObligation,
                "contract.routes",
                "route obligation or refinement-bridge declaration is incomplete",
            ));
        }
    }
    validate_non_claims(&contract.non_claims, &REQUIRED_NON_CLAIMS, "contract", &mut diagnostics);
    diagnostics.sort_by(|left, right| left.class.as_str().cmp(right.class.as_str()));
    diagnostics.dedup();
    diagnostics
}

pub(super) fn receipt(
    receipt: &VermilionReceipt,
    profile: &ProviderProfile,
    profile_identity: &str,
    contract: &SemanticContract,
    contract_identity: &str,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let vermilion = contract.routes.iter().find(|route| route.backend == VERMILION_ROUTE);
    if receipt.schema_version != RECEIPT_SCHEMA {
        diagnostics.push(diagnostic(DiagnosticClass::SchemaDrift, "schema_version", "receipt schema drifted"));
    }
    validate_provider(receipt, profile, profile_identity, contract, contract_identity, &mut diagnostics);
    validate_spec(receipt, contract, &mut diagnostics);
    validate_source(receipt, contract, &mut diagnostics);
    validate_toolchain(receipt, profile, &mut diagnostics);
    validate_command(receipt, profile, &mut diagnostics);
    validate_obligations(receipt, vermilion, &mut diagnostics);
    validate_artifacts(receipt, &mut diagnostics);
    validate_bridges(receipt, vermilion, &mut diagnostics);
    validate_checker_observation(receipt, &mut diagnostics);
    validate_inventory(receipt, profile, &mut diagnostics);
    validate_authority(receipt, &mut diagnostics);
    if receipt.non_claims != contract.non_claims {
        diagnostics.push(diagnostic(
            DiagnosticClass::NonClaimDrift,
            "non_claims",
            "receipt claim boundaries drifted from the reviewed contract",
        ));
    }
    match super::receipt_identity(receipt) {
        Ok(expected) if receipt.receipt_identity == expected => {}
        Ok(_) => diagnostics.push(diagnostic(
            DiagnosticClass::ReceiptTamper,
            "receipt_identity",
            "receipt projection identity is stale",
        )),
        Err(error) => diagnostics.push(diagnostic(DiagnosticClass::ReceiptTamper, "receipt_identity", &error)),
    }
    diagnostics.sort_by(|left, right| left.class.as_str().cmp(right.class.as_str()));
    diagnostics.dedup();
    diagnostics
}

fn validate_provider(
    receipt: &VermilionReceipt,
    profile: &ProviderProfile,
    profile_identity: &str,
    contract: &SemanticContract,
    contract_identity: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if receipt.provider.provider_id != PROVIDER_ID
        || receipt.provider.profile_id != profile.profile_id
        || receipt.provider.profile_identity != profile_identity
        || receipt.provider.compatibility_matrix_identity != super::COMPATIBILITY_MATRIX_IDENTITY
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::WrongProvider,
            "provider",
            "receipt provider or profile identity is not the reviewed Vermilion cohort",
        ));
    }
    if receipt.semantic_spec.contract_id != contract.contract_id
        || receipt.semantic_spec.revision != contract.revision
        || receipt.semantic_spec.statement_blake3 != contract.statement_blake3
        || contract_identity != super::CONTRACT_IDENTITY
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::SpecSubstitution,
            "semantic_spec",
            "receipt semantic-spec binding is not the reviewed contract revision",
        ));
    }
}

fn validate_spec(receipt: &VermilionReceipt, contract: &SemanticContract, diagnostics: &mut Vec<Diagnostic>) {
    let spec = &receipt.semantic_spec;
    if spec.precondition != contract.precondition || spec.postcondition != contract.postcondition {
        diagnostics.push(diagnostic(
            DiagnosticClass::WeakenedPrecondition,
            "semantic_spec.precondition",
            "receipt precondition or postcondition weakens the common contract",
        ));
    }
    if spec.overflow_behavior != contract.overflow_behavior {
        diagnostics.push(diagnostic(
            DiagnosticClass::ChangedOverflowSemantics,
            "semantic_spec.overflow_behavior",
            "receipt overflow behavior differs from the common checked contract",
        ));
    }
    if spec.width_bits != contract.width_bits {
        diagnostics.push(diagnostic(
            DiagnosticClass::NatU64Confusion,
            "semantic_spec.width_bits",
            "receipt arithmetic width differs from the u64 contract",
        ));
    }
}

fn validate_source(receipt: &VermilionReceipt, contract: &SemanticContract, diagnostics: &mut Vec<Diagnostic>) {
    if receipt.source_tree.len() != 1 {
        diagnostics.push(diagnostic(
            DiagnosticClass::SourceSubstitution,
            "source_tree",
            "receipt must bind exactly the selected pilot source",
        ));
        return;
    }
    let source = &receipt.source_tree[0];
    if source.path != contract.subject_path
        || source.blake3 != contract.subject_blake3
        || source.item != "increment_checked"
        || source.begin_line == 0
        || source.end_line < source.begin_line
        || !is_b3(&source.blake3)
        || unsafe_path(&source.path)
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::SourceSubstitution,
            "source_tree",
            "receipt source path, item, span, or bytes were substituted",
        ));
    }
}

fn validate_toolchain(receipt: &VermilionReceipt, profile: &ProviderProfile, diagnostics: &mut Vec<Diagnostic>) {
    let toolchain = &receipt.toolchain;
    if toolchain.backend != BACKEND {
        diagnostics.push(diagnostic(
            DiagnosticClass::WrongProvider,
            "toolchain.backend",
            "receipt backend is not the reviewed Lean backend",
        ));
    }
    if toolchain.verus_frontend_revision != profile.verus_fork.revision
        || toolchain.vstd_revision != profile.verus_fork.vstd_revision
        || toolchain.translator_revision != profile.translator.revision
        || toolchain.lean_toolchain != profile.lean.toolchain
        || toolchain.mathlib_revision != profile.lean.mathlib_revision
        || !toolchain.solver_revision.eq_ignore_ascii_case(
            profile
                .lean
                .packages
                .iter()
                .find(|package| package.name == "cvc5")
                .map_or("", |package| package.revision.as_str()),
        )
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::ToolchainDrift,
            "toolchain",
            "receipt toolchain identities drifted from the reviewed profile",
        ));
    }
    if toolchain.lake_manifest_blake3 != profile.lean.lake_manifest_blake3 || !is_b3(&toolchain.lake_manifest_blake3) {
        diagnostics.push(diagnostic(
            DiagnosticClass::MissingDependencyAudit,
            "toolchain.lake_manifest_blake3",
            "receipt dependency audit is missing or bound to other manifest bytes",
        ));
    }
}

fn validate_command(receipt: &VermilionReceipt, profile: &ProviderProfile, diagnostics: &mut Vec<Diagnostic>) {
    let command = &receipt.command;
    let limits = &command.limits;
    if command.argv != profile.checker.argv
        || command.environment_allowlist != profile.environment_allowlist
        || limits.timeout_ms == 0
        || limits.timeout_ms > profile.limits.timeout_ms
        || limits.stdout_max_bytes == 0
        || limits.stdout_max_bytes > profile.limits.stdout_max_bytes
        || limits.stderr_max_bytes == 0
        || limits.stderr_max_bytes > profile.limits.stderr_max_bytes
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::CommandDrift,
            "command",
            "receipt command, environment, or limits exceed the reviewed policy",
        ));
    }
}

fn validate_obligations(
    receipt: &VermilionReceipt,
    route: Option<&super::RouteObligations>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(route) = route else {
        diagnostics.push(diagnostic(
            DiagnosticClass::SpecSubstitution,
            "contract.routes",
            "contract has no Vermilion route",
        ));
        return;
    };
    let required = route
        .required_obligations
        .iter()
        .map(|item| (item.obligation_id.as_str(), item.kind.as_str()))
        .collect::<BTreeSet<_>>();
    let observed = receipt
        .obligations
        .iter()
        .map(|item| (item.obligation_id.as_str(), item.kind.as_str()))
        .collect::<BTreeSet<_>>();
    if !required.is_subset(&observed) {
        diagnostics.push(diagnostic(
            DiagnosticClass::MissingObligation,
            "obligations",
            "one or more required obligations are missing or have another kind",
        ));
    }
    if !observed.is_subset(&required) {
        diagnostics.push(diagnostic(
            DiagnosticClass::UnexpectedObligation,
            "obligations",
            "receipt contains obligations outside the reviewed contract",
        ));
    }
    let mut ids = BTreeSet::new();
    for obligation in &receipt.obligations {
        if !ids.insert(&obligation.obligation_id) {
            diagnostics.push(diagnostic(
                DiagnosticClass::UnexpectedObligation,
                "obligations",
                "receipt obligation identity is duplicated",
            ));
        }
        if obligation.theorem.trim().is_empty()
            || obligation.generated_artifact_role.trim().is_empty()
            || obligation.proof_twin_role.trim().is_empty()
        {
            diagnostics.push(diagnostic(
                DiagnosticClass::MissingObligation,
                "obligations",
                "obligation theorem or artifact role is empty",
            ));
        }
        match obligation.status {
            ObligationStatus::Checked => {}
            ObligationStatus::Unchecked => diagnostics.push(diagnostic(
                DiagnosticClass::UnresolvedProofHole,
                "obligations.status",
                "an obligation remains unproved",
            )),
            ObligationStatus::Missing => diagnostics.push(diagnostic(
                DiagnosticClass::MissingObligation,
                "obligations.status",
                "an obligation was not emitted",
            )),
            ObligationStatus::Rejected => diagnostics.push(diagnostic(
                DiagnosticClass::ProofRejected,
                "obligations.status",
                "the checker rejected an obligation",
            )),
        }
    }
}

fn validate_artifacts(receipt: &VermilionReceipt, diagnostics: &mut Vec<Diagnostic>) {
    let mut roles = BTreeSet::new();
    for artifact in &receipt.generated_artifacts {
        if artifact.role.trim().is_empty()
            || unsafe_path(&artifact.path)
            || !is_b3(&artifact.blake3)
            || !roles.insert(artifact.role.as_str())
        {
            diagnostics.push(diagnostic(
                DiagnosticClass::StaleGeneratedOutput,
                "generated_artifacts",
                "generated artifact role, path, identity, or uniqueness is invalid",
            ));
        }
    }
    let mut expected = BTreeSet::from([GENERATED_LEAN_ROLE, OBLIGATION_MANIFEST_ROLE]);
    for obligation in &receipt.obligations {
        expected.insert(obligation.generated_artifact_role.as_str());
        expected.insert(obligation.proof_twin_role.as_str());
    }
    if roles != expected {
        diagnostics.push(diagnostic(
            DiagnosticClass::StaleGeneratedOutput,
            "generated_artifacts",
            "generated artifact roles do not cover exactly the generated sources and proof twins",
        ));
    }
    for obligation in &receipt.obligations {
        if artifact_identity(receipt, &obligation.generated_artifact_role)
            != Some(obligation.generated_artifact_blake3.as_str())
        {
            diagnostics.push(diagnostic(
                DiagnosticClass::StaleGeneratedOutput,
                "obligations.generated_artifact_blake3",
                "obligation is bound to stale generated output",
            ));
        }
        if artifact_identity(receipt, &obligation.proof_twin_role) != Some(obligation.proof_twin_blake3.as_str()) {
            diagnostics.push(diagnostic(
                DiagnosticClass::StaleProofTwin,
                "obligations.proof_twin_blake3",
                "obligation is bound to a stale proof twin",
            ));
        }
    }
}

fn artifact_identity<'a>(receipt: &'a VermilionReceipt, role: &str) -> Option<&'a str> {
    receipt
        .generated_artifacts
        .iter()
        .find(|artifact| artifact.role == role)
        .map(|artifact| artifact.blake3.as_str())
}

fn validate_bridges(
    receipt: &VermilionReceipt,
    route: Option<&super::RouteObligations>,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(route) = route else {
        return;
    };
    let Some(bridge) = receipt.refinement_bridges.iter().find(|bridge| bridge.bridge_id == route.refinement_bridge_id)
    else {
        diagnostics.push(diagnostic(
            DiagnosticClass::MissingObligation,
            "refinement_bridges",
            "the declared refinement bridge is not accounted for",
        ));
        return;
    };
    if bridge.bridge_id.trim().is_empty() || bridge.statement.trim().is_empty() {
        diagnostics.push(diagnostic(
            DiagnosticClass::MissingObligation,
            "refinement_bridges",
            "refinement-bridge identity or statement is empty",
        ));
    }
    if receipt.refinement_bridges.len() > 1 {
        diagnostics.push(diagnostic(
            DiagnosticClass::UnexpectedObligation,
            "refinement_bridges",
            "receipt declares bridges outside the reviewed Vermilion route",
        ));
    }
}

fn validate_checker_observation(receipt: &VermilionReceipt, diagnostics: &mut Vec<Diagnostic>) {
    let checker = &receipt.checker;
    if checker.checker_id != "vrml_check" {
        diagnostics.push(diagnostic(
            DiagnosticClass::WrongProvider,
            "checker.checker_id",
            "receipt checker is not the reviewed `vrml_check`",
        ));
    }
    match checker.status {
        CheckerStatus::Passed => {}
        CheckerStatus::Failed => diagnostics.push(diagnostic(
            DiagnosticClass::ProofRejected,
            "checker.status",
            "the checker reported a failed obligation",
        )),
        CheckerStatus::Interrupted => diagnostics.push(diagnostic(
            DiagnosticClass::InterruptedChecking,
            "checker.status",
            "the checker was interrupted before completing",
        )),
        CheckerStatus::Unavailable => diagnostics.push(diagnostic(
            DiagnosticClass::MissingChecker,
            "checker.status",
            "the checker could not start",
        )),
    }
    if checker.output_empty {
        diagnostics.push(diagnostic(
            DiagnosticClass::EmptyOutput,
            "checker.output_empty",
            "the checker emitted no machine-readable result",
        ));
    }
    if !checker.complete
        || checker.exit_code != Some(0)
        || checker.required_obligations != checker.checked_obligations
        || checker.required_obligations != u64::try_from(receipt.obligations.len()).unwrap_or(u64::MAX)
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::InterruptedChecking,
            "checker",
            "the checker did not judge every required obligation with exit code 0",
        ));
    }
    if checker.status == CheckerStatus::Unavailable && checker.diagnostic_identity.is_none() {
        diagnostics.push(diagnostic(
            DiagnosticClass::MissingChecker,
            "checker.diagnostic_identity",
            "a missing checker must carry a bounded diagnostic identity",
        ));
    }
}

fn validate_inventory(receipt: &VermilionReceipt, profile: &ProviderProfile, diagnostics: &mut Vec<Diagnostic>) {
    let mut observed = BTreeSet::new();
    for theorem in &receipt.proof_inventory.theorems {
        observed.insert((theorem.obligation_id.as_str(), theorem.theorem.as_str()));
    }
    let expected = receipt
        .obligations
        .iter()
        .filter(|obligation| obligation.status == ObligationStatus::Checked)
        .map(|obligation| (obligation.obligation_id.as_str(), obligation.theorem.as_str()))
        .collect::<BTreeSet<_>>();
    if observed != expected {
        diagnostics.push(diagnostic(
            DiagnosticClass::MissingObligation,
            "proof_inventory.theorems",
            "theorem inventory does not cover each checked obligation exactly",
        ));
    }
    let mut assumptions = BTreeSet::new();
    for obligation in &receipt.obligations {
        for assumption in &obligation.assumptions {
            assumptions.insert(assumption.as_str());
        }
        for hole in &profile.assumption_policy.proof_holes {
            if obligation.assumptions.iter().any(|assumption| assumption.contains(hole.as_str()))
                || obligation.theorem.contains(hole.as_str())
            {
                diagnostics.push(diagnostic(
                    DiagnosticClass::UnresolvedProofHole,
                    "obligations",
                    "an obligation still depends on a proof hole",
                ));
            }
        }
    }
    let declared = receipt.proof_inventory.axioms.iter().map(String::as_str).collect::<BTreeSet<_>>();
    if declared != assumptions {
        diagnostics.push(diagnostic(
            DiagnosticClass::UnexpectedAxiom,
            "proof_inventory.axioms",
            "axiom inventory does not match the obligations' assumptions",
        ));
    }
    for axiom in &declared {
        let is_hole = profile
            .assumption_policy
            .proof_holes
            .iter()
            .any(|hole| axiom.to_ascii_lowercase().contains(&hole.to_ascii_lowercase()));
        if !is_hole && !profile.assumption_policy.allowed_axioms.iter().any(|allowed| allowed == axiom) {
            diagnostics.push(diagnostic(
                DiagnosticClass::UnexpectedAxiom,
                "proof_inventory.axioms",
                "an axiom outside the reviewed allowance is present",
            ));
        }
    }
    if declared.iter().any(|axiom| axiom.to_ascii_lowercase().contains("sorry")) {
        diagnostics.push(diagnostic(
            DiagnosticClass::UnresolvedProofHole,
            "proof_inventory.axioms",
            "a sorry axiom remains",
        ));
    }
}

fn validate_authority(receipt: &VermilionReceipt, diagnostics: &mut Vec<Diagnostic>) {
    if receipt.origin == ReceiptOrigin::SyntheticContractFixture
        && receipt.evidence_disposition != EvidenceDisposition::RecordedOnly
    {
        diagnostics.push(diagnostic(
            DiagnosticClass::EvidenceOverclaim,
            "evidence_disposition",
            "a synthetic contract fixture cannot request property evidence",
        ));
    }
    if let Some(comparison) = &receipt.comparison {
        if comparison.evidence_merge {
            diagnostics.push(diagnostic(
                DiagnosticClass::EvidenceOverclaim,
                "comparison.evidence_merge",
                "cross-provider agreement cannot merge evidence authority",
            ));
        }
        if comparison.participants.is_empty()
            || comparison.participants.iter().map(String::as_str).collect::<BTreeSet<_>>().len()
                != comparison.participants.len()
            || !comparison
                .open_obligations
                .iter()
                .any(|obligation| obligation == "pilot.increment.refinement-to-spec")
        {
            diagnostics.push(diagnostic(
                DiagnosticClass::MissingObligation,
                "comparison",
                "comparison metadata omits participants or open refinement obligations",
            ));
        }
    }
}

fn validate_non_claims(observed: &[String], required: &[&str], field: &str, diagnostics: &mut Vec<Diagnostic>) {
    for required in required {
        if !observed.iter().any(|claim| claim == required) {
            diagnostics.push(diagnostic(
                DiagnosticClass::NonClaimDrift,
                field,
                &format!("claim boundary `{required}` is missing"),
            ));
        }
    }
}

pub(super) fn open_obligations(receipt: &VermilionReceipt, contract: &SemanticContract) -> Vec<String> {
    let mut open = contract
        .routes
        .iter()
        .filter(|route| route.backend == VERMILION_ROUTE)
        .filter_map(|route| {
            let bridge =
                receipt.refinement_bridges.iter().find(|bridge| bridge.bridge_id == route.refinement_bridge_id)?;
            (bridge.status == BridgeStatus::Open).then(|| bridge.bridge_id.clone())
        })
        .collect::<Vec<_>>();
    if receipt
        .refinement_bridges
        .iter()
        .all(|bridge| bridge.bridge_id != "pilot.increment.refinement-to-spec")
    {
        open.push("pilot.increment.refinement-to-spec".to_owned());
    }
    open
}

fn is_git_revision(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn unsafe_path(value: &str) -> bool {
    value.is_empty()
        || Path::new(value).is_absolute()
        || value.split('/').any(|component| component.is_empty() || component == "..")
}
