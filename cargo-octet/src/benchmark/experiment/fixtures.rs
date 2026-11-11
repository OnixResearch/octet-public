use super::evaluate::scope_identity;
use super::model::AdoptionObservation;
use super::model::AdoptionPhase;
use super::model::Availability;
use super::model::CacheState;
use super::model::CompileStatus;
use super::model::ExperimentCase;
use super::model::ExperimentObservations;
use super::model::ExperimentProfile;
use super::model::MeasurementOrigin;
use super::model::OBSERVATIONS_SCHEMA;
use super::model::ObservationEnvironment;
use super::model::ObservationScope;
use super::model::ObservationStatus;
use super::model::ObservedFinding;
use super::model::ObservedSuppression;
use super::model::ToolAvailability;
use super::model::ToolObservation;
use super::model::ToolPin;
use super::model::WarmState;
use crate::benchmark::prefixed_identity;

const TARGET: &str = "x86_64-unknown-linux-gnu";

type RecordedOutcome = (ObservationStatus, CompileStatus, bool, bool, bool, Option<&'static str>);

pub(super) fn recorded_observations(profile: &ExperimentProfile) -> ExperimentObservations {
    let tool_availability = profile.tools.iter().map(tool_availability).collect::<Vec<_>>();
    let mut observations = Vec::new();
    for tool in &profile.tools {
        for entry in profile.applicability.iter().filter(|entry| entry.tool_id == tool.tool_id) {
            if entry.status == super::model::ApplicabilityStatus::Unsupported {
                continue;
            }
            for case_id in &entry.case_ids {
                let Some(case) = profile.corpus.iter().find(|case| case.case_id == *case_id) else {
                    continue;
                };
                if let Some(observation) = recorded_case(tool, case) {
                    observations.push(observation);
                }
            }
        }
    }
    let mut adoption = Vec::new();
    for plan in &profile.adoption.tools {
        let tool = profile.tools.iter().find(|tool| tool.tool_id == plan.tool_id).expect("profile tool");
        for (selection, phase) in [
            (&profile.adoption.clean_consumer.selection_id, AdoptionPhase::Initial),
            (&profile.adoption.clean_consumer.selection_id, AdoptionPhase::Incremental),
            (&profile.adoption.real_consumer.selection_id, AdoptionPhase::Initial),
            (&profile.adoption.real_consumer.selection_id, AdoptionPhase::Incremental),
        ] {
            adoption.push(recorded_adoption(profile, tool, selection, phase));
        }
    }
    ExperimentObservations {
        schema_version: OBSERVATIONS_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        tool_availability,
        observations,
        adoption,
    }
}

fn tool_availability(tool: &ToolPin) -> ToolAvailability {
    let unavailable = tool.tool_id == "archunit-rust"
        || tool.tool_id == "semgrep"
        || tool.tool_id == "codeql"
        || tool.tool_id == "cargo-deny"
        || tool.tool_id == "cargo-vet";
    if unavailable {
        return ToolAvailability {
            tool_id: tool.tool_id.clone(),
            status: Availability::Unavailable,
            version_output: String::new(),
            version_identity: prefixed_identity(b""),
            reason: Some(format!(
                "`{}` is unavailable or unusable in the recorded fixture environment",
                tool.executable
            )),
        };
    }
    let version_output = format!("{} {} (recorded fixture)", tool.tool_id, tool.version_pin);
    ToolAvailability {
        tool_id: tool.tool_id.clone(),
        status: Availability::Recorded,
        version_identity: prefixed_identity(version_output.as_bytes()),
        version_output,
        reason: None,
    }
}

fn recorded_case(tool: &ToolPin, case: &ExperimentCase) -> Option<ToolObservation> {
    let tool_availability = tool_availability(tool);
    if tool_availability.status == Availability::Unavailable {
        return Some(unavailable_observation(tool, case, &tool_availability));
    }
    let (status, compile_status, finding, suppressed_finding, suppression, failure) = outcome_for(tool, case)?;
    let mut findings = Vec::new();
    if finding {
        findings.push(ObservedFinding {
            case_id: case.case_id.clone(),
            rule_id: format!("{}.architecture.fixture-rule", tool.tool_id),
            path: case.source_path.clone(),
            line: 1,
            message: format!("recorded fixture finding for {}", case.case_id),
            suppressed: suppressed_finding,
        });
    }
    let suppressions = if suppression {
        vec![ObservedSuppression {
            case_id: case.case_id.clone(),
            rule_id: format!("{}.architecture.suppressed-note", tool.tool_id),
            reason: "recorded fixture suppression retained for review".to_owned(),
        }]
    } else {
        Vec::new()
    };
    let stdout = format!("{} {} {}", tool.tool_id, case.case_id, status_label(&status));
    let scope = scope_for(case);
    Some(ToolObservation {
        observation_id: format!("{}:{}:{}", tool.tool_id, case.task_id, case.case_id),
        tool_id: tool.tool_id.clone(),
        task_id: case.task_id.clone(),
        case_id: Some(case.case_id.clone()),
        source_identity: case.source_identity.clone(),
        tool_version: tool.version_pin.clone(),
        origin: MeasurementOrigin::RecordedFixture,
        environment: recorded_environment(),
        scope,
        status,
        compile_status,
        findings,
        suppressions,
        stdout_identity: prefixed_identity(stdout.as_bytes()),
        stderr_identity: prefixed_identity(b""),
        stdout,
        stderr: String::new(),
        exit_code: Some(0),
        wall_duration_ns: Some(1_000),
        warm_state: WarmState::Cold,
        sample_index: 0,
        failure_reason: failure.map(str::to_owned),
        unavailable_reason: None,
    })
}

fn outcome_for(tool: &ToolPin, case: &ExperimentCase) -> Option<RecordedOutcome> {
    let completed = |finding, suppressed, suppression| {
        Some((ObservationStatus::Completed, CompileStatus::Compiled, finding, suppressed, suppression, None))
    };
    match tool.tool_id.as_str() {
        "octet" => match case.case_id.as_str() {
            "dependency-boundary-compliant"
            | "reexport-alias-compliant"
            | "macro-expansion-compliant"
            | "trait-method-compliant"
            | "workspace-target-compliant"
            | "valid-evidence-control" => completed(false, false, false),
            "dependency-boundary-violating"
            | "reexport-alias-violating"
            | "trait-method-violating"
            | "workspace-target-violating"
            | "cfg-disputed"
            | "stale-evidence"
            | "changed-policy"
            | "missing-execution"
            | "invalid-proof-source-linkage" => completed(true, false, false),
            "macro-expansion-violating" => completed(false, false, false),
            "cfg-compliant" => completed(true, false, false),
            "cfg-violating" => Some((
                ObservationStatus::TimedOut,
                CompileStatus::Unknown,
                false,
                false,
                false,
                Some("recorded fixture timeout"),
            )),
            "empty-selected-scope" => {
                Some((ObservationStatus::Completed, CompileStatus::NotApplicable, false, false, false, None))
            }
            _ => None,
        },
        "cargo-pup" | "archunit-rust" => match case.case_id.as_str() {
            "dependency-boundary-compliant"
            | "reexport-alias-compliant"
            | "trait-method-compliant"
            | "workspace-target-compliant"
            | "empty-selected-scope" => completed(false, false, false),
            "dependency-boundary-violating"
            | "reexport-alias-violating"
            | "trait-method-violating"
            | "workspace-target-violating" => completed(true, false, false),
            _ => None,
        },
        "semgrep" | "codeql" => match case.case_id.as_str() {
            "dependency-boundary-compliant" | "reexport-alias-compliant" => completed(false, false, false),
            "dependency-boundary-violating" | "reexport-alias-violating" => completed(true, false, false),
            _ => None,
        },
        "repository-baseline-script" => match case.case_id.as_str() {
            "dependency-boundary-compliant" | "reexport-alias-compliant" | "valid-evidence-control" => {
                completed(false, false, false)
            }
            "dependency-boundary-violating" | "reexport-alias-violating" | "stale-evidence" | "changed-policy" => {
                completed(true, false, false)
            }
            _ => None,
        },
        _ => None,
    }
}

fn unavailable_observation(tool: &ToolPin, case: &ExperimentCase, availability: &ToolAvailability) -> ToolObservation {
    let stdout = String::new();
    ToolObservation {
        observation_id: format!("{}:{}:{}", tool.tool_id, case.task_id, case.case_id),
        tool_id: tool.tool_id.clone(),
        task_id: case.task_id.clone(),
        case_id: Some(case.case_id.clone()),
        source_identity: case.source_identity.clone(),
        tool_version: tool.version_pin.clone(),
        origin: MeasurementOrigin::RecordedFixture,
        environment: recorded_environment(),
        scope: scope_for(case),
        status: ObservationStatus::Unavailable,
        compile_status: CompileStatus::Unknown,
        findings: Vec::new(),
        suppressions: Vec::new(),
        stdout_identity: prefixed_identity(stdout.as_bytes()),
        stderr_identity: prefixed_identity(b""),
        stdout,
        stderr: String::new(),
        exit_code: None,
        wall_duration_ns: None,
        warm_state: WarmState::Cold,
        sample_index: 0,
        failure_reason: None,
        unavailable_reason: availability.reason.clone(),
    }
}

fn recorded_adoption(
    profile: &ExperimentProfile,
    tool: &ToolPin,
    selection: &str,
    phase: AdoptionPhase,
) -> AdoptionObservation {
    let phase_name = match phase {
        AdoptionPhase::Initial => "initial",
        AdoptionPhase::Incremental => "incremental",
    };
    let reason = if selection == profile.adoption.real_consumer.selection_id {
        "immutable real-consumer selection is not checked out in the recorded fixture environment"
    } else if tool.tool_id == "clippy" {
        "recorded fixture row; execute mode replaces it with a local clean-consumer timing when clippy is available"
    } else if tool_availability(tool).status == Availability::Unavailable {
        "executable is unavailable in the recorded fixture environment"
    } else {
        "no reviewed adoption command; probe-only recorded tool"
    };
    AdoptionObservation {
        observation_id: format!("{}:{selection}:{phase_name}", tool.tool_id),
        tool_id: tool.tool_id.clone(),
        target_selection: selection.to_owned(),
        phase,
        origin: MeasurementOrigin::RecordedFixture,
        measured: false,
        wall_duration_ns: None,
        warm_state: match phase {
            AdoptionPhase::Initial => WarmState::Cold,
            AdoptionPhase::Incremental => WarmState::Warm,
        },
        environment: recorded_environment_for(match phase {
            AdoptionPhase::Initial => CacheState::Cold,
            AdoptionPhase::Incremental => CacheState::Warm,
        }),
        stdout: String::new(),
        stderr: String::new(),
        stdout_identity: prefixed_identity(b""),
        stderr_identity: prefixed_identity(b""),
        exit_code: None,
        unavailable_reason: Some(reason.to_owned()),
        dependencies: profile.sample_plan.dependency_differences.clone(),
    }
}

fn recorded_environment() -> ObservationEnvironment {
    recorded_environment_for(CacheState::Cold)
}

fn recorded_environment_for(cache_state: CacheState) -> ObservationEnvironment {
    ObservationEnvironment {
        target: TARGET.to_owned(),
        host_os: "linux".to_owned(),
        host_arch: "x86_64".to_owned(),
        rust_toolchain: "nightly-2026-03-21".to_owned(),
        cache_state,
    }
}

fn scope_for(case: &ExperimentCase) -> ObservationScope {
    let selected_sources = case.scope.selected_sources.clone();
    let analyzed_sources = if case.scope.empty {
        Vec::new()
    } else {
        selected_sources.clone()
    };
    let scope = ObservationScope {
        selected_sources,
        analyzed_sources,
        scope_identity: String::new(),
    };
    ObservationScope {
        scope_identity: scope_identity(&scope),
        ..scope
    }
}

fn status_label(status: &ObservationStatus) -> &'static str {
    match status {
        ObservationStatus::Completed => "completed",
        ObservationStatus::TimedOut => "timed_out",
        ObservationStatus::Failed => "failed",
        ObservationStatus::Unavailable => "unavailable",
        ObservationStatus::Unsupported => "unsupported",
        ObservationStatus::NotRun => "not_run",
        ObservationStatus::Unparsed => "unparsed",
    }
}
