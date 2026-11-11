use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use cairn_core::{
    blake3_hex, completeness_receipt, review_ledger_analytics, semantic_review_issues_json,
    semantic_review_policy_profile_json, validate_review_metrics_evidence_boundary,
    validate_review_metrics_fixture_pairs, validate_review_metrics_guidance_sync,
    validate_review_metrics_registry, validate_review_receipt, validate_revision_handoff,
};
use serde_json::{json, Value};

use crate::support::{
    path_string, print_json, read_string_option, safe_relative_path, with_receipt_hash,
};

pub(crate) fn run_review(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(command) = iter.next() else {
        return Err(
            "review requires a subcommand: validate, ledger, revision, completeness, profiles, or metrics-regression"
                .into(),
        );
    };
    let rest = iter.collect::<Vec<_>>();
    match command.as_str() {
        "validate" => run_review_validate(rest),
        "ledger" => run_review_ledger(rest),
        "revision" => run_review_revision(rest),
        "completeness" => run_review_completeness(rest),
        "profiles" => run_review_profiles(rest),
        "metrics-regression" => run_review_metrics_regression(rest),
        unknown => Err(format!("unknown review subcommand: {unknown}").into()),
    }
}

fn run_review_validate(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.semantic_review.semantic_review_receipts.cli]
    // r[impl cairn.semantic_review.semantic_review_receipts.cli.cli.valid]
    // r[impl cairn.semantic_review.semantic_review_receipts.cli.cli.invalid]
    // r[impl cairn.semantic_review.semantic_review_boundary.fcis.fcis.cli_shell]
    let Some(path) = args.first() else {
        return Err("review validate requires a receipt path".into());
    };
    let root = read_root(&args);
    let receipt_path = PathBuf::from(path);
    let value = read_json_file(&receipt_path)?;
    let observed_hashes = observed_review_input_hashes(&root, &value);
    let report = validate_review_receipt(&value, &observed_hashes);
    let receipt = validation_receipt(
        "review validate",
        &root,
        &receipt_path,
        report.valid,
        &report.verdict,
        semantic_review_issues_json(&report.issues),
        json!({
            "review_receipt_hash": report.receipt_hash,
            "artifact_hashes": observed_hashes.iter().map(|(path, hash)| json!({"path": path, "hash": hash})).collect::<Vec<_>>()
        }),
    );
    print_json(receipt)?;
    if !report.valid {
        return Err("semantic review receipt invalid".into());
    }
    Ok(())
}

fn run_review_ledger(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.semantic_review.review_ledger_analytics.aggregate]
    // r[impl cairn.semantic_review.review_ledger_analytics.input_validation]
    // r[impl cairn.semantic_review.review_ledger_analytics.input_validation.input_validation.invalid_receipt]
    // r[impl cairn.semantic_review.review_ledger_analytics.input_validation.input_validation.sample_warning]
    let root = read_root(&args);
    let includes = read_all_options(&args, "--include");
    if includes.is_empty() {
        return Err("review ledger requires at least one --include path".into());
    }
    let mut paths = Vec::new();
    for include in includes {
        collect_json_inputs(&root.join(include), &mut paths)?;
    }
    paths.sort();
    let receipts = paths
        .iter()
        .map(read_json_file)
        .collect::<Result<Vec<_>, _>>()?;
    let analytics = review_ledger_analytics(&receipts);
    let receipt = with_receipt_hash(json!({
        "command": "review ledger",
        "root": path_string(&root),
        "inputs": paths.iter().map(|path| safe_relative_path(&root, path)).collect::<Vec<_>>(),
        "analytics": analytics,
        "valid": analytics.get("valid").and_then(Value::as_bool).unwrap_or(false),
        "verdict": analytics.get("verdict").and_then(Value::as_str).unwrap_or("FAIL"),
    }));
    let is_valid = receipt.get("valid").and_then(Value::as_bool) == Some(true);
    print_json(receipt)?;
    if !is_valid {
        return Err("review ledger inputs invalid".into());
    }
    Ok(())
}

fn run_review_revision(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.semantic_review.revision_handoff_receipts.validation]
    let mut iter = args.into_iter();
    let Some(command) = iter.next() else {
        return Err("review revision requires a subcommand: validate".into());
    };
    if command != "validate" {
        return Err(format!("unknown review revision subcommand: {command}").into());
    }
    let rest = iter.collect::<Vec<_>>();
    let Some(path) = rest.first() else {
        return Err("review revision validate requires a receipt path".into());
    };
    let root = read_root(&rest);
    let receipt_path = PathBuf::from(path);
    let value = read_json_file(&receipt_path)?;
    let report = validate_revision_handoff(&value);
    let receipt = validation_receipt(
        "review revision validate",
        &root,
        &receipt_path,
        report.valid,
        &report.verdict,
        semantic_review_issues_json(&report.issues),
        json!({"revision_handoff_receipt_hash": report.receipt_hash}),
    );
    print_json(receipt)?;
    if !report.valid {
        return Err("revision handoff receipt invalid".into());
    }
    Ok(())
}

