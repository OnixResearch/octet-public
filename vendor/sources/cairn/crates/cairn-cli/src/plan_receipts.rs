use std::path::Path;

use cairn_core::{
    canonical_json_hash, layout_name, migration_plan_identity, ArchivePlan, GateArtifact,
    GateReport, GateSnapshot, GateStage, LifecycleLayout, MigrationPlan, PlanAction,
    PlanActionKind, Policy, SyncPlan,
};
use serde_json::{json, Value};

use crate::native::change_status_json;
use crate::support::{path_string, portable_json_hash};

pub(crate) fn migration_plan_hash(root: &Path, policy: &Policy, plan: &MigrationPlan) -> String {
    portable_json_hash(
        root,
        &json!({
            "plan": migration_plan_identity(plan),
            "policy_hash": canonical_json_hash(&policy.raw),
        }),
    )
}

pub(crate) fn migration_plan_json(
    root: &Path,
    policy: &Policy,
    plan: &MigrationPlan,
    mutation_manifest: Value,
) -> Value {
    let plan_hash = migration_plan_hash(root, policy, plan);
    let mut value = json!({
        "command": format!("migrate_{}", layout_name(plan.source_layout)),
        "root": path_string(root),
        "source_layout": layout_name(plan.source_layout),
        "destination_layout": layout_name(plan.destination_layout),
        "dry_run": plan.dry_run,
        "blocked": plan.blocked,
        "mutated": plan.mutated,
        "actions": plan.actions.iter().map(|action| {
            json!({
                "source": action.source,
                "source_hash": action.source_hash,
                "destination": action.destination,
                "destination_hash": action.destination_hash,
            })
        }).collect::<Vec<_>>(),
        "conflicts": plan.conflicts,
        "policy": policy.name,
        "policy_hash": canonical_json_hash(&policy.raw),
        "plan_hash": plan_hash,
        "mutation_manifest": mutation_manifest,
    });
    value["source_manifest_hash"] = json!(portable_json_hash(
        root,
        &json!({
            "source_layout": layout_name(plan.source_layout),
            "sources": plan.actions.iter().map(|action| json!({
                "source": action.source,
                "source_hash": action.source_hash,
            })).collect::<Vec<_>>(),
        })
    ));
    value["destination_manifest_hash"] = json!(portable_json_hash(
        root,
        &json!({
            "destination_layout": layout_name(plan.destination_layout),
            "destinations": plan.actions.iter().map(|action| json!({
                "destination": action.destination,
                "destination_hash": action.destination_hash,
            })).collect::<Vec<_>>(),
        })
    ));
    value["manifest_hash"] = json!(portable_json_hash(root, &value));
    value
}

pub(crate) fn gate_input_json(
    layout: LifecycleLayout,
    stage: GateStage,
    policy: &Policy,
    snapshot: &GateSnapshot,
) -> Value {
    json!({
        "layout": layout_name(layout),
        "stage": stage.as_str(),
        "policy_hash": canonical_json_hash(&policy.raw),
        "change": change_status_json(&snapshot.change),
        "proposal": snapshot.proposal.as_ref().map(gate_artifact_json),
        "design": snapshot.design.as_ref().map(gate_artifact_json),
        "tasks": snapshot.tasks.as_ref().map(gate_artifact_json),
        "delta_specs": snapshot.delta_specs.iter().map(gate_artifact_json).collect::<Vec<_>>(),
    })
}

fn gate_artifact_json(artifact: &GateArtifact) -> Value {
    json!({
        "path": artifact.path,
        "markdown_hash": canonical_json_hash(&json!({"markdown": artifact.markdown})),
    })
}

pub(crate) fn sync_plan_json(
    root: &Path,
    layout: LifecycleLayout,
    policy: &Policy,
    plan: &SyncPlan,
    input_hash: &str,
    merge_preflight: &Value,
    mutation_manifest: Value,
) -> Value {
    let mut value = json!({
        "change": plan.change,
        "layout": layout_name(layout),
        "dry_run": plan.dry_run,
        "mutated": plan.mutated,
        "actions": plan.actions.iter().map(plan_action_json).collect::<Vec<_>>(),
        "delta_specs": plan.delta_specs,
        "spec_effect": plan.spec_effect,
        "blocked": plan.blocked,
        "reasons": plan.reasons,
        "policy": policy.name,
        "policy_hash": canonical_json_hash(&policy.raw),
        "input_hash": input_hash,
        "merge_preflight": merge_preflight,
        "mutation_manifest": mutation_manifest,
    });
    let plan_hash = portable_json_hash(root, &value);
    value["plan_hash"] = json!(plan_hash);
    value
}

pub(crate) fn archive_plan_json(
    root: &Path,
    layout: LifecycleLayout,
    policy: &Policy,
    plan: &ArchivePlan,
    input_hash: &str,
    mutation_manifest: Value,
) -> Value {
    let mut value = json!({
        "change": plan.change,
        "layout": layout_name(layout),
        "dry_run": plan.dry_run,
        "mutated": plan.mutated,
        "actions": plan.actions.iter().map(plan_action_json).collect::<Vec<_>>(),
        "spec_effect": plan.spec_effect,
        "blocked": plan.blocked,
        "reasons": plan.reasons,
        "policy": policy.name,
        "policy_hash": canonical_json_hash(&policy.raw),
        "input_hash": input_hash,
        "mutation_manifest": mutation_manifest,
    });
    let plan_hash = portable_json_hash(root, &value);
    value["plan_hash"] = json!(plan_hash);
    value
}

pub(crate) fn gate_report_json(
    layout: LifecycleLayout,
    policy: &Policy,
    report: &GateReport,
    input_hash: &str,
) -> Value {
    json!({
        "change": report.change,
        "stage": report.stage,
        "verdict": if report.issues.is_empty() { "PASS" } else { "FAIL" },
        "valid": report.issues.is_empty(),
        "issues": report.issues.iter().map(|issue| {
            json!({
                "code": issue.code,
                "message": issue.message,
                "scope": issue.scope,
                "promoted": issue.promoted,
            })
        }).collect::<Vec<_>>(),
        "substance": report.substance.iter().map(|inventory| json!({
            "kind": inventory.kind,
            "path": inventory.path,
            "substantive_lines": inventory.substantive_lines,
            "requirement_blocks": inventory.requirement_blocks,
            "substantive_requirement_blocks": inventory.substantive_requirement_blocks,
            "scenario_blocks": inventory.scenario_blocks,
            "checkbox_tasks": inventory.checkbox_tasks,
            "substantive_tasks": inventory.substantive_tasks,
            "task_done": inventory.task_done,
            "task_in_progress": inventory.task_in_progress,
            "task_todo": inventory.task_todo,
            "malformed_dependency_markers": inventory.malformed_dependency_markers,
        })).collect::<Vec<_>>(),
        "policy": policy.name,
        "policy_hash": canonical_json_hash(&policy.raw),
        "input_hash": input_hash,
        "layout": layout_name(layout),
    })
}

fn plan_action_json(action: &PlanAction) -> Value {
    json!({
        "kind": plan_action_kind(action.kind),
        "path": action.path,
        "description": action.description,
    })
}

fn plan_action_kind(kind: PlanActionKind) -> &'static str {
    match kind {
        PlanActionKind::SyncDeltaSpec => "sync_delta_spec",
        PlanActionKind::ArchiveChange => "archive_change",
    }
}
