use super::verify::{evidence_verify_receipt, EvidenceVerifyOptions};
use super::*;
use crate::support::{CapabilityRoot, CapabilityRootKind};

pub(crate) struct EvidenceImportOptions {
    pub(crate) root: PathBuf,
    pub(crate) bundle: Option<PathBuf>,
    pub(crate) into: Option<PathBuf>,
}

pub(super) fn run_evidence_import(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.supersede.evidence_import_consume_rail.command]
    // r[impl cairn.supersede.evidence_import_consume_rail.artifacts]
    // r[impl cairn.supersede.evidence_import_consume_rail.failures]
    let options = evidence_import_options(&args)?;
    let receipt = evidence_import_receipt(&options);
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

fn evidence_import_options(args: &[String]) -> Result<EvidenceImportOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let bundle = read_string_option(args, "--bundle").map(PathBuf::from);
    let into = read_string_option(args, "--into").map(PathBuf::from);
    if bundle.is_none() {
        return Err("evidence import requires --bundle <dir>".into());
    }
    if into.is_none() {
        return Err("evidence import requires --into <dir>".into());
    }
    Ok(EvidenceImportOptions { root, bundle, into })
}

pub(crate) fn evidence_import_receipt(options: &EvidenceImportOptions) -> Value {
    let root = options.root.clone();
    let bundle_dir = options.bundle.clone().unwrap_or_else(|| PathBuf::from("."));
    let into_dir = options.into.clone().unwrap_or_else(|| PathBuf::from("."));
    let mut issues = Vec::new();
    let verify_options = EvidenceVerifyOptions {
        root: root.clone(),
        policy: default_policy_path().into(),
        bundle: Some(bundle_dir.clone()),
        stack_provenance: None,
        stack_provenance_graph: None,
    };
    let verify_receipt = evidence_verify_receipt(&verify_options);
    let verification_receipt_hash = verify_receipt
        .get("receipt_hash")
        .cloned()
        .unwrap_or(Value::Null);
    if verify_receipt.get("valid").and_then(Value::as_bool) != Some(true) {
        issues.push(evidence_import_issue(
            "evidence_import.bundle_verification_failed",
            "bundle",
            "valid evidence verify receipt",
            "invalid",
        ));
        return evidence_import_final_receipt(&root, issues, Vec::new(), verification_receipt_hash);
    }

    let bundle_root = match CapabilityRoot::open(CapabilityRootKind::EvidenceBundle, &bundle_dir) {
        Ok(capability) => capability,
        Err(_) => {
            issues.push(evidence_import_issue(
                "evidence_import.bundle_read_failed",
                "bundle",
                "readable evidence bundle root",
                "unreadable",
            ));
            return evidence_import_final_receipt(
                &root,
                issues,
                Vec::new(),
                verification_receipt_hash,
            );
        }
    };
    let into_root =
        match CapabilityRoot::open_or_create(CapabilityRootKind::EvidenceBundle, &into_dir) {
            Ok(capability) => capability,
            Err(_) => {
                issues.push(evidence_import_issue(
                    "evidence_import.destination_create_failed",
                    "into",
                    "writable import directory",
                    "failed",
                ));
                return evidence_import_final_receipt(
                    &root,
                    issues,
                    Vec::new(),
                    verification_receipt_hash,
                );
            }
        };

    const EXPORT_RECEIPT_PATH: &str = "evidence-export-receipt.json";
    let export_receipt = match read_json_artifact_from_root(
        &bundle_root,
        Path::new(EXPORT_RECEIPT_PATH),
        AGENT_RECEIPT_MAX_BYTES,
    ) {
        Ok(receipt) => receipt,
        Err(_) => {
            issues.push(evidence_import_issue(
                "evidence_import.export_receipt_unreadable",
                EXPORT_RECEIPT_PATH,
                "readable export receipt",
                "unreadable",
            ));
            return evidence_import_final_receipt(
                &root,
                issues,
                Vec::new(),
                verification_receipt_hash,
            );
        }
    };

    let Some(artifacts) = export_receipt.get("artifacts").and_then(Value::as_array) else {
        issues.push(evidence_import_issue(
            "evidence_import.artifacts_missing",
            "artifacts",
            "artifact array",
            "missing",
        ));
        return evidence_import_final_receipt(&root, issues, Vec::new(), verification_receipt_hash);
    };

    let mut declared_files = std::collections::BTreeSet::new();
    declared_files.insert("evidence-export-receipt.json".to_string());
    let mut copied = Vec::new();
    for artifact in artifacts {
        let role = artifact
            .get("role")
            .and_then(Value::as_str)
            .unwrap_or("<missing>");
        let relative = artifact.get("path").and_then(Value::as_str).unwrap_or("");
        if safe_bundle_artifact_path(&bundle_dir, relative).is_none() {
            issues.push(evidence_import_issue(
                "evidence_import.artifact_path_unsafe",
                role,
                "safe relative artifact path",
                "unsafe",
            ));
            continue;
        }
        if safe_bundle_artifact_path(&into_dir, relative).is_none() {
            issues.push(evidence_import_issue(
                "evidence_import.destination_path_unsafe",
                role,
                "safe relative destination path",
                "unsafe",
            ));
            continue;
        }
        let relative_path = Path::new(relative);
        declared_files.insert(relative.to_string());
        let declared_hash = artifact.get("hash").and_then(Value::as_str).unwrap_or("");
        let source_bytes = match bundle_root.read(relative_path) {
            Ok(bytes) => bytes,
            Err(_) => {
                issues.push(evidence_import_issue(
                    "evidence_import.artifact_missing",
                    role,
                    "declared artifact file",
                    "missing",
                ));
                continue;
            }
        };
        let source_hash = blake3_hex(&source_bytes);
        if source_hash != declared_hash {
            issues.push(evidence_import_issue(
                "evidence_import.artifact_hash_mismatch",
                role,
                "declared artifact hash",
                "mismatch",
            ));
            continue;
        }
        if let Some(parent) = relative_path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            if into_root.create_dir_all(parent).is_err() {
                issues.push(evidence_import_issue(
                    "evidence_import.destination_create_failed",
                    role,
                    "writable destination parent",
                    "failed",
                ));
                continue;
            }
        }
        if into_root.write(relative_path, &source_bytes).is_err() {
            issues.push(evidence_import_issue(
                "evidence_import.artifact_copy_failed",
                role,
                "copied artifact file",
                "failed",
            ));
            continue;
        }
        let destination_hash = match into_root.read(relative_path) {
            Ok(bytes) => blake3_hex(&bytes),
            Err(_) => String::new(),
        };
        if destination_hash != source_hash {
            issues.push(evidence_import_issue(
                "evidence_import.destination_hash_mismatch",
                role,
                "copied artifact hash",
                "mismatch",
            ));
            continue;
        }
        copied.push(json!({
            "role": role,
            "path": relative,
            "hash": destination_hash,
        }));
    }

    match bundle_regular_files(&bundle_root) {
        Ok(files) => {
            for relative in files {
                if !declared_files.contains(&relative) {
                    issues.push(evidence_import_issue(
                        "evidence_import.undeclared_bundle_file",
                        &relative,
                        "declared export artifact",
                        "extra",
                    ));
                }
            }
        }
        Err(code) => issues.push(evidence_import_issue(
            code,
            "bundle",
            "bounded readable bundle files",
            "failed",
        )),
    }

    let mut receipt =
        evidence_import_final_receipt(&root, issues, copied, verification_receipt_hash);
    if receipt.get("valid").and_then(Value::as_bool) == Some(true)
        && into_root
            .write(
                Path::new("evidence-import-receipt.json"),
                &serde_json::to_vec_pretty(&receipt).unwrap_or_default(),
            )
            .is_err()
    {
        receipt = evidence_import_final_receipt(
            &root,
            vec![evidence_import_issue(
                "evidence_import.receipt_write_failed",
                "evidence-import-receipt.json",
                "writable import receipt",
                "failed",
            )],
            Vec::new(),
            receipt
                .get("verification_receipt_hash")
                .cloned()
                .unwrap_or(Value::Null),
        );
    }
    receipt
}

