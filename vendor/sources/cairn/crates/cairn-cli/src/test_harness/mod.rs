// r[impl test_harness.receipts_replay.receipts] r[impl test_harness.receipts_replay.receipts.schema]
// r[impl test_harness.receipts_replay.receipts.portable_hash] r[impl test_harness.receipts_replay.replay]
// r[impl test_harness.receipts_replay.replay.failure_command] r[impl test_harness.receipts_replay.replay.regression_fixture]
// r[impl test_harness.failure_taxonomy.classification] r[impl test_harness.failure_taxonomy.classification.product]
// r[impl test_harness.failure_taxonomy.classification.harness] r[impl test_harness.failure_taxonomy.classification.environment]
// r[impl test_harness.failure_taxonomy.classification.flake] r[impl test_harness.failure_taxonomy.usability]
// r[impl test_harness.failure_taxonomy.usability.summary] r[impl test_harness.fixture_artifact_integrity.artifacts]
// r[impl test_harness.fixture_artifact_integrity.artifacts.existence] r[impl test_harness.fixture_artifact_integrity.artifacts.schema_hash]
// r[impl test_harness.fixture_artifact_integrity.artifacts.corrupt] r[impl test_harness.fixture_artifact_integrity.fixtures]
// r[impl test_harness.fixture_artifact_integrity.fixtures.canonical] r[impl test_harness.fixture_artifact_integrity.fixtures.golden_update]
// r[impl test_harness.rail_split_selftest.rails] r[impl test_harness.rail_split_selftest.rails.fast]
// r[impl test_harness.rail_split_selftest.rails.full] r[impl test_harness.rail_split_selftest.self_tests]
// r[impl test_harness.rail_split_selftest.self_tests.intentional_failure] r[impl test_harness.rail_split_selftest.self_tests.intentional_success]
// r[impl test_harness.flake_triage.rerun] r[impl test_harness.flake_triage.rerun.bounded]
// r[impl test_harness.flake_triage.rerun.not_greenwash] r[impl test_harness.flake_triage.promotion]
// r[impl test_harness.flake_triage.promotion.evidence] r[impl test_harness.flake_triage.promotion.fixture]
// r[impl test_harness.real_rails.receipts] r[impl test_harness.real_rails.receipts.validate]
// r[impl test_harness.real_rails.receipts.tracey] r[impl test_harness.real_rails.receipts.release_readiness]
// r[impl test_harness.real_rails.classification] r[impl test_harness.real_rails.classification.product]
// r[impl test_harness.real_rails.classification.environment] r[impl test_harness.real_rails.selection]
// r[impl test_harness.real_rails.selection.known] r[impl test_harness.real_rails.selection.unknown]
// r[impl cli_maintainability.test_harness_submodule_split.focused_modules]
// r[impl cli_maintainability.test_harness_submodule_split.focused_modules.visible_boundaries]
// r[impl cli_maintainability.test_harness_submodule_split.focused_modules.narrow_bridges]
// r[impl cli_maintainability.test_harness_submodule_split.behavior_preserved]
// r[impl cli_maintainability.test_harness_submodule_split.behavior_preserved.focused_checks]
// r[impl cli_maintainability.test_harness_submodule_split.behavior_preserved.repository_checks]
mod artifacts;
mod cases;
mod model;
mod rails;
mod receipts;

#[cfg(test)]
mod tests;

use crate::support::read_string_option;
use std::error::Error;

use cases::case_report;
use rails::run_real_rail;
use receipts::render_case_receipt;

pub(crate) fn run_test_harness(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = crate::support::read_option(args.clone(), "--root", ".");
    if let Some(rail) = read_string_option(&args, "--rail") {
        return run_real_rail(&root, &rail, read_string_option(&args, "--out"));
    }
    let case = read_string_option(&args, "--case").unwrap_or_else(|| "pass".into());
    let seed = read_string_option(&args, "--seed").unwrap_or_else(|| "0".into());
    let max = read_string_option(&args, "--max-reruns")
        .and_then(|x| x.parse::<usize>().ok())
        .unwrap_or(2);
    let out = read_string_option(&args, "--out");
    let artifact = root
        .join("target/cairn-test-harness")
        .join(format!("{case}-{seed}.json"));
    let report = case_report(&root, &case, &seed, max, &artifact)?;
    let rendered = render_case_receipt(&root, &case, &seed, max, &artifact, &report)?;
    if let Some(out) = out {
        let p = root.join(out);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(
            p,
            format!(
                "{rendered}
"
            ),
        )?;
    }
    println!("{rendered}");
    if report.outcome == model::Outcome::Passed {
        Ok(())
    } else {
        Err(format!("test harness case failed: {}", report.code).into())
    }
}
