//! Deterministic receipt construction and validation for the spec-trace parity rail.
// r[impl octet.spec_trace_parity.receipt]
use super::CHECK_VERSION;
use super::PROJECTION_NON_CLAIM;
use super::RECEIPT_SCHEMA;
use super::REQUIRED_NON_CLAIMS;
use super::adapter::adapter_binding;
use super::adapter::validate_adapter;
use super::freshness::admit;
use super::freshness::validate_request;
use super::model::AdapterBinding;
use super::model::AdapterDeclaration;
use super::model::Admission;
use super::model::DivergenceSample;
use super::model::ParityReceipt;
use super::model::ParityRequest;
use super::model::ReceiptOutcome;
use super::model::ReplayRecord;
use super::model::RunBounds;
use super::model::VerdictClass;
use super::receipt_identity;
use super::replay;

/// Declared receipt fields captured before replay.
pub(super) struct ReceiptContext {
    model_identity: String,
    checker_identity: String,
    checker_version: String,
    seed: u64,
    trace_set_identity: String,
    trace_format: String,
    adapter: Option<AdapterBinding>,
    bounds: RunBounds,
}

impl ReceiptContext {
    /// Capture the declared pins and supplied adapter contract for one receipt.
    pub(super) fn from_request(request: &ParityRequest, adapter: Option<&AdapterDeclaration>) -> Self {
        Self {
            model_identity: request.pins.model_identity.clone(),
            checker_identity: request.pins.checker_identity.clone(),
            checker_version: request.pins.checker_version.clone(),
            seed: request.pins.seed,
            trace_set_identity: request.pins.trace_set_identity.clone(),
            trace_format: request.trace_set.format.clone(),
            adapter: adapter.map(adapter_binding),
            bounds: request.bounds,
        }
    }

    fn non_claims(&self) -> Vec<String> {
        let mut claims = REQUIRED_NON_CLAIMS.iter().map(|claim| (*claim).to_owned()).collect::<Vec<_>>();
        if self.adapter.as_ref().is_some_and(|binding| !binding.loss_bound.is_exact()) {
            claims.push(PROJECTION_NON_CLAIM.to_owned());
        }
        claims
    }
}

/// Build one deterministic receipt and bind its identity.
pub(super) fn build_receipt(
    context: &ReceiptContext,
    outcome: ReceiptOutcome,
    replayed_steps: u32,
    divergence_samples: Vec<DivergenceSample>,
) -> Result<ParityReceipt, String> {
    let mut receipt = ParityReceipt {
        schema_version: RECEIPT_SCHEMA.to_owned(),
        check_version: CHECK_VERSION.to_owned(),
        model_identity: context.model_identity.clone(),
        checker_identity: context.checker_identity.clone(),
        checker_version: context.checker_version.clone(),
        seed: context.seed,
        trace_set_identity: context.trace_set_identity.clone(),
        trace_format: context.trace_format.clone(),
        adapter: context.adapter.clone(),
        bounds: context.bounds,
        outcome,
        replayed_steps,
        divergence_samples,
        non_claims: context.non_claims(),
        receipt_identity: String::new(),
    };
    receipt.receipt_identity = receipt_identity(&receipt)?;
    Ok(receipt)
}

