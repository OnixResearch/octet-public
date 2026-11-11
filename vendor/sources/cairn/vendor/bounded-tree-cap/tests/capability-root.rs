#![cfg(unix)]

pub use std::fs;
pub use std::os::unix::fs::symlink;

pub use bounded_tree_cap::{ShellError, execute, prepare};
pub use bounded_tree_core::{LimitValues, SymlinkPolicy, TreeLimits, TreeLimitsError};
pub use cap_std::ambient_authority;
pub use cap_std::fs::Dir;

const ENTRIES_MAX: u64 = 16;
const DEPTH_MAX: u64 = 8;
const PATH_BYTES_MAX: u64 = 256;
const FILE_BYTES_MAX: u64 = 1_024;
const TOTAL_BYTES_MAX: u64 = 4_096;
const TARGET_BYTES_MAX: u64 = 128;
const SOURCE_FILE_BYTES: &[u8] = b"bounded-tree-content";
const CHANGED_FILE_BYTES: &[u8] = b"changed-tree-content";

#[derive(Debug)]
enum TestFailure {
    Io,
    Shell,
    Limits,
}

impl From<std::io::Error> for TestFailure {
    fn from(_: std::io::Error) -> Self {
        Self::Io
    }
}

impl From<ShellError> for TestFailure {
    fn from(_: ShellError) -> Self {
        Self::Shell
    }
}

impl From<TreeLimitsError> for TestFailure {
    fn from(_: TreeLimitsError) -> Self {
        Self::Limits
    }
}

#[test]
fn ambient_root_replacement_does_not_replace_the_opened_capability() -> Result<(), TestFailure> {
    let temp = tempfile::tempdir()?;
    let source_path = temp.path().join("source");
    let moved_source_path = temp.path().join("moved-source");
    let external_path = temp.path().join("external");
    let destination_path = temp.path().join("destination");
    fs::create_dir(&source_path)?;
    fs::write(source_path.join("data"), SOURCE_FILE_BYTES)?;
    fs::create_dir(&external_path)?;
    fs::write(external_path.join("data"), CHANGED_FILE_BYTES)?;
    fs::create_dir(&destination_path)?;
    let source = Dir::open_ambient_dir(&source_path, ambient_authority())?;
    let destination = Dir::open_ambient_dir(&destination_path, ambient_authority())?;
    let policy = TreeLimits::new(LimitValues {
        entries: ENTRIES_MAX,
        depth: DEPTH_MAX,
        path_bytes: PATH_BYTES_MAX,
        file_bytes: FILE_BYTES_MAX,
        total_bytes: TOTAL_BYTES_MAX,
        symlink_target_bytes: TARGET_BYTES_MAX,
    })?;
    let prepared = prepare(&source, policy, SymlinkPolicy::Reject)?;
    fs::rename(&source_path, &moved_source_path)?;
    symlink(&external_path, &source_path)?;

    execute(&prepared, &destination)?;

    assert!(moved_source_path.exists());
    assert_eq!(fs::read(destination_path.join("data"))?, SOURCE_FILE_BYTES);
    Ok(())
}
