use std::path::{Path, PathBuf};

use serde_json::Value;

pub(super) fn evidence_publish_source_bundle_dir(
    artifact_dir: &Path,
    source_index_receipt: Option<&Value>,
) -> PathBuf {
    let index_path = artifact_dir.join("release-evidence-index.json");
    let parent = index_path.parent().unwrap_or(artifact_dir);
    if let Some(relative) = source_index_receipt
        .and_then(|receipt| receipt.get("bundle_relative"))
        .and_then(Value::as_str)
    {
        if safe_relative_path_text(relative) {
            return parent.join(relative);
        }
    }
    artifact_dir.join("republished")
}

pub(super) fn safe_relative_path_text(relative: &str) -> bool {
    let path = PathBuf::from(relative);
    !relative.is_empty()
        && !path.is_absolute()
        && !path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
}

pub(super) fn publish_relative_path(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}
