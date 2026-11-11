use std::ffi::OsStr;
use std::io::Read as _;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use cap_std::ambient_authority;
use cap_std::fs::Dir;
use cap_std::fs::File;
#[cfg(unix)]
use cap_std::fs::OpenOptions;
#[cfg(unix)]
use cap_std::fs::OpenOptionsExt as _;

const CURRENT_DIR: &str = ".";
const PARENT_COMPONENT: &str = "..";
const ABSOLUTE_COMPONENT: &str = "absolute";
const PREFIX_COMPONENT: &str = "prefix";
const EMPTY_LOCATOR: &str = "empty";
const ATOMIC_STAGE_SUFFIX: &str = ".octet-stage";
/// Stage names tried before giving up; a name is skipped only when a leftover file holds it.
const ATOMIC_STAGE_CREATE_ATTEMPTS: u32 = 16;

/// With the process ID, gives every atomic write in this process its own stage name.
static ATOMIC_STAGE_SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RootRelativePath {
    inner: PathBuf,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RootFile {
    pub(crate) locator: String,
    pub(crate) contents: Vec<u8>,
}

pub(crate) struct ArtifactRoot {
    root: CapabilityRoot,
}

pub(crate) struct CacheRoot {
    root: CapabilityRoot,
}

pub(crate) struct ProofRoot {
    root: CapabilityRoot,
}

pub(crate) struct SourceRoot {
    root: CapabilityRoot,
}

pub(crate) struct WorkspaceRoot {
    root: CapabilityRoot,
}

struct CapabilityRoot {
    kind: &'static str,
    display_path: PathBuf,
    dir: Dir,
}

impl RootRelativePath {
    pub(crate) fn new(path: impl AsRef<Path>) -> Result<Self, String> {
        validate_relative_path(path.as_ref())
    }

    pub(crate) fn current() -> Self {
        Self {
            inner: PathBuf::from(CURRENT_DIR),
        }
    }

    pub(crate) fn as_path(&self) -> &Path {
        &self.inner
    }

    pub(crate) fn display(&self) -> String {
        self.inner.display().to_string().replace('\\', "/")
    }

    fn join_child(&self, child: &OsStr) -> Result<Self, String> {
        let mut joined = if self.inner == Path::new(CURRENT_DIR) {
            PathBuf::new()
        } else {
            self.inner.clone()
        };
        joined.push(child);
        validate_relative_path(&joined)
    }
}

impl ArtifactRoot {
    pub(crate) fn open_existing(path: &Path) -> Result<Self, String> {
        CapabilityRoot::open_existing("artifact", path).map(|root| Self { root })
    }

    pub(crate) fn open_or_create(path: &Path) -> Result<Self, String> {
        CapabilityRoot::open_or_create("artifact", path).map(|root| Self { root })
    }

    pub(crate) fn read_files_recursive(&self) -> Result<Vec<RootFile>, String> {
        self.root.read_files_recursive()
    }

    pub(crate) fn read_entry_names(&self) -> Result<Vec<String>, String> {
        self.root.read_entry_names(&RootRelativePath::current())
    }

    pub(crate) fn read_to_string(&self, relative: &RootRelativePath) -> Result<String, String> {
        self.root.read_to_string(relative)
    }

    pub(crate) fn read_bounded(&self, relative: &RootRelativePath, max_bytes: u64) -> Result<Vec<u8>, String> {
        self.root.read_bounded(relative, max_bytes)
    }

    pub(crate) fn is_file(&self, relative: &RootRelativePath) -> Result<bool, String> {
        self.root.is_file(relative)
    }

    pub(crate) fn create_dir_all(&self, relative: &RootRelativePath) -> Result<(), String> {
        self.root.create_dir_all(relative)
    }

    pub(crate) fn write(&self, relative: &RootRelativePath, bytes: &[u8]) -> Result<(), String> {
        self.root.write(relative, bytes)
    }

    pub(crate) fn write_atomic(&self, relative: &RootRelativePath, bytes: &[u8]) -> Result<(), String> {
        self.root.write_atomic(relative, bytes)
    }

    pub(crate) fn remove_file_or_dir_all(&self, relative: &RootRelativePath) -> Result<(), String> {
        self.root.remove_file_or_dir_all(relative)
    }
}

impl CacheRoot {
    pub(crate) fn open_or_create(path: &Path) -> Result<Self, String> {
        CapabilityRoot::open_or_create("cache", path).map(|root| Self { root })
    }

    pub(crate) fn read_to_string_optional(&self, relative: &RootRelativePath) -> Result<Option<String>, String> {
        self.root.read_to_string_optional(relative)
    }

    pub(crate) fn write_atomic(&self, relative: &RootRelativePath, bytes: &[u8]) -> Result<(), String> {
        self.root.write_atomic(relative, bytes)
    }
}

impl ProofRoot {
    pub(crate) fn open_existing(path: &Path) -> Result<Self, String> {
        CapabilityRoot::open_existing("proof", path).map(|root| Self { root })
    }

    pub(crate) fn read_to_string(&self, relative: &RootRelativePath) -> Result<String, String> {
        self.root.read_to_string(relative)
    }
}

impl SourceRoot {
    pub(crate) fn open_existing(path: &Path) -> Result<Self, String> {
        CapabilityRoot::open_existing("source", path).map(|root| Self { root })
    }

    pub(crate) fn read_files_recursive_from(&self, relative: &RootRelativePath) -> Result<Vec<RootFile>, String> {
        self.root.read_files_recursive_from(relative)
    }

    pub(crate) fn read_to_string(&self, relative: &RootRelativePath) -> Result<String, String> {
        self.root.read_to_string(relative)
    }

    pub(crate) fn read_bounded(&self, relative: &RootRelativePath, max_bytes: u64) -> Result<Vec<u8>, String> {
        self.root.read_bounded(relative, max_bytes)
    }

    pub(crate) fn is_dir(&self, relative: &RootRelativePath) -> Result<bool, String> {
        self.root.is_dir(relative)
    }
}

impl WorkspaceRoot {
    pub(crate) fn open_existing(path: &Path) -> Result<Self, String> {
        CapabilityRoot::open_existing("workspace", path).map(|root| Self { root })
    }

    pub(crate) fn read_to_string(&self, relative: &RootRelativePath) -> Result<String, String> {
        self.root.read_to_string(relative)
    }
}

impl CapabilityRoot {
    fn open_existing(kind: &'static str, path: &Path) -> Result<Self, String> {
        let dir = Dir::open_ambient_dir(path, ambient_authority())
            .map_err(|error| format!("open {kind} root {}: {error}", path.display()))?;
        Ok(Self {
            kind,
            display_path: path.to_path_buf(),
            dir,
        })
    }

    fn open_or_create(kind: &'static str, path: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(path).map_err(|error| format!("create {kind} root {}: {error}", path.display()))?;
        Self::open_existing(kind, path)
    }

    fn read(&self, relative: &RootRelativePath) -> Result<Vec<u8>, String> {
        let read_error =
            |error| format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display());
        let file = self.open_for_read(relative).map_err(read_error)?;
        let len = self.require_regular_file(relative, &file)?;
        let mut bytes = Vec::with_capacity(usize::try_from(len).unwrap_or(0));
        (&file).read_to_end(&mut bytes).map_err(read_error)?;
        Ok(bytes)
    }

    fn read_bounded(&self, relative: &RootRelativePath, max_bytes: u64) -> Result<Vec<u8>, String> {
        let file = self.open_for_read(relative).map_err(|error| {
            format!("open {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
        })?;
        if self.require_regular_file(relative, &file)? > max_bytes {
            return Err(format!(
                "{kind} path `{locator}` exceeds {max_bytes} byte limit",
                kind = self.kind,
                locator = relative.display()
            ));
        }
        let read_limit = max_bytes.checked_add(1).ok_or_else(|| {
            format!(
                "{kind} path `{locator}` byte limit cannot include the growth sentinel",
                kind = self.kind,
                locator = relative.display()
            )
        })?;
        let mut bytes = Vec::new();
        file.take(read_limit).read_to_end(&mut bytes).map_err(|error| {
            format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
        })?;
        let observed =
            u64::try_from(bytes.len()).map_err(|_| "bounded capability read length does not fit u64".to_owned())?;
        if observed > max_bytes {
            return Err(format!(
                "{kind} path `{locator}` grew beyond {max_bytes} byte limit",
                kind = self.kind,
                locator = relative.display()
            ));
        }
        Ok(bytes)
    }

    fn read_to_string(&self, relative: &RootRelativePath) -> Result<String, String> {
        let file = self.open_for_read(relative).map_err(|error| {
            format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
        })?;
        self.read_regular_to_string(relative, &file)
    }

    fn read_to_string_optional(&self, relative: &RootRelativePath) -> Result<Option<String>, String> {
        match self.open_for_read(relative) {
            Ok(file) => self.read_regular_to_string(relative, &file).map(Some),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => {
                Err(format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display()))
            }
        }
    }

    /// Opens an in-root file for reading without blocking on special files.
    ///
    /// On Unix the open uses `O_NONBLOCK`, so a FIFO with no writer cannot
    /// stall the shell before [`Self::require_regular_file`] rejects it. The
    /// flag does not change reads from regular files.
    fn open_for_read(&self, relative: &RootRelativePath) -> std::io::Result<File> {
        #[cfg(unix)]
        {
            let mut options = OpenOptions::new();
            options.read(true).custom_flags(libc::O_NONBLOCK);
            self.dir.open_with(relative.as_path(), &options)
        }
        #[cfg(not(unix))]
        {
            self.dir.open(relative.as_path())
        }
    }

    /// Inspects the opened handle and returns its length only for regular files.
    fn require_regular_file(&self, relative: &RootRelativePath, file: &File) -> Result<u64, String> {
        let metadata = file.metadata().map_err(|error| {
            format!("inspect {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
        })?;
        if !metadata.is_file() {
            return Err(format!(
                "{kind} path `{locator}` is not a regular file",
                kind = self.kind,
                locator = relative.display()
            ));
        }
        Ok(metadata.len())
    }

    fn read_regular_to_string(&self, relative: &RootRelativePath, file: &File) -> Result<String, String> {
        let len = self.require_regular_file(relative, file)?;
        let mut contents = String::with_capacity(usize::try_from(len).unwrap_or(0));
        let mut reader = file;
        reader.read_to_string(&mut contents).map_err(|error| {
            format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
        })?;
        Ok(contents)
    }

    fn write(&self, relative: &RootRelativePath, bytes: &[u8]) -> Result<(), String> {
        if let Some(parent) = relative.as_path().parent().filter(|parent| !parent.as_os_str().is_empty()) {
            let parent = RootRelativePath::new(parent)?;
            self.create_dir_all(&parent)?;
        }
        self.dir.write(relative.as_path(), bytes).map_err(|error| {
            format!("write {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
        })
    }

    /// Publishes `bytes` at `relative` by writing a stage file beside it and renaming the stage
    /// over the target, so readers see the old file or the new one, never a partial one. Every
    /// call stages under its own name, `.{file}.{pid}.{sequence}.octet-stage`, created
    /// exclusively, so concurrent writers to one locator, in this process or another, never
    /// share or delete each other's stage: all of them succeed and the last rename wins. A write
    /// that fails after creating its stage removes it. A process killed between creating and
    /// renaming its stage leaves that hidden file behind; nothing sweeps such files, and later
    /// writes never reuse an existing name.
    fn write_atomic(&self, relative: &RootRelativePath, bytes: &[u8]) -> Result<(), String> {
        if let Some(parent) = relative.as_path().parent().filter(|parent| !parent.as_os_str().is_empty()) {
            let parent = RootRelativePath::new(parent)?;
            self.create_dir_all(&parent)?;
        }
        let (stage, mut file) = self.create_atomic_stage(relative)?;
        let written = std::io::Write::write_all(&mut file, bytes).map_err(|error| {
            format!("write atomic {kind} stage `{locator}`: {error}", kind = self.kind, locator = stage.display())
        });
        drop(file);
        let published = written.and_then(|()| {
            self.dir.rename(stage.as_path(), &self.dir, relative.as_path()).map_err(|error| {
                format!(
                    "publish atomic {kind} path `{locator}`: {error}",
                    kind = self.kind,
                    locator = relative.display()
                )
            })
        });
        if published.is_err() {
            // Best effort: the publish error is the one to report.
            let _ = self.dir.remove_file(stage.as_path());
        }
        published
    }

    /// Creates a new, exclusively owned stage file for `relative` in the same directory.
    fn create_atomic_stage(&self, relative: &RootRelativePath) -> Result<(RootRelativePath, File), String> {
        let file_name = relative.as_path().file_name().and_then(OsStr::to_str).ok_or_else(|| {
            format!(
                "atomic {kind} path `{locator}` has no UTF-8 file name",
                kind = self.kind,
                locator = relative.display()
            )
        })?;
        let parent = relative.as_path().parent().filter(|parent| !parent.as_os_str().is_empty());
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        for _ in 0..ATOMIC_STAGE_CREATE_ATTEMPTS {
            let sequence = ATOMIC_STAGE_SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let stage_name = format!(".{file_name}.{}.{sequence}{ATOMIC_STAGE_SUFFIX}", std::process::id());
            let stage = RootRelativePath::new(
                parent.map_or_else(|| PathBuf::from(&stage_name), |parent| parent.join(&stage_name)),
            )?;
            match self.dir.open_with(stage.as_path(), &options) {
                Ok(file) => return Ok((stage, file)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => {
                    return Err(format!(
                        "create atomic {kind} stage `{locator}`: {error}",
                        kind = self.kind,
                        locator = stage.display()
                    ));
                }
            }
        }
        Err(format!(
            "create atomic {kind} stage for `{locator}`: {ATOMIC_STAGE_CREATE_ATTEMPTS} candidate names already exist",
            kind = self.kind,
            locator = relative.display()
        ))
    }

    fn create_dir_all(&self, relative: &RootRelativePath) -> Result<(), String> {
        self.dir.create_dir_all(relative.as_path()).map_err(|error| {
            format!(
                "create {kind} path `{locator}` under {}: {error}",
                self.display_path.display(),
                kind = self.kind,
                locator = relative.display()
            )
        })
    }

    fn read_entry_names(&self, relative: &RootRelativePath) -> Result<Vec<String>, String> {
        let mut names = self
            .dir
            .read_dir(relative.as_path())
            .map_err(|error| {
                format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
            })?
            .map(|entry| {
                entry
                    .map_err(|error| {
                        format!(
                            "read {kind} path `{locator}` entry: {error}",
                            kind = self.kind,
                            locator = relative.display()
                        )
                    })
                    .and_then(|entry| {
                        entry.file_name().into_string().map_err(|_| {
                            format!(
                                "read {kind} path `{locator}` entry has non-utf8 name",
                                kind = self.kind,
                                locator = relative.display()
                            )
                        })
                    })
            })
            .collect::<Result<Vec<_>, String>>()?;
        names.sort();
        Ok(names)
    }

    fn read_files_recursive(&self) -> Result<Vec<RootFile>, String> {
        self.read_files_recursive_from(&RootRelativePath::current())
    }

    fn read_files_recursive_from(&self, relative: &RootRelativePath) -> Result<Vec<RootFile>, String> {
        let mut files = Vec::new();
        self.append_files_recursive(relative, &mut files)?;
        files.sort_by(|left, right| left.locator.cmp(&right.locator));
        Ok(files)
    }

    fn is_dir(&self, relative: &RootRelativePath) -> Result<bool, String> {
        let metadata = self.dir.symlink_metadata(relative.as_path()).map_err(|error| {
            format!("inspect {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
        })?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "{kind} path `{locator}` is a symlink and is not allowed",
                kind = self.kind,
                locator = relative.display()
            ));
        }
        Ok(metadata.file_type().is_dir())
    }

    fn is_file(&self, relative: &RootRelativePath) -> Result<bool, String> {
        match self.dir.symlink_metadata(relative.as_path()) {
            Ok(metadata) => Ok(metadata.file_type().is_file()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => {
                Err(format!("inspect {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display()))
            }
        }
    }

    fn append_files_recursive(&self, relative: &RootRelativePath, files: &mut Vec<RootFile>) -> Result<(), String> {
        let mut entries = self
            .dir
            .read_dir(relative.as_path())
            .map_err(|error| {
                format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
            })?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|error| {
                format!("read {kind} path `{locator}` entry: {error}", kind = self.kind, locator = relative.display())
            })?;
        entries.sort_by_key(cap_std::fs::DirEntry::file_name);

        for entry in entries {
            let child = relative.join_child(&entry.file_name())?;
            let file_type = entry.file_type().map_err(|error| {
                format!("inspect {kind} path `{locator}`: {error}", kind = self.kind, locator = child.display())
            })?;
            if file_type.is_symlink() {
                return Err(format!(
                    "{kind} path `{locator}` is a symlink and is not allowed",
                    kind = self.kind,
                    locator = child.display()
                ));
            }
            if file_type.is_dir() {
                self.append_files_recursive(&child, files)?;
                continue;
            }
            if !file_type.is_file() {
                return Err(format!(
                    "{kind} path `{locator}` is not a regular file or directory",
                    kind = self.kind,
                    locator = child.display()
                ));
            }
            files.push(RootFile {
                locator: child.display(),
                contents: self.read(&child)?,
            });
        }
        Ok(())
    }

    fn remove_file_or_dir_all(&self, relative: &RootRelativePath) -> Result<(), String> {
        let metadata = match self.dir.symlink_metadata(relative.as_path()) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => {
                return Err(format!(
                    "inspect {kind} path `{locator}`: {error}",
                    kind = self.kind,
                    locator = relative.display()
                ));
            }
        };
        if metadata.file_type().is_dir() {
            self.dir.remove_dir_all(relative.as_path()).map_err(|error| {
                format!("remove {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
            })
        } else {
            self.dir.remove_file(relative.as_path()).map_err(|error| {
                format!("remove {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
            })
        }
    }
}

/// Resolves symlinks and `..` through the longest existing ancestor of `path` (made absolute
/// against the current directory), then applies the remaining components lexically, so paths that
/// do not exist yet still compare like the files a run will create.
pub(crate) fn resolve_through_existing_ancestor(path: &Path) -> PathBuf {
    let path = std::path::absolute(path).unwrap_or_else(|_| path.to_path_buf());
    for ancestor in path.ancestors() {
        let (Ok(mut resolved), Ok(remainder)) = (std::fs::canonicalize(ancestor), path.strip_prefix(ancestor)) else {
            continue;
        };
        for component in remainder.components() {
            match component {
                Component::ParentDir => {
                    resolved.pop();
                }
                Component::Normal(name) => resolved.push(name),
                Component::CurDir | Component::RootDir | Component::Prefix(_) => {}
            }
        }
        return resolved;
    }
    path
}

/// Refuses an `--artifact-dir` that is `workspace_root` or one of its ancestors. Commands prune
/// their own earlier outputs from the artifact directory, and a `proofs/` or `summary.txt` in a
/// workspace is not one of them, so the bundle must live in a directory of its own.
pub(crate) fn reject_artifact_dir_enclosing_workspace(
    artifact_dir: &Path,
    workspace_root: &Path,
    bundle_kind: &str,
) -> Result<(), String> {
    if resolve_through_existing_ancestor(workspace_root).starts_with(resolve_through_existing_ancestor(artifact_dir)) {
        return Err(format!(
            "`--artifact-dir` {} is the workspace root {} or one of its ancestors; choose a directory that holds \
             only the {bundle_kind} bundle",
            artifact_dir.display(),
            workspace_root.display()
        ));
    }
    Ok(())
}

fn validate_relative_path(path: &Path) -> Result<RootRelativePath, String> {
    if path.as_os_str().is_empty() {
        return Err(format!("capability locator must not be {EMPTY_LOCATOR}"));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                return Err(format!("capability locator `{}` contains `{PARENT_COMPONENT}`", path.display()));
            }
            Component::RootDir => {
                return Err(format!("capability locator `{}` is {ABSOLUTE_COMPONENT}", path.display()));
            }
            Component::Prefix(_) => {
                return Err(format!("capability locator `{}` contains a Windows {PREFIX_COMPONENT}", path.display()));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        normalized.push(CURRENT_DIR);
    }
    Ok(RootRelativePath { inner: normalized })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ARTIFACT_ROOT: &str = "artifact-root";
    const CACHE_ROOT: &str = "cache-root";
    const PROOF_ROOT: &str = "proof-root";
    const SOURCE_ROOT: &str = "source-root";
    const COMMAND_FILE: &str = "command.txt";
    const NESTED_SOURCE_FILE: &str = "src/lib.rs";
    const PROOF_RECEIPT_FILE: &str = "proofs/example.receipt";
    const CACHE_FILE: &str = "findings.json";
    const VALID_TEXT: &str = "valid fixture";
    const CACHE_TEXT: &str = "[]\n";
    const TRAVERSAL_LOCATOR: &str = "../outside";
    const ABSOLUTE_LOCATOR: &str = "/tmp/outside";

    #[test]
    fn capability_paths_accept_normal_relative_locators() {
        let locator = RootRelativePath::new(NESTED_SOURCE_FILE).expect("relative locator");

        assert_eq!(locator.display(), NESTED_SOURCE_FILE);
    }

    #[test]
    fn capability_paths_reject_traversal_absolute_and_empty_locators() {
        assert!(RootRelativePath::new(TRAVERSAL_LOCATOR).is_err());
        assert!(RootRelativePath::new(ABSOLUTE_LOCATOR).is_err());
        assert!(RootRelativePath::new("").is_err());
    }

    #[test]
    fn artifact_root_reads_valid_files_under_declared_root() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let root_path = tempdir.path().join(ARTIFACT_ROOT);
        std::fs::create_dir_all(root_path.join("nested")).expect("create nested root");
        std::fs::write(root_path.join(COMMAND_FILE), VALID_TEXT).expect("write command");
        std::fs::write(root_path.join("nested/status.json"), VALID_TEXT).expect("write nested artifact");

        let root = ArtifactRoot::open_existing(&root_path).expect("artifact root");
        let files = root.read_files_recursive().expect("read files");

        assert_eq!(files.iter().map(|file| file.locator.as_str()).collect::<Vec<_>>(), vec![
            COMMAND_FILE,
            "nested/status.json"
        ]);
        assert!(files.iter().all(|file| file.contents == VALID_TEXT.as_bytes()));
    }

    #[test]
    fn cache_root_round_trips_declared_cache_file() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let cache = CacheRoot::open_or_create(&tempdir.path().join(CACHE_ROOT)).expect("cache root");
        let locator = RootRelativePath::new(CACHE_FILE).expect("cache locator");

        assert!(cache.read_to_string_optional(&locator).expect("missing cache").is_none());
        cache.write_atomic(&locator, b"stale").expect("write cache");
        cache.write_atomic(&locator, CACHE_TEXT.as_bytes()).expect("replace cache");

        assert_eq!(cache.read_to_string_optional(&locator).expect("read cache"), Some(CACHE_TEXT.to_owned()));
        let entries = std::fs::read_dir(tempdir.path().join(CACHE_ROOT))
            .expect("list cache root")
            .map(|entry| entry.expect("cache entry").file_name())
            .collect::<Vec<_>>();
        assert_eq!(entries, [std::ffi::OsString::from(CACHE_FILE)], "the staged write is renamed into place");
    }

    #[test]
    fn source_and_proof_roots_read_declared_relative_files() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let source_path = tempdir.path().join(SOURCE_ROOT);
        let proof_path = tempdir.path().join(PROOF_ROOT);
        std::fs::create_dir_all(source_path.join("src")).expect("create source root");
        std::fs::create_dir_all(proof_path.join("proofs")).expect("create proof root");
        std::fs::write(source_path.join(NESTED_SOURCE_FILE), VALID_TEXT).expect("write source");
        std::fs::write(proof_path.join(PROOF_RECEIPT_FILE), VALID_TEXT).expect("write proof");

        let source = SourceRoot::open_existing(&source_path).expect("source root");
        let proof = ProofRoot::open_existing(&proof_path).expect("proof root");

        assert_eq!(
            source
                .read_to_string(&RootRelativePath::new(NESTED_SOURCE_FILE).expect("source locator"))
                .expect("read source"),
            VALID_TEXT
        );
        assert_eq!(
            proof
                .read_to_string(&RootRelativePath::new(PROOF_RECEIPT_FILE).expect("proof locator"))
                .expect("read proof"),
            VALID_TEXT
        );
    }

    #[test]
    fn artifact_root_atomically_replaces_declared_file() {
        const REPLACEMENT_TEXT: &str = "replacement fixture";

        let tempdir = tempfile::tempdir().expect("tempdir");
        let root_path = tempdir.path().join(ARTIFACT_ROOT);
        let root = ArtifactRoot::open_or_create(&root_path).expect("artifact root");
        let locator = RootRelativePath::new(COMMAND_FILE).expect("command locator");

        root.write_atomic(&locator, VALID_TEXT.as_bytes()).expect("publish initial file");
        root.write_atomic(&locator, REPLACEMENT_TEXT.as_bytes()).expect("replace file");

        assert_eq!(
            std::fs::read_to_string(root_path.join(COMMAND_FILE)).expect("read replaced file"),
            REPLACEMENT_TEXT
        );
        let leftovers = std::fs::read_dir(&root_path)
            .expect("list artifact root")
            .map(|entry| entry.expect("artifact entry").file_name())
            .collect::<Vec<_>>();
        assert_eq!(leftovers, [std::ffi::OsString::from(COMMAND_FILE)], "no stage file remains");
    }

    #[test]
    fn concurrent_atomic_writers_all_publish_one_complete_payload() {
        const WRITERS: usize = 8;
        const ROUNDS: usize = 25;
        const PAYLOAD_BYTES: usize = 64 * 1024;

        let tempdir = tempfile::tempdir().expect("tempdir");
        let root_path = tempdir.path().join(CACHE_ROOT);
        let cache = CacheRoot::open_or_create(&root_path).expect("cache root");
        let locator = RootRelativePath::new(CACHE_FILE).expect("cache locator");
        let payloads = (0..WRITERS)
            .map(|writer| vec![b'a' + u8::try_from(writer).expect("writer index fits in u8"); PAYLOAD_BYTES])
            .collect::<Vec<_>>();
        let start = std::sync::Barrier::new(WRITERS);

        let results = std::thread::scope(|scope| {
            let handles = payloads
                .iter()
                .map(|payload| {
                    let (cache, locator, start) = (&cache, &locator, &start);
                    scope.spawn(move || {
                        start.wait();
                        (0..ROUNDS).map(|_| cache.write_atomic(locator, payload)).collect::<Vec<_>>()
                    })
                })
                .collect::<Vec<_>>();
            handles.into_iter().flat_map(|handle| handle.join().expect("writer thread")).collect::<Vec<_>>()
        });

        assert!(results.iter().all(Result::is_ok), "every concurrent writer succeeds: {results:?}");
        let published = std::fs::read(root_path.join(CACHE_FILE)).expect("read published file");
        assert!(payloads.contains(&published), "the file holds exactly one writer's complete payload");
        let entries = std::fs::read_dir(&root_path)
            .expect("list cache root")
            .map(|entry| entry.expect("cache entry").file_name())
            .collect::<Vec<_>>();
        assert_eq!(entries, [std::ffi::OsString::from(CACHE_FILE)], "no stage file remains");
    }

    #[cfg(unix)]
    #[test]
    fn artifact_root_rejects_symlink_entries_before_reading_targets() {
        use std::os::unix::fs::symlink;

        let tempdir = tempfile::tempdir().expect("tempdir");
        let root_path = tempdir.path().join(ARTIFACT_ROOT);
        let outside_path = tempdir.path().join("outside.txt");
        std::fs::create_dir_all(&root_path).expect("create root");
        std::fs::write(&outside_path, VALID_TEXT).expect("write outside");
        symlink(&outside_path, root_path.join("escape-link")).expect("create symlink");

        let root = ArtifactRoot::open_existing(&root_path).expect("artifact root");
        let error = root.read_files_recursive().expect_err("symlink rejected");

        assert!(error.contains("symlink"));
        assert!(error.contains("escape-link"));
    }

    #[cfg(unix)]
    #[test]
    fn capability_reads_reject_fifo_without_blocking() {
        use std::time::Duration;
        use std::time::Instant;

        const ROOT_ENV: &str = "OCTET_CAP_FS_FIFO_TEST_ROOT";
        const FIFO_FILE: &str = "blocked.fifo";
        const READ_LIMIT: u64 = 1024;
        if let Some(root_path) = std::env::var_os(ROOT_ENV) {
            let root_path = Path::new(&root_path);
            let locator = RootRelativePath::new(FIFO_FILE).expect("FIFO locator");
            let source = SourceRoot::open_existing(root_path).expect("source root");
            let cache = CacheRoot::open_or_create(root_path).expect("cache root");
            let errors = [
                source.read_bounded(&locator, READ_LIMIT).expect_err("bounded read rejects FIFO"),
                source.read_to_string(&locator).expect_err("string read rejects FIFO"),
                cache.read_to_string_optional(&locator).expect_err("optional read rejects FIFO"),
            ];
            for error in errors {
                assert!(error.contains("not a regular file"), "{error}");
            }
            return;
        }
        let tempdir = tempfile::tempdir().expect("tempdir");
        let fifo = tempdir.path().join(FIFO_FILE);
        assert!(
            std::process::Command::new("mkfifo").arg(&fifo).status().expect("create FIFO").success(),
            "mkfifo must create the test input"
        );
        let mut child = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "cap_fs::tests::capability_reads_reject_fifo_without_blocking",
                "--nocapture",
            ])
            .env(ROOT_ENV, tempdir.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .expect("run isolated FIFO read");
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(status) = child.try_wait().expect("wait for FIFO read") {
                assert!(status.success(), "isolated FIFO read failed: {status}");
                break;
            }
            if Instant::now() >= deadline {
                child.kill().expect("stop blocked FIFO read");
                child.wait().expect("reap blocked FIFO read");
                panic!("capability read blocked on a FIFO");
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }

    #[cfg(unix)]
    #[test]
    fn path_resolution_sees_through_symlinks_and_parent_components_of_missing_paths() {
        let temporary = tempfile::tempdir().expect("resolution fixture");
        let root = std::fs::canonicalize(temporary.path()).expect("canonical fixture root");
        let artifacts = root.join("artifacts");
        std::fs::create_dir(&artifacts).expect("artifact dir");
        std::os::unix::fs::symlink(&artifacts, root.join("linked")).expect("artifact symlink");

        assert_eq!(
            resolve_through_existing_ancestor(&root.join("linked/missing/../baseline.json")),
            artifacts.join("baseline.json")
        );
        assert_eq!(resolve_through_existing_ancestor(&root.join("absent/./nested/..")), root.join("absent"));
        assert!(!resolve_through_existing_ancestor(&root.join("artifacts-baseline.json")).starts_with(&artifacts));
    }
}
