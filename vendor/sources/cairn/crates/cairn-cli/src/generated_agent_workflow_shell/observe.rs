use std::{
    collections::BTreeSet,
    error::Error,
    io,
    path::{Path, PathBuf},
};

use cairn_core::{
    blake3_hex, generated_manifest_relative_path, join_generated_destination,
    render_generated_agent_workflows, validate_generated_workflow_manifest,
    AdmittedGeneratedWorkflowCatalog, GeneratedWorkflowDestinationSnapshot,
    GeneratedWorkflowFileObservation, GeneratedWorkflowManifest,
    GeneratedWorkflowManifestObservation, GeneratedWorkflowObservedKind,
};

use crate::support::CapabilityRoot;

pub(super) fn observe_generated_workflow_destination(
    root: &CapabilityRoot,
    catalog: &AdmittedGeneratedWorkflowCatalog,
    adapter_id: &str,
    destination: &str,
) -> Result<GeneratedWorkflowDestinationSnapshot, Box<dyn Error>> {
    let max_path_bytes = catalog.policy.bounds.max_path_bytes;
    let manifest_path = join_generated_destination(
        destination,
        &generated_manifest_relative_path(adapter_id),
        max_path_bytes,
    )?;
    let manifest = observe_manifest(
        root,
        Path::new(&manifest_path),
        catalog.policy.bounds.max_file_bytes,
        catalog.policy.bounds.max_files_per_adapter,
        catalog.policy.bounds.max_path_bytes,
        destination,
        adapter_id,
    )?;
    let mut paths = render_generated_agent_workflows(catalog, adapter_id)
        .map_err(|issues| format!("generated workflow rendering failed: {issues:?}"))?
        .into_iter()
        .map(|file| join_generated_destination(destination, &file.relative_path, max_path_bytes))
        .collect::<Result<BTreeSet<_>, _>>()?;
    if let GeneratedWorkflowManifestObservation::Present(prior) = &manifest {
        paths.extend(prior.files.iter().map(|file| file.relative_path.clone()));
    }
    let files = paths
        .into_iter()
        .map(|path| observe_file(root, Path::new(&path), catalog.policy.bounds.max_file_bytes))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(GeneratedWorkflowDestinationSnapshot { files, manifest })
}

fn observe_manifest(
    root: &CapabilityRoot,
    path: &Path,
    max_file_bytes: usize,
    max_files: usize,
    max_path_bytes: usize,
    destination: &str,
    adapter_id: &str,
) -> Result<GeneratedWorkflowManifestObservation, Box<dyn Error>> {
    if has_symlink_component(root, path)? {
        return Ok(GeneratedWorkflowManifestObservation::Symlink);
    }
    let metadata = match root.symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(GeneratedWorkflowManifestObservation::Missing);
        }
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() {
        return Ok(GeneratedWorkflowManifestObservation::Symlink);
    }
    if !metadata.is_file() {
        return Ok(GeneratedWorkflowManifestObservation::Other);
    }
    let max_bytes = u64::try_from(max_file_bytes)?;
    let bytes = match root.read_bounded(path, max_bytes) {
        Ok(bytes) => bytes,
        Err(error) => {
            return Ok(GeneratedWorkflowManifestObservation::Invalid(
                error.to_string(),
            ));
        }
    };
    let parsed = serde_json::from_slice::<GeneratedWorkflowManifest>(&bytes);
    let manifest = match parsed {
        Ok(manifest) => manifest,
        Err(error) => {
            return Ok(GeneratedWorkflowManifestObservation::Invalid(
                error.to_string(),
            ));
        }
    };
    let path_prefix = format!("{destination}/");
    let paths_valid = manifest.files.len() <= max_files
        && manifest.files.iter().all(|file| {
            file.relative_path.starts_with(&path_prefix)
                && !file.relative_path.contains("/../")
                && file.relative_path.len() <= max_path_bytes
        });
    if !validate_generated_workflow_manifest(&manifest)
        || manifest.adapter_id != adapter_id
        || manifest.destination != destination
        || !paths_valid
    {
        return Ok(GeneratedWorkflowManifestObservation::Invalid(
            "manifest identity, adapter, destination, or file set is invalid".to_string(),
        ));
    }
    Ok(GeneratedWorkflowManifestObservation::Present(Box::new(
        manifest,
    )))
}

fn observe_file(
    root: &CapabilityRoot,
    path: &Path,
    max_file_bytes: usize,
) -> Result<GeneratedWorkflowFileObservation, Box<dyn Error>> {
    let relative_path = path
        .to_str()
        .ok_or("generated workflow path must be UTF-8")?
        .to_string();
    if has_symlink_component(root, path)? {
        return Ok(GeneratedWorkflowFileObservation {
            relative_path,
            kind: GeneratedWorkflowObservedKind::Symlink,
            content_blake3: None,
        });
    }
    let metadata = match root.symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(GeneratedWorkflowFileObservation {
                relative_path,
                kind: GeneratedWorkflowObservedKind::Missing,
                content_blake3: None,
            });
        }
        Err(error) => return Err(error.into()),
    };
    let (kind, content_blake3) = if metadata.file_type().is_symlink() {
        (GeneratedWorkflowObservedKind::Symlink, None)
    } else if metadata.is_file() {
        let bytes = root.read_bounded(path, u64::try_from(max_file_bytes)?)?;
        (
            GeneratedWorkflowObservedKind::Regular,
            Some(blake3_hex(&bytes)),
        )
    } else {
        (GeneratedWorkflowObservedKind::Other, None)
    };
    Ok(GeneratedWorkflowFileObservation {
        relative_path,
        kind,
        content_blake3,
    })
}

fn has_symlink_component(root: &CapabilityRoot, path: &Path) -> Result<bool, Box<dyn Error>> {
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component.as_os_str());
        match root.symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => return Ok(true),
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(false)
}
