use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fmt::Write as _;

use serde::Serialize;

use crate::baseline::BaselineEvaluation;
use crate::determinism::RunMetadata;
use crate::domain_types::FindingDomainView;
use crate::domain_types::validate_finding_domain;
use crate::evidence::EvidenceTier;

const FINDING_SCHEMA_VERSION: u32 = 2;
const EXIT_SUCCESS: i32 = 0;
const EXIT_LINT_FAILURE: i32 = 1;
const EXIT_INTEGRATION_FAILURE: i32 = 2;
const SUMMARY_HEADER: &str = "--- octet summary ---";
const SUMMARY_FOOTER: &str = "---";
const SARIF_SCHEMA_URL: &str =
    "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json";
const SARIF_VERSION: &str = "2.1.0";
const CARGO_PROCESS_SUCCESS: &str = "success";
const CARGO_PROCESS_EXIT_CODE: &str = "exit-code";
const CARGO_PROCESS_TERMINATED: &str = "terminated";
const WARNING_SEVERITY: &str = "warning";
const ERROR_SEVERITY: &str = "error";

pub struct LintMeta {
    pub group: &'static str,
    pub principle: &'static str,
    pub guidance: &'static str,
    pub effect_family: &'static str,
}

pub const UNKNOWN_LINT_META: LintMeta = LintMeta {
    group: "tigerstyle::unknown",
    principle: "unknown",
    guidance: "Review this diagnostic manually; Octet has no registered Tiger Style metadata for it.",
    effect_family: "unknown",
};

#[derive(Debug, Clone, Serialize)]
pub struct Finding {
    pub version: u32,
    pub index: String,
    pub stable_id: String,
    pub semantic_id: String,
    pub identity_status: String,
    pub normalized_snippet_hash: String,
    pub config_hash: String,
    pub cache_status: String,
    pub lint: String,
    pub group: String,
    pub severity: String,
    pub file: String,
    pub line: u32,
    pub column: u32,
    pub message: String,
    pub suggestion: Option<String>,
    pub applicability: Option<String>,
    pub principle: String,
    pub effect_family: String,
    pub evidence_tier: EvidenceTier,
    pub crate_name: String,
}

impl FindingDomainView for Finding {
    fn lint_name(&self) -> &str {
        &self.lint
    }

    fn group_name(&self) -> &str {
        &self.group
    }

    fn stable_id(&self) -> &str {
        &self.stable_id
    }

    fn semantic_id(&self) -> &str {
        &self.semantic_id
    }

    fn artifact_locator(&self) -> &str {
        &self.file
    }

    fn severity(&self) -> &str {
        &self.severity
    }

    fn line(&self) -> u32 {
        self.line
    }

    fn column(&self) -> u32 {
        self.column
    }
}

impl Finding {
    pub fn is_autofixable(&self) -> bool {
        self.applicability.as_deref() == Some("MachineApplicable")
    }

    pub fn location(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }

    pub fn severity_is_error(&self) -> bool {
        self.severity == ERROR_SEVERITY
    }
}

#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FindingCounts {
    pub total_findings: usize,
    pub warning_findings: usize,
    pub error_findings: usize,
    pub autofixable_findings: usize,
}

impl FindingCounts {
    pub fn from_findings(findings: &[Finding]) -> Self {
        let warning_findings = findings.iter().filter(|finding| !finding.severity_is_error()).count();
        let error_findings = findings.iter().filter(|finding| finding.severity_is_error()).count();
        let autofixable_findings = findings.iter().filter(|finding| finding.is_autofixable()).count();

        Self {
            total_findings: findings.len(),
            warning_findings,
            error_findings,
            autofixable_findings,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunStatus {
    Clean,
    WarningOnly,
    LintFailure,
    IntegrationFailure,
}

impl RunStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::WarningOnly => "warning-only",
            Self::LintFailure => "lint-failure",
            Self::IntegrationFailure => "integration-failure",
        }
    }

