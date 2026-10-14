const BLAKE3_PREFIX: &str = "b3:";
const REQUIRED_NON_CLAIM: &str = "source-boundary validation proves descriptor shape and identity completeness only";
const OVERCLAIM_PATTERNS: &[&str] = &[
    "proves dependency trust",
    "proves code correctness",
    "proves supply-chain safety",
    "proves release eligibility",
    "proves verifier soundness",
    "proves downstream behavioral correctness",
];
const DIAGNOSTIC_EMPTY_DESCRIPTOR_ID: &str = "empty_descriptor_id";
const DIAGNOSTIC_UNKNOWN_KIND: &str = "unknown_source_kind";
const DIAGNOSTIC_MISSING_REVISION: &str = "missing_revision";
const DIAGNOSTIC_DIRTY_DISALLOWED: &str = "dirty_source_disallowed";
const DIAGNOSTIC_RELEASE_LOCAL_OVERRIDE: &str = "local_override_release_eligible";
const DIAGNOSTIC_PATH_ESCAPE: &str = "selected_path_escape";
const DIAGNOSTIC_MISSING_DIGEST: &str = "missing_digest_inventory";
const DIAGNOSTIC_STALE_DIGEST: &str = "stale_digest_inventory";
const DIAGNOSTIC_MISSING_UPSTREAM: &str = "missing_upstream_source";
const DIAGNOSTIC_MISSING_PACKAGE: &str = "missing_registry_package";
const DIAGNOSTIC_MISSING_GENERATOR: &str = "missing_generator_identity";
const DIAGNOSTIC_MISSING_NON_CLAIM: &str = "missing_source_boundary_non_claim";
const DIAGNOSTIC_OVERCLAIM: &str = "source_boundary_overclaim";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceKind {
    Git,
    LocalOverride,
    VendoredSnapshot,
    RegistryPackage,
    GeneratedArtifact,
    UnknownManualReview,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirtyPolicy {
    CleanRequired,
    DirtyAllowedWithRationale,
    ManualReviewOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DigestRecord {
    path: String,
    expected_blake3: String,
    actual_blake3: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceDescriptor {
    descriptor_id: String,
    kind: SourceKind,
    revision: Option<String>,
    selected_paths: Vec<String>,
    digests: Vec<DigestRecord>,
    dirty: bool,
    dirty_policy: DirtyPolicy,
    release_eligible: bool,
    upstream_source: Option<String>,
    registry_package: Option<String>,
    generated_by: Option<String>,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceBoundaryReport {
    valid: bool,
    normalized_id: String,
    diagnostics: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ExistingSourceReceipt {
    source_id: String,
    source_kind: SourceKind,
    revision: Option<String>,
    path: String,
    digest: String,
    release_eligible: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DependencyTrustReceipt {
    dependency_id: String,
    source_kind: SourceKind,
    source_revision: Option<String>,
    selected_path: String,
    source_digest: String,
}

fn validate_source_descriptor(descriptor: &SourceDescriptor) -> SourceBoundaryReport {
    let mut diagnostics = Vec::new();
    if descriptor.descriptor_id.trim().is_empty() {
        diagnostics.push(DIAGNOSTIC_EMPTY_DESCRIPTOR_ID.to_owned());
    }
    diagnostics.extend(kind_diagnostics(descriptor));
    diagnostics.extend(path_diagnostics(&descriptor.selected_paths));
    diagnostics.extend(digest_diagnostics(&descriptor.digests));
    diagnostics.extend(non_claim_diagnostics(&descriptor.non_claims));
    diagnostics.sort();
    diagnostics.dedup();
    SourceBoundaryReport {
        valid: diagnostics.is_empty(),
        normalized_id: normalize_descriptor_id(&descriptor.descriptor_id),
        diagnostics,
    }
}

fn kind_diagnostics(descriptor: &SourceDescriptor) -> Vec<String> {
    let mut diagnostics = Vec::new();
    match descriptor.kind {
        SourceKind::Git => require_revision(descriptor, &mut diagnostics),
        SourceKind::LocalOverride => validate_local_override(descriptor, &mut diagnostics),
        SourceKind::VendoredSnapshot => validate_vendored_snapshot(descriptor, &mut diagnostics),
        SourceKind::RegistryPackage => validate_registry_package(descriptor, &mut diagnostics),
        SourceKind::GeneratedArtifact => validate_generated_artifact(descriptor, &mut diagnostics),
        SourceKind::UnknownManualReview => diagnostics.push(DIAGNOSTIC_UNKNOWN_KIND.to_owned()),
    }
    if descriptor.dirty && descriptor.dirty_policy == DirtyPolicy::CleanRequired {
        diagnostics.push(DIAGNOSTIC_DIRTY_DISALLOWED.to_owned());
    }
    diagnostics
}

fn require_revision(descriptor: &SourceDescriptor, diagnostics: &mut Vec<String>) {
    if descriptor.revision.as_deref().unwrap_or_default().trim().is_empty() {
        diagnostics.push(DIAGNOSTIC_MISSING_REVISION.to_owned());
    }
}

fn validate_local_override(descriptor: &SourceDescriptor, diagnostics: &mut Vec<String>) {
    if descriptor.release_eligible {
        diagnostics.push(DIAGNOSTIC_RELEASE_LOCAL_OVERRIDE.to_owned());
    }
    if descriptor.dirty && descriptor.dirty_policy != DirtyPolicy::DirtyAllowedWithRationale {
        diagnostics.push(DIAGNOSTIC_DIRTY_DISALLOWED.to_owned());
    }
}

fn validate_vendored_snapshot(descriptor: &SourceDescriptor, diagnostics: &mut Vec<String>) {
    require_revision(descriptor, diagnostics);
    if descriptor.upstream_source.as_deref().unwrap_or_default().trim().is_empty() {
        diagnostics.push(DIAGNOSTIC_MISSING_UPSTREAM.to_owned());
    }
}

fn validate_registry_package(descriptor: &SourceDescriptor, diagnostics: &mut Vec<String>) {
    if descriptor.registry_package.as_deref().unwrap_or_default().trim().is_empty() {
        diagnostics.push(DIAGNOSTIC_MISSING_PACKAGE.to_owned());
    }
}

fn validate_generated_artifact(descriptor: &SourceDescriptor, diagnostics: &mut Vec<String>) {
    if descriptor.generated_by.as_deref().unwrap_or_default().trim().is_empty() {
        diagnostics.push(DIAGNOSTIC_MISSING_GENERATOR.to_owned());
    }
}

fn path_diagnostics(paths: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if paths.is_empty() {
        diagnostics.push(DIAGNOSTIC_PATH_ESCAPE.to_owned());
    }
    for path in paths {
        if path.starts_with('/') || path.split('/').any(|segment| segment == "..") || path.trim().is_empty() {
            diagnostics.push(format!("{DIAGNOSTIC_PATH_ESCAPE}: `{path}`"));
        }
    }
    diagnostics
}

fn digest_diagnostics(digests: &[DigestRecord]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if digests.is_empty() {
        diagnostics.push(DIAGNOSTIC_MISSING_DIGEST.to_owned());
    }
    for digest in digests {
        if digest.path.trim().is_empty()
            || !digest.expected_blake3.starts_with(BLAKE3_PREFIX)
            || !digest.actual_blake3.starts_with(BLAKE3_PREFIX)
        {
            diagnostics.push(DIAGNOSTIC_MISSING_DIGEST.to_owned());
        }
        if digest.expected_blake3 != digest.actual_blake3 {
            diagnostics.push(format!("{DIAGNOSTIC_STALE_DIGEST}: `{}`", digest.path));
        }
    }
    diagnostics
}

fn non_claim_diagnostics(non_claims: &[String]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if !non_claims.iter().any(|non_claim| non_claim == REQUIRED_NON_CLAIM) {
        diagnostics.push(DIAGNOSTIC_MISSING_NON_CLAIM.to_owned());
    }
    let joined = non_claims.join(" ").to_lowercase();
    for pattern in OVERCLAIM_PATTERNS {
        if joined.contains(pattern) {
            diagnostics.push(format!("{DIAGNOSTIC_OVERCLAIM}: {pattern}"));
        }
    }
    diagnostics
}

fn normalize_descriptor_id(descriptor_id: &str) -> String {
    descriptor_id.trim().to_ascii_lowercase().replace(' ', "-")
}

fn adapt_source_receipt(receipt: ExistingSourceReceipt) -> SourceDescriptor {
    SourceDescriptor {
        descriptor_id: receipt.source_id,
        kind: receipt.source_kind,
        revision: receipt.revision,
        selected_paths: vec![receipt.path.clone()],
        digests: vec![digest(&receipt.path, &receipt.digest)],
        dirty: false,
        dirty_policy: DirtyPolicy::CleanRequired,
        release_eligible: receipt.release_eligible,
        upstream_source: None,
        registry_package: None,
        generated_by: None,
        non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
    }
}

fn adapt_dependency_trust_receipt(receipt: DependencyTrustReceipt) -> SourceDescriptor {
    SourceDescriptor {
        descriptor_id: receipt.dependency_id,
        kind: receipt.source_kind,
        revision: receipt.source_revision,
        selected_paths: vec![receipt.selected_path.clone()],
        digests: vec![digest(&receipt.selected_path, &receipt.source_digest)],
        dirty: false,
        dirty_policy: DirtyPolicy::CleanRequired,
        release_eligible: true,
        upstream_source: None,
        registry_package: Some("registry-package-from-dependency-trust".to_owned()),
        generated_by: None,
        non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
    }
}

fn digest(path: &str, value: &str) -> DigestRecord {
    DigestRecord {
        path: path.to_owned(),
        expected_blake3: value.to_owned(),
        actual_blake3: value.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_DIGEST: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const STALE_DIGEST: &str = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn source_boundary_accepts_supported_descriptor_kinds() {
        let descriptors = vec![
            git_descriptor(),
            local_override_descriptor(),
            vendored_descriptor(),
            registry_descriptor(),
            generated_descriptor(),
        ];

        for descriptor in descriptors {
            let report = validate_source_descriptor(&descriptor);
            assert!(report.valid, "{} diagnostics: {:?}", descriptor.descriptor_id, report.diagnostics);
            assert_eq!(report.normalized_id, normalize_descriptor_id(&descriptor.descriptor_id));
        }
    }

    #[test]
    fn source_boundary_rejects_missing_escape_stale_unknown_and_overclaim_inputs() {
        let mut missing_revision = git_descriptor();
        missing_revision.revision = None;
        let mut dirty_disallowed = git_descriptor();
        dirty_disallowed.dirty = true;
        let mut path_escape = git_descriptor();
        path_escape.selected_paths = vec!["../outside".to_owned(), "/absolute".to_owned()];
        let mut missing_digest = git_descriptor();
        missing_digest.digests.clear();
        let mut stale_vendored = vendored_descriptor();
        stale_vendored.digests[0].actual_blake3 = STALE_DIGEST.to_owned();
        let unknown = SourceDescriptor {
            kind: SourceKind::UnknownManualReview,
            ..git_descriptor()
        };
        let mut overclaim = git_descriptor();
        overclaim.non_claims.push("proves dependency trust and proves code correctness".to_owned());

        let cases = vec![
            (missing_revision, DIAGNOSTIC_MISSING_REVISION),
            (dirty_disallowed, DIAGNOSTIC_DIRTY_DISALLOWED),
            (path_escape, DIAGNOSTIC_PATH_ESCAPE),
            (missing_digest, DIAGNOSTIC_MISSING_DIGEST),
            (stale_vendored, DIAGNOSTIC_STALE_DIGEST),
            (unknown, DIAGNOSTIC_UNKNOWN_KIND),
            (overclaim, DIAGNOSTIC_OVERCLAIM),
        ];

        for (descriptor, expected) in cases {
            let report = validate_source_descriptor(&descriptor);
            assert!(!report.valid, "{expected} should fail for {}", descriptor.descriptor_id);
            assert!(
                report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
                "expected {expected} in {:?}",
                report.diagnostics
            );
        }
    }

    #[test]
    fn source_boundary_adapters_preserve_existing_receipt_source_fields() {
        let source_receipt = ExistingSourceReceipt {
            source_id: "octet-source-receipt".to_owned(),
            source_kind: SourceKind::Git,
            revision: Some("abc123".to_owned()),
            path: "src/lib.rs".to_owned(),
            digest: VALID_DIGEST.to_owned(),
            release_eligible: true,
        };
        let dependency_receipt = DependencyTrustReceipt {
            dependency_id: "octet-dependency-receipt".to_owned(),
            source_kind: SourceKind::RegistryPackage,
            source_revision: None,
            selected_path: "Cargo.lock".to_owned(),
            source_digest: VALID_DIGEST.to_owned(),
        };

        let source_descriptor = adapt_source_receipt(source_receipt);
        let dependency_descriptor = adapt_dependency_trust_receipt(dependency_receipt);
        let source_report = validate_source_descriptor(&source_descriptor);
        let dependency_report = validate_source_descriptor(&dependency_descriptor);

        assert!(source_report.valid, "unexpected source diagnostics: {:?}", source_report.diagnostics);
        assert!(dependency_report.valid, "unexpected dependency diagnostics: {:?}", dependency_report.diagnostics);
        assert_eq!(source_descriptor.selected_paths, vec!["src/lib.rs".to_owned()]);
        assert_eq!(dependency_descriptor.registry_package.as_deref(), Some("registry-package-from-dependency-trust"));
    }

    fn git_descriptor() -> SourceDescriptor {
        SourceDescriptor {
            descriptor_id: "Octet Git Source".to_owned(),
            kind: SourceKind::Git,
            revision: Some("abc123".to_owned()),
            selected_paths: vec!["src/lib.rs".to_owned()],
            digests: vec![digest("src/lib.rs", VALID_DIGEST)],
            dirty: false,
            dirty_policy: DirtyPolicy::CleanRequired,
            release_eligible: true,
            upstream_source: Some("git@github.com:OnixResearch/octet.git".to_owned()),
            registry_package: None,
            generated_by: None,
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }

    fn local_override_descriptor() -> SourceDescriptor {
        SourceDescriptor {
            descriptor_id: "Local Override".to_owned(),
            kind: SourceKind::LocalOverride,
            revision: None,
            selected_paths: vec!["local-overrides/valence".to_owned()],
            digests: vec![digest("local-overrides/valence", VALID_DIGEST)],
            dirty: true,
            dirty_policy: DirtyPolicy::DirtyAllowedWithRationale,
            release_eligible: false,
            upstream_source: None,
            registry_package: None,
            generated_by: None,
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }

    fn vendored_descriptor() -> SourceDescriptor {
        SourceDescriptor {
            descriptor_id: "Vendored Trellis".to_owned(),
            kind: SourceKind::VendoredSnapshot,
            revision: Some("trellis-revision".to_owned()),
            selected_paths: vec!["vendor/trellis".to_owned()],
            digests: vec![digest("vendor/trellis", VALID_DIGEST)],
            dirty: false,
            dirty_policy: DirtyPolicy::CleanRequired,
            release_eligible: true,
            upstream_source: Some("github:OnixResearch/trellis".to_owned()),
            registry_package: None,
            generated_by: None,
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }

    fn registry_descriptor() -> SourceDescriptor {
        SourceDescriptor {
            descriptor_id: "Registry Package".to_owned(),
            kind: SourceKind::RegistryPackage,
            revision: None,
            selected_paths: vec!["Cargo.lock".to_owned()],
            digests: vec![digest("Cargo.lock", VALID_DIGEST)],
            dirty: false,
            dirty_policy: DirtyPolicy::CleanRequired,
            release_eligible: true,
            upstream_source: None,
            registry_package: Some("serde 1.x".to_owned()),
            generated_by: None,
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }

    fn generated_descriptor() -> SourceDescriptor {
        SourceDescriptor {
            descriptor_id: "Generated Contract".to_owned(),
            kind: SourceKind::GeneratedArtifact,
            revision: None,
            selected_paths: vec!["standards/evidence/contracts/generated.ncl".to_owned()],
            digests: vec![digest("standards/evidence/contracts/generated.ncl", VALID_DIGEST)],
            dirty: false,
            dirty_policy: DirtyPolicy::ManualReviewOnly,
            release_eligible: true,
            upstream_source: None,
            registry_package: None,
            generated_by: Some("cargo-octet evidence contracts".to_owned()),
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }
}
