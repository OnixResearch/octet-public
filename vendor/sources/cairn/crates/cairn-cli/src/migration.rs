use std::{
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use cairn_core::{
    blake3_hex, build_cairn_migration_plan, build_ironspec_migration_plan,
    build_openspec_migration_plan, init_artifacts_for, layout_root, LifecycleArtifactKind,
    LifecycleLayout, MigrationInput, MigrationPlan,
};
use serde_json::Value;

use crate::plan_receipts::{migration_plan_hash, migration_plan_json};
use crate::support::{
    collect_files, default_policy_path, file_content_manifest, has_flag, load_policy_file,
    path_string, portable_json_hash, print_json, read_option, read_string_option,
    with_portable_receipt_hash,
};

// r[impl cairn.lifecycle_directory.legacy_diagnostic.migrate]
// r[impl cairn.lifecycle_directory.legacy_migration.plan_binding.execute]
pub(crate) fn run_migrate(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(source) = iter.next() else {
        return Err("migrate requires a source: cairn, openspec, or ironspec".into());
    };
    let rest = iter.collect::<Vec<_>>();
    let root = read_option(rest.clone(), "--root", ".");
    let is_execute = has_flag(&rest, "--execute");
    let expected_plan_hash = read_string_option(&rest, "--plan-hash");
    if is_execute && expected_plan_hash.is_none() {
        return Err("migration execution requires --plan-hash HASH from a reviewed preview".into());
    }
    if !is_execute && expected_plan_hash.is_some() {
        return Err("--plan-hash is valid only with --execute".into());
    }
    let policy_path = read_option(rest.clone(), "--policy", default_policy_path());
    let policy = load_policy_file(policy_path)?;
    let source_layout = migration_source_layout(&source)?;
    let inputs = collect_layout_migration_inputs(&root, source_layout)?;
    let mut plan = match source_layout {
        LifecycleLayout::LegacyCairn => build_cairn_migration_plan(inputs, is_execute),
        LifecycleLayout::LegacyOpenSpec => build_openspec_migration_plan(inputs, is_execute),
        LifecycleLayout::LegacyIronspec => build_ironspec_migration_plan(inputs, is_execute),
        LifecycleLayout::Native => return Err("cannot migrate from canonical .cairn layout".into()),
    };
    let actual_plan_hash = migration_plan_hash(&root, &policy, &plan);
    if let Some(expected) = expected_plan_hash {
        if expected != actual_plan_hash {
            return Err(format!(
                "migration plan hash mismatch: expected {expected}, actual {actual_plan_hash}"
            )
            .into());
        }
    }
    let mutation_paths = migration_mutation_paths(&plan);
    let before_manifest = file_content_manifest(&root, &mutation_paths);
    let mut mutation_manifest = Value::Null;
    if is_execute && !plan.blocked {
        execute_migration_plan(&root, &plan)?;
        plan.mutated = !plan.actions.is_empty();
        mutation_manifest = migration_mutation_manifest(&root, &plan, &before_manifest);
    }
    print_json(with_portable_receipt_hash(
        &root,
        migration_plan_json(&root, &policy, &plan, mutation_manifest),
    ))
}

pub(crate) fn collect_layout_migration_inputs(
    root: &Path,
    source_layout: LifecycleLayout,
) -> std::io::Result<Vec<MigrationInput>> {
    let source_root = root.join(layout_root(source_layout));
    if !source_root.exists() {
        return Ok(Vec::new());
    }
    let source_files = collect_files(&source_root)?;
    let mut inputs = Vec::with_capacity(source_files.len());
    for source in source_files {
        let relative = source.strip_prefix(&source_root).unwrap_or(&source);
        let destination = root.join(native_migration_destination(source_layout, relative));
        let source_hash = blake3_hex(&fs::read(&source)?);
        let destination_hash = if destination.exists() {
            Some(blake3_hex(&fs::read(&destination)?))
        } else {
            None
        };
        inputs.push(MigrationInput {
            source: path_string(&source),
            source_hash,
            destination: path_string(&destination),
            destination_hash,
            destination_exists: destination.exists(),
        });
    }
    Ok(inputs)
}

fn native_migration_destination(source_layout: LifecycleLayout, relative: &Path) -> PathBuf {
    let relative_text = path_string(relative);
    if source_layout == LifecycleLayout::LegacyOpenSpec {
        if let Some(rest) = relative_text.strip_prefix("changes/archive/") {
            return PathBuf::from(layout_root(LifecycleLayout::Native))
                .join("archive")
                .join(rest);
        }
    }
    PathBuf::from(layout_root(LifecycleLayout::Native)).join(relative)
}

pub(crate) fn migration_source_layout(source: &str) -> Result<LifecycleLayout, Box<dyn Error>> {
    match source {
        "cairn" => Ok(LifecycleLayout::LegacyCairn),
        "openspec" => Ok(LifecycleLayout::LegacyOpenSpec),
        "ironspec" => Ok(LifecycleLayout::LegacyIronspec),
        _ => Err(format!("unsupported migration source: {source}").into()),
    }
}

#[derive(Debug)]
struct PreparedMigrationFile {
    destination: PathBuf,
    bytes: Vec<u8>,
    permissions: fs::Permissions,
}

// r[impl cairn.lifecycle_directory.legacy_migration.no_replace]
fn prepare_migration_files(
    plan: &MigrationPlan,
) -> Result<Vec<PreparedMigrationFile>, Box<dyn Error>> {
    let mut prepared = Vec::with_capacity(plan.actions.len());
    for action in &plan.actions {
        let source = PathBuf::from(&action.source);
        let destination = PathBuf::from(&action.destination);
        let bytes = fs::read(&source).map_err(|source_error| {
            format!(
                "failed to read migration source {}: {source_error}",
                source.display()
            )
        })?;
        let permissions = fs::metadata(&source)
            .map_err(|source_error| {
                format!(
                    "failed to read migration source metadata {}: {source_error}",
                    source.display()
                )
            })?
            .permissions();
        let actual_source_hash = blake3_hex(&bytes);
        if actual_source_hash != action.source_hash {
            return Err(format!(
                "migration source hash mismatch for {}: expected {}, actual {}",
                source.display(),
                action.source_hash,
                actual_source_hash
            )
            .into());
        }
        if destination.exists() {
            return Err(format!(
                "migration destination appeared after planning: {}",
                destination.display()
            )
            .into());
        }
        prepared.push(PreparedMigrationFile {
            destination,
            bytes,
            permissions,
        });
    }
    Ok(prepared)
}

pub(crate) fn execute_migration_plan(
    root: &Path,
    plan: &MigrationPlan,
) -> Result<(), Box<dyn Error>> {
    let prepared = prepare_migration_files(plan)?;
    for artifact in init_artifacts_for(LifecycleLayout::Native) {
        let path = root.join(artifact.path);
        if matches!(artifact.kind, LifecycleArtifactKind::Directory) {
            fs::create_dir_all(path)?;
        }
    }
    for file in prepared {
        if let Some(parent) = file.destination.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut destination = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file.destination)
            .map_err(|source_error| {
                format!(
                    "failed to create migration destination {} without replacement: {source_error}",
                    file.destination.display()
                )
            })?;
        destination.write_all(&file.bytes).map_err(|source_error| {
            format!(
                "failed to write migration destination {}: {source_error}",
                file.destination.display()
            )
        })?;
        fs::set_permissions(&file.destination, file.permissions).map_err(|source_error| {
            format!(
                "failed to set migration destination permissions {}: {source_error}",
                file.destination.display()
            )
        })?;
    }
    Ok(())
}

fn migration_mutation_paths(plan: &MigrationPlan) -> Vec<PathBuf> {
    plan.actions
        .iter()
        .flat_map(|action| {
            [
                PathBuf::from(&action.source),
                PathBuf::from(&action.destination),
            ]
        })
        .collect()
}

fn migration_mutation_manifest(root: &Path, plan: &MigrationPlan, before: &Value) -> Value {
    let paths = migration_mutation_paths(plan);
    let after = file_content_manifest(root, &paths);
    serde_json::json!({
        "kind": "migration",
        "before": before,
        "after": after,
        "manifest_hash": portable_json_hash(root, &serde_json::json!({"before": before, "after": after})),
    })
}
