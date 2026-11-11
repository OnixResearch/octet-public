use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use cairn_core::{
    admit_lifecycle_store_identity, empty_lifecycle_store_locator,
    lifecycle_store_locator_projection, lifecycle_store_source_identity,
    plan_lifecycle_store_registration, LifecycleStoreIdentity, LifecycleStoreIdentityProjection,
    LIFECYCLE_STORE_IDENTITY_NON_CLAIMS, LIFECYCLE_STORE_IDENTITY_SCHEMA_V1,
};
use serde_json::{json, Value};

const PROJECT_SCHEMA: &str = "cairn.lifecycle-store-project.v1";
const STORE_SOURCE: &[u8] = b"{ schema = \"cairn.lifecycle-store-source.v1\" }\n";
const PROJECT_SOURCE: &[u8] = b"{ schema = \"cairn.lifecycle-store-project-source.v1\" }\n";
const PROJECT_NON_CLAIMS: &[&str] = &[
    "a project pointer selects one declared store but does not grant lifecycle authority",
    "a direct reference provides read-only discovery and does not satisfy dependencies, gates, evidence, or requirements",
    "reference indexing does not prove truth, freshness, ownership, implementation, or satisfaction",
    "project configuration does not authorize transitive reference traversal or transport",
    "project facts and advisory guidance do not override generated policy or command authorization",
    "project context checks do not prove truth, secrecy, semantic consistency, agent compliance, or release eligibility",
];

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy_path() -> PathBuf {
    repo_root().join("cairn-policy/generated/cairn-policy.json")
}

fn test_root(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let root = env::temp_dir().join(format!(
        "cairn-lifecycle-store-shell-{name}-{}",
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root)?;
    }
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn command(current_dir: &Path, data_dir: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command
        .current_dir(current_dir)
        .env("CAIRN_DATA_DIR", data_dir)
        .env_remove("XDG_DATA_HOME")
        .env_remove("CAIRN_GLOBAL_REGISTRY");
    command
}

fn run_success(command: &mut Command) -> Result<Value, Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        output.status.success(),
        "status: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn run_failure(command: &mut Command) -> Result<Output, Box<dyn Error>> {
    let output = command.output()?;
    assert!(
        !output.status.success(),
        "command unexpectedly succeeded:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(output)
}

fn create_native_root(root: &Path) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(root.join(".cairn/specs"))?;
    fs::create_dir_all(root.join(".cairn/changes"))?;
    fs::create_dir_all(root.join(".cairn/archive"))?;
    Ok(())
}

fn write_spec(root: &Path, id: &str, title: &str) -> Result<(), Box<dyn Error>> {
    let directory = root.join(".cairn/specs").join(id);
    fs::create_dir_all(&directory)?;
    fs::write(
        directory.join("spec.md"),
        format!("# {title}\n\n## Purpose\n\nPurpose for {title}.\n\n## Requirements\n"),
    )?;
    Ok(())
}

fn write_store(
    root: &Path,
    store_id: &str,
    specs: &[(&str, &str)],
) -> Result<LifecycleStoreIdentity, Box<dyn Error>> {
    create_native_root(root)?;
    fs::create_dir_all(root.join(".cairn-store"))?;
    fs::write(root.join(".cairn-store/store.ncl"), STORE_SOURCE)?;
    let projection = LifecycleStoreIdentityProjection {
        schema: LIFECYCLE_STORE_IDENTITY_SCHEMA_V1.to_string(),
        store_id: store_id.to_string(),
        remote_hint: None,
        non_claims: LIFECYCLE_STORE_IDENTITY_NON_CLAIMS
            .iter()
            .map(|claim| (*claim).to_string())
            .collect(),
        source_identity: lifecycle_store_source_identity(STORE_SOURCE),
    };
    fs::write(
        root.join(".cairn-store/store.json"),
        serde_json::to_vec_pretty(&projection)?,
    )?;
    for (id, title) in specs {
        write_spec(root, id, title)?;
    }
    admit_lifecycle_store_identity(&projection)
        .identity
        .ok_or_else(|| "test store identity admission failed".into())
}

fn write_locator(
    data_dir: &Path,
    stores: &[(&LifecycleStoreIdentity, &Path)],
) -> Result<(), Box<dyn Error>> {
    let mut state = empty_lifecycle_store_locator();
    for (identity, root) in stores {
        let plan = plan_lifecycle_store_registration(&state, identity, &root.to_string_lossy());
        assert!(
            !plan.blocked,
            "locator setup blocked: {:#?}",
            plan.diagnostics
        );
        state = plan.next_state.ok_or("locator plan omitted next state")?;
    }
    let path = data_dir.join("cairn/stores/registry.json");
    fs::create_dir_all(path.parent().ok_or("locator parent")?)?;
    fs::write(
        path,
        serde_json::to_vec_pretty(&lifecycle_store_locator_projection(&state))?,
    )?;
    Ok(())
}

