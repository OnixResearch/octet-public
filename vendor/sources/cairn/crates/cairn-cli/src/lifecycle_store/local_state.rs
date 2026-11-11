use std::{
    env,
    error::Error,
    fs, io,
    path::{Component, Path, PathBuf},
};

use cairn_core::{
    admit_lifecycle_store_locator, empty_lifecycle_store_locator,
    lifecycle_store_locator_projection, LifecycleStoreDiagnostic, LifecycleStoreLocatorAdmission,
    LifecycleStoreLocatorProjection, LifecycleStoreLocatorState,
};

use crate::support::{CapabilityRoot, CapabilityRootKind};

pub(super) const DATA_DIRECTORY_OVERRIDE: &str = "CAIRN_DATA_DIR";
const XDG_DATA_HOME: &str = "XDG_DATA_HOME";
const HOME: &str = "HOME";
const DEFAULT_DATA_RELATIVE: &str = ".local/share";
const LOCATOR_DIRECTORY: &str = "cairn/stores";
const LOCATOR_FILE: &str = "registry.json";
const LOCATOR_LOCK_FILE: &str = ".registry.json.lock";
const LOCATOR_TEMP_FILE: &str = ".registry.json.cairn-tmp";
const LOCATOR_MAX_BYTES: u64 = 4_194_304;
const DIAGNOSTIC_IDENTITY_INVALID: &str = "lifecycle_store.identity.invalid";
const DIAGNOSTIC_SYMLINK_PATH: &str = "lifecycle_store.locator.symlink_path";

pub(super) fn data_directory() -> Result<PathBuf, Box<dyn Error>> {
    let selected = if let Some(path) = env::var_os(DATA_DIRECTORY_OVERRIDE) {
        PathBuf::from(path)
    } else if let Some(path) = env::var_os(XDG_DATA_HOME) {
        PathBuf::from(path)
    } else {
        let home = env::var_os(HOME)
            .ok_or("Cairn data directory is unavailable; set CAIRN_DATA_DIR or XDG_DATA_HOME")?;
        PathBuf::from(home).join(DEFAULT_DATA_RELATIVE)
    };
    if !selected.is_absolute() {
        return Err("Cairn data directory must be an absolute path".into());
    }
    reject_symlink_components(&selected)?;
    Ok(selected)
}

pub(super) fn locator_path(data_directory: &Path) -> PathBuf {
    data_directory.join(LOCATOR_DIRECTORY).join(LOCATOR_FILE)
}

pub(super) fn load_locator(
    data_directory: &Path,
) -> Result<LifecycleStoreLocatorAdmission, Box<dyn Error>> {
    let directory = data_directory.join(LOCATOR_DIRECTORY);
    reject_symlink_components(&directory)?;
    if !directory.exists() {
        return Ok(empty_locator_admission());
    }
    let root = CapabilityRoot::open(CapabilityRootKind::Registry, &directory)?;
    load_locator_from_root(&root)
}

pub(super) fn with_locked_locator<T>(
    data_directory: &Path,
    operation: impl FnOnce(
        &LifecycleStoreLocatorAdmission,
    ) -> Result<(T, Option<LifecycleStoreLocatorState>), Box<dyn Error>>,
) -> Result<T, Box<dyn Error>> {
    let directory = data_directory.join(LOCATOR_DIRECTORY);
    create_capability_directory(&directory)?;
    let root = CapabilityRoot::open(CapabilityRootKind::Registry, &directory)?;
    let _lock = ExclusiveLock::acquire(&root)?;
    let current = load_locator_from_root(&root)?;
    let (value, replacement) = operation(&current)?;
    if let Some(replacement) = replacement {
        atomic_replace_locator(&root, &replacement)?;
    }
    Ok(value)
}

fn load_locator_from_root(
    root: &CapabilityRoot,
) -> Result<LifecycleStoreLocatorAdmission, Box<dyn Error>> {
    let relative = Path::new(LOCATOR_FILE);
    let metadata = match root.symlink_metadata(relative) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            return Ok(empty_locator_admission());
        }
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Ok(invalid_locator_admission(diagnostic(
            DIAGNOSTIC_SYMLINK_PATH,
            "registry",
            &root.path().join(relative).display().to_string(),
            "lifecycle-store locator must be one safe regular file",
        )));
    }
    let bytes = match root.read_bounded(relative, LOCATOR_MAX_BYTES) {
        Ok(bytes) => bytes,
        Err(error) => {
            return Ok(invalid_locator_admission(diagnostic(
                DIAGNOSTIC_IDENTITY_INVALID,
                "registry",
                &root.path().join(relative).display().to_string(),
                &format!("lifecycle-store locator cannot be read: {error}"),
            )));
        }
    };
    let projection = match serde_json::from_slice::<LifecycleStoreLocatorProjection>(&bytes) {
        Ok(projection) => projection,
        Err(error) => {
            return Ok(invalid_locator_admission(diagnostic(
                DIAGNOSTIC_IDENTITY_INVALID,
                "registry",
                &root.path().join(relative).display().to_string(),
                &format!("lifecycle-store locator is malformed: {error}"),
            )));
        }
    };
    Ok(admit_lifecycle_store_locator(&projection))
}

fn empty_locator_admission() -> LifecycleStoreLocatorAdmission {
    let projection = lifecycle_store_locator_projection(&empty_lifecycle_store_locator());
    admit_lifecycle_store_locator(&projection)
}

