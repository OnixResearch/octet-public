use std::{
    env,
    error::Error,
    fs, io,
    path::{Component, Path, PathBuf},
};

use cairn_core::{
    admit_lifecycle_store_identity, blake3_hex, lifecycle_store_source_identity,
    LifecycleStoreDiagnostic, LifecycleStoreIdentity, LifecycleStoreIdentityProjection,
    LIFECYCLE_STORE_IDENTITY_NON_CLAIMS, LIFECYCLE_STORE_IDENTITY_SCHEMA_V1,
};
use serde::Serialize;

use crate::support::{CapabilityRoot, CapabilityRootKind};

pub(super) const STORE_SOURCE_PATH: &str = ".cairn-store/store.ncl";
pub(super) const STORE_PROJECTION_PATH: &str = ".cairn-store/store.json";
const STORE_SOURCE_SCHEMA_V1: &str = "cairn.lifecycle-store-source.v1";
const STORE_DIRECTORY: &str = ".cairn-store";
const LIFECYCLE_DIRECTORIES: &[&str] = &[".cairn/specs", ".cairn/changes", ".cairn/archive"];
pub(super) const LIFECYCLE_DIRECTORIES_FOR_RECEIPT: &[&str] = &[
    STORE_DIRECTORY,
    ".cairn/specs",
    ".cairn/changes",
    ".cairn/archive",
];
const STORE_FILE_MAX_BYTES: u64 = 1_048_576;
const SETUP_SOURCE_TEMP_PATH: &str = ".cairn-store/.store.ncl.cairn-tmp";
const SETUP_PROJECTION_TEMP_PATH: &str = ".cairn-store/.store.json.cairn-tmp";
const SETUP_LOCK_SUFFIX: &str = ".cairn-setup-lock";
const DIAGNOSTIC_IDENTITY_INVALID: &str = "lifecycle_store.identity.invalid";
const DIAGNOSTIC_STALE_PROJECTION: &str = "lifecycle_store.identity.stale_projection";
const DIAGNOSTIC_SYMLINK_PATH: &str = "lifecycle_store.locator.symlink_path";
const DIAGNOSTIC_IDENTITY_CHANGED: &str = "lifecycle_store.locator.identity_changed";

#[derive(Debug, Clone, Serialize)]
pub(super) struct StoreHealthChecks {
    pub(super) path_safe: bool,
    pub(super) native_root_present: bool,
    pub(super) source_present: bool,
    pub(super) projection_present: bool,
    pub(super) projection_current: bool,
}

#[derive(Debug, Clone)]
pub(super) struct StoreInspection {
    pub(super) path: PathBuf,
    pub(super) identity: Option<LifecycleStoreIdentity>,
    pub(super) projection: Option<LifecycleStoreIdentityProjection>,
    pub(super) diagnostics: Vec<LifecycleStoreDiagnostic>,
    pub(super) checks: StoreHealthChecks,
}

impl StoreInspection {
    pub(super) fn valid(&self) -> bool {
        self.identity.is_some() && self.diagnostics.is_empty()
    }
}

#[derive(Debug, Clone)]
pub(super) struct StoreSetupMaterial {
    pub(super) source_bytes: Vec<u8>,
    pub(super) projection_bytes: Vec<u8>,
    pub(super) identity: LifecycleStoreIdentity,
}

