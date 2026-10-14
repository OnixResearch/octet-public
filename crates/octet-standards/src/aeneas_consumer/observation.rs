use serde::Deserialize;
use serde_json::Value;

use super::model::ConsumerArtifactIdentity;
use super::model::ConsumerOutcome;
use super::model::ConsumerProfile;
use super::model::ConsumerReceiptBinding;

/// Already-loaded source and provider artifacts for pure selection checks.
pub struct ObservationInput<'a> {
    /// Selected source path.
    pub source_path: &'a str,
    /// Exact selected source bytes as UTF-8 text.
    pub source_text: &'a str,
    /// Qualified selected Rust item.
    pub selected_item: &'a str,
    /// Maintained theorem path.
    pub theorem_path: &'a str,
    /// Exact maintained theorem bytes as UTF-8 text.
    pub theorem_text: &'a str,
    /// Exact Charon LLBC artifact.
    pub llbc_text: &'a str,
    /// Exact Aeneas translation manifest.
    pub translation_text: &'a str,
    /// Exact generated Lean source.
    pub generated_lean_text: &'a str,
}

#[derive(Deserialize)]
struct TranslationManifest {
    functions: Vec<TranslationFunction>,
}

#[derive(Deserialize)]
struct TranslationFunction {
    lean_name: String,
    rust_name: String,
    is_local: bool,
    source: TranslationSource,
}

#[derive(Deserialize)]
struct TranslationSource {
    file: String,
    begin_line: u64,
    end_line: u64,
}

/// Build a consumer receipt binding after exact-source validation.
///
/// # Errors
///
/// Returns diagnostics when source, selected-item, generated-artifact, or theorem bindings drift.
pub fn build_binding(
    profile: &ConsumerProfile,
    profile_identity: &str,
    input: &ObservationInput<'_>,
    outcome: ConsumerOutcome,
) -> Result<ConsumerReceiptBinding, Vec<String>> {
    let mut diagnostics = validate(profile, input);
    if !super::is_b3(profile_identity) {
        diagnostics.push("Aeneas consumer profile identity is malformed".to_owned());
    }
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let source = profile
        .source
        .paths
        .iter()
        .find(|source| source.path == input.source_path)
        .ok_or_else(|| vec!["Aeneas consumer selected source is absent".to_owned()])?;
    let selected_item = profile
        .source
        .selected_items
        .iter()
        .find(|item| item.qualified_name == input.selected_item)
        .cloned()
        .ok_or_else(|| vec!["Aeneas consumer selected item is absent".to_owned()])?;
    let theorem = profile
        .theorems
        .iter()
        .find(|theorem| theorem.path == input.theorem_path)
        .cloned()
        .ok_or_else(|| vec!["Aeneas consumer theorem mapping is absent".to_owned()])?;
    let mut receipt = ConsumerReceiptBinding {
        schema_version: super::RECEIPT_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        profile_identity: profile_identity.to_owned(),
        consumer_id: profile.consumer_id.clone(),
        provider_profile_identity: profile.provider.profile_identity.clone(),
        compatibility_matrix_identity: profile.provider.compatibility_matrix_identity.clone(),
        source_root: profile.source.root.clone(),
        source_path: source.path.clone(),
        source_identity: source.identity.clone(),
        selected_item,
        theorem,
        external_models: profile.external_models.clone(),
        trust_inputs: profile.trust_inputs.clone(),
        resource_policy: profile.resource_policy.clone(),
        outcome,
        stage_artifacts: stage_artifacts(input),
        verification_role: super::EXPECTED_VERIFICATION_ROLE.to_owned(),
        non_claims: profile.non_claims.clone(),
        receipt_identity: String::new(),
    };
    receipt.receipt_identity = super::receipt_identity(&receipt).map_err(|error| vec![error])?;
    Ok(receipt)
}

