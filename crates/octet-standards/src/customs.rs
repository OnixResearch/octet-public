//! Local policy-package customs gate core.
//!
//! Customs validation is a deterministic, local review aid. It checks package
//! metadata completeness, registry coverage, fixtures, generated artifact
//! freshness, provenance, secret boundaries, and remote-fetch boundaries without
//! granting trust to the package or its upstream source.

use serde::Deserialize;
use serde::Serialize;

use crate::artifacts::ArtifactManifest;
use crate::artifacts::ArtifactManifestEntry;
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

static SECRET_MARKERS: &[&str] = &["BEGIN PRIVATE KEY", "password =", "token =", "secret =", "api_key ="];
static REMOTE_FETCH_MARKERS: &[&str] = &["https://", "http://", "curl ", "wget ", "git clone"];

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
    /// Package kind, such as policy, profile, template, proof-rail, or generated-contract.
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
    /// Rationale required when raw secrets or non-candidate material is present.
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
    validate_secret_boundary(input.package, input.sources, &mut diagnostics);

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
    push_blank(package.kind.trim(), "kind", diagnostics);
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
    for artifact in &input.package.generated_artifacts {
        push_blank(&artifact.generation_command, &format!("{} generation_command", artifact.path), diagnostics);
        push_blank(&artifact.validation_command, &format!("{} validation_command", artifact.path), diagnostics);
        let Some(text) = artifact_text(input.artifact_texts, &artifact.path) else {
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
    match manifest_entry(manifest, path) {
        Some(actual) if actual.blake3 == expected.blake3 && actual.bytes == expected.bytes => {}
        Some(actual) => diagnostics.push(diagnostic(
            path,
            format!(
                "stale generated artifact `{path}`; expected blake3 {} bytes {}, got blake3 {} bytes {}",
                expected.blake3, expected.bytes, actual.blake3, actual.bytes
            ),
            REMEDIATION_ARTIFACTS,
        )),
        None => diagnostics.push(diagnostic(
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

fn validate_secret_boundary(
    package: &PolicyPackageManifest,
    sources: &[SourceFile],
    diagnostics: &mut Vec<CustomsDiagnostic>,
) {
    if !package.secret_exclusion_rationale.trim().is_empty() {
        return;
    }
    for path in &package.source_paths {
        let Some(source) = sources.iter().find(|source| source.path == *path) else {
            continue;
        };
        if contains_secret_marker(&source.text) {
            diagnostics.push(diagnostic(
                path,
                format!("package source `{path}` appears to contain raw secret material"),
                REMEDIATION_SECRETS,
            ));
        }
    }
}

fn is_known_status(status: &str) -> bool {
    matches!(status, STATUS_PROMOTABLE | STATUS_EXISTING | STATUS_NON_CANDIDATE | STATUS_DRAFT)
}

fn package_is_promotable(package: &PolicyPackageManifest) -> bool {
    matches!(package.status.as_str(), STATUS_PROMOTABLE | STATUS_EXISTING)
}

fn source_exists(sources: &[SourceFile], path: &str) -> bool {
    sources.iter().any(|source| source.path == path)
}

fn artifact_text<'a>(artifacts: &'a [SourceFile], path: &str) -> Option<&'a str> {
    artifacts.iter().find(|artifact| artifact.path == path).map(|artifact| artifact.text.as_str())
}

fn manifest_entry<'a>(manifest: &'a ArtifactManifest, path: &str) -> Option<&'a ArtifactManifestEntry> {
    manifest.artifacts.iter().find(|entry| entry.path == path)
}

fn contains_remote_fetch(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    REMOTE_FETCH_MARKERS.iter().any(|marker| lowered.contains(marker))
}

fn contains_secret_marker(value: &str) -> bool {
    let lowered = value.to_ascii_lowercase();
    SECRET_MARKERS.iter().any(|marker| lowered.contains(&marker.to_ascii_lowercase()))
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
