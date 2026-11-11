use super::support::*;
use super::types::*;

pub(crate) fn replay_cases_from_policy(policy: &Policy) -> Result<Vec<ReplayCase>, Box<dyn Error>> {
    let cases = policy
        .raw
        .get("determinism_replay_policy")
        .and_then(|value| value.get("cases"))
        .and_then(Value::as_array)
        .ok_or("policy missing determinism_replay_policy.cases")?;
    let mut parsed = cases
        .iter()
        .map(replay_case_from_policy_value)
        .collect::<Result<Vec<_>, _>>()?;
    parsed.sort_by_key(|case| case.id.clone());
    Ok(parsed)
}

pub(crate) fn replay_groups_from_policy(
    policy: &Policy,
) -> Result<Vec<ReplayGroup>, Box<dyn Error>> {
    let groups = policy
        .raw
        .get("determinism_replay_policy")
        .and_then(|value| value.get("groups"))
        .and_then(Value::as_array)
        .ok_or("policy missing determinism_replay_policy.groups")?;
    let mut parsed = groups
        .iter()
        .map(replay_group_from_policy_value)
        .collect::<Result<Vec<_>, _>>()?;
    parsed.sort_by_key(|group| group.id.clone());
    Ok(parsed)
}

pub(in crate::determinism) fn replay_entropy_inventory_from_policy(
    policy: &Policy,
) -> Result<Vec<ReplayEntropyEntry>, Box<dyn Error>> {
    let entries = policy
        .raw
        .get("determinism_replay_policy")
        .and_then(|value| value.get("entropy_inventory"))
        .and_then(Value::as_array)
        .ok_or("policy missing determinism_replay_policy.entropy_inventory")?;
    let mut parsed = entries
        .iter()
        .map(replay_entropy_entry_from_policy_value)
        .collect::<Result<Vec<_>, _>>()?;
    parsed.sort_by_key(|entry| entry.id.clone());
    Ok(parsed)
}

fn replay_entropy_entry_from_policy_value(
    value: &Value,
) -> Result<ReplayEntropyEntry, Box<dyn Error>> {
    Ok(ReplayEntropyEntry {
        id: required_string(value, "id")?.to_owned(),
        class: required_string(value, "class")?.to_owned(),
        source: required_string(value, "source")?.to_owned(),
        handling: required_string(value, "handling")?.to_owned(),
        receipt_field: required_string(value, "receipt_field")?.to_owned(),
        description: required_string(value, "description")?.to_owned(),
    })
}

pub(in crate::determinism) fn replay_environment_facts_from_policy(
    policy: &Policy,
) -> Result<Vec<ReplayEnvironmentFact>, Box<dyn Error>> {
    let entries = policy
        .raw
        .get("determinism_replay_policy")
        .and_then(|value| value.get("environment_attestation"))
        .and_then(Value::as_array)
        .ok_or("policy missing determinism_replay_policy.environment_attestation")?;
    let mut parsed = entries
        .iter()
        .map(replay_environment_fact_from_policy_value)
        .collect::<Result<Vec<_>, _>>()?;
    parsed.sort_by_key(|entry| entry.id.clone());
    Ok(parsed)
}

fn replay_environment_fact_from_policy_value(
    value: &Value,
) -> Result<ReplayEnvironmentFact, Box<dyn Error>> {
    Ok(ReplayEnvironmentFact {
        id: required_string(value, "id")?.to_owned(),
        class: required_string(value, "class")?.to_owned(),
        source: required_string(value, "source")?.to_owned(),
        handling: required_string(value, "handling")?.to_owned(),
        receipt_field: required_string(value, "receipt_field")?.to_owned(),
        description: required_string(value, "description")?.to_owned(),
    })
}

fn replay_group_from_policy_value(value: &Value) -> Result<ReplayGroup, Box<dyn Error>> {
    let _description = required_string(value, "description")?;
    Ok(ReplayGroup {
        id: required_string(value, "id")?.to_owned(),
        cases: required_string_array(value, "cases")?,
    })
}

pub(super) fn validate_replay_groups(
    cases: &[ReplayCase],
    groups: &[ReplayGroup],
) -> Result<(), Box<dyn Error>> {
    let case_ids = cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    for group in groups {
        for case_id in &group.cases {
            if !case_ids.contains(case_id.as_str()) {
                return Err(format!(
                    "policy replay group {} references unknown case {}",
                    group.id, case_id
                )
                .into());
            }
        }
    }
    Ok(())
}

