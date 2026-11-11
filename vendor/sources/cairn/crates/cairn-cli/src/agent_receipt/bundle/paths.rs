use std::path::{Path, PathBuf};

use serde_json::Value;

use super::issues::agent_bundle_issue;

pub(super) fn bundle_artifact_path(
    bundle: &Value,
    base: &Path,
    field: &str,
    issues: &mut Vec<Value>,
) -> Option<PathBuf> {
    match bundle.get(field).and_then(Value::as_str) {
        Some(text) if !text.is_empty() => {
            let path = PathBuf::from(text);
            Some(if path.is_absolute() {
                path
            } else {
                base.join(path)
            })
        }
        _ => {
            issues.push(agent_bundle_issue(
                "bundle.missing_field",
                field,
                "non-empty string",
                "missing",
            ));
            None
        }
    }
}

pub(super) fn optional_bundle_artifact_path(
    bundle: &Value,
    base: &Path,
    field: &str,
    issues: &mut Vec<Value>,
) -> Option<PathBuf> {
    match bundle.get(field) {
        None | Some(Value::Null) => None,
        Some(value) => match value.as_str() {
            Some(text) if !text.is_empty() => {
                let path = PathBuf::from(text);
                Some(if path.is_absolute() {
                    path
                } else {
                    base.join(path)
                })
            }
            _ => {
                issues.push(agent_bundle_issue(
                    "bundle.invalid_field",
                    field,
                    "non-empty string",
                    "invalid",
                ));
                None
            }
        },
    }
}
