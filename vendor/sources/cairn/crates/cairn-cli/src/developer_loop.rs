use std::{collections::BTreeSet, error::Error, path::Path, path::PathBuf};

use cairn_core::{
    admit_developer_loop_policy, build_developer_loop_plan, canonical_json_hash,
    developer_loop_plan_identity, developer_loop_profile_identity, find_developer_loop_profile,
    validate_developer_loop_receipt, AdmittedDeveloperLoopPolicy, Policy,
};
use serde_json::{json, Value};

use crate::support::{load_policy_file, print_json, CapabilityRoot, CapabilityRootKind};

const RECEIPT_MAX_BYTES: u64 = 65_536;

// r[impl cairn.developer_loop_profiles.cli]
// r[impl cairn.developer_loop_profiles.cli.inspect]
// r[impl cairn.developer_loop_profiles.cli.no_execution]
// r[impl cairn.developer_loop_profiles.dogfood]
// r[impl cairn.developer_loop_profiles.dogfood.current]
pub(crate) fn run_developer_loop(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("developer-loop requires list, plan, or verify".into());
    };
    match command {
        "list" => run_list(&args[1..]),
        "plan" => run_plan(&args[1..]),
        "verify" => run_verify(&args[1..]),
        unknown => Err(format!("unknown developer-loop command: {unknown}").into()),
    }
}

fn load_catalog(
    policy_path: &Path,
) -> Result<(Policy, AdmittedDeveloperLoopPolicy, String), Box<dyn Error>> {
    let policy = load_policy_file(policy_path.to_path_buf())?;
    let policy_hash = canonical_json_hash(&policy.raw);
    let admitted = admit_developer_loop_policy(policy.developer_loop_policy.clone())
        .map_err(|issues| format!("developer-loop policy is invalid: {issues:?}"))?;
    Ok((policy, admitted, policy_hash))
}

fn policy_option(args: &[String]) -> PathBuf {
    crate::support::read_option(
        args.to_vec(),
        "--policy",
        crate::support::default_policy_path(),
    )
}

fn run_list(args: &[String]) -> Result<(), Box<dyn Error>> {
    let policy_path = policy_option(args);
    let (policy, admitted, policy_hash) = load_catalog(&policy_path)?;
    let profiles = admitted
        .policy
        .profiles
        .iter()
        .map(|profile| {
            json!({
                "id": profile.id,
                "purpose": profile.purpose,
                "checks": profile.checks,
                "budget_id": profile.budget_id,
                "effect_posture": profile.effect_posture,
                "cache_mode": profile.cache_policy.mode,
                "full_gate_refs": profile.full_gate_refs,
                "profile_identity": developer_loop_profile_identity(profile),
            })
        })
        .collect::<Vec<_>>();
    let value = crate::support::with_receipt_hash(json!({
        "schema": "cairn.developer-loop-list.v1",
        "command": "developer-loop list",
        "valid": true,
        "profiles": profiles,
        "policy": policy.name,
        "policy_hash": policy_hash,
        "non_claims": admitted.policy.required_non_claims,
    }));
    print_json(value)
}

fn run_plan(args: &[String]) -> Result<(), Box<dyn Error>> {
    let policy_path = policy_option(args);
    let (policy, admitted, policy_hash) = load_catalog(&policy_path)?;
    let profile_id = args
        .iter()
        .find(|value| !value.starts_with("--"))
        .cloned()
        .ok_or("developer-loop plan requires a profile ID")?;
    let profile = find_developer_loop_profile(&admitted, &profile_id)?;
    let plan = build_developer_loop_plan(&admitted, &profile_id)
        .map_err(|issues| format!("developer-loop plan failed: {issues:?}"))?;
    let value = crate::support::with_receipt_hash(json!({
        "schema": "cairn.developer-loop-plan-receipt.v1",
        "command": "developer-loop plan",
        "valid": true,
        "dry_run": true,
        "profile": profile_id,
        "profile_identity": developer_loop_profile_identity(profile),
        "policy": policy.name,
        "policy_hash": policy_hash,
        "plan": plan,
        "plan_hash": developer_loop_plan_identity(&plan),
        "execution_note": "developer-loop commands do not execute checks; use the repository-owned execution path",
        "non_claims": admitted.policy.required_non_claims,
    }));
    print_json(value)
}

fn run_verify(args: &[String]) -> Result<(), Box<dyn Error>> {
    let policy_path = policy_option(args);
    let (_, admitted, policy_hash) = load_catalog(&policy_path)?;
    let profile_id = args
        .iter()
        .find(|value| !value.starts_with("--"))
        .cloned()
        .ok_or("developer-loop verify requires a profile ID")?;
    find_developer_loop_profile(&admitted, &profile_id)?;
    let receipt_path = crate::support::read_string_option(args, "--receipt")
        .ok_or("developer-loop verify requires --receipt FILE")?;
    let subject_hash = crate::support::read_string_option(args, "--subject-hash")
        .ok_or("developer-loop verify requires --subject-hash HASH")?;
    if !is_blake3_hex(&subject_hash) {
        return Err("developer-loop verify requires a BLAKE3 hex --subject-hash".into());
    }
    let dependencies = collect_repeated(args, "--dependency-hash")?;
    for dependency in &dependencies {
        if !is_blake3_hex(dependency) {
            return Err(
                "developer-loop verify requires BLAKE3 hex --dependency-hash values".into(),
            );
        }
    }
    let root = crate::support::read_option(args.to_vec(), "--root", ".");
    let capability = CapabilityRoot::open(CapabilityRootKind::Repository, &root)?;
    let bytes = capability.read_bounded_path(&PathBuf::from(&receipt_path), RECEIPT_MAX_BYTES)?;
    let receipt: Value = serde_json::from_slice(&bytes)
        .map_err(|_| "developer-loop receipt is not valid JSON".to_string())?;
    let summary = validate_developer_loop_receipt(
        &admitted,
        &profile_id,
        &policy_hash,
        &subject_hash,
        &dependencies,
        &receipt,
    );
    let value = crate::support::with_receipt_hash(json!(summary));
    print_json(value)
}

fn collect_repeated(args: &[String], flag: &str) -> Result<BTreeSet<String>, Box<dyn Error>> {
    let mut values = BTreeSet::new();
    let mut index = 0;
    while index < args.len() {
        if args[index] == flag {
            let value = args
                .get(index + 1)
                .filter(|value| !value.is_empty() && !value.starts_with("--"))
                .ok_or_else(|| format!("{flag} requires a value"))?;
            values.insert(value.clone());
            index += 2;
            continue;
        }
        index += 1;
    }
    Ok(values)
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == 64 && value.chars().all(|character| character.is_ascii_hexdigit())
}
