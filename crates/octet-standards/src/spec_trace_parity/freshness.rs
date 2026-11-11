//! Pure freshness admission for the spec-trace parity rail.
// r[impl octet.spec_trace_parity.freshness]
use std::collections::BTreeSet;

use super::MAX_STEP_BUDGET;
use super::MAX_TIME_BUDGET_MILLIS;
use super::MAX_TRACES;
use super::TRACE_FORMAT;
use super::adapter::adapter_identity;
use super::adapter::validate_adapter;
use super::is_b3;
use super::model::AdapterDeclaration;
use super::model::Admission;
use super::model::AdmittedIdentities;
use super::model::Denial;
use super::model::DenialLayer;
use super::model::ParityRequest;
use super::prefixed_identity;
use super::trace_set_identity;

/// Admit or deny a run against every declared pin without replaying a step.
#[must_use]
pub fn admit(request: &ParityRequest, adapter: Option<&AdapterDeclaration>) -> Admission {
    let request_diagnostics = validate_request(request);
    if !request_diagnostics.is_empty() {
        return Admission::Denied(vec![Denial {
            layer: DenialLayer::Request,
            expected: "well-formed parity request".to_owned(),
            observed: request_diagnostics.join("; "),
        }]);
    }

    let observed_model = prefixed_identity(&request.model_bytes);
    let observed_trace_set = trace_set_identity(&request.trace_set);
    let observed_adapter = adapter.map(adapter_identity);
    let mut denials = Vec::new();

    if observed_model != request.pins.model_identity {
        denials.push(Denial {
            layer: DenialLayer::Model,
            expected: request.pins.model_identity.clone(),
            observed: observed_model.clone(),
        });
    }
    if request.checker.identity != request.pins.checker_identity {
        denials.push(Denial {
            layer: DenialLayer::CheckerIdentity,
            expected: request.pins.checker_identity.clone(),
            observed: request.checker.identity.clone(),
        });
    }
    if request.checker.version != request.pins.checker_version {
        denials.push(Denial {
            layer: DenialLayer::CheckerVersion,
            expected: request.pins.checker_version.clone(),
            observed: request.checker.version.clone(),
        });
    }
    if request.seed != request.pins.seed {
        denials.push(Denial {
            layer: DenialLayer::Seed,
            expected: request.pins.seed.to_string(),
            observed: request.seed.to_string(),
        });
    }
    if observed_trace_set != request.pins.trace_set_identity {
        denials.push(Denial {
            layer: DenialLayer::TraceSet,
            expected: request.pins.trace_set_identity.clone(),
            observed: observed_trace_set.clone(),
        });
    }
    if request.trace_set.format != TRACE_FORMAT {
        denials.push(Denial {
            layer: DenialLayer::TraceFormat,
            expected: TRACE_FORMAT.to_owned(),
            observed: request.trace_set.format.clone(),
        });
    }
    if observed_adapter != request.pins.adapter_identity {
        denials.push(Denial {
            layer: DenialLayer::AdapterIdentity,
            expected: request.pins.adapter_identity.clone().unwrap_or_else(|| "none".to_owned()),
            observed: observed_adapter.clone().unwrap_or_else(|| "none".to_owned()),
        });
    }
    if let Some(declaration) = adapter {
        let diagnostics = validate_adapter(declaration);
        if !diagnostics.is_empty() {
            denials.push(Denial {
                layer: DenialLayer::AdapterDeclaration,
                expected: "complete adapter projection and loss bound".to_owned(),
                observed: diagnostics.join("; "),
            });
        }
        if declaration.trace_format != request.trace_set.format {
            denials.push(Denial {
                layer: DenialLayer::AdapterTraceFormat,
                expected: request.trace_set.format.clone(),
                observed: declaration.trace_format.clone(),
            });
        }
    }

    if denials.is_empty() {
        Admission::Admitted(Box::new(AdmittedIdentities {
            model_identity: observed_model,
            checker_identity: request.checker.identity.clone(),
            checker_version: request.checker.version.clone(),
            seed: request.seed,
            trace_set_identity: observed_trace_set,
            trace_format: request.trace_set.format.clone(),
            adapter: adapter.map(super::adapter::adapter_binding),
        }))
    } else {
        Admission::Denied(denials)
    }
}

/// Validate request shape without comparing declared pins.
#[must_use]
pub fn validate_request(request: &ParityRequest) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if request.model_bytes.is_empty() {
        diagnostics.push("behavioral model artifact is empty".to_owned());
    }
    if !is_b3(&request.pins.model_identity)
        || !is_b3(&request.pins.checker_identity)
        || !is_b3(&request.pins.trace_set_identity)
        || request.pins.adapter_identity.as_deref().is_some_and(|identity| !is_b3(identity))
    {
        diagnostics.push("declared pins are not canonical BLAKE3 identities".to_owned());
    }
    if request.pins.checker_version.trim().is_empty() {
        diagnostics.push("declared checker version is empty".to_owned());
    }
    if !is_b3(&request.checker.identity) || request.checker.version.trim().is_empty() {
        diagnostics.push("supplied checker identity or version is malformed".to_owned());
    }
    if request.bounds.max_steps == 0
        || request.bounds.max_steps > MAX_STEP_BUDGET
        || request.bounds.max_millis == 0
        || request.bounds.max_millis > MAX_TIME_BUDGET_MILLIS
    {
        diagnostics.push("declared step or time budget is outside the supported bounds".to_owned());
    }
    diagnostics.extend(validate_trace_set(request));
    diagnostics
}

fn validate_trace_set(request: &ParityRequest) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let trace_set = &request.trace_set;
    if trace_set.format.trim().is_empty() {
        diagnostics.push("trace-set format identity is empty".to_owned());
    }
    if trace_set.traces.is_empty() {
        diagnostics.push("trace set is empty".to_owned());
    }
    if trace_set.traces.len() > MAX_TRACES {
        diagnostics.push("trace set exceeds the supported trace count".to_owned());
    }
    let mut trace_ids = BTreeSet::new();
    for trace in &trace_set.traces {
        if trace.trace_id.trim().is_empty() || !trace_ids.insert(&trace.trace_id) {
            diagnostics.push("trace identifiers are empty or duplicated".to_owned());
        }
        if trace.steps.is_empty() {
            diagnostics.push("trace contains no steps".to_owned());
        }
        if trace
            .steps
            .iter()
            .any(|step| step.action.trim().is_empty() || step.expected_state.trim().is_empty())
        {
            diagnostics.push("trace step action or expected state is empty".to_owned());
        }
    }
    diagnostics
}
