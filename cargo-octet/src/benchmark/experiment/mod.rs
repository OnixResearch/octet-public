//! Opt-in cross-tool experiment surface.
//!
//! Public names below are the API the binary's CLI wiring consumes; the binary
//! owner adds the subcommand separately, so the library keeps the surface
//! available without treating it as dead output.
#![allow(unused_imports)]
//!
//! `benchmark experiment` is intentionally separate from the same-command
//! performance comparator. Recorded mode consumes a retained observation file;
//! execute mode performs bounded probes and adoption measurements. Neither mode
//! upgrades an unsupported tool or scope into clean evidence.

mod evaluate;
mod execution;
mod model;
mod profile;
mod report;

#[cfg(test)]
mod fixtures;
#[cfg(test)]
mod tests;

pub use evaluate::evaluate;
pub use evaluate::verify;
pub use execution::ExperimentMode;
pub use execution::ExperimentRun;
pub use execution::probe_tool;
pub use execution::run;
#[cfg(test)]
pub use execution::run_command;
pub use model::ARTIFACT_NAME;
pub use model::ARTIFACT_SCHEMA;
pub use model::AdoptionObservation;
pub use model::AdoptionPhase;
pub use model::AdoptionSummary;
pub use model::Applicability;
pub use model::ApplicabilityStatus;
pub use model::Availability;
pub use model::CacheState;
pub use model::Category;
pub use model::CompileStatus;
pub use model::ConfigurationStatus;
pub use model::ControlOutcome;
pub use model::CoverageBoundary;
pub use model::DerivedProjection;
pub use model::ExecutionSemantics;
pub use model::ExpectedOutcome;
pub use model::ExperimentArtifact;
pub use model::ExperimentCase;
pub use model::ExperimentEvaluation;
pub use model::ExperimentObservations;
pub use model::ExperimentProfile;
pub use model::ExperimentRegistry;
pub use model::ExperimentTask;
pub use model::MeasurementOrigin;
pub use model::NegativeControlDeclaration;
pub use model::NegativeControlRecord;
pub use model::OBSERVATIONS_NAME;
pub use model::OBSERVATIONS_SCHEMA;
pub use model::ObservationEnvironment;
pub use model::ObservationScope;
pub use model::ObservationStatus;
pub use model::ObservedFinding;
pub use model::ObservedSuppression;
pub use model::PROFILE_ID;
pub use model::PROFILES_SCHEMA;
pub use model::ParserKind;
pub use model::ProvenanceSummary;
pub use model::REPORT_NAME;
pub use model::REQUIRED_NON_CLAIM;
pub use model::SamplePlan;
pub use model::ToolAvailability;
pub use model::ToolObservation;
pub use model::ToolPin;
pub use model::ToolTaskMetrics;
pub use model::Variant;
pub use model::WarmState;
pub use profile::PROFILE_IDENTITY;
#[cfg(test)]
pub use profile::embedded_profile_identity;
#[cfg(test)]
pub use profile::profile_bytes_identity;
pub use profile::selected_profile;
pub use report::render_report;

/// Runs the `benchmark experiment` CLI surface.
pub(crate) fn run_cli(args: &crate::cli::BenchmarkExperimentArgs) -> Result<i32, String> {
    use std::fs;
    use std::path::Path;

    use crate::cli::OutputFormat;

    if args.verify {
        let artifact_path = args.artifact.clone().unwrap_or_else(|| args.artifact_dir.join(ARTIFACT_NAME));
        let bytes = fs::read(&artifact_path)
            .map_err(|error| format!("read experiment artifact {}: {error}", artifact_path.display()))?;
        let artifact: ExperimentArtifact = serde_json::from_slice(&bytes)
            .map_err(|error| format!("parse experiment artifact {}: {error}", artifact_path.display()))?;
        let diagnostics = verify(&args.profile, &artifact);
        let status = if diagnostics.is_empty() { "verified" } else { "invalid" };
        match args.output_format {
            OutputFormat::Json => {
                let report = serde_json::json!({
                    "schema_version": ARTIFACT_SCHEMA,
                    "status": status,
                    "artifact": artifact_path.display().to_string(),
                    "diagnostics": diagnostics,
                    "non_claims": [REQUIRED_NON_CLAIM],
                });
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report)
                        .map_err(|error| format!("serialize experiment verification: {error}"))?
                );
            }
            OutputFormat::Human | OutputFormat::Sarif => {
                println!(
                    "octet benchmark experiment verify artifact={} status={} diagnostics={}",
                    artifact_path.display(),
                    status,
                    diagnostics.len()
                );
                for diagnostic in &diagnostics {
                    println!("diagnostic: {diagnostic}");
                }
                println!("non-claim: {REQUIRED_NON_CLAIM}");
            }
        }
        return Ok(if diagnostics.is_empty() { 0 } else { 1 });
    }

    let corpus_root: &Path = if args.execute {
        &args.workspace_root
    } else {
        Path::new(".")
    };
    let input_text = match &args.input {
        Some(path) => Some(
            fs::read_to_string(path).map_err(|error| format!("read experiment input {}: {error}", path.display()))?,
        ),
        None => None,
    };
    let mode = if args.execute {
        ExperimentMode::Execute
    } else {
        ExperimentMode::Recorded
    };
    let run = run(mode, &args.profile, input_text.as_deref(), corpus_root)
        .map_err(|diagnostics| format!("benchmark experiment failed:\n- {}", diagnostics.join("\n- ")))?;
    fs::create_dir_all(&args.artifact_dir)
        .map_err(|error| format!("create experiment artifact directory {}: {error}", args.artifact_dir.display()))?;
    fs::write(args.artifact_dir.join(ARTIFACT_NAME), &run.artifact_bytes)
        .map_err(|error| format!("write experiment artifact: {error}"))?;
    fs::write(args.artifact_dir.join(REPORT_NAME), &run.report_bytes)
        .map_err(|error| format!("write experiment report: {error}"))?;
    let observations_bytes = crate::benchmark::pretty_bytes(&run.observations, "experiment observations")
        .map_err(|error| format!("serialize experiment observations: {error}"))?;
    fs::write(args.artifact_dir.join(OBSERVATIONS_NAME), observations_bytes)
        .map_err(|error| format!("write experiment observations: {error}"))?;
    match args.output_format {
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&run.artifact)
                .map_err(|error| format!("serialize experiment artifact: {error}"))?
        ),
        OutputFormat::Human | OutputFormat::Sarif => {
            println!(
                "octet benchmark experiment profile={} status=verified artifact={} report={} observations={} tool_observations={} controls={}",
                run.artifact.profile.profile_id,
                args.artifact_dir.join(ARTIFACT_NAME).display(),
                args.artifact_dir.join(REPORT_NAME).display(),
                args.artifact_dir.join(OBSERVATIONS_NAME).display(),
                run.artifact.observations.len(),
                run.artifact.controls.len()
            );
            println!("non-claim: {REQUIRED_NON_CLAIM}");
        }
    }
    Ok(0)
}
