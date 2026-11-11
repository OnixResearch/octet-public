use cairn_core::receipt_field_drift;
use serde_json::Value;

use super::support::*;
use super::types::*;

pub(super) fn replay_case_source_root(
    root: &Path,
    case: &ReplayCase,
) -> Result<PathBuf, Box<dyn Error>> {
    if case.fixture_root == "<root>" || case.fixture_root == "." {
        return Ok(root.to_path_buf());
    }
    let repository_root = CapabilityRoot::open(CapabilityRootKind::Replay, root)?;
    let fixture_relative = Path::new(&case.fixture_root);
    if let Ok(metadata) = repository_root.symlink_metadata(fixture_relative) {
        if metadata.file_type().is_symlink() {
            return Err(
                format!("replay case {} fixture root must not be a symlink", case.id).into(),
            );
        }
        if metadata.is_dir() {
            return Ok(repository_root.display_path(fixture_relative)?);
        }
    }
    let candidate = PathBuf::from(&case.fixture_root);
    if CapabilityRoot::open(CapabilityRootKind::Replay, &candidate).is_ok() {
        return Ok(candidate);
    }
    Err(format!(
        "replay case {} fixture root does not exist: {}",
        case.id, case.fixture_root
    )
    .into())
}

pub(super) fn replay_case_root(
    source: &Path,
    case: &ReplayCase,
    label: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    if case.fixture_root == "." {
        return Ok(source.to_path_buf());
    }
    replay_temp_root(source, &format!("{}-{label}", case.id))
}

pub(super) fn replay_manifest_for_case(
    root: &Path,
    case: &ReplayCase,
) -> Result<Value, Box<dyn Error>> {
    if case.fixture_root == "." {
        return Ok(json!({"manifest_hash": "not-collected-policy-root", "paths": []}));
    }
    Ok(file_content_manifest(root, &collect_files(root)?))
}

