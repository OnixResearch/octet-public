use std::{error::Error, path::PathBuf};

use cairn_core::{
    build_archive_plan_for_effect_with_dependency_blockers,
    build_sync_plan_for_effect_with_dependency_blockers, canonical_json_hash,
    classify_gate_evidence, dependency_blockers_for_change, finalize_gate_report, GateIssue,
    GateStage, PlanAction, PlanActionKind,
};
use serde_json::{json, Value};

use crate::cross_repo_dependencies::{
    cross_repo_blockers, evaluate_change_cross_repo_dependencies, load_cross_repo_evidence,
};
use crate::native::change_status_json;
use crate::plan_receipts::{archive_plan_json, gate_input_json, gate_report_json, sync_plan_json};
use crate::probe::observe_gate_probes;
use crate::support::{
    archive_mutation_manifest, default_policy_path, detect_layout, execute_archive_plan,
    execute_sync_snapshots, file_content_manifest, find_change, has_flag, load_policy_file,
    load_sync_merge_snapshots, mutation_gate_blockers, path_string, portable_json_hash, print_json,
    read_change_dependency_graph, read_gate_snapshot, read_option, read_repeated_path_options,
    sync_changed_paths, sync_merge_bindings, sync_merge_blockers, sync_mutation_manifest,
    tree_content_paths, with_portable_receipt_hash,
};

pub(crate) fn run_gate(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(stage_text) = iter.next() else {
        return Err("gate requires a stage: proposal, design, or tasks".into());
    };
    let stage = parse_gate_stage(&stage_text)?;
    let Some(change) = iter.next() else {
        return Err("gate requires a change name".into());
    };
    let rest = iter.collect::<Vec<_>>();
    let run_probes_requested = has_flag(&rest, "--run-probes");
    let root = read_option(rest.clone(), "--root", ".");
    let policy_path = read_option(rest, "--policy", default_policy_path());
    let policy = load_policy_file(policy_path)?;
    let selected = detect_layout(&root);
    let change_status = find_change(root.clone(), selected.layout, &change)?;
    let snapshot = read_gate_snapshot(&root, selected.layout, change_status)?;
    let profile_context =
        crate::workflow::load_workflow_profile_context(&root, selected.layout, &change, &policy)?;
    let report = profile_context.gate_report(&snapshot, stage, &policy);
    let mut external_issues = Vec::new();
    if run_probes_requested {
        external_issues.push(GateIssue {
            code: "cairn.gate.inline_probe_disabled".to_string(),
            message: "inline probe execution is disabled by policy".to_string(),
            scope: policy.gate_finding_scope.clone(),
            promoted: policy.gate_finding_promoted.clone(),
        });
    }
    let probe_evidence = if matches!(stage, GateStage::Tasks) {
        let observation = observe_gate_probes(&root, selected.layout, &change, &policy);
        external_issues.extend(observation.issues);
        observation.binding
    } else {
        Value::Null
    };
    let report = finalize_gate_report(report, external_issues);
    let mut gate_input = gate_input_json(selected.layout, stage, &policy, &snapshot);
    gate_input["probe_evidence"] = probe_evidence;
    if !profile_context.selection.compatibility_fallback {
        gate_input["workflow_profile"] = profile_context.receipt_fields();
    }
    let input_hash = portable_json_hash(&root, &gate_input);
    let mut receipt = gate_report_json(selected.layout, &policy, &report, &input_hash);
    receipt["probe_evidence"] = gate_input["probe_evidence"].clone();
    attach_profile_fields(&mut receipt, &profile_context);
    let evidence = classify_gate_evidence(&snapshot);
    receipt["evidence_summary"] = serde_json::json!({
        "mode": "advisory",
        "docs_only": evidence.docs_only,
        "fixture": evidence.fixture,
        "probe": evidence.probe,
        "property": evidence.property,
        "model": evidence.model,
        "formal": evidence.formal,
    });
    print_json(with_portable_receipt_hash(&root, receipt))
}

