pub use bounded_tree_core::{
    Blake3Digest, BlockerKind, CopyOperation, EntryKind, FileMode, LimitValues, PlanError,
    RawObservation, RelativePath, SymlinkPolicy, TreeLimits, TreeLimitsError, TreeRequest,
    plan_tree,
};

const ENTRIES_MAX: u64 = 16;
const DEPTH_MAX: u64 = 8;
const PATH_BYTES_MAX: u64 = 256;
const FILE_BYTES_MAX: u64 = 1_024;
const TOTAL_BYTES_MAX: u64 = 4_096;
const TARGET_BYTES_MAX: u64 = 128;
const FILE_BYTES: u64 = 4;
const DIRECTORY_MODE: u32 = 0o040_755;
const FILE_MODE: u32 = 0o100_644;
const SYMLINK_MODE: u32 = 0o120_777;
const DIGEST_BYTE: u8 = 0xA5;
const EXPECTED_MEMBER_COUNT: usize = 4;
const EXPECTED_SYMLINK_OPERATION_INDEX: usize = 3;

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

#[derive(Debug)]
struct TestError;

impl From<TreeLimitsError> for TestError {
    fn from(_: TreeLimitsError) -> Self {
        Self
    }
}

impl From<PlanError> for TestError {
    fn from(_: PlanError) -> Self {
        Self
    }
}

fn digest() -> Blake3Digest {
    Blake3Digest::from_bytes([DIGEST_BYTE; Blake3Digest::BYTE_COUNT])
}

fn directory(path_components: &[&[u8]]) -> RawObservation {
    RawObservation {
        path_components: owned_components(path_components),
        kind: EntryKind::Directory,
        mode: FileMode::new(DIRECTORY_MODE),
        file_bytes: 0,
        symlink_target: None,
        file_content: None,
    }
}

fn file(path_components: &[&[u8]], file_bytes: u64) -> RawObservation {
    RawObservation {
        path_components: owned_components(path_components),
        kind: EntryKind::File,
        mode: FileMode::new(FILE_MODE),
        file_bytes,
        symlink_target: None,
        file_content: Some(digest()),
    }
}

fn symlink(path_components: &[&[u8]], target: &[u8]) -> RawObservation {
    RawObservation {
        path_components: owned_components(path_components),
        kind: EntryKind::Symlink,
        mode: FileMode::new(SYMLINK_MODE),
        file_bytes: 0,
        symlink_target: Some(target.to_vec()),
        file_content: None,
    }
}

fn owned_components(components: &[&[u8]]) -> Vec<Vec<u8>> {
    components
        .iter()
        .map(|component| component.to_vec())
        .collect()
}

fn request(
    observations: Vec<RawObservation>,
    symlink_policy: SymlinkPolicy,
) -> Result<TreeRequest, TreeLimitsError> {
    Ok(TreeRequest {
        observations,
        limits: limits()?,
        symlink_policy,
    })
}

#[test]
fn valid_tree_is_deterministic_and_preserves_exact_member_facts() -> Result<(), TestError> {
    let observations = vec![
        symlink(&[b"latest"], b"nested/data"),
        file(&[b"nested", b"data"], FILE_BYTES),
        directory(&[b"nested"]),
        file(&[b"non-utf8-\xFF"], FILE_BYTES),
    ];
    let mut reversed = observations.clone();
    reversed.reverse();

    let first = plan_tree(request(observations, SymlinkPolicy::PreserveInternal)?)?;
    let second = plan_tree(request(reversed, SymlinkPolicy::PreserveInternal)?)?;

    assert_eq!(first, second);
    assert_eq!(first.member_facts().len(), EXPECTED_MEMBER_COUNT);
    assert_eq!(first.member_facts()[0].path().components()[0], b"latest");
    let expected_target = vec![b"nested".to_vec(), b"data".to_vec()];
    assert_eq!(
        first.member_facts()[0]
            .resolved_symlink_target()
            .map(RelativePath::components),
        Some(expected_target.as_slice())
    );
    assert!(matches!(
        first.operations()[0],
        CopyOperation::CreateDirectory { .. }
    ));
    assert!(matches!(
        first.operations()[EXPECTED_SYMLINK_OPERATION_INDEX],
        CopyOperation::CreateSymlink { .. }
    ));
    Ok(())
}

