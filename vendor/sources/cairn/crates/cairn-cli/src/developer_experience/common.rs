use std::{error::Error, fs, path::Path};

use cairn_core::{canonical_json_hash, PlannedFile};
use serde_json::Value;

use crate::support::safe_relative_path;

pub(super) fn existing_paths(root: &Path, files: &[PlannedFile]) -> Vec<String> {
    files
        .iter()
        .filter(|file| root.join(&file.path).exists())
        .map(|file| file.path.clone())
        .collect()
}

pub(super) fn write_planned_files(
    root: &Path,
    files: &[PlannedFile],
    existing: &[String],
    issues: &mut Vec<String>,
    written: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    for file in files {
        if existing.iter().any(|path| path == &file.path) {
            continue;
        }
        write_output_file(root, Path::new(&file.path), &file.content, issues, written)?;
    }
    Ok(())
}

pub(super) fn write_output_file(
    root: &Path,
    relative: &Path,
    content: &str,
    issues: &mut Vec<String>,
    written: &mut Vec<String>,
) -> Result<(), Box<dyn Error>> {
    let relative_text = relative.to_string_lossy().replace('\\', "/");
    if relative.is_absolute() || relative_text.contains("..") {
        issues.push(format!("unsafe output path: {relative_text}"));
        return Ok(());
    }
    let path = root.join(relative);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&path, content)?;
    written.push(safe_relative_path(root, &path));
    written.sort();
    Ok(())
}

#[allow(dead_code)]
pub(super) fn stable_hash(value: &Value) -> String {
    canonical_json_hash(value)
}
