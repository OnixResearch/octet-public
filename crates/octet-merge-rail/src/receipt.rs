//! Deterministic receipt construction and validation for the bounded merge rail.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::checker::CheckEvidence;
use crate::evaluate::BackendIdentity;
use crate::identity::canonical_identity;
use crate::identity::is_identity;
use crate::model::Assumptions;
use crate::model::Budget;
use crate::model::ItemMapping;
use crate::model::Outcome;
use crate::model::PijulProvenance;
use crate::model::Role;
use crate::model::RoleBinding;

/// Receipt schema identity.
pub const RECEIPT_SCHEMA: &str = "octet-merge-rail-receipt/v1";
/// Receipt identity domain.
pub const RECEIPT_DOMAIN: &str = "octet-merge-rail-receipt/v1";
/// Profile identity.
pub const PROFILE_ID: &str = "octet-merge-rail-bounded-v1";
/// Profile version.
pub const PROFILE_VERSION: &str = "1";
/// Claim scope bound into every receipt.
pub const CLAIM_SCOPE: &str = "bounded-four-version-observable-preservation";
/// Frontend identity for the hand-authored typed IR.
pub const FRONTEND_ID: &str = "typed-scalar-ir";
/// Frontend version.
pub const FRONTEND_VERSION: &str = "1";
/// Model identity.
pub const MODEL_ID: &str = "pure-scalar-observable-model";
/// Model version.
pub const MODEL_VERSION: &str = "1";
/// Verifier identity.
pub const VERIFIER_ID: &str = "bounded-relation-checker";
/// Verifier version.
pub const VERIFIER_VERSION: &str = "1";

/// Required consumer non-claims.
pub const NON_CLAIMS: [&str; 7] = [
    "verified means the four-version relation held over the declared bounded input domains, not a universal program proof",
    "the typed IR was authored by hand; Rust source to IR correspondence is unproven",
    "the rail checks a supplied candidate and never synthesizes, applies, or mutates a merge",
    "a verdict does not prove whole-program correctness, semantic equivalence, or developer intent",
    "a verdict is not release authority, provider trust, or an automatic admission decision",
    "unsafe, FFI, concurrency, opaque macros, dynamic dispatch, memory aliasing, and external effects are excluded",
    "Pijul native change identities are recorded-only provenance and never behavioral evidence",
];

/// Stable frontend, model, verifier, or backend identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ToolIdentity {
    /// Tool identifier.
    pub id: String,
    /// Tool version.
    pub version: String,
}

impl ToolIdentity {
    /// Build a tool identity.
    #[must_use]
    pub fn new(id: &str, version: &str) -> Self {
        Self {
            id: id.to_owned(),
            version: version.to_owned(),
        }
    }
}

/// Computed role-tagged identities.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoleIdentity {
    /// Role tag.
    pub role: Role,
    /// Source-text identity.
    pub source_identity: String,
    /// Role-local semantic-subject identity.
    pub subject_identity: String,
}

/// Observable identity and bounded-domain coverage.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ObservableIdentity {
    /// Logical observable identity.
    pub logical: String,
    /// Reviewed arity.
    pub arity: u16,
    /// Number of admitted input tuples.
    pub admitted_inputs: usize,
    /// Identity of the declared domain and precondition.
    pub domain_identity: String,
}

/// Shared dependency identity.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DependencyIdentity {
    /// Logical dependency identity.
    pub logical: String,
    /// Identity required to be identical across all four variants.
    pub subject_identity: String,
}

/// Deterministic work report. `work_units` is not wall-clock time.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct WorkReport {
    /// Backend evaluations performed.
    pub evaluations: usize,
    /// Relation checks performed.
    pub relation_checks: usize,
}

impl WorkReport {
    /// Deterministic work units.
    #[must_use]
    pub fn work_units(&self) -> usize {
        self.evaluations.saturating_add(self.relation_checks)
    }
}

