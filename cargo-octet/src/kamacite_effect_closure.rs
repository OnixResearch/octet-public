//! Pure Kamacite effect-closure handoff admission.
//!
//! Octet owns selected-source-token effect evidence. Kamacite owns canonical
//! Preserves identities and semantic effect-operation meaning. This module
//! compares the two bounded artifacts without rewriting either parent and
//! without claiming canonicalization, consumer, publication, or runtime
//! authority.
//!
//! The module is a functional core: it reads no files, environment, clock, or
//! process state. The runner shell loads profile and projection bytes and
//! writes artifacts.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::effect_closure;
use crate::effect_closure::AdmissionStatus;
use crate::effect_closure::Effect;
use crate::effect_closure::EffectClosureReceipt;
use crate::effect_closure::EvidenceReference;
use crate::effect_closure::ProviderIdentity;

pub const PROFILE_SCHEMA_VERSION: &str = "octet-effect-closure-kamacite-profile/v1";
pub const PROJECTION_SCHEMA_VERSION: &str = "octet-kamacite-effect-summary-projection/v1";
pub const COMPARISON_SCHEMA_VERSION: &str = "octet-kamacite-effect-comparison/v1";
pub const HANDOFF_SCHEMA_VERSION: &str = "octet-kamacite-effect-handoff/v1";
pub const CLAIM_SCOPE: &str = "selected-source-token-kamacite-effect-comparison";
pub const PRESERVES_PROFILE_SCHEMA: &str = "kamacite.preserves-profile.v1";
pub const EFFECT_SUMMARY_SCHEMA: &str = "kamacite.effect-summary.v1";
pub const PRESERVES_PROFILE_DOMAIN: &str = "projection";
pub const EFFECT_SUMMARY_DOMAIN: &str = "algebraic-effect";
pub const EFFECT_OPERATION_DOMAIN: &str = "effect-operation";
pub const KAMACITE_CANONICAL_IDENTITY_OWNER: &str = "kamacite-preserves-bytes";
pub const OCTET_PRODUCER_NAME: &str = "octet";
pub const OCTET_PRODUCER_ROLE: &str = "source-effect-evidence-producer";
pub const JSON_PROJECTION_ROLE: &str = "non-authoritative-compatibility-projection";
pub const CANONICALIZATION_OWNER: &str = "kamacite";
pub const DETERMINISTIC_CORE_PROFILE: &str = effect_closure::DETERMINISTIC_CORE_PROFILE;

const PROFILE_IDENTITY_DOMAIN: &str = "octet-kamacite-effect-profile/v1";
const PROJECTION_IDENTITY_DOMAIN: &str = "octet-kamacite-effect-summary-projection/v1";
const COMPARISON_IDENTITY_DOMAIN: &str = "octet-kamacite-effect-comparison/v1";
const HANDOFF_IDENTITY_DOMAIN: &str = "octet-kamacite-effect-handoff/v1";
const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const CLAUSE_NEGATION_WINDOW: usize = 120;

const DIAGNOSTIC_PROFILE_INVALID: &str = "kamacite_profile_invalid";
const DIAGNOSTIC_PROJECTION_INVALID: &str = "kamacite_projection_invalid";
const DIAGNOSTIC_BINDING_STALE: &str = "kamacite_binding_stale";
const DIAGNOSTIC_AMBIENT_DENIED: &str = "ambient_effect_denied";
const DIAGNOSTIC_UNMAPPED_AMBIENT: &str = "unmapped_ambient_effect";
const DIAGNOSTIC_CLOSURE_INVALID: &str = "closure_receipt_invalid";
const DIAGNOSTIC_CLOSURE_NOT_ADMITTED: &str = "closure_not_admitted";
const DIAGNOSTIC_STALE_EMISSION: &str = "stale_semantic_emission_identity";
const DIAGNOSTIC_AMBIGUOUS_EMISSION: &str = "ambiguous_semantic_emission";
const DIAGNOSTIC_EMITTED_NOT_DECLARED: &str = "emitted_not_declared";
const DIAGNOSTIC_SUMMARY_NOT_CLOSED: &str = "effect_summary_not_closed";
const DIAGNOSTIC_OVERCLAIM: &str = "effect_comparison_overclaim";
const DIAGNOSTIC_HANDOFF_INVALID: &str = "kamacite_handoff_invalid";
const DIAGNOSTIC_HANDOFF_IDENTITY: &str = "kamacite_handoff_identity_mismatch";
const DIAGNOSTIC_HANDOFF_PARENT: &str = "kamacite_handoff_parent_identity_mismatch";

const AMBIENT_EFFECT_ORDER: &[Effect] = &[
    Effect::Alloc,
    Effect::Panic,
    Effect::Unsafe,
    Effect::Clock,
    Effect::Random,
    Effect::Fs,
    Effect::Net,
    Effect::Spawn,
    Effect::Lock,
    Effect::Ffi,
    Effect::Unknown,
];

const REQUIRED_PROFILE_NON_CLAIMS: &[&str] = &[
    "does not prove complete Rust effects, semantic determinism, runtime reachability, handler correctness, replay truth, property satisfaction, or release eligibility",
    "does not canonicalize Kamacite data",
];

const REQUIRED_HANDOFF_NON_CLAIMS: &[&str] = &[
    "This handoff is a non-authoritative compatibility projection; Kamacite Preserves bytes remain the canonical identity owner.",
    "This handoff records bounded agreement between selected Octet source-token evidence and the supplied frozen Kamacite projection; it does not prove complete Rust effects, semantic determinism, runtime reachability, handler correctness, replay truth, property satisfaction, or release eligibility.",
    "Octet does not canonicalize Kamacite data and this handoff grants no consumer, publication, or runtime authority.",
];

