//! Bounded storage-fault assurance validation.
//!
//! The profile links exact saved producer and Cairn receipt identities. This
//! module validates supplied bytes only: it never starts, locates, or waits for
//! a producer executable, and it never executes a fault campaign.

use serde::Deserialize;
use serde::Serialize;

/// Schema version of the storage-fault assurance profile.
pub const STORAGE_FAULT_PROFILE_SCHEMA_VERSION: &str = "octet-storage-fault-assurance/v1";
/// Schema version of a saved producer receipt.
pub const STORAGE_FAULT_PRODUCER_SCHEMA_VERSION: &str = "octet-storage-fault-producer-receipt/v1";
/// Schema version of a saved Cairn receipt.
pub const STORAGE_FAULT_CAIRN_SCHEMA_VERSION: &str = "octet-storage-fault-cairn-receipt/v1";
/// The only accepted receipt claim scope.
pub const BOUNDED_CLAIM_SCOPE: &str = "bounded-observation";

/// One required evidence case.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequiredCase {
    /// Stable case identifier.
    pub case_id: String,
    /// Positive or negative case class.
    pub kind: CaseKind,
}

/// Positive or negative case class.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaseKind {
    /// A case that must succeed.
    Positive,
    /// A case that records an expected failure mode.
    Negative,
}

/// Exact saved receipt identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ReceiptBinding {
    /// Workspace-relative or absolute receipt path.
    pub path: String,
    /// Exact BLAKE3 identity of the saved receipt bytes.
    pub blake3: String,
}

/// The declared storage-fault assurance profile.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StorageFaultProfile {
    /// Declared schema version.
    pub schema_version: String,
    /// Stable profile identifier.
    pub profile_id: String,
    /// Exact BLAKE3 identity of the checked source set.
    pub source_blake3: String,
    /// Exact BLAKE3 identity of the checked candidate.
    pub candidate_blake3: String,
    /// Cases the saved evidence must cover.
    pub required_cases: Vec<RequiredCase>,
    /// Exact saved producer receipt identity.
    pub producer_receipt: ReceiptBinding,
    /// Exact saved Cairn receipt identity.
    pub cairn_receipt: ReceiptBinding,
    /// Claim non-claims preserved in every passing result.
    pub non_claims: Vec<String>,
}

/// One observed producer case.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ObservedCase {
    /// Stable case identifier.
    pub case_id: String,
    /// Recorded outcome.
    pub outcome: CaseOutcome,
}

/// Producer case outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CaseOutcome {
    /// The case passed.
    Passed,
    /// The case recorded a failure.
    Failed,
    /// The case did not run.
    Skipped,
}

/// Saved producer receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProducerReceipt {
    /// Declared schema version.
    pub schema_version: String,
    /// Producer name that authored the saved receipt.
    pub producer: String,
    /// Exact BLAKE3 identity of the checked source set.
    pub source_blake3: String,
    /// Exact BLAKE3 identity of the checked candidate.
    pub candidate_blake3: String,
    /// Bounded claim scope.
    pub claim_scope: String,
    /// Observed cases.
    pub cases: Vec<ObservedCase>,
    /// Claim non-claims.
    pub non_claims: Vec<String>,
}

/// Saved Cairn receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CairnReceipt {
    /// Declared schema version.
    pub schema_version: String,
    /// Exact BLAKE3 identity of the checked candidate.
    pub candidate_blake3: String,
    /// Exact identity of the producer receipt this receipt links.
    pub producer_receipt_blake3: String,
    /// Bounded claim scope.
    pub claim_scope: String,
    /// Case identifiers covered by the Cairn evidence.
    pub cases: Vec<String>,
    /// Claim non-claims.
    pub non_claims: Vec<String>,
}

/// Bounded passing result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct StorageFaultAssurance {
    /// Profile identifier that passed.
    pub profile_id: String,
    /// Exact source identity bound by the profile.
    pub source_blake3: String,
    /// Exact candidate identity bound by the profile.
    pub candidate_blake3: String,
    /// Computed identity of the saved producer receipt.
    pub producer_receipt_blake3: String,
    /// Computed identity of the saved Cairn receipt.
    pub cairn_receipt_blake3: String,
    /// Cases covered by the saved evidence.
    pub verified_cases: Vec<String>,
    /// Preserved claim non-claims.
    pub non_claims: Vec<String>,
}

