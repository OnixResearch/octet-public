use super::*;

struct Diagnostics {
    entries: Vec<String>,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(MAX_DIAGNOSTICS),
        }
    }

    fn push(&mut self, value: impl Into<String>) {
        if self.entries.len() < MAX_DIAGNOSTICS {
            self.entries.push(value.into());
        }
    }

    fn finish(mut self) -> Vec<String> {
        self.entries.sort();
        self.entries.dedup();
        self.entries
    }
}

pub fn evaluate(report: &str, profile: &Profile) -> Evaluation {
    let mut diagnostics = Diagnostics::new();
    validate_profile(profile, &mut diagnostics);
    let report_bytes = match u64::try_from(report.len()) {
        Ok(value) => value,
        Err(_) => {
            diagnostics.push("SARIF report byte count does not fit u64");
            return failed(diagnostics);
        }
    };
    if report_bytes > profile.max_report_bytes {
        diagnostics.push("SARIF report exceeds profile byte limit");
        return failed(diagnostics);
    }
    let value = match serde_json::from_str::<serde_json::Value>(report) {
        Ok(value) => value,
        Err(error) => {
            diagnostics.push(format!("SARIF report is malformed JSON: {error}"));
            return failed(diagnostics);
        }
    };
    if value.get("version").and_then(serde_json::Value::as_str) != Some(SUPPORTED_VERSION) {
        diagnostics.push("SARIF report version is unsupported");
    }
    let runs = value.get("runs").and_then(serde_json::Value::as_array);
    let Some(runs) = runs else {
        diagnostics.push("SARIF report is missing runs");
        return failed(diagnostics);
    };
    if runs.is_empty() || runs.len() > profile.max_runs {
        diagnostics.push("SARIF run count is outside profile limits");
        return failed(diagnostics);
    }
    let mut findings = Vec::with_capacity(profile.max_results);
    for run in runs {
        findings.extend(parse_run(run, profile, &mut diagnostics));
    }
    findings.sort();
    reject_duplicate_findings(&findings, &mut diagnostics);
    finalize(report, profile, findings, diagnostics)
}

fn finalize(report: &str, profile: &Profile, findings: Vec<Finding>, diagnostics: Diagnostics) -> Evaluation {
    let diagnostics = diagnostics.finish();
    if !diagnostics.is_empty() {
        return Evaluation {
            details: None,
            diagnostics,
        };
    }
    let mut projection = match serde_json::to_vec_pretty(&findings) {
        Ok(value) => value,
        Err(error) => {
            return Evaluation {
                details: None,
                diagnostics: vec![format!("serialize normalized SARIF findings: {error}")],
            };
        }
    };
    projection.push(b'\n');
    Evaluation {
        details: Some(ReceiptDetails {
            schema_version: RECEIPT_SCHEMA.to_owned(),
            profile_id: profile.profile_id.clone(),
            profile_identity: PROFILE_IDENTITY.to_owned(),
            raw_report_identity: prefixed_identity(report.as_bytes()),
            projection_identity: prefixed_identity(&projection),
            source_driver: profile.driver_name.clone(),
            supported_version: SUPPORTED_VERSION.to_owned(),
            findings,
            non_claims: profile
                .non_claims
                .iter()
                .cloned()
                .chain(std::iter::once(REQUIRED_NON_CLAIM.to_owned()))
                .collect(),
        }),
        diagnostics,
    }
}

fn validate_profile(profile: &Profile, diagnostics: &mut Diagnostics) {
    if profile.supported_version != SUPPORTED_VERSION {
        diagnostics.push("SARIF profile version is unsupported");
    }
    if !profile.require_rule_id || !profile.workspace_relative_only {
        diagnostics.push("SARIF profile weakens required rule or local-path policy");
    }
    if profile.evidence_tier != "static_lint" {
        diagnostics.push("SARIF profile evidence tier must remain static_lint");
    }
    if profile.non_claims.is_empty() {
        diagnostics.push("SARIF profile is missing non-claims");
    }
}

fn parse_run(run: &serde_json::Value, profile: &Profile, diagnostics: &mut Diagnostics) -> Vec<Finding> {
    let mut findings = Vec::with_capacity(profile.max_results);
    let driver = run.pointer("/tool/driver");
    let Some(driver) = driver else {
        diagnostics.push("SARIF run is missing tool driver");
        return findings;
    };
    if driver.get("name").and_then(serde_json::Value::as_str) != Some(profile.driver_name.as_str()) {
        diagnostics.push("SARIF driver does not match selected producer profile");
    }
    let rules = driver.get("rules").and_then(serde_json::Value::as_array).map_or(&[][..], Vec::as_slice);
    let rule_ids = declared_rules(rules, profile, diagnostics);
    let artifacts = run.get("artifacts").and_then(serde_json::Value::as_array).map_or(&[][..], Vec::as_slice);
    if artifacts.len() > profile.max_artifacts {
        diagnostics.push("SARIF artifact count exceeds profile limit");
        return findings;
    }
    let results = run.get("results").and_then(serde_json::Value::as_array).map_or(&[][..], Vec::as_slice);
    if results.len() > profile.max_results {
        diagnostics.push("SARIF result count exceeds profile limit");
        return findings;
    }
    for result in results {
        if let Some(finding) = parse_result(result, profile, &rule_ids, artifacts, diagnostics) {
            findings.push(finding);
        }
    }
    findings
}

