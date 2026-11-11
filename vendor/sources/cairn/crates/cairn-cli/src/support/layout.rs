use super::*;

#[derive(Debug, Clone, Copy)]
pub(crate) struct SelectedLayout {
    pub(crate) layout: LifecycleLayout,
}

pub(crate) fn detect_layout(_root: &Path) -> SelectedLayout {
    SelectedLayout {
        layout: LifecycleLayout::Native,
    }
}

fn lifecycle_subdirectory_is_present(root: &Path, layout: LifecycleLayout) -> bool {
    [
        specs_dir(layout),
        changes_dir(layout),
        archive_dir(layout).to_string(),
    ]
    .iter()
    .any(|relative| root.join(relative).is_dir())
}

pub(crate) fn legacy_only_lifecycle_root(root: &Path) -> bool {
    // r[impl cairn.lifecycle_directory.legacy_diagnostic]
    lifecycle_subdirectory_is_present(root, LifecycleLayout::LegacyCairn)
        && !lifecycle_subdirectory_is_present(root, LifecycleLayout::Native)
}

pub(crate) fn reject_legacy_only_lifecycle_root(root: &Path) -> Result<(), Box<dyn Error>> {
    if legacy_only_lifecycle_root(root) {
        return Err(format!(
            "cairn.lifecycle_directory.migration_required: {} contains old cairn/ lifecycle content but no native .cairn/ lifecycle content; preview `cairn migrate cairn --root {}`",
            root.display(),
            root.display()
        )
        .into());
    }
    Ok(())
}

pub(crate) fn load_policy_file(path: PathBuf) -> Result<Policy, Box<dyn Error>> {
    let text = fs::read_to_string(&path)
        .map_err(|source| format!("failed to read policy {}: {source}", path.display()))?;
    let value: Value = serde_json::from_str(&text)
        .map_err(|source| format!("failed to parse policy {}: {source}", path.display()))?;
    validate_policy_json(value.clone()).map_err(|source| {
        let diagnostic = policy_schema_compatibility_diagnostic(&value, &source)
            .unwrap_or_else(|| source.to_string());
        format!("failed to parse policy {}: {diagnostic}", path.display()).into()
    })
}

pub(crate) fn default_policy_path() -> &'static str {
    "cairn-policy/generated/cairn-policy.json"
}
