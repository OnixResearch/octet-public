use super::*;

const INPUT_SCHEMA: &str = "octet-cargo-mutants-input/v1";

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(crate) struct InputParts {
    pub outcomes_text: String,
    pub inventory_text: String,
    pub source_manifest_text: String,
    pub artifacts: std::collections::BTreeMap<String, String>,
    pub bounded_log_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub(super) struct ProfiledInput {
    pub(super) schema_version: String,
    pub(super) outcomes_text: String,
    pub(super) inventory_text: String,
    pub(super) source_manifest_text: String,
    pub(super) artifacts: std::collections::BTreeMap<String, String>,
    pub(super) bounded_log_identity: String,
}

pub(crate) fn render(parts: InputParts) -> Result<String, String> {
    serde_json::to_string(&ProfiledInput {
        schema_version: INPUT_SCHEMA.to_owned(),
        outcomes_text: parts.outcomes_text,
        inventory_text: parts.inventory_text,
        source_manifest_text: parts.source_manifest_text,
        artifacts: parts.artifacts,
        bounded_log_identity: parts.bounded_log_identity,
    })
    .map_err(|error| format!("serialize profiled mutation input: {error}"))
}

pub(crate) fn evaluate(text: &str, profile_id: &str) -> Evaluation {
    let input = match parse(text) {
        Ok(value) => value,
        Err(error) => {
            return Evaluation {
                details: None,
                diagnostics: vec![error],
            };
        }
    };
    let profile = match selected_profile(profile_id) {
        Ok(value) => value,
        Err(error) => {
            return Evaluation {
                details: None,
                diagnostics: vec![error],
            };
        }
    };
    validation::evaluate(&input, &profile)
}

pub(crate) fn bounded_log_identity(text: &str) -> Option<String> {
    parse(text).ok().map(|input| input.bounded_log_identity)
}

pub(crate) fn artifact_payloads(
    details: &ReceiptDetails,
    text: &str,
) -> Result<std::collections::BTreeMap<String, Vec<u8>>, String> {
    let input = parse(text)?;
    validate_identity("mutation outcomes", input.outcomes_text.as_bytes(), &details.native_outcomes_identity)?;
    validate_identity("mutation inventory", input.inventory_text.as_bytes(), &details.native_inventory_identity)?;
    validate_identity(
        "mutation source manifest",
        input.source_manifest_text.as_bytes(),
        &details.source_manifest_identity,
    )?;
    let projection = projection_bytes(details)?;
    validate_identity("mutation projection", &projection, &details.projection_identity)?;
    let mut output = std::collections::BTreeMap::new();
    output.insert(SOURCE_MANIFEST_ARTIFACT.to_owned(), input.source_manifest_text.into_bytes());
    output.insert(OUTCOMES_ARTIFACT.to_owned(), input.outcomes_text.into_bytes());
    output.insert(INVENTORY_ARTIFACT.to_owned(), input.inventory_text.into_bytes());
    output.insert(PROJECTION_ARTIFACT.to_owned(), projection);
    insert_retained(&mut output, &input.artifacts, &details.baseline.log)?;
    for mutant in &details.mutants {
        insert_retained(&mut output, &input.artifacts, &mutant.log)?;
        if let Some(diff) = mutant.diff.as_ref() {
            insert_retained(&mut output, &input.artifacts, diff)?;
        }
    }
    Ok(output)
}

fn insert_retained(
    output: &mut std::collections::BTreeMap<String, Vec<u8>>,
    inputs: &std::collections::BTreeMap<String, String>,
    artifact: &RetainedArtifact,
) -> Result<(), String> {
    let text = inputs
        .get(&artifact.source_locator)
        .ok_or_else(|| format!("mutation artifact `{}` is missing", artifact.source_locator))?;
    validate_identity("mutation retained artifact", text.as_bytes(), &artifact.identity)?;
    if output.insert(artifact.locator.clone(), text.as_bytes().to_vec()).is_some() {
        return Err(format!("duplicate retained mutation artifact `{}`", artifact.locator));
    }
    Ok(())
}

fn parse(text: &str) -> Result<ProfiledInput, String> {
    let input: ProfiledInput =
        serde_json::from_str(text).map_err(|error| format!("parse profiled mutation input: {error}"))?;
    if input.schema_version != INPUT_SCHEMA {
        return Err("profiled mutation input schema is unsupported".to_owned());
    }
    Ok(input)
}

fn validate_identity(label: &str, bytes: &[u8], expected: &str) -> Result<(), String> {
    if prefixed_identity(bytes) != expected {
        return Err(format!("{label} identity is stale"));
    }
    Ok(())
}
