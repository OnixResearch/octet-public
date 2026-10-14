use std::collections::BTreeSet;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use octet_lint_metadata::strict_profile::FunctionAddressModeFloor;
use octet_lint_metadata::strict_profile::FunctionAddressScope;
use octet_lint_metadata::strict_profile::StrictProfile;

use crate::function_address::FunctionAddressCheck;
use crate::function_address::FunctionAddressMode;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectedStrictProfile {
    pub policy: StrictProfile,
    pub policy_hash: String,
    pub core_source_scopes: Vec<PathBuf>,
}

pub fn prepare_selected_profile(
    profile_id: Option<&str>,
    workspace_root: &Path,
) -> Result<Option<SelectedStrictProfile>, String> {
    let Some(profile_id) = profile_id else {
        return Ok(None);
    };
    let policy = octet_lint_metadata::strict_profile::strict_profile(profile_id)?;
    let core_source_scopes =
        if policy.requires_core_scopes || policy.function_address_scope == FunctionAddressScope::Core {
            load_core_source_scopes(workspace_root, &policy)?
        } else {
            Vec::new()
        };
    let policy_hash = strict_profile_policy_hash(&policy)?;

    Ok(Some(SelectedStrictProfile {
        policy,
        policy_hash,
        core_source_scopes,
    }))
}

pub fn strict_profile_policy_hash(profile: &StrictProfile) -> Result<String, String> {
    let canonical = serde_json::to_string(&serde_json::json!({
        "schema_version": octet_lint_metadata::strict_profile::STRICT_PROFILE_SCHEMA_VERSION,
        "profile": profile,
    }))
    .map_err(|error| format!("serialize strict profile identity: {error}"))?;
    Ok(crate::determinism::full_hash(&canonical))
}

pub fn effective_function_address_check(
    selected: Option<&SelectedStrictProfile>,
    requested: &FunctionAddressCheck,
) -> FunctionAddressCheck {
    let Some(selected) = selected else {
        return requested.clone();
    };

    let mut effective = requested.clone();
    let minimum_mode = function_address_mode(selected.policy.minimum_function_address_mode);
    if effective.mode < minimum_mode {
        effective.mode = minimum_mode;
    }
    if effective.paths.is_empty() && selected.policy.function_address_scope == FunctionAddressScope::Core {
        effective.paths.clone_from(&selected.core_source_scopes);
    }
    effective
}

fn load_core_source_scopes(workspace_root: &Path, profile: &StrictProfile) -> Result<Vec<PathBuf>, String> {
    let config_path = workspace_root.join("dylint.toml");
    let source = match fs::read_to_string(&config_path) {
        Ok(source) => source,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(format!("read {}: {error}", config_path.display())),
    };
    core_source_scopes_from_toml(&source, profile)
}

fn core_source_scopes_from_toml(source: &str, profile: &StrictProfile) -> Result<Vec<PathBuf>, String> {
    let document = if source.trim().is_empty() {
        toml::Table::new()
    } else {
        source
            .parse::<toml::Table>()
            .map_err(|error| format!("parse dylint.toml for strict profile: {error}"))?
    };
    let profile_table = selected_config_table(&document);
    let raw_scopes = match profile_table.and_then(|table| table.get("core_source_scopes")) {
        Some(value) => value
            .as_array()
            .ok_or_else(|| "`core_source_scopes` must be an array of workspace-relative strings".to_owned())?
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| "`core_source_scopes` must contain only workspace-relative strings".to_owned())
            })
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    let scopes = normalize_core_source_scopes(&raw_scopes)?;
    if profile.requires_core_scopes && scopes.is_empty() {
        return Err(format!(
            "strict profile `{}` requires non-empty `core_source_scopes` in `dylint.toml` before Cargo runs",
            profile.profile_id
        ));
    }
    Ok(scopes)
}

fn selected_config_table(document: &toml::Table) -> Option<&toml::Table> {
    document
        .get("octet")
        .and_then(toml::Value::as_table)
        .or_else(|| document.get("slotcar").and_then(toml::Value::as_table))
        .or_else(|| document.get("tigerstyle").and_then(toml::Value::as_table))
}

fn normalize_core_source_scopes(raw_scopes: &[String]) -> Result<Vec<PathBuf>, String> {
    let mut seen = BTreeSet::new();
    let mut scopes = Vec::new();
    for raw_scope in raw_scopes {
        let scope = raw_scope.trim();
        if scope.is_empty() {
            return Err("`core_source_scopes` must not contain empty paths".to_owned());
        }
        let path = Path::new(scope);
        if path.is_absolute()
            || path
                .components()
                .any(|component| matches!(component, Component::ParentDir | Component::RootDir | Component::Prefix(_)))
        {
            return Err(format!("core source scope `{scope}` must stay within the workspace"));
        }
        let normalized = crate::determinism::normalize_path_text(scope.trim_start_matches("./"));
        if normalized.is_empty() || normalized == "." {
            return Err("`core_source_scopes` must not resolve to the workspace root".to_owned());
        }
        if seen.insert(normalized.clone()) {
            scopes.push(PathBuf::from(normalized));
        }
    }
    Ok(scopes)
}

