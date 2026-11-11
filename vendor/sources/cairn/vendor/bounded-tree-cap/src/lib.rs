#![forbid(unsafe_code)]

//! Capability-based filesystem shell for `bounded-tree-core`.
//!
//! Callers open source and destination directory capabilities. This crate does
//! not accept an ambient root path and makes no publication or rollback claim.

#[cfg(unix)]
mod failure;
#[cfg(unix)]
mod source;
#[cfg(unix)]
mod transfer;
#[cfg(unix)]
mod util;

#[cfg(unix)]
mod prelude {
    pub use std::ffi::OsString;
    pub use std::io::ErrorKind;
    pub use std::path::Path;

    pub use bounded_tree_core::{
        Blake3Digest, CopyOperation, EntryKind, FileMode, LimitKind, MemberFact, PlanError,
        RawObservation, RelativePath, SymlinkPolicy, TreeLimits, TreePlan, TreeRequest, plan_tree,
    };
    pub use cap_fs_ext::FollowSymlinks;
    pub use cap_std::fs::{Dir, Metadata, OpenOptions, Permissions};

    pub use crate::failure::{OperationKind, ShellError, ShellErrorKind, SourceChangeKind};
    pub use crate::source::PreparedTree;
    pub use crate::util::{
        copy_and_hash_open_file, hash_open_file, metadata_kind, metadata_mode, open_file_create,
        open_file_read, open_parent, set_directory_mode, set_file_mode, staging_directory_mode,
        validate_metadata,
    };
}

#[cfg(unix)]
pub use failure::{OperationKind, ShellError, ShellErrorKind, SourceChangeKind};
#[cfg(unix)]
pub use source::{PreparedTree, prepare};
#[cfg(unix)]
pub use transfer::{CopyReport, execute};
