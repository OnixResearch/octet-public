// r[impl octet.benchmark_evidence.signal_oracle]
use std::collections::BTreeMap;

use super::*;

const EXPECTED_OUTCOMES: [&str; 4] = ["finding", "suppressed", "clean", "inconclusive"];

pub(super) fn classify(oracle: &[OracleRecord], findings: &[SignalFinding]) -> Result<SignalSummary, Vec<String>> {
    let mut diagnostics = Vec::new();
    let oracle_by_case = index_oracle(oracle, &mut diagnostics);
    let findings_by_case = index_findings(findings, &mut diagnostics);
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }
    let mut summary = empty_summary();
    for (case_id, expected) in &oracle_by_case {
        let observed = findings_by_case.get(case_id);
        if let Some(finding) = observed
            && (finding.source_identity != expected.source_identity
                || finding.lint_id != expected.lint_id
                || finding.location_class != expected.location_class)
        {
            diagnostics.push(format!("benchmark signal case `{case_id}` has stale source, lint, or location identity"));
            continue;
        }
        classify_expected(expected, observed.copied(), &mut summary, &mut diagnostics);
    }
    for (case_id, finding) in &findings_by_case {
        if oracle_by_case.contains_key(case_id) {
            continue;
        }
        if finding.suppressed {
            increment(&mut summary.unexpected_suppression, "unexpected suppression", &mut diagnostics);
        } else {
            increment(&mut summary.false_positive, "false positive", &mut diagnostics);
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    if diagnostics.is_empty() {
        Ok(summary)
    } else {
        Err(diagnostics)
    }
}

fn index_oracle<'a>(oracle: &'a [OracleRecord], diagnostics: &mut Vec<String>) -> BTreeMap<&'a str, &'a OracleRecord> {
    let mut indexed = BTreeMap::new();
    for record in oracle {
        if record.case_id.is_empty()
            || record.lint_id.is_empty()
            || record.location_class.is_empty()
            || record.rationale.is_empty()
            || record.reviewer_role.is_empty()
            || !resources::is_b3(&record.source_identity)
            || !EXPECTED_OUTCOMES.contains(&record.expected.as_str())
        {
            diagnostics.push("benchmark signal oracle contains an incomplete or unsupported record".to_owned());
        }
        if indexed.insert(record.case_id.as_str(), record).is_some() {
            diagnostics.push(format!("benchmark signal oracle duplicates case `{}`", record.case_id));
        }
    }
    indexed
}

fn index_findings<'a>(
    findings: &'a [SignalFinding],
    diagnostics: &mut Vec<String>,
) -> BTreeMap<&'a str, &'a SignalFinding> {
    let mut indexed = BTreeMap::new();
    for finding in findings {
        if finding.case_id.is_empty()
            || finding.lint_id.is_empty()
            || finding.location_class.is_empty()
            || !resources::is_b3(&finding.source_identity)
        {
            diagnostics.push("benchmark signal findings contain an incomplete record".to_owned());
        }
        if indexed.insert(finding.case_id.as_str(), finding).is_some() {
            diagnostics.push(format!("benchmark signal findings duplicate case `{}`", finding.case_id));
        }
    }
    indexed
}

fn classify_expected(
    expected: &OracleRecord,
    observed: Option<&SignalFinding>,
    summary: &mut SignalSummary,
    diagnostics: &mut Vec<String>,
) {
    match (expected.expected.as_str(), observed) {
        ("finding", Some(finding)) if !finding.suppressed => {
            increment(&mut summary.true_positive, "true positive", diagnostics);
        }
        ("finding", Some(_)) => {
            increment(&mut summary.unexpected_suppression, "unexpected suppression", diagnostics);
        }
        ("finding", None) | ("suppressed", None) => {
            increment(&mut summary.false_negative, "false negative", diagnostics);
        }
        ("suppressed", Some(finding)) if finding.suppressed => {
            increment(&mut summary.expected_suppression, "expected suppression", diagnostics);
        }
        ("suppressed", Some(_)) => {
            increment(&mut summary.unexpected_suppression, "unexpected suppression", diagnostics);
        }
        ("clean", Some(finding)) if finding.suppressed => {
            increment(&mut summary.unexpected_suppression, "unexpected suppression", diagnostics);
        }
        ("clean", Some(_)) => {
            increment(&mut summary.false_positive, "false positive", diagnostics);
        }
        ("clean", None) => {}
        ("inconclusive", _) => {
            increment(&mut summary.inconclusive, "inconclusive", diagnostics);
        }
        _ => diagnostics.push(format!("benchmark signal oracle outcome `{}` is unsupported", expected.expected)),
    }
}

fn increment(value: &mut u64, label: &str, diagnostics: &mut Vec<String>) {
    match value.checked_add(1) {
        Some(next) => *value = next,
        None => diagnostics.push(format!("benchmark {label} count overflowed")),
    }
}

const fn empty_summary() -> SignalSummary {
    SignalSummary {
        true_positive: 0,
        false_positive: 0,
        false_negative: 0,
        expected_suppression: 0,
        unexpected_suppression: 0,
        inconclusive: 0,
    }
}