    pub fn exit_code(self) -> i32 {
        match self {
            Self::Clean | Self::WarningOnly => EXIT_SUCCESS,
            Self::LintFailure => EXIT_LINT_FAILURE,
            Self::IntegrationFailure => EXIT_INTEGRATION_FAILURE,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CargoProcessExit {
    pub classification: &'static str,
    pub code: Option<i32>,
}

impl CargoProcessExit {
    pub fn success() -> Self {
        Self {
            classification: CARGO_PROCESS_SUCCESS,
            code: Some(EXIT_SUCCESS),
        }
    }

    pub fn exit_code(code: i32) -> Self {
        if code == EXIT_SUCCESS {
            return Self::success();
        }

        Self {
            classification: CARGO_PROCESS_EXIT_CODE,
            code: Some(code),
        }
    }

    pub fn terminated() -> Self {
        Self {
            classification: CARGO_PROCESS_TERMINATED,
            code: None,
        }
    }

    pub fn is_success(&self) -> bool {
        self.classification == CARGO_PROCESS_SUCCESS
    }

    pub fn is_terminated(&self) -> bool {
        self.classification == CARGO_PROCESS_TERMINATED
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutcome {
    pub status: RunStatus,
    pub exit_code: i32,
    pub counts: FindingCounts,
    pub cargo_process_exit: CargoProcessExit,
}

impl RunOutcome {
    pub fn classify(findings: &[Finding], cargo_process_exit: CargoProcessExit) -> Self {
        let counts = FindingCounts::from_findings(findings);
        let status = classify_status(&counts, &cargo_process_exit);

        Self {
            status,
            exit_code: status.exit_code(),
            counts,
            cargo_process_exit,
        }
    }

    pub fn force_integration_failure(&self) -> Self {
        Self {
            status: RunStatus::IntegrationFailure,
            exit_code: RunStatus::IntegrationFailure.exit_code(),
            counts: self.counts.clone(),
            cargo_process_exit: self.cargo_process_exit.clone(),
        }
    }

    pub fn force_lint_failure(&self) -> Self {
        Self {
            status: RunStatus::LintFailure,
            exit_code: RunStatus::LintFailure.exit_code(),
            counts: self.counts.clone(),
            cargo_process_exit: self.cargo_process_exit.clone(),
        }
    }
}

#[derive(Serialize)]
struct StatusArtifact<'a> {
    status: RunStatus,
    exit_code: i32,
    output_format: &'a str,
    metadata: &'a RunMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    baseline: Option<&'a BaselineEvaluation>,
    total_findings: usize,
    warning_findings: usize,
    error_findings: usize,
    autofixable_findings: usize,
    cargo_process_exit: &'a CargoProcessExit,
}

fn classify_status(counts: &FindingCounts, cargo_process_exit: &CargoProcessExit) -> RunStatus {
    if !cargo_process_exit.is_success() {
        return RunStatus::IntegrationFailure;
    }

    if counts.error_findings > 0 {
        return RunStatus::LintFailure;
    }

    if counts.total_findings > 0 {
        return RunStatus::WarningOnly;
    }

    RunStatus::Clean
}

pub fn lint_meta_map() -> HashMap<&'static str, LintMeta> {
    octet_lint_metadata::all_metadata()
        .iter()
        .map(|meta| {
            (meta.lint_name, LintMeta {
                group: meta.domain_group,
                principle: meta.principle,
                guidance: meta.guidance,
                effect_family: meta.effect_family,
            })
        })
        .collect()
}

pub fn finding_version() -> u32 {
    FINDING_SCHEMA_VERSION
}

fn validate_finding_domains(findings: &[Finding]) -> Result<(), String> {
    let mut diagnostics = Vec::new();
    for finding in findings {
        if let Err(mut finding_diagnostics) = validate_finding_domain(finding) {
            diagnostics.append(&mut finding_diagnostics);
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(())
    } else {
        Err(format!("invalid finding domain values: {}", diagnostics.join("; ")))
    }
}

pub fn render_json_lines(findings: &[Finding]) -> Result<String, String> {
    validate_finding_domains(findings)?;
    let mut rendered = String::new();

    for finding in findings {
        let line = serde_json::to_string(finding).map_err(|error| format!("write JSON finding: {error}"))?;
        rendered.push_str(&line);
        rendered.push('\n');
    }

    Ok(rendered)
}

pub fn render_sarif(
    findings: &[Finding],
    meta_map: &HashMap<&str, LintMeta>,
    outcome: &RunOutcome,
    command_line: &str,
    metadata: &RunMetadata,
    baseline: Option<&BaselineEvaluation>,
) -> Result<String, String> {
    validate_finding_domains(findings)?;
    let mut seen_rules: Vec<&str> = Vec::new();
    for finding in findings {
        if !seen_rules.contains(&finding.lint.as_str()) {
            seen_rules.push(&finding.lint);
        }
    }

    let rules: Vec<serde_json::Value> = seen_rules
        .iter()
        .map(|lint_name| {
            let meta = meta_map.get(lint_name);
            let guidance = meta.map_or("unknown", |meta| meta.guidance);
            let group = meta.map_or("tigerstyle::unknown", |meta| meta.group);
            let effect_family = meta.map_or("unknown", |meta| meta.effect_family);
            serde_json::json!({
                "id": lint_name,
                "shortDescription": { "text": guidance },
                "properties": { "group": group, "effectFamily": effect_family }
            })
        })
        .collect();

    let results: Vec<serde_json::Value> = findings
        .iter()
        .map(|finding| {
            serde_json::json!({
                "ruleId": finding.lint,
                "level": finding.severity,
                "message": { "text": finding.message },
                "locations": [{
                    "physicalLocation": {
                        "artifactLocation": { "uri": finding.file },
                        "region": {
                            "startLine": finding.line,
                            "startColumn": finding.column
                        }
                    }
                }],
                "properties": {
                    "findingIndex": finding.index,
                    "stableId": finding.stable_id,
                    "semanticId": finding.semantic_id,
                    "effectFamily": finding.effect_family,
                    "evidenceTier": finding.evidence_tier.as_str(),
                    "cacheStatus": finding.cache_status
                }
            })
        })
        .collect();

    let mut properties = serde_json::json!({
        "status": outcome.status.as_str(),
        "exitCode": outcome.exit_code,
        "totalFindings": outcome.counts.total_findings,
        "warningFindings": outcome.counts.warning_findings,
        "errorFindings": outcome.counts.error_findings,
        "autofixableFindings": outcome.counts.autofixable_findings,
        "cargoProcessExitClassification": outcome.cargo_process_exit.classification,
        "cargoProcessExitCode": outcome.cargo_process_exit.code,
        "metadata": metadata,
    });
    if let Some(baseline) = baseline {
        properties["baseline"] = serde_json::json!(baseline);
    }

    let sarif = serde_json::json!({
        "$schema": SARIF_SCHEMA_URL,
        "version": SARIF_VERSION,
        "runs": [{
            "tool": {
                "driver": {
                    "name": "cargo-octet",
                    "version": env!("CARGO_PKG_VERSION"),
                    "rules": rules
                }
            },
            "invocations": [{
                "commandLine": command_line,
                "executionSuccessful": outcome.exit_code == EXIT_SUCCESS,
                "exitCode": outcome.exit_code
            }],
            "properties": properties,
            "results": results
        }]
    });

    let mut rendered = serde_json::to_string_pretty(&sarif).map_err(|error| format!("write SARIF: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn render_status_json(
    outcome: &RunOutcome,
    output_format: &str,
    metadata: &RunMetadata,
    baseline: Option<&BaselineEvaluation>,
) -> Result<String, String> {
    let artifact = StatusArtifact {
        status: outcome.status,
        exit_code: outcome.exit_code,
        output_format,
        metadata,
        baseline,
        total_findings: outcome.counts.total_findings,
        warning_findings: outcome.counts.warning_findings,
        error_findings: outcome.counts.error_findings,
        autofixable_findings: outcome.counts.autofixable_findings,
        cargo_process_exit: &outcome.cargo_process_exit,
    };

    let mut rendered =
        serde_json::to_string_pretty(&artifact).map_err(|error| format!("write status JSON: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

pub fn render_summary(
    findings: &[Finding],
    outcome: &RunOutcome,
    metadata: &RunMetadata,
    baseline: Option<&BaselineEvaluation>,
) -> Result<String, String> {
    let mut summary = String::new();

    writeln!(&mut summary, "{SUMMARY_HEADER}").map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Status: {}", outcome.status.as_str()).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Findings: {}", outcome.counts.total_findings)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Warnings: {}", outcome.counts.warning_findings)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Errors: {}", outcome.counts.error_findings)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Autofixable: {}", outcome.counts.autofixable_findings)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Tool: {} {}", metadata.tool_name, metadata.tool_version)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Toolchain: {}", metadata.toolchain).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Profile: {}", metadata.profile_name).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Config hash: {}", metadata.config_hash)
        .map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Profile hash: {}", metadata.profile_hash)
        .map_err(|error| format!("write summary: {error}"))?;

    if let Some(baseline) = baseline {
        writeln!(&mut summary, "Baseline hash: {}", baseline.baseline_hash)
            .map_err(|error| format!("write summary: {error}"))?;
        writeln!(
            &mut summary,
            "Baseline: {} accepted, {} current accepted, {} new, {} stale",
            baseline.accepted_total, baseline.accepted_current, baseline.new_total, baseline.stale_total
        )
        .map_err(|error| format!("write summary: {error}"))?;
    }

    if findings.is_empty() {
        writeln!(&mut summary, "{SUMMARY_FOOTER}").map_err(|error| format!("write summary: {error}"))?;
        return Ok(summary);
    }

    let mut by_crate: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        *by_crate.entry(&finding.crate_name).or_default() += 1;
    }
    let mut by_crate_sorted: Vec<_> = by_crate.into_iter().collect();
    by_crate_sorted.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
    let max_crate_name = by_crate_sorted.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

    writeln!(&mut summary).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "By crate:").map_err(|error| format!("write summary: {error}"))?;
    for (name, count) in &by_crate_sorted {
        writeln!(&mut summary, "  {name:<width$} {count}", width = max_crate_name + WARNING_SEVERITY.len())
            .map_err(|error| format!("write summary: {error}"))?;
    }

    let mut by_effect: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        *by_effect.entry(&finding.effect_family).or_default() += 1;
    }
    let mut by_effect_sorted: Vec<_> = by_effect.into_iter().collect();
    by_effect_sorted.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
    let max_effect_name = by_effect_sorted.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

    writeln!(&mut summary).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "By effect:").map_err(|error| format!("write summary: {error}"))?;
    for (name, count) in &by_effect_sorted {
        writeln!(&mut summary, "  {name:<width$} {count}", width = max_effect_name + WARNING_SEVERITY.len())
            .map_err(|error| format!("write summary: {error}"))?;
    }

    let mut by_lint: BTreeMap<&str, usize> = BTreeMap::new();
    for finding in findings {
        *by_lint.entry(&finding.lint).or_default() += 1;
    }
    let mut by_lint_sorted: Vec<_> = by_lint.into_iter().collect();
    by_lint_sorted.sort_by(|left, right| right.1.cmp(&left.1).then(left.0.cmp(right.0)));
    let max_lint_name = by_lint_sorted.iter().map(|(name, _)| name.len()).max().unwrap_or(0);

    writeln!(&mut summary).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "By lint:").map_err(|error| format!("write summary: {error}"))?;
    for (name, count) in &by_lint_sorted {
        writeln!(&mut summary, "  {name:<width$} {count}", width = max_lint_name + WARNING_SEVERITY.len())
            .map_err(|error| format!("write summary: {error}"))?;
    }

