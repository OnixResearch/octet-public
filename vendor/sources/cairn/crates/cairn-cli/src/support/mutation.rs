use super::*;

#[derive(Debug, Clone)]
pub(crate) struct SyncMergeSnapshot {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub delta_markdown: String,
    pub accepted_markdown: Option<String>,
    pub merge: DeltaMerge,
}

pub(crate) fn load_sync_merge_snapshots(
    root: &Path,
    layout: LifecycleLayout,
    change: &str,
    plan: &SyncPlan,
) -> Result<Vec<SyncMergeSnapshot>, Box<dyn Error>> {
    // r[impl cairn.sync_archive.delta_merge.mutation_truth]
    let delta_root = root.join(changes_dir(layout)).join(change).join("specs");
    let specs_root = root.join(specs_dir(layout));
    let mut snapshots = Vec::with_capacity(plan.delta_specs.len());
    for source_path in &plan.delta_specs {
        let source = PathBuf::from(source_path);
        let relative = source.strip_prefix(&delta_root).map_err(|_| {
            format!(
                "delta spec {} is outside expected change specs root {}",
                source.display(),
                delta_root.display()
            )
        })?;
        let destination = specs_root.join(relative);
        let delta_markdown = fs::read_to_string(&source).map_err(|source_error| {
            format!(
                "failed to read delta spec {}: {source_error}",
                source.display()
            )
        })?;
        let accepted_markdown = if destination.exists() {
            Some(fs::read_to_string(&destination).map_err(|source_error| {
                format!(
                    "failed to read accepted spec {}: {source_error}",
                    destination.display()
                )
            })?)
        } else {
            None
        };
        let spec_id = relative
            .parent()
            .map(path_string)
            .unwrap_or_else(|| path_string(relative));
        let merge = merge_delta_spec(accepted_markdown.as_deref(), &delta_markdown, &spec_id);
        snapshots.push(SyncMergeSnapshot {
            source,
            destination,
            delta_markdown,
            accepted_markdown,
            merge,
        });
    }
    Ok(snapshots)
}

pub(crate) fn sync_merge_blockers(root: &Path, snapshots: &[SyncMergeSnapshot]) -> Vec<String> {
    let mut blockers = snapshots
        .iter()
        .flat_map(|snapshot| {
            let source = safe_relative_path(root, &snapshot.source);
            snapshot.merge.diagnostics.iter().map(move |diagnostic| {
                format!("{source}: {}: {}", diagnostic.code, diagnostic.message)
            })
        })
        .collect::<Vec<_>>();
    blockers.sort();
    blockers.dedup();
    blockers
}

pub(crate) fn sync_merge_bindings(root: &Path, snapshots: &[SyncMergeSnapshot]) -> Value {
    // r[impl cairn.sync_archive.delta_merge.receipt_binding]
    json!({
        "documents": snapshots.iter().map(|snapshot| json!({
            "source": safe_relative_path(root, &snapshot.source),
            "destination": safe_relative_path(root, &snapshot.destination),
            "delta_hash": snapshot.merge.delta_hash,
            "accepted_before_exists": snapshot.accepted_markdown.is_some(),
            "accepted_before_hash": snapshot.merge.accepted_hash,
            "expected_after_hash": snapshot.merge.result_hash,
            "changed": snapshot.merge.changed,
            "blocked": snapshot.merge.blocked,
            "operations": snapshot.merge.operations,
            "outcomes": snapshot.merge.outcomes,
            "diagnostics": snapshot.merge.diagnostics,
        })).collect::<Vec<_>>(),
    })
}

