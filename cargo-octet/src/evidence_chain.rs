use std::collections::BTreeSet;

use serde::Serialize;

const BLAKE3_PREFIX: &str = "b3:";
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const JSON_AUTHORITY_FRAGMENT: &str = "json";
const EMPTY_FIELD_DIAGNOSTIC: &str = "missing required evidence-chain field";
const MALFORMED_DIGEST_DIAGNOSTIC: &str = "malformed BLAKE3 identity";
const DUPLICATE_ROOT_DIAGNOSTIC: &str = "duplicate canonical root";
const UNTYPED_PARENT_DIAGNOSTIC: &str = "untyped parent link";
const JSON_ONLY_AUTHORITY_DIAGNOSTIC: &str = "JSON projection cannot be canonical evidence-chain authority";
const NON_CLAIM_DIAGNOSTIC: &str = "evidence-chain handoff is missing required non-claim boundary";

pub const EVIDENCE_CHAIN_HANDOFF_SCHEMA: &str = "octet.evidence-chain.producer-handoff.v1";
pub const EVIDENCE_KIND_FUNCTION_ADDRESS: &str = "function-address";
pub const ROOT_KIND_FUNCTION_ADDRESS_RECORDS: &str = "function-address-record-root";
pub const PARENT_KIND_SOURCE_ENVELOPE: &str = "source-envelope";
pub const PROJECTION_KIND_COMPAT_JSON: &str = "compatibility-json";
pub const PROJECTION_AUTHORITY_NON_AUTHORITATIVE: &str = "non-authoritative-projection";

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EvidenceChainProducer {
    pub name: String,
    pub role: String,
    pub tool_version: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub struct TypedDigest {
    pub kind: String,
    pub digest: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EvidenceChainProjection {
    pub kind: String,
    pub artifact: String,
    pub digest: String,
    pub authority: String,
}

// r[impl octet.evidence_provenance.evidence_chain_handoff.contract]
#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
pub struct EvidenceChainHandoff {
    pub schema: String,
    pub evidence_kind: String,
    pub producer: EvidenceChainProducer,
    pub run_identity: String,
    pub policy_hash: String,
    pub profile_hash: String,
    pub canonical_authority: String,
    pub canonical_roots: Vec<TypedDigest>,
    pub parent_links: Vec<TypedDigest>,
    pub projections: Vec<EvidenceChainProjection>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct EvidenceChainValidationPolicy<'a> {
    pub required_canonical_authority: &'a str,
    pub required_non_claims: &'a [&'a str],
}

// r[impl octet.evidence_provenance.evidence_chain_handoff.core]
#[must_use]
pub fn validate_evidence_chain_handoff(
    handoff: &EvidenceChainHandoff,
    policy: EvidenceChainValidationPolicy<'_>,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    validate_required_text("schema", &handoff.schema, &mut diagnostics);
    validate_required_text("evidence_kind", &handoff.evidence_kind, &mut diagnostics);
    validate_required_text("producer.name", &handoff.producer.name, &mut diagnostics);
    validate_required_text("producer.role", &handoff.producer.role, &mut diagnostics);
    validate_required_text("producer.tool_version", &handoff.producer.tool_version, &mut diagnostics);
    validate_required_text("canonical_authority", &handoff.canonical_authority, &mut diagnostics);
    validate_required_digest("run_identity", &handoff.run_identity, &mut diagnostics);
    validate_required_digest("policy_hash", &handoff.policy_hash, &mut diagnostics);
    validate_required_digest("profile_hash", &handoff.profile_hash, &mut diagnostics);
    validate_canonical_authority(handoff, policy.required_canonical_authority, &mut diagnostics);
    validate_typed_digests("canonical_roots", &handoff.canonical_roots, &mut diagnostics);
    validate_canonical_roots(&handoff.canonical_roots, &mut diagnostics);
    validate_typed_digests("parent_links", &handoff.parent_links, &mut diagnostics);
    validate_projections(&handoff.projections, &mut diagnostics);
    validate_non_claims(&handoff.non_claims, policy.required_non_claims, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_required_text(field: &str, value: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{EMPTY_FIELD_DIAGNOSTIC}: {field}"));
    }
}

fn validate_required_digest(field: &str, value: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{EMPTY_FIELD_DIAGNOSTIC}: {field}"));
    } else if !is_blake3_identity(value) {
        diagnostics.push(format!("{MALFORMED_DIGEST_DIAGNOSTIC}: {field}"));
    }
}

fn validate_canonical_authority(
    handoff: &EvidenceChainHandoff,
    required_authority: &str,
    diagnostics: &mut Vec<String>,
) {
    if handoff.canonical_authority != required_authority
        || handoff.canonical_authority.to_ascii_lowercase().contains(JSON_AUTHORITY_FRAGMENT)
    {
        diagnostics.push(JSON_ONLY_AUTHORITY_DIAGNOSTIC.to_owned());
    }
}

fn validate_typed_digests(field: &str, digests: &[TypedDigest], diagnostics: &mut Vec<String>) {
    if digests.is_empty() {
        diagnostics.push(format!("{EMPTY_FIELD_DIAGNOSTIC}: {field}"));
    }
    for digest in digests {
        if digest.kind.trim().is_empty() {
            diagnostics.push(format!("{UNTYPED_PARENT_DIAGNOSTIC}: {field}"));
        }
        validate_required_digest(field, &digest.digest, diagnostics);
    }
}

fn validate_canonical_roots(roots: &[TypedDigest], diagnostics: &mut Vec<String>) {
    let mut seen = BTreeSet::new();
    for root in roots {
        if !seen.insert((root.kind.as_str(), root.digest.as_str())) {
            diagnostics.push(DUPLICATE_ROOT_DIAGNOSTIC.to_owned());
        }
    }
}

// r[impl octet.evidence_provenance.evidence_chain_handoff.projection]
fn validate_projections(projections: &[EvidenceChainProjection], diagnostics: &mut Vec<String>) {
    if projections.is_empty() {
        diagnostics.push(format!("{EMPTY_FIELD_DIAGNOSTIC}: projections"));
    }
    for projection in projections {
        validate_required_text("projection.kind", &projection.kind, diagnostics);
        validate_required_text("projection.artifact", &projection.artifact, diagnostics);
        validate_required_digest("projection.digest", &projection.digest, diagnostics);
        if projection.authority != PROJECTION_AUTHORITY_NON_AUTHORITATIVE {
            diagnostics.push(JSON_ONLY_AUTHORITY_DIAGNOSTIC.to_owned());
        }
    }
}

fn validate_non_claims(non_claims: &[String], required_non_claims: &[&str], diagnostics: &mut Vec<String>) {
    for required in required_non_claims {
        if !non_claims.iter().any(|non_claim| non_claim == required) {
            diagnostics.push(format!("{NON_CLAIM_DIAGNOSTIC}: {required}"));
        }
    }
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn is_blake3_identity(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(is_blake3_hex)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::determinism;

    const REQUIRED_NON_CLAIM: &str = "not release eligibility";
    const REQUIRED_AUTHORITY: &str = "kamacite-preserves-bytes";

    fn validation_policy() -> EvidenceChainValidationPolicy<'static> {
        EvidenceChainValidationPolicy {
            required_canonical_authority: REQUIRED_AUTHORITY,
            required_non_claims: &[REQUIRED_NON_CLAIM],
        }
    }

    fn valid_handoff() -> EvidenceChainHandoff {
        EvidenceChainHandoff {
            schema: EVIDENCE_CHAIN_HANDOFF_SCHEMA.to_owned(),
            evidence_kind: EVIDENCE_KIND_FUNCTION_ADDRESS.to_owned(),
            producer: EvidenceChainProducer {
                name: "octet".to_owned(),
                role: "rust-source-shape-producer".to_owned(),
                tool_version: env!("CARGO_PKG_VERSION").to_owned(),
            },
            run_identity: determinism::full_hash("run"),
            policy_hash: determinism::full_hash("policy"),
            profile_hash: determinism::full_hash("profile"),
            canonical_authority: REQUIRED_AUTHORITY.to_owned(),
            canonical_roots: vec![TypedDigest {
                kind: ROOT_KIND_FUNCTION_ADDRESS_RECORDS.to_owned(),
                digest: determinism::full_hash("records"),
            }],
            parent_links: vec![TypedDigest {
                kind: PARENT_KIND_SOURCE_ENVELOPE.to_owned(),
                digest: determinism::full_hash("source"),
            }],
            projections: vec![EvidenceChainProjection {
                kind: PROJECTION_KIND_COMPAT_JSON.to_owned(),
                artifact: "function-addresses.json".to_owned(),
                digest: determinism::full_hash("projection"),
                authority: PROJECTION_AUTHORITY_NON_AUTHORITATIVE.to_owned(),
            }],
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }

    // r[verify octet.evidence_provenance.evidence_chain_handoff.positive]
    #[test]
    fn valid_evidence_chain_handoff_passes() {
        let handoff = valid_handoff();

        let diagnostics = validate_evidence_chain_handoff(&handoff, validation_policy());

        assert!(diagnostics.is_empty());
    }

    // r[verify octet.evidence_provenance.evidence_chain_handoff.negative]
    #[test]
    fn invalid_evidence_chain_handoff_rejects_missing_untyped_json_only_and_weakened_non_claims() {
        let mut handoff = valid_handoff();
        handoff.evidence_kind.clear();
        handoff.policy_hash = "not-a-blake3".to_owned();
        handoff.canonical_authority = "json".to_owned();
        handoff.canonical_roots.push(handoff.canonical_roots[0].clone());
        handoff.parent_links[0].kind.clear();
        handoff.projections[0].authority = "canonical-json".to_owned();
        handoff.non_claims.clear();

        let diagnostics = validate_evidence_chain_handoff(&handoff, validation_policy());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("evidence_kind")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("policy_hash")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("untyped parent")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate canonical root")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("JSON projection")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("non-claim")));
    }
}
