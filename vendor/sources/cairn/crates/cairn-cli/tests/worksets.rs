use std::{
    env,
    error::Error,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
    time::SystemTime,
};

use serde_json::Value;

const OPENER_PROFILE: &str = "code-new-window";
const WORKSET_ID: &str = "shared-change";
const EXPECTED_MEMBER_COUNT: u64 = 2;
const EXECUTABLE_NAME: &str = "code";
const NO_SHELL_SENTINEL: &str = "shell-interpolation-did-not-run";

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy_path() -> PathBuf {
    repo_root().join("cairn-policy/generated/cairn-policy.json")
}

struct Fixture {
    root: PathBuf,
    data: PathBuf,
    first_member: PathBuf,
    second_member: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Result<Self, Box<dyn Error>> {
        let unique = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_nanos();
        let root = env::temp_dir().join(format!(
            "cairn-workset-cli-{name}-{}-{unique}",
            std::process::id()
        ));
        let data = root.join("data");
        let first_member = root.join("planning");
        let second_member = root.join("service");
        fs::create_dir_all(&first_member)?;
        fs::create_dir_all(&second_member)?;
        Ok(Self {
            root,
            data,
            first_member,
            second_member,
        })
    }

    fn command(&self) -> Command {
        let mut command = Command::new(env!("CARGO_BIN_EXE_cairn"));
        command
            .current_dir(&self.root)
            .env("CAIRN_DATA_DIR", &self.data);
        command
    }

    fn state_path(&self) -> PathBuf {
        self.data.join("cairn/worksets/state.json")
    }

    fn lock_path(&self) -> PathBuf {
        self.data.join("cairn/worksets/.state.json.lock")
    }

    fn cleanup(self) -> Result<(), Box<dyn Error>> {
        fs::remove_dir_all(self.root)?;
        Ok(())
    }
}

fn add_policy(command: &mut Command) {
    command.arg("--policy").arg(policy_path());
}

fn create_command(fixture: &Fixture, execute: bool) -> Command {
    let mut command = fixture.command();
    command
        .arg("workset")
        .arg("create")
        .arg(WORKSET_ID)
        .arg("--member")
        .arg(&fixture.first_member)
        .arg("--member")
        .arg(&fixture.second_member)
        .arg("--opener")
        .arg(OPENER_PROFILE);
    add_policy(&mut command);
    if execute {
        command.arg("--execute");
    }
    command
}

fn workset_command(fixture: &Fixture, subcommand: &str) -> Command {
    let mut command = fixture.command();
    command.arg("workset").arg(subcommand).arg(WORKSET_ID);
    add_policy(&mut command);
    command
}

fn list_command(fixture: &Fixture) -> Command {
    let mut command = fixture.command();
    command.arg("workset").arg("list");
    add_policy(&mut command);
    command
}

fn successful_json(output: Output) -> Result<Value, Box<dyn Error>> {
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(serde_json::from_slice(&output.stdout)?)
}

fn failed_stderr(output: Output) -> String {
    assert!(!output.status.success());
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn canonical(path: &Path) -> Result<String, Box<dyn Error>> {
    Ok(fs::canonicalize(path)?
        .to_str()
        .ok_or("test path is not UTF-8")?
        .to_string())
}

fn find_executable(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let path = env::var_os("PATH").ok_or("PATH is not set")?;
    env::split_paths(&path)
        .map(|directory| directory.join(name))
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| format!("executable not found on PATH: {name}").into())
}

fn install_fake_opener(fixture: &Fixture) -> Result<PathBuf, Box<dyn Error>> {
    let bin = fixture.root.join("bin");
    fs::create_dir_all(&bin)?;
    fs::copy(find_executable("true")?, bin.join(EXECUTABLE_NAME))?;
    Ok(bin)
}

