use std::fs;
use std::path::Path;
use std::process::Command;

fn cargo_octet() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-octet")
}

#[allow(clippy::needless_pass_by_value)]
fn write_json(path: &Path, value: serde_json::Value) {
    fs::write(path, serde_json::to_string_pretty(&value).unwrap()).unwrap();
}

fn run(args: &[&str], cwd: &Path) -> std::process::Output {
    Command::new(cargo_octet()).args(args).current_dir(cwd).output().unwrap()
}

fn stdout_json(output: &std::process::Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "stderr={} stdout={}",
        String::from_utf8_lossy(&output.stderr),
        String::from_utf8_lossy(&output.stdout)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn stderr_text(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).to_string()
}

#[test]
fn artifact_hash_inspect_and_replay_are_content_addressed() {
    let temp = tempfile::tempdir().unwrap();
    let dto_a = temp.path().join("a.json");
    let dto_b = temp.path().join("b.json");
    let store = temp.path().join("store");
    write_json(
        &dto_a,
        serde_json::json!({
            "name": "review-prompt",
            "body": "check this",
            "timestamp": "2026-01-01T00:00:00Z",
            "absolute_path": "~/secret/path",
            "raw_secret": "do-not-store"
        }),
    );
    write_json(
        &dto_b,
        serde_json::json!({
            "absolute_path": "/tmp/other",
            "raw_secret": "different-secret",
            "body": "check this",
            "name": "review-prompt",
            "timestamp": "2027-01-01T00:00:00Z"
        }),
    );

    let a = stdout_json(&run(
        &[
            "roadmap",
            "artifact",
            "store",
            "--kind",
            "prompt",
            "--input",
            dto_a.to_str().unwrap(),
            "--store-dir",
            store.to_str().unwrap(),
        ],
        temp.path(),
    ));
    let b = stdout_json(&run(
        &[
            "roadmap",
            "artifact",
            "store",
            "--kind",
            "prompt",
            "--input",
            dto_b.to_str().unwrap(),
            "--store-dir",
            store.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(a["hash"], b["hash"]);
    assert!(a["hash"].as_str().unwrap().starts_with("b3:"));
    assert_eq!(a["redaction_status"], "secret-like-and-local-fields-dropped-local-roots-normalized");
    assert!(!a.to_string().contains("do-not-store"));

    let hash = a["hash"].as_str().unwrap();
    let inspected = stdout_json(&run(
        &[
            "roadmap",
            "artifact",
            "inspect",
            "--hash",
            hash,
            "--store-dir",
            store.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(inspected["artifact_kind"], "prompt");
    let replay = stdout_json(&run(
        &[
            "roadmap",
            "artifact",
            "replay",
            "--hash",
            hash,
            "--store-dir",
            store.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(replay["operation"], "replay");
}

#[test]
fn replay_capsules_capture_inspect_replay_and_drift() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("capsule-input.json");
    let capsule = temp.path().join("capsule.json");
    let observed = temp.path().join("observed.json");
    let drifted = temp.path().join("drifted.json");
    write_json(
        &input,
        serde_json::json!({
            "command_argv": ["cargo", "octet", "check", "--output-format", "json"],
            "run_kind": "check",
            "octet_binary_identity": "b3:bin",
            "config_hash": "b3:cfg",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:findings"],
            "expected_exit_code": 0,
            "absolute_path": "/private/worktree",
            "raw_secret": "do-not-store"
        }),
    );
    write_json(
        &observed,
        serde_json::json!({
            "command_argv": ["cargo", "octet", "check", "--output-format", "json"],
            "run_kind": "check",
            "octet_binary_identity": "b3:bin",
            "config_hash": "b3:cfg",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:findings"],
            "exit_code": 0
        }),
    );
    let captured = stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "capture",
            "--input",
            input.to_str().unwrap(),
            "--capsule",
            capsule.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(captured["capsule_kind"], "octet-replay-capsule");
    assert_eq!(captured["manifest"]["octet_binary_identity"], "b3:bin");
    assert!(captured["manifest"].get("slotcar_binary_identity").is_none());
    assert!(captured["capsule_hash"].as_str().unwrap().starts_with("b3:"));
    assert!(!captured.to_string().contains("do-not-store"));

    let inspected =
        stdout_json(&run(&["roadmap", "capsule", "inspect", "--capsule", capsule.to_str().unwrap()], temp.path()));
    assert_eq!(inspected["run_kind"], "check");
    let replay = stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "replay",
            "--capsule",
            capsule.to_str().unwrap(),
            "--observed",
            observed.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(replay["status"], "replayed");

    write_json(
        &drifted,
        serde_json::json!({
            "command_argv": ["cargo", "octet", "check", "--output-format", "json"],
            "run_kind": "check",
            "octet_binary_identity": "b3:bin",
            "config_hash": "b3:changed",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:findings"],
            "exit_code": 0
        }),
    );
    let drift = run(
        &[
            "roadmap",
            "capsule",
            "replay",
            "--capsule",
            capsule.to_str().unwrap(),
            "--observed",
            drifted.to_str().unwrap(),
        ],
        temp.path(),
    );
    assert!(!drift.status.success());
    let drift_report: serde_json::Value = serde_json::from_slice(&drift.stdout).unwrap();
    assert_eq!(drift_report["status"], "drifted");
    assert_eq!(drift_report["mismatches"], serde_json::json!(["config_hash"]));
}

#[test]
fn replay_capsules_normalize_legacy_slotcar_identity_field() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("legacy-capsule-input.json");
    let capsule = temp.path().join("legacy-capsule.json");
    write_json(
        &input,
        serde_json::json!({
            "command_argv": ["cargo", "slotcar", "check"],
            "run_kind": "check",
            "slotcar_binary_identity": "b3:legacy-bin",
            "config_hash": "b3:cfg",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:findings"],
            "expected_exit_code": 0
        }),
    );

    let captured = stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "capture",
            "--input",
            input.to_str().unwrap(),
            "--capsule",
            capsule.to_str().unwrap(),
        ],
        temp.path(),
    ));

    assert_eq!(captured["manifest"]["octet_binary_identity"], "b3:legacy-bin");
    assert!(captured["manifest"].get("slotcar_binary_identity").is_none());
}

#[test]
fn replay_capsules_accept_expected_failures() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("fingerprint-input.json");
    let capsule = temp.path().join("capsule.json");
    write_json(
        &input,
        serde_json::json!({
            "command_argv": ["cargo", "octet", "fingerprint", "--check", "src/lib.rs"],
            "run_kind": "fingerprint",
            "octet_binary_identity": "b3:bin",
            "config_hash": "b3:cfg",
            "input_hashes": ["b3:src"],
            "output_hashes": ["b3:diff"],
            "expected_exit_code": 1
        }),
    );
    stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "capture",
            "--input",
            input.to_str().unwrap(),
            "--capsule",
            capsule.to_str().unwrap(),
        ],
        temp.path(),
    ));
    let mut observed: serde_json::Value = serde_json::from_str(&fs::read_to_string(&input).unwrap()).unwrap();
    observed["exit_code"] = serde_json::json!(1);
    write_json(&input, observed);
    let replay = stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "replay",
            "--capsule",
            capsule.to_str().unwrap(),
            "--observed",
            input.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(replay["status"], "expected-failure-replayed");
}

