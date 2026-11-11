use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};

use crate::support::{collect_files, read_string_option, safe_relative_path};

pub(crate) fn run_rename_classification(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.rename_cairn.classification]
    // r[impl cairn.supersede.rust_nickel_only_gates.rename_classification]
    let output = read_string_option(&args, "--output").map(PathBuf::from);
    let root = PathBuf::from(".");
    let occurrences = rename_classification_occurrences(&root)?;
    let mut buckets = std::collections::BTreeMap::<String, usize>::new();
    for occurrence in &occurrences {
        if let Some(bucket) = occurrence["bucket"].as_str() {
            *buckets.entry(bucket.to_string()).or_insert(0) += 1;
        }
    }
    let report = json!({
        "legacy": "ironspec",
        "canonical": "cairn",
        "total": occurrences.len(),
        "buckets": buckets,
        "occurrences": occurrences,
        "valid": buckets.get("needs-fix").copied().unwrap_or(0) == 0,
    });
    let rendered = serde_json::to_string_pretty(&report)? + "\n";
    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, &rendered)?;
    }
    print!("{rendered}");
    if report["valid"].as_bool() == Some(true) {
        Ok(())
    } else {
        Err("rename classification found needs-fix occurrences".into())
    }
}

fn rename_classification_occurrences(root: &Path) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut occurrences = Vec::new();
    for path in collect_files(root)? {
        let relative = safe_relative_path(root, &path);
        if !rename_classification_is_text_file(&relative)
            || rename_classification_is_skipped(&relative)
        {
            continue;
        }
        let Ok(text) = fs::read_to_string(&path) else {
            continue;
        };
        if relative == "crates/cairn-cli/src/rename_classification.rs" {
            continue;
        }
        for (line_index, line) in text.lines().enumerate() {
            if line.to_ascii_lowercase().contains("ironspec") {
                let bucket = rename_classification_bucket(&relative, line);
                occurrences.push(json!({
                    "path": relative,
                    "line": line_index + 1,
                    "bucket": bucket,
                    "text": line.trim(),
                }));
            }
        }
        if relative.to_ascii_lowercase().contains("ironspec") {
            let bucket = rename_classification_bucket(&relative, &relative);
            occurrences.push(json!({
                "path": relative,
                "line": 0,
                "bucket": bucket,
                "text": "<path>",
            }));
        }
    }
    occurrences.sort_by_key(|value| {
        format!(
            "{}:{:09}:{}:{}",
            value["path"].as_str().unwrap_or_default(),
            value["line"].as_u64().unwrap_or(0),
            value["bucket"].as_str().unwrap_or_default(),
            value["text"].as_str().unwrap_or_default()
        )
    });
    Ok(occurrences)
}

fn rename_classification_is_skipped(relative: &str) -> bool {
    relative
        .split('/')
        .any(|part| matches!(part, ".git" | "target" | "artifacts" | ".direnv"))
        || relative.ends_with("evidence/rename-classification.json")
}

fn rename_classification_is_text_file(relative: &str) -> bool {
    let suffixes = [
        ".md", ".rs", ".toml", ".nix", ".json", ".ncl", ".lock", ".txt", ".styx",
    ];
    relative == "Cargo.lock"
        || relative == "README.md"
        || suffixes.iter().any(|suffix| relative.ends_with(suffix))
}

fn rename_classification_bucket(path: &str, line: &str) -> &'static str {
    let text = line.to_ascii_lowercase();
    let path_text = path.to_ascii_lowercase();
    if path_text.contains("/archive/") || path_text.contains("openspec/changes/archive/") {
        return "historical-archive";
    }
    if path_text.starts_with("openspec/specs/") {
        return "requirement-id-history";
    }
    if path_text.contains("rename-ironspec-to-cairn")
        || path_text == "tools/check-rename-classification.rs"
    {
        return "intentional-reference";
    }
    if path_text.contains("ironspec")
        && (path_text.contains("docs/hermes")
            || path_text.contains("tool")
            || path_text.contains("src/ironspec.rs")
            || path_text.contains("ironspec-supersession"))
    {
        return "intentional-reference";
    }
    if text.contains("r[") && text.contains("ironspec.") {
        return "requirement-id-history";
    }
    if text.contains("ironspec") && text.contains("cairn") {
        return "intentional-reference";
    }
    let compatibility_tokens = [
        "legacy",
        "alias",
        "compat",
        "migration source",
        "migrate ironspec",
        "migrate_ironspec",
        "legacyironspec",
        "build_ironspec_migration_plan",
        "cargo_bin_exe_ironspec",
        "ironspec_archive_date",
        "name = \"ironspec\"",
        "path = \"src/ironspec.rs\"",
        "bin/ironspec",
        "invoked_as.ends_with(\"ironspec\")",
        "source_layout\"] == \"ironspec\"",
        "dry_run[\"source_layout\"], \"ironspec\"",
        "source_layout\", \"ironspec\"",
        "migrate-ironspec",
        "ironspec = {",
        "\"ironspec\" =>",
        "\"ironspec\",",
        "[\"cairn\", \"openspec\", \"ironspec\"]",
        "cairn, openspec, or ironspec",
        "<cairn|openspec|ironspec>",
    ];
    if text.contains("ironspec")
        && compatibility_tokens
            .iter()
            .any(|token| text.contains(token))
    {
        return "compatibility-alias";
    }
    "needs-fix"
}
