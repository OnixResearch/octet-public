//! Local policy-package customs gate core.
//!
//! Customs validation is a deterministic, local review aid. It checks package
//! metadata completeness, registry coverage, fixtures, generated artifact
//! freshness, provenance, secret boundaries, and remote-fetch boundaries without
//! granting trust to the package or its upstream source.

use serde::Deserialize;
use serde::Serialize;

use crate::artifacts::ArtifactManifest;
use crate::model::ConfigReceiptFamily;
use crate::model::ConfigReceiptFamilyStatus;
use crate::model::PolicyRegistry;
use crate::proof_artifact::artifact_text_identity;
use crate::source::SourceFile;

/// Policy-package manifest schema version.
pub const POLICY_PACKAGE_MANIFEST_SCHEMA_VERSION: &str = "octet-policy-package-manifest/v1";
/// Deterministic customs report schema version.
pub const POLICY_PACKAGE_CUSTOMS_REPORT_SCHEMA_VERSION: &str = "octet-policy-package-customs-report/v1";

const STATUS_PROMOTABLE: &str = "promotable";
const STATUS_EXISTING: &str = "existing";
const STATUS_NON_CANDIDATE: &str = "non-candidate";
const STATUS_DRAFT: &str = "draft";
const REMEDIATION_FIX_PACKAGE: &str = "complete the package manifest before promotion";
const REMEDIATION_REGISTRY: &str = "add or repair standards/policies/registry.ncl config_receipt_families coverage";
const REMEDIATION_FIXTURES: &str = "add positive and negative fixtures or classify the package as non-candidate";
const REMEDIATION_ARTIFACTS: &str = "regenerate package artifacts and refresh the BLAKE3 manifest";
const REMEDIATION_OFFLINE: &str = "use checked-in package sources, manifests, fixtures, and generated artifacts only";
const REMEDIATION_SECRETS: &str = "remove raw secret material or record a non-candidate exclusion rationale";

static PACKAGE_KINDS: &[&str] = &[
    "policy",
    "profile",
    "template",
    "proof-rail",
    "generated-contract",
    "contract",
];
/// Credential key names that mark an assignment such as `password = "..."` or `"api_key": "..."`.
static SECRET_KEY_NAMES: &[&str] = &[
    "password",
    "passwd",
    "token",
    "secret",
    "api_key",
    "apikey",
    "private_key",
];
const PEM_HEADER_START: &str = "-----begin ";
const PRIVATE_KEY_LABEL: &str = "private key";
const BARE_PRIVATE_KEY_HEADER: &str = "begin private key";
static REMOTE_FETCH_MARKERS: &[&str] = &["https://", "http://", "curl ", "wget ", "git clone"];
/// Kinds that define contracts, schemas, profiles, templates, or policy rules.
static EXPORT_COVERAGE_KINDS: &[&str] = &["policy", "profile", "template", "generated-contract", "contract"];
/// Generation-command markers for deterministic Nickel export coverage.
static EXPORT_COMMAND_MARKERS: &[&str] = &["nickel export", "nickel-export-manifest"];

/// One package-generated artifact tracked by customs.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PackageGeneratedArtifact {
    /// Generated artifact path.
    pub path: String,
    /// Generation or export command that writes the artifact.
    pub generation_command: String,
    /// Validation command or freshness check for the artifact.
    pub validation_command: String,
}

/// Local package manifest consumed by the customs gate.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PolicyPackageManifest {
    /// Manifest schema version.
    pub schema_version: String,
    /// Stable package ID.
    pub package_id: String,
    /// Package kind: policy, profile, template, proof-rail, generated-contract, or contract.
    pub kind: String,
    /// Owning team, module, or source path.
    pub owner: String,
    /// Promotion status: draft, promotable, existing, or non-candidate.
    pub status: String,
    /// Source paths that define the package.
    #[serde(default)]
    pub source_paths: Vec<String>,
    /// Generated artifacts owned by the package.
    #[serde(default)]
    pub generated_artifacts: Vec<PackageGeneratedArtifact>,
    /// Config or receipt family IDs that must be covered by the policy registry.
    #[serde(default)]
    pub registry_family_ids: Vec<String>,
    /// Package-level generation commands.
    #[serde(default)]
    pub generation_commands: Vec<String>,
    /// Package-level validation commands.
    #[serde(default)]
    pub validation_commands: Vec<String>,
    /// Positive fixtures accepted by the package.
    #[serde(default)]
    pub positive_fixtures: Vec<String>,
    /// Negative fixtures rejected by the package.
    #[serde(default)]
    pub negative_fixtures: Vec<String>,
    /// Provenance statement for local review.
    pub provenance: String,
    /// License or review notes.
    pub license_notes: String,
    /// Explicit non-claims retained in reports.
    #[serde(default)]
    pub non_claims: Vec<String>,
    /// Manual review actions remaining after automated checks.
    #[serde(default)]
    pub manual_review_actions: Vec<String>,
    /// True only for packages whose validation explicitly needs remote fetching.
    #[serde(default)]
    pub remote_fetches_allowed: bool,
    /// Exclusion rationale for secret-shaped material; honoured only for `non-candidate` packages.
    #[serde(default)]
    pub secret_exclusion_rationale: String,
}

