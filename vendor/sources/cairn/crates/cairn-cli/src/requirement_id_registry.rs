use std::{
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    canonical_json_hash, content_bound_requirement_registry_issues_json,
    evaluate_content_bound_requirement_registry, parse_content_bound_requirement_registry_value,
    requirement_id_registry_issues_json, validate_requirement_id_registry_value,
    verify_content_bound_requirement_registry, ContentBoundRequirementRegistry,
    ContentBoundRequirementRegistryIssue, ContentBoundRevision, ContentBoundSpecificationInput,
    CONTENT_BOUND_REQUIREMENT_REGISTRY_BOUNDARY,
    CONTENT_BOUND_REQUIREMENT_REGISTRY_RECEIPT_SCHEMA_V1,
    CONTENT_BOUND_REQUIREMENT_REGISTRY_SCHEMA_V1,
};
use serde_json::{json, Value};

use crate::support::{
    path_string, print_json, read_option, read_string_option, with_portable_receipt_hash,
    with_receipt_hash, CapabilityRoot, CapabilityRootKind,
};

const ACCEPTED_SPECIFICATIONS_DIRECTORY: &str = ".cairn/specs";
const GENERATED_POLICY_RELATIVE_PATH: &str = "cairn-policy/generated/cairn-policy.json";
const SPECIFICATION_FILE_NAME: &str = "spec.md";
const CONTENT_BOUND_EXPORT_RECEIPT_SCHEMA_V1: &str = "cairn.requirement-registry.export-receipt.v1";
const CONTENT_BOUND_VERIFY_RECEIPT_SCHEMA_V1: &str =
    "cairn.requirement-registry.current-root-verify-receipt.v1";
const LEGACY_VERIFY_RECEIPT_SCHEMA_V1: &str = "cairn.requirement-registry.verify-receipt.v1";
const BYTES_PER_KIBIBYTE: u64 = 1_024;
const KIBIBYTES_PER_MEBIBYTE: u64 = 1_024;
const MAX_SPECIFICATION_MEBIBYTES: u64 = 4;
const MAX_POLICY_MEBIBYTES: u64 = 16;
const MAX_REGISTRY_MEBIBYTES: u64 = 64;
const MAX_SPECIFICATION_BYTES: u64 =
    MAX_SPECIFICATION_MEBIBYTES * KIBIBYTES_PER_MEBIBYTE * BYTES_PER_KIBIBYTE;
const MAX_POLICY_BYTES: u64 = MAX_POLICY_MEBIBYTES * KIBIBYTES_PER_MEBIBYTE * BYTES_PER_KIBIBYTE;
const MAX_REGISTRY_BYTES: u64 =
    MAX_REGISTRY_MEBIBYTES * KIBIBYTES_PER_MEBIBYTE * BYTES_PER_KIBIBYTE;
const MAX_DIRECTORY_ENTRIES: usize = 4_096;
const MAX_SPECIFICATION_COUNT: usize = 4_096;

struct ContentBoundExportOptions {
    root: PathBuf,
    repository_id: String,
    revision: ContentBoundRevision,
    policy: PathBuf,
    out: PathBuf,
}

struct ContentBoundVerifyOptions {
    root: PathBuf,
    repository_id: String,
    policy: PathBuf,
    registry: PathBuf,
}

pub(crate) fn run_requirement_registry(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(subcommand) = args.first().map(String::as_str) else {
        return Err("requirement-registry requires a subcommand: export or verify".into());
    };
    match subcommand {
        "export" => run_content_bound_export(args.into_iter().skip(1).collect()),
        "verify" => run_requirement_registry_verify(args.into_iter().skip(1).collect()),
        other => Err(format!("unknown requirement-registry subcommand: {other}").into()),
    }
}

