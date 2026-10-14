use std::collections::BTreeMap;
use std::collections::BTreeSet;

use octet_standards::proof_artifact;
use serde::Deserialize;
use serde::Serialize;

pub const PROVENANCE_SCHEMA_VERSION: &str = "octet-provenance-record/v1";
pub const CHECKER_CARGO_OCTET: &str = "cargo-octet";
pub const CHECKER_OCTET_STANDARDS: &str = "octet-standards";
pub const CHECKER_OCTET_POLICY_VALIDATE: &str = "octet-policy-validate";
pub const CHECKER_OCTET_POLICY_EXPORT: &str = "octet-policy-export";
pub const CHECKER_OCTET_FINGERPRINT: &str = "octet-fingerprint";
pub const CHECKER_OCTET_TRANSCRIPT: &str = "octet-transcript";
pub const CHECKER_CARGO_DENY: &str = "cargo-deny";
pub const CHECKER_CLIPPY: &str = "clippy";
pub const CHECKER_CARGO_VET: &str = "cargo-vet";
pub const CHECKER_CARGO_SEMVER_CHECKS: &str = "cargo-semver-checks";
pub const CHECKER_CARGO_MUTANTS: &str = "cargo-mutants";
pub const CHECKER_CARGO_LLVM_COV: &str = "cargo-llvm-cov";
pub const CHECKER_CARGO_KANI: &str = "cargo-kani";
pub const CHECKER_HEGEL_RS: &str = "hegel-rs";
pub const CHECKER_OCTET_DYNAMIC_EVIDENCE: &str = "octet-dynamic-evidence";
pub const CHECKER_OCTET_RELEASE_SUPPLY_CHAIN: &str = "octet-release-supply-chain";
const FIELD_SCHEMA_VERSION: &str = "schema_version";
const FIELD_SOURCE_ID: &str = "source_id";
const FIELD_ARTIFACT_LOCATOR: &str = "artifact_locator";
const FIELD_VERIFIER_IDENTITY: &str = "verifier_identity";
const FIELD_PROOF_ARTIFACT_IDENTITY: &str = "proof_artifact_identity";
const FIELD_HUMAN_ACTION: &str = "human_action";
const UNKNOWN_CHECKER_PREFIX: &str = "unknown checker_id";
const HASH_MISMATCH_PREFIX: &str = "hash mismatch for artifact locator";
const SOURCE_RUNNER_PREFIX: &str = "octet.runner.";
const SOURCE_ARTIFACT_PREFIX: &str = "octet.artifact.";
const SOURCE_STANDARDS_PREFIX: &str = "octet.standards.";
const SOURCE_POLICY_PREFIX: &str = "octet.policy.";
const SOURCE_FINGERPRINT_PREFIX: &str = "octet.fingerprint.";
const SOURCE_TRANSCRIPT_PREFIX: &str = "octet.transcript.";
const SOURCE_TRANSFORM_PREFIX: &str = "octet.transform.";
const SOURCE_FUNCTION_ADDRESS_PREFIX: &str = "octet.function_address.";
const SOURCE_EXTERNAL_LINT_PREFIX: &str = "octet.external_lint.";
const SOURCE_DEPENDENCY_TRUST_PREFIX: &str = "octet.dependency_trust.";
const SOURCE_API_STABILITY_PREFIX: &str = "octet.api_stability.";
const SOURCE_MUTATION_TESTING_PREFIX: &str = "octet.mutation_testing.";
const SOURCE_COVERAGE_BASELINE_PREFIX: &str = "octet.coverage_baseline.";
const SOURCE_KANI_MODEL_CHECK_PREFIX: &str = "octet.kani_model_check.";
const SOURCE_HEGEL_PROPERTY_PREFIX: &str = "octet.dynamic_evidence.hegel-property";
const SOURCE_DYNAMIC_EVIDENCE_PREFIX: &str = "octet.dynamic_evidence.";
const SOURCE_RELEASE_SUPPLY_CHAIN_PREFIX: &str = "octet.release_supply_chain.";

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceTier {
    StaticLint,
    PolicyCheck,
    UiFixture,
    AutofixVerified,
    TranscriptAssertion,
    FormalProof,
    ManualReviewRequired,
}

impl EvidenceTier {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::StaticLint => "static_lint",
            Self::PolicyCheck => "policy_check",
            Self::UiFixture => "ui_fixture",
            Self::AutofixVerified => "autofix_verified",
            Self::TranscriptAssertion => "transcript_assertion",
            Self::FormalProof => "formal_proof",
            Self::ManualReviewRequired => "manual_review_required",
        }
    }
}

