use std::{error::Error, fs, path::PathBuf, process::Command};

use serde_json::Value;

const SCOPE: &str = "workspace:onix";
const STALE_HASH: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const TAMPERED_HASH: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command.current_dir(root());
    command
}

fn run_json(command: &mut Command) -> Result<Value, Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        output.status.success(),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn paths(name: &str) -> (PathBuf, PathBuf) {
    let base = std::env::temp_dir().join(format!(
        "cairn-global-registry-{name}-{}",
        std::process::id()
    ));
    (
        base.with_extension("json"),
        base.with_extension("export.json"),
    )
}

fn import_args(store: &str) -> Vec<String> {
    [
        "registry",
        "import",
        "--scope",
        SCOPE,
        "--adapter",
        "portable-json.v1",
        "--store",
        store,
        "--ingest",
        "cairn.canonical-records.v1",
        "--input",
        "fixtures/global-registry/canonical-records.json",
    ]
    .into_iter()
    .map(ToOwned::to_owned)
    .collect()
}

#[test]
// r[verify cairn.global_registry.scope]
// r[verify cairn.global_registry.adapters]
// r[verify cairn.global_registry.docs]
// r[verify cairn.global_registry.docs.non_claims]
// r[verify cairn.global_registry.tags]
// r[verify cairn.global_registry.dag_profiles]
fn generated_policy_owns_registry_adapters_tags_and_graph_profiles() -> Result<(), Box<dyn Error>> {
    let policy: Value = serde_json::from_slice(&fs::read(
        root().join("cairn-policy/generated/cairn-policy.json"),
    )?)?;
    let registry = &policy["global_registry_policy"];
    assert_eq!(registry["schema"], "cairn.global-registry-snapshot.v1");
    let adapter_ids = registry["adapters"]
        .as_array()
        .ok_or("adapters")?
        .iter()
        .filter_map(|row| row["id"].as_str())
        .collect::<Vec<_>>();
    assert!(adapter_ids.contains(&"portable-json.v1"));
    assert!(adapter_ids.contains(&"redb.v1"));
    assert!(registry["tag_namespaces"]
        .as_array()
        .ok_or("tag namespaces")?
        .iter()
        .any(|row| row["id"] == "cairn.policy" && row["authority"] == "reserved-policy"));
    assert!(registry["graph_profiles"]
        .as_array()
        .ok_or("graph profiles")?
        .iter()
        .any(|row| row["id"] == "change-dependencies" && row["require_dag"] == true));
    Ok(())
}

