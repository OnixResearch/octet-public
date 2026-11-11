use std::{
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{verify_runtime_evidence, RuntimeEvidencePolicy};
use serde_json::{json, Value};

use crate::agent_receipt::{read_json_artifact, AGENT_RECEIPT_MAX_BYTES};
use crate::support::{
    default_policy_path, load_policy_file, path_string, print_json, read_option,
    read_string_option, with_portable_content_hash_receipt,
};

pub(crate) struct RuntimeEvidenceVerifyOptions {
    pub(crate) root: PathBuf,
    pub(crate) policy: PathBuf,
    pub(crate) chain: Option<PathBuf>,
    pub(crate) checkpoint: Option<PathBuf>,
}

pub(crate) fn run_runtime_evidence(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("runtime-evidence requires a subcommand: verify".into());
    };
    match subcommand.as_str() {
        "verify" => run_runtime_evidence_verify(iter.collect()),
        unknown => Err(format!("unknown runtime-evidence subcommand: {unknown}").into()),
    }
}

fn run_runtime_evidence_verify(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.runtime_evidence.cli]
    let options = runtime_evidence_verify_options(&args)?;
    let receipt = runtime_evidence_verification_receipt(&options);
    let valid = receipt
        .get("valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    print_json(receipt)?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

pub(crate) fn runtime_evidence_verify_options(
    args: &[String],
) -> Result<RuntimeEvidenceVerifyOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let policy = read_option(args.to_vec(), "--policy", default_policy_path());
    let chain = read_string_option(args, "--chain").map(PathBuf::from);
    let checkpoint = read_string_option(args, "--checkpoint").map(PathBuf::from);
    if chain.is_none() {
        return Err("runtime-evidence verify requires --chain <chain.json>".into());
    }
    Ok(RuntimeEvidenceVerifyOptions {
        root,
        policy,
        chain,
        checkpoint,
    })
}

pub(crate) fn runtime_evidence_verification_receipt(
    options: &RuntimeEvidenceVerifyOptions,
) -> Value {
    let root = options.root.clone();
    let (policy, mut issues) = runtime_evidence_policy(&options.policy);
    let (records, chain_issue) = read_runtime_chain(options.chain.as_ref());
    append_optional_issue(&mut issues, chain_issue);
    let (checkpoints, checkpoint_issue) = read_runtime_checkpoints(options.checkpoint.as_ref());
    append_optional_issue(&mut issues, checkpoint_issue);
    let report = verify_runtime_evidence(&policy, &records, &checkpoints);
    issues.extend(report.issues.iter().map(|issue| issue.to_json()));
    issues.sort_by_key(cairn_core::canonical_json_hash);
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        &root,
        json!({
            "command": "runtime-evidence verify",
            "root": path_string(root.clone()),
            "chain": "<chain>",
            "checkpoint": if options.checkpoint.is_some() { json!("<checkpoint>") } else { Value::Null },
            "schema_version": report.to_json()["schema_version"].clone(),
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "subject": report.subject,
            "record_count": report.record_count,
            "chain_root_hash": report.chain_root_hash,
            "head_hash": report.head_hash,
            "checkpoint_hash": report.checkpoint_hash,
            "completeness": report.to_json()["completeness"].clone(),
            "capture_origin_summary": report.capture_origin_summary.to_json(),
            "boundary": report.boundary,
            "issues": issues,
        }),
    )
}

pub(crate) fn runtime_evidence_policy(path: &Path) -> (RuntimeEvidencePolicy, Vec<Value>) {
    match load_policy_file(path.to_path_buf()) {
        Ok(policy) => (policy.runtime_evidence_policy, Vec::new()),
        Err(_) => (
            RuntimeEvidencePolicy {
                allowed_capture_origins: Vec::new(),
                allowed_hash_algorithms: Vec::new(),
                max_summary_bytes: cairn_core::DEFAULT_RUNTIME_SUMMARY_MAX_BYTES,
                forbidden_summary_fragments: Vec::new(),
                aggregation_profiles: Vec::new(),
            },
            vec![runtime_cli_issue(
                "runtime_evidence.policy_unreadable",
                "policy",
                "parseable policy with runtime_evidence_policy",
                "unreadable",
            )],
        ),
    }
}

fn read_runtime_chain(path: Option<&PathBuf>) -> (Vec<Value>, Option<Value>) {
    let Some(path) = path else {
        return (
            Vec::new(),
            Some(runtime_cli_issue(
                "runtime_evidence.chain_missing",
                "chain",
                "runtime chain artifact",
                "missing",
            )),
        );
    };
    match read_json_artifact(path, AGENT_RECEIPT_MAX_BYTES) {
        Ok(Value::Array(records)) => (records, None),
        Ok(value) => match value.get("records").and_then(Value::as_array) {
            Some(records) => (records.clone(), None),
            None => (
                Vec::new(),
                Some(runtime_cli_issue(
                    "runtime_evidence.chain_not_array",
                    "chain",
                    "array or object.records runtime chain",
                    "invalid",
                )),
            ),
        },
        Err(_) => (
            Vec::new(),
            Some(runtime_cli_issue(
                "runtime_evidence.chain_unreadable",
                "chain",
                "readable JSON runtime chain",
                "unreadable",
            )),
        ),
    }
}

fn read_runtime_checkpoints(path: Option<&PathBuf>) -> (Vec<Value>, Option<Value>) {
    let Some(path) = path else {
        return (Vec::new(), None);
    };
    match read_json_artifact(path, AGENT_RECEIPT_MAX_BYTES) {
        Ok(Value::Array(checkpoints)) => (checkpoints, None),
        Ok(value) if value.is_object() => (vec![value], None),
        Ok(_) => (
            Vec::new(),
            Some(runtime_cli_issue(
                "runtime_evidence.checkpoint_not_object",
                "checkpoint",
                "object or array checkpoint artifact",
                "invalid",
            )),
        ),
        Err(_) => (
            Vec::new(),
            Some(runtime_cli_issue(
                "runtime_evidence.checkpoint_unreadable",
                "checkpoint",
                "readable JSON checkpoint artifact",
                "unreadable",
            )),
        ),
    }
}

fn append_optional_issue(issues: &mut Vec<Value>, issue: Option<Value>) {
    if let Some(issue) = issue {
        issues.push(issue);
    }
}

fn runtime_cli_issue(code: &str, field_path: &str, expected: &str, actual: &str) -> Value {
    json!({
        "code": code,
        "field_path": field_path,
        "expected": expected,
        "actual": actual,
    })
}