// r[verify cairn.lifecycle_stores.worksets.remove]
#[test]
fn create_list_show_and_remove_are_preview_first_and_preserve_members() -> Result<(), Box<dyn Error>>
{
    let fixture = Fixture::new("lifecycle")?;
    fs::write(fixture.first_member.join("keep"), "planning")?;
    fs::write(fixture.second_member.join("keep"), "service")?;

    let preview = successful_json(create_command(&fixture, false).output()?)?;
    assert_eq!(preview["executed"], false);
    assert_eq!(preview["mutated"], false);
    assert_eq!(preview["plan"]["execution"], "preview");
    assert!(!fixture.state_path().exists());

    let created = successful_json(create_command(&fixture, true).output()?)?;
    assert_eq!(created["executed"], true);
    assert_eq!(created["mutated"], true);
    assert!(fixture.state_path().is_file());

    let list = successful_json(list_command(&fixture).output()?)?;
    assert_eq!(list["read_only"], true);
    assert_eq!(
        list["worksets"].as_array().ok_or("worksets array")?.len(),
        1
    );
    assert_eq!(list["worksets"][0]["id"], WORKSET_ID);
    assert_eq!(list["worksets"][0]["member_count"], EXPECTED_MEMBER_COUNT);

    let shown = successful_json(workset_command(&fixture, "show").output()?)?;
    assert_eq!(shown["read_only"], true);
    assert_eq!(shown["workset"]["id"], WORKSET_ID);
    assert_eq!(
        shown["workset"]["members"]
            .as_array()
            .ok_or("member array")?
            .len(),
        EXPECTED_MEMBER_COUNT as usize
    );

    let remove_preview = successful_json(workset_command(&fixture, "remove").output()?)?;
    assert_eq!(remove_preview["mutated"], false);
    let still_listed = successful_json(list_command(&fixture).output()?)?;
    assert_eq!(
        still_listed["worksets"]
            .as_array()
            .ok_or("worksets array")?
            .len(),
        1
    );

    let mut remove_execute = workset_command(&fixture, "remove");
    remove_execute.arg("--execute");
    let removed = successful_json(remove_execute.output()?)?;
    assert_eq!(removed["mutated"], true);
    let empty = successful_json(list_command(&fixture).output()?)?;
    assert!(empty["worksets"]
        .as_array()
        .ok_or("worksets array")?
        .is_empty());
    assert_eq!(
        fs::read_to_string(fixture.first_member.join("keep"))?,
        "planning"
    );
    assert_eq!(
        fs::read_to_string(fixture.second_member.join("keep"))?,
        "service"
    );

    fixture.cleanup()
}

// r[verify cairn.lifecycle_stores.worksets.open]
#[test]
fn open_preview_has_exact_argv_and_execute_reports_the_process() -> Result<(), Box<dyn Error>> {
    let mut fixture = Fixture::new("open")?;
    let shell_named_member = fixture
        .root
        .join(format!("literal;touch {NO_SHELL_SENTINEL}"));
    fs::create_dir_all(&shell_named_member)?;
    fixture.second_member = shell_named_member;
    successful_json(create_command(&fixture, true).output()?)?;

    let empty_path = fixture.root.join("empty-path");
    fs::create_dir_all(&empty_path)?;
    let mut preview_command = workset_command(&fixture, "open");
    preview_command.env("PATH", &empty_path);
    let preview = successful_json(preview_command.output()?)?;
    assert_eq!(preview["executed"], false);
    assert!(preview["process"].is_null());
    assert_eq!(
        preview["plan"]["argv"],
        serde_json::json!([
            EXECUTABLE_NAME,
            "--new-window",
            canonical(&fixture.first_member)?,
            canonical(&fixture.second_member)?,
        ])
    );

    let fake_bin = install_fake_opener(&fixture)?;
    let mut execute = workset_command(&fixture, "open");
    execute.arg("--execute").env("PATH", fake_bin);
    let observed = successful_json(execute.output()?)?;
    assert_eq!(observed["executed"], true);
    assert_eq!(observed["mutated"], false);
    assert_eq!(observed["process"]["launched"], true);
    assert!(observed["process"]["success"].is_boolean());
    assert!(observed["process"]["exit_code"].is_number());
    assert_eq!(observed["process"]["argv"], observed["plan"]["argv"]);
    assert!(!fixture.root.join(NO_SHELL_SENTINEL).exists());

    fixture.cleanup()
}

