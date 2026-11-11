//! Pure checked core for the spec-trace parity rail.
// r[impl octet.spec_trace_parity.rail]
// r[impl octet.spec_trace_parity.freshness]
// r[impl octet.spec_trace_parity.replay]
// r[impl octet.spec_trace_parity.adapter]
// r[impl octet.spec_trace_parity.receipt]
//!
//! The core admits pinned model, checker, seed, trace-set, and adapter
//! identities, replays a bounded trace set through an injected adapter, and
//! builds deterministic receipts. It never discovers an adapter, reads a file,
//! spawns a process, or inspects a clock. The imperative shell locates the
//! crate, loads the adapter and traces, measures elapsed time, and persists
//! receipts; this module owns every admission, replay, and receipt decision.

/// Versioned spec-trace parity check identity.
pub const CHECK_VERSION: &str = "octet-spec-trace-parity/v1";
/// Receipt schema version.
pub const RECEIPT_SCHEMA: &str = "octet-spec-trace-parity-receipt/v1";
/// Declared stack trace interchange format identity.
pub const TRACE_FORMAT: &str = "octet.spec-trace/v1";
/// Hard upper bound on declared replay steps.
pub const MAX_STEP_BUDGET: u32 = 1_000_000;
/// Hard upper bound on declared replay time in milliseconds.
pub const MAX_TIME_BUDGET_MILLIS: u64 = 3_600_000;
/// Hard upper bound on traces per run.
pub const MAX_TRACES: usize = 10_000;
/// Maximum retained divergence samples per receipt.
pub const DIVERGENCE_SAMPLE_LIMIT: usize = 16;
/// Claim boundaries required on every receipt.
pub const REQUIRED_NON_CLAIMS: [&str; 4] = [
    "replayed parity covers only the admitted trace set",
    "does not prove behavioral model validity or checker soundness",
    "does not prove implementation behavior outside the traced set",
    "does not establish release eligibility",
];
/// Required disclosure when an adapter projection can lose divergence.
pub const PROJECTION_NON_CLAIM: &str = "parity is claimed only under the adapter's declared state projection and loss bound; divergences lost by projection are not detected";

mod adapter;
mod freshness;
mod model;
mod receipt;
mod replay;

#[cfg(test)]
mod tests;

pub use adapter::ReplayStep;
pub use adapter::StepObservation;
pub use adapter::TraceAdapter;
pub use adapter::adapter_binding;
pub use adapter::adapter_identity;
pub use adapter::validate_adapter;
pub use freshness::admit;
pub use freshness::validate_request;
pub use model::AdapterBinding;
pub use model::AdapterDeclaration;
pub use model::Admission;
pub use model::AdmittedIdentities;
pub use model::BoundDenial;
pub use model::CheckerIdentity;
pub use model::Denial;
pub use model::DenialLayer;
pub use model::DivergenceSample;
pub use model::LossBound;
pub use model::ParityPins;
pub use model::ParityReceipt;
pub use model::ParityRequest;
pub use model::ParityRun;
pub use model::ReceiptOutcome;
pub use model::ReplayPlan;
pub use model::ReplayRecord;
pub use model::RunBounds;
pub use model::StateProjection;
pub use model::StepRecord;
pub use model::Trace;
pub use model::TraceSet;
pub use model::TraceStep;
pub use model::VerdictClass;
pub use receipt::validate_receipt;

/// Plan a deterministic replay after admitting every declared pin.
///
/// # Errors
///
/// Returns the ordered denial layers when any pin is stale, malformed, or
/// unsupported.
pub fn plan(request: &ParityRequest, adapter: Option<&AdapterDeclaration>) -> Result<ReplayPlan, Vec<Denial>> {
    match admit(request, adapter) {
        Admission::Admitted(identities) => Ok(ReplayPlan {
            identities: *identities,
            bounds: request.bounds,
        }),
        Admission::Denied(denials) => Err(denials),
    }
}

/// Admit, replay, classify, and build one deterministic receipt.
///
/// # Errors
///
/// Returns an error only when a closed typed receipt cannot be serialized.
pub fn run(request: &ParityRequest, adapter: Option<&mut dyn TraceAdapter>) -> Result<ParityRun, String> {
    let declaration = adapter.as_ref().map(|value| value.declaration().clone());
    let context = receipt::ReceiptContext::from_request(request, declaration.as_ref());
    let planned = match plan(request, declaration.as_ref()) {
        Ok(planned) => planned,
        Err(denials) => {
            let receipt = receipt::build_receipt(&context, ReceiptOutcome::Denied { denials }, 0, Vec::new())?;
            return Ok(ParityRun { receipt, record: None });
        }
    };
    let Some(adapter) = adapter else {
        let receipt = receipt::build_receipt(
            &context,
            ReceiptOutcome::Verdict(VerdictClass::UnsupportedVerifier),
            0,
            Vec::new(),
        )?;
        return Ok(ParityRun { receipt, record: None });
    };
    let Some(binding) = planned.identities.adapter.clone() else {
        return Err("admitted adapter is missing its binding".to_owned());
    };
    let record = replay::execute(&planned, request, &binding, adapter);
    let replayed_steps = u32::try_from(record.steps.len()).unwrap_or(u32::MAX);
    let samples = replay::divergence_samples(&record);
    let outcome = if let Some(class) = replay::classify(&record) {
        ReceiptOutcome::Verdict(class)
    } else {
        let Some(denial) = replay::bound_denial(&record) else {
            return Err("bounded replay has no terminating denial".to_owned());
        };
        ReceiptOutcome::Denied { denials: vec![denial] }
    };
    let receipt = receipt::build_receipt(&context, outcome, replayed_steps, samples)?;
    Ok(ParityRun {
        receipt,
        record: Some(record),
    })
}

/// Compute the BLAKE3 identity of a receipt with its identity field cleared.
///
/// # Errors
///
/// Returns an error when the typed receipt cannot be serialized.
pub fn receipt_identity(receipt: &ParityReceipt) -> Result<String, String> {
    let mut projection = receipt.clone();
    projection.receipt_identity.clear();
    let mut bytes = serde_json::to_vec_pretty(&projection)
        .map_err(|error| format!("serialize spec-trace parity receipt: {error}"))?;
    bytes.push(b'\n');
    Ok(prefixed_identity(&bytes))
}

/// Return a canonical `b3:` identity for exact bytes.
#[must_use]
pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

/// Return whether a string is one canonical lowercase `b3:` identity.
#[must_use]
pub fn is_b3(value: &str) -> bool {
    value.strip_prefix("b3:").is_some_and(|hex| {
        hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

/// Return the canonical BLAKE3 identity of a trace set.
///
/// # Panics
///
/// Panics only if the closed trace-set type cannot be serialized, which cannot
/// happen for its string-only fields.
#[must_use]
pub fn trace_set_identity(trace_set: &TraceSet) -> String {
    prefixed_identity(&serde_json::to_vec(trace_set).expect("trace set must serialize"))
}
