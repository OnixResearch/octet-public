use std::{error::Error, fs, path::PathBuf};

use cairn_core::{
    canonical_json_hash, evaluate_registry_graph, merge_global_registry_snapshot,
    query_registry_exact_id, query_registry_tags, verify_global_registry_snapshot,
    GlobalRegistrySnapshot, RegistryGraphDirection,
};
use serde_json::{json, Value};

use crate::support::{
    default_policy_path, has_flag, load_policy_file, path_string, print_json, read_string_option,
    with_receipt_hash,
};

mod ingest;
mod policy;
mod storage;

use ingest::ingest_registry_artifact;
use policy::{
    global_registry_policy, require_adapter, validate_global_registry_policy, GlobalRegistryPolicy,
};
use storage::{atomic_write_json, read_snapshot, storage_adapter};

// r[impl cairn.global_registry.scope.explicit]
// r[impl cairn.global_registry.fcis.shell]
pub(crate) fn run_global_registry(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("registry requires import, query, graph, export, or verify".into());
    };
    let rest = args[1..].to_vec();
    let policy_path = read_string_option(&rest, "--policy")
        .map(PathBuf::from)
        .unwrap_or_else(|| default_policy_path().into());
    let policy = load_policy_file(policy_path)?;
    let registry_policy = global_registry_policy(&policy)?;
    validate_global_registry_policy(&registry_policy)?;
    match command {
        "import" => print_json(import_receipt(&rest, &registry_policy)?),
        "query" => print_json(query_receipt(&rest, &registry_policy)?),
        "graph" => print_json(graph_receipt(&rest, &registry_policy)?),
        "export" => print_json(export_receipt(&rest, &registry_policy)?),
        "verify" => print_json(verify_receipt(&rest, &registry_policy)?),
        unknown => Err(format!("unknown registry command: {unknown}").into()),
    }
}

fn import_receipt(args: &[String], policy: &GlobalRegistryPolicy) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.global_registry.adapters.storage]
    // r[impl cairn.global_registry.query]
    let scope_id = required_option(args, "--scope")?;
    let adapter_id = required_option(args, "--adapter")?;
    let ingest_id = required_option(args, "--ingest")?;
    let store = PathBuf::from(required_option(args, "--store")?);
    let input_path = PathBuf::from(required_option(args, "--input")?);
    let execute = has_flag(args, "--execute");
    require_adapter(policy, &adapter_id, "storage")?;
    let ingest_profile = require_adapter(policy, &ingest_id, "ingest")?;
    if execute && adapter_id == "memory.v1" {
        return Err("memory.v1 is a conformance adapter and cannot persist a CLI import".into());
    }
    let input = read_json(&input_path, policy.max_artifact_bytes)?;
    let candidates = ingest_registry_artifact(ingest_profile, &input)?;
    let mut storage = storage_adapter(&adapter_id)?;
    let observed = storage.load(&store, &scope_id, policy)?;
    let expected = read_string_option(args, "--expected-snapshot")
        .unwrap_or_else(|| observed.snapshot_hash.clone());
    let snapshot_report = merge_global_registry_snapshot(&observed, candidates, &policy.core);
    let expected_matches = expected == observed.snapshot_hash;
    let mut issues = snapshot_report
        .issues
        .iter()
        .map(|issue| serde_json::to_value(issue).expect("registry issue serializes"))
        .collect::<Vec<_>>();
    if !expected_matches {
        issues.push(json!({
            "code": "registry.storage_snapshot_conflict",
            "record_kind": "storage",
            "stable_id": adapter_id,
            "field": "expected_snapshot_hash",
            "expected": expected,
            "actual": observed.snapshot_hash,
            "message": "observed storage snapshot differs from the explicit expected prior snapshot",
        }));
    }
    issues.sort_by_key(canonical_json_hash);
    let blocked = snapshot_report.snapshot.is_none() || !expected_matches;
    let mut mutated = false;
    if execute && !blocked {
        let candidate = snapshot_report
            .snapshot
            .as_ref()
            .ok_or("candidate snapshot missing after successful validation")?;
        mutated = candidate.snapshot_hash != observed.snapshot_hash;
        if mutated {
            storage.commit(&store, &expected, candidate, policy)?;
        }
    }
    let valid = !blocked;
    Ok(with_receipt_hash(json!({
        "command": "registry import",
        "scope_id": scope_id,
        "adapter": storage.id(),
        "ingest_adapter": ingest_id,
        "store": path_string(store),
        "dry_run": !execute,
        "mutated": mutated,
        "blocked": blocked,
        "valid": valid,
        "verdict": if valid { "PASS" } else { "FAIL" },
        "observed_snapshot_hash": observed.snapshot_hash,
        "expected_snapshot_hash": expected,
        "candidate_snapshot": snapshot_report.snapshot,
        "issues": issues,
        "input_hash": canonical_json_hash(&input),
    })))
}

