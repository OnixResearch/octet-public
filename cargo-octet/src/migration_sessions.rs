#![allow(clippy::module_name_repetitions)]

use serde::Deserialize;
use serde::Serialize;

pub(crate) const SESSION_SCHEMA_VERSION: &str = "octet-structured-migration-session/v1";
pub(crate) const PLAN_REPORT_SCHEMA_VERSION: &str = "octet-migration-plan-report/v1";
pub(crate) const STATUS_REPORT_SCHEMA_VERSION: &str = "octet-migration-status-report/v1";
pub(crate) const VERIFY_REPORT_SCHEMA_VERSION: &str = "octet-migration-verify-report/v1";
pub(crate) const OPERATION_MANIFEST_SCHEMA_VERSION: &str = "octet-migration-operation-manifest/v1";
pub(crate) const MIGRATION_NON_CLAIM_BOUNDARY: &str = "Structured migration sessions prove reviewed name-pointer and artifact linkage only; they do not prove behavioral correctness, semantic equivalence, source compatibility, or release safety without separate evidence.";

const SUPPORTED_MIGRATION_KINDS: &[&str] = &[
    "namespace",
    "lint-policy",
    "config",
    "command",
    "artifact-layout",
    "evidence-schema",
];
const SUPPORTED_COMPATIBILITY_CLASSES: &[&str] = &["canonical", "compatibility-alias", "deprecated", "removal-target"];
const SUPPORTED_STATUSES: &[&str] = &["planned", "in-progress", "verified", "applied", "blocked"];
const MUTABILITY_ARCHIVE_READ_ONLY: &str = "archive-read-only";
const MUTABILITY_MUTABLE_ARCHIVE: &str = "mutable-archive";
const MUTABILITY_GENERATED: &str = "generated";
const OPERATION_RECORD_ONLY: &str = "record-only";
const OPERATION_REWRITE: &str = "rewrite";
const OPERATION_REPLACE: &str = "replace";
const SAFETY_SAFE: &str = "safe";
const STATUS_OPEN: &str = "open";

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct StructuredMigrationSession {
    pub(crate) schema_version: String,
    pub(crate) session_id: String,
    pub(crate) migration_kind: String,
    pub(crate) old_pointer: MigrationPointer,
    pub(crate) new_pointer: MigrationPointer,
    #[serde(default)]
    pub(crate) strict_identity: bool,
    #[serde(default)]
    pub(crate) identity_evidence: Vec<MigrationIdentityEvidence>,
    #[serde(default)]
    pub(crate) compatibility: Vec<MigrationCompatibilityPointer>,
    #[serde(default)]
    pub(crate) affected_surfaces: Vec<MigrationAffectedSurface>,
    #[serde(default)]
    pub(crate) planned_operations: Vec<MigrationPlannedOperation>,
    #[serde(default)]
    pub(crate) verification_commands: Vec<MigrationVerificationCommand>,
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) todos: Vec<MigrationTodo>,
    #[serde(default)]
    pub(crate) non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationPointer {
    pub(crate) pointer_kind: String,
    pub(crate) name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) locator: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationIdentityEvidence {
    pub(crate) identity_kind: String,
    pub(crate) value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) expected_blake3: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) actual_blake3: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationCompatibilityPointer {
    pub(crate) pointer: MigrationPointer,
    pub(crate) classification: String,
    #[serde(default)]
    pub(crate) coverage: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) removal_boundary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) blocking_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationAffectedSurface {
    pub(crate) surface_kind: String,
    pub(crate) locator: String,
    pub(crate) mutability: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) identity: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationPlannedOperation {
    pub(crate) operation_id: String,
    pub(crate) operation_kind: String,
    pub(crate) target_locator: String,
    pub(crate) safety: String,
    pub(crate) dry_run_preview: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) archive_mutation_rationale: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationVerificationCommand {
    pub(crate) command: String,
    pub(crate) expected: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationTodo {
    pub(crate) item: String,
    pub(crate) status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) blocker: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationPlanReport {
    pub(crate) schema_version: &'static str,
    pub(crate) session_id: String,
    pub(crate) migration_kind: String,
    pub(crate) status: String,
    pub(crate) old_pointer: MigrationPointer,
    pub(crate) new_pointer: MigrationPointer,
    pub(crate) live_surfaces: Vec<MigrationAffectedSurface>,
    pub(crate) archive_surfaces: Vec<MigrationAffectedSurface>,
    pub(crate) generated_artifacts: Vec<MigrationAffectedSurface>,
    pub(crate) compatibility_aliases: Vec<MigrationCompatibilityPointer>,
    pub(crate) planned_operations: Vec<MigrationPlannedOperation>,
    pub(crate) verification_commands: Vec<MigrationVerificationCommand>,
    pub(crate) unresolved_todos: Vec<MigrationTodo>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) non_claim_boundary: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationStatusReport {
    pub(crate) schema_version: &'static str,
    pub(crate) session_id: String,
    pub(crate) migration_kind: String,
    pub(crate) status: String,
    pub(crate) affected_surface_count: usize,
    pub(crate) compatibility_alias_count: usize,
    pub(crate) archive_surface_count: usize,
    pub(crate) planned_operation_count: usize,
    pub(crate) unresolved_todo_count: usize,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) non_claim_boundary: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationVerifyReport {
    pub(crate) schema_version: &'static str,
    pub(crate) session_id: String,
    pub(crate) valid: bool,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) checked_aliases: Vec<String>,
    pub(crate) checked_archive_surfaces: Vec<String>,
    pub(crate) checked_operations: Vec<String>,
    pub(crate) non_claim_boundary: &'static str,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct MigrationOperationManifest {
    pub(crate) schema_version: &'static str,
    pub(crate) session_id: String,
    pub(crate) dry_run: bool,
    pub(crate) pre_write_manifest: bool,
    pub(crate) operations: Vec<MigrationPlannedOperation>,
    pub(crate) diagnostics: Vec<String>,
    pub(crate) non_claim_boundary: &'static str,
}

