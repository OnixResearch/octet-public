use std::fmt;

const LINT_NAME_FIELD: &str = "lint";
const GROUP_FIELD: &str = "group";
const STABLE_ID_FIELD: &str = "stable_id";
const SEMANTIC_ID_FIELD: &str = "semantic_id";
const ARTIFACT_LOCATOR_FIELD: &str = "file";
const SEVERITY_FIELD: &str = "severity";
const LINE_FIELD: &str = "line";
const COLUMN_FIELD: &str = "column";
const WARNING_SEVERITY: &str = "warning";
const ERROR_SEVERITY: &str = "error";
const TIGERSTYLE_GROUP_PREFIX: &str = "tigerstyle::";
const PATH_SEPARATOR: char = '/';
const WINDOWS_PATH_SEPARATOR: char = '\\';
const CURRENT_DIRECTORY_SEGMENT: &str = ".";
const PARENT_DIRECTORY_SEGMENT: &str = "..";
#[cfg(test)]
const THRESHOLD_PERCENT_MAX: u8 = 100;

pub trait FindingDomainView {
    fn lint_name(&self) -> &str;
    fn group_name(&self) -> &str;
    fn stable_id(&self) -> &str;
    fn semantic_id(&self) -> &str;
    fn artifact_locator(&self) -> &str;
    fn severity(&self) -> &str;
    fn line(&self) -> u32;
    fn column(&self) -> u32;
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct LintName(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupName(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StableIdentity(String);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactLocator(String);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OneBasedPosition(u32);

#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ThresholdPercent(u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticSeverity {
    Warning,
    Error,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FindingGateState {
    ReviewInput,
    BlocksPromotion,
}

#[cfg(test)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonEmptyVec<T> {
    items: Vec<T>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedFindingDomain {
    pub lint_name: LintName,
    pub group_name: GroupName,
    pub stable_id: StableIdentity,
    pub semantic_id: StableIdentity,
    pub artifact_locator: ArtifactLocator,
    pub severity: DiagnosticSeverity,
    pub line: OneBasedPosition,
    pub column: OneBasedPosition,
    pub gate_state: FindingGateState,
}

impl LintName {
    pub fn parse(value: &str) -> Result<Self, String> {
        parse_identifier(value, LINT_NAME_FIELD).map(Self)
    }

    #[cfg(test)]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl GroupName {
    pub fn parse(value: &str) -> Result<Self, String> {
        let parsed = parse_no_whitespace(value, GROUP_FIELD)?;
        if !parsed.starts_with(TIGERSTYLE_GROUP_PREFIX) {
            return Err(format!("domain field `{GROUP_FIELD}` must start with `{TIGERSTYLE_GROUP_PREFIX}`"));
        }
        Ok(Self(parsed))
    }

    #[cfg(test)]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl StableIdentity {
    pub fn parse(field: &str, value: &str) -> Result<Self, String> {
        parse_no_whitespace(value, field).map(Self)
    }
}

impl ArtifactLocator {
    pub fn parse(value: &str) -> Result<Self, String> {
        let parsed = parse_non_empty(value, ARTIFACT_LOCATOR_FIELD)?;
        if parsed.starts_with(PATH_SEPARATOR) {
            return Err(format!("domain field `{ARTIFACT_LOCATOR_FIELD}` must be repo-relative, got `{parsed}`"));
        }
        if parsed.contains(WINDOWS_PATH_SEPARATOR) {
            return Err(format!("domain field `{ARTIFACT_LOCATOR_FIELD}` must use `/` separators, got `{parsed}`"));
        }
        if parsed.split(PATH_SEPARATOR).any(|segment| {
            segment.is_empty() || segment == CURRENT_DIRECTORY_SEGMENT || segment == PARENT_DIRECTORY_SEGMENT
        }) {
            return Err(format!(
                "domain field `{ARTIFACT_LOCATOR_FIELD}` must be canonical repo-relative path, got `{parsed}`"
            ));
        }
        Ok(Self(parsed))
    }

    #[cfg(test)]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl OneBasedPosition {
    pub fn new(field: &str, value: u32) -> Result<Self, String> {
        if value == 0 {
            return Err(format!("domain field `{field}` must be one-based"));
        }
        Ok(Self(value))
    }

    #[cfg(test)]
    #[must_use]
    pub fn get(self) -> u32 {
        self.0
    }
}

#[cfg(test)]
impl ThresholdPercent {
    pub fn new(value: u8) -> Result<Self, String> {
        if value > THRESHOLD_PERCENT_MAX {
            return Err(format!("threshold percent `{value}` exceeds {THRESHOLD_PERCENT_MAX}"));
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn get(self) -> u8 {
        self.0
    }
}

impl DiagnosticSeverity {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            WARNING_SEVERITY => Ok(Self::Warning),
            ERROR_SEVERITY => Ok(Self::Error),
            _ => Err(format!(
                "domain field `{SEVERITY_FIELD}` must be `{WARNING_SEVERITY}` or `{ERROR_SEVERITY}`, got `{value}`"
            )),
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Warning => WARNING_SEVERITY,
            Self::Error => ERROR_SEVERITY,
        }
    }

    #[must_use]
    pub fn gate_state(self) -> FindingGateState {
        match self {
            Self::Warning => FindingGateState::ReviewInput,
            Self::Error => FindingGateState::BlocksPromotion,
        }
    }
}

impl fmt::Display for DiagnosticSeverity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
impl<T> NonEmptyVec<T> {
    pub fn new(field: &str, items: Vec<T>) -> Result<Self, String> {
        if items.is_empty() {
            return Err(format!("domain field `{field}` must not be empty"));
        }
        Ok(Self { items })
    }

    #[must_use]
    pub fn as_slice(&self) -> &[T] {
        &self.items
    }
}

pub fn validate_finding_domain(finding: &impl FindingDomainView) -> Result<ValidatedFindingDomain, Vec<String>> {
    let mut diagnostics = Vec::new();
    let lint_name = collect_result(&mut diagnostics, LintName::parse(finding.lint_name()));
    let group_name = collect_result(&mut diagnostics, GroupName::parse(finding.group_name()));
    let stable_id = collect_result(&mut diagnostics, StableIdentity::parse(STABLE_ID_FIELD, finding.stable_id()));
    let semantic_id = collect_result(&mut diagnostics, StableIdentity::parse(SEMANTIC_ID_FIELD, finding.semantic_id()));
    let artifact_locator = collect_result(&mut diagnostics, ArtifactLocator::parse(finding.artifact_locator()));
    let severity = collect_result(&mut diagnostics, DiagnosticSeverity::parse(finding.severity()));
    let line = collect_result(&mut diagnostics, OneBasedPosition::new(LINE_FIELD, finding.line()));
    let column = collect_result(&mut diagnostics, OneBasedPosition::new(COLUMN_FIELD, finding.column()));

    match (lint_name, group_name, stable_id, semantic_id, artifact_locator, severity, line, column) {
        (
            Some(lint_name),
            Some(group_name),
            Some(stable_id),
            Some(semantic_id),
            Some(artifact_locator),
            Some(severity),
            Some(line),
            Some(column),
        ) => Ok(ValidatedFindingDomain {
            lint_name,
            group_name,
            stable_id,
            semantic_id,
            artifact_locator,
            severity,
            line,
            column,
            gate_state: severity.gate_state(),
        }),
        _ => {
            diagnostics.sort();
            diagnostics.dedup();
            Err(diagnostics)
        }
    }
}

fn collect_result<T>(diagnostics: &mut Vec<String>, result: Result<T, String>) -> Option<T> {
    match result {
        Ok(value) => Some(value),
        Err(diagnostic) => {
            diagnostics.push(diagnostic);
            None
        }
    }
}

fn parse_identifier(value: &str, field: &str) -> Result<String, String> {
    let parsed = parse_no_whitespace(value, field)?;
    if !parsed.chars().all(|character| character.is_ascii_alphanumeric() || character == '_') {
        return Err(format!("domain field `{field}` must use canonical identifier spelling, got `{parsed}`"));
    }
    Ok(parsed)
}

fn parse_no_whitespace(value: &str, field: &str) -> Result<String, String> {
    let parsed = parse_non_empty(value, field)?;
    if parsed.chars().any(char::is_whitespace) {
        return Err(format!("domain field `{field}` must not contain whitespace, got `{parsed}`"));
    }
    Ok(parsed)
}

fn parse_non_empty(value: &str, field: &str) -> Result<String, String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(format!("domain field `{field}` must not be empty"));
    }
    if trimmed != value {
        return Err(format!("domain field `{field}` must be canonical without surrounding whitespace"));
    }
    Ok(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINT: &str = "ambient_clock";
    const GROUP: &str = "tigerstyle::purity";
    const STABLE_ID: &str = "ambient_clock:src/lib.rs:1";
    const SEMANTIC_ID: &str = "ambient_clock:clock-read";
    const FILE: &str = "src/lib.rs";
    const WARNING: &str = "warning";
    const BAD_SEVERITY: &str = "fatal";
    const VALID_THRESHOLD: u8 = 80;
    const INVALID_THRESHOLD: u8 = 101;
    const FIRST_LINE: u32 = 1;
    const FIRST_COLUMN: u32 = 1;

    struct FindingFixture {
        lint: &'static str,
        group: &'static str,
        stable_id: &'static str,
        semantic_id: &'static str,
        file: &'static str,
        severity: &'static str,
        line: u32,
        column: u32,
    }

    impl FindingDomainView for FindingFixture {
        fn lint_name(&self) -> &str {
            self.lint
        }

        fn group_name(&self) -> &str {
            self.group
        }

        fn stable_id(&self) -> &str {
            self.stable_id
        }

        fn semantic_id(&self) -> &str {
            self.semantic_id
        }

        fn artifact_locator(&self) -> &str {
            self.file
        }

        fn severity(&self) -> &str {
            self.severity
        }

        fn line(&self) -> u32 {
            self.line
        }

        fn column(&self) -> u32 {
            self.column
        }
    }

    #[test]
    fn finding_domain_accepts_canonical_boundary_values() {
        let domain = validate_finding_domain(&valid_fixture()).expect("valid fixture should parse");

        assert_eq!(domain.lint_name.as_str(), LINT);
        assert_eq!(domain.group_name.as_str(), GROUP);
        assert_eq!(domain.artifact_locator.as_str(), FILE);
        assert_eq!(domain.severity, DiagnosticSeverity::Warning);
        assert_eq!(domain.severity.as_str(), WARNING);
        assert_eq!(domain.gate_state, FindingGateState::ReviewInput);
        assert_eq!(domain.line.get(), FIRST_LINE);
        assert_eq!(domain.column.get(), FIRST_COLUMN);
    }

    #[test]
    fn finding_domain_rejects_raw_malformed_identity_inputs() {
        let fixture = FindingFixture {
            lint: " ambient clock ",
            group: "safety",
            stable_id: "",
            semantic_id: "semantic id with spaces",
            file: "../src\\lib.rs",
            severity: BAD_SEVERITY,
            line: 0,
            column: 0,
        };

        let diagnostics = validate_finding_domain(&fixture).expect_err("bad fixture should fail");

        assert_contains(&diagnostics, "domain field `lint` must be canonical");
        assert_contains(&diagnostics, "domain field `group` must start with");
        assert_contains(&diagnostics, "domain field `stable_id` must not be empty");
        assert_contains(&diagnostics, "domain field `semantic_id` must not contain whitespace");
        assert_contains(&diagnostics, "domain field `file` must use `/` separators");
        assert_contains(&diagnostics, "domain field `severity` must be");
        assert_contains(&diagnostics, "domain field `line` must be one-based");
        assert_contains(&diagnostics, "domain field `column` must be one-based");
    }

    #[test]
    fn refined_collections_and_thresholds_fail_closed() {
        let non_empty = NonEmptyVec::new("required_artifacts", vec![LINT.to_owned()]).expect("non-empty vec");
        assert_eq!(non_empty.as_slice(), &[LINT.to_owned()]);
        assert_eq!(ThresholdPercent::new(VALID_THRESHOLD).expect("valid threshold").get(), VALID_THRESHOLD);

        let empty = NonEmptyVec::<String>::new("required_artifacts", Vec::new());
        assert!(empty.is_err());
        assert!(ThresholdPercent::new(INVALID_THRESHOLD).is_err());
    }

    #[test]
    fn severity_enum_encodes_gate_state_without_boolean_pairs() {
        assert_eq!(DiagnosticSeverity::Warning.gate_state(), FindingGateState::ReviewInput);
        assert_eq!(DiagnosticSeverity::Error.gate_state(), FindingGateState::BlocksPromotion);
    }

    fn valid_fixture() -> FindingFixture {
        FindingFixture {
            lint: LINT,
            group: GROUP,
            stable_id: STABLE_ID,
            semantic_id: SEMANTIC_ID,
            file: FILE,
            severity: WARNING,
            line: FIRST_LINE,
            column: FIRST_COLUMN,
        }
    }

    fn assert_contains(diagnostics: &[String], expected: &str) {
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
            "expected diagnostics to contain `{expected}`, got {diagnostics:?}"
        );
    }
}
