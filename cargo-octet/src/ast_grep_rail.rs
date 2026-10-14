use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

pub(crate) const ENVELOPE_SCHEMA_VERSION: &str = "octet-ast-grep-scan/v1";
pub(crate) const PROFILE_SCHEMA_VERSION: &str = "octet-ast-grep-structural-profile/v1";
pub(crate) const RECORDED_ONLY_ROLE: &str = "recorded_only";
pub(crate) const REQUIRED_NON_CLAIM: &str =
    "ast-grep findings are syntax-only candidate evidence and do not prove behavioral correctness";

const TOOL_NAME: &str = "ast-grep";
const SOURCE_FORMAT_JSON: &str = "json";
const SOURCE_FORMAT_SARIF: &str = "sarif";
const CODEMOD_MODE_DRY_RUN: &str = "dry-run-only";
const CODEMOD_MODE_INTERACTIVE: &str = "interactive";
const INVENTORY_LANE: &str = "inventory";
const ERROR_SEVERITY: &str = "error";
const SOURCE_COORDINATE_OFFSET: u64 = 1;
const MINIMUM_SOURCE_COORDINATE: u64 = 1;
const SUCCESS_EXIT_CODE: i32 = 0;
const BLAKE3_IDENTITY_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const BUNDLE_IDENTITY_DOMAIN: &[u8] = b"octet-ast-grep-rule-bundle/v1\0";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepProfile {
    pub(crate) schema_version: String,
    pub(crate) profile_id: String,
    pub(crate) tool: AstGrepTool,
    pub(crate) sgconfig_path: String,
    pub(crate) bundle_inputs: Vec<String>,
    pub(crate) bundle_identity: String,
    pub(crate) scan_scope: AstGrepScanScope,
    pub(crate) severity_posture: String,
    pub(crate) promotion_lanes: Vec<AstGrepPromotionLane>,
    pub(crate) rule_coverage: Vec<AstGrepRuleCoverage>,
    pub(crate) codemod_policy: AstGrepCodemodPolicy,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepTool {
    pub(crate) name: String,
    pub(crate) version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepScanScope {
    pub(crate) paths: Vec<String>,
    pub(crate) globs: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepPromotionLane {
    pub(crate) lane: String,
    pub(crate) blocking: bool,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepRuleCoverage {
    pub(crate) rule_id: String,
    pub(crate) lane: String,
    pub(crate) positive_cases: u64,
    pub(crate) negative_cases: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepCodemodPolicy {
    pub(crate) mode: String,
    pub(crate) preserve_proposed_diff: bool,
    pub(crate) validation_commands: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepScanEnvelope {
    pub(crate) schema_version: String,
    pub(crate) profile: AstGrepProfile,
    pub(crate) observed_tool_version: String,
    pub(crate) source_format: String,
    pub(crate) tool_exit_code: i32,
    pub(crate) tool_stderr: String,
    pub(crate) source_findings: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepSourcePosition {
    pub(crate) line: u64,
    pub(crate) column: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepFindingDetails {
    pub(crate) rule_id: String,
    pub(crate) severity: String,
    pub(crate) message: String,
    pub(crate) file: String,
    pub(crate) start: AstGrepSourcePosition,
    pub(crate) end: AstGrepSourcePosition,
    pub(crate) metavariables: Value,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub(crate) struct AstGrepReceiptDetails {
    pub(crate) profile_id: String,
    pub(crate) tool_version: String,
    pub(crate) bundle_identity: String,
    pub(crate) bundle_inputs: Vec<String>,
    pub(crate) scan_scope: AstGrepScanScope,
    pub(crate) source_format: String,
    pub(crate) severity_posture: String,
    pub(crate) promotion_lanes: Vec<AstGrepPromotionLane>,
    pub(crate) codemod_mode: String,
    pub(crate) verification_role: String,
    pub(crate) raw_findings: Value,
    pub(crate) findings: Vec<AstGrepFindingDetails>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AstGrepEvaluationStatus {
    Passed,
    ManualReview,
    Failed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AstGrepEvaluation {
    pub(crate) details: Option<AstGrepReceiptDetails>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) status: AstGrepEvaluationStatus,
}

pub(crate) fn parse_profile(text: &str) -> Result<AstGrepProfile, String> {
    serde_json::from_str(text).map_err(|error| format!("parse ast-grep structural profile: {error}"))
}

pub(crate) fn validate_profile(profile: &AstGrepProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if profile.schema_version != PROFILE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported ast-grep profile schema_version `{}`; expected `{PROFILE_SCHEMA_VERSION}`",
            profile.schema_version
        ));
    }
    if profile.profile_id.trim().is_empty() {
        diagnostics.push("ast-grep profile_id must not be empty".to_owned());
    }
    if profile.tool.name != TOOL_NAME {
        diagnostics.push(format!("ast-grep profile tool.name must be `{TOOL_NAME}`"));
    }
    if profile.tool.version.trim().is_empty() {
        diagnostics.push("ast-grep profile tool.version must not be empty".to_owned());
    }
    validate_path("sgconfig_path", &profile.sgconfig_path, &mut diagnostics);
    validate_paths("bundle_inputs", &profile.bundle_inputs, &mut diagnostics);
    validate_paths("scan_scope.paths", &profile.scan_scope.paths, &mut diagnostics);
    if !profile.bundle_inputs.iter().any(|path| path == &profile.sgconfig_path) {
        diagnostics.push("ast-grep bundle_inputs must include sgconfig_path".to_owned());
    }
    validate_bundle_identity(&profile.bundle_identity, &mut diagnostics);
    validate_promotion_lanes(profile, &mut diagnostics);
    validate_rule_coverage(profile, &mut diagnostics);
    if !matches!(profile.codemod_policy.mode.as_str(), CODEMOD_MODE_DRY_RUN | CODEMOD_MODE_INTERACTIVE) {
        diagnostics.push(format!(
            "ast-grep codemod mode `{}` is unsafe; expected `{CODEMOD_MODE_DRY_RUN}` or `{CODEMOD_MODE_INTERACTIVE}`",
            profile.codemod_policy.mode
        ));
    }
    if !profile.codemod_policy.preserve_proposed_diff {
        diagnostics.push("ast-grep codemod policy must preserve the proposed diff".to_owned());
    }
    if profile.codemod_policy.validation_commands.is_empty() {
        diagnostics.push("ast-grep codemod policy requires focused validation commands".to_owned());
    }
    if !profile.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
        diagnostics.push("ast-grep profile is missing the required syntax-only non-claim".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn compute_bundle_identity(
    profile: &AstGrepProfile,
    files: &BTreeMap<String, Vec<u8>>,
) -> Result<String, Vec<String>> {
    let mut diagnostics = validate_profile(profile);
    let expected_paths = profile.bundle_inputs.iter().cloned().collect::<BTreeSet<_>>();
    let observed_paths = files.keys().cloned().collect::<BTreeSet<_>>();
    for missing in expected_paths.difference(&observed_paths) {
        diagnostics.push(format!("ast-grep bundle input `{missing}` is missing"));
    }
    for undeclared in observed_paths.difference(&expected_paths) {
        diagnostics.push(format!("ast-grep bundle input `{undeclared}` is undeclared"));
    }
    if !diagnostics.is_empty() {
        diagnostics.sort();
        diagnostics.dedup();
        return Err(diagnostics);
    }

    let mut hasher = blake3::Hasher::new();
    hasher.update(BUNDLE_IDENTITY_DOMAIN);
    for path in &profile.bundle_inputs {
        let bytes = &files[path];
        update_hash_field(&mut hasher, path.as_bytes()).map_err(|error| vec![error])?;
        update_hash_field(&mut hasher, bytes).map_err(|error| vec![error])?;
    }
    for path in &profile.scan_scope.paths {
        update_hash_field(&mut hasher, path.as_bytes()).map_err(|error| vec![error])?;
    }
    for glob in &profile.scan_scope.globs {
        update_hash_field(&mut hasher, glob.as_bytes()).map_err(|error| vec![error])?;
    }
    Ok(format!("{BLAKE3_IDENTITY_PREFIX}{}", hasher.finalize().to_hex()))
}

pub(crate) fn scan_command_argv(profile: &AstGrepProfile) -> Vec<String> {
    let mut argv = vec![
        TOOL_NAME.to_owned(),
        "scan".to_owned(),
        "--config".to_owned(),
        profile.sgconfig_path.clone(),
        "--json=compact".to_owned(),
        "--include-metadata".to_owned(),
    ];
    for glob in &profile.scan_scope.globs {
        argv.push("--globs".to_owned());
        argv.push(glob.clone());
    }
    argv.extend(profile.scan_scope.paths.iter().cloned());
    argv
}

pub(crate) fn render_scan_envelope(
    profile: AstGrepProfile,
    observed_tool_version: String,
    source_findings: Value,
    tool_stderr: String,
    tool_exit_code: i32,
) -> Result<String, String> {
    let envelope = AstGrepScanEnvelope {
        schema_version: ENVELOPE_SCHEMA_VERSION.to_owned(),
        profile,
        observed_tool_version,
        source_format: SOURCE_FORMAT_JSON.to_owned(),
        tool_exit_code,
        tool_stderr,
        source_findings,
    };
    serde_json::to_string(&envelope).map_err(|error| format!("serialize ast-grep scan envelope: {error}"))
}

pub(crate) fn evaluate_envelope(text: &str) -> AstGrepEvaluation {
    let envelope: AstGrepScanEnvelope = match serde_json::from_str(text) {
        Ok(envelope) => envelope,
        Err(error) => {
            return AstGrepEvaluation {
                details: None,
                diagnostics: vec![format!("parse ast-grep scan envelope: {error}")],
                status: AstGrepEvaluationStatus::Failed,
            };
        }
    };
    let mut diagnostics = validate_profile(&envelope.profile);
    if envelope.schema_version != ENVELOPE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported ast-grep scan schema_version `{}`; expected `{ENVELOPE_SCHEMA_VERSION}`",
            envelope.schema_version
        ));
    }
    if envelope.observed_tool_version != envelope.profile.tool.version {
        diagnostics.push(format!(
            "ast-grep version drift; expected `{}`, got `{}`",
            envelope.profile.tool.version, envelope.observed_tool_version
        ));
    }
    if envelope.tool_exit_code != SUCCESS_EXIT_CODE {
        diagnostics.push(format!("ast-grep scan exited with code `{}`", envelope.tool_exit_code));
    }
    let mut findings = match envelope.source_format.as_str() {
        SOURCE_FORMAT_JSON => parse_json_findings(&envelope.source_findings, &mut diagnostics),
        SOURCE_FORMAT_SARIF => parse_sarif_findings(&envelope.source_findings, &mut diagnostics),
        other => {
            diagnostics.push(format!("unsupported ast-grep source_format `{other}`"));
            Vec::new()
        }
    };
    let promoted_rules = envelope
        .profile
        .rule_coverage
        .iter()
        .map(|coverage| coverage.rule_id.as_str())
        .collect::<BTreeSet<_>>();
    for finding in &findings {
        if !promoted_rules.contains(finding.rule_id.as_str()) {
            diagnostics.push(format!("ast-grep finding uses undeclared rule `{}`", finding.rule_id));
        }
    }
    findings.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then(left.start.line.cmp(&right.start.line))
            .then(left.start.column.cmp(&right.start.column))
            .then(left.rule_id.cmp(&right.rule_id))
    });
    diagnostics.sort();
    diagnostics.dedup();
    let status = if !diagnostics.is_empty() || findings.iter().any(|finding| finding.severity == ERROR_SEVERITY) {
        AstGrepEvaluationStatus::Failed
    } else if findings.is_empty() {
        AstGrepEvaluationStatus::Passed
    } else {
        AstGrepEvaluationStatus::ManualReview
    };
    AstGrepEvaluation {
        details: Some(AstGrepReceiptDetails {
            profile_id: envelope.profile.profile_id,
            tool_version: envelope.observed_tool_version,
            bundle_identity: envelope.profile.bundle_identity,
            bundle_inputs: envelope.profile.bundle_inputs,
            scan_scope: envelope.profile.scan_scope,
            source_format: envelope.source_format,
            severity_posture: envelope.profile.severity_posture,
            promotion_lanes: envelope.profile.promotion_lanes,
            codemod_mode: envelope.profile.codemod_policy.mode,
            verification_role: RECORDED_ONLY_ROLE.to_owned(),
            raw_findings: envelope.source_findings,
            findings,
        }),
        diagnostics,
        status,
    }
}

fn validate_promotion_lanes(profile: &AstGrepProfile, diagnostics: &mut Vec<String>) {
    if profile.promotion_lanes.is_empty() {
        diagnostics.push("ast-grep profile requires promotion lanes".to_owned());
        return;
    }
    let mut lane_names = BTreeSet::new();
    for lane in &profile.promotion_lanes {
        if lane.lane.trim().is_empty() {
            diagnostics.push("ast-grep promotion lane must not be empty".to_owned());
        }
        if !lane_names.insert(lane.lane.as_str()) {
            diagnostics.push(format!("duplicate ast-grep promotion lane `{}`", lane.lane));
        }
    }
}

fn validate_rule_coverage(profile: &AstGrepProfile, diagnostics: &mut Vec<String>) {
    if profile.rule_coverage.is_empty() {
        diagnostics.push("ast-grep profile requires rule coverage rows".to_owned());
        return;
    }
    let lanes = profile.promotion_lanes.iter().map(|lane| lane.lane.as_str()).collect::<BTreeSet<_>>();
    let mut rule_ids = BTreeSet::new();
    for coverage in &profile.rule_coverage {
        if !rule_ids.insert(coverage.rule_id.as_str()) {
            diagnostics.push(format!("duplicate ast-grep rule coverage row `{}`", coverage.rule_id));
        }
        if !lanes.contains(coverage.lane.as_str()) {
            diagnostics.push(format!(
                "ast-grep rule `{}` uses undeclared promotion lane `{}`",
                coverage.rule_id, coverage.lane
            ));
        }
        if coverage.lane != INVENTORY_LANE && coverage.positive_cases == 0 {
            diagnostics.push(format!("promoted ast-grep rule `{}` is missing positive cases", coverage.rule_id));
        }
        if coverage.lane != INVENTORY_LANE && coverage.negative_cases == 0 {
            diagnostics.push(format!("promoted ast-grep rule `{}` is missing negative cases", coverage.rule_id));
        }
    }
}

fn validate_paths(field: &str, paths: &[String], diagnostics: &mut Vec<String>) {
    if paths.is_empty() {
        diagnostics.push(format!("ast-grep {field} must not be empty"));
        return;
    }
    let mut observed = BTreeSet::new();
    for path in paths {
        validate_path(field, path, diagnostics);
        if !observed.insert(path.as_str()) {
            diagnostics.push(format!("ast-grep {field} contains duplicate path `{path}`"));
        }
    }
}

fn validate_path(field: &str, path: &str, diagnostics: &mut Vec<String>) {
    let candidate = std::path::Path::new(path);
    if path.trim().is_empty() || candidate.is_absolute() || candidate.components().any(|part| part.as_os_str() == "..")
    {
        diagnostics.push(format!("ast-grep {field} contains unsafe root-relative path `{path}`"));
    }
}

fn validate_bundle_identity(identity: &str, diagnostics: &mut Vec<String>) {
    let Some(hex) = identity.strip_prefix(BLAKE3_IDENTITY_PREFIX) else {
        diagnostics.push("ast-grep bundle_identity must use the b3: prefix".to_owned());
        return;
    };
    if hex.len() != BLAKE3_HEX_LENGTH || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        diagnostics.push("ast-grep bundle_identity must contain a full BLAKE3 hex digest".to_owned());
    }
}

fn update_hash_field(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), String> {
    let length = u64::try_from(bytes.len()).map_err(|_| "ast-grep bundle field length exceeds u64".to_owned())?;
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

fn parse_json_findings(value: &Value, diagnostics: &mut Vec<String>) -> Vec<AstGrepFindingDetails> {
    let Some(items) = value.as_array() else {
        diagnostics.push("ast-grep JSON findings must be an array".to_owned());
        return Vec::new();
    };
    items.iter().filter_map(|item| parse_json_finding(item, diagnostics)).collect()
}

fn parse_json_finding(item: &Value, diagnostics: &mut Vec<String>) -> Option<AstGrepFindingDetails> {
    let rule_id = required_string(item, "ruleId", "ast-grep JSON finding", diagnostics)?;
    let severity = required_string(item, "severity", "ast-grep JSON finding", diagnostics)?.to_ascii_lowercase();
    let message = item
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| item.get("note").and_then(Value::as_str))
        .unwrap_or(rule_id)
        .to_owned();
    let file = required_string(item, "file", "ast-grep JSON finding", diagnostics)?.to_owned();
    let Some(range) = item.get("range") else {
        diagnostics.push(format!("ast-grep JSON finding `{rule_id}` is missing range"));
        return None;
    };
    let start = parse_zero_based_position(range.get("start"), "start", diagnostics)?;
    let end = parse_zero_based_position(range.get("end"), "end", diagnostics)?;
    Some(AstGrepFindingDetails {
        rule_id: rule_id.to_owned(),
        severity,
        message,
        file,
        start,
        end,
        metavariables: item.get("metaVariables").cloned().unwrap_or(Value::Null),
    })
}

fn parse_sarif_findings(value: &Value, diagnostics: &mut Vec<String>) -> Vec<AstGrepFindingDetails> {
    let Some(runs) = value.get("runs").and_then(Value::as_array) else {
        diagnostics.push("ast-grep SARIF findings must contain runs".to_owned());
        return Vec::new();
    };
    let mut findings = Vec::new();
    for result in runs.iter().flat_map(|run| run.get("results").and_then(Value::as_array).into_iter().flatten()) {
        if let Some(finding) = parse_sarif_finding(result, diagnostics) {
            findings.push(finding);
        }
    }
    findings
}

fn parse_sarif_finding(item: &Value, diagnostics: &mut Vec<String>) -> Option<AstGrepFindingDetails> {
    let rule_id = required_string(item, "ruleId", "ast-grep SARIF result", diagnostics)?;
    let severity = required_string(item, "level", "ast-grep SARIF result", diagnostics)?.to_ascii_lowercase();
    let message = item
        .get("message")
        .and_then(|message| message.get("text"))
        .and_then(Value::as_str)
        .unwrap_or(rule_id)
        .to_owned();
    let physical = item
        .get("locations")
        .and_then(Value::as_array)
        .and_then(|locations| locations.first())
        .and_then(|location| location.get("physicalLocation"));
    let Some(physical) = physical else {
        diagnostics.push(format!("ast-grep SARIF result `{rule_id}` is missing physicalLocation"));
        return None;
    };
    let file = physical
        .get("artifactLocation")
        .and_then(|location| location.get("uri"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_owned();
    if file.is_empty() {
        diagnostics.push(format!("ast-grep SARIF result `{rule_id}` is missing artifact uri"));
        return None;
    }
    let Some(region) = physical.get("region") else {
        diagnostics.push(format!("ast-grep SARIF result `{rule_id}` is missing region"));
        return None;
    };
    let start = parse_one_based_position(region, "startLine", "startColumn", "start", diagnostics)?;
    let end = parse_one_based_position(region, "endLine", "endColumn", "end", diagnostics)?;
    Some(AstGrepFindingDetails {
        rule_id: rule_id.to_owned(),
        severity,
        message,
        file,
        start,
        end,
        metavariables: item
            .get("properties")
            .and_then(|properties| properties.get("metavariables"))
            .cloned()
            .unwrap_or(Value::Null),
    })
}

fn parse_zero_based_position(
    value: Option<&Value>,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<AstGrepSourcePosition> {
    let Some(value) = value else {
        diagnostics.push(format!("ast-grep JSON finding is missing {label} position"));
        return None;
    };
    let Some(line) = value.get("line").and_then(Value::as_u64) else {
        diagnostics.push(format!("ast-grep JSON finding is missing {label}.line"));
        return None;
    };
    let Some(column) = value.get("column").and_then(Value::as_u64) else {
        diagnostics.push(format!("ast-grep JSON finding is missing {label}.column"));
        return None;
    };
    let Some(line) = line.checked_add(SOURCE_COORDINATE_OFFSET) else {
        diagnostics.push(format!("ast-grep JSON finding {label}.line overflows source coordinates"));
        return None;
    };
    let Some(column) = column.checked_add(SOURCE_COORDINATE_OFFSET) else {
        diagnostics.push(format!("ast-grep JSON finding {label}.column overflows source coordinates"));
        return None;
    };
    Some(AstGrepSourcePosition { line, column })
}

fn parse_one_based_position(
    value: &Value,
    line_field: &str,
    column_field: &str,
    label: &str,
    diagnostics: &mut Vec<String>,
) -> Option<AstGrepSourcePosition> {
    let line = value.get(line_field).and_then(Value::as_u64).unwrap_or_default();
    let column = value.get(column_field).and_then(Value::as_u64).unwrap_or_default();
    if line < MINIMUM_SOURCE_COORDINATE || column < MINIMUM_SOURCE_COORDINATE {
        diagnostics.push(format!("ast-grep SARIF result has invalid {label} position"));
        return None;
    }
    Some(AstGrepSourcePosition { line, column })
}

fn required_string<'a>(value: &'a Value, field: &str, context: &str, diagnostics: &mut Vec<String>) -> Option<&'a str> {
    let result = value.get(field).and_then(Value::as_str).filter(|text| !text.trim().is_empty());
    if result.is_none() {
        diagnostics.push(format!("{context} is missing {field}"));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    const ZERO_HASH: &str = "b3:0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn checked_in_profile_bundle_is_fresh() {
        let profile = parse_profile(include_str!("../../standards/ast-grep/generated/profile.json"))
            .expect("checked-in profile should parse");
        let files = BTreeMap::from([
            (
                "standards/ast-grep/sgconfig.yml".to_owned(),
                include_bytes!("../../standards/ast-grep/sgconfig.yml").to_vec(),
            ),
            (
                "standards/ast-grep/rules/unsafe-block-review.yml".to_owned(),
                include_bytes!("../../standards/ast-grep/rules/unsafe-block-review.yml").to_vec(),
            ),
            (
                "standards/ast-grep/rule-tests/unsafe-block-review-test.yml".to_owned(),
                include_bytes!("../../standards/ast-grep/rule-tests/unsafe-block-review-test.yml").to_vec(),
            ),
            (
                "standards/ast-grep/rule-tests/__snapshots__/unsafe-block-review-snapshot.yml".to_owned(),
                include_bytes!("../../standards/ast-grep/rule-tests/__snapshots__/unsafe-block-review-snapshot.yml")
                    .to_vec(),
            ),
        ]);

        let observed = compute_bundle_identity(&profile, &files).expect("checked-in bundle should be valid");

        assert_eq!(profile.bundle_identity, observed, "regenerate the ast-grep profile bundle identity");
    }

    #[test]
    fn profile_requires_positive_and_negative_coverage_for_promoted_rules() {
        let mut profile = valid_profile();
        profile.rule_coverage[0].negative_cases = 0;

        let diagnostics = validate_profile(&profile);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("missing negative cases")));
    }

    #[test]
    fn profile_rejects_automatic_codemod_authority() {
        let mut profile = valid_profile();
        profile.codemod_policy.mode = "automatic".to_owned();

        let diagnostics = validate_profile(&profile);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("mode `automatic` is unsafe")));
    }

    #[test]
    fn bundle_identity_changes_with_rule_bytes_or_scope() {
        let profile = valid_profile();
        let mut files = BTreeMap::from([
            ("standards/ast-grep/sgconfig.yml".to_owned(), b"config".to_vec()),
            ("standards/ast-grep/rules/unsafe.yml".to_owned(), b"rule".to_vec()),
        ]);
        let first = compute_bundle_identity(&profile, &files).expect("valid bundle");
        files.insert("standards/ast-grep/rules/unsafe.yml".to_owned(), b"changed".to_vec());
        let second = compute_bundle_identity(&profile, &files).expect("changed bundle");
        let mut scoped = profile;
        scoped.scan_scope.paths.push("crates".to_owned());
        let third = compute_bundle_identity(&scoped, &files).expect("changed scope");

        assert_ne!(first, second);
        assert_ne!(second, third);
    }

    #[test]
    fn json_findings_preserve_span_metavariables_and_recorded_only_role() {
        let envelope = AstGrepScanEnvelope {
            schema_version: ENVELOPE_SCHEMA_VERSION.to_owned(),
            profile: valid_profile(),
            observed_tool_version: "0.41.1".to_owned(),
            source_format: SOURCE_FORMAT_JSON.to_owned(),
            tool_exit_code: SUCCESS_EXIT_CODE,
            tool_stderr: String::new(),
            source_findings: serde_json::json!([{
                "ruleId": "unsafe-block-review",
                "severity": "warning",
                "message": "review unsafe block",
                "file": "src/lib.rs",
                "range": {"start": {"line": 2, "column": 4}, "end": {"line": 4, "column": 5}},
                "metaVariables": {"single": {"$BODY": {"text": "call()"}}}
            }]),
        };
        let text = serde_json::to_string(&envelope).expect("serialize fixture");

        let evaluation = evaluate_envelope(&text);

        assert_eq!(evaluation.status, AstGrepEvaluationStatus::ManualReview);
        assert!(evaluation.diagnostics.is_empty());
        let details = evaluation.details.expect("details");
        assert_eq!(details.verification_role, RECORDED_ONLY_ROLE);
        assert_eq!(details.findings[0].start.line, 3);
        assert!(details.findings[0].metavariables.get("single").is_some());
    }

    #[test]
    fn malformed_or_undeclared_findings_fail_closed() {
        let envelope = AstGrepScanEnvelope {
            schema_version: ENVELOPE_SCHEMA_VERSION.to_owned(),
            profile: valid_profile(),
            observed_tool_version: "drifted".to_owned(),
            source_format: SOURCE_FORMAT_JSON.to_owned(),
            tool_exit_code: SUCCESS_EXIT_CODE,
            tool_stderr: String::new(),
            source_findings: serde_json::json!([{
                "ruleId": "undeclared-rule",
                "severity": "warning",
                "file": "src/lib.rs",
                "range": {"start": {"line": 0, "column": 0}, "end": {"line": 0, "column": 1}}
            }]),
        };
        let text = serde_json::to_string(&envelope).expect("serialize fixture");

        let evaluation = evaluate_envelope(&text);

        assert_eq!(evaluation.status, AstGrepEvaluationStatus::Failed);
        assert!(evaluation.diagnostics.iter().any(|diagnostic| diagnostic.contains("version drift")));
        assert!(evaluation.diagnostics.iter().any(|diagnostic| diagnostic.contains("undeclared rule")));
    }

    #[test]
    fn sarif_findings_are_normalized_without_role_promotion() {
        let envelope = AstGrepScanEnvelope {
            schema_version: ENVELOPE_SCHEMA_VERSION.to_owned(),
            profile: valid_profile(),
            observed_tool_version: "0.41.1".to_owned(),
            source_format: SOURCE_FORMAT_SARIF.to_owned(),
            tool_exit_code: SUCCESS_EXIT_CODE,
            tool_stderr: String::new(),
            source_findings: serde_json::json!({
                "runs": [{"results": [{
                    "ruleId": "unsafe-block-review",
                    "level": "warning",
                    "message": {"text": "review unsafe block"},
                    "locations": [{"physicalLocation": {
                        "artifactLocation": {"uri": "src/lib.rs"},
                        "region": {"startLine": 2, "startColumn": 3, "endLine": 2, "endColumn": 9}
                    }}],
                    "properties": {"metavariables": {"BODY": "call()"}}
                }]}]
            }),
        };
        let text = serde_json::to_string(&envelope).expect("serialize fixture");

        let evaluation = evaluate_envelope(&text);

        assert_eq!(evaluation.status, AstGrepEvaluationStatus::ManualReview);
        assert!(evaluation.diagnostics.is_empty());
        assert_eq!(evaluation.details.expect("details").findings[0].start.line, 2);
    }

    fn valid_profile() -> AstGrepProfile {
        AstGrepProfile {
            schema_version: PROFILE_SCHEMA_VERSION.to_owned(),
            profile_id: "octet-advisory".to_owned(),
            tool: AstGrepTool {
                name: TOOL_NAME.to_owned(),
                version: "0.41.1".to_owned(),
            },
            sgconfig_path: "standards/ast-grep/sgconfig.yml".to_owned(),
            bundle_inputs: vec![
                "standards/ast-grep/sgconfig.yml".to_owned(),
                "standards/ast-grep/rules/unsafe.yml".to_owned(),
            ],
            bundle_identity: ZERO_HASH.to_owned(),
            scan_scope: AstGrepScanScope {
                paths: vec!["src".to_owned()],
                globs: vec!["*.rs".to_owned()],
            },
            severity_posture: "advisory".to_owned(),
            promotion_lanes: vec![
                AstGrepPromotionLane {
                    lane: INVENTORY_LANE.to_owned(),
                    blocking: false,
                },
                AstGrepPromotionLane {
                    lane: "advisory".to_owned(),
                    blocking: false,
                },
            ],
            rule_coverage: vec![AstGrepRuleCoverage {
                rule_id: "unsafe-block-review".to_owned(),
                lane: "advisory".to_owned(),
                positive_cases: 1,
                negative_cases: 1,
            }],
            codemod_policy: AstGrepCodemodPolicy {
                mode: CODEMOD_MODE_DRY_RUN.to_owned(),
                preserve_proposed_diff: true,
                validation_commands: vec!["cargo test".to_owned()],
            },
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }
}