#[test]
// r[verify cairn.global_registry.scope.explicit]
// r[verify cairn.global_registry.query.tags]
// r[verify cairn.global_registry.query.graph]
// r[verify cairn.global_registry.portability]
// r[verify cairn.global_registry.portability.export]
// r[verify cairn.global_registry.portability.verify]
// r[verify cairn.global_registry.validation.positive]
// r[verify cairn.lifecycle_stores.root_cli.compatibility]
fn explicit_portable_registry_round_trip_is_snapshot_bound() -> Result<(), Box<dyn Error>> {
    let (store, output) = paths("round-trip");
    for path in [&store, &output] {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    let store_text = store.to_str().ok_or("store path")?;
    let mut dry_args = import_args(store_text);
    let dry = run_json(cairn().args(&dry_args))?;
    assert_eq!(dry["valid"], true, "{dry:#}");
    assert_eq!(dry["dry_run"], true);
    assert_eq!(dry["mutated"], false);
    assert!(!store.exists());

    dry_args.push("--execute".to_string());
    let imported = run_json(cairn().args(&dry_args))?;
    assert_eq!(imported["valid"], true, "{imported:#}");
    assert_eq!(imported["mutated"], true);
    assert!(store.exists());
    let snapshot_hash = imported["candidate_snapshot"]["snapshot_hash"].clone();

    let query = run_json(cairn().args([
        "registry",
        "query",
        "--scope",
        SCOPE,
        "--adapter",
        "portable-json.v1",
        "--registry",
        store_text,
        "--tag",
        "operator:channel=dev",
    ]))?;
    assert_eq!(query["valid"], true);
    assert_eq!(query["snapshot_hash"], snapshot_hash);
    assert_eq!(
        query["query"]["selected_ids"],
        serde_json::json!(["change:alpha"])
    );
    let exact = run_json(cairn().args([
        "registry",
        "query",
        "--scope",
        SCOPE,
        "--adapter",
        "portable-json.v1",
        "--registry",
        store_text,
        "--id",
        "change:beta",
    ]))?;
    assert_eq!(
        exact["query"]["selected_ids"],
        serde_json::json!(["change:beta"])
    );
    let missing = run_json(cairn().args([
        "registry",
        "query",
        "--scope",
        SCOPE,
        "--adapter",
        "portable-json.v1",
        "--registry",
        store_text,
        "--id",
        "change:missing",
    ]))?;
    assert_eq!(missing["valid"], false);
    assert_eq!(
        missing["query"]["issues"][0]["code"],
        "registry.query_id_unknown"
    );
    let repeated_query = run_json(cairn().args([
        "registry",
        "query",
        "--scope",
        SCOPE,
        "--adapter",
        "portable-json.v1",
        "--registry",
        store_text,
        "--tag",
        "operator:channel=dev",
    ]))?;
    assert_eq!(
        query["query"]["receipt_hash"],
        repeated_query["query"]["receipt_hash"]
    );

    let graph = run_json(cairn().args([
        "registry",
        "graph",
        "--scope",
        SCOPE,
        "--adapter",
        "portable-json.v1",
        "--registry",
        store_text,
        "--profile",
        "change-dependencies",
    ]))?;
    assert_eq!(graph["valid"], true, "{graph:#}");
    assert_eq!(
        graph["graph"]["topological_order"],
        serde_json::json!(["change:beta", "change:alpha"])
    );
    let repeated_graph = run_json(cairn().args([
        "registry",
        "graph",
        "--scope",
        SCOPE,
        "--adapter",
        "portable-json.v1",
        "--registry",
        store_text,
        "--profile",
        "change-dependencies",
    ]))?;
    assert_eq!(
        graph["graph"]["receipt_hash"],
        repeated_graph["graph"]["receipt_hash"]
    );

    let verified = run_json(cairn().args(["registry", "verify", "--registry", store_text]))?;
    assert_eq!(verified["valid"], true, "{verified:#}");

    let output_text = output.to_str().ok_or("output path")?;
    let exported = run_json(cairn().args([
        "registry",
        "export",
        "--scope",
        SCOPE,
        "--adapter",
        "portable-json.v1",
        "--registry",
        store_text,
        "--output",
        output_text,
        "--execute",
    ]))?;
    assert_eq!(exported["snapshot_hash"], snapshot_hash);
    assert_eq!(fs::read(&store)?, fs::read(&output)?);
    fs::remove_file(store)?;
    fs::remove_file(output)?;
    Ok(())
}

#[test]
// r[verify cairn.global_registry.fcis.commit]
// r[verify cairn.global_registry.query.failure]
// r[verify cairn.global_registry.validation.negative]
fn stale_snapshot_and_tampering_fail_without_corrupting_store() -> Result<(), Box<dyn Error>> {
    let (store, _) = paths("negative");
    if store.exists() {
        fs::remove_file(&store)?;
    }
    let store_text = store.to_str().ok_or("store path")?;
    let mut args = import_args(store_text);
    args.push("--execute".to_string());
    let imported = run_json(cairn().args(&args))?;
    assert_eq!(imported["valid"], true);
    let before = fs::read(&store)?;

    let mut stale = import_args(store_text);
    stale.extend([
        "--expected-snapshot".to_string(),
        STALE_HASH.to_string(),
        "--execute".to_string(),
    ]);
    let blocked = run_json(cairn().args(&stale))?;
    assert_eq!(blocked["valid"], false);
    assert_eq!(blocked["blocked"], true);
    assert_eq!(blocked["mutated"], false);
    assert_eq!(fs::read(&store)?, before);

    let mut value: Value = serde_json::from_slice(&before)?;
    value["nodes"][0]["content_hash"] = Value::String(TAMPERED_HASH.to_string());
    fs::write(&store, serde_json::to_vec_pretty(&value)?)?;
    let verification = run_json(cairn().args(["registry", "verify", "--registry", store_text]))?;
    assert_eq!(verification["valid"], false);
    assert!(verification["report"]["issues"]
        .as_array()
        .unwrap()
        .iter()
        .any(|issue| issue["code"] == "registry.record_hash_mismatch"));
    fs::remove_file(store)?;
    Ok(())
}

#[test]
// r[verify cairn.global_registry.scope.local_independence]
// r[verify cairn.global_registry.integration_boundary]
// r[verify cairn.global_registry.integration_boundary.dependency]
// r[verify cairn.global_registry.integration_boundary.readiness]
fn root_local_commands_ignore_ambient_registry_state() -> Result<(), Box<dyn Error>> {
    let ambient = std::env::temp_dir().join(format!(
        "cairn-invalid-ambient-registry-{}",
        std::process::id()
    ));
    fs::write(&ambient, b"not-json")?;
    let status = run_json(cairn().env("CAIRN_GLOBAL_REGISTRY", &ambient).args([
        "status",
        "--root",
        "fixtures/basic",
    ]))?;
    assert_eq!(status["layout"], "cairn");
    let validation = run_json(cairn().env("CAIRN_GLOBAL_REGISTRY", &ambient).args([
        "validate",
        "--root",
        "fixtures/basic",
    ]))?;
    assert_eq!(validation["valid"], true, "{validation:#}");
    fs::remove_file(ambient)?;
    Ok(())
}

#[cfg(feature = "redb-registry")]
#[test]
// r[verify cairn.global_registry.storage.redb]
// r[verify cairn.global_registry.portability.export]
fn redb_cli_exports_backend_independent_snapshot() -> Result<(), Box<dyn Error>> {
    let base = std::env::temp_dir().join(format!(
        "cairn-global-registry-redb-cli-{}",
        std::process::id()
    ));
    let store = base.with_extension("redb");
    let export = base.with_extension("json");
    for path in [&store, &export] {
        if path.exists() {
            fs::remove_file(path)?;
        }
    }
    let store_text = store.to_str().ok_or("redb path")?;
    let imported = run_json(cairn().args([
        "registry",
        "import",
        "--scope",
        SCOPE,
        "--adapter",
        "redb.v1",
        "--store",
        store_text,
        "--ingest",
        "cairn.canonical-records.v1",
        "--input",
        "fixtures/global-registry/canonical-records.json",
        "--execute",
    ]))?;
    assert_eq!(imported["valid"], true, "{imported:#}");
    let exported = run_json(cairn().args([
        "registry",
        "export",
        "--scope",
        SCOPE,
        "--adapter",
        "redb.v1",
        "--registry",
        store_text,
        "--output",
        export.to_str().ok_or("export path")?,
        "--execute",
    ]))?;
    assert_eq!(
        exported["snapshot_hash"],
        imported["candidate_snapshot"]["snapshot_hash"]
    );
    let verified = run_json(cairn().args([
        "registry",
        "verify",
        "--registry",
        export.to_str().ok_or("export path")?,
    ]))?;
    assert_eq!(verified["valid"], true);
    fs::remove_file(store)?;
    fs::remove_file(export)?;
    Ok(())
}

#[test]
// r[verify cairn.global_registry.adapters.unknown]
fn unknown_adapter_is_rejected_before_store_access() -> Result<(), Box<dyn Error>> {
    let missing = std::env::temp_dir().join("cairn-missing-registry.json");
    let output = cairn()
        .args([
            "registry",
            "query",
            "--scope",
            SCOPE,
            "--adapter",
            "unknown.v1",
            "--registry",
            missing.to_str().ok_or("missing path")?,
            "--id",
            "change:alpha",
        ])
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported registry adapter"));
    assert!(!missing.exists());
    Ok(())
}
