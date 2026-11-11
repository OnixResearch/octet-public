use super::common::*;
use super::*;

pub(super) fn mcp_sync(input: &Value, execute: bool) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.supersede.mcp_server.plan_tools.lifecycle]
    // r[impl cairn.supersede.mcp_server.mutation_guard.enabled]
    // r[impl cairn.sync_archive.delta_merge.mutation_truth]
    let root = mcp_root(input);
    let change = mcp_required_string(input, "change")?;
    reject_selector_escape(change)?;
    let policy = load_policy_file(mcp_policy(input))?;
    let selected = detect_layout(&root);
    let change_status = find_change(root.clone(), selected.layout, change)?;
    let evidence_inputs = crate::cross_repo_dependencies::evidence_paths_from_json(input)?;
    let loaded_evidence =
        crate::cross_repo_dependencies::load_cross_repo_evidence(&root, &evidence_inputs);
    let policy_hash = canonical_json_hash(&policy.raw);
    let cross_repo_report = crate::cross_repo_dependencies::evaluate_change_cross_repo_dependencies(
        &change_status,
        &loaded_evidence.summaries,
        &policy_hash,
    );
    let dependency_graph = read_change_dependency_graph(&root, selected.layout)?;
    let mut dependency_blockers = dependency_blockers_for_change(&dependency_graph, change);
    dependency_blockers.extend(crate::cross_repo_dependencies::cross_repo_blockers(
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
    let probe_binding =
        crate::probe::observe_gate_probes(&root, selected.layout, change, &policy).binding;
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
    if execute {
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
            let current_evidence =
                crate::cross_repo_dependencies::load_cross_repo_evidence(&root, &evidence_inputs);
            let current_report =
                crate::cross_repo_dependencies::evaluate_change_cross_repo_dependencies(
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
            let current_probe =
                crate::probe::observe_gate_probes(&root, selected.layout, change, &policy);
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
    value["mcp"] = json!({"read_only": !execute, "destructive": execute});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_archive(input: &Value, execute: bool) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let change = mcp_required_string(input, "change")?;
    reject_selector_escape(change)?;
    let policy = load_policy_file(mcp_policy(input))?;
    let selected = detect_layout(&root);
    let change_status = find_change(root.clone(), selected.layout, change)?;
    let evidence_inputs = crate::cross_repo_dependencies::evidence_paths_from_json(input)?;
    let loaded_evidence =
        crate::cross_repo_dependencies::load_cross_repo_evidence(&root, &evidence_inputs);
    let policy_hash = canonical_json_hash(&policy.raw);
    let cross_repo_report = crate::cross_repo_dependencies::evaluate_change_cross_repo_dependencies(
        &change_status,
        &loaded_evidence.summaries,
        &policy_hash,
    );
    let probe_binding =
        crate::probe::observe_gate_probes(&root, selected.layout, change, &policy).binding;
    let dependency_graph = read_change_dependency_graph(&root, selected.layout)?;
    let mut dependency_blockers = dependency_blockers_for_change(&dependency_graph, change);
    dependency_blockers.extend(crate::cross_repo_dependencies::cross_repo_blockers(
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
    if execute {
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
            let current_evidence =
                crate::cross_repo_dependencies::load_cross_repo_evidence(&root, &evidence_inputs);
            let current_report =
                crate::cross_repo_dependencies::evaluate_change_cross_repo_dependencies(
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
            let current_probe =
                crate::probe::observe_gate_probes(&root, selected.layout, change, &policy);
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
        Value::Null,
    );
    value["cross_repo_dependencies"] = cross_repo_report.to_json();
    value["cross_repo_evidence_issues"] = json!(loaded_evidence.issues);
    attach_profile_fields(&mut value, &profile_context);
    value["mcp"] = json!({"read_only": !execute, "destructive": execute});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_migrate(input: &Value, execute: bool) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.supersede.mcp_server.plan_tools.migration]
    let root = mcp_root(input);
    let source = mcp_required_string(input, "source")?;
    let policy = load_policy_file(mcp_policy(input))?;
    let source_layout = migration_source_layout(source)?;
    let inputs = collect_layout_migration_inputs(&root, source_layout)?;
    let mut plan = match source_layout {
        LifecycleLayout::LegacyCairn => build_cairn_migration_plan(inputs, execute),
        LifecycleLayout::LegacyOpenSpec => build_openspec_migration_plan(inputs, execute),
        LifecycleLayout::LegacyIronspec => build_ironspec_migration_plan(inputs, execute),
        LifecycleLayout::Native => return Err("cannot migrate from canonical .cairn layout".into()),
    };
    let actual_plan_hash = migration_plan_hash(&root, &policy, &plan);
    if execute {
        let expected_plan_hash = mcp_required_string(input, "plan_hash")?;
        if expected_plan_hash != actual_plan_hash {
            return Err(format!(
                "migration plan hash mismatch: expected {expected_plan_hash}, actual {actual_plan_hash}"
            )
            .into());
        }
    }
    if execute && !plan.blocked {
        execute_migration_plan(&root, &plan)?;
        plan.mutated = !plan.actions.is_empty();
    }
    let mut value = migration_plan_json(&root, &policy, &plan, Value::Null);
    value["mcp"] = json!({"read_only": !execute, "destructive": execute});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_init(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let artifacts = init_artifacts();
    let mut created = Vec::with_capacity(artifacts.len());
    let mut existing = Vec::with_capacity(artifacts.len());
    for artifact in artifacts {
        let path = root.join(&artifact.path);
        if path.exists() {
            existing.push(artifact.path);
        } else {
            match artifact.kind {
                LifecycleArtifactKind::Directory => fs::create_dir_all(&path)?,
                LifecycleArtifactKind::File => write_new_file(&path, "")?,
            }
            created.push(artifact.path);
        }
    }
    Ok(with_receipt_hash(
        json!({"command": "init", "root": path_string(root), "mutated": !created.is_empty(), "layout": layout_name(LifecycleLayout::Native), "created": created, "existing": existing, "mcp": {"read_only": false, "destructive": true}}),
    ))
}

pub(super) fn mcp_change_create(input: &Value) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.lifecycle_workflow_profiles.selection.create]
    let root = mcp_root(input);
    let change = mcp_required_string(input, "change")?;
    reject_selector_escape(change)?;
    let profile = input.get("profile").and_then(Value::as_str);
    let policy = load_policy_file(mcp_policy(input))?;
    let mut value = crate::native::create_change_with_profile(&root, change, profile, &policy)?;
    value["mcp"] = json!({"read_only": false, "destructive": true});
    Ok(with_portable_receipt_hash(&root, value))
}
