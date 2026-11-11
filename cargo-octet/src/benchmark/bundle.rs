// r[impl octet.benchmark_evidence.receipt]
use std::collections::BTreeMap;

use super::ALLOCATION_ARTIFACT;
use super::AllocationObservation;
use super::BASELINE_ARTIFACT;
use super::CANDIDATE_ARTIFACT;
use super::COPY_ARTIFACT;
use super::CopyObservation;
use super::FINDINGS_ARTIFACT;
use super::INPUT_SCHEMA;
use super::ORACLE_ARTIFACT;
use super::OracleRecord;
use super::PROJECTION_ARTIFACT;
use super::ReceiptDetails;
use super::RunSet;
use super::SignalFinding;
use super::prefixed_identity;
use super::request;
use super::selected_profile;
use super::validation;

/// Remeasure every benchmark artifact and reproduce the receipt from its raw inputs and policy.
pub(super) fn validate(details: &ReceiptDetails, artifacts: &BTreeMap<String, Vec<u8>>) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let profile = match selected_profile(&details.profile_id) {
        Ok(value) => value,
        Err(error) => return vec![error],
    };
    let baseline =
        parse::<RunSet>(artifacts, BASELINE_ARTIFACT, &details.baseline_identity, "baseline", &mut diagnostics);
    let candidate =
        parse::<RunSet>(artifacts, CANDIDATE_ARTIFACT, &details.candidate_identity, "candidate", &mut diagnostics);
    let allocations = parse::<Vec<AllocationObservation>>(
        artifacts,
        ALLOCATION_ARTIFACT,
        &details.allocation_identity,
        "allocations",
        &mut diagnostics,
    );
    let copies =
        parse::<Vec<CopyObservation>>(artifacts, COPY_ARTIFACT, &details.copy_identity, "copy sites", &mut diagnostics);
    let oracle = parse::<Vec<OracleRecord>>(
        artifacts,
        ORACLE_ARTIFACT,
        &details.oracle_identity,
        "signal oracle",
        &mut diagnostics,
    );
    let findings = parse::<Vec<SignalFinding>>(
        artifacts,
        FINDINGS_ARTIFACT,
        &details.findings_identity,
        "signal findings",
        &mut diagnostics,
    );
    measure(artifacts, PROJECTION_ARTIFACT, &details.projection_identity, "projection", &mut diagnostics);
    let (Some(baseline), Some(candidate), Some(allocations), Some(copies), Some(oracle), Some(findings)) =
        (baseline, candidate, allocations, copies, oracle, findings)
    else {
        diagnostics.sort();
        diagnostics.dedup();
        return diagnostics;
    };
    let input = request::Input {
        schema_version: INPUT_SCHEMA.to_owned(),
        baseline,
        candidate,
        allocations,
        copies,
        oracle,
        findings,
        public_claims: details.public_claims.clone(),
        bounded_log_identity: details.bounded_log_identity.clone(),
    };
    let evaluation = validation::evaluate(&input, &profile);
    diagnostics.extend(evaluation.diagnostics);
    match evaluation.details {
        Some(reproduced) if reproduced == *details => {}
        _ => diagnostics.push("benchmark receipt does not reproduce from its raw artifacts and policy".to_owned()),
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn parse<T: serde::de::DeserializeOwned>(
    artifacts: &BTreeMap<String, Vec<u8>>,
    name: &str,
    expected_identity: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<T> {
    let bytes = measure(artifacts, name, expected_identity, label, diagnostics)?;
    match serde_json::from_slice(bytes) {
        Ok(value) => Some(value),
        Err(error) => {
            diagnostics.push(format!("benchmark {label} artifact is malformed: {error}"));
            None
        }
    }
}

fn measure<'a>(
    artifacts: &'a BTreeMap<String, Vec<u8>>,
    name: &str,
    expected_identity: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<&'a [u8]> {
    let Some(bytes) = artifacts.get(name) else {
        diagnostics.push(format!("benchmark bundle is missing `{name}`"));
        return None;
    };
    if prefixed_identity(bytes) != expected_identity {
        diagnostics.push(format!("benchmark {label} identity is stale"));
    }
    Some(bytes)
}