fn query_receipt(args: &[String], policy: &GlobalRegistryPolicy) -> Result<Value, Box<dyn Error>> {
    let (adapter_id, registry, snapshot) = load_selected_snapshot(args, policy)?;
    let stable_id = read_string_option(args, "--id");
    let tag_values = option_values(args, "--tag");
    let query = match (stable_id, tag_values.is_empty()) {
        (Some(stable_id), true) => query_registry_exact_id(&snapshot, &stable_id),
        (None, false) => {
            let selectors = tag_values
                .into_iter()
                .map(|selector| parse_tag_selector(&selector))
                .collect::<Result<Vec<_>, _>>()?;
            query_registry_tags(&snapshot, &selectors)
        }
        _ => return Err("registry query requires exactly one of --id or --tag".into()),
    };
    Ok(with_receipt_hash(json!({
        "command": "registry query",
        "registry": path_string(registry),
        "adapter": adapter_id,
        "snapshot_hash": snapshot.snapshot_hash,
        "valid": query.valid,
        "verdict": query.verdict,
        "query": query,
    })))
}

fn graph_receipt(args: &[String], policy: &GlobalRegistryPolicy) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.global_registry.query.graph]
    let (adapter_id, registry, snapshot) = load_selected_snapshot(args, policy)?;
    let profile_id = required_option(args, "--profile")?;
    let profile = policy
        .graph_profiles
        .get(&profile_id)
        .ok_or_else(|| format!("unsupported registry graph profile: {profile_id}"))?;
    let start = read_string_option(args, "--start");
    let direction = match read_string_option(args, "--direction")
        .unwrap_or_else(|| "descendants".to_string())
        .as_str()
    {
        "ancestors" => RegistryGraphDirection::Ancestors,
        "descendants" => RegistryGraphDirection::Descendants,
        "both" => RegistryGraphDirection::Both,
        unknown => return Err(format!("unsupported registry graph direction: {unknown}").into()),
    };
    let graph = evaluate_registry_graph(&snapshot, profile, start.as_deref(), direction);
    Ok(with_receipt_hash(json!({
        "command": "registry graph",
        "registry": path_string(registry),
        "adapter": adapter_id,
        "snapshot_hash": snapshot.snapshot_hash,
        "valid": graph.valid,
        "verdict": graph.verdict,
        "graph": graph,
    })))
}

fn export_receipt(args: &[String], policy: &GlobalRegistryPolicy) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.global_registry.portability.export]
    let (adapter_id, registry, snapshot) = load_selected_snapshot(args, policy)?;
    let output = PathBuf::from(required_option(args, "--output")?);
    let execute = has_flag(args, "--execute");
    if execute {
        atomic_write_json(&output, &snapshot)?;
    }
    Ok(with_receipt_hash(json!({
        "command": "registry export",
        "registry": path_string(registry),
        "adapter": adapter_id,
        "output": path_string(output),
        "snapshot_hash": snapshot.snapshot_hash,
        "dry_run": !execute,
        "mutated": execute,
        "valid": true,
        "verdict": "PASS",
    })))
}

fn verify_receipt(args: &[String], policy: &GlobalRegistryPolicy) -> Result<Value, Box<dyn Error>> {
    let registry = PathBuf::from(required_option(args, "--registry")?);
    let snapshot = read_snapshot(&registry, policy.max_artifact_bytes)?;
    let report = verify_global_registry_snapshot(&snapshot, &policy.core);
    Ok(with_receipt_hash(json!({
        "command": "registry verify",
        "registry": path_string(registry),
        "valid": report.valid,
        "verdict": report.verdict,
        "report": report,
        "max_artifact_bytes": policy.max_artifact_bytes,
    })))
}

fn load_selected_snapshot(
    args: &[String],
    policy: &GlobalRegistryPolicy,
) -> Result<(String, PathBuf, GlobalRegistrySnapshot), Box<dyn Error>> {
    let scope_id = required_option(args, "--scope")?;
    let adapter_id = required_option(args, "--adapter")?;
    let registry = PathBuf::from(required_option(args, "--registry")?);
    require_adapter(policy, &adapter_id, "storage")?;
    if adapter_id == "memory.v1" {
        return Err("memory.v1 is available only to the storage conformance harness".into());
    }
    let storage = storage_adapter(&adapter_id)?;
    let snapshot = storage.load(&registry, &scope_id, policy)?;
    Ok((adapter_id, registry, snapshot))
}

fn read_json(path: &PathBuf, max_bytes: u64) -> Result<Value, Box<dyn Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err("registry input must not be a symlink".into());
    }
    if metadata.len() > max_bytes {
        return Err(format!(
            "registry input exceeds bounded read limit: {} > {max_bytes}",
            metadata.len()
        )
        .into());
    }
    Ok(serde_json::from_slice(&fs::read(path)?)?)
}

fn required_option(args: &[String], flag: &str) -> Result<String, Box<dyn Error>> {
    read_string_option(args, flag)
        .ok_or_else(|| format!("registry command requires {flag} VALUE").into())
}

fn option_values(args: &[String], flag: &str) -> Vec<String> {
    args.windows(2)
        .filter(|window| window[0] == flag)
        .map(|window| window[1].clone())
        .collect()
}

fn parse_tag_selector(value: &str) -> Result<(String, String, String), Box<dyn Error>> {
    let (namespace, rest) = value
        .split_once(':')
        .ok_or("tag selector must be namespace:key=value")?;
    let (key, selected_value) = rest
        .split_once('=')
        .ok_or("tag selector must be namespace:key=value")?;
    if namespace.is_empty() || key.is_empty() || selected_value.is_empty() {
        return Err("tag selector components must be non-empty".into());
    }
    Ok((
        namespace.to_string(),
        key.to_string(),
        selected_value.to_string(),
    ))
}
