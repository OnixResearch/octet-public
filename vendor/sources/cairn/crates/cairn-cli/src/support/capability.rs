use std::{
    ffi::OsString,
    fs, io,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

// r[impl cairn.release_evidence.cap_std_boundary.dependency]
use cap_std::{
    ambient_authority,
    fs::{Dir, FileType},
};

// r[verify cairn.release_evidence.cap_std_boundary.docs]
pub(crate) const CAPABILITY_PATH_BOUNDARY: &str =
    "capability roots constrain local filesystem authority only; they do not prove artifact truth, replay completeness, lifecycle correctness, or release readiness";

// r[impl cairn.release_evidence.cap_std_boundary.root_wrappers]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CapabilityRootKind {
    Repository,
    EvidenceBundle,
    LocalState,
    Registry,
    Release,
    Replay,
    Scan,
}

pub(crate) struct CapabilityRoot {
    kind: CapabilityRootKind,
    display_path: PathBuf,
    dir: Dir,
}

pub(crate) struct CapabilityDirEntry {
    name: OsString,
    file_type: FileType,
}

impl CapabilityRoot {
    pub(crate) fn open(kind: CapabilityRootKind, path: &Path) -> io::Result<Self> {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() {
            return Err(capability_path_error());
        }
        let dir = Dir::open_ambient_dir(path, ambient_authority())?;
        Ok(Self {
            kind,
            display_path: path.to_path_buf(),
            dir,
        })
    }

    pub(crate) fn open_or_create(kind: CapabilityRootKind, path: &Path) -> io::Result<Self> {
        fs::create_dir_all(path)?;
        Self::open(kind, path)
    }

    pub(crate) fn open_file_parent(
        kind: CapabilityRootKind,
        path: &Path,
        create: bool,
    ) -> io::Result<(Self, PathBuf)> {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let file_name = path
            .file_name()
            .ok_or_else(capability_path_error)
            .map(PathBuf::from)?;
        let root = if create {
            Self::open_or_create(kind, parent)?
        } else {
            Self::open(kind, parent)?
        };
        debug_assert_eq!(root.kind(), kind);
        Ok((root, file_name))
    }

    pub(crate) fn kind(&self) -> CapabilityRootKind {
        self.kind
    }

    pub(crate) fn path(&self) -> &Path {
        &self.display_path
    }

    pub(crate) fn relative(&self, path: &Path) -> io::Result<PathBuf> {
        validate_relative_path(path)
    }

    pub(crate) fn display_path(&self, relative: &Path) -> io::Result<PathBuf> {
        let relative = self.relative(relative)?;
        Ok(self.display_path.join(relative))
    }

    // r[impl cairn.release_evidence.cap_std_boundary.conversion]
    pub(crate) fn read(&self, relative: &Path) -> io::Result<Vec<u8>> {
        let relative = self.relative(relative)?;
        self.dir.read(relative)
    }

