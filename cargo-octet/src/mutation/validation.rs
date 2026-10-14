use super::*;

const SOURCE_MANIFEST_SCHEMA: &str = "octet-source-manifest/v1";
const NATIVE_BASELINE_SCENARIO: &str = "Baseline";
pub(super) const NATIVE_SUCCESS: &str = "Success";
const ADJACENT_ITEMS: usize = 2;
const DIAGNOSTIC_CAPACITY: usize = 16;

#[derive(serde::Deserialize)]
struct SourceManifest {
    schema_version: String,
    files: Vec<SourceFile>,
}

#[derive(serde::Deserialize)]
struct SourceFile {
    path: String,
    blake3: String,
}

#[derive(serde::Deserialize)]
pub(super) struct NativeMutant {
    pub(super) name: String,
    pub(super) package: String,
    pub(super) file: String,
    pub(super) replacement: String,
    pub(super) genre: String,
    pub(super) span: NativeSpan,
}

#[derive(serde::Deserialize)]
pub(super) struct NativeSpan {
    pub(super) start: NativePosition,
}

#[derive(serde::Deserialize)]
pub(super) struct NativePosition {
    pub(super) line: u64,
    pub(super) column: u64,
}

#[derive(serde::Deserialize)]
struct NativeOutcomes {
    outcomes: Vec<NativeOutcome>,
}

#[derive(serde::Deserialize)]
pub(super) struct NativeOutcome {
    pub(super) scenario: serde_json::Value,
    pub(super) summary: String,
    pub(super) log_path: String,
    pub(super) diff_path: Option<String>,
}

pub(super) fn evaluate(input: &request::ProfiledInput, profile: &Profile) -> Evaluation {
    let mut diagnostics = validate_profile(profile);
    let source_files = match source_files(&input.source_manifest_text, profile) {
        Ok(value) => value,
        Err(error) => return failed(error),
    };
    let inventory = match parse_inventory(&input.inventory_text, profile) {
        Ok(value) => value,
        Err(error) => return failed(error),
    };
    let outcomes = match parse_outcomes(&input.outcomes_text, profile) {
        Ok(value) => value,
        Err(error) => return failed(error),
    };
    if let Err(errors) = records::validate_artifacts(&input.artifacts, profile) {
        return failed(errors);
    }
    let (baseline, by_name) = match partition_outcomes(outcomes, &inventory) {
        Ok(value) => value,
        Err(error) => return failed(error),
    };
    let baseline = match records::baseline_record(&baseline, &input.artifacts, profile) {
        Ok(value) => value,
        Err(error) => return failed(vec![error]),
    };
    if baseline.outcome != "passed" {
        diagnostics.push("cargo-mutants unmutated baseline did not pass".to_owned());
    }
    let mut mutants = Vec::with_capacity(inventory.len());
    for native in inventory {
        let outcome = match by_name.get(&native.name) {
            Some(value) => value,
            None => return failed(vec![format!("mutation outcome is missing for `{}`", native.name)]),
        };
        let source_identity = match source_files.get(&native.file) {
            Some(value) => value.clone(),
            None => return failed(vec![format!("mutation source identity is missing for `{}`", native.file)]),
        };
        match records::mutant_record(native, outcome, source_identity, &input.artifacts, profile) {
            Ok(value) => mutants.push(value),
            Err(error) => return failed(vec![error]),
        }
    }
    finalize(input, profile, diagnostics, baseline, mutants)
}

fn finalize(
    input: &request::ProfiledInput,
    profile: &Profile,
    mut diagnostics: Vec<String>,
    baseline: BaselineRecord,
    mut mutants: Vec<MutantRecord>,
) -> Evaluation {
    mutants.sort_by(|left, right| left.stable_id.cmp(&right.stable_id));
    if let Some(error) = records::duplicate_id(&mutants) {
        diagnostics.push(error);
    }
    let outcome_counts = records::outcome_counts(&mutants);
    let gate_status = records::gate_status(&outcome_counts, &baseline.outcome);
    diagnostics.extend(records::policy_diagnostics(&gate_status));
    let projection_identity = match records::projection_identity(&mutants) {
        Ok(value) => value,
        Err(error) => return failed(vec![error]),
    };
    diagnostics.sort();
    diagnostics.dedup();
    Evaluation {
        details: Some(ReceiptDetails {
            schema_version: RECEIPT_SCHEMA.to_owned(),
            profile_id: profile.profile_id.clone(),
            profile_identity: PROFILE_IDENTITY.to_owned(),
            provider_identity: format!(
                "{}@{}:{}",
                profile.provider.package, profile.provider.version, profile.provider.closure_identity
            ),
            rust_toolchain: profile.rust_toolchain.clone(),
            command_argv: profile.command_argv.clone(),
            source_manifest_identity: prefixed_identity(input.source_manifest_text.as_bytes()),
            native_outcomes_identity: prefixed_identity(input.outcomes_text.as_bytes()),
            native_inventory_identity: prefixed_identity(input.inventory_text.as_bytes()),
            projection_identity,
            baseline,
            mutants,
            outcome_counts,
            gate_status,
            evidence_tier: profile.evidence_tier.clone(),
            non_claims: profile
                .non_claims
                .iter()
                .cloned()
                .chain(std::iter::once(REQUIRED_NON_CLAIM.to_owned()))
                .collect(),
        }),
        diagnostics,
    }
}

