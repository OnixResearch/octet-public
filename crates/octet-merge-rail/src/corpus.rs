//! Reviewed fixture corpus for the bounded four-version merge experiment.
//!
//! The corpus is deterministic input data, not an experimental result. Each
//! case carries an independently authored expected outcome. The companion
//! experiment binary compares actual receipts against that table.

use std::collections::BTreeMap;

use serde::Serialize;

use crate::checker::check_request_with_backend_and_command;
use crate::evaluate::IrBackend;
use crate::identity::canonical_identity;
use crate::identity::framed_blake3;
use crate::model::ArithmeticModel;
use crate::model::Assumptions;
use crate::model::Budget;
use crate::model::CompareOp;
use crate::model::CounterexampleClass;
use crate::model::DependencyPolicy;
use crate::model::Expr;
use crate::model::ExternalEffects;
use crate::model::Function;
use crate::model::InputDomain;
use crate::model::ItemMapping;
use crate::model::MemoryModel;
use crate::model::MergeRequest;
use crate::model::Observable;
use crate::model::ObservableCompleteness;
use crate::model::Outcome;
use crate::model::PreconditionSemantics;
use crate::model::Predicate;
use crate::model::Program;
use crate::model::Role;
use crate::model::RoleBinding;
use crate::model::RoleMap;
use crate::model::UnknownReason;
use crate::model::UnsupportedReason;
use crate::model::Variant;
use crate::receipt::PROFILE_ID;

/// Exact command recorded by the experiment receipts.
pub const EXPERIMENT_COMMAND: &str = "cargo run -p octet-merge-rail --bin octet-merge-rail-experiment";

/// Independently authored expectation for one fixture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Expected {
    /// The relation held over the bounded domains.
    Verified,
    /// The relation failed with this first counterexample.
    Counterexample {
        /// Expected observable.
        observable: &'static str,
        /// Expected failure class.
        class: CounterexampleClass,
    },
    /// The checker could not justify a verdict.
    Unknown {
        /// Expected reason.
        reason: UnknownReason,
    },
    /// The request is outside the admitted subset.
    Unsupported {
        /// Expected reason.
        reason: UnsupportedReason,
    },
    /// The run was interrupted by an explicit budget.
    Timeout,
    /// The injected backend failed.
    ToolError,
}

impl Expected {
    /// Stable expectation label.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Verified => "verified".to_owned(),
            Self::Counterexample { observable, class } => format!("counterexample:{}@{observable}", class.as_str()),
            Self::Unknown { reason } => format!("unknown:{}", reason.as_str()),
            Self::Unsupported { reason } => format!("unsupported:{}", reason.as_str()),
            Self::Timeout => "timeout".to_owned(),
            Self::ToolError => "tool-error".to_owned(),
        }
    }

    /// Return whether an outcome matches this expectation.
    #[must_use]
    pub fn matches(&self, outcome: &Outcome) -> bool {
        match (self, outcome) {
            (Self::Verified, Outcome::Verified { .. })
            | (Self::Timeout, Outcome::Timeout { .. })
            | (Self::ToolError, Outcome::ToolError { .. }) => true,
            (Self::Counterexample { observable, class }, Outcome::Counterexample { counterexamples, .. }) => {
                counterexamples.first().is_some_and(|counterexample| {
                    counterexample.observable == *observable && counterexample.class == *class
                })
            }
            (Self::Unknown { reason }, Outcome::Unknown { reason: actual, .. }) => reason == actual,
            (Self::Unsupported { reason }, Outcome::Unsupported { reason: actual, .. }) => reason == actual,
            _ => false,
        }
    }
}

/// One fixture case.
#[derive(Clone, Debug)]
pub struct CorpusCase {
    /// Stable case name.
    pub name: &'static str,
    /// Independently authored expectation.
    pub expected: Expected,
    /// Complete request.
    pub request: MergeRequest,
}

/// Deterministic outcome of one case.
#[derive(Clone, Debug, Serialize)]
pub struct CaseReport {
    /// Case name.
    pub name: &'static str,
    /// Expected label.
    pub expected: String,
    /// Actual label or admission error.
    pub actual: String,
    /// Whether the actual outcome matched the expectation.
    pub matched: bool,
    /// Stable outcome kind.
    pub outcome_kind: String,
    /// Receipt identity, when admission succeeded.
    pub receipt_identity: String,
    /// Admission identity, when admission succeeded.
    pub input_identity: String,
    /// Deterministic work units.
    pub work_units: usize,
}

/// Integer ratio, serialized without floating point.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct Ratio {
    /// Numerator.
    pub numerator: usize,
    /// Denominator.
    pub denominator: usize,
}

