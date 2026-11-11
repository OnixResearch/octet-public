use std::collections::BTreeSet;

use super::common::*;
use super::*;

pub(super) fn mcp_status(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let selected = detect_layout(&root);
    let mut value = repo_status_json(&scan_repo(root.clone(), selected.layout)?, selected.layout);
    value["mcp"] = json!({"read_only": true, "destructive": false});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_list_content(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let archive_root = root.join(archive_dir(selected.layout));
    let archived_changes = if archive_root.exists() {
        let mut changes = fs::read_dir(archive_root)?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect::<Vec<_>>();
        changes.sort();
        changes
    } else {
        Vec::new()
    };
    Ok(with_receipt_hash(
        json!({"root": path_string(root), "layout": layout_name(selected.layout), "accepted_specs": status.specs, "active_changes": status.changes.iter().map(change_status_json).collect::<Vec<_>>(), "archived_changes": archived_changes, "mcp": {"read_only": true, "destructive": false}}),
    ))
}

// r[impl cairn.agent_guidance_context.mcp]
pub(super) fn mcp_context(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let policy = if input.get("policy").is_some() {
        load_policy_file(mcp_policy(input))?
    } else {
        cairn_core::parse_policy_json(include_str!(
            "../../../../cairn-policy/generated/cairn-policy.json"
        ))?
    };
    let request = cairn_core::AgentGuidanceRequest {
        profile: input
            .get("profile")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        artifact: input
            .get("artifact")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        operation: Some(
            input
                .get("operation")
                .and_then(Value::as_str)
                .unwrap_or(cairn_core::AGENT_GUIDANCE_OPERATION_CONTEXT)
                .to_string(),
        ),
    };
    let guidance =
        crate::agent_guidance_shell::load_agent_guidance(&root, &root, request, &[], &policy)?;
    let valid = guidance.as_ref().is_none_or(|result| result.valid);
    let mut receipt = json!({
        "command": "context",
        "root": path_string(&root),
        "valid": valid,
        "guidance": Value::Null,
        "policy": policy.name,
        "policy_hash": canonical_json_hash(&policy.raw),
        "read_only": true,
        "mutated": false,
        "mcp": {"read_only": true, "destructive": false},
    });
    if let Some(guidance) = guidance {
        receipt["guidance"] = crate::agent_guidance_shell::agent_guidance_json(&guidance);
    }
    Ok(with_portable_receipt_hash(&root, receipt))
}

pub(super) fn mcp_read_spec(input: &Value) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.supersede.mcp_server.read_tools.artifacts]
    let root = mcp_root(input);
    let spec = mcp_required_string(input, "spec")?;
    reject_selector_escape(spec)?;
    let selected = detect_layout(&root);
    let detail = build_spec_detail(find_spec_source(&root, selected.layout, spec)?);
    let mut value = spec_detail_json(&path_string(&root), selected.layout, &detail);
    let markdown = value["spec"]["markdown"].as_str().unwrap_or_default();
    if markdown.len() > MCP_MAX_CONTENT_BYTES {
        return Err("spec content exceeds MCP bounded read limit".into());
    }
    value["mcp"] = json!({"read_only": true, "destructive": false, "max_content_bytes": MCP_MAX_CONTENT_BYTES});
    Ok(with_portable_receipt_hash(&root, value))
}

// r[impl cairn.artifact_workflow_contract.mcp]
pub(super) fn mcp_workflow_status(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let change = mcp_required_string(input, "change")?;
    reject_selector_escape(change)?;
    let schema = input.get("schema").and_then(Value::as_str);
    let value = crate::workflow::workflow_status_value(&root, change, schema, &mcp_policy(input))?;
    Ok(with_portable_receipt_hash(
        &root,
        crate::workflow::add_mcp_transport(value),
    ))
}

pub(super) fn mcp_workflow_instructions(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let change = mcp_required_string(input, "change")?;
    let artifact = mcp_required_string(input, "artifact")?;
    reject_selector_escape(change)?;
    reject_selector_escape(artifact)?;
    let schema = input.get("schema").and_then(Value::as_str);
    let value = crate::workflow::workflow_instructions_value(
        &root,
        change,
        artifact,
        schema,
        &mcp_policy(input),
    )?;
    Ok(with_portable_receipt_hash(
        &root,
        crate::workflow::add_mcp_transport(value),
    ))
}