pub(crate) fn parse_session(text: &str) -> Result<StructuredMigrationSession, String> {
    serde_json::from_str(text).map_err(|error| format!("parse structured migration session: {error}"))
}

pub(crate) fn validate_session(session: &StructuredMigrationSession) -> Vec<String> {
    let mut diagnostics = Vec::new();
    validate_required_shape(session, &mut diagnostics);
    validate_identity_evidence(session, &mut diagnostics);
    validate_compatibility(session, &mut diagnostics);
    validate_surfaces(session, &mut diagnostics);
    validate_operations(session, &mut diagnostics);
    validate_non_claims(session, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn plan_report(session: &StructuredMigrationSession) -> MigrationPlanReport {
    let diagnostics = validate_session(session);
    MigrationPlanReport {
        schema_version: PLAN_REPORT_SCHEMA_VERSION,
        session_id: session.session_id.clone(),
        migration_kind: session.migration_kind.clone(),
        status: session.status.clone(),
        old_pointer: session.old_pointer.clone(),
        new_pointer: session.new_pointer.clone(),
        live_surfaces: sorted_surfaces(session.affected_surfaces.iter().filter(|surface| {
            surface.mutability != MUTABILITY_ARCHIVE_READ_ONLY && surface.mutability != MUTABILITY_GENERATED
        })),
        archive_surfaces: sorted_surfaces(
            session
                .affected_surfaces
                .iter()
                .filter(|surface| surface.mutability == MUTABILITY_ARCHIVE_READ_ONLY),
        ),
        generated_artifacts: sorted_surfaces(
            session.affected_surfaces.iter().filter(|surface| surface.mutability == MUTABILITY_GENERATED),
        ),
        compatibility_aliases: sorted_compatibility(
            session.compatibility.iter().filter(|pointer| pointer.classification != "canonical"),
        ),
        planned_operations: sorted_operations(session.planned_operations.iter()),
        verification_commands: sorted_commands(session.verification_commands.iter()),
        unresolved_todos: sorted_todos(session.todos.iter().filter(|todo| todo.status == STATUS_OPEN)),
        diagnostics,
        non_claim_boundary: MIGRATION_NON_CLAIM_BOUNDARY,
    }
}

pub(crate) fn status_report(session: &StructuredMigrationSession) -> MigrationStatusReport {
    let diagnostics = validate_session(session);
    MigrationStatusReport {
        schema_version: STATUS_REPORT_SCHEMA_VERSION,
        session_id: session.session_id.clone(),
        migration_kind: session.migration_kind.clone(),
        status: session.status.clone(),
        affected_surface_count: session.affected_surfaces.len(),
        compatibility_alias_count: session
            .compatibility
            .iter()
            .filter(|pointer| pointer.classification != "canonical")
            .count(),
        archive_surface_count: session
            .affected_surfaces
            .iter()
            .filter(|surface| surface.mutability == MUTABILITY_ARCHIVE_READ_ONLY)
            .count(),
        planned_operation_count: session.planned_operations.len(),
        unresolved_todo_count: session.todos.iter().filter(|todo| todo.status == STATUS_OPEN).count(),
        diagnostics,
        non_claim_boundary: MIGRATION_NON_CLAIM_BOUNDARY,
    }
}

pub(crate) fn verify_report(session: &StructuredMigrationSession) -> MigrationVerifyReport {
    let diagnostics = validate_session(session);
    MigrationVerifyReport {
        schema_version: VERIFY_REPORT_SCHEMA_VERSION,
        session_id: session.session_id.clone(),
        valid: diagnostics.is_empty(),
        diagnostics,
        checked_aliases: session.compatibility.iter().map(|alias| alias.pointer.name.clone()).collect::<Vec<_>>(),
        checked_archive_surfaces: session
            .affected_surfaces
            .iter()
            .filter(|surface| surface.mutability == MUTABILITY_ARCHIVE_READ_ONLY)
            .map(|surface| surface.locator.clone())
            .collect::<Vec<_>>(),
        checked_operations: session
            .planned_operations
            .iter()
            .map(|operation| operation.operation_id.clone())
            .collect::<Vec<_>>(),
        non_claim_boundary: MIGRATION_NON_CLAIM_BOUNDARY,
    }
}

pub(crate) fn operation_manifest(session: &StructuredMigrationSession, dry_run: bool) -> MigrationOperationManifest {
    let mut diagnostics = validate_session(session);
    for operation in &session.planned_operations {
        if operation.safety != SAFETY_SAFE || operation.operation_kind != OPERATION_RECORD_ONLY {
            diagnostics.push(format!(
                "apply.unsupported_operation `{}` requires safety=safe and operation_kind=record-only",
                operation.operation_id
            ));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    MigrationOperationManifest {
        schema_version: OPERATION_MANIFEST_SCHEMA_VERSION,
        session_id: session.session_id.clone(),
        dry_run,
        pre_write_manifest: true,
        operations: sorted_operations(session.planned_operations.iter()),
        diagnostics,
        non_claim_boundary: MIGRATION_NON_CLAIM_BOUNDARY,
    }
}

#[cfg(test)]
pub(crate) fn output_has_no_migration_overclaim(text: &str) -> bool {
    text.contains(MIGRATION_NON_CLAIM_BOUNDARY)
        && !text.contains("proves semantic equivalence")
        && !text.contains("proves behavioral correctness")
        && !text.contains("guarantees source compatibility")
        && !text.contains("guarantees release safety")
}

fn validate_required_shape(session: &StructuredMigrationSession, diagnostics: &mut Vec<String>) {
    if session.schema_version != SESSION_SCHEMA_VERSION {
        diagnostics.push(format!(
            "session.schema_version unsupported `{}`; expected `{SESSION_SCHEMA_VERSION}`",
            session.schema_version
        ));
    }
    require_non_empty(&session.session_id, "session.session_id", diagnostics);
    if !SUPPORTED_MIGRATION_KINDS.contains(&session.migration_kind.as_str()) {
        diagnostics.push(format!("session.migration_kind unsupported `{}`", session.migration_kind));
    }
    validate_pointer(&session.old_pointer, "old_pointer", diagnostics);
    validate_pointer(&session.new_pointer, "new_pointer", diagnostics);
    if !SUPPORTED_STATUSES.contains(&session.status.as_str()) {
        diagnostics.push(format!("session.status unsupported `{}`", session.status));
    }
}

fn validate_identity_evidence(session: &StructuredMigrationSession, diagnostics: &mut Vec<String>) {
    if session.strict_identity && session.identity_evidence.is_empty() {
        diagnostics.push("identity_evidence missing for strict migration".to_owned());
    }
    for evidence in &session.identity_evidence {
        require_non_empty(&evidence.identity_kind, "identity_evidence.identity_kind", diagnostics);
        require_non_empty(&evidence.value, "identity_evidence.value", diagnostics);
        if let (Some(expected), Some(actual)) = (&evidence.expected_blake3, &evidence.actual_blake3)
            && expected != actual
        {
            diagnostics.push(format!(
                "identity_evidence.stale_hash `{}` expected {expected} got {actual}",
                evidence.identity_kind
            ));
        }
    }
}

fn validate_compatibility(session: &StructuredMigrationSession, diagnostics: &mut Vec<String>) {
    if !session
        .compatibility
        .iter()
        .any(|pointer| pointer.classification == "canonical" && pointer.pointer.name == session.new_pointer.name)
    {
        diagnostics.push("compatibility.canonical missing for new pointer".to_owned());
    }
    for pointer in &session.compatibility {
        validate_pointer(&pointer.pointer, "compatibility.pointer", diagnostics);
        if !SUPPORTED_COMPATIBILITY_CLASSES.contains(&pointer.classification.as_str()) {
            diagnostics.push(format!(
                "compatibility.classification unsupported `{}` for `{}`",
                pointer.classification, pointer.pointer.name
            ));
            continue;
        }
        if pointer.classification != "canonical"
            && blank_option(pointer.removal_boundary.as_deref())
            && blank_option(pointer.blocking_reason.as_deref())
        {
            diagnostics.push(format!("compatibility.removal_boundary missing for `{}`", pointer.pointer.name));
        }
        if pointer.classification == "compatibility-alias" && pointer.coverage.is_empty() {
            diagnostics.push(format!("compatibility.coverage missing for `{}`", pointer.pointer.name));
        }
    }
}

fn validate_surfaces(session: &StructuredMigrationSession, diagnostics: &mut Vec<String>) {
    if session.affected_surfaces.is_empty() {
        diagnostics.push("affected_surfaces missing".to_owned());
    }
    for surface in &session.affected_surfaces {
        require_non_empty(&surface.surface_kind, "affected_surface.surface_kind", diagnostics);
        require_non_empty(&surface.locator, "affected_surface.locator", diagnostics);
        require_non_empty(&surface.mutability, "affected_surface.mutability", diagnostics);
    }
}

fn validate_operations(session: &StructuredMigrationSession, diagnostics: &mut Vec<String>) {
    for operation in &session.planned_operations {
        require_non_empty(&operation.operation_id, "planned_operation.operation_id", diagnostics);
        require_non_empty(&operation.operation_kind, "planned_operation.operation_kind", diagnostics);
        require_non_empty(&operation.target_locator, "planned_operation.target_locator", diagnostics);
        require_non_empty(&operation.safety, "planned_operation.safety", diagnostics);
        if operation.operation_kind == OPERATION_REWRITE || operation.operation_kind == OPERATION_REPLACE {
            validate_archive_operation(operation, &session.affected_surfaces, diagnostics);
        }
    }
}

fn validate_archive_operation(
    operation: &MigrationPlannedOperation,
    surfaces: &[MigrationAffectedSurface],
    diagnostics: &mut Vec<String>,
) {
    let Some(surface) = surfaces.iter().find(|surface| surface.locator == operation.target_locator) else {
        return;
    };
    if surface.mutability == MUTABILITY_ARCHIVE_READ_ONLY {
        diagnostics.push(format!(
            "archive_boundary.read_only_rewrite `{}` targets `{}`",
            operation.operation_id, operation.target_locator
        ));
    }
    if surface.mutability == MUTABILITY_MUTABLE_ARCHIVE && blank_option(operation.archive_mutation_rationale.as_deref())
    {
        diagnostics
            .push(format!("archive_boundary.mutable_archive_rationale missing for `{}`", operation.operation_id));
    }
}

fn validate_non_claims(session: &StructuredMigrationSession, diagnostics: &mut Vec<String>) {
    let joined = session.non_claims.join("\n").to_ascii_lowercase();
    if !joined.contains("semantic equivalence") || !joined.contains("correctness") {
        diagnostics.push("non_claims missing correctness and semantic equivalence boundary".to_owned());
    }
}

fn validate_pointer(pointer: &MigrationPointer, field: &str, diagnostics: &mut Vec<String>) {
    require_non_empty(&pointer.pointer_kind, &format!("{field}.pointer_kind"), diagnostics);
    require_non_empty(&pointer.name, &format!("{field}.name"), diagnostics);
}

fn require_non_empty(value: &str, field: &str, diagnostics: &mut Vec<String>) {
    if value.trim().is_empty() {
        diagnostics.push(format!("{field} missing"));
    }
}

fn blank_option(value: Option<&str>) -> bool {
    value.is_none_or(|item| item.trim().is_empty())
}

fn sorted_surfaces<'a>(surfaces: impl Iterator<Item = &'a MigrationAffectedSurface>) -> Vec<MigrationAffectedSurface> {
    let mut sorted = surfaces.cloned().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.locator.cmp(&right.locator).then(left.surface_kind.cmp(&right.surface_kind)));
    sorted
}

fn sorted_compatibility<'a>(
    pointers: impl Iterator<Item = &'a MigrationCompatibilityPointer>,
) -> Vec<MigrationCompatibilityPointer> {
    let mut sorted = pointers.cloned().collect::<Vec<_>>();
    sorted.sort_by(|left, right| {
        left.pointer.name.cmp(&right.pointer.name).then(left.classification.cmp(&right.classification))
    });
    sorted
}

fn sorted_operations<'a>(
    operations: impl Iterator<Item = &'a MigrationPlannedOperation>,
) -> Vec<MigrationPlannedOperation> {
    let mut sorted = operations.cloned().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.operation_id.cmp(&right.operation_id));
    sorted
}

