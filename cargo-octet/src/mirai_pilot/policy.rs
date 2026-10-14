use std::collections::BTreeSet;
use std::path::Path;

use super::CORPUS_IDENTITY;
use super::EXPECTED_BLOCKER;
use super::EXPECTED_CHARON_PARENT;
use super::EXPECTED_COMMAND;
use super::EXPECTED_EFFECT_PARENT;
use super::EXPECTED_ENVIRONMENT;
use super::EXPECTED_FEATURES;
use super::EXPECTED_REPOSITORY;
use super::EXPECTED_REVISION;
use super::EXPECTED_SOURCE_HASH;
use super::EXPECTED_TOOLCHAIN;
use super::EXPECTED_VERSION;
use super::POLICY_IDENTITY;
use super::PROFILE_ID;
use super::model::PilotProfile;

const EXPECTED_LICENSE: &str = "MIT";
const EXPECTED_TARGET: &str = "x86_64-unknown-linux-gnu";
const EXPECTED_DERIVATION_ATTR: &str = "packages.x86_64-linux.mirai-source-provider";
const EXPECTED_DERIVATION_NAME: &str = "octet-mirai-source-provider-1.1.12";
const EXPECTED_EVIDENCE_TIER: &str = "transcript_assertion";
const EXPECTED_VERIFICATION_ROLE: &str = "recorded_only";
const EXPECTED_FIXTURE_COUNT: usize = 8;
const EXPECTED_SUPPORTED_COUNT: usize = 7;
const EXPECTED_MAX_FIXTURES: usize = 8;
const EXPECTED_MAX_SOURCE_BYTES: u64 = 4_096;
const EXPECTED_TIMEOUT_MS: u64 = 120_000;
const EXPECTED_MAX_LOG_BYTES: u64 = 1_048_576;
const REQUIRED_CLASSES: [&str; 7] = [
    "call-edge",
    "effect",
    "panic",
    "precondition",
    "safe",
    "support",
    "unsupported",
];
const REQUIRED_OUTCOMES: [&str; 6] = [
    "agree",
    "disagree",
    "mirai-only",
    "charon-only",
    "source-token-only",
    "unknown",
];
const REQUIRED_DECISIONS: [&str; 4] = ["adopt", "defer", "reject", "blocked"];
const EXPECTED_FIXTURES: [(&str, &str, &str, u64, &str, bool); 8] = [
    (
        "crate-manifest",
        "cargo-octet/tests/fixtures/mirai-pilot/Cargo.toml",
        "b3:7e291249f910d0a9ac9bb1b68ecf45dee40164388bd29707487b6c9a6c9ddfb4",
        124,
        "support",
        true,
    ),
    (
        "crate-root",
        "cargo-octet/tests/fixtures/mirai-pilot/src/lib.rs",
        "b3:d8b49406fd7e30b4f47e40a4e4381cd81b7430cc400fda7b7a8466121569bb30",
        112,
        "support",
        true,
    ),
    (
        "call-edge",
        "cargo-octet/tests/fixtures/mirai-pilot/src/call_edge.rs",
        "b3:a7d4153d3ded2a66d7e10fdcbfb419945d49eeb8ae9eb9e4ae24543a4b6660e1",
        115,
        "call-edge",
        true,
    ),
    (
        "effect-operation",
        "cargo-octet/tests/fixtures/mirai-pilot/src/effect.rs",
        "b3:fd7e4083059dc115c1a1f57d814cacbc31711296085b16942e5668a36f8f4eb5",
        112,
        "effect",
        true,
    ),
    (
        "reachable-panic",
        "cargo-octet/tests/fixtures/mirai-pilot/src/panic_path.rs",
        "b3:0a80e237f71f2b55c6fa5bcdaeda7dab4c054085b510f9335b6173e7af78fdc0",
        128,
        "panic",
        true,
    ),
    (
        "checked-precondition",
        "cargo-octet/tests/fixtures/mirai-pilot/src/precondition.rs",
        "b3:bafb910681fbeea8ae0094bd1fc3443255336257ba3869c734d4cf4562e0fd83",
        162,
        "precondition",
        true,
    ),
    (
        "safe-path",
        "cargo-octet/tests/fixtures/mirai-pilot/src/safe.rs",
        "b3:378d3f03bece29c1a5f67af9d26c87c1ea4b35650c4de4cc323580a7040392d7",
        81,
        "safe",
        true,
    ),
    (
        "macro-generated",
        "cargo-octet/tests/fixtures/mirai-pilot/src/unsupported.rs",
        "b3:f81b4ac527a8fe4350769457d3b10cba3e44029b7a27968b3ef16a2264886714",
        159,
        "unsupported",
        false,
    ),
];
const REQUIRED_CRITERIA: [(&str, u64); 8] = [
    ("supported-fixture-rate", 80),
    ("deterministic-replay-rate", 100),
    ("diagnostic-stability-rate", 100),
    ("unique-useful-findings", 1),
    ("false-positive-maximum", 0),
    ("runtime-within-budget", 100),
    ("maintenance-cost-accepted", 100),
    ("stable-machine-output-contract", 100),
];

