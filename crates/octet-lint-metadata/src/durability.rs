//! Declared durability-boundary profile and vocabulary.
//!
//! Consumers author the profile in Nickel under `standards/durability-claims/`
//! (or a copy in their own repository), export deterministic JSON, and point
//! `durability_profile_path` at that generated file. This module owns the Rust
//! projection and the fail-closed validation of the generated catalog and
//! profile; it performs no file or process access beyond reading the embedded
//! catalog.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

pub const DURABILITY_CATALOG_SCHEMA_VERSION: &str = "octet-durability-claims-catalog/v1";
pub const DURABILITY_PROFILE_SCHEMA_VERSION: &str = "octet-durability-claims-profile/v1";
pub const OBSERVATION_SYNCHRONIZATION: &str = "synchronization";
pub const OBSERVATION_ADAPTER: &str = "adapter-observation";

const DURABILITY_CATALOG_JSON: &str = include_str!("../../../standards/durability-claims/generated/catalog.json");

/// One rendered guidance entry for a durability finding.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DurabilityGuidance {
    pub finding: String,
    pub repair: String,
}

/// The checked-in durability vocabulary: observation classes, surfaces,
/// recovery operations, finding guidance, and claim non-claims.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DurabilityCatalog {
    pub schema_version: String,
    pub observation_classes: Vec<String>,
    pub synchronization_surfaces: Vec<String>,
    pub flush_surfaces: Vec<String>,
    pub recovery_operations: Vec<String>,
    pub findings: Vec<String>,
    pub repair_options: Vec<String>,
    pub guidance: Vec<DurabilityGuidance>,
    pub non_claims: Vec<String>,
}

/// One consumer-declared durable boundary.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DurabilityBoundary {
    pub boundary_id: String,
    pub source_scope: String,
    pub boundary_functions: Vec<String>,
    pub required_observation: String,
    pub uncertain_outcome_types: Vec<String>,
    #[serde(default)]
    pub commit_surfaces: Vec<String>,
    #[serde(default)]
    pub adapter_observations: Vec<String>,
}

/// One consumer-declared durability profile.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DurabilityProfile {
    pub schema_version: String,
    pub profile_id: String,
    pub boundaries: Vec<DurabilityBoundary>,
    pub non_claims: Vec<String>,
}

/// Loads the embedded durability catalog.
///
/// # Errors
///
/// Returns deterministic diagnostics when the generated JSON is malformed or
/// violates the Rust projection of the Nickel contract.
pub fn load_durability_catalog() -> Result<DurabilityCatalog, String> {
    parse_durability_catalog(DURABILITY_CATALOG_JSON)
}

/// Parses and validates the durability catalog without file access.
///
/// # Errors
///
/// Returns deterministic diagnostics for malformed or invalid catalog data.
pub fn parse_durability_catalog(text: &str) -> Result<DurabilityCatalog, String> {
    let catalog = serde_json::from_str::<DurabilityCatalog>(text)
        .map_err(|error| format!("parse durability catalog: {error}"))?;
    let diagnostics = durability_catalog_diagnostics(&catalog);
    if diagnostics.is_empty() {
        return Ok(catalog);
    }

    Err(format!("invalid durability catalog: {}", diagnostics.join("; ")))
}

/// Parses a generated durability profile and validates it against `catalog`.
///
/// # Errors
///
/// Returns deterministic diagnostics for malformed JSON, unknown fields, unknown
/// observation classes, missing uncertain-outcome declarations, and duplicate
/// boundary identities.
pub fn parse_durability_profile(text: &str, catalog: &DurabilityCatalog) -> Result<DurabilityProfile, String> {
    let profile = serde_json::from_str::<DurabilityProfile>(text)
        .map_err(|error| format!("parse durability profile: {error}"))?;
    let diagnostics = durability_profile_diagnostics(&profile, catalog);
    if diagnostics.is_empty() {
        return Ok(profile);
    }

    Err(format!("invalid durability profile: {}", diagnostics.join("; ")))
}

