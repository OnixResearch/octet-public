//! Typed model for a bounded four-version semantic merge relation.
//!
//! The model is deliberately small. It represents pure scalar functions over
//! `i64` arguments, integer arithmetic, conditionals, and direct calls to
//! logical items. It cannot express memory, pointers, unsafe code, FFI,
//! concurrency, macros, trait dispatch, or external effects.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;

/// Role of one program in the four-version relation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// Exact base program `O`.
    Base,
    /// First variant `A`.
    A,
    /// Second variant `B`.
    B,
    /// Proposed merge `M`.
    Merge,
}

impl Role {
    /// Canonical role order used by every deterministic traversal.
    pub const ALL: [Self; 4] = [Self::Base, Self::A, Self::B, Self::Merge];

    /// Return the stable role label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Base => "base",
            Self::A => "a",
            Self::B => "b",
            Self::Merge => "merge",
        }
    }
}

/// Arithmetic model applied uniformly to all four programs.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ArithmeticModel {
    /// Two's-complement wrapping arithmetic; division by zero aborts.
    Wrapping,
    /// Checked arithmetic; overflow and division by zero abort.
    Checked,
}

impl ArithmeticModel {
    /// Return the stable label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Wrapping => "wrapping",
            Self::Checked => "checked",
        }
    }
}

/// Memory and aliasing model admitted by the prototype.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum MemoryModel {
    /// No memory access, no aliasing, no mutation; arguments and results only.
    PureScalar,
}

/// Dependency-closure policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyPolicy {
    /// Every reachable callee must have an identical semantic identity in all
    /// four variants, unless the callee is itself a declared observable.
    IdenticalOrObservable,
}

/// Precondition semantics.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PreconditionSemantics {
    /// A conjunction of per-argument integer comparisons.
    IntegerConjunction,
}

/// External-effect policy.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExternalEffects {
    /// No external effects are representable in the admitted IR.
    Forbidden,
}

/// Observable-completeness assumption.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ObservableCompleteness {
    /// The declared observables are the entire reviewed observation set.
    DeclaredClosureOnly,
}

/// Explicit assumptions bound into the admission identity and receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Assumptions {
    /// Arithmetic and overflow behavior.
    pub arithmetic: ArithmeticModel,
    /// Memory and aliasing behavior.
    pub memory: MemoryModel,
    /// Dependency-closure policy.
    pub dependencies: DependencyPolicy,
    /// Precondition semantics.
    pub preconditions: PreconditionSemantics,
    /// External-effect policy.
    pub external_effects: ExternalEffects,
    /// Observable-completeness assumption.
    pub observables: ObservableCompleteness,
}

/// A unsupported construct that can appear in the typed IR.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnsupportedConstruct {
    /// Raw-pointer or memory operation.
    RawPointer,
    /// Unsafe block.
    Unsafe,
    /// Foreign-function call.
    Ffi,
    /// Thread, task, or concurrency primitive.
    Concurrency,
    /// Opaque macro expansion.
    MacroExpansion,
    /// Dynamic or trait dispatch.
    Dispatch,
    /// Unmodeled ambient effect.
    Effect,
    /// Anything outside the reviewed scalar subset.
    Other,
}

impl UnsupportedConstruct {
    /// Return the stable label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RawPointer => "raw-pointer",
            Self::Unsafe => "unsafe",
            Self::Ffi => "ffi",
            Self::Concurrency => "concurrency",
            Self::MacroExpansion => "macro-expansion",
            Self::Dispatch => "dispatch",
            Self::Effect => "effect",
            Self::Other => "other",
        }
    }
}

/// A typed scalar expression.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Expr {
    /// Integer literal.
    Int(i64),
    /// Positional argument reference.
    Arg(u16),
    /// Addition.
    Add(Box<Self>, Box<Self>),
    /// Subtraction.
    Sub(Box<Self>, Box<Self>),
    /// Multiplication.
    Mul(Box<Self>, Box<Self>),
    /// Division.
    Div(Box<Self>, Box<Self>),
    /// Remainder.
    Rem(Box<Self>, Box<Self>),
    /// Negation.
    Neg(Box<Self>),
    /// Conditional: zero is false, any other value is true.
    If(Box<Self>, Box<Self>, Box<Self>),
    /// Direct call to a logical item.
    Call {
        /// Logical item identity after item mapping.
        item: String,
        /// Argument expressions.
        args: Vec<Self>,
    },
    /// Explicitly unsupported construct.
    Unsupported {
        /// Reviewed construct class.
        construct: UnsupportedConstruct,
    },
}

