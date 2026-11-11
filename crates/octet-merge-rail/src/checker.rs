//! Canonical admission, four-version item resolution, and bounded relation checking.

use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::evaluate::Backend;
use crate::evaluate::EvalContext;
use crate::evaluate::EvalOutcome;
use crate::evaluate::IrBackend;
use crate::evaluate::ResolvedProgram;
use crate::identity::canonical_identity;
use crate::identity::framed_blake3;
use crate::model::Budget;
use crate::model::Counterexample;
use crate::model::CounterexampleClass;
use crate::model::Expr;
use crate::model::InputDomain;
use crate::model::ItemMapping;
use crate::model::MergeRequest;
use crate::model::Observable;
use crate::model::Outcome;
use crate::model::PijulProvenance;
use crate::model::Predicate;
use crate::model::Role;
use crate::model::RoleBinding;
use crate::model::TimeoutPhase;
use crate::model::UnknownReason;
use crate::model::UnsupportedReason;
use crate::model::Value;
use crate::model::Variant;
use crate::receipt::DependencyIdentity;
use crate::receipt::ObservableIdentity;
use crate::receipt::RoleIdentity;
use crate::receipt::WorkReport;

pub const SOURCE_DOMAIN: &str = "octet-merge-rail-source/v1";
pub const SUBJECT_DOMAIN: &str = "octet-merge-rail-subject/v1";
pub const FUNCTION_DOMAIN: &str = "octet-merge-rail-function/v1";
pub const DOMAIN_DOMAIN: &str = "octet-merge-rail-domain/v1";
pub const ADMISSION_DOMAIN: &str = "octet-merge-rail-admission/v1";
const MAX_COUNTEREXAMPLES: usize = 32;

/// Structural admission failure. No semantic verdict is issued.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmissionError {
    /// Stable error code.
    pub code: &'static str,
    /// Human-readable detail.
    pub detail: String,
}

