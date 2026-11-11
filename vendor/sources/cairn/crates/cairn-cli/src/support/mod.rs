use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
};

// r[verify cli_maintainability.main_compat_release_followup.behavior_preserved]
// r[verify cli_maintainability.main_compat_release_followup.behavior_preserved.focused_checks]
// r[verify cli_maintainability.main_compat_release_followup.behavior_preserved.repository_checks]
use cairn_core::{
    archive_dir, blake3_hex, build_change_dependency_graph, build_repo_status, canonical_json_hash,
    changes_dir, merge_delta_spec, policy_schema_compatibility_diagnostic, preserves_content_hash,
    specs_dir, validate_policy_json, validate_spec_sources_with_policy, validation_issue_code,
    ArtifactSubstanceInventory, ChangeDependencyGraph, ChangeSource, ChangeStatus, DeltaMerge,
    GateArtifact, GateSnapshot, GateStage, LifecycleLayout, Policy, RepoStatus, SpecSource,
    SpecSubstanceInventory, SyncPlan,
};
use serde_json::{json, Value};

mod args;
mod capability;
mod layout;
mod mutation;
mod paths;
mod receipts;
mod scan;
mod usage;

pub(crate) fn validation_findings_json(issues: &[String]) -> Vec<Value> {
    issues
        .iter()
        .map(|message| {
            json!({
                "code": validation_issue_code(message),
                "message": message,
            })
        })
        .collect()
}

pub(crate) fn spec_substance_inventory_json(inventories: &[SpecSubstanceInventory]) -> Vec<Value> {
    inventories
        .iter()
        .map(|inventory| {
            json!({
                "path": inventory.path,
                "requirement_blocks": inventory.requirement_blocks,
                "substantive_requirement_blocks": inventory.substantive_requirement_blocks,
                "scenario_blocks": inventory.scenario_blocks,
            })
        })
        .collect()
}

pub(crate) fn substance_inventory_json(inventories: &[ArtifactSubstanceInventory]) -> Vec<Value> {
    inventories
        .iter()
        .map(|inventory| {
            json!({
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
            })
        })
        .collect()
}

pub(crate) use args::{has_flag, read_option, read_repeated_path_options, read_string_option};
pub(crate) use capability::{CapabilityRoot, CapabilityRootKind};
pub(crate) use layout::{
    default_policy_path, detect_layout, load_policy_file, reject_legacy_only_lifecycle_root,
};
pub(crate) use mutation::{
    archive_mutation_manifest, execute_archive_plan, execute_sync_snapshots,
    load_sync_merge_snapshots, sync_changed_paths, sync_merge_bindings, sync_merge_blockers,
    sync_mutation_manifest,
};
pub(crate) use paths::{
    collect_files, file_content_manifest, path_string, portable_json_hash, portable_text,
    safe_relative_path, tree_content_paths,
};
pub(crate) use receipts::{
    print_json, with_portable_content_hash_receipt, with_portable_receipt_hash, with_receipt_hash,
};
pub(crate) use scan::{
    collect_archived_change_ids, collect_spec_sources, collect_validation_spec_sources,
    find_change, find_spec_source, mutation_gate_blockers, read_change_dependency_graph,
    read_gate_snapshot, scan_repo,
};
pub(crate) use usage::{print_usage, usage_text};

// r[impl cli_maintainability.support_helper_submodule_split.focused_modules]
// r[impl cli_maintainability.support_helper_submodule_split.focused_modules.visible_boundaries]
// r[impl cli_maintainability.support_helper_submodule_split.focused_modules.narrow_bridges]
// r[verify cli_maintainability.support_helper_submodule_split.behavior_preserved]
// r[verify cli_maintainability.support_helper_submodule_split.behavior_preserved.focused_checks]
// r[verify cli_maintainability.support_helper_submodule_split.behavior_preserved.repository_checks]
