use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::model::ARTIFACT_SCHEMA;
use super::model::AdoptionPhase;
use super::model::AdoptionSummary;
use super::model::Applicability;
use super::model::ApplicabilityStatus;
use super::model::Availability;
use super::model::CacheState;
use super::model::Category;
use super::model::CompileStatus;
use super::model::ControlOutcome;
use super::model::DerivedProjection;
use super::model::ExpectedOutcome;
use super::model::ExperimentArtifact;
use super::model::ExperimentCase;
use super::model::ExperimentEvaluation;
use super::model::ExperimentObservations;
use super::model::ExperimentProfile;
use super::model::NegativeControlRecord;
use super::model::OBSERVATIONS_SCHEMA;
use super::model::ObservationScope;
use super::model::ObservationStatus;
use super::model::ObservedFinding;
use super::model::PROFILE_ID;
use super::model::ProvenanceSummary;
use super::model::REQUIRED_NON_CLAIM;
use super::model::ToolObservation;
use super::model::ToolPin;
use super::model::ToolTaskMetrics;
use super::profile::PROFILE_IDENTITY;
use super::profile::selected_profile;
use crate::benchmark::prefixed_identity;
use crate::benchmark::pretty_bytes;

const BASIS_POINTS_SCALE: u128 = 10_000;
const REQUIRED_CONTROL_IDS: [&str; 8] = [
    "noncompiling-violation-counted-as-detection",
    "renamed-stale-oracle",
    "differing-scope-hidden",
    "dropped-timeouts",
    "stale-results",
    "hidden-toolchain-cache-difference",
    "unsupported-counted-clean",
    "bounded-corpus-presented-as-complete",
];

pub fn evaluate(profile_id: &str, observations: &ExperimentObservations) -> ExperimentEvaluation {
    let profile = match selected_profile(profile_id) {
        Ok(profile) => profile,
        Err(error) => return failed(vec![error]),
    };
    let mut diagnostics = validate_profile(&profile);
    if observations.schema_version != OBSERVATIONS_SCHEMA || observations.profile_id != profile.profile_id {
        diagnostics.push("experiment observations schema or profile is unsupported".to_owned());
    }
    diagnostics.extend(validate_observations(&profile, observations));
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return failed(diagnostics);
    }
    let mut artifact = match derive_artifact(&profile, observations) {
        Ok(artifact) => artifact,
        Err(errors) => return failed(errors),
    };
    let controls = run_negative_controls(&profile, observations, &artifact);
    let escaped: Vec<&str> = controls
        .iter()
        .filter(|record| record.outcome != ControlOutcome::Contained)
        .map(|record| record.control_id.as_str())
        .collect();
    if !escaped.is_empty() {
        return failed(escaped.into_iter().map(|id| format!("experiment negative control `{id}` escaped")).collect());
    }
    if let Ok(bytes) = pretty_bytes(&controls, "experiment negative controls") {
        artifact.controls_identity = prefixed_identity(&bytes);
    } else {
        return failed(vec!["experiment negative controls could not be serialized".to_owned()]);
    }
    artifact.controls = controls;
    let diagnostics = verify_artifact(&profile, &artifact);
    if diagnostics.is_empty() {
        ExperimentEvaluation {
            artifact: Some(artifact),
            diagnostics: Vec::new(),
        }
    } else {
        failed(diagnostics)
    }
}

