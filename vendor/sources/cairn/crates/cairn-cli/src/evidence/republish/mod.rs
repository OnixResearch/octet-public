// r[impl cli_maintainability.evidence_republish_submodule_split.focused_modules]
// r[impl cli_maintainability.evidence_republish_submodule_split.focused_modules.visible_boundaries]
// r[impl cli_maintainability.evidence_republish_submodule_split.focused_modules.narrow_bridges]
// r[impl cli_maintainability.evidence_republish_submodule_split.behavior_preserved]
// r[impl cli_maintainability.evidence_republish_submodule_split.behavior_preserved.focused_checks]
// r[impl cli_maintainability.evidence_republish_submodule_split.behavior_preserved.repository_checks]
mod import_receipt;
mod issue;
mod materialize;
mod options;
mod receipt;
mod scan;

use serde_json::Value;
use std::{
    error::Error,
    path::{Path, PathBuf},
};

use crate::support::{print_json, CapabilityRoot, CapabilityRootKind};

use import_receipt::{read_import_receipt, validate_import_receipt};
use issue::evidence_republish_issue;
use materialize::{copy_import_receipt_and_republish_receipt, materialize_declared_artifacts};
use options::evidence_republish_options;
use receipt::evidence_republish_final_receipt;
use scan::{detect_output_conflicts, reject_undeclared_imported_files};

pub(crate) use options::EvidenceRepublishOptions;

pub(super) fn run_evidence_republish(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.supersede.evidence_import_republish_rail.command]
    // r[impl cairn.supersede.evidence_import_republish_rail.artifacts]
    // r[impl cairn.supersede.evidence_import_republish_rail.failures]
    let options = evidence_republish_options(&args)?;
    let receipt = evidence_republish_receipt(&options);
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

pub(crate) fn evidence_republish_receipt(options: &EvidenceRepublishOptions) -> Value {
    let root = options.root.clone();
    let imported_dir = options
        .imported
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    let out_dir = options.out.clone().unwrap_or_else(|| PathBuf::from("."));
    let mut issues = Vec::new();
    let imported_root =
        match CapabilityRoot::open(CapabilityRootKind::EvidenceBundle, &imported_dir) {
            Ok(capability) => capability,
            Err(_) => {
                issues.push(evidence_republish_issue(
                    "evidence_republish.import_receipt_missing",
                    "evidence-import-receipt.json",
                    "readable imported evidence root",
                    "missing",
                ));
                return evidence_republish_final_receipt(&root, issues, Vec::new(), Value::Null);
            }
        };
    let import_receipt = match read_import_receipt(
        &imported_root,
        Path::new("evidence-import-receipt.json"),
        &mut issues,
    ) {
        Some(receipt) => receipt,
        None => return evidence_republish_final_receipt(&root, issues, Vec::new(), Value::Null),
    };
    let out_root =
        match CapabilityRoot::open_or_create(CapabilityRootKind::EvidenceBundle, &out_dir) {
            Ok(capability) => capability,
            Err(_) => {
                issues.push(evidence_republish_issue(
                    "evidence_republish.output_create_failed",
                    "out",
                    "writable output directory",
                    "failed",
                ));
                return evidence_republish_final_receipt(&root, issues, Vec::new(), Value::Null);
            }
        };

    let import_receipt_hash = validate_import_receipt(&root, &import_receipt, &mut issues);
    let Some(artifacts) = import_receipt.get("artifacts").and_then(Value::as_array) else {
        issues.push(evidence_republish_issue(
            "evidence_republish.artifacts_missing",
            "artifacts",
            "artifact array",
            "missing",
        ));
        return evidence_republish_final_receipt(&root, issues, Vec::new(), import_receipt_hash);
    };

    detect_output_conflicts(&out_root, &mut issues);
    reject_undeclared_imported_files(&imported_root, artifacts, &mut issues);
    let copied = materialize_declared_artifacts(&imported_root, &out_root, artifacts, &mut issues);

    let mut receipt = evidence_republish_final_receipt(&root, issues, copied, import_receipt_hash);
    if receipt.get("valid").and_then(Value::as_bool) == Some(true) {
        receipt =
            copy_import_receipt_and_republish_receipt(&root, &out_root, &import_receipt, receipt);
    }
    receipt
}