fn run_review_completeness(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = read_root(&args);
    let change = read_string_option(&args, "--change");
    let scope = if let Some(change) = change {
        root.join(".cairn/changes").join(change)
    } else {
        root.clone()
    };
    let markdown_files = collect_markdown_files(&scope)?;
    let mut requirements = Vec::new();
    let mut tasks = Vec::new();
    let mut implementation_refs = Vec::new();
    let mut verification_refs = Vec::new();
    for file in markdown_files {
        let text = fs::read_to_string(&file)?;
        requirements.extend(extract_requirement_refs(&text));
        tasks.extend(extract_task_refs(&text));
    }
    for file in collect_rust_files(&root.join("crates"))? {
        let text = fs::read_to_string(&file)?;
        implementation_refs.extend(extract_prefixed_refs(&text, "r[impl "));
        verification_refs.extend(extract_prefixed_refs(&text, "r[verify "));
    }
    let completeness =
        completeness_receipt(requirements, tasks, implementation_refs, verification_refs);
    let receipt = with_receipt_hash(json!({
        "command": "review completeness",
        "root": path_string(&root),
        "scope": safe_relative_path(&root, &scope),
        "valid": completeness.get("valid").and_then(Value::as_bool).unwrap_or(false),
        "verdict": completeness.get("verdict").and_then(Value::as_str).unwrap_or("FAIL"),
        "completeness": completeness,
    }));
    let is_valid = receipt.get("valid").and_then(Value::as_bool) == Some(true);
    print_json(receipt)?;
    if !is_valid {
        return Err("completeness check found omissions".into());
    }
    Ok(())
}

fn run_review_profiles(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    if !args.is_empty() {
        return Err("review profiles does not accept arguments".into());
    }
    // r[impl cairn.semantic_review.review_gate_policy_profiles.sources.sources.allowed]
    // r[impl cairn.semantic_review.review_gate_policy_profiles.sources.sources.no_secret]
    // r[impl cairn.semantic_review.semantic_review_receipts.docs]
    // r[impl cairn.semantic_review.semantic_review_receipts.docs.docs.non_claims]
    // r[impl cairn.semantic_review.semantic_review_boundary.boundary.boundary.claims]
    print_json(semantic_review_policy_profile_json())?;
    Ok(())
}

fn run_review_metrics_regression(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.semantic_review.review_metrics_regression_rail.downstream_category_ownership]
    // r[impl cairn.semantic_review.review_metrics_regression_rail.registry_binds_surfaces]
    // r[impl cairn.semantic_review.review_metrics_regression_rail.fixture_pair_enforcement]
    // r[impl cairn.semantic_review.review_metrics_regression_rail.guidance_task_sync]
    // r[impl cairn.semantic_review.review_metrics_regression_rail.secret_free_evidence]
    let root = read_root(&args);
    let registry_path = required_path_option(&args, "--registry")?;
    let fixtures_path = required_path_option(&args, "--fixtures")?;
    let guidance_path = required_path_option(&args, "--guidance")?;
    let evidence_path = required_path_option(&args, "--evidence")?;
    let registry = read_json_file(&registry_path)?;
    let fixtures = read_json_file(&fixtures_path)?;
    let guidance = read_json_file(&guidance_path)?;
    let evidence = read_json_file(&evidence_path)?;
    let registry_report = validate_review_metrics_registry(&registry);
    let fixture_report = validate_review_metrics_fixture_pairs(&registry, &fixtures);
    let guidance_report = validate_review_metrics_guidance_sync(&registry, &guidance);
    let evidence_report = validate_review_metrics_evidence_boundary(&registry, &evidence);
    let reports = [
        ("registry", registry_report),
        ("fixtures", fixture_report),
        ("guidance", guidance_report),
        ("evidence", evidence_report),
    ];
    let valid = reports.iter().all(|(_, report)| report.valid);
    let verdict = if valid { "PASS" } else { "FAIL" };
    let receipt = with_receipt_hash(json!({
        "command": "review metrics-regression",
        "root": path_string(&root),
        "valid": valid,
        "verdict": verdict,
        "inputs": [
            input_receipt(&root, &registry_path)?,
            input_receipt(&root, &fixtures_path)?,
            input_receipt(&root, &guidance_path)?,
            input_receipt(&root, &evidence_path)?,
        ],
        "reports": reports.iter().map(|(name, report)| json!({
            "name": name,
            "valid": report.valid,
            "verdict": report.verdict,
            "issues": semantic_review_issues_json(&report.issues),
            "receipt_hash": report.receipt_hash,
        })).collect::<Vec<_>>(),
    }));
    print_json(receipt)?;
    if !valid {
        return Err("review metrics-regression inputs invalid".into());
    }
    Ok(())
}