// r[impl octet.evidence_provenance.record]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProvenanceRecord {
    pub schema_version: String,
    pub source_id: String,
    pub checker_id: String,
    pub evidence_tier: EvidenceTier,
    pub artifact_locator: String,
    pub blake3: String,
    pub checked_scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_artifact_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_action: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_artifacts: Vec<String>,
}

pub struct ProvenanceValidationInput<'a> {
    pub known_checker_ids: &'a BTreeSet<String>,
    pub expected_artifact_hashes: &'a BTreeMap<String, String>,
}

#[derive(Clone, Copy)]
struct ProvenanceProducer {
    checker_id: &'static str,
    source_prefixes: &'static [&'static str],
    allowed_tiers: &'static [EvidenceTier],
    allows_formal_proof: bool,
}

#[must_use]
pub fn artifact_blake3_hex(bytes: &[u8]) -> String {
    proof_artifact::blake3_hex_bytes(bytes)
}

#[must_use]
pub fn known_provenance_checker_ids() -> BTreeSet<String> {
    provenance_producers().iter().map(|producer| producer.checker_id.to_owned()).collect()
}

// r[impl octet.evidence_provenance.registry]
// r[impl octet.evidence_provenance.cross_gate_coverage]
// r[impl octet.evidence_provenance.fail_closed]
#[must_use]
pub fn validate_provenance_chain(
    records: &[ProvenanceRecord],
    expected_artifact_hashes: &BTreeMap<String, String>,
) -> Vec<String> {
    let known_checker_ids = known_provenance_checker_ids();
    let input = ProvenanceValidationInput {
        known_checker_ids: &known_checker_ids,
        expected_artifact_hashes,
    };
    let mut diagnostics = Vec::new();
    for record in records {
        diagnostics.extend(validate_provenance_record(record, &input));
        diagnostics.extend(validate_producer_registry(record));
    }
    diagnostics.extend(validate_parent_artifact_chains(records));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

// r[impl octet.evidence_provenance.producer_registry]
fn provenance_producers() -> &'static [ProvenanceProducer] {
    &[
        ProvenanceProducer {
            checker_id: CHECKER_CARGO_OCTET,
            source_prefixes: &[
                SOURCE_RUNNER_PREFIX,
                SOURCE_ARTIFACT_PREFIX,
                SOURCE_TRANSFORM_PREFIX,
                SOURCE_FUNCTION_ADDRESS_PREFIX,
            ],
            allowed_tiers: &[
                EvidenceTier::StaticLint,
                EvidenceTier::PolicyCheck,
                EvidenceTier::TranscriptAssertion,
                EvidenceTier::ManualReviewRequired,
            ],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_OCTET_STANDARDS,
            source_prefixes: &[SOURCE_STANDARDS_PREFIX, SOURCE_POLICY_PREFIX],
            allowed_tiers: &[
                EvidenceTier::PolicyCheck,
                EvidenceTier::ManualReviewRequired,
                EvidenceTier::FormalProof,
            ],
            allows_formal_proof: true,
        },
        ProvenanceProducer {
            checker_id: CHECKER_OCTET_POLICY_VALIDATE,
            source_prefixes: &[SOURCE_POLICY_PREFIX],
            allowed_tiers: &[EvidenceTier::PolicyCheck],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_OCTET_POLICY_EXPORT,
            source_prefixes: &[SOURCE_POLICY_PREFIX],
            allowed_tiers: &[EvidenceTier::PolicyCheck],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_OCTET_FINGERPRINT,
            source_prefixes: &[SOURCE_FINGERPRINT_PREFIX],
            allowed_tiers: &[EvidenceTier::ManualReviewRequired],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_OCTET_TRANSCRIPT,
            source_prefixes: &[SOURCE_TRANSCRIPT_PREFIX],
            allowed_tiers: &[EvidenceTier::TranscriptAssertion],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_CARGO_DENY,
            source_prefixes: &[SOURCE_POLICY_PREFIX],
            allowed_tiers: &[EvidenceTier::PolicyCheck],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_CLIPPY,
            source_prefixes: &[SOURCE_EXTERNAL_LINT_PREFIX],
            allowed_tiers: &[EvidenceTier::StaticLint],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_CARGO_VET,
            source_prefixes: &[SOURCE_DEPENDENCY_TRUST_PREFIX],
            allowed_tiers: &[EvidenceTier::PolicyCheck],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_CARGO_SEMVER_CHECKS,
            source_prefixes: &[SOURCE_API_STABILITY_PREFIX],
            allowed_tiers: &[EvidenceTier::PolicyCheck],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_CARGO_MUTANTS,
            source_prefixes: &[SOURCE_MUTATION_TESTING_PREFIX],
            allowed_tiers: &[EvidenceTier::TranscriptAssertion],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_CARGO_LLVM_COV,
            source_prefixes: &[SOURCE_COVERAGE_BASELINE_PREFIX],
            allowed_tiers: &[EvidenceTier::TranscriptAssertion],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_CARGO_KANI,
            source_prefixes: &[SOURCE_KANI_MODEL_CHECK_PREFIX],
            allowed_tiers: &[EvidenceTier::TranscriptAssertion, EvidenceTier::ManualReviewRequired],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_HEGEL_RS,
            source_prefixes: &[SOURCE_HEGEL_PROPERTY_PREFIX],
            allowed_tiers: &[EvidenceTier::TranscriptAssertion],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_OCTET_DYNAMIC_EVIDENCE,
            source_prefixes: &[SOURCE_DYNAMIC_EVIDENCE_PREFIX],
            allowed_tiers: &[EvidenceTier::TranscriptAssertion],
            allows_formal_proof: false,
        },
        ProvenanceProducer {
            checker_id: CHECKER_OCTET_RELEASE_SUPPLY_CHAIN,
            source_prefixes: &[SOURCE_RELEASE_SUPPLY_CHAIN_PREFIX],
            allowed_tiers: &[EvidenceTier::PolicyCheck],
            allows_formal_proof: false,
        },
    ]
}

