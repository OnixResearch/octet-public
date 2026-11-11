use std::{error::Error, fs};

use serde_json::Value;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[cfg(unix)]
const EXECUTABLE_TEST_MODE: u32 = 0o750;
#[cfg(unix)]
const PERMISSION_MODE_MASK: u32 = 0o777;

use super::support::{assert_hash_field, cairn, ironspec_alias, run_json, temp_fixture};

fn migration_plan_hash(value: &Value) -> Result<&str, Box<dyn Error>> {
    value["plan_hash"]
        .as_str()
        .ok_or("migration plan hash".into())
}

fn execute_migration(source: &str, root: &str, plan_hash: &str) -> Result<Value, Box<dyn Error>> {
    run_json(&[
        "migrate",
        source,
        "--root",
        root,
        "--execute",
        "--plan-hash",
        plan_hash,
    ])
}

#[test]
fn migrate_execute_copies_legacy_to_native_and_prefers_native() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.migrate_openspec] r[verify cairn.supersede.migrate_openspec.execute]
    // r[verify cairn.supersede.migrate_openspec.dry_run] r[verify cairn.supersede.native_layout.prefer]
    // r[verify cairn.supersede.native_layout.prefer_cairn]
    // r[verify cairn.supersede.native_layout.native_only] r[verify cairn.supersede.migrate_openspec.boundary]
    // r[verify cairn.supersede.repo_native_verification] r[verify cairn.supersede.repo_native_verification.import]
    let fixture = temp_fixture("migrate-execute", "fixtures/basic")?;
    fs::remove_dir_all(fixture.join(".cairn"))?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;

    let preview = run_json(&["migrate", "openspec", "--root", root])?;
    let value = execute_migration("openspec", root, migration_plan_hash(&preview)?)?;
    assert_eq!(value["dry_run"], false);
    assert_eq!(value["blocked"], false);
    assert_eq!(value["mutated"], true);
    assert_hash_field(&value, "plan_hash")?;
    assert_hash_field(&value, "source_manifest_hash")?;
    assert_hash_field(&value, "destination_manifest_hash")?;
    assert_hash_field(&value, "manifest_hash")?;
    assert_hash_field(&value, "receipt_hash")?;
    assert_hash_field(&value["actions"][0], "source_hash")?;
    assert!(fixture.join(".cairn/specs/demo/spec.md").is_file());
    assert!(fixture
        .join(".cairn/changes/demo/specs/demo/spec.md")
        .is_file());

    let status = run_json(&["status", "--root", root])?;
    assert_eq!(status["layout"], "cairn");
    assert_eq!(
        status["changes"][0]["path"],
        format!("{root}/.cairn/changes/demo")
    );
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn migrate_execute_blocks_existing_native_destination() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.migrate_openspec.conflict]
    let fixture = temp_fixture("migrate-conflict", "fixtures/basic")?;
    fs::remove_dir_all(fixture.join(".cairn"))?;
    fs::create_dir_all(fixture.join(".cairn/specs/demo"))?;
    fs::write(fixture.join(".cairn/specs/demo/spec.md"), "existing")?;
    let root = fixture.to_str().ok_or("utf8 fixture path")?;

    let preview = run_json(&["migrate", "openspec", "--root", root])?;
    let value = execute_migration("openspec", root, migration_plan_hash(&preview)?)?;
    assert_eq!(value["dry_run"], false);
    assert_eq!(value["blocked"], true);
    assert_eq!(value["mutated"], false);
    assert!(value["actions"]
        .as_array()
        .ok_or("migration actions")?
        .iter()
        .any(|action| action["destination_hash"]
            .as_str()
            .unwrap_or_default()
            .len()
            == 64));
    assert_eq!(
        fs::read_to_string(fixture.join(".cairn/specs/demo/spec.md"))?,
        "existing"
    );
    fs::remove_dir_all(fixture)?;
    Ok(())
}

#[test]
fn legacy_ironspec_alias_warns_and_runs_cairn_behavior() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.rename_cairn.legacy_alias]
    // r[verify cairn.supersede.rename_cairn.cli]
    let mut command = ironspec_alias();
    command.args(["status", "--root", "fixtures/basic"]);
    let output = command.output()?;
    assert!(output.status.success());
    let stdout: Value = serde_json::from_slice(&output.stdout)?;
    assert_eq!(stdout["layout"], "cairn");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("compatibility alias"), "{stderr}");
    assert!(stderr.contains("cairn"), "{stderr}");
    Ok(())
}