/// Deterministic experiment metrics.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Metrics {
    /// Number of cases.
    pub cases: usize,
    /// Number of cases matching the independent expectation.
    pub matched_expected: usize,
    /// Expected agreement ratio.
    pub precision: Ratio,
    /// Fraction of cases that produced a receipt.
    pub coverage: Ratio,
    /// Outcome counts by kind.
    pub outcome_counts: BTreeMap<String, usize>,
    /// Deterministic work units, not wall-clock time.
    pub total_work_units: usize,
}

/// Deterministic experiment report.
#[derive(Clone, Debug, Serialize)]
pub struct ExperimentReport {
    /// Report schema.
    pub schema_version: &'static str,
    /// Reviewed profile.
    pub profile_id: &'static str,
    /// Exact command.
    pub command: &'static str,
    /// Per-case results.
    pub cases: Vec<CaseReport>,
    /// Aggregate metrics.
    pub metrics: Metrics,
}

/// Run the reviewed corpus through the built-in backend.
#[must_use]
pub fn run_experiment() -> ExperimentReport {
    run_cases(&corpus())
}

/// Run an explicit case list.
#[must_use]
pub fn run_cases(cases: &[CorpusCase]) -> ExperimentReport {
    let mut reports = Vec::with_capacity(cases.len());
    let mut matched_expected = 0usize;
    let mut produced_receipts = 0usize;
    let mut outcome_counts: BTreeMap<String, usize> = BTreeMap::from([
        ("verified".to_owned(), 0),
        ("counterexample".to_owned(), 0),
        ("unknown".to_owned(), 0),
        ("unsupported".to_owned(), 0),
        ("timeout".to_owned(), 0),
        ("tool-error".to_owned(), 0),
    ]);
    let mut total_work_units = 0usize;
    for case in cases {
        match check_request_with_backend_and_command(&case.request, &IrBackend, EXPERIMENT_COMMAND) {
            Ok(receipt) => {
                produced_receipts += 1;
                let matched = case.expected.matches(&receipt.outcome);
                if matched {
                    matched_expected += 1;
                }
                *outcome_counts.entry(receipt.outcome.kind().to_owned()).or_insert(0) += 1;
                total_work_units = total_work_units.saturating_add(receipt.work.work_units());
                reports.push(CaseReport {
                    name: case.name,
                    expected: case.expected.label(),
                    actual: outcome_label(&receipt.outcome),
                    matched,
                    outcome_kind: receipt.outcome.kind().to_owned(),
                    receipt_identity: receipt.receipt_identity,
                    input_identity: receipt.input_identity,
                    work_units: receipt.work.work_units(),
                });
            }
            Err(error) => {
                reports.push(CaseReport {
                    name: case.name,
                    expected: case.expected.label(),
                    actual: format!("admission-error:{}", error.code),
                    matched: false,
                    outcome_kind: "admission-error".to_owned(),
                    receipt_identity: String::new(),
                    input_identity: String::new(),
                    work_units: 0,
                });
            }
        }
    }
    let total = cases.len();
    ExperimentReport {
        schema_version: "octet-merge-rail-experiment/v1",
        profile_id: PROFILE_ID,
        command: EXPERIMENT_COMMAND,
        metrics: Metrics {
            cases: total,
            matched_expected,
            precision: Ratio {
                numerator: matched_expected,
                denominator: total,
            },
            coverage: Ratio {
                numerator: produced_receipts,
                denominator: total,
            },
            outcome_counts,
            total_work_units,
        },
        cases: reports,
    }
}

fn outcome_label(outcome: &Outcome) -> String {
    match outcome {
        Outcome::Verified { .. } => "verified".to_owned(),
        Outcome::Counterexample { counterexamples, .. } => counterexamples.first().map_or_else(
            || "counterexample".to_owned(),
            |counterexample| format!("counterexample:{}@{}", counterexample.class.as_str(), counterexample.observable),
        ),
        Outcome::Unknown { reason, .. } => format!("unknown:{}", reason.as_str()),
        Outcome::Unsupported { reason, .. } => format!("unsupported:{}", reason.as_str()),
        Outcome::Timeout { phase, .. } => format!("timeout:{}", phase.as_str()),
        Outcome::ToolError { .. } => "tool-error".to_owned(),
    }
}

/// Compute and install role bindings for a request's supplied variants.
///
/// # Panics
///
/// Panics only if the fixed role-binding serialization becomes infallible-in-name-only;
/// the serializer is total for these scalar identities.
#[must_use]
pub fn sealed_bindings(variants: &[Variant]) -> Vec<RoleBinding> {
    variants
        .iter()
        .map(|variant| RoleBinding {
            role: variant.role,
            source_identity: framed_blake3(crate::checker::SOURCE_DOMAIN, &[variant.source_text.as_bytes()]),
            subject_identity: canonical_identity(crate::checker::SUBJECT_DOMAIN, &variant.program)
                .expect("subject identity serialization must be infallible"),
        })
        .collect()
}

