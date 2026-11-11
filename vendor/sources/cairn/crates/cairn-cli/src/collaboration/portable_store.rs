use std::{
    collections::BTreeMap,
    error::Error,
    io,
    path::{Path, PathBuf},
};

use cairn_core::{
    validate_change_name, ChangeArtifactId, PortableChangeCollaboration, MAX_ARTIFACTS,
    MAX_ARTIFACT_BYTES,
};

use crate::support::{CapabilityRoot, CapabilityRootKind};

const PORTABLE_STORE_OVERHEAD_BYTES: u64 = 16_777_216;
const MAX_PORTABLE_STORE_BYTES: u64 =
    (MAX_ARTIFACTS as u64 * MAX_ARTIFACT_BYTES as u64) + PORTABLE_STORE_OVERHEAD_BYTES;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActiveChangeArtifacts {
    pub proposal: Option<String>,
    pub design: Option<String>,
    pub tasks: Option<String>,
    pub metadata: Option<String>,
    pub delta_specs: BTreeMap<String, String>,
}

pub(super) struct PortableStoreLock {
    root: CapabilityRoot,
    relative: PathBuf,
}

impl PortableStoreLock {
    pub(super) fn acquire(store: &Path, create_parent: bool) -> Result<Self, Box<dyn Error>> {
        let (root, store_name) =
            CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, store, create_parent)?;
        let file_name = store_name
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("portable collaboration store must have a UTF-8 file name")?;
        let relative = PathBuf::from(format!(".{file_name}.lock"));
        root.write_new(&relative, b"cairn portable collaboration lock\n")
            .map_err(|error| format!("portable collaboration lock unavailable: {error}"))?;
        Ok(Self { root, relative })
    }
}

impl Drop for PortableStoreLock {
    fn drop(&mut self) {
        let _ = self.root.remove_file(&self.relative);
    }
}

pub(crate) fn read_active_change_artifacts(
    repository_root: &Path,
    change: &str,
) -> Result<ActiveChangeArtifacts, Box<dyn Error>> {
    validate_change_name(change).map_err(|reason| format!("invalid change name: {reason}"))?;
    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, repository_root)?;
    let change_directory = PathBuf::from(".cairn/changes").join(change);
    let metadata = repository
        .symlink_metadata(&change_directory)
        .map_err(|error| {
            format!(
                "active native change {} is unavailable: {error}",
                change_directory.display()
            )
        })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "active native change path is not a safe directory: {}",
            change_directory.display()
        )
        .into());
    }
    let proposal = read_optional_artifact(&repository, &change_directory.join("proposal.md"))?;
    let design = read_optional_artifact(&repository, &change_directory.join("design.md"))?;
    let tasks = read_optional_artifact(&repository, &change_directory.join("tasks.md"))?;
    let metadata = read_optional_artifact(&repository, &change_directory.join("metadata.json"))?;
    let delta_specs = read_delta_specs(&repository, &change_directory.join("specs"))?;
    let artifact_count = delta_specs
        .len()
        .saturating_add(usize::from(proposal.is_some()))
        .saturating_add(usize::from(design.is_some()))
        .saturating_add(usize::from(tasks.is_some()))
        .saturating_add(usize::from(metadata.is_some()));
    if artifact_count > MAX_ARTIFACTS {
        return Err(format!(
            "active change has {artifact_count} artifacts; maximum is {MAX_ARTIFACTS}"
        )
        .into());
    }
    Ok(ActiveChangeArtifacts {
        proposal,
        design,
        tasks,
        metadata,
        delta_specs,
    })
}

pub(super) fn store_exists(store: &Path) -> Result<bool, Box<dyn Error>> {
    let (root, relative) =
        match CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, store, false) {
            Ok(value) => value,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.into()),
        };
    match root.symlink_metadata(&relative) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                return Err("portable collaboration store is not a safe regular file".into());
            }
            Ok(true)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

pub(super) fn load_store(store: &Path) -> Result<PortableChangeCollaboration, Box<dyn Error>> {
    let (root, relative) =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, store, false)?;
    let metadata = root.symlink_metadata(&relative)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("portable collaboration store is not a safe regular file".into());
    }
    let bytes = root.read_bounded(&relative, MAX_PORTABLE_STORE_BYTES)?;
    let envelope: PortableChangeCollaboration = serde_json::from_slice(&bytes)?;
    cairn_core::validate_portable_change_collaboration(&envelope)?;
    Ok(envelope)
}

