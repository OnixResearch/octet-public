use std::collections::BTreeMap;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;

use crate::determinism;
use crate::evidence_chain;
use crate::fingerprint;
use crate::output;

pub const FUNCTION_ADDRESS_ARTIFACT_NAME: &str = "function-addresses.json";
pub const FUNCTION_ADDRESS_DOMAIN: &str = "octet.function-address.v1";
// v2 keeps literal contents, including any quoted `//` and the code after it,
// and drops block comments. v1 addresses are not comparable and fail closed.
pub const FUNCTION_ADDRESS_CANONICALIZER: &str = "octet-source-token-function-address-v2";

const FUNCTION_ADDRESS_SCHEMA_VERSION: u32 = 1;
const LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR: &str = "function_address_duplicate_locator";
const LINT_FUNCTION_ADDRESS_MALFORMED: &str = "function_address_malformed";
const LINT_FUNCTION_ADDRESS_MISSING: &str = "function_address_missing";
const LINT_FUNCTION_ADDRESS_OVERCLAIM: &str = "function_address_overclaim";
const LINT_FUNCTION_ADDRESS_STALE: &str = "function_address_stale";
const LINT_FUNCTION_ADDRESS_UNSUPPORTED: &str = "function_address_unsupported";
const FUNCTION_ADDRESS_GROUP: &str = "tigerstyle::function_address";
const FUNCTION_ADDRESS_PRINCIPLE: &str = "content-addressed function identity";
const FUNCTION_ADDRESS_EFFECT_FAMILY: &str = "source-identity";
const FUNCTION_ADDRESS_RECEIPT_CLASS: &str = "function-address-receipt";
const FUNCTION_ADDRESS_PRODUCER: &str = "octet";
const FUNCTION_ADDRESS_PRODUCER_ROLE: &str = "rust-source-shape-producer";
const FUNCTION_ADDRESS_SOURCE_LANGUAGE: &str = "rust";
const DEPENDENCY_IDENTITY_KIND_SOURCE_TOKEN_SET: &str = "source-token-set";
const KAMACITE_CANONICAL_IDENTITY_OWNER: &str = "kamacite-preserves-bytes";
const KAMACITE_COMPATIBILITY_PROJECTION: &str = "kamacite.function-address-receipt.v1.compat-json";
const PRESERVES_HANDOFF_CONTRACT: &str = "octet-to-kamacite-function-address-v1";
const JSON_PROJECTION_ROLE: &str = "operator-debug-compatibility-projection";
const NO_REPARSE_REQUIREMENT: &str = "not-required";
const IDENTITY_ALGORITHM_BLAKE3: &str = "blake3";
const REQUIRED_HANDOFF_RECORD_FIELDS: &[&str] = &[
    "path",
    "name",
    "start_byte",
    "end_byte",
    "function_address",
    "dependency_set_hash",
    "dependency_identity_kind",
    "source_identity",
];
const EMPTY_SOURCE_ENVELOPE_SET_KIND: &str = "empty-source-envelope-set";
const FUNCTION_ADDRESS_EVIDENCE_CHAIN_PROJECTION_ARTIFACT: &str = FUNCTION_ADDRESS_ARTIFACT_NAME;
const OBSERVE_SEVERITY: &str = "warning";
const WARNING_SEVERITY: &str = "warning";
const ERROR_SEVERITY: &str = "error";
const DEFAULT_LINE: u32 = 1;
const DEFAULT_COLUMN: u32 = 1;
const LINE_OFFSET: u32 = 1;
const COLUMN_OFFSET: u32 = 1;
const NEWLINE_BYTE: u8 = b'\n';
const BLAKE3_PREFIX: &str = "b3:";
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const LOCATOR_SEPARATOR: &str = "\u{1f}";
const BASELINE_RECEIPT_PATH: &str = "function-address-baseline";
const BASELINE_RECEIPT_LOCATOR: &str = "receipt";
const PATH_SEPARATOR: char = '/';
const WINDOWS_PATH_SEPARATOR: char = '\\';
const CURRENT_DIRECTORY_SEGMENT: &str = ".";
const PARENT_DIRECTORY_SEGMENT: &str = "..";
const NON_CLAIM_SEMANTIC_CORRECTNESS: &str = "not Rust semantic correctness";
const NON_CLAIM_COMPILER_CORRECTNESS: &str = "not compiler correctness";
const NON_CLAIM_WHOLE_PROGRAM_SAFETY: &str = "not whole-program safety";
const NON_CLAIM_RELEASE_ELIGIBILITY: &str = "not release eligibility";
const REQUIRED_NON_CLAIMS: &[&str] = &[
    NON_CLAIM_SEMANTIC_CORRECTNESS,
    NON_CLAIM_COMPILER_CORRECTNESS,
    NON_CLAIM_WHOLE_PROGRAM_SAFETY,
    NON_CLAIM_RELEASE_ELIGIBILITY,
];
const FORBIDDEN_CLAIM_FRAGMENTS: &[&str] = &[
    "semantic correctness",
    "compiler correctness",
    "runtime behavior",
    "whole-program safety",
    "verifier soundness",
    "release eligibility",
    "resolved call graph",
    "semantic dependency closure",
    "complete semantic closure",
];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum FunctionAddressMode {
    #[default]
    Off,
    Observe,
    Warn,
    Deny,
}

impl FunctionAddressMode {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "off" => Ok(Self::Off),
            "observe" => Ok(Self::Observe),
            "warn" => Ok(Self::Warn),
            "deny" => Ok(Self::Deny),
            other => Err(format!(
                "unsupported function-address mode `{other}`; expected `off`, `observe`, `warn`, or `deny`"
            )),
        }
    }

    pub fn is_enabled(self) -> bool {
        self != Self::Off
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionAddressCheck {
    pub mode: FunctionAddressMode,
    pub paths: Vec<PathBuf>,
    pub baseline: Option<PathBuf>,
}

impl FunctionAddressCheck {
    pub fn off() -> Self {
        Self {
            mode: FunctionAddressMode::Off,
            paths: Vec::new(),
            baseline: None,
        }
    }

    pub fn enabled_paths(&self, workspace_root: &Path) -> Vec<PathBuf> {
        if self.paths.is_empty() {
            vec![workspace_root.to_path_buf()]
        } else {
            self.paths.clone()
        }
    }

    /// Rejects function-address scopes that escape the workspace root; see
    /// [`reject_workspace_escaping_scopes`]. `origin` names where the paths came from, so the
    /// error points at the flag or config entry the user wrote.
    pub fn validate_paths(&self, workspace_root: &Path, origin: ScopeOrigin) -> Result<(), String> {
        reject_workspace_escaping_scopes(workspace_root, &self.paths, origin)
    }
}

/// Where a set of source scopes was declared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScopeOrigin {
    FunctionAddressPathFlag,
    CoreSourceScopes,
}

impl ScopeOrigin {
    fn label(self) -> &'static str {
        match self {
            Self::FunctionAddressPathFlag => "`--function-address-path`",
            Self::CoreSourceScopes => "`core_source_scopes` in `dylint.toml`",
        }
    }
}

/// Rejects scopes that escape the workspace root: a `..` component, an absolute path outside the
/// root, or a path whose canonical form (after symlinks) leaves the root. Record paths are
/// workspace-relative, so an escaping scope would emit records that a later baseline comparison
/// rejects as malformed.
pub fn reject_workspace_escaping_scopes(
    workspace_root: &Path,
    paths: &[PathBuf],
    origin: ScopeOrigin,
) -> Result<(), String> {
    let canonical_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize workspace root {}: {error}", workspace_root.display()))?;
    for path in paths {
        let escapes = path.components().any(|component| component == std::path::Component::ParentDir)
            || (path.is_absolute() && !path.starts_with(workspace_root))
            || std::fs::canonicalize(workspace_root.join(path))
                .is_ok_and(|canonical| !canonical.starts_with(&canonical_root));
        if escapes {
            return Err(format!(
                "{} {} must stay within the workspace root {}",
                origin.label(),
                path.display(),
                workspace_root.display()
            ));
        }
    }
    Ok(())
}