/// One deterministic customs diagnostic.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CustomsDiagnostic {
    /// Package field or artifact path.
    pub field: String,
    /// Diagnostic message.
    pub message: String,
    /// Suggested remediation.
    pub remediation: String,
}

/// Deterministic customs report.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct CustomsReport {
    /// Report schema version.
    pub schema_version: String,
    /// Checked package ID.
    pub package_id: String,
    /// Pass/fail status.
    pub status: String,
    /// Blocking diagnostics.
    pub diagnostics: Vec<CustomsDiagnostic>,
    /// Accepted generated artifact paths.
    pub accepted_artifacts: Vec<String>,
    /// Explicit non-claims copied from the package.
    pub non_claims: Vec<String>,
    /// Remaining manual review actions.
    pub manual_review_actions: Vec<String>,
}

/// Pure customs validation input.
pub struct CustomsInput<'a> {
    /// Package manifest.
    pub package: &'a PolicyPackageManifest,
    /// Policy registry used for family coverage checks.
    pub registry: &'a PolicyRegistry,
    /// Optional artifact manifest used for freshness checks.
    pub artifact_manifest: Option<&'a ArtifactManifest>,
    /// Source and fixture inventory available to the gate.
    pub sources: &'a [SourceFile],
    /// Generated artifact texts available to the gate.
    pub artifact_texts: &'a [SourceFile],
}

/// Validate a package and render a deterministic report.
#[must_use]
pub fn validate_policy_package(input: &CustomsInput<'_>) -> CustomsReport {
    let mut diagnostics = Vec::new();
    validate_package_shape(input.package, &mut diagnostics);
    validate_source_inventory(input.package, input.sources, &mut diagnostics);
    validate_fixture_coverage(input.package, input.sources, &mut diagnostics);
    validate_registry_coverage(input.package, input.registry, &mut diagnostics);
    validate_generated_artifacts(input, &mut diagnostics);
    validate_offline_boundary(input.package, &mut diagnostics);
    validate_secret_boundary(input, &mut diagnostics);

    let status = if diagnostics.is_empty() { "pass" } else { "fail" }.to_owned();
    let mut accepted_artifacts =
        input.package.generated_artifacts.iter().map(|artifact| artifact.path.clone()).collect::<Vec<_>>();
    accepted_artifacts.sort();
    CustomsReport {
        schema_version: POLICY_PACKAGE_CUSTOMS_REPORT_SCHEMA_VERSION.to_owned(),
        package_id: input.package.package_id.clone(),
        status,
        diagnostics,
        accepted_artifacts,
        non_claims: input.package.non_claims.clone(),
        manual_review_actions: input.package.manual_review_actions.clone(),
    }
}

fn validate_package_shape(package: &PolicyPackageManifest, diagnostics: &mut Vec<CustomsDiagnostic>) {
    if package.schema_version != POLICY_PACKAGE_MANIFEST_SCHEMA_VERSION {
        diagnostics.push(diagnostic(
            "schema_version",
            format!(
                "unsupported package schema `{}`; expected `{POLICY_PACKAGE_MANIFEST_SCHEMA_VERSION}`",
                package.schema_version
            ),
            REMEDIATION_FIX_PACKAGE,
        ));
    }
    push_blank(package.package_id.trim(), "package_id", diagnostics);
    if !is_known_kind(&package.kind) {
        diagnostics.push(diagnostic(
            "kind",
            format!("unknown package kind `{}`; expected one of {}", package.kind, PACKAGE_KINDS.join(", ")),
            REMEDIATION_FIX_PACKAGE,
        ));
    }
    push_blank(package.owner.trim(), "owner", diagnostics);
    push_blank(package.provenance.trim(), "provenance", diagnostics);
    push_blank(package.license_notes.trim(), "license_notes", diagnostics);
    if package.non_claims.is_empty() {
        diagnostics.push(diagnostic(
            "non_claims",
            "customs packages must state at least one non-claim".to_owned(),
            REMEDIATION_FIX_PACKAGE,
        ));
    }
    if package.manual_review_actions.is_empty() {
        diagnostics.push(diagnostic(
            "manual_review_actions",
            "customs reports must retain at least one manual review action".to_owned(),
            REMEDIATION_FIX_PACKAGE,
        ));
    }
    if !is_known_status(&package.status) {
        diagnostics.push(diagnostic(
            "status",
            format!("unknown package status `{}`", package.status),
            REMEDIATION_FIX_PACKAGE,
        ));
    }
}

