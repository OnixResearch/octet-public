use iddqd::IdOrdItem;
use iddqd::IdOrdMap;
use iddqd::id_upcast;
use serde::Deserialize;
use serde::Serialize;

const PROOF_CLAIM_CATALOG_SCHEMA_VERSION: &str = "octet-proof-claim-catalog/v1";
const STATUS_ACTIVE: &str = "active";
const STATUS_DEPRECATED: &str = "deprecated";
const STATUS_ALIAS: &str = "alias";
const EVIDENCE_TIER_FORMAL_PROOF: &str = "formal_proof";
const EVIDENCE_TIER_MANUAL_REVIEW_REQUIRED: &str = "manual_review_required";
const REQUIRED_NON_CLAIMS: &[&str] = &[
    "not whole-program correctness",
    "not certification",
    "not business-domain validation",
    "not semantic equivalence",
    "not full safety",
    "not verified runtime",
];
const OVERCLAIM_PATTERNS: &[(&str, &str)] = &[
    ("proves whole-program correctness", "whole-program correctness"),
    ("provides certification", "certification"),
    ("provides business-domain validation", "business-domain validation"),
    ("proves semantic equivalence", "semantic equivalence"),
    ("guarantees full safety", "full safety"),
    ("proves verified runtime", "verified runtime"),
    ("guarantees verified runtime", "verified runtime"),
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProofClaimCatalog {
    schema_version: String,
    claims: Vec<ProofClaimEntry>,
    #[serde(default)]
    aliases: Vec<ProofClaimAlias>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProofClaimEntry {
    claim_id: String,
    source_requirement_or_policy_id: String,
    checked_scope: String,
    expected_evidence_tier: String,
    allowed_proof_or_property_ids: Vec<String>,
    required_artifacts: Vec<String>,
    documentation_anchors: Vec<String>,
    owner_surface: String,
    status: String,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ProofClaimAlias {
    alias_id: String,
    target_claim_id: String,
    status: String,
    rationale: String,
}

impl IdOrdItem for ProofClaimEntry {
    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        &self.claim_id
    }

    id_upcast!();
}

impl IdOrdItem for ProofClaimAlias {
    type Key<'a> = &'a str;

    fn key(&self) -> Self::Key<'_> {
        &self.alias_id
    }

    id_upcast!();
}

struct CatalogIndex {
    claims: IdOrdMap<ProofClaimEntry>,
    aliases: IdOrdMap<ProofClaimAlias>,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ProofClaimReference {
    proof_or_property_id: String,
    source_requirement_or_policy_id: String,
    checked_scope: String,
    evidence_tier: String,
    status: String,
    artifacts: Vec<String>,
    documentation_anchor: String,
    non_claims: Vec<String>,
}

pub(crate) fn validate_catalog(catalog: &ProofClaimCatalog, known_documentation_anchors: &[String]) -> Vec<String> {
    let index = build_catalog_index(catalog);
    let mut diagnostics = index.diagnostics.clone();
    if catalog.schema_version != PROOF_CLAIM_CATALOG_SCHEMA_VERSION {
        diagnostics.push(format!(
            "proof claim catalog field `schema_version` expected `{PROOF_CLAIM_CATALOG_SCHEMA_VERSION}` but got `{}`",
            catalog.schema_version
        ));
    }
    diagnostics.extend(alias_diagnostics(&index, &catalog.aliases));
    for claim in &catalog.claims {
        diagnostics.extend(claim_shape_diagnostics(claim, known_documentation_anchors));
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn validate_claim_reference(catalog: &ProofClaimCatalog, reference: &ProofClaimReference) -> Vec<String> {
    let index = build_catalog_index(catalog);
    let mut diagnostics = index.diagnostics.clone();
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return diagnostics;
    }
    let claim = match resolve_claim(&index, &reference.proof_or_property_id) {
        Ok(claim) => claim,
        Err(diagnostic) => return vec![diagnostic],
    };
    if !claim.allowed_proof_or_property_ids.iter().any(|id| id == &reference.proof_or_property_id)
        && claim.claim_id != reference.proof_or_property_id
    {
        diagnostics.push(format!(
            "proof claim reference `{}` is not an allowed proof_or_property_id for claim `{}`",
            reference.proof_or_property_id, claim.claim_id
        ));
    }
    push_mismatch(
        &mut diagnostics,
        "source_requirement_or_policy_id",
        &claim.source_requirement_or_policy_id,
        &reference.source_requirement_or_policy_id,
    );
    push_mismatch(&mut diagnostics, "checked_scope", &claim.checked_scope, &reference.checked_scope);
    push_mismatch(&mut diagnostics, "evidence_tier", &claim.expected_evidence_tier, &reference.evidence_tier);
    if claim.expected_evidence_tier != EVIDENCE_TIER_FORMAL_PROOF
        && reference.evidence_tier == EVIDENCE_TIER_FORMAL_PROOF
    {
        diagnostics.push(format!(
            "proof claim reference `{}` attempts invalid formal_proof promotion from catalog tier `{}`",
            reference.proof_or_property_id, claim.expected_evidence_tier
        ));
    }
    if reference.status.trim().is_empty() {
        diagnostics.push("proof claim reference field `status` is empty".to_owned());
    }
    for required in &claim.required_artifacts {
        if !reference.artifacts.iter().any(|artifact| artifact == required) {
            diagnostics.push(format!(
                "proof claim reference `{}` missing required artifact `{required}`",
                reference.proof_or_property_id
            ));
        }
    }
    if !claim.documentation_anchors.iter().any(|anchor| anchor == &reference.documentation_anchor) {
        diagnostics.push(format!(
            "proof claim reference `{}` uses undocumented anchor `{}`",
            reference.proof_or_property_id, reference.documentation_anchor
        ));
    }
    diagnostics.extend(non_claim_diagnostics("proof claim reference", &reference.non_claims));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn resolve_claim<'a>(index: &'a CatalogIndex, claim_id: &str) -> Result<&'a ProofClaimEntry, String> {
    if let Some(claim) = index.claims.get(claim_id) {
        return Ok(claim);
    }
    if let Some(alias) = index.aliases.get(claim_id) {
        return index.claims.get(alias.target_claim_id.as_str()).ok_or_else(|| {
            format!("proof claim alias `{}` targets missing claim `{}`", alias.alias_id, alias.target_claim_id)
        });
    }
    Err(format!("proof claim reference `{claim_id}` is not declared in the proof claim catalog"))
}

fn build_catalog_index(catalog: &ProofClaimCatalog) -> CatalogIndex {
    let mut diagnostics = Vec::new();
    let mut claims = IdOrdMap::<ProofClaimEntry>::new();
    for claim in &catalog.claims {
        let claim_id = claim.claim_id.clone();
        if claims.insert_unique(claim.clone()).is_err() {
            diagnostics.push(format!("duplicate proof claim id `{claim_id}`"));
        }
    }

    let mut aliases = IdOrdMap::<ProofClaimAlias>::new();
    for alias in &catalog.aliases {
        let alias_id = alias.alias_id.clone();
        if claims.get(alias_id.as_str()).is_some() {
            diagnostics.push(format!("proof claim alias `{alias_id}` conflicts with a claim id"));
        }
        if aliases.insert_unique(alias.clone()).is_err() {
            diagnostics.push(format!("duplicate proof claim alias `{alias_id}`"));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    CatalogIndex {
        claims,
        aliases,
        diagnostics,
    }
}

fn alias_diagnostics(index: &CatalogIndex, aliases: &[ProofClaimAlias]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for alias in aliases {
        push_required(&mut diagnostics, "alias_id", &alias.alias_id);
        push_required(&mut diagnostics, "target_claim_id", &alias.target_claim_id);
        push_required(&mut diagnostics, "rationale", &alias.rationale);
        if !matches!(alias.status.as_str(), STATUS_ALIAS | STATUS_DEPRECATED) {
            diagnostics
                .push(format!("proof claim alias `{}` has unsupported status `{}`", alias.alias_id, alias.status));
        }
        if alias.alias_id == alias.target_claim_id {
            diagnostics.push(format!("proof claim alias `{}` must not target itself", alias.alias_id));
        }
        if !alias.target_claim_id.is_empty() && index.claims.get(alias.target_claim_id.as_str()).is_none() {
            diagnostics.push(format!(
                "proof claim alias `{}` targets missing claim `{}`",
                alias.alias_id, alias.target_claim_id
            ));
        }
    }
    diagnostics
}

fn claim_shape_diagnostics(claim: &ProofClaimEntry, known_documentation_anchors: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_required(&mut diagnostics, "claim_id", &claim.claim_id);
    push_required(&mut diagnostics, "source_requirement_or_policy_id", &claim.source_requirement_or_policy_id);
    push_required(&mut diagnostics, "checked_scope", &claim.checked_scope);
    push_required(&mut diagnostics, "owner_surface", &claim.owner_surface);
    if !matches!(claim.status.as_str(), STATUS_ACTIVE | STATUS_DEPRECATED) {
        diagnostics.push(format!("proof claim `{}` has unsupported status `{}`", claim.claim_id, claim.status));
    }
    if !matches!(
        claim.expected_evidence_tier.as_str(),
        EVIDENCE_TIER_FORMAL_PROOF | EVIDENCE_TIER_MANUAL_REVIEW_REQUIRED
    ) {
        diagnostics.push(format!(
            "proof claim `{}` has unsupported expected_evidence_tier `{}`",
            claim.claim_id, claim.expected_evidence_tier
        ));
    }
    push_required_collection(
        &mut diagnostics,
        &claim.claim_id,
        "allowed_proof_or_property_ids",
        &claim.allowed_proof_or_property_ids,
    );
    push_required_collection(&mut diagnostics, &claim.claim_id, "required_artifacts", &claim.required_artifacts);
    push_required_collection(&mut diagnostics, &claim.claim_id, "documentation_anchors", &claim.documentation_anchors);
    for anchor in &claim.documentation_anchors {
        if !known_documentation_anchors.iter().any(|known| known == anchor) {
            diagnostics
                .push(format!("proof claim `{}` references missing documentation anchor `{anchor}`", claim.claim_id));
        }
    }
    diagnostics.extend(non_claim_diagnostics(&format!("proof claim `{}`", claim.claim_id), &claim.non_claims));
    diagnostics
}

fn non_claim_diagnostics(owner: &str, non_claims: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for required in REQUIRED_NON_CLAIMS {
        if !non_claims.iter().any(|claim| claim == required) {
            diagnostics.push(format!("{owner} missing non-claim `{required}`"));
        }
    }
    let joined = non_claims.join(" ").to_lowercase();
    for (needle, label) in OVERCLAIM_PATTERNS {
        if joined.contains(needle) {
            diagnostics.push(format!("{owner} contains unscoped {label} claim"));
        }
    }
    diagnostics
}

fn push_required(diagnostics: &mut Vec<String>, field_name: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("proof claim catalog field `{field_name}` is empty"));
    }
}

fn push_required_collection(diagnostics: &mut Vec<String>, claim_id: &str, field_name: &str, values: &[String]) {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        diagnostics.push(format!("proof claim `{claim_id}` field `{field_name}` is empty"));
    }
}

fn push_mismatch(diagnostics: &mut Vec<String>, field_name: &str, expected: &str, actual: &str) {
    if expected != actual {
        diagnostics
            .push(format!("proof claim reference field `{field_name}` expected `{expected}` but got `{actual}`"));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CLAIM_ID: &str = "octet.verus_proof_rails.addition-bound#rail-0";
    const ALIAS_ID: &str = "octet.proof_claims.addition-bound-fixture";
    const DEPRECATED_ALIAS_ID: &str = "octet.proof_claims.addition-bound-legacy";
    const POLICY_ID: &str = "octet.verus_proof_rails.addition-bound";
    const CHECKED_SCOPE: &str = "formal-proof:fixture-only:octet.verus_proof_rails.addition-bound#rail-0";
    const DOC_ANCHOR: &str = "docs/trust-boundary-evidence.md#verus-proof-path-lane";
    const SOURCE_ARTIFACT: &str = "src/verified/addition.rs";
    const PROOF_ARTIFACT: &str = "verus/proofs/addition_bound.rs";
    const RECEIPT_ARTIFACT: &str = "proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt";

    #[test]
    fn proof_claim_catalog_generated_fixture_validates() {
        let catalog = generated_catalog();
        let diagnostics = validate_catalog(&catalog, &known_anchors());

        assert!(diagnostics.is_empty(), "unexpected catalog diagnostics: {diagnostics:?}");
        assert_eq!(catalog.claims[0].claim_id, CLAIM_ID);
    }

    #[test]
    fn proof_claim_catalog_rejects_duplicate_ids_and_missing_anchor() {
        let mut catalog = sample_catalog();
        catalog.claims.push(catalog.claims[0].clone());
        catalog.claims[0].documentation_anchors = vec!["docs/missing.md#anchor".to_owned()];

        let diagnostics = validate_catalog(&catalog, &known_anchors());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate proof claim id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("missing documentation anchor")));
    }

    #[test]
    fn proof_claim_reference_links_matching_card_receipt_and_docs() {
        let catalog = sample_catalog();
        let reference = sample_reference(CLAIM_ID);

        let diagnostics = validate_claim_reference(&catalog, &reference);

        assert!(diagnostics.is_empty(), "unexpected reference diagnostics: {diagnostics:?}");
    }

    #[test]
    fn proof_claim_reference_rejects_stale_artifact_tier_and_unknown_claim() {
        let catalog = sample_catalog();
        let mut stale = sample_reference(CLAIM_ID);
        stale.artifacts.retain(|artifact| artifact != RECEIPT_ARTIFACT);
        stale.evidence_tier = EVIDENCE_TIER_MANUAL_REVIEW_REQUIRED.to_owned();
        stale.documentation_anchor = "docs/other.md#claim".to_owned();

        let diagnostics = validate_claim_reference(&catalog, &stale);
        let unknown = validate_claim_reference(&catalog, &sample_reference("unknown.claim"));

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("missing required artifact")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("evidence_tier")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("undocumented anchor")));
        assert!(unknown.iter().any(|diagnostic| diagnostic.contains("not declared")));
    }

    #[test]
    fn proof_claim_aliases_resolve_only_when_explicit() {
        let catalog = sample_catalog();
        let alias_diagnostics = validate_claim_reference(&catalog, &sample_reference(ALIAS_ID));
        let deprecated_alias_diagnostics = validate_claim_reference(&catalog, &sample_reference(DEPRECATED_ALIAS_ID));
        let mut broken = catalog.clone();
        broken.aliases[0].target_claim_id = "missing.claim".to_owned();

        assert!(alias_diagnostics.is_empty(), "explicit alias should resolve: {alias_diagnostics:?}");
        assert!(
            deprecated_alias_diagnostics.is_empty(),
            "deprecated alias should still resolve when explicit: {deprecated_alias_diagnostics:?}"
        );
        assert!(
            validate_catalog(&broken, &known_anchors())
                .iter()
                .any(|diagnostic| diagnostic.contains("targets missing claim"))
        );
    }

    #[test]
    fn proof_claim_alias_index_rejects_non_bijective_hazards_without_rejecting_shared_targets() {
        let mut catalog = sample_catalog();
        catalog.aliases.push(ProofClaimAlias {
            alias_id: catalog.claims[0].claim_id.clone(),
            target_claim_id: CLAIM_ID.to_owned(),
            status: STATUS_ALIAS.to_owned(),
            rationale: "conflicts with canonical claim id".to_owned(),
        });
        catalog.aliases.push(ProofClaimAlias {
            alias_id: ALIAS_ID.to_owned(),
            target_claim_id: CLAIM_ID.to_owned(),
            status: STATUS_ALIAS.to_owned(),
            rationale: "duplicate alias id".to_owned(),
        });
        catalog.aliases.push(ProofClaimAlias {
            alias_id: "octet.proof_claims.self-target".to_owned(),
            target_claim_id: "octet.proof_claims.self-target".to_owned(),
            status: STATUS_ALIAS.to_owned(),
            rationale: "self target should fail".to_owned(),
        });
        catalog.aliases.push(ProofClaimAlias {
            alias_id: "octet.proof_claims.bad-status".to_owned(),
            target_claim_id: CLAIM_ID.to_owned(),
            status: STATUS_ACTIVE.to_owned(),
            rationale: "alias rows cannot be active claims".to_owned(),
        });

        let diagnostics = validate_catalog(&catalog, &known_anchors());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("conflicts with a claim id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate proof claim alias")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("must not target itself")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported status")));
    }

    #[test]
    fn proof_claim_catalog_rejects_invalid_tiers_and_weakened_reference_non_claims() {
        let mut catalog = sample_catalog();
        catalog.claims[0].expected_evidence_tier = "dependency_trust".to_owned();
        let catalog_diagnostics = validate_catalog(&catalog, &known_anchors());

        let mut reference = sample_reference(CLAIM_ID);
        reference.non_claims.retain(|claim| claim != "not certification");
        let reference_diagnostics = validate_claim_reference(&sample_catalog(), &reference);

        assert!(catalog_diagnostics.iter().any(|diagnostic| diagnostic.contains("expected_evidence_tier")));
        assert!(
            reference_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("missing non-claim `not certification`"))
        );
    }

    #[test]
    fn proof_claim_catalog_rejects_overbroad_claim_wording() {
        let mut catalog = sample_catalog();
        catalog.claims[0].non_claims.push("proves whole-program correctness".to_owned());

        let diagnostics = validate_catalog(&catalog, &known_anchors());

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("whole-program correctness")));
    }

    fn generated_catalog() -> ProofClaimCatalog {
        serde_json::from_str(include_str!("../../standards/proof-claims/generated/catalog.json"))
            .expect("generated claim catalog JSON should parse")
    }

    fn sample_catalog() -> ProofClaimCatalog {
        ProofClaimCatalog {
            schema_version: PROOF_CLAIM_CATALOG_SCHEMA_VERSION.to_owned(),
            claims: vec![ProofClaimEntry {
                claim_id: CLAIM_ID.to_owned(),
                source_requirement_or_policy_id: POLICY_ID.to_owned(),
                checked_scope: CHECKED_SCOPE.to_owned(),
                expected_evidence_tier: EVIDENCE_TIER_FORMAL_PROOF.to_owned(),
                allowed_proof_or_property_ids: vec![
                    CLAIM_ID.to_owned(),
                    ALIAS_ID.to_owned(),
                    DEPRECATED_ALIAS_ID.to_owned(),
                ],
                required_artifacts: vec![
                    SOURCE_ARTIFACT.to_owned(),
                    PROOF_ARTIFACT.to_owned(),
                    RECEIPT_ARTIFACT.to_owned(),
                ],
                documentation_anchors: vec![DOC_ANCHOR.to_owned()],
                owner_surface: "cargo-octet prove".to_owned(),
                status: STATUS_ACTIVE.to_owned(),
                non_claims: REQUIRED_NON_CLAIMS.iter().map(|claim| (*claim).to_owned()).collect(),
            }],
            aliases: vec![
                ProofClaimAlias {
                    alias_id: ALIAS_ID.to_owned(),
                    target_claim_id: CLAIM_ID.to_owned(),
                    status: STATUS_ALIAS.to_owned(),
                    rationale: "short reviewer-facing alias".to_owned(),
                },
                ProofClaimAlias {
                    alias_id: DEPRECATED_ALIAS_ID.to_owned(),
                    target_claim_id: CLAIM_ID.to_owned(),
                    status: STATUS_DEPRECATED.to_owned(),
                    rationale: "legacy proof-card name retained for archived receipts".to_owned(),
                },
            ],
        }
    }

    fn sample_reference(proof_or_property_id: &str) -> ProofClaimReference {
        ProofClaimReference {
            proof_or_property_id: proof_or_property_id.to_owned(),
            source_requirement_or_policy_id: POLICY_ID.to_owned(),
            checked_scope: CHECKED_SCOPE.to_owned(),
            evidence_tier: EVIDENCE_TIER_FORMAL_PROOF.to_owned(),
            status: "passed".to_owned(),
            artifacts: vec![
                SOURCE_ARTIFACT.to_owned(),
                PROOF_ARTIFACT.to_owned(),
                RECEIPT_ARTIFACT.to_owned(),
            ],
            documentation_anchor: DOC_ANCHOR.to_owned(),
            non_claims: REQUIRED_NON_CLAIMS.iter().map(|claim| (*claim).to_owned()).collect(),
        }
    }

    fn known_anchors() -> Vec<String> {
        vec![DOC_ANCHOR.to_owned()]
    }
}