fn capsule_manifest_input() -> serde_json::Value {
    serde_json::json!({
        "command_argv": ["cargo", "octet", "check", "--output-format", "json"],
        "run_kind": "check",
        "octet_binary_identity": "b3:bin",
        "config_hash": "b3:cfg",
        "input_hashes": ["b3:src"],
        "output_hashes": ["b3:findings"],
        "expected_exit_code": 0
    })
}

fn capsule_replay(capsule: &Path, observed: &Path, cwd: &Path) -> std::process::Output {
    run(
        &[
            "roadmap",
            "capsule",
            "replay",
            "--capsule",
            capsule.to_str().unwrap(),
            "--observed",
            observed.to_str().unwrap(),
        ],
        cwd,
    )
}

#[test]
fn replay_capsules_fail_closed_on_malformed_capsule_manifests() {
    let temp = tempfile::tempdir().unwrap();
    let capsule = temp.path().join("capsule.json");
    let observed = temp.path().join("observed.json");

    write_json(&observed, serde_json::json!({"exit_code": 0}));
    write_json(&capsule, serde_json::json!({"capsule_hash": "b3:anything", "capsule_kind": "octet-replay-capsule"}));
    let missing_manifest = capsule_replay(&capsule, &observed, temp.path());
    assert_eq!(
        missing_manifest.status.code(),
        Some(2),
        "stdout={}",
        String::from_utf8_lossy(&missing_manifest.stdout)
    );
    assert!(stderr_text(&missing_manifest).contains("manifest must be a JSON object"));

    let mut partial_manifest = capsule_manifest_input();
    let partial_object = partial_manifest.as_object_mut().unwrap();
    partial_object.remove("octet_binary_identity");
    partial_object.remove("expected_exit_code");
    let mut partial_observed = partial_manifest.clone();
    partial_observed["exit_code"] = serde_json::json!(0);
    write_json(&observed, partial_observed);
    write_json(&capsule, serde_json::json!({"capsule_hash": "b3:anything", "manifest": partial_manifest}));
    let missing_field = capsule_replay(&capsule, &observed, temp.path());
    assert_eq!(missing_field.status.code(), Some(2), "stdout={}", String::from_utf8_lossy(&missing_field.stdout));
    assert!(stderr_text(&missing_field).contains("capsule manifest missing octet_binary_identity"));
}

#[test]
fn replay_capsules_capture_requires_integer_expected_exit_code() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("capsule-input.json");
    let capsule = temp.path().join("capsule.json");
    for expected_exit_code in [serde_json::json!("1"), serde_json::json!(1.5), serde_json::Value::Null] {
        let mut manifest = capsule_manifest_input();
        manifest["expected_exit_code"] = expected_exit_code.clone();
        write_json(&input, manifest);
        let output = run(
            &[
                "roadmap",
                "capsule",
                "capture",
                "--input",
                input.to_str().unwrap(),
                "--capsule",
                capsule.to_str().unwrap(),
            ],
            temp.path(),
        );
        assert_eq!(output.status.code(), Some(2), "expected_exit_code={expected_exit_code}");
        assert!(
            stderr_text(&output).contains("expected_exit_code must be an integer"),
            "expected_exit_code={expected_exit_code} stderr={}",
            stderr_text(&output)
        );
        assert!(!capsule.exists(), "expected_exit_code={expected_exit_code} wrote a capsule");
    }
}

#[test]
fn replay_capsules_detect_manifest_tampering_via_capsule_hash() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("capsule-input.json");
    let capsule = temp.path().join("capsule.json");
    let observed = temp.path().join("observed.json");
    write_json(&input, capsule_manifest_input());
    stdout_json(&run(
        &[
            "roadmap",
            "capsule",
            "capture",
            "--input",
            input.to_str().unwrap(),
            "--capsule",
            capsule.to_str().unwrap(),
        ],
        temp.path(),
    ));

    let mut tampered: serde_json::Value = serde_json::from_str(&fs::read_to_string(&capsule).unwrap()).unwrap();
    tampered["manifest"]["output_hashes"] = serde_json::json!(["b3:forged"]);
    write_json(&capsule, tampered);
    let mut forged_run = capsule_manifest_input();
    forged_run["output_hashes"] = serde_json::json!(["b3:forged"]);
    forged_run["exit_code"] = serde_json::json!(0);
    write_json(&observed, forged_run);

    let replay = capsule_replay(&capsule, &observed, temp.path());
    assert_eq!(replay.status.code(), Some(2), "stdout={}", String::from_utf8_lossy(&replay.stdout));
    assert!(stderr_text(&replay).contains("capsule_hash mismatch"), "stderr={}", stderr_text(&replay));
}

