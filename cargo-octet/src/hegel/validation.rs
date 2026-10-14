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
const RESULT_PASSED: &str = "passed";
const RESULT_COUNTEREXAMPLE: &str = "counterexample";

pub(super) fn evaluate(input: &request::Input, profile: &Profile) -> Evaluation {
    let mut diagnostics = profile_validation::validate(profile);
    diagnostics.extend(validate_input(input, profile));
    let summary: ProducerSummary = match serde_json::from_str(&input.summary_text) {
        Ok(value) => value,
        Err(error) => return failed(vec![format!("parse dedicated Hegel producer summary: {error}")]),
    };
    diagnostics.extend(validate_summary(&summary, input, profile));
    if !diagnostics.is_empty() {
        return failed(diagnostics);
    }
    let source_manifest_identity = match request::manifest_identity(&input.source_manifest, "Hegel source manifest") {
        Ok(value) => value,
        Err(error) => return failed(vec![error]),
    };
    let generator_manifest_identity =
        match request::manifest_identity(&input.generator_manifest, "Hegel generator manifest") {
            Ok(value) => value,
            Err(error) => return failed(vec![error]),
        };
    let mut details = ReceiptDetails {
        schema_version: RECEIPT_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        profile_identity: PROFILE_IDENTITY.to_owned(),
        law_catalog_identity: LAW_CATALOG_IDENTITY.to_owned(),
        property_id: summary.property_id.clone(),
        linked_ids: summary.linked_ids.clone(),
        source_scope: summary.source_scope.clone(),
        producer: profile.producer.clone(),
        settings: summary.settings.clone(),
        generator: summary.generator.clone(),
        command_argv: profile.command_argv.clone(),
        cargo_lock_identity: input.cargo_lock_identity.clone(),
        package_manifest_identity: input.package_manifest_identity.clone(),
        source_manifest_identity,
        generator_manifest_identity,
        summary_identity: prefixed_identity(input.summary_text.as_bytes()),
        bounded_log_identity: prefixed_identity(&input.bounded_log),
        counterexample_identity: input.counterexample.as_ref().map(|bytes| prefixed_identity(bytes)),
        result: summary.result,
        cases_completed: summary.cases_completed,
        issue_codes: normalize_issue_codes(&summary.issue_codes),
        projection_identity: String::new(),
        evidence_tier: profile.evidence_tier.clone(),
        verification_role: profile.verification_role.clone(),
        non_claims: profile.non_claims.clone(),
    };
    details.projection_identity = match projection_bytes(&details) {
        Ok(bytes) => prefixed_identity(&bytes),
        Err(error) => return failed(vec![error]),
    };
    Evaluation {
        details: Some(details),
        diagnostics: Vec::new(),
    }
}