pub(super) fn run_replay_case(case: &ReplayCase, root: &Path) -> Result<Value, Box<dyn Error>> {
    match &case.kind {
        ReplayCaseKind::AgentCompletionValidate => {
            let contract_selector = case
                .args
                .get(3)
                .ok_or("agent completion replay case missing contract path")?;
            let results_selector = case
                .args
                .get(5)
                .ok_or("agent completion replay case missing results path")?;
            let subject_hash = case
                .args
                .get(7)
                .ok_or("agent completion replay case missing subject hash")?;
            let contract_path = replay_fixture_path(root, contract_selector)?;
            let results_path = replay_fixture_path(root, results_selector)?;
            let contract = read_json_artifact(&contract_path, AGENT_RECEIPT_MAX_BYTES)?;
            let results = read_json_artifact(&results_path, AGENT_RECEIPT_MAX_BYTES)?;
            let policy = load_policy_file(default_policy_path().into())?;
            let mut value = agent_completion_validation_receipt(AgentCompletionValidationInput {
                command: "agent-receipt complete",
                contract_path: Some(contract_path),
                results_path: Some(results_path),
                contract: &contract,
                results: &results,
                current_subject_hash: subject_hash,
                policy: &policy,
                input_kind: "path",
            });
            value
                .as_object_mut()
                .map(|object| object.remove("receipt_hash"));
            Ok(with_portable_receipt_hash(root, value))
        }
        ReplayCaseKind::AgentReceiptPreservesHash => {
            // r[impl cairn.determinism.preserves_hash_cli.replay_execution]
            let Some(path) = case.args.get(2) else {
                return Err("agent receipt Preserves hash replay case missing fixture path".into());
            };
            let receipt_path = replay_fixture_path(root, path)?;
            let receipt = read_agent_receipt_file(&receipt_path)?;
            let mut value = agent_preserves_hash_receipt(receipt_path, &receipt);
            value
                .as_object_mut()
                .map(|object| object.remove("receipt_hash"));
            Ok(with_portable_receipt_hash(root, value))
        }
        ReplayCaseKind::AgentReceiptValidate => {
            let Some(path) = case.args.get(2) else {
                return Err("agent receipt replay case missing fixture path".into());
            };
            let receipt_path = replay_fixture_path(root, path)?;
            let receipt = read_agent_receipt_file(&receipt_path)?;
            let policy = load_policy_file(default_policy_path().into())?;
            let mut value = agent_receipt_validation_receipt(
                "agent-receipt validate",
                Some(receipt_path),
                &receipt,
                false,
                &policy,
            );
            value
                .as_object_mut()
                .map(|object| object.remove("receipt_hash"));
            Ok(with_portable_receipt_hash(root, value))
        }
        ReplayCaseKind::ArchiveDryRun => {
            let selected = detect_layout(root);
            let change = replay_change_arg(case, "archive")?;
            let policy = load_policy_file(default_policy_path().into())?;
            let change_status = find_change(root.to_path_buf(), selected.layout, change)?;
            let input_hash = portable_json_hash(root, &change_status_json(&change_status));
            let mut plan = build_archive_plan(&change_status);
            if policy.archive_require_gate && !plan.blocked {
                plan.reasons.extend(mutation_gate_blockers(
                    root,
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
            Ok(with_portable_receipt_hash(
                root,
                archive_plan_json(
                    root,
                    selected.layout,
                    &policy,
                    &plan,
                    &input_hash,
                    Value::Null,
                ),
            ))
        }
        ReplayCaseKind::PreservesCorpusFreshness => {
            // r[impl cairn.determinism.preserves_corpus_policy_inventory.replay_case]
            let Some(path) = case.args.get(3) else {
                return Err("Preserves corpus freshness replay case missing corpus path".into());
            };
            let corpus_path = replay_fixture_path(root, path)?;
            let mut value = preserves_corpus_freshness_receipt(corpus_path);
            value
                .as_object_mut()
                .map(|object| object.remove("receipt_hash"));
            Ok(with_portable_receipt_hash(root, value))
        }
        ReplayCaseKind::Gate { stage, change } => {
            let policy = load_policy_file(default_policy_path().into())?;
            let selected = detect_layout(root);
            let change_status = find_change(root.to_path_buf(), selected.layout, change)?;
            let snapshot = read_gate_snapshot(root, selected.layout, change_status)?;
            let report = evaluate_gate_snapshot(&snapshot, *stage, &policy);
            let input_hash = portable_json_hash(
                root,
                &gate_input_json(selected.layout, *stage, &policy, &snapshot),
            );
            Ok(with_portable_receipt_hash(
                root,
                gate_report_json(selected.layout, &policy, &report, &input_hash),
            ))
        }
        ReplayCaseKind::MigrateDryRun => {
            let policy = load_policy_file(default_policy_path().into())?;
            let source_layout = LifecycleLayout::LegacyOpenSpec;
            let inputs = collect_layout_migration_inputs(root, source_layout)?;
            let plan = build_openspec_migration_plan(inputs, false);
            Ok(with_portable_receipt_hash(
                root,
                migration_plan_json(root, &policy, &plan, Value::Null),
            ))
        }
        ReplayCaseKind::McpDescribe => Ok(mcp_tool_catalog(false)),
        ReplayCaseKind::McpStatus => {
            mcp_call_tool("cairn_status", &json!({"root": path_string(root)}), false)
        }
        ReplayCaseKind::ReceiptPreservesDiff => {
            // r[impl cairn.determinism.preserves_receipt_field_drift.validation]
            // args: ["receipt-preserves-diff", "--baseline", "<path>", "--candidate", "<path>"]
            let Some(baseline_flag) = case.args.get(1) else {
                return Err("receipt-preserves-diff replay case missing --baseline flag".into());
            };
            if baseline_flag != "--baseline" {
                return Err(format!(
                    "receipt-preserves-diff replay case expected --baseline, got {baseline_flag}"
                )
                .into());
            }
            let Some(baseline_path) = case.args.get(2) else {
                return Err("receipt-preserves-diff replay case missing baseline path".into());
            };
            let Some(candidate_flag) = case.args.get(3) else {
                return Err("receipt-preserves-diff replay case missing --candidate flag".into());
            };
            if candidate_flag != "--candidate" {
                return Err(format!(
                    "receipt-preserves-diff replay case expected --candidate, got {candidate_flag}"
                )
                .into());
            }
            let Some(candidate_path) = case.args.get(4) else {
                return Err("receipt-preserves-diff replay case missing candidate path".into());
            };
            let baseline_full = replay_fixture_path(root, baseline_path)?;
            let candidate_full = replay_fixture_path(root, candidate_path)?;
            let (baseline_root, baseline_relative) = CapabilityRoot::open_file_parent(
                CapabilityRootKind::Replay,
                &baseline_full,
                false,
            )?;
            let (candidate_root, candidate_relative) = CapabilityRoot::open_file_parent(
                CapabilityRootKind::Replay,
                &candidate_full,
                false,
            )?;
            let baseline_text = baseline_root.read_to_string(&baseline_relative)?;
            let candidate_text = candidate_root.read_to_string(&candidate_relative)?;
            let baseline_val: Value =
                serde_json::from_str(&baseline_text).map_err(|e| format!("baseline parse: {e}"))?;
            let candidate_val: Value = serde_json::from_str(&candidate_text)
                .map_err(|e| format!("candidate parse: {e}"))?;
            let policy = load_policy_file(default_policy_path().into())?;
            let source_map = crate::receipt_preserves_diff::parse_source_id_map(&policy);
            let report = receipt_field_drift(&baseline_val, &candidate_val, &source_map);
            let mut value = crate::receipt_preserves_diff::diff_receipt_json(
                &baseline_full,
                &candidate_full,
                &report,
                &policy,
            );
            value
                .as_object_mut()
                .map(|object| object.remove("receipt_hash"));
            Ok(crate::support::with_portable_receipt_hash(root, value))
        }
        ReplayCaseKind::ReleaseReadiness => Ok(release_readiness_receipt(
            root.to_path_buf(),
            default_policy_path().into(),
        )),
        ReplayCaseKind::Status => {
            let selected = detect_layout(root);
            let status = scan_repo(root.to_path_buf(), selected.layout)?;
            Ok(repo_status_json(&status, selected.layout))
        }
        ReplayCaseKind::SyncDryRun => {
            let selected = detect_layout(root);
            let change = replay_change_arg(case, "sync")?;
            let policy = load_policy_file(default_policy_path().into())?;
            let change_status = find_change(root.to_path_buf(), selected.layout, change)?;
            let mut plan = build_sync_plan(&change_status);
            let merge_snapshots = load_sync_merge_snapshots(root, selected.layout, change, &plan)?;
            plan.reasons
                .extend(sync_merge_blockers(root, &merge_snapshots));
            plan.reasons.sort();
            plan.reasons.dedup();
            plan.blocked = !plan.reasons.is_empty();
            if plan.blocked {
                plan.actions.clear();
            }
            if policy.sync_require_gate && !plan.blocked {
                plan.reasons.extend(mutation_gate_blockers(
                    root,
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
            let merge_bindings = sync_merge_bindings(root, &merge_snapshots);
            let input_hash = portable_json_hash(
                root,
                &json!({
                    "change": change_status_json(&change_status),
                    "merge_preflight": merge_bindings,
                    "policy": policy.name,
                    "policy_hash": canonical_json_hash(&policy.raw),
                }),
            );
            Ok(with_portable_receipt_hash(
                root,
                sync_plan_json(
                    root,
                    selected.layout,
                    &policy,
                    &plan,
                    &input_hash,
                    &merge_bindings,
                    Value::Null,
                ),
            ))
        }
        ReplayCaseKind::Validate => {
            cairn_validation_receipt(root.to_path_buf(), default_policy_path().into())
        }
        ReplayCaseKind::View { view } => {
            // r[impl cairn.lifecycle_views.determinism]
            crate::view::lifecycle_view_receipt_for_replay(root, view)
        }
    }
}

fn replay_change_arg<'a>(case: &'a ReplayCase, command: &str) -> Result<&'a str, Box<dyn Error>> {
    case.args
        .get(1)
        .map(String::as_str)
        .ok_or_else(|| format!("{command} replay case missing change argument").into())
}

fn replay_fixture_path(root: &Path, path: &str) -> Result<PathBuf, Box<dyn Error>> {
    let relative = Path::new(path);
    let replay_root = CapabilityRoot::open(CapabilityRootKind::Replay, root)?;
    let replay_relative = replay_root.relative(relative)?;
    if capability_regular_file_exists(&replay_root, &replay_relative)? {
        return Ok(replay_root.display_path(&replay_relative)?);
    }

    let repository_path = PathBuf::from(path);
    let (repository_root, repository_relative) =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Repository, &repository_path, false)?;
    if capability_regular_file_exists(&repository_root, &repository_relative)? {
        return Ok(repository_root.display_path(&repository_relative)?);
    }
    Err(format!("replay fixture must be a regular file: {path}").into())
}

fn capability_regular_file_exists(
    root: &CapabilityRoot,
    relative: &Path,
) -> Result<bool, Box<dyn Error>> {
    match root.symlink_metadata(relative) {
        Ok(metadata) if metadata.file_type().is_symlink() => Err(format!(
            "replay fixture must not be a symlink: {}",
            relative.display()
        )
        .into()),
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn replay_output_mutated(value: &Value) -> bool {
    value
        .get("mutated")
        .and_then(Value::as_bool)
        .unwrap_or(false)
}

pub(super) fn replay_reproduce_command(
    root: &Path,
    case: Option<&str>,
    group: Option<&str>,
) -> String {
    let mut command = format!("cairn determinism replay --root {}", path_string(root));
    if let Some(case) = case {
        command.push_str(&format!(" --case {case}"));
    }
    if let Some(group) = group {
        command.push_str(&format!(" --group {group}"));
    }
    command
}

fn replay_temp_root(source: &Path, label: &str) -> Result<PathBuf, Box<dyn Error>> {
    let temp_directory = env::temp_dir();
    let destination_relative =
        PathBuf::from(format!("cairn-replay-{}-{label}", std::process::id()));
    let destination = temp_directory.join(&destination_relative);
    let temp_root = CapabilityRoot::open_or_create(CapabilityRootKind::Replay, &temp_directory)?;
    match temp_root.symlink_metadata(&destination_relative) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            return Err("replay destination must not be a symlink".into());
        }
        Ok(metadata) if metadata.is_dir() => {
            temp_root.remove_dir_all(&destination_relative)?;
        }
        Ok(_) => return Err("replay destination must be a directory".into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    temp_root.create_dir_all(&destination_relative)?;
    let source_root = CapabilityRoot::open(CapabilityRootKind::Replay, source)?;
    let destination_root = CapabilityRoot::open(CapabilityRootKind::Replay, &destination)?;
    copy_tree(&source_root, &destination_root)?;
    Ok(destination)
}

const REPLAY_COPY_EXCLUDED_DIRECTORY: &str = "target";

fn copy_tree(
    source_root: &CapabilityRoot,
    destination_root: &CapabilityRoot,
) -> Result<(), Box<dyn Error>> {
    let mut directories = vec![PathBuf::from(".")];
    while let Some(relative_directory) = directories.pop() {
        destination_root.create_dir_all(&relative_directory)?;
        let mut entries = source_root.entries(&relative_directory)?;
        entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
        for entry in entries {
            let relative = if relative_directory == Path::new(".") {
                PathBuf::from(entry.file_name())
            } else {
                relative_directory.join(entry.file_name())
            };
            if entry.file_type().is_symlink() {
                return Err(
                    format!("replay fixture contains a symlink: {}", relative.display()).into(),
                );
            }
            if entry.file_type().is_dir() && entry.file_name() == REPLAY_COPY_EXCLUDED_DIRECTORY {
                continue;
            }
            if entry.file_type().is_dir() {
                directories.push(relative);
            } else if entry.file_type().is_file() {
                source_root.copy_to(&relative, destination_root, &relative)?;
            } else {
                return Err(format!(
                    "replay fixture contains an unsupported entry: {}",
                    relative.display()
                )
                .into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const TEST_DIRECTORY_PREFIX: &str = "cairn-replay-capability";
    const TEST_FILE_CONTENT: &[u8] = b"replay fixture\n";

    fn test_directory(name: &str) -> PathBuf {
        env::temp_dir().join(format!(
            "{TEST_DIRECTORY_PREFIX}-{name}-{}",
            std::process::id()
        ))
    }

    fn remove_test_directory(path: &Path) -> Result<(), Box<dyn Error>> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    #[test]
    fn copy_tree_materializes_regular_replay_files() -> Result<(), Box<dyn Error>> {
        let source = test_directory("positive-source");
        let destination = test_directory("positive-destination");
        remove_test_directory(&source)?;
        remove_test_directory(&destination)?;
        fs::create_dir_all(source.join("nested"))?;
        fs::create_dir_all(&destination)?;
        fs::write(source.join("nested/fixture.json"), TEST_FILE_CONTENT)?;
        let source_root = CapabilityRoot::open(CapabilityRootKind::Replay, &source)?;
        let destination_root = CapabilityRoot::open(CapabilityRootKind::Replay, &destination)?;

        copy_tree(&source_root, &destination_root)?;

        assert_eq!(
            destination_root.read(Path::new("nested/fixture.json"))?,
            TEST_FILE_CONTENT
        );
        fs::remove_dir_all(source)?;
        fs::remove_dir_all(destination)?;
        Ok(())
    }

    // r[verify cairn.supersede.determinism_replay.cross_root]
    #[test]
    fn copy_tree_excludes_cargo_build_outputs() -> Result<(), Box<dyn Error>> {
        let source = test_directory("excluded-target-source");
        let destination = test_directory("excluded-target-destination");
        remove_test_directory(&source)?;
        remove_test_directory(&destination)?;
        fs::create_dir_all(source.join("target/debug"))?;
        fs::create_dir_all(source.join("nested"))?;
        fs::create_dir_all(&destination)?;
        fs::write(source.join("target/debug/derived.bin"), TEST_FILE_CONTENT)?;
        fs::write(source.join("nested/fixture.json"), TEST_FILE_CONTENT)?;
        let source_root = CapabilityRoot::open(CapabilityRootKind::Replay, &source)?;
        let destination_root = CapabilityRoot::open(CapabilityRootKind::Replay, &destination)?;

        copy_tree(&source_root, &destination_root)?;

        assert!(destination_root
            .read(Path::new("target/debug/derived.bin"))
            .is_err());
        assert_eq!(
            destination_root.read(Path::new("nested/fixture.json"))?,
            TEST_FILE_CONTENT
        );
        fs::remove_dir_all(source)?;
        fs::remove_dir_all(destination)?;
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn copy_tree_rejects_replay_symlink_escape() -> Result<(), Box<dyn Error>> {
        let source = test_directory("symlink-source");
        let destination = test_directory("symlink-destination");
        let outside = test_directory("symlink-outside");
        remove_test_directory(&source)?;
        remove_test_directory(&destination)?;
        remove_test_directory(&outside)?;
        fs::create_dir_all(&source)?;
        fs::create_dir_all(&destination)?;
        fs::create_dir_all(&outside)?;
        let outside_file = outside.join("outside.json");
        fs::write(&outside_file, TEST_FILE_CONTENT)?;
        std::os::unix::fs::symlink(&outside_file, source.join("escape.json"))?;
        let source_root = CapabilityRoot::open(CapabilityRootKind::Replay, &source)?;
        let destination_root = CapabilityRoot::open(CapabilityRootKind::Replay, &destination)?;

        let result = copy_tree(&source_root, &destination_root);

        assert!(result.is_err());
        assert!(destination_root.read(Path::new("escape.json")).is_err());
        fs::remove_dir_all(source)?;
        fs::remove_dir_all(destination)?;
        fs::remove_dir_all(outside)?;
        Ok(())
    }
}
