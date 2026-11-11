use super::Evaluation;
use super::Finding;
use super::IdentityInput;
use super::MAX_DIAGNOSTICS;
use super::MIN_COORDINATE;
use super::PROFILE_IDENTITY;
use super::Profile;
use super::RECEIPT_SCHEMA;
use super::REQUIRED_NON_CLAIM;
use super::ReceiptDetails;
use super::SOURCE_ROOT_BASE_ID;
use super::SUPPORTED_VERSION;
use super::finding_identity;
use super::mapped_level;
use super::prefixed_identity;
use super::safe_path;

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
    let Ok(report_bytes) = u64::try_from(report.len()) else {
        diagnostics.push("SARIF report byte count does not fit u64");
        return failed(diagnostics);
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
    let Some(tool) = run.get("tool").filter(|tool| tool.get("driver").is_some()) else {
        diagnostics.push("SARIF run is missing tool driver");
        return findings;
    };
    if !tool
        .pointer("/driver/name")
        .and_then(serde_json::Value::as_str)
        .is_some_and(|name| super::driver_name_matches(&profile.driver_name, name))
    {
        diagnostics.push("SARIF driver does not match selected producer profile");
    }
    let Some(invocations) = optional_array(run, "invocations", "run invocations", diagnostics) else {
        return findings;
    };
    if invocations
        .iter()
        .any(|invocation| invocation.get("executionSuccessful").and_then(serde_json::Value::as_bool) == Some(false))
    {
        diagnostics.push("SARIF run reports an unsuccessful tool execution");
        return findings;
    }
    let Some(rules) = declared_rules(tool, profile, diagnostics) else {
        return findings;
    };
    let Some(artifacts) = optional_array(run, "artifacts", "run artifacts", diagnostics) else {
        return findings;
    };
    if artifacts.len() > profile.max_artifacts {
        diagnostics.push("SARIF artifact count exceeds profile limit");
        return findings;
    }
    // SARIF 2.1.0 §3.14.23: absent or `null` results mean the tool failed to produce a result set.
    let Some(results) = run.get("results").and_then(serde_json::Value::as_array) else {
        diagnostics.push("SARIF run results are missing or not an array");
        return findings;
    };
    if results.len() > profile.max_results {
        diagnostics.push("SARIF result count exceeds profile limit");
        return findings;
    }
    for result in results {
        if let Some(finding) = parse_result(result, profile, &rules, artifacts, diagnostics) {
            findings.push(finding);
        }
    }
    findings
}

/// Reads an optional SARIF array property. An absent property is empty; any other non-array
/// value, including `null`, is a structural error.
fn optional_array<'a>(
    object: &'a serde_json::Value,
    field: &str,
    label: &str,
    diagnostics: &mut Diagnostics,
) -> Option<&'a [serde_json::Value]> {
    match object.get(field) {
        None => Some(&[]),
        Some(serde_json::Value::Array(values)) => Some(values),
        Some(_) => {
            diagnostics.push(format!("SARIF {label} must be an array"));
            None
        }
    }
}

/// Rules declared by the run's tool components.
struct RuleTable<'a> {
    /// Component `0` is `tool.driver`; component `n + 1` is `tool.extensions[n]`.
    components: Vec<&'a [serde_json::Value]>,
    /// Maps each declared rule id to its `defaultConfiguration.level`, when present.
    levels: std::collections::BTreeMap<&'a str, Option<&'a str>>,
}

