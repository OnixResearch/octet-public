use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::model::ARTIFACT_NAME;
use super::model::ApplicabilityStatus;
use super::model::Availability;
use super::model::Category;
use super::model::ExperimentArtifact;
use super::model::OBSERVATIONS_NAME;
use super::model::ObservationStatus;
use super::model::ToolTaskMetrics;

pub fn render_report(artifact: &ExperimentArtifact) -> String {
    let mut output = String::new();
    let profile = &artifact.profile;
    let _ = writeln!(output, "# Cross-tool benchmark experiment report");
    let _ = writeln!(output);
    let _ = writeln!(output, "Source-linked report for profile `{}`.", profile.profile_id);
    let _ = writeln!(output);
    let _ = writeln!(output, "- Profile identity: `{}`", artifact.profile_identity);
    let _ = writeln!(output, "- Source revision: `{}`", profile.source_revision);
    let _ = writeln!(output, "- Rust toolchain: `{}`", profile.rust_toolchain);
    let _ = writeln!(output, "- Evidence tier: `{}`", profile.evidence_tier);
    let _ = writeln!(output, "- Machine artifact: `{}`", ARTIFACT_NAME);
    let _ = writeln!(output, "- Retained observations: `{}`", OBSERVATIONS_NAME);
    let _ = writeln!(output, "- Declared before measurement: `{}`", profile.declared_before_measurement);
    let _ = writeln!(output);

    write_provenance(&mut output, artifact);
    write_tool_pins(&mut output, artifact);
    write_applicability(&mut output, artifact);
    write_corpus(&mut output, artifact);
    write_metrics(
        &mut output,
        "Architecture detection results",
        &artifact.derived.architecture,
        Category::Architecture,
    );
    write_metrics(&mut output, "Assurance results", &artifact.derived.assurance, Category::Assurance);
    write_retained_failures(&mut output, artifact);
    write_adoption(&mut output, artifact);
    write_controls(&mut output, artifact);
    write_boundary(&mut output, artifact);
    output
}

fn write_provenance(output: &mut String, artifact: &ExperimentArtifact) {
    let provenance = &artifact.derived.provenance;
    let _ = writeln!(output, "## Measurement provenance");
    let _ = writeln!(output);
    let _ = writeln!(output, "Executed locally:");
    let _ = writeln!(output);
    write_list(output, &provenance.executed_locally, "none");
    let _ = writeln!(output);
    let _ = writeln!(output, "Recorded fixtures:");
    let _ = writeln!(output);
    write_list(output, &provenance.recorded_fixtures, "none");
    let _ = writeln!(output);
    let _ = writeln!(output, "Unavailable executables:");
    let _ = writeln!(output);
    write_list(output, &provenance.unavailable, "none");
    let _ = writeln!(output);
}

fn write_tool_pins(output: &mut String, artifact: &ExperimentArtifact) {
    let _ = writeln!(output, "## Tool and configuration pins");
    let _ = writeln!(output);
    let _ = writeln!(output, "| Tool | Version | Source pin | Configuration | Availability |");
    let _ = writeln!(output, "| --- | --- | --- | --- | --- |");
    let availability = artifact
        .tool_availability
        .iter()
        .map(|record| (record.tool_id.as_str(), record))
        .collect::<BTreeMap<_, _>>();
    for tool in &artifact.profile.tools {
        let status = availability
            .get(tool.tool_id.as_str())
            .map(|record| availability_label(&record.status))
            .unwrap_or("missing");
        let config = if tool.config_path.is_empty() {
            format!("{:?}: {}", tool.configuration_status, tool.rules_pin)
        } else {
            format!("{} ({:?})", tool.config_path.join(", "), tool.configuration_status)
        };
        let _ = writeln!(
            output,
            "| `{}` | {} | {} | {} | {} |",
            tool.tool_id, tool.version_pin, tool.source_pin, config, status
        );
    }
    for tool in &artifact.profile.tools {
        let status = availability.get(tool.tool_id.as_str());
        if let Some(record) = status
            && let Some(reason) = record.reason.as_ref()
        {
            let _ = writeln!(output);
            let _ = writeln!(output, "- `{}` unavailable: {reason}", tool.tool_id);
        }
    }
    let _ = writeln!(output);
}