/// Deterministic catalog diagnostics, empty when the catalog is valid.
#[must_use]
pub fn durability_catalog_diagnostics(catalog: &DurabilityCatalog) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if catalog.schema_version != DURABILITY_CATALOG_SCHEMA_VERSION {
        diagnostics.push(format!(
            "schema_version must be `{DURABILITY_CATALOG_SCHEMA_VERSION}`, got `{}`",
            catalog.schema_version
        ));
    }
    for (field, values) in [
        ("observation_classes", &catalog.observation_classes),
        ("synchronization_surfaces", &catalog.synchronization_surfaces),
        ("flush_surfaces", &catalog.flush_surfaces),
        ("recovery_operations", &catalog.recovery_operations),
        ("findings", &catalog.findings),
        ("repair_options", &catalog.repair_options),
        ("non_claims", &catalog.non_claims),
    ] {
        diagnostics.extend(non_empty_unique_diagnostics(field, values));
    }
    let mut guidance_findings = BTreeSet::new();
    for guidance in &catalog.guidance {
        if guidance.finding.trim().is_empty() || guidance.repair.trim().is_empty() {
            diagnostics.push("guidance entries must name a finding and a repair".to_owned());
            continue;
        }
        if !catalog.findings.contains(&guidance.finding) {
            diagnostics.push(format!("guidance names unknown finding `{}`", guidance.finding));
        }
        if !guidance_findings.insert(guidance.finding.as_str()) {
            diagnostics.push(format!("duplicate guidance for finding `{}`", guidance.finding));
        }
    }
    for finding in &catalog.findings {
        if !guidance_findings.contains(finding.as_str()) {
            diagnostics.push(format!("finding `{finding}` has no guidance entry"));
        }
    }
    diagnostics
}

/// Deterministic profile diagnostics, empty when the profile is valid.
#[must_use]
pub fn durability_profile_diagnostics(profile: &DurabilityProfile, catalog: &DurabilityCatalog) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if profile.schema_version != DURABILITY_PROFILE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "schema_version must be `{DURABILITY_PROFILE_SCHEMA_VERSION}`, got `{}`",
            profile.schema_version
        ));
    }
    if profile.profile_id.trim().is_empty() {
        diagnostics.push("profile_id must not be empty".to_owned());
    }
    diagnostics.extend(non_empty_unique_diagnostics("non_claims", &profile.non_claims));
    if profile.boundaries.is_empty() {
        diagnostics.push("boundaries must not be empty".to_owned());
    }
    let mut boundary_ids = BTreeSet::new();
    for boundary in &profile.boundaries {
        diagnostics.extend(boundary_diagnostics(boundary, catalog));
        if boundary.boundary_id.trim().is_empty() {
            diagnostics.push("boundary_id must not be empty".to_owned());
        } else if !boundary_ids.insert(boundary.boundary_id.as_str()) {
            diagnostics.push(format!("duplicate boundary_id `{}`", boundary.boundary_id));
        }
    }
    diagnostics
}

fn boundary_diagnostics(boundary: &DurabilityBoundary, catalog: &DurabilityCatalog) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let label = if boundary.boundary_id.trim().is_empty() {
        "<unnamed>".to_owned()
    } else {
        boundary.boundary_id.clone()
    };
    if boundary.source_scope.trim().is_empty() {
        diagnostics.push(format!("boundary `{label}` source_scope must not be empty"));
    }
    diagnostics.extend(non_empty_unique_diagnostics(
        &format!("boundary `{label}` boundary_functions"),
        &boundary.boundary_functions,
    ));
    diagnostics.extend(non_empty_unique_diagnostics(
        &format!("boundary `{label}` uncertain_outcome_types"),
        &boundary.uncertain_outcome_types,
    ));
    if !catalog.observation_classes.contains(&boundary.required_observation) {
        diagnostics.push(format!(
            "boundary `{label}` required_observation `{}` is not a declared observation class",
            boundary.required_observation
        ));
    }
    if boundary.required_observation == OBSERVATION_ADAPTER && boundary.adapter_observations.is_empty() {
        diagnostics
            .push(format!("boundary `{label}` requires an adapter observation but declares no adapter_observations"));
    }
    for (field, values) in [
        ("commit_surfaces", &boundary.commit_surfaces),
        ("adapter_observations", &boundary.adapter_observations),
    ] {
        if values.iter().any(|value| value.trim().is_empty()) {
            diagnostics.push(format!("boundary `{label}` {field} entries must not be empty"));
        }
        if duplicate_values(values) {
            diagnostics.push(format!("boundary `{label}` {field} has duplicate entries"));
        }
    }
    diagnostics
}

