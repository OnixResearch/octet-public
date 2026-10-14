use super::*;

const REQUIRED_METRIC_COUNT: usize = 4;
const REQUIRED_METRICS: [&str; REQUIRED_METRIC_COUNT] = ["lines", "branches", "functions", "regions"];
const EXPECTED_PROVIDER_PACKAGE: &str = "cargo-llvm-cov";
const EXPECTED_PROVIDER_VERSION: &str = "0.8.3";
const EXPECTED_EVIDENCE_TIER: &str = "transcript_assertion";
const EXPECTED_ROUNDING_POLICY: &str = "floor";
const EXPECTED_REGRESSION_POLICY: &str = "deny_decrease";
const EXPECTED_DENOMINATOR_POLICY: &str = "require_equal";
const EXPECTED_PRECISION_DIGITS: u8 = 2;
const DIAGNOSTIC_CAPACITY: usize = 16;

pub(super) fn evaluate(
    current_text: &str,
    baseline_envelope_text: &str,
    profile: &Profile,
    source_identity: &str,
) -> Evaluation {
    let mut diagnostics = validate_profile(profile);
    if !diagnostics.is_empty() {
        return failed(diagnostics);
    }
    let current = match native::normalize(current_text, profile, source_identity) {
        Ok(value) => value,
        Err(errors) => return failed(errors),
    };
    let baseline_envelope = match parse_baseline_envelope(baseline_envelope_text, profile) {
        Ok(value) => value,
        Err(errors) => return failed(errors),
    };
    let expected_baseline =
        match native::normalize(&baseline_envelope.raw_report_text, profile, &baseline_envelope.report.source_identity)
        {
            Ok(value) => value,
            Err(errors) => return failed(errors),
        };
    if expected_baseline != baseline_envelope.report {
        return failed(vec!["coverage baseline envelope report is stale".to_owned()]);
    }
    let (comparison, comparison_diagnostics) = comparison::compare(&current, &expected_baseline, profile);
    diagnostics.extend(comparison_diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    let details = match details(DetailInput {
        current_text,
        baseline_envelope_text,
        current,
        baseline: expected_baseline,
        comparison,
        profile,
    }) {
        Ok(value) => value,
        Err(error) => return failed(vec![error]),
    };
    Evaluation {
        details: Some(details),
        diagnostics,
    }
}

pub(super) fn validate_details(details: &ReceiptDetails) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(DIAGNOSTIC_CAPACITY);
    if details.schema_version != RECEIPT_SCHEMA || details.profile_id != PROFILE_ID {
        diagnostics.push("coverage receipt schema or profile is unsupported".to_owned());
    }
    if details.profile_identity != PROFILE_IDENTITY {
        diagnostics.push("coverage receipt profile identity is stale".to_owned());
    }
    let profile = match selected_profile(&details.profile_id) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(error);
            return diagnostics;
        }
    };
    if details.command_argv != profile.command_argv {
        diagnostics.push("coverage receipt command does not match the checked profile".to_owned());
    }
    let (comparison, _) = comparison::compare(&details.current, &details.baseline, &profile);
    if comparison != details.comparison {
        diagnostics.push("coverage receipt comparison is stale".to_owned());
    }
    match projection_bytes(details) {
        Ok(bytes) if prefixed_identity(&bytes) == details.projection_identity => {}
        Ok(_) => diagnostics.push("coverage receipt projection identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    if details.evidence_tier != EXPECTED_EVIDENCE_TIER
        || !details.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM)
    {
        diagnostics.push("coverage receipt evidence tier or non-claim is unsupported".to_owned());
    }
    diagnostics
}