pub(super) fn build_setup_material(
    store_id: &str,
    remote_hint: Option<&str>,
) -> Result<StoreSetupMaterial, Vec<LifecycleStoreDiagnostic>> {
    let source = render_store_source(store_id, remote_hint);
    let source_bytes = source.into_bytes();
    let mut non_claims = LIFECYCLE_STORE_IDENTITY_NON_CLAIMS
        .iter()
        .map(|value| (*value).to_string())
        .collect::<Vec<_>>();
    non_claims.sort();
    let projection = LifecycleStoreIdentityProjection {
        schema: LIFECYCLE_STORE_IDENTITY_SCHEMA_V1.to_string(),
        store_id: store_id.to_string(),
        remote_hint: remote_hint.map(str::to_string),
        non_claims,
        source_identity: lifecycle_store_source_identity(&source_bytes),
    };
    let admission = admit_lifecycle_store_identity(&projection);
    let Some(identity) = admission.identity else {
        return Err(admission.diagnostics);
    };
    let mut projection_bytes = serde_json::to_vec_pretty(&projection)
        .expect("lifecycle-store projection serialization is infallible");
    projection_bytes.push(b'\n');
    Ok(StoreSetupMaterial {
        source_bytes,
        projection_bytes,
        identity,
    })
}

pub(super) fn normalized_setup_destination(
    path: &Path,
) -> Result<PathBuf, LifecycleStoreDiagnostic> {
    let absolute = absolute_path(path).map_err(|error| {
        diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "path",
            &path.display().to_string(),
            &format!("store setup path is unavailable: {error}"),
        )
    })?;
    reject_symlink_components(&absolute)?;
    if fs::symlink_metadata(&absolute).is_ok() {
        return Err(diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "path",
            &absolute.display().to_string(),
            "store setup refuses an existing destination",
        ));
    }
    let parent = absolute.parent().ok_or_else(|| {
        diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "path",
            &absolute.display().to_string(),
            "store setup path must have one existing parent directory",
        )
    })?;
    let file_name = absolute.file_name().ok_or_else(|| {
        diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "path",
            &absolute.display().to_string(),
            "store setup path must name one destination directory",
        )
    })?;
    let normalized_parent = fs::canonicalize(parent).map_err(|error| {
        diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "path",
            &absolute.display().to_string(),
            &format!("store setup parent is unavailable: {error}"),
        )
    })?;
    reject_symlink_components(&normalized_parent)?;
    Ok(normalized_parent.join(file_name))
}

pub(super) fn atomic_materialize_store(
    destination: &Path,
    material: &StoreSetupMaterial,
) -> Result<(), Box<dyn Error>> {
    let checked_destination =
        normalized_setup_destination(destination).map_err(|diagnostic| diagnostic.message)?;
    if checked_destination != destination {
        return Err("store setup destination changed after plan construction".into());
    }
    let parent = destination
        .parent()
        .ok_or("store setup destination requires one parent directory")?;
    let file_name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("store setup destination requires a UTF-8 directory name")?;
    let parent_root = CapabilityRoot::open(CapabilityRootKind::Repository, parent)?;
    let lock_relative = PathBuf::from(format!(".{file_name}{SETUP_LOCK_SUFFIX}"));
    let _lock = ExclusiveCapabilityFile::acquire(&parent_root, &lock_relative)?;
    if path_exists(&parent_root, Path::new(file_name))? {
        return Err("store setup destination appeared after plan construction".into());
    }
    parent_root.create_dir(Path::new(file_name))?;
    let mut destination_guard = PreparedDirectory::new(&parent_root, PathBuf::from(file_name));
    let destination_root = CapabilityRoot::open(CapabilityRootKind::Repository, destination)?;
    destination_root.create_dir_all(Path::new(STORE_DIRECTORY))?;
    for directory in LIFECYCLE_DIRECTORIES {
        destination_root.create_dir_all(Path::new(directory))?;
    }
    destination_root.write_new(Path::new(SETUP_SOURCE_TEMP_PATH), &material.source_bytes)?;
    destination_root.write_new(
        Path::new(SETUP_PROJECTION_TEMP_PATH),
        &material.projection_bytes,
    )?;
    require_file_hash(
        &destination_root,
        Path::new(SETUP_SOURCE_TEMP_PATH),
        &blake3_hex(&material.source_bytes),
    )?;
    require_file_hash(
        &destination_root,
        Path::new(SETUP_PROJECTION_TEMP_PATH),
        &blake3_hex(&material.projection_bytes),
    )?;
    destination_root.rename(
        Path::new(SETUP_SOURCE_TEMP_PATH),
        &destination_root,
        Path::new(STORE_SOURCE_PATH),
    )?;
    destination_root.rename(
        Path::new(SETUP_PROJECTION_TEMP_PATH),
        &destination_root,
        Path::new(STORE_PROJECTION_PATH),
    )?;
    let final_inspection = inspect_store(destination, None);
    if !final_inspection.valid() || final_inspection.identity.as_ref() != Some(&material.identity) {
        return Err(format!(
            "materialized lifecycle store failed identity recheck: {:?}",
            final_inspection.diagnostics
        )
        .into());
    }
    destination_guard.disarm();
    Ok(())
}

