use super::*;

pub(super) fn read_allocation_file(
    workspace_root: &std::path::Path,
    path: &std::path::Path,
    profile: &Profile,
) -> Result<Vec<AllocationObservation>, String> {
    let root = crate::cap_fs::SourceRoot::open_existing(workspace_root)?;
    let bytes = root.read_bounded(&crate::cap_fs::RootRelativePath::new(path)?, profile.limits.max_artifact_bytes)?;
    let text = String::from_utf8(bytes).map_err(|error| format!("benchmark allocation input is not UTF-8: {error}"))?;
    resources::parse_allocation_observations(&text)
}

pub(super) fn scan_llvm_file(
    workspace_root: &std::path::Path,
    path: &std::path::Path,
    profile: &Profile,
    workload_id: &str,
    compiler_identity: &str,
    target: &str,
    optimization_profile: &str,
) -> Result<Vec<CopyObservation>, String> {
    let root = crate::cap_fs::SourceRoot::open_existing(workspace_root)?;
    let bytes = root.read_bounded(&crate::cap_fs::RootRelativePath::new(path)?, profile.limits.max_artifact_bytes)?;
    let text = String::from_utf8(bytes).map_err(|error| format!("benchmark LLVM input is not UTF-8: {error}"))?;
    resources::scan_llvm_copy_sites(workload_id, &text, compiler_identity, target, optimization_profile)
}
