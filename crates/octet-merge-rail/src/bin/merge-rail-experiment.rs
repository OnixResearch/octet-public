//! Run the reviewed bounded merge corpus and print a deterministic report.

use std::process::ExitCode;

use octet_merge_rail::corpus::run_experiment;

fn main() -> ExitCode {
    let report = run_experiment();
    match serde_json::to_string_pretty(&report) {
        Ok(text) => println!("{text}"),
        Err(error) => {
            eprintln!("serialize merge-rail experiment report: {error}");
            return ExitCode::from(2);
        }
    }
    if report.metrics.matched_expected == report.metrics.cases {
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "{} of {} corpus cases did not match the independent expectation",
            report.metrics.cases - report.metrics.matched_expected,
            report.metrics.cases
        );
        ExitCode::FAILURE
    }
}