    let max_index_width = findings.iter().map(|finding| finding.index.len()).max().unwrap_or(0);
    let max_lint_width = findings.iter().map(|finding| finding.lint.len()).max().unwrap_or(0);
    let max_crate_width = findings.iter().map(|finding| finding.crate_name.len()).max().unwrap_or(0);

    writeln!(&mut summary).map_err(|error| format!("write summary: {error}"))?;
    writeln!(&mut summary, "Index:").map_err(|error| format!("write summary: {error}"))?;
    for finding in findings {
        writeln!(
            &mut summary,
            "  {:<index_width$}  {:<lint_width$} {:<crate_width$}  {}",
            finding.index,
            finding.lint,
            finding.crate_name,
            finding.location(),
            index_width = max_index_width,
            lint_width = max_lint_width,
            crate_width = max_crate_width,
        )
        .map_err(|error| format!("write summary: {error}"))?;
    }

    writeln!(&mut summary, "{SUMMARY_FOOTER}").map_err(|error| format!("write summary: {error}"))?;
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_FINDING_INDEX: &str = "F1";
    const TEST_STABLE_ID: &str = "stable-id";
    const TEST_CONFIG_HASH: &str = "config-hash";
    const TEST_CACHE_STATUS: &str = "disabled";
    const TEST_LINT: &str = "ambient_clock";
    const TEST_GROUP: &str = "tigerstyle::purity";
    const TEST_SEVERITY: &str = "warning";
    const TEST_FILE: &str = "src/lib.rs";
    const TEST_MESSAGE: &str = "clock read belongs at the shell boundary";
    const TEST_PRINCIPLE: &str = "Functional Core";
    const TEST_EFFECT_FAMILY: &str = "time";
    const TEST_CRATE_NAME: &str = "example";
    const CANONICAL_GROUP_NAMESPACE: &str = "tigerstyle::";