fn non_empty_unique_diagnostics(field: &str, values: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if values.is_empty() {
        diagnostics.push(format!("{field} must not be empty"));
        return diagnostics;
    }
    if values.iter().any(|value| value.trim().is_empty()) {
        diagnostics.push(format!("{field} entries must not be empty"));
    }
    if duplicate_values(values) {
        diagnostics.push(format!("{field} has duplicate entries"));
    }
    diagnostics
}

fn duplicate_values(values: &[String]) -> bool {
    let mut seen = BTreeSet::new();
    !values.iter().all(|value| seen.insert(value.as_str()))
}

impl DurabilityCatalog {
    /// Whether `name` is a declared observation class.
    #[must_use]
    pub fn is_observation_class(&self, name: &str) -> bool {
        self.observation_classes.iter().any(|class| class == name)
    }

    /// Whether `def_path` resolves to a declared synchronization surface.
    #[must_use]
    pub fn is_synchronization_surface(&self, def_path: &str) -> bool {
        self.synchronization_surfaces.iter().any(|surface| surface_matches(surface, def_path))
    }

    /// Whether `def_path` resolves to a declared flush surface.
    #[must_use]
    pub fn is_flush_surface(&self, def_path: &str) -> bool {
        self.flush_surfaces.iter().any(|surface| surface_matches(surface, def_path))
    }

    /// Whether `name` is a declared recovery operation.
    #[must_use]
    pub fn is_recovery_operation(&self, name: &str) -> bool {
        self.recovery_operations.iter().any(|operation| operation == name)
    }

    /// Shared bounded repair options rendered in every durability diagnostic.
    #[must_use]
    pub fn repair_options_text(&self) -> String {
        self.repair_options.join("; ")
    }

    /// Guidance repair text for `finding`, when declared.
    #[must_use]
    pub fn repair_for(&self, finding: &str) -> Option<&str> {
        self.guidance
            .iter()
            .find(|guidance| guidance.finding == finding)
            .map(|guidance| guidance.repair.as_str())
    }
}

impl DurabilityBoundary {
    /// Whether this boundary owns the resolved function `def_path`.
    #[must_use]
    pub fn claims_function(&self, def_path: &str) -> bool {
        self.boundary_functions.iter().any(|function| surface_matches(function, def_path))
    }

    /// Whether `def_path` resolves to one of this boundary's commit surfaces.
    #[must_use]
    pub fn is_commit_surface(&self, def_path: &str) -> bool {
        self.commit_surfaces.iter().any(|surface| surface_matches(surface, def_path))
    }

    /// Whether `def_path` resolves to one of this boundary's adapter observations.
    #[must_use]
    pub fn is_adapter_observation(&self, def_path: &str) -> bool {
        self.adapter_observations.iter().any(|surface| surface_matches(surface, def_path))
    }

    /// Whether `relative_path` lies inside this boundary's declared source scope.
    #[must_use]
    pub fn owns_source(&self, relative_path: &str) -> bool {
        path_in_scope(relative_path, &self.source_scope)
    }
}

impl DurabilityProfile {
    /// First declared boundary that owns `relative_path` and `def_path`.
    #[must_use]
    pub fn resolve(&self, relative_path: &str, def_path: &str) -> Option<&DurabilityBoundary> {
        self.boundaries
            .iter()
            .find(|boundary| boundary.owns_source(relative_path) && boundary.claims_function(def_path))
    }
}

