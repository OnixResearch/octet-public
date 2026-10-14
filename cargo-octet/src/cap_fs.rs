use std::ffi::OsStr;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use cap_std::ambient_authority;
use cap_std::fs::Dir;

const CURRENT_DIR: &str = ".";
const PARENT_COMPONENT: &str = "..";
const ABSOLUTE_COMPONENT: &str = "absolute";
const PREFIX_COMPONENT: &str = "prefix";
const EMPTY_LOCATOR: &str = "empty";

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

    pub(crate) fn create_dir_all(&self, relative: &RootRelativePath) -> Result<(), String> {
        self.root.create_dir_all(relative)
    }

    pub(crate) fn write(&self, relative: &RootRelativePath, bytes: &[u8]) -> Result<(), String> {
        self.root.write(relative, bytes)
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

    pub(crate) fn write(&self, relative: &RootRelativePath, bytes: &[u8]) -> Result<(), String> {
        self.root.write(relative, bytes)
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
        self.dir.read(relative.as_path()).map_err(|error| {
            format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
        })
    }

    fn read_to_string(&self, relative: &RootRelativePath) -> Result<String, String> {
        self.dir.read_to_string(relative.as_path()).map_err(|error| {
            format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display())
        })
    }

    fn read_to_string_optional(&self, relative: &RootRelativePath) -> Result<Option<String>, String> {
        match self.dir.read_to_string(relative.as_path()) {
            Ok(contents) => Ok(Some(contents)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => {
                Err(format!("read {kind} path `{locator}`: {error}", kind = self.kind, locator = relative.display()))
            }
        }
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
        cache.write(&locator, CACHE_TEXT.as_bytes()).expect("write cache");

        assert_eq!(cache.read_to_string_optional(&locator).expect("read cache"), Some(CACHE_TEXT.to_owned()));
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
}