/// Reviewed assumptions for every corpus case.
#[must_use]
pub fn assumptions() -> Assumptions {
    Assumptions {
        arithmetic: ArithmeticModel::Checked,
        memory: MemoryModel::PureScalar,
        dependencies: DependencyPolicy::IdenticalOrObservable,
        preconditions: PreconditionSemantics::IntegerConjunction,
        external_effects: ExternalEffects::Forbidden,
        observables: ObservableCompleteness::DeclaredClosureOnly,
    }
}

/// Reviewed budget for corpus cases.
#[must_use]
pub fn budget() -> Budget {
    Budget::new(4_096, 4_096)
}

/// Build the six reviewed fixture cases.
#[must_use]
pub fn corpus() -> Vec<CorpusCase> {
    vec![
        identical_case(),
        independent_changes_case(),
        equivalent_edits_case(),
        incompatible_changes_case(),
        lost_changes_case(),
        text_clean_interference_case(),
    ]
}

fn identical_case() -> CorpusCase {
    let body = Expr::plus(Expr::Arg(0), Expr::Int(1));
    let variants = all_roles("base: f(x) = x + 1", &[("f", unary(body.clone()))]);
    let mut request = request(variants, vec![mapping("f")], vec![unary_observable("f", &[-1, 0, 1], 0)]);
    request.bindings = sealed_bindings(&request.variants);
    CorpusCase {
        name: "identical",
        expected: Expected::Verified,
        request,
    }
}

fn independent_changes_case() -> CorpusCase {
    let base_f = unary(Expr::plus(Expr::Arg(0), Expr::Int(1)));
    let base_g = unary(Expr::plus(Expr::Arg(0), Expr::Int(2)));
    let changed_f = unary(Expr::plus(Expr::Arg(0), Expr::Int(2)));
    let changed_g = unary(Expr::plus(Expr::Arg(0), Expr::Int(3)));
    let variants = vec![
        variant(Role::Base, "base: f(x) = x + 1; g(x) = x + 2", &[("f", base_f.clone()), ("g", base_g.clone())]),
        variant(Role::A, "a: f(x) = x + 2; g(x) = x + 2", &[("f", changed_f.clone()), ("g", base_g.clone())]),
        variant(Role::B, "b: f(x) = x + 1; g(x) = x + 3", &[("f", base_f), ("g", changed_g.clone())]),
        variant(Role::Merge, "merge: f(x) = x + 2; g(x) = x + 3", &[("f", changed_f), ("g", changed_g)]),
    ];
    let mut request = request(variants, vec![mapping("f"), mapping("g")], vec![
        unary_observable("f", &[-1, 0, 1], 0),
        unary_observable("g", &[-1, 0, 1], 0),
    ]);
    request.bindings = sealed_bindings(&request.variants);
    CorpusCase {
        name: "independent-changes",
        expected: Expected::Verified,
        request,
    }
}

fn equivalent_edits_case() -> CorpusCase {
    let base = unary(Expr::plus(Expr::Arg(0), Expr::Int(1)));
    let a = unary(Expr::plus(Expr::Int(1), Expr::Arg(0)));
    let b = unary(Expr::minus(Expr::Arg(0), Expr::Int(-1)));
    let merge = unary(Expr::minus(Expr::plus(Expr::Arg(0), Expr::Int(2)), Expr::Int(1)));
    let variants = vec![
        variant(Role::Base, "base: f(x) = x + 1", &[("f", base)]),
        variant(Role::A, "a: f(x) = 1 + x", &[("f", a)]),
        variant(Role::B, "b: f(x) = x - -1", &[("f", b)]),
        variant(Role::Merge, "merge: f(x) = (x + 2) - 1", &[("f", merge)]),
    ];
    let mut request = request(variants, vec![mapping("f")], vec![unary_observable("f", &[-1, 0, 1], 0)]);
    request.bindings = sealed_bindings(&request.variants);
    CorpusCase {
        name: "equivalent-edits",
        expected: Expected::Verified,
        request,
    }
}

fn incompatible_changes_case() -> CorpusCase {
    let variants = vec![
        variant(Role::Base, "base: f(x) = x", &[("f", unary(Expr::Arg(0)))]),
        variant(Role::A, "a: f(x) = x + 1", &[("f", unary(Expr::plus(Expr::Arg(0), Expr::Int(1))))]),
        variant(Role::B, "b: f(x) = x + 2", &[("f", unary(Expr::plus(Expr::Arg(0), Expr::Int(2))))]),
        variant(Role::Merge, "merge: f(x) = x + 1", &[("f", unary(Expr::plus(Expr::Arg(0), Expr::Int(1))))]),
    ];
    let mut request = request(variants, vec![mapping("f")], vec![unary_observable("f", &[-1, 0, 1], 0)]);
    request.bindings = sealed_bindings(&request.variants);
    CorpusCase {
        name: "incompatible-changes",
        expected: Expected::Counterexample {
            observable: "f",
            class: CounterexampleClass::Incompatible,
        },
        request,
    }
}

