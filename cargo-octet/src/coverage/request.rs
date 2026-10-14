use super::*;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
struct ProfiledInput {
    schema_version: String,
    current_report_text: String,
    baseline_envelope_text: String,
    source_identity: String,
    source_manifest_text: String,
    bounded_log_identity: String,
}

const PROFILED_INPUT_SCHEMA: &str = "octet-coverage-profiled-input/v1";

pub(crate) fn render(
    current_report_text: String,
    baseline_envelope_text: String,
    source_identity: String,
    source_manifest_text: String,
    bounded_log_identity: String,
) -> Result<String, String> {
    serde_json::to_string(&ProfiledInput {
        schema_version: PROFILED_INPUT_SCHEMA.to_owned(),
        current_report_text,
        baseline_envelope_text,
        source_identity,
        source_manifest_text,
        bounded_log_identity,
    })
    .map_err(|error| format!("serialize profiled coverage input: {error}"))
}

pub(crate) fn evaluate(text: &str, profile_id: &str) -> Evaluation {
    let request = match parse(text) {
        Ok(value) => value,
        Err(error) => {
            return Evaluation {
                details: None,
                diagnostics: vec![error],
            };
        }
    };
    let observed_source_identity = format!("source:{}", prefixed_identity(request.source_manifest_text.as_bytes()));
    if observed_source_identity != request.source_identity {
        return Evaluation {
            details: None,
            diagnostics: vec!["profiled coverage source manifest identity is stale".to_owned()],
        };
    }
    evaluate_profiled(
        &request.current_report_text,
        &request.baseline_envelope_text,
        profile_id,
        &request.source_identity,
    )
}

pub(crate) fn bounded_log_identity(text: &str) -> Option<String> {
    parse(text).ok().map(|request| request.bounded_log_identity)
}

pub(crate) fn baseline_identity(text: &str) -> Option<String> {
    parse(text).ok().map(|request| prefixed_identity(request.baseline_envelope_text.as_bytes()))
}

pub(crate) fn artifact_payloads(
    details: &ReceiptDetails,
    text: &str,
) -> Result<std::collections::BTreeMap<String, Vec<u8>>, String> {
    let request = parse(text)?;
    let current_envelope = envelope_bytes(&details.current, &request.current_report_text)?;
    let projection = projection_bytes(details)?;
    validate_identity("current coverage envelope", &current_envelope, &details.current_envelope_identity)?;
    validate_identity(
        "baseline coverage envelope",
        request.baseline_envelope_text.as_bytes(),
        &details.baseline_envelope_identity,
    )?;
    validate_identity("coverage comparison projection", &projection, &details.projection_identity)?;
    if prefixed_identity(request.current_report_text.as_bytes()) != details.current.raw_report_identity {
        return Err("current coverage raw report identity is stale".to_owned());
    }
    if format!("source:{}", prefixed_identity(request.source_manifest_text.as_bytes()))
        != details.current.source_identity
    {
        return Err("coverage source manifest identity is stale".to_owned());
    }
    let mut artifacts = std::collections::BTreeMap::new();
    artifacts.insert(SOURCE_MANIFEST_ARTIFACT.to_owned(), request.source_manifest_text.into_bytes());
    artifacts.insert(CURRENT_RAW_ARTIFACT.to_owned(), request.current_report_text.into_bytes());
    artifacts.insert(CURRENT_ENVELOPE_ARTIFACT.to_owned(), current_envelope);
    artifacts.insert(BASELINE_ENVELOPE_ARTIFACT.to_owned(), request.baseline_envelope_text.into_bytes());
    artifacts.insert(PROJECTION_ARTIFACT.to_owned(), projection);
    Ok(artifacts)
}

fn parse(text: &str) -> Result<ProfiledInput, String> {
    let input: ProfiledInput =
        serde_json::from_str(text).map_err(|error| format!("parse profiled coverage input: {error}"))?;
    if input.schema_version != PROFILED_INPUT_SCHEMA {
        return Err("profiled coverage input schema is unsupported".to_owned());
    }
    if !input.bounded_log_identity.starts_with("b3:") {
        return Err("profiled coverage input is missing bounded log identity".to_owned());
    }
    Ok(input)
}

fn validate_identity(label: &str, bytes: &[u8], expected: &str) -> Result<(), String> {
    if prefixed_identity(bytes) != expected {
        return Err(format!("{label} identity is stale"));
    }
    Ok(())
}
