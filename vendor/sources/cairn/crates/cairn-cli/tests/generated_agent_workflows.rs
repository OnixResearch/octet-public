use std::{
    error::Error,
    fs, io,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::Value;

const ADAPTER_SKILLS: &str = "agent-skills";
const SKILLS_DESTINATION: &str = ".agents/skills";
const EXPECTED_WORKFLOW_COUNT: usize = 6;
const MODIFIED_CONTENT: &str = "operator-owned content\n";
const UPDATE_GOAL_SUFFIX: &str = " Use the reviewed update policy.";

// r[verify cairn.generated_agent_workflows.descriptors]
// r[verify cairn.generated_agent_workflows.descriptors.valid]
// r[verify cairn.generated_agent_workflows.descriptors.invalid]
// r[verify cairn.generated_agent_workflows.catalog]
// r[verify cairn.generated_agent_workflows.catalog.list]
// r[verify cairn.generated_agent_workflows.catalog.contracts]
// r[verify cairn.generated_agent_workflows.explore]
// r[verify cairn.generated_agent_workflows.explore.valid]
// r[verify cairn.generated_agent_workflows.explore.invalid]
// r[verify cairn.generated_agent_workflows.adapters]
// r[verify cairn.generated_agent_workflows.adapters.valid]
// r[verify cairn.generated_agent_workflows.adapters.authority]
// r[verify cairn.generated_agent_workflows.adapters.unknown]
// r[verify cairn.generated_agent_workflows.commands]
// r[verify cairn.generated_agent_workflows.commands.read]
// r[verify cairn.generated_agent_workflows.commands.destination]
// r[verify cairn.generated_agent_workflows.manifest]
// r[verify cairn.generated_agent_workflows.manifest.plan]
// r[verify cairn.generated_agent_workflows.manifest.freshness]
// r[verify cairn.generated_agent_workflows.install]
// r[verify cairn.generated_agent_workflows.install.fresh]
// r[verify cairn.generated_agent_workflows.install.conflict]
// r[verify cairn.generated_agent_workflows.install.obsolete]
// r[verify cairn.generated_agent_workflows.boundary]
// r[verify cairn.generated_agent_workflows.boundary.nonclaim]
// r[verify cairn.generated_agent_workflows.boundary.effects]
// r[verify cairn.generated_agent_workflows.functional_boundary]
// r[verify cairn.generated_agent_workflows.functional_boundary.pure]
// r[verify cairn.generated_agent_workflows.functional_boundary.shell]
// r[verify cairn.generated_agent_workflows.compatibility]
// r[verify cairn.generated_agent_workflows.compatibility.existing]
// r[verify cairn.generated_agent_workflows.verification]
// r[verify cairn.generated_agent_workflows.verification.suites]
// r[verify cairn.generated_agent_workflows.docs]
// r[verify cairn.generated_agent_workflows.docs.install]
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repository root")
        .to_path_buf()
}

fn policy_path() -> PathBuf {
    repo_root().join("cairn-policy/generated/cairn-policy.json")
}

fn test_root(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let root = std::env::temp_dir().join(format!(
        "cairn-generated-agent-workflows-{name}-{}",
        std::process::id()
    ));
    match fs::remove_dir_all(&root) {
        Ok(()) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    fs::create_dir_all(root.join(".cairn"))?;
    fs::create_dir_all(root.join(".cairn/specs"))?;
    fs::create_dir_all(root.join(".cairn/changes"))?;
    fs::create_dir_all(root.join(".cairn/archive"))?;
    Ok(root)
}

fn cairn() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
    command
        .current_dir(repo_root())
        .env_remove("CAIRN_GLOBAL_REGISTRY");
    command
}

fn run(args: &[&str]) -> Result<(Output, Option<Value>), Box<dyn Error>> {
    let output = cairn().args(args).output()?;
    let value = if output.stdout.is_empty() {
        None
    } else {
        Some(serde_json::from_slice(&output.stdout)?)
    };
    Ok((output, value))
}

fn destination_args(root: &Path, command: &str, adapter: &str, destination: &str) -> Vec<String> {
    vec![
        "agent".to_string(),
        "workflow".to_string(),
        command.to_string(),
        "--root".to_string(),
        root.display().to_string(),
        "--policy".to_string(),
        policy_path().display().to_string(),
        "--adapter".to_string(),
        adapter.to_string(),
        "--destination".to_string(),
        destination.to_string(),
    ]
}