#[test]
fn capability_registry_and_summary_report_known_unknown_and_advisory() {
    let temp = tempfile::tempdir().unwrap();
    let registry = stdout_json(&run(&["roadmap", "ability", "registry"], temp.path()));
    assert_eq!(registry["status"], "approved-capability-vocabulary");
    assert!(registry["families"].to_string().contains("filesystem"));
    assert!(
        registry["families"]
            .to_string()
            .contains("core-prohibited-unless-explicit-data-or-named-capability")
    );

    let findings = temp.path().join("findings.json");
    write_json(
        &findings,
        serde_json::json!([
            {"lint": "no_system_time", "effect_family": "time", "capability_status": "known", "enforcement": "hard"},
            {"lint": "no_absolute_path", "effect_family": "filesystem", "capability_status": "known", "enforcement": "advisory"},
            {"lint": "new_lint", "effect_family": "unknown", "capability_status": "unknown", "enforcement": "advisory"}
        ]),
    );
    let summary = stdout_json(&run(
        &[
            "roadmap",
            "ability",
            "summary",
            "--findings",
            findings.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(summary["counts_by_capability"]["time"], 1);
    assert_eq!(summary["counts_by_capability"]["filesystem"], 1);
    assert_eq!(summary["counts_by_capability"]["unknown"], 1);
    assert_eq!(summary["advisory_counts_by_capability"]["filesystem"], 1);
    assert_eq!(summary["unknown_capability_visible"], true);
}

#[test]
fn ability_manifest_denies_undeclared_effects() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = temp.path().join("manifest.json");
    let trace = temp.path().join("trace.json");
    write_json(&manifest, serde_json::json!({"allowed_effects": ["FileRead"]}));
    write_json(&trace, serde_json::json!({"effects": ["FileRead", "Network"]}));
    let output = run(
        &[
            "roadmap",
            "ability",
            "enforce",
            "--manifest",
            manifest.to_str().unwrap(),
            "--trace",
            trace.to_str().unwrap(),
        ],
        temp.path(),
    );
    assert!(!output.status.success());
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["status"], "denied");
    assert_eq!(report["denied_effects"], serde_json::json!(["Network"]));
}

#[test]
fn name_pointer_updates_are_rollbackable() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("registry.json");
    let receipt1 = temp.path().join("receipt1.json");
    let receipt2 = temp.path().join("receipt2.json");
    stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "set",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
            "--kind",
            "prompt",
            "--hash",
            "b3:old",
            "--receipt",
            receipt1.to_str().unwrap(),
        ],
        temp.path(),
    ));
    let update = stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "set",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
            "--kind",
            "prompt",
            "--hash",
            "b3:new",
            "--receipt",
            receipt2.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(update["old_hash"], "b3:old");
    let inspected = stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "inspect",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
        ],
        temp.path(),
    ));
    assert_eq!(inspected["target_hash"], "b3:new");
    assert_eq!(inspected["human_label"], "done-review");
    assert_eq!(inspected["previous_targets"][0]["hash"], "b3:old");
    assert_eq!(inspected["review_identity_status"], "matched-identity-not-correctness-proof");
    assert_eq!(inspected["correctness_claim"], "none");
    stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "rollback",
            "--registry",
            registry.to_str().unwrap(),
            "--receipt",
            receipt2.to_str().unwrap(),
        ],
        temp.path(),
    ));
    let rolled_back = stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "inspect",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
        ],
        temp.path(),
    ));
    assert_eq!(rolled_back["target_hash"], "b3:old");
}

fn pointer_set(registry: &Path, kind: &str, hash: &str, receipt: &Path, cwd: &Path) -> serde_json::Value {
    stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "set",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
            "--kind",
            kind,
            "--hash",
            hash,
            "--receipt",
            receipt.to_str().unwrap(),
        ],
        cwd,
    ))
}

fn pointer_rollback(registry: &Path, receipt: &Path, cwd: &Path) -> std::process::Output {
    run(
        &[
            "roadmap",
            "pointer",
            "rollback",
            "--registry",
            registry.to_str().unwrap(),
            "--receipt",
            receipt.to_str().unwrap(),
        ],
        cwd,
    )
}

fn pointer_inspect(registry: &Path, cwd: &Path) -> serde_json::Value {
    stdout_json(&run(
        &[
            "roadmap",
            "pointer",
            "inspect",
            "--registry",
            registry.to_str().unwrap(),
            "--name",
            "done-review",
        ],
        cwd,
    ))
}

#[test]
fn pointer_rollback_restores_previous_artifact_kind() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("registry.json");
    let receipt1 = temp.path().join("receipt1.json");
    let receipt2 = temp.path().join("receipt2.json");
    pointer_set(&registry, "prompt", "b3:old", &receipt1, temp.path());
    let update = pointer_set(&registry, "policy", "b3:new", &receipt2, temp.path());
    assert_eq!(update["old_kind"], "prompt");

    stdout_json(&pointer_rollback(&registry, &receipt2, temp.path()));
    let rolled_back = pointer_inspect(&registry, temp.path());
    assert_eq!(rolled_back["target_hash"], "b3:old");
    assert_eq!(rolled_back["target_artifact_kind"], "prompt");
}

#[test]
fn pointer_rollback_recovers_previous_kind_for_receipts_without_old_kind() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("registry.json");
    let receipt1 = temp.path().join("receipt1.json");
    let receipt2 = temp.path().join("receipt2.json");
    pointer_set(&registry, "prompt", "b3:old", &receipt1, temp.path());
    pointer_set(&registry, "policy", "b3:new", &receipt2, temp.path());
    let mut legacy_receipt: serde_json::Value = serde_json::from_str(&fs::read_to_string(&receipt2).unwrap()).unwrap();
    legacy_receipt.as_object_mut().unwrap().remove("old_kind");
    write_json(&receipt2, legacy_receipt);

    stdout_json(&pointer_rollback(&registry, &receipt2, temp.path()));
    let rolled_back = pointer_inspect(&registry, temp.path());
    assert_eq!(rolled_back["target_hash"], "b3:old");
    assert_eq!(rolled_back["target_artifact_kind"], "prompt");
}

#[test]
fn pointer_rollback_rejects_receipts_superseded_by_later_updates() {
    let temp = tempfile::tempdir().unwrap();
    let registry = temp.path().join("registry.json");
    let receipt1 = temp.path().join("receipt1.json");
    let receipt2 = temp.path().join("receipt2.json");
    let receipt3 = temp.path().join("receipt3.json");
    pointer_set(&registry, "prompt", "b3:old", &receipt1, temp.path());
    pointer_set(&registry, "prompt", "b3:new", &receipt2, temp.path());
    pointer_set(&registry, "prompt", "b3:newer", &receipt3, temp.path());
    let registry_before = fs::read_to_string(&registry).unwrap();

    let stale = pointer_rollback(&registry, &receipt2, temp.path());
    assert_eq!(stale.status.code(), Some(2), "stdout={}", String::from_utf8_lossy(&stale.stdout));
    assert!(stderr_text(&stale).contains("b3:newer"), "stderr={}", stderr_text(&stale));
    assert!(stderr_text(&stale).contains("b3:new`"), "stderr={}", stderr_text(&stale));
    assert_eq!(fs::read_to_string(&registry).unwrap(), registry_before);
    assert_eq!(pointer_inspect(&registry, temp.path())["target_hash"], "b3:newer");

    stdout_json(&pointer_rollback(&registry, &receipt3, temp.path()));
    assert_eq!(pointer_inspect(&registry, temp.path())["target_hash"], "b3:new");
}