fn invalid_locator_admission(
    diagnostic: LifecycleStoreDiagnostic,
) -> LifecycleStoreLocatorAdmission {
    LifecycleStoreLocatorAdmission {
        valid: false,
        state: None,
        diagnostics: vec![diagnostic],
    }
}

fn atomic_replace_locator(
    root: &CapabilityRoot,
    state: &LifecycleStoreLocatorState,
) -> Result<(), Box<dyn Error>> {
    reject_existing_symlink(root, Path::new(LOCATOR_FILE))?;
    reject_any_existing_path(root, Path::new(LOCATOR_TEMP_FILE))?;
    let projection = lifecycle_store_locator_projection(state);
    let mut bytes = serde_json::to_vec_pretty(&projection)?;
    bytes.push(b'\n');
    root.write_new(Path::new(LOCATOR_TEMP_FILE), &bytes)?;
    let temporary_guard = TemporaryFile::new(root, PathBuf::from(LOCATOR_TEMP_FILE));
    let reloaded_temporary = root.read_bounded(Path::new(LOCATOR_TEMP_FILE), LOCATOR_MAX_BYTES)?;
    let temporary_projection: LifecycleStoreLocatorProjection =
        serde_json::from_slice(&reloaded_temporary)?;
    let temporary_admission = admit_lifecycle_store_locator(&temporary_projection);
    if temporary_admission.state.as_ref() != Some(state) {
        return Err("prepared lifecycle-store locator failed state recheck".into());
    }
    reject_existing_symlink(root, Path::new(LOCATOR_FILE))?;
    root.rename(Path::new(LOCATOR_TEMP_FILE), root, Path::new(LOCATOR_FILE))?;
    temporary_guard.disarm();
    let committed = load_locator_from_root(root)?;
    if committed.state.as_ref() != Some(state) {
        return Err("committed lifecycle-store locator failed state recheck".into());
    }
    Ok(())
}

fn create_capability_directory(path: &Path) -> Result<(), Box<dyn Error>> {
    reject_symlink_components(path)?;
    if path.exists() {
        let metadata = fs::symlink_metadata(path)?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err("Cairn local-state path must be one safe directory".into());
        }
        return Ok(());
    }
    let mut missing = Vec::new();
    let mut existing = path.to_path_buf();
    while !existing.exists() {
        let name = existing
            .file_name()
            .ok_or("Cairn local-state path has no existing capability parent")?
            .to_os_string();
        missing.push(name);
        existing = existing
            .parent()
            .ok_or("Cairn local-state path has no existing capability parent")?
            .to_path_buf();
    }
    reject_symlink_components(&existing)?;
    let mut root = CapabilityRoot::open(CapabilityRootKind::Registry, &existing)?;
    while let Some(name) = missing.pop() {
        root.create_dir(Path::new(&name))?;
        existing.push(&name);
        root = CapabilityRoot::open(CapabilityRootKind::Registry, &existing)?;
    }
    Ok(())
}

fn reject_symlink_components(path: &Path) -> Result<(), Box<dyn Error>> {
    let mut observed = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => observed.push(prefix.as_os_str()),
            Component::RootDir => observed.push(Path::new("/")),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err("Cairn local-state path must not contain parent traversal".into());
            }
            Component::Normal(value) => observed.push(value),
        }
        match fs::symlink_metadata(&observed) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(format!(
                    "lifecycle_store.locator.symlink_path: {}",
                    observed.display()
                )
                .into());
            }
            Ok(_) => {}
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn reject_existing_symlink(root: &CapabilityRoot, relative: &Path) -> Result<(), Box<dyn Error>> {
    match root.symlink_metadata(relative) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err("lifecycle_store.locator.symlink_path: locator path is a symlink".into())
        }
        Ok(metadata) if !metadata.is_file() => {
            Err("lifecycle-store locator path is not a regular file".into())
        }
        Ok(_) => Ok(()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn reject_any_existing_path(root: &CapabilityRoot, relative: &Path) -> Result<(), Box<dyn Error>> {
    match root.symlink_metadata(relative) {
        Ok(_) => Err(format!(
            "prepared lifecycle-store locator already exists: {}",
            relative.display()
        )
        .into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
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

struct ExclusiveLock<'a> {
    root: &'a CapabilityRoot,
}

impl<'a> ExclusiveLock<'a> {
    fn acquire(root: &'a CapabilityRoot) -> Result<Self, Box<dyn Error>> {
        root.write_new(
            Path::new(LOCATOR_LOCK_FILE),
            b"cairn lifecycle-store locator lock\n",
        )
        .map_err(|error| format!("lifecycle-store locator lock is unavailable: {error}"))?;
        Ok(Self { root })
    }
}

impl Drop for ExclusiveLock<'_> {
    fn drop(&mut self) {
        let _ = self.root.remove_file(Path::new(LOCATOR_LOCK_FILE));
    }
}

struct TemporaryFile<'a> {
    root: &'a CapabilityRoot,
    relative: PathBuf,
    armed: std::cell::Cell<bool>,
}

impl<'a> TemporaryFile<'a> {
    fn new(root: &'a CapabilityRoot, relative: PathBuf) -> Self {
        Self {
            root,
            relative,
            armed: std::cell::Cell::new(true),
        }
    }

    fn disarm(&self) {
        self.armed.set(false);
    }
}

impl Drop for TemporaryFile<'_> {
    fn drop(&mut self) {
        if self.armed.get() {
            let _ = self.root.remove_file(&self.relative);
        }
    }
}
