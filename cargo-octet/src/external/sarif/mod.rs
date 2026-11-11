// r[impl octet.external_lint.sarif_intake_profile]
// r[impl octet.external_lint.sarif_intake_identity]
pub const PROFILE_SCHEMA: &str = "octet-sarif-profiles/v1";
pub const PROFILE_IDENTITY: &str = "b3:81a482fe3b807b973898bd7454e73857605f6c5e48e5092d1d6e47b3fe7a9ce7";
pub const RECEIPT_SCHEMA: &str = "octet-sarif-intake-receipt/v1";
pub const SUPPORTED_VERSION: &str = "2.1.0";
pub const RAW_ARTIFACT: &str = "sarif-raw-report.json";
pub const PROJECTION_ARTIFACT: &str = "sarif-normalized-findings.json";
pub const MAX_REPORT_BYTES: u64 = 1_048_576;
pub const REQUIRED_NON_CLAIM: &str = "SARIF intake preserves bounded external static findings only; it does not prove complete vulnerability detection, semantic equivalence, whole-program correctness, or release eligibility.";

const PROFILE_JSON: &str = include_str!("../../../../standards/sarif/profiles.json");
const MAX_DIAGNOSTICS: usize = 32;
const MIN_COORDINATE: u64 = 1;
/// The `uriBaseId` that `CodeQL` uses for the analyzed source root, which Octet binds to the
/// workspace root.
const SOURCE_ROOT_BASE_ID: &str = "%SRCROOT%";

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Levels {
    pub error: String,
    pub warning: String,
    pub note: String,
    pub none: String,
}

// `profile_id` is a serialized profile JSON key; renaming the field would change the wire format.
#[allow(clippy::struct_field_names)]
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub profile_id: String,
    pub driver_name: String,
    pub supported_version: String,
    pub require_rule_id: bool,
    pub workspace_relative_only: bool,
    pub source_levels: Levels,
    pub max_report_bytes: u64,
    pub max_runs: usize,
    pub max_rules: usize,
    pub max_artifacts: usize,
    pub max_results: usize,
    pub max_locations_per_result: usize,
    pub max_message_bytes: usize,
    pub max_path_bytes: usize,
    pub evidence_tier: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
struct Registry {
    schema_version: String,
    profiles: Vec<Profile>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct Finding {
    pub stable_id: String,
    pub source_driver: String,
    pub source_rule_id: String,
    pub source_level: String,
    pub message: String,
    pub file: String,
    pub line: u64,
    pub column: u64,
    pub gate_severity: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiptDetails {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub raw_report_identity: String,
    pub projection_identity: String,
    pub source_driver: String,
    pub supported_version: String,
    pub findings: Vec<Finding>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub details: Option<ReceiptDetails>,
    pub diagnostics: Vec<String>,
}

/// Returns true when a report driver name matches the checked profile name.
///
/// Semgrep CLI 1.180.0 emits `Semgrep OSS` in SARIF while older reports and the checked profiles
/// use `Semgrep`. Both spellings identify the same local OSS producer; the profile and rule
/// identities still establish which tool and rules produced the report.
#[must_use]
pub(crate) fn driver_name_matches(expected: &str, actual: &str) -> bool {
    expected == actual || (expected == "Semgrep" && actual == "Semgrep OSS")
}

// r[impl octet.external_lint.sarif_intake]
// r[impl octet.external_lint.sarif_intake_boundary]
pub fn evaluate(report: &str, profile_id: &str) -> Evaluation {
    let profile = match selected_profile(profile_id) {
        Ok(profile) => profile,
        Err(error) => {
            return Evaluation {
                details: None,
                diagnostics: vec![error],
            };
        }
    };
    validation::evaluate(report, &profile)
}

pub fn selected_profile(profile_id: &str) -> Result<Profile, String> {
    let registry: Registry = serde_json::from_str(PROFILE_JSON)
        .map_err(|error| format!("parse embedded SARIF profile registry: {error}"))?;
    if registry.schema_version != PROFILE_SCHEMA {
        return Err("embedded SARIF profile registry schema is unsupported".to_owned());
    }
    registry
        .profiles
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown SARIF producer profile `{profile_id}`"))
}

pub fn projection_bytes(details: &ReceiptDetails) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(&details.findings)
        .map_err(|error| format!("serialize normalized SARIF projection: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

/// Decodes a SARIF artifact URI reference into the workspace-relative path it names, or `None`
/// when the reference is unsafe or ambiguous. The path policy runs on the decoded path, so
/// percent-encoded traversal, separators, drive letters, and schemes cannot slip past it.
fn safe_path(uri: &str, max_bytes: usize) -> Option<String> {
    // A raw `?` or `#` starts a query or fragment, which a local file location cannot carry.
    if uri.len() > max_bytes || uri.contains(['?', '#']) {
        return None;
    }
    let path = percent_decode(uri)?;
    // A colon in the first segment reads as a drive letter (`C:/x`, `c:x`) or a scheme
    // (`file:/x`); in later segments it is an ordinary file name character, as in `src/a:b.rs`.
    let safe = !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && path.split(['/', '\\']).next().is_some_and(|first| !first.contains(':'))
        && !path.chars().any(char::is_control)
        && path.split(['/', '\\']).all(|part| !matches!(part, "" | "." | ".."));
    safe.then_some(path)
}

/// Decodes RFC 3986 percent escapes; a malformed escape or non-UTF-8 result yields `None`.
fn percent_decode(uri: &str) -> Option<String> {
    let bytes = uri.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        if byte == b'%' {
            let digit = |offset: usize| bytes.get(index + offset).and_then(|digit| char::from(*digit).to_digit(16));
            decoded.push(u8::try_from(digit(1)? * 16 + digit(2)?).ok()?);
            index += 3;
        } else {
            decoded.push(byte);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn mapped_level(level: &str, levels: &Levels) -> Option<String> {
    match level {
        "error" => Some(levels.error.clone()),
        "warning" => Some(levels.warning.clone()),
        "note" => Some(levels.note.clone()),
        "none" => Some(levels.none.clone()),
        _ => None,
    }
}

struct IdentityInput<'a> {
    driver: &'a str,
    rule: &'a str,
    level: &'a str,
    message: &'a str,
    file: &'a str,
    line: u64,
    column: u64,
}

fn finding_identity(input: &IdentityInput<'_>) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"octet-sarif-finding-v1\0");
    for value in [input.driver, input.rule, input.level, input.message, input.file] {
        hasher.update(&value.len().to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.update(&input.line.to_le_bytes());
    hasher.update(&input.column.to_le_bytes());
    format!("b3:{}", hasher.finalize().to_hex())
}

pub(crate) mod producer;

mod validation;

#[cfg(test)]
mod tests;