fn validate_source_inventory(
    package: &PolicyPackageManifest,
    sources: &[SourceFile],
    diagnostics: &mut Vec<CustomsDiagnostic>,
) {
    for path in &package.source_paths {
        if !source_exists(sources, path) {
            diagnostics.push(diagnostic(
                path,
                format!("package source path `{path}` is missing from checked-in inventory"),
                REMEDIATION_FIX_PACKAGE,
            ));
        }
    }
}

fn validate_fixture_coverage(
    package: &PolicyPackageManifest,
    sources: &[SourceFile],
    diagnostics: &mut Vec<CustomsDiagnostic>,
) {
    if package_is_promotable(package) {
        if package.positive_fixtures.is_empty() {
            diagnostics.push(diagnostic(
                "positive_fixtures",
                "promotable package declares no positive fixtures".to_owned(),
                REMEDIATION_FIXTURES,
            ));
        }
        if package.negative_fixtures.is_empty() {
            diagnostics.push(diagnostic(
                "negative_fixtures",
                "promotable package declares no negative fixtures".to_owned(),
                REMEDIATION_FIXTURES,
            ));
        }
    }
    for path in package.positive_fixtures.iter().chain(&package.negative_fixtures) {
        if !source_exists(sources, path) {
            diagnostics.push(diagnostic(
                path,
                format!("fixture `{path}` is missing from checked-in inventory"),
                REMEDIATION_FIXTURES,
            ));
        }
    }
}

fn validate_registry_coverage(
    package: &PolicyPackageManifest,
    registry: &PolicyRegistry,
    diagnostics: &mut Vec<CustomsDiagnostic>,
) {
    if package_is_promotable(package) && package.registry_family_ids.is_empty() {
        diagnostics.push(diagnostic(
            "registry_family_ids",
            "promotable package declares no registry family coverage".to_owned(),
            REMEDIATION_REGISTRY,
        ));
    }
    for family_id in &package.registry_family_ids {
        match registry.config_receipt_families.iter().find(|family| family.family_id == *family_id) {
            Some(family) => validate_registry_family(package, family, diagnostics),
            None => diagnostics.push(diagnostic(
                family_id,
                format!("package family `{family_id}` is not registered"),
                REMEDIATION_REGISTRY,
            )),
        }
    }
}

fn validate_registry_family(
    package: &PolicyPackageManifest,
    family: &ConfigReceiptFamily,
    diagnostics: &mut Vec<CustomsDiagnostic>,
) {
    push_blank(&family.owner_source_path, &format!("{} owner_source_path", family.family_id), diagnostics);
    push_blank(&family.generation_command, &format!("{} generation_command", family.family_id), diagnostics);
    push_blank(&family.validation_command, &format!("{} validation_command", family.family_id), diagnostics);
    if package_is_promotable(package) && family.status == ConfigReceiptFamilyStatus::NonCandidate {
        diagnostics.push(diagnostic(
            &family.family_id,
            "promotable package references a non-candidate registry family".to_owned(),
            REMEDIATION_REGISTRY,
        ));
    }
}

fn validate_generated_artifacts(input: &CustomsInput<'_>, diagnostics: &mut Vec<CustomsDiagnostic>) {
    let package = input.package;
    if package_is_promotable(package)
        && EXPORT_COVERAGE_KINDS.contains(&package.kind.as_str())
        && package.generated_artifacts.is_empty()
        && !package.generation_commands.iter().any(|command| is_export_command(command))
    {
        diagnostics.push(diagnostic(
            "generated_artifacts",
            format!(
                "promotable {} package declares no generated-artifact freshness or deterministic export coverage",
                package.kind
            ),
            REMEDIATION_ARTIFACTS,
        ));
    }
    for artifact in &package.generated_artifacts {
        push_blank(&artifact.generation_command, &format!("{} generation_command", artifact.path), diagnostics);
        push_blank(&artifact.validation_command, &format!("{} validation_command", artifact.path), diagnostics);
        let Some(text) = text_at(input.artifact_texts, &artifact.path) else {
            diagnostics.push(diagnostic(
                &artifact.path,
                format!("generated artifact `{}` is missing from artifact inventory", artifact.path),
                REMEDIATION_ARTIFACTS,
            ));
            continue;
        };
        let Some(manifest) = input.artifact_manifest else {
            diagnostics.push(diagnostic(
                &artifact.path,
                "generated artifact freshness requires an artifact manifest".to_owned(),
                REMEDIATION_ARTIFACTS,
            ));
            continue;
        };
        validate_artifact_identity(manifest, &artifact.path, text, diagnostics);
    }
}

