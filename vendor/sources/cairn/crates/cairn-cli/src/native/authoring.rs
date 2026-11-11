use std::{error::Error, fs, path::Path};

use cairn_core::{
    change_metadata_with_dependencies, changes_dir, design_template, init_artifacts, layout_name,
    normalized_change_dependencies, plan_change_dependency_edit, proposal_template,
    render_change_metadata_json, tasks_template, validate_change_name,
    workflow_change_profile_metadata_from_value, workflow_markdown_is_structurally_present,
    ChangeDependencyEditAction, LifecycleArtifactKind, LifecycleLayout, Policy,
    WorkflowArtifactPolicy, WorkflowProfileRef, WorkflowSchemaPolicy,
    WORKFLOW_ADMISSION_ACCEPTANCE, WORKFLOW_ADMISSION_DESIGN, WORKFLOW_ADMISSION_PROPOSAL,
    WORKFLOW_ADMISSION_RISK, WORKFLOW_ADMISSION_SEMANTIC_REVIEW, WORKFLOW_ADMISSION_TASKS,
    WORKFLOW_ADMISSION_VERIFICATION, WORKFLOW_PATH_KIND_FILE, WORKFLOW_PATH_KIND_SPEC_DIRECTORY,
    WORKFLOW_PROFILE_SOURCE_CREATE, WORKFLOW_SPEC_EFFECT_NONE, WORKFLOW_TASK_LINK_ACCEPTANCE,
};
use serde_json::{json, Value};

use crate::support::{
    collect_archived_change_ids, default_policy_path, detect_layout, file_content_manifest,
    has_flag, load_policy_file, path_string, portable_json_hash, print_json, read_option,
    read_string_option, safe_relative_path, scan_repo, with_portable_receipt_hash,
};

pub(crate) fn run_init(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = read_option(args, "--root", ".");
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
    print_json(json!({
        "command": "init",
        "root": path_string(root),
        "mutated": !created.is_empty(),
        "layout": layout_name(LifecycleLayout::Native),
        "created": created,
        "existing": existing,
    }))
}

pub(super) fn run_change_create(args: Vec<String>, change: &str) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.clone(), "--root", ".");
    let profile = read_string_option(&args, "--profile");
    let policy_path = read_option(args, "--policy", default_policy_path());
    let policy = load_policy_file(policy_path)?;
    let value = create_change_with_profile(&root, change, profile.as_deref(), &policy)?;
    print_json(with_portable_receipt_hash(&root, value))
}

pub(crate) fn create_change_with_profile(
    root: &Path,
    change: &str,
    profile_id: Option<&str>,
    policy: &Policy,
) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.lifecycle_workflow_profiles.selection.create]
    // r[impl cairn.lifecycle_workflow_profiles.selection.invalid]
    validate_change_name(change)?;
    let (selection, schema) =
        crate::workflow::select_workflow_profile_for_create(policy, profile_id)?;
    let change_root_relative = Path::new(&changes_dir(LifecycleLayout::Native)).join(change);
    let change_root = root.join(&change_root_relative);
    preflight_existing_profile(
        &change_root,
        &selection.profile_id,
        selection.profile_version,
        policy,
    )?;

    let metadata_path_relative = change_root_relative.join("metadata.json");
    let metadata_path = root.join(&metadata_path_relative);
    let metadata = profile_metadata_value(&metadata_path, &schema)?;
    let metadata_text = format!("{}\n", serde_json::to_string_pretty(&metadata)?);
    let mut created = Vec::with_capacity(schema.artifacts.len().saturating_add(2));
    let mut existing = Vec::with_capacity(schema.artifacts.len().saturating_add(2));

    create_profile_directory(root, &change_root_relative, &mut created, &mut existing)?;
    for artifact in &schema.artifacts {
        let relative = change_root_relative.join(&artifact.output_path);
        let path = root.join(&relative);
        if path.exists() {
            existing.push(path_string(relative));
            continue;
        }
        match artifact.path_kind.as_str() {
            WORKFLOW_PATH_KIND_FILE => write_new_file(
                &path,
                &profile_artifact_template(change, artifact, &schema)?,
            )?,
            WORKFLOW_PATH_KIND_SPEC_DIRECTORY => fs::create_dir_all(&path)?,
            unsupported => {
                return Err(
                    format!("unsupported workflow scaffold path kind: {unsupported}").into(),
                )
            }
        }
        created.push(path_string(relative));
    }
    if metadata_path.exists() {
        let current = fs::read_to_string(&metadata_path)?;
        if current == metadata_text {
            existing.push(path_string(metadata_path_relative.clone()));
        } else {
            fs::write(&metadata_path, &metadata_text)?;
            created.push(path_string(metadata_path_relative.clone()));
        }
    } else {
        write_new_file(&metadata_path, &metadata_text)?;
        created.push(path_string(metadata_path_relative.clone()));
    }
    created.sort();
    existing.sort();
    let scaffold = schema
        .artifacts
        .iter()
        .map(|artifact| {
            json!({
                "artifact_id": artifact.id,
                "path": artifact.output_path,
                "path_kind": artifact.path_kind,
                "structural_admission": artifact.structural_admission,
            })
        })
        .collect::<Vec<_>>();
    let scaffold_hash = portable_json_hash(
        root,
        &json!({
            "profile": selection.profile_id,
            "version": selection.profile_version,
            "artifacts": scaffold,
        }),
    );
    Ok(json!({
        "command": "change_create",
        "root": path_string(root),
        "change": change,
        "mutated": !created.is_empty(),
        "layout": layout_name(LifecycleLayout::Native),
        "workflow_profile": selection.profile_id,
        "workflow_profile_version": selection.profile_version,
        "workflow_profile_source": selection.source,
        "spec_effect": schema.spec_effect,
        "created": created,
        "existing": existing,
        "scaffold": scaffold,
        "scaffold_hash": scaffold_hash,
        "non_claims": schema.required_non_claims,
    }))
}