pub(super) fn commit_store(
    store: &Path,
    envelope: &PortableChangeCollaboration,
) -> Result<(), Box<dyn Error>> {
    let (root, relative) =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, store, true)?;
    if path_exists(&root, &relative)? {
        return Err("portable collaboration store appeared before commit".into());
    }
    let file_name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("portable collaboration store must have a UTF-8 file name")?;
    let prepared = PathBuf::from(format!(".{file_name}.{}.prepared", envelope.envelope_hash));
    if path_exists(&root, &prepared)? {
        return Err("portable collaboration prepared file already exists".into());
    }
    let bytes = serde_json::to_vec_pretty(envelope)?;
    let byte_count = u64::try_from(bytes.len())?;
    if byte_count > MAX_PORTABLE_STORE_BYTES {
        return Err(format!(
            "portable collaboration store exceeds byte limit: {byte_count} > {MAX_PORTABLE_STORE_BYTES}"
        )
        .into());
    }
    root.write_new(&prepared, &bytes)?;
    let verified = verify_prepared_store(&root, &prepared);
    if let Err(error) = verified {
        let _ = root.remove_file(&prepared);
        return Err(error);
    }
    if path_exists(&root, &relative)? {
        root.remove_file(&prepared)?;
        return Err("portable collaboration store appeared before atomic exposure".into());
    }
    root.rename(&prepared, &root, &relative)?;
    let reloaded = load_store(store)?;
    if &reloaded != envelope {
        return Err("portable collaboration store changed during commit verification".into());
    }
    Ok(())
}

pub(super) fn replace_store(
    store: &Path,
    expected: &PortableChangeCollaboration,
    updated: &PortableChangeCollaboration,
) -> Result<(), Box<dyn Error>> {
    let current = load_store(store)?;
    if &current != expected {
        return Err("portable collaboration store changed before replacement".into());
    }
    let (root, relative) =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, store, false)?;
    let file_name = relative
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or("portable collaboration store must have a UTF-8 file name")?;
    let prepared = PathBuf::from(format!(".{file_name}.{}.prepared", updated.envelope_hash));
    if path_exists(&root, &prepared)? {
        return Err("portable collaboration prepared file already exists".into());
    }
    let bytes = serde_json::to_vec_pretty(updated)?;
    let byte_count = u64::try_from(bytes.len())?;
    if byte_count > MAX_PORTABLE_STORE_BYTES {
        return Err(format!(
            "portable collaboration store exceeds byte limit: {byte_count} > {MAX_PORTABLE_STORE_BYTES}"
        )
        .into());
    }
    root.write_new(&prepared, &bytes)?;
    if let Err(error) = verify_prepared_store(&root, &prepared) {
        let _ = root.remove_file(&prepared);
        return Err(error);
    }
    let confirmed = load_store(store)?;
    if &confirmed != expected {
        root.remove_file(&prepared)?;
        return Err("portable collaboration store changed before atomic replacement".into());
    }
    root.rename(&prepared, &root, &relative)?;
    let reloaded = load_store(store)?;
    if &reloaded != updated {
        return Err("portable collaboration store changed during replacement verification".into());
    }
    Ok(())
}

fn verify_prepared_store(root: &CapabilityRoot, prepared: &Path) -> Result<(), Box<dyn Error>> {
    let bytes = root.read_bounded(prepared, MAX_PORTABLE_STORE_BYTES)?;
    let envelope: PortableChangeCollaboration = serde_json::from_slice(&bytes)?;
    cairn_core::validate_portable_change_collaboration(&envelope)?;
    Ok(())
}

fn read_optional_artifact(
    repository: &CapabilityRoot,
    relative: &Path,
) -> Result<Option<String>, Box<dyn Error>> {
    let metadata = match repository.symlink_metadata(relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "collaboration artifact is not a safe regular file: {}",
            relative.display()
        )
        .into());
    }
    let max_bytes = u64::try_from(MAX_ARTIFACT_BYTES)?;
    let bytes = repository.read_bounded(relative, max_bytes)?;
    Ok(Some(String::from_utf8(bytes).map_err(|error| {
        format!(
            "collaboration artifact is not UTF-8 at {}: {error}",
            relative.display()
        )
    })?))
}

fn read_delta_specs(
    repository: &CapabilityRoot,
    specs_directory: &Path,
) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
    let metadata = match repository.symlink_metadata(specs_directory) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "change specs path is not a safe directory: {}",
            specs_directory.display()
        )
        .into());
    }
    let mut entries = repository.entries_bounded(specs_directory, MAX_ARTIFACTS)?;
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    let mut specs = BTreeMap::new();
    for entry in entries {
        if !entry.file_type().is_dir() {
            return Err(format!(
                "unsupported entry in change specs directory: {}",
                specs_directory.join(entry.file_name()).display()
            )
            .into());
        }
        let spec_id = entry
            .file_name()
            .to_str()
            .ok_or("delta-spec directory name is not UTF-8")?
            .to_string();
        let artifact = ChangeArtifactId::delta_spec(&spec_id)?;
        debug_assert!(artifact.as_str().starts_with("spec:"));
        let spec_path = specs_directory.join(entry.file_name()).join("spec.md");
        let content = read_optional_artifact(repository, &spec_path)?
            .ok_or_else(|| format!("delta-spec file is missing: {}", spec_path.display()))?;
        specs.insert(spec_id, content);
    }
    Ok(specs)
}

fn path_exists(root: &CapabilityRoot, relative: &Path) -> Result<bool, io::Error> {
    match root.symlink_metadata(relative) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}
