//! Deterministic, seed-bound replay for the spec-trace parity rail.
// r[impl octet.spec_trace_parity.replay]
use super::DIVERGENCE_SAMPLE_LIMIT;
use super::adapter::ReplayStep;
use super::adapter::TraceAdapter;
use super::model::AdapterBinding;
use super::model::BoundDenial;
use super::model::Denial;
use super::model::DenialLayer;
use super::model::DivergenceSample;
use super::model::ParityRequest;
use super::model::ReplayPlan;
use super::model::ReplayRecord;
use super::model::StepRecord;
use super::model::VerdictClass;

/// Replay every trace in the plan against the injected adapter.
pub(super) fn execute(
    plan: &ReplayPlan,
    request: &ParityRequest,
    binding: &AdapterBinding,
    adapter: &mut dyn TraceAdapter,
) -> ReplayRecord {
    let total = request.trace_set.traces.iter().map(|trace| trace.steps.len()).sum::<usize>();
    let total_steps = u32::try_from(total).unwrap_or(u32::MAX);
    if total_steps > plan.bounds.max_steps {
        return ReplayRecord {
            adapter: binding.clone(),
            steps: Vec::new(),
            bound_denial: Some(BoundDenial::StepBudget {
                limit: plan.bounds.max_steps,
                observed: total_steps,
            }),
        };
    }

    let mut steps = Vec::with_capacity(total);
    for trace in &request.trace_set.traces {
        for (index, step) in trace.steps.iter().enumerate() {
            let step_index = u32::try_from(index).unwrap_or(u32::MAX);
            let observation = adapter.observe(&ReplayStep {
                seed: plan.identities.seed,
                trace_id: trace.trace_id.clone(),
                step_index,
                action: step.action.clone(),
                expected_state: step.expected_state.clone(),
            });
            steps.push(StepRecord {
                trace_id: trace.trace_id.clone(),
                step_index,
                action: step.action.clone(),
                expected_state: step.expected_state.clone(),
                observed_state: observation.implementation_state,
            });
            if observation.elapsed_millis > plan.bounds.max_millis {
                return ReplayRecord {
                    adapter: binding.clone(),
                    steps,
                    bound_denial: Some(BoundDenial::TimeBudget {
                        limit_millis: plan.bounds.max_millis,
                    }),
                };
            }
        }
    }
    ReplayRecord {
        adapter: binding.clone(),
        steps,
        bound_denial: None,
    }
}

/// Classify a completed replay record into the closed verdict vocabulary.
///
/// A bound denial without an observed divergence yields no verdict: the run is
/// denied instead of claiming parity over an incomplete trace set.
#[must_use]
pub(super) fn classify(record: &ReplayRecord) -> Option<VerdictClass> {
    if record.steps.iter().any(|step| step.expected_state != step.observed_state) {
        Some(VerdictClass::Divergence)
    } else if record.bound_denial.is_some() {
        None
    } else {
        Some(VerdictClass::Parity)
    }
}

/// Return bounded divergence samples in replay order.
#[must_use]
pub(super) fn divergence_samples(record: &ReplayRecord) -> Vec<DivergenceSample> {
    record
        .steps
        .iter()
        .filter(|step| step.expected_state != step.observed_state)
        .take(DIVERGENCE_SAMPLE_LIMIT)
        .map(|step| DivergenceSample {
            trace_id: step.trace_id.clone(),
            step_index: step.step_index,
            action: step.action.clone(),
            expected_state: step.expected_state.clone(),
            observed_state: step.observed_state.clone(),
        })
        .collect()
}

/// Convert a terminating bound into one deterministic denial.
#[must_use]
pub(super) fn bound_denial(record: &ReplayRecord) -> Option<Denial> {
    match record.bound_denial {
        Some(BoundDenial::StepBudget { limit, observed }) => Some(Denial {
            layer: DenialLayer::StepBudget,
            expected: limit.to_string(),
            observed: observed.to_string(),
        }),
        Some(BoundDenial::TimeBudget { limit_millis }) => Some(Denial {
            layer: DenialLayer::TimeBudget,
            expected: limit_millis.to_string(),
            observed: "exceeded".to_owned(),
        }),
        None => None,
    }
}

/// Validate that a replay record is a deterministic prefix of the declared trace set.
#[must_use]
pub(super) fn validate_record(request: &ParityRequest, record: &ReplayRecord) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let expected = request
        .trace_set
        .traces
        .iter()
        .flat_map(|trace| {
            trace
                .steps
                .iter()
                .enumerate()
                .map(move |(index, step)| (&trace.trace_id, index, &step.action, &step.expected_state))
        })
        .collect::<Vec<_>>();
    if record.steps.len() > expected.len() {
        diagnostics.push("replay record contains more steps than the trace set".to_owned());
    }
    for (observed, expected) in record.steps.iter().zip(&expected) {
        let step_index = u32::try_from(expected.1).unwrap_or(u32::MAX);
        if observed.trace_id != *expected.0
            || observed.step_index != step_index
            || observed.action != *expected.2
            || observed.expected_state != *expected.3
        {
            diagnostics.push("replay record is not a prefix of the declared trace set".to_owned());
            break;
        }
    }
    match record.bound_denial {
        None => {
            if record.steps.len() != expected.len() {
                diagnostics.push("completed replay did not replay every declared step".to_owned());
            }
        }
        Some(BoundDenial::StepBudget { limit, observed }) => {
            if !record.steps.is_empty() {
                diagnostics.push("step-budget denial must not replay any step".to_owned());
            }
            if limit != request.bounds.max_steps {
                diagnostics.push("step-budget denial does not match the declared bound".to_owned());
            }
            if observed != u32::try_from(expected.len()).unwrap_or(u32::MAX) {
                diagnostics.push("step-budget denial does not bind the declared step count".to_owned());
            }
        }
        Some(BoundDenial::TimeBudget { limit_millis }) => {
            if record.steps.is_empty() && !expected.is_empty() {
                diagnostics.push("time-budget denial must follow at least one replayed step".to_owned());
            }
            if limit_millis != request.bounds.max_millis {
                diagnostics.push("time-budget denial does not match the declared bound".to_owned());
            }
        }
    }
    diagnostics
}