    #[test]
    fn json_lines_include_static_lint_evidence_tier() {
        let rendered = render_json_lines(&[test_finding()]).expect("finding JSON should render");

        assert!(rendered.contains("\"version\":2"));
        assert!(rendered.contains("\"evidence_tier\":\"static_lint\""));
    }

    #[test]
    fn renderers_reject_malformed_domain_values_before_output() {
        let mut finding = test_finding();
        finding.lint = "ambient clock".to_owned();
        finding.group = "safety".to_owned();
        finding.file = "../src/lib.rs".to_owned();
        finding.severity = "fatal".to_owned();
        finding.line = 0;
        finding.column = 0;

        let error = render_json_lines(&[finding]).expect_err("malformed finding should fail before rendering");

        assert!(error.contains("invalid finding domain values"));
        assert!(error.contains("domain field `lint`"));
        assert!(error.contains("domain field `group`"));
        assert!(error.contains("domain field `file`"));
        assert!(error.contains("domain field `severity`"));
        assert!(error.contains("domain field `line`"));
        assert!(error.contains("domain field `column`"));
    }

    fn runner_metadata_coverage_diagnostics(
        catalog: &[octet_lint_metadata::LintMetadata],
        metadata: &std::collections::HashMap<&str, LintMeta>,
    ) -> Vec<String> {
        let mut diagnostics = Vec::new();
        for entry in catalog {
            match metadata.get(entry.lint_name) {
                Some(meta) => {
                    if meta.group != entry.domain_group {
                        diagnostics.push(format!("runner metadata for `{}` has stale group", entry.lint_name));
                    }
                    if meta.principle != entry.principle {
                        diagnostics.push(format!("runner metadata for `{}` has stale principle", entry.lint_name));
                    }
                    if meta.guidance != entry.guidance {
                        diagnostics.push(format!("runner metadata for `{}` has stale guidance", entry.lint_name));
                    }
                    if meta.effect_family != entry.effect_family {
                        diagnostics.push(format!("runner metadata for `{}` has stale effect family", entry.lint_name));
                    }
                }
                None => diagnostics.push(format!("runner metadata missing catalog lint `{}`", entry.lint_name)),
            }
        }
        for lint_name in metadata.keys() {
            if !catalog.iter().any(|entry| entry.lint_name == *lint_name) {
                diagnostics.push(format!("runner metadata has stale lint `{lint_name}`"));
            }
        }
        diagnostics
    }