impl Default for FunctionAddressCheck {
    fn default() -> Self {
        Self::off()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FunctionAddressRecord {
    pub path: String,
    pub name: String,
    pub line: u32,
    pub column: u32,
    pub start_byte: usize,
    pub end_byte: usize,
    pub function_address: String,
    pub normalized_body_hash: String,
    pub dependency_set_hash: String,
    pub dependency_identity_kind: String,
    pub dependency_identity_status: String,
    pub source_identity: String,
    pub review: Option<String>,
    pub evidence: Option<String>,
    pub caveats: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct SourceEnvelopeRecord {
    pub path: String,
    pub language: &'static str,
    pub source_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FunctionAddressProducer {
    pub name: &'static str,
    pub role: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct KamaciteProjection {
    pub artifact_class: &'static str,
    pub compatibility_projection: &'static str,
    pub canonical_identity_owner: &'static str,
    pub identity_algorithm: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct PreservesHandoff {
    pub contract: &'static str,
    pub canonical_identity_owner: &'static str,
    pub json_projection_role: &'static str,
    pub rust_reparse_required: &'static str,
    pub span_recomputation_required: &'static str,
    pub required_record_fields: Vec<&'static str>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct FunctionAddressReport {
    pub schema: &'static str,
    pub schema_version: u32,
    pub receipt_class: &'static str,
    pub mode: &'static str,
    pub domain: &'static str,
    pub canonicalizer: &'static str,
    pub producer: FunctionAddressProducer,
    pub tool: &'static str,
    pub source_envelopes: Vec<SourceEnvelopeRecord>,
    pub skipped_directory_links: Vec<String>,
    pub records: Vec<FunctionAddressRecord>,
    pub diagnostic_count: usize,
    pub kamacite_projection: KamaciteProjection,
    pub preserves_handoff: PreservesHandoff,
    pub evidence_chain_handoff: evidence_chain::EvidenceChainHandoff,
    pub non_claims: Vec<&'static str>,
}

/// A symlinked directory the implicit whole-workspace scope skipped because it resolves outside the
/// workspace root: `path` is the workspace-relative link and `target` the link text as written.
/// Only `path` enters the artifact; the target is machine-specific (a Nix `result` link names a
/// fresh store path after every rebuild), so it is reported to the operator but never recorded.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SkippedDirectoryLinkRecord {
    pub path: String,
    pub target: String,
}

#[derive(Debug, Clone)]
pub struct FunctionAddressRun {
    pub findings: Vec<output::Finding>,
    pub artifact_json: String,
    pub skipped_directory_links: Vec<SkippedDirectoryLinkRecord>,
}

#[derive(Debug, Deserialize)]
struct BaselineReport {
    records: Vec<BaselineRecord>,
    non_claims: Option<Vec<String>>,
    claims: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct BaselineRecord {
    path: String,
    name: String,
    function_address: String,
    source_identity: Option<String>,
}

#[derive(Debug, Default)]
struct BaselineIndex {
    addresses: BTreeMap<String, BaselineAddress>,
    source_identities: BTreeMap<String, String>,
    issues: Vec<BaselineIssue>,
}

#[derive(Debug)]
struct BaselineAddress {
    path: String,
    name: String,
    function_address: String,
}

#[derive(Debug)]
struct BaselineIssue {
    lint: &'static str,
    path: String,
    name: String,
    function_address: String,
    message: &'static str,
}

#[derive(Debug)]
struct HandoffIssue {
    lint: &'static str,
    path: String,
    name: String,
    function_address: String,
    source_identity: String,
    line: u32,
    column: u32,
    message: String,
}

// Collection walks files, matches baselines, and renders the artifact as one sequential pipeline
// over shared run state. Callers validate declared scopes with `validate_paths` before Cargo runs,
// where the error can name the flag or config entry; collection itself still refuses any scope,
// nested `.rs` symlink, or nested directory symlink inside a declared scope that resolves outside
// the workspace root, and any record path that is not canonical and workspace-relative. The
// implicit whole-workspace scope skips escaping directory symlinks, such as a Nix `result` link,
// without following them, and records each link's workspace-relative path in
// `skipped_directory_links`.
#[allow(clippy::too_many_lines)]
pub fn collect_run(
    check: &FunctionAddressCheck,
    workspace_root: &Path,
    config_hash: &str,
) -> Result<Option<FunctionAddressRun>, String> {
    if !check.mode.is_enabled() {
        return Ok(None);
    }
    let canonical_root = std::fs::canonicalize(workspace_root)
        .map_err(|error| format!("canonicalize workspace root {}: {error}", workspace_root.display()))?;

    let mut files = Vec::new();
    let mut skipped_links = Vec::new();
    let mut scope_prefixes = Vec::new();
    let escaping_directories = if check.paths.is_empty() {
        fingerprint::EscapingDirectoryLinks::Skip
    } else {
        fingerprint::EscapingDirectoryLinks::Refuse
    };
    for path in check.enabled_paths(workspace_root) {
        let absolute = if path.is_absolute() {
            path
        } else {
            workspace_root.join(path)
        };
        fingerprint::collect_contained_rust_files(
            &absolute,
            &canonical_root,
            escaping_directories,
            &mut files,
            &mut skipped_links,
        )?;
        scope_prefixes.extend(checked_scope_prefix(workspace_root, &absolute));
    }
    files.sort();
    files.dedup();
    let mut skipped_directory_links = skipped_links
        .iter()
        .map(|skipped| {
            Ok(SkippedDirectoryLinkRecord {
                path: function_record_path(workspace_root, &skipped.link)?,
                target: determinism::normalize_path(&skipped.target),
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    skipped_directory_links.sort();
    skipped_directory_links.dedup();

    let baseline = load_baseline(check.baseline.as_deref(), workspace_root)?;
    let mut records = Vec::new();
    let mut findings = baseline
        .issues
        .iter()
        .map(|issue| finding_for_baseline_issue(issue, check.mode, config_hash))
        .collect::<Vec<_>>();
    let mut source_envelopes = BTreeMap::new();
    let mut seen_locators = BTreeMap::new();
    for file in files {
        let source = std::fs::read_to_string(&file).map_err(|error| format!("read {}: {error}", file.display()))?;
        let source_identity = source_identity(&source);
        let normalized_path = function_record_path(workspace_root, &file)?;
        source_envelopes.insert(normalized_path.clone(), SourceEnvelopeRecord {
            path: normalized_path.clone(),
            language: FUNCTION_ADDRESS_SOURCE_LANGUAGE,
            source_identity: source_identity.clone(),
        });
        for span in fingerprint::find_functions(&source) {
            let function_source = function_source_for_span(&source, &span)?;
            let normalized = fingerprint::normalize_tokens(function_source);
            let marker = function_address_marker(span.marker.as_ref(), &normalized);
            let record = function_address_record(
                &normalized_path,
                &source,
                &source_identity,
                &span,
                function_source,
                &normalized,
                marker,
            );
            let locator = function_locator(&record.path, &record.name);
            if seen_locators.insert(locator.clone(), record.function_address.clone()).is_some() {
                findings.push(finding_for_record(
                    &record,
                    check.mode,
                    LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR,
                    "function locator appears more than once in the checked scope",
                    config_hash,
                ));
            }
            if let Some(marker_finding) = marker_finding_for_record(&record, marker, check.mode, config_hash) {
                findings.push(marker_finding);
            }
            if let Some(expected) = baseline.addresses.get(&locator) {
                if expected.function_address != record.function_address {
                    findings.push(finding_for_record(
                        &record,
                        check.mode,
                        LINT_FUNCTION_ADDRESS_STALE,
                        "function address changed from reviewed baseline; update the reviewed receipt or inspect the change",
                        config_hash,
                    ));
                }
                if let Some(expected_source_identity) = baseline.source_identities.get(&locator)
                    && expected_source_identity != &record.source_identity
                {
                    findings.push(finding_for_record(
                        &record,
                        check.mode,
                        LINT_FUNCTION_ADDRESS_STALE,
                        "source envelope identity changed from reviewed baseline; update the reviewed receipt or inspect the change",
                        config_hash,
                    ));
                }
            } else if marker.is_none() {
                findings.push(finding_for_record(
                    &record,
                    check.mode,
                    LINT_FUNCTION_ADDRESS_MISSING,
                    "function has no reviewed content-address marker or baseline entry; run observe mode to baseline or add a reviewed receipt",
                    config_hash,
                ));
            }
            if record.dependency_identity_status == "unsupported-macro-context" {
                findings.push(finding_for_record(
                    &record,
                    check.mode,
                    LINT_FUNCTION_ADDRESS_UNSUPPORTED,
                    "function contains macro context that blocks dependency-closure identity; add reviewed caveat or exclusion before deny mode",
                    config_hash,
                ));
            }
            records.push(record);
        }
    }
    for (locator, expected) in &baseline.addresses {
        if !seen_locators.contains_key(locator) && path_in_checked_scope(&expected.path, &scope_prefixes) {
            let issue = BaselineIssue {
                lint: LINT_FUNCTION_ADDRESS_STALE,
                path: expected.path.clone(),
                name: expected.name.clone(),
                function_address: canonical_hash_identity(&expected.function_address),
                message: "reviewed baseline function is no longer present in the checked scope; it was deleted or renamed, so update the reviewed receipt or inspect the change",
            };
            findings.push(finding_for_baseline_issue(&issue, check.mode, config_hash));
        }
    }

    let source_envelopes = source_envelopes.into_values().collect::<Vec<_>>();
    let evidence_chain_handoff = function_address_evidence_chain_handoff(config_hash, &source_envelopes, &records)?;
    let mut report = FunctionAddressReport {
        schema: FUNCTION_ADDRESS_DOMAIN,
        schema_version: FUNCTION_ADDRESS_SCHEMA_VERSION,
        receipt_class: FUNCTION_ADDRESS_RECEIPT_CLASS,
        mode: mode_name(check.mode),
        domain: FUNCTION_ADDRESS_DOMAIN,
        canonicalizer: FUNCTION_ADDRESS_CANONICALIZER,
        producer: FunctionAddressProducer {
            name: FUNCTION_ADDRESS_PRODUCER,
            role: FUNCTION_ADDRESS_PRODUCER_ROLE,
        },
        tool: env!("CARGO_PKG_VERSION"),
        source_envelopes,
        skipped_directory_links: skipped_directory_links.iter().map(|skipped| skipped.path.clone()).collect(),
        records,
        diagnostic_count: findings.len(),
        kamacite_projection: KamaciteProjection {
            artifact_class: FUNCTION_ADDRESS_RECEIPT_CLASS,
            compatibility_projection: KAMACITE_COMPATIBILITY_PROJECTION,
            canonical_identity_owner: KAMACITE_CANONICAL_IDENTITY_OWNER,
            identity_algorithm: IDENTITY_ALGORITHM_BLAKE3,
        },
        preserves_handoff: PreservesHandoff {
            contract: PRESERVES_HANDOFF_CONTRACT,
            canonical_identity_owner: KAMACITE_CANONICAL_IDENTITY_OWNER,
            json_projection_role: JSON_PROJECTION_ROLE,
            rust_reparse_required: NO_REPARSE_REQUIREMENT,
            span_recomputation_required: NO_REPARSE_REQUIREMENT,
            required_record_fields: REQUIRED_HANDOFF_RECORD_FIELDS.to_vec(),
        },
        evidence_chain_handoff,
        non_claims: REQUIRED_NON_CLAIMS.to_vec(),
    };
    findings.extend(
        validate_preserves_handoff(&report)
            .iter()
            .map(|issue| finding_for_handoff_issue(issue, check.mode, config_hash)),
    );
    report.diagnostic_count = findings.len();
    let artifact_json = render_report_json(&report)?;

    Ok(Some(FunctionAddressRun {
        findings,
        artifact_json,
        skipped_directory_links,
    }))
}

pub fn render_report_json(report: &FunctionAddressReport) -> Result<String, String> {
    let mut rendered =
        serde_json::to_string_pretty(report).map_err(|error| format!("serialize function-address report: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

// r[impl octet.evidence_provenance.evidence_chain_handoff.function_address]
fn function_address_evidence_chain_handoff(
    config_hash: &str,
    source_envelopes: &[SourceEnvelopeRecord],
    records: &[FunctionAddressRecord],
) -> Result<evidence_chain::EvidenceChainHandoff, String> {
    let profile_payload = serde_json::json!({
        "source_envelopes": source_envelopes,
        "records": records,
        "canonicalizer": FUNCTION_ADDRESS_CANONICALIZER,
    });
    let profile_payload_json = serde_json::to_string(&profile_payload)
        .map_err(|error| format!("serialize function-address evidence-chain profile payload: {error}"))?;
    let profile_root = determinism::full_hash(&profile_payload_json);
    let run_identity = determinism::full_hash(
        &serde_json::json!({
            "schema": evidence_chain::EVIDENCE_CHAIN_HANDOFF_SCHEMA,
            "evidence_kind": evidence_chain::EVIDENCE_KIND_FUNCTION_ADDRESS,
            "profile_root": profile_root,
            "config_hash": config_hash,
            "tool": env!("CARGO_PKG_VERSION"),
        })
        .to_string(),
    );
    let mut parent_links = source_envelopes
        .iter()
        .map(|envelope| evidence_chain::TypedDigest {
            kind: evidence_chain::PARENT_KIND_SOURCE_ENVELOPE.to_owned(),
            digest: envelope.source_identity.clone(),
        })
        .collect::<Vec<_>>();
    if parent_links.is_empty() {
        parent_links.push(evidence_chain::TypedDigest {
            kind: EMPTY_SOURCE_ENVELOPE_SET_KIND.to_owned(),
            digest: determinism::full_hash(EMPTY_SOURCE_ENVELOPE_SET_KIND),
        });
    }

    Ok(evidence_chain::EvidenceChainHandoff {
        schema: evidence_chain::EVIDENCE_CHAIN_HANDOFF_SCHEMA.to_owned(),
        evidence_kind: evidence_chain::EVIDENCE_KIND_FUNCTION_ADDRESS.to_owned(),
        producer: evidence_chain::EvidenceChainProducer {
            name: FUNCTION_ADDRESS_PRODUCER.to_owned(),
            role: FUNCTION_ADDRESS_PRODUCER_ROLE.to_owned(),
            tool_version: env!("CARGO_PKG_VERSION").to_owned(),
        },
        run_identity,
        policy_hash: canonical_hash_identity(config_hash),
        profile_hash: determinism::full_hash(FUNCTION_ADDRESS_CANONICALIZER),
        canonical_authority: KAMACITE_CANONICAL_IDENTITY_OWNER.to_owned(),
        canonical_roots: vec![evidence_chain::TypedDigest {
            kind: evidence_chain::ROOT_KIND_FUNCTION_ADDRESS_RECORDS.to_owned(),
            digest: profile_root.clone(),
        }],
        parent_links,
        projections: vec![evidence_chain::EvidenceChainProjection {
            kind: evidence_chain::PROJECTION_KIND_COMPAT_JSON.to_owned(),
            artifact: FUNCTION_ADDRESS_EVIDENCE_CHAIN_PROJECTION_ARTIFACT.to_owned(),
            digest: profile_root,
            authority: evidence_chain::PROJECTION_AUTHORITY_NON_AUTHORITATIVE.to_owned(),
        }],
        non_claims: REQUIRED_NON_CLAIMS.iter().map(|non_claim| (*non_claim).to_owned()).collect(),
    })
}

fn canonical_hash_identity(value: &str) -> String {
    if is_blake3_identity(value) {
        value.to_owned()
    } else {
        determinism::full_hash(value)
    }
}

fn load_baseline(path: Option<&Path>, workspace_root: &Path) -> Result<BaselineIndex, String> {
    let Some(path) = path else {
        return Ok(BaselineIndex::default());
    };
    let contents = std::fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let report: BaselineReport =
        serde_json::from_str(&contents).map_err(|error| format!("parse {}: {error}", path.display()))?;
    let mut addresses = BTreeMap::new();
    let mut source_identities = BTreeMap::new();
    let mut issues = Vec::new();
    for record in report.records {
        let Some(normalized_path) = normalize_baseline_record_path(workspace_root, &record, &mut issues) else {
            continue;
        };
        let locator = function_locator(&normalized_path, &record.name);
        if !is_blake3_identity(&record.function_address) {
            issues.push(BaselineIssue {
                lint: LINT_FUNCTION_ADDRESS_MALFORMED,
                path: normalized_path.clone(),
                name: record.name.clone(),
                function_address: determinism::full_hash(&record.function_address),
                message: "baseline receipt contains a malformed BLAKE3 function address",
            });
        }
        if let Some(source_identity) = record.source_identity.as_ref() {
            source_identities.insert(locator.clone(), source_identity.clone());
        }
        if addresses
            .insert(locator, BaselineAddress {
                path: normalized_path.clone(),
                name: record.name.clone(),
                function_address: record.function_address.clone(),
            })
            .is_some()
        {
            issues.push(BaselineIssue {
                lint: LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR,
                path: normalized_path,
                name: record.name,
                function_address: record.function_address,
                message: "baseline receipt contains a duplicate function locator",
            });
        }
    }
    if !baseline_non_claims_cover_required(report.non_claims.as_deref()) {
        issues.push(BaselineIssue {
            lint: LINT_FUNCTION_ADDRESS_OVERCLAIM,
            path: BASELINE_RECEIPT_PATH.to_owned(),
            name: BASELINE_RECEIPT_LOCATOR.to_owned(),
            function_address: determinism::full_hash(BASELINE_RECEIPT_LOCATOR),
            message: "baseline receipt is missing required non-claim boundaries",
        });
    }
    for claim in report.claims.unwrap_or_default() {
        if is_forbidden_claim(&claim) {
            issues.push(BaselineIssue {
                lint: LINT_FUNCTION_ADDRESS_OVERCLAIM,
                path: BASELINE_RECEIPT_PATH.to_owned(),
                name: BASELINE_RECEIPT_LOCATOR.to_owned(),
                function_address: determinism::full_hash(&claim),
                message: "baseline receipt contains forbidden overclaim text",
            });
        }
    }
    Ok(BaselineIndex {
        addresses,
        source_identities,
        issues,
    })
}

fn function_locator(path: &str, name: &str) -> String {
    format!("{path}{LOCATOR_SEPARATOR}{name}")
}

/// Workspace-relative prefix of one checked scope path; empty means the whole
/// workspace. Scopes outside the workspace or with `..` cannot hold baseline
/// entries and yield `None`.
fn checked_scope_prefix(workspace_root: &Path, scope: &Path) -> Option<String> {
    let relative = scope.strip_prefix(workspace_root).ok()?;
    let mut segments = Vec::new();
    for component in relative.components() {
        match component {
            std::path::Component::Normal(segment) => segments.push(segment.to_str()?),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    Some(segments.join("/"))
}

fn path_in_checked_scope(path: &str, scope_prefixes: &[String]) -> bool {
    scope_prefixes.iter().any(|prefix| {
        prefix.is_empty()
            || path
                .strip_prefix(prefix.as_str())
                .is_some_and(|rest| rest.is_empty() || rest.starts_with(PATH_SEPARATOR))
    })
}

fn normalize_baseline_record_path(
    workspace_root: &Path,
    record: &BaselineRecord,
    issues: &mut Vec<BaselineIssue>,
) -> Option<String> {
    let raw_path = Path::new(&record.path);
    let normalized = if raw_path.is_absolute() {
        let Ok(relative) = raw_path.strip_prefix(workspace_root) else {
            issues.push(baseline_path_issue(record, "baseline receipt path is outside workspace root"));
            return None;
        };
        determinism::normalize_path(relative)
    } else {
        determinism::normalize_path_text(&record.path)
    };

    if is_canonical_relative_path(&normalized) {
        Some(normalized)
    } else {
        issues.push(baseline_path_issue(record, "baseline receipt path must be canonical and repo-relative"));
        None
    }
}

fn baseline_path_issue(record: &BaselineRecord, message: &'static str) -> BaselineIssue {
    BaselineIssue {
        lint: LINT_FUNCTION_ADDRESS_MALFORMED,
        path: BASELINE_RECEIPT_PATH.to_owned(),
        name: record.name.clone(),
        function_address: determinism::full_hash(&record.path),
        message,
    }
}

fn is_canonical_relative_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with(PATH_SEPARATOR) || path.contains(WINDOWS_PATH_SEPARATOR) {
        return false;
    }
    path.split(PATH_SEPARATOR).all(|segment| {
        !segment.is_empty() && segment != CURRENT_DIRECTORY_SEGMENT && segment != PARENT_DIRECTORY_SEGMENT
    })
}

fn mode_name(mode: FunctionAddressMode) -> &'static str {
    match mode {
        FunctionAddressMode::Off => "off",
        FunctionAddressMode::Observe => "observe",
        FunctionAddressMode::Warn => "warn",
        FunctionAddressMode::Deny => "deny",
    }
}

fn function_record_path(workspace_root: &Path, file: &Path) -> Result<String, String> {
    let relative = file
        .strip_prefix(workspace_root)
        .ok()
        .map(determinism::normalize_path)
        .filter(|relative| is_canonical_relative_path(relative))
        .ok_or_else(|| {
            format!(
                "function-address source path `{}` is outside workspace root `{}`",
                file.display(),
                workspace_root.display()
            )
        })?;
    Ok(relative)
}

/// The function-address marker for a span, if any. `cargo octet fingerprint`
/// writes the same `// octet-fingerprint:` syntax with its token hash; a marker
/// holding that hash is fingerprint evidence, not a reviewed function address,
/// so it neither satisfies nor stales the function-address check. A marker with
/// no hash stays a function-address marker and is reported as malformed.
fn function_address_marker<'a>(
    marker: Option<&'a fingerprint::Marker>,
    normalized: &str,
) -> Option<&'a fingerprint::Marker> {
    marker.filter(|marker| marker.stored_hash() != Some(fingerprint::function_hash(normalized).as_str()))
}

fn function_address_record(
    path: &str,
    source: &str,
    source_identity: &str,
    span: &fingerprint::FunctionSpan,
    function_source: &str,
    normalized: &str,
    marker: Option<&fingerprint::Marker>,
) -> FunctionAddressRecord {
    let dependency_set_hash = fingerprint::dependency_set_hash(function_source);
    let dependency_identity_status = fingerprint::dependency_identity_status(function_source).to_owned();
    let function_address = function_address(normalized, &dependency_set_hash, &dependency_identity_status);
    let (line, column) = line_column_for_offset(source, span.start);
    let mut caveats = Vec::new();
    if span.macro_calls {
        caveats.push("macro-context-review-required".to_owned());
    }

    FunctionAddressRecord {
        path: path.to_owned(),
        name: span.name.clone(),
        line,
        column,
        start_byte: span.start,
        end_byte: span.end,
        function_address,
        normalized_body_hash: determinism::full_hash(normalized),
        dependency_set_hash,
        dependency_identity_kind: DEPENDENCY_IDENTITY_KIND_SOURCE_TOKEN_SET.to_owned(),
        dependency_identity_status,
        source_identity: source_identity.to_owned(),
        review: marker.and_then(|marker| marker.review().map(str::to_owned)),
        evidence: marker.and_then(|marker| marker.evidence().map(str::to_owned)),
        caveats,
    }
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.chars().all(|character| character.is_ascii_hexdigit())
}

fn is_blake3_identity(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(is_blake3_hex)
}

fn marker_function_address(stored_hash: &str) -> String {
    format!("{BLAKE3_PREFIX}{stored_hash}")
}

#[cfg(test)]
fn marker_digest(function_address: &str) -> &str {
    function_address.strip_prefix(BLAKE3_PREFIX).unwrap_or(function_address)
}

fn baseline_non_claims_cover_required(non_claims: Option<&[String]>) -> bool {
    let Some(non_claims) = non_claims else {
        return false;
    };
    REQUIRED_NON_CLAIMS
        .iter()
        .all(|required| non_claims.iter().any(|non_claim| non_claim.as_str() == *required))
}

fn is_forbidden_claim(claim: &str) -> bool {
    let lowered = claim.to_ascii_lowercase();
    FORBIDDEN_CLAIM_FRAGMENTS.iter().any(|fragment| lowered.contains(fragment))
}

fn function_address(normalized: &str, dependency_set_hash: &str, dependency_identity_status: &str) -> String {
    let payload = serde_json::json!({
        "domain": FUNCTION_ADDRESS_DOMAIN,
        "canonicalizer": FUNCTION_ADDRESS_CANONICALIZER,
        "normalized": normalized,
        "dependency_set_hash": dependency_set_hash,
        "dependency_identity_status": dependency_identity_status,
        "tool": env!("CARGO_PKG_VERSION"),
    });
    determinism::full_hash(&payload.to_string())
}

fn source_identity(source: &str) -> String {
    determinism::full_hash(source)
}

fn function_source_for_span<'a>(source: &'a str, span: &fingerprint::FunctionSpan) -> Result<&'a str, String> {
    if span.start > span.end {
        return Err(format!(
            "function-address span for `{}` has start_byte {} greater than end_byte {}",
            span.name, span.start, span.end
        ));
    }
    source.get(span.start..span.end).ok_or_else(|| {
        format!(
            "function-address span for `{}` is outside source envelope or not on UTF-8 boundaries: start_byte={} end_byte={} source_len={}",
            span.name,
            span.start,
            span.end,
            source.len()
        )
    })
}

fn validate_preserves_handoff(report: &FunctionAddressReport) -> Vec<HandoffIssue> {
    let mut issues = Vec::new();
    validate_handoff_metadata(report, &mut issues);

    let mut envelopes = BTreeMap::new();
    for envelope in &report.source_envelopes {
        if !is_canonical_relative_path(&envelope.path) {
            issues.push(handoff_report_issue(
                LINT_FUNCTION_ADDRESS_MALFORMED,
                "source envelope path is not canonical and repo-relative",
            ));
        }
        if envelope.language != FUNCTION_ADDRESS_SOURCE_LANGUAGE {
            issues.push(handoff_report_issue(
                LINT_FUNCTION_ADDRESS_MALFORMED,
                "source envelope declares unsupported source language",
            ));
        }
        if !is_blake3_identity(&envelope.source_identity) {
            issues.push(handoff_report_issue(
                LINT_FUNCTION_ADDRESS_MALFORMED,
                "source envelope identity is not a BLAKE3 identity",
            ));
        }
        if envelopes.insert(envelope.path.as_str(), envelope).is_some() {
            issues.push(handoff_report_issue(
                LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR,
                "source envelope path appears more than once in the handoff",
            ));
        }
    }

    for record in &report.records {
        validate_handoff_record(record, &envelopes, &mut issues);
    }

    issues
}

fn validate_handoff_metadata(report: &FunctionAddressReport, issues: &mut Vec<HandoffIssue>) {
    if report.producer.name != FUNCTION_ADDRESS_PRODUCER || report.producer.role != FUNCTION_ADDRESS_PRODUCER_ROLE {
        issues.push(handoff_report_issue(
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address handoff declares unsupported producer metadata",
        ));
    }
    if report.kamacite_projection.canonical_identity_owner != KAMACITE_CANONICAL_IDENTITY_OWNER
        || report.preserves_handoff.canonical_identity_owner != KAMACITE_CANONICAL_IDENTITY_OWNER
    {
        issues.push(handoff_report_issue(
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address handoff does not delegate canonical identity to Kamacite Preserves bytes",
        ));
    }
    if report.preserves_handoff.contract != PRESERVES_HANDOFF_CONTRACT
        || report.preserves_handoff.json_projection_role != JSON_PROJECTION_ROLE
        || report.preserves_handoff.rust_reparse_required != NO_REPARSE_REQUIREMENT
        || report.preserves_handoff.span_recomputation_required != NO_REPARSE_REQUIREMENT
    {
        issues.push(handoff_report_issue(
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address Preserves handoff metadata is incomplete",
        ));
    }
    if !required_handoff_fields_are_declared(&report.preserves_handoff.required_record_fields) {
        issues.push(handoff_report_issue(
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address Preserves handoff omits required record fields",
        ));
    }
    if !report_non_claims_cover_required(&report.non_claims) {
        issues.push(handoff_report_issue(
            LINT_FUNCTION_ADDRESS_OVERCLAIM,
            "function-address handoff is missing required non-claim boundaries",
        ));
    }
    let evidence_chain_policy = evidence_chain::EvidenceChainValidationPolicy {
        required_canonical_authority: KAMACITE_CANONICAL_IDENTITY_OWNER,
        required_non_claims: REQUIRED_NON_CLAIMS,
    };
    for diagnostic in
        evidence_chain::validate_evidence_chain_handoff(&report.evidence_chain_handoff, evidence_chain_policy)
    {
        issues.push(handoff_report_issue(
            LINT_FUNCTION_ADDRESS_MALFORMED,
            format!("evidence-chain handoff invalid: {diagnostic}"),
        ));
    }
}

fn validate_handoff_record(
    record: &FunctionAddressRecord,
    envelopes: &BTreeMap<&str, &SourceEnvelopeRecord>,
    issues: &mut Vec<HandoffIssue>,
) {
    if !is_canonical_relative_path(&record.path) {
        issues.push(handoff_issue_for_record(
            record,
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address handoff record path is not canonical and repo-relative",
        ));
    }
    if record.start_byte >= record.end_byte {
        issues.push(handoff_issue_for_record(
            record,
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address handoff record has missing or invalid byte span",
        ));
    }
    if !is_blake3_identity(&record.function_address)
        || !is_blake3_identity(&record.normalized_body_hash)
        || !is_blake3_identity(&record.dependency_set_hash)
        || !is_blake3_identity(&record.source_identity)
    {
        issues.push(handoff_issue_for_record(
            record,
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address handoff record contains malformed BLAKE3 identity",
        ));
    }
    if record.dependency_identity_kind != DEPENDENCY_IDENTITY_KIND_SOURCE_TOKEN_SET {
        issues.push(handoff_issue_for_record(
            record,
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address handoff record is missing source-token dependency identity kind",
        ));
    }
    match envelopes.get(record.path.as_str()) {
        Some(envelope) if envelope.source_identity == record.source_identity => {}
        Some(_) => issues.push(handoff_issue_for_record(
            record,
            LINT_FUNCTION_ADDRESS_STALE,
            "function-address handoff record source identity does not match its source envelope",
        )),
        None => issues.push(handoff_issue_for_record(
            record,
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address handoff record has no matching source envelope",
        )),
    }
    if record_contains_forbidden_claim(record) {
        issues.push(handoff_issue_for_record(
            record,
            LINT_FUNCTION_ADDRESS_OVERCLAIM,
            "function-address handoff record contains forbidden overclaim text",
        ));
    }
}

fn required_handoff_fields_are_declared(fields: &[&str]) -> bool {
    REQUIRED_HANDOFF_RECORD_FIELDS.iter().all(|required| fields.iter().any(|field| field == required))
}

fn report_non_claims_cover_required(non_claims: &[&str]) -> bool {
    REQUIRED_NON_CLAIMS.iter().all(|required| non_claims.iter().any(|non_claim| non_claim == required))
}

fn record_contains_forbidden_claim(record: &FunctionAddressRecord) -> bool {
    record
        .review
        .iter()
        .chain(record.evidence.iter())
        .chain(record.caveats.iter())
        .any(|text| is_forbidden_claim(text))
}

fn handoff_report_issue(lint: &'static str, message: impl Into<String>) -> HandoffIssue {
    HandoffIssue {
        lint,
        path: FUNCTION_ADDRESS_ARTIFACT_NAME.to_owned(),
        name: BASELINE_RECEIPT_LOCATOR.to_owned(),
        function_address: determinism::full_hash(PRESERVES_HANDOFF_CONTRACT),
        source_identity: determinism::full_hash(PRESERVES_HANDOFF_CONTRACT),
        line: DEFAULT_LINE,
        column: DEFAULT_COLUMN,
        message: message.into(),
    }
}

fn handoff_issue_for_record(
    record: &FunctionAddressRecord,
    lint: &'static str,
    message: impl Into<String>,
) -> HandoffIssue {
    HandoffIssue {
        lint,
        path: record.path.clone(),
        name: record.name.clone(),
        function_address: record.function_address.clone(),
        source_identity: record.source_identity.clone(),
        line: record.line,
        column: record.column,
        message: message.into(),
    }
}

fn marker_finding_for_record(
    record: &FunctionAddressRecord,
    marker: Option<&fingerprint::Marker>,
    mode: FunctionAddressMode,
    config_hash: &str,
) -> Option<output::Finding> {
    let marker = marker?;
    let Some(stored_hash) = marker.stored_hash().filter(|stored_hash| is_blake3_hex(stored_hash)) else {
        return Some(finding_for_record(
            record,
            mode,
            LINT_FUNCTION_ADDRESS_MALFORMED,
            "function-address marker contains a malformed BLAKE3 digest",
            config_hash,
        ));
    };
    if marker_function_address(stored_hash) != record.function_address {
        return Some(finding_for_record(
            record,
            mode,
            LINT_FUNCTION_ADDRESS_STALE,
            "function-address marker does not match the computed function address",
            config_hash,
        ));
    }
    None
}

fn finding_for_handoff_issue(issue: &HandoffIssue, mode: FunctionAddressMode, config_hash: &str) -> output::Finding {
    let record = FunctionAddressRecord {
        path: issue.path.clone(),
        name: issue.name.clone(),
        line: issue.line,
        column: issue.column,
        start_byte: 0,
        end_byte: 0,
        function_address: issue.function_address.clone(),
        normalized_body_hash: issue.function_address.clone(),
        dependency_set_hash: issue.function_address.clone(),
        dependency_identity_kind: DEPENDENCY_IDENTITY_KIND_SOURCE_TOKEN_SET.to_owned(),
        dependency_identity_status: "preserves-handoff".to_owned(),
        source_identity: issue.source_identity.clone(),
        review: None,
        evidence: None,
        caveats: Vec::new(),
    };
    finding_for_record(&record, mode, issue.lint, &issue.message, config_hash)
}

fn finding_for_baseline_issue(issue: &BaselineIssue, mode: FunctionAddressMode, config_hash: &str) -> output::Finding {
    let record = FunctionAddressRecord {
        path: issue.path.clone(),
        name: issue.name.clone(),
        line: DEFAULT_LINE,
        column: DEFAULT_COLUMN,
        start_byte: 0,
        end_byte: 0,
        function_address: issue.function_address.clone(),
        normalized_body_hash: issue.function_address.clone(),
        dependency_set_hash: issue.function_address.clone(),
        dependency_identity_kind: DEPENDENCY_IDENTITY_KIND_SOURCE_TOKEN_SET.to_owned(),
        dependency_identity_status: "baseline-receipt".to_owned(),
        source_identity: issue.function_address.clone(),
        review: None,
        evidence: None,
        caveats: Vec::new(),
    };
    finding_for_record(&record, mode, issue.lint, issue.message, config_hash)
}

fn finding_for_record(
    record: &FunctionAddressRecord,
    mode: FunctionAddressMode,
    lint: &str,
    message: &str,
    config_hash: &str,
) -> output::Finding {
    let severity = severity_for_mode(mode);
    let semantic_id = determinism::semantic_finding_id(
        lint,
        "function-address",
        message,
        None,
        &record.function_address,
        Some(config_hash),
    );
    output::Finding {
        version: output::finding_version(),
        index: String::new(),
        stable_id: semantic_id.clone(),
        semantic_id,
        identity_status: "computed".to_owned(),
        normalized_snippet_hash: record.function_address.clone(),
        config_hash: config_hash.to_owned(),
        cache_status: "disabled".to_owned(),
        lint: lint.to_owned(),
        group: FUNCTION_ADDRESS_GROUP.to_owned(),
        severity: severity.to_owned(),
        file: record.path.clone(),
        line: record.line,
        column: record.column,
        message: format!(
            "{message}; function={} function_address={} source_identity={} canonicalizer={}",
            record.name, record.function_address, record.source_identity, FUNCTION_ADDRESS_CANONICALIZER
        ),
        suggestion: None,
        applicability: None,
        principle: FUNCTION_ADDRESS_PRINCIPLE.to_owned(),
        effect_family: FUNCTION_ADDRESS_EFFECT_FAMILY.to_owned(),
        evidence_tier: crate::evidence::EvidenceTier::StaticLint,
        crate_name: "function-address".to_owned(),
    }
}

fn severity_for_mode(mode: FunctionAddressMode) -> &'static str {
    match mode {
        FunctionAddressMode::Off | FunctionAddressMode::Observe => OBSERVE_SEVERITY,
        FunctionAddressMode::Warn => WARNING_SEVERITY,
        FunctionAddressMode::Deny => ERROR_SEVERITY,
    }
}

fn line_column_for_offset(source: &str, offset: usize) -> (u32, u32) {
    let mut line = DEFAULT_LINE;
    let mut column = DEFAULT_COLUMN;
    for byte in source.as_bytes().iter().take(offset) {
        if *byte == NEWLINE_BYTE {
            line = line.saturating_add(LINE_OFFSET);
            column = DEFAULT_COLUMN;
        } else {
            column = column.saturating_add(COLUMN_OFFSET);
        }
    }
    (line, column)
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    fn collect_test_findings(
        check: &FunctionAddressCheck,
        workspace_root: &Path,
        config_hash: &str,
    ) -> Vec<output::Finding> {
        collect_test_run(check, workspace_root, config_hash).findings
    }

    fn collect_test_run(check: &FunctionAddressCheck, workspace_root: &Path, config_hash: &str) -> FunctionAddressRun {
        collect_run(check, workspace_root, config_hash)
            .expect("collect run")
            .expect("enabled mode returns run")
    }

    fn artifact_value(run: &FunctionAddressRun) -> serde_json::Value {
        serde_json::from_str(&run.artifact_json).expect("parse artifact JSON")
    }

    fn valid_baseline_json(records: &serde_json::Value) -> String {
        serde_json::json!({
            "records": records,
            "non_claims": REQUIRED_NON_CLAIMS,
        })
        .to_string()
    }

    fn valid_preserves_handoff_report() -> FunctionAddressReport {
        const VALID_START_BYTE: usize = 1;
        const VALID_END_BYTE: usize = 12;
        const VALID_LINE: u32 = 1;
        const VALID_COLUMN: u32 = 2;
        const VALID_PATH: &str = "src/lib.rs";
        const VALID_FUNCTION: &str = "f";
        let source_identity = determinism::full_hash("source envelope bytes");
        let function_address = determinism::full_hash("function address payload");
        let normalized_body_hash = determinism::full_hash("normalized body");
        let dependency_set_hash = determinism::full_hash("dependency set");

        FunctionAddressReport {
            schema: FUNCTION_ADDRESS_DOMAIN,
            schema_version: FUNCTION_ADDRESS_SCHEMA_VERSION,
            receipt_class: FUNCTION_ADDRESS_RECEIPT_CLASS,
            mode: "observe",
            domain: FUNCTION_ADDRESS_DOMAIN,
            canonicalizer: FUNCTION_ADDRESS_CANONICALIZER,
            producer: FunctionAddressProducer {
                name: FUNCTION_ADDRESS_PRODUCER,
                role: FUNCTION_ADDRESS_PRODUCER_ROLE,
            },
            tool: env!("CARGO_PKG_VERSION"),
            source_envelopes: vec![SourceEnvelopeRecord {
                path: VALID_PATH.to_owned(),
                language: FUNCTION_ADDRESS_SOURCE_LANGUAGE,
                source_identity: source_identity.clone(),
            }],
            skipped_directory_links: Vec::new(),
            records: vec![FunctionAddressRecord {
                path: VALID_PATH.to_owned(),
                name: VALID_FUNCTION.to_owned(),
                line: VALID_LINE,
                column: VALID_COLUMN,
                start_byte: VALID_START_BYTE,
                end_byte: VALID_END_BYTE,
                function_address,
                normalized_body_hash,
                dependency_set_hash,
                dependency_identity_kind: DEPENDENCY_IDENTITY_KIND_SOURCE_TOKEN_SET.to_owned(),
                dependency_identity_status: "computed".to_owned(),
                source_identity,
                review: None,
                evidence: None,
                caveats: Vec::new(),
            }],
            diagnostic_count: 0,
            kamacite_projection: KamaciteProjection {
                artifact_class: FUNCTION_ADDRESS_RECEIPT_CLASS,
                compatibility_projection: KAMACITE_COMPATIBILITY_PROJECTION,
                canonical_identity_owner: KAMACITE_CANONICAL_IDENTITY_OWNER,
                identity_algorithm: IDENTITY_ALGORITHM_BLAKE3,
            },
            preserves_handoff: PreservesHandoff {
                contract: PRESERVES_HANDOFF_CONTRACT,
                canonical_identity_owner: KAMACITE_CANONICAL_IDENTITY_OWNER,
                json_projection_role: JSON_PROJECTION_ROLE,
                rust_reparse_required: NO_REPARSE_REQUIREMENT,
                span_recomputation_required: NO_REPARSE_REQUIREMENT,
                required_record_fields: REQUIRED_HANDOFF_RECORD_FIELDS.to_vec(),
            },
            evidence_chain_handoff: evidence_chain::EvidenceChainHandoff {
                schema: evidence_chain::EVIDENCE_CHAIN_HANDOFF_SCHEMA.to_owned(),
                evidence_kind: evidence_chain::EVIDENCE_KIND_FUNCTION_ADDRESS.to_owned(),
                producer: evidence_chain::EvidenceChainProducer {
                    name: FUNCTION_ADDRESS_PRODUCER.to_owned(),
                    role: FUNCTION_ADDRESS_PRODUCER_ROLE.to_owned(),
                    tool_version: env!("CARGO_PKG_VERSION").to_owned(),
                },
                run_identity: determinism::full_hash("run identity"),
                policy_hash: determinism::full_hash("policy"),
                profile_hash: determinism::full_hash("profile"),
                canonical_authority: KAMACITE_CANONICAL_IDENTITY_OWNER.to_owned(),
                canonical_roots: vec![evidence_chain::TypedDigest {
                    kind: evidence_chain::ROOT_KIND_FUNCTION_ADDRESS_RECORDS.to_owned(),
                    digest: determinism::full_hash("records"),
                }],
                parent_links: vec![evidence_chain::TypedDigest {
                    kind: evidence_chain::PARENT_KIND_SOURCE_ENVELOPE.to_owned(),
                    digest: determinism::full_hash("source envelope bytes"),
                }],
                projections: vec![evidence_chain::EvidenceChainProjection {
                    kind: evidence_chain::PROJECTION_KIND_COMPAT_JSON.to_owned(),
                    artifact: FUNCTION_ADDRESS_EVIDENCE_CHAIN_PROJECTION_ARTIFACT.to_owned(),
                    digest: determinism::full_hash("projection"),
                    authority: evidence_chain::PROJECTION_AUTHORITY_NON_AUTHORITATIVE.to_owned(),
                }],
                non_claims: REQUIRED_NON_CLAIMS.iter().map(|non_claim| (*non_claim).to_owned()).collect(),
            },
            non_claims: REQUIRED_NON_CLAIMS.to_vec(),
        }
    }

    #[test]
    fn adoption_modes_parse_and_classify() {
        assert_eq!(FunctionAddressMode::parse("off"), Ok(FunctionAddressMode::Off));
        assert_eq!(FunctionAddressMode::parse("observe"), Ok(FunctionAddressMode::Observe));
        assert_eq!(FunctionAddressMode::parse("warn"), Ok(FunctionAddressMode::Warn));
        assert_eq!(FunctionAddressMode::parse("deny"), Ok(FunctionAddressMode::Deny));
        assert!(FunctionAddressMode::parse("strict").is_err());
    }

    #[test]
    fn observe_mode_emits_non_failing_missing_marker_finding() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![source],
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_MISSING);
        assert_eq!(findings[0].group, FUNCTION_ADDRESS_GROUP);
        assert_eq!(findings[0].severity, OBSERVE_SEVERITY);
        assert_eq!(findings[0].file, "lib.rs");
        assert!(findings[0].message.contains(FUNCTION_ADDRESS_CANONICALIZER));
    }

    #[test]
    fn observe_mode_missing_marker_finding_renders_as_valid_output_domain() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![source],
            baseline: None,
        };

        let run = collect_test_run(&check, temp.path(), "config");
        let rendered = output::render_json_lines(&run.findings).expect("function-address findings render");

        assert!(rendered.contains(LINT_FUNCTION_ADDRESS_MISSING));
        assert!(artifact_value(&run)["records"].as_array().is_some_and(|records| records.len() == 1));
    }

    #[test]
    fn deny_mode_promotes_missing_marker_to_error() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].severity, ERROR_SEVERITY);
    }

    #[test]
    fn named_scope_that_is_missing_or_not_rust_fails_instead_of_checking_nothing() {
        let temp = TempDir::new().expect("temp dir");
        std::fs::write(temp.path().join("notes.txt"), "fn f() -> u32 { 1 }\n").expect("write non-rust file");
        for named in ["typo", "notes.txt"] {
            let check = FunctionAddressCheck {
                mode: FunctionAddressMode::Deny,
                paths: vec![PathBuf::from(named)],
                baseline: None,
            };
            let Err(error) = collect_run(&check, temp.path(), "config") else {
                panic!("{named} must not pass as an empty function-address scope");
            };
            assert!(error.contains(named), "{named}: {error}");
        }
    }

    #[test]
    fn explicit_scope_outside_workspace_root_is_rejected() {
        let temp = TempDir::new().expect("temp dir");
        let workspace = temp.path().join("workspace");
        let shared = temp.path().join("shared");
        std::fs::create_dir_all(workspace.join("src")).expect("create workspace source");
        std::fs::create_dir_all(shared.join("src")).expect("create shared source");
        std::fs::write(workspace.join("src/lib.rs"), "fn inside() -> u32 { 1 }\n").expect("write inside source");
        std::fs::write(shared.join("src/lib.rs"), "fn outside() -> u32 { 1 }\n").expect("write shared source");
        std::os::unix::fs::symlink(&shared, workspace.join("linked")).expect("symlink shared into workspace");
        let check_for = |path: PathBuf| FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![path],
            baseline: None,
        };

        for escaping in [
            PathBuf::from("../shared/src"),
            PathBuf::from("src/../../shared/src"),
            shared.join("src"),
            PathBuf::from("linked/src"),
        ] {
            let check = check_for(escaping.clone());
            if let Ok(run) = collect_run(&check, &workspace, "config") {
                let records = run.map(|run| artifact_value(&run)["records"].clone());
                panic!("{} must not be collected; got records {records:?}", escaping.display());
            }
            for (origin, named) in [
                (ScopeOrigin::FunctionAddressPathFlag, "`--function-address-path`"),
                (ScopeOrigin::CoreSourceScopes, "`core_source_scopes`"),
            ] {
                let Err(error) = check.validate_paths(&workspace, origin) else {
                    panic!("{} must be rejected before Cargo runs", escaping.display());
                };
                assert!(error.contains(named), "{}: {error}", escaping.display());
            }
        }

        for inside in [PathBuf::from("src"), PathBuf::from("./src"), workspace.join("src")] {
            let check = check_for(inside.clone());
            check
                .validate_paths(&workspace, ScopeOrigin::FunctionAddressPathFlag)
                .unwrap_or_else(|error| panic!("{}: {error}", inside.display()));
            let run = collect_test_run(&check, &workspace, "config");
            let artifact = artifact_value(&run);
            assert_eq!(artifact["records"][0]["path"], "src/lib.rs", "{}", inside.display());
        }
    }