fn function_address_mode(floor: FunctionAddressModeFloor) -> FunctionAddressMode {
    match floor {
        FunctionAddressModeFloor::Off => FunctionAddressMode::Off,
        FunctionAddressModeFloor::Observe => FunctionAddressMode::Observe,
        FunctionAddressModeFloor::Warn => FunctionAddressMode::Warn,
        FunctionAddressModeFloor::Deny => FunctionAddressMode::Deny,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_core_scopes_are_deduplicated_and_preserve_order() {
        let profile =
            octet_lint_metadata::strict_profile::strict_profile("deterministic-core").expect("deterministic profile");
        let source = r#"
[octet]
core_source_scopes = ["crates/domain/src", "./crates/domain/src", "crates/model/src"]
"#;

        let scopes = core_source_scopes_from_toml(source, &profile).expect("valid core scopes");

        assert_eq!(scopes, vec![PathBuf::from("crates/domain/src"), PathBuf::from("crates/model/src")]);
    }

    #[test]
    fn deterministic_core_rejects_missing_and_escaping_scopes() {
        let profile =
            octet_lint_metadata::strict_profile::strict_profile("deterministic-core").expect("deterministic profile");

        let missing_error = core_source_scopes_from_toml("[octet]\n", &profile).expect_err("missing scopes fail");
        let empty_error = core_source_scopes_from_toml("[octet]\ncore_source_scopes = []\n", &profile)
            .expect_err("empty scopes fail");
        let escape_error = core_source_scopes_from_toml("[octet]\ncore_source_scopes = [\"../outside\"]\n", &profile)
            .expect_err("escaping scope fails");
        let absolute_error = core_source_scopes_from_toml("[octet]\ncore_source_scopes = [\"/outside\"]\n", &profile)
            .expect_err("absolute scope fails");
        let root_error = core_source_scopes_from_toml("[octet]\ncore_source_scopes = [\".\"]\n", &profile)
            .expect_err("workspace-root scope fails");

        assert!(missing_error.contains("deterministic-core"));
        assert!(missing_error.contains("core_source_scopes"));
        assert!(empty_error.contains("deterministic-core"));
        assert!(escape_error.contains("must stay within the workspace"));
        assert!(absolute_error.contains("must stay within the workspace"));
        assert!(root_error.contains("workspace root"));
    }

    #[test]
    fn selected_profile_policy_hash_is_stable_and_profile_specific() {
        let robust = octet_lint_metadata::strict_profile::strict_profile("robust").expect("robust profile");
        let bounded = octet_lint_metadata::strict_profile::strict_profile("bounded-systems").expect("bounded profile");

        let robust_hash = strict_profile_policy_hash(&robust).expect("robust hash");
        let repeated_robust_hash = strict_profile_policy_hash(&robust).expect("repeated robust hash");
        let bounded_hash = strict_profile_policy_hash(&bounded).expect("bounded hash");

        assert_eq!(robust_hash, repeated_robust_hash);
        assert_ne!(robust_hash, bounded_hash);
        assert!(robust_hash.starts_with("b3:"));
    }

    #[test]
    fn profile_function_address_floor_raises_weaker_mode_and_defaults_core_paths() {
        let policy =
            octet_lint_metadata::strict_profile::strict_profile("deterministic-core").expect("deterministic profile");
        let selected = SelectedStrictProfile {
            policy,
            policy_hash: "b3:profile".to_owned(),
            core_source_scopes: vec![PathBuf::from("crates/domain/src")],
        };
        let requested = FunctionAddressCheck::off();

        let effective = effective_function_address_check(Some(&selected), &requested);

        assert_eq!(effective.mode, FunctionAddressMode::Deny);
        assert_eq!(effective.paths, selected.core_source_scopes);
    }

    #[test]
    fn profile_function_address_floor_preserves_stronger_explicit_configuration() {
        let policy = octet_lint_metadata::strict_profile::strict_profile("robust").expect("robust profile");
        let selected = SelectedStrictProfile {
            policy,
            policy_hash: "b3:profile".to_owned(),
            core_source_scopes: Vec::new(),
        };
        let requested = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("src/selected.rs")],
            baseline: Some(PathBuf::from("artifacts/functions.json")),
        };

        let effective = effective_function_address_check(Some(&selected), &requested);

        assert_eq!(effective, requested);
    }

    #[test]
    fn absent_profile_preserves_requested_function_address_configuration() {
        let requested = FunctionAddressCheck {
            mode: FunctionAddressMode::Warn,
            paths: vec![PathBuf::from("src")],
            baseline: None,
        };

        assert_eq!(effective_function_address_check(None, &requested), requested);
    }
}
