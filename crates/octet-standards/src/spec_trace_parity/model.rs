use serde::Deserialize;
use serde::Serialize;

/// Declared admission pins for one spec-trace parity run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParityPins {
    /// BLAKE3 identity of the behavioral model artifact.
    pub model_identity: String,
    /// BLAKE3 identity of the model checker build.
    pub checker_identity: String,
    /// Declared model checker version.
    pub checker_version: String,
    /// Explicit replay seed.
    pub seed: u64,
    /// BLAKE3 identity of the canonical trace set.
    pub trace_set_identity: String,
    /// Declared replay-adapter identity, absent when no adapter is declared.
    pub adapter_identity: Option<String>,
}

/// Supplied checker identity and version.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CheckerIdentity {
    /// BLAKE3 identity of the model checker build.
    pub identity: String,
    /// Model checker version.
    pub version: String,
}

/// Supplied model, checker, seed, trace set, and bounds for one run.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParityRequest {
    /// Declared admission pins.
    pub pins: ParityPins,
    /// Exact behavioral model bytes the pin must match.
    pub model_bytes: Vec<u8>,
    /// Supplied checker identity and version.
    pub checker: CheckerIdentity,
    /// Supplied replay seed.
    pub seed: u64,
    /// Supplied trace set.
    pub trace_set: TraceSet,
    /// Declared step and time bounds.
    pub bounds: RunBounds,
}

/// One canonical, stack-identified trace set.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TraceSet {
    /// Declared interchange format identity.
    pub format: String,
    /// Ordered traces.
    pub traces: Vec<Trace>,
}

/// One model-generated trace.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Trace {
    /// Stable trace identifier.
    pub trace_id: String,
    /// Ordered trace steps.
    pub steps: Vec<TraceStep>,
}

/// One action and its expected model state.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TraceStep {
    /// Declared action or event name.
    pub action: String,
    /// Expected model state after the action.
    pub expected_state: String,
}

/// Declared replay bounds.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RunBounds {
    /// Maximum replayed steps across every trace.
    pub max_steps: u32,
    /// Maximum cumulative replay time in milliseconds.
    pub max_millis: u64,
}

/// Declared adapter state projection.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StateProjection {
    /// Declared model-state vocabulary.
    pub model_states: String,
    /// Declared implementation-state vocabulary.
    pub implementation_states: String,
    /// Human-readable projection description.
    pub description: String,
}

/// Declared adapter loss bound.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LossBound {
    /// The projection preserves every observable state distinction.
    Exact,
    /// The projection can merge distinct implementation states.
    Bounded {
        /// Declared bound or collision description.
        bound: String,
    },
}

impl LossBound {
    /// Return whether the projection claims to preserve every state distinction.
    #[must_use]
    pub fn is_exact(&self) -> bool {
        matches!(self, Self::Exact)
    }
}

/// Injected replay-adapter declaration.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterDeclaration {
    /// Stable adapter identifier.
    pub adapter_id: String,
    /// Trace interchange format the adapter consumes.
    pub trace_format: String,
    /// Declared state projection.
    pub projection: StateProjection,
    /// Declared loss bound.
    pub loss_bound: LossBound,
}

/// Adapter identity and declared projection bound into a plan or receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AdapterBinding {
    /// BLAKE3 identity of the adapter declaration.
    pub identity: String,
    /// Declared state projection.
    pub projection: StateProjection,
    /// Declared loss bound.
    pub loss_bound: LossBound,
}

/// Admission layers checked before any replay step.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DenialLayer {
    /// Request shape is malformed.
    Request,
    /// Behavioral model identity drifted.
    Model,
    /// Model checker identity drifted.
    CheckerIdentity,
    /// Model checker version drifted.
    CheckerVersion,
    /// Replay seed drifted.
    Seed,
    /// Trace-set identity drifted or is malformed.
    TraceSet,
    /// Trace interchange format is not supported.
    TraceFormat,
    /// Replay-adapter identity drifted.
    AdapterIdentity,
    /// Replay-adapter declaration omits or malforms its contract.
    AdapterDeclaration,
    /// Replay adapter does not consume the supplied trace format.
    AdapterTraceFormat,
    /// Step budget was exceeded.
    StepBudget,
    /// Time budget was exceeded.
    TimeBudget,
}

/// One deterministic denial bound to an admission layer.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Denial {
    /// Denied admission layer.
    pub layer: DenialLayer,
    /// Declared or required value.
    pub expected: String,
    /// Observed value.
    pub observed: String,
}

