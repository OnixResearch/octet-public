//! Suppression parsing and validation.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::FamilySelection;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::model::Severity;
use crate::source::SourceFile;

const SUPPRESSION_MARKERS: &[&str] = &["octet-allow:", "slotcar-allow:", "tigerstyle-allow:"];
const POLICY_FIELD: &str = "policy";
const SOURCE_FIELD: &str = "source";
const OWNER_FIELD: &str = "owner";
const RATIONALE_FIELD: &str = "rationale";
const REVIEW_FIELD: &str = "review";
const SAFER_DEFAULT_FIELD: &str = "safer_default";
const SCOPE_FIELD: &str = "scope";
const LINE_SCOPE: &str = "line";
const NEXT_LINE_SCOPE: &str = "next-line";
const UNKNOWN_POLICY_ID: &str = "tigerstyle.suppression.unknown-policy";
const SUPPRESSION_POLICY_ID: &str = "tigerstyle.devtools.lint-suppressions";
const MISSING_FIELD_REMEDIATION: &str = "add required suppression metadata or remove the suppression";
const UNKNOWN_POLICY_REMEDIATION: &str = "cite a policy ID present in the Tiger Style policy registry";
const BLANKET_SCOPE_REMEDIATION: &str = "narrow the suppression to a single line or next line";
const KEY_VALUE_SEPARATOR: char = '=';
const FIELD_SEPARATOR: char = ';';
const FIRST_LINE_NUMBER_OFFSET: usize = 1;

/// Parsed suppression metadata.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Suppression {
    /// Policy ID being suppressed.
    pub policy_id: String,
    /// Tiger Style source section or anchor.
    pub source_section: Option<String>,
    /// Suppression owner.
    pub owner: Option<String>,
    /// Suppression rationale.
    pub rationale: Option<String>,
    /// Review trigger.
    pub review_trigger: Option<String>,
    /// Safer default being bypassed.
    pub safer_default: Option<String>,
    /// Suppression scope.
    pub scope: String,
    /// Source path where the suppression appears.
    pub path: String,
    /// One-based line number where the suppression appears.
    pub line: u32,
}

/// Result of validating suppressions for one source set.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct SuppressionReport {
    /// Valid active suppressions.
    pub active_suppressions: Vec<Suppression>,
    /// Suppression diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// Parses and validates all suppressions in selected rule families.
