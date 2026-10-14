use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde_json::Value;

use super::model::ANALYSIS_SCHEMA;
use super::model::CharonSemanticProfile;
use super::model::EXPECTED_CHARON_REVISION;
use super::model::EXPECTED_CHARON_RUDRA_REVISION;
use super::model::EXPECTED_COMPATIBILITY_IDENTITY;
use super::model::EXPECTED_MECHANIZED_LLBC_REVISION;
use super::model::EXPECTED_PROVIDER_PROFILE_IDENTITY;
use super::model::EffectCandidate;
use super::model::PROFILE_ID;
use super::model::PROFILE_SCHEMA;
use super::model::PROJECTION_SCHEMA;
use super::model::SemanticAnalysis;
use super::model::SemanticEdge;
use super::model::SemanticFinding;
use super::model::SemanticItem;
use super::model::SemanticProjection;
use super::model::SourceIdentity;
use super::model::TargetMetadata;
use super::model::TraitResolutionRow;
use super::model::UnknownFact;

const EXPECTED_PROVIDER_PACKAGE: &str = "octet-aeneas-lean-provider-closure";
const EXPECTED_PROVIDER_PROFILE: &str = "octet-aeneas-lean-0.1.0-bc2eb6f";
const EXPECTED_COMPATIBILITY_MATRIX: &str = "octet-aeneas-lean-compatibility-0.1.0-bc2eb6f-cb50ff16";
const EXPECTED_EXECUTABLE_ENV: &str = "OCTET_AENEAS_CHARON";
const EXPECTED_SANDBOX_ENV: &str = "OCTET_AENEAS_NETWORK_SANDBOX";
const EXPECTED_SERIALIZATION_FORMAT: &str = "llbc-json-v1";
const EXPECTED_VERIFICATION_ROLE: &str = "recorded_only";
const BLAKE3_PREFIX: &str = "b3:";
const EXPECTED_FACT_KINDS: [&str; 10] = [
    "selected-item",
    "source-span",
    "local-call",
    "item-dependency",
    "trait-resolution",
    "control-flow-shape",
    "call-cycle",
    "opacity",
    "unsupported-marker",
    "target-metadata",
];
const EXPECTED_EFFECT_CANDIDATES: [&str; 4] = ["panic", "drop", "unsafe-boundary", "unknown-call"];
const REQUIRED_ARTIFACT_ROLES: [&str; 5] = [
    "raw-charon-snapshot",
    "normalized-semantic-projection",
    "bounded-semantic-analysis",
    "source-token-comparison",
    "recorded-only-receipt",
];
const REQUIRED_NON_CLAIM_MARKERS: [&str; 6] = [
    "Charon or rustc correctness",
    "complete Rust support",
    "effect absence",
    "formal-proof",
    "mechanized-llbc or charon-rudra",
    "Valence or Kamacite",
];
const KNOWN_STATEMENT_KINDS: [&str; 12] = [
    "StorageLive",
    "StorageDead",
    "Assign",
    "Return",
    "Call",
    "Drop",
    "Assert",
    "Nop",
    "Break",
    "Continue",
    "Switch",
    "Loop",
];

#[derive(Debug)]
pub(crate) struct CoreOutput {
    pub(crate) projection: SemanticProjection,
    pub(crate) analysis: SemanticAnalysis,
    pub(crate) source_identity: String,
    pub(crate) findings: Vec<SemanticFinding>,
}

#[derive(Clone, Debug)]
struct RawFunction {
    run_local_id: u64,
    qualified_name: String,
    source_file: String,
    begin_line: u64,
    end_line: u64,
    opacity: String,
    control_flow: String,
    statement_count: u64,
    body: Option<Value>,
    signature: Value,
    local_calls: Vec<u64>,
    unknown_statement_kinds: Vec<String>,
}

pub(crate) fn parse_profile(text: &str) -> Result<CharonSemanticProfile, String> {
    serde_json::from_str(text).map_err(|error| format!("parse Charon semantic profile: {error}"))
}