fn write_project(
    root: &Path,
    lifecycle_store: Option<&str>,
    references: &[(&str, Option<&str>)],
    source: &[u8],
) -> Result<(), Box<dyn Error>> {
    fs::create_dir_all(root.join(".cairn"))?;
    fs::write(root.join(".cairn/project.ncl"), source)?;
    let pointer = lifecycle_store.map(|store_id| {
        json!({
            "store_id": store_id,
            "remote_hint": null,
        })
    });
    let references = references
        .iter()
        .map(|(store_id, remote_hint)| {
            json!({
                "store_id": store_id,
                "remote_hint": remote_hint,
            })
        })
        .collect::<Vec<_>>();
    fs::write(
        root.join(".cairn/project.json"),
        serde_json::to_vec_pretty(&json!({
            "schema": PROJECT_SCHEMA,
            "lifecycle_store": pointer,
            "references": references,
            "source_blake3": lifecycle_store_source_identity(source),
            "non_claims": PROJECT_NON_CLAIMS,
        }))?,
    )?;
    Ok(())
}

#[test]
fn explicit_store_selects_status_and_spec_reads() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_stores.root_resolution.explicit]
    // r[verify cairn.lifecycle_stores.root_cli.spec_fetch]
    // r[verify cairn.lifecycle_stores.verification]
    let base = test_root("explicit")?;
    let store = base.join("shared");
    let data = base.join("data");
    let identity = write_store(&store, "shared", &[("alpha", "Alpha")])?;
    write_locator(&data, &[(&identity, &store)])?;

    let status =
        run_success(command(&base, &data).args(["status", "--lifecycle-store", "shared"]))?;
    assert_eq!(
        status["resolution"]["selected"]["source"],
        "explicit_lifecycle_store"
    );
    assert_eq!(status["resolution"]["selected"]["store_id"], "shared");

    let list =
        run_success(command(&base, &data).args(["spec", "list", "--lifecycle-store", "shared"]))?;
    assert_eq!(list["specs"][0]["id"], "alpha");

    let shown = run_success(command(&base, &data).args([
        "spec",
        "show",
        "alpha",
        "--lifecycle-store",
        "shared",
    ]))?;
    assert_eq!(shown["spec"]["title"], "Alpha");
    assert_eq!(
        shown["resolution"]["selected"]["store_identity"],
        identity.store_identity.as_str()
    );
    fs::remove_dir_all(base)?;
    Ok(())
}

#[test]
fn setup_register_and_spec_show_share_the_data_override() -> Result<(), Box<dyn Error>> {
    let base = test_root("cross-command-data-override")?;
    let store = base.join("shared");
    let data = base.join("data");
    let store_text = store.to_str().ok_or("store path")?;

    let setup = run_success(command(&base, &data).args([
        "store",
        "setup",
        "shared",
        "--path",
        store_text,
        "--execute",
    ]))?;
    assert_eq!(setup["valid"], true);
    write_spec(&store, "alpha", "Alpha")?;

    let registered = run_success(command(&base, &data).args([
        "store",
        "register",
        store_text,
        "--id",
        "shared",
        "--execute",
    ]))?;
    assert_eq!(registered["valid"], true);
    assert!(data.join("cairn/stores/registry.json").is_file());

    let shown = run_success(command(&base, &data).args([
        "spec",
        "show",
        "alpha",
        "--lifecycle-store",
        "shared",
    ]))?;
    assert_eq!(shown["spec"]["title"], "Alpha");
    assert_eq!(
        shown["resolution"]["selected"]["source"],
        "explicit_lifecycle_store"
    );

    fs::remove_dir_all(base)?;
    Ok(())
}

#[test]
fn selector_conflict_fails_before_lifecycle_reads() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_stores.root_resolution.conflict]
    // r[verify cairn.lifecycle_stores.verification.suites]
    let base = test_root("selector-conflict")?;
    let output = run_failure(command(&base, &base.join("data")).args([
        "status",
        "--root",
        "/definitely/missing/root",
        "--lifecycle-store",
        "missing",
    ]))?;
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("lifecycle_store.root.selector_conflict"),
        "{stderr}"
    );
    assert!(!stderr.contains("failed to read"), "{stderr}");
    fs::remove_dir_all(base)?;
    Ok(())
}

