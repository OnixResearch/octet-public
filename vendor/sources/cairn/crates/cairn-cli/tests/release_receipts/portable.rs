use super::support::*;
use std::error::Error;

#[test]
fn portable_receipt_hashes_survive_distinct_roots() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.blake3_receipts.lifecycle_plans]
    // r[verify cairn.supersede.portable_receipts] r[verify cairn.supersede.portable_receipts.cross_root]
    // r[verify cairn.supersede.portable_receipts.display_paths] r[verify cairn.supersede.portable_receipts.explicit_absolute_inputs]
    // r[verify cairn.supersede.determinism_replay] r[verify cairn.supersede.determinism_replay.cross_root]
    // r[verify cairn.supersede.determinism_replay.repeated]
    let first_root = temp_fixture("portable-a", "fixtures/basic")?;
    let second_root = temp_fixture("portable-b", "fixtures/basic")?;
    let first = first_root.display().to_string();
    let second = second_root.display().to_string();

    let gate_a = run_json_strings(&[
        "gate".into(),
        "tasks".into(),
        "demo".into(),
        "--root".into(),
        first.clone(),
    ])?;
    let gate_b = run_json_strings(&[
        "gate".into(),
        "tasks".into(),
        "demo".into(),
        "--root".into(),
        second.clone(),
    ])?;
    assert_hash_fields_equal(&gate_a, &gate_b, &["input_hash", "receipt_hash"])?;

    let sync_a = run_json_strings(&["sync".into(), "demo".into(), "--root".into(), first.clone()])?;
    let sync_b = run_json_strings(&[
        "sync".into(),
        "demo".into(),
        "--root".into(),
        second.clone(),
    ])?;
    assert_hash_fields_equal(
        &sync_a,
        &sync_b,
        &["input_hash", "plan_hash", "receipt_hash"],
    )?;

    let migrate_a =
        run_json_strings(&["migrate".into(), "openspec".into(), "--root".into(), first])?;
    let migrate_b =
        run_json_strings(&["migrate".into(), "openspec".into(), "--root".into(), second])?;
    assert_hash_fields_equal(
        &migrate_a,
        &migrate_b,
        &[
            "source_manifest_hash",
            "destination_manifest_hash",
            "manifest_hash",
            "plan_hash",
            "receipt_hash",
        ],
    )
}
