use super::*;

const SOURCE_IDENTITY: &str = "source:b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const OTHER_SOURCE_IDENTITY: &str = "source:b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const BASELINE_COVERED: u64 = 90;
const CURRENT_COVERED: u64 = 92;
const TOTAL_COUNT: u64 = 100;
const EXPECTED_COVERED_DELTA: i128 = 2;
const EXPECTED_CURRENT_BASIS_POINTS: u64 = 9_200;
const PRECISION_NUMERATOR: u64 = 2;
const PRECISION_DENOMINATOR: u64 = 3;
const EXPECTED_PRECISION_BASIS_POINTS: u64 = 6_666;
const OWNER_ONLY_PERMISSIONS: u32 = 0o700;
const SUPPLIED_PERCENTAGE: f64 = 100.0;
const COVERED_FIELD: &str = "\"covered\":90";
const PERCENTAGE_ONLY_FIELD: &str = "\"percent\":90.0";
const TINY_LOG_LIMIT: usize = 2;
const TOOL_FAILURE_EXIT: i32 = 7;

#[test]
fn profile_registry_and_identity_are_fresh() {
    // r[verify octet.coverage_baseline.profile]
    assert_eq!(prefixed_identity(PROFILE_JSON.as_bytes()), PROFILE_IDENTITY);
    let profile = selected_profile(PROFILE_ID).expect("selected coverage profile");
    assert_eq!(profile.provider.package, "cargo-llvm-cov");
    assert_eq!(profile.provider.version, "0.8.3");
    assert_eq!(profile.metrics, ["lines", "branches", "functions", "regions"]);
    assert!(profile.command_argv.contains(&"--output-path".to_owned()));
    assert!(profile.command_argv.contains(&"--offline".to_owned()));
}

#[test]
fn stable_and_improved_counts_compare_deterministically() {
    // r[verify octet.coverage_baseline.native_artifacts]
    // r[verify octet.coverage_baseline.comparison]
    let baseline_raw = native_report(BASELINE_COVERED, TOTAL_COUNT, "cargo-octet/src/lib.rs");
    let baseline = baseline_envelope(&baseline_raw, SOURCE_IDENTITY);
    let current = native_report(CURRENT_COVERED, TOTAL_COUNT, "cargo-octet/src/lib.rs");
    let first = evaluate_profiled(&current, &baseline, PROFILE_ID, SOURCE_IDENTITY);
    let second = evaluate_profiled(&current, &baseline, PROFILE_ID, SOURCE_IDENTITY);
    assert_eq!(first, second);
    let details = first.details.expect("coverage receipt details");
    assert!(first.diagnostics.is_empty(), "{:?}", first.diagnostics);
    assert!(details.comparison.passed);
    assert_eq!(details.comparison.lines.covered_delta, EXPECTED_COVERED_DELTA);
    assert_eq!(details.comparison.lines.current.percentage_basis_points, EXPECTED_CURRENT_BASIS_POINTS);
    assert_eq!(details.current.source_identity, SOURCE_IDENTITY);
}

#[test]
fn regression_denominator_and_scope_mismatch_fail_closed() {
    let baseline_raw = native_report(BASELINE_COVERED, TOTAL_COUNT, "cargo-octet/src/lib.rs");
    let baseline = baseline_envelope(&baseline_raw, SOURCE_IDENTITY);
    let regression = native_report(BASELINE_COVERED - 1, TOTAL_COUNT, "cargo-octet/src/lib.rs");
    let result = evaluate_profiled(&regression, &baseline, PROFILE_ID, SOURCE_IDENTITY);
    assert!(result.details.is_some());
    assert!(result.diagnostics.iter().any(|value| value.contains("regressed")));

    let denominator = native_report(BASELINE_COVERED, TOTAL_COUNT + 1, "cargo-octet/src/lib.rs");
    let result = evaluate_profiled(&denominator, &baseline, PROFILE_ID, SOURCE_IDENTITY);
    assert!(result.diagnostics.iter().any(|value| value.contains("denominator")));

    let other_baseline = baseline_envelope(&baseline_raw, OTHER_SOURCE_IDENTITY);
    let result = evaluate_profiled(&baseline_raw, &other_baseline, PROFILE_ID, SOURCE_IDENTITY);
    assert!(result.diagnostics.iter().any(|value| value.contains("source identity")));
}

