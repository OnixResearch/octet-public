use super::execution::replay_reproduce_command;
use super::support::*;
use super::types::*;

fn replay_environment_facts_json(entries: &[ReplayEnvironmentFact]) -> Vec<Value> {
    entries
        .iter()
        .map(|entry| {
            json!({
                "id": &entry.id,
                "class": &entry.class,
                "source": &entry.source,
                "handling": &entry.handling,
                "receipt_field": &entry.receipt_field,
                "description": &entry.description,
            })
        })
        .collect()
}

pub(in crate::determinism) fn replay_attestation_policy_hash(
    entries: &[ReplayEnvironmentFact],
) -> String {
    canonical_json_hash(&json!(replay_environment_facts_json(entries)))
}

pub(super) fn replay_environment_attestation(
    policy: &Policy,
    entries: &[ReplayEnvironmentFact],
    entropy_policy_hash: &str,
    selected_cases: &[String],
    selected_groups: &[String],
    selected_case: Option<&str>,
    selected_group: Option<&str>,
) -> Value {
    // r[impl cairn.supersede.replay_environment_attestation.policy]
    // r[impl cairn.supersede.replay_environment_attestation.receipt]
    let policy_facts = replay_environment_facts_json(entries);
    let attestation_policy_hash = replay_attestation_policy_hash(entries);
    let mut scope = json!({
        "selector": "default",
        "selected_case": Value::Null,
        "selected_group": Value::Null,
    });
    if let Some(case_id) = selected_case {
        scope["selector"] = Value::String("case".to_string());
        scope["selected_case"] = Value::String(case_id.to_string());
    }
    if let Some(group_id) = selected_group {
        scope["selector"] = Value::String("group".to_string());
        scope["selected_group"] = Value::String(group_id.to_string());
    }
    if selected_case.is_some() && selected_group.is_some() {
        scope["selector"] = Value::String("conflict".to_string());
    }
    let mut attestation = json!({
        "cairn_package": "cairn",
        "cairn_version": env!("CARGO_PKG_VERSION"),
        "generated_policy_hash": canonical_json_hash(&policy.raw),
        "entropy_policy_hash": entropy_policy_hash,
        "attestation_policy_hash": attestation_policy_hash,
        "selected_cases": selected_cases,
        "selected_groups": selected_groups,
        "replay_scope": scope,
        "policy_facts": policy_facts,
        "normalization_rules": ["canonical_json", "no_host_paths", "no_raw_environment_values"],
    });
    let attestation_hash = canonical_json_hash(&attestation);
    attestation["attestation_hash"] = Value::String(attestation_hash);
    attestation
}

pub(super) fn replay_entropy_inventory_json(entries: &[ReplayEntropyEntry]) -> Vec<Value> {
    entries
        .iter()
        .map(|entry| {
            json!({
                "id": &entry.id,
                "class": &entry.class,
                "source": &entry.source,
                "handling": &entry.handling,
                "receipt_field": &entry.receipt_field,
                "description": &entry.description,
            })
        })
        .collect()
}

pub(super) fn replay_entropy_assumptions(entries: &[ReplayEntropyEntry]) -> Vec<Value> {
    entries
        .iter()
        .filter(|entry| entry.handling != "reject")
        .map(|entry| {
            json!({
                "id": &entry.id,
                "class": &entry.class,
                "source": &entry.source,
                "handling": &entry.handling,
                "receipt_field": &entry.receipt_field,
            })
        })
        .collect()
}

pub(in crate::determinism) fn replay_entropy_policy_hash(entries: &[ReplayEntropyEntry]) -> String {
    canonical_json_hash(&json!(replay_entropy_inventory_json(entries)))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn replay_receipt(
    root: PathBuf,
    cases: Vec<Value>,
    mismatches: Vec<Value>,
    mutability_observations: Vec<Value>,
    skipped: Vec<Value>,
    nondeterminism_failures: Vec<Value>,
    available_cases: Vec<String>,
    available_groups: Vec<String>,
    selected_cases: Vec<String>,
    selected_groups: Vec<String>,
    entropy_inventory: Vec<Value>,
    entropy_assumptions: Vec<Value>,
    entropy_policy_hash: String,
    environment_attestation: Value,
) -> Value {
    let valid = mismatches.is_empty() && nondeterminism_failures.is_empty();
    let failed_cases = replay_failed_cases(&mismatches, &nondeterminism_failures);
    let summary = json!({
        "total_cases": cases.len(),
        "passed_cases": cases.len().saturating_sub(failed_cases.len()),
        "failed_cases": failed_cases,
        "skipped": skipped.len(),
        "mismatch_count": mismatches.len(),
        "nondeterminism_failure_count": nondeterminism_failures.len(),
    });
    json!({
        "command": "determinism replay",
        "root": path_string(root),
        "valid": valid,
        "verdict": if valid { "pass" } else { "fail" },
        "available_cases": available_cases,
        "available_groups": available_groups,
        "selected_cases": selected_cases,
        "selected_groups": selected_groups,
        "entropy_inventory": entropy_inventory,
        "entropy_assumptions": entropy_assumptions,
        "entropy_policy_hash": entropy_policy_hash,
        "environment_attestation": environment_attestation,
        "summary": summary,
        "cases": cases,
        "skipped": skipped,
        "mismatches": mismatches,
        "nondeterminism_failures": nondeterminism_failures,
        "mutability_observations": mutability_observations,
        "normalization_rules": ["portable_root_paths", "canonical_json"],
    })
}

fn replay_failed_cases(mismatches: &[Value], nondeterminism_failures: &[Value]) -> Vec<String> {
    let mut cases = mismatches
        .iter()
        .chain(nondeterminism_failures.iter())
        .filter_map(|value| {
            value
                .get("case")
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        })
        .collect::<Vec<_>>();
    cases.sort();
    cases.dedup();
    cases
}

pub(super) fn replay_nondeterminism_failures(
    entropy_inventory: &[ReplayEntropyEntry],
) -> Vec<Value> {
    // r[impl cairn.supersede.determinism_replay.hidden_entropy]
    // r[impl cairn.supersede.determinism_entropy_inventory.failures]
    let mut failures = entropy_inventory
        .iter()
        .filter(|entry| entry.handling == "reject" && entry.class == "environment")
        .flat_map(|entry| {
            entry
                .source
                .split(',')
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .filter_map(|name| {
                    env::var(name).ok().map(|_| {
                        json!({
                            "case": "workspace",
                            "class": &entry.class,
                            "code": "replay.hidden_entropy.environment",
                            "inventory_id": &entry.id,
                            "source": name,
                            "handling": &entry.handling,
                            "message": format!("{name} is replay-relevant entropy; unset it or declare a policy fixture before replay"),
                        })
                    })
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();
    failures.sort_by_key(canonical_json_hash);
    failures
}

pub(super) fn replay_mismatch(
    case: &ReplayCase,
    comparison: &str,
    field_path: &str,
    expected: &str,
    actual: &str,
    class: &str,
) -> Value {
    json!({
        "case": case.id,
        "comparison": comparison,
        "field_path": field_path,
        "class": class,
        "expected": expected,
        "actual": actual,
        "reproduce": replay_reproduce_command(Path::new(&case.fixture_root), Some(&case.id), None),
    })
}