impl AdmissionError {
    fn new(code: &'static str, detail: impl Into<String>) -> Self {
        Self {
            code,
            detail: detail.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct UnsupportedDetail {
    reason: UnsupportedReason,
    detail: String,
}

#[derive(Clone, Debug)]
struct Admitted {
    assumptions: crate::model::Assumptions,
    variants: BTreeMap<Role, Variant>,
    bindings: BTreeMap<Role, RoleBinding>,
    items: Vec<ItemMapping>,
    observables: Vec<Observable>,
    budget: Budget,
    provenance: Option<PijulProvenance>,
}

#[derive(serde::Serialize)]
struct AdmissionPayload<'a> {
    profile_id: &'a str,
    profile_version: &'a str,
    claim_scope: &'a str,
    assumptions: &'a crate::model::Assumptions,
    variants: Vec<&'a Variant>,
    bindings: Vec<&'a RoleBinding>,
    items: &'a [ItemMapping],
    observables: &'a [Observable],
    budget: &'a Budget,
    provenance: &'a Option<PijulProvenance>,
}

/// Compute the deterministic admission identity of a request.
///
/// # Errors
///
/// Returns [`AdmissionError`] when the request is structurally malformed.
pub fn admission_identity(request: &MergeRequest) -> Result<String, AdmissionError> {
    let admitted = admit(request)?;
    Ok(admitted.identity())
}

/// Check a request with the built-in deterministic scalar backend.
///
/// # Errors
///
/// Returns [`AdmissionError`] when the request is structurally malformed.
pub fn check_request(request: &MergeRequest) -> Result<crate::receipt::MergeRailReceipt, AdmissionError> {
    check_request_with_backend_and_command(request, &IrBackend, DEFAULT_COMMAND)
}

/// Check a request with an injected backend and default command string.
///
/// # Errors
///
/// Returns [`AdmissionError`] when the request is structurally malformed.
pub fn check_request_with_backend(
    request: &MergeRequest,
    backend: &dyn Backend,
) -> Result<crate::receipt::MergeRailReceipt, AdmissionError> {
    check_request_with_backend_and_command(request, backend, DEFAULT_COMMAND)
}

/// Check a request with an injected backend and an exact command string.
///
/// # Errors
///
/// Returns [`AdmissionError`] when the request is structurally malformed.
pub fn check_request_with_backend_and_command(
    request: &MergeRequest,
    backend: &dyn Backend,
    command: &str,
) -> Result<crate::receipt::MergeRailReceipt, AdmissionError> {
    if command.trim().is_empty() {
        return Err(AdmissionError::new("invalid-command", "command must be non-empty"));
    }
    let admitted = admit(request)?;
    let evidence = run_check(&admitted, backend);
    let mut receipt = crate::receipt::MergeRailReceipt::new(
        &admitted.assumptions,
        &admitted.items,
        &admitted.bindings,
        &admitted.budget,
        admitted.provenance.as_ref(),
        &backend.identity(),
        command,
        admitted.identity(),
        evidence,
    );
    receipt.receipt_identity = receipt.computed_identity();
    Ok(receipt)
}

/// Default command label for library callers.
pub const DEFAULT_COMMAND: &str = "octet-merge-rail check --profile bounded";

#[allow(
    clippy::too_many_lines,
    reason = "admission keeps the closed role, item, observable, budget, and provenance checks together so a new check cannot be skipped"
)]
fn admit(request: &MergeRequest) -> Result<Admitted, AdmissionError> {
    let mut variants = BTreeMap::new();
    for variant in &request.variants {
        if variants.insert(variant.role, variant.clone()).is_some() {
            return Err(AdmissionError::new(
                UnsupportedReason::DuplicateRole.as_str(),
                format!("role `{}` appears more than once", variant.role.as_str()),
            ));
        }
    }
    for role in Role::ALL {
        if !variants.contains_key(&role) {
            return Err(AdmissionError::new(
                UnsupportedReason::MissingRole.as_str(),
                format!("role `{}` is missing", role.as_str()),
            ));
        }
    }
    let mut bindings = BTreeMap::new();
    for binding in &request.bindings {
        if bindings.insert(binding.role, binding.clone()).is_some() {
            return Err(AdmissionError::new(
                UnsupportedReason::DuplicateBinding.as_str(),
                format!("binding for role `{}` appears more than once", binding.role.as_str()),
            ));
        }
    }
    for role in Role::ALL {
        if !bindings.contains_key(&role) {
            return Err(AdmissionError::new(
                UnsupportedReason::MissingBinding.as_str(),
                format!("binding for role `{}` is missing", role.as_str()),
            ));
        }
    }
    if request.budget.max_evaluations == 0 || request.budget.max_steps_per_evaluation == 0 {
        return Err(AdmissionError::new(UnsupportedReason::InvalidRequest.as_str(), "budgets must be positive"));
    }
    if request.observables.is_empty() {
        return Err(AdmissionError::new(
            UnsupportedReason::EmptyObservableSet.as_str(),
            "at least one observable is required",
        ));
    }
    let mut items = Vec::with_capacity(request.items.len());
    let mut item_names = BTreeSet::new();
    for item in &request.items {
        if item.logical.trim().is_empty() {
            return Err(AdmissionError::new(
                UnsupportedReason::InvalidRequest.as_str(),
                "logical item names must be non-empty",
            ));
        }
        if item.names.base.is_empty()
            || item.names.a.is_empty()
            || item.names.b.is_empty()
            || item.names.merge.is_empty()
        {
            return Err(AdmissionError::new(
                UnsupportedReason::InvalidRequest.as_str(),
                format!("item `{}` has an empty per-role name", item.logical),
            ));
        }
        if !item_names.insert(item.logical.clone()) {
            return Err(AdmissionError::new(
                UnsupportedReason::InvalidRequest.as_str(),
                format!("logical item `{}` appears more than once", item.logical),
            ));
        }
        items.push(item.clone());
    }
    items.sort_by(|left, right| left.logical.cmp(&right.logical));

    let mut observables = Vec::with_capacity(request.observables.len());
    let mut observable_names = BTreeSet::new();
    for observable in &request.observables {
        if observable.logical.trim().is_empty() {
            return Err(AdmissionError::new(
                UnsupportedReason::InvalidRequest.as_str(),
                "observable names must be non-empty",
            ));
        }
        if !observable_names.insert(observable.logical.clone()) {
            return Err(AdmissionError::new(
                UnsupportedReason::InvalidRequest.as_str(),
                format!("observable `{}` appears more than once", observable.logical),
            ));
        }
        if observable.domain.max_cases == 0 {
            return Err(AdmissionError::new(
                UnsupportedReason::InvalidRequest.as_str(),
                format!("observable `{}` has a zero case bound", observable.logical),
            ));
        }
        let mut domain = observable.domain.clone();
        domain.values.sort_unstable();
        domain.values.dedup();
        domain.precondition.sort_by(|left, right| {
            left.arg
                .cmp(&right.arg)
                .then_with(|| left.op.cmp(&right.op))
                .then_with(|| left.value.cmp(&right.value))
        });
        for predicate in &domain.precondition {
            if predicate.arg >= observable.arity {
                return Err(AdmissionError::new(
                    UnsupportedReason::InvalidRequest.as_str(),
                    format!(
                        "observable `{}` has a precondition for argument {} outside arity {}",
                        observable.logical, predicate.arg, observable.arity
                    ),
                ));
            }
        }
        let mut canonical = observable.clone();
        canonical.domain = domain;
        observables.push(canonical);
    }
    observables.sort_by(|left, right| left.logical.cmp(&right.logical));

    if let Some(provenance) = &request.provenance
        && (provenance.note.trim().is_empty()
            || provenance.change_identities.iter().any(|identity| identity.trim().is_empty()))
    {
        return Err(AdmissionError::new(
            UnsupportedReason::InvalidRequest.as_str(),
            "Pijul provenance must be recorded completely or omitted",
        ));
    }

    Ok(Admitted {
        assumptions: request.assumptions.clone(),
        variants,
        bindings,
        items,
        observables,
        budget: request.budget.clone(),
        provenance: request.provenance.clone(),
    })
}