pub(super) fn validate_details(details: &ReceiptDetails) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if details.schema_version != RECEIPT_SCHEMA
        || details.profile_id != PROFILE_ID
        || details.profile_identity != PROFILE_IDENTITY
        || details.law_catalog_identity != LAW_CATALOG_IDENTITY
    {
        diagnostics.push("profiled Hegel receipt schema or checked identity is unsupported".to_owned());
    }
    if details.property_id.trim().is_empty()
        || details.linked_ids.is_empty()
        || details.source_scope.trim().is_empty()
        || details.producer.package != "cargo-octet"
        || details.producer.hegel_version != "0.14.27"
        || details.generator.content_hash != LAW_CATALOG_IDENTITY
        || details.settings.case_budget == 0
        || details.command_argv.iter().map(String::as_str).collect::<Vec<_>>() != EXPECTED_COMMAND
        || details.evidence_tier != EXPECTED_EVIDENCE_TIER
        || details.verification_role != EXPECTED_VERIFICATION_ROLE
        || !details.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM)
    {
        diagnostics.push("profiled Hegel receipt authority or evidence boundary is unsupported".to_owned());
    }
    for identity in [
        &details.cargo_lock_identity,
        &details.package_manifest_identity,
        &details.source_manifest_identity,
        &details.generator_manifest_identity,
        &details.summary_identity,
        &details.bounded_log_identity,
        &details.projection_identity,
    ] {
        validate_b3(identity, "profiled Hegel receipt", &mut diagnostics);
    }
    if let Some(identity) = details.counterexample_identity.as_ref() {
        validate_b3(identity, "profiled Hegel counterexample", &mut diagnostics);
    }
    if details.issue_codes != normalize_issue_codes(&details.issue_codes) {
        diagnostics.push("profiled Hegel receipt issue codes are not canonical".to_owned());
    }
    match projection_bytes(details) {
        Ok(bytes) if prefixed_identity(&bytes) == details.projection_identity => {}
        Ok(_) => diagnostics.push("profiled Hegel projection identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_input(input: &request::Input, profile: &Profile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if byte_len(input.summary_text.as_bytes()) > profile.limits.max_summary_bytes
        || input.bounded_log.len() > profile.limits.max_log_bytes
        || input.source_manifest.len() != profile.source_scope.len()
        || input.generator_manifest.len() != profile.generator_inputs.len()
        || input
            .counterexample
            .as_ref()
            .is_some_and(|bytes| byte_len(bytes) > profile.limits.max_counterexample_bytes)
    {
        diagnostics.push("profiled Hegel input exceeds a checked profile limit".to_owned());
    }
    validate_manifest(&input.source_manifest, &profile.source_scope, "source", &mut diagnostics);
    validate_manifest(&input.generator_manifest, &profile.generator_inputs, "generator", &mut diagnostics);
    validate_b3(&input.cargo_lock_identity, "Cargo lock", &mut diagnostics);
    validate_b3(&input.package_manifest_identity, "package manifest", &mut diagnostics);
    diagnostics
}

fn validate_manifest(manifest: &[FileIdentity], expected: &[String], label: &str, diagnostics: &mut Vec<String>) {
    let expected = expected.iter().collect::<BTreeSet<_>>();
    let mut observed = BTreeSet::new();
    for file in manifest {
        if profile_validation::unsafe_relative_path(&file.locator) || file.bytes == 0 || !observed.insert(&file.locator)
        {
            diagnostics.push(format!("Hegel {label} manifest contains an unsafe, empty, or duplicate entry"));
        }
        validate_b3(&file.identity, &format!("Hegel {label} manifest"), diagnostics);
    }
    if observed != expected {
        diagnostics.push(format!("Hegel {label} manifest does not match the checked profile"));
    }
}

fn validate_summary(summary: &ProducerSummary, input: &request::Input, profile: &Profile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let source_identity = request::manifest_identity(&input.source_manifest, "Hegel source manifest").ok();
    let generator_identity = request::manifest_identity(&input.generator_manifest, "Hegel generator manifest").ok();
    let law_id = law_catalog().ok().and_then(|catalog| catalog.laws.first().map(|law| law.law_id.clone()));
    if summary.schema_version != SUMMARY_SCHEMA
        || summary.profile_id != profile.profile_id
        || summary.profile_identity != PROFILE_IDENTITY
        || summary.package != profile.producer.package
        || summary.package_version != profile.producer.package_version
        || summary.hegel_version != profile.producer.hegel_version
        || summary.backend != profile.producer.backend
        || summary.property_catalog != profile.property_catalog
        || Some(&summary.property_id) != law_id.as_ref()
        || summary.source_scope != profile.source_scope.join(",")
        || summary.settings != profile.settings
        || summary.generator.content_hash != LAW_CATALOG_IDENTITY
        || source_identity.as_ref() != Some(&summary.source_manifest_identity)
        || generator_identity.as_ref() != Some(&summary.generator_manifest_identity)
        || summary.cargo_lock_identity != input.cargo_lock_identity
        || summary.package_manifest_identity != input.package_manifest_identity
    {
        diagnostics.push("Hegel producer summary does not match measured profile or source facts".to_owned());
    }
    if summary.cases_completed > profile.settings.case_budget
        || summary.issue_codes.len() > profile.limits.max_issue_codes
        || summary.issue_codes != normalize_issue_codes(&summary.issue_codes)
    {
        diagnostics.push("Hegel producer summary result or issue-code limit is invalid".to_owned());
    }
    match (summary.result.as_str(), &summary.counterexample_identity, &input.counterexample) {
        (RESULT_PASSED, None, None) if summary.cases_completed == profile.settings.case_budget => {}
        (RESULT_COUNTEREXAMPLE, Some(declared), Some(bytes)) if prefixed_identity(bytes) == *declared => {}
        _ => diagnostics.push("Hegel producer summary result and retained counterexample are inconsistent".to_owned()),
    }
    diagnostics
}

fn validate_b3(value: &str, label: &str, diagnostics: &mut Vec<String>) {
    let valid = value.strip_prefix("b3:").is_some_and(|hex| {
        hex.len() == blake3::OUT_LEN * 2 && hex.bytes().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
    });
    if !valid {
        diagnostics.push(format!("{label} identity is not canonical BLAKE3"));
    }
}

fn byte_len(bytes: &[u8]) -> u64 {
    u64::try_from(bytes.len()).unwrap_or(u64::MAX)
}

fn failed(mut diagnostics: Vec<String>) -> Evaluation {
    diagnostics.sort();
    diagnostics.dedup();
    Evaluation {
        details: None,
        diagnostics,
    }
}