    #[test]
    fn nested_symlink_resolving_outside_workspace_root_is_refused() {
        let temp = TempDir::new().expect("temp dir");
        let workspace = temp.path().join("workspace");
        let shared = temp.path().join("shared");
        std::fs::create_dir_all(workspace.join("src")).expect("create workspace source");
        std::fs::create_dir_all(workspace.join("vendored")).expect("create in-root link target");
        std::fs::create_dir_all(&shared).expect("create shared source");
        std::fs::write(workspace.join("src/lib.rs"), "fn inside() -> u32 { 1 }\n").expect("write inside source");
        std::fs::write(workspace.join("vendored/mod.rs"), "fn vendored() -> u32 { 1 }\n").expect("write vendored");
        std::fs::write(shared.join("lib.rs"), "fn outside() -> u32 { 1 }\n").expect("write shared source");
        std::fs::write(shared.join("notes.txt"), "not rust\n").expect("write shared note");
        std::os::unix::fs::symlink(workspace.join("vendored"), workspace.join("src/inner")).expect("in-root link");
        std::os::unix::fs::symlink(shared.join("notes.txt"), workspace.join("src/notes.txt")).expect("note link");
        std::os::unix::fs::symlink(temp.path().join("missing"), workspace.join("src/dangling")).expect("dangling");
        let observe = |paths: Vec<PathBuf>| FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths,
            baseline: None,
        };