pub(super) fn inspect_store(path: &Path, expected_id: Option<&str>) -> StoreInspection {
    let mut diagnostics = Vec::new();
    let mut checks = StoreHealthChecks {
        path_safe: false,
        native_root_present: false,
        source_present: false,
        projection_present: false,
        projection_current: false,
    };
    let normalized = match normalize_existing_directory(path) {
        Ok(path) => {
            checks.path_safe = true;
            path
        }
        Err(diagnostic) => {
            diagnostics.push(diagnostic);
            return StoreInspection {
                path: absolute_path(path).unwrap_or_else(|_| path.to_path_buf()),
                identity: None,
                projection: None,
                diagnostics,
                checks,
            };
        }
    };
    let root = match CapabilityRoot::open(CapabilityRootKind::Repository, &normalized) {
        Ok(root) => root,
        Err(error) => {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_IDENTITY_INVALID,
                "path",
                &normalized.display().to_string(),
                &format!("store capability root is unavailable: {error}"),
            ));
            return StoreInspection {
                path: normalized,
                identity: None,
                projection: None,
                diagnostics,
                checks,
            };
        }
    };
    checks.native_root_present = LIFECYCLE_DIRECTORIES
        .iter()
        .all(|relative| safe_directory_exists(&root, Path::new(relative)));
    if !checks.native_root_present {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "cairn",
            &normalized.display().to_string(),
            "native lifecycle directories are missing or unsafe",
        ));
    }
    let source_bytes = match read_regular_file(&root, Path::new(STORE_SOURCE_PATH)) {
        Ok(bytes) => {
            checks.source_present = true;
            Some(bytes)
        }
        Err(diagnostic) => {
            diagnostics.push(diagnostic);
            None
        }
    };
    let projection_bytes = match read_regular_file(&root, Path::new(STORE_PROJECTION_PATH)) {
        Ok(bytes) => {
            checks.projection_present = true;
            Some(bytes)
        }
        Err(diagnostic) => {
            diagnostics.push(diagnostic);
            None
        }
    };
    let projection = projection_bytes.as_ref().and_then(|bytes| {
        serde_json::from_slice::<LifecycleStoreIdentityProjection>(bytes)
            .map_err(|error| {
                diagnostics.push(diagnostic(
                    DIAGNOSTIC_IDENTITY_INVALID,
                    STORE_PROJECTION_PATH,
                    &normalized.display().to_string(),
                    &format!("store identity projection is malformed: {error}"),
                ));
            })
            .ok()
    });
    if let (Some(source), Some(projection)) = (&source_bytes, &projection) {
        let observed_source_identity = lifecycle_store_source_identity(source);
        checks.projection_current = observed_source_identity == projection.source_identity;
        if !checks.projection_current {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_STALE_PROJECTION,
                "source_blake3",
                &normalized.display().to_string(),
                "store identity projection does not match the current Nickel source",
            ));
        }
    }
    let identity = projection.as_ref().and_then(|projection| {
        let admission = admit_lifecycle_store_identity(projection);
        diagnostics.extend(admission.diagnostics);
        admission.identity
    });
    if let (Some(expected), Some(identity)) = (expected_id, &identity) {
        if identity.store_id.as_str() != expected {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_IDENTITY_CHANGED,
                "store_id",
                identity.store_id.as_str(),
                &format!("store identity does not match requested ID {expected}"),
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    let identity = diagnostics.is_empty().then_some(identity).flatten();
    StoreInspection {
        path: normalized,
        identity,
        projection,
        diagnostics,
        checks,
    }
}

pub(super) fn setup_file_hashes(material: &StoreSetupMaterial) -> [(&'static str, String); 2] {
    [
        (STORE_SOURCE_PATH, blake3_hex(&material.source_bytes)),
        (
            STORE_PROJECTION_PATH,
            blake3_hex(&material.projection_bytes),
        ),
    ]
}

fn render_store_source(store_id: &str, remote_hint: Option<&str>) -> String {
    let remote = remote_hint.map_or_else(|| "null".to_string(), nickel_string);
    let mut non_claims = LIFECYCLE_STORE_IDENTITY_NON_CLAIMS.to_vec();
    non_claims.sort();
    let rendered_non_claims = non_claims
        .iter()
        .map(|value| format!("    {},", nickel_string(value)))
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "# Typed Cairn lifecycle-store identity source.\n\
let StoreId = std.contract.from_predicate (fun value =>\n\
  std.is_string value && std.string.is_match \"^[a-z0-9][a-z0-9._-]*$\" value\n\
) in\n\
\n\
let RemoteHint = std.contract.from_predicate (fun value =>\n\
  value == null || std.is_string value\n\
) in\n\
\n\
let NonEmptyStrings = std.contract.from_predicate (fun value =>\n\
  std.is_array value && std.array.length value > 0\n\
) in\n\
\n\
let StoreIdentitySource = {{\n\
  schema | String,\n\
  store_id | StoreId,\n\
  remote_hint | RemoteHint,\n\
  non_claims | NonEmptyStrings | Array String,\n\
}} in\n\
\n\
{{\n\
  schema = {},\n\
  store_id = {},\n\
  remote_hint = {remote},\n\
  non_claims = [\n{rendered_non_claims}\n  ],\n\
}} | StoreIdentitySource\n",
        nickel_string(STORE_SOURCE_SCHEMA_V1),
        nickel_string(store_id),
    )
}

fn nickel_string(value: &str) -> String {
    serde_json::to_string(value).expect("safe lifecycle-store text serializes")
}

fn normalize_existing_directory(path: &Path) -> Result<PathBuf, LifecycleStoreDiagnostic> {
    let absolute = absolute_path(path).map_err(|error| {
        diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "path",
            &path.display().to_string(),
            &format!("store path is unavailable: {error}"),
        )
    })?;
    reject_symlink_components(&absolute)?;
    let normalized = fs::canonicalize(&absolute).map_err(|error| {
        diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "path",
            &absolute.display().to_string(),
            &format!("store path is unavailable: {error}"),
        )
    })?;
    let metadata = fs::symlink_metadata(&normalized).map_err(|error| {
        diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "path",
            &normalized.display().to_string(),
            &format!("store path metadata is unavailable: {error}"),
        )
    })?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err(diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            "path",
            &normalized.display().to_string(),
            "store path must be one safe directory",
        ));
    }
    Ok(normalized)
}