fn preflight_existing_profile(
    change_root: &Path,
    profile_id: &str,
    profile_version: u64,
    policy: &Policy,
) -> Result<(), Box<dyn Error>> {
    if !change_root.exists() {
        return Ok(());
    }
    let metadata_path = change_root.join("metadata.json");
    let metadata_value = if metadata_path.exists() {
        serde_json::from_str::<Value>(&fs::read_to_string(&metadata_path)?)?
    } else {
        json!({})
    };
    let metadata = workflow_change_profile_metadata_from_value(&metadata_value)?;
    if let Some(existing) = metadata.workflow_profile {
        if existing.id != profile_id || existing.version != profile_version {
            return Err("cairn.workflow.profile.selection: workflow profile selection is immutable after change creation".into());
        }
        return Ok(());
    }
    if change_contains_substantive_artifacts(change_root)? {
        return Err("cairn.workflow.profile.selection: cannot add or change a workflow profile after substantive authoring".into());
    }
    let fallback = cairn_core::select_workflow_profile(
        &policy.workflow_profile_policy,
        &policy.workflow_schemas,
        &Default::default(),
    )?;
    if fallback.0.profile_id != profile_id {
        return Err("cairn.workflow.profile.selection: existing unprofiled change can only retain the compatibility profile".into());
    }
    Ok(())
}