#[test]
fn native_root_precedes_pointer_and_unknown_boundary_stays_local() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_stores.root_resolution.native_precedence]
    // r[verify cairn.lifecycle_stores.root_resolution.boundary]
    let base = test_root("native-precedence")?;
    let repo = base.join("repo");
    let nested = repo.join("nested");
    fs::create_dir_all(repo.join(".git"))?;
    fs::create_dir_all(&nested)?;
    create_native_root(&repo)?;
    write_spec(&repo, "local", "Local")?;
    write_project(&repo, Some("unfollowed"), &[], PROJECT_SOURCE)?;

    let status = run_success(command(&nested, &base.join("data")).arg("status"))?;
    assert_eq!(status["resolution"]["selected"]["source"], "native_root");
    assert_eq!(
        status["resolution"]["selected"]["root"],
        repo.to_string_lossy().as_ref()
    );
    assert!(status["resolution"]["diagnostics"]
        .as_array()
        .is_some_and(|findings| findings
            .iter()
            .any(|finding| { finding["code"] == "lifecycle_store.root.project_pointer_stale" })));

    let unrelated = base.join("unrelated");
    let child = unrelated.join("child");
    create_native_root(&unrelated)?;
    fs::create_dir_all(&child)?;
    let fallback = run_success(command(&child, &base.join("data")).arg("status"))?;
    assert_eq!(
        fallback["resolution"]["selected"]["source"],
        "classic_fallback"
    );
    assert_eq!(
        fallback["resolution"]["selected"]["root"],
        child.to_string_lossy().as_ref()
    );
    assert!(fallback["specs"].as_array().is_some_and(Vec::is_empty));
    fs::remove_dir_all(base)?;
    Ok(())
}

#[test]
fn checked_project_pointer_selects_registered_store() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_stores.project_config.pointer]
    // r[verify cairn.lifecycle_directory.hidden_default.runtime]
    // r[verify cairn.lifecycle_directory.hidden_default.project_pointer]
    let base = test_root("project-pointer")?;
    let store = base.join("shared");
    let project = base.join("project");
    let nested = project.join("src");
    let data = base.join("data");
    let identity = write_store(&store, "shared", &[("alpha", "Alpha")])?;
    write_locator(&data, &[(&identity, &store)])?;
    fs::create_dir_all(project.join(".git"))?;
    fs::create_dir_all(&nested)?;
    write_project(&project, Some("shared"), &[], PROJECT_SOURCE)?;

    let list = run_success(command(&nested, &data).args(["spec", "list"]))?;
    assert_eq!(list["resolution"]["selected"]["source"], "project_pointer");
    assert_eq!(list["resolution"]["selected"]["store_id"], "shared");
    assert_eq!(list["specs"][0]["id"], "alpha");
    fs::remove_dir_all(base)?;
    Ok(())
}

#[test]
fn missing_and_stale_store_rows_fail_closed() -> Result<(), Box<dyn Error>> {
    let base = test_root("missing-stale")?;
    let data = base.join("data");
    let missing =
        run_failure(command(&base, &data).args(["status", "--lifecycle-store", "missing"]))?;
    assert!(String::from_utf8_lossy(&missing.stderr).contains("lifecycle_store.root.store_missing"));

    let store = base.join("shared");
    let identity = write_store(&store, "shared", &[])?;
    write_locator(&data, &[(&identity, &store)])?;
    fs::write(
        store.join(".cairn-store/store.ncl"),
        b"{ schema = \"changed\" }\n",
    )?;
    let stale = run_failure(command(&base, &data).args(["status", "--lifecycle-store", "shared"]))?;
    assert!(String::from_utf8_lossy(&stale.stderr).contains("lifecycle_store.root.store_stale"));
    fs::remove_dir_all(base)?;
    Ok(())
}