fn absolute_path(path: &Path) -> io::Result<PathBuf> {
    if path.is_absolute() {
        Ok(path.to_path_buf())
    } else {
        Ok(env::current_dir()?.join(path))
    }
}

fn reject_symlink_components(path: &Path) -> Result<(), LifecycleStoreDiagnostic> {
    let mut observed = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => observed.push(prefix.as_os_str()),
            Component::RootDir => observed.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(diagnostic(
                    DIAGNOSTIC_IDENTITY_INVALID,
                    "path",
                    &path.display().to_string(),
                    "store path must not contain parent traversal",
                ));
            }
            Component::Normal(value) => observed.push(value),
        }
        match fs::symlink_metadata(&observed) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(diagnostic(
                    DIAGNOSTIC_SYMLINK_PATH,
                    "path",
                    &observed.display().to_string(),
                    "store path must not contain symlinks",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(diagnostic(
                    DIAGNOSTIC_IDENTITY_INVALID,
                    "path",
                    &observed.display().to_string(),
                    &format!("store path metadata is unavailable: {error}"),
                ));
            }
        }
    }
    Ok(())
}

fn safe_directory_exists(root: &CapabilityRoot, relative: &Path) -> bool {
    root.symlink_metadata(relative)
        .is_ok_and(|metadata| metadata.is_dir() && !metadata.file_type().is_symlink())
}

