//! Explicit positive bounds for all admitted tree resources.

/// Limit categories used by typed errors and blockers.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LimitKind {
    /// Total observed entries.
    Entries,
    /// Relative component depth.
    Depth,
    /// Encoded relative path bytes.
    PathBytes,
    /// One regular-file length.
    FileBytes,
    /// Sum of regular-file lengths.
    TotalBytes,
    /// One symlink target length.
    SymlinkTargetBytes,
}

/// One complete caller-supplied limit set before positive-value admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LimitValues {
    /// Maximum observed entries.
    pub entries: u64,
    /// Maximum relative component depth.
    pub depth: u64,
    /// Maximum encoded relative path bytes.
    pub path_bytes: u64,
    /// Maximum bytes in one regular file.
    pub file_bytes: u64,
    /// Maximum bytes in all regular files.
    pub total_bytes: u64,
    /// Maximum bytes in one symlink target.
    pub symlink_target_bytes: u64,
}

/// A zero-valued limit rejection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeLimitsError {
    kind: LimitKind,
}

impl TreeLimitsError {
    /// Returns the rejected limit category.
    #[must_use]
    pub const fn kind(self) -> LimitKind {
        self.kind
    }
}

/// Complete, explicit, and positive limits for one tree operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TreeLimits {
    values: LimitValues,
}

impl TreeLimits {
    /// Admits one complete positive limit set.
    ///
    /// # Errors
    ///
    /// Returns the first limit category whose value is zero.
    pub const fn new(values: LimitValues) -> Result<Self, TreeLimitsError> {
        if values.entries == 0 {
            return Err(TreeLimitsError {
                kind: LimitKind::Entries,
            });
        }
        if values.depth == 0 {
            return Err(TreeLimitsError {
                kind: LimitKind::Depth,
            });
        }
        if values.path_bytes == 0 {
            return Err(TreeLimitsError {
                kind: LimitKind::PathBytes,
            });
        }
        if values.file_bytes == 0 {
            return Err(TreeLimitsError {
                kind: LimitKind::FileBytes,
            });
        }
        if values.total_bytes == 0 {
            return Err(TreeLimitsError {
                kind: LimitKind::TotalBytes,
            });
        }
        if values.symlink_target_bytes == 0 {
            return Err(TreeLimitsError {
                kind: LimitKind::SymlinkTargetBytes,
            });
        }
        Ok(Self { values })
    }

    /// Returns the entry bound.
    #[must_use]
    pub const fn entries(self) -> u64 {
        self.values.entries
    }

    /// Returns the depth bound.
    #[must_use]
    pub const fn depth(self) -> u64 {
        self.values.depth
    }

    /// Returns the encoded path-byte bound.
    #[must_use]
    pub const fn path_bytes(self) -> u64 {
        self.values.path_bytes
    }

    /// Returns the per-file byte bound.
    #[must_use]
    pub const fn file_bytes(self) -> u64 {
        self.values.file_bytes
    }

    /// Returns the total regular-file byte bound.
    #[must_use]
    pub const fn total_bytes(self) -> u64 {
        self.values.total_bytes
    }

    /// Returns the symlink-target byte bound.
    #[must_use]
    pub const fn symlink_target_bytes(self) -> u64 {
        self.values.symlink_target_bytes
    }
}