fn validate_profile(profile: &Profile) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(DIAGNOSTIC_CAPACITY);
    if profile.profile_id != PROFILE_ID {
        diagnostics.push("coverage profile id is unsupported".to_owned());
    }
    if profile.provider.package != EXPECTED_PROVIDER_PACKAGE || profile.provider.version != EXPECTED_PROVIDER_VERSION {
        diagnostics.push("coverage provider identity is unsupported".to_owned());
    }
    if profile.metrics != REQUIRED_METRICS {
        diagnostics.push("coverage metric set is unsupported".to_owned());
    }
    if profile.command_argv.is_empty() || profile.command_argv.first().map(String::as_str) != Some("cargo") {
        diagnostics.push("coverage profile command is missing its fixed cargo executable".to_owned());
    }
    if profile.evidence_tier != EXPECTED_EVIDENCE_TIER || profile.non_claims.is_empty() {
        diagnostics.push("coverage profile evidence tier or non-claims are unsupported".to_owned());
    }
    if profile.rounding_policy != EXPECTED_ROUNDING_POLICY || profile.precision_digits != EXPECTED_PRECISION_DIGITS {
        diagnostics.push("coverage display precision policy is unsupported".to_owned());
    }
    if profile.regression_policy != EXPECTED_REGRESSION_POLICY
        || profile.denominator_policy != EXPECTED_DENOMINATOR_POLICY
    {
        diagnostics.push("coverage comparison policy is unsupported".to_owned());
    }
    if profile.max_report_bytes == 0
        || profile.max_files == 0
        || profile.max_path_bytes == 0
        || profile.max_log_bytes == 0
        || profile.timeout_ms == 0
        || profile.poll_interval_ms == 0
        || profile.teardown_timeout_ms == 0
    {
        diagnostics.push("coverage profile contains a zero resource limit".to_owned());
    }
    diagnostics
}

fn parse_baseline_envelope(text: &str, profile: &Profile) -> Result<NativeEnvelope, Vec<String>> {
    let bytes = u64::try_from(text.len())
        .map_err(|_| vec!["coverage baseline envelope byte count does not fit u64".to_owned()])?;
    if bytes > profile.max_report_bytes {
        return Err(vec!["coverage baseline envelope exceeds profile byte limit".to_owned()]);
    }
    let envelope: NativeEnvelope = serde_json::from_str(text)
        .map_err(|error| vec![format!("coverage baseline envelope is malformed JSON: {error}")])?;
    if envelope.schema_version != NATIVE_SCHEMA || envelope.report.schema_version != NATIVE_SCHEMA {
        return Err(vec!["coverage baseline envelope schema is unsupported".to_owned()]);
    }
    Ok(envelope)
}

struct DetailInput<'a> {
    current_text: &'a str,
    baseline_envelope_text: &'a str,
    current: NativeReport,
    baseline: NativeReport,
    comparison: Comparison,
    profile: &'a Profile,
}

fn details(input: DetailInput<'_>) -> Result<ReceiptDetails, String> {
    let current_envelope = envelope_bytes(&input.current, input.current_text)?;
    let projection = pretty_bytes(&input.comparison, "coverage comparison projection")?;
    Ok(ReceiptDetails {
        schema_version: RECEIPT_SCHEMA.to_owned(),
        profile_id: input.profile.profile_id.clone(),
        profile_identity: PROFILE_IDENTITY.to_owned(),
        command_argv: input.profile.command_argv.clone(),
        current: input.current,
        baseline: input.baseline,
        current_envelope_identity: prefixed_identity(&current_envelope),
        baseline_envelope_identity: prefixed_identity(input.baseline_envelope_text.as_bytes()),
        projection_identity: prefixed_identity(&projection),
        comparison: input.comparison,
        evidence_tier: input.profile.evidence_tier.clone(),
        non_claims: input
            .profile
            .non_claims
            .iter()
            .cloned()
            .chain(std::iter::once(REQUIRED_NON_CLAIM.to_owned()))
            .collect(),
    })
}

fn failed(mut diagnostics: Vec<String>) -> Evaluation {
    diagnostics.sort();
    diagnostics.dedup();
    Evaluation {
        details: None,
        diagnostics,
    }
}