impl Expr {
    /// Build an addition.
    #[must_use]
    pub fn plus(left: Self, right: Self) -> Self {
        Self::Add(Box::new(left), Box::new(right))
    }

    /// Build a subtraction.
    #[must_use]
    pub fn minus(left: Self, right: Self) -> Self {
        Self::Sub(Box::new(left), Box::new(right))
    }

    /// Build a multiplication.
    #[must_use]
    pub fn times(left: Self, right: Self) -> Self {
        Self::Mul(Box::new(left), Box::new(right))
    }

    /// Build a division.
    #[must_use]
    pub fn divide(left: Self, right: Self) -> Self {
        Self::Div(Box::new(left), Box::new(right))
    }

    /// Build a remainder.
    #[must_use]
    pub fn remainder(left: Self, right: Self) -> Self {
        Self::Rem(Box::new(left), Box::new(right))
    }

    /// Build a negation.
    #[must_use]
    pub fn negate(value: Self) -> Self {
        Self::Neg(Box::new(value))
    }

    /// Build a conditional.
    #[must_use]
    pub fn condition(test: Self, then_value: Self, else_value: Self) -> Self {
        Self::If(Box::new(test), Box::new(then_value), Box::new(else_value))
    }

    /// Build a direct call.
    #[must_use]
    pub fn call(item: impl Into<String>, args: Vec<Self>) -> Self {
        Self::Call {
            item: item.into(),
            args,
        }
    }
}

/// One admitted function.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Function {
    /// Number of positional `i64` parameters.
    pub params: u16,
    /// Function body.
    pub body: Expr,
}

impl Function {
    /// Build a function.
    #[must_use]
    pub fn new(params: u16, body: Expr) -> Self {
        Self { params, body }
    }
}

/// Role-local function table.
pub type Program = BTreeMap<String, Function>;

/// One role-tagged variant with its reviewed source text and typed program.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Variant {
    /// Role tag for this variant.
    pub role: Role,
    /// Reviewed source text used as the source-identity preimage.
    pub source_text: String,
    /// Role-local typed program.
    pub program: Program,
}

/// Declared binding from a role to its source and semantic-subject identities.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoleBinding {
    /// Role being bound.
    pub role: Role,
    /// Expected BLAKE3 identity of the reviewed source text.
    pub source_identity: String,
    /// Expected BLAKE3 identity of the mapped logical program.
    pub subject_identity: String,
}

/// Per-role item names for one logical item.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RoleMap {
    /// Base item name.
    pub base: String,
    /// Variant `A` item name.
    pub a: String,
    /// Variant `B` item name.
    pub b: String,
    /// Merge item name.
    pub merge: String,
}

impl RoleMap {
    /// Resolve the item name for a role.
    #[must_use]
    pub fn for_role(&self, role: Role) -> &str {
        match role {
            Role::Base => &self.base,
            Role::A => &self.a,
            Role::B => &self.b,
            Role::Merge => &self.merge,
        }
    }
}

/// Validated four-version mapping for one logical item.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ItemMapping {
    /// Logical item identity used by observables, calls, and receipts.
    pub logical: String,
    /// Per-role source-level names, which may differ after a rename.
    pub names: RoleMap,
}

/// Integer comparison used by a precondition.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompareOp {
    /// Equality.
    Equal,
    /// Inequality.
    NotEqual,
    /// Strictly less.
    Less,
    /// Less or equal.
    LessOrEqual,
    /// Strictly greater.
    Greater,
    /// Greater or equal.
    GreaterOrEqual,
}

impl CompareOp {
    /// Evaluate the comparison.
    #[must_use]
    pub fn evaluate(self, left: i64, right: i64) -> bool {
        match self {
            Self::Equal => left == right,
            Self::NotEqual => left != right,
            Self::Less => left < right,
            Self::LessOrEqual => left <= right,
            Self::Greater => left > right,
            Self::GreaterOrEqual => left >= right,
        }
    }
}

/// One precondition predicate over a positional argument.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Predicate {
    /// Zero-based argument index.
    pub arg: u16,
    /// Comparison operator.
    pub op: CompareOp,
    /// Compared value.
    pub value: i64,
}

impl Predicate {
    /// Build a predicate.
    #[must_use]
    pub fn new(arg: u16, op: CompareOp, value: i64) -> Self {
        Self { arg, op, value }
    }

    /// Return whether the predicate admits the supplied arguments.
    #[must_use]
    pub fn admits(&self, args: &[i64]) -> bool {
        args.get(usize::from(self.arg)).is_some_and(|value| self.op.evaluate(*value, self.value))
    }
}