pub(crate) fn parse_gate_stage(stage: &str) -> Result<GateStage, Box<dyn Error>> {
    match stage {
        "proposal" => Ok(GateStage::Proposal),
        "design" => Ok(GateStage::Design),
        "tasks" => Ok(GateStage::Tasks),
        unknown => Err(format!("unknown gate stage: {unknown}").into()),
    }
}

pub(crate) fn run_change_context(args: Vec<String>, change: &str) -> Result<(), Box<dyn Error>> {
    let root = read_option(args, "--root", ".");
    let selected = detect_layout(&root);
    let change_status = find_change(root, selected.layout, change)?;
    print_json(change_status_json(&change_status))
}

pub(crate) fn run_sync(args: Vec<String>, change: &str) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.clone(), "--root", ".");
    let is_execute = has_flag(&args, "--execute");
    let policy_path = read_option(args.clone(), "--policy", default_policy_path());
    let policy = load_policy_file(policy_path)?;
    let selected = detect_layout(&root);
    let change_status = find_change(root.clone(), selected.layout, change)?;
    let dependency_graph = read_change_dependency_graph(&root, selected.layout)?;
    let evidence_inputs = read_repeated_path_options(&args, "--cross-repo-evidence");
    let loaded_evidence = load_cross_repo_evidence(&root, &evidence_inputs);
    let policy_hash = canonical_json_hash(&policy.raw);
    let cross_repo_report = evaluate_change_cross_repo_dependencies(
        &change_status,
        &loaded_evidence.summaries,
        &policy_hash,
    );
    let mut dependency_blockers = dependency_blockers_for_change(&dependency_graph, change);
    dependency_blockers.extend(cross_repo_blockers(
        &change_status,
        &loaded_evidence.summaries,
        &policy_hash,
    ));
    dependency_blockers.extend(loaded_evidence.issues.iter().cloned());
    let profile_context =
        crate::workflow::load_workflow_profile_context(&root, selected.layout, change, &policy)?;
    let profile_blockers = if profile_context.selection.compatibility_fallback {
        Vec::new()
    } else {
        profile_context.completion_blockers()
    };
    dependency_blockers.sort();
    dependency_blockers.dedup();
    let mut plan = build_sync_plan_for_effect_with_dependency_blockers(
        &change_status,
        &dependency_blockers,
        &profile_context.schema.spec_effect,
    );
    let base_plan_unblocked = !plan.blocked;
    plan.reasons.extend(profile_blockers);
    plan.reasons.sort();
    plan.reasons.dedup();
    plan.blocked = !plan.reasons.is_empty();
    if plan.blocked {
        plan.actions.clear();
    }

    // Load and compute every destination before the first possible write.
    // Semantic blockers are all-or-nothing across a multi-spec change.
    let merge_snapshots = load_sync_merge_snapshots(&root, selected.layout, change, &plan)?;
    plan.reasons
        .extend(sync_merge_blockers(&root, &merge_snapshots));
    plan.reasons.sort();
    plan.reasons.dedup();
    plan.blocked = !plan.reasons.is_empty();
    if plan.blocked {
        plan.actions.clear();
    }

    if policy.sync_require_gate && base_plan_unblocked {
        plan.reasons.extend(mutation_gate_blockers(
            &root,
            selected.layout,
            &change_status,
            GateStage::Tasks,
            &policy,
        )?);
        plan.reasons.sort();
        plan.reasons.dedup();
        plan.blocked = !plan.reasons.is_empty();
        if plan.blocked {
            plan.actions.clear();
        }
    }

    let merge_bindings = sync_merge_bindings(&root, &merge_snapshots);
    let probe_binding = observe_gate_probes(&root, selected.layout, change, &policy).binding;
    let input_hash = portable_json_hash(
        &root,
        &json!({
            "change": change_status_json(&change_status),
            "merge_preflight": merge_bindings,
            "probe_evidence": probe_binding,
            "cross_repo_dependencies": cross_repo_report.to_json(),
            "cross_repo_evidence_issues": loaded_evidence.issues,
            "workflow_profile": profile_context.receipt_fields(),
            "policy": policy.name,
            "policy_hash": policy_hash,
        }),
    );
    let changed_paths = sync_changed_paths(&merge_snapshots);
    let before_manifest = file_content_manifest(&root, &changed_paths);
    let mut mutation_manifest = Value::Null;

    if is_execute {
        plan.dry_run = false;
        if !plan.blocked {
            let current_profile = crate::workflow::load_workflow_profile_context(
                &root,
                selected.layout,
                change,
                &policy,
            )?;
            if current_profile.profile_identity_hash != profile_context.profile_identity_hash {
                plan.blocked = true;
                plan.reasons
                    .push("workflow profile facts changed after preflight".to_string());
                plan.actions.clear();
            }
        }
        if !plan.blocked {
            let current_evidence = load_cross_repo_evidence(&root, &evidence_inputs);
            let current_report = evaluate_change_cross_repo_dependencies(
                &change_status,
                &current_evidence.summaries,
                &policy_hash,
            );
            if !current_evidence.issues.is_empty()
                || current_report.receipt_hash != cross_repo_report.receipt_hash
                || !current_report.valid
            {
                plan.blocked = true;
                plan.reasons.push(
                    "cross-repo dependency evidence changed or failed after preflight".to_string(),
                );
                plan.actions.clear();
            }
        }
        if !plan.blocked {
            let current_probe = observe_gate_probes(&root, selected.layout, change, &policy);
            if !current_probe.issues.is_empty() || current_probe.binding != probe_binding {
                plan.blocked = true;
                plan.reasons.push(
                    "gate tasks: probe evidence changed or failed after preflight".to_string(),
                );
                plan.actions.clear();
            }
        }
        if !plan.blocked {
            let changed_files = execute_sync_snapshots(&merge_snapshots)?;
            plan.mutated = changed_files > 0;
            if plan.mutated {
                mutation_manifest =
                    sync_mutation_manifest(&root, &merge_snapshots, &before_manifest);
            }
        }
    }
    let mut value = sync_plan_json(
        &root,
        selected.layout,
        &policy,
        &plan,
        &input_hash,
        &merge_bindings,
        mutation_manifest,
    );
    value["cross_repo_dependencies"] = cross_repo_report.to_json();
    value["cross_repo_evidence_issues"] = json!(loaded_evidence.issues);
    attach_profile_fields(&mut value, &profile_context);
    print_json(with_portable_receipt_hash(&root, value))
}

