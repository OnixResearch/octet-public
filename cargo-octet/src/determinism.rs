use std::fs;
use std::path::Path;
use std::process::Command;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct RunMetadata {
    pub tool_name: &'static str,
    pub tool_version: &'static str,
    pub rustc_version: String,
    pub toolchain: String,
    pub profile_name: String,
    pub profile_hash: String,
    pub config_hash: String,
}

#[derive(Debug, Clone, Copy)]
pub struct StrictProfileIdentity<'a> {
    pub profile_id: &'a str,
    pub policy_hash: &'a str,
}

#[derive(Debug, Clone)]
pub struct MetadataInput<'a> {
    pub workspace_root: &'a Path,
    pub toolchain: &'a str,
    pub scope_args: &'a [String],
    pub cargo_check_args: &'a [String],
    pub output_format: &'a str,
    pub strict_profile: Option<StrictProfileIdentity<'a>>,
}

pub fn build_run_metadata(input: &MetadataInput<'_>) -> RunMetadata {
    let config_hash = config_hash(input.workspace_root, input.scope_args, input.cargo_check_args, input.strict_profile);
    let profile_hash =
        profile_hash(input.scope_args, input.cargo_check_args, input.output_format, &config_hash, input.strict_profile);

    RunMetadata {
        tool_name: "cargo-octet",
        tool_version: env!("CARGO_PKG_VERSION"),
        rustc_version: rustc_version(input.toolchain),
        toolchain: input.toolchain.to_owned(),
        profile_name: input
            .strict_profile
            .map_or_else(|| "workspace-metadata".to_owned(), |profile| profile.profile_id.to_owned()),
        profile_hash,
        config_hash,
    }
}

pub fn semantic_finding_id(
    lint: &str,
    crate_name: &str,
    message: &str,
    suggestion: Option<&str>,
    normalized_snippet_hash: &str,
    config_hash: Option<&str>,
) -> String {
    let canonical = format!(
        "lint={lint}\ncrate={crate_name}\nmessage={message}\nsuggestion={}\nsnippet={normalized_snippet_hash}\nconfig={}\ntool=cargo-octet@{}\n",
        suggestion.unwrap_or(""),
        config_hash.unwrap_or("unknown"),
        env!("CARGO_PKG_VERSION"),
    );
    short_hash(&canonical)
}

pub fn normalized_snippet_hash(snippet: &str) -> String {
    let normalized = snippet.split_whitespace().collect::<Vec<_>>().join(" ");
    full_hash(&normalized)
}

pub fn crate_graph_hash(workspace_root: &Path) -> String {
    let payload = serde_json::json!({
        "cargo_toml": file_hash(&workspace_root.join("Cargo.toml")),
        "cargo_lock": file_hash(&workspace_root.join("Cargo.lock")),
    });
    full_hash(&payload.to_string())
}

pub fn short_hash(input: &str) -> String {
    let hash = blake3::hash(input.as_bytes()).to_hex().to_string();
    format!("b3:{}", &hash[..16])
}

pub fn full_hash(input: &str) -> String {
    format!("b3:{}", blake3::hash(input.as_bytes()).to_hex())
}

pub fn normalize_path(path: &Path) -> String {
    normalize_path_text(&path.display().to_string())
}

pub fn normalize_path_text(path: &str) -> String {
    path.replace('\\', "/")
}

fn profile_hash(
    scope_args: &[String],
    cargo_check_args: &[String],
    output_format: &str,
    config_hash: &str,
    strict_profile: Option<StrictProfileIdentity<'_>>,
) -> String {
    let payload = match strict_profile {
        Some(profile) => serde_json::json!({
            "scope_args": scope_args,
            "cargo_check_args": cargo_check_args,
            "output_format": output_format,
            "config_hash": config_hash,
            "strict_profile": {
                "profile_id": profile.profile_id,
                "policy_hash": profile.policy_hash,
            },
        }),
        None => serde_json::json!({
            "scope_args": scope_args,
            "cargo_check_args": cargo_check_args,
            "output_format": output_format,
            "config_hash": config_hash,
        }),
    };
    full_hash(&payload.to_string())
}

/// `[workspace.metadata.<name>]` tables the runner reads from the root manifest (current table
/// first, then the legacy fallbacks in `config::workspace_metadata_table`). Every other manifest
/// key stays out of finding identity.
const RUNNER_METADATA_TABLES: [&str; 3] = ["octet", "slotcar", "tigerstyle"];

fn config_hash(
    workspace_root: &Path,
    scope_args: &[String],
    cargo_check_args: &[String],
    strict_profile: Option<StrictProfileIdentity<'_>>,
) -> String {
    let runner_manifest_config = runner_manifest_config(&workspace_root.join("Cargo.toml"));
    let dylint_toml = file_hash(&workspace_root.join("dylint.toml"));
    let payload = match strict_profile {
        Some(profile) => serde_json::json!({
            "runner_manifest_config": runner_manifest_config,
            "dylint_toml": dylint_toml,
            "effective_scope_args": scope_args,
            "effective_cargo_check_args": cargo_check_args,
            "strict_profile": {
                "profile_id": profile.profile_id,
                "policy_hash": profile.policy_hash,
            },
        }),
        None => serde_json::json!({
            "runner_manifest_config": runner_manifest_config,
            "dylint_toml": dylint_toml,
            "effective_scope_args": scope_args,
            "effective_cargo_check_args": cargo_check_args,
        }),
    };
    full_hash(&payload.to_string())
}