        let run = collect_test_run(&observe(vec![PathBuf::from("src")]), &workspace, "config");
        let paths = artifact_value(&run)["source_envelopes"]
            .as_array()
            .expect("source envelopes")
            .iter()
            .map(|envelope| envelope["path"].as_str().expect("envelope path").to_owned())
            .collect::<Vec<_>>();
        assert_eq!(paths, ["src/inner/mod.rs", "src/lib.rs"], "in-root links are followed");

        // A declared scope refuses both escaping kinds; the implicit workspace scope still refuses
        // an escaping `.rs` file link but skips an escaping directory link.
        let declared = vec![PathBuf::from("src")];
        for (link, target, refusing_scopes) in [
            ("src/escape", shared.clone(), vec![declared.clone()]),
            ("src/escape.rs", shared.join("lib.rs"), vec![declared.clone(), Vec::new()]),
        ] {
            std::os::unix::fs::symlink(&target, workspace.join(link)).expect("escaping link");
            for scope in refusing_scopes {
                match collect_run(&observe(scope.clone()), &workspace, "config") {
                    Ok(run) => panic!(
                        "{link} under {scope:?} must be refused; got records {:?}",
                        run.map(|run| artifact_value(&run)["records"].clone())
                    ),
                    Err(error) => assert!(error.contains(link), "{link} under {scope:?}: {error}"),
                }
            }
            std::fs::remove_file(workspace.join(link)).expect("remove escaping link");
        }
    }

    #[test]
    fn implicit_workspace_scope_skips_and_records_outside_root_directory_links() {
        let temp = TempDir::new().expect("temp dir");
        let workspace = temp.path().join("workspace");
        let store = temp.path().join("store");
        let monorepo = temp.path().join("monorepo");
        std::fs::create_dir_all(workspace.join("src")).expect("create workspace source");
        std::fs::create_dir_all(workspace.join("crates")).expect("create workspace crates");
        std::fs::create_dir_all(workspace.join(".direnv/flake-inputs")).expect("create direnv inputs");
        std::fs::create_dir_all(store.join("docs/share")).expect("create store output without Rust");
        std::fs::create_dir_all(store.join("source/src")).expect("create store output with Rust");
        std::fs::create_dir_all(monorepo.join("shared/src")).expect("create linked-in crate");
        std::fs::write(workspace.join("src/lib.rs"), "fn inside() -> u32 { 1 }\n").expect("write inside source");
        std::fs::write(store.join("docs/share/readme.txt"), "not rust\n").expect("write store note");
        std::fs::write(store.join("source/src/lib.rs"), "fn outside() -> u32 { 1 }\n").expect("write store source");
        std::fs::write(monorepo.join("shared/src/lib.rs"), "fn shared() -> u32 { 1 }\n").expect("write crate source");
        let implicit = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: Vec::new(),
            baseline: None,
        };
        let mut result_artifacts = Vec::new();

        for (link, target) in [
            ("result", store.join("docs")),
            ("result", store.join("source")),
            (".direnv/flake-inputs/abc-source", store.join("source")),
            ("crates/shared", PathBuf::from("../../monorepo/shared")),
        ] {
            std::os::unix::fs::symlink(&target, workspace.join(link)).expect("escaping directory link");
            let run = collect_run(&implicit, &workspace, "config")
                .unwrap_or_else(|error| panic!("{link} -> {}: {error}", target.display()))
                .expect("enabled run");
            let artifact = artifact_value(&run);
            let paths = artifact["source_envelopes"]
                .as_array()
                .expect("source envelopes")
                .iter()
                .map(|envelope| envelope["path"].as_str().expect("envelope path").to_owned())
                .collect::<Vec<_>>();
            assert_eq!(paths, ["src/lib.rs"], "{link} -> {}", target.display());
            assert_eq!(artifact["records"][0]["path"], "src/lib.rs", "{link}");
            assert_eq!(
                artifact["skipped_directory_links"],
                serde_json::json!([link]),
                "{link} -> {}",
                target.display()
            );
            assert_eq!(
                run.skipped_directory_links,
                [SkippedDirectoryLinkRecord {
                    path: link.to_owned(),
                    target: target.display().to_string(),
                }],
                "the operator note keeps the link target"
            );
            if link == "result" {
                result_artifacts.push(run.artifact_json);
            }
            std::fs::remove_file(workspace.join(link)).expect("remove escaping link");
        }
        // A rebuilt Nix `result` names a new absolute store path; the artifact must not change.
        assert_eq!(result_artifacts.len(), 2);
        assert_eq!(result_artifacts[0], result_artifacts[1], "artifact bytes must not depend on link targets");

        std::os::unix::fs::symlink(store.join("source"), workspace.join("result")).expect("result link");
        std::os::unix::fs::symlink("../../monorepo/shared", workspace.join("crates/shared")).expect("crate link");
        let run = collect_test_run(&implicit, &workspace, "config");
        assert_eq!(artifact_value(&run)["skipped_directory_links"], serde_json::json!(["crates/shared", "result"]));
        std::fs::remove_file(workspace.join("result")).expect("remove result link");
        std::fs::remove_file(workspace.join("crates/shared")).expect("remove crate link");

        std::fs::create_dir_all(workspace.join("vendored")).expect("create in-root link target");
        std::fs::write(workspace.join("vendored/mod.rs"), "fn vendored() -> u32 { 1 }\n").expect("write vendored");
        std::os::unix::fs::symlink(workspace.join("vendored"), workspace.join("crates/inner")).expect("in-root link");
        let run = collect_test_run(&implicit, &workspace, "config");
        let artifact = artifact_value(&run);
        assert_eq!(artifact["skipped_directory_links"], serde_json::json!([]), "in-root links are followed");
        assert!(run.skipped_directory_links.is_empty());
        let declared = collect_test_run(
            &FunctionAddressCheck {
                paths: vec![PathBuf::from("src")],
                ..implicit
            },
            &workspace,
            "config",
        );
        assert_eq!(artifact_value(&declared)["skipped_directory_links"], serde_json::json!([]));
    }

    #[test]
    fn macro_context_gets_unsupported_finding() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() { println!(\"hi\"); }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Warn,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 2);
        assert!(findings.iter().any(|finding| finding.lint == LINT_FUNCTION_ADDRESS_MISSING));
        assert!(findings.iter().any(|finding| finding.lint == LINT_FUNCTION_ADDRESS_UNSUPPORTED));
        assert!(findings.iter().all(|finding| finding.severity == WARNING_SEVERITY));
    }

    #[test]
    fn enabled_run_renders_function_address_artifact_with_non_claims() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let run = collect_run(&check, temp.path(), "config").expect("collect run").expect("enabled mode returns run");

        assert_eq!(run.findings.len(), 1);
        let artifact = artifact_value(&run);
        assert_eq!(artifact["schema"], FUNCTION_ADDRESS_DOMAIN);
        assert_eq!(artifact["receipt_class"], FUNCTION_ADDRESS_RECEIPT_CLASS);
        assert_eq!(artifact["producer"]["role"], FUNCTION_ADDRESS_PRODUCER_ROLE);
        assert_eq!(artifact["records"][0]["dependency_identity_kind"], DEPENDENCY_IDENTITY_KIND_SOURCE_TOKEN_SET);
        assert_eq!(artifact["kamacite_projection"]["compatibility_projection"], KAMACITE_COMPATIBILITY_PROJECTION);
        assert_eq!(artifact["preserves_handoff"]["contract"], PRESERVES_HANDOFF_CONTRACT);
        assert_eq!(artifact["preserves_handoff"]["json_projection_role"], JSON_PROJECTION_ROLE);
        assert_eq!(artifact["preserves_handoff"]["span_recomputation_required"], NO_REPARSE_REQUIREMENT);
        assert_eq!(artifact["evidence_chain_handoff"]["schema"], evidence_chain::EVIDENCE_CHAIN_HANDOFF_SCHEMA);
        assert_eq!(artifact["evidence_chain_handoff"]["evidence_kind"], evidence_chain::EVIDENCE_KIND_FUNCTION_ADDRESS);
        assert_eq!(artifact["evidence_chain_handoff"]["canonical_authority"], KAMACITE_CANONICAL_IDENTITY_OWNER);
        assert!(run.artifact_json.contains(NON_CLAIM_SEMANTIC_CORRECTNESS));
        assert!(run.artifact_json.contains("\"source_envelopes\""));
        assert!(run.artifact_json.contains("\"records\""));
    }

    #[test]
    fn preserves_handoff_validation_accepts_complete_report() {
        let report = valid_preserves_handoff_report();

        let issues = validate_preserves_handoff(&report);

        assert!(issues.is_empty());
    }

    #[test]
    fn preserves_handoff_validation_rejects_incomplete_records_and_overclaims() {
        const INVALID_SPAN_BYTE: usize = 8;
        let mut report = valid_preserves_handoff_report();
        report.producer.role = "semantic-proof-producer";
        report.source_envelopes[0].source_identity = "not-a-blake3-identity".to_owned();
        report.records[0].path = "/absolute/src/lib.rs".to_owned();
        report.records[0].start_byte = INVALID_SPAN_BYTE;
        report.records[0].end_byte = INVALID_SPAN_BYTE;
        report.records[0].function_address = "not-a-blake3-identity".to_owned();
        report.records[0].dependency_identity_kind.clear();
        report.records[0].caveats.push("proves whole-program safety".to_owned());

        let issues = validate_preserves_handoff(&report);

        assert!(issues.iter().any(|issue| issue.message.contains("producer metadata")));
        assert!(issues.iter().any(|issue| issue.message.contains("source envelope identity")));
        assert!(issues.iter().any(|issue| issue.message.contains("repo-relative")));
        assert!(issues.iter().any(|issue| issue.message.contains("byte span")));
        assert!(issues.iter().any(|issue| issue.message.contains("BLAKE3")));
        assert!(issues.iter().any(|issue| issue.message.contains("source-token")));
        assert!(issues.iter().any(|issue| issue.lint == LINT_FUNCTION_ADDRESS_OVERCLAIM));
    }

    #[test]
    fn evidence_chain_handoff_validation_rejects_profile_neutral_boundary_errors() {
        let mut report = valid_preserves_handoff_report();
        report.evidence_chain_handoff.evidence_kind.clear();
        report.evidence_chain_handoff.policy_hash = "not-a-blake3-identity".to_owned();
        report
            .evidence_chain_handoff
            .canonical_roots
            .push(report.evidence_chain_handoff.canonical_roots[0].clone());
        report.evidence_chain_handoff.parent_links[0].kind.clear();
        report.evidence_chain_handoff.canonical_authority = "json".to_owned();
        report.evidence_chain_handoff.projections[0].authority = "canonical-json".to_owned();
        report.evidence_chain_handoff.non_claims.clear();

        let issues = validate_preserves_handoff(&report);

        assert!(issues.iter().any(|issue| issue.message.contains("evidence_kind")));
        assert!(issues.iter().any(|issue| issue.message.contains("policy_hash")));
        assert!(issues.iter().any(|issue| issue.message.contains("duplicate canonical root")));
        assert!(issues.iter().any(|issue| issue.message.contains("untyped parent")));
        assert!(issues.iter().any(|issue| issue.message.contains("JSON projection")));
        assert!(issues.iter().any(|issue| issue.message.contains("non-claim")));
    }

    #[test]
    fn exported_byte_span_slices_hashed_function_source() {
        let temp = TempDir::new().expect("temp dir");
        let source_path = temp.path().join("src/lib.rs");
        let source_text = "// header\n#[inline]\npub fn f() -> u32 { 1 }\n";
        std::fs::create_dir_all(source_path.parent().expect("source parent")).expect("create src dir");
        std::fs::write(&source_path, source_text).expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("src/lib.rs")],
            baseline: None,
        };

        let artifact = artifact_value(&collect_test_run(&check, temp.path(), "config"));
        let record = &artifact["records"][0];
        let start = usize::try_from(record["start_byte"].as_u64().expect("start byte")).expect("usize start byte");
        let end = usize::try_from(record["end_byte"].as_u64().expect("end byte")).expect("usize end byte");
        let slice = &source_text[start..end];

        assert!(slice.starts_with("#[inline]"));
        assert!(slice.contains("pub fn f"));
        assert_eq!(record["normalized_body_hash"], determinism::full_hash(&fingerprint::normalize_tokens(slice)));
    }

    #[test]
    fn invalid_byte_spans_fail_before_slicing_source() {
        const INVERTED_START_BYTE: usize = 2;
        const INVERTED_END_BYTE: usize = 1;
        const OUT_OF_RANGE_END_BYTE: usize = 100;
        let inverted = fingerprint::FunctionSpan {
            name: "f".to_owned(),
            start: INVERTED_START_BYTE,
            end: INVERTED_END_BYTE,
            marker: None,
            marker_block: 0..0,
            macro_calls: false,
        };
        let out_of_range = fingerprint::FunctionSpan {
            name: "g".to_owned(),
            start: 0,
            end: OUT_OF_RANGE_END_BYTE,
            marker: None,
            marker_block: 0..0,
            macro_calls: false,
        };

        let inverted_error = function_source_for_span("fn f() {}", &inverted).expect_err("inverted span fails");
        let range_error = function_source_for_span("fn g() {}", &out_of_range).expect_err("out-of-range span fails");

        assert!(inverted_error.contains("start_byte"));
        assert!(range_error.contains("outside source envelope"));
    }

    #[test]
    fn relocated_checkout_emits_stable_relative_paths_and_locators() {
        let first = TempDir::new().expect("first temp dir");
        let second = TempDir::new().expect("second temp dir");
        for root in [first.path(), second.path()] {
            let src = root.join("src");
            std::fs::create_dir_all(&src).expect("create src dir");
            std::fs::write(src.join("lib.rs"), "pub fn f() -> u32 { 1 }\n").expect("write source");
        }
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("src/lib.rs")],
            baseline: None,
        };

        let first_artifact = artifact_value(&collect_test_run(&check, first.path(), "config"));
        let second_artifact = artifact_value(&collect_test_run(&check, second.path(), "config"));

        assert_eq!(first_artifact["source_envelopes"][0]["path"], "src/lib.rs");
        assert_eq!(first_artifact["records"][0]["path"], "src/lib.rs");
        assert_eq!(first_artifact["source_envelopes"], second_artifact["source_envelopes"]);
        assert_eq!(first_artifact["records"][0]["path"], second_artifact["records"][0]["path"]);
        assert_eq!(first_artifact["records"][0]["function_address"], second_artifact["records"][0]["function_address"]);
        assert_eq!(function_locator("src/lib.rs", "f"), function_locator("src/lib.rs", "f"));
    }

    #[test]
    fn baseline_suppresses_unchanged_missing_marker_debt() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let baseline_run = collect_run(&observe, temp.path(), "config")
            .expect("collect baseline")
            .expect("enabled mode returns run");
        std::fs::write(&baseline, baseline_run.artifact_json).expect("write baseline");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&deny, temp.path(), "config");

        assert!(findings.is_empty());
    }

    #[test]
    fn baseline_reports_stale_function_address() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let baseline_run = collect_run(&observe, temp.path(), "config")
            .expect("collect baseline")
            .expect("enabled mode returns run");
        std::fs::write(&baseline, baseline_run.artifact_json).expect("write baseline");
        std::fs::write(&source, "fn f() -> u32 { 2 }\n").expect("write changed source");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&deny, temp.path(), "config");

        assert_eq!(findings.len(), 2);
        assert!(findings.iter().all(|finding| finding.lint == LINT_FUNCTION_ADDRESS_STALE));
        assert!(findings.iter().all(|finding| finding.severity == ERROR_SEVERITY));
    }

    #[test]
    fn baseline_reports_stale_source_identity_without_function_change() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let baseline_run = collect_test_run(&observe, temp.path(), "config");
        std::fs::write(&baseline, baseline_run.artifact_json).expect("write baseline");
        std::fs::write(&source, "// source envelope changed\nfn f() -> u32 { 1 }\n").expect("write changed source");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&deny, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_STALE);
        assert!(findings[0].message.contains("source envelope identity changed"));
    }

    #[test]
    fn baseline_reports_deleted_or_renamed_function_as_stale() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&source, "fn f() -> u32 { 1 }\nfn g() -> u32 { 2 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let baseline_run = collect_test_run(&observe, temp.path(), "config");
        let reviewed_g = artifact_value(&baseline_run)["records"][1]["function_address"]
            .as_str()
            .expect("reviewed address for g")
            .to_owned();
        std::fs::write(&baseline, baseline_run.artifact_json).expect("write baseline");
        for mode in [FunctionAddressMode::Warn, FunctionAddressMode::Deny] {
            let check = FunctionAddressCheck {
                mode,
                paths: vec![PathBuf::from("lib.rs")],
                baseline: Some(baseline.clone()),
            };
            for edited in [
                "// g deleted\nfn f() -> u32 { 1 }\n",
                "// g renamed\nfn f() -> u32 { 1 }\nfn h() -> u32 { 2 }\n",
            ] {
                std::fs::write(&source, edited).expect("write edited source");

                let findings = collect_test_findings(&check, temp.path(), "config");

                let removed =
                    findings.iter().filter(|finding| finding.message.contains("function=g ")).collect::<Vec<_>>();
                assert_eq!(removed.len(), 1, "{edited}: {findings:#?}");
                assert_eq!(removed[0].lint, LINT_FUNCTION_ADDRESS_STALE);
                assert_eq!(removed[0].severity, severity_for_mode(mode));
                assert_eq!(removed[0].file, "lib.rs");
                assert!(removed[0].message.contains("no longer present"));
                assert!(removed[0].message.contains(&reviewed_g));
                output::render_json_lines(&findings).expect("stale baseline entry renders");
            }
        }
    }

    #[test]
    fn baseline_entries_outside_checked_scope_are_not_reported_stale() {
        let temp = TempDir::new().expect("temp dir");
        let core = temp.path().join("core/lib.rs");
        let shell = temp.path().join("shell/lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        for path in [&core, &shell] {
            std::fs::create_dir_all(path.parent().expect("source parent")).expect("create source dir");
            std::fs::write(path, "fn f() -> u32 { 1 }\n").expect("write source");
        }
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: Vec::new(),
            baseline: None,
        };
        let baseline_run = collect_test_run(&observe, temp.path(), "config");
        std::fs::write(&baseline, baseline_run.artifact_json).expect("write baseline");
        let core_only = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("core/")],
            baseline: Some(baseline),
        };

        assert!(collect_test_findings(&core_only, temp.path(), "config").is_empty());
    }

    #[test]
    fn baseline_receipt_missing_non_claims_reports_overclaim_boundary() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&baseline, serde_json::json!({"records": []}).to_string()).expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_OVERCLAIM);
    }

    #[test]
    fn matching_marker_satisfies_deny_mode_and_exports_review_fields() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let observed = collect_test_run(&observe, temp.path(), "config");
        let address = artifact_value(&observed)["records"][0]["function_address"]
            .as_str()
            .expect("function address")
            .to_owned();
        std::fs::write(
            &source,
            format!(
                "// octet-fingerprint: b3:{}; review=R-1; evidence=E-1\nfn f() -> u32 {{ 1 }}\n",
                marker_digest(&address)
            ),
        )
        .expect("write marked source");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let run = collect_test_run(&deny, temp.path(), "config");
        let artifact = artifact_value(&run);

        assert!(run.findings.is_empty());
        assert_eq!(artifact["records"][0]["review"], "R-1");
        assert_eq!(artifact["records"][0]["evidence"], "E-1");
    }

    #[test]
    fn marker_text_inside_block_comment_cannot_supply_a_reviewed_address() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        for prefix in [
            "/* nested /* comment */\n// octet-fingerprint: b3:",
            "/*\n#[fingerprint(b3:",
        ] {
            let unreviewed =
                format!("{prefix}{}; review=R-stale; evidence=E-stale\n*/ pub fn f() -> u32 {{ 1 }}\n", "0".repeat(64));
            std::fs::write(&source, &unreviewed).expect("write forged source");
            let observed = collect_test_run(&check, temp.path(), "config");
            let address = artifact_value(&observed)["records"][0]["function_address"]
                .as_str()
                .expect("observed address")
                .to_owned();
            let reviewed = unreviewed.replace(&"0".repeat(64), marker_digest(&address));
            std::fs::write(&source, reviewed).expect("write matching forged digest");

            let denied = collect_test_run(&check, temp.path(), "config");
            assert_eq!(denied.findings.len(), 1, "{prefix}");
            assert_eq!(denied.findings[0].lint, LINT_FUNCTION_ADDRESS_MISSING, "{prefix}");
            assert!(artifact_value(&denied)["records"][0]["review"].is_null(), "{prefix}");
        }
    }

    #[test]
    fn reviewed_marker_goes_stale_when_comment_separated_export_name_changes() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let alpha = "#[unsafe(export_name = \"alpha\")]\n// review gap\npub extern \"C\" fn exported() -> u32 { 1 }\n";
        std::fs::write(&source, alpha).expect("write alpha source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let observed = collect_test_run(&check, temp.path(), "config");
        let reviewed_address = artifact_value(&observed)["records"][0]["function_address"]
            .as_str()
            .expect("alpha function address")
            .to_owned();
        let reviewed = format!(
            "// octet-fingerprint: b3:{}; review=R-export; evidence=E-export\n{alpha}",
            marker_digest(&reviewed_address)
        );
        std::fs::write(&source, &reviewed).expect("write reviewed marker");
        assert!(collect_test_run(&check, temp.path(), "config").findings.is_empty());

        std::fs::write(&source, reviewed.replace("alpha", "omega")).expect("change exported symbol");
        let changed = collect_test_run(&check, temp.path(), "config");
        assert_eq!(changed.findings.len(), 1);
        assert_eq!(changed.findings[0].lint, LINT_FUNCTION_ADDRESS_STALE);
        assert_ne!(artifact_value(&changed)["records"][0]["function_address"], reviewed_address);
    }

    #[test]
    fn reviewed_marker_stales_on_multiline_export_and_split_visibility_edits() {
        for (original, edited) in [
            (
                "#[unsafe(export_name =\n    \"alpha\"\n)]\n// gap\npub extern \"C\" fn exported() -> u32 { 1 }\n",
                "#[unsafe(export_name =\n    \"omega\"\n)]\n// gap\npub extern \"C\" fn exported() -> u32 { 1 }\n",
            ),
            ("pub\nfn exported() -> u32 { 1 }\n", "fn exported() -> u32 { 1 }\n"),
        ] {
            let temp = TempDir::new().expect("temp dir");
            let source = temp.path().join("lib.rs");
            let check = FunctionAddressCheck {
                mode: FunctionAddressMode::Deny,
                paths: vec![PathBuf::from("lib.rs")],
                baseline: None,
            };
            std::fs::write(&source, original).expect("write original function");
            let observed = collect_test_run(&check, temp.path(), "config");
            let reviewed_address = artifact_value(&observed)["records"][0]["function_address"]
                .as_str()
                .expect("original function address")
                .to_owned();
            let marker = format!(
                "// octet-fingerprint: b3:{}; review=R-export; evidence=E-export\n",
                marker_digest(&reviewed_address)
            );
            std::fs::write(&source, format!("{marker}{original}")).expect("write original review marker");
            assert!(collect_test_run(&check, temp.path(), "config").findings.is_empty());

            std::fs::write(&source, format!("{marker}{edited}")).expect("change reviewed source identity");
            let changed = collect_test_run(&check, temp.path(), "config");
            assert_eq!(changed.findings.len(), 1, "{edited}");
            assert_eq!(changed.findings[0].lint, LINT_FUNCTION_ADDRESS_STALE);
            assert_ne!(artifact_value(&changed)["records"][0]["function_address"], reviewed_address);
        }
    }

    #[test]
    fn stale_marker_reports_stale_function_address() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let wrong_address = determinism::full_hash("wrong-address");
        std::fs::write(
            &source,
            format!(
                "// octet-fingerprint: b3:{}; review=R; evidence=E\nfn f() -> u32 {{ 1 }}\n",
                marker_digest(&wrong_address)
            ),
        )
        .expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_STALE);
    }

    #[test]
    fn fingerprint_writer_marker_is_not_a_function_address_marker() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let written = fingerprint::run_in(temp.path(), &crate::cli::FingerprintArgs {
            mode: crate::cli::FingerprintMode::Write,
            paths: vec![source.clone()],
            output_format: crate::cli::OutputFormat::Json,
            review_manifest: None,
        })
        .expect("fingerprint write");
        assert_eq!(written, 0);
        let marked = std::fs::read_to_string(&source).expect("read fingerprinted source");
        assert!(marked.starts_with("// octet-fingerprint: b3:"), "{marked}");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let unbaselined = collect_test_run(&deny, temp.path(), "config");
        let lints = unbaselined.findings.iter().map(|finding| finding.lint.as_str()).collect::<Vec<_>>();
        assert_eq!(lints, [LINT_FUNCTION_ADDRESS_MISSING]);
        assert!(artifact_value(&unbaselined)["records"][0]["review"].is_null());

        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            ..deny.clone()
        };
        std::fs::write(&baseline, collect_test_run(&observe, temp.path(), "config").artifact_json)
            .expect("write baseline");
        let baselined = FunctionAddressCheck {
            baseline: Some(baseline),
            ..deny
        };

        let findings = collect_test_findings(&baselined, temp.path(), "config");

        assert!(findings.is_empty(), "{findings:#?}");
    }

    #[test]
    fn malformed_marker_reports_malformed_digest() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "// octet-fingerprint: b3:abc; review=R; evidence=E\nfn f() -> u32 { 1 }\n")
            .expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_MALFORMED);
    }

    #[test]
    fn duplicate_source_locator_reports_duplicate_locator() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(&source, "fn f() -> u32 { 1 }\nfn f() -> u32 { 2 }\n").expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Warn,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert!(findings.iter().any(|finding| finding.lint == LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR));
    }

    #[test]
    fn documentation_examples_do_not_create_missing_or_duplicate_locators() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(
            &source,
            "/// ```compile_fail\n/// fn forge() {}\n/// ```\n///\n/// ```compile_fail\n/// fn forge() {}\n/// ```\npub fn real() -> u32 { 1 }\n",
        )
        .expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let run = collect_test_run(&check, temp.path(), "config");

        assert_eq!(run.findings.len(), 1);
        assert_eq!(run.findings[0].lint, LINT_FUNCTION_ADDRESS_MISSING);
        assert!(run.findings[0].message.contains("; function=real "));
        let artifact = artifact_value(&run);
        let records = artifact["records"].as_array().expect("records array");
        let names = records.iter().map(|record| record["name"].as_str()).collect::<Vec<_>>();
        assert_eq!(names, [Some("real")]);
    }

    #[test]
    fn array_typed_signatures_are_checked_instead_of_skipped() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let text = "trait Pairs {\n    fn first(&self) -> [u8; 2];\n}\nfn pair(bytes: [u8; 2]) -> [bool; 2] {\n    [bytes[0] == 0, bytes[1] == 0]\n}\n";
        std::fs::write(&source, text).expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let run = collect_test_run(&check, temp.path(), "config");

        assert_eq!(run.findings.len(), 1);
        assert_eq!(run.findings[0].lint, LINT_FUNCTION_ADDRESS_MISSING);
        assert!(run.findings[0].message.contains("; function=pair "));
        let artifact = artifact_value(&run);
        let records = artifact["records"].as_array().expect("records array");
        assert_eq!(records.len(), 1);
        assert_eq!(records[0]["start_byte"], text.find("fn pair").expect("pair definition"));
        assert_eq!(records[0]["end_byte"], text.trim_end().len());
    }

    #[test]
    fn macro_context_follows_real_macro_calls_not_keywords_or_quoted_text() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        std::fs::write(
            &source,
            "fn guarded(ready: bool) -> bool { if !(ready) { return false; } true }\n\
             fn quoted() -> &'static str { \"fake!(x) at https://example.invalid\" }\n\
             fn after_url() -> bool { let _url = \"https://example.invalid\"; assert!(true); true }\n\
             fn listed() -> Vec<u8> { vec![1] }\n",
        )
        .expect("write source");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };

        let run = collect_test_run(&check, temp.path(), "config");

        let artifact = artifact_value(&run);
        let statuses = artifact["records"]
            .as_array()
            .expect("records array")
            .iter()
            .map(|record| (record["name"].as_str(), record["dependency_identity_status"].as_str()))
            .collect::<Vec<_>>();
        assert_eq!(statuses, [
            (Some("guarded"), Some("source-scanner-token-set")),
            (Some("quoted"), Some("source-scanner-token-set")),
            (Some("after_url"), Some("unsupported-macro-context")),
            (Some("listed"), Some("unsupported-macro-context")),
        ]);
        let unsupported = run
            .findings
            .iter()
            .filter(|finding| finding.lint == LINT_FUNCTION_ADDRESS_UNSUPPORTED)
            .filter_map(|finding| finding.message.split("; function=").nth(1)?.split(' ').next())
            .collect::<Vec<_>>();
        assert_eq!(unsupported, ["after_url", "listed"]);
        assert!(run.findings.iter().all(|finding| finding.severity == ERROR_SEVERITY));
    }

    #[test]
    fn reviewed_marker_fails_closed_for_code_after_quoted_slashes() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("lib.rs");
        let reviewed = "pub fn fetch() -> u32 {\n    let _endpoint = \"https://example.invalid/api\"; 1\n}\n";
        std::fs::write(&source, reviewed).expect("write reviewed source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("lib.rs")],
            baseline: None,
        };
        let reviewed_run = collect_test_run(&observe, temp.path(), "config");
        let address = artifact_value(&reviewed_run)["records"][0]["function_address"]
            .as_str()
            .expect("reviewed function address")
            .to_owned();
        let marker = format!("// octet-fingerprint: {address}; review=R-1; evidence=E-1\n");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            ..observe
        };

        let comment_only = reviewed.replace("; 1", "; 1 // still one");
        std::fs::write(&source, format!("{marker}{comment_only}")).expect("write comment-only edit");
        assert!(collect_test_findings(&deny, temp.path(), "config").is_empty());

        let aborts = reviewed.replace("; 1", "; std::process::abort()");
        std::fs::write(&source, format!("{marker}{aborts}")).expect("write hidden behavior edit");
        let findings = collect_test_findings(&deny, temp.path(), "config");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_STALE);
        assert!(findings[0].message.contains("; function=fetch "));
    }

    #[test]
    fn duplicate_baseline_receipt_reports_duplicate_locator() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(
            &baseline,
            valid_baseline_json(&serde_json::json!([
                {"path": "src/lib.rs", "name": "f", "function_address": determinism::full_hash("one")},
                {"path": "src/lib.rs", "name": "f", "function_address": determinism::full_hash("two")}
            ])),
        )
        .expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        let lints = findings.iter().map(|finding| finding.lint.as_str()).collect::<Vec<_>>();
        assert_eq!(lints, [LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR, LINT_FUNCTION_ADDRESS_STALE]);
        assert!(findings[1].message.contains("no longer present"));
    }

    #[test]
    fn absolute_baseline_path_under_workspace_matches_relative_record() {
        let temp = TempDir::new().expect("temp dir");
        let source = temp.path().join("src/lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::create_dir_all(source.parent().expect("source parent")).expect("create src dir");
        std::fs::write(&source, "fn f() -> u32 { 1 }\n").expect("write source");
        let observe = FunctionAddressCheck {
            mode: FunctionAddressMode::Observe,
            paths: vec![PathBuf::from("src/lib.rs")],
            baseline: None,
        };
        let mut baseline_value = artifact_value(&collect_test_run(&observe, temp.path(), "config"));
        baseline_value["records"][0]["path"] = serde_json::json!(source.display().to_string());
        std::fs::write(&baseline, baseline_value.to_string()).expect("write baseline");
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: vec![PathBuf::from("src/lib.rs")],
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&deny, temp.path(), "config");

        assert!(findings.is_empty());
    }

    #[test]
    fn baseline_path_outside_workspace_is_rejected_with_valid_finding_domain() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(
            &baseline,
            valid_baseline_json(&serde_json::json!([
                {"path": "/outside/src/lib.rs", "name": "f", "function_address": determinism::full_hash("one")}
            ])),
        )
        .expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_MALFORMED);
        assert_eq!(findings[0].file, BASELINE_RECEIPT_PATH);
        assert!(findings[0].message.contains("outside workspace root"));
        output::render_json_lines(&findings).expect("path diagnostic renders");
    }

    #[test]
    fn baseline_path_escape_is_rejected_with_valid_finding_domain() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(
            &baseline,
            valid_baseline_json(&serde_json::json!([
                {"path": "src/../lib.rs", "name": "f", "function_address": determinism::full_hash("one")}
            ])),
        )
        .expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Warn,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_MALFORMED);
        assert_eq!(findings[0].severity, WARNING_SEVERITY);
        assert!(findings[0].message.contains("canonical and repo-relative"));
        output::render_json_lines(&findings).expect("escape diagnostic renders");
    }

    #[test]
    fn duplicate_baseline_receipt_reports_duplicate_after_absolute_path_normalization() {
        let temp = TempDir::new().expect("temp dir");
        let absolute_path = temp.path().join("src/lib.rs");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(
            &baseline,
            valid_baseline_json(&serde_json::json!([
                {"path": absolute_path.display().to_string(), "name": "f", "function_address": determinism::full_hash("one")},
                {"path": "src/lib.rs", "name": "f", "function_address": determinism::full_hash("two")}
            ])),
        )
        .expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        let lints = findings.iter().map(|finding| finding.lint.as_str()).collect::<Vec<_>>();
        assert_eq!(lints, [LINT_FUNCTION_ADDRESS_DUPLICATE_LOCATOR, LINT_FUNCTION_ADDRESS_STALE]);
        assert!(findings.iter().all(|finding| finding.file == "src/lib.rs"));
    }

    #[test]
    fn malformed_baseline_receipt_reports_valid_warn_and_deny_domains() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        std::fs::write(
            &baseline,
            valid_baseline_json(&serde_json::json!([
                {"path": "src/lib.rs", "name": "f", "function_address": "not-a-blake3-address"}
            ])),
        )
        .expect("write baseline");
        let warn = FunctionAddressCheck {
            mode: FunctionAddressMode::Warn,
            paths: Vec::new(),
            baseline: Some(baseline.clone()),
        };
        let deny = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let warn_findings = collect_test_findings(&warn, temp.path(), "config");
        let deny_findings = collect_test_findings(&deny, temp.path(), "config");

        let lints = warn_findings.iter().map(|finding| finding.lint.as_str()).collect::<Vec<_>>();
        assert_eq!(lints, [LINT_FUNCTION_ADDRESS_MALFORMED, LINT_FUNCTION_ADDRESS_STALE]);
        assert!(warn_findings.iter().all(|finding| finding.severity == WARNING_SEVERITY));
        assert_eq!(deny_findings.len(), lints.len());
        assert!(deny_findings.iter().all(|finding| finding.severity == ERROR_SEVERITY));
        output::render_json_lines(&warn_findings).expect("warn findings render");
        output::render_json_lines(&deny_findings).expect("deny findings render");
    }

    #[test]
    fn baseline_receipt_overclaim_reports_overclaim() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        let mut receipt = serde_json::from_str::<serde_json::Value>(&valid_baseline_json(&serde_json::json!([])))
            .expect("baseline JSON");
        receipt["claims"] = serde_json::json!(["proves release eligibility"]);
        std::fs::write(&baseline, receipt.to_string()).expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Deny,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_OVERCLAIM);
    }

    #[test]
    fn baseline_receipt_dependency_closure_overclaim_reports_overclaim() {
        let temp = TempDir::new().expect("temp dir");
        let baseline = temp.path().join("function-addresses.json");
        let mut receipt = serde_json::from_str::<serde_json::Value>(&valid_baseline_json(&serde_json::json!([])))
            .expect("baseline JSON");
        receipt["claims"] = serde_json::json!(["records complete semantic closure"]);
        std::fs::write(&baseline, receipt.to_string()).expect("write baseline");
        let check = FunctionAddressCheck {
            mode: FunctionAddressMode::Warn,
            paths: Vec::new(),
            baseline: Some(baseline),
        };

        let findings = collect_test_findings(&check, temp.path(), "config");

        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].lint, LINT_FUNCTION_ADDRESS_OVERCLAIM);
        assert_eq!(findings[0].severity, WARNING_SEVERITY);
    }
}
