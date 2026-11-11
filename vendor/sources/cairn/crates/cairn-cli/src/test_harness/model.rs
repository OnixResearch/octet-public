use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Outcome {
    Passed,
    Failed,
    Flaky,
}

impl Outcome {
    pub(super) fn s(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Flaky => "flaky_or_timeout",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Class {
    None,
    Product,
    Harness,
    Environment,
    Flaky,
}

impl Class {
    pub(super) fn s(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Product => "product",
            Self::Harness => "harness",
            Self::Environment => "environment",
            Self::Flaky => "flaky_or_timeout",
        }
    }
}

pub(super) struct Report {
    pub(super) outcome: Outcome,
    pub(super) class: Class,
    pub(super) code: &'static str,
    pub(super) diag: &'static str,
    pub(super) attempts: Vec<Value>,
}