/// Canonical JSON of the runner metadata tables in the root manifest. A missing manifest and an
/// unparsable one hash differently from each other and from any parsed manifest; the unparsable
/// case keeps the raw bytes in identity.
fn runner_manifest_config(manifest_path: &Path) -> serde_json::Value {
    let Ok(bytes) = fs::read(manifest_path) else {
        return serde_json::json!({ "status": "missing" });
    };
    let document = std::str::from_utf8(&bytes).ok().and_then(|text| text.parse::<toml::Table>().ok());
    let Some(document) = document else {
        return serde_json::json!({
            "status": "unparsable",
            "hash": format!("b3:{}", blake3::hash(&bytes).to_hex()),
        });
    };
    let metadata = document
        .get("workspace")
        .and_then(toml::Value::as_table)
        .and_then(|workspace| workspace.get("metadata"))
        .and_then(toml::Value::as_table);
    let tables = RUNNER_METADATA_TABLES
        .iter()
        .filter_map(|name| {
            let value = metadata?.get(*name)?;
            Some(((*name).to_owned(), canonical_toml_json(value)))
        })
        .collect::<serde_json::Map<_, _>>();
    serde_json::json!({ "status": "parsed", "workspace_metadata": tables })
}

/// Converts TOML to JSON with object keys inserted in sorted order, so the rendered text does not
/// depend on manifest key order or on whether `serde_json` preserves insertion order.
fn canonical_toml_json(value: &toml::Value) -> serde_json::Value {
    match value {
        toml::Value::String(text) => serde_json::Value::String(text.clone()),
        toml::Value::Integer(number) => serde_json::Value::from(*number),
        toml::Value::Float(number) => serde_json::Number::from_f64(*number)
            .map_or_else(|| serde_json::Value::String(number.to_string()), serde_json::Value::Number),
        toml::Value::Boolean(flag) => serde_json::Value::Bool(*flag),
        toml::Value::Datetime(datetime) => serde_json::Value::String(datetime.to_string()),
        toml::Value::Array(items) => serde_json::Value::Array(items.iter().map(canonical_toml_json).collect()),
        toml::Value::Table(table) => {
            let mut keys = table.keys().collect::<Vec<_>>();
            keys.sort();
            serde_json::Value::Object(
                keys.into_iter().map(|key| (key.clone(), canonical_toml_json(&table[key.as_str()]))).collect(),
            )
        }
    }
}

fn file_hash(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| format!("b3:{}", blake3::hash(&bytes).to_hex()))
}

/// `RunMetadata::rustc_version` when `rustc -Vv` cannot be run or read.
pub const UNKNOWN_RUSTC_VERSION: &str = "unknown";