/// Validate a receipt against the admitted request, adapter, and observed replay.
///
/// The observed record is required for any verdict or bound denial so the
/// receipt class is recomputed instead of trusted.
#[must_use]
pub fn validate_receipt(
    receipt: &ParityReceipt,
    request: &ParityRequest,
    adapter: Option<&AdapterDeclaration>,
    record: Option<&ReplayRecord>,
) -> Vec<String> {
    let mut diagnostics = validate_request(request);
    let context = ReceiptContext::from_request(request, adapter);
    validate_fields(receipt, &context, &mut diagnostics);

    match (&receipt.outcome, admit(request, adapter)) {
        (ReceiptOutcome::Denied { denials }, Admission::Denied(expected)) => {
            if denials != &expected {
                diagnostics.push("receipt denial layers do not match freshness admission".to_owned());
            }
            if record.is_some() {
                diagnostics.push("freshness denial must not carry a replay record".to_owned());
            }
        }
        (ReceiptOutcome::Denied { denials }, Admission::Admitted(_)) => {
            validate_bound_denial(receipt, denials, request, record, &mut diagnostics);
        }
        (ReceiptOutcome::Verdict(_), Admission::Denied(_)) => {
            diagnostics.push("receipt claims a verdict for a denied run".to_owned());
        }
        (ReceiptOutcome::Verdict(class), Admission::Admitted(_)) => {
            validate_verdict(*class, receipt, request, adapter, record, &mut diagnostics);
        }
    }

    if receipt.non_claims != context.non_claims() {
        diagnostics.push("receipt non-claims do not match the declared claim boundary".to_owned());
    }
    match receipt_identity(receipt) {
        Ok(expected) if receipt.receipt_identity == expected => {}
        Ok(_) => diagnostics.push("receipt identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_fields(receipt: &ParityReceipt, context: &ReceiptContext, diagnostics: &mut Vec<String>) {
    if receipt.schema_version != RECEIPT_SCHEMA
        || receipt.check_version != CHECK_VERSION
        || receipt.model_identity != context.model_identity
        || receipt.checker_identity != context.checker_identity
        || receipt.checker_version != context.checker_version
        || receipt.seed != context.seed
        || receipt.trace_set_identity != context.trace_set_identity
        || receipt.trace_format != context.trace_format
        || receipt.adapter != context.adapter
        || receipt.bounds != context.bounds
    {
        diagnostics.push("receipt identity binding drifted from the admitted request".to_owned());
    }
}

fn validate_bound_denial(
    receipt: &ParityReceipt,
    denials: &[super::model::Denial],
    request: &ParityRequest,
    record: Option<&ReplayRecord>,
    diagnostics: &mut Vec<String>,
) {
    let Some(record) = record else {
        diagnostics.push("denied receipt has no bound denial evidence".to_owned());
        return;
    };
    diagnostics.extend(replay::validate_record(request, record));
    match replay::bound_denial(record) {
        Some(expected) => {
            if denials.len() != 1 || denials[0] != expected {
                diagnostics.push("receipt denial layers do not match the observed bound".to_owned());
            }
        }
        None => diagnostics.push("denied receipt has no terminating bound".to_owned()),
    }
    if replay::classify(record).is_some() {
        diagnostics.push("denied receipt must not classify a completed replay".to_owned());
    }
    validate_record_binding(receipt, record, diagnostics);
}

fn validate_verdict(
    class: VerdictClass,
    receipt: &ParityReceipt,
    request: &ParityRequest,
    adapter: Option<&AdapterDeclaration>,
    record: Option<&ReplayRecord>,
    diagnostics: &mut Vec<String>,
) {
    match (adapter, record) {
        (None, None) => {
            if class != VerdictClass::UnsupportedVerifier {
                diagnostics.push("receipt claims parity or divergence without a declared adapter".to_owned());
            }
            if receipt.replayed_steps != 0 || !receipt.divergence_samples.is_empty() {
                diagnostics.push("unsupported-verifier receipt must not report replayed evidence".to_owned());
            }
        }
        (None, Some(_)) => {
            diagnostics.push("unsupported-verifier receipt must not carry a replay record".to_owned());
        }
        (Some(declaration), Some(record)) => {
            if !validate_adapter(declaration).is_empty() {
                diagnostics.push("receipt binds an invalid adapter declaration".to_owned());
            }
            diagnostics.extend(replay::validate_record(request, record));
            match replay::classify(record) {
                Some(expected) if expected == class => {}
                Some(_) => diagnostics.push("receipt verdict class does not match the observed outcome".to_owned()),
                None => diagnostics.push("receipt verdict class covers an incomplete replay".to_owned()),
            }
            if receipt.adapter.as_ref() != Some(&record.adapter) {
                diagnostics.push("receipt adapter binding does not match the replay record".to_owned());
            }
            validate_record_binding(receipt, record, diagnostics);
        }
        (Some(_), None) => diagnostics.push("verdict receipt is missing its replay record".to_owned()),
    }
}

fn validate_record_binding(receipt: &ParityReceipt, record: &ReplayRecord, diagnostics: &mut Vec<String>) {
    if receipt.replayed_steps != u32::try_from(record.steps.len()).unwrap_or(u32::MAX) {
        diagnostics.push("receipt replayed-step count does not match the replay record".to_owned());
    }
    if receipt.divergence_samples != replay::divergence_samples(record) {
        diagnostics.push("receipt divergence samples do not match the replay record".to_owned());
    }
}