impl Admitted {
    fn identity(&self) -> String {
        let payload = AdmissionPayload {
            profile_id: crate::PROFILE_ID,
            profile_version: crate::PROFILE_VERSION,
            claim_scope: crate::CLAIM_SCOPE,
            assumptions: &self.assumptions,
            variants: Role::ALL.iter().filter_map(|role| self.variants.get(role)).collect(),
            bindings: Role::ALL.iter().filter_map(|role| self.bindings.get(role)).collect(),
            items: &self.items,
            observables: &self.observables,
            budget: &self.budget,
            provenance: &self.provenance,
        };
        canonical_identity(ADMISSION_DOMAIN, &payload).expect("admission identity serialization must be infallible")
    }
}

pub(crate) struct CheckEvidence {
    pub(crate) roles: Vec<RoleIdentity>,
    pub(crate) observables: Vec<ObservableIdentity>,
    pub(crate) dependencies: Vec<DependencyIdentity>,
    pub(crate) outcome: Outcome,
    pub(crate) work: WorkReport,
}

#[allow(
    clippy::too_many_lines,
    reason = "the finite-domain evaluator and receipt projection share one admitted request and must remain visibly in order"
)]
fn run_check(admitted: &Admitted, backend: &dyn Backend) -> CheckEvidence {
    let roles = computed_role_identities(admitted);
    let mut observable_identities = admitted
        .observables
        .iter()
        .map(|observable| ObservableIdentity {
            logical: observable.logical.clone(),
            arity: observable.arity,
            admitted_inputs: 0,
            domain_identity: domain_identity(observable),
        })
        .collect::<Vec<_>>();
    let dependencies = Vec::new();
    let work = WorkReport::default();
    let stop = |observable_identities: Vec<ObservableIdentity>,
                dependencies: Vec<DependencyIdentity>,
                outcome: Outcome,
                work: WorkReport| CheckEvidence {
        roles: roles.clone(),
        observables: observable_identities,
        dependencies,
        outcome,
        work,
    };

    let mut admitted_inputs = BTreeMap::new();
    for (index, observable) in admitted.observables.iter().enumerate() {
        match enumerate_inputs(observable) {
            Ok(inputs) => {
                observable_identities[index].admitted_inputs = inputs.len();
                admitted_inputs.insert(observable.logical.clone(), inputs);
            }
            Err(InputError::ExceedsBound) => {
                return stop(
                    observable_identities,
                    dependencies,
                    Outcome::Unsupported {
                        reason: UnsupportedReason::DomainExceedsBound,
                        detail: format!(
                            "observable `{}` exceeded its {} case bound",
                            observable.logical, observable.domain.max_cases
                        ),
                    },
                    work,
                );
            }
            Err(InputError::NoValues) => {
                return stop(
                    observable_identities,
                    dependencies,
                    Outcome::Unknown {
                        reason: UnknownReason::NoAdmissibleInputs,
                        detail: format!("observable `{}` has no candidate values", observable.logical),
                    },
                    work,
                );
            }
            Err(InputError::Vacuous) => {
                return stop(
                    observable_identities,
                    dependencies,
                    Outcome::Unknown {
                        reason: UnknownReason::VacuousPrecondition,
                        detail: format!("observable `{}` precondition admits no input", observable.logical),
                    },
                    work,
                );
            }
        }
    }

    let mut resolved = BTreeMap::new();
    for role in Role::ALL {
        let variant = &admitted.variants[&role];
        match resolve_variant(variant, role, &admitted.items) {
            Ok(program) => {
                resolved.insert(role, program);
            }
            Err(detail) => {
                let reason = if admitted.observables.iter().any(|observable| observable.logical == detail.detail) {
                    UnsupportedReason::MissingObservable
                } else {
                    detail.reason
                };
                return stop(
                    observable_identities,
                    dependencies,
                    Outcome::Unsupported {
                        reason,
                        detail: format!("role `{}` could not resolve `{}`", role.as_str(), detail.detail),
                    },
                    work,
                );
            }
        }
    }

    if let Err(detail) = check_bindings(admitted, &roles) {
        return stop(
            observable_identities,
            dependencies,
            Outcome::Unsupported {
                reason: detail.reason,
                detail: detail.detail,
            },
            work,
        );
    }

    for observable in &admitted.observables {
        for role in Role::ALL {
            let Some(function) = resolved[&role].get(&observable.logical) else {
                return stop(
                    observable_identities,
                    dependencies,
                    Outcome::Unsupported {
                        reason: UnsupportedReason::MissingObservable,
                        detail: format!("observable `{}` is missing from role `{}`", observable.logical, role.as_str()),
                    },
                    work,
                );
            };
            if function.params != observable.arity {
                return stop(
                    observable_identities,
                    dependencies,
                    Outcome::Unsupported {
                        reason: UnsupportedReason::SignatureMismatch,
                        detail: format!(
                            "observable `{}` has arity {} in role `{}` but {} was declared",
                            observable.logical,
                            function.params,
                            role.as_str(),
                            observable.arity
                        ),
                    },
                    work,
                );
            }
        }
    }

    let mut closures = BTreeMap::new();
    for role in Role::ALL {
        let roots = admitted.observables.iter().map(|observable| observable.logical.clone()).collect::<Vec<_>>();
        match closure(&resolved[&role], &roots) {
            Ok(closure) => {
                closures.insert(role, closure);
            }
            Err(detail) => {
                return stop(
                    observable_identities,
                    dependencies,
                    Outcome::Unsupported {
                        reason: detail.reason,
                        detail: format!("role `{}`: {}", role.as_str(), detail.detail),
                    },
                    work,
                );
            }
        }
    }

    if let Err(detail) = validate_signatures(&resolved, &closures) {
        return stop(
            observable_identities,
            dependencies,
            Outcome::Unsupported {
                reason: detail.reason,
                detail: detail.detail,
            },
            work,
        );
    }

    let mut dependency_set: Option<BTreeSet<String>> = None;
    for role in Role::ALL {
        let mut reachable = closures[&role].clone();
        for observable in &admitted.observables {
            reachable.remove(&observable.logical);
        }
        if let Some(expected) = &dependency_set {
            if expected != &reachable {
                return stop(
                    observable_identities,
                    dependencies,
                    Outcome::Unsupported {
                        reason: UnsupportedReason::ChangedDependency,
                        detail: format!("dependency closure differs in role `{}`", role.as_str()),
                    },
                    work,
                );
            }
        } else {
            dependency_set = Some(reachable);
        }
    }
    let dependency_set = dependency_set.unwrap_or_default();
    let mut dependency_identities = Vec::with_capacity(dependency_set.len());
    for logical in &dependency_set {
        let mut identity: Option<String> = None;
        for role in Role::ALL {
            let function = &resolved[&role][logical];
            let current = canonical_identity(FUNCTION_DOMAIN, function)
                .expect("function identity serialization must be infallible");
            if let Some(expected) = &identity {
                if expected != &current {
                    return stop(
                        observable_identities,
                        dependencies,
                        Outcome::Unsupported {
                            reason: UnsupportedReason::ChangedDependency,
                            detail: format!(
                                "dependency `{logical}` has a different semantic identity in role `{}`",
                                role.as_str()
                            ),
                        },
                        work,
                    );
                }
            } else {
                identity = Some(current);
            }
        }
        dependency_identities.push(DependencyIdentity {
            logical: logical.clone(),
            subject_identity: identity.expect("every dependency has at least one role"),
        });
    }

    let total_inputs = admitted_inputs.values().map(Vec::len).sum::<usize>();
    let mut evaluations = 0usize;
    let mut observations = BTreeMap::new();
    let context = EvalContext {
        arithmetic: admitted.assumptions.arithmetic,
        max_steps: admitted.budget.max_steps_per_evaluation,
    };
    for observable in &admitted.observables {
        let inputs = &admitted_inputs[&observable.logical];
        for (input_index, input) in inputs.iter().enumerate() {
            for role in Role::ALL {
                if evaluations >= admitted.budget.max_evaluations {
                    return stop(
                        observable_identities,
                        dependency_identities,
                        Outcome::Timeout {
                            phase: TimeoutPhase::Evaluation,
                            completed: evaluations,
                            limit: admitted.budget.max_evaluations,
                        },
                        WorkReport {
                            evaluations,
                            relation_checks: 0,
                        },
                    );
                }
                let args = input.iter().copied().map(Value::Int).collect::<Vec<_>>();
                match backend.evaluate(&resolved[&role], &observable.logical, &args, &context) {
                    Ok(EvalOutcome::Value(value)) => {
                        evaluations += 1;
                        observations.insert((observable.logical.clone(), input_index, role), value);
                    }
                    Ok(EvalOutcome::Exhausted) => {
                        return stop(
                            observable_identities,
                            dependency_identities,
                            Outcome::Timeout {
                                phase: TimeoutPhase::Evaluation,
                                completed: evaluations,
                                limit: admitted.budget.max_steps_per_evaluation,
                            },
                            WorkReport {
                                evaluations,
                                relation_checks: 0,
                            },
                        );
                    }
                    Ok(EvalOutcome::Unsupported { reason, detail }) => {
                        return stop(
                            observable_identities,
                            dependency_identities,
                            Outcome::Unsupported {
                                reason,
                                detail: format!(
                                    "role `{}` observable `{}`: {detail}",
                                    role.as_str(),
                                    observable.logical
                                ),
                            },
                            WorkReport {
                                evaluations,
                                relation_checks: 0,
                            },
                        );
                    }
                    Err(error) => {
                        return stop(
                            observable_identities,
                            dependency_identities,
                            Outcome::ToolError {
                                backend: backend.identity().id,
                                message: error.message,
                            },
                            WorkReport {
                                evaluations,
                                relation_checks: 0,
                            },
                        );
                    }
                }
            }
        }
    }

    let mut counterexamples = Vec::new();
    let mut truncated = false;
    let mut relation_checks = 0usize;
    for observable in &admitted.observables {
        let inputs = &admitted_inputs[&observable.logical];
        for (input_index, input) in inputs.iter().enumerate() {
            relation_checks += 1;
            let key = |role: Role| (observable.logical.clone(), input_index, role);
            let base = observations[&key(Role::Base)];
            let a = observations[&key(Role::A)];
            let b = observations[&key(Role::B)];
            let merge = observations[&key(Role::Merge)];
            let class = classify(base, a, b, merge);
            let Some(class) = class else {
                continue;
            };
            if counterexamples.len() == MAX_COUNTEREXAMPLES {
                truncated = true;
                break;
            }
            counterexamples.push(Counterexample {
                observable: observable.logical.clone(),
                input: input.clone(),
                base,
                a,
                b,
                merge,
                class,
            });
        }
        if truncated {
            break;
        }
    }
    let work = WorkReport {
        evaluations,
        relation_checks,
    };
    let outcome = if counterexamples.is_empty() {
        Outcome::Verified {
            observables: admitted.observables.len(),
            inputs: total_inputs,
        }
    } else {
        Outcome::Counterexample {
            counterexamples,
            truncated,
        }
    };
    CheckEvidence {
        roles,
        observables: observable_identities,
        dependencies: dependency_identities,
        outcome,
        work,
    }
}

