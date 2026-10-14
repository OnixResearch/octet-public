use std::collections::BTreeSet;

use super::*;

const EXPECTED_COMMAND: [&str; 12] = [
    "cargo",
    "test",
    "--locked",
    "-p",
    "cargo-octet",
    "--lib",
    "hegel::tests::hegel_profile_adapter",
    "--",
    "--exact",
    "--nocapture",
    "--test-threads",
    "1",
];
const EXPECTED_EVIDENCE_TIER: &str = "transcript_assertion";
const EXPECTED_VERIFICATION_ROLE: &str = "recorded_only";

pub(super) fn validate(profile: &Profile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if profile.profile_id != PROFILE_ID
        || profile.producer.package != "cargo-octet"
        || profile.producer.package_version != env!("CARGO_PKG_VERSION")
        || profile.producer.hegel_version != "0.14.27"
        || profile.producer.backend != "native"
        || profile.property_catalog != LAW_CATALOG_SCHEMA
        || profile.command_argv.iter().map(String::as_str).collect::<Vec<_>>() != EXPECTED_COMMAND
    {
        diagnostics.push("Hegel producer profile identity or fixed command is unsupported".to_owned());
    }
    if profile.evidence_tier != EXPECTED_EVIDENCE_TIER
        || profile.verification_role != EXPECTED_VERIFICATION_ROLE
        || !profile.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM)
    {
        diagnostics.push("Hegel producer profile evidence boundary is unsupported".to_owned());
    }
    let paths = profile.source_scope.iter().chain(profile.generator_inputs.iter()).chain([
        &profile.output_root,
        &profile.summary_path,
        &profile.counterexample_path,
    ]);
    for path in paths {
        if unsafe_relative_path(path) {
            diagnostics.push("Hegel producer profile contains an unsafe path".to_owned());
        }
    }
    if profile.source_scope.is_empty()
        || profile.source_scope.len() > profile.limits.max_source_files
        || profile.generator_inputs.is_empty()
        || profile.generator_inputs.len() > profile.limits.max_generator_files
        || profile.settings.case_budget == 0
        || profile.limits.max_summary_bytes == 0
        || profile.limits.max_counterexample_bytes == 0
        || profile.limits.max_log_bytes == 0
        || profile.limits.max_issue_codes == 0
        || profile.limits.timeout_ms == 0
        || profile.limits.poll_interval_ms == 0
        || profile.limits.teardown_timeout_ms == 0
    {
        diagnostics.push("Hegel producer profile contains an invalid limit".to_owned());
    }
    diagnostics.extend(validate_catalog(profile));
    diagnostics
}

pub(super) fn unsafe_relative_path(path: &str) -> bool {
    path.is_empty()
        || std::path::Path::new(path).is_absolute()
        || path.split('/').any(|component| component.is_empty() || component == "." || component == "..")
}

fn validate_catalog(profile: &Profile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let catalog = match law_catalog() {
        Ok(value) => value,
        Err(error) => return vec![error],
    };
    if catalog.laws.is_empty() {
        diagnostics.push("Hegel law catalog is empty".to_owned());
    }
    let mut ids = BTreeSet::new();
    for law in catalog.laws {
        if law.law_id.trim().is_empty()
            || law.pure_core.trim().is_empty()
            || law.generator.trim().is_empty()
            || law.positive_property.trim().is_empty()
            || law.negative_property.trim().is_empty()
            || law.generated_input_benefit.trim().is_empty()
            || law.replay_method.trim().is_empty()
            || law.fixture_complement.trim().is_empty()
            || !law.effect_free
            || law.case_budget != profile.settings.case_budget
            || !ids.insert(law.law_id)
        {
            diagnostics.push("Hegel law catalog entry is incomplete, duplicated, or effectful".to_owned());
        }
    }
    diagnostics
}
