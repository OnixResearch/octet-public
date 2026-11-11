use super::*;

pub(super) fn evidence_manifest_with_relative_paths(manifest: &Value, out_dir: &Path) -> Value {
    let mut value = manifest.clone();
    if let Some(entries) = value.get_mut("artifacts").and_then(Value::as_array_mut) {
        for entry in entries {
            if let Some(path_text) = entry.get("path").and_then(Value::as_str) {
                let path = PathBuf::from(path_text);
                if let Ok(relative) = path.strip_prefix(out_dir) {
                    entry["path"] = json!(path_string(relative));
                }
            }
        }
    }
    value
}
