use super::*;

#[derive(Default, Clone)]
pub(crate) struct EvidenceRegistryConsumeSelectors {
    pub(super) bundle_kind: Option<String>,
    pub(super) policy_hash: Option<String>,
    pub(super) channel: Option<String>,
    pub(super) release: Option<String>,
    pub(super) platform: Option<String>,
    pub(super) compatibility_class: Option<String>,
    pub(super) index_receipt_hash: Option<String>,
    pub(super) index_verification_receipt_hash: Option<String>,
    pub(super) bundle_receipt_hash: Option<String>,
    pub(super) artifact_role: Option<String>,
    pub(super) artifact_hash: Option<String>,
    pub(super) coverage_audit_receipt_hash: Option<String>,
    pub(super) release_readiness_receipt_hash: Option<String>,
}

pub(crate) fn evidence_registry_consume_selectors(
    args: &[String],
) -> EvidenceRegistryConsumeSelectors {
    EvidenceRegistryConsumeSelectors {
        bundle_kind: read_string_option(args, "--bundle-kind"),
        policy_hash: read_string_option(args, "--policy-hash"),
        channel: read_string_option(args, "--channel"),
        release: read_string_option(args, "--release"),
        platform: read_string_option(args, "--platform"),
        compatibility_class: read_string_option(args, "--compatibility-class"),
        index_receipt_hash: read_string_option(args, "--index-receipt-hash"),
        index_verification_receipt_hash: read_string_option(
            args,
            "--index-verification-receipt-hash",
        ),
        bundle_receipt_hash: read_string_option(args, "--bundle-receipt-hash"),
        artifact_role: read_string_option(args, "--artifact-role"),
        artifact_hash: read_string_option(args, "--artifact-hash"),
        coverage_audit_receipt_hash: read_string_option(args, "--coverage-audit-receipt-hash"),
        release_readiness_receipt_hash: read_string_option(
            args,
            "--release-readiness-receipt-hash",
        ),
    }
}
