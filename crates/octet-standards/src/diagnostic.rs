//! Diagnostic model for Tiger Style standards checks.

use serde::Deserialize;
use serde::Serialize;

use crate::model::Severity;

/// Stable checker diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Diagnostic {
    /// Policy ID that produced this diagnostic.
    pub policy_id: String,
    /// Tiger Style source section for the policy.
    pub source_section: String,
    /// Path where the diagnostic applies.
    pub path: String,
    /// One-based source line when available.
    pub line: Option<u32>,
    /// Diagnostic severity.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Human-readable remediation summary.
    pub remediation: String,
}

impl Diagnostic {
    /// Creates a diagnostic.
    #[must_use]
    pub fn new(input: DiagnosticInput) -> Self {
        Self {
            policy_id: input.policy_id,
            source_section: input.source_section,
            path: input.path,
            line: input.line,
            severity: input.severity,
            message: input.message,
            remediation: input.remediation,
        }
    }

    /// Returns true when this diagnostic blocks default CI mode.
    #[must_use]
    pub fn is_blocking(&self) -> bool {
        self.severity == Severity::Error
    }
}

/// Named inputs for constructing diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticInput {
    /// Policy ID that produced this diagnostic.
    pub policy_id: String,
    /// Tiger Style source section for the policy.
    pub source_section: String,
    /// Path where the diagnostic applies.
    pub path: String,
    /// One-based source line when available.
    pub line: Option<u32>,
    /// Diagnostic severity.
    pub severity: Severity,
    /// Human-readable message.
    pub message: String,
    /// Human-readable remediation summary.
    pub remediation: String,
}