pub(crate) fn execute_sync_snapshots(
    snapshots: &[SyncMergeSnapshot],
) -> Result<usize, Box<dyn Error>> {
    // Recheck every source and destination before the first write so a stale
    // semantic preflight cannot authorize a partially drifted operation set.
    for snapshot in snapshots {
        let current_delta = fs::read_to_string(&snapshot.source).map_err(|source_error| {
            format!(
                "failed to re-read delta spec {}: {source_error}",
                snapshot.source.display()
            )
        })?;
        if current_delta != snapshot.delta_markdown {
            return Err(format!(
                "delta spec changed after sync preflight: {}",
                snapshot.source.display()
            )
            .into());
        }
        let current_accepted = if snapshot.destination.exists() {
            Some(
                fs::read_to_string(&snapshot.destination).map_err(|source_error| {
                    format!(
                        "failed to re-read accepted spec {}: {source_error}",
                        snapshot.destination.display()
                    )
                })?,
            )
        } else {
            None
        };
        if current_accepted != snapshot.accepted_markdown {
            return Err(format!(
                "accepted spec changed after sync preflight: {}",
                snapshot.destination.display()
            )
            .into());
        }
    }

    let mut changed_files = 0;
    for snapshot in snapshots.iter().filter(|snapshot| snapshot.merge.changed) {
        if let Some(parent) = snapshot.destination.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&snapshot.destination, &snapshot.merge.markdown).map_err(|source_error| {
            format!(
                "failed to sync {} to {}: {source_error}",
                snapshot.source.display(),
                snapshot.destination.display()
            )
        })?;
        changed_files += 1;
    }
    Ok(changed_files)
}

pub(crate) fn execute_archive_plan(
    root: &Path,
    layout: LifecycleLayout,
    change: &str,
    change_path: &str,
) -> Result<PathBuf, Box<dyn Error>> {
    let source = PathBuf::from(change_path);
    let archive_date = archive_date()?;
    let destination = root
        .join(archive_dir(layout))
        .join(format!("{archive_date}-{change}"));
    if destination.exists() {
        return Err(format!(
            "archive destination already exists: {}",
            destination.display()
        )
        .into());
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&source, &destination).map_err(|source_error| {
        format!(
            "failed to archive {} to {}: {source_error}",
            source.display(),
            destination.display()
        )
    })?;
    Ok(destination)
}

fn archive_date() -> Result<String, Box<dyn Error>> {
    let date = env::var("CAIRN_ARCHIVE_DATE")
        .or_else(|_| env::var("IRONSPEC_ARCHIVE_DATE"))
        .unwrap_or_else(|_| "1970-01-01".to_string());
    if is_archive_date(&date) {
        Ok(date)
    } else {
        Err(format!(
            "invalid archive date: expected YYYY-MM-DD in CAIRN_ARCHIVE_DATE or IRONSPEC_ARCHIVE_DATE, got {date:?}"
        )
        .into())
    }
}

fn is_archive_date(date: &str) -> bool {
    let bytes = date.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

pub(crate) fn sync_changed_paths(snapshots: &[SyncMergeSnapshot]) -> Vec<PathBuf> {
    snapshots
        .iter()
        .filter(|snapshot| snapshot.merge.changed)
        .map(|snapshot| snapshot.destination.clone())
        .collect()
}

pub(crate) fn sync_mutation_manifest(
    root: &Path,
    snapshots: &[SyncMergeSnapshot],
    before: &Value,
) -> Value {
    let paths = sync_changed_paths(snapshots);
    let after = file_content_manifest(root, &paths);
    json!({
        "kind": "sync",
        "before": before,
        "after": after,
        "manifest_hash": portable_json_hash(root, &json!({"before": before, "after": after})),
    })
}

pub(crate) fn archive_mutation_manifest(
    root: &Path,
    source: &Path,
    destination: &Path,
    before: &Value,
) -> Value {
    let mut after_paths =
        tree_content_paths(root, destination).unwrap_or_else(|_| vec![destination.to_path_buf()]);
    after_paths.push(source.to_path_buf());
    let after = file_content_manifest(root, &after_paths);
    json!({
        "kind": "archive",
        "before": before,
        "after": after,
        "manifest_hash": portable_json_hash(root, &json!({"before": before, "after": after})),
    })
}
