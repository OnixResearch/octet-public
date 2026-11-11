//! Injected replay-adapter contract for the spec-trace parity rail.
// r[impl octet.spec_trace_parity.adapter]
use super::TRACE_FORMAT;
use super::model::AdapterBinding;
use super::model::AdapterDeclaration;
use super::prefixed_identity;

/// One replay step handed to an injected adapter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReplayStep {
    /// Admitted replay seed.
    pub seed: u64,
    /// Trace identifier.
    pub trace_id: String,
    /// Zero-based step index inside the trace.
    pub step_index: u32,
    /// Declared action name.
    pub action: String,
    /// Expected model state.
    pub expected_state: String,
}

/// One adapter observation for a replay step.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StepObservation {
    /// Observed implementation state under the declared projection.
    pub implementation_state: String,
    /// Cumulative replay time reported by the imperative shell.
    pub elapsed_millis: u64,
}

/// Injected per-crate replay adapter.
///
/// The pure core never discovers an adapter, guesses state mapping, or performs
/// I/O. The imperative shell supplies one adapter per target crate.
pub trait TraceAdapter {
    /// Return the adapter's declared contract.
    fn declaration(&self) -> &AdapterDeclaration;

    /// Observe the implementation state for one replay step.
    fn observe(&mut self, step: &ReplayStep) -> StepObservation;
}

/// Return the canonical BLAKE3 identity of an adapter declaration.
///
/// # Panics
///
/// Panics only if the closed adapter declaration cannot be serialized, which
/// cannot happen for its string-only fields.
#[must_use]
pub fn adapter_identity(declaration: &AdapterDeclaration) -> String {
    prefixed_identity(&serde_json::to_vec(declaration).expect("adapter declaration must serialize"))
}

/// Bind an adapter declaration to its exact identity.
#[must_use]
pub fn adapter_binding(declaration: &AdapterDeclaration) -> AdapterBinding {
    AdapterBinding {
        identity: adapter_identity(declaration),
        projection: declaration.projection.clone(),
        loss_bound: declaration.loss_bound.clone(),
    }
}

/// Validate an adapter declaration without external effects.
///
/// An adapter that omits its state projection or loss bound is rejected.
#[must_use]
pub fn validate_adapter(declaration: &AdapterDeclaration) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if declaration.adapter_id.trim().is_empty() {
        diagnostics.push("adapter declaration omits its stable identifier".to_owned());
    }
    if declaration.trace_format != TRACE_FORMAT {
        diagnostics.push(format!("adapter declares unsupported trace format `{}`", declaration.trace_format));
    }
    if declaration.projection.model_states.trim().is_empty()
        || declaration.projection.implementation_states.trim().is_empty()
        || declaration.projection.description.trim().is_empty()
    {
        diagnostics.push("adapter declaration omits its state projection".to_owned());
    }
    if let super::model::LossBound::Bounded { bound } = &declaration.loss_bound
        && bound.trim().is_empty()
    {
        diagnostics.push("adapter declaration omits its loss bound".to_owned());
    }
    diagnostics
}