#[must_use]
pub fn validate_suppressions(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> SuppressionReport {
    let mut report = SuppressionReport::default();
    for source in sources {
        validate_source_suppressions(registry, source, selection, &mut report);
    }
    report
}

fn validate_source_suppressions(
    registry: &PolicyRegistry,
    source: &SourceFile,
    selection: &FamilySelection,
    report: &mut SuppressionReport,
) {
    for (line_index, line_text) in source.text.lines().enumerate() {
        let Some(raw_metadata) = suppression_metadata(line_text) else {
            continue;
        };
        let line = line_number_from_index(line_index);
        validate_suppression_line(registry, source, line, raw_metadata, selection, report);
    }
}

fn suppression_metadata(line_text: &str) -> Option<&str> {
    let mut selected: Option<(usize, &str)> = None;
    for marker in SUPPRESSION_MARKERS {
        let Some(marker_index) = line_text.find(marker) else {
            continue;
        };
        if selected.is_none_or(|(selected_index, _)| marker_index < selected_index) {
            selected = Some((marker_index, marker));
        }
    }
    selected.map(|(marker_index, marker)| &line_text[marker_index + marker.len()..])
}

fn validate_suppression_line(
    registry: &PolicyRegistry,
    source: &SourceFile,
    line: u32,
    raw_metadata: &str,
    selection: &FamilySelection,
    report: &mut SuppressionReport,
) {
    let fields = parse_metadata_fields(raw_metadata);
    let policy_id = field_value(&fields, POLICY_FIELD).unwrap_or(UNKNOWN_POLICY_ID);
    let Some(policy) = registry.policy_by_id(policy_id) else {
        report.diagnostics.push(unknown_policy_diagnostic(source, line, policy_id));
        return;
    };
    if !selection.includes(&policy.pillar) {
        return;
    }

    let suppression = suppression_from_fields(source, line, policy_id, &fields);
    append_metadata_diagnostics(policy, source, line, &suppression, &mut report.diagnostics);
    append_scope_diagnostics(policy, source, line, &suppression, &mut report.diagnostics);
    if !has_suppression_diagnostic(&report.diagnostics, &source.path, line) {
        report.active_suppressions.push(suppression);
    }
}

fn parse_metadata_fields(raw_metadata: &str) -> BTreeMap<String, String> {
    let mut fields = BTreeMap::new();
    for raw_field in raw_metadata.split(FIELD_SEPARATOR) {
        let trimmed_field = raw_field.trim();
        let Some((key, value)) = trimmed_field.split_once(KEY_VALUE_SEPARATOR) else {
            continue;
        };
        let key = key.trim();
        let value = value.trim();
        if key.is_empty() || value.is_empty() {
            continue;
        }
        fields.insert(key.to_owned(), value.to_owned());
    }
    fields
}

fn suppression_from_fields(
    source: &SourceFile,
    line: u32,
    policy_id: &str,
    fields: &BTreeMap<String, String>,
) -> Suppression {
    Suppression {
        policy_id: policy_id.to_owned(),
        source_section: field_value(fields, SOURCE_FIELD).map(ToOwned::to_owned),
        owner: field_value(fields, OWNER_FIELD).map(ToOwned::to_owned),
        rationale: field_value(fields, RATIONALE_FIELD).map(ToOwned::to_owned),
        review_trigger: field_value(fields, REVIEW_FIELD).map(ToOwned::to_owned),
        safer_default: field_value(fields, SAFER_DEFAULT_FIELD).map(ToOwned::to_owned),
        scope: field_value(fields, SCOPE_FIELD).unwrap_or(LINE_SCOPE).to_owned(),
        path: source.path.clone(),
        line,
    }
}

fn field_value<'a>(fields: &'a BTreeMap<String, String>, field: &str) -> Option<&'a str> {
    fields.get(field).map(String::as_str)
}

fn append_metadata_diagnostics(
    policy: &PolicyEntry,
    source: &SourceFile,
    line: u32,
    suppression: &Suppression,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let requirements = &policy.suppression_requirements;
    if requirements.requires_source_section && suppression.source_section.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, SOURCE_FIELD));
    }
    if requirements.requires_owner && suppression.owner.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, OWNER_FIELD));
    }
    if requirements.requires_rationale && suppression.rationale.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, RATIONALE_FIELD));
    }
    if requirements.requires_review_trigger && suppression.review_trigger.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, REVIEW_FIELD));
    }
    if requirements.requires_safer_default && suppression.safer_default.is_none() {
        diagnostics.push(missing_field_diagnostic(policy, source, line, SAFER_DEFAULT_FIELD));
    }
}

fn append_scope_diagnostics(
    policy: &PolicyEntry,
    source: &SourceFile,
    line: u32,
    suppression: &Suppression,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if suppression.scope == LINE_SCOPE || suppression.scope == NEXT_LINE_SCOPE {
        return;
    }
    diagnostics.push(Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: source.path.clone(),
        line: Some(line),
        severity: Severity::Error,
        message: format!("blanket suppression scope `{}` is not allowed", suppression.scope),
        remediation: BLANKET_SCOPE_REMEDIATION.to_owned(),
    }));
}

fn missing_field_diagnostic(policy: &PolicyEntry, source: &SourceFile, line: u32, field: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: source.path.clone(),
        line: Some(line),
        severity: Severity::Error,
        message: format!("suppression is missing required `{field}` metadata"),
        remediation: MISSING_FIELD_REMEDIATION.to_owned(),
    })
}

fn unknown_policy_diagnostic(source: &SourceFile, line: u32, policy_id: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: SUPPRESSION_POLICY_ID.to_owned(),
        source_section: String::from("docs/standards/source/devtools.adoc#enforcing-not-advising"),
        path: source.path.clone(),
        line: Some(line),
        severity: Severity::Error,
        message: format!("suppression references unknown policy `{policy_id}`"),
        remediation: UNKNOWN_POLICY_REMEDIATION.to_owned(),
    })
}

