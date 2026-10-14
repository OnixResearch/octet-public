use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

pub(crate) const CATALOG_SCHEMA_VERSION: &str = "octet-clippy-restriction-profiles/v1";
pub(crate) const CLAIM_SCOPE: &str = "clippy-restriction-diagnostics";

const BLANKET_RESTRICTION_GROUP: &str = "restriction";
const SHIPPED_CATALOG_JSON: &str = include_str!("../../standards/clippy-restrictions/generated/profiles.json");
const SUPPORTED_LINTS: &[&str] = &[
    "dbg_macro",
    "exit",
    "host_endian_bytes",
    "iter_over_hash_type",
    "mem_forget",
    "multiple_unsafe_ops_per_block",
    "undocumented_unsafe_blocks",
];

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum ClippyLintLevel {
    Warn,
    Deny,
}

impl ClippyLintLevel {
    const fn command_flag(self) -> &'static str {
        match self {
            Self::Warn => "-W",
            Self::Deny => "-D",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ClippyRestrictionProfile {
    pub(crate) profile_id: String,
    pub(crate) summary: String,
    pub(crate) lint_level: ClippyLintLevel,
    pub(crate) lints: Vec<String>,
    pub(crate) claim_scope: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ClippyRestrictionCatalog {
    pub(crate) schema_version: String,
    pub(crate) profiles: Vec<ClippyRestrictionProfile>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct ClippyRestrictionReceiptDetails {
    pub(crate) profile: ClippyRestrictionProfile,
    pub(crate) profile_identity: String,
}

pub(crate) fn parse_catalog(text: &str) -> Result<ClippyRestrictionCatalog, String> {
    serde_json::from_str(text).map_err(|error| format!("parse Clippy restriction profile catalog: {error}"))
}

pub(crate) fn shipped_catalog() -> Result<ClippyRestrictionCatalog, String> {
    checked_catalog(SHIPPED_CATALOG_JSON)
}

pub(crate) fn shipped_catalog_identity() -> String {
    crate::determinism::full_hash(SHIPPED_CATALOG_JSON)
}

pub(crate) fn checked_catalog(text: &str) -> Result<ClippyRestrictionCatalog, String> {
    let catalog = parse_catalog(text)?;
    let diagnostics = catalog_diagnostics(&catalog);
    if diagnostics.is_empty() {
        Ok(catalog)
    } else {
        Err(format!("invalid Clippy restriction profile catalog:\n- {}", diagnostics.join("\n- ")))
    }
}

pub(crate) fn catalog_diagnostics(catalog: &ClippyRestrictionCatalog) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if catalog.schema_version != CATALOG_SCHEMA_VERSION {
        diagnostics
            .push(format!("schema_version must be `{CATALOG_SCHEMA_VERSION}`, got `{}`", catalog.schema_version));
    }
    if catalog.profiles.is_empty() {
        diagnostics.push("Clippy restriction profiles must not be empty".to_owned());
    }

    let mut profile_ids = BTreeSet::new();
    for profile in &catalog.profiles {
        let profile_id = profile.profile_id.as_str();
        if profile_id.trim().is_empty() {
            diagnostics.push("Clippy restriction profile_id must not be empty".to_owned());
        } else if !profile_ids.insert(profile_id) {
            diagnostics.push(format!("duplicate Clippy restriction profile_id `{profile_id}`"));
        }
        diagnostics.extend(profile_diagnostics(profile));
    }

    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn select_profile(
    catalog: &ClippyRestrictionCatalog,
    profile_id: &str,
) -> Result<ClippyRestrictionProfile, String> {
    let diagnostics = catalog_diagnostics(catalog);
    if !diagnostics.is_empty() {
        return Err(format!("invalid Clippy restriction profile catalog:\n- {}", diagnostics.join("\n- ")));
    }
    if let Some(profile) = catalog.profiles.iter().find(|profile| profile.profile_id == profile_id) {
        return Ok(profile.clone());
    }

    let mut supported = catalog.profiles.iter().map(|profile| profile.profile_id.as_str()).collect::<Vec<_>>();
    supported.sort_unstable();
    Err(format!(
        "unknown Clippy restriction profile `{profile_id}`; supported profiles: {}",
        supported.join(", ")
    ))
}

pub(crate) fn profile_identity(profile: &ClippyRestrictionProfile) -> Result<String, String> {
    let canonical = serde_json::to_string(&serde_json::json!({
        "schema_version": CATALOG_SCHEMA_VERSION,
        "profile": profile,
    }))
    .map_err(|error| format!("serialize Clippy restriction profile identity: {error}"))?;
    Ok(crate::determinism::full_hash(&canonical))
}

pub(crate) fn receipt_details(profile: &ClippyRestrictionProfile) -> Result<ClippyRestrictionReceiptDetails, String> {
    Ok(ClippyRestrictionReceiptDetails {
        profile: profile.clone(),
        profile_identity: profile_identity(profile)?,
    })
}

pub(crate) fn command_argv(profile: &ClippyRestrictionProfile) -> Vec<String> {
    let mut argv = vec![
        "cargo".to_owned(),
        "clippy".to_owned(),
        "--quiet".to_owned(),
        "--workspace".to_owned(),
        "--all-targets".to_owned(),
        "--message-format=json".to_owned(),
        "--".to_owned(),
    ];
    for lint in &profile.lints {
        argv.push(profile.lint_level.command_flag().to_owned());
        argv.push(format!("clippy::{lint}"));
    }
    argv
}

pub(crate) fn receipt_details_diagnostics(details: &ClippyRestrictionReceiptDetails) -> Vec<String> {
    let mut diagnostics = profile_diagnostics(&details.profile);
    match profile_identity(&details.profile) {
        Ok(expected) if details.profile_identity != expected => {
            diagnostics.push("Clippy restriction selected-profile BLAKE3 identity mismatch".to_owned());
        }
        Err(error) => diagnostics.push(error),
        Ok(_) => {}
    }
    if !is_blake3_identity(&details.profile_identity) {
        diagnostics.push("Clippy restriction selected-profile identity must be BLAKE3".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn is_blake3_identity(value: &str) -> bool {
    const BLAKE3_PREFIX: &str = "b3:";
    const BLAKE3_HEX_LENGTH: usize = 64;

    value
        .strip_prefix(BLAKE3_PREFIX)
        .is_some_and(|digest| digest.len() == BLAKE3_HEX_LENGTH && digest.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn profile_diagnostics(profile: &ClippyRestrictionProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let profile_id = profile.profile_id.as_str();
    if profile.summary.trim().is_empty() {
        diagnostics.push(format!("Clippy restriction profile `{profile_id}` summary must not be empty"));
    }
    if profile.claim_scope != CLAIM_SCOPE {
        diagnostics.push(format!("Clippy restriction profile `{profile_id}` claim_scope must be `{CLAIM_SCOPE}`"));
    }
    if profile.lints.is_empty() {
        diagnostics.push(format!("Clippy restriction profile `{profile_id}` lints must not be empty"));
    }
    if profile.non_claims.is_empty() || profile.non_claims.iter().any(|claim| claim.trim().is_empty()) {
        diagnostics.push(format!("Clippy restriction profile `{profile_id}` non_claims must contain non-empty text"));
    }

    let mut lint_ids = BTreeSet::new();
    for lint in &profile.lints {
        if lint == BLANKET_RESTRICTION_GROUP {
            diagnostics.push(format!(
                "Clippy restriction profile `{profile_id}` must not enable the blanket `clippy::restriction` group"
            ));
            continue;
        }
        if !lint_ids.insert(lint.as_str()) {
            diagnostics.push(format!("Clippy restriction profile `{profile_id}` repeats lint `{lint}`"));
        }
        if !SUPPORTED_LINTS.contains(&lint.as_str()) {
            diagnostics.push(format!("Clippy restriction profile `{profile_id}` names unsupported lint `{lint}`"));
        }
    }
    diagnostics
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXPECTED_PROFILE_IDS: &[&str] = &["canonical-determinism", "production-safety", "shell-boundary-inventory"];

    #[test]
    fn shipped_catalog_selects_bounded_profiles_and_builds_exact_commands() {
        let catalog = shipped_catalog().expect("shipped catalog should validate");
        let mut profile_ids = catalog.profiles.iter().map(|profile| profile.profile_id.as_str()).collect::<Vec<_>>();
        profile_ids.sort_unstable();
        assert_eq!(profile_ids, EXPECTED_PROFILE_IDS);

        let production = select_profile(&catalog, "production-safety").expect("production profile");
        let canonical = select_profile(&catalog, "canonical-determinism").expect("canonical profile");
        let production_command = command_argv(&production);
        assert_eq!(production.lint_level, ClippyLintLevel::Deny);
        assert!(production.lints.iter().any(|lint| lint == "mem_forget"));
        assert!(canonical.lints.iter().any(|lint| lint == "host_endian_bytes"));
        assert!(canonical.lints.iter().any(|lint| lint == "iter_over_hash_type"));
        let expected_prefix = [
            "cargo",
            "clippy",
            "--quiet",
            "--workspace",
            "--all-targets",
            "--message-format=json",
            "--",
        ]
        .map(str::to_owned);
        assert!(production_command.starts_with(&expected_prefix));
        assert!(production_command.windows(2).any(|pair| pair == ["-D", "clippy::dbg_macro"]));
        assert!(!production_command.iter().any(|arg| arg == "clippy::restriction"));

        let first_identity = profile_identity(&production).expect("profile identity");
        let repeated_identity = profile_identity(&production).expect("repeated profile identity");
        let canonical_identity = profile_identity(&canonical).expect("canonical identity");
        assert_eq!(first_identity, repeated_identity);
        assert_ne!(first_identity, canonical_identity);
        assert!(is_blake3_identity(&first_identity));
        assert!(is_blake3_identity(&shipped_catalog_identity()));
    }

    #[test]
    fn catalog_rejects_duplicates_unknown_lints_blanket_group_and_invalid_fields() {
        let mut catalog = shipped_catalog().expect("fixture catalog");
        catalog.schema_version = "unsupported".to_owned();
        catalog.profiles[0].summary.clear();
        catalog.profiles[0].claim_scope = "overclaim".to_owned();
        catalog.profiles[0].non_claims.clear();
        let duplicate_lint = catalog.profiles[0].lints[0].clone();
        catalog.profiles[0].lints.push(duplicate_lint);
        catalog.profiles[0].lints.push("restriction".to_owned());
        catalog.profiles[0].lints.push("unknown_lint".to_owned());
        let duplicate_profile_id = catalog.profiles[0].profile_id.clone();
        catalog.profiles[1].profile_id = duplicate_profile_id;

        let diagnostics = catalog_diagnostics(&catalog);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("schema_version")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("summary must not be empty")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("claim_scope")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("non_claims")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("repeats lint")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("blanket")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported lint `unknown_lint`")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate Clippy restriction profile_id")));
    }

    #[test]
    fn unknown_profile_and_tampered_receipt_fail_deterministically() {
        let catalog = shipped_catalog().expect("shipped catalog");
        let unknown = select_profile(&catalog, "missing").expect_err("unknown profile should fail");
        assert!(unknown.contains("unknown Clippy restriction profile `missing`"));
        assert!(unknown.contains("canonical-determinism, production-safety, shell-boundary-inventory"));

        let profile = select_profile(&catalog, "production-safety").expect("production profile");
        let mut details = receipt_details(&profile).expect("receipt details");
        assert!(receipt_details_diagnostics(&details).is_empty());
        details.profile.lints.push("exit".to_owned());
        let diagnostics = receipt_details_diagnostics(&details);
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("identity mismatch")));
    }
}