fn computed_role_identities(admitted: &Admitted) -> Vec<RoleIdentity> {
    Role::ALL
        .iter()
        .map(|role| {
            let variant = &admitted.variants[role];
            RoleIdentity {
                role: *role,
                source_identity: framed_blake3(SOURCE_DOMAIN, &[variant.source_text.as_bytes()]),
                subject_identity: canonical_identity(SUBJECT_DOMAIN, &variant.program)
                    .expect("subject identity serialization must be infallible"),
            }
        })
        .collect()
}

fn domain_identity(observable: &Observable) -> String {
    #[derive(serde::Serialize)]
    struct DomainPayload<'a> {
        arity: u16,
        domain: &'a InputDomain,
    }
    canonical_identity(DOMAIN_DOMAIN, &DomainPayload {
        arity: observable.arity,
        domain: &observable.domain,
    })
    .expect("domain identity serialization must be infallible")
}

fn check_bindings(admitted: &Admitted, roles: &[RoleIdentity]) -> Result<(), UnsupportedDetail> {
    let mut mismatched = Vec::new();
    for identity in roles {
        let binding = &admitted.bindings[&identity.role];
        if binding.source_identity != identity.source_identity || binding.subject_identity != identity.subject_identity
        {
            mismatched.push(identity.role);
        }
    }
    if mismatched.is_empty() {
        return Ok(());
    }
    let source = |role: Role| {
        roles
            .iter()
            .find(|identity| identity.role == role)
            .map_or("", |identity| identity.source_identity.as_str())
    };
    let is_swap = mismatched.iter().all(|role| {
        mismatched.iter().any(|other| {
            other != role
                && source(*role) == admitted.bindings[other].source_identity.as_str()
                && source(*other) == admitted.bindings[role].source_identity.as_str()
        })
    });
    Err(UnsupportedDetail {
        reason: if is_swap {
            UnsupportedReason::RoleMismatch
        } else {
            UnsupportedReason::BindingMismatch
        },
        detail: "declared source/IR binding does not match the supplied variant".to_owned(),
    })
}

