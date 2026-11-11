#![cfg(unix)]

pub use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::PermissionsExt;
pub use std::os::unix::fs::symlink;

pub use bounded_tree_cap::{ShellError, ShellErrorKind, SourceChangeKind, execute, prepare};
pub use bounded_tree_core::{LimitKind, LimitValues, SymlinkPolicy, TreeLimits, TreeLimitsError};
pub use cap_std::ambient_authority;
pub use cap_std::fs::Dir;

const ENTRIES_MAX: u64 = 32;
const DEPTH_MAX: u64 = 8;
const PATH_BYTES_MAX: u64 = 512;
const FILE_BYTES_MAX: u64 = 4_096;
const TOTAL_BYTES_MAX: u64 = 8_192;
const TARGET_BYTES_MAX: u64 = 256;
const SOURCE_FILE_BYTES: &[u8] = b"bounded-tree-content";
const CHANGED_FILE_BYTES: &[u8] = b"changed-tree-content";
const FILE_MODE: u32 = 0o640;
const CHANGED_FILE_MODE: u32 = 0o600;
const EXPECTED_ENTRIES: u64 = 5;
const EXPECTED_FILE_BYTES: u64 = 60;
const TIGHT_BOUND: u64 = 1;

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

type TestResult = Result<(), TestFailure>;

fn limit_values() -> LimitValues {
    LimitValues {
        entries: ENTRIES_MAX,
        depth: DEPTH_MAX,
        path_bytes: PATH_BYTES_MAX,
        file_bytes: FILE_BYTES_MAX,
        total_bytes: TOTAL_BYTES_MAX,
        symlink_target_bytes: TARGET_BYTES_MAX,
    }
}

fn limits() -> Result<TreeLimits, TreeLimitsError> {
    TreeLimits::new(limit_values())
}

fn open_dir(path: &std::path::Path) -> Result<Dir, std::io::Error> {
    Dir::open_ambient_dir(path, ambient_authority())
}

fn write_source_tree(root: &std::path::Path) -> Result<(), std::io::Error> {
    fs::create_dir_all(root.join("nested"))?;
    fs::write(root.join("nested/data"), SOURCE_FILE_BYTES)?;
    fs::write(root.join("other"), SOURCE_FILE_BYTES)?;
    fs::set_permissions(
        root.join("nested/data"),
        fs::Permissions::from_mode(FILE_MODE),
    )?;
    symlink("nested/data", root.join("latest"))?;
    Ok(())
}

#[test]
fn capability_copy_preserves_files_links_modes_and_non_utf8_names() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_path = temp.path().join("source");
    let destination_path = temp.path().join("destination");
    write_source_tree(&source_path)?;
    let non_utf8_name = std::ffi::OsString::from_vec(vec![b'n', b'o', b'n', b'-', 0xFF]);
    fs::write(source_path.join(&non_utf8_name), SOURCE_FILE_BYTES)?;
    fs::create_dir(&destination_path)?;
    let source = open_dir(&source_path)?;
    let destination = open_dir(&destination_path)?;

    let prepared = prepare(&source, limits()?, SymlinkPolicy::PreserveInternal)?;
    let outcome = execute(&prepared, &destination)?;

    assert_eq!(outcome.entries_copied(), EXPECTED_ENTRIES);
    assert_eq!(outcome.file_bytes_copied(), EXPECTED_FILE_BYTES);
    assert_eq!(
        fs::read(destination_path.join("nested/data"))?,
        SOURCE_FILE_BYTES
    );
    assert_eq!(
        fs::read_link(destination_path.join("latest"))?,
        std::path::Path::new("nested/data")
    );
    assert_eq!(
        fs::read(destination_path.join(non_utf8_name))?,
        SOURCE_FILE_BYTES
    );
    assert_eq!(
        fs::metadata(destination_path.join("nested/data"))?
            .permissions()
            .mode()
            & 0o777,
        FILE_MODE
    );
    Ok(())
}

