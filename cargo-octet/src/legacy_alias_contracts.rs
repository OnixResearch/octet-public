const REQUIRED_NON_CLAIM: &str =
    "legacy identity approval preserves compatibility only; it does not make the legacy identity canonical";
const DIAGNOSTIC_MISSING_CONTRACT: &str = "missing_alias_contract";
const DIAGNOSTIC_SURFACE_NOT_ALLOWED: &str = "alias_surface_not_allowed";
const DIAGNOSTIC_BLOCKED_ALIAS: &str = "blocked_alias_occurrence";
const DIAGNOSTIC_MISSING_RETIREMENT: &str = "missing_retirement_condition";
const DIAGNOSTIC_MISSING_NON_CLAIM: &str = "missing_alias_non_claim";
const DIAGNOSTIC_MISSING_OWNER: &str = "missing_alias_owner";
const DIAGNOSTIC_MISSING_CANONICAL_ID: &str = "missing_canonical_id";
const DIAGNOSTIC_MISSING_LEGACY_ID: &str = "missing_legacy_id";
const DIAGNOSTIC_MISSING_ALLOWED_SURFACE: &str = "missing_allowed_surface";
const DIAGNOSTIC_OVERCLAIM_CANONICAL: &str = "legacy_alias_claims_canonical_identity";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AliasStatus {
    Approved,
    Deprecated,
    Blocked,
    ArchiveOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AliasSurface {
    Active,
    Compatibility,
    Archive,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum AliasClassification {
    Approved,
    Deprecated,
    Blocked,
    ArchiveOnly,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LegacyAliasContract {
    canonical_id: String,
    legacy_id: String,
    allowed_surfaces: Vec<AliasSurface>,
    owner: String,
    status: AliasStatus,
    retirement_condition: String,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct LegacyAliasOccurrence {
    legacy_id: String,
    surface: AliasSurface,
    path: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AliasOccurrenceReport {
    legacy_id: String,
    canonical_id: String,
    path: String,
    classification: AliasClassification,
    non_claim: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AliasValidationReport {
    valid: bool,
    diagnostics: Vec<String>,
    occurrences: Vec<AliasOccurrenceReport>,
}

fn validate_alias_occurrences(
    contracts: &[LegacyAliasContract],
    occurrences: &[LegacyAliasOccurrence],
) -> AliasValidationReport {
    let mut diagnostics = contract_diagnostics(contracts);
    let mut reports = Vec::new();
    for occurrence in occurrences {
        match contracts.iter().find(|contract| contract.legacy_id == occurrence.legacy_id) {
            Some(contract) => validate_occurrence(contract, occurrence, &mut diagnostics, &mut reports),
            None => diagnostics.push(format!(
                "{DIAGNOSTIC_MISSING_CONTRACT}: legacy alias `{}` at `{}` has no approved contract",
                occurrence.legacy_id, occurrence.path
            )),
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    reports.sort_by(|left, right| left.legacy_id.cmp(&right.legacy_id).then(left.path.cmp(&right.path)));
    AliasValidationReport {
        valid: diagnostics.is_empty(),
        diagnostics,
        occurrences: reports,
    }
}

fn contract_diagnostics(contracts: &[LegacyAliasContract]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for contract in contracts {
        push_required(
            &mut diagnostics,
            &contract.legacy_id,
            DIAGNOSTIC_MISSING_CANONICAL_ID,
            "canonical_id",
            &contract.canonical_id,
        );
        push_required(
            &mut diagnostics,
            &contract.legacy_id,
            DIAGNOSTIC_MISSING_LEGACY_ID,
            "legacy_id",
            &contract.legacy_id,
        );
        push_required(&mut diagnostics, &contract.legacy_id, DIAGNOSTIC_MISSING_OWNER, "owner", &contract.owner);
        push_required(
            &mut diagnostics,
            &contract.legacy_id,
            DIAGNOSTIC_MISSING_RETIREMENT,
            "retirement_condition",
            &contract.retirement_condition,
        );
        if contract.allowed_surfaces.is_empty() {
            diagnostics.push(format!(
                "{DIAGNOSTIC_MISSING_ALLOWED_SURFACE}: legacy alias `{}` declares no allowed surfaces",
                contract.legacy_id
            ));
        }
        validate_non_claims(contract, &mut diagnostics);
    }
    diagnostics
}

fn push_required(diagnostics: &mut Vec<String>, legacy_id: &str, diagnostic_code: &str, field_name: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics
            .push(format!("{diagnostic_code}: legacy alias `{legacy_id}` field `{field_name}` must not be empty"));
    }
}

fn validate_non_claims(contract: &LegacyAliasContract, diagnostics: &mut Vec<String>) {
    if !contract.non_claims.iter().any(|non_claim| non_claim == REQUIRED_NON_CLAIM) {
        diagnostics.push(format!(
            "{DIAGNOSTIC_MISSING_NON_CLAIM}: legacy alias `{}` missing required compatibility non-claim",
            contract.legacy_id
        ));
    }
    let joined = contract.non_claims.join(" ").to_lowercase();
    if joined.contains("legacy identity is canonical") || joined.contains("legacy name is canonical") {
        diagnostics.push(format!(
            "{DIAGNOSTIC_OVERCLAIM_CANONICAL}: legacy alias `{}` claims canonical identity",
            contract.legacy_id
        ));
    }
}

fn validate_occurrence(
    contract: &LegacyAliasContract,
    occurrence: &LegacyAliasOccurrence,
    diagnostics: &mut Vec<String>,
    reports: &mut Vec<AliasOccurrenceReport>,
) {
    if contract.status == AliasStatus::Blocked {
        diagnostics.push(format!(
            "{DIAGNOSTIC_BLOCKED_ALIAS}: legacy alias `{}` is blocked at `{}`",
            occurrence.legacy_id, occurrence.path
        ));
        return;
    }
    if !contract.allowed_surfaces.iter().any(|surface| surface == &occurrence.surface) {
        diagnostics.push(format!(
            "{DIAGNOSTIC_SURFACE_NOT_ALLOWED}: legacy alias `{}` is not allowed on {:?} surface `{}`",
            occurrence.legacy_id, occurrence.surface, occurrence.path
        ));
        return;
    }
    reports.push(AliasOccurrenceReport {
        legacy_id: occurrence.legacy_id.clone(),
        canonical_id: contract.canonical_id.clone(),
        path: occurrence.path.clone(),
        classification: classify_occurrence(contract.status, occurrence.surface),
        non_claim: REQUIRED_NON_CLAIM.to_owned(),
    });
}

fn classify_occurrence(status: AliasStatus, surface: AliasSurface) -> AliasClassification {
    match (status, surface) {
        (AliasStatus::Blocked, _) => AliasClassification::Blocked,
        (AliasStatus::ArchiveOnly, _) | (_, AliasSurface::Archive) => AliasClassification::ArchiveOnly,
        (AliasStatus::Deprecated, _) => AliasClassification::Deprecated,
        (AliasStatus::Approved, _) => AliasClassification::Approved,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OCTET_ID: &str = "octet";
    const TIGERSTYLE_ID: &str = "tigerstyle";
    const SLOTCAR_ID: &str = "slotcar";
    const RETIREMENT_CONDITION: &str =
        "remove after namespace migration and archived replay docs no longer need the alias";

    #[test]
    fn legacy_alias_contracts_accept_approved_deprecated_and_archive_only_surfaces() {
        let contracts = vec![
            alias_contract(TIGERSTYLE_ID, AliasStatus::Approved, vec![AliasSurface::Compatibility]),
            alias_contract(SLOTCAR_ID, AliasStatus::Deprecated, vec![
                AliasSurface::Compatibility,
                AliasSurface::Archive,
            ]),
            alias_contract("tigerstyle-archive", AliasStatus::ArchiveOnly, vec![AliasSurface::Archive]),
        ];
        let occurrences = vec![
            occurrence(TIGERSTYLE_ID, AliasSurface::Compatibility, "README.md"),
            occurrence(SLOTCAR_ID, AliasSurface::Compatibility, "docs/consumer-guide.md"),
            occurrence("tigerstyle-archive", AliasSurface::Archive, "cairn/archive/2026-01-01-legacy/tasks.md"),
        ];

        let report = validate_alias_occurrences(&contracts, &occurrences);

        assert!(report.valid, "unexpected diagnostics: {:?}", report.diagnostics);
        assert!(report.occurrences.iter().any(|item| item.classification == AliasClassification::Approved));
        assert!(report.occurrences.iter().any(|item| item.classification == AliasClassification::Deprecated));
        assert!(report.occurrences.iter().any(|item| item.classification == AliasClassification::ArchiveOnly));
        assert!(report.occurrences.iter().all(|item| item.non_claim == REQUIRED_NON_CLAIM));
    }

    #[test]
    fn legacy_alias_contracts_reject_unapproved_active_aliases_and_missing_retirement() {
        let mut contracts = vec![alias_contract(TIGERSTYLE_ID, AliasStatus::Approved, vec![
            AliasSurface::Compatibility,
        ])];
        contracts[0].retirement_condition.clear();
        let occurrences = vec![
            occurrence(TIGERSTYLE_ID, AliasSurface::Active, "src/lib.rs"),
            occurrence("basalt", AliasSurface::Active, "docs/adoption-story.md"),
        ];

        let report = validate_alias_occurrences(&contracts, &occurrences);

        assert!(!report.valid);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_MISSING_RETIREMENT)));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_SURFACE_NOT_ALLOWED)));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_MISSING_CONTRACT)));
    }

    #[test]
    fn legacy_alias_contracts_reject_blocked_aliases_and_canonical_overclaims() {
        let mut blocked = alias_contract("molten", AliasStatus::Blocked, vec![AliasSurface::Compatibility]);
        blocked.non_claims.push("legacy identity is canonical for new docs".to_owned());
        let report = validate_alias_occurrences(&[blocked], &[occurrence(
            "molten",
            AliasSurface::Compatibility,
            "docs/index.md",
        )]);

        assert!(!report.valid);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_BLOCKED_ALIAS)));
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains(DIAGNOSTIC_OVERCLAIM_CANONICAL)));
    }

    fn alias_contract(
        legacy_id: &str,
        status: AliasStatus,
        allowed_surfaces: Vec<AliasSurface>,
    ) -> LegacyAliasContract {
        LegacyAliasContract {
            canonical_id: OCTET_ID.to_owned(),
            legacy_id: legacy_id.to_owned(),
            allowed_surfaces,
            owner: "octet maintainers".to_owned(),
            status,
            retirement_condition: RETIREMENT_CONDITION.to_owned(),
            non_claims: vec![REQUIRED_NON_CLAIM.to_owned()],
        }
    }

    fn occurrence(legacy_id: &str, surface: AliasSurface, path: &str) -> LegacyAliasOccurrence {
        LegacyAliasOccurrence {
            legacy_id: legacy_id.to_owned(),
            surface,
            path: path.to_owned(),
        }
    }
}
