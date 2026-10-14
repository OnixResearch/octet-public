use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

// r[impl octet.valence_project_extraction.phase5.hardened_upstream_dependency]
// r[impl octet.valence_project_extraction.phase5.standalone_dependency_manifest]
// r[impl octet.valence_project_extraction.phase5.exact_source_pin]
// r[impl octet.valence_project_extraction.phase5.unique_package_identity]
// r[impl octet.valence_project_extraction.phase5.compatibility]
// r[impl octet.valence_project_extraction.phase5.compatibility.positive]
// r[impl octet.valence_project_extraction.phase5.compatibility.negative]
// r[impl octet.valence_project_extraction.phase5.cutover_validation]

const REQUIRED_SCHEMA_VERSION: &str = "octet-valence-core-cutover-manifest/v2";
const REQUIRED_NON_CLAIM: &str =
    "standalone Valence cutover proves dependency/source ownership and DTO compatibility only";
const REQUIRED_COORDINATE_MARKER: &str = "OnixResearch/valence";
const REQUIRED_CRATE_TARGET: &str = "valence-core";
const REQUIRED_LEGACY_PACKAGE: &str = "octet-valence-legacy-fixture";
const REQUIRED_DISPOSITION: &str = "legacy-fixture";
const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const GIT_REVISION_HEX_LENGTH: usize = 40;
const CANONICAL_PACKAGE_COUNT: usize = 1;
const FIELD_SCHEMA: &str = "schema_version";
const FIELD_STANDALONE_SOURCE: &str = "standalone_source";
const FIELD_MODULE: &str = "module";
const FIELD_FIXTURE: &str = "fixture";
const FIELD_MIGRATION: &str = "migration_target";
const FIELD_NON_CLAIMS: &str = "non_claims";
const FIELD_SUPPORTED_CLAIM: &str = "supported_claim";
const FIELD_PACKAGE_IDENTITY: &str = "package_identity";
const FIELD_SHIM: &str = "shim";
const ISSUE_SCHEMA_UNSUPPORTED: &str = "schema_version_unsupported";
const ISSUE_REQUIRED_FIELD_EMPTY: &str = "required_field_empty";
const ISSUE_WRONG_SOURCE: &str = "standalone_source_wrong_coordinate";
const ISSUE_WRONG_CRATE: &str = "standalone_source_wrong_crate";
const ISSUE_REVISION_INVALID: &str = "standalone_revision_invalid";
const ISSUE_REVISION_MISMATCH: &str = "standalone_revision_mismatch";
const ISSUE_LOCK_SOURCE_MISMATCH: &str = "lock_source_mismatch";
const ISSUE_ARCHIVE_MISMATCH: &str = "hardened_profile_archive_mismatch";
const ISSUE_ARCHIVE_MISSING: &str = "hardened_profile_archive_missing";
const ISSUE_EMPTY_MODULES: &str = "module_rows_missing";
const ISSUE_DUPLICATE_MODULE: &str = "module_row_duplicate";
const ISSUE_UNKNOWN_DISPOSITION: &str = "unknown_compatibility_disposition";
const ISSUE_SOURCE_HASH_INVALID: &str = "standalone_source_hash_invalid";
const ISSUE_SOURCE_HASH_MISMATCH: &str = "standalone_source_hash_mismatch";
const ISSUE_MODULE_FIXTURE_MISSING: &str = "module_fixture_refs_missing";
const ISSUE_MODULE_FIXTURE_UNKNOWN: &str = "module_fixture_ref_unknown";
const ISSUE_EMPTY_FIXTURES: &str = "fixture_rows_missing";
const ISSUE_DUPLICATE_FIXTURE: &str = "fixture_row_duplicate";
const ISSUE_FIXTURE_MODULE_UNKNOWN: &str = "fixture_module_unknown";
const ISSUE_FIXTURE_OBSERVATION_MISSING: &str = "fixture_observation_missing";
const ISSUE_FIXTURE_HASH_MISMATCH: &str = "fixture_hash_mismatch";
const ISSUE_MIGRATION_REVISION_MISSING: &str = "migration_revision_missing";
const ISSUE_CANONICAL_PACKAGE_COUNT: &str = "canonical_package_count_invalid";
const ISSUE_HOSTED_PATH_FALLBACK: &str = "hosted_path_fallback";
const ISSUE_LEGACY_PACKAGE_IDENTITY: &str = "legacy_package_identity_invalid";
const ISSUE_LEGACY_PACKAGE_PUBLISHABLE: &str = "legacy_package_publishable";
const ISSUE_SHIM_NOT_DELEGATING: &str = "shim_not_delegating";
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

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct CutoverManifest {
    pub(crate) schema_version: String,
    pub(crate) standalone_source: StandaloneValenceSource,
    pub(crate) modules: Vec<ValenceModuleRow>,
    pub(crate) fixtures: Vec<CompatibilityFixtureRow>,
    pub(crate) migration_targets: Vec<MigrationTarget>,
    pub(crate) supported_claim: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct StandaloneValenceSource {
    pub(crate) coordinate: String,
    pub(crate) cargo_git_url: String,
    pub(crate) crate_target: String,
    pub(crate) revision: String,
    pub(crate) cargo_source: String,
    pub(crate) hardened_profile_archive: String,
    pub(crate) local_override_policy: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct ValenceModuleRow {
    pub(crate) module_or_schema: String,
    pub(crate) standalone_path: String,
    pub(crate) standalone_source_b3: String,
    pub(crate) legacy_fixture_path: String,
    pub(crate) compatibility_disposition: String,
    pub(crate) fixture_refs: Vec<String>,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct CompatibilityFixtureRow {
    pub(crate) fixture_id: String,
    pub(crate) module_or_schema: String,
    pub(crate) expected_behavior: String,
    pub(crate) compatibility_command: String,
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct MigrationTarget {
    pub(crate) consumer: String,
    pub(crate) current_dependency: String,
    pub(crate) target_dependency: String,
    pub(crate) validation_command: String,
    pub(crate) non_claim: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CutoverAuthority {
    pub(crate) revision: String,
    pub(crate) cargo_source: String,
    pub(crate) hardened_profile_archive: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CutoverObservations {
    pub(crate) locked_cargo_source: String,
    pub(crate) package_identity: PackageIdentityObservation,
    pub(crate) upstream: UpstreamObservation,
    pub(crate) source_hashes: BTreeMap<String, String>,
    pub(crate) fixture_hashes: BTreeMap<String, FixtureHashObservation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PackageIdentityObservation {
    pub(crate) canonical_package_count: usize,
    pub(crate) uses_hosted_path_dependency: bool,
    pub(crate) legacy_package_name: String,
    pub(crate) legacy_package_publishable: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UpstreamObservation {
    pub(crate) hardened_profile_archive_present: bool,
    pub(crate) shim_delegates_to_standalone: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct FixtureHashObservation {
    pub(crate) expected_b3: String,
    pub(crate) observed_b3: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub(crate) struct CutoverReport {
    pub(crate) valid: bool,
    pub(crate) supported_claim: String,
    pub(crate) issues: Vec<CutoverIssue>,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize)]
pub(crate) struct CutoverIssue {
    pub(crate) field: &'static str,
    pub(crate) code: &'static str,
    pub(crate) row_id: String,
}

pub(crate) fn validate_valence_cutover(
    manifest: &CutoverManifest,
    authority: &CutoverAuthority,
    observations: &CutoverObservations,
) -> CutoverReport {
    let mut issues = Vec::new();
    validate_manifest_shape(manifest, authority, observations, &mut issues);
    validate_modules(manifest, observations, &mut issues);
    validate_fixtures(manifest, observations, &mut issues);
    validate_migrations(manifest, authority, &mut issues);
    validate_package_identity(observations, &mut issues);
    validate_supported_claim(&manifest.supported_claim, REQUIRED_CRATE_TARGET, &mut issues);
    validate_non_claims(&manifest.non_claims, REQUIRED_CRATE_TARGET, &mut issues);
    issues.sort();
    issues.dedup();
    CutoverReport {
        valid: issues.is_empty(),
        supported_claim: manifest.supported_claim.clone(),
        issues,
    }
}

fn validate_manifest_shape(
    manifest: &CutoverManifest,
    authority: &CutoverAuthority,
    observations: &CutoverObservations,
    issues: &mut Vec<CutoverIssue>,
) {
    let source = &manifest.standalone_source;
    if manifest.schema_version != REQUIRED_SCHEMA_VERSION {
        push_issue(issues, FIELD_SCHEMA, ISSUE_SCHEMA_UNSUPPORTED, &manifest.schema_version);
    }
    if !source.coordinate.contains(REQUIRED_COORDINATE_MARKER)
        || !source.cargo_git_url.contains(REQUIRED_COORDINATE_MARKER)
    {
        push_issue(issues, FIELD_STANDALONE_SOURCE, ISSUE_WRONG_SOURCE, &source.coordinate);
    }
    validate_required_text(&source.local_override_policy, FIELD_STANDALONE_SOURCE, "local_override_policy", issues);
    if source.crate_target != REQUIRED_CRATE_TARGET {
        push_issue(issues, FIELD_STANDALONE_SOURCE, ISSUE_WRONG_CRATE, &source.crate_target);
    }
    if !is_lower_hex(&source.revision, GIT_REVISION_HEX_LENGTH) {
        push_issue(issues, FIELD_STANDALONE_SOURCE, ISSUE_REVISION_INVALID, &source.revision);
    }
    if source.revision != authority.revision {
        push_issue(issues, FIELD_STANDALONE_SOURCE, ISSUE_REVISION_MISMATCH, &source.revision);
    }
    if source.cargo_source != authority.cargo_source || observations.locked_cargo_source != authority.cargo_source {
        push_issue(issues, FIELD_STANDALONE_SOURCE, ISSUE_LOCK_SOURCE_MISMATCH, &source.cargo_source);
    }
    if source.hardened_profile_archive != authority.hardened_profile_archive {
        push_issue(issues, FIELD_STANDALONE_SOURCE, ISSUE_ARCHIVE_MISMATCH, &source.hardened_profile_archive);
    }
    if !observations.upstream.hardened_profile_archive_present {
        push_issue(issues, FIELD_STANDALONE_SOURCE, ISSUE_ARCHIVE_MISSING, &source.hardened_profile_archive);
    }
}

fn validate_modules(manifest: &CutoverManifest, observations: &CutoverObservations, issues: &mut Vec<CutoverIssue>) {
    if manifest.modules.is_empty() {
        push_issue(issues, FIELD_MODULE, ISSUE_EMPTY_MODULES, REQUIRED_CRATE_TARGET);
    }
    let fixture_ids: BTreeSet<&str> = manifest.fixtures.iter().map(|fixture| fixture.fixture_id.as_str()).collect();
    let mut module_ids = BTreeSet::new();
    for module in &manifest.modules {
        validate_required_text(&module.standalone_path, FIELD_MODULE, &module.module_or_schema, issues);
        validate_required_text(&module.legacy_fixture_path, FIELD_MODULE, &module.module_or_schema, issues);
        if !module_ids.insert(module.module_or_schema.as_str()) {
            push_issue(issues, FIELD_MODULE, ISSUE_DUPLICATE_MODULE, &module.module_or_schema);
        }
        if module.compatibility_disposition != REQUIRED_DISPOSITION {
            push_issue(issues, FIELD_MODULE, ISSUE_UNKNOWN_DISPOSITION, &module.module_or_schema);
        }
        if !is_blake3_hex(&module.standalone_source_b3) {
            push_issue(issues, FIELD_MODULE, ISSUE_SOURCE_HASH_INVALID, &module.module_or_schema);
        }
        match observations.source_hashes.get(&module.module_or_schema) {
            Some(observed) if observed == &module.standalone_source_b3 => {}
            _ => push_issue(issues, FIELD_MODULE, ISSUE_SOURCE_HASH_MISMATCH, &module.module_or_schema),
        }
        if module.fixture_refs.is_empty() {
            push_issue(issues, FIELD_FIXTURE, ISSUE_MODULE_FIXTURE_MISSING, &module.module_or_schema);
        }
        for fixture_ref in &module.fixture_refs {
            if !fixture_ids.contains(fixture_ref.as_str()) {
                push_issue(issues, FIELD_FIXTURE, ISSUE_MODULE_FIXTURE_UNKNOWN, fixture_ref);
            }
        }
        validate_non_claims(&module.non_claims, &module.module_or_schema, issues);
    }
}

fn validate_fixtures(manifest: &CutoverManifest, observations: &CutoverObservations, issues: &mut Vec<CutoverIssue>) {
    if manifest.fixtures.is_empty() {
        push_issue(issues, FIELD_FIXTURE, ISSUE_EMPTY_FIXTURES, REQUIRED_CRATE_TARGET);
    }
    let module_ids: BTreeSet<&str> = manifest.modules.iter().map(|module| module.module_or_schema.as_str()).collect();
    let mut fixture_ids = BTreeSet::new();
    for fixture in &manifest.fixtures {
        validate_required_text(&fixture.expected_behavior, FIELD_FIXTURE, &fixture.fixture_id, issues);
        validate_required_text(&fixture.compatibility_command, FIELD_FIXTURE, &fixture.fixture_id, issues);
        if !fixture_ids.insert(fixture.fixture_id.as_str()) {
            push_issue(issues, FIELD_FIXTURE, ISSUE_DUPLICATE_FIXTURE, &fixture.fixture_id);
        }
        if !module_ids.contains(fixture.module_or_schema.as_str()) {
            push_issue(issues, FIELD_FIXTURE, ISSUE_FIXTURE_MODULE_UNKNOWN, &fixture.fixture_id);
        }
        match observations.fixture_hashes.get(&fixture.fixture_id) {
            None => push_issue(issues, FIELD_FIXTURE, ISSUE_FIXTURE_OBSERVATION_MISSING, &fixture.fixture_id),
            Some(observation) if observation.expected_b3 == observation.observed_b3 => {}
            Some(_) => push_issue(issues, FIELD_FIXTURE, ISSUE_FIXTURE_HASH_MISMATCH, &fixture.fixture_id),
        }
        validate_non_claims(&fixture.non_claims, &fixture.fixture_id, issues);
    }
}

fn validate_migrations(manifest: &CutoverManifest, authority: &CutoverAuthority, issues: &mut Vec<CutoverIssue>) {
    for migration in &manifest.migration_targets {
        validate_required_text(&migration.current_dependency, FIELD_MIGRATION, &migration.consumer, issues);
        validate_required_text(&migration.validation_command, FIELD_MIGRATION, &migration.consumer, issues);
        if !migration.target_dependency.contains(&authority.revision) {
            push_issue(issues, FIELD_MIGRATION, ISSUE_MIGRATION_REVISION_MISSING, &migration.consumer);
        }
        validate_non_claims(std::slice::from_ref(&migration.non_claim), &migration.consumer, issues);
    }
}

fn validate_package_identity(observations: &CutoverObservations, issues: &mut Vec<CutoverIssue>) {
    let package = &observations.package_identity;
    if package.canonical_package_count != CANONICAL_PACKAGE_COUNT {
        push_issue(issues, FIELD_PACKAGE_IDENTITY, ISSUE_CANONICAL_PACKAGE_COUNT, REQUIRED_CRATE_TARGET);
    }
    if package.uses_hosted_path_dependency {
        push_issue(issues, FIELD_PACKAGE_IDENTITY, ISSUE_HOSTED_PATH_FALLBACK, REQUIRED_CRATE_TARGET);
    }
    if package.legacy_package_name != REQUIRED_LEGACY_PACKAGE {
        push_issue(issues, FIELD_PACKAGE_IDENTITY, ISSUE_LEGACY_PACKAGE_IDENTITY, &package.legacy_package_name);
    }
    if package.legacy_package_publishable {
        push_issue(issues, FIELD_PACKAGE_IDENTITY, ISSUE_LEGACY_PACKAGE_PUBLISHABLE, REQUIRED_LEGACY_PACKAGE);
    }
    if !observations.upstream.shim_delegates_to_standalone {
        push_issue(issues, FIELD_SHIM, ISSUE_SHIM_NOT_DELEGATING, REQUIRED_CRATE_TARGET);
    }
}

fn validate_required_text(value: &str, field: &'static str, row_id: &str, issues: &mut Vec<CutoverIssue>) {
    if value.trim().is_empty() {
        push_issue(issues, field, ISSUE_REQUIRED_FIELD_EMPTY, row_id);
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

fn is_blake3_hex(value: &str) -> bool {
    value.strip_prefix(BLAKE3_PREFIX).is_some_and(|hex| is_lower_hex(hex, BLAKE3_HEX_LENGTH))
}

fn is_lower_hex(value: &str, expected_length: usize) -> bool {
    value.len() == expected_length && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
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

    const MANIFEST_JSON: &str = include_str!("../../standards/valence-core-cutover/generated-manifest.json");
    const CARGO_LOCK: &str = include_str!("../../Cargo.lock");
    const CARGO_OCTET_MANIFEST: &str = include_str!("../Cargo.toml");
    const LEGACY_MANIFEST: &str = include_str!("../../crates/valence-core/Cargo.toml");
    const SHIM_SOURCE: &str = include_str!("valence_core_shim.rs");

    // r[verify octet.valence_project_extraction.phase5.compatibility.positive]
    #[test]
    fn valence_cutover_accepts_exact_standalone_dependency_and_compatibility_observations() {
        let manifest = parsed_manifest();
        let authority = authority(&manifest);
        let observations = valid_observations(&manifest);

        let report = validate_valence_cutover(&manifest, &authority, &observations);

        assert!(report.valid, "unexpected cutover issues: {:?}", report.issues);
        assert!(report.supported_claim.contains("dependency/source ownership"));
    }

    // r[verify octet.valence_project_extraction.phase5.compatibility.negative]
    #[test]
    fn valence_cutover_rejects_stale_source_archive_package_and_shim_evidence() {
        let mut manifest = parsed_manifest();
        let authority = authority(&manifest);
        let mut observations = valid_observations(&manifest);
        manifest.standalone_source.revision = "0000000000000000000000000000000000000000".to_owned();
        observations.locked_cargo_source = "git+https://github.com/OnixResearch/valence.git#stale".to_owned();
        observations.package_identity.canonical_package_count += CANONICAL_PACKAGE_COUNT;
        observations.package_identity.uses_hosted_path_dependency = true;
        observations.package_identity.legacy_package_name = REQUIRED_CRATE_TARGET.to_owned();
        observations.package_identity.legacy_package_publishable = true;
        observations.upstream.hardened_profile_archive_present = false;
        observations.upstream.shim_delegates_to_standalone = false;

        let report = validate_valence_cutover(&manifest, &authority, &observations);

        assert_issue(&report, ISSUE_REVISION_MISMATCH);
        assert_issue(&report, ISSUE_LOCK_SOURCE_MISMATCH);
        assert_issue(&report, ISSUE_ARCHIVE_MISSING);
        assert_issue(&report, ISSUE_CANONICAL_PACKAGE_COUNT);
        assert_issue(&report, ISSUE_HOSTED_PATH_FALLBACK);
        assert_issue(&report, ISSUE_LEGACY_PACKAGE_IDENTITY);
        assert_issue(&report, ISSUE_LEGACY_PACKAGE_PUBLISHABLE);
        assert_issue(&report, ISSUE_SHIM_NOT_DELEGATING);
    }

    // r[verify octet.valence_project_extraction.phase5.standalone_dependency_manifest]
    #[test]
    fn valence_cutover_rejects_missing_rows_hash_drift_and_weakened_claims() {
        let mut manifest = parsed_manifest();
        let authority = authority(&manifest);
        let mut observations = valid_observations(&manifest);
        let removed = manifest.modules.remove(0);
        manifest.fixtures[0].module_or_schema = "missing-module".to_owned();
        manifest.non_claims.clear();
        manifest.supported_claim = "cutover proves semantic equivalence and release eligibility".to_owned();
        observations.source_hashes.insert(removed.module_or_schema, blake3_id(b"stale source"));
        let fixture = manifest.fixtures[0].fixture_id.clone();
        observations.fixture_hashes.insert(fixture, FixtureHashObservation {
            expected_b3: blake3_id(b"expected"),
            observed_b3: blake3_id(b"stale"),
        });

        let report = validate_valence_cutover(&manifest, &authority, &observations);

        assert_issue(&report, ISSUE_FIXTURE_MODULE_UNKNOWN);
        assert_issue(&report, ISSUE_FIXTURE_HASH_MISMATCH);
        assert_issue(&report, ISSUE_NON_CLAIM_MISSING);
        assert_issue(&report, ISSUE_OVERCLAIM);
    }

    fn parsed_manifest() -> CutoverManifest {
        serde_json::from_str(MANIFEST_JSON).expect("generated Valence cutover manifest should deserialize")
    }

    fn authority(manifest: &CutoverManifest) -> CutoverAuthority {
        CutoverAuthority {
            revision: manifest.standalone_source.revision.clone(),
            cargo_source: manifest.standalone_source.cargo_source.clone(),
            hardened_profile_archive: manifest.standalone_source.hardened_profile_archive.clone(),
        }
    }

    fn valid_observations(manifest: &CutoverManifest) -> CutoverObservations {
        let lock: toml::Value = toml::from_str(CARGO_LOCK).expect("Cargo.lock should parse");
        let packages = lock.get("package").and_then(toml::Value::as_array).expect("Cargo.lock package rows");
        let canonical: Vec<&toml::Value> = packages
            .iter()
            .filter(|package| package.get("name").and_then(toml::Value::as_str) == Some(REQUIRED_CRATE_TARGET))
            .collect();
        let locked_cargo_source = canonical
            .first()
            .and_then(|package| package.get("source"))
            .and_then(toml::Value::as_str)
            .expect("standalone Valence lock source")
            .to_owned();
        let dependency: toml::Value = toml::from_str(CARGO_OCTET_MANIFEST).expect("cargo-octet manifest should parse");
        let uses_hosted_path_dependency = dependency
            .get("dependencies")
            .and_then(|dependencies| dependencies.get(REQUIRED_CRATE_TARGET))
            .and_then(toml::Value::as_table)
            .is_some_and(|row| row.contains_key("path"));
        let legacy: toml::Value = toml::from_str(LEGACY_MANIFEST).expect("legacy fixture manifest should parse");
        let legacy_package = legacy.get("package").and_then(toml::Value::as_table).expect("legacy package row");
        let legacy_package_name =
            legacy_package.get("name").and_then(toml::Value::as_str).expect("legacy package name").to_owned();
        let legacy_package_publishable = legacy_package.get("publish").and_then(toml::Value::as_bool).unwrap_or(true);
        let source_hashes = manifest
            .modules
            .iter()
            .map(|module| (module.module_or_schema.clone(), module.standalone_source_b3.clone()))
            .collect();
        let fixture_hashes = manifest
            .fixtures
            .iter()
            .map(|fixture| {
                let digest = blake3_id(fixture.expected_behavior.as_bytes());
                (fixture.fixture_id.clone(), FixtureHashObservation {
                    expected_b3: digest.clone(),
                    observed_b3: digest,
                })
            })
            .collect();
        CutoverObservations {
            locked_cargo_source,
            package_identity: PackageIdentityObservation {
                canonical_package_count: canonical.len(),
                uses_hosted_path_dependency,
                legacy_package_name,
                legacy_package_publishable,
            },
            upstream: UpstreamObservation {
                hardened_profile_archive_present: true,
                shim_delegates_to_standalone: SHIM_SOURCE.contains("valence_core::"),
            },
            source_hashes,
            fixture_hashes,
        }
    }

    fn blake3_id(bytes: &[u8]) -> String {
        format!("{BLAKE3_PREFIX}{}", blake3::hash(bytes).to_hex())
    }

    fn assert_issue(report: &CutoverReport, code: &str) {
        assert!(!report.valid);
        assert!(report.issues.iter().any(|issue| issue.code == code), "missing issue {code}: {:?}", report.issues);
    }
}