fn change_contains_substantive_artifacts(change_root: &Path) -> Result<bool, Box<dyn Error>> {
    let mut pending = vec![change_root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory)? {
            let entry = entry?;
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                pending.push(entry.path());
                continue;
            }
            if !file_type.is_file() || entry.file_name() == "metadata.json" {
                continue;
            }
            let text = fs::read_to_string(entry.path()).unwrap_or_default();
            if workflow_markdown_is_structurally_present(&text) {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn profile_metadata_value(
    path: &Path,
    schema: &WorkflowSchemaPolicy,
) -> Result<Value, Box<dyn Error>> {
    let mut value = if path.exists() {
        serde_json::from_str::<Value>(&fs::read_to_string(path)?)?
    } else {
        json!({})
    };
    let object = value
        .as_object_mut()
        .ok_or("change metadata must contain a JSON object")?;
    object.insert(
        "workflow_profile".to_string(),
        serde_json::to_value(WorkflowProfileRef {
            id: schema.id.clone(),
            version: schema.profile_version,
            source: WORKFLOW_PROFILE_SOURCE_CREATE.to_string(),
        })?,
    );
    if schema.spec_effect == WORKFLOW_SPEC_EFFECT_NONE {
        object.insert("spec_effect".to_string(), json!(WORKFLOW_SPEC_EFFECT_NONE));
        object
            .entry("spec_effect_rationale".to_string())
            .or_insert_with(|| {
                json!("CAIRN_PLACEHOLDER: explain why this change has no accepted-spec effect")
            });
    }
    Ok(value)
}

fn create_profile_directory(
    root: &Path,
    relative: &Path,
    created: &mut Vec<String>,
    existing: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let path = root.join(relative);
    if path.exists() {
        existing.push(path_string(relative));
    } else {
        fs::create_dir_all(&path)?;
        created.push(path_string(relative));
    }
    Ok(())
}

fn profile_artifact_template(
    change: &str,
    artifact: &WorkflowArtifactPolicy,
    schema: &WorkflowSchemaPolicy,
) -> Result<String, Box<dyn Error>> {
    match artifact.structural_admission.as_str() {
        WORKFLOW_ADMISSION_PROPOSAL => Ok(proposal_template(change)),
        WORKFLOW_ADMISSION_DESIGN => Ok(design_template(change)),
        WORKFLOW_ADMISSION_TASKS if schema.task_link_class == WORKFLOW_TASK_LINK_ACCEPTANCE => {
            Ok(format!(
                "## Phase 1: Implementation\n\n- [ ] [serial] CAIRN_PLACEHOLDER: implement the local criterion. a[{change}.acceptance]\n- [ ] [serial] CAIRN_PLACEHOLDER: verify positive and negative cases. a[{change}.acceptance]\n"
            ))
        }
        WORKFLOW_ADMISSION_TASKS => Ok(tasks_template()),
        WORKFLOW_ADMISSION_ACCEPTANCE => Ok(format!(
            "# Change-local acceptance\n\na[{change}.acceptance] CAIRN_PLACEHOLDER: define a bounded acceptance criterion.\n"
        )),
        WORKFLOW_ADMISSION_RISK => Ok(format!(
            "# Risk analysis for {change}\n\n## Assets and hazards\n\nCAIRN_PLACEHOLDER: identify bounded assets and hazards.\n\n## Mitigations and residual risks\n\nCAIRN_PLACEHOLDER: identify mitigations, owners, and residual risks.\n"
        )),
        WORKFLOW_ADMISSION_VERIFICATION => Ok(format!(
            "# Verification plan for {change}\n\n## Positive checks\n\nCAIRN_PLACEHOLDER: list commands and expected evidence.\n\n## Negative checks and gaps\n\nCAIRN_PLACEHOLDER: list failure cases and residual gaps.\n"
        )),
        WORKFLOW_ADMISSION_SEMANTIC_REVIEW => semantic_review_template(artifact, schema),
        _ => Ok(format!(
            "# {}\n\nCAIRN_PLACEHOLDER: complete this workflow artifact for `{change}`.\n",
            artifact.id
        )),
    }
}

fn semantic_review_template(
    artifact: &WorkflowArtifactPolicy,
    schema: &WorkflowSchemaPolicy,
) -> Result<String, Box<dyn Error>> {
    let obligation = schema
        .review_obligations
        .iter()
        .find(|obligation| obligation.artifact_id == artifact.id)
        .ok_or("semantic review artifact has no policy obligation")?;
    let reviewed_inputs = obligation
        .required_artifacts
        .iter()
        .filter_map(|artifact_id| {
            schema
                .artifacts
                .iter()
                .find(|candidate| candidate.id == *artifact_id)
        })
        .map(|required| {
            json!({
                "path": required.output_path,
                "hash": "CAIRN_PLACEHOLDER",
            })
        })
        .collect::<Vec<_>>();
    let value = json!({
        "schema_version": "cairn.semantic-review.review-receipt.v1",
        "review_kind": obligation.review_kind,
        "decision": "unknown",
        "reviewed_inputs": reviewed_inputs,
        "reviewer": {
            "source": "manual",
            "id": "CAIRN_PLACEHOLDER",
        },
        "findings": [{
            "id": "CAIRN_PLACEHOLDER",
            "category": "omission",
            "severity": "high",
        }],
    });
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

pub(super) fn run_change_depend(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("change depend requires a subcommand: list, add, or remove".into());
    };
    match subcommand.as_str() {
        "list" => {
            let Some(change) = iter.next() else {
                return Err("change depend list requires a change name".into());
            };
            run_change_depend_list(iter.collect(), &change)
        }
        "add" => {
            let Some(change) = iter.next() else {
                return Err("change depend add requires a change name".into());
            };
            let Some(dependency) = iter.next() else {
                return Err("change depend add requires a dependency name".into());
            };
            run_change_depend_edit(
                iter.collect(),
                &change,
                &dependency,
                ChangeDependencyEditAction::Add,
            )
        }
        "remove" => {
            let Some(change) = iter.next() else {
                return Err("change depend remove requires a change name".into());
            };
            let Some(dependency) = iter.next() else {
                return Err("change depend remove requires a dependency name".into());
            };
            run_change_depend_edit(
                iter.collect(),
                &change,
                &dependency,
                ChangeDependencyEditAction::Remove,
            )
        }
        unknown => Err(format!("unknown change depend subcommand: {unknown}").into()),
    }
}

fn run_change_depend_list(args: Vec<String>, change: &str) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.change_dependency_graph.authoring_cli]
    let root = read_option(args, "--root", ".");
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let Some(change_status) = status
        .changes
        .iter()
        .find(|candidate| candidate.name == change)
    else {
        return Err(format!("change not found: {change}").into());
    };
    let dependencies = normalized_change_dependencies(change_status);
    let valid = change_status.metadata_issues.is_empty();
    let receipt = with_portable_receipt_hash(
        &root,
        json!({
            "command": "change depend list",
            "root": path_string(&root),
            "layout": layout_name(selected.layout),
            "change": change,
            "valid": valid,
            "mutated": false,
            "dependencies": dependencies,
            "issues": change_status.metadata_issues,
        }),
    );
    print_json(receipt)?;
    if valid {
        Ok(())
    } else {
        Err("change dependency metadata invalid".into())
    }
}