/// Affirmative phrases that would promote Octet's comparison beyond its bounded
/// agreement claim. Negated forms are permitted and expected in non-claims.
const OVERCLAIM_PHRASES: &[&str] = &[
    "proves semantic determinism",
    "proves runtime determinism",
    "proves semantic equivalence",
    "proves complete rust effects",
    "proves runtime reachability",
    "proves handler correctness",
    "proves replay truth",
    "proves release eligibility",
    "establishes semantic determinism",
    "establishes runtime determinism",
    "guarantees semantic determinism",
    "guarantees runtime determinism",
    "owns canonical kamacite semantics",
    "canonicalizes kamacite",
    "octet is the canonical",
];

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct KamaciteIdentity {
    pub domain: String,
    pub algorithm: String,
    pub hex: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AmbientDisposition {
    ProfileLocal,
    Denied,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct AmbientDispositionRule {
    pub effect: Effect,
    pub disposition: AmbientDisposition,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct KamaciteBinding {
    pub preserves_profile_schema: String,
    pub preserves_profile_identity: KamaciteIdentity,
    pub effect_summary_schema: String,
    pub effect_summary_identity: KamaciteIdentity,
    pub projection: String,
    pub projection_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct SemanticEmissionRule {
    pub function_address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_site_id: Option<String>,
    pub provider: ProviderIdentity,
    pub operation_identity: KamaciteIdentity,
    pub evidence: EvidenceReference,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct KamaciteEffectProfile {
    pub schema_version: String,
    pub profile_id: String,
    pub mapping_id: String,
    pub kamacite: KamaciteBinding,
    pub ambient_dispositions: Vec<AmbientDispositionRule>,
    pub semantic_emissions: Vec<SemanticEmissionRule>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct KamaciteOperation {
    pub operation_identity: KamaciteIdentity,
    pub effect_family_identity: KamaciteIdentity,
    pub display_name: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct KamaciteEffectSummaryProjection {
    pub schema_version: String,
    pub kamacite_schema_version: String,
    pub preserves_profile_schema: String,
    pub preserves_profile_identity: KamaciteIdentity,
    pub effect_summary_identity: KamaciteIdentity,
    pub effect_summary_closed: bool,
    pub operations: Vec<KamaciteOperation>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComparisonClass {
    EmittedAndDeclared,
    ProfileLocal,
    DeclaredNotEmitted,
    EmittedNotDeclared,
    StaleIdentity,
    AmbiguousEmission,
    Unknown,
}

impl ComparisonClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmittedAndDeclared => "emitted-and-declared",
            Self::ProfileLocal => "profile-local",
            Self::DeclaredNotEmitted => "declared-not-emitted",
            Self::EmittedNotDeclared => "emitted-not-declared",
            Self::StaleIdentity => "stale-identity",
            Self::AmbiguousEmission => "ambiguous-emission",
            Self::Unknown => "unknown",
        }
    }

    pub fn is_admitted(self) -> bool {
        matches!(self, Self::EmittedAndDeclared | Self::ProfileLocal | Self::DeclaredNotEmitted)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ComparisonRow {
    pub class: ComparisonClass,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effect: Option<Effect>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_site_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<ProviderIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_identity: Option<KamaciteIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct ComparisonDiagnostic {
    pub code: String,
    pub class: ComparisonClass,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_site_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub operation_identity: Option<KamaciteIdentity>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct KamaciteEffectComparison {
    pub schema_version: String,
    pub status: AdmissionStatus,
    pub claim_scope: String,
    pub producer_name: String,
    pub producer_role: String,
    pub producer_tool_version: String,
    pub profile_id: String,
    pub mapping_id: String,
    pub mapping_profile_identity: String,
    pub closure_receipt_identity: String,
    pub projection_identity: String,
    pub preserves_profile_schema: String,
    pub preserves_profile_identity: KamaciteIdentity,
    pub effect_summary_schema: String,
    pub effect_summary_identity: KamaciteIdentity,
    pub rows: Vec<ComparisonRow>,
    pub diagnostics: Vec<ComparisonDiagnostic>,
    pub non_claims: Vec<String>,
    pub comparison_identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HandoffProducer {
    pub name: String,
    pub role: String,
    pub tool_version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct KamaciteEffectHandoff {
    pub schema_version: String,
    pub producer: HandoffProducer,
    pub canonical_authority: String,
    pub json_projection_role: String,
    pub canonicalization_owner: String,
    pub claim_scope: String,
    pub profile_id: String,
    pub mapping_id: String,
    pub mapping_profile_identity: String,
    pub closure_receipt_identity: String,
    pub projection_identity: String,
    pub preserves_profile_schema: String,
    pub preserves_profile_identity: KamaciteIdentity,
    pub effect_summary_schema: String,
    pub effect_summary_identity: KamaciteIdentity,
    pub rows: Vec<ComparisonRow>,
    pub diagnostics: Vec<ComparisonDiagnostic>,
    pub semantic_claims: Vec<String>,
    pub non_claims: Vec<String>,
    pub handoff_identity: String,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct ValidationIssue {
    pub code: &'static str,
    pub message: String,
}

impl ValidationIssue {
    fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}

#[derive(Serialize)]
struct CanonicalProfile<'a> {
    schema_version: &'a str,
    profile_id: &'a str,
    mapping_id: &'a str,
    preserves_profile_schema: &'a str,
    preserves_profile_identity: &'a KamaciteIdentity,
    effect_summary_schema: &'a str,
    effect_summary_identity: &'a KamaciteIdentity,
    ambient_dispositions: Vec<&'a AmbientDispositionRule>,
    semantic_emissions: Vec<&'a SemanticEmissionRule>,
    non_claims: Vec<&'a str>,
}

#[derive(Serialize)]
struct CanonicalProjection<'a> {
    schema_version: &'a str,
    kamacite_schema_version: &'a str,
    preserves_profile_schema: &'a str,
    preserves_profile_identity: &'a KamaciteIdentity,
    effect_summary_identity: &'a KamaciteIdentity,
    effect_summary_closed: bool,
    operations: Vec<&'a KamaciteOperation>,
    non_claims: Vec<&'a str>,
}

fn identity(domain: &str, value: &impl Serialize) -> String {
    let json = serde_json::to_string(value).expect("canonical identity payload serializes");
    let mut hasher = blake3::Hasher::new();
    hasher.update(domain.as_bytes());
    hasher.update(&[0]);
    hasher.update(json.as_bytes());
    format!("{BLAKE3_PREFIX}{}", hasher.finalize().to_hex())
}

fn sorted_non_claims(items: &[String]) -> Vec<&str> {
    let mut values = items.iter().map(String::as_str).collect::<Vec<_>>();
    values.sort_unstable();
    values.dedup();
    values
}

pub fn profile_identity(profile: &KamaciteEffectProfile) -> String {
    let mut dispositions = profile.ambient_dispositions.iter().collect::<Vec<_>>();
    dispositions.sort();
    let mut emissions = profile.semantic_emissions.iter().collect::<Vec<_>>();
    emissions.sort();
    identity(PROFILE_IDENTITY_DOMAIN, &CanonicalProfile {
        schema_version: &profile.schema_version,
        profile_id: &profile.profile_id,
        mapping_id: &profile.mapping_id,
        preserves_profile_schema: &profile.kamacite.preserves_profile_schema,
        preserves_profile_identity: &profile.kamacite.preserves_profile_identity,
        effect_summary_schema: &profile.kamacite.effect_summary_schema,
        effect_summary_identity: &profile.kamacite.effect_summary_identity,
        ambient_dispositions: dispositions,
        semantic_emissions: emissions,
        non_claims: sorted_non_claims(&profile.non_claims),
    })
}

pub fn projection_identity(projection: &KamaciteEffectSummaryProjection) -> String {
    let mut operations = projection.operations.iter().collect::<Vec<_>>();
    operations.sort();
    identity(PROJECTION_IDENTITY_DOMAIN, &CanonicalProjection {
        schema_version: &projection.schema_version,
        kamacite_schema_version: &projection.kamacite_schema_version,
        preserves_profile_schema: &projection.preserves_profile_schema,
        preserves_profile_identity: &projection.preserves_profile_identity,
        effect_summary_identity: &projection.effect_summary_identity,
        effect_summary_closed: projection.effect_summary_closed,
        operations,
        non_claims: sorted_non_claims(&projection.non_claims),
    })
}

fn comparison_identity(comparison: &KamaciteEffectComparison) -> String {
    let mut canonical = comparison.clone();
    canonical.comparison_identity.clear();
    identity(COMPARISON_IDENTITY_DOMAIN, &canonical)
}

fn handoff_identity(handoff: &KamaciteEffectHandoff) -> String {
    let mut canonical = handoff.clone();
    canonical.handoff_identity.clear();
    identity(HANDOFF_IDENTITY_DOMAIN, &canonical)
}

pub fn canonical_json<T: Serialize>(value: &T) -> Result<String, String> {
    let mut rendered = serde_json::to_string_pretty(value).map_err(|error| format!("serialize JSON: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

fn is_lower_blake3_hex(hex: &str) -> bool {
    hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn is_blake3_identity(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(is_lower_blake3_hex)
}

fn is_sorted_unique<T: Ord>(items: &[T]) -> bool {
    items.windows(2).all(|window| window[0] < window[1])
}

fn validate_identity(
    label: &str,
    identity: &KamaciteIdentity,
    expected_domain: &str,
    code: &'static str,
    issues: &mut Vec<ValidationIssue>,
) {
    if identity.domain != expected_domain {
        issues.push(ValidationIssue::new(
            code,
            format!("{label} domain must be `{expected_domain}`, got `{}`", identity.domain),
        ));
    }
    if identity.algorithm != "blake3" {
        issues.push(ValidationIssue::new(
            code,
            format!("{label} algorithm must be `blake3`, got `{}`", identity.algorithm),
        ));
    }
    if !is_lower_blake3_hex(&identity.hex) {
        issues.push(ValidationIssue::new(code, format!("{label} hex is not a full lowercase BLAKE3 digest")));
    }
}

fn validate_provider(provider: &ProviderIdentity, label: &str, code: &'static str, issues: &mut Vec<ValidationIssue>) {
    if provider.name.trim().is_empty()
        || provider.version.trim().is_empty()
        || provider.source.trim().is_empty()
        || provider.target.trim().is_empty()
    {
        issues.push(ValidationIssue::new(code, format!("{label} requires exact name, version, source, and target")));
    }
    if !is_sorted_unique(&provider.features) {
        issues.push(ValidationIssue::new(code, format!("{label} features must be sorted and unique")));
    }
}

fn validate_evidence(evidence: &EvidenceReference, label: &str, code: &'static str, issues: &mut Vec<ValidationIssue>) {
    if evidence.kind.trim().is_empty() || evidence.locator.trim().is_empty() || !is_blake3_identity(&evidence.blake3) {
        issues.push(ValidationIssue::new(code, format!("{label} requires an exact evidence reference")));
    }
}

// r[impl octet.effect_closure.kamacite_boundary]
/// Rejects affirmative promotion of the comparison beyond bounded agreement.
/// Negated statements, which non-claims use, are permitted.
pub fn validate_claim_boundary(text: &str) -> Vec<ValidationIssue> {
    let lower = text.to_lowercase();
    let mut issues = Vec::new();
    for phrase in OVERCLAIM_PHRASES {
        let mut search = 0;
        while let Some(offset) = lower[search..].find(phrase) {
            let start = search + offset;
            let clause_start = lower[..start]
                .rfind(['.', ';', '\n'])
                .map_or(start.saturating_sub(CLAUSE_NEGATION_WINDOW), |index| index + 1);
            let clause = &lower[clause_start..start];
            if !is_negated(clause) {
                issues.push(ValidationIssue::new(DIAGNOSTIC_OVERCLAIM, format!("affirmative overclaim `{phrase}`")));
            }
            search = start + phrase.len();
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

fn is_negated(clause: &str) -> bool {
    [
        "not ",
        "never ",
        "cannot ",
        "can't ",
        "doesn't ",
        "must not ",
        "without ",
        "isn't ",
        "no ",
    ]
    .iter()
    .any(|marker| clause.contains(marker))
}

// r[impl octet.effect_closure.kamacite_mapping]
pub fn validate_profile(profile: &KamaciteEffectProfile) -> Vec<ValidationIssue> {
    let code = DIAGNOSTIC_PROFILE_INVALID;
    let mut issues = Vec::new();
    if profile.schema_version != PROFILE_SCHEMA_VERSION {
        issues.push(ValidationIssue::new(
            code,
            format!("schema_version must be `{PROFILE_SCHEMA_VERSION}`, got `{}`", profile.schema_version),
        ));
    }
    if profile.profile_id != DETERMINISTIC_CORE_PROFILE {
        issues.push(ValidationIssue::new(
            code,
            format!("profile_id must be `{DETERMINISTIC_CORE_PROFILE}`, got `{}`", profile.profile_id),
        ));
    }
    if profile.mapping_id.trim().is_empty() {
        issues.push(ValidationIssue::new(code, "mapping_id must not be empty"));
    }
    issues.extend(validate_claim_boundary(&profile.mapping_id));
    if profile.kamacite.preserves_profile_schema != PRESERVES_PROFILE_SCHEMA {
        issues.push(ValidationIssue::new(
            code,
            format!(
                "preserves_profile_schema must be `{PRESERVES_PROFILE_SCHEMA}`, got `{}`",
                profile.kamacite.preserves_profile_schema
            ),
        ));
    }
    if profile.kamacite.effect_summary_schema != EFFECT_SUMMARY_SCHEMA {
        issues.push(ValidationIssue::new(
            code,
            format!(
                "effect_summary_schema must be `{EFFECT_SUMMARY_SCHEMA}`, got `{}`",
                profile.kamacite.effect_summary_schema
            ),
        ));
    }
    validate_identity(
        "preserves_profile_identity",
        &profile.kamacite.preserves_profile_identity,
        PRESERVES_PROFILE_DOMAIN,
        code,
        &mut issues,
    );
    validate_identity(
        "effect_summary_identity",
        &profile.kamacite.effect_summary_identity,
        EFFECT_SUMMARY_DOMAIN,
        code,
        &mut issues,
    );
    if !is_workspace_relative(&profile.kamacite.projection) {
        issues.push(ValidationIssue::new(code, "projection must be a non-empty workspace-relative path"));
    }
    if !is_blake3_identity(&profile.kamacite.projection_blake3) {
        issues.push(ValidationIssue::new(code, "projection_blake3 must be a full BLAKE3 identity"));
    }
    validate_dispositions(profile, &mut issues);
    let mut emission_rules = BTreeSet::new();
    for rule in &profile.semantic_emissions {
        if !emission_rules.insert(rule) {
            issues.push(ValidationIssue::new(
                code,
                "semantic_emissions must not repeat one exact site, provider, and operation identity",
            ));
        }
    }
    for (index, rule) in profile.semantic_emissions.iter().enumerate() {
        let label = format!("semantic emission {}", index + 1);
        if !is_blake3_identity(&rule.function_address) {
            issues.push(ValidationIssue::new(code, format!("{label} function_address is not a full BLAKE3 identity")));
        }
        if rule.call_site_id.as_deref().is_some_and(|id| !is_blake3_identity(id)) {
            issues.push(ValidationIssue::new(code, format!("{label} call_site_id is not a full BLAKE3 identity")));
        }
        validate_provider(&rule.provider, &label, code, &mut issues);
        validate_identity("operation_identity", &rule.operation_identity, EFFECT_OPERATION_DOMAIN, code, &mut issues);
        validate_evidence(&rule.evidence, &label, code, &mut issues);
        if rule.non_claims.is_empty() || rule.non_claims.iter().any(|non_claim| non_claim.trim().is_empty()) {
            issues.push(ValidationIssue::new(code, format!("{label} requires explicit non-claims")));
        }
    }
    if profile.non_claims.is_empty() {
        issues.push(ValidationIssue::new(code, "non_claims must not be empty"));
    }
    let joined = profile.non_claims.join("\n");
    issues.extend(validate_claim_boundary(&joined));
    for required in REQUIRED_PROFILE_NON_CLAIMS {
        if !profile.non_claims.iter().any(|non_claim| non_claim.contains(required)) {
            issues.push(ValidationIssue::new(
                code,
                format!("profile non_claims are missing required boundary `{required}`"),
            ));
        }
    }
    issues.sort();
    issues.dedup();
    issues
}

fn validate_dispositions(profile: &KamaciteEffectProfile, issues: &mut Vec<ValidationIssue>) {
    let code = DIAGNOSTIC_PROFILE_INVALID;
    let mut seen = BTreeSet::new();
    for rule in &profile.ambient_dispositions {
        if !seen.insert(rule.effect) {
            issues.push(ValidationIssue::new(
                code,
                format!("ambient effect `{}` has more than one disposition", rule.effect.as_str()),
            ));
        }
    }
    let expected = AMBIENT_EFFECT_ORDER.iter().copied().collect::<BTreeSet<_>>();
    if seen != expected {
        issues.push(ValidationIssue::new(code, "ambient_dispositions must list every effect exactly once"));
    }
    for rule in &profile.ambient_dispositions {
        let allowed_profile_local = rule.effect == Effect::Alloc;
        match (rule.effect, rule.disposition) {
            (Effect::Alloc, AmbientDisposition::ProfileLocal) => {}
            (_, AmbientDisposition::ProfileLocal) => issues.push(ValidationIssue::new(
                code,
                format!("only `alloc` may be profile-local, got `{}`", rule.effect.as_str()),
            )),
            (_, AmbientDisposition::Denied) if !allowed_profile_local => {}
            (_, AmbientDisposition::Denied) => {
                issues.push(ValidationIssue::new(code, "`alloc` must remain profile-local in the v1 mapping profile"));
            }
        }
    }
}

pub fn validate_projection(projection: &KamaciteEffectSummaryProjection) -> Vec<ValidationIssue> {
    let code = DIAGNOSTIC_PROJECTION_INVALID;
    let mut issues = Vec::new();
    if projection.schema_version != PROJECTION_SCHEMA_VERSION {
        issues.push(ValidationIssue::new(
            code,
            format!("schema_version must be `{PROJECTION_SCHEMA_VERSION}`, got `{}`", projection.schema_version),
        ));
    }
    if projection.kamacite_schema_version != EFFECT_SUMMARY_SCHEMA {
        issues.push(ValidationIssue::new(
            code,
            format!(
                "kamacite_schema_version must be `{EFFECT_SUMMARY_SCHEMA}`, got `{}`",
                projection.kamacite_schema_version
            ),
        ));
    }
    if projection.preserves_profile_schema != PRESERVES_PROFILE_SCHEMA {
        issues
            .push(ValidationIssue::new(code, "preserves_profile_schema does not match the supported Kamacite profile"));
    }
    validate_identity(
        "preserves_profile_identity",
        &projection.preserves_profile_identity,
        PRESERVES_PROFILE_DOMAIN,
        code,
        &mut issues,
    );
    validate_identity(
        "effect_summary_identity",
        &projection.effect_summary_identity,
        EFFECT_SUMMARY_DOMAIN,
        code,
        &mut issues,
    );
    if !projection.effect_summary_closed {
        issues.push(ValidationIssue::new(
            DIAGNOSTIC_SUMMARY_NOT_CLOSED,
            "the supplied Kamacite effect summary is open; unknown operations cannot be treated as pure",
        ));
    }
    let mut operation_ids = BTreeSet::new();
    for operation in &projection.operations {
        if !operation_ids.insert(&operation.operation_identity) {
            issues.push(ValidationIssue::new(
                DIAGNOSTIC_AMBIGUOUS_EMISSION,
                "closed effect summary declares the same operation identity more than once",
            ));
        }
        validate_identity(
            "operation_identity",
            &operation.operation_identity,
            EFFECT_OPERATION_DOMAIN,
            code,
            &mut issues,
        );
        validate_identity(
            "effect_family_identity",
            &operation.effect_family_identity,
            EFFECT_SUMMARY_DOMAIN,
            code,
            &mut issues,
        );
        if operation.display_name.trim().is_empty() {
            issues.push(ValidationIssue::new(code, "operation display_name must not be empty"));
        }
        if operation.non_claims.is_empty() {
            issues.push(ValidationIssue::new(code, "operation requires explicit non-claims"));
        }
    }
    if projection.non_claims.is_empty() {
        issues.push(ValidationIssue::new(code, "non_claims must not be empty"));
    }
    issues.extend(validate_claim_boundary(&projection.non_claims.join("\n")));
    issues.sort();
    issues.dedup();
    issues
}

fn is_workspace_relative(path: &str) -> bool {
    !path.trim().is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.split(['/', '\\']).any(|component| component == "..")
}

// r[impl octet.effect_closure.kamacite_comparison]
/// Compares verified closure evidence and reviewed emissions with one supplied
/// Kamacite projection. The result never rewrites a parent artifact.
pub fn compare(
    receipt: &EffectClosureReceipt,
    profile: &KamaciteEffectProfile,
    projection: &KamaciteEffectSummaryProjection,
) -> KamaciteEffectComparison {
    let mut rows = Vec::new();
    let mut diagnostics = Vec::new();

    let receipt_diagnostics = effect_closure::verify_receipt_identity(receipt);
    if !receipt_diagnostics.is_empty() {
        diagnostics.push(ComparisonDiagnostic {
            code: DIAGNOSTIC_CLOSURE_INVALID.to_owned(),
            class: ComparisonClass::Unknown,
            message: receipt_diagnostics.join("; "),
            function_address: None,
            call_site_id: None,
            operation_identity: None,
        });
    }
    for issue in validate_profile(profile) {
        diagnostics.push(comparison_diagnostic(issue, ComparisonClass::Unknown));
    }
    for issue in validate_projection(projection) {
        diagnostics.push(comparison_diagnostic(issue, ComparisonClass::Unknown));
    }
    if receipt.status != AdmissionStatus::Admitted {
        diagnostics.push(ComparisonDiagnostic {
            code: DIAGNOSTIC_CLOSURE_NOT_ADMITTED.to_owned(),
            class: ComparisonClass::Unknown,
            message: "effect closure is not admitted; semantic-emission comparison cannot be admitted".to_owned(),
            function_address: None,
            call_site_id: None,
            operation_identity: None,
        });
    }
    validate_binding(profile, projection, &mut diagnostics);
    collect_ambient_rows(receipt, profile, &mut rows, &mut diagnostics);
    if receipt.status == AdmissionStatus::Admitted {
        let emitted = collect_emission_rows(receipt, profile, projection, &mut rows, &mut diagnostics);
        collect_declared_rows(projection, &emitted, &mut rows, &mut diagnostics);
    }

    rows.sort();
    rows.dedup();
    diagnostics.sort();
    diagnostics.dedup();
    let admitted_rows = rows.iter().all(|row| row.class.is_admitted());
    let status = if receipt.status == AdmissionStatus::Admitted && diagnostics.is_empty() && admitted_rows {
        AdmissionStatus::Admitted
    } else {
        AdmissionStatus::Denied
    };
    let mut non_claims = profile.non_claims.iter().chain(projection.non_claims.iter()).cloned().collect::<Vec<_>>();
    non_claims.extend(REQUIRED_HANDOFF_NON_CLAIMS.iter().map(|value| (*value).to_owned()));
    non_claims.sort();
    non_claims.dedup();
    let mut comparison = KamaciteEffectComparison {
        schema_version: COMPARISON_SCHEMA_VERSION.to_owned(),
        status,
        claim_scope: CLAIM_SCOPE.to_owned(),
        producer_name: OCTET_PRODUCER_NAME.to_owned(),
        producer_role: OCTET_PRODUCER_ROLE.to_owned(),
        producer_tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        profile_id: profile.profile_id.clone(),
        mapping_id: profile.mapping_id.clone(),
        mapping_profile_identity: profile_identity(profile),
        closure_receipt_identity: receipt.receipt_identity.clone(),
        projection_identity: projection_identity(projection),
        preserves_profile_schema: projection.preserves_profile_schema.clone(),
        preserves_profile_identity: projection.preserves_profile_identity.clone(),
        effect_summary_schema: projection.kamacite_schema_version.clone(),
        effect_summary_identity: projection.effect_summary_identity.clone(),
        rows,
        diagnostics,
        non_claims,
        comparison_identity: String::new(),
    };
    let computed = comparison_identity(&comparison);
    comparison.comparison_identity = computed;
    comparison
}

fn comparison_diagnostic(issue: ValidationIssue, class: ComparisonClass) -> ComparisonDiagnostic {
    ComparisonDiagnostic {
        code: issue.code.to_owned(),
        class,
        message: issue.message,
        function_address: None,
        call_site_id: None,
        operation_identity: None,
    }
}

fn validate_binding(
    profile: &KamaciteEffectProfile,
    projection: &KamaciteEffectSummaryProjection,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) {
    let mut stale = |message: String| {
        diagnostics.push(ComparisonDiagnostic {
            code: DIAGNOSTIC_BINDING_STALE.to_owned(),
            class: ComparisonClass::StaleIdentity,
            message,
            function_address: None,
            call_site_id: None,
            operation_identity: None,
        });
    };
    if profile.kamacite.preserves_profile_schema != projection.preserves_profile_schema
        || profile.kamacite.preserves_profile_identity != projection.preserves_profile_identity
    {
        stale("profile preserves-profile identity does not match the supplied Kamacite projection".to_owned());
    }
    if profile.kamacite.effect_summary_schema != projection.kamacite_schema_version
        || profile.kamacite.effect_summary_identity != projection.effect_summary_identity
    {
        stale("profile effect-summary identity does not match the supplied Kamacite projection".to_owned());
    }
}

// r[impl octet.effect_closure.kamacite_dispositions]
fn collect_ambient_rows(
    receipt: &EffectClosureReceipt,
    profile: &KamaciteEffectProfile,
    rows: &mut Vec<ComparisonRow>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) {
    let dispositions = profile
        .ambient_dispositions
        .iter()
        .map(|rule| (rule.effect, rule.disposition))
        .collect::<BTreeMap<_, _>>();
    for node in &receipt.nodes {
        for effect in &node.closure_effects {
            let row = |class: ComparisonClass| ComparisonRow {
                class,
                effect: Some(*effect),
                function_address: Some(node.function_address.clone()),
                path: Some(node.path.clone()),
                call_site_id: None,
                provider: None,
                operation_identity: None,
                detail: Some(format!("ambient effect `{}`", effect.as_str())),
            };
            match dispositions.get(effect).copied() {
                Some(AmbientDisposition::ProfileLocal) => rows.push(row(ComparisonClass::ProfileLocal)),
                Some(AmbientDisposition::Denied) if *effect != Effect::Unknown => {
                    rows.push(row(ComparisonClass::Unknown));
                    diagnostics.push(ComparisonDiagnostic {
                        code: DIAGNOSTIC_AMBIENT_DENIED.to_owned(),
                        class: ComparisonClass::Unknown,
                        message: format!(
                            "ambient effect `{}` remains denied and cannot become a semantic operation",
                            effect.as_str()
                        ),
                        function_address: Some(node.function_address.clone()),
                        call_site_id: None,
                        operation_identity: None,
                    });
                }
                Some(AmbientDisposition::Denied) | None => {
                    rows.push(row(ComparisonClass::Unknown));
                    diagnostics.push(ComparisonDiagnostic {
                        code: DIAGNOSTIC_UNMAPPED_AMBIENT.to_owned(),
                        class: ComparisonClass::Unknown,
                        message: format!("ambient effect `{}` has no supported mapping disposition", effect.as_str()),
                        function_address: Some(node.function_address.clone()),
                        call_site_id: None,
                        operation_identity: None,
                    });
                }
            }
        }
    }
}

fn collect_emission_rows(
    receipt: &EffectClosureReceipt,
    profile: &KamaciteEffectProfile,
    projection: &KamaciteEffectSummaryProjection,
    rows: &mut Vec<ComparisonRow>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) -> BTreeSet<KamaciteIdentity> {
    let declared = projection
        .operations
        .iter()
        .map(|operation| operation.operation_identity.clone())
        .collect::<BTreeSet<_>>();
    let mut nodes = BTreeMap::<&str, Vec<&effect_closure::FunctionClosureNode>>::new();
    for node in &receipt.nodes {
        nodes.entry(node.function_address.as_str()).or_default().push(node);
    }
    let site_key = |rule: &SemanticEmissionRule| (rule.function_address.clone(), rule.call_site_id.clone());
    let mut site_counts = BTreeMap::<_, usize>::new();
    for rule in &profile.semantic_emissions {
        *site_counts.entry(site_key(rule)).or_default() += 1;
    }

    let mut emitted = BTreeSet::new();
    for rule in &profile.semantic_emissions {
        let key = site_key(rule);
        if site_counts.get(&key).copied().unwrap_or_default() > 1 {
            push_ambiguous(rows, diagnostics, rule, "one emission site matches multiple semantic-emission rules");
            continue;
        }
        let Some(candidates) = nodes.get(rule.function_address.as_str()) else {
            push_stale(rows, diagnostics, rule, "semantic-emission function address is absent from closure evidence");
            continue;
        };
        if candidates.len() > 1 {
            push_ambiguous(
                rows,
                diagnostics,
                rule,
                "semantic-emission function address matches more than one closure node",
            );
            continue;
        }
        let node = candidates[0];
        if let Some(call_site_id) = &rule.call_site_id {
            let Some(call) = node.calls.iter().find(|call| &call.call_site_id == call_site_id) else {
                push_stale(
                    rows,
                    diagnostics,
                    rule,
                    "semantic-emission call-site identity is absent from the named function",
                );
                continue;
            };
            if call.resolution != effect_closure::CallResolution::ReviewedSummary {
                push_ambiguous(
                    rows,
                    diagnostics,
                    rule,
                    "semantic-emission call site is not an exact reviewed-summary edge",
                );
                continue;
            }
            if call.provider.as_ref() != Some(&rule.provider) {
                push_stale(
                    rows,
                    diagnostics,
                    rule,
                    "semantic-emission provider identity does not match the reviewed call",
                );
                continue;
            }
        } else if !receipt.providers.contains(&rule.provider) {
            push_stale(
                rows,
                diagnostics,
                rule,
                "function-scoped semantic-emission provider identity is absent from closure evidence",
            );
            continue;
        }
        emitted.insert(rule.operation_identity.clone());
        if declared.contains(&rule.operation_identity) {
            rows.push(emission_row(ComparisonClass::EmittedAndDeclared, rule, node, None));
        } else {
            rows.push(emission_row(
                ComparisonClass::EmittedNotDeclared,
                rule,
                node,
                Some("operation identity is absent from the supplied closed effect summary".to_owned()),
            ));
            diagnostics.push(ComparisonDiagnostic {
                code: DIAGNOSTIC_EMITTED_NOT_DECLARED.to_owned(),
                class: ComparisonClass::EmittedNotDeclared,
                message: "emitted semantic operation is absent from the supplied closed effect summary".to_owned(),
                function_address: Some(rule.function_address.clone()),
                call_site_id: rule.call_site_id.clone(),
                operation_identity: Some(rule.operation_identity.clone()),
            });
        }
    }
    emitted
}

fn emission_row(
    class: ComparisonClass,
    rule: &SemanticEmissionRule,
    node: &effect_closure::FunctionClosureNode,
    detail: Option<String>,
) -> ComparisonRow {
    ComparisonRow {
        class,
        effect: None,
        function_address: Some(rule.function_address.clone()),
        path: Some(node.path.clone()),
        call_site_id: rule.call_site_id.clone(),
        provider: Some(rule.provider.clone()),
        operation_identity: Some(rule.operation_identity.clone()),
        detail,
    }
}

fn push_stale(
    rows: &mut Vec<ComparisonRow>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
    rule: &SemanticEmissionRule,
    message: &str,
) {
    rows.push(ComparisonRow {
        class: ComparisonClass::StaleIdentity,
        effect: None,
        function_address: Some(rule.function_address.clone()),
        path: None,
        call_site_id: rule.call_site_id.clone(),
        provider: Some(rule.provider.clone()),
        operation_identity: Some(rule.operation_identity.clone()),
        detail: Some(message.to_owned()),
    });
    diagnostics.push(ComparisonDiagnostic {
        code: DIAGNOSTIC_STALE_EMISSION.to_owned(),
        class: ComparisonClass::StaleIdentity,
        message: message.to_owned(),
        function_address: Some(rule.function_address.clone()),
        call_site_id: rule.call_site_id.clone(),
        operation_identity: Some(rule.operation_identity.clone()),
    });
}

fn push_ambiguous(
    rows: &mut Vec<ComparisonRow>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
    rule: &SemanticEmissionRule,
    message: &str,
) {
    rows.push(ComparisonRow {
        class: ComparisonClass::AmbiguousEmission,
        effect: None,
        function_address: Some(rule.function_address.clone()),
        path: None,
        call_site_id: rule.call_site_id.clone(),
        provider: Some(rule.provider.clone()),
        operation_identity: Some(rule.operation_identity.clone()),
        detail: Some(message.to_owned()),
    });
    diagnostics.push(ComparisonDiagnostic {
        code: DIAGNOSTIC_AMBIGUOUS_EMISSION.to_owned(),
        class: ComparisonClass::AmbiguousEmission,
        message: message.to_owned(),
        function_address: Some(rule.function_address.clone()),
        call_site_id: rule.call_site_id.clone(),
        operation_identity: Some(rule.operation_identity.clone()),
    });
}

fn collect_declared_rows(
    projection: &KamaciteEffectSummaryProjection,
    emitted: &BTreeSet<KamaciteIdentity>,
    rows: &mut Vec<ComparisonRow>,
    diagnostics: &mut Vec<ComparisonDiagnostic>,
) {
    let mut seen = BTreeSet::new();
    for operation in &projection.operations {
        if !seen.insert(operation.operation_identity.clone()) {
            diagnostics.push(ComparisonDiagnostic {
                code: DIAGNOSTIC_AMBIGUOUS_EMISSION.to_owned(),
                class: ComparisonClass::AmbiguousEmission,
                message: "closed effect summary declares the same operation identity more than once".to_owned(),
                function_address: None,
                call_site_id: None,
                operation_identity: Some(operation.operation_identity.clone()),
            });
            continue;
        }
        if !emitted.contains(&operation.operation_identity) {
            rows.push(ComparisonRow {
                class: ComparisonClass::DeclaredNotEmitted,
                effect: None,
                function_address: None,
                path: None,
                call_site_id: None,
                provider: None,
                operation_identity: Some(operation.operation_identity.clone()),
                detail: Some(format!(
                    "declared operation `{}` has no selected Octet emission site; this is not a dead-code claim",
                    operation.display_name
                )),
            });
        }
    }
}

// r[impl octet.effect_closure.kamacite_handoff]
/// Builds the non-authoritative handoff for an admitted comparison.
pub fn build_handoff(comparison: &KamaciteEffectComparison) -> Result<KamaciteEffectHandoff, String> {
    if comparison.status != AdmissionStatus::Admitted {
        return Err("Kamacite effect handoff requires an admitted comparison".to_owned());
    }
    if !comparison.diagnostics.is_empty() || !comparison.rows.iter().all(|row| row.class.is_admitted()) {
        return Err("Kamacite effect handoff requires only admitted comparison rows and no diagnostics".to_owned());
    }
    let mut non_claims = comparison
        .non_claims
        .iter()
        .cloned()
        .chain(REQUIRED_HANDOFF_NON_CLAIMS.iter().map(|value| (*value).to_owned()))
        .collect::<Vec<_>>();
    non_claims.sort();
    non_claims.dedup();
    let mut handoff = KamaciteEffectHandoff {
        schema_version: HANDOFF_SCHEMA_VERSION.to_owned(),
        producer: HandoffProducer {
            name: OCTET_PRODUCER_NAME.to_owned(),
            role: OCTET_PRODUCER_ROLE.to_owned(),
            tool_version: comparison.producer_tool_version.clone(),
        },
        canonical_authority: KAMACITE_CANONICAL_IDENTITY_OWNER.to_owned(),
        json_projection_role: JSON_PROJECTION_ROLE.to_owned(),
        canonicalization_owner: CANONICALIZATION_OWNER.to_owned(),
        claim_scope: CLAIM_SCOPE.to_owned(),
        profile_id: comparison.profile_id.clone(),
        mapping_id: comparison.mapping_id.clone(),
        mapping_profile_identity: comparison.mapping_profile_identity.clone(),
        closure_receipt_identity: comparison.closure_receipt_identity.clone(),
        projection_identity: comparison.projection_identity.clone(),
        preserves_profile_schema: comparison.preserves_profile_schema.clone(),
        preserves_profile_identity: comparison.preserves_profile_identity.clone(),
        effect_summary_schema: comparison.effect_summary_schema.clone(),
        effect_summary_identity: comparison.effect_summary_identity.clone(),
        rows: comparison.rows.clone(),
        diagnostics: comparison.diagnostics.clone(),
        semantic_claims: Vec::new(),
        non_claims,
        handoff_identity: String::new(),
    };
    let computed = handoff_identity(&handoff);
    handoff.handoff_identity = computed;
    Ok(handoff)
}

/// Verifies handoff structure, role boundary, and BLAKE3 self-identity.
pub fn validate_handoff(handoff: &KamaciteEffectHandoff) -> Vec<ValidationIssue> {
    let code = DIAGNOSTIC_HANDOFF_INVALID;
    let mut issues = Vec::new();
    if handoff.schema_version != HANDOFF_SCHEMA_VERSION {
        issues.push(ValidationIssue::new(
            code,
            format!("schema_version must be `{HANDOFF_SCHEMA_VERSION}`, got `{}`", handoff.schema_version),
        ));
    }
    if handoff.producer.name != OCTET_PRODUCER_NAME || handoff.producer.role != OCTET_PRODUCER_ROLE {
        issues.push(ValidationIssue::new(
            DIAGNOSTIC_OVERCLAIM,
            "handoff producer role is not the bounded Octet source-effect evidence producer",
        ));
    }
    if handoff.producer.tool_version.trim().is_empty() {
        issues.push(ValidationIssue::new(code, "producer tool_version must not be empty"));
    }
    if handoff.canonical_authority != KAMACITE_CANONICAL_IDENTITY_OWNER
        || handoff.canonicalization_owner != CANONICALIZATION_OWNER
    {
        issues.push(ValidationIssue::new(
            DIAGNOSTIC_OVERCLAIM,
            "handoff must delegate canonical identity to Kamacite Preserves bytes",
        ));
    }
    if handoff.json_projection_role != JSON_PROJECTION_ROLE {
        issues
            .push(ValidationIssue::new(DIAGNOSTIC_OVERCLAIM, "handoff JSON projection must remain non-authoritative"));
    }
    if handoff.claim_scope != CLAIM_SCOPE {
        issues.push(ValidationIssue::new(code, format!("claim_scope must be `{CLAIM_SCOPE}`")));
    }
    if handoff.profile_id != DETERMINISTIC_CORE_PROFILE {
        issues.push(ValidationIssue::new(code, "handoff profile_id must be `deterministic-core`"));
    }
    for (label, identity) in [
        ("mapping_profile_identity", &handoff.mapping_profile_identity),
        ("closure_receipt_identity", &handoff.closure_receipt_identity),
        ("projection_identity", &handoff.projection_identity),
    ] {
        if !is_blake3_identity(identity) {
            issues.push(ValidationIssue::new(code, format!("{label} is not a full BLAKE3 identity")));
        }
    }
    validate_identity(
        "preserves_profile_identity",
        &handoff.preserves_profile_identity,
        PRESERVES_PROFILE_DOMAIN,
        code,
        &mut issues,
    );
    validate_identity(
        "effect_summary_identity",
        &handoff.effect_summary_identity,
        EFFECT_SUMMARY_DOMAIN,
        code,
        &mut issues,
    );
    if !handoff.semantic_claims.is_empty() {
        issues.push(ValidationIssue::new(DIAGNOSTIC_OVERCLAIM, "handoff must not carry semantic claims"));
    }
    if !handoff.diagnostics.is_empty() {
        issues.push(ValidationIssue::new(code, "admitted handoff must not carry diagnostics"));
    }
    if !is_sorted_unique(&handoff.rows) {
        issues.push(ValidationIssue::new(code, "handoff rows must be canonically sorted and unique"));
    }
    if handoff.rows.iter().any(|row| !row.class.is_admitted()) {
        issues.push(ValidationIssue::new(code, "handoff rows must contain only admitted comparison classes"));
    }
    if handoff.non_claims.is_empty() {
        issues.push(ValidationIssue::new(code, "handoff non_claims must not be empty"));
    }
    issues.extend(validate_claim_boundary(&handoff.non_claims.join("\n")));
    let expected = handoff_identity(handoff);
    if handoff.handoff_identity != expected {
        issues.push(ValidationIssue::new(
            DIAGNOSTIC_HANDOFF_IDENTITY,
            format!("expected `{expected}`, got `{}`", handoff.handoff_identity),
        ));
    }
    issues.sort();
    issues.dedup();
    issues
}

/// Re-derives the handoff from stored inputs and rejects any parent-identity
/// edit, including one that recomputes the unkeyed handoff identity.
pub fn verify_handoff_against(
    handoff: &KamaciteEffectHandoff,
    comparison: &KamaciteEffectComparison,
) -> Vec<ValidationIssue> {
    let mut issues = validate_handoff(handoff);
    match build_handoff(comparison) {
        Ok(expected) => match (canonical_json(handoff), canonical_json(&expected)) {
            (Ok(actual), Ok(expected)) if actual == expected => {}
            (Ok(_), Ok(_)) => issues.push(ValidationIssue::new(
                DIAGNOSTIC_HANDOFF_PARENT,
                "stored handoff does not match the handoff re-derived from its closure, profile, projection, and comparison parents",
            )),
            (Err(error), _) | (_, Err(error)) => issues.push(ValidationIssue::new(code_of(&error), error)),
        },
        Err(error) => issues.push(ValidationIssue::new(DIAGNOSTIC_HANDOFF_INVALID, error)),
    }
    issues.sort();
    issues.dedup();
    issues
}

fn code_of(error: &str) -> &'static str {
    if error.starts_with("serialize JSON") {
        DIAGNOSTIC_HANDOFF_INVALID
    } else {
        DIAGNOSTIC_HANDOFF_PARENT
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::effect_closure::ClosureCallEdge;
    use crate::effect_closure::ClosureInputIdentities;
    use crate::effect_closure::ClosureSourceIdentity;
    use crate::effect_closure::FunctionClosureNode;
    use crate::effect_closure::ModuleRole;
    use crate::effect_closure::ProviderKind;

    const TEST_PATH: &str = "src/lib.rs";
    const TEST_FUNCTION: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    fn test_call_site() -> String {
        effect_closure::call_site_identity(TEST_FUNCTION, TEST_PATH, "module=self", "call:emit", 0)
    }
    const TEST_EVIDENCE: &str = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
    const TEST_SOURCE: &str = "b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
    const TEST_HASH: &str = "b3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee";
    const OP_ONE: &str = "7fa2709c205ccfa8c4b8773ab97396193a5130a2e42b2f94916802b04e07f5f8";
    const OP_TWO: &str = "c598d03de53f54b01e8bc368840e6394e5cc3334b4e1be118f563ddfb8cfba2a";

    fn identity_with(domain: &str, hex: &str) -> KamaciteIdentity {
        KamaciteIdentity {
            domain: domain.to_owned(),
            algorithm: "blake3".to_owned(),
            hex: hex.to_owned(),
        }
    }

    fn operation_identity(hex: &str) -> KamaciteIdentity {
        identity_with(EFFECT_OPERATION_DOMAIN, hex)
    }

    fn family_identity(hex: &str) -> KamaciteIdentity {
        identity_with(EFFECT_SUMMARY_DOMAIN, hex)
    }

    fn provider() -> ProviderIdentity {
        ProviderIdentity {
            kind: ProviderKind::Workspace,
            name: "octet".to_owned(),
            version: "0.1.0".to_owned(),
            source: "workspace".to_owned(),
            target: "x86_64-unknown-linux-gnu".to_owned(),
            features: Vec::new(),
        }
    }

    fn evidence() -> EvidenceReference {
        EvidenceReference {
            kind: "reviewed-semantic-emission-v1".to_owned(),
            locator: "src/lib.rs#root".to_owned(),
            blake3: TEST_EVIDENCE.to_owned(),
        }
    }

    fn emission(function_address: &str, call_site_id: Option<&str>, operation: &str) -> SemanticEmissionRule {
        SemanticEmissionRule {
            function_address: function_address.to_owned(),
            call_site_id: call_site_id.map(str::to_owned),
            provider: provider(),
            operation_identity: operation_identity(operation),
            evidence: evidence(),
            non_claims: vec!["This reviewed emission does not prove runtime reachability.".to_owned()],
        }
    }

    fn profile(emissions: Vec<SemanticEmissionRule>) -> KamaciteEffectProfile {
        KamaciteEffectProfile {
            schema_version: PROFILE_SCHEMA_VERSION.to_owned(),
            profile_id: DETERMINISTIC_CORE_PROFILE.to_owned(),
            mapping_id: "octet-kamacite-effect-mapping-v1".to_owned(),
            kamacite: KamaciteBinding {
                preserves_profile_schema: PRESERVES_PROFILE_SCHEMA.to_owned(),
                preserves_profile_identity: identity_with(PRESERVES_PROFILE_DOMAIN, "1ae88fe233cb3df52e2e9438ae6b910c2d83b2e35025cfb99b3eda4f94370ece"),
                effect_summary_schema: EFFECT_SUMMARY_SCHEMA.to_owned(),
                effect_summary_identity: family_identity("64471a04261cbb5f5d9e9b3df5ba9f2454145535355601760e779d2f6b0a46ac"),
                projection: "kamacite-effect-summary.json".to_owned(),
                projection_blake3: TEST_HASH.to_owned(),
            },
            ambient_dispositions: AMBIENT_EFFECT_ORDER
                .iter()
                .map(|effect| AmbientDispositionRule {
                    effect: *effect,
                    disposition: if *effect == Effect::Alloc {
                        AmbientDisposition::ProfileLocal
                    } else {
                        AmbientDisposition::Denied
                    },
                })
                .collect(),
            semantic_emissions: emissions,
            non_claims: vec![
                "This mapping profile binds one exact frozen Kamacite compatibility projection; it does not prove complete Rust effects, semantic determinism, runtime reachability, handler correctness, replay truth, property satisfaction, or release eligibility.".to_owned(),
                "Octet does not canonicalize Kamacite data and this profile grants no consumer, publication, or runtime authority.".to_owned(),
            ],
        }
    }

    fn projection(operations: Vec<KamaciteOperation>, closed: bool) -> KamaciteEffectSummaryProjection {
        KamaciteEffectSummaryProjection {
            schema_version: PROJECTION_SCHEMA_VERSION.to_owned(),
            kamacite_schema_version: EFFECT_SUMMARY_SCHEMA.to_owned(),
            preserves_profile_schema: PRESERVES_PROFILE_SCHEMA.to_owned(),
            preserves_profile_identity: identity_with(
                PRESERVES_PROFILE_DOMAIN,
                "1ae88fe233cb3df52e2e9438ae6b910c2d83b2e35025cfb99b3eda4f94370ece",
            ),
            effect_summary_identity: family_identity(
                "64471a04261cbb5f5d9e9b3df5ba9f2454145535355601760e779d2f6b0a46ac",
            ),
            effect_summary_closed: closed,
            operations,
            non_claims: vec!["This frozen local projection is not a live Kamacite cohort.".to_owned()],
        }
    }

    fn operation(hex: &str, family: &str) -> KamaciteOperation {
        KamaciteOperation {
            operation_identity: operation_identity(hex),
            effect_family_identity: family_identity(family),
            display_name: format!("operation-{hex}"),
            non_claims: vec!["Operation identity does not prove handler behavior.".to_owned()],
        }
    }

    fn reviewed_call(effects: Vec<Effect>, call_site_id: &str) -> ClosureCallEdge {
        ClosureCallEdge {
            call_site_id: call_site_id.to_owned(),
            occurrence: 0,
            shape: "call:emit".to_owned(),
            scope: "module=self".to_owned(),
            resolution: effect_closure::CallResolution::ReviewedSummary,
            callee_function_address: None,
            provider: Some(provider()),
            effects,
            review_label: Some("reviewed semantic emission".to_owned()),
            evidence: Some(evidence()),
        }
    }

    fn receipt(effects: Vec<Effect>, call: Option<ClosureCallEdge>, admitted: bool) -> EffectClosureReceipt {
        let mut node = FunctionClosureNode {
            path: TEST_PATH.to_owned(),
            name: "root".to_owned(),
            function_address: TEST_FUNCTION.to_owned(),
            role: ModuleRole::Core,
            direct_effects: effects.clone(),
            closure_effects: effects,
            calls: call.into_iter().collect(),
        };
        node.calls.sort();
        let receipt = EffectClosureReceipt {
            schema_version: effect_closure::RECEIPT_SCHEMA_VERSION.to_owned(),
            status: if admitted {
                AdmissionStatus::Admitted
            } else {
                AdmissionStatus::Denied
            },
            claim_scope: effect_closure::CLAIM_SCOPE.to_owned(),
            resolver: "octet-selected-source-token-call-resolver-v1".to_owned(),
            canonicalizer: "octet-effect-closure-canonicalizer-v1".to_owned(),
            inputs: ClosureInputIdentities {
                profile_id: DETERMINISTIC_CORE_PROFILE.to_owned(),
                profile_hash: TEST_HASH.to_owned(),
                config_hash: TEST_HASH.to_owned(),
                command_blake3: TEST_HASH.to_owned(),
                check_status_blake3: TEST_HASH.to_owned(),
                function_address_artifact_blake3: TEST_HASH.to_owned(),
                function_address_record_root: TEST_HASH.to_owned(),
                workspace_policy_blake3: TEST_HASH.to_owned(),
                cargo_graph_blake3: TEST_HASH.to_owned(),
                cargo_lock_blake3: TEST_HASH.to_owned(),
                target: "x86_64-unknown-linux-gnu".to_owned(),
                toolchain: "test-toolchain".to_owned(),
            },
            sources: vec![ClosureSourceIdentity {
                path: TEST_PATH.to_owned(),
                source_identity: TEST_SOURCE.to_owned(),
            }],
            providers: if node.calls.is_empty() {
                Vec::new()
            } else {
                vec![provider()]
            },
            nodes: vec![node],
            diagnostics: if admitted {
                Vec::new()
            } else {
                vec![effect_closure::ClosureDiagnostic {
                    code: "closure_effect_denied".to_owned(),
                    message: "test denial".to_owned(),
                    path: Some(TEST_PATH.to_owned()),
                    function_address: Some(TEST_FUNCTION.to_owned()),
                    call_site_id: None,
                }]
            },
            non_claims: vec!["selected source-token closure only".to_owned()],
            receipt_identity: String::new(),
        };
        effect_closure::finalize_receipt(receipt)
    }

    fn exact_receipt() -> EffectClosureReceipt {
        receipt(vec![Effect::Alloc], Some(reviewed_call(vec![Effect::Alloc], &test_call_site())), true)
    }

    fn assert_issue(issues: &[ValidationIssue], code: &str) {
        assert!(issues.iter().any(|issue| issue.code == code), "missing `{code}` in {issues:?}");
    }

    // r[verify octet.effect_closure.kamacite_mapping]
    // r[verify octet.effect_closure.kamacite_comparison]
    #[test]
    fn exact_semantic_emission_is_emitted_and_declared() {
        let receipt = exact_receipt();
        let profile = profile(vec![emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE)]);
        let projection = projection(
            vec![operation(
                OP_ONE,
                "1111111111111111111111111111111111111111111111111111111111111111",
            )],
            true,
        );
        let comparison = compare(&receipt, &profile, &projection);
        assert_eq!(comparison.status, AdmissionStatus::Admitted, "{:?}", comparison.diagnostics);
        assert!(comparison.rows.iter().any(|row| row.class == ComparisonClass::EmittedAndDeclared
            && row.operation_identity.as_ref().is_some_and(|id| id.hex == OP_ONE)));
        assert!(
            comparison
                .rows
                .iter()
                .any(|row| row.class == ComparisonClass::ProfileLocal && row.effect == Some(Effect::Alloc))
        );
        let handoff = build_handoff(&comparison).expect("admitted handoff");
        assert_eq!(handoff.canonical_authority, KAMACITE_CANONICAL_IDENTITY_OWNER);
        assert!(validate_handoff(&handoff).is_empty());
    }

    // r[verify octet.effect_closure.kamacite_dispositions]
    #[test]
    fn profile_local_allocation_remains_visible() {
        let receipt = receipt(vec![Effect::Alloc], None, true);
        let comparison = compare(&receipt, &profile(Vec::new()), &projection(Vec::new(), true));
        assert_eq!(comparison.status, AdmissionStatus::Admitted);
        assert_eq!(comparison.rows.len(), 1);
        assert_eq!(comparison.rows[0].class, ComparisonClass::ProfileLocal);
        assert_eq!(comparison.rows[0].effect, Some(Effect::Alloc));
    }

    #[test]
    fn declared_not_emitted_is_visible_and_allowed() {
        let receipt = exact_receipt();
        let profile = profile(vec![emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE)]);
        let projection = projection(
            vec![
                operation(OP_ONE, "1111111111111111111111111111111111111111111111111111111111111111"),
                operation(OP_TWO, "2222222222222222222222222222222222222222222222222222222222222222"),
            ],
            true,
        );
        let comparison = compare(&receipt, &profile, &projection);
        assert_eq!(comparison.status, AdmissionStatus::Admitted, "{:?}", comparison.diagnostics);
        assert!(comparison.rows.iter().any(|row| row.class == ComparisonClass::DeclaredNotEmitted
            && row.operation_identity.as_ref().is_some_and(|id| id.hex == OP_TWO)));
    }

    // r[verify octet.effect_closure.kamacite_comparison]
    #[test]
    fn input_order_does_not_change_comparison_or_handoff_identity() {
        let receipt = exact_receipt();
        let first = profile(vec![
            emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE),
            emission(TEST_FUNCTION, None, OP_TWO),
        ]);
        let mut second = first.clone();
        second.semantic_emissions.reverse();
        second.ambient_dispositions.reverse();
        second.non_claims.reverse();
        let first_projection = projection(
            vec![
                operation(OP_ONE, "1111111111111111111111111111111111111111111111111111111111111111"),
                operation(OP_TWO, "2222222222222222222222222222222222222222222222222222222222222222"),
            ],
            true,
        );
        let mut second_projection = first_projection.clone();
        second_projection.operations.reverse();
        second_projection.non_claims.reverse();
        let first_comparison = compare(&receipt, &first, &first_projection);
        let second_comparison = compare(&receipt, &second, &second_projection);
        assert_eq!(first_comparison.comparison_identity, second_comparison.comparison_identity);
        assert_eq!(first_comparison.rows, second_comparison.rows);
        let first_handoff = build_handoff(&first_comparison).expect("handoff");
        let second_handoff = build_handoff(&second_comparison).expect("handoff");
        assert_eq!(first_handoff.handoff_identity, second_handoff.handoff_identity);
    }

    // r[verify octet.effect_closure.kamacite_dispositions]
    // r[verify octet.effect_closure.kamacite_boundary]
    #[test]
    fn ambient_clock_cannot_masquerade_as_a_semantic_emission() {
        let clock = receipt(vec![Effect::Clock], Some(reviewed_call(vec![Effect::Clock], &test_call_site())), false);
        let profile = profile(vec![emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE)]);
        let projection = projection(
            vec![operation(
                OP_ONE,
                "1111111111111111111111111111111111111111111111111111111111111111",
            )],
            true,
        );
        let comparison = compare(&clock, &profile, &projection);
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_AMBIENT_DENIED));
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_CLOSURE_NOT_ADMITTED));
        assert!(!comparison.rows.iter().any(|row| row.class == ComparisonClass::EmittedAndDeclared));
        assert!(build_handoff(&comparison).is_err());
    }

    #[test]
    fn unknown_ambient_effect_fails_with_unmapped_diagnostic() {
        let unknown = receipt(vec![Effect::Unknown], None, false);
        let comparison = compare(&unknown, &profile(Vec::new()), &projection(Vec::new(), true));
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_UNMAPPED_AMBIENT));
    }

    #[test]
    fn emitted_undeclared_operation_fails() {
        let receipt = exact_receipt();
        let profile = profile(vec![emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE)]);
        let projection = projection(Vec::new(), true);
        let comparison = compare(&receipt, &profile, &projection);
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_EMITTED_NOT_DECLARED));
    }

    #[test]
    fn stale_function_and_provider_identities_fail() {
        let receipt = exact_receipt();
        let stale_function = profile(vec![emission(
            "b3:9999999999999999999999999999999999999999999999999999999999999999",
            Some(&test_call_site()),
            OP_ONE,
        )]);
        let comparison = compare(&receipt, &stale_function, &projection(Vec::new(), true));
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_STALE_EMISSION));

        let mut stale_provider = emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE);
        stale_provider.provider.name = "other".to_owned();
        let comparison = compare(&receipt, &profile(vec![stale_provider]), &projection(Vec::new(), true));
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_STALE_EMISSION));
    }

    #[test]
    fn duplicate_emission_and_unresolved_site_are_ambiguous() {
        let base = exact_receipt();
        let duplicate = profile(vec![
            emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE),
            emission(TEST_FUNCTION, Some(&test_call_site()), OP_TWO),
        ]);
        let comparison = compare(&base, &duplicate, &projection(Vec::new(), true));
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_AMBIGUOUS_EMISSION));

        let mut local_call = reviewed_call(vec![Effect::Alloc], &test_call_site());
        local_call.resolution = effect_closure::CallResolution::Local;
        local_call.provider = None;
        let local = receipt(vec![Effect::Alloc], Some(local_call), true);
        let comparison = compare(
            &local,
            &profile(vec![emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE)]),
            &projection(Vec::new(), true),
        );
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_AMBIGUOUS_EMISSION));
    }

    #[test]
    fn open_summary_and_stale_binding_fail_closed() {
        let receipt = exact_receipt();
        let comparison = compare(&receipt, &profile(Vec::new()), &projection(Vec::new(), false));
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_SUMMARY_NOT_CLOSED));

        let mut stale = profile(Vec::new());
        stale.kamacite.effect_summary_identity =
            family_identity("3333333333333333333333333333333333333333333333333333333333333333");
        let comparison = compare(&receipt, &stale, &projection(Vec::new(), true));
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_BINDING_STALE));
    }

    #[test]
    fn profile_rejects_clock_profile_local_and_broad_scope() {
        let mut clock = profile(Vec::new());
        clock.ambient_dispositions = AMBIENT_EFFECT_ORDER
            .iter()
            .map(|effect| AmbientDispositionRule {
                effect: *effect,
                disposition: if matches!(*effect, Effect::Alloc | Effect::Clock) {
                    AmbientDisposition::ProfileLocal
                } else {
                    AmbientDisposition::Denied
                },
            })
            .collect();
        assert_issue(&validate_profile(&clock), DIAGNOSTIC_PROFILE_INVALID);

        let mut broad = profile(vec![emission("", None, OP_ONE)]);
        assert_issue(&validate_profile(&broad), DIAGNOSTIC_PROFILE_INVALID);
        broad.semantic_emissions[0].function_address = TEST_FUNCTION.to_owned();
        broad.semantic_emissions[0].operation_identity.domain = "semantic-effect".to_owned();
        assert_issue(&validate_profile(&broad), DIAGNOSTIC_PROFILE_INVALID);
    }

    #[test]
    fn profile_and_projection_reject_duplicates_and_overclaims() {
        let duplicate = profile(vec![
            emission(TEST_FUNCTION, None, OP_ONE),
            emission(TEST_FUNCTION, None, OP_ONE),
        ]);
        assert_issue(&validate_profile(&duplicate), DIAGNOSTIC_PROFILE_INVALID);

        let mut overclaim = profile(Vec::new());
        overclaim.non_claims.push("Octet comparison proves semantic determinism.".to_owned());
        assert_issue(&validate_profile(&overclaim), DIAGNOSTIC_OVERCLAIM);

        let duplicate_operations = projection(
            vec![
                operation(OP_ONE, "1111111111111111111111111111111111111111111111111111111111111111"),
                operation(OP_ONE, "2222222222222222222222222222222222222222222222222222222222222222"),
            ],
            true,
        );
        assert_issue(&validate_projection(&duplicate_operations), DIAGNOSTIC_AMBIGUOUS_EMISSION);
    }

    // r[verify octet.effect_closure.kamacite_handoff]
    // r[verify octet.effect_closure.kamacite_boundary]
    #[test]
    fn handoff_rejects_role_promotion_and_tampering() {
        let receipt = exact_receipt();
        let profile = profile(vec![emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE)]);
        let projection = projection(
            vec![operation(
                OP_ONE,
                "1111111111111111111111111111111111111111111111111111111111111111",
            )],
            true,
        );
        let comparison = compare(&receipt, &profile, &projection);
        let handoff = build_handoff(&comparison).expect("handoff");

        let mut promoted = handoff.clone();
        promoted.canonical_authority = "octet-canonicalizer".to_owned();
        assert_issue(&validate_handoff(&promoted), DIAGNOSTIC_OVERCLAIM);

        let mut claimed = handoff.clone();
        claimed.semantic_claims.push("comparison proves semantic determinism".to_owned());
        assert_issue(&validate_handoff(&claimed), DIAGNOSTIC_OVERCLAIM);

        let mut tampered = handoff.clone();
        tampered.rows[0].operation_identity = Some(operation_identity(OP_TWO));
        assert_issue(&validate_handoff(&tampered), DIAGNOSTIC_HANDOFF_IDENTITY);

        let mut reparented = handoff.clone();
        reparented.closure_receipt_identity =
            "b3:1111111111111111111111111111111111111111111111111111111111111111".to_owned();
        reparented.handoff_identity = handoff_identity(&reparented);
        assert!(validate_handoff(&reparented).is_empty());
        assert_issue(&verify_handoff_against(&reparented, &comparison), DIAGNOSTIC_HANDOFF_PARENT);
    }

    // r[verify octet.effect_closure.kamacite_boundary]
    #[test]
    fn claim_boundary_rejects_affirmative_overclaims_only() {
        assert!(validate_claim_boundary(
            "This comparison does not prove semantic determinism, runtime reachability, handler correctness, replay truth, or release eligibility."
        )
        .is_empty());
        assert_issue(&validate_claim_boundary("Octet comparison proves semantic determinism."), DIAGNOSTIC_OVERCLAIM);
        assert_issue(&validate_claim_boundary("Octet owns canonical Kamacite semantics."), DIAGNOSTIC_OVERCLAIM);
    }

    // r[verify octet.effect_closure.kamacite_mapping]
    // r[verify octet.effect_closure.kamacite_fixtures]
    #[test]
    fn shipped_profile_and_frozen_projection_are_bound() {
        let profile: KamaciteEffectProfile =
            serde_json::from_str(include_str!("../../standards/effect-closure/generated/kamacite-effect-profile.json"))
                .expect("parse shipped profile");
        let projection: KamaciteEffectSummaryProjection =
            serde_json::from_str(include_str!("../../standards/effect-closure/fixtures/kamacite/effect-summary.json"))
                .expect("parse frozen projection");
        assert!(validate_profile(&profile).is_empty(), "{:?}", validate_profile(&profile));
        assert!(validate_projection(&projection).is_empty(), "{:?}", validate_projection(&projection));
        assert_eq!(profile.kamacite.preserves_profile_identity, projection.preserves_profile_identity);
        assert_eq!(profile.kamacite.effect_summary_identity, projection.effect_summary_identity);
        let bytes = include_bytes!("../../standards/effect-closure/fixtures/kamacite/effect-summary.json");
        assert_eq!(profile.kamacite.projection_blake3, format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex()));
    }

    #[test]
    fn frozen_projection_drift_fails_binding() {
        let profile = profile(Vec::new());
        let mut drifted = projection(Vec::new(), true);
        drifted.effect_summary_identity =
            family_identity("4444444444444444444444444444444444444444444444444444444444444444");
        let receipt = receipt(vec![Effect::Alloc], None, true);
        let comparison = compare(&receipt, &profile, &drifted);
        assert_eq!(comparison.status, AdmissionStatus::Denied);
        assert!(comparison.diagnostics.iter().any(|diagnostic| diagnostic.code == DIAGNOSTIC_BINDING_STALE));
    }

    #[test]
    fn comparison_identity_is_sensitive_to_every_parent() {
        let receipt = exact_receipt();
        let base_profile = profile(vec![emission(TEST_FUNCTION, Some(&test_call_site()), OP_ONE)]);
        let base_projection = projection(
            vec![operation(
                OP_ONE,
                "1111111111111111111111111111111111111111111111111111111111111111",
            )],
            true,
        );
        let base = compare(&receipt, &base_profile, &base_projection);

        let mut changed_profile = base_profile.clone();
        changed_profile.mapping_id = "other-mapping".to_owned();
        assert_ne!(compare(&receipt, &changed_profile, &base_projection).comparison_identity, base.comparison_identity);

        let mut changed_projection = base_projection.clone();
        changed_projection.non_claims.push("A different frozen projection.".to_owned());
        assert_ne!(compare(&receipt, &base_profile, &changed_projection).comparison_identity, base.comparison_identity);

        let mut changed_receipt = receipt.clone();
        changed_receipt.inputs.target = "other-target".to_owned();
        changed_receipt = effect_closure::finalize_receipt(changed_receipt);
        assert_ne!(
            compare(&changed_receipt, &base_profile, &base_projection).comparison_identity,
            base.comparison_identity
        );
    }

    #[test]
    fn documentation_does_not_overclaim() {
        let documentation = include_str!("../../docs/effect-closure-admission.md");
        assert!(validate_claim_boundary(documentation).is_empty(), "{:?}", validate_claim_boundary(documentation));
    }
}