fn resolve_variant(variant: &Variant, role: Role, items: &[ItemMapping]) -> Result<ResolvedProgram, UnsupportedDetail> {
    let mut reverse = BTreeMap::new();
    for item in items {
        let local = item.names.for_role(role);
        if let Some(previous) = reverse.insert(local, item.logical.as_str())
            && previous != item.logical
        {
            return Err(UnsupportedDetail {
                reason: UnsupportedReason::AmbiguousItemMapping,
                detail: "one role-local name maps to more than one logical item".to_owned(),
            });
        }
    }
    let mut program = ResolvedProgram::new();
    for item in items {
        let local = item.names.for_role(role);
        let Some(function) = variant.program.get(local) else {
            return Err(UnsupportedDetail {
                reason: UnsupportedReason::MissingItem,
                detail: item.logical.clone(),
            });
        };
        let mut function = function.clone();
        rewrite_calls(&mut function.body, &reverse)?;
        program.insert(item.logical.clone(), function);
    }
    Ok(program)
}

fn rewrite_calls(expression: &mut Expr, reverse: &BTreeMap<&str, &str>) -> Result<(), UnsupportedDetail> {
    match expression {
        Expr::Int(_) | Expr::Arg(_) | Expr::Unsupported { .. } => Ok(()),
        Expr::Add(left, right)
        | Expr::Sub(left, right)
        | Expr::Mul(left, right)
        | Expr::Div(left, right)
        | Expr::Rem(left, right) => {
            rewrite_calls(left, reverse)?;
            rewrite_calls(right, reverse)
        }
        Expr::Neg(value) => rewrite_calls(value, reverse),
        Expr::If(test, then_value, else_value) => {
            rewrite_calls(test, reverse)?;
            rewrite_calls(then_value, reverse)?;
            rewrite_calls(else_value, reverse)
        }
        Expr::Call { item, args } => {
            let Some(logical) = reverse.get(item.as_str()) else {
                return Err(UnsupportedDetail {
                    reason: UnsupportedReason::MissingItemMapping,
                    detail: "a call target has no reviewed item mapping".to_owned(),
                });
            };
            *item = (*logical).to_owned();
            for arg in args {
                rewrite_calls(arg, reverse)?;
            }
            Ok(())
        }
    }
}

