use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

use crate::determinism;
use crate::determinism::RunMetadata;
use crate::output::Finding;

const BASELINE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize)]
pub struct BaselineEvaluation {
    pub baseline_hash: String,
    pub accepted_total: usize,
    pub current_total: usize,
    pub accepted_current: usize,
    pub new_total: usize,
    pub stale_total: usize,
    pub new_finding_ids: Vec<String>,
    pub stale_finding_ids: Vec<String>,
}

impl BaselineEvaluation {
    pub fn has_new_findings(&self) -> bool {
        self.new_total > 0
    }
}

#[derive(Debug, Serialize)]
struct BaselineFile<'a> {
    schema_version: u32,
    finding_schema_version: u32,
    metadata: &'a RunMetadata,
    baseline_hash: String,
    findings: Vec<BaselineFinding<'a>>,
}

#[derive(Debug, Serialize)]
struct BaselineFinding<'a> {
    stable_id: &'a str,
    semantic_id: &'a str,
    identity_status: &'a str,
    lint: &'a str,
    group: &'a str,
    severity: &'a str,
    file: &'a str,
    line: u32,
    column: u32,
    message: &'a str,
    crate_name: &'a str,
}

#[derive(Debug, Deserialize)]
struct StoredBaselineFile {
    schema_version: u32,
    findings: Vec<StoredBaselineFinding>,
}

#[derive(Debug, Deserialize)]
struct StoredBaselineFinding {
    stable_id: String,
}

pub fn write_baseline(path: &Path, findings: &[Finding], metadata: &RunMetadata) -> Result<(), String> {
    let baseline_hash = baseline_hash(findings.iter().map(|finding| finding.stable_id.as_str()));
    let baseline_findings = findings
        .iter()
        .map(|finding| BaselineFinding {
            stable_id: &finding.stable_id,
            semantic_id: &finding.semantic_id,
            identity_status: &finding.identity_status,
            lint: &finding.lint,
            group: &finding.group,
            severity: &finding.severity,
            file: &finding.file,
            line: finding.line,
            column: finding.column,
            message: &finding.message,
            crate_name: &finding.crate_name,
        })
        .collect();

    let baseline = BaselineFile {
        schema_version: BASELINE_SCHEMA_VERSION,
        finding_schema_version: crate::output::finding_version(),
        metadata,
        baseline_hash,
        findings: baseline_findings,
    };

    let mut rendered =
        serde_json::to_string_pretty(&baseline).map_err(|error| format!("serialize baseline: {error}"))?;
    rendered.push('\n');

    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }

    fs::write(path, rendered).map_err(|error| format!("write {}: {error}", path.display()))
}

pub fn evaluate_baseline(path: &Path, findings: &[Finding]) -> Result<BaselineEvaluation, String> {
    let contents = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let stored: StoredBaselineFile =
        serde_json::from_str(&contents).map_err(|error| format!("parse {}: {error}", path.display()))?;

    if stored.schema_version != BASELINE_SCHEMA_VERSION {
        return Err(format!(
            "unsupported baseline schema_version {}; expected {BASELINE_SCHEMA_VERSION}",
            stored.schema_version
        ));
    }

    let accepted_ids: BTreeSet<String> = stored.findings.into_iter().map(|finding| finding.stable_id).collect();
    let current_ids: BTreeSet<String> = findings.iter().map(|finding| finding.stable_id.clone()).collect();

    let new_finding_ids: Vec<String> = current_ids.difference(&accepted_ids).cloned().collect();
    let stale_finding_ids: Vec<String> = accepted_ids.difference(&current_ids).cloned().collect();
    let accepted_current = current_ids.intersection(&accepted_ids).count();
    let baseline_hash = baseline_hash(accepted_ids.iter().map(String::as_str));

    Ok(BaselineEvaluation {
        baseline_hash,
        accepted_total: accepted_ids.len(),
        current_total: current_ids.len(),
        accepted_current,
        new_total: new_finding_ids.len(),
        stale_total: stale_finding_ids.len(),
        new_finding_ids,
        stale_finding_ids,
    })
}

fn baseline_hash<'a>(stable_ids: impl IntoIterator<Item = &'a str>) -> String {
    let mut ids: Vec<&str> = stable_ids.into_iter().collect();
    ids.sort_unstable();
    ids.dedup();
    let payload = serde_json::json!({
        "schema_version": BASELINE_SCHEMA_VERSION,
        "stable_ids": ids,
    });
    determinism::full_hash(&payload.to_string())
}
