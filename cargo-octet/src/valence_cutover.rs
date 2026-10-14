use serde::Serialize;

const REQUIRED_NON_CLAIM: &str =
    "standalone Valence cutover proves dependency/source ownership and DTO compatibility only";
const REQUIRED_COORDINATE_MARKER: &str = "OnixResearch/valence";
const REQUIRED_CRATE_TARGET: &str = "valence-core";
const DISPOSITION_COMPATIBLE: &str = "compatible";
const DISPOSITION_SHIM: &str = "octet-hosted-shim";
const DISPOSITION_EXCEPTION: &str = "migration-exception";
const FIELD_STANDALONE_SOURCE: &str = "standalone_source";
const FIELD_MODULE: &str = "module";
const FIELD_FIXTURE: &str = "fixture";
const FIELD_DRIFT: &str = "drift";
const FIELD_NON_CLAIMS: &str = "non_claims";
const FIELD_SUPPORTED_CLAIM: &str = "supported_claim";
const FIELD_DISPOSITION: &str = "compatibility_disposition";
const FIELD_DIGEST: &str = "digest";
const FIELD_FIXTURE_REFS: &str = "fixture_refs";
const ISSUE_EMPTY_MODULES: &str = "module_rows_missing";
const ISSUE_EMPTY_FIXTURES: &str = "fixture_rows_missing";
const ISSUE_WRONG_SOURCE: &str = "standalone_source_wrong_coordinate";
const ISSUE_WRONG_CRATE: &str = "standalone_source_wrong_crate";
const ISSUE_MODULE_FIXTURE_MISSING: &str = "module_fixture_refs_missing";
const ISSUE_MODULE_FIXTURE_UNKNOWN: &str = "module_fixture_ref_unknown";
const ISSUE_UNKNOWN_DISPOSITION: &str = "unknown_compatibility_disposition";
const ISSUE_FIXTURE_HASH_MISMATCH: &str = "fixture_hash_mismatch";
const ISSUE_DRIFT_STALE: &str = "hosted_module_drift_without_exception";
const ISSUE_EXCEPTION_OWNER_MISSING: &str = "migration_exception_owner_missing";
const ISSUE_NON_CLAIM_MISSING: &str = "required_non_claim_missing";
const ISSUE_OVERCLAIM: &str = "unsupported_cutover_overclaim";
const OVERCLAIM_PATTERNS: &[&str] = &[
    "behavioral correctness",
    "semantic equivalence",
    "pure-cache eligibility",
    "external consumer compatibility",
    "cross-toolchain determinism",
    "release eligibility",
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StandaloneValenceSource {
    pub(crate) coordinate: String,
    pub(crate) crate_target: String,
    pub(crate) revision_or_local_override: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValenceCutoverManifest {
    pub(crate) standalone_source: StandaloneValenceSource,
    pub(crate) modules: Vec<ValenceModuleRow>,
    pub(crate) fixtures: Vec<CompatibilityFixtureRow>,
    pub(crate) supported_claim: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ValenceModuleRow {
    pub(crate) module_or_schema: String,
    pub(crate) standalone_path: String,
    pub(crate) octet_hosted_path: Option<String>,
    pub(crate) compatibility_disposition: String,
    pub(crate) fixture_refs: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CompatibilityFixtureRow {
    pub(crate) fixture_id: String,
    pub(crate) module_or_schema: String,
    pub(crate) expected_b3: String,
    pub(crate) observed_b3: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HostedDriftRow {
    pub(crate) row_id: String,
    pub(crate) canonical_b3: String,
    pub(crate) observed_b3: String,
    pub(crate) compatibility_disposition: String,
    pub(crate) migration_exception_owner: Option<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct ValenceCutoverReport {
    pub(crate) valid: bool,
    pub(crate) supported_claim: String,
    pub(crate) issues: Vec<CutoverIssue>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CutoverIssue {
    pub(crate) field: &'static str,
    pub(crate) code: &'static str,
    pub(crate) row_id: String,
}

pub(crate) fn validate_valence_cutover(
    manifest: &ValenceCutoverManifest,
    drift_rows: &[HostedDriftRow],
) -> ValenceCutoverReport {
    let mut issues = Vec::new();
    validate_manifest_shape(manifest, &mut issues);
    for module in &manifest.modules {
        validate_module(module, &manifest.fixtures, &mut issues);
    }
    for fixture in &manifest.fixtures {
        validate_fixture(fixture, &mut issues);
    }
    for drift in drift_rows {
        validate_drift(drift, &mut issues);
    }
    issues.sort_by(|left, right| {
        left.row_id.cmp(&right.row_id).then(left.field.cmp(right.field)).then(left.code.cmp(right.code))
    });
    issues.dedup();
    ValenceCutoverReport {
        valid: issues.is_empty(),
        supported_claim: manifest.supported_claim.clone(),
        issues,
    }
}

fn validate_manifest_shape(manifest: &ValenceCutoverManifest, issues: &mut Vec<CutoverIssue>) {
    if !manifest.standalone_source.coordinate.contains(REQUIRED_COORDINATE_MARKER) {
        push_issue(issues, FIELD_STANDALONE_SOURCE, ISSUE_WRONG_SOURCE, &manifest.standalone_source.coordinate);
    }
    if manifest.standalone_source.crate_target != REQUIRED_CRATE_TARGET {
        push_issue(issues, FIELD_STANDALONE_SOURCE, ISSUE_WRONG_CRATE, &manifest.standalone_source.crate_target);
    }
    if manifest.modules.is_empty() {
        push_issue(issues, FIELD_MODULE, ISSUE_EMPTY_MODULES, REQUIRED_CRATE_TARGET);
    }
    if manifest.fixtures.is_empty() {
        push_issue(issues, FIELD_FIXTURE, ISSUE_EMPTY_FIXTURES, REQUIRED_CRATE_TARGET);
    }
    validate_non_claims(&manifest.non_claims, REQUIRED_CRATE_TARGET, issues);
    validate_supported_claim(&manifest.supported_claim, REQUIRED_CRATE_TARGET, issues);
}

fn validate_module(module: &ValenceModuleRow, fixtures: &[CompatibilityFixtureRow], issues: &mut Vec<CutoverIssue>) {
    validate_disposition(&module.compatibility_disposition, &module.module_or_schema, issues);
    validate_non_claims(&module.non_claims, &module.module_or_schema, issues);
    if module.fixture_refs.is_empty() {
        push_issue(issues, FIELD_FIXTURE_REFS, ISSUE_MODULE_FIXTURE_MISSING, &module.module_or_schema);
    }
    for fixture_ref in &module.fixture_refs {
        if !fixtures.iter().any(|fixture| fixture.fixture_id == *fixture_ref) {
            push_issue(issues, FIELD_FIXTURE_REFS, ISSUE_MODULE_FIXTURE_UNKNOWN, fixture_ref);
        }
    }
}

fn validate_fixture(fixture: &CompatibilityFixtureRow, issues: &mut Vec<CutoverIssue>) {
    validate_non_claims(&fixture.non_claims, &fixture.fixture_id, issues);
    if fixture.expected_b3 != fixture.observed_b3 {
        push_issue(issues, FIELD_FIXTURE, ISSUE_FIXTURE_HASH_MISMATCH, &fixture.fixture_id);
    }
}

fn validate_drift(row: &HostedDriftRow, issues: &mut Vec<CutoverIssue>) {
    validate_disposition(&row.compatibility_disposition, &row.row_id, issues);
    validate_non_claims(&row.non_claims, &row.row_id, issues);
    if row.canonical_b3 == row.observed_b3 {
        return;
    }
    let has_exception = row.compatibility_disposition == DISPOSITION_EXCEPTION
        && row.migration_exception_owner.as_deref().is_some_and(|owner| !owner.trim().is_empty());
    if has_exception {
        return;
    }
    if row.compatibility_disposition == DISPOSITION_EXCEPTION {
        push_issue(issues, FIELD_DRIFT, ISSUE_EXCEPTION_OWNER_MISSING, &row.row_id);
    } else {
        push_issue(issues, FIELD_DRIFT, ISSUE_DRIFT_STALE, &row.row_id);
    }
}

fn validate_disposition(disposition: &str, row_id: &str, issues: &mut Vec<CutoverIssue>) {
    if ![DISPOSITION_COMPATIBLE, DISPOSITION_SHIM, DISPOSITION_EXCEPTION].contains(&disposition) {
        push_issue(issues, FIELD_DISPOSITION, ISSUE_UNKNOWN_DISPOSITION, row_id);
    }
}

fn validate_non_claims(non_claims: &[String], row_id: &str, issues: &mut Vec<CutoverIssue>) {
    if !non_claims.iter().any(|claim| claim.contains(REQUIRED_NON_CLAIM)) {
        push_issue(issues, FIELD_NON_CLAIMS, ISSUE_NON_CLAIM_MISSING, row_id);
    }
    validate_claim_text(&non_claims.join(" "), row_id, issues);
}

fn validate_supported_claim(supported_claim: &str, row_id: &str, issues: &mut Vec<CutoverIssue>) {
    if !supported_claim.contains("dependency/source ownership") || !supported_claim.contains("DTO compatibility") {
        push_issue(issues, FIELD_SUPPORTED_CLAIM, ISSUE_NON_CLAIM_MISSING, row_id);
    }
    validate_claim_text(supported_claim, row_id, issues);
}

fn validate_claim_text(text: &str, row_id: &str, issues: &mut Vec<CutoverIssue>) {
    for pattern in OVERCLAIM_PATTERNS {
        let allowed_negative = text.contains("does not prove") || text.contains(&format!("not {pattern}"));
        if text.contains(pattern) && !allowed_negative {
            push_issue(issues, FIELD_NON_CLAIMS, ISSUE_OVERCLAIM, row_id);
        }
    }
}

fn push_issue(issues: &mut Vec<CutoverIssue>, field: &'static str, code: &'static str, row_id: &str) {
    issues.push(CutoverIssue {
        field,
        code,
        row_id: row_id.to_owned(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const FUNCTION_OBJECT_MODULE: &str = "function_object";
    const EVIDENCE_INDEX_MODULE: &str = "evidence_index";
    const EVIDENCE_CODEBASE_INDEX_MODULE: &str = "evidence_codebase_index";
    const FUNCTION_OBJECT_FIXTURE: &str = "function-object-v1-compatible";
    const EVIDENCE_INDEX_FIXTURE: &str = "evidence-index-compatible";
    const EVIDENCE_CODEBASE_INDEX_FIXTURE: &str = "evidence-codebase-index-compatible";
    const CANONICAL_BYTES: &[u8] = b"canonical valence-core dto fixture";
    const DRIFT_BYTES: &[u8] = b"stale octet hosted dto fixture";

    #[test]
    fn valence_cutover_accepts_compatible_shim_manifest_and_matching_drift() {
        let manifest = valid_manifest();
        let drift = vec![matching_drift(FUNCTION_OBJECT_MODULE)];

        let report = validate_valence_cutover(&manifest, &drift);

        assert!(report.valid, "unexpected cutover issues: {:?}", report.issues);
        assert!(report.supported_claim.contains("dependency/source ownership"));
    }

    #[test]
    fn valence_cutover_rejects_wrong_source_missing_fixtures_and_overclaims() {
        let mut manifest = valid_manifest();
        manifest.standalone_source.coordinate = "github:example/not-valence".to_owned();
        manifest.standalone_source.crate_target = "not-valence-core".to_owned();
        manifest.modules[0].fixture_refs.clear();
        manifest.fixtures[0].observed_b3 = digest(DRIFT_BYTES);
        manifest.non_claims.clear();
        manifest.supported_claim = "cutover proves semantic equivalence and release eligibility".to_owned();

        let report = validate_valence_cutover(&manifest, &[]);

        assert!(!report.valid);
        assert_issue(&report, ISSUE_WRONG_SOURCE);
        assert_issue(&report, ISSUE_WRONG_CRATE);
        assert_issue(&report, ISSUE_MODULE_FIXTURE_MISSING);
        assert_issue(&report, ISSUE_FIXTURE_HASH_MISMATCH);
        assert_issue(&report, ISSUE_NON_CLAIM_MISSING);
        assert_issue(&report, ISSUE_OVERCLAIM);
    }

    #[test]
    fn valence_cutover_rejects_stale_hosted_drift_without_owned_exception() {
        let manifest = valid_manifest();
        let stale = HostedDriftRow {
            row_id: FUNCTION_OBJECT_MODULE.to_owned(),
            canonical_b3: digest(CANONICAL_BYTES),
            observed_b3: digest(DRIFT_BYTES),
            compatibility_disposition: DISPOSITION_SHIM.to_owned(),
            migration_exception_owner: None,
            non_claims: cutover_non_claims(),
        };
        let missing_owner_exception = HostedDriftRow {
            compatibility_disposition: DISPOSITION_EXCEPTION.to_owned(),
            ..stale.clone()
        };

        let stale_report = validate_valence_cutover(&manifest, &[stale]);
        let exception_report = validate_valence_cutover(&manifest, &[missing_owner_exception]);

        assert_issue(&stale_report, ISSUE_DRIFT_STALE);
        assert_issue(&exception_report, ISSUE_EXCEPTION_OWNER_MISSING);
    }

    fn valid_manifest() -> ValenceCutoverManifest {
        ValenceCutoverManifest {
            standalone_source: StandaloneValenceSource {
                coordinate: "github:OnixResearch/valence?dir=crates/valence-core".to_owned(),
                crate_target: REQUIRED_CRATE_TARGET.to_owned(),
                revision_or_local_override: "pinned revision or local path ../valence/crates/valence-core".to_owned(),
            },
            modules: vec![
                module(FUNCTION_OBJECT_MODULE, FUNCTION_OBJECT_FIXTURE),
                module(EVIDENCE_INDEX_MODULE, EVIDENCE_INDEX_FIXTURE),
                module(EVIDENCE_CODEBASE_INDEX_MODULE, EVIDENCE_CODEBASE_INDEX_FIXTURE),
            ],
            fixtures: vec![
                fixture(FUNCTION_OBJECT_FIXTURE, FUNCTION_OBJECT_MODULE),
                fixture(EVIDENCE_INDEX_FIXTURE, EVIDENCE_INDEX_MODULE),
                fixture(EVIDENCE_CODEBASE_INDEX_FIXTURE, EVIDENCE_CODEBASE_INDEX_MODULE),
            ],
            supported_claim: "cutover validation proves dependency/source ownership and DTO compatibility only; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility".to_owned(),
            non_claims: cutover_non_claims(),
        }
    }

    fn module(module_or_schema: &str, fixture_ref: &str) -> ValenceModuleRow {
        ValenceModuleRow {
            module_or_schema: module_or_schema.to_owned(),
            standalone_path: format!("crates/valence-core/src/{module_or_schema}.rs"),
            octet_hosted_path: Some(format!("crates/valence-core/src/{module_or_schema}.rs")),
            compatibility_disposition: DISPOSITION_SHIM.to_owned(),
            fixture_refs: vec![fixture_ref.to_owned()],
            non_claims: cutover_non_claims(),
        }
    }

    fn fixture(fixture_id: &str, module_or_schema: &str) -> CompatibilityFixtureRow {
        let digest = digest(CANONICAL_BYTES);
        CompatibilityFixtureRow {
            fixture_id: fixture_id.to_owned(),
            module_or_schema: module_or_schema.to_owned(),
            expected_b3: digest.clone(),
            observed_b3: digest,
            non_claims: cutover_non_claims(),
        }
    }

    fn matching_drift(row_id: &str) -> HostedDriftRow {
        let digest = digest(CANONICAL_BYTES);
        HostedDriftRow {
            row_id: row_id.to_owned(),
            canonical_b3: digest.clone(),
            observed_b3: digest,
            compatibility_disposition: DISPOSITION_SHIM.to_owned(),
            migration_exception_owner: None,
            non_claims: cutover_non_claims(),
        }
    }

    fn cutover_non_claims() -> Vec<String> {
        vec![format!(
            "{REQUIRED_NON_CLAIM}; it does not prove behavioral correctness, semantic equivalence, pure-cache eligibility, external consumer compatibility, cross-toolchain determinism, or release eligibility"
        )]
    }

    fn digest(bytes: &[u8]) -> String {
        format!("b3:{}", blake3::hash(bytes).to_hex())
    }

    fn assert_issue(report: &ValenceCutoverReport, code: &str) {
        assert!(!report.valid);
        assert!(report.issues.iter().any(|issue| issue.code == code), "missing issue {code}: {:?}", report.issues);
    }
}