#[test]
fn context_reads_only_direct_accepted_specs_without_authority() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_stores.references.index]
    // r[verify cairn.lifecycle_stores.references.non_authority]
    // r[verify cairn.lifecycle_stores.boundary]
    // r[verify cairn.lifecycle_stores.boundary.bounded]
    // r[verify cairn.lifecycle_stores.boundary.overclaim]
    let base = test_root("context-boundary")?;
    let subject = base.join("subject");
    let shared = base.join("shared");
    let data = base.join("data");
    create_native_root(&subject)?;
    write_project(&subject, None, &[("shared", None)], PROJECT_SOURCE)?;
    let shared_identity = write_store(&shared, "shared", &[("alpha", "Alpha")])?;
    fs::create_dir_all(shared.join(".cairn/changes/ready/specs/secret"))?;
    fs::write(
        shared.join(".cairn/changes/ready/specs/secret/spec.md"),
        "# Secret active change\n",
    )?;
    fs::create_dir_all(shared.join(".cairn"))?;
    fs::write(shared.join(".cairn/project.json"), "not-json")?;
    write_locator(&data, &[(&shared_identity, &shared)])?;

    let context = run_success(command(&base, &data).args([
        "context",
        "--root",
        subject.to_str().ok_or("subject path")?,
    ]))?;
    assert_eq!(context["specs"].as_array().map(Vec::len), Some(1));
    assert_eq!(context["resolution"]["selected"]["source"], "explicit_root");
    assert_eq!(context["specs"][0]["spec_id"], "alpha");
    assert_eq!(
        context["specs"][0]["fetch_command"],
        "cairn spec show alpha --lifecycle-store shared"
    );
    assert_eq!(context["boundary"]["traversal"], "direct_only");
    assert_eq!(context["boundary"]["content"], "accepted_specs_only");
    assert_eq!(context["boundary"]["authority_effect"], "none");
    assert!(!context.to_string().contains("Secret active change"));

    let validation = run_success(command(&base, &data).args([
        "validate",
        "--root",
        subject.to_str().ok_or("subject path")?,
        "--policy",
        policy_path().to_str().ok_or("policy path")?,
    ]))?;
    assert_eq!(validation["valid"], true);
    assert_eq!(
        validation["resolution"]["selected"]["root"],
        subject.to_string_lossy().as_ref()
    );
    fs::remove_dir_all(base)?;
    Ok(())
}

#[test]
fn duplicate_and_self_references_are_rejected() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_stores.project_config.invalid]
    let base = test_root("duplicate-self")?;
    let subject = base.join("subject");
    let data = base.join("data");
    let identity = write_store(&subject, "subject", &[])?;
    write_project(
        &subject,
        None,
        &[("subject", None), ("subject", None)],
        PROJECT_SOURCE,
    )?;
    write_locator(&data, &[(&identity, &subject)])?;

    let output =
        run_failure(command(&base, &data).args(["context", "--lifecycle-store", "subject"]))?;
    let context: Value = serde_json::from_slice(&output.stdout)?;
    let codes = context["reference_health"]
        .as_array()
        .ok_or("reference health")?
        .iter()
        .map(|finding| finding["code"].as_str().unwrap_or_default())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"lifecycle_store.reference.duplicate"));
    assert!(codes.contains(&"lifecycle_store.reference.self"));
    assert!(context["specs"].as_array().is_some_and(Vec::is_empty));
    fs::remove_dir_all(base)?;
    Ok(())
}

#[test]
fn doctor_reports_missing_reference_and_global_registry_is_ignored() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.lifecycle_stores.context.missing]
    // r[verify cairn.lifecycle_stores.transport_boundary.context]
    let base = test_root("doctor-isolation")?;
    let subject = base.join("subject");
    let data = base.join("data");
    create_native_root(&subject)?;
    write_project(
        &subject,
        None,
        &[("missing", Some("ssh://example.invalid/missing"))],
        PROJECT_SOURCE,
    )?;
    let ambient = base.join("ambient-global-registry.json");
    fs::write(&ambient, "not-json")?;

    let doctor = run_success(
        command(&base, &data)
            .env("CAIRN_GLOBAL_REGISTRY", &ambient)
            .args([
                "doctor",
                "--root",
                subject.to_str().ok_or("subject path")?,
                "--policy",
                policy_path().to_str().ok_or("policy path")?,
            ]),
    )?;
    assert_eq!(doctor["resolution"]["selected"]["source"], "explicit_root");
    let health = &doctor["lifecycle_store_context"]["reference_health"][0];
    assert_eq!(health["code"], "lifecycle_store.reference.store_missing");
    assert_eq!(
        health["repair_command"],
        "cairn store register <checkout-path> --id missing"
    );
    assert_eq!(
        health["manual_clone_hint"],
        "git clone 'ssh://example.invalid/missing' '<checkout-path>'"
    );
    assert_eq!(
        doctor["lifecycle_store_context"]["boundary"]["registry_lookup"],
        "local_locator_only"
    );
    fs::remove_dir_all(base)?;
    Ok(())
}
