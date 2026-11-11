//! Owned observations, member facts, and copy operations.

/// Supported filesystem entry classes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum EntryKind {
    /// Directory.
    Directory,
    /// Regular file.
    File,
    /// Symbolic link.
    Symlink,
    /// Socket, device, FIFO, or another unsupported class.
    Unsupported,
}

/// Explicit symbolic-link policy.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymlinkPolicy {
    /// Reject every symbolic link.
    Reject,
    /// Preserve only links that resolve to admitted members.
    PreserveInternal,
}

/// Raw Unix mode bits observed by the shell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct FileMode(u32);

impl FileMode {
    /// Preserves exact mode bits supplied by the observation shell.
    #[must_use]
    pub const fn new(bits: u32) -> Self {
        Self(bits)
    }

    /// Returns exact mode bits.
    #[must_use]
    pub const fn bits(self) -> u32 {
        self.0
    }
}

/// BLAKE3 content identity for one regular file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Blake3Digest([u8; Self::BYTE_COUNT]);

impl Blake3Digest {
    /// BLAKE3 output length.
    pub const BYTE_COUNT: usize = 32;

    /// Constructs a digest from exact BLAKE3 output bytes.
    #[must_use]
    pub const fn from_bytes(bytes: [u8; Self::BYTE_COUNT]) -> Self {
        Self(bytes)
    }

    /// Returns exact digest bytes.
    #[must_use]
    pub const fn as_bytes(&self) -> &[u8; Self::BYTE_COUNT] {
        &self.0
    }
}

/// One owned shell observation supplied to the pure core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawObservation {
    /// Exact relative path components.
    pub path_components: alloc::vec::Vec<alloc::vec::Vec<u8>>,
    /// Observed entry class.
    pub kind: EntryKind,
    /// Observed mode bits.
    pub mode: FileMode,
    /// Regular-file length, or zero for non-files.
    pub file_bytes: u64,
    /// Exact symlink target bytes, for links only.
    pub symlink_target: Option<alloc::vec::Vec<u8>>,
    /// BLAKE3 regular-file content identity, for files only.
    pub file_content: Option<Blake3Digest>,
}

/// One deterministic admitted member fact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberFact {
    path: crate::RelativePath,
    kind: EntryKind,
    mode: FileMode,
    file_bytes: u64,
    symlink_target: Option<alloc::vec::Vec<u8>>,
    resolved_symlink_target: Option<crate::RelativePath>,
    file_content: Option<Blake3Digest>,
}

pub(crate) struct MemberFactParts {
    pub path: crate::RelativePath,
    pub kind: EntryKind,
    pub mode: FileMode,
    pub file_bytes: u64,
    pub symlink_target: Option<alloc::vec::Vec<u8>>,
    pub resolved_symlink_target: Option<crate::RelativePath>,
    pub file_content: Option<Blake3Digest>,
}

impl MemberFact {
    pub(crate) fn from_parts(parts: MemberFactParts) -> Self {
        Self {
            path: parts.path,
            kind: parts.kind,
            mode: parts.mode,
            file_bytes: parts.file_bytes,
            symlink_target: parts.symlink_target,
            resolved_symlink_target: parts.resolved_symlink_target,
            file_content: parts.file_content,
        }
    }

    /// Returns the exact relative path.
    #[must_use]
    pub const fn path(&self) -> &crate::RelativePath {
        &self.path
    }

    /// Returns the entry class.
    #[must_use]
    pub const fn kind(&self) -> EntryKind {
        self.kind
    }

    /// Returns observed mode bits.
    #[must_use]
    pub const fn mode(&self) -> FileMode {
        self.mode
    }

    /// Returns regular-file bytes.
    #[must_use]
    pub const fn file_bytes(&self) -> u64 {
        self.file_bytes
    }

    /// Returns exact symlink target bytes.
    #[must_use]
    pub fn symlink_target(&self) -> Option<&[u8]> {
        self.symlink_target.as_deref()
    }

    /// Returns the resolved admitted symlink target.
    #[must_use]
    pub const fn resolved_symlink_target(&self) -> Option<&crate::RelativePath> {
        self.resolved_symlink_target.as_ref()
    }

    /// Returns the regular-file content identity.
    #[must_use]
    pub const fn file_content(&self) -> Option<Blake3Digest> {
        self.file_content
    }
}

/// A deterministic destination operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CopyOperation {
    /// Create one directory.
    CreateDirectory {
        /// Destination path.
        path: crate::RelativePath,
        /// Final mode.
        mode: FileMode,
    },
    /// Copy and verify one regular file.
    CopyFile {
        /// Destination path.
        path: crate::RelativePath,
        /// Final mode.
        mode: FileMode,
        /// Planned byte length.
        file_bytes: u64,
        /// Planned BLAKE3 content identity.
        file_content: Blake3Digest,
    },
    /// Create one admitted symbolic link.
    CreateSymlink {
        /// Destination path.
        path: crate::RelativePath,
        /// Exact target bytes.
        target: alloc::vec::Vec<u8>,
        /// Resolved admitted target.
        resolved_target: crate::RelativePath,
    },
}

impl CopyOperation {
    /// Returns the destination path.
    #[must_use]
    pub const fn path(&self) -> &crate::RelativePath {
        match self {
            Self::CreateDirectory { path, .. }
            | Self::CopyFile { path, .. }
            | Self::CreateSymlink { path, .. } => path,
        }
    }
}

/// Complete immutable member facts plus execution operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreePlan {
    member_facts: alloc::vec::Vec<MemberFact>,
    operations: alloc::vec::Vec<CopyOperation>,
}

impl TreePlan {
    pub(crate) const fn new(
        member_facts: alloc::vec::Vec<MemberFact>,
        operations: alloc::vec::Vec<CopyOperation>,
    ) -> Self {
        Self {
            member_facts,
            operations,
        }
    }

    /// Returns sorted admitted member facts.
    #[must_use]
    pub fn member_facts(&self) -> &[MemberFact] {
        &self.member_facts
    }

    /// Returns phase-ordered operations.
    #[must_use]
    pub fn operations(&self) -> &[CopyOperation] {
        &self.operations
    }
}