#[test]
fn malformed_paths_and_shapes_fail_with_typed_blockers() -> Result<(), TreeLimitsError> {
    let malformed = [
        (Vec::new(), BlockerKind::EmptyPath),
        (vec![Vec::new()], BlockerKind::EmptyComponent),
        (vec![b".".to_vec()], BlockerKind::CurrentDirectoryComponent),
        (vec![b"..".to_vec()], BlockerKind::ParentDirectoryComponent),
        (vec![b"a\0b".to_vec()], BlockerKind::PathContainsNul),
        (vec![b"a/b".to_vec()], BlockerKind::PathContainsForwardSlash),
        (vec![b"a\\b".to_vec()], BlockerKind::PathContainsBackslash),
    ];

    for (path_components, expected) in malformed {
        let mut candidate = file(&[b"valid"], FILE_BYTES);
        candidate.path_components = path_components;
        let error = plan_tree(request(vec![candidate], SymlinkPolicy::Reject)?)
            .expect_err("malformed path must fail");
        assert!(
            error
                .blockers()
                .iter()
                .any(|blocker| blocker.kind() == expected)
        );
    }

    let cases = [
        (
            vec![
                file(&[b"duplicate"], FILE_BYTES),
                file(&[b"duplicate"], FILE_BYTES),
            ],
            BlockerKind::DuplicatePath,
        ),
        (
            vec![file(&[b"missing", b"child"], FILE_BYTES)],
            BlockerKind::MissingParent,
        ),
        (
            vec![
                file(&[b"parent"], FILE_BYTES),
                file(&[b"parent", b"child"], FILE_BYTES),
            ],
            BlockerKind::ParentNotDirectory,
        ),
        (
            vec![RawObservation {
                path_components: owned_components(&[b"socket"]),
                kind: EntryKind::Unsupported,
                mode: FileMode::new(0),
                file_bytes: 0,
                symlink_target: None,
                file_content: None,
            }],
            BlockerKind::UnsupportedKind,
        ),
    ];

    for (observations, expected) in cases {
        let error = plan_tree(request(observations, SymlinkPolicy::Reject)?)
            .expect_err("invalid shape must fail");
        assert!(
            error
                .blockers()
                .iter()
                .any(|blocker| blocker.kind() == expected)
        );
    }
    Ok(())
}

#[test]
fn symlink_policy_rejects_ambient_or_missing_targets() -> Result<(), TreeLimitsError> {
    let rejected = plan_tree(request(
        vec![
            file(&[b"target"], FILE_BYTES),
            symlink(&[b"link"], b"target"),
        ],
        SymlinkPolicy::Reject,
    )?)
    .expect_err("reject policy must reject symlinks");
    assert!(
        rejected
            .blockers()
            .iter()
            .any(|blocker| blocker.kind() == BlockerKind::SymlinkRejected)
    );

    let cases = [
        (
            b"/absolute".as_slice(),
            BlockerKind::SymlinkTargetNotRelative,
        ),
        (b"../escape".as_slice(), BlockerKind::SymlinkTargetEscapes),
        (b"missing".as_slice(), BlockerKind::SymlinkTargetMissing),
        (b"link".as_slice(), BlockerKind::SymlinkTargetSelf),
    ];
    for (target, expected) in cases {
        let error = plan_tree(request(
            vec![symlink(&[b"link"], target)],
            SymlinkPolicy::PreserveInternal,
        )?)
        .expect_err("invalid symlink must fail");
        assert!(
            error
                .blockers()
                .iter()
                .any(|blocker| blocker.kind() == expected)
        );
    }
    Ok(())
}