/// Deterministic four-version receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MergeRailReceipt {
    /// Receipt schema.
    pub schema_version: String,
    /// Profile identity.
    pub profile_id: String,
    /// Profile version.
    pub profile_version: String,
    /// Claim scope.
    pub claim_scope: String,
    /// Frontend identity.
    pub frontend: ToolIdentity,
    /// Model identity.
    pub model: ToolIdentity,
    /// Verifier identity.
    pub verifier: ToolIdentity,
    /// Backend identity.
    pub backend: ToolIdentity,
    /// Solver identity; absent for the finite-domain prototype.
    pub solver: Option<ToolIdentity>,
    /// Explicit assumptions.
    pub assumptions: Assumptions,
    /// Role-tagged identities.
    pub roles: Vec<RoleIdentity>,
    /// Canonical role bindings.
    pub bindings: Vec<RoleBinding>,
    /// Canonical item mappings.
    pub items: Vec<ItemMapping>,
    /// Observable coverage.
    pub observables: Vec<ObservableIdentity>,
    /// Shared dependency identities.
    pub dependencies: Vec<DependencyIdentity>,
    /// Applied work bounds.
    pub budget: Budget,
    /// Admission identity the verdict applies to.
    pub input_identity: String,
    /// Exact command or library invocation label.
    pub command: String,
    /// Observed work.
    pub work: WorkReport,
    /// Deterministic outcome.
    pub outcome: Outcome,
    /// Recorded-only native provenance.
    pub provenance: Option<PijulProvenance>,
    /// Consumer non-claims.
    pub non_claims: Vec<String>,
    /// Receipt identity.
    pub receipt_identity: String,
}

impl MergeRailReceipt {
    /// Build an unsealed receipt.
    #[must_use]
    pub(crate) fn new(
        assumptions: &Assumptions,
        items: &[ItemMapping],
        bindings: &BTreeMap<Role, RoleBinding>,
        budget: &Budget,
        provenance: Option<&PijulProvenance>,
        backend: &BackendIdentity,
        command: &str,
        input_identity: String,
        evidence: CheckEvidence,
    ) -> Self {
        Self {
            schema_version: RECEIPT_SCHEMA.to_owned(),
            profile_id: PROFILE_ID.to_owned(),
            profile_version: PROFILE_VERSION.to_owned(),
            claim_scope: CLAIM_SCOPE.to_owned(),
            frontend: ToolIdentity::new(FRONTEND_ID, FRONTEND_VERSION),
            model: ToolIdentity::new(MODEL_ID, MODEL_VERSION),
            verifier: ToolIdentity::new(VERIFIER_ID, VERIFIER_VERSION),
            backend: ToolIdentity::new(&backend.id, &backend.version),
            solver: None,
            assumptions: assumptions.clone(),
            roles: evidence.roles,
            bindings: Role::ALL.iter().filter_map(|role| bindings.get(role).cloned()).collect(),
            items: items.to_vec(),
            observables: evidence.observables,
            dependencies: evidence.dependencies,
            budget: budget.clone(),
            input_identity,
            command: command.to_owned(),
            work: evidence.work,
            outcome: evidence.outcome,
            provenance: provenance.cloned(),
            non_claims: NON_CLAIMS.iter().map(|claim| (*claim).to_owned()).collect(),
            receipt_identity: String::new(),
        }
    }

    /// Compute the receipt identity over the payload with a cleared identity field.
    ///
    /// # Panics
    ///
    /// Panics only if the fixed receipt payload cannot be serialized; the serializer is total for
    /// the receipt's scalar and string fields.
    #[must_use]
    pub fn computed_identity(&self) -> String {
        let mut copy = self.clone();
        copy.receipt_identity.clear();
        canonical_identity(RECEIPT_DOMAIN, &copy).expect("receipt identity serialization must be infallible")
    }
}

/// One receipt validation issue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceiptIssue {
    /// Stable issue code.
    pub code: &'static str,
    /// Human-readable detail.
    pub detail: String,
}