#[test]
fn session_records_todos_and_migrations_are_typed() {
    let temp = tempfile::tempdir().unwrap();
    let session = temp.path().join("session.json");
    let migrated = temp.path().join("migrated.json");
    write_json(
        &session,
        serde_json::json!({
            "identity": "b3:session",
            "original_schema_version": 0,
            "records": [
                {"kind": "model_request", "schema_version": 1, "identity": "b3:req", "redaction_status": "redacted"},
                {"kind": "tool_call", "schema_version": 1, "identity": "b3:tool", "redaction_status": "redacted"}
            ],
            "todos": [
                {"kind": "failing_check", "status": "blocked", "source": "V1", "next_action": "rerun", "blocker": "fixture missing"},
                {"kind": "pending_edit", "status": "completed", "source": "I1", "next_action": "none"},
                {"kind": "review_findings", "status": "deferred", "source": "H1", "next_action": "human review"}
            ]
        }),
    );
    let check = stdout_json(&run(&["roadmap", "session", "check", "--input", session.to_str().unwrap()], temp.path()));
    assert_eq!(check["status"], "typed-versioned");
    let migration = stdout_json(&run(
        &[
            "roadmap",
            "session",
            "migrate",
            "--input",
            session.to_str().unwrap(),
            "--output",
            migrated.to_str().unwrap(),
        ],
        temp.path(),
    ));
    assert_eq!(migration["original_identity"], "b3:session");
    assert_eq!(migration["identity_change"], false);
    let todos = stdout_json(&run(&["roadmap", "session", "todo", "--input", session.to_str().unwrap()], temp.path()));
    assert_eq!(todos["counts_by_status"]["blocked"], 1);
    assert_eq!(todos["counts_by_status"]["completed"], 1);
    assert_eq!(todos["counts_by_status"]["deferred"], 1);
}

#[test]
fn pure_cache_hits_only_matching_key_components() {
    let temp = tempfile::tempdir().unwrap();
    let cache = temp.path().join("cache");
    let key1 = temp.path().join("key1.json");
    let key2 = temp.path().join("key2.json");
    write_json(
        &key1,
        serde_json::json!({
            "command_identity": "cargo-check",
            "input_hashes": ["b3:src"],
            "config_hash": "b3:cfg",
            "tool_revision": "rev1",
            "environment_allowlist": {"RUSTFLAGS": ""}
        }),
    );
    write_json(
        &key2,
        serde_json::json!({
            "command_identity": "cargo-check",
            "input_hashes": ["b3:src"],
            "config_hash": "b3:cfg",
            "tool_revision": "rev2",
            "environment_allowlist": {"RUSTFLAGS": ""}
        }),
    );
    let miss = stdout_json(&run(
        &[
            "roadmap",
            "pure-cache",
            "check",
            "--key",
            key1.to_str().unwrap(),
            "--store-dir",
            cache.to_str().unwrap(),
            "--output-hash",
            "b3:out",
        ],
        temp.path(),
    ));
    assert_eq!(miss["status"], "miss");
    let hit = stdout_json(&run(
        &[
            "roadmap",
            "pure-cache",
            "check",
            "--key",
            key1.to_str().unwrap(),
            "--store-dir",
            cache.to_str().unwrap(),
            "--output-hash",
            "b3:out",
        ],
        temp.path(),
    ));
    assert_eq!(hit["status"], "hit");
    assert_eq!(hit["stored_output_hash"], "b3:out");
    let drift = stdout_json(&run(
        &[
            "roadmap",
            "pure-cache",
            "check",
            "--key",
            key2.to_str().unwrap(),
            "--store-dir",
            cache.to_str().unwrap(),
            "--output-hash",
            "b3:out2",
        ],
        temp.path(),
    ));
    assert_eq!(drift["status"], "miss");
    assert_eq!(drift["miss_reason"], "changed-key-components");
    assert_eq!(drift["changed_key_components"], serde_json::json!(["tool_revision"]));
}

#[test]
fn roadmap_docs_transcript_covers_agent_surfaces() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root");
    let transcript_path = root.join("docs/examples/roadmap-agent-surfaces/transcript.md");
    let transcript = fs::read_to_string(&transcript_path).expect("read roadmap transcript");

    let output = run(&["transcript", transcript_path.to_str().unwrap()], root);
    assert!(output.status.success(), "stderr={}", stderr_text(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("command_blocks=10"));
    assert!(stdout.contains("expected_output_blocks=10"));
    assert!(stdout.contains("status=passed"));

    for surface in ["artifact", "ability", "pointer", "session", "todo", "pure-cache"] {
        assert!(transcript.contains(surface), "roadmap transcript missing {surface}");
    }
}

