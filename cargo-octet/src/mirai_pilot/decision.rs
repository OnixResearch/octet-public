use super::CORPUS_IDENTITY;
use super::DECISION_SCHEMA;
use super::EXPECTED_BLOCKER;
use super::EXPECTED_CHARON_PARENT;
use super::EXPECTED_EFFECT_PARENT;
use super::POLICY_IDENTITY;
use super::comparison;
use super::model::AnalyzerFact;
use super::model::DecisionReceipt;
use super::model::Measurement;
use super::model::PilotProfile;

const DECISION_BLOCKED: &str = "blocked";
const AVAILABILITY_BLOCKED: &str = "blocked-output-contract";
const CLAIM_PINNED_SOURCE: &str = "pinned-source-provider-recorded";
const CLAIM_BLOCKED_DECISION: &str = "blocked-output-contract-recorded";
const FORBIDDEN_CLAIMS: [&str; 5] = [
    "soundness",
    "complete Rust support",
    "whole-program correctness",
    "semantic equivalence",
    "release eligibility",
];

pub fn blocked_receipt(profile: &PilotProfile) -> Result<DecisionReceipt, String> {
    let measurements = profile
        .promotion_criteria
        .iter()
        .map(|criterion| Measurement {
            criterion_id: criterion.criterion_id.clone(),
            observed: if criterion.criterion_id == "stable-machine-output-contract" {
                Some(0)
            } else {
                None
            },
            passed: false,
            availability: AVAILABILITY_BLOCKED.to_owned(),
        })
        .collect::<Vec<_>>();
    let mut failed_criteria = measurements.iter().map(|item| item.criterion_id.clone()).collect::<Vec<_>>();
    failed_criteria.sort();
    let comparisons = profile
        .corpus
        .iter()
        .map(|fixture| {
            let charon = AnalyzerFact {
                parent_identity: EXPECTED_CHARON_PARENT.to_owned(),
                source_identity: fixture.source_identity.clone(),
                value: "not-measured".to_owned(),
            };
            let source_token = AnalyzerFact {
                parent_identity: EXPECTED_EFFECT_PARENT.to_owned(),
                source_identity: fixture.source_identity.clone(),
                value: "not-measured".to_owned(),
            };
            comparison::compare(fixture, None, Some(&charon), Some(&source_token))
        })
        .collect::<Vec<_>>();
    let mut receipt = DecisionReceipt {
        schema_version: DECISION_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        profile_identity: POLICY_IDENTITY.to_owned(),
        corpus_identity: CORPUS_IDENTITY.to_owned(),
        provider_revision: profile.provider.revision.clone(),
        provider_source_hash: profile.provider.source_hash.clone(),
        rust_toolchain: profile.provider.rust_toolchain.clone(),
        target: profile.provider.target.clone(),
        derivation_attr: profile.provider.derivation_attr.clone(),
        closure_components: profile.provider.closure_components.clone(),
        command_argv: profile.provider.command_argv.clone(),
        environment: profile.provider.environment.clone(),
        decision: DECISION_BLOCKED.to_owned(),
        blocker_code: EXPECTED_BLOCKER.to_owned(),
        failed_criteria,
        measurements,
        comparisons,
        execution_attempted: false,
        reproducible_decision: true,
        claims: vec![CLAIM_PINNED_SOURCE.to_owned(), CLAIM_BLOCKED_DECISION.to_owned()],
        evidence_tier: profile.evidence_tier.clone(),
        verification_role: profile.verification_role.clone(),
        non_claims: profile.non_claims.clone(),
        receipt_identity: String::new(),
    };
    receipt.receipt_identity = identity(&receipt)?;
    Ok(receipt)
}

pub fn validate_receipt(receipt: &DecisionReceipt, profile: &PilotProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != DECISION_SCHEMA
        || receipt.profile_id != profile.profile_id
        || receipt.profile_identity != POLICY_IDENTITY
        || receipt.corpus_identity != CORPUS_IDENTITY
        || receipt.provider_revision != profile.provider.revision
        || receipt.provider_source_hash != profile.provider.source_hash
        || receipt.rust_toolchain != profile.provider.rust_toolchain
        || receipt.target != profile.provider.target
        || receipt.derivation_attr != profile.provider.derivation_attr
    {
        diagnostics.push("MIRAI pilot decision provider, policy, or corpus identity drifted".to_owned());
    }
    if receipt.closure_components != profile.provider.closure_components
        || receipt.command_argv != profile.provider.command_argv
        || receipt.environment != profile.provider.environment
        || receipt.decision != DECISION_BLOCKED
        || receipt.blocker_code != EXPECTED_BLOCKER
        || receipt.execution_attempted
        || !receipt.reproducible_decision
    {
        diagnostics.push("MIRAI pilot blocked decision or execution boundary drifted".to_owned());
    }
    validate_measurements(receipt, profile, &mut diagnostics);
    validate_comparisons(receipt, profile, &mut diagnostics);
    if receipt.claims != [CLAIM_PINNED_SOURCE, CLAIM_BLOCKED_DECISION]
        || receipt
            .claims
            .iter()
            .any(|claim| FORBIDDEN_CLAIMS.iter().any(|forbidden| claim.contains(forbidden)))
        || receipt.evidence_tier != profile.evidence_tier
        || receipt.verification_role != profile.verification_role
        || receipt.non_claims != profile.non_claims
    {
        diagnostics.push("MIRAI pilot receipt overclaims or changes its evidence boundary".to_owned());
    }
    match identity(receipt) {
        Ok(expected) if receipt.receipt_identity == expected && super::is_b3_identity(&receipt.receipt_identity) => {}
        Ok(_) => diagnostics.push("MIRAI pilot decision identity is stale or malformed".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_measurements(receipt: &DecisionReceipt, profile: &PilotProfile, diagnostics: &mut Vec<String>) {
    let mut expected = profile
        .promotion_criteria
        .iter()
        .map(|criterion| criterion.criterion_id.clone())
        .collect::<Vec<_>>();
    expected.sort();
    if receipt.failed_criteria != expected
        || receipt.measurements.len() != profile.promotion_criteria.len()
        || receipt.measurements.iter().any(|measurement| {
            measurement.passed
                || measurement.availability != AVAILABILITY_BLOCKED
                || !expected.contains(&measurement.criterion_id)
        })
    {
        diagnostics.push("MIRAI pilot failed-criterion accounting is incomplete".to_owned());
    }
}

fn validate_comparisons(receipt: &DecisionReceipt, profile: &PilotProfile, diagnostics: &mut Vec<String>) {
    let expected_parents = [EXPECTED_EFFECT_PARENT.to_owned(), EXPECTED_CHARON_PARENT.to_owned()];
    if receipt.comparisons.len() != profile.corpus.len()
        || receipt.comparisons.iter().zip(&profile.corpus).any(|(row, fixture)| {
            row.fixture_id != fixture.fixture_id
                || row.source_identity != fixture.source_identity
                || row.property_class != fixture.property_class
                || row.outcome != "unknown"
                || row.parent_identities != expected_parents
        })
    {
        diagnostics.push("MIRAI pilot comparison rows lost source or parent authority".to_owned());
    }
}

pub fn receipt_bytes(receipt: &DecisionReceipt) -> Result<Vec<u8>, String> {
    super::pretty_bytes(receipt, "MIRAI pilot decision receipt")
}

fn identity(receipt: &DecisionReceipt) -> Result<String, String> {
    let mut projection = receipt.clone();
    projection.receipt_identity.clear();
    receipt_bytes(&projection).map(|bytes| super::prefixed_identity(&bytes))
}