fn run_content_bound_export(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.requirement_registry.content_bound_export]
    // r[impl cairn.requirement_registry.current_root_verification]
    let options = content_bound_export_options(&args)?;
    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, &options.root)?;
    let specifications = read_accepted_specifications(&repository)?;
    let policy_bytes = repository.read_bounded(&options.policy, MAX_POLICY_BYTES)?;
    let evaluation = evaluate_content_bound_requirement_registry(
        specifications,
        &policy_bytes,
        &options.repository_id,
        options.revision.clone(),
    );
    let Some(registry) = evaluation.registry else {
        let receipt =
            content_bound_export_receipt(&options, None, evaluation.issues, false, Value::Null);
        print_json(receipt)?;
        std::process::exit(1);
    };
    let bytes = registry_bytes(&registry)?;
    let (output_root, output_relative) =
        CapabilityRoot::open_file_parent(CapabilityRootKind::EvidenceBundle, &options.out, true)?;
    output_root.write_new(&output_relative, &bytes)?;
    print_json(content_bound_export_receipt(
        &options,
        Some(&registry),
        Vec::new(),
        true,
        Value::String(path_string(&options.out)),
    ))
}

fn run_requirement_registry_verify(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    if args.iter().any(|argument| argument == "--input") {
        return run_legacy_requirement_registry_verify(args);
    }
    run_content_bound_verify(args)
}

fn run_content_bound_verify(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.requirement_registry.current_root_verification]
    let options = content_bound_verify_options(&args)?;
    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, &options.root)?;
    let specifications = read_accepted_specifications(&repository)?;
    let policy_bytes = repository.read_bounded(&options.policy, MAX_POLICY_BYTES)?;
    let value = read_bounded_json_artifact(&options.registry, MAX_REGISTRY_BYTES)?;
    let registry = match parse_content_bound_requirement_registry_value(&value) {
        Ok(registry) => registry,
        Err(issue) => {
            print_json(content_bound_verify_receipt(&options, None, vec![issue]))?;
            std::process::exit(1);
        }
    };
    let report = verify_content_bound_requirement_registry(
        &registry,
        specifications,
        &policy_bytes,
        &options.repository_id,
    );
    let valid = report.valid;
    print_json(with_portable_receipt_hash(
        &options.root,
        json!({
            "command": "requirement-registry verify",
            "schema": CONTENT_BOUND_VERIFY_RECEIPT_SCHEMA_V1,
            "root": path_string(&options.root),
            "registry": path_string(&options.registry),
            "repository_id": report.repository_id,
            "revision_kind": report.revision_kind,
            "revision": report.revision,
            "registry_blake3": report.registry_blake3,
            "registry_receipt_blake3": report.receipt_blake3,
            "specification_count": report.specification_count,
            "row_count": report.row_count,
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "boundary": report.boundary,
            "issues": content_bound_requirement_registry_issues_json(&report.issues),
        }),
    ))?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

fn run_legacy_requirement_registry_verify(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.requirement_id_registry.validation]
    // r[impl cairn.requirement_id_registry.validation.receipt]
    let input = read_option(args, "--input", "");
    if input.as_os_str().is_empty() {
        return Err("requirement-registry verify requires --input <registry-input.json>".into());
    }
    let value = read_unbounded_legacy_registry_input(&input)?;
    let report = validate_requirement_id_registry_value(&value);
    let receipt = json!({
        "command": "requirement-registry verify",
        "schema": LEGACY_VERIFY_RECEIPT_SCHEMA_V1,
        "input": input.to_string_lossy(),
        "input_hash": canonical_json_hash(&value),
        "registry_hash": report.registry_hash,
        "policy_hash": legacy_policy_hash(&value),
        "valid": report.valid,
        "verdict": if report.valid { "pass" } else { "fail" },
        "row_count": report.row_count,
        "checked_reference_count": report.checked_reference_count,
        "dangling_reference_count": report.dangling_reference_count,
        "stale_alias_count": report.stale_alias_count,
        "boundary": report.boundary,
        "issues": requirement_id_registry_issues_json(&report),
    });
    print_json(with_receipt_hash(receipt))?;
    if !report.valid {
        std::process::exit(1);
    }
    Ok(())
}