#[test]
fn roadmap_error_paths_fail_closed_with_operator_context() {
    let temp = tempfile::tempdir().unwrap();
    let missing_hash = format!("b3:{}", "0".repeat(64));
    let missing_artifact = run(
        &[
            "roadmap",
            "artifact",
            "inspect",
            "--hash",
            &missing_hash,
            "--store-dir",
            temp.path().join("store").to_str().unwrap(),
        ],
        temp.path(),
    );
    assert!(!missing_artifact.status.success());
    assert!(stderr_text(&missing_artifact).contains("read"));
    assert!(stderr_text(&missing_artifact).contains(&format!("{}.json", missing_hash.replace(':', "_"))));

    let malformed_manifest = temp.path().join("manifest.json");
    let trace = temp.path().join("trace.json");
    fs::write(&malformed_manifest, "not json").unwrap();
    write_json(&trace, serde_json::json!({"effects": ["Network"]}));
    let malformed = run(
        &[
            "roadmap",
            "ability",
            "enforce",
            "--manifest",
            malformed_manifest.to_str().unwrap(),
            "--trace",
            trace.to_str().unwrap(),
        ],
        temp.path(),
    );
    assert!(!malformed.status.success());
    assert!(stderr_text(&malformed).contains("parse"));
    assert!(stderr_text(&malformed).contains("manifest.json"));

    let bad_session = temp.path().join("session.json");
    write_json(&bad_session, serde_json::json!({"records": [{"kind": "tool_call"}]}));
    let session = run(&["roadmap", "session", "check", "--input", bad_session.to_str().unwrap()], temp.path());
    assert!(!session.status.success());
    assert!(
        stderr_text(&session).contains("session record missing kind, schema_version, identity, or redaction_status")
    );

    let bad_key = temp.path().join("key.json");
    write_json(&bad_key, serde_json::json!({"command_identity": "cargo-check"}));
    let cache = run(
        &[
            "roadmap",
            "pure-cache",
            "check",
            "--key",
            bad_key.to_str().unwrap(),
            "--store-dir",
            temp.path().join("cache").to_str().unwrap(),
            "--output-hash",
            "b3:out",
        ],
        temp.path(),
    );
    assert!(!cache.status.success());
    assert!(stderr_text(&cache).contains("cache key missing input_hashes"));
}

#[test]
fn session_migrate_rejects_non_object_sessions_without_panicking() {
    let temp = tempfile::tempdir().unwrap();
    let migrated = temp.path().join("migrated.json");
    for (index, session_json) in ["[]", "1", "\"x\"", "true", "null"].iter().enumerate() {
        let session = temp.path().join(format!("session-{index}.json"));
        fs::write(&session, session_json).unwrap();
        let output = run(
            &[
                "roadmap",
                "session",
                "migrate",
                "--input",
                session.to_str().unwrap(),
                "--output",
                migrated.to_str().unwrap(),
            ],
            temp.path(),
        );
        assert_eq!(output.status.code(), Some(2), "session={session_json} stderr={}", stderr_text(&output));
        assert!(
            stderr_text(&output).contains("session must be a JSON object"),
            "session={session_json} stderr={}",
            stderr_text(&output)
        );
        assert!(!migrated.exists(), "session={session_json} wrote migrated output");
    }
}

fn store_artifact(input: &Path, store: &Path, cwd: &Path) -> std::process::Output {
    run(
        &[
            "roadmap",
            "artifact",
            "store",
            "--kind",
            "prompt",
            "--input",
            input.to_str().unwrap(),
            "--store-dir",
            store.to_str().unwrap(),
        ],
        cwd,
    )
}

fn artifact_by_hash(operation: &str, hash: &str, store: &Path, cwd: &Path) -> std::process::Output {
    run(
        &[
            "roadmap",
            "artifact",
            operation,
            "--hash",
            hash,
            "--store-dir",
            store.to_str().unwrap(),
        ],
        cwd,
    )
}

fn stored_hash(dto: serde_json::Value, name: &str, store: &Path, cwd: &Path) -> String {
    let input = cwd.join(name);
    write_json(&input, dto);
    stdout_json(&store_artifact(&input, store, cwd))["hash"].as_str().unwrap().to_owned()
}

fn record_path(store: &Path, hash: &str) -> std::path::PathBuf {
    store.join(format!("{}.json", hash.replace(':', "_")))
}

fn enforce(manifest: serde_json::Value, trace: serde_json::Value, cwd: &Path) -> std::process::Output {
    let manifest_path = cwd.join("enforce-manifest.json");
    let trace_path = cwd.join("enforce-trace.json");
    write_json(&manifest_path, manifest);
    write_json(&trace_path, trace);
    run(
        &[
            "roadmap",
            "ability",
            "enforce",
            "--manifest",
            manifest_path.to_str().unwrap(),
            "--trace",
            trace_path.to_str().unwrap(),
        ],
        cwd,
    )
}

#[test]
fn ability_enforce_requires_string_arrays_for_manifest_and_trace() {
    let temp = tempfile::tempdir().unwrap();
    let manifest = serde_json::json!({"allowed_effects": ["FileRead"]});
    for trace in [
        serde_json::json!({"effects": [{"family": "network"}]}),
        serde_json::json!({"effects": "network"}),
        serde_json::json!({"effect": ["network"]}),
        serde_json::json!({"effects": ["FileRead", null]}),
    ] {
        let output = enforce(manifest.clone(), trace.clone(), temp.path());
        assert_eq!(output.status.code(), Some(2), "trace={trace} stdout={}", String::from_utf8_lossy(&output.stdout));
        assert!(output.stdout.is_empty(), "trace={trace} printed a verdict");
    }
    for bad_manifest in [
        serde_json::json!({"allowed": ["FileRead"]}),
        serde_json::json!({"allowed_effects": "FileRead"}),
        serde_json::json!({"allowed_effects": [1]}),
    ] {
        let output = enforce(bad_manifest.clone(), serde_json::json!({"effects": []}), temp.path());
        assert_eq!(output.status.code(), Some(2), "manifest={bad_manifest}");
    }

    let allowed = stdout_json(&enforce(manifest.clone(), serde_json::json!({"effects": ["FileRead"]}), temp.path()));
    assert_eq!(allowed["status"], "allowed");
    let empty = stdout_json(&enforce(manifest, serde_json::json!({"effects": []}), temp.path()));
    assert_eq!(empty["status"], "allowed");
}

#[test]
fn artifact_hash_lookup_rejects_malformed_hashes_before_touching_the_store() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    let planted = stored_hash(serde_json::json!({"name": "planted"}), "planted.json", &store, temp.path());
    // A record outside the store directory that `b3:../outside` would resolve to.
    fs::create_dir_all(store.join("b3_")).unwrap();
    fs::copy(record_path(&store, &planted), temp.path().join("outside.json")).unwrap();

    for hash in [
        "b3:../../outside",
        "b3:missing",
        "../outside",
        &format!("b3:{}", planted[3..].to_uppercase()),
        &planted[..planted.len() - 1],
        &planted.replacen("b3:", "b2:", 1),
    ] {
        for operation in ["inspect", "replay"] {
            let output = artifact_by_hash(operation, hash, &store, temp.path());
            assert_eq!(
                output.status.code(),
                Some(2),
                "{operation} {hash}: stdout={}",
                String::from_utf8_lossy(&output.stdout)
            );
        }
    }
}

