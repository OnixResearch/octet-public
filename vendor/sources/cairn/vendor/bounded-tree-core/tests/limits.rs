pub use bounded_tree_core::{
    Blake3Digest, BlockerKind, EntryKind, FileMode, LimitValues, RawObservation, SymlinkPolicy,
    TreeLimits, TreeLimitsError, TreeRequest, plan_tree,
};

const ENTRIES_MAX: u64 = 16;
const DEPTH_MAX: u64 = 8;
const PATH_BYTES_MAX: u64 = 256;
const FILE_BYTES_MAX: u64 = 1_024;
const TOTAL_BYTES_MAX: u64 = 4_096;
const TARGET_BYTES_MAX: u64 = 128;
const FILE_BYTES: u64 = 4;
const FILE_MODE: u32 = 0o100_644;
const DIGEST_BYTE: u8 = 0xA5;
const TIGHT_BYTE_BOUND: u64 = 1;

pub(crate) fn limit_values() -> LimitValues {
    LimitValues {
        entries: ENTRIES_MAX,
        depth: DEPTH_MAX,
        path_bytes: PATH_BYTES_MAX,
        file_bytes: FILE_BYTES_MAX,
        total_bytes: TOTAL_BYTES_MAX,
        symlink_target_bytes: TARGET_BYTES_MAX,
    }
}

pub(crate) fn file(path: &[u8], file_bytes: u64) -> RawObservation {
    RawObservation {
        path_components: vec![path.to_vec()],
        kind: EntryKind::File,
        mode: FileMode::new(FILE_MODE),
        file_bytes,
        symlink_target: None,
        file_content: Some(Blake3Digest::from_bytes(
            [DIGEST_BYTE; Blake3Digest::BYTE_COUNT],
        )),
    }
}

#[test]
fn zero_limits_are_rejected_by_category() {
    let valid = (
        ENTRIES_MAX,
        DEPTH_MAX,
        PATH_BYTES_MAX,
        FILE_BYTES_MAX,
        TOTAL_BYTES_MAX,
        TARGET_BYTES_MAX,
    );
    let cases = [
        (0, valid.1, valid.2, valid.3, valid.4, valid.5),
        (valid.0, 0, valid.2, valid.3, valid.4, valid.5),
        (valid.0, valid.1, 0, valid.3, valid.4, valid.5),
        (valid.0, valid.1, valid.2, 0, valid.4, valid.5),
        (valid.0, valid.1, valid.2, valid.3, 0, valid.5),
        (valid.0, valid.1, valid.2, valid.3, valid.4, 0),
    ];

    for values in cases {
        let result = TreeLimits::new(LimitValues {
            entries: values.0,
            depth: values.1,
            path_bytes: values.2,
            file_bytes: values.3,
            total_bytes: values.4,
            symlink_target_bytes: values.5,
        });
        assert_ne!(values, valid);
        assert!(result.is_err());
    }
}

#[test]
pub(crate) fn byte_limits_and_overflow_fail_closed() -> Result<(), TreeLimitsError> {
    let file_policy = TreeLimits::new(LimitValues {
        file_bytes: TIGHT_BYTE_BOUND,
        ..limit_values()
    })?;
    let file_error = plan_tree(TreeRequest {
        observations: vec![file(b"file", FILE_BYTES)],
        limits: file_policy,
        symlink_policy: SymlinkPolicy::Reject,
    })
    .expect_err("oversized file must fail");
    assert!(
        file_error
            .blockers()
            .iter()
            .any(|blocker| blocker.kind() == BlockerKind::FileBytesExceeded)
    );

    let total_policy = TreeLimits::new(LimitValues {
        total_bytes: TIGHT_BYTE_BOUND,
        ..limit_values()
    })?;
    let total_error = plan_tree(TreeRequest {
        observations: vec![file(b"file", FILE_BYTES)],
        limits: total_policy,
        symlink_policy: SymlinkPolicy::Reject,
    })
    .expect_err("oversized total must fail");
    assert!(
        total_error
            .blockers()
            .iter()
            .any(|blocker| blocker.kind() == BlockerKind::TotalBytesExceeded)
    );

    let overflow_policy = TreeLimits::new(LimitValues {
        file_bytes: u64::MAX,
        total_bytes: u64::MAX,
        ..limit_values()
    })?;
    let overflow_error = plan_tree(TreeRequest {
        observations: vec![file(b"first", u64::MAX), file(b"second", TIGHT_BYTE_BOUND)],
        limits: overflow_policy,
        symlink_policy: SymlinkPolicy::Reject,
    })
    .expect_err("overflow must fail");
    assert!(
        overflow_error
            .blockers()
            .iter()
            .any(|blocker| blocker.kind() == BlockerKind::TotalBytesOverflow)
    );
    Ok(())
}