pub(super) fn validate(profile: &PilotProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    validate_provider(profile, &mut diagnostics);
    validate_contract(profile, &mut diagnostics);
    validate_corpus(profile, &mut diagnostics);
    validate_decision_policy(profile, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_provider(profile: &PilotProfile, diagnostics: &mut Vec<String>) {
    let provider = &profile.provider;
    if profile.profile_id != PROFILE_ID
        || provider.repository != EXPECTED_REPOSITORY
        || provider.revision != EXPECTED_REVISION
        || provider.source_hash != EXPECTED_SOURCE_HASH
        || provider.license != EXPECTED_LICENSE
        || provider.version != EXPECTED_VERSION
        || provider.rust_toolchain != EXPECTED_TOOLCHAIN
        || provider.target != EXPECTED_TARGET
        || provider.features != EXPECTED_FEATURES
        || provider.derivation_attr != EXPECTED_DERIVATION_ATTR
        || provider.derivation_name != EXPECTED_DERIVATION_NAME
        || provider.command_argv != EXPECTED_COMMAND
        || provider.environment != EXPECTED_ENVIRONMENT
    {
        diagnostics.push("MIRAI pilot provider identity or fixed execution contract drifted".to_owned());
    }
    if provider.closure_components.len() != 2
        || provider.closure_components.iter().any(|component| component.trim().is_empty())
    {
        diagnostics.push("MIRAI pilot provider closure graph is incomplete".to_owned());
    }
}

fn validate_contract(profile: &PilotProfile, diagnostics: &mut Vec<String>) {
    if profile.output_contract.status != "blocked"
        || profile.output_contract.reason_code != EXPECTED_BLOCKER
        || profile.output_contract.stable_machine_schema
    {
        diagnostics.push("MIRAI pilot output-contract blocker drifted".to_owned());
    }
    if profile.evidence_tier != EXPECTED_EVIDENCE_TIER
        || profile.verification_role != EXPECTED_VERIFICATION_ROLE
        || profile.non_claims.len() < 4
    {
        diagnostics.push("MIRAI pilot evidence boundary is incomplete".to_owned());
    }
}

fn validate_corpus(profile: &PilotProfile, diagnostics: &mut Vec<String>) {
    if profile.corpus.len() != EXPECTED_FIXTURE_COUNT
        || profile.corpus.len() > profile.limits.max_fixtures
        || profile.corpus.iter().filter(|fixture| fixture.supported).count() != EXPECTED_SUPPORTED_COUNT
    {
        diagnostics.push("MIRAI pilot corpus size or support accounting drifted".to_owned());
    }
    let mut fixture_ids = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut classes = BTreeSet::new();
    for fixture in &profile.corpus {
        if fixture.fixture_id.trim().is_empty()
            || fixture.expected_observation.trim().is_empty()
            || unsafe_relative_path(&fixture.path)
            || fixture.bytes == 0
            || fixture.bytes > profile.limits.max_source_bytes
            || !super::is_b3_identity(&fixture.source_identity)
            || !fixture_ids.insert(&fixture.fixture_id)
            || !paths.insert(&fixture.path)
        {
            diagnostics.push("MIRAI pilot corpus contains an unsafe, empty, duplicate, or unbound fixture".to_owned());
        }
        if fixture.supported == fixture.unsupported_reason.is_empty() {
            classes.insert(fixture.property_class.as_str());
        } else {
            diagnostics.push("MIRAI pilot unsupported fixture label is inconsistent".to_owned());
        }
    }
    let required = REQUIRED_CLASSES.into_iter().collect::<BTreeSet<_>>();
    if classes != required {
        diagnostics.push("MIRAI pilot corpus property classes are incomplete".to_owned());
    }
    if profile.corpus.iter().zip(EXPECTED_FIXTURES).any(|(fixture, expected)| {
        fixture.fixture_id != expected.0
            || fixture.path != expected.1
            || fixture.source_identity != expected.2
            || fixture.bytes != expected.3
            || fixture.property_class != expected.4
            || fixture.supported != expected.5
    }) {
        diagnostics.push("MIRAI pilot corpus source identity or oracle selection drifted".to_owned());
    }
}

fn validate_decision_policy(profile: &PilotProfile, diagnostics: &mut Vec<String>) {
    let criteria = profile
        .promotion_criteria
        .iter()
        .map(|criterion| (criterion.criterion_id.as_str(), criterion.threshold))
        .collect::<Vec<_>>();
    if profile.comparison_outcomes != REQUIRED_OUTCOMES
        || profile.decision_states != REQUIRED_DECISIONS
        || criteria != REQUIRED_CRITERIA
        || profile.promotion_criteria.iter().any(|criterion| !criterion.required)
    {
        diagnostics.push("MIRAI pilot comparison or promotion policy is incomplete".to_owned());
    }
    if profile.comparison_parents.charon_profile != EXPECTED_CHARON_PARENT
        || profile.comparison_parents.source_token_catalog != EXPECTED_EFFECT_PARENT
        || !super::is_b3_identity(POLICY_IDENTITY)
        || !super::is_b3_identity(CORPUS_IDENTITY)
    {
        diagnostics.push("MIRAI pilot comparison parent or checked policy identity drifted".to_owned());
    }
    if profile.limits.max_fixtures != EXPECTED_MAX_FIXTURES
        || profile.limits.max_source_bytes != EXPECTED_MAX_SOURCE_BYTES
        || profile.limits.timeout_ms != EXPECTED_TIMEOUT_MS
        || profile.limits.max_log_bytes != EXPECTED_MAX_LOG_BYTES
    {
        diagnostics.push("MIRAI pilot resource limits drifted".to_owned());
    }
}

fn unsafe_relative_path(value: &str) -> bool {
    value.is_empty()
        || Path::new(value).is_absolute()
        || value.split('/').any(|component| component.is_empty() || component == "." || component == "..")
}
