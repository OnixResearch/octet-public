//! Resource-bound checks for embedded Tiger Style policies.

use crate::diagnostic::Diagnostic;
use crate::diagnostic::DiagnosticInput;
use crate::model::FamilySelection;
use crate::model::Mechanism;
use crate::model::MechanismKind;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::source::SourceFile;

const EMBEDDED_FAMILY: &str = "embedded";
const BOUNDED_QUEUE_POLICY_ID: &str = "tigerstyle.embedded.bounded-queues";
const FIRST_LINE_NUMBER_OFFSET: usize = 1;
const FALLBACK_FORBIDDEN_APIS: &[&str] = &[
    "tokio::sync::mpsc::unbounded_channel",
    "mpsc::unbounded_channel",
    "crossbeam_channel::unbounded",
    "std::sync::mpsc::channel",
];
const FALLBACK_APPROVED_BOUNDED_APIS: &[&str] = &[
    "tokio::sync::mpsc::channel",
    "mpsc::channel",
    "crossbeam_channel::bounded",
];

/// Runs resource-bound policy checks over source files.
#[must_use]
pub fn check_resources(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    selection: &FamilySelection,
) -> Vec<Diagnostic> {
    if !selection.includes(EMBEDDED_FAMILY) {
        return Vec::new();
    }

    let mut diagnostics = Vec::new();
    check_bounded_queue_obligations(registry, sources, &mut diagnostics);
    diagnostics
}

fn check_bounded_queue_obligations(
    registry: &PolicyRegistry,
    sources: &[SourceFile],
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(policy) = registry.policy_by_id(BOUNDED_QUEUE_POLICY_ID) else {
        return;
    };
    let obligations = BoundedQueueObligations::from_policy(policy);
    for source in sources {
        append_source_bounded_queue_diagnostics(policy, &obligations, source, diagnostics);
    }
}

#[derive(Debug)]
struct BoundedQueueObligations {
    forbidden_apis: Vec<String>,
    approved_bounded_apis: Vec<String>,
    approved_wrapper_scopes: Vec<String>,
    capacity_witnesses: Vec<String>,
    backpressure_behaviors: Vec<String>,
}

impl BoundedQueueObligations {
    fn from_policy(policy: &PolicyEntry) -> Self {
        let bounded_mechanism =
            policy.mechanisms.iter().find(|mechanism| mechanism.kind == MechanismKind::BoundedPrimitive);
        Self {
            forbidden_apis: mechanism_field_or_fallback(
                bounded_mechanism,
                |mechanism| &mechanism.forbidden_apis,
                FALLBACK_FORBIDDEN_APIS,
            ),
            approved_bounded_apis: mechanism_field_or_fallback(
                bounded_mechanism,
                |mechanism| &mechanism.approved_bounded_apis,
                FALLBACK_APPROVED_BOUNDED_APIS,
            ),
            approved_wrapper_scopes: bounded_mechanism
                .map_or_else(Vec::new, |mechanism| mechanism.approved_wrapper_scopes.clone()),
            capacity_witnesses: bounded_mechanism
                .map_or_else(Vec::new, |mechanism| mechanism.capacity_witnesses.clone()),
            backpressure_behaviors: bounded_mechanism
                .map_or_else(Vec::new, |mechanism| mechanism.backpressure_behaviors.clone()),
        }
    }
}

fn mechanism_field_or_fallback(
    mechanism: Option<&Mechanism>,
    field: impl FnOnce(&Mechanism) -> &Vec<String>,
    fallback: &[&str],
) -> Vec<String> {
    mechanism.map_or_else(
        || fallback.iter().map(|value| (*value).to_owned()).collect(),
        |mechanism| {
            let values = field(mechanism);
            if values.is_empty() {
                fallback.iter().map(|value| (*value).to_owned()).collect()
            } else {
                values.clone()
            }
        },
    )
}