pub(super) fn validate_details(details: &ReceiptDetails) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(DIAGNOSTIC_CAPACITY);
    if details.schema_version != RECEIPT_SCHEMA || details.profile_id != PROFILE_ID {
        diagnostics.push("mutation receipt schema or profile is unsupported".to_owned());
    }
    if details.profile_identity != PROFILE_IDENTITY {
        diagnostics.push("mutation receipt profile identity is stale".to_owned());
    }
    let profile = match selected_profile(&details.profile_id) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(error);
            return diagnostics;
        }
    };
    if details.command_argv != profile.command_argv {
        diagnostics.push("mutation receipt command does not match the checked profile".to_owned());
    }
    let counts = records::outcome_counts(&details.mutants);
    if counts != details.outcome_counts
        || records::gate_status(&counts, &details.baseline.outcome) != details.gate_status
    {
        diagnostics.push("mutation receipt gate decision is stale".to_owned());
    }
    match projection_bytes(details) {
        Ok(bytes) if prefixed_identity(&bytes) == details.projection_identity => {}
        Ok(_) => diagnostics.push("mutation receipt projection identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    if details.evidence_tier != "transcript_assertion"
        || !details.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM)
    {
        diagnostics.push("mutation receipt evidence tier or non-claim is unsupported".to_owned());
    }
    diagnostics
}

fn validate_profile(profile: &Profile) -> Vec<String> {
    let mut diagnostics = Vec::with_capacity(DIAGNOSTIC_CAPACITY);
    if profile.profile_id != PROFILE_ID || profile.provider.package != "cargo-mutants" {
        diagnostics.push("mutation profile or provider identity is unsupported".to_owned());
    }
    if profile.provider.version != "27.0.0" || profile.evidence_tier != "transcript_assertion" {
        diagnostics.push("mutation provider version or evidence tier is unsupported".to_owned());
    }
    if !profile.retain_logs || !profile.retain_diffs || profile.non_claims.is_empty() {
        diagnostics.push("mutation retention policy or non-claims are incomplete".to_owned());
    }
    diagnostics
}

fn source_files(text: &str, profile: &Profile) -> Result<std::collections::BTreeMap<String, String>, Vec<String>> {
    let manifest: SourceManifest = serde_json::from_str(text)
        .map_err(|error| vec![format!("mutation source manifest is malformed JSON: {error}")])?;
    if manifest.schema_version != SOURCE_MANIFEST_SCHEMA || manifest.schema_version != profile.source_manifest_schema {
        return Err(vec!["mutation source manifest schema is unsupported".to_owned()]);
    }
    let mut files = std::collections::BTreeMap::new();
    for file in manifest.files {
        if !records::safe_path(&file.path) || !records::is_b3(&file.blake3) {
            return Err(vec!["mutation source manifest contains an unsafe path or identity".to_owned()]);
        }
        if files.insert(file.path, file.blake3).is_some() {
            return Err(vec!["mutation source manifest contains a duplicate path".to_owned()]);
        }
    }
    if files.is_empty() {
        return Err(vec!["mutation source manifest is empty".to_owned()]);
    }
    Ok(files)
}

fn parse_inventory(text: &str, profile: &Profile) -> Result<Vec<NativeMutant>, Vec<String>> {
    let mut inventory: Vec<NativeMutant> =
        serde_json::from_str(text).map_err(|error| vec![format!("mutation inventory is malformed JSON: {error}")])?;
    if inventory.is_empty() || inventory.len() > profile.max_mutants {
        return Err(vec!["mutation inventory count is outside profile limits".to_owned()]);
    }
    inventory.sort_by(|left, right| left.name.cmp(&right.name));
    if inventory.windows(ADJACENT_ITEMS).any(|items| items[0].name == items[1].name) {
        return Err(vec!["mutation inventory contains a duplicate name".to_owned()]);
    }
    Ok(inventory)
}

fn parse_outcomes(text: &str, profile: &Profile) -> Result<NativeOutcomes, Vec<String>> {
    let outcomes: NativeOutcomes =
        serde_json::from_str(text).map_err(|error| vec![format!("mutation outcomes are malformed JSON: {error}")])?;
    if outcomes.outcomes.is_empty() || outcomes.outcomes.len() > profile.max_mutants + 1 {
        return Err(vec!["mutation outcome count is outside profile limits".to_owned()]);
    }
    Ok(outcomes)
}

fn partition_outcomes(
    outcomes: NativeOutcomes,
    inventory: &[NativeMutant],
) -> Result<(NativeOutcome, std::collections::BTreeMap<String, NativeOutcome>), Vec<String>> {
    let mut baseline = None;
    let mut mutants = std::collections::BTreeMap::new();
    for outcome in outcomes.outcomes {
        if outcome.scenario.as_str() == Some(NATIVE_BASELINE_SCENARIO) {
            if baseline.replace(outcome).is_some() {
                return Err(vec!["mutation outcomes contain duplicate baselines".to_owned()]);
            }
            continue;
        }
        let name = outcome
            .scenario
            .pointer("/Mutant/name")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| vec!["mutation outcome scenario is unsupported".to_owned()])?
            .to_owned();
        if mutants.insert(name, outcome).is_some() {
            return Err(vec!["mutation outcomes contain a duplicate mutant".to_owned()]);
        }
    }
    if mutants.len() != inventory.len() {
        return Err(vec!["mutation outcomes do not partition the complete inventory".to_owned()]);
    }
    Ok((
        baseline.ok_or_else(|| vec!["mutation outcomes are missing the unmutated baseline".to_owned()])?,
        mutants,
    ))
}

fn failed(mut diagnostics: Vec<String>) -> Evaluation {
    diagnostics.sort();
    diagnostics.dedup();
    Evaluation {
        details: None,
        diagnostics,
    }
}