/// Validate exact source, span, translation, generated, and theorem facts.
#[must_use]
pub fn validate(profile: &ConsumerProfile, input: &ObservationInput<'_>) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let source_identity = super::prefixed_identity(input.source_text.as_bytes());
    let source_matches = profile
        .source
        .paths
        .iter()
        .filter(|source| source.path == input.source_path && source.identity == source_identity)
        .count();
    let selected = profile
        .source
        .selected_items
        .iter()
        .filter(|item| item.qualified_name == input.selected_item && item.source_identity == source_identity)
        .collect::<Vec<_>>();
    if source_matches != 1 || selected.len() != 1 {
        diagnostics.push("Aeneas consumer source or selected item is stale, missing, copied, or ambiguous".to_owned());
        return diagnostics;
    }
    let item = selected[0];
    validate_translation(item, input.translation_text, &mut diagnostics);
    validate_llbc(item, input, &mut diagnostics);
    if !generated_declaration_present(&item.generated_name, input.generated_lean_text) {
        diagnostics.push("Aeneas consumer generated Lean does not contain the selected declaration".to_owned());
    }
    let theorems = profile.theorems.iter().filter(|theorem| theorem.path == input.theorem_path).collect::<Vec<_>>();
    if theorems.len() == 1 {
        let theorem = theorems[0];
        let observed = super::prefixed_identity(input.theorem_text.as_bytes());
        let declaration = format!("theorem {}", theorem.declaration_name);
        if theorem.approved_statement_blake3 != observed
            || !input.theorem_text.contains(&declaration)
            || input.theorem_text.contains("sorry")
            || input.theorem_text.contains("admit")
        {
            diagnostics.push("Aeneas consumer maintained theorem source or approved statement drifted".to_owned());
        }
    } else {
        diagnostics.push("Aeneas consumer theorem selection is missing or ambiguous".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_translation(item: &super::model::SelectedItem, text: &str, diagnostics: &mut Vec<String>) {
    let manifest = match serde_json::from_str::<TranslationManifest>(text) {
        Ok(manifest) => manifest,
        Err(error) => {
            diagnostics.push(format!("parse Aeneas consumer translation manifest: {error}"));
            return;
        }
    };
    let matches = manifest
        .functions
        .iter()
        .filter(|function| {
            function.rust_name == item.qualified_name
                && function.lean_name == item.generated_name
                && function.is_local
                && function.source.file == item.source_file
                && function.source.begin_line == item.begin_line
                && function.source.end_line == item.end_line
        })
        .count();
    if matches != 1 {
        diagnostics.push("Aeneas consumer translation manifest selection is stale or ambiguous".to_owned());
    }
}

fn validate_llbc(item: &super::model::SelectedItem, input: &ObservationInput<'_>, diagnostics: &mut Vec<String>) {
    let value = match serde_json::from_str::<Value>(input.llbc_text) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(format!("parse Aeneas consumer LLBC: {error}"));
            return;
        }
    };
    let translated = &value["translated"];
    let source_match = translated["files"].as_array().is_some_and(|files| {
        files.iter().any(|file| {
            file["name"]["Local"] == item.source_file && file["contents"].as_str() == Some(input.source_text)
        })
    });
    let function_match = translated["fun_decls"].as_array().is_some_and(|functions| {
        functions
            .iter()
            .filter(|function| {
                function["item_meta"]["source_text"]
                    .as_str()
                    .is_some_and(|source| source.trim_end() == input.source_text.trim_end())
                    && function["item_meta"]["span"]["data"]["beg"]["line"] == item.begin_line
                    && function["item_meta"]["span"]["data"]["end"]["line"] == item.end_line
            })
            .count()
            == 1
    });
    if value["has_errors"] != false {
        diagnostics.push("Aeneas consumer LLBC reports extraction errors".to_owned());
    }
    if !source_match {
        diagnostics.push("Aeneas consumer LLBC source-file bytes drifted".to_owned());
    }
    if !function_match {
        diagnostics.push("Aeneas consumer LLBC selected-item span drifted".to_owned());
    }
}

fn generated_declaration_present(generated_name: &str, text: &str) -> bool {
    text.lines().any(|line| line.trim_start().starts_with(&format!("def {generated_name} ")))
        || generated_name.rsplit_once('.').is_some_and(|(namespace, declaration)| {
            text.lines().any(|line| line.trim() == format!("namespace {namespace}"))
                && text.lines().any(|line| line.trim_start().starts_with(&format!("def {declaration} ")))
        })
}

fn stage_artifacts(input: &ObservationInput<'_>) -> Vec<ConsumerArtifactIdentity> {
    [
        ("rust-source", input.source_text),
        ("llbc", input.llbc_text),
        ("translation-manifest", input.translation_text),
        ("generated-lean", input.generated_lean_text),
        ("maintained-theorem", input.theorem_text),
    ]
    .into_iter()
    .map(|(role, text)| ConsumerArtifactIdentity {
        role: role.to_owned(),
        blake3: super::prefixed_identity(text.as_bytes()),
    })
    .collect()
}