fn validate_artifact_identity(
    manifest: &ArtifactManifest,
    path: &str,
    text: &str,
    diagnostics: &mut Vec<CustomsDiagnostic>,
) {
    let expected = artifact_text_identity(path, text);
    let mut entries = manifest.artifacts.iter().filter(|entry| entry.path == path);
    match (entries.next(), entries.next()) {
        (Some(_), Some(_)) => diagnostics.push(diagnostic(
            path,
            format!("artifact manifest has duplicate entries for generated artifact `{path}`"),
            REMEDIATION_ARTIFACTS,
        )),
        (Some(actual), None) if actual.blake3 == expected.blake3 && actual.bytes == expected.bytes => {}
        (Some(actual), None) => diagnostics.push(diagnostic(
            path,
            format!(
                "stale generated artifact `{path}`; expected blake3 {} bytes {}, got blake3 {} bytes {}",
                expected.blake3, expected.bytes, actual.blake3, actual.bytes
            ),
            REMEDIATION_ARTIFACTS,
        )),
        (None, _) => diagnostics.push(diagnostic(
            path,
            format!("artifact manifest is missing generated artifact `{path}`"),
            REMEDIATION_ARTIFACTS,
        )),
    }
}

fn validate_offline_boundary(package: &PolicyPackageManifest, diagnostics: &mut Vec<CustomsDiagnostic>) {
    if package.remote_fetches_allowed {
        return;
    }
    for value in package.generation_commands.iter().chain(&package.validation_commands).chain(
        package
            .generated_artifacts
            .iter()
            .flat_map(|artifact| [&artifact.generation_command, &artifact.validation_command]),
    ) {
        if contains_remote_fetch(value) {
            diagnostics.push(diagnostic(
                "offline",
                format!("customs validation command `{value}` performs or references a remote fetch"),
                REMEDIATION_OFFLINE,
            ));
        }
    }
}

fn validate_secret_boundary(input: &CustomsInput<'_>, diagnostics: &mut Vec<CustomsDiagnostic>) {
    let package = input.package;
    if package.status == STATUS_NON_CANDIDATE && !package.secret_exclusion_rationale.trim().is_empty() {
        return;
    }
    let sources = package.source_paths.iter().map(|path| ("package source", path, input.sources));
    let fixtures = package
        .positive_fixtures
        .iter()
        .chain(&package.negative_fixtures)
        .map(|path| ("package fixture", path, input.sources));
    let artifacts = package
        .generated_artifacts
        .iter()
        .map(|artifact| ("generated artifact", &artifact.path, input.artifact_texts));
    for (label, path, texts) in sources.chain(fixtures).chain(artifacts) {
        if text_at(texts, path).is_some_and(contains_secret_marker) {
            diagnostics.push(diagnostic(
                path,
                format!("{label} `{path}` appears to contain raw secret material"),
                REMEDIATION_SECRETS,
            ));
        }
    }
}

fn is_known_status(status: &str) -> bool {
    matches!(status, STATUS_PROMOTABLE | STATUS_EXISTING | STATUS_NON_CANDIDATE | STATUS_DRAFT)
}

fn is_known_kind(kind: &str) -> bool {
    PACKAGE_KINDS.contains(&kind)
}

fn package_is_promotable(package: &PolicyPackageManifest) -> bool {
    matches!(package.status.as_str(), STATUS_PROMOTABLE | STATUS_EXISTING)
}

fn source_exists(sources: &[SourceFile], path: &str) -> bool {
    sources.iter().any(|source| source.path == path)
}

fn text_at<'a>(texts: &'a [SourceFile], path: &str) -> Option<&'a str> {
    texts.iter().find(|text| text.path == path).map(|text| text.text.as_str())
}