fn validation_receipt(
    command: &str,
    root: &Path,
    path: &Path,
    valid: bool,
    verdict: &str,
    issues: Value,
    extra: Value,
) -> Value {
    let mut receipt = json!({
        "command": command,
        "root": path_string(root),
        "path": safe_relative_path(root, path),
        "valid": valid,
        "verdict": verdict,
        "issues": issues,
    });
    if let (Some(object), Some(extra_object)) = (receipt.as_object_mut(), extra.as_object()) {
        for (key, value) in extra_object {
            object.insert(key.clone(), value.clone());
        }
    }
    with_receipt_hash(receipt)
}

fn read_root(args: &[String]) -> PathBuf {
    read_string_option(args, "--root")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn read_all_options(args: &[String], flag: &str) -> Vec<String> {
    let mut values = Vec::new();
    let mut index = 0;
    while index + 1 < args.len() {
        if args[index] == flag {
            values.push(args[index + 1].clone());
            index += 2;
        } else {
            index += 1;
        }
    }
    values
}

fn required_path_option(args: &[String], flag: &str) -> Result<PathBuf, Box<dyn Error>> {
    read_string_option(args, flag)
        .map(PathBuf::from)
        .ok_or_else(|| format!("review metrics-regression requires {flag} path").into())
}

fn input_receipt(root: &Path, path: &Path) -> Result<Value, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    Ok(json!({
        "path": safe_relative_path(root, path),
        "hash": blake3_hex(&bytes),
    }))
}

fn read_json_file(path: impl AsRef<Path>) -> Result<Value, Box<dyn Error>> {
    let text = fs::read_to_string(path.as_ref())?;
    Ok(serde_json::from_str(&text)?)
}

fn observed_review_input_hashes(root: &Path, value: &Value) -> Vec<(String, String)> {
    value
        .get("reviewed_inputs")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|input| input.get("path").and_then(Value::as_str))
        .filter_map(|path| {
            let full_path = root.join(path);
            fs::read(&full_path)
                .ok()
                .map(|bytes| (path.to_string(), blake3_hex(&bytes)))
        })
        .collect()
}

fn collect_json_inputs(path: &Path, paths: &mut Vec<PathBuf>) -> Result<(), Box<dyn Error>> {
    if path.is_file() {
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(path.to_path_buf());
        }
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if child.is_dir() {
            collect_json_inputs(&child, paths)?;
        } else if child.extension().and_then(|ext| ext.to_str()) == Some("json") {
            paths.push(child);
        }
    }
    Ok(())
}

fn collect_markdown_files(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    collect_files_with_extension(path, "md")
}

fn collect_rust_files(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    collect_files_with_extension(path, "rs")
}

fn collect_files_with_extension(
    path: &Path,
    extension: &str,
) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    let mut files = Vec::new();
    if !path.exists() {
        return Ok(files);
    }
    if path.is_file() {
        if path.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            files.push(path.to_path_buf());
        }
        return Ok(files);
    }
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if child.is_dir() {
            files.extend(collect_files_with_extension(&child, extension)?);
        } else if child.extension().and_then(|ext| ext.to_str()) == Some(extension) {
            files.push(child);
        }
    }
    files.sort();
    Ok(files)
}

fn extract_requirement_refs(text: &str) -> Vec<String> {
    extract_refs(text)
        .into_iter()
        .filter(|reference| reference.starts_with("cairn."))
        .collect()
}

fn extract_task_refs(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.trim_start().starts_with("- ["))
        .flat_map(extract_refs)
        .collect()
}

fn extract_prefixed_refs(text: &str, prefix: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut remainder = text;
    while let Some(start) = remainder.find(prefix) {
        let after_prefix = &remainder[start + prefix.len()..];
        if let Some(end) = after_prefix.find(']') {
            refs.push(after_prefix[..end].to_string());
            remainder = &after_prefix[end + 1..];
        } else {
            break;
        }
    }
    refs
}

fn extract_refs(text: &str) -> Vec<String> {
    let mut refs = Vec::new();
    let mut remainder = text;
    while let Some(start) = remainder.find("r[") {
        let after_prefix = &remainder[start + 2..];
        if let Some(end) = after_prefix.find(']') {
            refs.push(after_prefix[..end].to_string());
            remainder = &after_prefix[end + 1..];
        } else {
            break;
        }
    }
    refs
}