pub(crate) fn run_archive(args: Vec<String>, change: &str) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.lifecycle_views.verification.archive]
    let root = read_option(args.clone(), "--root", ".");
    let is_execute = has_flag(&args, "--execute");
    let policy_path = read_option(args.clone(), "--policy", default_policy_path());
    let policy = load_policy_file(policy_path)?;
    let selected = detect_layout(&root);
    let change_status = find_change(root.clone(), selected.layout, change)?;
    let evidence_inputs = read_repeated_path_options(&args, "--cross-repo-evidence");
    let loaded_evidence = load_cross_repo_evidence(&root, &evidence_inputs);
    let policy_hash = canonical_json_hash(&policy.raw);
    let cross_repo_report = evaluate_change_cross_repo_dependencies(
        &change_status,
        &loaded_evidence.summaries,
        &policy_hash,
    );
    let probe_binding = observe_gate_probes(&root, selected.layout, change, &policy).binding;
    let dependency_graph = read_change_dependency_graph(&root, selected.layout)?;
    let mut dependency_blockers = dependency_blockers_for_change(&dependency_graph, change);
    dependency_blockers.extend(cross_repo_blockers(
        &change_status,
        &loaded_evidence.summaries,
        &policy_hash,
    ));
    dependency_blockers.extend(loaded_evidence.issues.iter().cloned());
    let profile_context =
        crate::workflow::load_workflow_profile_context(&root, selected.layout, change, &policy)?;
    let profile_blockers = if profile_context.selection.compatibility_fallback {
        Vec::new()
    } else {
        profile_context.completion_blockers()
    };
    let input_hash = portable_json_hash(
        &root,
        &json!({
            "change": change_status_json(&change_status),
            "probe_evidence": probe_binding,
            "cross_repo_dependencies": cross_repo_report.to_json(),
            "cross_repo_evidence_issues": loaded_evidence.issues,
            "workflow_profile": profile_context.receipt_fields(),
            "policy_hash": policy_hash,
        }),
    );
    dependency_blockers.sort();
    dependency_blockers.dedup();
    let mut plan = build_archive_plan_for_effect_with_dependency_blockers(
        &change_status,
        &dependency_blockers,
        &profile_context.schema.spec_effect,
    );
    let base_plan_unblocked = !plan.blocked;
    plan.reasons.extend(profile_blockers);
    plan.reasons.sort();
    plan.reasons.dedup();
    plan.blocked = !plan.reasons.is_empty();
    if plan.blocked {
        plan.actions.clear();
    }
    let archive_source = PathBuf::from(&change_status.path);
    let before_paths =
        tree_content_paths(&root, &archive_source).unwrap_or_else(|_| vec![archive_source.clone()]);
    let before_manifest = file_content_manifest(&root, &before_paths);
    let mut mutation_manifest = Value::Null;
    if policy.archive_require_gate && base_plan_unblocked {
        plan.reasons.extend(mutation_gate_blockers(
            &root,
            selected.layout,
            &change_status,
            GateStage::Tasks,
            &policy,
        )?);
        plan.reasons.sort();
        plan.reasons.dedup();
        plan.blocked = !plan.reasons.is_empty();
        if plan.blocked {
            plan.actions.clear();
        }
    }
    if is_execute {
        plan.dry_run = false;
        if !plan.blocked {
            let current_profile = crate::workflow::load_workflow_profile_context(
                &root,
                selected.layout,
                change,
                &policy,
            )?;
            if current_profile.profile_identity_hash != profile_context.profile_identity_hash {
                plan.blocked = true;
                plan.reasons
                    .push("workflow profile facts changed after preflight".to_string());
                plan.actions.clear();
            }
        }
        if !plan.blocked {
            let current_evidence = load_cross_repo_evidence(&root, &evidence_inputs);
            let current_report = evaluate_change_cross_repo_dependencies(
                &change_status,
                &current_evidence.summaries,
                &policy_hash,
            );
            if !current_evidence.issues.is_empty()
                || current_report.receipt_hash != cross_repo_report.receipt_hash
                || !current_report.valid
            {
                plan.blocked = true;
                plan.reasons.push(
                    "cross-repo dependency evidence changed or failed after preflight".to_string(),
                );
                plan.actions.clear();
            }
        }
        if !plan.blocked {
            let current_probe = observe_gate_probes(&root, selected.layout, change, &policy);
            if !current_probe.issues.is_empty() || current_probe.binding != probe_binding {
                plan.blocked = true;
                plan.reasons.push(
                    "gate tasks: probe evidence changed or failed after preflight".to_string(),
                );
                plan.actions.clear();
            }
        }
        if !plan.blocked {
            let archive_path =
                execute_archive_plan(&root, selected.layout, change, &change_status.path)?;
            mutation_manifest =
                archive_mutation_manifest(&root, &archive_source, &archive_path, &before_manifest);
            plan.actions = vec![PlanAction {
                kind: PlanActionKind::ArchiveChange,
                path: path_string(archive_path),
                description: format!("move active change to archive: {change}"),
            }];
            plan.mutated = true;
        }
    }
    let mut value = archive_plan_json(
        &root,
        selected.layout,
        &policy,
        &plan,
        &input_hash,
        mutation_manifest,
    );
    value["cross_repo_dependencies"] = cross_repo_report.to_json();
    value["cross_repo_evidence_issues"] = json!(loaded_evidence.issues);
    attach_profile_fields(&mut value, &profile_context);
    print_json(with_portable_receipt_hash(&root, value))
}

fn attach_profile_fields(value: &mut Value, context: &crate::workflow::WorkflowProfileContext) {
    if context.selection.compatibility_fallback {
        if let Some(target) = value.as_object_mut() {
            target.remove("spec_effect");
        }
        return;
    }
    let Some(target) = value.as_object_mut() else {
        return;
    };
    let fields = context.receipt_fields();
    let Some(fields) = fields.as_object() else {
        return;
    };
    for (key, field) in fields {
        target.insert(key.clone(), field.clone());
    }
}