fn validate_producer_registry(record: &ProvenanceRecord) -> Vec<String> {
    let Some(producer) = provenance_producers().iter().find(|producer| producer.checker_id == record.checker_id) else {
        return Vec::new();
    };

    let mut diagnostics = Vec::new();
    if !producer.source_prefixes.iter().any(|prefix| record.source_id.starts_with(prefix)) {
        diagnostics.push(format!(
            "unsupported source_id family `{}` for checker_id `{}`",
            record.source_id, record.checker_id
        ));
    }
    if !producer.allowed_tiers.contains(&record.evidence_tier) {
        diagnostics.push(format!(
            "invalid evidence_tier `{}` for checker_id `{}`",
            record.evidence_tier.as_str(),
            record.checker_id
        ));
    }
    if record.evidence_tier == EvidenceTier::FormalProof && !producer.allows_formal_proof {
        diagnostics.push(format!(
            "formal_proof overclaim by checker_id `{}` for artifact `{}`",
            record.checker_id, record.artifact_locator
        ));
    }
    diagnostics
}

// r[impl octet.evidence_provenance.parent_chain]
fn validate_parent_artifact_chains(records: &[ProvenanceRecord]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut graph = BTreeMap::<String, BTreeSet<String>>::new();
    for record in records {
        graph.entry(record.artifact_locator.clone()).or_default();
    }
    for record in records {
        for parent in &record.parent_artifacts {
            if !graph.contains_key(parent) {
                diagnostics
                    .push(format!("missing parent artifact `{parent}` referenced by `{}`", record.artifact_locator));
                continue;
            }
            graph.entry(record.artifact_locator.clone()).or_default().insert(parent.clone());
        }
    }
    diagnostics.extend(parent_cycle_diagnostics(graph));
    diagnostics
}

fn parent_cycle_diagnostics(mut graph: BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    loop {
        let removable = graph
            .iter()
            .filter(|(_, parents)| parents.iter().all(|parent| !graph.contains_key(parent)))
            .map(|(artifact, _)| artifact.clone())
            .collect::<Vec<_>>();
        if removable.is_empty() {
            break;
        }
        for artifact in removable {
            graph.remove(&artifact);
        }
    }

    graph
        .keys()
        .next()
        .map(|artifact| vec![format!("parent artifact cycle includes `{artifact}`")])
        .unwrap_or_default()
}