fn closure(program: &ResolvedProgram, roots: &[String]) -> Result<BTreeSet<String>, UnsupportedDetail> {
    let mut visited = BTreeSet::new();
    let mut visiting = BTreeSet::new();
    for root in roots {
        visit(root, program, &mut visited, &mut visiting)?;
    }
    Ok(visited)
}

fn visit(
    item: &str,
    program: &ResolvedProgram,
    visited: &mut BTreeSet<String>,
    visiting: &mut BTreeSet<String>,
) -> Result<(), UnsupportedDetail> {
    if visited.contains(item) {
        return Ok(());
    }
    if !visiting.insert(item.to_owned()) {
        return Err(UnsupportedDetail {
            reason: UnsupportedReason::Recursion,
            detail: "the reachable call graph contains a cycle".to_owned(),
        });
    }
    let Some(function) = program.get(item) else {
        return Err(UnsupportedDetail {
            reason: UnsupportedReason::UndefinedDependency,
            detail: "a reachable logical item is undefined".to_owned(),
        });
    };
    let mut calls = Vec::new();
    collect_calls(&function.body, &mut calls);
    for call in calls {
        visit(call, program, visited, visiting)?;
    }
    visiting.remove(item);
    visited.insert(item.to_owned());
    Ok(())
}

fn collect_calls<'a>(expression: &'a Expr, calls: &mut Vec<&'a str>) {
    match expression {
        Expr::Int(_) | Expr::Arg(_) | Expr::Unsupported { .. } => {}
        Expr::Add(left, right)
        | Expr::Sub(left, right)
        | Expr::Mul(left, right)
        | Expr::Div(left, right)
        | Expr::Rem(left, right) => {
            collect_calls(left, calls);
            collect_calls(right, calls);
        }
        Expr::Neg(value) => collect_calls(value, calls),
        Expr::If(test, then_value, else_value) => {
            collect_calls(test, calls);
            collect_calls(then_value, calls);
            collect_calls(else_value, calls);
        }
        Expr::Call { item, args } => {
            calls.push(item);
            for arg in args {
                collect_calls(arg, calls);
            }
        }
    }
}

