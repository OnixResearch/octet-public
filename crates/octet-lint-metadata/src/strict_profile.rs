use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::EFFECT_CONCURRENCY;
use crate::EFFECT_FAILURE;
use crate::EFFECT_RESOURCE;
use crate::EFFECT_TIME;
use crate::EFFECT_UNKNOWN;
use crate::metadata_for_lint;

pub const STRICT_PROFILE_ENV: &str = "OCTET_STRICT_PROFILE";
pub const STRICT_PROFILE_SCHEMA_VERSION: &str = "octet-strict-rust-profiles/v1";
pub const STRICT_PROFILE_CLAIM_SCOPE: &str = "direct-source-shape-policy";

const STRICT_PROFILE_CATALOG_JSON: &str =
    include_str!("../../../standards/strict-profiles/generated/strict-rust-profiles.json");
const SUPPORTED_EFFECT_FAMILIES: &[&str] = &[
    EFFECT_CONCURRENCY,
    EFFECT_FAILURE,
    EFFECT_RESOURCE,
    EFFECT_TIME,
    EFFECT_UNKNOWN,
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FunctionAddressModeFloor {
    Off,
    Observe,
    Warn,
    Deny,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FunctionAddressScope {
    Workspace,
    Core,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StrictProfile {
    pub profile_id: String,
    pub summary: String,
    pub required_lints: Vec<String>,
    pub effect_families: Vec<String>,
    pub requires_core_scopes: bool,
    pub minimum_function_address_mode: FunctionAddressModeFloor,
    pub function_address_scope: FunctionAddressScope,
    pub claim_scope: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct StrictProfileCatalog {
    pub schema_version: String,
    pub boolean_api_policy: BooleanApiPolicy,
    pub profiles: Vec<StrictProfile>,
}

/// Closed policy for the direct boolean-parameter lint, exported from Nickel.
///
/// The policy names the covered callable shapes, the private direct-boolean
/// minimum, the compatibility exclusions, the narrow expectation format, and
/// the prohibited broad suppressions. Validation fails closed before lint
/// execution when a value would weaken the full-catalog gate.
#[allow(
    clippy::struct_excessive_bools,
    reason = "the policy records independent prohibition flags, not mutually exclusive states"
)]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct BooleanApiPolicy {
    pub public_scope: Vec<String>,
    pub private_min_direct_bool_parameters: usize,
    pub parameter_type: String,
    pub skip_expansions: bool,
    pub skip_external_trait_impls: bool,
    pub exception_attribute: String,
    pub exception_reason_required: bool,
    pub exception_owner_required: bool,
    pub broad_suppression: String,
    pub finding_baseline: String,
    pub warning_budget: String,
    pub unknown_scope: String,
    pub non_claims: Vec<String>,
}

/// Returns the checked raw-boolean policy and a BLAKE3 identity of its canonical JSON.
///
/// # Errors
///
/// Returns a deterministic diagnostic if the embedded policy is invalid or its identity cannot be
/// serialized.
pub fn boolean_api_policy() -> Result<(BooleanApiPolicy, String), String> {
    let policy = load_strict_profile_catalog()?.boolean_api_policy;
    let identity = boolean_api_policy_identity(&policy)?;
    Ok((policy, identity))
}

fn boolean_api_policy_identity(policy: &BooleanApiPolicy) -> Result<String, String> {
    let canonical =
        serde_json::to_vec(policy).map_err(|error| format!("serialize boolean API policy identity: {error}"))?;
    Ok(format!("b3:{}", blake3::hash(&canonical).to_hex()))
}

fn validate_boolean_api_policy(policy: &BooleanApiPolicy, diagnostics: &mut Vec<String>) {
    let required_scopes = ["free-function", "inherent-method", "crate-owned-trait"];
    if policy.public_scope.iter().map(String::as_str).collect::<Vec<_>>() != required_scopes {
        diagnostics.push("boolean API policy public_scope must name the closed callable set".to_owned());
    }
    if policy.private_min_direct_bool_parameters != 2 || policy.parameter_type != "direct-bool" {
        diagnostics.push("boolean API policy must cover direct bool and two private parameters".to_owned());
    }
    if !policy.skip_expansions || !policy.skip_external_trait_impls {
        diagnostics.push("boolean API policy must preserve generated and external trait signatures".to_owned());
    }
    if policy.exception_attribute != "local-expect"
        || !policy.exception_reason_required
        || !policy.exception_owner_required
        || policy.broad_suppression != "deny"
        || policy.finding_baseline != "deny"
        || policy.warning_budget != "deny"
        || policy.unknown_scope != "deny"
    {
        diagnostics.push(
            "boolean API policy must reject broad suppressions, baselines, budgets, and unknown scopes".to_owned(),
        );
    }
    if policy.non_claims.is_empty() || policy.non_claims.iter().any(|claim| claim.trim().is_empty()) {
        diagnostics.push("boolean API policy requires explicit non-claims".to_owned());
    }
}

/// Loads and validates the embedded strict-profile catalog.
///
/// # Errors
///
/// Returns a deterministic diagnostic when the embedded JSON is malformed or violates the catalog
/// contract.
pub fn load_strict_profile_catalog() -> Result<StrictProfileCatalog, String> {
    let catalog = serde_json::from_str::<StrictProfileCatalog>(STRICT_PROFILE_CATALOG_JSON)
        .map_err(|error| format!("parse embedded strict-profile catalog: {error}"))?;
    let diagnostics = strict_profile_catalog_diagnostics(&catalog);
    if diagnostics.is_empty() {
        Ok(catalog)
    } else {
        Err(format!("invalid embedded strict-profile catalog: {}", diagnostics.join("; ")))
    }
}

/// Returns one validated strict profile by identity.
///
/// # Errors
///
/// Returns a deterministic diagnostic when the catalog is invalid or `profile_id` is unknown.
pub fn strict_profile(profile_id: &str) -> Result<StrictProfile, String> {
    let catalog = load_strict_profile_catalog()?;
    if let Some(profile) = catalog.profiles.iter().find(|profile| profile.profile_id == profile_id) {
        return Ok(profile.clone());
    }

    let supported = sorted_profile_ids(&catalog);
    Err(format!("unknown strict profile `{profile_id}`; supported profiles: {}", supported.join(", ")))
}

/// Returns the sorted identities from the validated strict-profile catalog.
///
/// # Errors
///
/// Returns a deterministic diagnostic when the embedded catalog is malformed or invalid.
pub fn supported_profile_ids() -> Result<Vec<String>, String> {
    load_strict_profile_catalog().map(|catalog| sorted_profile_ids(&catalog))
}

/// Reports deterministic contract violations for an in-memory strict-profile catalog.
#[must_use]
pub fn strict_profile_catalog_diagnostics(catalog: &StrictProfileCatalog) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if catalog.schema_version != STRICT_PROFILE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "schema_version must be `{STRICT_PROFILE_SCHEMA_VERSION}`, got `{}`",
            catalog.schema_version
        ));
    }
    if catalog.profiles.is_empty() {
        diagnostics.push("profiles must not be empty".to_owned());
    }
    validate_boolean_api_policy(&catalog.boolean_api_policy, &mut diagnostics);

    let mut profile_ids = BTreeSet::new();
    for profile in &catalog.profiles {
        validate_profile(profile, &mut profile_ids, &mut diagnostics);
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_profile(profile: &StrictProfile, profile_ids: &mut BTreeSet<String>, diagnostics: &mut Vec<String>) {
    let profile_id = profile.profile_id.as_str();
    if profile_id.trim().is_empty() {
        diagnostics.push("profile_id must not be empty".to_owned());
    } else if !profile_ids.insert(profile.profile_id.clone()) {
        diagnostics.push(format!("duplicate profile_id `{profile_id}`"));
    }
    if profile.summary.trim().is_empty() {
        diagnostics.push(format!("profile `{profile_id}` summary must not be empty"));
    }
    validate_required_lints(profile, diagnostics);
    validate_effect_families(profile, diagnostics);
    validate_scope_contract(profile, diagnostics);
    validate_non_claims(profile, diagnostics);
}

fn validate_required_lints(profile: &StrictProfile, diagnostics: &mut Vec<String>) {
    let profile_id = profile.profile_id.as_str();
    if profile.required_lints.is_empty() {
        diagnostics.push(format!("profile `{profile_id}` required_lints must not be empty"));
        return;
    }

    let mut lint_names = BTreeSet::new();
    for lint_name in &profile.required_lints {
        if !lint_names.insert(lint_name.as_str()) {
            diagnostics.push(format!("profile `{profile_id}` repeats required lint `{lint_name}`"));
        }
        if metadata_for_lint(lint_name).is_none() {
            diagnostics.push(format!("profile `{profile_id}` names unknown required lint `{lint_name}`"));
        }
    }
}

fn validate_effect_families(profile: &StrictProfile, diagnostics: &mut Vec<String>) {
    let profile_id = profile.profile_id.as_str();
    if profile.effect_families.is_empty() {
        diagnostics.push(format!("profile `{profile_id}` effect_families must not be empty"));
        return;
    }

    let mut effect_families = BTreeSet::new();
    for effect_family in &profile.effect_families {
        if !effect_families.insert(effect_family.as_str()) {
            diagnostics.push(format!("profile `{profile_id}` repeats effect family `{effect_family}`"));
        }
        if !SUPPORTED_EFFECT_FAMILIES.contains(&effect_family.as_str()) {
            diagnostics.push(format!("profile `{profile_id}` names unknown effect family `{effect_family}`"));
        }
    }
}

fn validate_scope_contract(profile: &StrictProfile, diagnostics: &mut Vec<String>) {
    let profile_id = profile.profile_id.as_str();
    if profile.claim_scope != STRICT_PROFILE_CLAIM_SCOPE {
        diagnostics.push(format!("profile `{profile_id}` claim_scope must be `{STRICT_PROFILE_CLAIM_SCOPE}`"));
    }
    if profile.requires_core_scopes && profile.function_address_scope != FunctionAddressScope::Core {
        diagnostics
            .push(format!("profile `{profile_id}` requires core scopes but function_address_scope is not `core`"));
    }
    if !profile.requires_core_scopes && profile.function_address_scope == FunctionAddressScope::Core {
        diagnostics
            .push(format!("profile `{profile_id}` selects core function-address scope without requiring core scopes"));
    }
    if profile.minimum_function_address_mode == FunctionAddressModeFloor::Off {
        diagnostics.push(format!("profile `{profile_id}` function-address floor must not be `off`"));
    }
}

fn validate_non_claims(profile: &StrictProfile, diagnostics: &mut Vec<String>) {
    let profile_id = profile.profile_id.as_str();
    if profile.non_claims.is_empty() {
        diagnostics.push(format!("profile `{profile_id}` non_claims must not be empty"));
        return;
    }
    if profile.non_claims.iter().any(|non_claim| non_claim.trim().is_empty()) {
        diagnostics.push(format!("profile `{profile_id}` non_claims must not contain empty text"));
    }
}

fn sorted_profile_ids(catalog: &StrictProfileCatalog) -> Vec<String> {
    let mut ids = catalog.profiles.iter().map(|profile| profile.profile_id.clone()).collect::<Vec<_>>();
    ids.sort();
    ids
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_PROFILE_IDS: &[&str] = &["bounded-systems", "deterministic-core", "robust"];

    #[test]
    fn embedded_catalog_accepts_every_shipped_profile() {
        let catalog = load_strict_profile_catalog().expect("embedded strict-profile catalog should validate");
        let ids = sorted_profile_ids(&catalog);

        assert_eq!(ids, EXPECTED_PROFILE_IDS);
        for profile_id in EXPECTED_PROFILE_IDS {
            let profile = strict_profile(profile_id).expect("shipped profile should resolve");
            assert_eq!(profile.profile_id, *profile_id);
            assert!(!profile.required_lints.is_empty());
            assert!(!profile.non_claims.is_empty());
        }
    }

    #[test]
    fn unknown_profile_lists_supported_ids_in_deterministic_order() {
        let error = strict_profile("not-a-profile").expect_err("unknown profile should fail");

        assert!(error.contains("unknown strict profile `not-a-profile`"));
        assert!(error.contains("bounded-systems, deterministic-core, robust"));
    }

    #[test]
    fn malformed_catalog_reports_duplicate_unknown_and_scope_errors() {
        let mut catalog = load_strict_profile_catalog().expect("fixture catalog");
        let duplicate_id = catalog.profiles[0].profile_id.clone();
        catalog.profiles[1].profile_id = duplicate_id;
        catalog.profiles[1].required_lints.push("not_a_lint".to_owned());
        catalog.profiles[1].requires_core_scopes = true;
        catalog.profiles[1].function_address_scope = FunctionAddressScope::Workspace;
        catalog.profiles[1].non_claims.clear();

        let diagnostics = strict_profile_catalog_diagnostics(&catalog);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate profile_id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unknown required lint `not_a_lint`")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("requires core scopes")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("non_claims must not be empty")));
    }

    #[test]
    fn boolean_api_policy_rejects_suppression_and_rekeys_on_change() {
        let mut catalog = load_strict_profile_catalog().expect("checked policy");
        let initial = boolean_api_policy_identity(&catalog.boolean_api_policy).expect("policy hash");
        let (_, repeated) = boolean_api_policy().expect("repeat policy hash");
        assert_eq!(initial, repeated);
        catalog.boolean_api_policy.warning_budget = "allow".to_owned();
        catalog.boolean_api_policy.broad_suppression = "allow".to_owned();
        catalog.boolean_api_policy.public_scope.push("unknown-scope".to_owned());

        let diagnostics = strict_profile_catalog_diagnostics(&catalog);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("broad suppressions, baselines, budgets")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("closed callable set")));
        assert_ne!(initial, boolean_api_policy_identity(&catalog.boolean_api_policy).expect("changed policy hash"));
    }

    #[test]
    fn boolean_api_policy_retains_flat_generated_fields() {
        let source: serde_json::Value =
            serde_json::from_str(STRICT_PROFILE_CATALOG_JSON).expect("generated strict-profile catalog");
        let policy = load_strict_profile_catalog().expect("checked policy").boolean_api_policy;
        let serialized = serde_json::to_value(policy).expect("serialize checked policy");
        assert_eq!(serialized, source["boolean_api_policy"]);
    }
}
