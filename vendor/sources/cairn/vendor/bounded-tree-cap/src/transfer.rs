//! Empty-destination plan execution with source revalidation.

mod destination;
mod revalidate;

use crate::prelude::{CopyOperation, Dir, LimitKind, PreparedTree, ShellError, ShellErrorKind};

/// Bounded facts observed during copy execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CopyReport {
    entries_copied: u64,
    file_bytes_copied: u64,
}

impl CopyReport {
    /// Returns the number of destination entries created.
    #[must_use]
    pub const fn entries_copied(self) -> u64 {
        self.entries_copied
    }

    /// Returns the regular-file bytes copied.
    #[must_use]
    pub const fn file_bytes_copied(self) -> u64 {
        self.file_bytes_copied
    }
}

/// Revalidates a prepared source and executes its plan into an empty capability.
///
/// # Errors
///
/// Returns a typed source-change, destination, limit, or I/O failure.
// r[impl bounded_tree.shell.revalidation]
// r[impl bounded_tree.shell.copy_execution]
pub fn execute(prepared: &PreparedTree, destination: &Dir) -> Result<CopyReport, ShellError> {
    destination::require_empty(destination)?;
    revalidate::all(prepared)?;
    let outcome = execute_operations(prepared, destination)?;
    destination::finalize_directory_modes(prepared, destination)?;
    Ok(outcome)
}

fn execute_operations(
    prepared: &PreparedTree,
    destination: &Dir,
) -> Result<CopyReport, ShellError> {
    debug_assert_eq!(
        prepared.plan.operations().len(),
        prepared.plan.member_facts().len()
    );
    let mut outcome = CopyReport {
        entries_copied: 0,
        file_bytes_copied: 0,
    };
    for operation in prepared.plan.operations() {
        let copied_bytes = execute_operation(prepared, destination, operation)?;
        let entry_update = CountUpdate {
            base_value: outcome.entries_copied,
            added_value: 1,
            kind: LimitKind::Entries,
            operation,
        };
        outcome.entries_copied = checked_add(&entry_update)?;
        let byte_update = CountUpdate {
            base_value: outcome.file_bytes_copied,
            added_value: copied_bytes,
            kind: LimitKind::TotalBytes,
            operation,
        };
        outcome.file_bytes_copied = checked_add(&byte_update)?;
    }
    debug_assert_eq!(
        u64::try_from(prepared.plan.operations().len()).ok(),
        Some(outcome.entries_copied)
    );
    Ok(outcome)
}

struct CountUpdate<'a> {
    base_value: u64,
    added_value: u64,
    kind: LimitKind,
    operation: &'a CopyOperation,
}

fn checked_add(update: &CountUpdate<'_>) -> Result<u64, ShellError> {
    update
        .base_value
        .checked_add(update.added_value)
        .ok_or_else(|| {
            ShellError::new(
                ShellErrorKind::LimitExceeded(update.kind),
                update.operation.path().components().to_vec(),
            )
        })
}

fn execute_operation(
    prepared: &PreparedTree,
    destination: &Dir,
    operation: &CopyOperation,
) -> Result<u64, ShellError> {
    match operation {
        CopyOperation::CreateDirectory { path, .. } => {
            destination::create_directory(destination, path)?;
            Ok(0)
        }
        CopyOperation::CopyFile {
            path,
            mode,
            file_bytes,
            file_content,
        } => {
            let request = destination::FileCopy {
                source: &prepared.source,
                destination,
                path,
                mode: *mode,
                file_bytes: *file_bytes,
                expected_digest: *file_content,
            };
            destination::copy_file(&request)
        }
        CopyOperation::CreateSymlink { path, target, .. } => {
            destination::create_symlink(destination, path, target)?;
            Ok(0)
        }
    }
}