/// Validates the profile and both saved receipts without executing a producer.
///
/// # Errors
///
/// Returns deterministic diagnostics for malformed JSON, unknown fields,
/// missing required cases, stale source or candidate identities, receipt
/// identity drift, producer failures, and overclaiming receipts.
pub fn validate_storage_fault_assurance(
    profile_text: &str,
    producer_text: &str,
    cairn_text: &str,
) -> Result<StorageFaultAssurance, Vec<String>> {
    let profile = serde_json::from_str::<StorageFaultProfile>(profile_text)
        .map_err(|error| vec![format!("parse storage-fault profile: {error}")])?;
    let producer = serde_json::from_str::<ProducerReceipt>(producer_text)
        .map_err(|error| vec![format!("parse storage-fault producer receipt: {error}")])?;
    let cairn = serde_json::from_str::<CairnReceipt>(cairn_text)
        .map_err(|error| vec![format!("parse storage-fault Cairn receipt: {error}")])?;

    let producer_blake3 = blake3_hex(producer_text.as_bytes());
    let cairn_blake3 = blake3_hex(cairn_text.as_bytes());
    let mut diagnostics = profile_diagnostics(&profile);
    diagnostics.extend(producer_diagnostics(&profile, &producer));
    diagnostics.extend(receipt_identity_diagnostics(&profile, &producer_blake3, &cairn_blake3));
    diagnostics.extend(cairn_diagnostics(&profile, &cairn, &producer_blake3));
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }

    Ok(StorageFaultAssurance {
        profile_id: profile.profile_id,
        source_blake3: profile.source_blake3,
        candidate_blake3: profile.candidate_blake3,
        producer_receipt_blake3: producer_blake3,
        cairn_receipt_blake3: cairn_blake3,
        verified_cases: profile.required_cases.into_iter().map(|case| case.case_id).collect(),
        non_claims: profile.non_claims,
    })
}

fn profile_diagnostics(profile: &StorageFaultProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if profile.schema_version != STORAGE_FAULT_PROFILE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "profile schema_version must be `{STORAGE_FAULT_PROFILE_SCHEMA_VERSION}`, got `{}`",
            profile.schema_version
        ));
    }
    for (field, value) in [
        ("profile_id", &profile.profile_id),
        ("source_blake3", &profile.source_blake3),
        ("candidate_blake3", &profile.candidate_blake3),
        ("producer_receipt.path", &profile.producer_receipt.path),
        ("producer_receipt.blake3", &profile.producer_receipt.blake3),
        ("cairn_receipt.path", &profile.cairn_receipt.path),
        ("cairn_receipt.blake3", &profile.cairn_receipt.blake3),
    ] {
        if value.trim().is_empty() {
            diagnostics.push(format!("profile {field} must not be empty"));
        }
    }
    if profile.required_cases.is_empty() {
        diagnostics.push("profile required_cases must not be empty".to_owned());
    }
    if profile.non_claims.is_empty() {
        diagnostics.push("profile non_claims must not be empty".to_owned());
    }
    let mut seen = std::collections::BTreeSet::new();
    for case in &profile.required_cases {
        if case.case_id.trim().is_empty() {
            diagnostics.push("profile required case_id must not be empty".to_owned());
        } else if !seen.insert(case.case_id.as_str()) {
            diagnostics.push(format!("profile has duplicate required case `{}`", case.case_id));
        }
    }
    if !profile.required_cases.iter().any(|case| case.kind == CaseKind::Negative) {
        diagnostics.push("profile must require at least one negative case".to_owned());
    }
    diagnostics
}