#[test]
fn source_changes_fail_before_destination_copy() -> TestResult {
    let cases = [
        SourceMutation::Content,
        SourceMutation::Growth,
        SourceMutation::Truncation,
        SourceMutation::Mode,
        SourceMutation::Kind,
        SourceMutation::Target,
    ];
    for mutation in cases {
        let temp = tempfile::tempdir()?;
        let source_path = temp.path().join("source");
        let destination_path = temp.path().join("destination");
        write_source_tree(&source_path)?;
        fs::create_dir(&destination_path)?;
        let source = open_dir(&source_path)?;
        let destination = open_dir(&destination_path)?;
        let prepared = prepare(&source, limits()?, SymlinkPolicy::PreserveInternal)?;
        mutation.apply(&source_path)?;

        let error = execute(&prepared, &destination).expect_err("changed source must fail");

        assert!(matches!(error.kind(), ShellErrorKind::SourceChanged(_)));
        assert!(fs::read_dir(&destination_path)?.next().is_none());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceMutation {
    Content,
    Growth,
    Truncation,
    Mode,
    Kind,
    Target,
}

impl SourceMutation {
    fn apply(self, source: &std::path::Path) -> Result<(), std::io::Error> {
        if self == Self::Content {
            fs::write(source.join("other"), CHANGED_FILE_BYTES)?;
            return Ok(());
        }
        if self == Self::Growth {
            fs::write(
                source.join("other"),
                [SOURCE_FILE_BYTES, b"growth"].concat(),
            )?;
            return Ok(());
        }
        if self == Self::Truncation {
            fs::write(source.join("other"), b"short")?;
            return Ok(());
        }
        if self == Self::Mode {
            fs::set_permissions(
                source.join("nested/data"),
                fs::Permissions::from_mode(CHANGED_FILE_MODE),
            )?;
            return Ok(());
        }
        if self == Self::Kind {
            fs::remove_file(source.join("other"))?;
            symlink("nested/data", source.join("other"))?;
            return Ok(());
        }
        fs::remove_file(source.join("latest"))?;
        symlink("other", source.join("latest"))?;
        Ok(())
    }
}

#[test]
fn malformed_links_special_files_and_limits_fail_during_prepare() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_path = temp.path().join("source");
    fs::create_dir(&source_path)?;
    symlink("../escape", source_path.join("escape"))?;
    let source = open_dir(&source_path)?;
    let link_error = prepare(&source, limits()?, SymlinkPolicy::PreserveInternal)
        .expect_err("escaping link must fail");
    assert!(matches!(link_error.kind(), ShellErrorKind::PlanRejected(_)));

    fs::remove_file(source_path.join("escape"))?;
    let _listener = std::os::unix::net::UnixListener::bind(source_path.join("socket"))?;
    let special_error =
        prepare(&source, limits()?, SymlinkPolicy::Reject).expect_err("special file must fail");
    assert!(matches!(
        special_error.kind(),
        ShellErrorKind::PlanRejected(_)
    ));

    fs::remove_file(source_path.join("socket"))?;
    fs::write(source_path.join("large"), SOURCE_FILE_BYTES)?;
    let file_policy = TreeLimits::new(LimitValues {
        file_bytes: TIGHT_BOUND,
        ..limit_values()
    })?;
    let file_error =
        prepare(&source, file_policy, SymlinkPolicy::Reject).expect_err("file bound must fail");
    assert!(matches!(
        file_error.kind(),
        ShellErrorKind::LimitExceeded(LimitKind::FileBytes)
    ));

    let total_policy = TreeLimits::new(LimitValues {
        total_bytes: TIGHT_BOUND,
        ..limit_values()
    })?;
    let total_error =
        prepare(&source, total_policy, SymlinkPolicy::Reject).expect_err("total bound must fail");
    assert!(matches!(
        total_error.kind(),
        ShellErrorKind::LimitExceeded(LimitKind::TotalBytes)
    ));

    let entry_policy = TreeLimits::new(LimitValues {
        entries: TIGHT_BOUND,
        ..limit_values()
    })?;
    fs::write(source_path.join("second"), SOURCE_FILE_BYTES)?;
    let entry_error =
        prepare(&source, entry_policy, SymlinkPolicy::Reject).expect_err("entry bound must fail");
    assert!(matches!(
        entry_error.kind(),
        ShellErrorKind::LimitExceeded(LimitKind::Entries)
    ));
    Ok(())
}

#[test]
fn non_empty_destination_and_missing_source_fail_without_unrelated_writes() -> TestResult {
    let temp = tempfile::tempdir()?;
    let source_path = temp.path().join("source");
    let destination_path = temp.path().join("destination");
    write_source_tree(&source_path)?;
    fs::create_dir(&destination_path)?;
    fs::write(destination_path.join("sentinel"), b"sentinel")?;
    let source = open_dir(&source_path)?;
    let destination = open_dir(&destination_path)?;
    let prepared = prepare(&source, limits()?, SymlinkPolicy::PreserveInternal)?;

    let destination_error =
        execute(&prepared, &destination).expect_err("non-empty destination must fail");
    assert_eq!(
        destination_error.kind(),
        &ShellErrorKind::DestinationNotEmpty
    );
    assert_eq!(fs::read(destination_path.join("sentinel"))?, b"sentinel");

    fs::remove_file(destination_path.join("sentinel"))?;
    fs::remove_file(source_path.join("other"))?;
    let source_error = execute(&prepared, &destination).expect_err("missing source must fail");
    assert_eq!(
        source_error.kind(),
        &ShellErrorKind::SourceChanged(SourceChangeKind::Missing)
    );
    assert!(fs::read_dir(destination_path)?.next().is_none());
    Ok(())
}
