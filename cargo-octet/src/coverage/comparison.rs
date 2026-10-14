use super::*;

const BASIS_POINTS_SCALE: u128 = 10_000;
const COMPARISON_DIAGNOSTIC_CAPACITY: usize = 16;

struct Diagnostics {
    entries: Vec<String>,
}

impl Diagnostics {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(COMPARISON_DIAGNOSTIC_CAPACITY),
        }
    }

    fn push(&mut self, value: String) {
        self.entries.push(value);
    }

    fn finish(self) -> Vec<String> {
        self.entries
    }
}

pub(super) fn basis_points(covered: u64, total: u64) -> Result<u64, String> {
    if total == 0 {
        return Err("coverage metric total must be positive".to_owned());
    }
    if covered > total {
        return Err("coverage metric covered count exceeds total".to_owned());
    }
    let scaled = u128::from(covered)
        .checked_mul(BASIS_POINTS_SCALE)
        .ok_or_else(|| "coverage percentage multiplication overflow".to_owned())?;
    let divisor = u128::from(total);
    let basis_points = scaled.checked_div(divisor).ok_or_else(|| "coverage percentage divisor is zero".to_owned())?;
    u64::try_from(basis_points).map_err(|_| "coverage percentage does not fit u64".to_owned())
}

pub(super) fn compare(current: &NativeReport, baseline: &NativeReport, profile: &Profile) -> (Comparison, Vec<String>) {
    let mut diagnostics = Diagnostics::new();
    compare_identity("profile", &current.profile_identity, &baseline.profile_identity, &mut diagnostics);
    compare_identity("provider", &current.provider_identity, &baseline.provider_identity, &mut diagnostics);
    compare_identity("Rust toolchain", &current.rust_toolchain, &baseline.rust_toolchain, &mut diagnostics);
    compare_identity(
        "LLVM tools component",
        &current.llvm_tools_component,
        &baseline.llvm_tools_component,
        &mut diagnostics,
    );
    compare_identity("source scope", &current.source_scope_identity, &baseline.source_scope_identity, &mut diagnostics);
    compare_identity("path policy", &current.path_policy_identity, &baseline.path_policy_identity, &mut diagnostics);
    compare_identity("source", &current.source_identity, &baseline.source_identity, &mut diagnostics);

    let lines = compare_metric(current.counts.lines, baseline.counts.lines, profile.threshold_basis_points.lines);
    let branches =
        compare_metric(current.counts.branches, baseline.counts.branches, profile.threshold_basis_points.branches);
    let functions =
        compare_metric(current.counts.functions, baseline.counts.functions, profile.threshold_basis_points.functions);
    let regions =
        compare_metric(current.counts.regions, baseline.counts.regions, profile.threshold_basis_points.regions);
    append_metric_diagnostics("lines", &lines, &mut diagnostics);
    append_metric_diagnostics("branches", &branches, &mut diagnostics);
    append_metric_diagnostics("functions", &functions, &mut diagnostics);
    append_metric_diagnostics("regions", &regions, &mut diagnostics);

    let is_passed = metric_passed(&lines)
        && metric_passed(&branches)
        && metric_passed(&functions)
        && metric_passed(&regions)
        && diagnostics.entries.is_empty();
    (
        Comparison {
            regression_policy: profile.regression_policy.clone(),
            denominator_policy: profile.denominator_policy.clone(),
            rounding_policy: profile.rounding_policy.clone(),
            precision_digits: profile.precision_digits,
            lines,
            branches,
            functions,
            regions,
            passed: is_passed,
        },
        diagnostics.finish(),
    )
}

fn compare_identity(label: &str, current: &str, baseline: &str, diagnostics: &mut Diagnostics) {
    if current != baseline {
        diagnostics.push(format!("coverage {label} identity is not comparable"));
    }
}

fn compare_metric(current: MetricCount, baseline: MetricCount, threshold_basis_points: u64) -> MetricComparison {
    let is_threshold_passed = u128::from(current.covered) * BASIS_POINTS_SCALE
        >= u128::from(threshold_basis_points) * u128::from(current.total);
    let is_regression_passed = u128::from(current.covered) * u128::from(baseline.total)
        >= u128::from(baseline.covered) * u128::from(current.total);
    let is_denominator_passed = current.total == baseline.total;
    MetricComparison {
        baseline,
        current,
        covered_delta: i128::from(current.covered) - i128::from(baseline.covered),
        total_delta: i128::from(current.total) - i128::from(baseline.total),
        threshold_basis_points,
        threshold_passed: is_threshold_passed,
        regression_passed: is_regression_passed,
        denominator_passed: is_denominator_passed,
    }
}

fn append_metric_diagnostics(name: &str, comparison: &MetricComparison, diagnostics: &mut Diagnostics) {
    if !comparison.denominator_passed {
        diagnostics.push(format!("coverage {name} denominator changed under require_equal policy"));
    }
    if !comparison.regression_passed {
        diagnostics.push(format!("coverage {name} regressed"));
    }
    if !comparison.threshold_passed {
        diagnostics.push(format!("coverage {name} is below its threshold"));
    }
}

const fn metric_passed(comparison: &MetricComparison) -> bool {
    comparison.threshold_passed && comparison.regression_passed && comparison.denominator_passed
}