pub(super) fn mcp_change_show(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let change = mcp_required_string(input, "change")?;
    reject_selector_escape(change)?;
    let selected = detect_layout(&root);
    let detail = build_change_detail(find_change(root.clone(), selected.layout, change)?);
    let mut value = change_detail_json(&path_string(&root), selected.layout, &detail);
    value["mcp"] = json!({"read_only": true, "destructive": false});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_doctor(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let policy_path = mcp_policy(input);
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let policy_result = load_policy_file(policy_path.clone());
    let (policy_name, policy_error, validation_issue_count) = match policy_result {
        Ok(policy) => {
            let change_issue_count = validate_changes_with_archives(
                &status.changes,
                &collect_archived_change_ids(&root, selected.layout)?,
                &policy,
            )
            .len();
            let spec_issue_count = validate_spec_sources_with_policy(
                &collect_validation_spec_sources(&root, selected.layout)?,
                &policy.artifact_substance_policy,
            )
            .len();
            let mut substance_issue_count = 0;
            for change in &status.changes {
                let snapshot = read_gate_snapshot(&root, selected.layout, change.clone())?;
                substance_issue_count += lifecycle_substance_issues(&snapshot, &policy).len();
            }
            (
                Some(policy.name.clone()),
                None,
                change_issue_count + spec_issue_count + substance_issue_count,
            )
        }
        Err(error) => (None, Some(error.to_string()), 0),
    };
    let report = build_doctor_report(DoctorSnapshot {
        root: path_string(&root),
        selected_layout: selected.layout,
        native_layout_present: root.join(layout_root(LifecycleLayout::Native)).exists(),
        legacy_layout_present: root
            .join(layout_root(LifecycleLayout::LegacyOpenSpec))
            .exists(),
        specs_dir_present: root.join(specs_dir(selected.layout)).exists(),
        changes_dir_present: root.join(changes_dir(selected.layout)).exists(),
        archive_dir_present: root.join(archive_dir(selected.layout)).exists(),
        policy_path: path_string(policy_path),
        policy_name,
        policy_error,
        spec_count: status.specs.len(),
        change_count: status.changes.len(),
        validation_issue_count,
    });
    let mut value = doctor_report_json(&report);
    value["mcp"] = json!({"read_only": true, "destructive": false});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_validate(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let policy = load_policy_file(mcp_policy(input))?;
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let archived_changes = collect_archived_change_ids(&root, selected.layout)?;
    let mut change_issues =
        validate_changes_with_archives(&status.changes, &archived_changes, &policy);
    let evidence_paths = crate::cross_repo_dependencies::evidence_paths_from_json(input)?;
    let loaded_evidence =
        crate::cross_repo_dependencies::load_cross_repo_evidence(&root, &evidence_paths);
    let policy_hash = canonical_json_hash(&policy.raw);
    let cross_repo_reports = crate::cross_repo_dependencies::cross_repo_reports(
        &status.changes,
        &loaded_evidence.summaries,
        &policy_hash,
    );
    for (change, report) in &cross_repo_reports {
        change_issues.extend(
            cairn_core::dependency_blocker_messages(report)
                .into_iter()
                .map(|blocker| format!("{change}: {blocker}")),
        );
    }
    change_issues.extend(loaded_evidence.issues.iter().cloned());
    change_issues.sort();
    change_issues.dedup();
    let spec_sources = collect_validation_spec_sources(&root, selected.layout)?;
    let mut spec_issues = Vec::new();
    let mut spec_substance = Vec::new();
    for source in &spec_sources {
        let (inventory, source_issues) =
            validate_spec_source_with_policy(source, &policy.artifact_substance_policy);
        spec_substance.push(inventory);
        spec_issues.extend(source_issues);
    }
    spec_issues.sort();
    spec_issues.dedup();
    spec_substance.sort_by(|left, right| left.path.cmp(&right.path));
    let mut substance_issues = Vec::new();
    let mut substance = Vec::new();
    let mut profile_issues = Vec::new();
    let mut workflow_profiles = Vec::new();
    for change in &status.changes {
        let snapshot = read_gate_snapshot(&root, selected.layout, change.clone())?;
        match crate::workflow::load_workflow_profile_context(
            &root,
            selected.layout,
            &change.name,
            &policy,
        ) {
            Ok(profile_context) => {
                let report = profile_context.gate_report(&snapshot, GateStage::Tasks, &policy);
                substance_issues.extend(
                    report
                        .issues
                        .iter()
                        .map(|issue| format!("{}: {}", issue.code, issue.message)),
                );
                substance.extend(report.substance);
                if !profile_context.selection.compatibility_fallback {
                    profile_issues.extend(profile_context.completion_blockers());
                    workflow_profiles.push(profile_context.receipt_fields());
                }
            }
            Err(error) => profile_issues.push(format!(
                "{}: cairn.workflow.profile.selection: {error}",
                change.name
            )),
        }
    }
    profile_issues.sort();
    profile_issues.dedup();
    substance_issues.sort();
    substance_issues.dedup();
    substance.sort_by(|left, right| left.path.cmp(&right.path).then(left.kind.cmp(&right.kind)));
    let issue_capacity = change_issues
        .len()
        .saturating_add(spec_issues.len())
        .saturating_add(substance_issues.len())
        .saturating_add(profile_issues.len());
    let mut issues = Vec::with_capacity(issue_capacity);
    issues.extend(change_issues.iter().cloned());
    issues.extend(spec_issues.iter().cloned());
    issues.extend(substance_issues.iter().cloned());
    issues.extend(profile_issues.iter().cloned());
    issues.sort();
    issues.dedup();
    let mut receipt = json!({"valid": issues.is_empty(), "findings": validation_findings_json(&issues), "issues": issues, "change_issues": change_issues, "cross_repo_dependencies": crate::cross_repo_dependencies::reports_json(&cross_repo_reports), "cross_repo_evidence_issues": loaded_evidence.issues, "spec_findings": validation_findings_json(&spec_issues), "spec_issues": spec_issues, "spec_substance": spec_substance_inventory_json(&spec_substance), "substance_findings": validation_findings_json(&substance_issues), "substance_issues": substance_issues, "substance": substance_inventory_json(&substance), "changes": status.changes.len(), "specs_validated": spec_sources.len(), "policy": policy.name, "layout": layout_name(selected.layout), "mcp": {"read_only": true, "destructive": false}});
    if !workflow_profiles.is_empty() || !profile_issues.is_empty() {
        let object = receipt
            .as_object_mut()
            .expect("MCP validation receipt object");
        object.insert("profile_issues".to_string(), json!(profile_issues));
        object.insert("workflow_profiles".to_string(), json!(workflow_profiles));
    }
    Ok(with_receipt_hash(receipt))
}

pub(super) fn mcp_gate(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let change = mcp_required_string(input, "change")?;
    reject_selector_escape(change)?;
    let stage = parse_gate_stage(mcp_required_string(input, "stage")?)?;
    let policy = load_policy_file(mcp_policy(input))?;
    let selected = detect_layout(&root);
    let snapshot = read_gate_snapshot(
        &root,
        selected.layout,
        find_change(root.clone(), selected.layout, change)?,
    )?;
    let profile_context =
        crate::workflow::load_workflow_profile_context(&root, selected.layout, change, &policy)?;
    let report = profile_context.gate_report(&snapshot, stage, &policy);
    let probe_evidence = if matches!(stage, GateStage::Tasks) {
        crate::probe::observe_gate_probes(&root, selected.layout, change, &policy)
    } else {
        crate::probe::ProbeGateEvidence {
            issues: Vec::new(),
            binding: Value::Null,
        }
    };
    let report = finalize_gate_report(report, probe_evidence.issues);
    let mut gate_input = gate_input_json(selected.layout, stage, &policy, &snapshot);
    gate_input["probe_evidence"] = probe_evidence.binding;
    if !profile_context.selection.compatibility_fallback {
        gate_input["workflow_profile"] = profile_context.receipt_fields();
    }
    let input_hash = portable_json_hash(&root, &gate_input);
    let mut value = gate_report_json(selected.layout, &policy, &report, &input_hash);
    value["probe_evidence"] = gate_input["probe_evidence"].clone();
    attach_profile_fields(&mut value, &profile_context);
    value["mcp"] = json!({"read_only": true, "destructive": false});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_gate_context(input: &Value) -> Result<Value, Box<dyn Error>> {
    let root = mcp_root(input);
    let change = mcp_required_string(input, "change")?;
    reject_selector_escape(change)?;
    let selected = detect_layout(&root);
    let mut value = change_status_json(&find_change(root.clone(), selected.layout, change)?);
    value["mcp"] = json!({"read_only": true, "destructive": false});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_change_graph(input: &Value) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.change_dependency_graph.drain_order]
    let root = mcp_root(input);
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let archived_changes = collect_archived_change_ids(&root, selected.layout)?;
    let graph = build_change_dependency_graph(&status.changes, &archived_changes);
    let mut value = change_dependency_graph_json(
        &path_string(&root),
        selected.layout,
        &archived_changes,
        &graph,
    );
    value["mcp"] = json!({"read_only": true, "destructive": false});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_drain_plan(input: &Value) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.mcp_drain_all.plan]
    // r[impl cairn.mcp_drain_all.plan.read_only]
    // r[impl cairn.mcp_drain_all.plan.readiness]
    // r[impl cairn.mcp_drain_all.plan.next_actions]
    // r[impl cairn.mcp_drain_all.plan.mutation_boundary]
    let root = mcp_root(input);
    let policy = load_policy_file(mcp_policy(input))?;
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let archived_changes = collect_archived_change_ids(&root, selected.layout)?;
    let dependency_graph = build_change_dependency_graph(&status.changes, &archived_changes);
    let evidence_paths = crate::cross_repo_dependencies::evidence_paths_from_json(input)?;
    let loaded_evidence =
        crate::cross_repo_dependencies::load_cross_repo_evidence(&root, &evidence_paths);
    let policy_hash = canonical_json_hash(&policy.raw);
    let ordered_change_names = drain_order_with_remaining(&status.changes, &dependency_graph);
    let mut changes = Vec::with_capacity(status.changes.len());
    let mut next_actions = Vec::new();
    let mut ready_change_count = 0usize;

    for change_name in ordered_change_names {
        let Some(change) = status
            .changes
            .iter()
            .find(|candidate| candidate.name == change_name)
        else {
            continue;
        };
        let snapshot = read_gate_snapshot(&root, selected.layout, change.clone())?;
        let profile_context = crate::workflow::load_workflow_profile_context(
            &root,
            selected.layout,
            &change.name,
            &policy,
        )?;
        let proposal_gate =
            drain_gate_summary(&snapshot, GateStage::Proposal, &policy, &profile_context);
        let design_gate =
            drain_gate_summary(&snapshot, GateStage::Design, &policy, &profile_context);
        let tasks_gate = drain_gate_summary(&snapshot, GateStage::Tasks, &policy, &profile_context);
        let mut blockers = Vec::new();
        collect_gate_blocker(&mut blockers, "proposal_gate", &proposal_gate);
        collect_gate_blocker(&mut blockers, "design_gate", &design_gate);
        collect_gate_blocker(&mut blockers, "tasks_gate", &tasks_gate);
        if !tasks_are_complete(change.task_counts) {
            blockers.push(json!({"class": "unfinished_tasks", "message": "all task checkboxes must be completed before archive"}));
        }

        let cross_repo_report =
            crate::cross_repo_dependencies::evaluate_change_cross_repo_dependencies(
                change,
                &loaded_evidence.summaries,
                &policy_hash,
            );
        let mut dependency_blockers =
            dependency_blockers_for_change(&dependency_graph, &change.name);
        append_dependency_blockers(&mut blockers, &dependency_graph, &change.name);
        let cross_repo_blockers = cairn_core::dependency_blocker_messages(&cross_repo_report);
        for message in &cross_repo_blockers {
            blockers.push(json!({
                "class": "cross_repo_dependency",
                "message": message,
            }));
        }
        dependency_blockers.extend(cross_repo_blockers);
        dependency_blockers.extend(loaded_evidence.issues.iter().cloned());
        if !profile_context.selection.compatibility_fallback {
            dependency_blockers.extend(profile_context.completion_blockers());
        }
        dependency_blockers.sort();
        dependency_blockers.dedup();

        let mut sync_plan = build_sync_plan_for_effect_with_dependency_blockers(
            change,
            &dependency_blockers,
            &profile_context.schema.spec_effect,
        );
        if policy.sync_require_gate && !sync_plan.blocked {
            sync_plan.reasons.extend(mutation_gate_blockers(
                &root,
                selected.layout,
                change,
                GateStage::Tasks,
                &policy,
            )?);
            sync_plan.reasons.sort();
            sync_plan.reasons.dedup();
            sync_plan.blocked = !sync_plan.reasons.is_empty();
        }
        append_plan_blockers(&mut blockers, "sync_blocked", &sync_plan.reasons);

        let mut archive_plan = build_archive_plan_for_effect_with_dependency_blockers(
            change,
            &dependency_blockers,
            &profile_context.schema.spec_effect,
        );
        if policy.archive_require_gate && !archive_plan.blocked {
            archive_plan.reasons.extend(mutation_gate_blockers(
                &root,
                selected.layout,
                change,
                GateStage::Tasks,
                &policy,
            )?);
            archive_plan.reasons.sort();
            archive_plan.reasons.dedup();
            archive_plan.blocked = !archive_plan.reasons.is_empty();
        }
        append_plan_blockers(&mut blockers, "archive_blocked", &archive_plan.reasons);

        let is_ready = blockers.is_empty();
        if is_ready {
            ready_change_count += 1;
        }
        let change_next_actions = drain_next_actions(&change.name, is_ready, &blockers);
        next_actions.extend(change_next_actions.iter().cloned());
        let dependency_node = dependency_graph.node(&change.name);
        let dependency_issues = dependency_issues_for_change(&dependency_graph, &change.name);

        changes.push(json!({
            "name": change.name,
            "path": change.path,
            "dependencies": dependency_node.map(|node| node.dependencies.clone()).unwrap_or_default(),
            "blocked_by": dependency_node.map(|node| node.blocked_by.clone()).unwrap_or_default(),
            "blocks": dependency_node.map(|node| node.blocks.clone()).unwrap_or_default(),
            "dependency_ready": dependency_node.is_some_and(|node| node.dependency_ready),
            "dependency_issues": dependency_issues,
            "cross_repo_dependencies": cross_repo_report.to_json(),
            "cross_repo_evidence_issues": loaded_evidence.issues,
            "workflow_profile": profile_context.receipt_fields(),
            "task_counts": change.task_counts.map(|counts| json!({
                "total": counts.total,
                "done": counts.done,
                "in_progress": counts.in_progress,
                "todo": counts.todo,
                "unmarked": counts.unmarked,
            })),
            "gates": {
                "proposal": proposal_gate,
                "design": design_gate,
                "tasks": tasks_gate,
            },
            "sync_plan": {"blocked": sync_plan.blocked, "reasons": sync_plan.reasons},
            "archive_plan": {"blocked": archive_plan.blocked, "reasons": archive_plan.reasons},
            "drain_ready": is_ready,
            "blockers": blockers,
            "next_actions": change_next_actions,
        }));
    }

    Ok(with_portable_receipt_hash(
        &root,
        json!({
            "command": "mcp cairn_drain_plan",
            "root": path_string(&root),
            "layout": layout_name(selected.layout),
            "policy": policy.name,
            "active_change_count": changes.len(),
            "ready_change_count": ready_change_count,
            "drain_complete": changes.is_empty(),
            "archived_changes": archived_changes,
            "dependency_graph": {
                "valid": dependency_graph.valid,
                "nodes": dependency_graph.nodes.iter().map(change_dependency_node_json).collect::<Vec<_>>(),
                "edges": dependency_graph.edges.iter().map(change_dependency_edge_json).collect::<Vec<_>>(),
                "issues": dependency_graph.issues.iter().map(change_dependency_issue_json).collect::<Vec<_>>(),
                "drain_order": dependency_graph.drain_order,
            },
            "drain_order": dependency_graph.drain_order,
            "changes": changes,
            "next_actions": next_actions,
            "mcp": {"read_only": true, "destructive": false},
        }),
    ))
}

fn drain_order_with_remaining(
    changes: &[ChangeStatus],
    dependency_graph: &ChangeDependencyGraph,
) -> Vec<String> {
    let mut ordered = dependency_graph.drain_order.clone();
    let mut seen = ordered.iter().cloned().collect::<BTreeSet<_>>();
    for change in changes {
        if seen.insert(change.name.clone()) {
            ordered.push(change.name.clone());
        }
    }
    ordered
}

fn dependency_issues_for_change(
    dependency_graph: &ChangeDependencyGraph,
    change_name: &str,
) -> Vec<Value> {
    dependency_graph
        .issues
        .iter()
        .filter(|issue| issue.change == change_name)
        .map(change_dependency_issue_json)
        .collect()
}

fn append_dependency_blockers(
    blockers: &mut Vec<Value>,
    dependency_graph: &ChangeDependencyGraph,
    change_name: &str,
) {
    if let Some(node) = dependency_graph.node(change_name) {
        blockers.extend(node.blocked_by.iter().map(|dependency| {
            json!({
                "class": "dependency_blocked",
                "dependency": dependency,
                "message": format!("dependency {dependency} must be archived before {change_name}"),
            })
        }));
    }
    blockers.extend(
        dependency_graph
            .issues
            .iter()
            .filter(|issue| issue.change == change_name)
            .map(|issue| {
                json!({
                    "class": "dependency_issue",
                    "code": issue.code,
                    "dependency": issue.dependency,
                    "message": issue.message,
                })
            }),
    );
}

fn drain_gate_summary(
    snapshot: &cairn_core::GateSnapshot,
    stage: GateStage,
    policy: &cairn_core::Policy,
    profile_context: &crate::workflow::WorkflowProfileContext,
) -> Value {
    let report = profile_context.gate_report(snapshot, stage, policy);
    json!({
        "stage": report.stage,
        "valid": report.issues.is_empty(),
        "verdict": report.verdict,
        "issue_count": report.issues.len(),
        "issues": report.issues.iter().map(|issue| json!({
            "message": issue.message,
            "scope": issue.scope,
            "promoted": issue.promoted,
        })).collect::<Vec<_>>(),
    })
}

fn collect_gate_blocker(blockers: &mut Vec<Value>, class: &str, gate: &Value) {
    if gate["valid"].as_bool() == Some(true) {
        return;
    }
    blockers.push(json!({
        "class": class,
        "message": format!("{} gate did not pass", gate["stage"].as_str().unwrap_or("unknown")),
        "issue_count": gate["issue_count"],
    }));
}

fn append_plan_blockers(blockers: &mut Vec<Value>, class: &str, reasons: &[String]) {
    blockers.extend(
        reasons
            .iter()
            .map(|reason| json!({"class": class, "message": reason})),
    );
}

fn tasks_are_complete(counts: Option<cairn_core::TaskCounts>) -> bool {
    counts
        .map(|counts| {
            counts.total > 0 && counts.todo == 0 && counts.in_progress == 0 && counts.unmarked == 0
        })
        .unwrap_or(false)
}

fn drain_next_actions(change: &str, is_ready: bool, blockers: &[Value]) -> Vec<Value> {
    if is_ready {
        return vec![
            json!({"change": change, "tool": "cairn_sync_execute", "requires_allow_mutations": true}),
            json!({"change": change, "tool": "cairn_archive_execute", "requires_allow_mutations": true}),
        ];
    }
    let mut actions = Vec::new();
    if blockers
        .iter()
        .any(|blocker| blocker["class"] == "proposal_gate")
    {
        actions.push(json!({"change": change, "tool": "cairn_gate", "arguments": {"change": change, "stage": "proposal"}}));
    }
    if blockers
        .iter()
        .any(|blocker| blocker["class"] == "design_gate")
    {
        actions.push(json!({"change": change, "tool": "cairn_gate", "arguments": {"change": change, "stage": "design"}}));
    }
    if blockers
        .iter()
        .any(|blocker| blocker["class"] == "tasks_gate")
    {
        actions.push(json!({"change": change, "tool": "cairn_gate", "arguments": {"change": change, "stage": "tasks"}}));
    }
    if blockers
        .iter()
        .any(|blocker| blocker["class"] == "unfinished_tasks")
    {
        actions.push(json!({"change": change, "action": "complete implementation tasks and mark tasks.md checkboxes"}));
    }
    if blockers
        .iter()
        .any(|blocker| blocker["class"] == "dependency_blocked")
    {
        actions.push(json!({"change": change, "action": "archive dependency changes first"}));
    }
    if blockers
        .iter()
        .any(|blocker| blocker["class"] == "dependency_issue")
    {
        actions.push(json!({"change": change, "tool": "cairn_change_graph"}));
    }
    if blockers
        .iter()
        .any(|blocker| blocker["class"] == "sync_blocked")
    {
        actions.push(json!({"change": change, "tool": "cairn_sync_plan"}));
    }
    if blockers
        .iter()
        .any(|blocker| blocker["class"] == "archive_blocked")
    {
        actions.push(json!({"change": change, "tool": "cairn_archive_plan"}));
    }
    actions
}

pub(super) fn mcp_agent_receipt_validate(input: &Value) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.supersede.agent_receipt_validation_surface.mcp]
    // r[impl cairn.supersede.agent_receipt_validation_surface.mcp.read_only]
    let root = mcp_root(input);
    let (path, receipt) = if input.get("receipt").is_some() {
        let receipt = input["receipt"].clone();
        ensure_inline_receipt_is_bounded(&receipt)?;
        (None, receipt)
    } else {
        let selector = mcp_required_string(input, "path")?;
        reject_selector_escape(selector)?;
        let path = root.join(selector);
        let receipt = read_agent_receipt_file(&path)?;
        (Some(path), receipt)
    };
    let policy = load_policy_file(mcp_policy(input))?;
    let mut value = agent_receipt_validation_receipt(
        "mcp cairn_agent_receipt_validate",
        path,
        &receipt,
        true,
        &policy,
    );
    value["mcp"] = json!({"read_only": true, "destructive": false, "max_content_bytes": AGENT_RECEIPT_MAX_BYTES});
    Ok(with_portable_receipt_hash(&root, value))
}

pub(super) fn mcp_agent_completion_validate(input: &Value) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.supersede.agent_run_receipts.schema_migration.parity]
    // r[impl cairn.supersede.agent_run_receipts.execution_boundary.no_execution]
    let root = mcp_root(input);
    let contract = input
        .get("contract")
        .cloned()
        .ok_or("contract is required")?;
    let results = input
        .get("results")
        .cloned()
        .ok_or("results are required")?;
    ensure_inline_receipt_is_bounded(&contract)?;
    ensure_inline_receipt_is_bounded(&results)?;
    let subject_hash = mcp_required_string(input, "subject_hash")?;
    let policy = load_policy_file(mcp_policy(input))?;
    let mut value = agent_completion_validation_receipt(AgentCompletionValidationInput {
        command: "mcp cairn_agent_completion_validate",
        contract_path: None,
        results_path: None,
        contract: &contract,
        results: &results,
        current_subject_hash: subject_hash,
        policy: &policy,
        input_kind: "inline",
    });
    value["mcp"] = json!({
        "read_only": true,
        "destructive": false,
        "checks_executed": false,
        "max_content_bytes": AGENT_RECEIPT_MAX_BYTES,
    });
    Ok(with_portable_receipt_hash(&root, value))
}