pub fn verify(profile_id: &str, artifact: &ExperimentArtifact) -> Vec<String> {
    let profile = match selected_profile(profile_id) {
        Ok(profile) => profile,
        Err(error) => return vec![error],
    };
    let mut diagnostics = verify_artifact(&profile, artifact);
    let observations = ExperimentObservations {
        schema_version: OBSERVATIONS_SCHEMA.to_owned(),
        profile_id: artifact.profile.profile_id.clone(),
        tool_availability: artifact.tool_availability.clone(),
        observations: artifact.observations.clone(),
        adoption: artifact.adoption.clone(),
    };
    let controls = run_negative_controls(&profile, &observations, artifact);
    for record in &controls {
        if record.outcome != ControlOutcome::Contained {
            diagnostics.push(format!("experiment negative control `{}` escaped", record.control_id));
        }
    }
    if controls != artifact.controls {
        diagnostics.push("experiment negative-control records do not reproduce".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn derive_artifact(
    profile: &ExperimentProfile,
    observations: &ExperimentObservations,
) -> Result<ExperimentArtifact, Vec<String>> {
    let mut diagnostics = validate_profile(profile);
    diagnostics.extend(validate_observations(profile, observations));
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let derived = derive_projection(profile, observations)?;
    let observations_identity = identity_of(observations, "experiment observations").map_err(|error| vec![error])?;
    let adoption_identity =
        identity_of(&observations.adoption, "experiment adoption observations").map_err(|error| vec![error])?;
    let derived_identity = identity_of(&derived, "experiment derived projection").map_err(|error| vec![error])?;
    Ok(ExperimentArtifact {
        schema_version: ARTIFACT_SCHEMA.to_owned(),
        profile_identity: PROFILE_IDENTITY.to_owned(),
        profile: profile.clone(),
        tool_availability: observations.tool_availability.clone(),
        observations_identity,
        observations: observations.observations.clone(),
        adoption_identity,
        adoption: observations.adoption.clone(),
        derived,
        derived_identity,
        controls: Vec::new(),
        controls_identity: identity_of(&Vec::<NegativeControlRecord>::new(), "experiment negative controls")
            .map_err(|error| vec![error])?,
    })
}

pub(crate) fn verify_artifact(profile: &ExperimentProfile, artifact: &ExperimentArtifact) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if artifact.schema_version != ARTIFACT_SCHEMA {
        diagnostics.push("experiment artifact schema is unsupported".to_owned());
    }
    if artifact.profile_identity != PROFILE_IDENTITY {
        diagnostics.push("experiment artifact profile identity is stale".to_owned());
    }
    if artifact.profile != *profile {
        diagnostics.push("experiment artifact profile does not match the embedded profile".to_owned());
    }
    if artifact.profile.profile_id != PROFILE_ID
        || artifact.profile.coverage_boundary.complete_rust_coverage
        || !artifact.profile.coverage_boundary.bounded
    {
        diagnostics.push("experiment artifact presents the bounded corpus as complete Rust coverage".to_owned());
    }
    let observations = ExperimentObservations {
        schema_version: OBSERVATIONS_SCHEMA.to_owned(),
        profile_id: artifact.profile.profile_id.clone(),
        tool_availability: artifact.tool_availability.clone(),
        observations: artifact.observations.clone(),
        adoption: artifact.adoption.clone(),
    };
    diagnostics.extend(validate_observations(profile, &observations));
    match identity_of(&observations, "experiment observations") {
        Ok(identity) if identity == artifact.observations_identity => {}
        Ok(_) => diagnostics.push("experiment observation identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    match identity_of(&artifact.adoption, "experiment adoption observations") {
        Ok(identity) if identity == artifact.adoption_identity => {}
        Ok(_) => diagnostics.push("experiment adoption identity is stale".to_owned()),
        Err(error) => diagnostics.push(error),
    }
    match derive_projection(profile, &observations) {
        Ok(derived) => {
            if derived != artifact.derived {
                diagnostics.push("experiment derived comparison does not reproduce from retained inputs".to_owned());
            }
            match identity_of(&derived, "experiment derived projection") {
                Ok(identity) if identity == artifact.derived_identity => {}
                Ok(_) => diagnostics.push("experiment derived projection identity is stale".to_owned()),
                Err(error) => diagnostics.push(error),
            }
        }
        Err(errors) => diagnostics.extend(errors),
    }
    if let Ok(bytes) = pretty_bytes(&artifact.controls, "experiment negative controls")
        && prefixed_identity(&bytes) != artifact.controls_identity
    {
        diagnostics.push("experiment negative-control identity is stale".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

pub(crate) fn validate_profile(profile: &ExperimentProfile) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if profile.profile_id != PROFILE_ID
        || profile.source_revision.is_empty()
        || profile.rust_toolchain.is_empty()
        || profile.evidence_tier != "transcript_assertion"
        || !profile.declared_before_measurement
    {
        diagnostics.push(
            "experiment profile identity, revision, toolchain, or declaration boundary is unsupported".to_owned(),
        );
    }
    if !profile.non_claims.iter().any(|claim| claim == REQUIRED_NON_CLAIM) {
        diagnostics.push("experiment profile is missing the required non-claim".to_owned());
    }
    if !profile.coverage_boundary.bounded
        || profile.coverage_boundary.complete_rust_coverage
        || profile.coverage_boundary.statement.is_empty()
    {
        diagnostics.push("experiment coverage boundary must stay bounded and incomplete".to_owned());
    }
    validate_cases(profile, &mut diagnostics);
    validate_tasks(profile, &mut diagnostics);
    validate_tools(profile, &mut diagnostics);
    validate_applicability(profile, &mut diagnostics);
    validate_adoption_plan(profile, &mut diagnostics);
    validate_sample_plan(profile, &mut diagnostics);
    validate_controls(profile, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_cases(profile: &ExperimentProfile, diagnostics: &mut Vec<String>) {
    let mut case_ids = BTreeSet::new();
    let tasks = profile.tasks.iter().map(|task| (task.task_id.as_str(), task)).collect::<BTreeMap<_, _>>();
    for case in &profile.corpus {
        if case.case_id.is_empty()
            || !case_ids.insert(case.case_id.as_str())
            || case.source_path.is_empty()
            || case.rationale.is_empty()
            || case.reviewer_role.is_empty()
            || !is_b3(&case.source_identity)
        {
            diagnostics.push("experiment corpus contains an incomplete or duplicate case".to_owned());
        }
        let Some(task) = tasks.get(case.task_id.as_str()) else {
            diagnostics.push(format!("experiment case `{}` names an unknown task", case.case_id));
            continue;
        };
        if task.category != case.category {
            diagnostics.push(format!("experiment case `{}` category differs from its task", case.case_id));
        }
        if case.scope.empty != (case.scope.selected_sources.is_empty() && case.scope.expected_analyzed_sources == 0) {
            diagnostics.push(format!("experiment case `{}` has an inconsistent empty-scope declaration", case.case_id));
        }
    }
    for task in &profile.tasks {
        if !profile.corpus.iter().any(|case| case.task_id == task.task_id) {
            diagnostics.push(format!("experiment task `{}` has no corpus cases", task.task_id));
        }
    }
}

fn validate_tasks(profile: &ExperimentProfile, diagnostics: &mut Vec<String>) {
    let mut task_ids = BTreeSet::new();
    for task in &profile.tasks {
        if task.task_id.is_empty()
            || task.description.is_empty()
            || task.comparable_subset.is_empty()
            || task.required_scope.is_empty()
            || !task_ids.insert(task.task_id.as_str())
        {
            diagnostics.push("experiment task is incomplete or duplicated".to_owned());
        }
    }
    for category in [Category::Architecture, Category::Assurance] {
        if !profile.tasks.iter().any(|task| task.category == category) {
            diagnostics.push("experiment profile is missing a required task category".to_owned());
        }
    }
}

fn validate_tools(profile: &ExperimentProfile, diagnostics: &mut Vec<String>) {
    let mut tool_ids = BTreeSet::new();
    for tool in &profile.tools {
        if tool.tool_id.is_empty()
            || tool.executable.is_empty()
            || tool.version_pin.is_empty()
            || tool.source_pin.is_empty()
            || tool.rules_pin.is_empty()
            || tool.setup_source.is_empty()
            || tool.setup_steps.is_empty()
            || tool.probe_argv.is_empty()
            || tool.claims.is_empty()
            || !tool_ids.insert(tool.tool_id.as_str())
        {
            diagnostics.push("experiment tool pin is incomplete or duplicated".to_owned());
        }
        if tool.config_path.len() != tool.config_identity.len()
            || tool.config_identity.iter().any(|identity| !is_b3(identity))
        {
            diagnostics.push(format!("experiment tool `{}` config pin is inconsistent", tool.tool_id));
        }
        if tool.configuration_status == super::model::ConfigurationStatus::CheckedIn
            && (tool.config_path.is_empty() || tool.config_identity.is_empty())
        {
            diagnostics.push(format!("experiment tool `{}` is missing a checked-in config pin", tool.tool_id));
        }
    }
}

fn validate_applicability(profile: &ExperimentProfile, diagnostics: &mut Vec<String>) {
    let case_map = super::model::case_map(profile);
    let task_map = super::model::task_map(profile);
    let tool_map = super::model::tool_map(profile);
    let mut seen = BTreeSet::new();
    for entry in &profile.applicability {
        if !seen.insert((entry.tool_id.as_str(), entry.task_id.as_str())) {
            diagnostics.push("experiment applicability matrix contains a duplicate cell".to_owned());
        }
        let Some(task) = task_map.get(entry.task_id.as_str()) else {
            diagnostics.push(format!("experiment applicability names unknown task `{}`", entry.task_id));
            continue;
        };
        if !tool_map.contains_key(entry.tool_id.as_str()) {
            diagnostics.push(format!("experiment applicability names unknown tool `{}`", entry.tool_id));
        }
        if entry.comparable_subset != task.comparable_subset || entry.scope != task.required_scope {
            diagnostics.push(format!(
                "experiment applicability for `{}`/`{}` does not bind the task subset or scope",
                entry.tool_id, entry.task_id
            ));
        }
        match entry.status {
            ApplicabilityStatus::Unsupported => {
                if !entry.case_ids.is_empty() {
                    diagnostics.push(format!(
                        "experiment applicability for unsupported `{}`/`{}` still claims cases",
                        entry.tool_id, entry.task_id
                    ));
                }
            }
            ApplicabilityStatus::Supported | ApplicabilityStatus::Partial => {
                if entry.case_ids.is_empty() {
                    diagnostics.push(format!(
                        "experiment applicability for `{}`/`{}` claims no cases",
                        entry.tool_id, entry.task_id
                    ));
                }
                for case_id in &entry.case_ids {
                    match case_map.get(case_id.as_str()) {
                        Some(case) if case.task_id == entry.task_id => {}
                        Some(_) => diagnostics.push(format!(
                            "experiment applicability for `{}`/`{}` claims case `{case_id}` from another task",
                            entry.tool_id, entry.task_id
                        )),
                        None => diagnostics.push(format!(
                            "experiment applicability for `{}`/`{}` claims unknown case `{case_id}`",
                            entry.tool_id, entry.task_id
                        )),
                    }
                }
            }
        }
    }
    for tool in &profile.tools {
        for task in &profile.tasks {
            if !seen.contains(&(tool.tool_id.as_str(), task.task_id.as_str())) {
                diagnostics
                    .push(format!("experiment applicability matrix is missing `{}`/`{}`", tool.tool_id, task.task_id));
            }
        }
    }
}

fn validate_adoption_plan(profile: &ExperimentProfile, diagnostics: &mut Vec<String>) {
    let clean = &profile.adoption.clean_consumer;
    let real = &profile.adoption.real_consumer;
    if clean.selection_id.is_empty()
        || clean.source_repository.is_empty()
        || clean.source_revision.is_empty()
        || clean.workspace_path.is_empty()
        || !clean.immutable
    {
        diagnostics.push("experiment clean-consumer adoption target is incomplete".to_owned());
    }
    if real.selection_id.is_empty()
        || real.selection_id == clean.selection_id
        || real.source_repository.is_empty()
        || real.source_revision.is_empty()
        || !real.immutable
    {
        diagnostics.push("experiment real-consumer adoption target must be a distinct immutable selection".to_owned());
    }
    let mut planned = BTreeSet::new();
    for plan in &profile.adoption.tools {
        if plan.tool_id.is_empty()
            || plan.setup_steps.is_empty()
            || plan.initial_command.is_empty()
            || plan.incremental_command.is_empty()
            || !planned.insert(plan.tool_id.as_str())
        {
            diagnostics.push("experiment adoption plan is incomplete or duplicated".to_owned());
        }
    }
    for tool in &profile.tools {
        if !planned.contains(tool.tool_id.as_str()) {
            diagnostics.push(format!("experiment adoption plan is missing tool `{}`", tool.tool_id));
        }
    }
}

fn validate_sample_plan(profile: &ExperimentProfile, diagnostics: &mut Vec<String>) {
    if profile.sample_plan.sample_count == 0
        || profile.sample_plan.timeout_ms == 0
        || profile.sample_plan.target.is_empty()
        || profile.sample_plan.warm_state != super::model::WarmState::Cold
        || profile.sample_plan.cache_state != CacheState::Cold
    {
        diagnostics.push("experiment sample plan is incomplete or does not declare a cold run".to_owned());
    }
}

fn validate_controls(profile: &ExperimentProfile, diagnostics: &mut Vec<String>) {
    let mut ids = BTreeSet::new();
    for control in &profile.negative_controls {
        if control.control_id.is_empty() || control.description.is_empty() || !ids.insert(control.control_id.as_str()) {
            diagnostics.push("experiment negative-control declaration is incomplete or duplicated".to_owned());
        }
    }
    for required in REQUIRED_CONTROL_IDS {
        if !ids.contains(required) {
            diagnostics.push(format!("experiment profile is missing negative control `{required}`"));
        }
    }
}

pub(crate) fn validate_observations(profile: &ExperimentProfile, observations: &ExperimentObservations) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if observations.schema_version != OBSERVATIONS_SCHEMA || observations.profile_id != profile.profile_id {
        diagnostics.push("experiment observations schema or profile is unsupported".to_owned());
    }
    let tool_map = super::model::tool_map(profile);
    let task_map = super::model::task_map(profile);
    let case_map = super::model::case_map(profile);
    let applicability = super::model::applicability_map(profile);
    let mut availability = BTreeMap::new();
    for record in &observations.tool_availability {
        if record.tool_id.is_empty()
            || !tool_map.contains_key(record.tool_id.as_str())
            || availability.insert(record.tool_id.as_str(), record).is_some()
        {
            diagnostics.push("experiment tool availability is incomplete or duplicated".to_owned());
            continue;
        }
        if !is_b3(&record.version_identity) {
            diagnostics.push(format!("experiment tool `{}` version identity is malformed", record.tool_id));
        }
        if record.status == Availability::Unavailable {
            if record.reason.as_deref().unwrap_or_default().is_empty() {
                diagnostics.push(format!("experiment unavailable tool `{}` has no reason", record.tool_id));
            }
        } else if record.version_output.is_empty() {
            diagnostics.push(format!("experiment available tool `{}` has no version output", record.tool_id));
        }
    }
    for tool in &profile.tools {
        if !availability.contains_key(tool.tool_id.as_str()) {
            diagnostics.push(format!("experiment tool availability is missing `{}`", tool.tool_id));
        }
    }

    let mut observation_ids = BTreeSet::new();
    let mut observation_keys = BTreeSet::new();
    for observation in &observations.observations {
        if observation.observation_id.is_empty()
            || !observation_ids.insert(observation.observation_id.as_str())
            || !observation_keys.insert((
                observation.tool_id.as_str(),
                observation.task_id.as_str(),
                observation.case_id.as_deref(),
            ))
        {
            diagnostics.push("experiment observation is incomplete or duplicated".to_owned());
        }
        if !tool_map.contains_key(observation.tool_id.as_str())
            || !task_map.contains_key(observation.task_id.as_str())
            || !is_b3(&observation.source_identity)
            || observation.tool_version.is_empty()
            || observation.environment.target.is_empty()
            || observation.environment.host_os.is_empty()
            || observation.environment.host_arch.is_empty()
            || observation.environment.rust_toolchain.is_empty()
        {
            diagnostics.push(format!("experiment observation `{}` is incomplete", observation.observation_id));
        }
        validate_observation_payload(observation, &mut diagnostics);
        match observation.case_id.as_deref() {
            Some(case_id) => match case_map.get(case_id) {
                Some(case) => {
                    if case.task_id != observation.task_id || observation.source_identity != case.source_identity {
                        diagnostics.push(format!(
                            "experiment observation `{}` has a stale oracle source or task identity",
                            observation.observation_id
                        ));
                    }
                    if observation.scope.selected_sources != case.scope.selected_sources {
                        diagnostics.push(format!(
                            "experiment observation `{}` analyzed a different scope than the frozen case",
                            observation.observation_id
                        ));
                    }
                    if let Some(entry) =
                        applicability.get(&(observation.tool_id.as_str(), observation.task_id.as_str()))
                    {
                        if entry.status == ApplicabilityStatus::Unsupported {
                            diagnostics.push(format!(
                                "experiment observation `{}` targets an unsupported applicability cell",
                                observation.observation_id
                            ));
                        } else if !entry.case_ids.iter().any(|declared| declared == case_id) {
                            diagnostics.push(format!(
                                "experiment observation `{}` targets a case the tool does not claim",
                                observation.observation_id
                            ));
                        }
                    }
                    if case.expected == ExpectedOutcome::Finding
                        && observation.status == ObservationStatus::Completed
                        && observation.compile_status == CompileStatus::NotCompiled
                        && observation.findings.iter().any(|finding| !finding.suppressed)
                    {
                        diagnostics.push(format!(
                            "experiment observation `{}` counts a noncompiling violation as detection",
                            observation.observation_id
                        ));
                    }
                }
                None => diagnostics
                    .push(format!("experiment observation `{}` names an unknown case", observation.observation_id)),
            },
            None => {
                if !matches!(observation.status, ObservationStatus::Unsupported | ObservationStatus::Unavailable) {
                    diagnostics.push(format!(
                        "experiment tool-level observation `{}` is not unsupported",
                        observation.observation_id
                    ));
                }
            }
        }
    }

    for entry in &profile.applicability {
        if entry.status == ApplicabilityStatus::Unsupported {
            continue;
        }
        for case_id in &entry.case_ids {
            let present = observations.observations.iter().any(|observation| {
                observation.tool_id == entry.tool_id
                    && observation.task_id == entry.task_id
                    && observation.case_id.as_deref() == Some(case_id.as_str())
            });
            if !present {
                diagnostics.push(format!(
                    "experiment observation for `{}`/`{case_id}` is missing (dropped execution or timeout)",
                    entry.tool_id
                ));
            }
        }
    }

    validate_adoption_observations(profile, observations, &mut diagnostics);
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_observation_payload(observation: &ToolObservation, diagnostics: &mut Vec<String>) {
    if observation.scope.scope_identity != scope_identity(&observation.scope) {
        diagnostics.push(format!("experiment observation `{}` scope identity is stale", observation.observation_id));
    }
    if observation.stdout_identity != prefixed_identity(observation.stdout.as_bytes())
        || observation.stderr_identity != prefixed_identity(observation.stderr.as_bytes())
    {
        diagnostics
            .push(format!("experiment observation `{}` raw output identity is stale", observation.observation_id));
    }
    match observation.status {
        ObservationStatus::Completed => {
            if observation.exit_code.is_none() || observation.wall_duration_ns.is_none() {
                diagnostics.push(format!(
                    "experiment completed observation `{}` lacks exit or duration",
                    observation.observation_id
                ));
            }
            if observation.compile_status == CompileStatus::Unknown {
                diagnostics.push(format!(
                    "experiment completed observation `{}` has unknown compile status",
                    observation.observation_id
                ));
            }
        }
        ObservationStatus::TimedOut | ObservationStatus::Failed => {
            if observation.failure_reason.as_deref().unwrap_or_default().is_empty() {
                diagnostics.push(format!(
                    "experiment failed observation `{}` has no failure reason",
                    observation.observation_id
                ));
            }
        }
        ObservationStatus::Unavailable | ObservationStatus::Unsupported => {
            if observation.unavailable_reason.as_deref().unwrap_or_default().is_empty() {
                diagnostics
                    .push(format!("experiment unsupported observation `{}` has no reason", observation.observation_id));
            }
        }
        ObservationStatus::NotRun => {
            if observation.failure_reason.as_deref().unwrap_or_default().is_empty()
                && observation.unavailable_reason.as_deref().unwrap_or_default().is_empty()
            {
                diagnostics
                    .push(format!("experiment not-run observation `{}` has no reason", observation.observation_id));
            }
        }
        ObservationStatus::Unparsed => {
            if observation.unavailable_reason.as_deref().unwrap_or_default().is_empty()
                && observation.failure_reason.as_deref().unwrap_or_default().is_empty()
            {
                diagnostics
                    .push(format!("experiment unparsed observation `{}` has no reason", observation.observation_id));
            }
        }
    }
    for finding in &observation.findings {
        if finding.case_id.is_empty()
            || finding.rule_id.is_empty()
            || finding.path.is_empty()
            || finding.message.is_empty()
            || observation.case_id.as_deref() != Some(finding.case_id.as_str())
        {
            diagnostics.push(format!("experiment observation `{}` has an invalid finding", observation.observation_id));
        }
    }
    for suppression in &observation.suppressions {
        if suppression.case_id.is_empty()
            || suppression.rule_id.is_empty()
            || suppression.reason.is_empty()
            || observation.case_id.as_deref() != Some(suppression.case_id.as_str())
        {
            diagnostics
                .push(format!("experiment observation `{}` has an invalid suppression", observation.observation_id));
        }
    }
}

fn validate_adoption_observations(
    profile: &ExperimentProfile,
    observations: &ExperimentObservations,
    diagnostics: &mut Vec<String>,
) {
    let plans = profile.adoption.tools.iter().map(|plan| (plan.tool_id.as_str(), plan)).collect::<BTreeMap<_, _>>();
    let clean = profile.adoption.clean_consumer.selection_id.as_str();
    let real = profile.adoption.real_consumer.selection_id.as_str();
    let mut keys = BTreeSet::new();
    for observation in &observations.adoption {
        if observation.observation_id.is_empty()
            || !plans.contains_key(observation.tool_id.as_str())
            || observation.target_selection.is_empty()
            || !keys.insert((
                observation.tool_id.as_str(),
                observation.target_selection.as_str(),
                adoption_phase_key(&observation.phase),
            ))
        {
            diagnostics.push("experiment adoption observation is incomplete or duplicated".to_owned());
        }
        if observation.target_selection != clean && observation.target_selection != real {
            diagnostics.push(format!(
                "experiment adoption observation `{}` names an unknown target selection",
                observation.observation_id
            ));
        }
        if observation.stdout_identity != prefixed_identity(observation.stdout.as_bytes())
            || observation.stderr_identity != prefixed_identity(observation.stderr.as_bytes())
        {
            diagnostics.push(format!(
                "experiment adoption observation `{}` raw output identity is stale",
                observation.observation_id
            ));
        }
        if observation.measured {
            if observation.exit_code != Some(0) || observation.wall_duration_ns.is_none_or(|value| value == 0) {
                diagnostics.push(format!(
                    "experiment adoption observation `{}` claims a measured run without a successful timing",
                    observation.observation_id
                ));
            }
        } else if observation.unavailable_reason.as_deref().unwrap_or_default().is_empty()
            && observation.exit_code.is_none_or(|code| code == 0)
        {
            diagnostics.push(format!(
                "experiment adoption observation `{}` is unmeasured without a reason or failing exit",
                observation.observation_id
            ));
        }
        if observation.environment.target.is_empty()
            || observation.environment.rust_toolchain.is_empty()
            || observation.dependencies.is_empty()
        {
            diagnostics.push(format!(
                "experiment adoption observation `{}` lacks environment facts",
                observation.observation_id
            ));
        }
        let availability = observations
            .tool_availability
            .iter()
            .find(|record| record.tool_id == observation.tool_id)
            .map(|record| record.status.clone());
        if availability == Some(Availability::Unavailable) && observation.measured {
            diagnostics.push(format!(
                "experiment adoption observation `{}` claims a measurement for an unavailable tool",
                observation.observation_id
            ));
        }
    }
    for plan in &profile.adoption.tools {
        for target in [clean, real] {
            for phase in [AdoptionPhase::Initial, AdoptionPhase::Incremental] {
                if !keys.contains(&(plan.tool_id.as_str(), target, adoption_phase_key(&phase))) {
                    diagnostics.push(format!(
                        "experiment adoption observation for `{}`/{target}/{} is missing",
                        plan.tool_id,
                        adoption_phase_key(&phase)
                    ));
                }
            }
        }
    }
}

fn adoption_phase_key(phase: &AdoptionPhase) -> &'static str {
    match phase {
        AdoptionPhase::Initial => "initial",
        AdoptionPhase::Incremental => "incremental",
    }
}

fn derive_projection(
    profile: &ExperimentProfile,
    observations: &ExperimentObservations,
) -> Result<DerivedProjection, Vec<String>> {
    let availability = observations
        .tool_availability
        .iter()
        .map(|record| (record.tool_id.as_str(), record.status.clone()))
        .collect::<BTreeMap<_, _>>();
    let observation_map: BTreeMap<(&str, &str, &str), &ToolObservation> = observations
        .observations
        .iter()
        .filter_map(|observation| {
            observation
                .case_id
                .as_deref()
                .map(|case_id| ((observation.tool_id.as_str(), observation.task_id.as_str(), case_id), observation))
        })
        .collect();
    let mut architecture = Vec::new();
    let mut assurance = Vec::new();
    for entry in &profile.applicability {
        let Some(task) = profile.tasks.iter().find(|task| task.task_id == entry.task_id) else {
            return Err(vec![format!(
                "experiment applicability names unknown task `{}`",
                entry.task_id
            )]);
        };
        let Some(tool) = profile.tools.iter().find(|tool| tool.tool_id == entry.tool_id) else {
            return Err(vec![format!(
                "experiment applicability names unknown tool `{}`",
                entry.tool_id
            )]);
        };
        let tool_availability = availability.get(entry.tool_id.as_str()).cloned().unwrap_or(Availability::Unavailable);
        let mut metrics = empty_metrics(entry, task.category.clone(), tool_availability.clone());
        if entry.status == ApplicabilityStatus::Unsupported {
            metrics.unsupported_cases = u64::try_from(task_case_count(profile, &task.task_id)).unwrap_or(u64::MAX);
            push_metrics(&mut architecture, &mut assurance, task.category.clone(), metrics);
            continue;
        }
        for case_id in &entry.case_ids {
            let Some(case) = profile.corpus.iter().find(|case| case.case_id == *case_id) else {
                return Err(vec![format!("experiment applicability names unknown case `{case_id}`")]);
            };
            let observation =
                observation_map.get(&(entry.tool_id.as_str(), entry.task_id.as_str(), case.case_id.as_str()));
            classify_case(profile, tool, case, observation.copied(), &mut metrics);
        }
        let declared = entry.case_ids.iter().collect::<BTreeSet<_>>();
        let unclaimed = profile
            .corpus
            .iter()
            .filter(|case| case.task_id == entry.task_id && !declared.contains(&case.case_id))
            .count();
        metrics.unsupported_cases =
            metrics.unsupported_cases.saturating_add(u64::try_from(unclaimed).unwrap_or(u64::MAX));
        metrics.comparable_cases = metrics
            .true_positive
            .saturating_add(metrics.false_positive)
            .saturating_add(metrics.false_negative)
            .saturating_add(metrics.true_negative);
        metrics.precision_basis_points =
            ratio_basis_points(metrics.true_positive, metrics.true_positive.saturating_add(metrics.false_positive));
        metrics.recall_basis_points =
            ratio_basis_points(metrics.true_positive, metrics.true_positive.saturating_add(metrics.false_negative));
        metrics.reasons.sort();
        metrics.reasons.dedup();
        push_metrics(&mut architecture, &mut assurance, task.category.clone(), metrics);
    }
    let adoption = derive_adoption(profile, observations);
    let provenance = derive_provenance(observations);
    Ok(DerivedProjection {
        architecture,
        assurance,
        adoption,
        provenance,
    })
}

fn push_metrics(
    architecture: &mut Vec<ToolTaskMetrics>,
    assurance: &mut Vec<ToolTaskMetrics>,
    category: Category,
    metrics: ToolTaskMetrics,
) {
    match category {
        Category::Architecture => architecture.push(metrics),
        Category::Assurance => assurance.push(metrics),
    }
}

fn empty_metrics(entry: &Applicability, category: Category, availability: Availability) -> ToolTaskMetrics {
    ToolTaskMetrics {
        category,
        task_id: entry.task_id.clone(),
        tool_id: entry.tool_id.clone(),
        applicability_status: entry.status.clone(),
        comparable_subset: entry.comparable_subset.clone(),
        availability,
        comparable_cases: 0,
        true_positive: 0,
        false_positive: 0,
        false_negative: 0,
        true_negative: 0,
        unsupported_cases: 0,
        inconclusive_cases: 0,
        incomparable_cases: 0,
        suppressed_cases: 0,
        empty_scope_cases: 0,
        disputed_cases: 0,
        missing_execution_cases: 0,
        precision_basis_points: None,
        recall_basis_points: None,
        reasons: Vec::new(),
    }
}

fn classify_case(
    profile: &ExperimentProfile,
    tool: &ToolPin,
    case: &ExperimentCase,
    observation: Option<&ToolObservation>,
    metrics: &mut ToolTaskMetrics,
) {
    match case.expected {
        ExpectedOutcome::Disputed => {
            metrics.disputed_cases = metrics.disputed_cases.saturating_add(1);
            metrics.reasons.push(format!("case `{}` has a disputed expected outcome", case.case_id));
            return;
        }
        ExpectedOutcome::Unsupported => {
            metrics.unsupported_cases = metrics.unsupported_cases.saturating_add(1);
            metrics.reasons.push(format!("case `{}` is declared unsupported", case.case_id));
            return;
        }
        ExpectedOutcome::EmptyScope => {
            match observation {
                Some(observation)
                    if observation.status == ObservationStatus::Completed
                        && observation.scope.selected_sources.is_empty()
                        && observation.scope.analyzed_sources.is_empty() =>
                {
                    metrics.empty_scope_cases = metrics.empty_scope_cases.saturating_add(1);
                    metrics.reasons.push(format!(
                        "case `{}` selected an empty scope; zero findings are not clean evidence",
                        case.case_id
                    ));
                }
                Some(observation)
                    if matches!(
                        observation.status,
                        ObservationStatus::Unavailable | ObservationStatus::Unsupported
                    ) =>
                {
                    metrics.unsupported_cases = metrics.unsupported_cases.saturating_add(1);
                }
                Some(observation) => {
                    metrics.incomparable_cases = metrics.incomparable_cases.saturating_add(1);
                    metrics.reasons.push(format!(
                        "case `{}` expected an empty scope but observation `{}` analyzed another scope",
                        case.case_id, observation.observation_id
                    ));
                }
                None => {
                    metrics.missing_execution_cases = metrics.missing_execution_cases.saturating_add(1);
                }
            }
            return;
        }
        ExpectedOutcome::Finding | ExpectedOutcome::Clean => {}
    }
    let Some(observation) = observation else {
        metrics.missing_execution_cases = metrics.missing_execution_cases.saturating_add(1);
        metrics.reasons.push(format!("case `{}` has no retained observation", case.case_id));
        return;
    };
    match observation.status {
        ObservationStatus::Unavailable | ObservationStatus::Unsupported => {
            metrics.unsupported_cases = metrics.unsupported_cases.saturating_add(1);
            metrics.reasons.push(format!(
                "case `{}` is unsupported for `{}`; absence is not clean evidence",
                case.case_id, tool.tool_id
            ));
            return;
        }
        ObservationStatus::TimedOut
        | ObservationStatus::Failed
        | ObservationStatus::NotRun
        | ObservationStatus::Unparsed => {
            metrics.inconclusive_cases = metrics.inconclusive_cases.saturating_add(1);
            metrics
                .reasons
                .push(format!("case `{}` observation `{}` did not complete", case.case_id, observation.observation_id));
            return;
        }
        ObservationStatus::Completed => {}
    }
    if observation.compile_status != CompileStatus::Compiled {
        metrics.incomparable_cases = metrics.incomparable_cases.saturating_add(1);
        metrics.reasons.push(format!(
            "case `{}` observation `{}` did not compile and cannot be compared",
            case.case_id, observation.observation_id
        ));
        return;
    }
    let mut environment_changed = false;
    if observation.environment.rust_toolchain != profile.rust_toolchain
        || observation.environment.target != profile.sample_plan.target
        || observation.environment.cache_state != profile.sample_plan.cache_state
        || observation.warm_state != profile.sample_plan.warm_state
    {
        environment_changed = true;
    }
    if observation.tool_version != tool.version_pin {
        environment_changed = true;
    }
    if environment_changed {
        metrics.incomparable_cases = metrics.incomparable_cases.saturating_add(1);
        metrics.reasons.push(format!(
            "case `{}` observation `{}` has a differing toolchain, tool, or cache identity",
            case.case_id, observation.observation_id
        ));
        return;
    }
    let scope_matches = observation.scope.selected_sources == case.scope.selected_sources
        && observation.scope.analyzed_sources.len() == case.scope.expected_analyzed_sources;
    if !scope_matches {
        metrics.incomparable_cases = metrics.incomparable_cases.saturating_add(1);
        metrics.reasons.push(format!(
            "case `{}` observation `{}` analyzed a differing scope",
            case.case_id, observation.observation_id
        ));
        return;
    }
    let detected = observation.findings.iter().any(|finding| !finding.suppressed);
    let suppressed =
        observation.findings.iter().any(|finding| finding.suppressed) || !observation.suppressions.is_empty();
    match case.expected {
        ExpectedOutcome::Finding => {
            if detected {
                metrics.true_positive = metrics.true_positive.saturating_add(1);
            } else if suppressed {
                metrics.inconclusive_cases = metrics.inconclusive_cases.saturating_add(1);
                metrics.suppressed_cases = metrics.suppressed_cases.saturating_add(1);
                metrics.reasons.push(format!("case `{}` finding was suppressed", case.case_id));
            } else {
                metrics.false_negative = metrics.false_negative.saturating_add(1);
            }
        }
        ExpectedOutcome::Clean => {
            if detected {
                metrics.false_positive = metrics.false_positive.saturating_add(1);
            } else if suppressed {
                metrics.inconclusive_cases = metrics.inconclusive_cases.saturating_add(1);
                metrics.suppressed_cases = metrics.suppressed_cases.saturating_add(1);
                metrics.reasons.push(format!("case `{}` has an unexpected suppression", case.case_id));
            } else {
                metrics.true_negative = metrics.true_negative.saturating_add(1);
            }
        }
        ExpectedOutcome::EmptyScope | ExpectedOutcome::Disputed | ExpectedOutcome::Unsupported => {}
    }
}

fn ratio_basis_points(numerator: u64, denominator: u64) -> Option<u64> {
    if denominator == 0 {
        return None;
    }
    u128::from(numerator)
        .checked_mul(BASIS_POINTS_SCALE)
        .and_then(|scaled| scaled.checked_div(u128::from(denominator)))
        .and_then(|value| u64::try_from(value).ok())
}

fn task_case_count(profile: &ExperimentProfile, task_id: &str) -> usize {
    profile.corpus.iter().filter(|case| case.task_id == task_id).count()
}

fn derive_adoption(profile: &ExperimentProfile, observations: &ExperimentObservations) -> Vec<AdoptionSummary> {
    let availability = observations
        .tool_availability
        .iter()
        .map(|record| (record.tool_id.as_str(), record.status.clone()))
        .collect::<BTreeMap<_, _>>();
    let clean = profile.adoption.clean_consumer.selection_id.as_str();
    let real = profile.adoption.real_consumer.selection_id.as_str();
    let mut summaries = Vec::new();
    for plan in &profile.adoption.tools {
        let mut clean_initial = None;
        let mut clean_incremental = None;
        let mut real_initial = None;
        let mut real_incremental = None;
        let mut measured_targets = BTreeSet::new();
        let mut unavailable_reasons = BTreeSet::new();
        for observation in observations.adoption.iter().filter(|observation| observation.tool_id == plan.tool_id) {
            let duration = if observation.measured
                && observation.exit_code == Some(0)
                && observation.wall_duration_ns.is_some_and(|value| value > 0)
            {
                observation.wall_duration_ns
            } else {
                None
            };
            if duration.is_some() {
                measured_targets.insert(observation.target_selection.clone());
            } else if let Some(reason) = observation.unavailable_reason.as_ref() {
                unavailable_reasons.insert(reason.clone());
            }
            match (observation.target_selection.as_str(), &observation.phase) {
                (target, AdoptionPhase::Initial) if target == clean => clean_initial = duration,
                (target, AdoptionPhase::Incremental) if target == clean => clean_incremental = duration,
                (target, AdoptionPhase::Initial) if target == real => real_initial = duration,
                (target, AdoptionPhase::Incremental) if target == real => real_incremental = duration,
                _ => {}
            }
        }
        let incremental_over_initial_basis_points = match (clean_initial, clean_incremental) {
            (Some(initial), Some(incremental)) if initial > 0 => ratio_basis_points(incremental, initial),
            _ => None,
        };
        summaries.push(AdoptionSummary {
            tool_id: plan.tool_id.clone(),
            availability: availability.get(plan.tool_id.as_str()).cloned().unwrap_or(Availability::Unavailable),
            setup_steps: u64::try_from(plan.setup_steps.len()).unwrap_or(u64::MAX),
            config_files: u64::try_from(plan.required_config_files.len()).unwrap_or(u64::MAX),
            clean_initial_ns: clean_initial,
            clean_incremental_ns: clean_incremental,
            real_initial_ns: real_initial,
            real_incremental_ns: real_incremental,
            incremental_over_initial_basis_points,
            measured_targets: measured_targets.into_iter().collect(),
            unavailable_metrics: plan.unavailable_metrics.clone(),
            unavailable_reasons: unavailable_reasons.into_iter().collect(),
        });
    }
    summaries
}

fn derive_provenance(observations: &ExperimentObservations) -> ProvenanceSummary {
    let mut executed_locally = BTreeSet::new();
    let mut recorded_fixtures = BTreeSet::new();
    let mut unavailable = BTreeSet::new();
    for record in &observations.tool_availability {
        match record.status {
            Availability::Local => {
                executed_locally.insert(record.tool_id.clone());
            }
            Availability::Recorded => {
                recorded_fixtures.insert(record.tool_id.clone());
            }
            Availability::Unavailable => {
                unavailable.insert(record.tool_id.clone());
            }
        }
    }
    ProvenanceSummary {
        executed_locally: executed_locally.into_iter().collect(),
        recorded_fixtures: recorded_fixtures.into_iter().collect(),
        unavailable: unavailable.into_iter().collect(),
    }
}

fn run_negative_controls(
    profile: &ExperimentProfile,
    observations: &ExperimentObservations,
    artifact: &ExperimentArtifact,
) -> Vec<NegativeControlRecord> {
    vec![
        control_noncompiling(profile, artifact),
        control_stale_oracle(profile, artifact),
        control_differing_scope(profile, artifact),
        control_dropped_timeout(profile, artifact),
        control_stale_results(profile, artifact),
        control_hidden_environment(profile, artifact),
        control_unsupported_clean(profile, observations),
        control_bounded_corpus(profile, artifact),
    ]
}

fn control_record(control_id: &str, diagnostics: &[String]) -> NegativeControlRecord {
    let outcome = if diagnostics.is_empty() {
        ControlOutcome::Escaped
    } else {
        ControlOutcome::Contained
    };
    let preferred = [
        "noncompiling",
        "stale oracle",
        "scope",
        "missing",
        "toolchain",
        "complete Rust",
    ];
    let evidence = diagnostics
        .iter()
        .find(|diagnostic| preferred.iter().any(|token| diagnostic.contains(token)))
        .or_else(|| diagnostics.first())
        .cloned()
        .unwrap_or_default();
    NegativeControlRecord {
        control_id: control_id.to_owned(),
        outcome,
        evidence,
    }
}

fn control_noncompiling(profile: &ExperimentProfile, artifact: &ExperimentArtifact) -> NegativeControlRecord {
    let mut mutated = artifact.clone();
    let target = mutated.observations.iter_mut().find(|observation| {
        observation
            .case_id
            .as_deref()
            .and_then(|case_id| profile.corpus.iter().find(|case| case.case_id == case_id))
            .is_some_and(|case| case.expected == ExpectedOutcome::Finding)
            && observation.findings.iter().any(|finding| !finding.suppressed)
    });
    if let Some(observation) = target {
        observation.compile_status = CompileStatus::NotCompiled;
    } else {
        let Some(observation) = mutated.observations.iter_mut().find(|observation| {
            observation.case_id.as_deref().is_some_and(|case_id| {
                profile
                    .corpus
                    .iter()
                    .any(|case| case.case_id == case_id && case.expected == ExpectedOutcome::Finding)
            })
        }) else {
            return NegativeControlRecord {
                control_id: "noncompiling-violation-counted-as-detection".to_owned(),
                outcome: ControlOutcome::NotRun,
                evidence: "no violating case observation".to_owned(),
            };
        };
        let Some(case) = observation
            .case_id
            .as_deref()
            .and_then(|case_id| profile.corpus.iter().find(|case| case.case_id == case_id))
        else {
            return NegativeControlRecord {
                control_id: "noncompiling-violation-counted-as-detection".to_owned(),
                outcome: ControlOutcome::NotRun,
                evidence: "violating case is missing from the profile".to_owned(),
            };
        };
        observation.status = ObservationStatus::Completed;
        observation.compile_status = CompileStatus::NotCompiled;
        observation.exit_code = Some(0);
        observation.wall_duration_ns = Some(1);
        observation.findings.push(ObservedFinding {
            case_id: case.case_id.clone(),
            rule_id: "negative-control.noncompiling".to_owned(),
            path: case.source_path.clone(),
            line: 1,
            message: "negative-control finding attached to a noncompiling case".to_owned(),
            suppressed: false,
        });
    }
    let diagnostics = verify_artifact(profile, &mutated);
    control_record("noncompiling-violation-counted-as-detection", &diagnostics)
}

fn control_stale_oracle(profile: &ExperimentProfile, artifact: &ExperimentArtifact) -> NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(observation) = mutated.observations.first_mut() else {
        return NegativeControlRecord {
            control_id: "renamed-stale-oracle".to_owned(),
            outcome: ControlOutcome::NotRun,
            evidence: "no retained observations".to_owned(),
        };
    };
    observation.source_identity = prefixed_identity(b"renamed-oracle");
    let diagnostics = verify_artifact(profile, &mutated);
    control_record("renamed-stale-oracle", &diagnostics)
}

fn control_differing_scope(profile: &ExperimentProfile, artifact: &ExperimentArtifact) -> NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(observation) = mutated.observations.first_mut() else {
        return NegativeControlRecord {
            control_id: "differing-scope-hidden".to_owned(),
            outcome: ControlOutcome::NotRun,
            evidence: "no retained observations".to_owned(),
        };
    };
    observation.scope.selected_sources.push("hidden/other.rs".to_owned());
    let diagnostics = verify_artifact(profile, &mutated);
    control_record("differing-scope-hidden", &diagnostics)
}

fn control_dropped_timeout(profile: &ExperimentProfile, artifact: &ExperimentArtifact) -> NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(index) = mutated
        .observations
        .iter()
        .position(|observation| observation.status == ObservationStatus::TimedOut)
        .or_else(|| {
            mutated.observations.iter().position(|observation| {
                observation.case_id.is_some()
                    && !matches!(observation.status, ObservationStatus::Unavailable | ObservationStatus::Unsupported)
            })
        })
    else {
        return NegativeControlRecord {
            control_id: "dropped-timeouts".to_owned(),
            outcome: ControlOutcome::NotRun,
            evidence: "no comparable observation to drop".to_owned(),
        };
    };
    if let Some(observation) = mutated.observations.get_mut(index)
        && observation.status != ObservationStatus::TimedOut
    {
        observation.status = ObservationStatus::TimedOut;
        observation.failure_reason = Some("negative-control timeout".to_owned());
    }
    if index < mutated.observations.len() {
        mutated.observations.remove(index);
    }
    let diagnostics = verify_artifact(profile, &mutated);
    control_record("dropped-timeouts", &diagnostics)
}

fn control_stale_results(profile: &ExperimentProfile, artifact: &ExperimentArtifact) -> NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(metrics) = mutated.derived.architecture.first_mut() else {
        return NegativeControlRecord {
            control_id: "stale-results".to_owned(),
            outcome: ControlOutcome::NotRun,
            evidence: "no derived architecture metrics".to_owned(),
        };
    };
    metrics.true_positive = metrics.true_positive.saturating_add(1);
    let diagnostics = verify_artifact(profile, &mutated);
    control_record("stale-results", &diagnostics)
}

fn control_hidden_environment(profile: &ExperimentProfile, artifact: &ExperimentArtifact) -> NegativeControlRecord {
    let mut mutated = artifact.clone();
    let Some(observation) = mutated.observations.iter_mut().find(|observation| observation.case_id.is_some()) else {
        return NegativeControlRecord {
            control_id: "hidden-toolchain-cache-difference".to_owned(),
            outcome: ControlOutcome::NotRun,
            evidence: "no case observation".to_owned(),
        };
    };
    observation.environment.rust_toolchain = "nightly-unreviewed".to_owned();
    observation.environment.cache_state = CacheState::Warm;
    observation.warm_state = super::model::WarmState::Warm;
    let diagnostics = verify_artifact(profile, &mutated);
    control_record("hidden-toolchain-cache-difference", &diagnostics)
}

fn control_unsupported_clean(
    profile: &ExperimentProfile,
    observations: &ExperimentObservations,
) -> NegativeControlRecord {
    let Some(case) = profile
        .corpus
        .iter()
        .find(|case| case.expected == ExpectedOutcome::Clean && case.category == Category::Architecture)
    else {
        return NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: ControlOutcome::NotRun,
            evidence: "no clean architecture case".to_owned(),
        };
    };
    let Some(entry) = profile
        .applicability
        .iter()
        .find(|entry| entry.task_id == case.task_id && entry.case_ids.iter().any(|id| id == &case.case_id))
    else {
        return NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: ControlOutcome::NotRun,
            evidence: "no claimed clean case".to_owned(),
        };
    };
    let Ok(before) = derive_projection(profile, observations) else {
        return NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: ControlOutcome::Escaped,
            evidence: "baseline projection failed".to_owned(),
        };
    };
    let before_metrics = find_metrics(&before, &entry.tool_id, &entry.task_id);
    let mut mutated = observations.clone();
    let Some(observation) = mutated.observations.iter_mut().find(|observation| {
        observation.tool_id == entry.tool_id
            && observation.task_id == entry.task_id
            && observation.case_id.as_deref() == Some(case.case_id.as_str())
    }) else {
        return NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: ControlOutcome::NotRun,
            evidence: "no observation for the selected clean case".to_owned(),
        };
    };
    observation.status = ObservationStatus::Unsupported;
    observation.unavailable_reason = Some("negative-control unsupported scope".to_owned());
    let Ok(after) = derive_projection(profile, &mutated) else {
        return NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: ControlOutcome::Contained,
            evidence: "projection rejected the unsupported observation".to_owned(),
        };
    };
    let after_metrics = find_metrics(&after, &entry.tool_id, &entry.task_id);
    let clean_evidence_increased = after_metrics.is_some_and(|after_metrics| {
        before_metrics.is_some_and(|before_metrics| after_metrics.true_negative > before_metrics.true_negative)
    });
    if clean_evidence_increased {
        NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: ControlOutcome::Escaped,
            evidence: "unsupported scope increased true negatives".to_owned(),
        }
    } else {
        NegativeControlRecord {
            control_id: "unsupported-counted-clean".to_owned(),
            outcome: ControlOutcome::Contained,
            evidence: "unsupported scope did not increase clean evidence".to_owned(),
        }
    }
}

