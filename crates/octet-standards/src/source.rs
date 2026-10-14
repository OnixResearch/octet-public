//! Source text abstractions for pure checker logic.

/// Source file content passed to pure checks.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceFile {
    /// Repository-relative or display path.
    pub path: String,
    /// Source text.
    pub text: String,
}

impl SourceFile {
    /// Creates a source file value.
    #[must_use]
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }
}
