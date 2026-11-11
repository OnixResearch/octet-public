use std::{
    collections::BTreeMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use cairn_core::{
    blake3_hex, canonical_json_hash, classify_traceability_coverage, Policy,
    TraceabilityCoverageReport, TraceabilityFile, TraceabilityProfile, TraceabilitySource,
};
use serde_json::{json, Value};

use crate::support::{
    collect_files, default_policy_path, has_flag, load_policy_file, path_string, print_json,
    read_option, read_string_option, safe_relative_path, with_portable_receipt_hash,
};

pub(crate) fn traceability_coverage_receipt(
    root: PathBuf,
    policy_path: PathBuf,
) -> Result<Value, Box<dyn Error>> {
    // r[verify cairn.determinism.preserves_content_hash_binding_readiness.tracey]
    traceability_coverage_report(&root, &policy_path, None)
}

pub(crate) fn load_traceability_coverage_model(
    root: &Path,
    policy_path: &Path,
) -> Result<TraceabilityCoverageReport, Box<dyn Error>> {
    // r[impl cairn.lifecycle_views.knowledge_graph]
    let policy = load_policy_file(policy_path.to_path_buf())?;
    let selected_profile = selected_traceability_profile(&policy, None)?;
    let requirement_files =
        collect_traceability_files(root, &selected_profile.requirement_sources)?;
    let evidence_files = collect_traceability_files(root, &selected_profile.evidence_sources)?;
    Ok(classify_traceability_coverage(
        &selected_profile,
        &requirement_files,
        &evidence_files,
    ))
}

pub(crate) fn repository_gate_summary() -> Value {
    // r[impl cairn.supersede.traceability_ownership.optional_tracey]
    // r[verify cairn.supersede.traceability_ownership.optional_tracey.gates]
    // r[verify cairn.supersede.traceability_ownership.optional_tracey.docs]
    json!({
        "policy": "maintained-local-gates",
        "required_commands": [
            "cargo fmt --check",
            "cargo test --workspace",
            "cargo clippy --workspace --all-targets -- -D warnings",
            "nix build .#checks.x86_64-linux.slotcar --no-link -L",
            "nix flake check path:$PWD -L"
        ],
        "note": "The bundle records the maintained release gate policy; operators still run heavyweight repository gates outside this local receipt command. The maintained traceability gate is Cairn-owned and does not require the external Tracey binary."
    })
}

pub(crate) fn run_traceability(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.supersede.traceability_ownership.command]
    // r[impl cairn.supersede.traceability_ownership.command.maintained]
    run_coverage_command(args, "traceability")
}

pub(crate) fn run_tracey(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.supersede.rust_nickel_only_gates.tracey_coverage]
    // r[impl cairn.supersede.traceability_ownership.command]
    // r[impl cairn.supersede.traceability_ownership.command.alias]
    run_coverage_command(args, "tracey")
}

fn run_coverage_command(args: Vec<String>, namespace: &str) -> Result<(), Box<dyn Error>> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(format!("{namespace} requires a command: coverage").into());
    };
    if command != "coverage" {
        return Err(format!("unknown {namespace} command: {command}").into());
    }
    let rest = args.into_iter().skip(1).collect::<Vec<_>>();
    let root = read_option(rest.clone(), "--root", ".");
    let policy_path = read_option(rest.clone(), "--policy", default_policy_path());
    let profile_id = read_string_option(&rest, "--profile");
    let is_json = has_flag(&rest, "--json");
    let report = traceability_coverage_report(&root, &policy_path, profile_id.as_deref())?;
    if is_json || report["valid"].as_bool() != Some(true) {
        print_json(report.clone())?;
    } else {
        println!(
            "traceability coverage ok: {}/{} referenced (profile {})",
            report["referenced"].as_u64().unwrap_or_default(),
            report["requirements"].as_u64().unwrap_or_default(),
            report["profile_id"].as_str().unwrap_or("unknown")
        );
    }
    if report["valid"].as_bool() == Some(true) {
        Ok(())
    } else {
        Err(format!("{namespace} coverage failed").into())
    }
}

fn traceability_coverage_report(
    root: &Path,
    policy_path: &Path,
    profile_id: Option<&str>,
) -> Result<Value, Box<dyn Error>> {
    let policy = load_policy_file(policy_path.to_path_buf())?;
    let selected_profile = selected_traceability_profile(&policy, profile_id)?;
    let requirement_files =
        collect_traceability_files(root, &selected_profile.requirement_sources)?;
    let evidence_files = collect_traceability_files(root, &selected_profile.evidence_sources)?;
    let report =
        classify_traceability_coverage(&selected_profile, &requirement_files, &evidence_files);
    Ok(traceability_receipt(TraceabilityReceiptInput {
        root,
        policy: &policy,
        profile: &selected_profile,
        requirement_files: &requirement_files,
        evidence_files: &evidence_files,
        report: &report,
    }))
}