/// Explicit bounded input domain for one observable.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct InputDomain {
    /// Candidate scalar values.
    pub values: Vec<i64>,
    /// Maximum enumerated input tuples for this observable.
    pub max_cases: usize,
    /// Conjunction of per-argument predicates.
    pub precondition: Vec<Predicate>,
}

impl InputDomain {
    /// Build a domain over the cartesian product of `values`.
    #[must_use]
    pub fn new(values: Vec<i64>, max_cases: usize, precondition: Vec<Predicate>) -> Self {
        Self {
            values,
            max_cases,
            precondition,
        }
    }
}

/// One observable logical item and its bounded input domain.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Observable {
    /// Logical item identity.
    pub logical: String,
    /// Reviewed argument count.
    pub arity: u16,
    /// Bounded input domain and precondition.
    pub domain: InputDomain,
}

/// Explicit deterministic work budget.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Budget {
    /// Maximum number of backend evaluations across all roles and inputs.
    pub max_evaluations: usize,
    /// Maximum evaluation steps for one backend evaluation.
    pub max_steps_per_evaluation: usize,
}

impl Budget {
    /// Build a budget.
    #[must_use]
    pub fn new(max_evaluations: usize, max_steps_per_evaluation: usize) -> Self {
        Self {
            max_evaluations,
            max_steps_per_evaluation,
        }
    }
}

/// Optional Pijul-native provenance. It is recorded, never evaluated.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct PijulProvenance {
    /// Native change identities supplied by the source workflow.
    pub change_identities: Vec<String>,
    /// Recorded-only note about how the histories were obtained.
    pub note: String,
}

/// Complete input to the bounded relation checker.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct MergeRequest {
    /// Explicit assumptions.
    pub assumptions: Assumptions,
    /// Exactly one variant per role.
    pub variants: Vec<Variant>,
    /// Exactly one binding per role.
    pub bindings: Vec<RoleBinding>,
    /// Logical item mappings covering observables and reachable dependencies.
    pub items: Vec<ItemMapping>,
    /// Declared observable closure.
    pub observables: Vec<Observable>,
    /// Work bounds.
    pub budget: Budget,
    /// Optional recorded-only native provenance.
    pub provenance: Option<PijulProvenance>,
}

/// A scalar value or an abnormal termination outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AbortKind {
    /// Integer overflow.
    Overflow,
    /// Division or remainder by zero.
    DivideByZero,
}

impl AbortKind {
    /// Return the stable label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overflow => "overflow",
            Self::DivideByZero => "divide-by-zero",
        }
    }
}

/// Normal and abnormal observation outcomes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Value {
    /// Normal integer result.
    Int(i64),
    /// Abnormal termination.
    Aborted(AbortKind),
}

impl std::fmt::Display for Value {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int(value) => write!(formatter, "int:{value}"),
            Self::Aborted(kind) => write!(formatter, "abort:{}", kind.as_str()),
        }
    }
}

/// Why the four-version relation failed at one observation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CounterexampleClass {
    /// `A` and `B` changed the same observation incompatibly.
    Incompatible,
    /// The merge does not match `A`'s change.
    LostAChange,
    /// The merge does not match `B`'s change.
    LostBChange,
    /// Neither variant changed the observation, but the merge did.
    Interference,
}

impl CounterexampleClass {
    /// Return the stable label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Incompatible => "incompatible",
            Self::LostAChange => "lost-a-change",
            Self::LostBChange => "lost-b-change",
            Self::Interference => "interference",
        }
    }
}

/// One deterministic counterexample.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Counterexample {
    /// Logical observable identity.
    pub observable: String,
    /// Concrete argument tuple.
    pub input: Vec<i64>,
    /// Base observation.
    pub base: Value,
    /// Variant `A` observation.
    pub a: Value,
    /// Variant `B` observation.
    pub b: Value,
    /// Candidate merge observation.
    pub merge: Value,
    /// Relation failure class.
    pub class: CounterexampleClass,
}