#[test]
fn artifact_inspect_and_replay_recompute_record_hashes() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    let hash = stored_hash(serde_json::json!({"name": "review", "body": "check"}), "a.json", &store, temp.path());
    let other = stored_hash(serde_json::json!({"name": "other"}), "b.json", &store, temp.path());
    let path = record_path(&store, &hash);
    let pristine = fs::read_to_string(&path).unwrap();
    for operation in ["inspect", "replay"] {
        assert!(artifact_by_hash(operation, &hash, &store, temp.path()).status.success());
    }

    let mut edited_body: serde_json::Value = serde_json::from_str(&pristine).unwrap();
    edited_body["normalized"]["body"] = serde_json::json!("tampered");
    let mut edited_kind: serde_json::Value = serde_json::from_str(&pristine).unwrap();
    edited_kind["artifact_kind"] = serde_json::json!("tool_schema");
    let mut edited_deps: serde_json::Value = serde_json::from_str(&pristine).unwrap();
    edited_deps["dependencies"] = serde_json::json!([other]);
    let mut renamed: serde_json::Value = serde_json::from_str(&pristine).unwrap();
    renamed["hash"] = serde_json::json!(other);
    let substituted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(record_path(&store, &other)).unwrap()).unwrap();
    for tampered in [edited_body, edited_kind, edited_deps, renamed, substituted] {
        write_json(&path, tampered.clone());
        for operation in ["inspect", "replay"] {
            let output = artifact_by_hash(operation, &hash, &store, temp.path());
            assert_eq!(output.status.code(), Some(2), "{operation} accepted tampered record {tampered}");
        }
    }
}

#[test]
fn artifact_replay_verifies_the_full_dependency_closure() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    let leaf = stored_hash(serde_json::json!({"name": "leaf"}), "leaf.json", &store, temp.path());
    let middle =
        stored_hash(serde_json::json!({"name": "middle", "dependencies": [leaf]}), "middle.json", &store, temp.path());
    let root =
        stored_hash(serde_json::json!({"name": "root", "dependencies": [middle]}), "root.json", &store, temp.path());

    let replay = stdout_json(&artifact_by_hash("replay", &root, &store, temp.path()));
    assert_eq!(replay["operation"], "replay");
    assert_eq!(replay["dependencies"], serde_json::json!([middle]));

    // A transitive dependency that is missing fails closed with exit 1.
    let leaf_record = fs::read_to_string(record_path(&store, &leaf)).unwrap();
    fs::remove_file(record_path(&store, &leaf)).unwrap();
    let missing = artifact_by_hash("replay", &root, &store, temp.path());
    assert_eq!(missing.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&missing.stdout).unwrap();
    assert_eq!(report["status"], "failed-closed");
    assert_eq!(report["missing_dependency"], leaf);

    // A dependency file that exists but does not hash to its name is not a satisfied prerequisite.
    let mut tampered: serde_json::Value = serde_json::from_str(&leaf_record).unwrap();
    tampered["normalized"]["name"] = serde_json::json!("forged");
    write_json(&record_path(&store, &leaf), tampered);
    assert_eq!(artifact_by_hash("replay", &root, &store, temp.path()).status.code(), Some(2));
    assert_eq!(artifact_by_hash("replay", &middle, &store, temp.path()).status.code(), Some(2));
}

#[test]
fn artifact_store_rejects_malformed_dependencies() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    let input = temp.path().join("dto.json");
    let valid = format!("b3:{}", "0".repeat(64));
    for dependencies in [
        serde_json::json!([1]),
        serde_json::json!([valid, {"hash": valid}]),
        serde_json::json!(valid),
        serde_json::json!(["../outside"]),
        serde_json::json!(["b3:missing"]),
    ] {
        write_json(&input, serde_json::json!({"name": "x", "dependencies": dependencies}));
        let output = store_artifact(&input, &store, temp.path());
        assert_eq!(output.status.code(), Some(2), "dependencies={dependencies}");
    }
    write_json(&input, serde_json::json!({"name": "x", "dependencies": [valid]}));
    let stored = stdout_json(&store_artifact(&input, &store, temp.path()));
    assert_eq!(stored["dependencies"], serde_json::json!([valid]));
}

fn run_with_home(args: &[&str], cwd: &Path, home: &str) -> std::process::Output {
    Command::new(cargo_octet()).args(args).current_dir(cwd).env("HOME", home).output().unwrap()
}

/// A fresh checkout directory and its canonical path, which `current_dir` reports as the workspace.
fn checkout() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().canonicalize().unwrap().to_str().unwrap().to_owned();
    (dir, root)
}

#[test]
fn artifact_identity_excludes_secret_fields_and_local_root_presentation() {
    let mut hashes = Vec::new();
    for (index, home) in ["/home/alice", "/Users/bob"].iter().enumerate() {
        let (dir, workspace) = checkout();
        let store = dir.path().join("store");
        let input = dir.path().join("dto.json");
        let mut dto = serde_json::json!({
            "prompt": "x",
            "workspace": workspace,
            "file": format!("{workspace}/src/lib.rs"),
            "nested": [{"accessToken": format!("tok-{index}"), "config": format!("{home}/.config/octet.toml")}],
            "max_tokens": 5
        });
        dto[["api_key", "API_KEY"][index]] = serde_json::json!(format!("sk-live-{index}"));
        write_json(&input, dto);
        let args = [
            "roadmap",
            "artifact",
            "store",
            "--kind",
            "prompt",
            "--input",
            input.to_str().unwrap(),
            "--store-dir",
            store.to_str().unwrap(),
        ];
        let hash = stdout_json(&run_with_home(&args, dir.path(), home))["hash"].as_str().unwrap().to_owned();
        let stored = fs::read_to_string(record_path(&store, &hash)).unwrap();
        for leaked in [
            format!("sk-live-{index}"),
            format!("tok-{index}"),
            (*home).to_owned(),
            workspace.clone(),
        ] {
            assert!(!stored.contains(&leaked), "stored record leaked {leaked}: {stored}");
        }
        let record: serde_json::Value = serde_json::from_str(&stored).unwrap();
        assert_eq!(record["normalized"]["workspace"], "<workspace>");
        assert_eq!(record["normalized"]["file"], "<workspace>/src/lib.rs");
        assert_eq!(record["normalized"]["nested"][0]["config"], "<home>/.config/octet.toml");
        assert_eq!(record["normalized"]["max_tokens"], 5);
        hashes.push(hash);
    }
    assert_eq!(hashes[0], hashes[1], "DTOs differing only in secrets and local roots must share an identity");

    let (dir, workspace) = checkout();
    let store = dir.path().join("store");
    let identity = |file: &str, max_tokens: u32, name: &str| {
        stored_hash(
            serde_json::json!({"prompt": "x", "file": file, "max_tokens": max_tokens}),
            name,
            &store,
            dir.path(),
        )
    };
    let base = identity(&format!("{workspace}/src/lib.rs"), 5, "base.json");
    assert_ne!(
        identity(&format!("{workspace}/src/main.rs"), 5, "relative.json"),
        base,
        "in-root remainders are content"
    );
    assert_ne!(
        identity(&format!("{workspace}/src/lib.rs"), 6, "budget.json"),
        base,
        "non-secret token counts are content"
    );
}