fn selected_traceability_profile(
    policy: &Policy,
    profile_id: Option<&str>,
) -> Result<TraceabilityProfile, Box<dyn Error>> {
    let selected_id = profile_id.unwrap_or(&policy.traceability_policy.default_profile);
    policy
        .traceability_policy
        .profiles
        .iter()
        .find(|profile| profile.id == selected_id)
        .cloned()
        .ok_or_else(|| format!("traceability profile not found: {selected_id}").into())
}

fn collect_traceability_files(
    root: &Path,
    sources: &[TraceabilitySource],
) -> Result<Vec<TraceabilityFile>, Box<dyn Error>> {
    let mut files = BTreeMap::new();
    for source in sources {
        let source_root = root.join(&source.root);
        let paths = traceability_source_paths(&source_root)?;
        for path in paths {
            if !traceability_path_has_allowed_extension(&path, &source.extensions) {
                continue;
            }
            let relative = safe_relative_path(root, &path);
            let text = fs::read_to_string(&path).map_err(|source_error| {
                format!("failed to read traceability source {relative}: {source_error}")
            })?;
            files.insert(
                relative.clone(),
                TraceabilityFile {
                    path: relative,
                    text,
                },
            );
        }
    }
    Ok(files.into_values().collect())
}

fn traceability_source_paths(source_root: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
    if !source_root.exists() {
        return Ok(Vec::new());
    }
    if source_root.is_file() {
        return Ok(vec![source_root.to_path_buf()]);
    }
    Ok(collect_files(source_root)?)
}

fn traceability_path_has_allowed_extension(path: &Path, extensions: &[String]) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extensions.iter().any(|allowed| allowed == extension))
}

struct TraceabilityReceiptInput<'a> {
    root: &'a Path,
    policy: &'a Policy,
    profile: &'a TraceabilityProfile,
    requirement_files: &'a [TraceabilityFile],
    evidence_files: &'a [TraceabilityFile],
    report: &'a TraceabilityCoverageReport,
}

fn traceability_receipt(input: TraceabilityReceiptInput<'_>) -> Value {
    let profile_json = traceability_profile_json(input.profile);
    let source_summary = traceability_source_summary(input.requirement_files, input.evidence_files);
    let policy_hash = canonical_json_hash(&input.policy.raw);
    let input_hash = canonical_json_hash(&json!({
        "profile": profile_json.clone(),
        "source_summary": source_summary.clone(),
        "policy_hash": policy_hash.clone(),
    }));
    with_portable_receipt_hash(
        input.root,
        json!({
            "command": "traceability coverage",
            "root": path_string(input.root),
            "profile_id": input.profile.id.clone(),
            "profile": profile_json,
            "requirements": input.report.requirement_count,
            "referenced": input.report.referenced_count,
            "missing": input.report.missing.clone(),
            "dangling": input.report.dangling.clone(),
            "source_summary": source_summary,
            "policy": input.policy.name.clone(),
            "policy_hash": policy_hash,
            "input_hash": input_hash,
            "valid": input.report.valid,
            "verdict": if input.report.valid { "pass" } else { "fail" },
        }),
    )
}

fn traceability_profile_json(profile: &TraceabilityProfile) -> Value {
    json!({
        "id": profile.id.clone(),
        "requirement_sources": traceability_sources_json(&profile.requirement_sources),
        "evidence_sources": traceability_sources_json(&profile.evidence_sources),
        "marker_verbs": profile.marker_verbs.clone(),
    })
}

fn traceability_sources_json(sources: &[TraceabilitySource]) -> Value {
    Value::Array(
        sources
            .iter()
            .map(|source| {
                json!({
                    "root": source.root.clone(),
                    "source_kind": source.source_kind.clone(),
                    "extensions": source.extensions.clone(),
                })
            })
            .collect(),
    )
}

fn traceability_source_summary(
    requirement_files: &[TraceabilityFile],
    evidence_files: &[TraceabilityFile],
) -> Value {
    let requirement_manifest = traceability_file_manifest(requirement_files);
    let evidence_manifest = traceability_file_manifest(evidence_files);
    json!({
        "requirement_files": requirement_files.len(),
        "evidence_files": evidence_files.len(),
        "requirement_manifest_hash": canonical_json_hash(&requirement_manifest),
        "evidence_manifest_hash": canonical_json_hash(&evidence_manifest),
    })
}

fn traceability_file_manifest(files: &[TraceabilityFile]) -> Value {
    let entries = files
        .iter()
        .map(|file| {
            json!({
                "path": file.path.clone(),
                "content_hash": blake3_hex(file.text.as_bytes()),
            })
        })
        .collect::<Vec<_>>();
    json!({ "files": entries })
}
