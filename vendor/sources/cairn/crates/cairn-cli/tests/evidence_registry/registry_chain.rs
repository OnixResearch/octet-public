use std::{error::Error, fs};

use serde_json::Value;

use super::support::{
    assert_hash_field, run_json_failure_strings, run_json_strings, EvidenceChain, RegistryChain,
};

pub(crate) fn verify(chain: &EvidenceChain) -> Result<RegistryChain, Box<dyn Error>> {
    let dir = &chain.dir;
    let republish_dir = &chain.republish_dir;
    let index_path = &chain.index_path;
    let index = &chain.index;
    let republish = &chain.republish;
    let registry_path = republish_dir.join("evidence-registry-receipt.json");
    let registry = run_json_strings(&[
        "evidence".into(),
        "registry".into(),
        "build".into(),
        "--root".into(),
        ".".into(),
        "--index".into(),
        index_path.display().to_string(),
        "--out".into(),
        registry_path.display().to_string(),
        "--channel".into(),
        "stable".into(),
        "--release".into(),
        "2026-05-19".into(),
        "--platform".into(),
        "x86_64-linux".into(),
        "--compatibility-class".into(),
        "current".into(),
    ])?;
    assert_eq!(registry["valid"], true);
    assert_eq!(registry["verdict"], "pass");
    assert_eq!(registry["out"], "<out>");
    assert_eq!(registry["entry_count"], 1);
    assert_hash_field(&registry, "receipt_hash")?;
    let registry_entry = &registry["entries"][0];
    assert_eq!(registry_entry["index_receipt_hash"], index["receipt_hash"]);
    assert_eq!(registry_entry["bundle_kind"], "republished");
    assert_eq!(registry_entry["labels"]["channel"], "stable");
    assert_eq!(registry_entry["labels"]["release"], "2026-05-19");
    assert_eq!(registry_entry["labels"]["platform"], "x86_64-linux");
    assert_eq!(registry_entry["labels"]["compatibility_class"], "current");
    assert_eq!(
        registry_entry["bundle_receipt_hash"],
        republish["receipt_hash"]
    );
    assert_eq!(
        registry_entry["index_relative"],
        "evidence-index-receipt.json"
    );
    let saved_registry: Value = serde_json::from_slice(&fs::read(&registry_path)?)?;
    assert_eq!(saved_registry["receipt_hash"], registry["receipt_hash"]);
    let registry_verify = run_json_strings(&[
        "evidence".into(),
        "registry".into(),
        "verify".into(),
        "--root".into(),
        ".".into(),
        "--registry".into(),
        registry_path.display().to_string(),
    ])?;
    let registry_verify_path = republish_dir.join("evidence-registry-verify-receipt.json");
    fs::write(
        &registry_verify_path,
        serde_json::to_string_pretty(&registry_verify)? + "\n",
    )?;
    assert_eq!(registry_verify["valid"], true);
    assert_eq!(registry_verify["verdict"], "pass");
    assert_eq!(registry_verify["registry_hash"], registry["receipt_hash"]);
    assert_eq!(registry_verify["entry_count"], 1);
    assert_hash_field(&registry_verify, "receipt_hash")?;

    let mirror_verify = run_json_strings(&[
        "evidence".into(),
        "registry".into(),
        "mirror-verify".into(),
        "--root".into(),
        ".".into(),
        "--registry".into(),
        registry_path.display().to_string(),
        "--mirror".into(),
        republish_dir.display().to_string(),
    ])?;
    assert_eq!(mirror_verify["valid"], true);
    assert_eq!(mirror_verify["verdict"], "pass");
    assert_eq!(mirror_verify["registry"], "<registry>");
    assert_eq!(mirror_verify["mirror"], "<mirror>");
    assert_eq!(mirror_verify["registry_hash"], registry["receipt_hash"]);
    assert_eq!(
        mirror_verify["registry_verification_receipt_hash"],
        registry_verify["receipt_hash"]
    );
    assert_eq!(mirror_verify["entry_count"], 1);
    assert_eq!(
        mirror_verify["entries"][0]["index_receipt_hash"],
        index["receipt_hash"]
    );
    assert_hash_field(&mirror_verify, "receipt_hash")?;
    let mirror_text = serde_json::to_string(&mirror_verify)?;
    assert!(
        !mirror_text.contains(&dir.display().to_string()),
        "{mirror_text}"
    );

    let missing_mirror = dir.join("missing-index-mirror");
    fs::create_dir_all(&missing_mirror)?;
    fs::copy(
        &registry_path,
        missing_mirror.join("evidence-registry-receipt.json"),
    )?;
    let missing_mirror_failure = run_json_failure_strings(&[
        "evidence".into(),
        "registry".into(),
        "mirror-verify".into(),
        "--root".into(),
        ".".into(),
        "--registry".into(),
        registry_path.display().to_string(),
        "--mirror".into(),
        missing_mirror.display().to_string(),
    ])?;
    assert_eq!(missing_mirror_failure["valid"], false);
    let missing_mirror_codes = missing_mirror_failure["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(missing_mirror_codes.contains(&"evidence_registry_mirror_verify.index_missing"));

    let tampered_mirror = dir.join("tampered-index-mirror");
    fs::create_dir_all(&tampered_mirror)?;
    let tampered_registry_path = tampered_mirror.join("evidence-registry-receipt.json");
    let tampered_index_path = tampered_mirror.join("evidence-index-receipt.json");
    fs::copy(&registry_path, &tampered_registry_path)?;
    fs::copy(index_path, &tampered_index_path)?;
    let mut tampered_index: Value = serde_json::from_slice(&fs::read(&tampered_index_path)?)?;
    tampered_index["receipt_hash"] =
        Value::String("0000000000000000000000000000000000000000000000000000000000000000".into());
    fs::write(
        &tampered_index_path,
        serde_json::to_string_pretty(&tampered_index)? + "\n",
    )?;
    let tampered_mirror_failure = run_json_failure_strings(&[
        "evidence".into(),
        "registry".into(),
        "mirror-verify".into(),
        "--root".into(),
        ".".into(),
        "--registry".into(),
        tampered_registry_path.display().to_string(),
        "--mirror".into(),
        tampered_mirror.display().to_string(),
    ])?;
    assert_eq!(tampered_mirror_failure["valid"], false);
    let tampered_mirror_codes = tampered_mirror_failure["issues"]
        .as_array()
        .unwrap()
        .iter()
        .map(|issue| issue["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(tampered_mirror_codes.contains(&"evidence_registry_mirror_verify.index_hash_mismatch"));

    let selected_index_path = dir.join("selected-cairn-evidence-index.json");
    let selection = run_json_strings(&[
        "evidence".into(),
        "registry".into(),
        "select".into(),
        "--root".into(),
        ".".into(),
        "--registry".into(),
        registry_path.display().to_string(),
        "--bundle-kind".into(),
        "republished".into(),
        "--channel".into(),
        "stable".into(),
        "--release".into(),
        "2026-05-19".into(),
        "--platform".into(),
        "x86_64-linux".into(),
        "--compatibility-class".into(),
        "current".into(),
        "--out".into(),
        selected_index_path.display().to_string(),
    ])?;
    assert_eq!(selection["valid"], true);
    assert_eq!(selection["verdict"], "pass");
    assert_eq!(selection["registry"], "<registry>");
    assert_eq!(selection["out"], "<out>");
    assert_eq!(selection["registry_receipt_hash"], registry["receipt_hash"]);
    assert_eq!(
        selection["registry_verification_receipt_hash"],
        registry_verify["receipt_hash"]
    );
    assert_eq!(selection["selectors"]["bundle_kind"], "republished");
    assert_eq!(selection["selectors"]["channel"], "stable");
    assert_eq!(selection["selectors"]["release"], "2026-05-19");
    assert_eq!(selection["selectors"]["platform"], "x86_64-linux");
    assert_eq!(selection["selectors"]["compatibility_class"], "current");
    assert_eq!(
        selection["selected_entry"]["index_receipt_hash"],
        index["receipt_hash"]
    );
    assert_hash_field(&selection, "receipt_hash")?;
    let selection_receipt_path = dir.join("evidence-registry-selection-receipt.json");
    fs::write(
        &selection_receipt_path,
        serde_json::to_string_pretty(&selection)? + "\n",
    )?;
    let selected_index: Value = serde_json::from_slice(&fs::read(&selected_index_path)?)?;
    assert_eq!(selected_index["receipt_hash"], index["receipt_hash"]);
    let selection_text = serde_json::to_string(&selection)?;
    assert!(
        !selection_text.contains(&dir.display().to_string()),
        "{selection_text}"
    );

    let fetch_plan_path = dir.join("cairn-evidence-fetch-plan.json");
    let fetch_plan = run_json_strings(&[
        "evidence".into(),
        "registry".into(),
        "fetch-plan".into(),
        "--root".into(),
        ".".into(),
        "--registry".into(),
        registry_path.display().to_string(),
        "--channel".into(),
        "stable".into(),
        "--platform".into(),
        "x86_64-linux".into(),
        "--out".into(),
        fetch_plan_path.display().to_string(),
    ])?;
    assert_eq!(fetch_plan["valid"], true);
    assert_eq!(fetch_plan["verdict"], "pass");
    assert_eq!(fetch_plan["registry"], "<registry>");
    assert_eq!(fetch_plan["out"], "<out>");
    assert_eq!(
        fetch_plan["registry_receipt_hash"],
        registry["receipt_hash"]
    );
    assert_eq!(fetch_plan["entry_count"], 1);
    assert_eq!(fetch_plan["selectors"]["channel"], "stable");
    assert_eq!(fetch_plan["selectors"]["platform"], "x86_64-linux");
    assert_eq!(
        fetch_plan["entries"][0]["index_receipt_hash"],
        index["receipt_hash"]
    );
    assert_hash_field(&fetch_plan, "receipt_hash")?;
    let saved_fetch_plan: Value = serde_json::from_slice(&fs::read(&fetch_plan_path)?)?;
    assert_eq!(saved_fetch_plan["receipt_hash"], fetch_plan["receipt_hash"]);
    Ok(RegistryChain {
        registry_path,
        registry_verify_path,
        selection_receipt_path,
        fetch_plan_path,
        registry,
        fetch_plan,
    })
}