pub(super) fn select_replay_cases(
    cases: &[ReplayCase],
    groups: &[ReplayGroup],
    selected_case: Option<&str>,
    selected_group: Option<&str>,
) -> Result<ReplaySelection, Box<dyn Error>> {
    if selected_case.is_some() && selected_group.is_some() {
        return Ok(ReplaySelection {
            cases: Vec::new(),
            selected_groups: Vec::new(),
        });
    }
    if let Some(selected) = selected_case {
        return Ok(ReplaySelection {
            cases: cases
                .iter()
                .filter(|case| case.id == selected)
                .cloned()
                .collect(),
            selected_groups: Vec::new(),
        });
    }
    if let Some(selected) = selected_group {
        let Some(group) = groups.iter().find(|group| group.id == selected) else {
            return Ok(ReplaySelection {
                cases: Vec::new(),
                selected_groups: Vec::new(),
            });
        };
        let selected_ids = group
            .cases
            .iter()
            .map(String::as_str)
            .collect::<std::collections::BTreeSet<_>>();
        return Ok(ReplaySelection {
            cases: cases
                .iter()
                .filter(|case| selected_ids.contains(case.id.as_str()))
                .cloned()
                .collect(),
            selected_groups: vec![group.id.clone()],
        });
    }
    Ok(ReplaySelection {
        cases: cases.to_vec(),
        selected_groups: Vec::new(),
    })
}

fn replay_case_from_policy_value(value: &Value) -> Result<ReplayCase, Box<dyn Error>> {
    let id = required_string(value, "id")?.to_owned();
    let command_kind = required_string(value, "command_kind")?.to_owned();
    let args = required_string_array(value, "args")?;
    let fixture_root = required_string(value, "fixture_root")?.to_owned();
    let comparison_modes = required_string_array(value, "comparison_modes")?;
    let hash_fields = required_string_array(value, "hash_fields")?;
    let display_only_paths = required_string_array(value, "display_only_paths")?;
    let normalization_rules = required_string_array(value, "normalization_rules")?;
    let kind = replay_case_kind(&command_kind, &args)?;
    Ok(ReplayCase {
        id,
        command_kind,
        args,
        fixture_root,
        comparison_modes,
        kind,
        hash_fields,
        display_only_paths,
        normalization_rules,
    })
}

pub(in crate::determinism) fn required_string<'a>(
    value: &'a Value,
    field: &str,
) -> Result<&'a str, Box<dyn Error>> {
    value
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("policy replay case missing string field {field}").into())
}

pub(in crate::determinism) fn required_string_array(
    value: &Value,
    field: &str,
) -> Result<Vec<String>, Box<dyn Error>> {
    Ok(value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("policy replay case missing array field {field}"))?
        .iter()
        .map(|item| {
            item.as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("policy replay case field {field} contains non-string"))
        })
        .collect::<Result<Vec<_>, _>>()?)
}

fn replay_case_kind(command_kind: &str, args: &[String]) -> Result<ReplayCaseKind, Box<dyn Error>> {
    match command_kind {
        "agent_completion_validate" => Ok(ReplayCaseKind::AgentCompletionValidate),
        "agent_receipt_preserves_hash" => Ok(ReplayCaseKind::AgentReceiptPreservesHash),
        "agent_receipt_validate" => Ok(ReplayCaseKind::AgentReceiptValidate),
        "archive_dry_run" => Ok(ReplayCaseKind::ArchiveDryRun),
        "preserves_corpus_freshness" => Ok(ReplayCaseKind::PreservesCorpusFreshness),
        "gate" => {
            let stage = args
                .get(1)
                .ok_or("policy gate replay case missing stage argument")?;
            let change = args
                .get(2)
                .ok_or("policy gate replay case missing change argument")?
                .clone();
            Ok(ReplayCaseKind::Gate {
                stage: parse_gate_stage(stage)?,
                change,
            })
        }
        "receipt_preserves_diff" => Ok(ReplayCaseKind::ReceiptPreservesDiff),
        "mcp_describe" => Ok(ReplayCaseKind::McpDescribe),
        "mcp_call" => Ok(ReplayCaseKind::McpStatus),
        "migrate_dry_run" => Ok(ReplayCaseKind::MigrateDryRun),
        "release_readiness" => Ok(ReplayCaseKind::ReleaseReadiness),
        "status" => Ok(ReplayCaseKind::Status),
        "sync_dry_run" => Ok(ReplayCaseKind::SyncDryRun),
        "validate" => Ok(ReplayCaseKind::Validate),
        "view" => {
            let view = args
                .get(1)
                .ok_or("policy view replay case missing view argument")?
                .clone();
            if !matches!(view.as_str(), "kanban" | "mindmap" | "graph") {
                return Err(format!("unsupported policy replay view: {view}").into());
            }
            Ok(ReplayCaseKind::View { view })
        }
        unknown => Err(format!("unknown replay command kind in policy: {unknown}").into()),
    }
}
