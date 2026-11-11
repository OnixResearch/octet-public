//! Typed shell failures.

use crate::prelude::{ErrorKind, LimitKind, PlanError};

/// Filesystem shell operation category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    /// Clone a caller-owned capability.
    CloneCapability,
    /// Enumerate a directory.
    ReadDirectory,
    /// Read no-follow metadata.
    ReadMetadata,
    /// Open a no-follow directory.
    OpenDirectory,
    /// Open a no-follow file.
    OpenFile,
    /// Read a file or symlink target.
    ReadSource,
    /// Create a destination entry.
    CreateDestination,
    /// Write a destination file.
    WriteDestination,
    /// Set destination mode bits.
    SetMode,
}

/// Observable source change category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceChangeKind {
    /// The source entry disappeared.
    Missing,
    /// The no-follow entry kind changed.
    Kind,
    /// The regular-file length changed.
    Length,
    /// Mode bits changed.
    Mode,
    /// Symlink target bytes changed.
    SymlinkTarget,
    /// Regular-file bytes changed without a length change.
    Content,
}

/// Typed shell failure category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShellErrorKind {
    /// A resource bound stopped observation.
    LimitExceeded(LimitKind),
    /// The pure core rejected complete supplied facts.
    PlanRejected(PlanError),
    /// A source changed after planning.
    SourceChanged(SourceChangeKind),
    /// The destination contained an entry before copy.
    DestinationNotEmpty,
    /// An exact Unix component could not be represented.
    UnsupportedPlatformPath,
    /// A capability operation failed.
    Io {
        /// Attempted operation.
        operation: OperationKind,
        /// Stable standard I/O category.
        error_kind: ErrorKind,
    },
}

/// One path-scoped shell failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellError {
    kind: ShellErrorKind,
    path_components: Vec<Vec<u8>>,
}

impl ShellError {
    pub(crate) const fn new(kind: ShellErrorKind, path_components: Vec<Vec<u8>>) -> Self {
        Self {
            kind,
            path_components,
        }
    }

    pub(crate) fn io(
        operation: OperationKind,
        path_components: &[Vec<u8>],
        error: std::io::Error,
    ) -> Self {
        let error_kind = error.kind();
        drop(error);
        Self::new(
            ShellErrorKind::Io {
                operation,
                error_kind,
            },
            path_components.to_vec(),
        )
    }

    /// Returns the typed failure category.
    #[must_use]
    pub const fn kind(&self) -> &ShellErrorKind {
        &self.kind
    }

    /// Returns the exact relative path, or an empty slice for the root.
    #[must_use]
    pub fn path_components(&self) -> &[Vec<u8>] {
        &self.path_components
    }
}
