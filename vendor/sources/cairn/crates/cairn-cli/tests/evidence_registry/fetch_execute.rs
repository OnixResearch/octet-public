use std::{error::Error, fs};

use serde_json::Value;

use super::support::{
    assert_hash_field, assert_schema_keys, copy_dir, receipt_schema, run_json_failure_strings,
    run_json_strings, EvidenceChain, RegistryChain,
};

pub(crate) fn verify(
    chain: &EvidenceChain,
    registry_chain: &RegistryChain,
) -> Result<(), Box<dyn Error>> {
    let dir = &chain.dir;
    let republish_dir = &chain.republish_dir;
    let fetch_plan_path = &registry_chain.fetch_plan_path;
    let fetch_plan = &registry_chain.fetch_plan;
    let registry = &registry_chain.registry;
    let fetch_execute_dir = dir.join("materialized-fetch");
    let fetch_execute = run_json_strings(&[
        "evidence".into(),
        "registry".into(),
        "fetch-execute".into(),
        "--root".into(),
        ".".into(),
        "--plan".into(),
        fetch_plan_path.display().to_string(),
        "--source".into(),
        republish_dir.display().to_string(),
        "--out".into(),
        fetch_execute_dir.display().to_string(),
    ])?;
    assert_eq!(fetch_execute["valid"], true);
    assert_eq!(fetch_execute["verdict"], "pass");
    assert_eq!(fetch_execute["plan"], "<plan>");
    assert_eq!(fetch_execute["source"], "<source>");
    assert_eq!(fetch_execute["out"], "<out>");
    assert_eq!(
        fetch_execute["fetch_plan_receipt_hash"],
        fetch_plan["receipt_hash"]
    );
    assert_eq!(
        fetch_execute["registry_receipt_hash"],
        registry["receipt_hash"]
    );
    let planned_artifact_count = fetch_plan["entries"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["artifacts"].as_array().unwrap().len())
        .sum::<usize>();
    assert_eq!(fetch_execute["materialized_count"], planned_artifact_count);
    for artifact in fetch_execute["materialized"].as_array().unwrap() {
        assert_hash_field(artifact, "expected_hash")?;
        assert_hash_field(artifact, "observed_hash")?;
        assert_eq!(artifact["expected_hash"], artifact["observed_hash"]);
        let path = artifact["path"].as_str().unwrap();
        assert!(fetch_execute_dir.join(path).exists());
    }
    assert_hash_field(&fetch_execute, "receipt_hash")?;
    let fetch_execute_schema = receipt_schema("evidence registry fetch-execute")?;
    assert_schema_keys(&fetch_execute, &fetch_execute_schema, "top_level_keys")?;
    // r[cairn.supersede.fetch_execute_policy_docs.policy.schema]
    // r[cairn.supersede.fetch_execute_policy_docs.policy.inventory]
    let fetch_execute_text = serde_json::to_string(&fetch_execute)?;
    assert!(
        !fetch_execute_text.contains(&dir.display().to_string()),
        "{fetch_execute_text}"
    );

    let mut unsafe_fetch_plan = fetch_plan.clone();
    unsafe_fetch_plan["entries"][0]["artifacts"][0]["path"] = Value::String("../escape".into());
    let unsafe_fetch_plan_path = dir.join("unsafe-fetch-plan.json");
    fs::write(
        &unsafe_fetch_plan_path,
        serde_json::to_string_pretty(&unsafe_fetch_plan)? + "\n",
    )?;
    let unsafe_fetch_execute = run_json_failure_strings(&[
        "evidence".into(),
        "registry".into(),
        "fetch-execute".into(),
        "--root".into(),
        ".".into(),
        "--plan".into(),
        unsafe_fetch_plan_path.display().to_string(),
        "--source".into(),
        republish_dir.display().to_string(),
        "--out".into(),
        dir.join("unsafe-materialized-fetch").display().to_string(),
    ])?;
    assert_eq!(unsafe_fetch_execute["valid"], false);
    let unsafe_fetch_codes = unsafe_fetch_execute["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(unsafe_fetch_codes.contains(&"evidence_registry_fetch_execute.artifact_path_unsafe"));

    let mut unverified_fetch_plan = fetch_plan.clone();
    unverified_fetch_plan["valid"] = Value::Bool(false);
    let unverified_fetch_plan_path = dir.join("unverified-fetch-plan.json");
    fs::write(
        &unverified_fetch_plan_path,
        serde_json::to_string_pretty(&unverified_fetch_plan)? + "\n",
    )?;
    let unverified_fetch_execute = run_json_failure_strings(&[
        "evidence".into(),
        "registry".into(),
        "fetch-execute".into(),
        "--root".into(),
        ".".into(),
        "--plan".into(),
        unverified_fetch_plan_path.display().to_string(),
        "--source".into(),
        republish_dir.display().to_string(),
        "--out".into(),
        dir.join("unverified-materialized-fetch")
            .display()
            .to_string(),
    ])?;
    assert_eq!(unverified_fetch_execute["valid"], false);
    let unverified_fetch_codes = unverified_fetch_execute["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(unverified_fetch_codes.contains(&"evidence_registry_fetch_execute.plan_unverified"));

    let missing_fetch_source = dir.join("missing-fetch-source");
    copy_dir(republish_dir, &missing_fetch_source)?;
    let first_artifact = fetch_plan["entries"][0]["artifacts"][0]["path"]
        .as_str()
        .unwrap();
    fs::remove_file(missing_fetch_source.join(first_artifact))?;
    let missing_fetch_execute = run_json_failure_strings(&[
        "evidence".into(),
        "registry".into(),
        "fetch-execute".into(),
        "--root".into(),
        ".".into(),
        "--plan".into(),
        fetch_plan_path.display().to_string(),
        "--source".into(),
        missing_fetch_source.display().to_string(),
        "--out".into(),
        dir.join("missing-materialized-fetch").display().to_string(),
    ])?;
    assert_eq!(missing_fetch_execute["valid"], false);
    let missing_fetch_codes = missing_fetch_execute["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(missing_fetch_codes.contains(&"evidence_registry_fetch_execute.artifact_missing"));

    let tampered_fetch_source = dir.join("tampered-fetch-source");
    copy_dir(republish_dir, &tampered_fetch_source)?;
    let tampered_artifact = fetch_plan["entries"][0]["artifacts"][0]["path"]
        .as_str()
        .unwrap();
    fs::write(tampered_fetch_source.join(tampered_artifact), b"tampered")?;
    let tampered_fetch_execute = run_json_failure_strings(&[
        "evidence".into(),
        "registry".into(),
        "fetch-execute".into(),
        "--root".into(),
        ".".into(),
        "--plan".into(),
        fetch_plan_path.display().to_string(),
        "--source".into(),
        tampered_fetch_source.display().to_string(),
        "--out".into(),
        dir.join("tampered-materialized-fetch")
            .display()
            .to_string(),
    ])?;
    assert_eq!(tampered_fetch_execute["valid"], false);
    let tampered_fetch_codes = tampered_fetch_execute["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(
        tampered_fetch_codes.contains(&"evidence_registry_fetch_execute.artifact_hash_mismatch")
    );
    Ok(())
}
