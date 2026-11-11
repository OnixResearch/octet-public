use std::{error::Error, fs};

use cairn_core::positive_oxide_lifecycle_patterns_fixture;
use serde_json::Value;

#[allow(dead_code)]
#[path = "release_receipts/support.rs"]
mod support;

use support::{cairn, run_json_command, run_json_failure_command, temp_project};

const EXPECTED_BUNDLE_MEMBER_COUNT: u64 = 1;

#[test]
fn lifecycle_patterns_cli_accepts_reviewed_bundle() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_orchestration.oxide_patterns.bundle_inspector.list]
    let temp = temp_project("oxide-patterns-positive")?;
    let input = temp.join("patterns.json");
    fs::write(
        &input,
        serde_json::to_vec_pretty(&positive_oxide_lifecycle_patterns_fixture())?,
    )?;
    let input_text = input.display().to_string();
    let mut command = cairn();
    command.args(["lifecycle-patterns", "validate", "--input", &input_text]);
    let receipt = run_json_command(&mut command)?;
    assert_eq!(receipt["valid"], true);
    assert_eq!(receipt["verdict"], "pass");
    assert_eq!(
        receipt["bundle_member_count"].as_u64(),
        Some(EXPECTED_BUNDLE_MEMBER_COUNT)
    );
    Ok(())
}

#[test]
fn lifecycle_patterns_cli_rejects_corrupt_bundle() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_orchestration.oxide_patterns.bundle_inspector.corrupt]
    let temp = temp_project("oxide-patterns-negative")?;
    let input = temp.join("patterns.json");
    let mut value = positive_oxide_lifecycle_patterns_fixture();
    value["bundle_members"][0]["observed_hash"] = Value::String(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
    );
    fs::write(&input, serde_json::to_vec_pretty(&value)?)?;
    let input_text = input.display().to_string();
    let mut command = cairn();
    command.args(["lifecycle-patterns", "validate", "--input", &input_text]);
    let receipt = run_json_failure_command(&mut command)?;
    assert_eq!(receipt["valid"], false);
    assert!(receipt["issues"].as_array().is_some_and(|issues| issues
        .iter()
        .any(|issue| issue["code"] == "oxide_patterns.bundle_hash_mismatch")));
    Ok(())
}