impl ReceiptIssue {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

/// Validate a receipt against a current admission identity.
///
/// # Errors
///
/// Returns every deterministic issue when the receipt is stale, tampered, or
/// structurally inconsistent.
pub fn validate_receipt(receipt: &MergeRailReceipt, current_input_identity: &str) -> Result<(), Vec<ReceiptIssue>> {
    let mut issues = Vec::new();
    if receipt.schema_version != RECEIPT_SCHEMA {
        issues.push(ReceiptIssue::new("schema_mismatch", "unexpected receipt schema"));
    }
    if receipt.profile_id != PROFILE_ID || receipt.profile_version != PROFILE_VERSION {
        issues.push(ReceiptIssue::new("profile_mismatch", "unexpected profile identity"));
    }
    if receipt.claim_scope != CLAIM_SCOPE {
        issues.push(ReceiptIssue::new("claim_scope_mismatch", "unexpected claim scope"));
    }
    if receipt.receipt_identity != receipt.computed_identity() {
        issues.push(ReceiptIssue::new("receipt_identity_mismatch", "receipt payload does not match its identity"));
    }
    if receipt.input_identity != current_input_identity {
        issues.push(ReceiptIssue::new("stale_input", "receipt admission identity does not match the current request"));
    }
    if !is_identity(&receipt.input_identity) {
        issues.push(ReceiptIssue::new("invalid_identity", "admission identity is malformed"));
    }
    validate_roles(receipt, &mut issues);
    validate_bindings(receipt, &mut issues);
    validate_items(receipt, &mut issues);
    validate_observables(receipt, &mut issues);
    validate_dependencies(receipt, &mut issues);
    if receipt.budget.max_evaluations == 0 || receipt.budget.max_steps_per_evaluation == 0 {
        issues.push(ReceiptIssue::new("invalid_budget", "budgets must be positive"));
    }
    if receipt.command.trim().is_empty() {
        issues.push(ReceiptIssue::new("empty_command", "command must be recorded"));
    }
    if receipt.work.evaluations > receipt.budget.max_evaluations {
        issues.push(ReceiptIssue::new("work_exceeds_budget", "recorded evaluations exceed the admitted budget"));
    }
    let admitted_inputs = receipt.observables.iter().map(|observable| observable.admitted_inputs).sum::<usize>();
    if receipt.work.relation_checks > admitted_inputs {
        issues.push(ReceiptIssue::new("work_mismatch", "recorded relation checks exceed admitted inputs"));
    }
    if matches!(&receipt.outcome, Outcome::Verified { .. } | Outcome::Counterexample { .. })
        && receipt.work.evaluations != admitted_inputs.saturating_mul(4)
    {
        issues.push(ReceiptIssue::new(
            "work_mismatch",
            "a complete verdict must evaluate every role over every admitted input",
        ));
    }
    for non_claim in NON_CLAIMS {
        if !receipt.non_claims.iter().any(|claim| claim == non_claim) {
            issues.push(ReceiptIssue::new("missing_non_claim", format!("required non-claim is absent: {non_claim}")));
        }
    }
    validate_outcome(receipt, &mut issues);
    if issues.is_empty() { Ok(()) } else { Err(issues) }
}

fn validate_roles(receipt: &MergeRailReceipt, issues: &mut Vec<ReceiptIssue>) {
    let mut roles = BTreeSet::new();
    for identity in &receipt.roles {
        if !roles.insert(identity.role) {
            issues.push(ReceiptIssue::new("role_inventory_invalid", "duplicate role identity"));
        }
        if !is_identity(&identity.source_identity) || !is_identity(&identity.subject_identity) {
            issues.push(ReceiptIssue::new(
                "invalid_identity",
                format!("role `{}` has a malformed identity", identity.role.as_str()),
            ));
        }
    }
    for role in Role::ALL {
        if !roles.contains(&role) {
            issues.push(ReceiptIssue::new("role_inventory_invalid", format!("role `{}` is missing", role.as_str())));
        }
    }
}

fn validate_bindings(receipt: &MergeRailReceipt, issues: &mut Vec<ReceiptIssue>) {
    let mut roles = BTreeSet::new();
    for binding in &receipt.bindings {
        if !roles.insert(binding.role) {
            issues.push(ReceiptIssue::new("binding_inventory_invalid", "duplicate role binding"));
        }
        let identity = receipt.roles.iter().find(|identity| identity.role == binding.role);
        match identity {
            Some(identity)
                if binding.source_identity != identity.source_identity
                    || binding.subject_identity != identity.subject_identity =>
            {
                issues.push(ReceiptIssue::new(
                    "binding_mismatch",
                    format!("binding for role `{}` disagrees with its identity", binding.role.as_str()),
                ));
            }
            None => {
                issues.push(ReceiptIssue::new("binding_inventory_invalid", "binding has no matching role identity"));
            }
            _ => {}
        }
    }
    for role in Role::ALL {
        if !roles.contains(&role) {
            issues.push(ReceiptIssue::new(
                "binding_inventory_invalid",
                format!("binding for role `{}` is missing", role.as_str()),
            ));
        }
    }
}

fn validate_items(receipt: &MergeRailReceipt, issues: &mut Vec<ReceiptIssue>) {
    let mut previous: Option<&str> = None;
    for item in &receipt.items {
        if previous.is_some_and(|previous| previous >= item.logical.as_str()) {
            issues
                .push(ReceiptIssue::new("item_order_invalid", "item mappings are not strictly sorted by logical name"));
        }
        previous = Some(&item.logical);
        if item.logical.trim().is_empty()
            || item.names.base.is_empty()
            || item.names.a.is_empty()
            || item.names.b.is_empty()
            || item.names.merge.is_empty()
        {
            issues
                .push(ReceiptIssue::new("item_mapping_invalid", "item mapping has an empty logical or per-role name"));
        }
    }
}

fn validate_observables(receipt: &MergeRailReceipt, issues: &mut Vec<ReceiptIssue>) {
    let mut previous: Option<&str> = None;
    for observable in &receipt.observables {
        if previous.is_some_and(|previous| previous >= observable.logical.as_str()) {
            issues.push(ReceiptIssue::new(
                "observable_order_invalid",
                "observables are not strictly sorted by logical name",
            ));
        }
        previous = Some(&observable.logical);
        if !is_identity(&observable.domain_identity) {
            issues.push(ReceiptIssue::new(
                "invalid_identity",
                format!("observable `{}` has a malformed domain identity", observable.logical),
            ));
        }
    }
}

fn validate_dependencies(receipt: &MergeRailReceipt, issues: &mut Vec<ReceiptIssue>) {
    let mut previous: Option<&str> = None;
    for dependency in &receipt.dependencies {
        if previous.is_some_and(|previous| previous >= dependency.logical.as_str()) {
            issues.push(ReceiptIssue::new(
                "dependency_order_invalid",
                "dependencies are not strictly sorted by logical name",
            ));
        }
        previous = Some(&dependency.logical);
        if !is_identity(&dependency.subject_identity) {
            issues.push(ReceiptIssue::new(
                "invalid_identity",
                format!("dependency `{}` has a malformed identity", dependency.logical),
            ));
        }
    }
}

fn validate_outcome(receipt: &MergeRailReceipt, issues: &mut Vec<ReceiptIssue>) {
    match &receipt.outcome {
        Outcome::Verified { observables, inputs } => {
            if *observables != receipt.observables.len() || *inputs == 0 {
                issues.push(ReceiptIssue::new(
                    "outcome_inconsistent",
                    "verified outcome does not match observable coverage",
                ));
            }
            if receipt.observables.iter().any(|observable| observable.admitted_inputs == 0) {
                issues.push(ReceiptIssue::new(
                    "outcome_inconsistent",
                    "verified outcome contains an observable with no admitted input",
                ));
            }
            if receipt.work.evaluations == 0 || receipt.work.relation_checks == 0 {
                issues.push(ReceiptIssue::new("outcome_inconsistent", "verified outcome records no work"));
            }
        }
        Outcome::Counterexample { counterexamples, .. } => {
            if counterexamples.is_empty() {
                issues.push(ReceiptIssue::new("outcome_inconsistent", "counterexample outcome has no counterexample"));
            }
        }
        Outcome::Timeout { limit, .. } => {
            if *limit == 0 {
                issues.push(ReceiptIssue::new("outcome_inconsistent", "timeout outcome has a zero limit"));
            }
        }
        Outcome::ToolError { backend, message } => {
            if backend.trim().is_empty() || message.trim().is_empty() {
                issues.push(ReceiptIssue::new("outcome_inconsistent", "tool-error outcome is incomplete"));
            }
        }
        Outcome::Unknown { .. } | Outcome::Unsupported { .. } => {}
    }
}
