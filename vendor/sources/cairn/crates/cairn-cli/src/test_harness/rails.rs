use cairn_core::blake3_hex;
use serde_json::json;
use std::{error::Error, fs, path::Path, process::Command};

use super::model::{Class, Outcome, Report};

pub(super) fn run_real_rail(
    root: &Path,
    rail: &str,
    out: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let report = real_rail_report(root, rail);
    let receipt = crate::support::with_portable_receipt_hash(
        root,
        json!({
            "schema":"cairn.test-harness.rail-receipt.v1",
            "command":"test-harness",
            "rail_id":rail,
            "root":".",
            "outcome":report.outcome.s(),
            "failure_class":report.class.s(),
            "issue_code":report.code,
            "diagnostic":report.diag,
            "replay_command":format!("cairn test-harness --root . --rail {rail}"),
            "rail":report.attempts.first().cloned().unwrap_or_else(|| json!({"rail_id":rail})),
        }),
    );
    let rendered = serde_json::to_string_pretty(&receipt)?;
    if let Some(out) = out {
        let p = root.join(out);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(
            p,
            format!(
                "{rendered}
"
            ),
        )?;
    }
    println!("{rendered}");
    if report.outcome == Outcome::Passed {
        Ok(())
    } else {
        Err(format!("test harness rail failed: {}", report.code).into())
    }
}

pub(super) fn real_rail_report(root: &Path, rail: &str) -> Report {
    let Some(args) = rail_args(rail) else {
        return Report {
            outcome: Outcome::Failed,
            class: Class::Harness,
            code: "unknown_harness_rail",
            diag: "choose a supported real rail id",
            attempts: vec![json!({"rail_id":rail,"status":"not_launched"})],
        };
    };
    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(_) => {
            return Report {
                outcome: Outcome::Failed,
                class: Class::Environment,
                code: "rail_launcher_unavailable",
                diag: "current executable path is unavailable",
                attempts: vec![json!({"rail_id":rail,"status":"not_launched"})],
            };
        }
    };
    let output = Command::new(exe).args(&args).current_dir(root).output();
    match output {
        Ok(output) => rail_output_report(
            rail,
            &args,
            output.status.code(),
            &output.stdout,
            &output.stderr,
        ),
        Err(_) => Report {
            outcome: Outcome::Failed,
            class: Class::Environment,
            code: "rail_launch_failed",
            diag: "the selected rail could not be launched in this environment",
            attempts: vec![json!({"rail_id":rail,"argv":args,"status":"launch_failed"})],
        },
    }
}

pub(super) fn rail_output_report(
    rail: &str,
    args: &[String],
    exit_code: Option<i32>,
    stdout: &[u8],
    stderr: &[u8],
) -> Report {
    let passed = exit_code == Some(0);
    Report {
        outcome: if passed {
            Outcome::Passed
        } else {
            Outcome::Failed
        },
        class: if passed { Class::None } else { Class::Product },
        code: if passed { "ok" } else { "rail_failed" },
        diag: if passed {
            "real verification rail passed"
        } else {
            "debug the selected verification rail output before changing harness plumbing"
        },
        attempts: vec![json!({
            "rail_id":rail,
            "argv":args,
            "status":if passed {"passed"} else {"failed"},
            "exit_code":exit_code,
            "stdout_blake3":blake3_hex(stdout),
            "stderr_blake3":blake3_hex(stderr),
            "stdout_bytes":stdout.len(),
            "stderr_bytes":stderr.len(),
        })],
    }
}

pub(super) fn rail_args(rail: &str) -> Option<Vec<String>> {
    let args = match rail {
        "validate" => vec!["validate", "--root", "."],
        "traceability-coverage" | "tracey-coverage" => {
            vec!["traceability", "coverage", "--root", "."]
        }
        "release-readiness" => vec!["release-readiness", "--root", "."],
        "policy-export-check" => vec!["policy", "export", "--root", ".", "--check"],
        _ => return None,
    };
    Some(args.into_iter().map(str::to_string).collect())
}