fn validate_signatures(
    resolved: &BTreeMap<Role, ResolvedProgram>,
    closures: &BTreeMap<Role, BTreeSet<String>>,
) -> Result<(), UnsupportedDetail> {
    for role in Role::ALL {
        let program = &resolved[&role];
        for item in &closures[&role] {
            let function = &program[item];
            let mut calls = Vec::new();
            collect_call_sites(&function.body, &mut calls);
            for (callee, arity) in calls {
                let Some(target) = program.get(callee) else {
                    return Err(UnsupportedDetail {
                        reason: UnsupportedReason::UndefinedDependency,
                        detail: "a reachable logical item is undefined".to_owned(),
                    });
                };
                if usize::from(target.params) != arity {
                    return Err(UnsupportedDetail {
                        reason: UnsupportedReason::SignatureMismatch,
                        detail: "a call arity differs from the reviewed callee signature".to_owned(),
                    });
                }
            }
        }
    }
    Ok(())
}

fn collect_call_sites<'a>(expression: &'a Expr, calls: &mut Vec<(&'a str, usize)>) {
    match expression {
        Expr::Int(_) | Expr::Arg(_) | Expr::Unsupported { .. } => {}
        Expr::Add(left, right)
        | Expr::Sub(left, right)
        | Expr::Mul(left, right)
        | Expr::Div(left, right)
        | Expr::Rem(left, right) => {
            collect_call_sites(left, calls);
            collect_call_sites(right, calls);
        }
        Expr::Neg(value) => collect_call_sites(value, calls),
        Expr::If(test, then_value, else_value) => {
            collect_call_sites(test, calls);
            collect_call_sites(then_value, calls);
            collect_call_sites(else_value, calls);
        }
        Expr::Call { item, args } => {
            calls.push((item, args.len()));
            for arg in args {
                collect_call_sites(arg, calls);
            }
        }
    }
}

