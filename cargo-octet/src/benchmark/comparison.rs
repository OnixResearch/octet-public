// r[impl octet.benchmark_evidence.comparison]
use super::*;

const BASIS_POINTS_SCALE: u128 = 10_000;
const MEDIAN_PAIR_WIDTH: usize = 2;

pub(super) fn compare(baseline: &RunSet, candidate: &RunSet, policy: ComparisonPolicy) -> PerformanceComparison {
    let mut reasons = comparability_reasons(baseline, candidate);
    if !reasons.is_empty() {
        return result(ComparisonResult::Incomparable, None, None, policy, reasons);
    }
    append_failed_sample_reasons("baseline", &baseline.samples, &mut reasons);
    append_failed_sample_reasons("candidate", &candidate.samples, &mut reasons);
    let baseline_median = successful_median(&baseline.samples);
    let candidate_median = successful_median(&candidate.samples);
    if baseline_median.is_none() || candidate_median.is_none() {
        reasons.push("benchmark samples have no positive successful wall durations".to_owned());
    }
    if baseline.samples.len() < policy.minimum_samples || candidate.samples.len() < policy.minimum_samples {
        reasons.push("benchmark sample count is below the comparison minimum".to_owned());
    }
    if let Some(median) = baseline_median {
        append_uncertainty_reason("baseline", &baseline.samples, median, policy, &mut reasons);
    }
    if let Some(median) = candidate_median {
        append_uncertainty_reason("candidate", &candidate.samples, median, policy, &mut reasons);
    }
    if !reasons.is_empty() {
        reasons.sort();
        reasons.dedup();
        return result(ComparisonResult::Inconclusive, baseline_median, candidate_median, policy, reasons);
    }
    let (Some(baseline_value), Some(candidate_value)) = (baseline_median, candidate_median) else {
        return result(ComparisonResult::Inconclusive, baseline_median, candidate_median, policy, vec![
            "benchmark medians are unavailable".to_owned(),
        ]);
    };
    let classification = classify_delta(baseline_value, candidate_value, policy.practical_threshold_basis_points);
    result(classification, Some(baseline_value), Some(candidate_value), policy, Vec::new())
}

fn comparability_reasons(baseline: &RunSet, candidate: &RunSet) -> Vec<String> {
    let mut reasons = Vec::new();
    compare_field("workload identity", &baseline.workload_identity, &candidate.workload_identity, &mut reasons);
    compare_field("command ID", &baseline.command_id, &candidate.command_id, &mut reasons);
    if baseline.command_argv != candidate.command_argv {
        reasons.push("benchmark command arguments differ".to_owned());
    }
    compare_field("mode", &baseline.mode, &candidate.mode, &mut reasons);
    compare_field(
        "source state identity",
        &baseline.source_state_identity,
        &candidate.source_state_identity,
        &mut reasons,
    );
    if baseline.environment != candidate.environment {
        reasons.push("benchmark environments differ".to_owned());
    }
    reasons.sort();
    reasons.dedup();
    reasons
}

fn compare_field(label: &str, baseline: &str, candidate: &str, reasons: &mut Vec<String>) {
    if baseline != candidate {
        reasons.push(format!("benchmark {label} differs"));
    }
}

fn append_failed_sample_reasons(label: &str, samples: &[Sample], reasons: &mut Vec<String>) {
    for sample in samples {
        if sample.outcome != "succeeded" {
            reasons.push(format!("{label} sample {} has outcome `{}`", sample.sample_index, sample.outcome));
        }
    }
}

fn successful_median(samples: &[Sample]) -> Option<u64> {
    let mut values: Vec<u64> = samples
        .iter()
        .filter(|sample| sample.outcome == "succeeded" && sample.wall_duration_ns > 0)
        .map(|sample| sample.wall_duration_ns)
        .collect();
    values.sort_unstable();
    let midpoint = values.len().checked_div(MEDIAN_PAIR_WIDTH)?;
    if values.is_empty() {
        return None;
    }
    if values.len() % MEDIAN_PAIR_WIDTH == 1 {
        return values.get(midpoint).copied();
    }
    let lower = *values.get(midpoint.checked_sub(1)?)?;
    let upper = *values.get(midpoint)?;
    lower.checked_add(upper)?.checked_div(u64::try_from(MEDIAN_PAIR_WIDTH).ok()?)
}

fn append_uncertainty_reason(
    label: &str,
    samples: &[Sample],
    median: u64,
    policy: ComparisonPolicy,
    reasons: &mut Vec<String>,
) {
    let values: Vec<u64> = samples
        .iter()
        .filter(|sample| sample.outcome == "succeeded")
        .map(|sample| sample.wall_duration_ns)
        .collect();
    let Some(minimum) = values.iter().min().copied() else {
        return;
    };
    let Some(maximum) = values.iter().max().copied() else {
        return;
    };
    if median == 0 {
        reasons.push(format!("{label} median is zero"));
        return;
    }
    let spread = u128::from(maximum.saturating_sub(minimum));
    let spread_basis_points = spread
        .checked_mul(BASIS_POINTS_SCALE)
        .and_then(|scaled| scaled.checked_div(u128::from(median)))
        .unwrap_or(u128::MAX);
    if spread_basis_points > u128::from(policy.uncertainty_threshold_basis_points) {
        reasons.push(format!("{label} sample uncertainty exceeds the checked policy"));
    }
}

fn classify_delta(baseline: u64, candidate: u64, practical_threshold_basis_points: u64) -> ComparisonResult {
    if baseline == 0 {
        return ComparisonResult::Inconclusive;
    }
    let absolute_delta = baseline.abs_diff(candidate);
    let delta_basis_points = u128::from(absolute_delta)
        .checked_mul(BASIS_POINTS_SCALE)
        .and_then(|scaled| scaled.checked_div(u128::from(baseline)))
        .unwrap_or(u128::MAX);
    if delta_basis_points < u128::from(practical_threshold_basis_points) {
        ComparisonResult::Stable
    } else if candidate < baseline {
        ComparisonResult::Improved
    } else {
        ComparisonResult::Regressed
    }
}

fn result(
    classification: ComparisonResult,
    baseline_median_ns: Option<u64>,
    candidate_median_ns: Option<u64>,
    policy: ComparisonPolicy,
    reasons: Vec<String>,
) -> PerformanceComparison {
    PerformanceComparison {
        result: classification,
        baseline_median_ns,
        candidate_median_ns,
        practical_threshold_basis_points: policy.practical_threshold_basis_points,
        uncertainty_threshold_basis_points: policy.uncertainty_threshold_basis_points,
        reasons,
    }
}