fn producer_diagnostics(profile: &StorageFaultProfile, producer: &ProducerReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if producer.schema_version != STORAGE_FAULT_PRODUCER_SCHEMA_VERSION {
        diagnostics.push(format!(
            "producer schema_version must be `{STORAGE_FAULT_PRODUCER_SCHEMA_VERSION}`, got `{}`",
            producer.schema_version
        ));
    }
    if producer.producer.trim().is_empty() {
        diagnostics.push("producer receipt must name its producer".to_owned());
    }
    diagnostics.extend(claim_scope_diagnostics("producer", &producer.claim_scope, &producer.non_claims));
    diagnostics.extend(stale_identity_diagnostics(
        "producer",
        &producer.source_blake3,
        &producer.candidate_blake3,
        &profile.source_blake3,
        &profile.candidate_blake3,
    ));
    for required in &profile.required_cases {
        match producer.cases.iter().find(|case| case.case_id == required.case_id) {
            None => diagnostics.push(format!("producer receipt is missing required case `{}`", required.case_id)),
            Some(case) if case.outcome != CaseOutcome::Passed => diagnostics.push(format!(
                "producer case `{}` recorded outcome `{:?}`; required evidence must pass",
                required.case_id, case.outcome
            )),
            Some(_case) => {}
        }
    }
    diagnostics
}

fn cairn_diagnostics(profile: &StorageFaultProfile, cairn: &CairnReceipt, producer_blake3: &str) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if cairn.schema_version != STORAGE_FAULT_CAIRN_SCHEMA_VERSION {
        diagnostics.push(format!(
            "Cairn receipt schema_version must be `{STORAGE_FAULT_CAIRN_SCHEMA_VERSION}`, got `{}`",
            cairn.schema_version
        ));
    }
    diagnostics.extend(claim_scope_diagnostics("Cairn", &cairn.claim_scope, &cairn.non_claims));
    if !cairn.candidate_blake3.eq_ignore_ascii_case(&profile.candidate_blake3) {
        diagnostics.push("Cairn receipt candidate identity is stale for this profile".to_owned());
    }
    if !cairn.producer_receipt_blake3.eq_ignore_ascii_case(producer_blake3) {
        diagnostics.push("Cairn receipt does not link the supplied producer receipt identity".to_owned());
    }
    for required in &profile.required_cases {
        if !cairn.cases.iter().any(|case| case == &required.case_id) {
            diagnostics.push(format!("Cairn receipt is missing required case `{}`", required.case_id));
        }
    }
    diagnostics
}

fn receipt_identity_diagnostics(
    profile: &StorageFaultProfile,
    producer_blake3: &str,
    cairn_blake3: &str,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if !profile.producer_receipt.blake3.eq_ignore_ascii_case(producer_blake3) {
        diagnostics.push("saved producer receipt identity is stale for this profile".to_owned());
    }
    if !profile.cairn_receipt.blake3.eq_ignore_ascii_case(cairn_blake3) {
        diagnostics.push("saved Cairn receipt identity is stale for this profile".to_owned());
    }
    diagnostics
}

fn stale_identity_diagnostics(
    surface: &str,
    source_blake3: &str,
    candidate_blake3: &str,
    expected_source: &str,
    expected_candidate: &str,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if !source_blake3.eq_ignore_ascii_case(expected_source) {
        diagnostics.push(format!("{surface} receipt source identity is stale for this profile"));
    }
    if !candidate_blake3.eq_ignore_ascii_case(expected_candidate) {
        diagnostics.push(format!("{surface} receipt candidate identity is stale for this profile"));
    }
    diagnostics
}

fn claim_scope_diagnostics(surface: &str, claim_scope: &str, non_claims: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if claim_scope != BOUNDED_CLAIM_SCOPE {
        diagnostics.push(format!(
            "{surface} receipt overclaims: claim_scope must be `{BOUNDED_CLAIM_SCOPE}`, got `{claim_scope}`"
        ));
    }
    if non_claims.is_empty() {
        diagnostics.push(format!("{surface} receipt must declare non_claims"));
    }
    diagnostics
}