/// Collects rule declarations from `tool.driver.rules`, which must be an array, and from each
/// `tool.extensions[].rules`, where `CodeQL`'s `--sarif-group-rules-by-pack` places them.
fn declared_rules<'a>(
    tool: &'a serde_json::Value,
    profile: &Profile,
    diagnostics: &mut Diagnostics,
) -> Option<RuleTable<'a>> {
    let Some(driver_rules) = tool.pointer("/driver/rules").and_then(serde_json::Value::as_array) else {
        diagnostics.push("SARIF driver rules are missing or not an array");
        return None;
    };
    let extensions = optional_array(tool, "extensions", "tool extensions", diagnostics)?;
    let mut components = Vec::with_capacity(extensions.len() + 1);
    components.push(driver_rules.as_slice());
    for extension in extensions {
        components.push(optional_array(extension, "rules", "tool extension rules", diagnostics)?);
    }
    if components.iter().map(|rules| rules.len()).sum::<usize>() > profile.max_rules {
        diagnostics.push("SARIF rule count exceeds profile limit");
        return None;
    }
    let mut levels = std::collections::BTreeMap::new();
    for rule in components.iter().flat_map(|rules| rules.iter()) {
        let id = rule.get("id").and_then(serde_json::Value::as_str).unwrap_or_default();
        let default_level = rule.pointer("/defaultConfiguration/level").and_then(serde_json::Value::as_str);
        if id.is_empty() {
            diagnostics.push("SARIF rule declaration is missing id");
        } else if levels.insert(id, default_level).is_some() {
            diagnostics.push(format!("duplicate SARIF rule id `{id}`"));
        }
    }
    Some(RuleTable { components, levels })
}

/// Checks the result's `ruleIndex` and `rule` reference against the declared rule tables: every
/// index must be in range of its tool component and name the result's `ruleId`.
fn rule_reference_matches(
    result: &serde_json::Value,
    rule_id: &str,
    rules: &RuleTable<'_>,
    diagnostics: &mut Diagnostics,
) -> bool {
    let reference = result.get("rule");
    if reference.and_then(|reference| reference.get("id")).is_some_and(|id| id.as_str() != Some(rule_id)) {
        diagnostics.push(format!("SARIF result `{rule_id}` rule reference id disagrees with ruleId"));
        return false;
    }
    let component = match reference.and_then(|reference| reference.get("toolComponent")) {
        None => 0,
        Some(component) => {
            let SarifIndex::At(index) = component.get("index").map_or(SarifIndex::Absent, sarif_index) else {
                diagnostics.push(format!("SARIF result `{rule_id}` tool component reference is unsupported"));
                return false;
            };
            index.saturating_add(1)
        }
    };
    let Some(component_rules) = rules.components.get(component) else {
        diagnostics.push(format!("SARIF result `{rule_id}` tool component index is invalid"));
        return false;
    };
    let mut declared_index = None;
    for index in [
        result.get("ruleIndex"),
        reference.and_then(|reference| reference.get("index")),
    ]
    .into_iter()
    .flatten()
    {
        match sarif_index(index) {
            SarifIndex::Absent => {}
            SarifIndex::Invalid => {
                diagnostics.push(format!("SARIF result `{rule_id}` rule index is invalid"));
                return false;
            }
            SarifIndex::At(index) if declared_index.is_some_and(|declared| declared != index) => {
                diagnostics.push(format!("SARIF result `{rule_id}` ruleIndex disagrees with its rule reference"));
                return false;
            }
            SarifIndex::At(index) => declared_index = Some(index),
        }
    }
    let declares_rule = |rule: &serde_json::Value| rule.get("id").and_then(serde_json::Value::as_str) == Some(rule_id);
    let matches = match declared_index {
        Some(index) => component_rules.get(index).is_some_and(declares_rule),
        None => component == 0 || component_rules.iter().any(declares_rule),
    };
    if !matches {
        diagnostics.push(format!("SARIF result `{rule_id}` rule index does not name its ruleId"));
    }
    matches
}

/// A SARIF array index property value.
enum SarifIndex {
    /// `-1`, the SARIF default meaning "no index".
    Absent,
    At(usize),
    /// Anything other than `-1` or a non-negative integer.
    Invalid,
}

fn sarif_index(value: &serde_json::Value) -> SarifIndex {
    if value.as_i64() == Some(-1) {
        return SarifIndex::Absent;
    }
    value
        .as_u64()
        .and_then(|index| usize::try_from(index).ok())
        .map_or(SarifIndex::Invalid, SarifIndex::At)
}