fn content_bound_export_options(
    args: &[String],
) -> Result<ContentBoundExportOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let repository_id = required_string_option(args, "--repository-id", "export")?;
    let revision_kind = required_string_option(args, "--revision-kind", "export")?;
    let revision = required_string_option(args, "--revision", "export")?;
    let out = PathBuf::from(required_string_option(args, "--out", "export")?);
    let policy = PathBuf::from(
        read_string_option(args, "--policy")
            .unwrap_or_else(|| GENERATED_POLICY_RELATIVE_PATH.to_string()),
    );
    Ok(ContentBoundExportOptions {
        root,
        repository_id,
        revision: ContentBoundRevision {
            kind: revision_kind,
            value: revision,
        },
        policy,
        out,
    })
}

fn content_bound_verify_options(
    args: &[String],
) -> Result<ContentBoundVerifyOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let repository_id = required_string_option(args, "--repository-id", "verify")?;
    let registry = PathBuf::from(required_string_option(args, "--registry", "verify")?);
    let policy = PathBuf::from(
        read_string_option(args, "--policy")
            .unwrap_or_else(|| GENERATED_POLICY_RELATIVE_PATH.to_string()),
    );
    Ok(ContentBoundVerifyOptions {
        root,
        repository_id,
        policy,
        registry,
    })
}

fn required_string_option(
    args: &[String],
    option: &str,
    command: &str,
) -> Result<String, Box<dyn Error>> {
    read_string_option(args, option)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| format!("requirement-registry {command} requires {option} <VALUE>").into())
}

fn read_accepted_specifications(
    repository: &CapabilityRoot,
) -> Result<Vec<ContentBoundSpecificationInput>, Box<dyn Error>> {
    let accepted_root = PathBuf::from(ACCEPTED_SPECIFICATIONS_DIRECTORY);
    let mut pending = vec![accepted_root];
    let mut specifications = Vec::new();
    let mut observed_entries = 0usize;
    while let Some(directory) = pending.pop() {
        let entries = repository.entries_bounded(&directory, MAX_DIRECTORY_ENTRIES)?;
        observed_entries = observed_entries.saturating_add(entries.len());
        if observed_entries > MAX_DIRECTORY_ENTRIES {
            return Err("accepted specification tree exceeds the bounded entry limit".into());
        }
        for entry in entries {
            let relative = directory.join(entry.file_name());
            if entry.file_type().is_symlink() {
                return Err(format!(
                    "accepted specification tree contains a symlink: {}",
                    path_string(&relative)
                )
                .into());
            }
            if entry.file_type().is_dir() {
                pending.push(relative);
                continue;
            }
            if !entry.file_type().is_file()
                || relative
                    .file_name()
                    .is_none_or(|name| name != SPECIFICATION_FILE_NAME)
            {
                continue;
            }
            if specifications.len() >= MAX_SPECIFICATION_COUNT {
                return Err("accepted specification count exceeds the bounded limit".into());
            }
            let bytes = repository.read_bounded(&relative, MAX_SPECIFICATION_BYTES)?;
            let markdown = String::from_utf8(bytes).map_err(|_error| {
                format!(
                    "accepted specification is not UTF-8: {}",
                    path_string(&relative)
                )
            })?;
            specifications.push(ContentBoundSpecificationInput {
                path: portable_relative_path(&relative),
                markdown,
            });
        }
    }
    specifications.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(specifications)
}

fn portable_relative_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn registry_bytes(registry: &ContentBoundRequirementRegistry) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut bytes = serde_json::to_vec_pretty(registry)?;
    bytes.push(b'\n');
    Ok(bytes)
}

fn read_bounded_json_artifact(path: &Path, max_bytes: u64) -> Result<Value, Box<dyn Error>> {
    let (root, relative) =
        CapabilityRoot::open_file_parent(CapabilityRootKind::EvidenceBundle, path, false)?;
    let bytes = root.read_bounded(&relative, max_bytes)?;
    serde_json::from_slice(&bytes).map_err(|error| {
        format!("failed to parse JSON artifact {}: {error}", path.display()).into()
    })
}