// r[verify cairn.lifecycle_stores.worksets.opener_reject]
#[test]
fn unknown_opener_and_shell_interpolation_are_rejected_before_launch() -> Result<(), Box<dyn Error>>
{
    let fixture = Fixture::new("opener-reject")?;
    successful_json(create_command(&fixture, true).output()?)?;

    let mut unknown = workset_command(&fixture, "open");
    unknown.arg("--opener").arg("unknown-editor");
    let unknown_error = failed_stderr(unknown.output()?);
    assert!(
        unknown_error.contains("lifecycle_store.workset.unknown_opener"),
        "{unknown_error}"
    );

    let sentinel =
        env::temp_dir().join(format!("cairn-workset-shell-pwned-{}", std::process::id()));
    let _ = fs::remove_file(&sentinel);
    let interpolation = format!("$(touch${{IFS}}{})", sentinel.display());
    let mut injected = workset_command(&fixture, "open");
    injected.arg("--opener").arg(interpolation).arg("--execute");
    let injected_error = failed_stderr(injected.output()?);
    assert!(
        injected_error.contains("lifecycle_store.workset.invalid"),
        "{injected_error}"
    );
    assert!(!sentinel.exists());

    fixture.cleanup()
}

// r[verify cairn.lifecycle_stores.functional_boundary.shell]
#[test]
fn changed_member_path_blocks_open() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new("changed-member")?;
    successful_json(create_command(&fixture, true).output()?)?;
    let moved = fixture.root.join("planning-moved");
    fs::rename(&fixture.first_member, &moved)?;

    let error = failed_stderr(workset_command(&fixture, "open").output()?);
    assert!(
        error.contains("lifecycle_store.workset.changed_member"),
        "{error}"
    );
    assert!(moved.is_dir());
    assert!(fixture.state_path().is_file());

    fixture.cleanup()
}

#[test]
fn exclusive_lock_blocks_state_replacement() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new("lock")?;
    fs::create_dir_all(fixture.lock_path().parent().ok_or("lock parent")?)?;
    fs::write(fixture.lock_path(), "held")?;

    let error = failed_stderr(create_command(&fixture, true).output()?);
    assert!(
        error.contains("lifecycle_store.workset.stale_plan"),
        "{error}"
    );
    assert!(!fixture.state_path().exists());

    fixture.cleanup()
}

#[test]
fn tampered_member_path_hash_is_rejected() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new("tampered-state")?;
    successful_json(create_command(&fixture, true).output()?)?;
    let changed_member = fixture.root.join("changed-member");
    fs::create_dir_all(&changed_member)?;
    let bytes = fs::read(fixture.state_path())?;
    let mut state: Value = serde_json::from_slice(&bytes)?;
    state["worksets"][0]["members"][0]["display_path"] = Value::String(canonical(&changed_member)?);
    fs::write(fixture.state_path(), serde_json::to_vec_pretty(&state)?)?;

    let error = failed_stderr(list_command(&fixture).output()?);
    assert!(
        error.contains("lifecycle_store.workset.changed_member"),
        "{error}"
    );

    fixture.cleanup()
}

#[test]
fn malformed_policy_shell_profile_is_rejected() -> Result<(), Box<dyn Error>> {
    let fixture = Fixture::new("malformed-policy")?;
    let policy_bytes = fs::read(policy_path())?;
    let mut policy: Value = serde_json::from_slice(&policy_bytes)?;
    policy["lifecycle_store_policy"]["opener_profiles"][0]["fixed_args"] =
        serde_json::json!(["$(touch-pwned)"]);
    let unsafe_policy = fixture.root.join("unsafe-policy.json");
    fs::write(&unsafe_policy, serde_json::to_vec_pretty(&policy)?)?;

    let mut command = fixture.command();
    command
        .arg("workset")
        .arg("create")
        .arg(WORKSET_ID)
        .arg("--member")
        .arg(&fixture.first_member)
        .arg("--opener")
        .arg(OPENER_PROFILE)
        .arg("--policy")
        .arg(unsafe_policy);
    let error = failed_stderr(command.output()?);
    assert!(
        error
            .contains("policy has invalid field lifecycle_store_policy.opener_profiles.fixed_args"),
        "{error}"
    );
    assert!(!fixture.state_path().exists());

    fixture.cleanup()
}
