use std::{
    io::{self, Read, Write},
    path::PathBuf,
    process::{Command, Stdio},
    thread,
};

use cairn_core::{
    build_machine_outcome, MachineContractBounds, MachineContractPolicy, MachineFailureKind,
    MachineOutcome, MachineProcessOutcome, CLI_MACHINE_MAX_PROCESS_STATUS,
    CLI_MACHINE_UNKNOWN_COMMAND,
};
use serde_json::Value;

use crate::support::{default_policy_path, load_policy_file};

pub(crate) const MACHINE_BOOTSTRAP_POLICY_STATUS: u8 = 4;
pub(crate) const MACHINE_HOST_OUTPUT_STATUS: u8 = 70;
const MACHINE_ENVELOPE_OVERHEAD_BYTES: usize = 64 * 1_024;
const INITIAL_CAPTURE_CAPACITY: usize = 8 * 1_024;
const JSON_FORMAT: &str = "json";

const KNOWN_COMMANDS: &[&str] = &[
    "init",
    "change",
    "compact",
    "template",
    "scaffold",
    "discover-specs",
    "spec",
    "status",
    "store",
    "context",
    "config",
    "doctor",
    "validate",
    "view",
    "workflow",
    "workset",
    "release-readiness",
    "release-decision",
    "release-evidence",
    "release-run",
    "lifecycle",
    "lifecycle-patterns",
    "determinism",
    "developer-loop",
    "agent",
    "adr",
    "agent-receipt",
    "evidence",
    "evidence-manifest",
    "hash-policy",
    "registry",
    "requirement-registry",
    "policy",
    "policy-exception",
    "preserves-corpus",
    "probe",
    "runtime-evidence",
    "rfd",
    "verification-obligation",
    "collaboration",
    "cob",
    "radicle",
    "state-machine-receipt",
    "traceability",
    "tracey",
    "review",
    "steel-orchestration",
    "test-harness",
    "toolchain-profile",
    "workspace-contract",
    "rename-classification",
    "migrate",
    "mcp",
    "gate",
    "gate-context",
    "sync",
    "archive",
    "receipt-preserves-diff",
    "receipt-verify-content-hash",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum MachineRunError {
    PolicyUnavailable,
    HostOutputFailure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MachineInvocation {
    command: String,
    args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum InvocationAdmission {
    Ready(MachineInvocation),
    Failure {
        command: String,
        kind: MachineFailureKind,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct BoundedBytes {
    bytes: Vec<u8>,
    exceeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ChildObservation {
    stdout: BoundedBytes,
    stderr: BoundedBytes,
    process_status: Option<u8>,
}

pub(crate) fn run_machine(raw_args: Vec<String>) -> Result<u8, MachineRunError> {
    // r[impl cairn.cli_machine_contract.mode]
    // r[impl cairn.cli_machine_contract.mode.select]
    // r[impl cairn.cli_machine_contract.envelope]
    // r[impl cairn.cli_machine_contract.exit_status]
    // r[impl cairn.cli_machine_contract.compatibility]
    // r[impl cairn.cli_machine_contract.functional_boundary.shell]
    // r[impl cairn.cli_machine_contract.boundary]
    let policy_path = selected_policy_path(&raw_args);
    let policy = load_policy_file(policy_path).map_err(|_| MachineRunError::PolicyUnavailable)?;
    let machine_policy = &policy.machine_contract_policy;
    let admission = admit_invocation(&raw_args);
    let outcome = match admission {
        InvocationAdmission::Failure { command, kind } => {
            controlled_failure(machine_policy, &command, vec![kind])?
        }
        InvocationAdmission::Ready(invocation) => {
            execute_machine_invocation(machine_policy, &invocation)?
        }
    };
    write_machine_outcome(machine_policy, outcome)
}

fn selected_policy_path(raw_args: &[String]) -> PathBuf {
    raw_args
        .windows(2)
        .find_map(|pair| (pair[0] == "--policy").then(|| PathBuf::from(&pair[1])))
        .unwrap_or_else(|| PathBuf::from(default_policy_path()))
}

fn admit_invocation(raw_args: &[String]) -> InvocationAdmission {
    let Some(command) = raw_args.first() else {
        return InvocationAdmission::Failure {
            command: CLI_MACHINE_UNKNOWN_COMMAND.to_string(),
            kind: MachineFailureKind::MissingCommand,
        };
    };
    if !is_known_command(command) {
        return InvocationAdmission::Failure {
            command: CLI_MACHINE_UNKNOWN_COMMAND.to_string(),
            kind: MachineFailureKind::UnknownCommand,
        };
    }
    let args = raw_args[1..].to_vec();
    if has_incompatible_presentation(command, &args) {
        return InvocationAdmission::Failure {
            command: command.clone(),
            kind: MachineFailureKind::IncompatibleOption,
        };
    }
    InvocationAdmission::Ready(MachineInvocation {
        command: command.clone(),
        args,
    })
}

fn is_known_command(command: &str) -> bool {
    KNOWN_COMMANDS.contains(&command)
}

fn has_incompatible_presentation(command: &str, args: &[String]) -> bool {
    if matches!(command, "mcp" | "--help" | "-h" | "help") {
        return true;
    }
    if args.iter().any(|arg| {
        matches!(
            arg.as_str(),
            "--machine" | "--interactive" | "--help" | "-h"
        )
    }) || args.first().map(String::as_str) == Some("help")
    {
        return true;
    }
    if args
        .windows(2)
        .any(|pair| pair[0] == "--format" && pair[1] != JSON_FORMAT)
    {
        return true;
    }
    if command == "policy" && args.first().map(String::as_str) == Some("export") {
        return true;
    }
    matches!(command, "traceability" | "tracey") && !args.iter().any(|arg| arg == "--json")
}

fn execute_machine_invocation(
    policy: &MachineContractPolicy,
    invocation: &MachineInvocation,
) -> Result<MachineProcessOutcome, MachineRunError> {
    let executable = std::env::current_exe().map_err(|_| MachineRunError::HostOutputFailure)?;
    let mut child_args = Vec::with_capacity(invocation.args.len().saturating_add(1));
    child_args.push(invocation.command.clone());
    child_args.extend(invocation.args.iter().cloned());
    let observation = match execute_child(executable, &child_args, &policy.bounds) {
        Ok(observation) => observation,
        Err(()) => {
            return controlled_failure(
                policy,
                &invocation.command,
                vec![MachineFailureKind::Internal],
            );
        }
    };
    if observation.stdout.exceeded || observation.stderr.exceeded {
        return controlled_failure(
            policy,
            &invocation.command,
            vec![MachineFailureKind::OutputTooLarge],
        );
    }
    let Some(process_status) = observation.process_status else {
        return controlled_failure(
            policy,
            &invocation.command,
            vec![MachineFailureKind::Internal],
        );
    };
    match serde_json::from_slice::<Value>(&observation.stdout.bytes) {
        Ok(result) if is_admitted_command_result(&result) => build_machine_outcome(
            policy,
            &invocation.command,
            MachineOutcome::CommandResult {
                result,
                process_status,
            },
        )
        .map_err(|_| MachineRunError::PolicyUnavailable),
        Ok(_) => controlled_failure(
            policy,
            &invocation.command,
            vec![MachineFailureKind::OutputInvalid],
        ),
        Err(_) if process_status == 0 => controlled_failure(
            policy,
            &invocation.command,
            vec![MachineFailureKind::OutputInvalid],
        ),
        Err(_) => controlled_failure(
            policy,
            &invocation.command,
            vec![classify_command_failure(
                &invocation.command,
                &invocation.args,
            )],
        ),
    }
}

fn is_admitted_command_result(value: &Value) -> bool {
    value.is_object()
}

fn controlled_failure(
    policy: &MachineContractPolicy,
    command: &str,
    failures: Vec<MachineFailureKind>,
) -> Result<MachineProcessOutcome, MachineRunError> {
    build_machine_outcome(
        policy,
        command,
        MachineOutcome::ControlledFailure { failures },
    )
    .map_err(|_| MachineRunError::PolicyUnavailable)
}

fn classify_command_failure(command: &str, args: &[String]) -> MachineFailureKind {
    let subcommand = args.first().map(String::as_str);
    if command == "policy" {
        return MachineFailureKind::Policy;
    }
    if matches!(command, "sync" | "archive") {
        return if has_positional_argument(args, 0) {
            MachineFailureKind::Blocked
        } else {
            MachineFailureKind::InvocationPreparation
        };
    }
    if command == "gate" {
        return if has_positional_argument(args, 0) && has_positional_argument(args, 1) {
            MachineFailureKind::NotFound
        } else {
            MachineFailureKind::InvocationPreparation
        };
    }
    if command == "gate-context" {
        return if has_positional_argument(args, 0) {
            MachineFailureKind::NotFound
        } else {
            MachineFailureKind::InvocationPreparation
        };
    }
    let lookup_identity_index = match (command, subcommand) {
        ("change", Some("show"))
        | ("spec", Some("show"))
        | ("store", Some("show" | "doctor" | "unregister"))
        | ("workset", Some("show" | "open" | "remove")) => Some(1),
        _ => None,
    };
    if let Some(index) = lookup_identity_index {
        return if has_positional_argument(args, index) {
            MachineFailureKind::NotFound
        } else {
            MachineFailureKind::InvocationPreparation
        };
    }
    MachineFailureKind::InvocationPreparation
}

fn has_positional_argument(args: &[String], index: usize) -> bool {
    args.get(index)
        .is_some_and(|argument| !argument.starts_with('-'))
}

fn execute_child(
    executable: PathBuf,
    args: &[String],
    bounds: &MachineContractBounds,
) -> Result<ChildObservation, ()> {
    let mut child = Command::new(executable)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| ())?;
    let Some(stdout) = child.stdout.take() else {
        terminate_child(&mut child);
        return Err(());
    };
    let Some(stderr) = child.stderr.take() else {
        terminate_child(&mut child);
        return Err(());
    };
    let stdout_bound = bounds.max_stdout_bytes;
    let stderr_bound = bounds.max_stderr_bytes;
    let stdout_reader = thread::spawn(move || read_bounded(stdout, stdout_bound));
    let stderr_reader = thread::spawn(move || read_bounded(stderr, stderr_bound));
    let status = match child.wait() {
        Ok(status) => status,
        Err(_) => {
            terminate_child(&mut child);
            return Err(());
        }
    };
    let stdout = stdout_reader.join().map_err(|_| ())?.map_err(|_| ())?;
    let stderr = stderr_reader.join().map_err(|_| ())?.map_err(|_| ())?;
    let process_status = status.code().and_then(admit_process_status);
    Ok(ChildObservation {
        stdout,
        stderr,
        process_status,
    })
}

fn terminate_child(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn read_bounded(mut reader: impl Read, maximum: usize) -> io::Result<BoundedBytes> {
    let read_limit = maximum.checked_add(1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "machine capture bound overflow",
        )
    })?;
    let initial_capacity = maximum.min(INITIAL_CAPTURE_CAPACITY);
    let mut bytes = Vec::with_capacity(initial_capacity);
    reader
        .by_ref()
        .take(u64::try_from(read_limit).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "machine capture bound conversion",
            )
        })?)
        .read_to_end(&mut bytes)?;
    let exceeded = bytes.len() > maximum;
    if exceeded {
        bytes.truncate(maximum);
    }
    Ok(BoundedBytes { bytes, exceeded })
}

fn admit_process_status(status: i32) -> Option<u8> {
    let converted = u8::try_from(status).ok()?;
    (converted <= CLI_MACHINE_MAX_PROCESS_STATUS).then_some(converted)
}

fn write_machine_outcome(
    policy: &MachineContractPolicy,
    mut outcome: MachineProcessOutcome,
) -> Result<u8, MachineRunError> {
    let maximum = policy
        .bounds
        .max_stdout_bytes
        .checked_add(MACHINE_ENVELOPE_OVERHEAD_BYTES)
        .ok_or(MachineRunError::PolicyUnavailable)?;
    let mut bytes = serde_json::to_vec_pretty(&outcome.envelope)
        .map_err(|_| MachineRunError::PolicyUnavailable)?;
    if bytes.len() > maximum && outcome.envelope.ok {
        outcome = controlled_failure(
            policy,
            &outcome.envelope.command,
            vec![MachineFailureKind::OutputTooLarge],
        )?;
        bytes = serde_json::to_vec_pretty(&outcome.envelope)
            .map_err(|_| MachineRunError::PolicyUnavailable)?;
    }
    if bytes.len() > maximum {
        return Err(MachineRunError::PolicyUnavailable);
    }
    bytes.push(b'\n');
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    handle
        .write_all(&bytes)
        .map_err(|_| MachineRunError::HostOutputFailure)?;
    Ok(outcome.process_status)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_CAPTURE_BOUND: usize = 4;

    #[test]
    fn admits_known_json_command() {
        let args = vec!["status".to_string(), "--root".to_string(), ".".to_string()];
        assert_eq!(
            admit_invocation(&args),
            InvocationAdmission::Ready(MachineInvocation {
                command: "status".to_string(),
                args: vec!["--root".to_string(), ".".to_string()],
            })
        );
    }

    #[test]
    fn admits_workflow_command() {
        let args = vec![
            "workflow".to_string(),
            "status".to_string(),
            "demo".to_string(),
        ];
        assert!(matches!(
            admit_invocation(&args),
            InvocationAdmission::Ready(_)
        ));
    }

    #[test]
    fn rejects_unknown_command_without_echoing_it() {
        let secret_like_command = "token-private-value".to_string();
        let admission = admit_invocation(std::slice::from_ref(&secret_like_command));
        assert_eq!(
            admission,
            InvocationAdmission::Failure {
                command: CLI_MACHINE_UNKNOWN_COMMAND.to_string(),
                kind: MachineFailureKind::UnknownCommand,
            }
        );
        assert!(!format!("{admission:?}").contains(&secret_like_command));
    }

    #[test]
    fn rejects_missing_command_and_command_arguments() {
        assert_eq!(
            admit_invocation(&[]),
            InvocationAdmission::Failure {
                command: CLI_MACHINE_UNKNOWN_COMMAND.to_string(),
                kind: MachineFailureKind::MissingCommand,
            }
        );
        assert_eq!(
            classify_command_failure("view", &[]),
            MachineFailureKind::InvocationPreparation
        );
        assert_eq!(
            classify_command_failure("sync", &[]),
            MachineFailureKind::InvocationPreparation
        );
        assert_eq!(
            classify_command_failure("gate", &["proposal".to_string()]),
            MachineFailureKind::InvocationPreparation
        );
    }

    #[test]
    fn literal_help_values_do_not_select_help_presentation() {
        let args = vec![
            "scaffold".to_string(),
            "change".to_string(),
            "demo".to_string(),
            "--request".to_string(),
            "help".to_string(),
        ];
        assert!(matches!(
            admit_invocation(&args),
            InvocationAdmission::Ready(_)
        ));
    }

    #[test]
    fn rejects_mcp_and_alternate_formats() {
        assert!(matches!(
            admit_invocation(&["mcp".to_string()]),
            InvocationAdmission::Failure {
                kind: MachineFailureKind::IncompatibleOption,
                ..
            }
        ));
        assert!(matches!(
            admit_invocation(&[
                "view".to_string(),
                "graph".to_string(),
                "--format".to_string(),
                "mermaid".to_string(),
            ]),
            InvocationAdmission::Failure {
                kind: MachineFailureKind::IncompatibleOption,
                ..
            }
        ));
    }

    #[test]
    fn non_object_json_is_not_an_admitted_command_result() {
        let primitive = serde_json::json!("not-an-object");
        assert!(!is_admitted_command_result(&primitive));
        let object = serde_json::json!({"valid": true});
        assert!(is_admitted_command_result(&object));
    }

    #[test]
    fn bounded_reader_reports_truncation() {
        let input = b"12345".as_slice();
        let captured = read_bounded(input, TEST_CAPTURE_BOUND).expect("bounded read succeeds");
        assert!(captured.exceeded);
        assert_eq!(captured.bytes, b"1234");
    }

    #[test]
    fn bounded_reader_accepts_exact_limit() {
        let input = b"1234".as_slice();
        let captured = read_bounded(input, TEST_CAPTURE_BOUND).expect("bounded read succeeds");
        assert!(!captured.exceeded);
        assert_eq!(captured.bytes, b"1234");
    }
}