fn rustc_version(toolchain: &str) -> String {
    Command::new("rustc")
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .arg("-Vv")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .and_then(|text| text.lines().next().map(ToOwned::to_owned))
        .unwrap_or_else(|| UNKNOWN_RUSTC_VERSION.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONFIG_HASH: &str = "b3:config";
    const ROBUST_POLICY_HASH: &str = "b3:robust";
    const BOUNDED_POLICY_HASH: &str = "b3:bounded";

    #[test]
    fn strict_profile_identity_changes_config_hash_deterministically() {
        let root = tempfile::tempdir().expect("profile hash tempdir");
        std::fs::write(root.path().join("Cargo.toml"), "[workspace]\n").expect("write Cargo.toml");
        std::fs::write(root.path().join("dylint.toml"), "[octet]\n").expect("write dylint.toml");
        let scope_args = vec!["-p".to_owned(), "domain".to_owned()];
        let cargo_args = vec!["--lib".to_owned()];
        let robust = StrictProfileIdentity {
            profile_id: "robust",
            policy_hash: ROBUST_POLICY_HASH,
        };
        let bounded = StrictProfileIdentity {
            profile_id: "bounded-systems",
            policy_hash: BOUNDED_POLICY_HASH,
        };

        let default_hash = config_hash(root.path(), &scope_args, &cargo_args, None);
        let expected_default_hash = full_hash(
            &serde_json::json!({
                "runner_manifest_config": { "status": "parsed", "workspace_metadata": {} },
                "dylint_toml": file_hash(&root.path().join("dylint.toml")),
                "effective_scope_args": scope_args,
                "effective_cargo_check_args": cargo_args,
            })
            .to_string(),
        );
        let robust_hash = config_hash(root.path(), &scope_args, &cargo_args, Some(robust));
        let repeated_robust_hash = config_hash(root.path(), &scope_args, &cargo_args, Some(robust));
        let bounded_hash = config_hash(root.path(), &scope_args, &cargo_args, Some(bounded));

        assert_eq!(default_hash, expected_default_hash);
        assert_eq!(robust_hash, repeated_robust_hash);
        assert_ne!(default_hash, robust_hash);
        assert_ne!(robust_hash, bounded_hash);
    }

    const BASE_MANIFEST: &str = r#"[workspace]
members = ["core"]
resolver = "2"

[workspace.dependencies]
serde = "1.0.200"

[workspace.metadata.octet]
default_scope = ["--workspace"]
cargo_check_args = ["--lib"]

[package]
name = "root"
version = "0.1.0"
"#;

    fn config_hash_for(manifest: &str, dylint: &str) -> String {
        let root = tempfile::tempdir().expect("config hash tempdir");
        std::fs::write(root.path().join("Cargo.toml"), manifest).expect("write Cargo.toml");
        std::fs::write(root.path().join("dylint.toml"), dylint).expect("write dylint.toml");
        config_hash(root.path(), &["--workspace".to_owned()], &["--lib".to_owned()], None)
    }

    #[test]
    fn config_hash_ignores_manifest_edits_outside_runner_metadata() {
        let baseline = config_hash_for(BASE_MANIFEST, "[octet]\n");
        let dependency_bump = BASE_MANIFEST.replace("serde = \"1.0.200\"", "serde = \"1.0.228\"");
        let new_member = BASE_MANIFEST.replace("members = [\"core\"]", "members = [\"core\", \"cli\"]");
        let package_edit = BASE_MANIFEST.replace("version = \"0.1.0\"", "version = \"0.2.0\"\nedition = \"2024\"");
        let reordered_metadata = BASE_MANIFEST.replace(
            "default_scope = [\"--workspace\"]\ncargo_check_args = [\"--lib\"]",
            "cargo_check_args = [\"--lib\"]\ndefault_scope = [\"--workspace\"]",
        );

        assert!(baseline.starts_with("b3:") && baseline.len() == "b3:".len() + 64);
        for edited in [dependency_bump, new_member, package_edit, reordered_metadata] {
            assert_ne!(edited, BASE_MANIFEST);
            assert_eq!(config_hash_for(&edited, "[octet]\n"), baseline, "manifest edit changed config hash:\n{edited}");
        }
    }

    #[test]
    fn config_hash_tracks_runner_metadata_and_dylint_config() {
        let baseline = config_hash_for(BASE_MANIFEST, "[octet]\n");
        let octet_edit = BASE_MANIFEST.replace("cargo_check_args = [\"--lib\"]", "cargo_check_args = [\"--tests\"]");
        let legacy_table =
            format!("{BASE_MANIFEST}\n[workspace.metadata.slotcar]\ndefault_scope = [\"-p\", \"core\"]\n");
        let dylint_edit = config_hash_for(BASE_MANIFEST, "[octet]\ndisabled_lints = [\"no_unwrap\"]\n");
        let missing_manifest = {
            let root = tempfile::tempdir().expect("missing manifest tempdir");
            config_hash(root.path(), &["--workspace".to_owned()], &["--lib".to_owned()], None)
        };
        let unparsable = config_hash_for("[workspace\n", "[octet]\n");

        assert_ne!(config_hash_for(&octet_edit, "[octet]\n"), baseline);
        assert_ne!(config_hash_for(&legacy_table, "[octet]\n"), baseline);
        assert_ne!(dylint_edit, baseline);
        assert_eq!(unparsable, config_hash_for("[workspace\n", "[octet]\n"));
        assert_ne!(unparsable, config_hash_for("[workspace\n\n", "[octet]\n"));
        assert_ne!(unparsable, baseline);
        assert_ne!(missing_manifest, baseline);
    }

    #[test]
    fn strict_profile_identity_changes_profile_hash_deterministically() {
        let scope_args = vec!["-p".to_owned(), "domain".to_owned()];
        let cargo_args = vec!["--lib".to_owned()];
        let robust = StrictProfileIdentity {
            profile_id: "robust",
            policy_hash: ROBUST_POLICY_HASH,
        };
        let bounded = StrictProfileIdentity {
            profile_id: "bounded-systems",
            policy_hash: BOUNDED_POLICY_HASH,
        };

        let default_hash = profile_hash(&scope_args, &cargo_args, "json", CONFIG_HASH, None);
        let expected_default_hash = full_hash(
            &serde_json::json!({
                "scope_args": scope_args,
                "cargo_check_args": cargo_args,
                "output_format": "json",
                "config_hash": CONFIG_HASH,
            })
            .to_string(),
        );
        let robust_hash = profile_hash(&scope_args, &cargo_args, "json", CONFIG_HASH, Some(robust));
        let repeated_robust_hash = profile_hash(&scope_args, &cargo_args, "json", CONFIG_HASH, Some(robust));
        let bounded_hash = profile_hash(&scope_args, &cargo_args, "json", CONFIG_HASH, Some(bounded));

        assert_eq!(default_hash, expected_default_hash);
        assert_eq!(robust_hash, repeated_robust_hash);
        assert_ne!(default_hash, robust_hash);
        assert_ne!(robust_hash, bounded_hash);
    }
}