#[test]
fn migrate_old_cairn_directory_explicitly_without_runtime_fallback() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_directory.legacy_migration]
    // r[verify cairn.lifecycle_directory.legacy_migration.preview]
    // r[verify cairn.lifecycle_directory.legacy_migration.execute]
    // r[verify cairn.lifecycle_directory.legacy_migration.plan_binding]
    // r[verify cairn.lifecycle_directory.hidden_default.runtime]
    // r[verify cairn.lifecycle_directory.legacy_diagnostic]
    // r[verify cairn.lifecycle_directory.legacy_diagnostic.normal_command]
    // r[verify cairn.lifecycle_directory.legacy_diagnostic.migrate]
    let root = temp_fixture("migrate-old-cairn", "fixtures/basic")?;
    fs::rename(root.join(".cairn"), root.join("cairn"))?;
    #[cfg(unix)]
    fs::set_permissions(
        root.join("cairn/specs/demo/spec.md"),
        fs::Permissions::from_mode(EXECUTABLE_TEST_MODE),
    )?;
    let root_text = root.to_string_lossy().to_string();

    let status = cairn().args(["status", "--root", &root_text]).output()?;
    assert!(!status.status.success());
    let status_error = String::from_utf8_lossy(&status.stderr);
    assert!(status_error.contains("cairn.lifecycle_directory.migration_required"));
    assert!(status_error.contains("cairn migrate cairn"));

    let dry_run = run_json(&["migrate", "cairn", "--root", &root_text])?;
    assert_eq!(dry_run["command"], "migrate_legacy-cairn");
    assert_eq!(dry_run["source_layout"], "legacy-cairn");
    assert_eq!(dry_run["destination_layout"], "cairn");
    assert_eq!(dry_run["dry_run"], true);
    assert_eq!(dry_run["mutated"], false);
    assert_hash_field(&dry_run, "plan_hash")?;
    assert!(!dry_run["actions"].as_array().ok_or("actions")?.is_empty());
    assert!(!root.join(".cairn").exists());

    let executed = execute_migration("cairn", &root_text, migration_plan_hash(&dry_run)?)?;
    assert_eq!(executed["dry_run"], false);
    assert_eq!(executed["mutated"], true);
    assert_eq!(executed["plan_hash"], dry_run["plan_hash"]);
    assert!(root.join("cairn/specs/demo/spec.md").is_file());
    assert!(root.join(".cairn/specs/demo/spec.md").is_file());
    #[cfg(unix)]
    assert_eq!(
        fs::metadata(root.join(".cairn/specs/demo/spec.md"))?
            .permissions()
            .mode()
            & PERMISSION_MODE_MASK,
        EXECUTABLE_TEST_MODE
    );
    let migrated = run_json(&["status", "--root", &root_text])?;
    assert_eq!(migrated["layout"], "cairn");
    assert_eq!(migrated["specs"].as_array().ok_or("specs")?.len(), 1);
    Ok(())
}

#[test]
fn migrate_execute_requires_plan_hash() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_directory.legacy_migration.plan_binding.execute]
    let root = temp_fixture("migrate-missing-plan-hash", "fixtures/basic")?;
    fs::rename(root.join(".cairn"), root.join("cairn"))?;
    let root_text = root.to_string_lossy().to_string();

    let output = cairn()
        .args(["migrate", "cairn", "--root", &root_text, "--execute"])
        .output()?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requires --plan-hash HASH"), "{stderr}");
    assert!(!root.join(".cairn").exists());
    Ok(())
}