fn read_regular_file(
    root: &CapabilityRoot,
    relative: &Path,
) -> Result<Vec<u8>, LifecycleStoreDiagnostic> {
    let subject = root.path().display().to_string();
    let metadata = root.symlink_metadata(relative).map_err(|error| {
        diagnostic(
            DIAGNOSTIC_IDENTITY_INVALID,
            &relative.display().to_string(),
            &subject,
            &format!("store identity artifact is unavailable: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(diagnostic(
            DIAGNOSTIC_SYMLINK_PATH,
            &relative.display().to_string(),
            &subject,
            "store identity artifact must be one safe regular file",
        ));
    }
    root.read_bounded(relative, STORE_FILE_MAX_BYTES)
        .map_err(|error| {
            diagnostic(
                DIAGNOSTIC_IDENTITY_INVALID,
                &relative.display().to_string(),
                &subject,
                &format!("store identity artifact cannot be read: {error}"),
            )
        })
}

fn path_exists(root: &CapabilityRoot, relative: &Path) -> io::Result<bool> {
    match root.symlink_metadata(relative) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error),
    }
}

fn require_file_hash(
    root: &CapabilityRoot,
    relative: &Path,
    expected: &str,
) -> Result<(), Box<dyn Error>> {
    let observed = blake3_hex(&root.read_bounded(relative, STORE_FILE_MAX_BYTES)?);
    if observed == expected {
        Ok(())
    } else {
        Err(format!(
            "prepared store file identity mismatch for {}",
            relative.display()
        )
        .into())
    }
}

fn diagnostic(code: &str, field: &str, subject: &str, message: &str) -> LifecycleStoreDiagnostic {
    LifecycleStoreDiagnostic {
        code: code.to_string(),
        field: field.to_string(),
        subject: subject.to_string(),
        message: message.to_string(),
    }
}

struct ExclusiveCapabilityFile<'a> {
    root: &'a CapabilityRoot,
    relative: PathBuf,
}

impl<'a> ExclusiveCapabilityFile<'a> {
    fn acquire(root: &'a CapabilityRoot, relative: &Path) -> Result<Self, Box<dyn Error>> {
        root.write_new(relative, b"cairn lifecycle-store setup lock\n")
            .map_err(|error| format!("store setup lock is unavailable: {error}"))?;
        Ok(Self {
            root,
            relative: relative.to_path_buf(),
        })
    }
}

impl Drop for ExclusiveCapabilityFile<'_> {
    fn drop(&mut self) {
        let _ = self.root.remove_file(&self.relative);
    }
}

struct PreparedDirectory<'a> {
    root: &'a CapabilityRoot,
    relative: PathBuf,
    armed: bool,
}

impl<'a> PreparedDirectory<'a> {
    fn new(root: &'a CapabilityRoot, relative: PathBuf) -> Self {
        Self {
            root,
            relative,
            armed: true,
        }
    }

    fn disarm(&mut self) {
        self.armed = false;
    }
}

impl Drop for PreparedDirectory<'_> {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.root.remove_dir_all(&self.relative);
        }
    }
}