pub(crate) fn validate_profile(profile: &CharonSemanticProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    require_equal(&mut diagnostics, "schema_version", &profile.schema_version, PROFILE_SCHEMA);
    require_equal(&mut diagnostics, "profile_id", &profile.profile_id, PROFILE_ID);
    require_equal(&mut diagnostics, "provider.package", &profile.provider.package, EXPECTED_PROVIDER_PACKAGE);
    require_equal(&mut diagnostics, "provider.profile_id", &profile.provider.profile_id, EXPECTED_PROVIDER_PROFILE);
    require_equal(
        &mut diagnostics,
        "provider.profile_blake3",
        &format!("{BLAKE3_PREFIX}{}", profile.provider.profile_blake3),
        EXPECTED_PROVIDER_PROFILE_IDENTITY,
    );
    require_equal(
        &mut diagnostics,
        "provider.compatibility_matrix_id",
        &profile.provider.compatibility_matrix_id,
        EXPECTED_COMPATIBILITY_MATRIX,
    );
    require_equal(
        &mut diagnostics,
        "provider.compatibility_matrix_blake3",
        &format!("{BLAKE3_PREFIX}{}", profile.provider.compatibility_matrix_blake3),
        EXPECTED_COMPATIBILITY_IDENTITY,
    );
    require_equal(
        &mut diagnostics,
        "provider.charon_revision",
        &profile.provider.charon_revision,
        EXPECTED_CHARON_REVISION,
    );
    require_equal(
        &mut diagnostics,
        "provider.executable_env",
        &profile.provider.executable_env,
        EXPECTED_EXECUTABLE_ENV,
    );
    require_equal(
        &mut diagnostics,
        "provider.network_sandbox_env",
        &profile.provider.network_sandbox_env,
        EXPECTED_SANDBOX_ENV,
    );
    validate_profile_selection(profile, &mut diagnostics);
    validate_profile_extraction(profile, &mut diagnostics);
    validate_exact_vocabulary(&mut diagnostics, "fact_vocabulary", &profile.fact_vocabulary, &EXPECTED_FACT_KINDS);
    validate_exact_vocabulary(
        &mut diagnostics,
        "effect_candidate_vocabulary",
        &profile.effect_candidate_vocabulary,
        &EXPECTED_EFFECT_CANDIDATES,
    );
    validate_positive_limits(profile, &mut diagnostics);
    validate_exact_vocabulary(&mut diagnostics, "artifact_roles", &profile.artifact_roles, &REQUIRED_ARTIFACT_ROLES);
    validate_references(profile, &mut diagnostics);
    require_equal(&mut diagnostics, "verification_role", &profile.verification_role, EXPECTED_VERIFICATION_ROLE);
    for marker in REQUIRED_NON_CLAIM_MARKERS {
        if !profile.non_claims.iter().any(|non_claim| non_claim.contains(marker)) {
            diagnostics.push(format!("profile non-claims are missing required boundary marker `{marker}`"));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_profile_selection(profile: &CharonSemanticProfile, diagnostics: &mut Vec<String>) {
    if profile.source.root.trim().is_empty() || profile.source.crate_name.trim().is_empty() {
        diagnostics.push("profile source root and crate name must be non-empty".to_owned());
    }
    if profile.source.broad_selection_allowed {
        diagnostics.push("profile must reject broad source or item selection".to_owned());
    }
    if profile.source.selected_items.is_empty() {
        diagnostics.push("profile must select at least one exact item".to_owned());
    }
    let mut names = BTreeSet::new();
    for item in &profile.source.selected_items {
        if !names.insert(item.qualified_name.clone()) {
            diagnostics.push(format!("profile contains duplicate selected item `{}`", item.qualified_name));
        }
        if item.kind != "function" {
            diagnostics
                .push(format!("profile selected item `{}` has unsupported kind `{}`", item.qualified_name, item.kind));
        }
        if item.begin_line == 0 || item.end_line < item.begin_line {
            diagnostics.push(format!("profile selected item `{}` has an invalid source span", item.qualified_name));
        }
        if !super::is_b3_identity(&item.source_identity) {
            diagnostics
                .push(format!("profile selected item `{}` has a malformed source identity", item.qualified_name));
        }
    }
}

fn validate_profile_extraction(profile: &CharonSemanticProfile, diagnostics: &mut Vec<String>) {
    let expected_argv = [
        "charon",
        "cargo",
        "--preset",
        "aeneas",
        "--dest-file",
        "snapshot.llbc",
        "--start-from",
        "real_fixture::safe_add",
    ];
    if profile.extraction.command_argv.iter().map(String::as_str).collect::<Vec<_>>() != expected_argv {
        diagnostics.push("profile extraction argv must exactly reuse the reviewed Charon cargo preset".to_owned());
    }
    if profile.extraction.has_errors {
        diagnostics.push("profile must reject errorful Charon extraction".to_owned());
    }
    if profile.extraction.skip_borrowck || !profile.extraction.borrow_check_preserved {
        diagnostics.push("profile must preserve Charon borrow checking".to_owned());
    }
    if profile.extraction.preset != "Aeneas" {
        diagnostics.push("profile extraction preset must be `Aeneas`".to_owned());
    }
    if profile.extraction.serialization_format != EXPECTED_SERIALIZATION_FORMAT {
        diagnostics.push(format!("profile serialization format must be `{EXPECTED_SERIALIZATION_FORMAT}`"));
    }
    let expected_environment = ["PATH", "CHARON_TOOLCHAIN_IS_IN_PATH", "LD_LIBRARY_PATH", "HOME"];
    if profile.extraction.environment_allowlist.iter().map(String::as_str).collect::<Vec<_>>() != expected_environment {
        diagnostics.push("profile extraction environment allowlist drifted".to_owned());
    }
}

fn validate_positive_limits(profile: &CharonSemanticProfile, diagnostics: &mut Vec<String>) {
    if profile.limits.timeout_ms == 0
        || profile.limits.stdout_max_bytes == 0
        || profile.limits.stderr_max_bytes == 0
        || profile.limits.snapshot_max_bytes == 0
        || profile.limits.item_max_count == 0
        || profile.limits.edge_max_count == 0
    {
        diagnostics.push("profile limits must all be positive".to_owned());
    }
}

fn validate_references(profile: &CharonSemanticProfile, diagnostics: &mut Vec<String>) {
    let references = profile
        .references
        .iter()
        .map(|reference| (reference.repository.as_str(), reference))
        .collect::<BTreeMap<_, _>>();
    validate_reference(
        &references,
        "https://github.com/AeneasVerif/mechanized-llbc",
        EXPECTED_MECHANIZED_LLBC_REVISION,
        "README.md:Mechanized LLBC",
        diagnostics,
    );
    validate_reference(
        &references,
        "https://github.com/AeneasVerif/charon-rudra",
        EXPECTED_CHARON_RUDRA_REVISION,
        "src/main.rs:crate::rudra::lib::analyze",
        diagnostics,
    );
}

fn validate_reference(
    references: &BTreeMap<&str, &super::model::ReferenceMetadata>,
    repository: &str,
    revision: &str,
    sentinel: &str,
    diagnostics: &mut Vec<String>,
) {
    let Some(reference) = references.get(repository) else {
        diagnostics.push(format!("profile is missing recorded-only reference `{repository}`"));
        return;
    };
    if reference.revision != revision
        || reference.license != "Apache-2.0"
        || reference.source_sentinel != sentinel
        || reference.r#use.trim().is_empty()
        || !reference.recorded_only
    {
        diagnostics.push(format!("profile reference `{repository}` has stale or promoted metadata"));
    }
}

fn validate_exact_vocabulary(diagnostics: &mut Vec<String>, field: &str, observed: &[String], expected: &[&str]) {
    let observed = observed.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let expected = expected.iter().copied().collect::<BTreeSet<_>>();
    if observed != expected {
        diagnostics.push(format!("profile `{field}` does not match the closed reviewed vocabulary"));
    }
}

fn require_equal(diagnostics: &mut Vec<String>, field: &str, observed: &str, expected: &str) {
    if observed != expected {
        diagnostics.push(format!("profile `{field}` must be `{expected}`, observed `{observed}`"));
    }
}

pub(crate) fn analyze_snapshot(
    profile: &CharonSemanticProfile,
    snapshot_text: &str,
) -> Result<CoreOutput, Vec<String>> {
    let mut diagnostics = validate_profile(profile);
    if snapshot_text.len() > profile.limits.snapshot_max_bytes {
        diagnostics.push(format!(
            "Charon snapshot exceeds configured byte limit: {} > {}",
            snapshot_text.len(),
            profile.limits.snapshot_max_bytes
        ));
    }
    let snapshot = match serde_json::from_str::<Value>(snapshot_text) {
        Ok(snapshot) => snapshot,
        Err(error) => {
            diagnostics.push(format!("parse Charon snapshot: {error}"));
            return Err(sorted(diagnostics));
        }
    };
    let Some(translated) = snapshot.get("translated") else {
        diagnostics.push("Charon snapshot is missing `translated`".to_owned());
        return Err(sorted(diagnostics));
    };
    validate_extraction_observation(profile, &snapshot, translated, &mut diagnostics);
    let sources = project_sources(translated, &mut diagnostics);
    let source_identity = validate_selected_sources(profile, &sources, translated, &mut diagnostics);
    let raw_functions = project_raw_functions(translated, &sources, &mut diagnostics);
    validate_selected_items(profile, &raw_functions, &mut diagnostics);
    if raw_functions.len() > profile.limits.item_max_count {
        diagnostics.push(format!(
            "Charon item count exceeds configured limit: {} > {}",
            raw_functions.len(),
            profile.limits.item_max_count
        ));
    }
    let (items, edges, mut unknowns, mut candidates) = normalize_functions(&raw_functions, &mut diagnostics);
    let trait_rows = project_trait_rows(translated, &raw_functions, &mut diagnostics);
    if edges.len() > profile.limits.edge_max_count {
        diagnostics.push(format!(
            "Charon edge count exceeds configured limit: {} > {}",
            edges.len(),
            profile.limits.edge_max_count
        ));
    }
    validate_candidate_vocabulary(profile, &candidates, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    candidates.sort();
    candidates.dedup();
    unknowns.sort();
    unknowns.dedup();
    let cycles = call_cycles(&items, &edges);
    let status = if unknowns.is_empty() { "recorded" } else { "unknown" };
    let analysis = SemanticAnalysis {
        schema_version: ANALYSIS_SCHEMA.to_owned(),
        status: status.to_owned(),
        item_count: u64::try_from(items.len()).unwrap_or(u64::MAX),
        edge_count: u64::try_from(edges.len()).unwrap_or(u64::MAX),
        cycle_count: u64::try_from(cycles.len()).unwrap_or(u64::MAX),
        cycles,
        effect_candidates: candidates.clone(),
        unknowns: unknowns.clone(),
    };
    let mut findings = findings_for_analysis(&candidates, &unknowns, &analysis.cycles);
    findings.sort_by(|left, right| {
        (&left.family, &left.message, &left.file, left.line).cmp(&(
            &right.family,
            &right.message,
            &right.file,
            right.line,
        ))
    });
    let target = target_metadata(profile, translated).expect("validated target metadata must be present");
    let projection = SemanticProjection {
        schema_version: PROJECTION_SCHEMA.to_owned(),
        crate_name: profile.source.crate_name.clone(),
        charon_version: snapshot.get("charon_version").and_then(Value::as_str).unwrap_or("unknown").to_owned(),
        target,
        sources,
        items,
        edges,
        trait_rows,
    };
    Ok(CoreOutput {
        projection,
        analysis,
        source_identity,
        findings,
    })
}

fn validate_extraction_observation(
    profile: &CharonSemanticProfile,
    snapshot: &Value,
    translated: &Value,
    diagnostics: &mut Vec<String>,
) {
    if snapshot.get("has_errors").and_then(Value::as_bool) != Some(false) {
        diagnostics.push("Charon snapshot is partial or errorful".to_owned());
    }
    if translated.pointer("/options/skip_borrowck").and_then(Value::as_bool) != Some(false) {
        diagnostics.push("Charon snapshot skipped borrow checking".to_owned());
    }
    if translated.pointer("/options/preset").and_then(Value::as_str) != Some(profile.extraction.preset.as_str()) {
        diagnostics.push("Charon snapshot preset does not match the profile".to_owned());
    }
    if translated.pointer("/options/start_from_pub").and_then(Value::as_bool) == Some(true) {
        diagnostics.push("Charon snapshot used broad public-item selection".to_owned());
    }
    if translated.get("crate_name").and_then(Value::as_str) != Some(profile.source.crate_name.as_str()) {
        diagnostics.push("Charon snapshot crate name does not match the profile".to_owned());
    }
    if target_metadata(profile, translated).is_none() {
        diagnostics.push("Charon snapshot target metadata does not match the profile".to_owned());
    }
}

fn target_metadata(profile: &CharonSemanticProfile, translated: &Value) -> Option<TargetMetadata> {
    let targets = translated.get("target_information")?.as_array()?;
    let row = targets
        .iter()
        .find(|row| row.get("key").and_then(Value::as_str) == Some(&profile.extraction.target_triple))?;
    let pointer_width_bytes = row.pointer("/value/target_pointer_size")?.as_u64()?;
    let little_endian = row.pointer("/value/is_little_endian")?.as_bool()?;
    let endianness = if little_endian { "little" } else { "big" };
    if pointer_width_bytes != profile.extraction.pointer_width_bytes || endianness != profile.extraction.endianness {
        return None;
    }
    Some(TargetMetadata {
        triple: profile.extraction.target_triple.clone(),
        pointer_width_bytes,
        endianness: endianness.to_owned(),
    })
}

fn project_sources(translated: &Value, diagnostics: &mut Vec<String>) -> Vec<SourceIdentity> {
    let Some(files) = translated.get("files").and_then(Value::as_array) else {
        diagnostics.push("Charon snapshot is missing files".to_owned());
        return Vec::new();
    };
    let mut sources = files
        .iter()
        .filter_map(|file| {
            let path = file.pointer("/name/Local").and_then(Value::as_str)?;
            let contents = file.get("contents").and_then(Value::as_str)?;
            Some(SourceIdentity {
                path: path.to_owned(),
                identity: super::prefixed_identity(contents.as_bytes()),
            })
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.path.cmp(&right.path));
    sources.dedup();
    sources
}

fn validate_selected_sources(
    profile: &CharonSemanticProfile,
    sources: &[SourceIdentity],
    translated: &Value,
    diagnostics: &mut Vec<String>,
) -> String {
    let source_map = sources
        .iter()
        .map(|source| (source.path.as_str(), source.identity.as_str()))
        .collect::<BTreeMap<_, _>>();
    for item in &profile.source.selected_items {
        if source_map.get(item.source_file.as_str()).copied() != Some(item.source_identity.as_str()) {
            diagnostics.push(format!("selected source identity mismatch for `{}`", item.source_file));
        }
        validate_profile_span_against_source(item, translated, diagnostics);
    }
    profile.source.selected_items.first().map_or_else(String::new, |item| item.source_identity.clone())
}

fn validate_profile_span_against_source(
    item: &super::model::SelectedItem,
    translated: &Value,
    diagnostics: &mut Vec<String>,
) {
    let contents = translated
        .get("files")
        .and_then(Value::as_array)
        .and_then(|files| {
            files
                .iter()
                .find(|file| file.pointer("/name/Local").and_then(Value::as_str) == Some(&item.source_file))
        })
        .and_then(|file| file.get("contents"))
        .and_then(Value::as_str);
    let Some(contents) = contents else {
        return;
    };
    let line_count = u64::try_from(contents.lines().count()).unwrap_or(u64::MAX);
    if item.begin_line == 0 || item.end_line < item.begin_line || item.end_line > line_count {
        diagnostics.push(format!("profile source span for `{}` is stale", item.qualified_name));
    }
}

fn project_raw_functions(
    translated: &Value,
    sources: &[SourceIdentity],
    diagnostics: &mut Vec<String>,
) -> Vec<RawFunction> {
    let Some(functions) = translated.get("fun_decls").and_then(Value::as_array) else {
        diagnostics.push("Charon snapshot is missing function declarations".to_owned());
        return Vec::new();
    };
    let file_names = file_name_map(translated);
    let source_paths = sources.iter().map(|source| source.path.as_str()).collect::<BTreeSet<_>>();
    let mut projected = Vec::new();
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for function in functions.iter().filter(|function| !function.is_null()) {
        if function.pointer("/item_meta/is_local").and_then(Value::as_bool) != Some(true) {
            continue;
        }
        let Some(run_local_id) = function.get("def_id").and_then(Value::as_u64) else {
            diagnostics.push("local Charon function is missing def_id".to_owned());
            continue;
        };
        let qualified_name = qualified_name(function.pointer("/item_meta/name").and_then(Value::as_array));
        let span = function.pointer("/item_meta/span/data");
        let file_id = span.and_then(|span| span.get("file_id")).and_then(Value::as_u64);
        let source_file = file_id.and_then(|id| file_names.get(&id)).cloned().unwrap_or_default();
        let begin_line = span.and_then(|span| span.pointer("/beg/line")).and_then(Value::as_u64).unwrap_or_default();
        let end_line = span.and_then(|span| span.pointer("/end/line")).and_then(Value::as_u64).unwrap_or_default();
        let opacity = function.pointer("/item_meta/opacity").and_then(Value::as_str).unwrap_or("Unknown").to_owned();
        let body = function.get("body").filter(|body| !body.is_null()).cloned();
        let control_flow = body.as_ref().map_or("unsupported", |body| {
            if body.get("Structured").is_some() {
                "structured"
            } else if body.get("Unstructured").is_some() {
                "unstructured"
            } else {
                "unknown"
            }
        });
        let statement_count = body.as_ref().map_or(0, statement_count);
        let mut local_calls = Vec::new();
        if let Some(body) = &body {
            collect_local_calls(body, &mut local_calls);
        }
        local_calls.sort_unstable();
        local_calls.dedup();
        let mut unknown_statement_kinds = Vec::new();
        if let Some(body) = &body {
            collect_unknown_statement_kinds(body, &mut unknown_statement_kinds);
        }
        unknown_statement_kinds.sort();
        unknown_statement_kinds.dedup();
        if !ids.insert(run_local_id) {
            diagnostics.push(format!("duplicate local Charon function def_id `{run_local_id}`"));
        }
        if !names.insert(qualified_name.clone()) {
            diagnostics.push(format!("duplicate local Charon function name `{qualified_name}`"));
        }
        if qualified_name.is_empty()
            || !source_paths.contains(source_file.as_str())
            || begin_line == 0
            || end_line < begin_line
        {
            diagnostics.push(format!("local Charon function `{qualified_name}` has stale name or source span"));
        }
        projected.push(RawFunction {
            run_local_id,
            qualified_name,
            source_file,
            begin_line,
            end_line,
            opacity,
            control_flow: control_flow.to_owned(),
            statement_count,
            body,
            signature: function.get("signature").cloned().unwrap_or(Value::Null),
            local_calls,
            unknown_statement_kinds,
        });
    }
    projected.sort_by(|left, right| left.qualified_name.cmp(&right.qualified_name));
    projected
}

fn file_name_map(translated: &Value) -> BTreeMap<u64, String> {
    translated
        .get("files")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|file| {
            let id = file.get("id")?.as_u64()?;
            let path = file.pointer("/name/Local")?.as_str()?;
            Some((id, path.to_owned()))
        })
        .collect()
}

fn qualified_name(segments: Option<&Vec<Value>>) -> String {
    segments
        .into_iter()
        .flatten()
        .filter_map(|segment| {
            if let Some(identifier) = segment.pointer("/Ident/0").and_then(Value::as_str) {
                Some(identifier.to_owned())
            } else if segment.get("Impl").is_some() {
                Some("{impl}".to_owned())
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("::")
}

fn validate_selected_items(profile: &CharonSemanticProfile, functions: &[RawFunction], diagnostics: &mut Vec<String>) {
    let expected = profile
        .source
        .selected_items
        .iter()
        .map(|item| item.qualified_name.as_str())
        .collect::<BTreeSet<_>>();
    let observed = functions.iter().map(|function| function.qualified_name.as_str()).collect::<BTreeSet<_>>();
    if observed != expected {
        diagnostics.push(format!("exact selected item set mismatch: expected {expected:?}, observed {observed:?}"));
    }
    let profile_items = profile
        .source
        .selected_items
        .iter()
        .map(|item| (item.qualified_name.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    for function in functions {
        let Some(expected_item) = profile_items.get(function.qualified_name.as_str()) else {
            continue;
        };
        if function.source_file != expected_item.source_file
            || function.begin_line != expected_item.begin_line
            || function.end_line != expected_item.end_line
        {
            diagnostics.push(format!("selected item `{}` has stale source linkage", function.qualified_name));
        }
    }
}

fn normalize_functions(
    functions: &[RawFunction],
    diagnostics: &mut Vec<String>,
) -> (Vec<SemanticItem>, Vec<SemanticEdge>, Vec<UnknownFact>, Vec<EffectCandidate>) {
    let id_to_name = functions
        .iter()
        .map(|function| (function.run_local_id, function.qualified_name.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut items = Vec::new();
    let mut edges = Vec::new();
    let mut unknowns = Vec::new();
    let mut candidates = Vec::new();
    for function in functions {
        let stable_id = stable_item_id(function);
        let unsupported = function.body.is_none();
        items.push(SemanticItem {
            stable_id,
            run_local_id: function.run_local_id,
            qualified_name: function.qualified_name.clone(),
            kind: "function".to_owned(),
            source_file: function.source_file.clone(),
            begin_line: function.begin_line,
            end_line: function.end_line,
            opacity: function.opacity.clone(),
            control_flow: function.control_flow.clone(),
            statement_count: function.statement_count,
            unsupported,
        });
        if function.opacity != "Transparent" {
            unknowns.push(UnknownFact {
                item: function.qualified_name.clone(),
                fact: "opacity".to_owned(),
                reason: format!("selected item opacity is `{}`", function.opacity),
            });
        }
        if unsupported {
            unknowns.push(UnknownFact {
                item: function.qualified_name.clone(),
                fact: "unsupported-marker".to_owned(),
                reason: "selected item has no translated body".to_owned(),
            });
        }
        for kind in &function.unknown_statement_kinds {
            unknowns.push(UnknownFact {
                item: function.qualified_name.clone(),
                fact: "control-flow-shape".to_owned(),
                reason: format!("unknown explicit statement kind `{kind}`"),
            });
        }
        for target_id in &function.local_calls {
            let Some(target_name) = id_to_name.get(target_id) else {
                diagnostics.push(format!(
                    "local call from `{}` references missing function def_id `{target_id}`",
                    function.qualified_name
                ));
                candidates.push(EffectCandidate {
                    item: function.qualified_name.clone(),
                    candidate: "unknown-call".to_owned(),
                    evidence: format!("missing local function def_id {target_id}"),
                });
                continue;
            };
            edges.push(SemanticEdge {
                from: function.qualified_name.clone(),
                to: target_name.clone(),
                kind: "local-call".to_owned(),
            });
            edges.push(SemanticEdge {
                from: function.qualified_name.clone(),
                to: target_name.clone(),
                kind: "item-dependency".to_owned(),
            });
        }
        if function.signature.pointer("/is_unsafe").and_then(Value::as_bool) == Some(true) {
            candidates.push(EffectCandidate {
                item: function.qualified_name.clone(),
                candidate: "unsafe-boundary".to_owned(),
                evidence: "explicit unsafe function signature".to_owned(),
            });
        }
        if let Some(body) = &function.body {
            if contains_string(body, "Panic") || contains_object_key(body, "Assert") {
                candidates.push(EffectCandidate {
                    item: function.qualified_name.clone(),
                    candidate: "panic".to_owned(),
                    evidence: "explicit Charon panic/assert construct".to_owned(),
                });
            }
            if contains_object_key(body, "Drop") {
                candidates.push(EffectCandidate {
                    item: function.qualified_name.clone(),
                    candidate: "drop".to_owned(),
                    evidence: "explicit Charon drop construct".to_owned(),
                });
            }
            if contains_object_key(body, "RawPtr") {
                candidates.push(EffectCandidate {
                    item: function.qualified_name.clone(),
                    candidate: "unsafe-boundary".to_owned(),
                    evidence: "explicit Charon raw-pointer construct".to_owned(),
                });
            }
        }
    }
    items.sort_by(|left, right| left.qualified_name.cmp(&right.qualified_name));
    edges.sort();
    edges.dedup();
    (items, edges, unknowns, candidates)
}

fn stable_item_id(function: &RawFunction) -> String {
    let identity_input = format!(
        "function\0{}\0{}\0{}\0{}\0{}",
        function.qualified_name, function.source_file, function.begin_line, function.end_line, function.opacity
    );
    super::prefixed_identity(identity_input.as_bytes())
}

fn statement_count(value: &Value) -> u64 {
    match value {
        Value::Array(values) => values.iter().map(statement_count).sum(),
        Value::Object(values) => {
            let own = values
                .get("statements")
                .and_then(Value::as_array)
                .map_or(0, |statements| u64::try_from(statements.len()).unwrap_or(u64::MAX));
            own.saturating_add(values.values().map(statement_count).sum())
        }
        _ => 0,
    }
}

fn collect_local_calls(value: &Value, calls: &mut Vec<u64>) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_local_calls(value, calls);
            }
        }
        Value::Object(values) => {
            let call_id = values
                .get("Call")
                .and_then(|call| call.pointer("/call/func/Regular/kind/Fun/Regular"))
                .and_then(Value::as_u64);
            if let Some(id) = call_id {
                calls.push(id);
            }
            for value in values.values() {
                collect_local_calls(value, calls);
            }
        }
        _ => {}
    }
}

fn collect_unknown_statement_kinds(value: &Value, unknowns: &mut Vec<String>) {
    match value {
        Value::Array(values) => {
            for value in values {
                collect_unknown_statement_kinds(value, unknowns);
            }
        }
        Value::Object(values) => {
            let statement_kind = values
                .get("id")
                .and_then(Value::as_u64)
                .and_then(|_| values.get("kind"))
                .and_then(Value::as_object)
                .and_then(|kind| kind.keys().next());
            match statement_kind {
                Some(kind) if !KNOWN_STATEMENT_KINDS.contains(&kind.as_str()) => unknowns.push(kind.clone()),
                Some(_) | None => {}
            }
            for value in values.values() {
                collect_unknown_statement_kinds(value, unknowns);
            }
        }
        _ => {}
    }
}

fn contains_string(value: &Value, needle: &str) -> bool {
    match value {
        Value::String(text) => text == needle,
        Value::Array(values) => values.iter().any(|value| contains_string(value, needle)),
        Value::Object(values) => values.values().any(|value| contains_string(value, needle)),
        _ => false,
    }
}

fn contains_object_key(value: &Value, needle: &str) -> bool {
    match value {
        Value::Array(values) => values.iter().any(|value| contains_object_key(value, needle)),
        Value::Object(values) => {
            values.contains_key(needle) || values.values().any(|value| contains_object_key(value, needle))
        }
        _ => false,
    }
}

fn project_trait_rows(
    translated: &Value,
    functions: &[RawFunction],
    diagnostics: &mut Vec<String>,
) -> Vec<TraitResolutionRow> {
    let trait_names = translated
        .get("trait_decls")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|row| !row.is_null())
        .filter_map(|row| {
            let id = row.get("def_id")?.as_u64()?;
            let name = qualified_name(row.pointer("/item_meta/name").and_then(Value::as_array));
            Some((id, name))
        })
        .collect::<BTreeMap<_, _>>();
    let function_names = functions
        .iter()
        .map(|function| (function.run_local_id, function.qualified_name.clone()))
        .collect::<BTreeMap<_, _>>();
    let mut rows = Vec::new();
    for implementation in translated.get("trait_impls").and_then(Value::as_array).into_iter().flatten() {
        if implementation.is_null()
            || implementation.pointer("/item_meta/is_local").and_then(Value::as_bool) != Some(true)
        {
            continue;
        }
        let trait_id = implementation.pointer("/impl_trait/id").and_then(Value::as_u64);
        let Some(trait_name) = trait_id.and_then(|id| trait_names.get(&id)).cloned() else {
            diagnostics.push("local Charon trait implementation references a missing trait declaration".to_owned());
            continue;
        };
        let implementation_name = qualified_name(implementation.pointer("/item_meta/name").and_then(Value::as_array));
        let mut method_items = implementation
            .get("methods")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter_map(|method| method.pointer("/skip_binder/id").and_then(Value::as_u64))
            .filter_map(|id| function_names.get(&id).cloned())
            .collect::<Vec<_>>();
        method_items.sort();
        method_items.dedup();
        rows.push(TraitResolutionRow {
            trait_name,
            implementation_name,
            method_items,
        });
    }
    rows.sort();
    rows.dedup();
    rows
}

fn validate_candidate_vocabulary(
    profile: &CharonSemanticProfile,
    candidates: &[EffectCandidate],
    diagnostics: &mut Vec<String>,
) {
    let allowed = profile.effect_candidate_vocabulary.iter().map(String::as_str).collect::<BTreeSet<_>>();
    for candidate in candidates {
        if !allowed.contains(candidate.candidate.as_str()) {
            diagnostics.push(format!("unknown effect candidate `{}`", candidate.candidate));
        }
    }
}

fn call_cycles(items: &[SemanticItem], edges: &[SemanticEdge]) -> Vec<Vec<String>> {
    let graph = edges.iter().filter(|edge| edge.kind == "local-call").fold(
        BTreeMap::<String, Vec<String>>::new(),
        |mut graph, edge| {
            graph.entry(edge.from.clone()).or_default().push(edge.to.clone());
            graph
        },
    );
    let mut cycles = BTreeSet::new();
    for item in items {
        let mut path = Vec::new();
        find_cycles(&item.qualified_name, &graph, &mut path, &mut cycles);
    }
    cycles.into_iter().collect()
}

fn find_cycles(
    node: &str,
    graph: &BTreeMap<String, Vec<String>>,
    path: &mut Vec<String>,
    cycles: &mut BTreeSet<Vec<String>>,
) {
    if let Some(position) = path.iter().position(|entry| entry == node) {
        let mut cycle = path[position..].to_vec();
        cycle.push(node.to_owned());
        cycles.insert(canonical_cycle(cycle));
        return;
    }
    path.push(node.to_owned());
    if let Some(targets) = graph.get(node) {
        for target in targets {
            find_cycles(target, graph, path, cycles);
        }
    }
    let removed = path.pop();
    assert!(removed.is_some(), "cycle traversal path must remain balanced");
}

fn canonical_cycle(mut cycle: Vec<String>) -> Vec<String> {
    if cycle.len() <= 2 {
        return cycle;
    }
    let closing = cycle.pop();
    assert!(closing.is_some(), "cycle must have a closing node");
    let Some((index, _)) = cycle.iter().enumerate().min_by_key(|(_, value)| *value) else {
        return cycle;
    };
    cycle.rotate_left(index);
    if let Some(first) = cycle.first().cloned() {
        cycle.push(first);
    }
    cycle
}

fn findings_for_analysis(
    candidates: &[EffectCandidate],
    unknowns: &[UnknownFact],
    cycles: &[Vec<String>],
) -> Vec<SemanticFinding> {
    let mut findings = candidates
        .iter()
        .map(|candidate| SemanticFinding {
            family: "effect-candidate".to_owned(),
            severity: "manual_review".to_owned(),
            message: format!("{}: {}", candidate.candidate, candidate.evidence),
            file: None,
            line: None,
        })
        .collect::<Vec<_>>();
    findings.extend(unknowns.iter().map(|unknown| SemanticFinding {
        family: "unknown-fact".to_owned(),
        severity: "manual_review".to_owned(),
        message: format!("{}: {}", unknown.fact, unknown.reason),
        file: None,
        line: None,
    }));
    findings.extend(cycles.iter().map(|cycle| SemanticFinding {
        family: "call-cycle".to_owned(),
        severity: "manual_review".to_owned(),
        message: format!("resolved local call cycle: {}", cycle.join(" -> ")),
        file: None,
        line: None,
    }));
    findings
}

fn sorted(mut diagnostics: Vec<String>) -> Vec<String> {
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    const PROFILE_JSON: &str = include_str!("../../../standards/charon-semantic/generated/profile.json");
    const SNAPSHOT_JSON: &str =
        include_str!("../../../standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/llbc.llbc");
    const SELF_CALL_ID: u64 = 0;
    const MISSING_CALL_ID: u64 = 99;

    fn profile() -> CharonSemanticProfile {
        parse_profile(PROFILE_JSON).expect("embedded test profile must parse")
    }

    fn snapshot() -> Value {
        serde_json::from_str(SNAPSHOT_JSON).expect("frozen Charon snapshot must parse")
    }

    fn rendered(value: &Value) -> String {
        serde_json::to_string(value).expect("snapshot mutation must serialize")
    }

    fn add_call(snapshot: &mut Value, target_id: u64) {
        snapshot["translated"]["fun_decls"][0]["body"]["Structured"]["body"]["statements"] = serde_json::json!([{
            "id": 0,
            "kind": {
                "Call": {
                    "call": {
                        "func": { "Regular": { "kind": { "Fun": { "Regular": target_id } } } }
                    }
                }
            }
        }]);
    }

    #[test]
    fn exact_snapshot_projects_deterministically() {
        let profile = profile();
        let first = analyze_snapshot(&profile, SNAPSHOT_JSON).expect("positive snapshot must pass");
        let second = analyze_snapshot(&profile, SNAPSHOT_JSON).expect("positive snapshot must replay");
        assert_eq!(first.projection, second.projection);
        assert_eq!(first.analysis, second.analysis);
        assert_eq!(first.projection.items.len(), 1);
        assert!(first.projection.edges.is_empty());
        assert!(first.analysis.effect_candidates.iter().any(|candidate| candidate.candidate == "panic"));
    }

    #[test]
    fn rejects_partial_borrow_skipped_target_and_source_drift() {
        let profile = profile();
        let mut value = snapshot();
        value["has_errors"] = Value::Bool(true);
        value["translated"]["options"]["skip_borrowck"] = Value::Bool(true);
        value["translated"]["target_information"][0]["value"]["target_pointer_size"] = Value::from(4_u64);
        value["translated"]["files"][0]["contents"] = Value::String("pub fn drift() {}\n".to_owned());
        let diagnostics = analyze_snapshot(&profile, &rendered(&value)).expect_err("drift must fail");
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("errorful")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("borrow checking")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("target metadata")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("source identity mismatch")));
    }

    #[test]
    fn rejects_missing_call_endpoint() {
        let profile = profile();
        let mut value = snapshot();
        add_call(&mut value, MISSING_CALL_ID);
        let diagnostics = analyze_snapshot(&profile, &rendered(&value)).expect_err("missing call endpoint must fail");
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("missing function def_id")));
    }

    #[test]
    fn records_opaque_unsupported_and_unknown_statement_facts() {
        let profile = profile();
        let mut value = snapshot();
        value["translated"]["fun_decls"][0]["item_meta"]["opacity"] = Value::String("Opaque".to_owned());
        value["translated"]["fun_decls"][0]["body"] = serde_json::json!({
            "Structured": {
                "body": {
                    "statements": [{ "id": 0, "kind": { "MagicEffect": null } }]
                }
            }
        });
        let output = analyze_snapshot(&profile, &rendered(&value)).expect("unknown facts remain recorded");
        assert_eq!(output.analysis.status, "unknown");
        assert!(output.analysis.unknowns.iter().any(|unknown| unknown.fact == "opacity"));
        assert!(output.analysis.unknowns.iter().any(|unknown| unknown.reason.contains("MagicEffect")));
    }

    #[test]
    fn records_resolved_call_cycles_without_promoting_them() {
        let profile = profile();
        let mut value = snapshot();
        add_call(&mut value, SELF_CALL_ID);
        let output = analyze_snapshot(&profile, &rendered(&value)).expect("resolved self-call is valid");
        assert_eq!(output.analysis.cycle_count, 1);
        assert_eq!(output.analysis.cycles[0], vec!["real_fixture::safe_add", "real_fixture::safe_add"]);
    }

    #[test]
    fn rejects_broad_or_duplicate_profile_selection() {
        let mut profile = profile();
        profile.source.broad_selection_allowed = true;
        profile.source.selected_items.push(profile.source.selected_items[0].clone());
        let diagnostics = validate_profile(&profile);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("broad")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate selected")));
    }
}