fn blake3_hex(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::BOUNDED_CLAIM_SCOPE;
    use super::validate_storage_fault_assurance;

    const CURRENT_PROFILE: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/current/profile.json"
    );
    const CURRENT_PRODUCER: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/current/producer.json"
    );
    const CURRENT_CAIRN: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/current/cairn.json"
    );
    const STALE_PROFILE: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/stale/profile.json"
    );
    const STALE_PRODUCER: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/stale/producer.json"
    );
    const STALE_CAIRN: &str =
        include_str!("../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/stale/cairn.json");
    const MISSING_CASE_PROFILE: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/missing_case/profile.json"
    );
    const MISSING_CASE_PRODUCER: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/missing_case/producer.json"
    );
    const MISSING_CASE_CAIRN: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/missing_case/cairn.json"
    );
    const PRODUCER_FAILURE_PROFILE: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/producer_failure/profile.json"
    );
    const PRODUCER_FAILURE_PRODUCER: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/producer_failure/producer.json"
    );
    const PRODUCER_FAILURE_CAIRN: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/producer_failure/cairn.json"
    );
    const OVERCLAIM_PROFILE: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/overclaim/profile.json"
    );
    const OVERCLAIM_PRODUCER: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/overclaim/producer.json"
    );
    const OVERCLAIM_CAIRN: &str = include_str!(
        "../../../standards/policies/assurance-profiles/storage-fault/fixtures/evidence/overclaim/cairn.json"
    );

    fn rejected(profile: &str, producer: &str, cairn: &str, needle: &str) {
        let diagnostics =
            validate_storage_fault_assurance(profile, producer, cairn).expect_err("evidence must be rejected");
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains(needle)),
            "expected `{needle}` in {diagnostics:#?}"
        );
    }

    #[test]
    fn current_evidence_passes_without_a_producer() {
        // The validator takes saved bytes only; no producer path, environment
        // lookup, or process launch exists in this code path.
        let result = validate_storage_fault_assurance(CURRENT_PROFILE, CURRENT_PRODUCER, CURRENT_CAIRN)
            .expect("current evidence passes");
        assert_eq!(result.verified_cases.len(), 3);
        assert_eq!(result.producer_receipt_blake3.len(), 64);
        assert_eq!(result.cairn_receipt_blake3.len(), 64);
        assert!(!result.non_claims.is_empty());
    }

    #[test]
    fn stale_missing_failed_and_overclaiming_evidence_is_rejected() {
        rejected(STALE_PROFILE, STALE_PRODUCER, STALE_CAIRN, "source identity is stale");
        rejected(
            MISSING_CASE_PROFILE,
            MISSING_CASE_PRODUCER,
            MISSING_CASE_CAIRN,
            "missing required case `flush-error`",
        );
        rejected(
            PRODUCER_FAILURE_PROFILE,
            PRODUCER_FAILURE_PRODUCER,
            PRODUCER_FAILURE_CAIRN,
            "recorded outcome `Failed`",
        );
        rejected(OVERCLAIM_PROFILE, OVERCLAIM_PRODUCER, OVERCLAIM_CAIRN, "overclaims");
    }

    #[test]
    fn receipt_identity_drift_and_unknown_fields_are_rejected() {
        let drifted = CURRENT_PROFILE.replace(
            &format!("\"blake3\": \"{}\"", first_receipt_identity(CURRENT_PROFILE)),
            &format!("\"blake3\": \"{}\"", "0".repeat(64)),
        );
        rejected(&drifted, CURRENT_PRODUCER, CURRENT_CAIRN, "saved producer receipt identity is stale");

        let unknown = CURRENT_PROFILE.replace("\"profile_id\"", "\"extra\": true,\n  \"profile_id\"");
        rejected(&unknown, CURRENT_PRODUCER, CURRENT_CAIRN, "unknown field");

        rejected("{", CURRENT_PRODUCER, CURRENT_CAIRN, "parse storage-fault profile");
        rejected(CURRENT_PROFILE, "{", CURRENT_CAIRN, "parse storage-fault producer receipt");
        rejected(CURRENT_PROFILE, CURRENT_PRODUCER, "{", "parse storage-fault Cairn receipt");
    }

    #[test]
    fn bounded_claim_scope_is_the_only_accepted_scope() {
        assert_eq!(BOUNDED_CLAIM_SCOPE, "bounded-observation");
        let producer = CURRENT_PRODUCER.replace(BOUNDED_CLAIM_SCOPE, "durable-persistence-proof");
        rejected(CURRENT_PROFILE, &producer, CURRENT_CAIRN, "overclaims");
    }

    fn first_receipt_identity(profile: &str) -> String {
        profile
            .lines()
            .find(|line| line.trim_start().starts_with("\"blake3\": \""))
            .and_then(|line| line.split('"').nth(3))
            .expect("profile binds a receipt identity")
            .to_owned()
    }
}