fn append_source_bounded_queue_diagnostics(
    policy: &PolicyEntry,
    obligations: &BoundedQueueObligations,
    source: &SourceFile,
    diagnostics: &mut Vec<Diagnostic>,
) {
    for (line_index, line) in source.text.lines().enumerate() {
        if let Some(api) = matching_pattern(line, &obligations.forbidden_apis) {
            diagnostics.push(forbidden_api_diagnostic(policy, source, line_index, api));
            continue;
        }
        if let Some(api) = bounded_api_missing_capacity(line, obligations) {
            diagnostics.push(missing_witness_diagnostic(policy, source, line_index, api, "capacity witness"));
            continue;
        }
        if let Some(wrapper) = wrapper_missing_witness(line, obligations) {
            diagnostics.push(missing_witness_diagnostic(
                policy,
                source,
                line_index,
                wrapper,
                "capacity or backpressure witness",
            ));
        }
    }
}

fn bounded_api_missing_capacity<'a>(line: &str, obligations: &'a BoundedQueueObligations) -> Option<&'a str> {
    matching_pattern(line, &obligations.approved_bounded_apis).and_then(|api| {
        if has_capacity_argument_after_pattern(line, api) {
            None
        } else {
            Some(api)
        }
    })
}

fn wrapper_missing_witness<'a>(line: &str, obligations: &'a BoundedQueueObligations) -> Option<&'a str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
        return None;
    }
    matching_pattern(line, &obligations.approved_wrapper_scopes).and_then(|wrapper| {
        let has_capacity = contains_any(line, &obligations.capacity_witnesses);
        let has_backpressure = contains_any(line, &obligations.backpressure_behaviors);
        if has_capacity && has_backpressure {
            None
        } else {
            Some(wrapper)
        }
    })
}

fn matching_pattern<'a>(line: &str, patterns: &'a [String]) -> Option<&'a str> {
    patterns.iter().map(String::as_str).find(|pattern| line.contains(pattern))
}

fn contains_any(line: &str, patterns: &[String]) -> bool {
    patterns.iter().any(|pattern| line.contains(pattern))
}

fn has_capacity_argument_after_pattern(line: &str, pattern: &str) -> bool {
    let Some(pattern_start) = line.find(pattern) else {
        return false;
    };
    let Some(open_paren_offset) = line[pattern_start..].find('(') else {
        return false;
    };
    let argument_start = pattern_start + open_paren_offset + 1;
    let Some(close_paren_offset) = line[argument_start..].find(')') else {
        return true;
    };
    !line[argument_start..argument_start + close_paren_offset].trim().is_empty()
}

fn forbidden_api_diagnostic(policy: &PolicyEntry, source: &SourceFile, line_index: usize, api: &str) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: source.path.clone(),
        line: Some(line_number_from_index(line_index)),
        severity: policy.default_severity,
        message: format!("structural bounded primitive violation: forbidden unbounded API `{api}` is not allowed"),
        remediation: "use an approved bounded queue/channel with explicit capacity and backpressure behavior"
            .to_owned(),
    })
}

fn missing_witness_diagnostic(
    policy: &PolicyEntry,
    source: &SourceFile,
    line_index: usize,
    construct: &str,
    missing: &str,
) -> Diagnostic {
    Diagnostic::new(DiagnosticInput {
        policy_id: policy.policy_id.clone(),
        source_section: policy.source_anchor.clone(),
        path: source.path.clone(),
        line: Some(line_number_from_index(line_index)),
        severity: policy.default_severity,
        message: format!("structural bounded primitive violation: `{construct}` is missing {missing}"),
        remediation: "declare an explicit capacity witness and backpressure or overflow behavior for the bounded queue"
            .to_owned(),
    })
}

fn line_number_from_index(index: usize) -> u32 {
    u32::try_from(index.saturating_add(FIRST_LINE_NUMBER_OFFSET)).unwrap_or(u32::MAX)
}