    pub(crate) fn read_bounded(&self, relative: &Path, max_bytes: u64) -> io::Result<Vec<u8>> {
        let relative = self.relative(relative)?;
        let metadata = self.dir.symlink_metadata(&relative)?;
        if metadata.file_type().is_symlink() {
            return Err(capability_path_error());
        }
        if metadata.len() > max_bytes {
            return Err(bounded_read_limit_error());
        }
        let read_limit = max_bytes.saturating_add(1);
        let mut reader = self.dir.open(&relative)?.take(read_limit);
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes)?;
        let byte_count = u64::try_from(bytes.len()).map_err(|_| bounded_read_limit_error())?;
        if byte_count > max_bytes {
            return Err(bounded_read_limit_error());
        }
        Ok(bytes)
    }

    pub(crate) fn read_to_string(&self, relative: &Path) -> io::Result<String> {
        let relative = self.relative(relative)?;
        self.dir.read_to_string(relative)
    }

    pub(crate) fn write(&self, relative: &Path, contents: &[u8]) -> io::Result<()> {
        let relative = self.relative(relative)?;
        self.dir.write(relative, contents)
    }

    pub(crate) fn write_new(&self, relative: &Path, contents: &[u8]) -> io::Result<()> {
        let relative = self.relative(relative)?;
        let mut options = cap_std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        let mut file = self.dir.open_with(relative, &options)?;
        file.write_all(contents)?;
        file.sync_all()
    }

    pub(crate) fn create_dir(&self, relative: &Path) -> io::Result<()> {
        let relative = self.relative(relative)?;
        self.dir.create_dir(relative)
    }

    pub(crate) fn create_dir_all(&self, relative: &Path) -> io::Result<()> {
        let relative = self.relative(relative)?;
        self.dir.create_dir_all(relative)
    }

    pub(crate) fn metadata(&self, relative: &Path) -> io::Result<cap_std::fs::Metadata> {
        let relative = self.relative(relative)?;
        self.dir.metadata(relative)
    }

    pub(crate) fn symlink_metadata(&self, relative: &Path) -> io::Result<cap_std::fs::Metadata> {
        let relative = self.relative(relative)?;
        self.dir.symlink_metadata(relative)
    }

    pub(crate) fn remove_dir_all(&self, relative: &Path) -> io::Result<()> {
        let relative = self.relative(relative)?;
        self.dir.remove_dir_all(relative)
    }

    pub(crate) fn remove_file(&self, relative: &Path) -> io::Result<()> {
        let relative = self.relative(relative)?;
        self.dir.remove_file(relative)
    }

    pub(crate) fn rename(
        &self,
        source_relative: &Path,
        destination: &Self,
        destination_relative: &Path,
    ) -> io::Result<()> {
        let source_relative = self.relative(source_relative)?;
        let destination_relative = destination.relative(destination_relative)?;
        self.dir
            .rename(source_relative, &destination.dir, destination_relative)
    }

    pub(crate) fn copy_to(
        &self,
        source_relative: &Path,
        destination: &Self,
        destination_relative: &Path,
    ) -> io::Result<u64> {
        let source_relative = self.relative(source_relative)?;
        let destination_relative = destination.relative(destination_relative)?;
        self.dir
            .copy(source_relative, &destination.dir, destination_relative)
    }

    pub(crate) fn entries(&self, relative: &Path) -> io::Result<Vec<CapabilityDirEntry>> {
        self.entries_inner(relative, None)
    }

    pub(crate) fn entries_bounded(
        &self,
        relative: &Path,
        max_entries: usize,
    ) -> io::Result<Vec<CapabilityDirEntry>> {
        self.entries_inner(relative, Some(max_entries))
    }

    pub(crate) fn read_path(&self, path: &Path) -> io::Result<Vec<u8>> {
        let relative = self.relative_from_display_path(path)?;
        self.read(&relative)
    }

    pub(crate) fn read_bounded_path(&self, path: &Path, max_bytes: u64) -> io::Result<Vec<u8>> {
        let relative = self.relative_from_display_path(path)?;
        self.read_bounded(&relative, max_bytes)
    }

    pub(crate) fn read_to_string_path(&self, path: &Path) -> io::Result<String> {
        let relative = self.relative_from_display_path(path)?;
        self.read_to_string(&relative)
    }

    pub(crate) fn write_path(&self, path: &Path, contents: &[u8]) -> io::Result<()> {
        let relative = self.relative_from_display_path(path)?;
        self.write(&relative, contents)
    }

    pub(crate) fn create_dir_all_path(&self, path: &Path) -> io::Result<()> {
        let relative = self.relative_from_display_path(path)?;
        self.create_dir_all(&relative)
    }

    pub(crate) fn metadata_path(&self, path: &Path) -> io::Result<cap_std::fs::Metadata> {
        let relative = self.relative_from_display_path(path)?;
        self.metadata(&relative)
    }

    pub(crate) fn symlink_metadata_path(&self, path: &Path) -> io::Result<cap_std::fs::Metadata> {
        let relative = self.relative_from_display_path(path)?;
        self.symlink_metadata(&relative)
    }

    pub(crate) fn copy_path_to(
        &self,
        source_path: &Path,
        destination: &Self,
        destination_path: &Path,
    ) -> io::Result<u64> {
        let source_relative = self.relative_from_display_path(source_path)?;
        let destination_relative = destination.relative_from_display_path(destination_path)?;
        self.copy_to(&source_relative, destination, &destination_relative)
    }

    fn entries_inner(
        &self,
        relative: &Path,
        max_entries: Option<usize>,
    ) -> io::Result<Vec<CapabilityDirEntry>> {
        let relative = self.relative(relative)?;
        let directory = if relative == Path::new(".") {
            self.dir.try_clone()?
        } else {
            self.dir.open_dir(&relative)?
        };
        let mut entries = Vec::new();
        for entry in directory.entries()? {
            if max_entries.is_some_and(|limit| entries.len() >= limit) {
                return Err(bounded_entry_limit_error());
            }
            let entry = entry?;
            let name = entry.file_name();
            let file_type = entry.file_type()?;
            entries.push(CapabilityDirEntry { name, file_type });
        }
        Ok(entries)
    }

    fn relative_from_display_path(&self, path: &Path) -> io::Result<PathBuf> {
        let relative = if path == self.display_path {
            PathBuf::from(".")
        } else if let Ok(relative) = path.strip_prefix(&self.display_path) {
            relative.to_path_buf()
        } else if self.display_path == Path::new(".") && !path.is_absolute() {
            path.to_path_buf()
        } else {
            return Err(capability_path_error());
        };
        self.relative(&relative)
    }
}

impl CapabilityDirEntry {
    pub(crate) fn file_name(&self) -> &OsString {
        &self.name
    }

    pub(crate) fn file_type(&self) -> &FileType {
        &self.file_type
    }
}

fn validate_relative_path(path: &Path) -> io::Result<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(capability_path_error());
    }
    let mut relative = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::Normal(segment) => relative.push(segment),
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(capability_path_error())
            }
        }
    }
    if relative.as_os_str().is_empty() {
        relative.push(".");
    }
    Ok(relative)
}