fn is_export_command(command: &str) -> bool {
    EXPORT_COMMAND_MARKERS.iter().any(|marker| command.contains(marker))
}

fn contains_remote_fetch(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    REMOTE_FETCH_MARKERS.iter().any(|marker| lowered.contains(marker))
}

fn contains_secret_marker(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    lowered.lines().any(|line| has_private_key_header(line) || has_secret_assignment(line))
}

/// Match PEM private-key headers such as `-----BEGIN EC PRIVATE KEY-----`.
fn has_private_key_header(line: &str) -> bool {
    line.contains(BARE_PRIVATE_KEY_HEADER)
        || line.find(PEM_HEADER_START).is_some_and(|start| line[start..].contains(PRIVATE_KEY_LABEL))
}

/// Match a credential key name followed by optional quotes and whitespace, then `=` or `:`.
fn has_secret_assignment(line: &str) -> bool {
    SECRET_KEY_NAMES.iter().any(|key| {
        line.match_indices(key).any(|(start, _)| {
            let rest = line[start + key.len()..].trim_start_matches(|character: char| {
                character == '"' || character == '\'' || character.is_whitespace()
            });
            rest.starts_with('=') || rest.starts_with(':')
        })
    })
}

fn push_blank(value: &str, field: &str, diagnostics: &mut Vec<CustomsDiagnostic>) {
    if value.trim().is_empty() {
        diagnostics.push(diagnostic(
            field,
            format!("customs package field `{field}` must not be empty"),
            REMEDIATION_FIX_PACKAGE,
        ));
    }
}