fn write_applicability(output: &mut String, artifact: &ExperimentArtifact) {
    let _ = writeln!(output, "## Applicability matrix");
    let _ = writeln!(output);
    let tools = &artifact.profile.tools;
    let mut header = String::from("| Task |");
    for tool in tools {
        let _ = write!(header, " {} |", tool.tool_id);
    }
    let _ = writeln!(output, "{header}");
    let mut separator = String::from("| --- |");
    for _ in tools {
        separator.push_str(" --- |");
    }
    let _ = writeln!(output, "{separator}");
    let matrix = artifact
        .profile
        .applicability
        .iter()
        .map(|entry| ((entry.task_id.as_str(), entry.tool_id.as_str()), &entry.status))
        .collect::<BTreeMap<_, _>>();
    for task in &artifact.profile.tasks {
        let mut row = format!("| `{}` |", task.task_id);
        for tool in tools {
            let status = matrix
                .get(&(task.task_id.as_str(), tool.tool_id.as_str()))
                .map(|status| applicability_mark(status))
                .unwrap_or("?");
            let _ = write!(row, " {status} |");
        }
        let _ = writeln!(output, "{row}");
    }
    let _ = writeln!(output);
    let _ = writeln!(output, "S = supported, P = partial, U = unsupported. Unsupported cells are not clean evidence.");
    let _ = writeln!(output);
}

fn write_corpus(output: &mut String, artifact: &ExperimentArtifact) {
    let _ = writeln!(output, "## Frozen corpus and adjudication");
    let _ = writeln!(output);
    let _ = writeln!(output, "| Case | Variant | Expected | Source link | Source identity | Rationale | Reviewer |");
    let _ = writeln!(output, "| --- | --- | --- | --- | --- | --- | --- |");
    for case in &artifact.profile.corpus {
        let _ = writeln!(
            output,
            "| `{}` | {:?} | {:?} | [{}]({}) | `{}` | {} | {} |",
            case.case_id,
            case.variant,
            case.expected,
            case.source_path,
            source_link(&case.source_path, &case.case_id),
            case.source_identity,
            case.rationale,
            case.reviewer_role
        );
    }
    let _ = writeln!(output);
}

fn write_metrics(output: &mut String, title: &str, rows: &[ToolTaskMetrics], category: Category) {
    let _ = writeln!(output, "## {title}");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| Tool | Task | Applicability | Availability | Comparable | TP | FP | FN | TN | Unsupported | Inconclusive | Incomparable | Precision bp | Recall bp |"
    );
    let _ = writeln!(output, "| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    for metrics in rows.iter().filter(|metrics| metrics.category == category) {
        let _ = writeln!(
            output,
            "| `{}` | `{}` | {:?} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |",
            metrics.tool_id,
            metrics.task_id,
            metrics.applicability_status,
            availability_label(&metrics.availability),
            metrics.comparable_cases,
            metrics.true_positive,
            metrics.false_positive,
            metrics.false_negative,
            metrics.true_negative,
            metrics.unsupported_cases,
            metrics.inconclusive_cases,
            metrics.incomparable_cases,
            option_number(metrics.precision_basis_points),
            option_number(metrics.recall_basis_points)
        );
    }
    let _ = writeln!(output);
    let _ = writeln!(output, "Precision and recall use only the declared comparable subset.");
    let _ = writeln!(output);
    let _ = writeln!(output, "### Unsupported applicability cells");
    let _ = writeln!(output);
    let mut unsupported_by_tool = BTreeMap::new();
    for metrics in rows.iter().filter(|metrics| metrics.category == category) {
        if metrics.applicability_status == ApplicabilityStatus::Unsupported {
            *unsupported_by_tool.entry(metrics.tool_id.as_str()).or_insert(0_u64) += 1;
        }
    }
    for (tool, count) in unsupported_by_tool {
        let _ = writeln!(output, "- `{tool}`: {count} unsupported task cells");
    }
    let _ = writeln!(output);
}

fn write_retained_failures(output: &mut String, artifact: &ExperimentArtifact) {
    let _ = writeln!(output, "## Retained failures, timeouts, and unparsed runs");
    let _ = writeln!(output);
    let mut retained = false;
    for observation in &artifact.observations {
        if observation.status == ObservationStatus::Completed {
            continue;
        }
        retained = true;
        let case = observation.case_id.as_deref().unwrap_or("<tool-level>");
        let reason = observation
            .failure_reason
            .as_deref()
            .or(observation.unavailable_reason.as_deref())
            .unwrap_or("no reason recorded");
        let _ = writeln!(
            output,
            "- `{}` / `{}` / `{}`: {:?} ({reason})",
            observation.tool_id, observation.task_id, case, observation.status
        );
    }
    if !retained {
        let _ = writeln!(output, "None.");
    }
    let _ = writeln!(output);
}

