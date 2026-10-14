use std::collections::BTreeSet;
use std::path::Path;

use super::EXPECTED_BACKEND;
use super::EXPECTED_CLAIM_CLASS;
use super::EXPECTED_COMPATIBILITY_ID;
use super::EXPECTED_COMPATIBILITY_IDENTITY;
use super::EXPECTED_ENVIRONMENT;
use super::EXPECTED_PROVIDER_ID;
use super::EXPECTED_PROVIDER_IDENTITY;
use super::EXPECTED_VERIFICATION_ROLE;
use super::PROFILE_SCHEMA;
use super::RECEIPT_SCHEMA;
use super::REQUIRED_NON_CLAIMS;
use super::model::ConsumerOutcome;
use super::model::ConsumerProfile;
use super::model::ConsumerReceiptBinding;

const PROVIDER_TIMEOUT_MAX_MS: u64 = 900_000;
const PROVIDER_LOG_MAX_BYTES: usize = 1_048_576;

pub(super) fn profile(profile: &ConsumerProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    validate_provider(profile, &mut diagnostics);
    validate_sources(profile, &mut diagnostics);
    validate_theorems(profile, &mut diagnostics);
    validate_trust_and_resources(profile, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_provider(profile: &ConsumerProfile, diagnostics: &mut Vec<String>) {
    if profile.schema_version != PROFILE_SCHEMA
        || profile.profile_id.trim().is_empty()
        || profile.consumer_id.trim().is_empty()
        || profile.provider.backend != EXPECTED_BACKEND
        || profile.provider.profile_id != EXPECTED_PROVIDER_ID
        || profile.provider.profile_identity != EXPECTED_PROVIDER_IDENTITY
        || profile.provider.compatibility_matrix_id != EXPECTED_COMPATIBILITY_ID
        || profile.provider.compatibility_matrix_identity != EXPECTED_COMPATIBILITY_IDENTITY
        || profile.provider.network_allowed
    {
        diagnostics.push("Aeneas consumer provider or compatibility identity drifted".to_owned());
    }
    if profile.outcomes != ["supported", "blocked", "failed"] || !profile.trust_closure_complete {
        diagnostics.push("Aeneas consumer outcome or trust-closure policy is incomplete".to_owned());
    }
    for required in REQUIRED_NON_CLAIMS {
        if !profile.non_claims.iter().any(|claim| claim == required) {
            diagnostics.push(format!("Aeneas consumer non-claims missing `{required}`"));
        }
    }
}

fn validate_sources(profile: &ConsumerProfile, diagnostics: &mut Vec<String>) {
    if unsafe_path(&profile.source.root) || profile.source.paths.is_empty() || profile.source.selected_items.is_empty()
    {
        diagnostics.push("Aeneas consumer source root or selected source set is invalid".to_owned());
    }
    if profile.source.broad_selection_allowed {
        diagnostics.push("Aeneas consumer broad source selection is forbidden".to_owned());
    }
    let mut paths = BTreeSet::new();
    for source in &profile.source.paths {
        if unsafe_path(&source.path) || !super::is_b3(&source.identity) || !paths.insert(&source.path) {
            diagnostics.push("Aeneas consumer source paths are unsafe, stale, or duplicated".to_owned());
        }
    }
    let mut items = BTreeSet::new();
    for item in &profile.source.selected_items {
        let source_matches = profile
            .source
            .paths
            .iter()
            .any(|source| source.path == item.source_file && source.identity == item.source_identity);
        if item.qualified_name.trim().is_empty()
            || item.kind != "function"
            || item.generated_name.trim().is_empty()
            || item.begin_line == 0
            || item.end_line < item.begin_line
            || !source_matches
            || !items.insert(&item.qualified_name)
        {
            diagnostics.push("Aeneas consumer selected item is broad, ambiguous, or detached from source".to_owned());
        }
    }
}

fn validate_theorems(profile: &ConsumerProfile, diagnostics: &mut Vec<String>) {
    if profile.theorems.is_empty() {
        diagnostics.push("Aeneas consumer theorem mapping is empty".to_owned());
    }
    let mut ids = BTreeSet::new();
    for theorem in &profile.theorems {
        if unsafe_path(&theorem.path)
            || theorem.theorem_id.trim().is_empty()
            || theorem.declaration_name.trim().is_empty()
            || theorem.requirement_id.trim().is_empty()
            || theorem.claim_class != EXPECTED_CLAIM_CLASS
            || theorem.source_disposition != "maintained"
            || !super::is_b3(&theorem.approved_statement_blake3)
            || !ids.insert(&theorem.theorem_id)
        {
            diagnostics.push("Aeneas consumer theorem mapping is incomplete, stale, or duplicated".to_owned());
        }
    }
}

fn validate_trust_and_resources(profile: &ConsumerProfile, diagnostics: &mut Vec<String>) {
    let mut trust_ids = BTreeSet::new();
    for input in profile.external_models.iter().chain(&profile.trust_inputs) {
        if input.input_id.trim().is_empty() || !super::is_b3(&input.identity) || !trust_ids.insert(&input.input_id) {
            diagnostics.push("Aeneas consumer trust input is empty, malformed, or duplicated".to_owned());
        }
    }
    let resource = &profile.resource_policy;
    let environment = resource.environment_allowlist.iter().map(String::as_str).collect::<Vec<_>>();
    if resource.timeout_ms == 0
        || resource.timeout_ms > PROVIDER_TIMEOUT_MAX_MS
        || resource.stdout_max_bytes == 0
        || resource.stdout_max_bytes > PROVIDER_LOG_MAX_BYTES
        || resource.stderr_max_bytes == 0
        || resource.stderr_max_bytes > PROVIDER_LOG_MAX_BYTES
        || environment != EXPECTED_ENVIRONMENT
    {
        diagnostics.push("Aeneas consumer resource or environment policy exceeds provider authority".to_owned());
    }
}

pub(super) fn receipt(
    receipt: &ConsumerReceiptBinding,
    profile: &ConsumerProfile,
    profile_identity: &str,
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != RECEIPT_SCHEMA
        || receipt.profile_id != profile.profile_id
        || receipt.profile_identity != profile_identity
        || receipt.consumer_id != profile.consumer_id
        || receipt.provider_profile_identity != profile.provider.profile_identity
        || receipt.compatibility_matrix_identity != profile.provider.compatibility_matrix_identity
        || receipt.source_root != profile.source.root
        || receipt.outcome != ConsumerOutcome::Supported
        || receipt.verification_role != EXPECTED_VERIFICATION_ROLE
        || receipt.non_claims != profile.non_claims
    {
        diagnostics.push("Aeneas consumer receipt policy, outcome, or evidence role drifted".to_owned());
    }
    if !profile.source.paths.iter().any(|source| {
        source.path == receipt.source_path
            && source.identity == receipt.source_identity
            && receipt.selected_item.source_file == source.path
            && receipt.selected_item.source_identity == source.identity
    }) || !profile.source.selected_items.contains(&receipt.selected_item)
        || !profile.theorems.contains(&receipt.theorem)
        || receipt.external_models != profile.external_models
        || receipt.trust_inputs != profile.trust_inputs
        || receipt.resource_policy != profile.resource_policy
    {
        diagnostics.push("Aeneas consumer receipt source, theorem, model, or trust binding drifted".to_owned());
    }
    let mut roles = BTreeSet::new();
    if receipt.stage_artifacts.is_empty()
        || receipt.stage_artifacts.iter().any(|artifact| {
            artifact.role.trim().is_empty() || !super::is_b3(&artifact.blake3) || !roles.insert(&artifact.role)
        })
    {
        diagnostics.push("Aeneas consumer receipt stage artifacts are empty, malformed, or duplicated".to_owned());
    }
    match super::receipt_identity(receipt) {
        Ok(expected) if receipt.receipt_identity == expected => {}
        Ok(_) => diagnostics.push("Aeneas consumer receipt identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn unsafe_path(value: &str) -> bool {
    value.is_empty()
        || Path::new(value).is_absolute()
        || value.split('/').any(|component| component.is_empty() || component == "..")
}