fn sorted_commands<'a>(
    commands: impl Iterator<Item = &'a MigrationVerificationCommand>,
) -> Vec<MigrationVerificationCommand> {
    let mut sorted = commands.cloned().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.command.cmp(&right.command));
    sorted
}

fn sorted_todos<'a>(todos: impl Iterator<Item = &'a MigrationTodo>) -> Vec<MigrationTodo> {
    let mut sorted = todos.cloned().collect::<Vec<_>>();
    sorted.sort_by(|left, right| left.item.cmp(&right.item));
    sorted
}

#[cfg(test)]
mod tests {
    use super::*;

    const VALID_SESSION_ID: &str = "rename-cargo-slotcar";
    const CANONICAL_NAME: &str = "cargo-octet";
    const ALIAS_NAME: &str = "cargo-slotcar";
    const LIVE_DOC: &str = "docs/release-candidate.md";
    const ARCHIVE_DOC: &str = "cairn/archive/2026-01-01-legacy/tasks.md";
    const OPERATION_ID: &str = "record-command-alias";

    #[test]
    fn valid_receipt_records_review_identity_and_non_claims() {
        let session = valid_session();
        let diagnostics = validate_session(&session);
        let plan = plan_report(&session);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        assert_eq!(plan.session_id, VALID_SESSION_ID);
        assert_eq!(plan.compatibility_aliases.len(), 1);
        assert_eq!(plan.archive_surfaces[0].locator, ARCHIVE_DOC);
        assert!(plan.non_claim_boundary.contains("semantic equivalence"));
    }