/// Whole-component suffix match for a configured `::` path against a resolved
/// definition path, so `WriteTransaction::commit` matches
/// `redb::transactions::WriteTransaction::commit` without matching
/// `redb::transactions::WriteTransaction::commit_and_close`. The declared
/// trailing segments must be spelled exactly; intermediate module segments may
/// be omitted.
#[must_use]
pub fn surface_matches(configured: &str, def_path: &str) -> bool {
    // Resolved definition paths are crate-name-qualified, so a leading
    // `crate::` in a consumer declaration is dropped before the suffix check.
    let declared = configured_segments(configured.strip_prefix("crate::").unwrap_or(configured));
    let resolved = configured_segments(def_path);
    !declared.is_empty()
        && declared.len() <= resolved.len()
        && declared
            .iter()
            .rev()
            .zip(resolved.iter().rev())
            .all(|(configured, resolved)| configured == resolved)
}

/// Whole-component path-prefix match for a workspace-relative source path, so
/// `crates/core` covers `crates/core/src/lib.rs` but not
/// `crates/core-shell/src/lib.rs`.
#[must_use]
pub fn path_in_scope(relative_path: &str, scope: &str) -> bool {
    let scope = scope.trim().trim_matches('/');
    if scope.is_empty() {
        return false;
    }
    let relative_path = relative_path.trim().trim_start_matches("./");
    relative_path == scope || relative_path.strip_prefix(scope).is_some_and(|rest| rest.starts_with('/'))
}

fn configured_segments(path: &str) -> Vec<&str> {
    path.split("::").map(str::trim).filter(|segment| !segment.is_empty()).collect()
}

#[cfg(test)]
mod tests {
    use super::DURABILITY_PROFILE_SCHEMA_VERSION;
    use super::DurabilityCatalog;
    use super::durability_catalog_diagnostics;
    use super::load_durability_catalog;
    use super::parse_durability_catalog;
    use super::parse_durability_profile;
    use super::path_in_scope;
    use super::surface_matches;

    const SHIPPED_PROFILE: &str = include_str!("../../../standards/durability-claims/generated/profile.json");

    fn catalog() -> DurabilityCatalog {
        load_durability_catalog().expect("embedded durability catalog is valid")
    }