    #[test]
    fn runner_metadata_covers_catalog_entries() {
        let metadata = lint_meta_map();
        let diagnostics = runner_metadata_coverage_diagnostics(octet_lint_metadata::all_metadata(), &metadata);

        assert!(diagnostics.is_empty(), "runner metadata drifted: {diagnostics:#?}");
    }

    #[test]
    fn runner_metadata_rejects_missing_catalog_entries() {
        let first = octet_lint_metadata::all_metadata().first().expect("catalog has fixture entry");
        let mut metadata = lint_meta_map();
        metadata.remove(first.lint_name);
        let diagnostics = runner_metadata_coverage_diagnostics(octet_lint_metadata::all_metadata(), &metadata);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("runner metadata missing catalog lint")));
    }

    #[test]
    fn lint_metadata_groups_use_current_namespace() {
        let metadata = lint_meta_map();

        for (lint_name, meta) in metadata {
            assert!(
                meta.group.starts_with(CANONICAL_GROUP_NAMESPACE),
                "lint `{lint_name}` group `{}` should use the current namespace",
                meta.group
            );
        }
    }

    #[test]
    fn sarif_results_include_static_lint_evidence_tier() {
        let finding = test_finding();
        let outcome = RunOutcome::classify(std::slice::from_ref(&finding), CargoProcessExit::success());
        let metadata = RunMetadata {
            tool_name: "cargo-octet",
            tool_version: "0.1.0",
            rustc_version: "rustc-test".to_owned(),
            toolchain: "nightly".to_owned(),
            profile_name: "default".to_owned(),
            profile_hash: "profile-hash".to_owned(),
            config_hash: TEST_CONFIG_HASH.to_owned(),
        };

        let rendered = render_sarif(
            &[finding],
            &lint_meta_map(),
            &outcome,
            "cargo octet check --output-format sarif",
            &metadata,
            None,
        )
        .expect("SARIF should render");

        assert!(rendered.contains("\"evidenceTier\": \"static_lint\""));
    }

    fn test_finding() -> Finding {
        Finding {
            version: finding_version(),
            index: TEST_FINDING_INDEX.to_owned(),
            stable_id: TEST_STABLE_ID.to_owned(),
            semantic_id: TEST_STABLE_ID.to_owned(),
            identity_status: "computed".to_owned(),
            normalized_snippet_hash: "snippet-hash".to_owned(),
            config_hash: TEST_CONFIG_HASH.to_owned(),
            cache_status: TEST_CACHE_STATUS.to_owned(),
            lint: TEST_LINT.to_owned(),
            group: TEST_GROUP.to_owned(),
            severity: TEST_SEVERITY.to_owned(),
            file: TEST_FILE.to_owned(),
            line: 1,
            column: 1,
            message: TEST_MESSAGE.to_owned(),
            suggestion: None,
            applicability: None,
            principle: TEST_PRINCIPLE.to_owned(),
            effect_family: TEST_EFFECT_FAMILY.to_owned(),
            evidence_tier: EvidenceTier::StaticLint,
            crate_name: TEST_CRATE_NAME.to_owned(),
        }
    }
}