#[test]
fn artifact_identity_keeps_absolute_strings_outside_local_roots() {
    let (dir, _) = checkout();
    let store = dir.path().join("store");
    for (index, (left, right)) in [
        ("/v1/chat/completions", "/v1/embeddings"),
        ("/definitions/finding/properties/id", "/definitions/finding/properties/path"),
        ("/srv/shared/a.toml", "/srv/shared/b.toml"),
    ]
    .iter()
    .enumerate()
    {
        let left_hash =
            stored_hash(serde_json::json!({"route": left}), &format!("left-{index}.json"), &store, dir.path());
        let right_hash =
            stored_hash(serde_json::json!({"route": right}), &format!("right-{index}.json"), &store, dir.path());
        assert_ne!(left_hash, right_hash, "{left} and {right} must keep distinct identities");
        let record: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(record_path(&store, &left_hash)).unwrap()).unwrap();
        assert_eq!(record["normalized"]["route"], *left);
    }
}

fn capture_capsule(manifest: serde_json::Value, name: &str, cwd: &Path, home: &str) -> serde_json::Value {
    let input = cwd.join(format!("{name}-input.json"));
    let capsule = cwd.join(format!("{name}.json"));
    write_json(&input, manifest);
    stdout_json(&run_with_home(
        &[
            "roadmap",
            "capsule",
            "capture",
            "--input",
            input.to_str().unwrap(),
            "--capsule",
            capsule.to_str().unwrap(),
        ],
        cwd,
        home,
    ))
}

fn replay_observation(observed: serde_json::Value, name: &str, cwd: &Path, home: &str) -> std::process::Output {
    let observed_path = cwd.join(format!("{name}-observed.json"));
    write_json(&observed_path, observed);
    run_with_home(
        &[
            "roadmap",
            "capsule",
            "replay",
            "--capsule",
            cwd.join(format!("{name}.json")).to_str().unwrap(),
            "--observed",
            observed_path.to_str().unwrap(),
        ],
        cwd,
        home,
    )
}

#[test]
fn replay_capsule_identity_excludes_local_root_presentation() {
    let mut hashes = Vec::new();
    for (index, home) in ["/home/alice", "/Users/bob"].iter().enumerate() {
        let (dir, workspace) = checkout();
        let mut manifest = capsule_manifest_input();
        manifest["command_argv"] = serde_json::json!([
            "cargo",
            "octet",
            "check",
            "--manifest-path",
            format!("{workspace}/Cargo.toml"),
            "--config",
            format!("{home}/.config/octet.toml")
        ]);
        let mut observation = manifest.clone();
        manifest["client_secret"] = serde_json::json!(format!("secret-{index}"));
        let captured = capture_capsule(manifest, "capsule", dir.path(), home);
        assert_eq!(captured["manifest"]["command_argv"][4], "<workspace>/Cargo.toml");
        assert_eq!(captured["manifest"]["command_argv"][6], "<home>/.config/octet.toml");
        let text = fs::read_to_string(dir.path().join("capsule.json")).unwrap();
        assert!(!text.contains(home), "capsule kept local path {home}");
        assert!(!text.contains(&workspace), "capsule kept workspace root {workspace}");
        assert!(!text.contains(&format!("secret-{index}")), "capsule kept secret");
        hashes.push(captured["capsule_hash"].clone());

        observation["exit_code"] = serde_json::json!(0);
        let replay = stdout_json(&replay_observation(observation, "capsule", dir.path(), home));
        assert_eq!(replay["status"], "replayed");
    }
    assert_eq!(hashes[0], hashes[1]);
}

#[test]
fn replay_capsule_reports_in_workspace_argv_and_binary_identity_drift() {
    let (dir, workspace) = checkout();
    let home = "/home/alice";
    let mut manifest = capsule_manifest_input();
    manifest["command_argv"] = serde_json::json!([
        "cargo",
        "octet",
        "check",
        "--manifest-path",
        format!("{workspace}/crates/a/Cargo.toml")
    ]);
    manifest["octet_binary_identity"] = serde_json::json!("/nix/store/aaaa-cargo-octet/bin/cargo-octet");
    let captured = capture_capsule(manifest.clone(), "capsule", dir.path(), home);
    assert_eq!(captured["manifest"]["octet_binary_identity"], "/nix/store/aaaa-cargo-octet/bin/cargo-octet");

    let mut observation = manifest;
    observation["exit_code"] = serde_json::json!(0);
    let exact = stdout_json(&replay_observation(observation.clone(), "capsule", dir.path(), home));
    assert_eq!(exact["status"], "replayed");

    for (field, drifted) in [
        (
            "command_argv",
            serde_json::json!([
                "cargo",
                "octet",
                "check",
                "--manifest-path",
                format!("{workspace}/crates/b/Cargo.toml")
            ]),
        ),
        ("octet_binary_identity", serde_json::json!("/nix/store/bbbb-cargo-octet/bin/cargo-octet")),
    ] {
        let mut observed = observation.clone();
        observed[field] = drifted;
        let output = replay_observation(observed, "capsule", dir.path(), home);
        assert_eq!(output.status.code(), Some(1), "{field} drift must fail closed");
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["status"], "drifted", "{field}");
        assert_eq!(report["mismatches"], serde_json::json!([field]));
    }
}