fn write_adoption(output: &mut String, artifact: &ExperimentArtifact) {
    let _ = writeln!(output, "## Adoption cost");
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "Clean consumer: `{}` ({}@{}).",
        artifact.profile.adoption.clean_consumer.selection_id,
        artifact.profile.adoption.clean_consumer.source_repository,
        artifact.profile.adoption.clean_consumer.source_revision
    );
    let _ = writeln!(
        output,
        "Real consumer: `{}` ({}@{}).",
        artifact.profile.adoption.real_consumer.selection_id,
        artifact.profile.adoption.real_consumer.source_repository,
        artifact.profile.adoption.real_consumer.source_revision
    );
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| Tool | Availability | Setup steps | Config files | Clean initial | Clean incremental | Real initial | Real incremental | Incremental/initial |"
    );
    let _ = writeln!(output, "| --- | --- | --- | --- | --- | --- | --- | --- | --- |");
    for summary in &artifact.derived.adoption {
        let _ = writeln!(
            output,
            "| `{}` | {} | {} | {} | {} | {} | {} | {} | {} |",
            summary.tool_id,
            availability_label(&summary.availability),
            summary.setup_steps,
            summary.config_files,
            option_duration(summary.clean_initial_ns),
            option_duration(summary.clean_incremental_ns),
            option_duration(summary.real_initial_ns),
            option_duration(summary.real_incremental_ns),
            option_ratio(summary.incremental_over_initial_basis_points)
        );
    }
    let _ = writeln!(output);
    for summary in &artifact.derived.adoption {
        if summary.unavailable_reasons.is_empty() {
            continue;
        }
        let _ = writeln!(output, "- `{}` unavailable rows:", summary.tool_id);
        for reason in &summary.unavailable_reasons {
            let _ = writeln!(output, "  - {reason}");
        }
        if !summary.unavailable_metrics.is_empty() {
            let _ = writeln!(output, "  - unavailable metrics: {}", summary.unavailable_metrics.join(", "));
        }
    }
    let _ = writeln!(output);
    let _ = writeln!(output, "### Documented setup steps");
    let _ = writeln!(output);
    for tool in &artifact.profile.tools {
        let _ = writeln!(output, "- `{}` (source: {}):", tool.tool_id, tool.setup_source);
        for step in &tool.setup_steps {
            let _ = writeln!(output, "  - {step}");
        }
        if !tool.required_config_files.is_empty() {
            let _ = writeln!(output, "  - required config: {}", tool.required_config_files.join(", "));
        }
    }
    let _ = writeln!(output);
}

fn write_controls(output: &mut String, artifact: &ExperimentArtifact) {
    let _ = writeln!(output, "## Negative controls");
    let _ = writeln!(output);
    let _ = writeln!(output, "| Control | Outcome | Evidence |");
    let _ = writeln!(output, "| --- | --- | --- |");
    for control in &artifact.controls {
        let _ = writeln!(output, "| `{}` | {:?} | {} |", control.control_id, control.outcome, control.evidence);
    }
    let _ = writeln!(output);
}

fn write_boundary(output: &mut String, artifact: &ExperimentArtifact) {
    let boundary = &artifact.profile.coverage_boundary;
    let _ = writeln!(output, "## Coverage boundary and non-claims");
    let _ = writeln!(output);
    let _ = writeln!(output, "- Corpus: `{}`", boundary.corpus_id);
    let _ = writeln!(output, "- Bounded: `{}`", boundary.bounded);
    let _ = writeln!(output, "- Complete Rust coverage: `{}`", boundary.complete_rust_coverage);
    let _ = writeln!(output, "- {}", boundary.statement);
    let _ = writeln!(output);
    for claim in &artifact.profile.non_claims {
        let _ = writeln!(output, "- {claim}");
    }
}

fn write_list(output: &mut String, values: &[String], empty: &str) {
    if values.is_empty() {
        let _ = writeln!(output, "- {empty}");
        return;
    }
    for value in values {
        let _ = writeln!(output, "- `{value}`");
    }
}

fn availability_label(availability: &Availability) -> &'static str {
    match availability {
        Availability::Local => "local",
        Availability::Recorded => "recorded",
        Availability::Unavailable => "unavailable",
    }
}

fn applicability_mark(status: &ApplicabilityStatus) -> &'static str {
    match status {
        ApplicabilityStatus::Supported => "S",
        ApplicabilityStatus::Partial => "P",
        ApplicabilityStatus::Unsupported => "U",
    }
}

fn option_number(value: Option<u64>) -> String {
    value.map_or_else(|| "n/a".to_owned(), |value| value.to_string())
}

fn option_duration(value: Option<u64>) -> String {
    value.map_or_else(|| "unavailable".to_owned(), |value| format!("{} ns", value))
}

fn option_ratio(value: Option<u64>) -> String {
    value.map_or_else(|| "unavailable".to_owned(), |value| format!("{} bp", value))
}

fn source_link(path: &str, case_id: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        return path.to_owned();
    }
    format!("../../{path}#{case_id}")
}