/// Freshness admission decision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Admission {
    /// Every pin matched the supplied artifacts.
    Admitted(Box<AdmittedIdentities>),
    /// At least one pin was stale, malformed, or unsupported.
    Denied(Vec<Denial>),
}

/// Exact identities admitted by one freshness check.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AdmittedIdentities {
    /// Admitted behavioral model identity.
    pub model_identity: String,
    /// Admitted model checker identity.
    pub checker_identity: String,
    /// Admitted model checker version.
    pub checker_version: String,
    /// Admitted replay seed.
    pub seed: u64,
    /// Admitted trace-set identity.
    pub trace_set_identity: String,
    /// Admitted trace interchange format.
    pub trace_format: String,
    /// Admitted adapter binding, absent when no adapter is declared.
    pub adapter: Option<AdapterBinding>,
}

/// Deterministic replay plan recorded before execution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayPlan {
    /// Admitted identities.
    pub identities: AdmittedIdentities,
    /// Declared bounds.
    pub bounds: RunBounds,
}

/// Closed verdict vocabulary for a completed replay.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum VerdictClass {
    /// Every replayed step matched the model under the declared projection.
    Parity,
    /// At least one replayed step diverged from the model.
    Divergence,
    /// No declared adapter can verify the trace set.
    UnsupportedVerifier,
}

/// One observed divergence sample.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DivergenceSample {
    /// Trace identifier.
    pub trace_id: String,
    /// Zero-based step index inside the trace.
    pub step_index: u32,
    /// Declared action name.
    pub action: String,
    /// Expected model state.
    pub expected_state: String,
    /// Observed implementation state.
    pub observed_state: String,
}

/// One replayed step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepRecord {
    /// Trace identifier.
    pub trace_id: String,
    /// Zero-based step index inside the trace.
    pub step_index: u32,
    /// Declared action name.
    pub action: String,
    /// Expected model state.
    pub expected_state: String,
    /// Observed implementation state.
    pub observed_state: String,
}

/// Deterministic replay record produced by the injected adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayRecord {
    /// Adapter binding that produced this record.
    pub adapter: AdapterBinding,
    /// Replayed steps in execution order.
    pub steps: Vec<StepRecord>,
    /// Bound denial that terminated the replay early.
    pub bound_denial: Option<BoundDenial>,
}

/// Bound that terminated a replay early.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BoundDenial {
    /// The declared step budget cannot cover the trace set.
    StepBudget {
        /// Declared step limit.
        limit: u32,
        /// Required step count.
        observed: u32,
    },
    /// The declared time budget was exceeded.
    TimeBudget {
        /// Declared millisecond limit.
        limit_millis: u64,
    },
}

/// Closed receipt outcome: a verdict class or a deterministic denial.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "kebab-case")]
pub enum ReceiptOutcome {
    /// Completed replay with one of the closed verdict classes.
    Verdict(VerdictClass),
    /// Fail-closed denial before or during replay.
    Denied {
        /// Ordered denial layers.
        denials: Vec<Denial>,
    },
}

/// Deterministic spec-trace parity receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ParityReceipt {
    /// Receipt schema version.
    pub schema_version: String,
    /// Check version that produced the receipt.
    pub check_version: String,
    /// Pinned behavioral model identity.
    pub model_identity: String,
    /// Pinned model checker identity.
    pub checker_identity: String,
    /// Pinned model checker version.
    pub checker_version: String,
    /// Pinned replay seed.
    pub seed: u64,
    /// Pinned trace-set identity.
    pub trace_set_identity: String,
    /// Supplied trace interchange format.
    pub trace_format: String,
    /// Adapter binding, absent only for an unsupported-verifier result.
    pub adapter: Option<AdapterBinding>,
    /// Declared bounds.
    pub bounds: RunBounds,
    /// Verdict or denial.
    pub outcome: ReceiptOutcome,
    /// Number of replayed steps.
    pub replayed_steps: u32,
    /// Bounded divergence samples.
    pub divergence_samples: Vec<DivergenceSample>,
    /// Explicit claim boundaries.
    pub non_claims: Vec<String>,
    /// BLAKE3 identity of the receipt with this field cleared.
    pub receipt_identity: String,
}

/// One run's receipt and observed replay record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ParityRun {
    /// Deterministic receipt.
    pub receipt: ParityReceipt,
    /// Observed replay record, absent when no replay step ran.
    pub record: Option<ReplayRecord>,
}
