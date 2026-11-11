use serde_json::Value;
use std::{collections::BTreeSet, path::Path};

use crate::support::CapabilityRoot;

use super::issue::evidence_republish_issue;

pub(super) fn detect_output_conflicts(out_root: &CapabilityRoot, issues: &mut Vec<Value>) {
    match bundle_regular_files_with_prefix(out_root, "evidence_republish") {
        Ok(files) if !files.is_empty() => issues.push(evidence_republish_issue(
            "evidence_republish.output_not_empty",
            "out",
            "empty output directory",
            "non_empty",
        )),
        Err(code) => issues.push(evidence_republish_issue(
            code,
            "out",
            "bounded readable output files",
            "failed",
        )),
        _ => {}
    }
}

pub(super) fn reject_undeclared_imported_files(
    imported_root: &CapabilityRoot,
    artifacts: &[Value],
    issues: &mut Vec<Value>,
) {
    let mut declared_files = BTreeSet::new();
    declared_files.insert("evidence-import-receipt.json".to_string());
    match bundle_regular_files_with_prefix(imported_root, "evidence_republish") {
        Ok(files) => {
            for relative in files {
                if !declared_files.contains(&relative)
                    && !artifacts.iter().any(|artifact| {
                        artifact.get("path").and_then(Value::as_str) == Some(relative.as_str())
                    })
                {
                    issues.push(evidence_republish_issue(
                        "evidence_republish.undeclared_imported_file",
                        &relative,
                        "declared imported artifact",
                        "extra",
                    ));
                }
            }
        }
        Err(code) => issues.push(evidence_republish_issue(
            code,
            "imported",
            "bounded readable imported files",
            "failed",
        )),
    }
}

pub(super) fn bundle_regular_files_with_prefix(
    bundle_root: &CapabilityRoot,
    prefix: &'static str,
) -> Result<BTreeSet<String>, &'static str> {
    let mut out = BTreeSet::new();
    collect_bundle_regular_files_with_prefix(bundle_root, Path::new("."), &mut out, prefix)?;
    Ok(out)
}

fn collect_bundle_regular_files_with_prefix(
    bundle_root: &CapabilityRoot,
    relative_dir: &Path,
    out: &mut BTreeSet<String>,
    prefix: &'static str,
) -> Result<(), &'static str> {
    let read_failure = || match prefix {
        "evidence_republish" => "evidence_republish.bundle_read_failed",
        _ => "evidence_import.bundle_read_failed",
    };
    let mut entries = bundle_root
        .entries(relative_dir)
        .map_err(|_| read_failure())?;
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    for entry in entries {
        let relative = if relative_dir == Path::new(".") {
            entry.file_name().into()
        } else {
            relative_dir.join(entry.file_name())
        };
        if entry.file_type().is_symlink() {
            return Err(read_failure());
        }
        if entry.file_type().is_dir() {
            collect_bundle_regular_files_with_prefix(bundle_root, &relative, out, prefix)?;
        } else if entry.file_type().is_file() {
            if out.len() >= super::super::import::EVIDENCE_IMPORT_MAX_FILES {
                return Err(match prefix {
                    "evidence_republish" => "evidence_republish.bundle_file_limit_exceeded",
                    _ => "evidence_import.bundle_file_limit_exceeded",
                });
            }
            out.insert(relative.to_string_lossy().replace('\\', "/"));
        }
    }
    Ok(())
}