#[test]
fn migrate_execute_rejects_stale_source_plan() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_directory.legacy_migration.plan_binding.stale]
    let root = temp_fixture("migrate-stale-source", "fixtures/basic")?;
    fs::rename(root.join(".cairn"), root.join("cairn"))?;
    let root_text = root.to_string_lossy().to_string();
    let preview = run_json(&["migrate", "cairn", "--root", &root_text])?;
    let reviewed_hash = migration_plan_hash(&preview)?.to_string();
    fs::write(
        root.join("cairn/specs/demo/spec.md"),
        "changed after preview",
    )?;

    let output = cairn()
        .args([
            "migrate",
            "cairn",
            "--root",
            &root_text,
            "--execute",
            "--plan-hash",
            &reviewed_hash,
        ])
        .output()?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("migration plan hash mismatch"), "{stderr}");
    assert!(!root.join(".cairn").exists());
    Ok(())
}

#[test]
fn migrate_execute_rejects_destination_created_after_preview() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_directory.legacy_migration.no_replace]
    let root = temp_fixture("migrate-destination-race", "fixtures/basic")?;
    fs::rename(root.join(".cairn"), root.join("cairn"))?;
    let root_text = root.to_string_lossy().to_string();
    let preview = run_json(&["migrate", "cairn", "--root", &root_text])?;
    let reviewed_hash = migration_plan_hash(&preview)?.to_string();
    fs::create_dir_all(root.join(".cairn/specs/demo"))?;
    fs::write(root.join(".cairn/specs/demo/spec.md"), "appeared")?;

    let output = cairn()
        .args([
            "migrate",
            "cairn",
            "--root",
            &root_text,
            "--execute",
            "--plan-hash",
            &reviewed_hash,
        ])
        .output()?;
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("migration plan hash mismatch"), "{stderr}");
    assert_eq!(
        fs::read_to_string(root.join(".cairn/specs/demo/spec.md"))?,
        "appeared"
    );
    Ok(())
}

#[test]
fn migrate_old_cairn_directory_blocks_existing_destination() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_directory.legacy_migration.conflict]
    let root = temp_fixture("migrate-old-cairn-conflict", "fixtures/basic")?;
    fs::rename(root.join(".cairn"), root.join("cairn"))?;
    fs::create_dir_all(root.join(".cairn/specs/demo"))?;
    fs::write(root.join(".cairn/specs/demo/spec.md"), "existing")?;
    let root_text = root.to_string_lossy().to_string();

    let preview = run_json(&["migrate", "cairn", "--root", &root_text])?;
    let executed = execute_migration("cairn", &root_text, migration_plan_hash(&preview)?)?;
    assert_eq!(executed["blocked"], true);
    assert_eq!(executed["mutated"], false);
    assert_eq!(
        fs::read_to_string(root.join(".cairn/specs/demo/spec.md"))?,
        "existing"
    );
    assert!(root.join("cairn/specs/demo/spec.md").is_file());
    Ok(())
}

#[test]
fn migrate_legacy_ironspec_to_cairn_explicitly() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.migrate_openspec.ironspec_dry_run]
    // r[verify cairn.supersede.migrate_openspec.ironspec_execute]
    // r[verify cairn.supersede.native_layout.no_implicit_legacy_after_rename]
    let root = temp_fixture("migrate-ironspec", "fixtures/basic")?;
    fs::rename(root.join(".cairn"), root.join("ironspec"))?;
    let root_text = root.to_string_lossy().to_string();

    let status = run_json(&["status", "--root", &root_text])?;
    assert_eq!(status["layout"], "cairn");
    assert_eq!(status["specs"].as_array().ok_or("specs")?.len(), 0);
    assert_eq!(status["changes"].as_array().ok_or("changes")?.len(), 0);

    let dry_run = run_json(&["migrate", "ironspec", "--root", &root_text])?;
    assert_eq!(dry_run["command"], "migrate_ironspec");
    assert_eq!(dry_run["source_layout"], "ironspec");
    assert_eq!(dry_run["destination_layout"], "cairn");
    assert_eq!(dry_run["dry_run"], true);
    assert_eq!(dry_run["mutated"], false);
    assert!(!dry_run["actions"].as_array().ok_or("actions")?.is_empty());

    let executed = execute_migration("ironspec", &root_text, migration_plan_hash(&dry_run)?)?;
    assert_eq!(executed["dry_run"], false);
    assert_eq!(executed["mutated"], true);
    let migrated = run_json(&["status", "--root", &root_text])?;
    assert_eq!(migrated["layout"], "cairn");
    assert_eq!(migrated["specs"].as_array().ok_or("specs")?.len(), 1);
    Ok(())
}