fn parse_result(
    result: &serde_json::Value,
    profile: &Profile,
    rules: &RuleTable<'_>,
    artifacts: &[serde_json::Value],
    diagnostics: &mut Diagnostics,
) -> Option<Finding> {
    let rule_id = result.get("ruleId").and_then(serde_json::Value::as_str).unwrap_or_default();
    let Some(default_level) = rules.levels.get(rule_id) else {
        diagnostics.push("SARIF result ruleId is missing or undeclared");
        return None;
    };
    if !rule_reference_matches(result, rule_id, rules, diagnostics) {
        return None;
    }
    let explicit_level = result.get("level").and_then(serde_json::Value::as_str);
    // SARIF 2.1.0 §3.27.9: an absent kind means `fail`; only `fail` results are findings.
    match result.get("kind").and_then(serde_json::Value::as_str).unwrap_or("fail") {
        "fail" => {}
        kind @ ("pass" | "notApplicable" | "informational") => {
            if explicit_level.is_some_and(|level| level != "none") {
                diagnostics.push(format!("SARIF result `{rule_id}` of kind `{kind}` has a level other than `none`"));
            }
            return None;
        }
        kind @ ("review" | "open") => {
            diagnostics.push(format!("SARIF result `{rule_id}` has undecided kind `{kind}`"));
            return None;
        }
        kind => {
            diagnostics.push(format!("SARIF result `{rule_id}` has unsupported kind `{kind}`"));
            return None;
        }
    }
    // SARIF 2.1.0 §3.27.10: an absent level takes the rule's `defaultConfiguration.level`, then
    // `warning`.
    let level = explicit_level.or(*default_level).unwrap_or("warning");
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
    // SARIF 2.1.0 §3.30.6: an absent `startColumn` means column 1; CodeQL omits it there.
    let column = region
        .get("startColumn")
        .map_or(Some(MIN_COORDINATE), serde_json::Value::as_u64)
        .unwrap_or_default();
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

/// Resolves a result's artifact location to a workspace-relative path. A location may carry a
/// `uri`, an `index` into the run artifact table, or both; when both are present they must name
/// the same path.
fn resolve_path(
    location: &serde_json::Value,
    artifacts: &[serde_json::Value],
    profile: &Profile,
    diagnostics: &mut Diagnostics,
) -> Option<String> {
    let indexed = match location.get("index").map_or(SarifIndex::Absent, sarif_index) {
        SarifIndex::Absent => None,
        SarifIndex::Invalid => {
            diagnostics.push("SARIF artifact index is invalid");
            return None;
        }
        SarifIndex::At(index) => {
            let Some(artifact_location) = artifacts.get(index).and_then(|artifact| artifact.get("location")) else {
                diagnostics.push("SARIF artifact index is invalid");
                return None;
            };
            Some(location_path(artifact_location, profile, diagnostics)?)
        }
    };
    let direct = match location.get("uri") {
        None => None,
        Some(_) => Some(location_path(location, profile, diagnostics)?),
    };
    match (direct, indexed) {
        (Some(direct), Some(indexed)) if direct != indexed => {
            diagnostics.push(format!("SARIF artifact location `{direct}` disagrees with its artifact index"));
            None
        }
        (Some(path), _) | (None, Some(path)) => Some(path),
        (None, None) => {
            diagnostics.push("SARIF artifact location is missing uri or index");
            None
        }
    }
}

/// Reads one `artifactLocation` `uri`. A relative reference without `uriBaseId`, or with the
/// `%SRCROOT%` base that `CodeQL` emits, resolves against the workspace root; any other base is
/// unsupported because Octet cannot bind it to a local directory.
fn location_path(location: &serde_json::Value, profile: &Profile, diagnostics: &mut Diagnostics) -> Option<String> {
    if location.get("uriBaseId").is_some_and(|base| base.as_str() != Some(SOURCE_ROOT_BASE_ID)) {
        diagnostics.push("SARIF artifact location uriBaseId is unsupported");
        return None;
    }
    let Some(uri) = location.get("uri").and_then(serde_json::Value::as_str) else {
        diagnostics.push("SARIF artifact location uri is missing or not a string");
        return None;
    };
    let path = safe_path(uri, profile.max_path_bytes);
    if path.is_none() {
        diagnostics.push(format!("unsafe SARIF artifact location `{uri}`"));
    }
    path
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
