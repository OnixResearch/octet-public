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

fn config_hash(
    workspace_root: &Path,
    scope_args: &[String],
    cargo_check_args: &[String],
    strict_profile: Option<StrictProfileIdentity<'_>>,
) -> String {
    let mut files = Vec::new();
    push_file_hash(&workspace_root.join("Cargo.toml"), &mut files);
    push_file_hash(&workspace_root.join("dylint.toml"), &mut files);
    let payload = match strict_profile {
        Some(profile) => serde_json::json!({
            "files": files,
            "effective_scope_args": scope_args,
            "effective_cargo_check_args": cargo_check_args,
            "strict_profile": {
                "profile_id": profile.profile_id,
                "policy_hash": profile.policy_hash,
            },
        }),
        None => serde_json::json!({
            "files": files,
            "effective_scope_args": scope_args,
            "effective_cargo_check_args": cargo_check_args,
        }),
    };
    full_hash(&payload.to_string())
}

fn push_file_hash(path: &Path, files: &mut Vec<serde_json::Value>) {
    let relative = path.file_name().and_then(|name| name.to_str()).unwrap_or("unknown");
    files.push(serde_json::json!({
        "path": relative,
        "hash": file_hash(path),
    }));
}

fn file_hash(path: &Path) -> Option<String> {
    fs::read(path).ok().map(|bytes| format!("b3:{}", blake3::hash(&bytes).to_hex()))
}

fn rustc_version(toolchain: &str) -> String {
    Command::new("rustc")
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .arg("-Vv")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map_or_else(|| "unknown".to_owned(), |text| text.lines().next().unwrap_or("unknown").to_owned())
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
                "files": [
                    { "path": "Cargo.toml", "hash": file_hash(&root.path().join("Cargo.toml")) },
                    { "path": "dylint.toml", "hash": file_hash(&root.path().join("dylint.toml")) },
                ],
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