fn diagnostic(field: impl Into<String>, message: String, remediation: &str) -> CustomsDiagnostic {
    CustomsDiagnostic {
        field: field.into(),
        message,
        remediation: remediation.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::artifacts::ARTIFACT_MANIFEST_SCHEMA_VERSION;
    use crate::artifacts::ArtifactManifestEntry;
    use crate::model::ConfigReceiptOwnership;
    use crate::model::PolicyRegistry;
    use crate::proof_artifact::artifact_text_identity;

    const PACKAGE_ID: &str = "octet.external-schema-contract-library";
    const FAMILY_ID: &str = "octet.external-schema-contract-library";
    const SOURCE_PATH: &str = "standards/external-schemas/contracts/generated.ncl";
    const POSITIVE_FIXTURE: &str = "standards/external-schemas/fixtures/valid/dependency_audit_report.ncl";
    const NEGATIVE_FIXTURE: &str = "standards/external-schemas/fixtures/invalid/dependency_audit_wrong_type.ncl";
    const ARTIFACT_TEXT: &str = "{ CargoDenyReportV1 = { diagnostics | Array String } }\n";

    #[test]
    fn complete_package_passes_with_deterministic_report() {
        let package = valid_package();
        let registry = valid_registry();
        let artifact_manifest = valid_manifest();
        let sources = valid_sources();
        let artifacts = valid_artifacts();

        let report = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);

        assert_eq!(report.status, "pass");
        assert!(report.diagnostics.is_empty());
        assert_eq!(report.accepted_artifacts, vec![SOURCE_PATH.to_owned()]);
        assert!(report.non_claims[0].contains("does not trust"));
        assert!(report.manual_review_actions[0].contains("review upstream schema"));
    }

    #[test]
    fn missing_registry_fixture_artifact_provenance_and_source_fail() {
        let mut package = valid_package();
        package.registry_family_ids = vec!["octet.missing-family".to_owned()];
        package.negative_fixtures.clear();
        package.provenance.clear();
        package.source_paths.push("missing/source.ncl".to_owned());
        let mut artifact_manifest = valid_manifest();
        artifact_manifest.artifacts[0].blake3 = "stale".to_owned();
        let registry = valid_registry();
        let sources = valid_sources();
        let artifacts = valid_artifacts();

        let report = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);

        assert_eq!(report.status, "fail");
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("not registered")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("no negative fixtures")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.field == "provenance"));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("missing/source.ncl")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("stale generated artifact")));
    }

    #[test]
    fn secret_bearing_and_remote_fetch_packages_fail_closed() {
        let mut package = valid_package();
        package.generation_commands.push("curl https://schema.example.invalid/schema.json".to_owned());
        let mut sources = valid_sources();
        sources.push(SourceFile::new("secret-package.ncl", "token = \"do-not-commit\"\n"));
        package.source_paths.push("secret-package.ncl".to_owned());
        let registry = valid_registry();
        let artifact_manifest = valid_manifest();
        let artifacts = valid_artifacts();

        let report = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);

        assert_eq!(report.status, "fail");
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("remote fetch")));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("raw secret material")));
    }

    #[test]
    fn secret_exclusion_rationale_only_excuses_non_candidate_packages() {
        let mut package = valid_package();
        package.secret_exclusion_rationale = "fixture credentials are inert".to_owned();
        let mut sources = valid_sources();
        sources.push(SourceFile::new("secret-package.ncl", "token = \"do-not-commit\"\n"));
        package.source_paths.push("secret-package.ncl".to_owned());
        let registry = valid_registry();
        let artifact_manifest = valid_manifest();
        let artifacts = valid_artifacts();

        let promotable = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);
        assert!(
            promotable.diagnostics.iter().any(|diagnostic| diagnostic.field == "secret-package.ncl"
                && diagnostic.message.contains("raw secret material")),
            "{:?}",
            promotable.diagnostics
        );

        package.status = STATUS_NON_CANDIDATE.to_owned();
        let excluded = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);
        assert!(
            !excluded.diagnostics.iter().any(|diagnostic| diagnostic.message.contains("raw secret material")),
            "{:?}",
            excluded.diagnostics
        );
    }

    #[test]
    fn secret_scan_covers_fixtures_and_generated_artifacts() {
        let package = valid_package();
        let secret_artifact = "{ password = \"do-not-commit\" }\n";
        let sources = vec![
            SourceFile::new(SOURCE_PATH, ARTIFACT_TEXT),
            SourceFile::new(POSITIVE_FIXTURE, "api_key = \"do-not-commit\"\n"),
            SourceFile::new(NEGATIVE_FIXTURE, "-----BEGIN PRIVATE KEY-----\n"),
        ];
        let artifacts = vec![SourceFile::new(SOURCE_PATH, secret_artifact)];
        let identity = artifact_text_identity(SOURCE_PATH, secret_artifact);
        let mut artifact_manifest = valid_manifest();
        artifact_manifest.artifacts[0].blake3 = identity.blake3;
        artifact_manifest.artifacts[0].bytes = identity.bytes;
        let registry = valid_registry();

        let report = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);

        let secret_fields = report
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.message.contains("raw secret material"))
            .map(|diagnostic| diagnostic.field.as_str())
            .collect::<Vec<_>>();
        assert!(secret_fields.contains(&POSITIVE_FIXTURE), "{:?}", report.diagnostics);
        assert!(secret_fields.contains(&NEGATIVE_FIXTURE), "{:?}", report.diagnostics);
        assert!(secret_fields.contains(&SOURCE_PATH), "{:?}", report.diagnostics);
    }

    #[test]
    fn secret_scan_matches_key_headers_and_assignment_spellings() {
        let secret_texts = [
            "-----BEGIN RSA PRIVATE KEY-----\n",
            "-----BEGIN OPENSSH PRIVATE KEY-----\n",
            "-----BEGIN EC PRIVATE KEY-----\n",
            "-----BEGIN ENCRYPTED PRIVATE KEY-----\n",
            "BEGIN PRIVATE KEY\n",
            "password=\"hunter2\"\n",
            "password = \"hunter2\"\n",
            "{\"api_key\": \"abc\"}\n",
            "{ 'apikey' : 'abc' }\n",
            "API_KEY=abc\n",
            "api_key = \"abc\"\n",
            "passwd: hunter2\n",
            "token=abc\n",
            "token = abc\n",
            "\"token\":\"abc\"\n",
            "client_secret = \"abc\"\n",
            "secret=abc\n",
            "secret = abc\n",
            "private_key: abc\n",
        ];
        let accepted = secret_texts.iter().filter(|text| !contains_secret_marker(text)).collect::<Vec<_>>();
        assert!(accepted.is_empty(), "secret material passed the scan: {accepted:?}");

        // Markers pinned by `nickel-export-core` at the Cargo-locked revision.
        let core_markers = [
            "begin private key",
            "password=",
            "password =",
            "token=",
            "token =",
            "secret=",
            "secret =",
            "api_key=",
            "api_key =",
        ];
        let missed = core_markers
            .iter()
            .filter(|marker| !contains_secret_marker(&format!("{marker}x\n")))
            .collect::<Vec<_>>();
        assert!(missed.is_empty(), "pinned nickel-export-core markers passed the scan: {missed:?}");

        let ordinary_texts = [
            "tokenizer = \"bpe\"\n",
            "secretary: \"Ada\"\n",
            "secret_exclusion_rationale = \"\"\n",
            "passwords are rotated by the operator\n",
            "the token expires after one hour\n",
            "-----BEGIN PUBLIC KEY-----\n",
            "api_keys_rotated = true\n",
        ];
        let rejected = ordinary_texts.iter().filter(|text| contains_secret_marker(text)).collect::<Vec<_>>();
        assert!(rejected.is_empty(), "ordinary text tripped the secret scan: {rejected:?}");
    }

    #[test]
    fn secret_scan_rejects_alternate_spellings_in_package_sources() {
        let mut package = valid_package();
        let mut sources = valid_sources();
        for (path, text) in [
            ("rsa-key.pem", "-----BEGIN RSA PRIVATE KEY-----\nMIIE\n-----END RSA PRIVATE KEY-----\n"),
            ("compact.ncl", "{ password=\"hunter2\" }\n"),
            ("json-key.json", "{\"api_key\": \"abc\"}\n"),
        ] {
            sources.push(SourceFile::new(path, text));
            package.source_paths.push(path.to_owned());
        }
        let registry = valid_registry();
        let artifact_manifest = valid_manifest();
        let artifacts = valid_artifacts();

        let report = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);

        for path in ["rsa-key.pem", "compact.ncl", "json-key.json"] {
            assert!(
                report
                    .diagnostics
                    .iter()
                    .any(|diagnostic| diagnostic.field == path && diagnostic.remediation == REMEDIATION_SECRETS),
                "{path}: {:?}",
                report.diagnostics
            );
        }
    }

    #[test]
    fn promotable_package_requires_registry_family_coverage() {
        let registry = valid_registry();
        let artifact_manifest = valid_manifest();
        let sources = valid_sources();
        let artifacts = valid_artifacts();
        let mut package = valid_package();
        package.registry_family_ids.clear();

        let promotable = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);
        assert_eq!(promotable.status, "fail");
        assert!(
            promotable.diagnostics.iter().any(|diagnostic| diagnostic.field == "registry_family_ids"),
            "{:?}",
            promotable.diagnostics
        );

        package.status = STATUS_DRAFT.to_owned();
        let draft = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);
        assert!(draft.diagnostics.is_empty(), "{:?}", draft.diagnostics);
    }

    #[test]
    fn promotable_export_kinds_require_generated_artifact_or_export_coverage() {
        let registry = valid_registry();
        let artifact_manifest = valid_manifest();
        let sources = valid_sources();
        let artifacts = valid_artifacts();
        let mut package = valid_package();
        package.generated_artifacts.clear();
        for kind in ["policy", "profile", "template", "generated-contract", "contract"] {
            kind.clone_into(&mut package.kind);
            let report = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic.field == "generated_artifacts"),
                "{kind}: {:?}",
                report.diagnostics
            );
        }

        "proof-rail".clone_into(&mut package.kind);
        let proof_rail = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);
        assert!(proof_rail.diagnostics.is_empty(), "{:?}", proof_rail.diagnostics);

        "profile".clone_into(&mut package.kind);
        package.generation_commands =
            vec!["nickel export --format json templates/dynamic-evidence-profiles.ncl".to_owned()];
        let exported = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);
        assert!(exported.diagnostics.is_empty(), "{:?}", exported.diagnostics);

        package.generation_commands.clear();
        package.status = STATUS_DRAFT.to_owned();
        let draft = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);
        assert!(draft.diagnostics.is_empty(), "{:?}", draft.diagnostics);
    }

    #[test]
    fn package_kind_must_match_contract_enum() {
        let registry = valid_registry();
        let artifact_manifest = valid_manifest();
        let sources = valid_sources();
        let artifacts = valid_artifacts();
        let mut package = valid_package();
        for kind in [
            "policy",
            "profile",
            "template",
            "proof-rail",
            "generated-contract",
            "contract",
        ] {
            kind.clone_into(&mut package.kind);
            let report = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);
            assert!(report.diagnostics.is_empty(), "{kind}: {:?}", report.diagnostics);
        }

        package.kind = "plugin".to_owned();
        let report = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);

        assert_eq!(report.status, "fail");
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.field == "kind" && diagnostic.message.contains("unknown package kind")),
            "{:?}",
            report.diagnostics
        );
    }

    #[test]
    fn duplicate_artifact_manifest_paths_fail_closed() {
        let package = valid_package();
        let registry = valid_registry();
        let sources = valid_sources();
        let artifacts = valid_artifacts();
        let mut artifact_manifest = valid_manifest();
        let mut stale_duplicate = artifact_manifest.artifacts[0].clone();
        stale_duplicate.blake3 = "stale".to_owned();
        artifact_manifest.artifacts.push(stale_duplicate);

        let report = report_for(&package, &registry, &artifact_manifest, &sources, &artifacts);

        assert_eq!(report.status, "fail");
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.field == SOURCE_PATH && diagnostic.message.contains("duplicate entries")),
            "{:?}",
            report.diagnostics
        );
    }

    fn report_for(
        package: &PolicyPackageManifest,
        registry: &PolicyRegistry,
        artifact_manifest: &ArtifactManifest,
        sources: &[SourceFile],
        artifacts: &[SourceFile],
    ) -> CustomsReport {
        validate_policy_package(&CustomsInput {
            package,
            registry,
            artifact_manifest: Some(artifact_manifest),
            sources,
            artifact_texts: artifacts,
        })
    }

    fn valid_package() -> PolicyPackageManifest {
        PolicyPackageManifest {
            schema_version: POLICY_PACKAGE_MANIFEST_SCHEMA_VERSION.to_owned(),
            package_id: PACKAGE_ID.to_owned(),
            kind: "generated-contract".to_owned(),
            owner: "standards".to_owned(),
            status: STATUS_PROMOTABLE.to_owned(),
            source_paths: vec![SOURCE_PATH.to_owned()],
            generated_artifacts: vec![PackageGeneratedArtifact {
                path: SOURCE_PATH.to_owned(),
                generation_command: "nix develop -c cargo test -p cargo-octet external_schema_contracts".to_owned(),
                validation_command:
                    "nickel export --format json standards/external-schemas/fixtures/valid/dependency_audit_report.ncl"
                        .to_owned(),
            }],
            registry_family_ids: vec![FAMILY_ID.to_owned()],
            generation_commands: vec!["nix develop -c cargo test -p cargo-octet external_schema_contracts".to_owned()],
            validation_commands: vec!["nix run .#octet-policy-validate".to_owned()],
            positive_fixtures: vec![POSITIVE_FIXTURE.to_owned()],
            negative_fixtures: vec![NEGATIVE_FIXTURE.to_owned()],
            provenance: "checked-in pinned schemas only".to_owned(),
            license_notes: "upstream schema license reviewed before refresh".to_owned(),
            non_claims: vec!["passing customs does not trust the external tool result".to_owned()],
            manual_review_actions: vec!["review upstream schema license and generator caveats".to_owned()],
            remote_fetches_allowed: false,
            secret_exclusion_rationale: String::new(),
        }
    }

    fn valid_registry() -> PolicyRegistry {
        PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: Vec::new(),
            policies: Vec::new(),
            seed_manifests: Vec::new(),
            config_receipt_families: vec![ConfigReceiptFamily {
                family_id: FAMILY_ID.to_owned(),
                ownership: ConfigReceiptOwnership::RustDerived,
                owner_source_path: "cargo-octet/src/external_schema_contracts.rs".to_owned(),
                artifact_path: SOURCE_PATH.to_owned(),
                generation_command: "nix develop -c cargo test -p cargo-octet external_schema_contracts".to_owned(),
                validation_command: "nix develop -c cargo test -p cargo-octet external_schema_contracts".to_owned(),
                status: ConfigReceiptFamilyStatus::Existing,
                config_digest_required: false,
                positive_fixtures: vec![POSITIVE_FIXTURE.to_owned()],
                negative_fixtures: vec![NEGATIVE_FIXTURE.to_owned()],
                exclusion_rationale: String::new(),
            }],
        }
    }

    fn valid_manifest() -> ArtifactManifest {
        let identity = artifact_text_identity(SOURCE_PATH, ARTIFACT_TEXT);
        ArtifactManifest {
            schema_version: ARTIFACT_MANIFEST_SCHEMA_VERSION.to_owned(),
            generated_from: "customs-test".to_owned(),
            registry_schema_version: "test".to_owned(),
            artifacts: vec![ArtifactManifestEntry {
                path: identity.path,
                blake3: identity.blake3,
                bytes: identity.bytes,
            }],
        }
    }

    fn valid_sources() -> Vec<SourceFile> {
        vec![
            SourceFile::new(SOURCE_PATH, ARTIFACT_TEXT),
            SourceFile::new(POSITIVE_FIXTURE, "valid fixture\n"),
            SourceFile::new(NEGATIVE_FIXTURE, "invalid fixture\n"),
        ]
    }

    fn valid_artifacts() -> Vec<SourceFile> {
        vec![SourceFile::new(SOURCE_PATH, ARTIFACT_TEXT)]
    }
}