fn find_metrics<'a>(projection: &'a DerivedProjection, tool_id: &str, task_id: &str) -> Option<&'a ToolTaskMetrics> {
    projection
        .architecture
        .iter()
        .chain(projection.assurance.iter())
        .find(|metrics| metrics.tool_id == tool_id && metrics.task_id == task_id)
}

fn control_bounded_corpus(profile: &ExperimentProfile, artifact: &ExperimentArtifact) -> NegativeControlRecord {
    let mut mutated = artifact.clone();
    mutated.profile.coverage_boundary.complete_rust_coverage = true;
    let diagnostics = verify_artifact(profile, &mutated);
    control_record("bounded-corpus-presented-as-complete", &diagnostics)
}

fn identity_of<T: serde::Serialize>(value: &T, label: &str) -> Result<String, String> {
    pretty_bytes(value, label).map(|bytes| prefixed_identity(&bytes))
}

pub(crate) fn scope_identity(scope: &ObservationScope) -> String {
    let mut selected = scope.selected_sources.clone();
    selected.sort();
    let mut analyzed = scope.analyzed_sources.clone();
    analyzed.sort();
    let payload = serde_json::json!({
        "selected_sources": selected,
        "analyzed_sources": analyzed,
    });
    match serde_json::to_vec(&payload) {
        Ok(bytes) => prefixed_identity(&bytes),
        Err(_) => String::new(),
    }
}

fn is_b3(value: &str) -> bool {
    value
        .strip_prefix("b3:")
        .is_some_and(|hex| hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}

fn failed(mut diagnostics: Vec<String>) -> ExperimentEvaluation {
    diagnostics.sort();
    diagnostics.dedup();
    ExperimentEvaluation {
        artifact: None,
        diagnostics,
    }
}