fn declared_rules(
    rules: &[serde_json::Value],
    profile: &Profile,
    diagnostics: &mut Diagnostics,
) -> std::collections::BTreeSet<String> {
    let mut ids = std::collections::BTreeSet::new();
    if rules.len() > profile.max_rules {
        diagnostics.push("SARIF rule count exceeds profile limit");
        return ids;
    }
    for rule in rules {
        let id = rule.get("id").and_then(serde_json::Value::as_str).unwrap_or_default();
        if id.is_empty() {
            diagnostics.push("SARIF rule declaration is missing id");
        } else if !ids.insert(id.to_owned()) {
            diagnostics.push(format!("duplicate SARIF rule id `{id}`"));
        }
    }
    ids
}

fn parse_result(
    result: &serde_json::Value,
    profile: &Profile,
    rule_ids: &std::collections::BTreeSet<String>,
    artifacts: &[serde_json::Value],
    diagnostics: &mut Diagnostics,
) -> Option<Finding> {
    let rule_id = result.get("ruleId").and_then(serde_json::Value::as_str).unwrap_or_default();
    if rule_id.is_empty() || !rule_ids.contains(rule_id) {
        diagnostics.push("SARIF result ruleId is missing or undeclared");
        return None;
    }
    let level = result.get("level").and_then(serde_json::Value::as_str).unwrap_or("warning");
    let Some(gate_severity) = mapped_level(level, &profile.source_levels) else {
        diagnostics.push(format!("SARIF result `{rule_id}` has unsupported level `{level}`"));
        return None;
    };
    let message = result.pointer("/message/text").and_then(serde_json::Value::as_str).unwrap_or_default();
    if message.is_empty() || message.len() > profile.max_message_bytes {
        diagnostics.push(format!("SARIF result `{rule_id}` message is missing or over limit"));
        return None;
    }
    let locations = result.get("locations").and_then(serde_json::Value::as_array).map_or(&[][..], Vec::as_slice);
    if locations.len() != profile.max_locations_per_result {
        diagnostics.push(format!("SARIF result `{rule_id}` location count is unsupported"));
        return None;
    }
    let Some(physical) = locations[0].get("physicalLocation") else {
        diagnostics.push(format!("SARIF result `{rule_id}` physical location is missing"));
        return None;
    };
    let Some(artifact_location) = physical.get("artifactLocation") else {
        diagnostics.push(format!("SARIF result `{rule_id}` artifact location is missing"));
        return None;
    };
    let file = resolve_path(artifact_location, artifacts, profile, diagnostics)?;
    let Some(region) = physical.get("region") else {
        diagnostics.push(format!("SARIF result `{rule_id}` region is missing"));
        return None;
    };
    let line = region.get("startLine").and_then(serde_json::Value::as_u64).unwrap_or_default();
    let column = region.get("startColumn").and_then(serde_json::Value::as_u64).unwrap_or_default();
    if line < MIN_COORDINATE || column < MIN_COORDINATE {
        diagnostics.push(format!("SARIF result `{rule_id}` region is invalid"));
        return None;
    }
    let stable_id = finding_identity(&IdentityInput {
        driver: &profile.driver_name,
        rule: rule_id,
        level,
        message,
        file: &file,
        line,
        column,
    });
    Some(Finding {
        stable_id,
        source_driver: profile.driver_name.clone(),
        source_rule_id: rule_id.to_owned(),
        source_level: level.to_owned(),
        message: message.to_owned(),
        file,
        line,
        column,
        gate_severity,
    })
}

fn resolve_path(
    location: &serde_json::Value,
    artifacts: &[serde_json::Value],
    profile: &Profile,
    diagnostics: &mut Diagnostics,
) -> Option<String> {
    if location.get("uriBaseId").is_some() {
        diagnostics.push("SARIF artifact location uriBaseId is unsupported");
        return None;
    }
    let uri = if let Some(uri) = location.get("uri").and_then(serde_json::Value::as_str) {
        uri
    } else {
        let Some(index) = location.get("index").and_then(serde_json::Value::as_u64) else {
            diagnostics.push("SARIF artifact location is missing uri or index");
            return None;
        };
        let Ok(index) = usize::try_from(index) else {
            diagnostics.push("SARIF artifact index does not fit usize");
            return None;
        };
        let Some(uri) = artifacts
            .get(index)
            .and_then(|artifact| artifact.pointer("/location/uri"))
            .and_then(serde_json::Value::as_str)
        else {
            diagnostics.push("SARIF artifact index is invalid");
            return None;
        };
        uri
    };
    if !safe_path(uri, profile.max_path_bytes) {
        diagnostics.push(format!("unsafe SARIF artifact location `{uri}`"));
        return None;
    }
    Some(uri.to_owned())
}

fn reject_duplicate_findings(findings: &[Finding], diagnostics: &mut Diagnostics) {
    let mut ids = std::collections::BTreeSet::new();
    for finding in findings {
        if !ids.insert(&finding.stable_id) {
            diagnostics.push(format!("duplicate normalized SARIF finding `{}`", finding.stable_id));
        }
    }
}

fn failed(diagnostics: Diagnostics) -> Evaluation {
    Evaluation {
        details: None,
        diagnostics: diagnostics.finish(),
    }
}