enum InputError {
    ExceedsBound,
    NoValues,
    Vacuous,
}

fn enumerate_inputs(observable: &Observable) -> Result<Vec<Vec<i64>>, InputError> {
    if observable.arity == 0 {
        return Ok(vec![Vec::new()]);
    }
    if observable.domain.values.is_empty() {
        return Err(InputError::NoValues);
    }
    let mut tuples = vec![Vec::new()];
    for _ in 0..observable.arity {
        let mut next = Vec::new();
        for tuple in &tuples {
            for value in &observable.domain.values {
                let mut extended = tuple.clone();
                extended.push(*value);
                next.push(extended);
                if next.len() > observable.domain.max_cases {
                    return Err(InputError::ExceedsBound);
                }
            }
        }
        tuples = next;
    }
    let admitted = tuples
        .into_iter()
        .filter(|tuple| observable.domain.precondition.iter().all(|predicate: &Predicate| predicate.admits(tuple)))
        .collect::<Vec<_>>();
    if admitted.is_empty() {
        return Err(InputError::Vacuous);
    }
    Ok(admitted)
}

fn classify(base: Value, a: Value, b: Value, merge: Value) -> Option<CounterexampleClass> {
    if a != base && b != base && a != b {
        return Some(CounterexampleClass::Incompatible);
    }
    if a != base && merge != a {
        return Some(CounterexampleClass::LostAChange);
    }
    if b != base && merge != b {
        return Some(CounterexampleClass::LostBChange);
    }
    if a == base && b == base && merge != base {
        return Some(CounterexampleClass::Interference);
    }
    None
}
