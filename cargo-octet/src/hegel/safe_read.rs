pub(crate) fn read_checked_input(
    workspace_root: &std::path::Path,
    path: &std::path::Path,
    limit: u64,
) -> Result<Vec<u8>, String> {
    let workspace_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize Hegel input workspace {}: {error}", workspace_root.display()))?;
    let relative = crate::cap_fs::RootRelativePath::new(path)?;
    let root = crate::cap_fs::SourceRoot::open_existing(&workspace_root)?;
    read_no_symlinks(&workspace_root, &root, &relative, limit)
}

pub(super) fn read_no_symlinks(
    workspace_root: &std::path::Path,
    root: &crate::cap_fs::SourceRoot,
    relative: &crate::cap_fs::RootRelativePath,
    limit: u64,
) -> Result<Vec<u8>, String> {
    let mut observed = std::path::PathBuf::new();
    for component in relative.as_path().components() {
        let std::path::Component::Normal(component) = component else {
            continue;
        };
        observed.push(component);
        let path = workspace_root.join(&observed);
        let metadata = std::fs::symlink_metadata(&path)
            .map_err(|error| format!("inspect Hegel path {} without symlinks: {error}", path.display()))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("Hegel path {} contains a symlink and is not allowed", path.display()));
        }
    }
    root.read_bounded(relative, limit)
}