pub(super) const EVIDENCE_IMPORT_MAX_FILES: usize = 64;

fn read_json_artifact_from_root(
    root: &CapabilityRoot,
    relative: &Path,
    max_bytes: u64,
) -> Result<Value, Box<dyn Error>> {
    let bytes = root.read(relative)?;
    let byte_count = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if byte_count > max_bytes {
        return Err("JSON artifact exceeds the byte limit".into());
    }
    Ok(serde_json::from_slice(&bytes)?)
}

fn bundle_regular_files(
    bundle_root: &CapabilityRoot,
) -> Result<std::collections::BTreeSet<String>, &'static str> {
    let mut out = std::collections::BTreeSet::new();
    collect_bundle_regular_files(bundle_root, Path::new("."), &mut out)?;
    Ok(out)
}

fn collect_bundle_regular_files(
    bundle_root: &CapabilityRoot,
    relative_dir: &Path,
    out: &mut std::collections::BTreeSet<String>,
) -> Result<(), &'static str> {
    let entries = bundle_root
        .entries(relative_dir)
        .map_err(|_| "evidence_import.bundle_read_failed")?;
    let mut entries = entries;
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    for entry in entries {
        let relative = if relative_dir == Path::new(".") {
            PathBuf::from(entry.file_name())
        } else {
            relative_dir.join(entry.file_name())
        };
        if entry.file_type().is_symlink() {
            return Err("evidence_import.bundle_read_failed");
        }
        if entry.file_type().is_dir() {
            collect_bundle_regular_files(bundle_root, &relative, out)?;
        } else if entry.file_type().is_file() {
            if out.len() >= EVIDENCE_IMPORT_MAX_FILES {
                return Err("evidence_import.bundle_file_limit_exceeded");
            }
            out.insert(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}

fn evidence_import_issue(code: &str, field_path: &str, expected: &str, actual: &str) -> Value {
    json!({
        "code": code,
        "field_path": field_path,
        "expected": expected,
        "actual": actual,
    })
}

fn evidence_import_final_receipt(
    root: &Path,
    mut issues: Vec<Value>,
    mut artifacts: Vec<Value>,
    verification_receipt_hash: Value,
) -> Value {
    issues.sort_by_key(canonical_json_hash);
    artifacts.sort_by_key(|artifact| {
        format!(
            "{}:{}",
            artifact.get("role").and_then(Value::as_str).unwrap_or(""),
            artifact.get("path").and_then(Value::as_str).unwrap_or("")
        )
    });
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        root,
        json!({
            "command": "evidence import",
            "root": path_string(root),
            "bundle": "<bundle>",
            "into": "<into>",
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "artifacts": artifacts,
            "verification_receipt_hash": verification_receipt_hash,
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    const TEST_FILE_CONTENT: &[u8] = b"evidence\n";
    const TEST_DIRECTORY_PREFIX: &str = "cairn-evidence-import-capability";

    fn test_directory(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "{TEST_DIRECTORY_PREFIX}-{name}-{}",
            std::process::id()
        ))
    }

    #[test]
    fn bundle_regular_files_collects_relative_files() -> Result<(), Box<dyn Error>> {
        let directory = test_directory("positive");
        let nested = directory.join("nested");
        fs::create_dir_all(&nested)?;
        fs::write(directory.join("root.json"), TEST_FILE_CONTENT)?;
        fs::write(nested.join("child.json"), TEST_FILE_CONTENT)?;
        let capability = CapabilityRoot::open(CapabilityRootKind::EvidenceBundle, &directory)?;

        let files = bundle_regular_files(&capability)?;

        assert_eq!(
            files,
            std::collections::BTreeSet::from([
                "nested/child.json".to_string(),
                "root.json".to_string(),
            ])
        );
        fs::remove_dir_all(directory)?;
        Ok(())
    }

    #[test]
    fn bundle_regular_files_rejects_symlink_escape() -> Result<(), Box<dyn Error>> {
        let directory = test_directory("symlink-negative");
        let outside = test_directory("outside");
        fs::create_dir_all(&directory)?;
        fs::create_dir_all(&outside)?;
        let outside_file = outside.join("outside.json");
        fs::write(&outside_file, TEST_FILE_CONTENT)?;
        std::os::unix::fs::symlink(&outside_file, directory.join("escape.json"))?;
        let capability = CapabilityRoot::open(CapabilityRootKind::EvidenceBundle, &directory)?;

        let result = bundle_regular_files(&capability);

        assert_eq!(result, Err("evidence_import.bundle_read_failed"));
        fs::remove_dir_all(directory)?;
        fs::remove_dir_all(outside)?;
        Ok(())
    }
}
