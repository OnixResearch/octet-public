use std::{error::Error, path::Path};

use cairn_core::verified::{
    change_dependencies::build_change_dependency_graph,
    lifecycle::{changes_dir, specs_dir, LifecycleLayout},
    lifecycle_views::{
        build_lifecycle_view_model, LifecycleViewChangeInput, LifecycleViewInput,
        LifecycleViewModel,
    },
    repo::{ChangeStatus, SpecSource},
};

use crate::{
    support::{
        collect_archived_change_ids, collect_validation_spec_sources, detect_layout,
        read_gate_snapshot, safe_relative_path, scan_repo,
    },
    tracey::load_traceability_coverage_model,
};

const PORTABLE_REPOSITORY_LABEL: &str = "repository";

pub(super) fn load_lifecycle_view_model(
    root: &Path,
    policy_path: &Path,
) -> Result<LifecycleViewModel, Box<dyn Error>> {
    // r[impl cairn.lifecycle_views.semantic_model]
    let selected = detect_layout(root);
    let status = scan_repo(root.to_path_buf(), selected.layout)?;
    let archived_changes = collect_archived_change_ids(root, selected.layout)?;
    let mut changes = Vec::with_capacity(status.changes.len());
    for change_status in status.changes {
        let snapshot = read_gate_snapshot(root, selected.layout, change_status.clone())?;
        let tasks_markdown = snapshot.tasks.map(|artifact| artifact.markdown);
        changes.push(LifecycleViewChangeInput {
            status: portable_change_status(root, change_status),
            tasks_markdown,
        });
    }
    let graph_statuses = changes
        .iter()
        .map(|change| change.status.clone())
        .collect::<Vec<_>>();
    let dependency_graph = build_change_dependency_graph(&graph_statuses, &archived_changes);
    let specs = collect_validation_spec_sources(root, selected.layout)?
        .into_iter()
        .map(|source| portable_spec_source(root, selected.layout, source))
        .collect();
    let traceability = load_traceability_coverage_model(root, policy_path)?;
    Ok(build_lifecycle_view_model(LifecycleViewInput {
        repository: PORTABLE_REPOSITORY_LABEL.to_string(),
        changes,
        specs,
        dependency_graph,
        traceability: Some(traceability),
    }))
}

fn portable_change_status(root: &Path, mut status: ChangeStatus) -> ChangeStatus {
    status.path = safe_relative_path(root, Path::new(&status.path));
    status.delta_specs = status
        .delta_specs
        .into_iter()
        .map(|path| safe_relative_path(root, Path::new(&path)))
        .collect();
    status
}

fn portable_spec_source(
    root: &Path,
    layout: LifecycleLayout,
    mut source: SpecSource,
) -> SpecSource {
    source.id = portable_spec_id(layout, &source.id);
    let reconstructed = source
        .id
        .strip_prefix("change/")
        .and_then(|rest| rest.split_once('/'))
        .map(|(change, spec)| {
            root.join(changes_dir(layout))
                .join(change)
                .join("specs")
                .join(spec)
                .join("spec.md")
        })
        .unwrap_or_else(|| {
            root.join(specs_dir(layout))
                .join(&source.id)
                .join("spec.md")
        });
    source.path = safe_relative_path(root, &reconstructed);
    source
}

fn portable_spec_id(layout: LifecycleLayout, source_id: &str) -> String {
    if let Some(rest) = source_id.strip_prefix("change/") {
        let Some((change, raw_spec)) = rest.split_once('/') else {
            return source_id.to_string();
        };
        let duplicated_prefix = format!("{}/{change}/specs/", changes_dir(layout));
        let spec = raw_spec
            .strip_prefix(&duplicated_prefix)
            .unwrap_or(raw_spec);
        return format!("change/{change}/{spec}");
    }
    let duplicated_prefix = format!("{}/", specs_dir(layout));
    source_id
        .strip_prefix(&duplicated_prefix)
        .unwrap_or(source_id)
        .to_string()
}