fn content_bound_export_receipt(
    options: &ContentBoundExportOptions,
    registry: Option<&ContentBoundRequirementRegistry>,
    issues: Vec<ContentBoundRequirementRegistryIssue>,
    valid: bool,
    output: Value,
) -> Value {
    let registry_blake3 = registry
        .map(|registry| Value::String(registry.registry_blake3.clone()))
        .unwrap_or(Value::Null);
    let registry_receipt_blake3 = registry
        .map(|registry| Value::String(registry.receipt.receipt_blake3.clone()))
        .unwrap_or(Value::Null);
    let policy_blake3 = registry
        .map(|registry| Value::String(registry.policy_blake3.clone()))
        .unwrap_or(Value::Null);
    let specification_count = registry.map_or(0, |registry| registry.specifications.len());
    let row_count = registry.map_or(0, |registry| registry.rows.len());
    with_portable_receipt_hash(
        &options.root,
        json!({
            "command": "requirement-registry export",
            "schema": CONTENT_BOUND_EXPORT_RECEIPT_SCHEMA_V1,
            "registry_schema": CONTENT_BOUND_REQUIREMENT_REGISTRY_SCHEMA_V1,
            "registry_receipt_schema": CONTENT_BOUND_REQUIREMENT_REGISTRY_RECEIPT_SCHEMA_V1,
            "root": path_string(&options.root),
            "repository_id": options.repository_id,
            "revision_kind": options.revision.kind,
            "revision": options.revision.value,
            "output": output,
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "specification_count": specification_count,
            "row_count": row_count,
            "policy_blake3": policy_blake3,
            "registry_blake3": registry_blake3,
            "registry_receipt_blake3": registry_receipt_blake3,
            "boundary": CONTENT_BOUND_REQUIREMENT_REGISTRY_BOUNDARY,
            "issues": content_bound_requirement_registry_issues_json(&issues),
        }),
    )
}

fn content_bound_verify_receipt(
    options: &ContentBoundVerifyOptions,
    registry: Option<&ContentBoundRequirementRegistry>,
    issues: Vec<ContentBoundRequirementRegistryIssue>,
) -> Value {
    let repository_id = registry
        .map(|registry| registry.repository_id.clone())
        .unwrap_or_else(|| options.repository_id.clone());
    let revision_kind = registry
        .map(|registry| Value::String(registry.revision.kind.clone()))
        .unwrap_or(Value::Null);
    let revision = registry
        .map(|registry| Value::String(registry.revision.value.clone()))
        .unwrap_or(Value::Null);
    let registry_blake3 = registry
        .map(|registry| Value::String(registry.registry_blake3.clone()))
        .unwrap_or(Value::Null);
    let registry_receipt_blake3 = registry
        .map(|registry| Value::String(registry.receipt.receipt_blake3.clone()))
        .unwrap_or(Value::Null);
    with_portable_receipt_hash(
        &options.root,
        json!({
            "command": "requirement-registry verify",
            "schema": CONTENT_BOUND_VERIFY_RECEIPT_SCHEMA_V1,
            "root": path_string(&options.root),
            "registry": path_string(&options.registry),
            "repository_id": repository_id,
            "revision_kind": revision_kind,
            "revision": revision,
            "registry_blake3": registry_blake3,
            "registry_receipt_blake3": registry_receipt_blake3,
            "specification_count": registry.map_or(0, |registry| registry.specifications.len()),
            "row_count": registry.map_or(0, |registry| registry.rows.len()),
            "valid": false,
            "verdict": "fail",
            "boundary": CONTENT_BOUND_REQUIREMENT_REGISTRY_BOUNDARY,
            "issues": content_bound_requirement_registry_issues_json(&issues),
        }),
    )
}

fn legacy_policy_hash(value: &Value) -> String {
    value
        .get("policy_hash")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .unwrap_or_else(|| canonical_json_hash(&Value::Null))
}

fn read_unbounded_legacy_registry_input(path: &Path) -> Result<Value, Box<dyn Error>> {
    let text = std::fs::read_to_string(path).map_err(|source| {
        format!(
            "failed to read requirement registry input {}: {source}",
            path.display()
        )
    })?;
    serde_json::from_str(&text).map_err(|source| {
        format!(
            "failed to parse requirement registry input {}: {source}",
            path.display()
        )
        .into()
    })
}