fn cache_check(key: serde_json::Value, output_hash: &str, cwd: &Path) -> std::process::Output {
    let key_path = cwd.join("cache-key.json");
    write_json(&key_path, key);
    run(
        &[
            "roadmap",
            "pure-cache",
            "check",
            "--key",
            key_path.to_str().unwrap(),
            "--store-dir",
            cwd.join("cache").to_str().unwrap(),
            "--output-hash",
            output_hash,
        ],
        cwd,
    )
}

#[test]
fn pure_cache_miss_names_changed_components_and_hit_checks_output_hash() {
    let temp = tempfile::tempdir().unwrap();
    let base = serde_json::json!({
        "command_identity": "cargo-check",
        "input_hashes": ["b3:src"],
        "config_hash": "b3:cfg",
        "tool_revision": "rev1",
        "environment_allowlist": {"RUSTFLAGS": "-Dwarnings"}
    });
    let first = stdout_json(&cache_check(base.clone(), "b3:out", temp.path()));
    assert_eq!(first["status"], "miss");
    assert_eq!(first["miss_reason"], "new-key");
    assert_eq!(first["changed_key_components"], serde_json::json!([]));

    let mut drifted = base.clone();
    drifted["tool_revision"] = serde_json::json!("rev2");
    drifted["environment_allowlist"] = serde_json::json!({"RUSTFLAGS": "-Cdebuginfo=0"});
    let drift_output = cache_check(drifted, "b3:out2", temp.path());
    let drift = stdout_json(&drift_output);
    assert_eq!(drift["status"], "miss");
    assert_eq!(drift["miss_reason"], "changed-key-components");
    assert_eq!(drift["changed_key_components"], serde_json::json!(["environment_allowlist", "tool_revision"]));
    assert!(
        !String::from_utf8_lossy(&drift_output.stdout).contains("debuginfo"),
        "miss receipt dumped the environment"
    );

    let mut other_command = base.clone();
    other_command["command_identity"] = serde_json::json!("cargo-doc");
    let new_command = stdout_json(&cache_check(other_command, "b3:doc", temp.path()));
    assert_eq!(new_command["miss_reason"], "new-key");

    let hit = stdout_json(&cache_check(base.clone(), "b3:out", temp.path()));
    assert_eq!(hit["status"], "hit");
    let mismatch = cache_check(base, "b3:different", temp.path());
    assert_eq!(mismatch.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&mismatch.stdout).unwrap();
    assert_eq!(report["status"], "output-mismatch");
    assert_eq!(report["stored_output_hash"], "b3:out");
    assert_eq!(report["observed_output_hash"], "b3:different");
}

#[test]
fn pure_cache_miss_skips_foreign_store_files_but_fails_closed_on_malformed_entries() {
    let key = serde_json::json!({
        "command_identity": "cargo-check",
        "input_hashes": ["b3:src"],
        "config_hash": "b3:cfg",
        "tool_revision": "rev1",
        "environment_allowlist": {}
    });
    let shared = tempfile::tempdir().unwrap();
    let cache = shared.path().join("cache");
    stored_hash(serde_json::json!({"name": "shared-store-artifact"}), "artifact.json", &cache, shared.path());
    write_json(&cache.join("notes.json"), serde_json::json!(["not", "a", "cache", "entry"]));
    let key_in_store = cache.join("key.json");
    write_json(&key_in_store, key.clone());
    let check_key_in_store = || {
        run(
            &[
                "roadmap",
                "pure-cache",
                "check",
                "--key",
                key_in_store.to_str().unwrap(),
                "--store-dir",
                cache.to_str().unwrap(),
                "--output-hash",
                "b3:out",
            ],
            shared.path(),
        )
    };
    let miss = stdout_json(&check_key_in_store());
    assert_eq!(miss["status"], "miss");
    assert_eq!(miss["miss_reason"], "new-key");
    let mut revised = key.clone();
    revised["tool_revision"] = serde_json::json!("rev2");
    write_json(&key_in_store, revised);
    let changed = stdout_json(&check_key_in_store());
    assert_eq!(changed["changed_key_components"], serde_json::json!(["tool_revision"]));

    for (index, malformed) in ["{", "[]", r#"{"cache_key": "b3:planted", "output_hash": "b3:out"}"#].iter().enumerate()
    {
        let temp = tempfile::tempdir().unwrap();
        let planted = temp.path().join("cache").join(format!("b3_{}.json", "0".repeat(64)));
        fs::create_dir_all(planted.parent().unwrap()).unwrap();
        fs::write(&planted, malformed).unwrap();
        let output = cache_check(key.clone(), "b3:out", temp.path());
        assert_eq!(output.status.code(), Some(2), "malformed entry {index} must fail closed");
        assert!(stderr_text(&output).contains(&format!("b3_{}.json", "0".repeat(64))), "{}", stderr_text(&output));
    }
}

fn migrate_session(session: serde_json::Value, cwd: &Path) -> (std::process::Output, std::path::PathBuf) {
    let input = cwd.join("migrate-input.json");
    let output = cwd.join("migrate-output.json");
    let _ = fs::remove_file(&output);
    write_json(&input, session);
    let result = run(
        &[
            "roadmap",
            "session",
            "migrate",
            "--input",
            input.to_str().unwrap(),
            "--output",
            output.to_str().unwrap(),
        ],
        cwd,
    );
    (result, output)
}

#[test]
fn session_migrate_reads_schema_version_and_requires_identity() {
    let temp = tempfile::tempdir().unwrap();
    let (current, written) = migrate_session(serde_json::json!({"identity": "b3:s", "schema_version": 1}), temp.path());
    assert_eq!(stdout_json(&current)["from_schema_version"], 1);
    assert!(written.exists());
    let (legacy, _) =
        migrate_session(serde_json::json!({"identity": "b3:s", "original_schema_version": 0}), temp.path());
    assert_eq!(stdout_json(&legacy)["from_schema_version"], 0);

    for session in [
        serde_json::json!({"identity": "b3:s", "schema_version": 2}),
        serde_json::json!({"identity": "b3:s", "original_schema_version": 7}),
        serde_json::json!({"identity": "b3:s", "schema_version": "1"}),
        serde_json::json!({"schema_version": 1}),
        serde_json::json!({"identity": 42, "schema_version": 1}),
    ] {
        let (output, written) = migrate_session(session.clone(), temp.path());
        assert_eq!(output.status.code(), Some(2), "session={session}");
        assert!(!written.exists(), "session={session} wrote migrated output");
    }
}
