// r[impl octet.benchmark_evidence.public_claims]
use std::collections::BTreeSet;

use super::*;

const SUPPORTED_OUTCOMES: [&str; 5] = ["succeeded", "failed", "timed_out", "output_limit", "cancelled"];

pub(super) fn evaluate(input: &request::Input, profile: &Profile) -> Evaluation {
    let mut diagnostics = profile_validation::validate(profile);
    let workload = profile.workloads.iter().find(|workload| workload.workload_id == input.baseline.workload_id);
    let Some(workload) = workload else {
        diagnostics.push("benchmark baseline selects an unknown workload".to_owned());
        return failed(diagnostics);
    };
    validate_run(&input.baseline, workload, profile, "baseline", &mut diagnostics);
    validate_run(&input.candidate, workload, profile, "candidate", &mut diagnostics);
    if !resources::is_b3(&input.bounded_log_identity) {
        diagnostics.push("benchmark bounded log identity is malformed".to_owned());
    }
    let resources = match resources::summarize(&input.allocations, &input.copies) {
        Ok(value) => value,
        Err(errors) => {
            diagnostics.extend(errors);
            empty_resource_summary()
        }
    };
    let signals = match signal::classify(&input.oracle, &input.findings) {
        Ok(value) => value,
        Err(errors) => {
            diagnostics.extend(errors);
            empty_signal_summary()
        }
    };
    let performance = comparison::compare(&input.baseline, &input.candidate, profile.comparison);
    let projection = BenchmarkProjection {
        performance,
        resources,
        signals,
    };
    let projection_bytes = match projection_bytes(&projection) {
        Ok(value) => value,
        Err(error) => return failed(vec![error]),
    };
    let projection_identity = prefixed_identity(&projection_bytes);
    validate_public_claims(&input.public_claims, &projection_identity, &projection, profile, &mut diagnostics);
    let identities = match input_identities(input) {
        Ok(value) => value,
        Err(error) => return failed(vec![error]),
    };
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return failed(diagnostics);
    }
    Evaluation {
        details: Some(ReceiptDetails {
            schema_version: RECEIPT_SCHEMA.to_owned(),
            profile_id: profile.profile_id.clone(),
            profile_identity: PROFILE_IDENTITY.to_owned(),
            provider_identity: format!(
                "{}@{}:{}",
                profile.provider.package, profile.provider.version, profile.provider.closure_identity
            ),
            command_argv: workload.command_argv.clone(),
            baseline_identity: identities.baseline,
            candidate_identity: identities.candidate,
            allocation_identity: identities.allocations,
            copy_identity: identities.copies,
            oracle_identity: identities.oracle,
            findings_identity: identities.findings,
            projection_identity,
            bounded_log_identity: input.bounded_log_identity.clone(),
            projection,
            public_claims: input.public_claims.clone(),
            evidence_tier: profile.evidence_tier.clone(),
            non_claims: profile
                .non_claims
                .iter()
                .cloned()
                .chain(std::iter::once(REQUIRED_NON_CLAIM.to_owned()))
                .collect(),
        }),
        diagnostics: Vec::new(),
    }
}

