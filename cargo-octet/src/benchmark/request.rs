use std::collections::BTreeMap;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Input {
    pub schema_version: String,
    pub baseline: RunSet,
    pub candidate: RunSet,
    pub allocations: Vec<AllocationObservation>,
    pub copies: Vec<CopyObservation>,
    pub oracle: Vec<OracleRecord>,
    pub findings: Vec<SignalFinding>,
    pub public_claims: Vec<PublicClaim>,
    pub bounded_log_identity: String,
}

pub(crate) fn render(input: &Input) -> Result<String, String> {
    serde_json::to_string(input).map_err(|error| format!("serialize profiled benchmark input: {error}"))
}

pub(crate) fn evaluate(text: &str, profile_id: &str) -> Evaluation {
    let input = match parse(text) {
        Ok(value) => value,
        Err(error) => return failed(vec![error]),
    };
    let profile = match selected_profile(profile_id) {
        Ok(value) => value,
        Err(error) => return failed(vec![error]),
    };
    validation::evaluate(&input, &profile)
}

pub(crate) fn bounded_log_identity(text: &str) -> Option<String> {
    parse(text).ok().map(|input| input.bounded_log_identity)
}

pub(crate) fn baseline_identity(text: &str) -> Option<String> {
    parse(text)
        .ok()
        .and_then(|input| payload(&input.baseline, "benchmark baseline").ok())
        .map(|bytes| prefixed_identity(&bytes))
}

pub(crate) fn artifact_payloads(details: &ReceiptDetails, text: &str) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let input = parse(text)?;
    let baseline = checked_payload(&input.baseline, "benchmark baseline", &details.baseline_identity)?;
    let candidate = checked_payload(&input.candidate, "benchmark candidate", &details.candidate_identity)?;
    let allocations = checked_payload(&input.allocations, "benchmark allocations", &details.allocation_identity)?;
    let copies = checked_payload(&input.copies, "benchmark copy sites", &details.copy_identity)?;
    let oracle = checked_payload(&input.oracle, "benchmark signal oracle", &details.oracle_identity)?;
    let findings = checked_payload(&input.findings, "benchmark signal findings", &details.findings_identity)?;
    let projection = projection_bytes(&details.projection)?;
    validate_identity("benchmark projection", &projection, &details.projection_identity)?;
    Ok(BTreeMap::from([
        (BASELINE_ARTIFACT.to_owned(), baseline),
        (CANDIDATE_ARTIFACT.to_owned(), candidate),
        (ALLOCATION_ARTIFACT.to_owned(), allocations),
        (COPY_ARTIFACT.to_owned(), copies),
        (ORACLE_ARTIFACT.to_owned(), oracle),
        (FINDINGS_ARTIFACT.to_owned(), findings),
        (PROJECTION_ARTIFACT.to_owned(), projection),
    ]))
}

pub(crate) fn parse(text: &str) -> Result<Input, String> {
    let input: Input =
        serde_json::from_str(text).map_err(|error| format!("parse profiled benchmark input: {error}"))?;
    if input.schema_version != INPUT_SCHEMA {
        return Err("profiled benchmark input schema is unsupported".to_owned());
    }
    Ok(input)
}

pub(crate) fn payload<T: serde::Serialize>(value: &T, label: &str) -> Result<Vec<u8>, String> {
    pretty_bytes(value, label)
}

fn checked_payload<T: serde::Serialize>(value: &T, label: &str, identity: &str) -> Result<Vec<u8>, String> {
    let bytes = payload(value, label)?;
    validate_identity(label, &bytes, identity)?;
    Ok(bytes)
}

fn validate_identity(label: &str, bytes: &[u8], expected: &str) -> Result<(), String> {
    if prefixed_identity(bytes) != expected {
        return Err(format!("{label} identity is stale"));
    }
    Ok(())
}

fn failed(mut diagnostics: Vec<String>) -> Evaluation {
    diagnostics.sort();
    diagnostics.dedup();
    Evaluation {
        details: None,
        diagnostics,
    }
}