fn has_suppression_diagnostic(diagnostics: &[Diagnostic], path: &str, line: u32) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.path == path && diagnostic.line == Some(line) && diagnostic.is_blocking())
}

fn line_number_from_index(index: usize) -> u32 {
    u32::try_from(index.saturating_add(FIRST_LINE_NUMBER_OFFSET)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Classification;
    use crate::model::EnforcementLevel;
    use crate::model::EnforcementMode;
    use crate::model::EvidenceCommand;
    use crate::model::FixtureCoverage;
    use crate::model::FixtureStatus;
    use crate::model::Mechanism;
    use crate::model::PolicyEntry;
    use crate::model::SuppressionRequirements;

    const TEST_POLICY_ID: &str = "octet.test.required";
    const TEST_PILLAR: &str = "test-pillar";
    const TEST_PATH: &str = "src/lib.rs";

    fn registry() -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["test-checker".to_owned()],
            policies: vec![PolicyEntry {
                policy_id: TEST_POLICY_ID.to_owned(),
                pillar: TEST_PILLAR.to_owned(),
                source_path: "docs/test.adoc".to_owned(),
                source_heading: "Test".to_owned(),
                source_anchor: "docs/test.adoc#test".to_owned(),
                summary: "test policy".to_owned(),
                classification: Classification::Hard,
                enforcement_level: EnforcementLevel::Hard,
                current_enforcement_mode: EnforcementMode::Gate,
                target_enforcement_mode: EnforcementMode::Gate,
                escalation_criteria: String::new(),
                mechanisms: Vec::<Mechanism>::new(),
                deterministic_checker: EvidenceCommand::default(),
                proof_rails: Vec::new(),
                checker_owner: "octet".to_owned(),
                checker_id: "test-checker".to_owned(),
                default_severity: Severity::Error,
                suppression_requirements: SuppressionRequirements {
                    requires_policy_id: true,
                    requires_source_section: true,
                    requires_owner: true,
                    requires_rationale: true,
                    requires_review_trigger: true,
                    requires_safer_default: true,
                },
                fixture_coverage: FixtureCoverage {
                    status: FixtureStatus::NotApplicable,
                    compliant: Vec::new(),
                    noncompliant: Vec::new(),
                },
                verification_evidence_requirements: Vec::new(),
                classification_rationale: "test".to_owned(),
                false_positive_risk: "test".to_owned(),
                existing_enforcement: Vec::new(),
                custom_checker_justification: "test".to_owned(),
            }],
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        }
    }

    fn complete_suppression(marker: &str) -> String {
        format!(
            "// {marker} policy={TEST_POLICY_ID}; source=docs/test; owner=team; rationale=fixture; review=expiry; safer_default=remove; scope=next-line\nfn main() {{}}"
        )
    }

    fn report_for(text: String) -> SuppressionReport {
        validate_suppressions(&registry(), &[SourceFile::new(TEST_PATH, text)], &FamilySelection::all())
    }

    #[test]
    fn advertised_waiver_markers_validate_with_complete_metadata() {
        for marker in SUPPRESSION_MARKERS {
            let report = report_for(complete_suppression(marker));
            assert!(report.diagnostics.is_empty(), "{marker} diagnostics: {report:#?}");
            assert_eq!(report.active_suppressions.len(), 1);
            assert_eq!(report.active_suppressions[0].policy_id, TEST_POLICY_ID);
            assert_eq!(report.active_suppressions[0].scope, NEXT_LINE_SCOPE);
        }
    }

    #[test]
    fn missing_structured_waiver_fields_fail_closed() {
        let report = report_for(format!("// octet-allow: policy={TEST_POLICY_ID}\nfn main() {{}}"));
        assert!(report.active_suppressions.is_empty());
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("`owner`")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("`rationale`")));
    }

    #[test]
    fn blanket_waiver_scope_fails_closed() {
        let text = complete_suppression("slotcar-allow:").replace("scope=next-line", "scope=module");
        let report = report_for(text);
        assert!(report.active_suppressions.is_empty());
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains("blanket suppression scope `module`"))
        );
    }
}
