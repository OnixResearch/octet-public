use std::collections::BTreeSet;

use super::*;

const SUPPORTED_MODES: [&str; 4] = ["cold", "warm", "no_change", "incremental"];
const REQUIRED_METRICS: [&str; 2] = ["wall_duration_ns", "output_bytes"];
const EXPECTED_CORPUS_COUNT: usize = 2;
const REQUIRED_ENVIRONMENT_FACTS: [&str; 9] = [
    "host_os",
    "host_arch",
    "kernel_identity",
    "cpu_identity",
    "governor",
    "load_class",
    "rust_toolchain",
    "nix_closure_identity",
    "isolation",
];

pub(super) fn validate(profile: &Profile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if profile.profile_id != PROFILE_ID
        || profile.provider.package != "cargo-octet"
        || profile.provider.version != "0.1.0"
    {
        diagnostics.push("benchmark profile or provider identity is unsupported".to_owned());
    }
    if profile.evidence_tier != "transcript_assertion" || profile.non_claims.is_empty() {
        diagnostics.push("benchmark evidence tier or non-claims are incomplete".to_owned());
    }
    if profile.resource_observation.allocation_input_path.is_empty()
        || profile.resource_observation.llvm_input_path.is_empty()
        || profile.resource_observation.target.is_empty()
        || profile.resource_observation.optimization_profile.is_empty()
        || !resources::is_b3(&profile.resource_observation.compiler_identity)
    {
        diagnostics.push("benchmark resource observation profile is incomplete".to_owned());
    }
    if profile.comparison.minimum_samples == 0
        || profile.comparison.minimum_samples > profile.limits.max_samples
        || profile.workloads.len() != SUPPORTED_MODES.len()
    {
        diagnostics.push("benchmark profile sample limits or workload count are unsupported".to_owned());
    }
    validate_corpora(profile, &mut diagnostics);
    validate_workloads(profile, &mut diagnostics);
    diagnostics
}

fn validate_corpora(profile: &Profile, diagnostics: &mut Vec<String>) {
    let mut corpus_ids = BTreeSet::new();
    if profile.signal_corpus.len() != EXPECTED_CORPUS_COUNT {
        diagnostics.push("benchmark signal corpus count is unsupported".to_owned());
    }
    for corpus in &profile.signal_corpus {
        if corpus.corpus_id.is_empty()
            || corpus.source_repository.is_empty()
            || corpus.source_revision.is_empty()
            || corpus.representativeness_boundary.is_empty()
            || !matches!(corpus.source_kind.as_str(), "synthetic" | "cross_project")
            || !corpus_ids.insert(corpus.corpus_id.as_str())
        {
            diagnostics.push("benchmark signal corpus profile is incomplete".to_owned());
        }
    }
}

fn validate_workloads(profile: &Profile, diagnostics: &mut Vec<String>) {
    let mut modes = BTreeSet::new();
    let mut workload_ids = BTreeSet::new();
    for workload in &profile.workloads {
        if !SUPPORTED_MODES.contains(&workload.mode.as_str())
            || !modes.insert(workload.mode.as_str())
            || !workload_ids.insert(workload.workload_id.as_str())
            || workload.command_id != "cargo-metadata-no-deps"
            || workload.command_argv != ["cargo", "metadata", "--no-deps", "--format-version", "1"]
            || workload.sample_count < profile.comparison.minimum_samples
            || workload.sample_count > profile.limits.max_samples
            || !resources::is_b3(&workload.workload_identity)
            || !resources::is_b3(&workload.source_state_identity)
            || !resources::is_b3(&workload.environment_identity)
            || workload.required_environment_facts.len() != REQUIRED_ENVIRONMENT_FACTS.len()
            || !REQUIRED_ENVIRONMENT_FACTS
                .iter()
                .all(|fact| workload.required_environment_facts.iter().any(|required| required == fact))
            || !REQUIRED_METRICS
                .iter()
                .all(|metric| workload.required_metrics.iter().any(|required| required == metric))
        {
            diagnostics.push("benchmark profile contains an unsupported workload".to_owned());
        }
    }
}