fn run_change_depend_edit(
    args: Vec<String>,
    change: &str,
    dependency: &str,
    action: ChangeDependencyEditAction,
) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.change_dependency_graph.authoring_cli.preview]
    // r[impl cairn.change_dependency_graph.authoring_cli.invalid]
    let root = read_option(args.clone(), "--root", ".");
    let execute = has_flag(&args, "--execute");
    let selected = detect_layout(&root);
    let status = scan_repo(root.clone(), selected.layout)?;
    let archived_changes = collect_archived_change_ids(&root, selected.layout)?;
    let plan = plan_change_dependency_edit(
        &status.changes,
        &archived_changes,
        change,
        dependency,
        action,
    );
    let metadata_path = root
        .join(changes_dir(selected.layout))
        .join(change)
        .join("metadata.json");
    let before_manifest = file_content_manifest(&root, std::slice::from_ref(&metadata_path));
    let after_metadata_text = status
        .changes
        .iter()
        .find(|candidate| candidate.name == change)
        .map(|change_status| {
            render_dependency_metadata_preserving_profile(
                &metadata_path,
                change_status,
                plan.after.clone(),
            )
        })
        .transpose()?;

    let mut mutated = false;
    if plan.valid && execute && plan.changed {
        let Some(text) = &after_metadata_text else {
            return Err("valid dependency edit missing rendered metadata".into());
        };
        if let Some(parent) = metadata_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&metadata_path, text)?;
        mutated = true;
    }

    let mutation_manifest = if execute && plan.valid {
        let after_manifest = file_content_manifest(&root, std::slice::from_ref(&metadata_path));
        json!({
            "kind": "change_dependency_metadata",
            "before": before_manifest,
            "after": after_manifest,
            "manifest_hash": portable_json_hash(&root, &json!({"before": before_manifest, "after": after_manifest})),
        })
    } else {
        Value::Null
    };
    let receipt = dependency_edit_receipt(DependencyEditReceiptInput {
        root: &root,
        layout: selected.layout,
        metadata_path: &metadata_path,
        plan: &plan,
        execute,
        mutated,
        after_metadata_text,
        mutation_manifest,
    });
    let valid = plan.valid;
    print_json(receipt)?;
    if valid {
        Ok(())
    } else {
        Err("change dependency edit invalid".into())
    }
}

fn render_dependency_metadata_preserving_profile(
    metadata_path: &Path,
    change_status: &cairn_core::ChangeStatus,
    dependencies: Vec<String>,
) -> Result<String, Box<dyn Error>> {
    let fallback = change_metadata_with_dependencies(change_status, dependencies.clone());
    let mut value: Value = if metadata_path.is_file() {
        serde_json::from_slice(&fs::read(metadata_path)?)?
    } else {
        serde_json::from_str(&render_change_metadata_json(&fallback)?)?
    };
    let object = value
        .as_object_mut()
        .ok_or("change metadata must be a JSON object")?;
    if dependencies.is_empty() {
        object.remove("depends_on");
    } else {
        object.insert(
            "depends_on".to_string(),
            serde_json::to_value(dependencies)?,
        );
    }
    Ok(format!("{}\n", serde_json::to_string_pretty(&value)?))
}

struct DependencyEditReceiptInput<'a> {
    root: &'a Path,
    layout: LifecycleLayout,
    metadata_path: &'a Path,
    plan: &'a cairn_core::ChangeDependencyEditPlan,
    execute: bool,
    mutated: bool,
    after_metadata_text: Option<String>,
    mutation_manifest: Value,
}

fn dependency_edit_receipt(input: DependencyEditReceiptInput<'_>) -> Value {
    with_portable_receipt_hash(
        input.root,
        json!({
            "command": format!("change depend {}", input.plan.action.as_str()),
            "root": path_string(input.root),
            "layout": layout_name(input.layout),
            "change": input.plan.change,
            "dependency": input.plan.dependency,
            "metadata_path": safe_relative_path(input.root, input.metadata_path),
            "valid": input.plan.valid,
            "dry_run": !input.execute,
            "mutated": input.mutated,
            "changed": input.plan.changed,
            "before": input.plan.before,
            "after": input.plan.after,
            "after_metadata": input.after_metadata_text,
            "issues": input.plan.issues.iter().map(super::json::change_dependency_issue_json).collect::<Vec<_>>(),
            "mutation_manifest": input.mutation_manifest,
        }),
    )
}

pub(crate) fn write_new_file(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, content)
}
