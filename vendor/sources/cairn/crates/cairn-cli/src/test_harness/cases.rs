use crate::support::safe_relative_path;
use serde_json::json;
use std::{error::Error, fs, path::Path};

use super::{
    artifacts::write_artifact,
    model::{Class, Outcome, Report},
};

pub(super) fn case_report(
    root: &Path,
    case: &str,
    seed: &str,
    max: usize,
    artifact: &Path,
) -> Result<Report, Box<dyn Error>> {
    if let Some(parent) = artifact.parent() {
        fs::create_dir_all(parent)?;
    }
    match case {
        "pass" => {
            write_artifact(artifact, case, seed, "ok")?;
            Ok(Report {
                outcome: Outcome::Passed,
                class: Class::None,
                code: "ok",
                diag: "harness case passed and artifact integrity verified",
                attempts: vec![json!({"attempt":1,"status":"passed"})],
            })
        }
        "product-fail" => {
            write_artifact(artifact, case, seed, "product_assertion_failed")?;
            Ok(Report {
                outcome: Outcome::Failed,
                class: Class::Product,
                code: "product_invariant_failed",
                diag: "debug the product invariant before changing harness plumbing",
                attempts: vec![json!({"attempt":1,"status":"failed"})],
            })
        }
        "harness-fail" => {
            fs::write(
                artifact,
                b"not-json
",
            )?;
            Ok(Report {
                outcome: Outcome::Failed,
                class: Class::Harness,
                code: "harness_artifact_invalid",
                diag: "debug the harness artifact contract",
                attempts: vec![json!({"attempt":1,"status":"failed"})],
            })
        }
        "environment-fail" => Ok(Report {
            outcome: Outcome::Failed,
            class: Class::Environment,
            code: "environment_precondition_missing",
            diag: "satisfy the local precondition before debugging product behavior",
            attempts: vec![
                json!({"attempt":1,"status":"failed","precondition":safe_relative_path(root,&root.join("target/cairn-test-harness"))}),
            ],
        }),
        "timeout-flake" => {
            write_artifact(artifact, case, seed, "timeout_then_passed")?;
            let attempts = (0..=max)
                .map(|i| json!({"attempt":i+1,"status":if i==0{"timeout"}else{"passed"}}))
                .collect();
            Ok(Report {
                outcome: Outcome::Flaky,
                class: Class::Flaky,
                code: "flaky_or_timeout_reproduced",
                diag: "promote this case identity to a regression fixture if it repeats",
                attempts,
            })
        }
        _ => Ok(Report {
            outcome: Outcome::Failed,
            class: Class::Harness,
            code: "unknown_harness_case",
            diag: "choose a supported case id",
            attempts: vec![json!({"attempt":1,"status":"failed"})],
        }),
    }
}
