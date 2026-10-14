const RECEIPT_SCHEMA_VERSION: &str = "octet-source-acquisition-receipt/v1";
const REQUIRED_NON_CLAIM: &str = "source acquisition records source identity and policy posture only; it is not dependency audit, vulnerability proof, or formal proof";
const DEFAULT_LOCAL_OVERRIDE_POLICY: &str =
    "development-only; not release evidence unless a separate release source identity is declared";
const DIAGNOSTIC_SCHEMA_VERSION: &str = "source_acquisition_schema_version";
const DIAGNOSTIC_EMPTY_LABEL: &str = "source_row_empty_label";
const DIAGNOSTIC_EMPTY_LOCATOR: &str = "source_row_empty_locator";
const DIAGNOSTIC_UNPINNED_GIT: &str = "source_row_unpinned_git";
const DIAGNOSTIC_PATH_RELEASE: &str = "source_row_path_release_eligible";
const DIAGNOSTIC_MISSING_CREDENTIAL: &str = "source_row_missing_credential_boundary";
const DIAGNOSTIC_MISSING_LOCK: &str = "source_row_missing_lock_identity";
const DIAGNOSTIC_STALE_LOCK: &str = "source_row_stale_lock_identity";
const DIAGNOSTIC_MISSING_NON_CLAIM: &str = "source_row_missing_non_claim";
const DIAGNOSTIC_OVERCLAIM: &str = "source_row_overclaim";
const DIAGNOSTIC_TIER_MERGE: &str = "source_evidence_tier_merged";
const DIAGNOSTIC_MISSING_PARENT_ARTIFACT: &str = "source_parent_artifact_missing";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AcquisitionSourceKind {
    Registry,
    PublicGit,
    PrivateSshGit,
    VendoredSnapshot,
    NixFlake,
    LocalOverride,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SourceEvidenceTier {
    PolicySourceValidation,
    ManualReview,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceAcquisitionRow {
    label: String,
    kind: AcquisitionSourceKind,
    locator: String,
    revision: Option<String>,
    lock_identity: Option<String>,
    observed_lock_identity: Option<String>,
    credential_boundary: Option<String>,
    release_eligible: bool,
    local_override_policy: Option<String>,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceAcquisitionReceipt {
    schema_version: String,
    evidence_tier: SourceEvidenceTier,
    rows: Vec<SourceAcquisitionRow>,
    parent_artifacts: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct SourceAcquisitionReport {
    valid: bool,
    diagnostics: Vec<String>,
    source_kinds: Vec<AcquisitionSourceKind>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct CargoSourceMetadata {
    package_label: String,
    source: String,
    precise_revision: Option<String>,
    lock_identity: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NixInputMetadata {
    input_label: String,
    url: String,
    locked_revision: Option<String>,
    nar_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct DependencyTrustSummaryLink {
    source_label: String,
    source_tier: SourceEvidenceTier,
    dependency_trust_tier: &'static str,
    parent_artifact: String,
}

fn validate_source_acquisition_receipt(receipt: &SourceAcquisitionReceipt) -> SourceAcquisitionReport {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != RECEIPT_SCHEMA_VERSION {
        diagnostics.push(format!(
            "{DIAGNOSTIC_SCHEMA_VERSION}: expected `{RECEIPT_SCHEMA_VERSION}` got `{}`",
            receipt.schema_version
        ));
    }
    if receipt.parent_artifacts.is_empty() {
        diagnostics.push(DIAGNOSTIC_MISSING_PARENT_ARTIFACT.to_owned());
    }
    for row in &receipt.rows {
        diagnostics.extend(row_diagnostics(row));
    }
    diagnostics.sort();
    diagnostics.dedup();
    SourceAcquisitionReport {
        valid: diagnostics.is_empty(),
        diagnostics,
        source_kinds: receipt.rows.iter().map(|row| row.kind).collect(),
    }
}

fn row_diagnostics(row: &SourceAcquisitionRow) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if row.label.trim().is_empty() {
        diagnostics.push(DIAGNOSTIC_EMPTY_LABEL.to_owned());
    }
    if row.locator.trim().is_empty() {
        diagnostics.push(format!("{DIAGNOSTIC_EMPTY_LOCATOR}: `{}`", row.label));
    }
    match row.kind {
        AcquisitionSourceKind::Registry | AcquisitionSourceKind::NixFlake => {
            require_lock(row, &mut diagnostics);
        }
        AcquisitionSourceKind::PublicGit => require_git_revision(row, &mut diagnostics),
        AcquisitionSourceKind::PrivateSshGit => validate_private_ssh(row, &mut diagnostics),
        AcquisitionSourceKind::VendoredSnapshot => validate_vendored(row, &mut diagnostics),
        AcquisitionSourceKind::LocalOverride => validate_local_override(row, &mut diagnostics),
    }
    validate_lock_identity(row, &mut diagnostics);
    validate_non_claims(row, &mut diagnostics);
    diagnostics
}

fn require_git_revision(row: &SourceAcquisitionRow, diagnostics: &mut Vec<String>) {
    if row.revision.as_deref().unwrap_or_default().trim().is_empty() {
        diagnostics.push(format!("{DIAGNOSTIC_UNPINNED_GIT}: `{}`", row.label));
    }
}

fn validate_private_ssh(row: &SourceAcquisitionRow, diagnostics: &mut Vec<String>) {
    require_git_revision(row, diagnostics);
    if row.credential_boundary.as_deref().unwrap_or_default().trim().is_empty() {
        diagnostics.push(format!("{DIAGNOSTIC_MISSING_CREDENTIAL}: `{}`", row.label));
    }
}

fn validate_vendored(row: &SourceAcquisitionRow, diagnostics: &mut Vec<String>) {
    require_git_revision(row, diagnostics);
    require_lock(row, diagnostics);
}

fn validate_local_override(row: &SourceAcquisitionRow, diagnostics: &mut Vec<String>) {
    if row.release_eligible {
        diagnostics.push(format!("{DIAGNOSTIC_PATH_RELEASE}: `{}`", row.label));
    }
    if row.local_override_policy.as_deref().unwrap_or_default().trim().is_empty() {
        diagnostics.push(format!("{DIAGNOSTIC_PATH_RELEASE}: `{}` missing local override policy", row.label));
    }
}

fn require_lock(row: &SourceAcquisitionRow, diagnostics: &mut Vec<String>) {
    if row.lock_identity.as_deref().unwrap_or_default().trim().is_empty() {
        diagnostics.push(format!("{DIAGNOSTIC_MISSING_LOCK}: `{}`", row.label));
    }
}

fn validate_lock_identity(row: &SourceAcquisitionRow, diagnostics: &mut Vec<String>) {
    if let (Some(expected), Some(observed)) = (&row.lock_identity, &row.observed_lock_identity)
        && expected != observed
    {
        diagnostics.push(format!("{DIAGNOSTIC_STALE_LOCK}: `{}`", row.label));
    }
}

fn validate_non_claims(row: &SourceAcquisitionRow, diagnostics: &mut Vec<String>) {
    if !row.non_claims.iter().any(|non_claim| non_claim == REQUIRED_NON_CLAIM) {
        diagnostics.push(format!("{DIAGNOSTIC_MISSING_NON_CLAIM}: `{}`", row.label));
    }
    let joined = row.non_claims.join(" ").to_lowercase();
    if joined.contains("vulnerability-free")
        || joined.contains("proves formal proof")
        || joined.contains("is formal proof")
        || joined.contains("proves dependency audit")
        || joined.contains("is dependency audit")
    {
        diagnostics.push(format!("{DIAGNOSTIC_OVERCLAIM}: `{}`", row.label));
    }
}

fn cargo_metadata_to_source_row(metadata: CargoSourceMetadata) -> SourceAcquisitionRow {
    let kind = if metadata.source.starts_with("registry+") {
        AcquisitionSourceKind::Registry
    } else if metadata.source.starts_with("git+ssh") {
        AcquisitionSourceKind::PrivateSshGit
    } else if metadata.source.starts_with("git+") {
        AcquisitionSourceKind::PublicGit
    } else {
        AcquisitionSourceKind::LocalOverride
    };
    SourceAcquisitionRow {
        label: metadata.package_label,
        kind,
        locator: metadata.source,
        revision: metadata.precise_revision,
        lock_identity: Some(metadata.lock_identity.clone()),
        observed_lock_identity: Some(metadata.lock_identity),
        credential_boundary: None,
        release_eligible: kind != AcquisitionSourceKind::LocalOverride,
        local_override_policy: local_override_policy(kind),
        non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
    }
}

fn nix_input_to_source_row(metadata: NixInputMetadata) -> SourceAcquisitionRow {
    SourceAcquisitionRow {
        label: metadata.input_label,
        kind: AcquisitionSourceKind::NixFlake,
        locator: metadata.url,
        revision: metadata.locked_revision,
        lock_identity: Some(metadata.nar_hash.clone()),
        observed_lock_identity: Some(metadata.nar_hash),
        credential_boundary: None,
        release_eligible: true,
        local_override_policy: None,
        non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
    }
}

fn attach_dependency_trust_parent(
    receipt: &SourceAcquisitionReceipt,
    row_label: &str,
    parent_artifact: &str,
) -> Result<DependencyTrustSummaryLink, String> {
    let row = receipt
        .rows
        .iter()
        .find(|row| row.label == row_label)
        .ok_or_else(|| format!("source row `{row_label}` missing for dependency-trust parent link"))?;
    if receipt.evidence_tier != SourceEvidenceTier::PolicySourceValidation
        && receipt.evidence_tier != SourceEvidenceTier::ManualReview
    {
        return Err(DIAGNOSTIC_TIER_MERGE.to_owned());
    }
    Ok(DependencyTrustSummaryLink {
        source_label: row.label.clone(),
        source_tier: receipt.evidence_tier,
        dependency_trust_tier: "dependency-trust-audit",
        parent_artifact: parent_artifact.to_owned(),
    })
}

fn local_override_policy(kind: AcquisitionSourceKind) -> Option<String> {
    if kind == AcquisitionSourceKind::LocalOverride {
        Some(DEFAULT_LOCAL_OVERRIDE_POLICY.to_owned())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOCK_IDENTITY: &str = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const STALE_LOCK_IDENTITY: &str = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";

    #[test]
    fn source_acquisition_accepts_registry_git_private_vendored_nix_and_local_rows() {
        let receipt = valid_receipt(vec![
            row(
                "serde",
                AcquisitionSourceKind::Registry,
                "registry+https://github.com/rust-lang/crates.io-index",
                None,
            ),
            row(
                "public-git",
                AcquisitionSourceKind::PublicGit,
                "git+https://github.com/OnixResearch/trellis",
                Some("abc123"),
            ),
            private_ssh_row(),
            row("vendored-trellis", AcquisitionSourceKind::VendoredSnapshot, "vendor/trellis", Some("trellis-rev")),
            nix_input_to_source_row(NixInputMetadata {
                input_label: "onix-core".to_owned(),
                url: "github:OnixResearch/onix-core".to_owned(),
                locked_revision: Some("flake-rev".to_owned()),
                nar_hash: LOCK_IDENTITY.to_owned(),
            }),
            local_override_row(),
        ]);

        let report = validate_source_acquisition_receipt(&receipt);

        assert!(report.valid, "unexpected diagnostics: {:?}", report.diagnostics);
        assert!(report.source_kinds.contains(&AcquisitionSourceKind::Registry));
        assert!(report.source_kinds.contains(&AcquisitionSourceKind::PrivateSshGit));
        assert!(report.source_kinds.contains(&AcquisitionSourceKind::LocalOverride));
    }

    #[test]
    fn source_acquisition_rejects_unpinned_paths_missing_credentials_stale_locks_and_overclaims() {
        let mut unpinned =
            row("unpinned", AcquisitionSourceKind::PublicGit, "git+https://github.com/OnixResearch/octet", None);
        unpinned.lock_identity = None;
        let mut path_release = local_override_row();
        path_release.release_eligible = true;
        let mut missing_credential = private_ssh_row();
        missing_credential.credential_boundary = None;
        let mut stale = row(
            "stale",
            AcquisitionSourceKind::Registry,
            "registry+https://github.com/rust-lang/crates.io-index",
            None,
        );
        stale.observed_lock_identity = Some(STALE_LOCK_IDENTITY.to_owned());
        let mut weakened = row(
            "weakened",
            AcquisitionSourceKind::Registry,
            "registry+https://github.com/rust-lang/crates.io-index",
            None,
        );
        weakened.non_claims =
            vec!["source acquisition proves dependency audit and vulnerability-free status".to_owned()];
        let receipt = valid_receipt(vec![unpinned, path_release, missing_credential, stale, weakened]);

        let report = validate_source_acquisition_receipt(&receipt);

        assert!(!report.valid);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_UNPINNED_GIT)));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_PATH_RELEASE)));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_MISSING_CREDENTIAL)));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_STALE_LOCK)));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_MISSING_NON_CLAIM)));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_OVERCLAIM)));
    }

    #[test]
    fn source_acquisition_shell_metadata_extractors_are_data_only() {
        let cargo_registry = cargo_metadata_to_source_row(CargoSourceMetadata {
            package_label: "serde".to_owned(),
            source: "registry+https://github.com/rust-lang/crates.io-index".to_owned(),
            precise_revision: None,
            lock_identity: LOCK_IDENTITY.to_owned(),
        });
        let cargo_git = cargo_metadata_to_source_row(CargoSourceMetadata {
            package_label: "trellis".to_owned(),
            source: "git+https://github.com/OnixResearch/trellis".to_owned(),
            precise_revision: Some("abc123".to_owned()),
            lock_identity: LOCK_IDENTITY.to_owned(),
        });
        let nix_input = nix_input_to_source_row(NixInputMetadata {
            input_label: "onix-core".to_owned(),
            url: "github:OnixResearch/onix-core".to_owned(),
            locked_revision: Some("flake-rev".to_owned()),
            nar_hash: LOCK_IDENTITY.to_owned(),
        });

        assert_eq!(cargo_registry.kind, AcquisitionSourceKind::Registry);
        assert_eq!(cargo_git.kind, AcquisitionSourceKind::PublicGit);
        assert_eq!(nix_input.kind, AcquisitionSourceKind::NixFlake);
    }

    #[test]
    fn source_acquisition_parent_link_preserves_evidence_tier_separation() {
        let receipt = valid_receipt(vec![private_ssh_row()]);
        let link = attach_dependency_trust_parent(&receipt, "private-stack", "artifacts/source-acquisition.json")
            .expect("source row should link");
        let mut manual_receipt = receipt.clone();
        manual_receipt.evidence_tier = SourceEvidenceTier::ManualReview;
        let manual_link = attach_dependency_trust_parent(
            &manual_receipt,
            "private-stack",
            "artifacts/source-acquisition-manual.json",
        )
        .expect("manual-review source row should link without tier merging");

        assert_eq!(link.source_label, "private-stack");
        assert_eq!(link.source_tier, SourceEvidenceTier::PolicySourceValidation);
        assert_eq!(link.dependency_trust_tier, "dependency-trust-audit");
        assert_eq!(link.parent_artifact, "artifacts/source-acquisition.json");
        assert_eq!(manual_link.source_tier, SourceEvidenceTier::ManualReview);
    }

    fn valid_receipt(rows: Vec<SourceAcquisitionRow>) -> SourceAcquisitionReceipt {
        SourceAcquisitionReceipt {
            schema_version: RECEIPT_SCHEMA_VERSION.to_owned(),
            evidence_tier: SourceEvidenceTier::PolicySourceValidation,
            rows,
            parent_artifacts: vec!["artifacts/source-boundary.json".to_owned()],
        }
    }

    fn row(label: &str, kind: AcquisitionSourceKind, locator: &str, revision: Option<&str>) -> SourceAcquisitionRow {
        SourceAcquisitionRow {
            label: label.to_owned(),
            kind,
            locator: locator.to_owned(),
            revision: revision.map(str::to_owned),
            lock_identity: Some(LOCK_IDENTITY.to_owned()),
            observed_lock_identity: Some(LOCK_IDENTITY.to_owned()),
            credential_boundary: None,
            release_eligible: true,
            local_override_policy: None,
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }

    fn private_ssh_row() -> SourceAcquisitionRow {
        SourceAcquisitionRow {
            credential_boundary: Some("requires OnixResearch deploy key; not public release source".to_owned()),
            ..row(
                "private-stack",
                AcquisitionSourceKind::PrivateSshGit,
                "git+ssh://git@github.com/OnixResearch/private-stack",
                Some("private-rev"),
            )
        }
    }

    fn local_override_row() -> SourceAcquisitionRow {
        SourceAcquisitionRow {
            label: "local-valence".to_owned(),
            kind: AcquisitionSourceKind::LocalOverride,
            locator: "../valence".to_owned(),
            revision: None,
            lock_identity: Some(LOCK_IDENTITY.to_owned()),
            observed_lock_identity: Some(LOCK_IDENTITY.to_owned()),
            credential_boundary: None,
            release_eligible: false,
            local_override_policy: Some(DEFAULT_LOCAL_OVERRIDE_POLICY.to_owned()),
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }
}