#[test]
fn malformed_percentage_only_unsafe_stale_and_limited_reports_fail() {
    // r[verify octet.coverage_baseline.verification]
    let baseline_raw = native_report(BASELINE_COVERED, TOTAL_COUNT, "cargo-octet/src/lib.rs");
    let baseline = baseline_envelope(&baseline_raw, SOURCE_IDENTITY);
    let percentages = baseline_raw.replace(COVERED_FIELD, PERCENTAGE_ONLY_FIELD);
    assert!(evaluate_profiled(&percentages, &baseline, PROFILE_ID, SOURCE_IDENTITY).details.is_none());

    let unsafe_report = native_report(BASELINE_COVERED, TOTAL_COUNT, "../escape.rs");
    assert!(evaluate_profiled(&unsafe_report, &baseline, PROFILE_ID, SOURCE_IDENTITY).details.is_none());

    let mut stale: NativeEnvelope = serde_json::from_str(&baseline).expect("parse baseline envelope");
    stale.report.counts.lines.covered = BASELINE_COVERED - 1;
    let stale = serde_json::to_string(&stale).expect("serialize stale baseline");
    assert!(evaluate_profiled(&baseline_raw, &stale, PROFILE_ID, SOURCE_IDENTITY).details.is_none());

    let mut profile = selected_profile(PROFILE_ID).expect("coverage profile");
    profile.max_report_bytes = 1;
    assert!(native::normalize(&baseline_raw, &profile, SOURCE_IDENTITY).is_err());
}

#[test]
fn exact_integer_precision_does_not_use_supplied_percentages() {
    let report = native_report(PRECISION_NUMERATOR, PRECISION_DENOMINATOR, "cargo-octet/src/lib.rs");
    let profile = selected_profile(PROFILE_ID).expect("coverage profile");
    let normalized = native::normalize(&report, &profile, SOURCE_IDENTITY).expect("normalize exact counts");
    assert_eq!(normalized.counts.lines.percentage_basis_points, EXPECTED_PRECISION_BASIS_POINTS);
}

#[cfg(unix)]
#[test]
fn bounded_native_run_uses_fixed_argv_and_dedicated_report() {
    // r[verify octet.coverage_baseline.profile]
    // r[verify octet.coverage_baseline.boundary]
    use std::os::unix::fs::PermissionsExt as _;

    let directory = tempfile::tempdir().expect("create coverage shell fixture");
    let report = native_report(CURRENT_COVERED, TOTAL_COUNT, "cargo-octet/src/lib.rs");
    let script = directory.path().join("cargo-fixture");
    let config = directory.path().join("cargo-config.toml");
    std::fs::write(&config, "[net]\noffline = true\n").expect("write Cargo config fixture");
    std::fs::write(&script, format!("#!/bin/sh\nprintf '%s' '{report}' > .octet/coverage/current-native.json\n"))
        .expect("write coverage executable fixture");
    let mut permissions = std::fs::metadata(&script).expect("inspect fixture executable").permissions();
    permissions.set_mode(OWNER_ONLY_PERMISSIONS);
    std::fs::set_permissions(&script, permissions).expect("make fixture executable");

    let profile = selected_profile(PROFILE_ID).expect("coverage profile");
    let observation = shell::run_with(directory.path(), &profile, shell::RuntimeTools {
        cargo: script.clone(),
        path: "/bin".into(),
        cargo_config: config.clone(),
        library_path: "/lib".into(),
    })
    .expect("run bounded coverage fixture");
    assert_eq!(observation.current_text, report);
    assert_eq!(observation.exit_code, 0);

    std::fs::write(&script, "#!/bin/sh\nprintf 'overflow'\n").expect("write output-limit fixture");
    let mut limited_profile = profile.clone();
    limited_profile.max_log_bytes = TINY_LOG_LIMIT;
    let error = shell::run_with(directory.path(), &limited_profile, shell::RuntimeTools {
        cargo: script.clone(),
        path: "/bin".into(),
        cargo_config: config.clone(),
        library_path: "/lib".into(),
    })
    .expect_err("coverage output limit is rejected");
    assert!(error.contains("did not succeed"));

    std::fs::write(&script, format!("#!/bin/sh\nexit {TOOL_FAILURE_EXIT}\n")).expect("write tool-failure fixture");
    let error = shell::run_with(directory.path(), &profile, shell::RuntimeTools {
        cargo: script,
        path: "/bin".into(),
        cargo_config: config,
        library_path: "/lib".into(),
    })
    .expect_err("coverage tool failure is rejected");
    assert!(error.contains("did not succeed"));
}

fn baseline_envelope(raw_report: &str, source_identity: &str) -> String {
    let profile = selected_profile(PROFILE_ID).expect("coverage profile");
    let report = native::normalize(raw_report, &profile, source_identity).expect("normalize baseline report");
    String::from_utf8(envelope_bytes(&report, raw_report).expect("render baseline envelope"))
        .expect("baseline envelope is UTF-8")
}

fn native_report(covered: u64, total: u64, path: &str) -> String {
    serde_json::to_string(&serde_json::json!({
        "data": [{
            "files": [{"filename": path, "summary": {}}],
            "totals": {
                "branches": {"count": total, "covered": covered, "percent": SUPPLIED_PERCENTAGE},
                "functions": {"count": total, "covered": covered, "percent": SUPPLIED_PERCENTAGE},
                "lines": {"count": total, "covered": covered, "percent": SUPPLIED_PERCENTAGE},
                "regions": {"count": total, "covered": covered, "percent": SUPPLIED_PERCENTAGE}
            }
        }],
        "type": "llvm.coverage.json.export",
        "version": "2.0.1"
    }))
    .expect("serialize native coverage fixture")
}