fn run_owned(args: &[String]) -> Result<(Output, Option<Value>), Box<dyn Error>> {
    let borrowed = args.iter().map(String::as_str).collect::<Vec<_>>();
    run(&borrowed)
}

fn install(root: &Path, adapter: &str, destination: &str) -> Result<Value, Box<dyn Error>> {
    let preview_args = destination_args(root, "install", adapter, destination);
    let (preview_output, preview) = run_owned(&preview_args)?;
    assert!(preview_output.status.success());
    let preview = preview.ok_or("missing preview")?;
    let plan_hash = preview["plan_hash"].as_str().ok_or("missing plan hash")?;
    let mut execute_args = preview_args;
    execute_args.extend([
        "--execute".to_string(),
        "--plan-hash".to_string(),
        plan_hash.to_string(),
    ]);
    let (output, receipt) = run_owned(&execute_args)?;
    if !output.status.success() {
        return Err(format!(
            "install failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }
    receipt.ok_or_else(|| "missing install receipt".into())
}

#[test]
fn lists_descriptors_and_plans_each_adapter_without_writes() -> Result<(), Box<dyn Error>> {
    let policy = policy_path().display().to_string();
    let (list_output, list) = run(&["agent", "workflow", "list", "--policy", &policy])?;
    assert!(list_output.status.success());
    let list = list.ok_or("missing list receipt")?;
    assert_eq!(
        list["workflows"].as_array().map(Vec::len),
        Some(EXPECTED_WORKFLOW_COUNT)
    );
    assert_eq!(list["adapters"].as_array().map(Vec::len), Some(3));
    let (show_output, show) = run(&["agent", "workflow", "show", "explore", "--policy", &policy])?;
    assert!(show_output.status.success());
    let show = show.ok_or("missing show receipt")?;
    assert_eq!(show["workflow"]["mutation_class"], "read_only");
    assert!(show["workflow"]["constraints"]
        .as_array()
        .is_some_and(|values| values.iter().any(|value| value
            .as_str()
            .is_some_and(|text| text.contains("Do not create a change")))));

    for (adapter, destination, expected_suffix) in [
        ("agent-skills", ".agents/skills", "/SKILL.md"),
        ("claude", ".claude/commands", ".md"),
        ("codex", ".codex/prompts", ".md"),
    ] {
        let root = test_root(adapter)?;
        let args = destination_args(&root, "plan", adapter, destination);
        let (output, receipt) = run_owned(&args)?;
        assert!(output.status.success());
        let receipt = receipt.ok_or("missing plan receipt")?;
        assert_eq!(receipt["dry_run"], true);
        assert_eq!(receipt["mutated"], false);
        assert!(receipt["files"].as_array().is_some_and(|files| {
            files.len() == EXPECTED_WORKFLOW_COUNT
                && files.iter().all(|file| {
                    file["relative_path"]
                        .as_str()
                        .is_some_and(|path| path.ends_with(expected_suffix))
                })
        }));
        assert!(!root.join(destination).exists());
    }
    Ok(())
}

#[test]
fn machine_mode_preserves_catalog_receipt_identity() -> Result<(), Box<dyn Error>> {
    let policy = policy_path().display().to_string();
    let (default_output, default_receipt) =
        run(&["agent", "workflow", "list", "--policy", &policy])?;
    assert!(default_output.status.success());
    let default_receipt = default_receipt.ok_or("default catalog receipt")?;
    let (machine_output, machine) = run(&[
        "--machine",
        "agent",
        "workflow",
        "list",
        "--policy",
        &policy,
    ])?;
    assert!(machine_output.status.success());
    let machine = machine.ok_or("machine envelope")?;
    assert_eq!(machine["ok"], true);
    assert_eq!(
        machine["result"]["receipt_hash"],
        default_receipt["receipt_hash"]
    );
    Ok(())
}

#[test]
fn installs_and_checks_manifest_owned_files() -> Result<(), Box<dyn Error>> {
    let root = test_root("install-remove")?;
    let installed = install(&root, ADAPTER_SKILLS, SKILLS_DESTINATION)?;
    assert_eq!(installed["executed"], true);
    assert_eq!(installed["mutated"], true);
    let files = installed["files"].as_array().ok_or("missing files")?;
    assert_eq!(files.len(), EXPECTED_WORKFLOW_COUNT);
    for file in files {
        let path = file["relative_path"].as_str().ok_or("missing path")?;
        let contents = fs::read_to_string(root.join(path))?;
        assert!(contents.starts_with("---\nname: cairn-"));
        assert!(contents.contains("## Non-claims"));
    }

    let check_args = destination_args(&root, "check", ADAPTER_SKILLS, SKILLS_DESTINATION);
    let (check_output, check) = run_owned(&check_args)?;
    assert!(check_output.status.success());
    assert_eq!(check.ok_or("missing check")?["current"], true);

    for file in files {
        let path = file["relative_path"].as_str().ok_or("missing path")?;
        assert!(root.join(path).exists());
    }
    Ok(())
}

#[test]
fn managed_policy_update_replaces_only_stale_generated_files() -> Result<(), Box<dyn Error>> {
    let root = test_root("managed-update")?;
    let installed = install(&root, "codex", ".codex/prompts")?;
    let files = installed["files"].as_array().ok_or("installed files")?;
    let unchanged_path = files
        .iter()
        .find(|file| file["workflow_id"] == "verify")
        .and_then(|file| file["relative_path"].as_str())
        .ok_or("unchanged path")?;
    let unchanged_before = fs::read(root.join(unchanged_path))?;

    let mut policy: Value = serde_json::from_slice(&fs::read(policy_path())?)?;
    let descriptors = policy["generated_agent_workflow_policy"]["descriptors"]
        .as_array_mut()
        .ok_or("descriptors")?;
    let apply = descriptors
        .iter_mut()
        .find(|descriptor| descriptor["id"] == "apply")
        .ok_or("apply descriptor")?;
    let goal = apply["goal"].as_str().ok_or("apply goal")?.to_string();
    apply["goal"] = Value::String(format!("{goal}{UPDATE_GOAL_SUFFIX}"));
    let updated_policy_path = root.join("updated-policy.json");
    fs::write(&updated_policy_path, serde_json::to_vec_pretty(&policy)?)?;

    let mut preview_args = destination_args(&root, "install", "codex", ".codex/prompts");
    let policy_index = preview_args
        .iter()
        .position(|value| value == "--policy")
        .ok_or("policy option")?;
    preview_args[policy_index + 1] = updated_policy_path.display().to_string();
    let (preview_output, preview) = run_owned(&preview_args)?;
    assert!(preview_output.status.success());
    let preview = preview.ok_or("updated preview")?;
    assert!(preview["files"].as_array().is_some_and(|files| files
        .iter()
        .any(|file| file["workflow_id"] == "apply" && file["freshness"] == "stale-source")));
    let mut execute_args = preview_args;
    execute_args.extend([
        "--execute".to_string(),
        "--plan-hash".to_string(),
        preview["plan_hash"]
            .as_str()
            .ok_or("updated plan")?
            .to_string(),
    ]);
    let (execute_output, receipt) = run_owned(&execute_args)?;
    assert!(
        execute_output.status.success(),
        "{}",
        String::from_utf8_lossy(&execute_output.stderr)
    );
    let receipt = receipt.ok_or("updated receipt")?;
    let apply_path = receipt["files"]
        .as_array()
        .and_then(|files| files.iter().find(|file| file["workflow_id"] == "apply"))
        .and_then(|file| file["relative_path"].as_str())
        .ok_or("updated apply path")?;
    assert!(fs::read_to_string(root.join(apply_path))?.contains(UPDATE_GOAL_SUFFIX));
    assert_eq!(fs::read(root.join(unchanged_path))?, unchanged_before);
    Ok(())
}

#[test]
fn stale_plan_and_modified_managed_file_fail_without_replacement() -> Result<(), Box<dyn Error>> {
    let root = test_root("stale-modified")?;
    let preview_args = destination_args(&root, "install", "claude", ".claude/commands");
    let (preview_output, preview) = run_owned(&preview_args)?;
    assert!(preview_output.status.success());
    let preview = preview.ok_or("missing preview")?;
    let first_path = preview["files"][0]["relative_path"]
        .as_str()
        .ok_or("missing path")?;
    fs::create_dir_all(root.join(first_path).parent().ok_or("missing parent")?)?;
    fs::write(root.join(first_path), MODIFIED_CONTENT)?;
    let mut stale_args = preview_args.clone();
    stale_args.extend([
        "--execute".to_string(),
        "--plan-hash".to_string(),
        preview["plan_hash"]
            .as_str()
            .ok_or("missing plan")?
            .to_string(),
    ]);
    let (stale_output, _) = run_owned(&stale_args)?;
    assert!(!stale_output.status.success());
    assert!(String::from_utf8_lossy(&stale_output.stderr).contains("generated_workflow.plan.stale"));
    assert_eq!(fs::read_to_string(root.join(first_path))?, MODIFIED_CONTENT);
    assert_eq!(fs::read_dir(root.join(".claude/commands"))?.count(), 1);

    fs::remove_dir_all(root.join(".claude"))?;
    let installed = install(&root, "claude", ".claude/commands")?;
    let managed_path = installed["files"][0]["relative_path"]
        .as_str()
        .ok_or("missing managed path")?;
    fs::write(root.join(managed_path), MODIFIED_CONTENT)?;
    let (blocked_output, blocked) = run_owned(&preview_args)?;
    assert!(blocked_output.status.success());
    let blocked = blocked.ok_or("missing blocked plan")?;
    assert_eq!(blocked["blocked"], true);
    assert!(blocked["files"]
        .as_array()
        .is_some_and(|files| files.iter().any(|file| file["freshness"] == "modified")));
    assert_eq!(
        fs::read_to_string(root.join(managed_path))?,
        MODIFIED_CONTENT
    );
    Ok(())
}

#[test]
fn rejects_unsafe_destination_unknown_adapter_and_ambient_home_path() -> Result<(), Box<dyn Error>>
{
    let root = test_root("unsafe")?;
    for (adapter, destination) in [
        ("agent-skills", "../home"),
        ("agent-skills", "~/.agents/skills"),
        ("remote-package", ".agents/skills"),
    ] {
        let args = destination_args(&root, "plan", adapter, destination);
        let (output, receipt) = run_owned(&args)?;
        assert!(!output.status.success());
        assert!(receipt.is_none());
    }
    assert_eq!(fs::read_dir(&root)?.count(), 1);
    Ok(())
}

#[cfg(unix)]
#[test]
fn symlink_destination_is_blocked_without_outside_writes() -> Result<(), Box<dyn Error>> {
    use std::os::unix::fs::symlink;

    let root = test_root("symlink")?;
    let outside = test_root("symlink-outside")?;
    symlink(&outside, root.join(".agents"))?;
    let args = destination_args(&root, "install", ADAPTER_SKILLS, SKILLS_DESTINATION);
    let (output, preview) = run_owned(&args)?;
    assert!(output.status.success());
    let preview = preview.ok_or("missing preview")?;
    assert_eq!(preview["blocked"], true);
    assert!(preview["files"].as_array().is_some_and(|files| files
        .iter()
        .all(|file| file["freshness"] == "symlink-conflict")));
    assert_eq!(fs::read_dir(&outside)?.count(), 1);
    Ok(())
}

#[test]
fn maintained_hermes_source_remains_byte_compatible() {
    const EXPECTED_HERMES_BLAKE3: &str =
        "70ea834d431ac63c269a9de4ad400299701c4220902fc007299b4c9c324631d8";
    let bytes = include_bytes!("../../../docs/hermes/cairn-workflow-skill.md");
    assert_eq!(
        blake3::hash(bytes).to_hex().as_str(),
        EXPECTED_HERMES_BLAKE3
    );
}

#[test]
fn plan_receipts_are_checkout_independent_and_do_not_expose_roots() -> Result<(), Box<dyn Error>> {
    let first = test_root("portable-first")?;
    let second = test_root("portable-second")?;
    let first_args = destination_args(&first, "plan", "codex", ".codex/prompts");
    let second_args = destination_args(&second, "plan", "codex", ".codex/prompts");
    let (_, first_receipt) = run_owned(&first_args)?;
    let (_, second_receipt) = run_owned(&second_args)?;
    let first_receipt = first_receipt.ok_or("missing first receipt")?;
    let second_receipt = second_receipt.ok_or("missing second receipt")?;
    assert_eq!(first_receipt["plan_hash"], second_receipt["plan_hash"]);
    assert_eq!(
        first_receipt["receipt_hash"],
        second_receipt["receipt_hash"]
    );
    let serialized = serde_json::to_string(&first_receipt)?;
    assert!(!serialized.contains(&first.display().to_string()));
    assert!(!serialized.contains("HOME"));
    assert!(!serialized.contains("raw_prompt"));
    Ok(())
}