/// Unsupported-input reasons.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnsupportedReason {
    /// A required role is missing.
    MissingRole,
    /// A role appears more than once.
    DuplicateRole,
    /// A required binding is missing.
    MissingBinding,
    /// A binding appears more than once.
    DuplicateBinding,
    /// A structural request field is malformed.
    InvalidRequest,
    /// An observed item is absent from one or more variants.
    MissingObservable,
    /// A mapped item is absent from one or more variants.
    MissingItem,
    /// A called item has no reviewed mapping.
    MissingItemMapping,
    /// One role-local name maps to more than one logical item.
    AmbiguousItemMapping,
    /// A declared binding does not match the supplied source or IR.
    BindingMismatch,
    /// Role-tagged source identities are a permutation of the declared roles.
    RoleMismatch,
    /// A function's signature differs from the reviewed signature.
    SignatureMismatch,
    /// A called logical item is missing from an admitted program.
    UndefinedDependency,
    /// A reachable dependency changed without being declared observable.
    ChangedDependency,
    /// The reachable call graph contains a cycle.
    Recursion,
    /// A construct is outside the admitted subset.
    UnsupportedConstruct,
    /// The requested input domain exceeds its explicit bound.
    DomainExceedsBound,
    /// No observables were declared.
    EmptyObservableSet,
}

impl UnsupportedReason {
    /// Return the stable reason code.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MissingRole => "missing-role",
            Self::DuplicateRole => "duplicate-role",
            Self::MissingBinding => "missing-binding",
            Self::DuplicateBinding => "duplicate-binding",
            Self::InvalidRequest => "invalid-request",
            Self::MissingObservable => "missing-observable",
            Self::MissingItem => "missing-item",
            Self::MissingItemMapping => "missing-item-mapping",
            Self::AmbiguousItemMapping => "ambiguous-item-mapping",
            Self::BindingMismatch => "binding-mismatch",
            Self::RoleMismatch => "role-mismatch",
            Self::SignatureMismatch => "signature-mismatch",
            Self::UndefinedDependency => "undefined-dependency",
            Self::ChangedDependency => "changed-dependency",
            Self::Recursion => "recursion",
            Self::UnsupportedConstruct => "unsupported-construct",
            Self::DomainExceedsBound => "domain-exceeds-bound",
            Self::EmptyObservableSet => "empty-observable-set",
        }
    }
}

/// Unknown-verdict reasons.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnknownReason {
    /// An observable's precondition admits no input.
    VacuousPrecondition,
    /// No observable admitted any input.
    NoAdmissibleInputs,
    /// A dependency summary could not be justified.
    IncompleteDependencySummary,
}

impl UnknownReason {
    /// Return the stable reason code.
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::VacuousPrecondition => "vacuous-precondition",
            Self::NoAdmissibleInputs => "no-admissible-inputs",
            Self::IncompleteDependencySummary => "incomplete-dependency-summary",
        }
    }
}

/// Phase in which a bounded run was interrupted.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TimeoutPhase {
    /// Blocked on domain enumeration or admission.
    Admission,
    /// Interrupted while evaluating observations.
    Evaluation,
    /// Interrupted while checking the relation.
    Relation,
}

impl TimeoutPhase {
    /// Return the stable label.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Admission => "admission",
            Self::Evaluation => "evaluation",
            Self::Relation => "relation",
        }
    }
}

/// Deterministic result of one bounded four-version check.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum Outcome {
    /// Every declared observation and input satisfied the relation.
    Verified {
        /// Number of observables checked.
        observables: usize,
        /// Number of admitted input tuples checked.
        inputs: usize,
    },
    /// At least one observation and input violated the relation.
    Counterexample {
        /// Deterministic counterexamples.
        counterexamples: Vec<Counterexample>,
        /// Whether the counterexample list was truncated by the receipt bound.
        truncated: bool,
    },
    /// The checker could not justify a verdict.
    Unknown {
        /// Unknown reason.
        reason: UnknownReason,
        /// Human-readable detail.
        detail: String,
    },
    /// The request is outside the admitted subset or lacks exact mappings.
    Unsupported {
        /// Unsupported reason.
        reason: UnsupportedReason,
        /// Human-readable detail.
        detail: String,
    },
    /// The explicit budget interrupted the run.
    Timeout {
        /// Interrupted phase.
        phase: TimeoutPhase,
        /// Completed evaluations.
        completed: usize,
        /// Limit that stopped the run.
        limit: usize,
    },
    /// The injected backend failed.
    ToolError {
        /// Backend identity.
        backend: String,
        /// Backend error message.
        message: String,
    },
}

impl Outcome {
    /// Return the stable outcome kind.
    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self {
            Self::Verified { .. } => "verified",
            Self::Counterexample { .. } => "counterexample",
            Self::Unknown { .. } => "unknown",
            Self::Unsupported { .. } => "unsupported",
            Self::Timeout { .. } => "timeout",
            Self::ToolError { .. } => "tool-error",
        }
    }
}