    #[test]
    fn invalid_receipt_rejects_unknowns_missing_identity_and_bad_hash() {
        let mut session = valid_session();
        session.migration_kind = "runtime-magic".to_owned();
        session.strict_identity = true;
        session.identity_evidence.clear();
        session.compatibility[0].classification = "forever-alias".to_owned();
        session.compatibility[1].coverage.clear();
        session.compatibility[1].removal_boundary = None;
        session.compatibility[1].blocking_reason = None;

        let diagnostics = validate_session(&session);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("migration_kind unsupported")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("identity_evidence missing")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("classification unsupported")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("removal_boundary missing")));
    }

    #[test]
    fn alias_coverage_fails_when_live_alias_lacks_coverage() {
        let mut session = valid_session();
        session.compatibility[1].coverage.clear();

        let report = verify_report(&session);

        assert!(!report.valid);
        assert!(report.diagnostics.iter().any(|diagnostic| diagnostic.contains("compatibility.coverage missing")));
    }

    #[test]
    fn archive_boundary_rejects_read_only_rewrite() {
        let mut session = valid_session();
        session.planned_operations.push(MigrationPlannedOperation {
            operation_id: "rewrite-archive".to_owned(),
            operation_kind: OPERATION_REWRITE.to_owned(),
            target_locator: ARCHIVE_DOC.to_owned(),
            safety: SAFETY_SAFE.to_owned(),
            dry_run_preview: "would rewrite archive".to_owned(),
            archive_mutation_rationale: None,
        });

        let report = verify_report(&session);

        assert!(!report.valid);
        assert!(
            report
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("archive_boundary.read_only_rewrite"))
        );
    }

    #[test]
    fn apply_manifest_matches_dry_run_operations_and_is_idempotent() {
        let session = valid_session();
        let dry_run = operation_manifest(&session, true);
        let apply = operation_manifest(&session, false);
        let repeated_apply = operation_manifest(&session, false);

        assert!(dry_run.diagnostics.is_empty());
        assert_eq!(dry_run.operations, apply.operations);
        assert_eq!(apply, repeated_apply);
        assert!(apply.pre_write_manifest);
    }

    #[test]
    fn boundary_helper_rejects_semantic_equivalence_overclaim() {
        let session = valid_session();
        let report = verify_report(&session);
        let rendered = serde_json::to_string(&report).expect("serialize report");
        let overclaim = rendered
            .replace("do not prove behavioral correctness, semantic equivalence", "proves semantic equivalence");

        assert!(output_has_no_migration_overclaim(&rendered));
        assert!(!output_has_no_migration_overclaim(&overclaim));
    }

    fn valid_session() -> StructuredMigrationSession {
        StructuredMigrationSession {
            schema_version: SESSION_SCHEMA_VERSION.to_owned(),
            session_id: VALID_SESSION_ID.to_owned(),
            migration_kind: "command".to_owned(),
            old_pointer: MigrationPointer {
                pointer_kind: "command".to_owned(),
                name: ALIAS_NAME.to_owned(),
                locator: Some("cargo-octet/src/bin/cargo-slotcar.rs".to_owned()),
            },
            new_pointer: MigrationPointer {
                pointer_kind: "command".to_owned(),
                name: CANONICAL_NAME.to_owned(),
                locator: Some("cargo-octet/src/main.rs".to_owned()),
            },
            strict_identity: true,
            identity_evidence: vec![MigrationIdentityEvidence {
                identity_kind: "binary-wrapper".to_owned(),
                value: "cargo-octet-wrapper".to_owned(),
                expected_blake3: Some("same-hash".to_owned()),
                actual_blake3: Some("same-hash".to_owned()),
            }],
            compatibility: vec![
                MigrationCompatibilityPointer {
                    pointer: MigrationPointer {
                        pointer_kind: "command".to_owned(),
                        name: CANONICAL_NAME.to_owned(),
                        locator: Some("cargo-octet/src/main.rs".to_owned()),
                    },
                    classification: "canonical".to_owned(),
                    coverage: vec!["release smoke".to_owned()],
                    removal_boundary: None,
                    blocking_reason: None,
                },
                MigrationCompatibilityPointer {
                    pointer: MigrationPointer {
                        pointer_kind: "command".to_owned(),
                        name: ALIAS_NAME.to_owned(),
                        locator: Some("cargo-octet/src/bin/cargo-slotcar.rs".to_owned()),
                    },
                    classification: "compatibility-alias".to_owned(),
                    coverage: vec!["consumer integration alias test".to_owned()],
                    removal_boundary: Some("keep until downstream release notes remove cargo-slotcar".to_owned()),
                    blocking_reason: None,
                },
            ],
            affected_surfaces: vec![
                MigrationAffectedSurface {
                    surface_kind: "doc".to_owned(),
                    locator: LIVE_DOC.to_owned(),
                    mutability: "live".to_owned(),
                    identity: Some("doc-hash".to_owned()),
                },
                MigrationAffectedSurface {
                    surface_kind: "archive".to_owned(),
                    locator: ARCHIVE_DOC.to_owned(),
                    mutability: MUTABILITY_ARCHIVE_READ_ONLY.to_owned(),
                    identity: Some("archive-hash".to_owned()),
                },
                MigrationAffectedSurface {
                    surface_kind: "generated-artifact".to_owned(),
                    locator: "artifacts/migration/session.json".to_owned(),
                    mutability: MUTABILITY_GENERATED.to_owned(),
                    identity: None,
                },
            ],
            planned_operations: vec![MigrationPlannedOperation {
                operation_id: OPERATION_ID.to_owned(),
                operation_kind: OPERATION_RECORD_ONLY.to_owned(),
                target_locator: "artifacts/migration/session.json".to_owned(),
                safety: SAFETY_SAFE.to_owned(),
                dry_run_preview: "record alias coverage only".to_owned(),
                archive_mutation_rationale: None,
            }],
            verification_commands: vec![MigrationVerificationCommand {
                command: "cargo test -p cargo-octet migrate_session".to_owned(),
                expected: "session verification passes".to_owned(),
            }],
            status: "planned".to_owned(),
            todos: vec![MigrationTodo {
                item: "review downstream release note".to_owned(),
                status: STATUS_OPEN.to_owned(),
                blocker: Some("downstream schedule".to_owned()),
            }],
            non_claims: vec![
                "migration pointer linkage is not semantic equivalence".to_owned(),
                "migration pointer linkage is not behavioral correctness".to_owned(),
            ],
        }
    }
}