    fn valid_profile(observation: &str) -> String {
        format!(
            r#"{{
                "schema_version": "{DURABILITY_PROFILE_SCHEMA_VERSION}",
                "profile_id": "test-profile",
                "boundaries": [{{
                    "boundary_id": "persist",
                    "source_scope": "crates/store/src/",
                    "boundary_functions": ["crate::store::persist"],
                    "required_observation": "{observation}",
                    "uncertain_outcome_types": ["DurabilityOutcome"],
                    "commit_surfaces": ["redb::WriteTransaction::commit"],
                    "adapter_observations": ["durable_file_publication::Publication::commit"]
                }}],
                "non_claims": ["Bounded source-shape checks only."]
            }}"#
        )
    }

    #[test]
    fn embedded_catalog_is_valid_and_declares_every_finding() {
        let catalog = catalog();
        assert!(durability_catalog_diagnostics(&catalog).is_empty());
        for finding in [
            "discarded_sync_result",
            "flush_only_durability",
            "retry_sync_after_error",
        ] {
            assert!(catalog.findings.iter().any(|declared| declared == finding));
            assert!(catalog.repair_for(finding).is_some());
        }
        assert!(!catalog.non_claims.is_empty());
    }

    #[test]
    fn shipped_profile_parses_and_resolves_declared_boundaries() {
        let catalog = catalog();
        let profile = parse_durability_profile(SHIPPED_PROFILE, &catalog).expect("shipped profile is valid");
        assert!(profile.resolve("crates/store/src/wal.rs", "store::wal::checkpoint").is_some());
        assert!(profile.resolve("crates/publication/src/lib.rs", "publication::publish").is_some());
        assert!(profile.resolve("crates/store/src/wal.rs", "store::wal::read").is_none());
        assert!(profile.resolve("crates/other/src/wal.rs", "store::wal::checkpoint").is_none());
    }

    #[test]
    fn valid_profiles_for_both_observation_classes_are_accepted() {
        let catalog = catalog();
        for observation in ["synchronization", "adapter-observation"] {
            assert!(
                parse_durability_profile(&valid_profile(observation), &catalog).is_ok(),
                "observation class `{observation}` must be accepted"
            );
        }
    }

    #[test]
    fn unknown_observation_class_fails_closed() {
        let catalog = catalog();
        let error =
            parse_durability_profile(&valid_profile("best-effort"), &catalog).expect_err("unknown class rejected");
        assert!(error.contains("not a declared observation class"), "{error}");
    }

    #[test]
    fn missing_uncertainty_type_fails_closed() {
        let catalog = catalog();
        let text = valid_profile("synchronization").replace(r#"["DurabilityOutcome"]"#, "[]");
        let error = parse_durability_profile(&text, &catalog).expect_err("empty uncertainty rejected");
        assert!(error.contains("uncertain_outcome_types must not be empty"), "{error}");
    }

    #[test]
    fn adapter_observation_without_surface_fails_closed() {
        let catalog = catalog();
        let text = valid_profile("adapter-observation").replace(
            r#""adapter_observations": ["durable_file_publication::Publication::commit"]"#,
            r#""adapter_observations": []"#,
        );
        let error = parse_durability_profile(&text, &catalog).expect_err("missing adapter surface rejected");
        assert!(error.contains("declares no adapter_observations"), "{error}");
    }

    #[test]
    fn duplicate_boundary_and_unknown_field_fail_closed() {
        let catalog = catalog();
        let duplicate = valid_profile("synchronization").replace(
            r#""boundaries": [{"#,
            r#""boundaries": [{"boundary_id": "persist", "source_scope": "crates/other/src/", "boundary_functions": ["crate::other::persist"], "required_observation": "synchronization", "uncertain_outcome_types": ["DurabilityOutcome"]}, {"#,
        );
        let error = parse_durability_profile(&duplicate, &catalog).expect_err("duplicate boundary rejected");
        assert!(error.contains("duplicate boundary_id"), "{error}");

        let unknown_field = valid_profile("synchronization").replace(
            r#""non_claims": ["Bounded source-shape checks only."]"#,
            r#""non_claims": ["Bounded source-shape checks only."], "best_effort": true"#,
        );
        let error = parse_durability_profile(&unknown_field, &catalog).expect_err("unknown field rejected");
        assert!(error.contains("unknown field"), "{error}");
    }

    #[test]
    fn malformed_catalog_and_profile_json_fail_closed() {
        let error = parse_durability_catalog("{").expect_err("malformed catalog rejected");
        assert!(error.contains("parse durability catalog"), "{error}");
        let catalog = catalog();
        let error = parse_durability_profile("{", &catalog).expect_err("malformed profile rejected");
        assert!(error.contains("parse durability profile"), "{error}");
    }

    #[test]
    fn surface_matching_requires_whole_trailing_segments() {
        assert!(surface_matches("std::fs::File::sync_all", "std::fs::File::sync_all"));
        assert!(surface_matches("WriteTransaction::commit", "redb::transactions::WriteTransaction::commit"));
        assert!(surface_matches("crate::store::persist", "store::persist"));
        assert!(!surface_matches("redb::WriteTransaction::commit", "redb::transactions::WriteTransaction::commit"));
        assert!(!surface_matches("redb::WriteTransaction::commit", "redb::WriteTransaction::commit_and_close"));
        assert!(!surface_matches("commit", "redb::WriteTransaction::precommit"));
        assert!(!surface_matches("", "std::fs::File::sync_all"));
    }

    #[test]
    fn path_scope_matching_uses_whole_components() {
        assert!(path_in_scope("crates/core/src/lib.rs", "crates/core"));
        assert!(path_in_scope("crates/core", "crates/core/"));
        assert!(!path_in_scope("crates/core-shell/src/lib.rs", "crates/core"));
        assert!(!path_in_scope("crates/core/src/lib.rs", ""));
    }
}