#[must_use]
pub fn validate_provenance_record(record: &ProvenanceRecord, input: &ProvenanceValidationInput<'_>) -> Vec<String> {
    let mut diagnostics = Vec::new();

    if is_blank(&record.schema_version) {
        diagnostics.push(missing_field(FIELD_SCHEMA_VERSION));
    } else if record.schema_version != PROVENANCE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported provenance schema_version `{}`; expected `{}`",
            record.schema_version, PROVENANCE_SCHEMA_VERSION
        ));
    }

    if is_blank(&record.source_id) {
        diagnostics.push(missing_field(FIELD_SOURCE_ID));
    }

    if !input.known_checker_ids.contains(&record.checker_id) {
        diagnostics.push(format!("{UNKNOWN_CHECKER_PREFIX} `{}`", record.checker_id));
    }

    if is_blank(&record.artifact_locator) {
        diagnostics.push(missing_field(FIELD_ARTIFACT_LOCATOR));
    } else {
        match input.expected_artifact_hashes.get(&record.artifact_locator) {
            Some(expected_hash) if expected_hash == &record.blake3 => {}
            Some(expected_hash) => diagnostics.push(format!(
                "{HASH_MISMATCH_PREFIX} `{}`; expected blake3 {}, got {}",
                record.artifact_locator, expected_hash, record.blake3
            )),
            None => diagnostics.push(format!(
                "{HASH_MISMATCH_PREFIX} `{}`; expected artifact was not provided",
                record.artifact_locator
            )),
        }
    }

    if record.evidence_tier == EvidenceTier::FormalProof {
        if option_is_blank(record.verifier_identity.as_deref()) {
            diagnostics.push(missing_field(FIELD_VERIFIER_IDENTITY));
        }
        if option_is_blank(record.proof_artifact_identity.as_deref()) {
            diagnostics.push(missing_field(FIELD_PROOF_ARTIFACT_IDENTITY));
        }
    }

    if record.evidence_tier == EvidenceTier::ManualReviewRequired && option_is_blank(record.human_action.as_deref()) {
        diagnostics.push(missing_field(FIELD_HUMAN_ACTION));
    }

    diagnostics
}