pub(super) fn validate_details(details: &ReceiptDetails) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if details.schema_version != RECEIPT_SCHEMA || details.profile_id != PROFILE_ID {
        diagnostics.push("benchmark receipt schema or profile is unsupported".to_owned());
    }
    if details.profile_identity != PROFILE_IDENTITY {
        diagnostics.push("benchmark receipt profile identity is stale".to_owned());
    }
    let profile = match selected_profile(&details.profile_id) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(error);
            return diagnostics;
        }
    };
    if !profile.workloads.iter().any(|workload| workload.command_argv == details.command_argv) {
        diagnostics.push("benchmark receipt command is not in the checked profile".to_owned());
    }
    match projection_bytes(&details.projection) {
        Ok(bytes) if prefixed_identity(&bytes) == details.projection_identity => {}
        Ok(_) => diagnostics.push("benchmark receipt projection identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    validate_public_claims(
        &details.public_claims,
        &details.projection_identity,
        &details.projection,
        &profile,
        &mut diagnostics,
    );
    if details.evidence_tier != "transcript_assertion"
        || !details.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM)
    {
        diagnostics.push("benchmark receipt evidence tier or non-claim is unsupported".to_owned());
    }
    for identity in [
        &details.baseline_identity,
        &details.candidate_identity,
        &details.allocation_identity,
        &details.copy_identity,
        &details.oracle_identity,
        &details.findings_identity,
        &details.bounded_log_identity,
    ] {
        if !resources::is_b3(identity) {
            diagnostics.push("benchmark receipt contains a malformed artifact identity".to_owned());
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_run(
    run: &RunSet,
    workload: &WorkloadProfile,
    profile: &Profile,
    label: &str,
    diagnostics: &mut Vec<String>,
) {
    if run.workload_id != workload.workload_id
        || run.workload_identity != workload.workload_identity
        || run.command_id != workload.command_id
        || run.command_argv != workload.command_argv
        || run.mode != workload.mode
        || run.source_state_identity != workload.source_state_identity
        || run.environment.environment_identity != workload.environment_identity
        || run.environment.rust_toolchain != profile.rust_toolchain
        || run.environment.nix_closure_identity != profile.provider.closure_identity
    {
        diagnostics.push(format!("benchmark {label} does not match the checked workload profile"));
    }
    if run.environment.host_os.is_empty()
        || run.environment.host_arch.is_empty()
        || run.environment.kernel_identity.is_empty()
        || run.environment.cpu_identity.is_empty()
        || run.environment.governor.is_empty()
        || run.environment.load_class.is_empty()
        || run.environment.isolation.is_empty()
    {
        diagnostics.push(format!("benchmark {label} environment facts are incomplete"));
    }
    if run.samples.len() != workload.sample_count || run.samples.len() > profile.limits.max_samples {
        diagnostics.push(format!("benchmark {label} sample count is outside profile limits"));
    }
    let mut indices = BTreeSet::new();
    for sample in &run.samples {
        if !indices.insert(sample.sample_index)
            || !SUPPORTED_OUTCOMES.contains(&sample.outcome.as_str())
            || !resources::is_b3(&sample.log_identity)
            || (sample.outcome == "succeeded" && sample.wall_duration_ns == 0)
            || run.environment.cpu_time_supported != sample.cpu_duration_ns.is_some()
            || run.environment.peak_memory_supported != sample.peak_memory_bytes.is_some()
        {
            diagnostics.push(format!("benchmark {label} contains an invalid sample"));
        }
    }
}

fn validate_public_claims(
    claims: &[PublicClaim],
    projection_identity: &str,
    projection: &BenchmarkProjection,
    profile: &Profile,
    diagnostics: &mut Vec<String>,
) {
    for claim in claims {
        if claim.metric.is_empty()
            || claim.value.is_empty()
            || claim.corpus.is_empty()
            || claim.environment.is_empty()
            || claim.sample_range.is_empty()
            || claim.interpretation_boundary.is_empty()
            || claim.projection_identity != projection_identity
            || !profile.signal_corpus.iter().any(|corpus| corpus.corpus_id == claim.corpus)
            || (claim.metric == "copy-bytes" && projection.resources.unknown_copy_length_count > 0)
        {
            diagnostics.push("benchmark public claim lacks verified evidence or an interpretation boundary".to_owned());
        }
    }
}

struct InputIdentities {
    baseline: String,
    candidate: String,
    allocations: String,
    copies: String,
    oracle: String,
    findings: String,
}

fn input_identities(input: &request::Input) -> Result<InputIdentities, String> {
    Ok(InputIdentities {
        baseline: identity_of(&input.baseline, "benchmark baseline")?,
        candidate: identity_of(&input.candidate, "benchmark candidate")?,
        allocations: identity_of(&input.allocations, "benchmark allocations")?,
        copies: identity_of(&input.copies, "benchmark copy sites")?,
        oracle: identity_of(&input.oracle, "benchmark signal oracle")?,
        findings: identity_of(&input.findings, "benchmark signal findings")?,
    })
}

fn identity_of<T: serde::Serialize>(value: &T, label: &str) -> Result<String, String> {
    request::payload(value, label).map(|bytes| prefixed_identity(&bytes))
}

fn failed(mut diagnostics: Vec<String>) -> Evaluation {
    diagnostics.sort();
    diagnostics.dedup();
    Evaluation {
        details: None,
        diagnostics,
    }
}

const fn empty_resource_summary() -> ResourceSummary {
    ResourceSummary {
        allocation_events: 0,
        allocated_bytes: 0,
        copy_site_count: 0,
        known_copy_bytes: 0,
        unknown_copy_length_count: 0,
    }
}

const fn empty_signal_summary() -> SignalSummary {
    SignalSummary {
        true_positive: 0,
        false_positive: 0,
        false_negative: 0,
        expected_suppression: 0,
        unexpected_suppression: 0,
        inconclusive: 0,
    }
}
