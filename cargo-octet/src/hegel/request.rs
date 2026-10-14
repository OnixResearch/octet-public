use std::collections::BTreeMap;

use super::*;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct Input {
    pub schema_version: String,
    pub summary_text: String,
    pub bounded_log: Vec<u8>,
    pub source_manifest: Vec<FileIdentity>,
    pub generator_manifest: Vec<FileIdentity>,
    pub cargo_lock_identity: String,
    pub package_manifest_identity: String,
    pub counterexample: Option<Vec<u8>>,
}

pub(crate) fn render(input: &Input) -> Result<String, String> {
    serde_json::to_string(input).map_err(|error| format!("serialize profiled Hegel input: {error}"))
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

pub(crate) fn parse(text: &str) -> Result<Input, String> {
    let input: Input = serde_json::from_str(text).map_err(|error| format!("parse profiled Hegel input: {error}"))?;
    if input.schema_version != REQUEST_SCHEMA {
        return Err("profiled Hegel input schema is unsupported".to_owned());
    }
    Ok(input)
}

pub(crate) fn bounded_log_identity(text: &str) -> Option<String> {
    parse(text).ok().map(|input| prefixed_identity(&input.bounded_log))
}

pub(crate) fn artifact_payloads(details: &ReceiptDetails, text: &str) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let input = parse(text)?;
    let summary = input.summary_text.into_bytes();
    validate_identity("Hegel producer summary", &summary, &details.summary_identity)?;
    validate_identity("Hegel bounded log", &input.bounded_log, &details.bounded_log_identity)?;
    let sources = checked_payload(&input.source_manifest, "Hegel source manifest", &details.source_manifest_identity)?;
    let generators =
        checked_payload(&input.generator_manifest, "Hegel generator manifest", &details.generator_manifest_identity)?;
    let projection = projection_bytes(details)?;
    validate_identity("Hegel projection", &projection, &details.projection_identity)?;
    let mut artifacts = BTreeMap::from([
        (SUMMARY_ARTIFACT.to_owned(), summary),
        (LOG_ARTIFACT.to_owned(), input.bounded_log),
        (SOURCE_MANIFEST_ARTIFACT.to_owned(), sources),
        (GENERATOR_MANIFEST_ARTIFACT.to_owned(), generators),
        (PROJECTION_ARTIFACT.to_owned(), projection),
    ]);
    match (input.counterexample, details.counterexample_identity.as_ref()) {
        (Some(bytes), Some(identity)) => {
            validate_identity("Hegel counterexample", &bytes, identity)?;
            artifacts.insert(COUNTEREXAMPLE_ARTIFACT.to_owned(), bytes);
        }
        (None, None) => {}
        _ => return Err("Hegel counterexample role or identity is stale".to_owned()),
    }
    Ok(artifacts)
}

pub(crate) fn manifest_identity(manifest: &[FileIdentity], label: &str) -> Result<String, String> {
    pretty_bytes(manifest, label).map(|bytes| prefixed_identity(&bytes))
}

fn checked_payload<T: serde::Serialize>(value: &T, label: &str, identity: &str) -> Result<Vec<u8>, String> {
    let bytes = pretty_bytes(value, label)?;
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