fn missing_field(field: &str) -> String {
    format!("missing required provenance field `{field}`")
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn option_is_blank(value: Option<&str>) -> bool {
    value.is_none_or(is_blank)
}

#[cfg(test)]
mod tests {
    use super::*;

    const KNOWN_CHECKER_ID: &str = "octet-standards";
    const SOURCE_ID: &str = "r[octet.evidence_provenance.record]";
    const ARTIFACT_LOCATOR: &str = "artifacts/octet/status.json";
    const ARTIFACT_BYTES: &[u8] = b"{\"status\":\"clean\"}\n";
    const HUMAN_ACTION: &str = "review the named suppression before release";
    const PROOF_VERIFIER: &str = "verus@2026-06-30";
    const PROOF_ARTIFACT: &str = "proofs/bounds.verus.receipt";

    #[test]
    fn all_evidence_tiers_round_trip_with_stable_discriminants() {
        let cases = [
            (EvidenceTier::StaticLint, "static_lint"),
            (EvidenceTier::PolicyCheck, "policy_check"),
            (EvidenceTier::UiFixture, "ui_fixture"),
            (EvidenceTier::AutofixVerified, "autofix_verified"),
            (EvidenceTier::TranscriptAssertion, "transcript_assertion"),
            (EvidenceTier::FormalProof, "formal_proof"),
            (EvidenceTier::ManualReviewRequired, "manual_review_required"),
        ];

        for (tier, discriminant) in cases {
            assert_eq!(tier.as_str(), discriminant);
            let rendered = serde_json::to_string(&tier).expect("evidence tier should serialize");
            assert_eq!(rendered, format!("\"{discriminant}\""));
            let parsed: EvidenceTier = serde_json::from_str(&rendered).expect("evidence tier should deserialize");
            assert_eq!(parsed, tier);
        }
    }

    #[test]
    fn unknown_evidence_tier_rejects() {
        let parsed = serde_json::from_str::<EvidenceTier>("\"runtime_magic\"");

        assert!(parsed.is_err());
    }

    // r[verify octet.evidence_provenance.record.links]
    #[test]
    fn valid_static_lint_provenance_record_passes() {
        let record = valid_record(EvidenceTier::StaticLint);

        let diagnostics = validate_with_defaults(&record);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn formal_proof_provenance_requires_proof_identity() {
        let record = valid_record(EvidenceTier::FormalProof);

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec![
            "missing required provenance field `verifier_identity`".to_owned(),
            "missing required provenance field `proof_artifact_identity`".to_owned(),
        ]);
    }

    #[test]
    fn formal_proof_provenance_accepts_scoped_proof_identity() {
        let record = ProvenanceRecord {
            verifier_identity: Some(PROOF_VERIFIER.to_owned()),
            proof_artifact_identity: Some(PROOF_ARTIFACT.to_owned()),
            ..valid_record(EvidenceTier::FormalProof)
        };

        let diagnostics = validate_with_defaults(&record);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn manual_review_required_provenance_requires_human_action() {
        let record = valid_record(EvidenceTier::ManualReviewRequired);

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec!["missing required provenance field `human_action`".to_owned()]);
    }

    #[test]
    fn manual_review_required_provenance_accepts_human_action() {
        let record = ProvenanceRecord {
            human_action: Some(HUMAN_ACTION.to_owned()),
            ..valid_record(EvidenceTier::ManualReviewRequired)
        };

        let diagnostics = validate_with_defaults(&record);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    // r[verify octet.evidence_provenance.fail_closed.missing_source]
    #[test]
    fn provenance_validation_rejects_missing_source_id() {
        let record = ProvenanceRecord {
            source_id: String::new(),
            ..valid_record(EvidenceTier::PolicyCheck)
        };

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec!["missing required provenance field `source_id`".to_owned()]);
    }

    // r[verify octet.evidence_provenance.producer_registry.unknown]
    // r[verify octet.evidence_provenance.fail_closed.unknown_checker]
    #[test]
    fn provenance_validation_rejects_unknown_checker_id() {
        let record = ProvenanceRecord {
            checker_id: "unknown-checker".to_owned(),
            ..valid_record(EvidenceTier::PolicyCheck)
        };

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec!["unknown checker_id `unknown-checker`".to_owned()]);
    }

    // r[verify octet.evidence_provenance.cross_gate_coverage.release_chain]
    // r[verify octet.evidence_provenance.registry.covered]
    // r[verify octet.evidence_provenance.producer_registry.known]
    // r[verify octet.evidence_provenance.parent_chain.links]
    #[test]
    fn cross_gate_provenance_chain_accepts_registered_producers_and_parents() {
        let records = release_chain_records();
        let expected_hashes = release_chain_hashes(&records);

        let diagnostics = validate_provenance_chain(&records, &expected_hashes);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    // r[verify octet.evidence_provenance.parent_chain.invalid]
    #[test]
    fn cross_gate_provenance_chain_rejects_unknowns_missing_parents_cycles_hash_mismatch_and_overclaims() {
        const RUNNER_RECORD_INDEX: usize = 0;
        const POLICY_RECORD_INDEX: usize = 1;
        const FINGERPRINT_RECORD_INDEX: usize = 2;
        const TRANSCRIPT_RECORD_INDEX: usize = 3;

        let mut records = release_chain_records();
        let mut expected_hashes = release_chain_hashes(&records);
        records[RUNNER_RECORD_INDEX].checker_id = "unknown-checker".to_owned();
        records[POLICY_RECORD_INDEX].parent_artifacts = vec!["missing-parent.json".to_owned()];
        records[FINGERPRINT_RECORD_INDEX].parent_artifacts = vec![TRANSCRIPT_ARTIFACT.to_owned()];
        records[TRANSCRIPT_RECORD_INDEX].parent_artifacts = vec![FINGERPRINT_ARTIFACT.to_owned()];
        records[TRANSCRIPT_RECORD_INDEX].evidence_tier = EvidenceTier::PolicyCheck;
        records[FINGERPRINT_RECORD_INDEX].evidence_tier = EvidenceTier::FormalProof;
        records[FINGERPRINT_RECORD_INDEX].verifier_identity = Some("verus@fixture".to_owned());
        records[FINGERPRINT_RECORD_INDEX].proof_artifact_identity = Some("proofs/fingerprint.receipt".to_owned());
        expected_hashes.insert(RUNNER_ARTIFACT.to_owned(), artifact_blake3_hex(b"different"));

        let diagnostics = validate_provenance_chain(&records, &expected_hashes);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unknown checker_id `unknown-checker`")));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing parent artifact `missing-parent.json`"))
        );
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("parent artifact cycle includes")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("hash mismatch for artifact locator")));
        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("invalid evidence_tier `policy_check` for checker_id `octet-transcript`")
        }));
        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("formal_proof overclaim by checker_id `octet-fingerprint`"))
        );
    }

    // r[verify octet.evidence_provenance.fail_closed.hash_mismatch]
    #[test]
    fn provenance_validation_rejects_hash_mismatch() {
        let record = ProvenanceRecord {
            blake3: artifact_blake3_hex(b"different artifact bytes"),
            ..valid_record(EvidenceTier::TranscriptAssertion)
        };

        let diagnostics = validate_with_defaults(&record);

        assert_eq!(diagnostics, vec![format!(
            "hash mismatch for artifact locator `{ARTIFACT_LOCATOR}`; expected blake3 {}, got {}",
            artifact_blake3_hex(ARTIFACT_BYTES),
            artifact_blake3_hex(b"different artifact bytes")
        )]);
    }

    const RUNNER_ARTIFACT: &str = "artifacts/octet/status.json";
    const POLICY_ARTIFACT: &str = "artifacts/policy/report.json";
    const FINGERPRINT_ARTIFACT: &str = "artifacts/fingerprint-review.json";
    const TRANSCRIPT_ARTIFACT: &str = "docs/examples/release-operator-smoke/transcript.md";

    fn release_chain_records() -> Vec<ProvenanceRecord> {
        vec![
            chain_record(
                RUNNER_ARTIFACT,
                b"runner-status",
                CHECKER_CARGO_OCTET,
                "octet.runner.artifact-bundle",
                EvidenceTier::ManualReviewRequired,
                Vec::new(),
            ),
            chain_record(
                POLICY_ARTIFACT,
                b"policy-report",
                CHECKER_OCTET_STANDARDS,
                "octet.standards.policy-check",
                EvidenceTier::PolicyCheck,
                vec![RUNNER_ARTIFACT.to_owned()],
            ),
            chain_record(
                FINGERPRINT_ARTIFACT,
                b"fingerprint-review",
                CHECKER_OCTET_FINGERPRINT,
                "octet.fingerprint.review-manifest",
                EvidenceTier::ManualReviewRequired,
                vec![POLICY_ARTIFACT.to_owned()],
            ),
            chain_record(
                TRANSCRIPT_ARTIFACT,
                b"transcript-smoke",
                CHECKER_OCTET_TRANSCRIPT,
                "octet.transcript.assertion",
                EvidenceTier::TranscriptAssertion,
                vec![RUNNER_ARTIFACT.to_owned()],
            ),
        ]
    }

    fn release_chain_hashes(records: &[ProvenanceRecord]) -> BTreeMap<String, String> {
        records.iter().map(|record| (record.artifact_locator.clone(), record.blake3.clone())).collect()
    }

    fn chain_record(
        artifact_locator: &str,
        artifact_bytes: &[u8],
        checker_id: &str,
        source_id: &str,
        evidence_tier: EvidenceTier,
        parent_artifacts: Vec<String>,
    ) -> ProvenanceRecord {
        ProvenanceRecord {
            schema_version: PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: source_id.to_owned(),
            checker_id: checker_id.to_owned(),
            evidence_tier,
            artifact_locator: artifact_locator.to_owned(),
            blake3: artifact_blake3_hex(artifact_bytes),
            checked_scope: "release smoke fixture".to_owned(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: (evidence_tier == EvidenceTier::ManualReviewRequired).then(|| HUMAN_ACTION.to_owned()),
            parent_artifacts,
        }
    }

    fn valid_record(evidence_tier: EvidenceTier) -> ProvenanceRecord {
        ProvenanceRecord {
            schema_version: PROVENANCE_SCHEMA_VERSION.to_owned(),
            source_id: SOURCE_ID.to_owned(),
            checker_id: KNOWN_CHECKER_ID.to_owned(),
            evidence_tier,
            artifact_locator: ARTIFACT_LOCATOR.to_owned(),
            blake3: artifact_blake3_hex(ARTIFACT_BYTES),
            checked_scope: "cargo-octet sidecar fixture".to_owned(),
            verifier_identity: None,
            proof_artifact_identity: None,
            human_action: None,
            parent_artifacts: Vec::new(),
        }
    }

    fn validate_with_defaults(record: &ProvenanceRecord) -> Vec<String> {
        let mut known_checker_ids = BTreeSet::new();
        known_checker_ids.insert(KNOWN_CHECKER_ID.to_owned());

        let mut expected_artifact_hashes = BTreeMap::new();
        expected_artifact_hashes.insert(ARTIFACT_LOCATOR.to_owned(), artifact_blake3_hex(ARTIFACT_BYTES));

        validate_provenance_record(record, &ProvenanceValidationInput {
            known_checker_ids: &known_checker_ids,
            expected_artifact_hashes: &expected_artifact_hashes,
        })
    }
}