fn bounded_read_limit_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "file exceeds the bounded read limit",
    )
}

fn bounded_entry_limit_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidData,
        "directory exceeds the bounded entry limit",
    )
}

fn capability_path_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::PermissionDenied,
        format!("path is outside the declared capability root; {CAPABILITY_PATH_BOUNDARY}"),
    )
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    use super::{CapabilityRoot, CapabilityRootKind};

    const TEST_DIRECTORY_PREFIX: &str = "cairn-capability-root";
    const TEST_FILE_NAME: &str = "inside.txt";
    const TEST_FILE_CONTENT: &[u8] = b"inside\n";

    fn test_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{TEST_DIRECTORY_PREFIX}-{name}-{}",
            std::process::id()
        ))
    }

    fn remove_test_directory(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }

    #[test]
    fn valid_relative_file_stays_inside_root() -> Result<(), Box<dyn std::error::Error>> {
        // r[verify cairn.release_evidence.cap_std_boundary.tests.positive]
        let directory = test_directory("positive");
        remove_test_directory(&directory)?;
        fs::create_dir_all(&directory)?;
        let root = CapabilityRoot::open(CapabilityRootKind::EvidenceBundle, &directory)?;

        root.write(Path::new(TEST_FILE_NAME), TEST_FILE_CONTENT)?;
        assert_eq!(root.read(Path::new(TEST_FILE_NAME))?, TEST_FILE_CONTENT);
        assert_eq!(root.kind(), CapabilityRootKind::EvidenceBundle);

        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn bounded_reads_accept_the_limit_and_reject_larger_files(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("bounded-read");
        remove_test_directory(&directory)?;
        fs::create_dir_all(&directory)?;
        let root = CapabilityRoot::open(CapabilityRootKind::Repository, &directory)?;
        let max_bytes = u64::try_from(TEST_FILE_CONTENT.len())?;

        root.write(Path::new(TEST_FILE_NAME), TEST_FILE_CONTENT)?;
        assert_eq!(
            root.read_bounded(Path::new(TEST_FILE_NAME), max_bytes)?,
            TEST_FILE_CONTENT
        );
        root.write(Path::new(TEST_FILE_NAME), b"inside\nextra")?;
        assert!(root
            .read_bounded(Path::new(TEST_FILE_NAME), max_bytes)
            .is_err());

        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn bounded_entries_and_exclusive_directory_creation_fail_closed(
    ) -> Result<(), Box<dyn std::error::Error>> {
        const ENTRY_LIMIT: usize = 1;
        const PREPARED_DIRECTORY: &str = "prepared";
        const SECOND_FILE_NAME: &str = "second.txt";

        let directory = test_directory("bounded-entries");
        remove_test_directory(&directory)?;
        fs::create_dir_all(&directory)?;
        let root = CapabilityRoot::open(CapabilityRootKind::Repository, &directory)?;
        root.write(Path::new(TEST_FILE_NAME), TEST_FILE_CONTENT)?;
        assert_eq!(
            root.entries_bounded(Path::new("."), ENTRY_LIMIT)?.len(),
            ENTRY_LIMIT
        );
        root.write(Path::new(SECOND_FILE_NAME), TEST_FILE_CONTENT)?;
        assert!(root.entries_bounded(Path::new("."), ENTRY_LIMIT).is_err());

        root.create_dir(Path::new(PREPARED_DIRECTORY))?;
        assert!(root.create_dir(Path::new(PREPARED_DIRECTORY)).is_err());

        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn missing_root_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
        // r[verify cairn.release_evidence.cap_std_boundary.tests.negative]
        // r[verify cairn.release_evidence.cap_std_boundary.validation]
        let directory = test_directory("missing-root");
        remove_test_directory(&directory)?;

        let result = CapabilityRoot::open(CapabilityRootKind::Release, &directory);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn traversal_and_absolute_paths_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
        let directory = test_directory("negative-path");
        remove_test_directory(&directory)?;
        fs::create_dir_all(&directory)?;
        let root = CapabilityRoot::open(CapabilityRootKind::Registry, &directory)?;

        assert!(root.read(Path::new("../outside.txt")).is_err());
        assert!(root.read(Path::new("/outside.txt")).is_err());

        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn symlink_escape_is_rejected() -> Result<(), Box<dyn std::error::Error>> {
        use std::os::unix::fs::symlink;

        let directory = test_directory("negative-symlink");
        let outside = test_directory("negative-symlink-outside");
        remove_test_directory(&directory)?;
        remove_test_directory(&outside)?;
        fs::create_dir_all(&directory)?;
        fs::create_dir_all(&outside)?;
        fs::write(outside.join(TEST_FILE_NAME), TEST_FILE_CONTENT)?;
        symlink(&outside, directory.join("escape"))?;
        let root = CapabilityRoot::open(CapabilityRootKind::Scan, &directory)?;

        assert!(root.read(Path::new("escape/inside.txt")).is_err());

        fs::remove_dir_all(directory)?;
        fs::remove_dir_all(outside)?;
        Ok(())
    }
}