fn lost_changes_case() -> CorpusCase {
    let base_f = unary(Expr::Arg(0));
    let base_g = unary(Expr::Arg(0));
    let variants = vec![
        variant(Role::Base, "base: f(x) = x; g(x) = x", &[("f", base_f.clone()), ("g", base_g.clone())]),
        variant(Role::A, "a: f(x) = x + 1; g(x) = x", &[
            ("f", unary(Expr::plus(Expr::Arg(0), Expr::Int(1)))),
            ("g", base_g.clone()),
        ]),
        variant(Role::B, "b: f(x) = x; g(x) = x + 2", &[
            ("f", base_f.clone()),
            ("g", unary(Expr::plus(Expr::Arg(0), Expr::Int(2)))),
        ]),
        variant(Role::Merge, "merge: f(x) = x; g(x) = x", &[("f", base_f), ("g", base_g)]),
    ];
    let mut request = request(variants, vec![mapping("f"), mapping("g")], vec![
        unary_observable("f", &[-1, 0, 1], 0),
        unary_observable("g", &[-1, 0, 1], 0),
    ]);
    request.bindings = sealed_bindings(&request.variants);
    CorpusCase {
        name: "lost-changes",
        expected: Expected::Counterexample {
            observable: "f",
            class: CounterexampleClass::LostAChange,
        },
        request,
    }
}

fn text_clean_interference_case() -> CorpusCase {
    let variants = vec![
        variant(Role::Base, "base: f() = 1; g() = 1", &[("f", nullary(Expr::Int(1))), ("g", nullary(Expr::Int(1)))]),
        variant(Role::A, "a: f() = g(); g() = 1", &[
            ("f", nullary(Expr::call("g", Vec::new()))),
            ("g", nullary(Expr::Int(1))),
        ]),
        variant(Role::B, "b: f() = 1; g() = 2", &[("f", nullary(Expr::Int(1))), ("g", nullary(Expr::Int(2)))]),
        variant(Role::Merge, "merge: f() = g(); g() = 2", &[
            ("f", nullary(Expr::call("g", Vec::new()))),
            ("g", nullary(Expr::Int(2))),
        ]),
    ];
    let mut request =
        request(variants, vec![mapping("f"), mapping("g")], vec![nullary_observable("f"), nullary_observable("g")]);
    request.bindings = sealed_bindings(&request.variants);
    CorpusCase {
        name: "text-clean-semantic-interference",
        expected: Expected::Counterexample {
            observable: "f",
            class: CounterexampleClass::Interference,
        },
        request,
    }
}

fn variant(role: Role, source_text: &str, functions: &[(&str, Function)]) -> Variant {
    Variant {
        role,
        source_text: source_text.to_owned(),
        program: functions.iter().map(|(name, function)| ((*name).to_owned(), function.clone())).collect::<Program>(),
    }
}

fn all_roles(source_text: &str, functions: &[(&str, Function)]) -> Vec<Variant> {
    Role::ALL.iter().map(|role| variant(*role, source_text, functions)).collect()
}

fn mapping(logical: &str) -> ItemMapping {
    ItemMapping {
        logical: logical.to_owned(),
        names: RoleMap {
            base: logical.to_owned(),
            a: logical.to_owned(),
            b: logical.to_owned(),
            merge: logical.to_owned(),
        },
    }
}

fn unary(body: Expr) -> Function {
    Function::new(1, body)
}

fn nullary(body: Expr) -> Function {
    Function::new(0, body)
}

fn unary_observable(logical: &str, values: &[i64], minimum: i64) -> Observable {
    Observable {
        logical: logical.to_owned(),
        arity: 1,
        domain: InputDomain::new(values.to_vec(), 8, vec![Predicate::new(0, CompareOp::GreaterOrEqual, minimum)]),
    }
}

fn nullary_observable(logical: &str) -> Observable {
    Observable {
        logical: logical.to_owned(),
        arity: 0,
        domain: InputDomain::new(Vec::new(), 1, Vec::new()),
    }
}

fn request(variants: Vec<Variant>, items: Vec<ItemMapping>, observables: Vec<Observable>) -> MergeRequest {
    MergeRequest {
        assumptions: assumptions(),
        variants,
        bindings: Vec::new(),
        items,
        observables,
        budget: budget(),
        provenance: None,
    }
}
