// r[impl cli_maintainability.evidence_index_submodule_split.focused_modules]
// r[impl cli_maintainability.evidence_index_submodule_split.focused_modules.visible_boundaries]
// r[impl cli_maintainability.evidence_index_submodule_split.focused_modules.narrow_bridges]
// r[impl cli_maintainability.evidence_index_submodule_split.behavior_preserved]
// r[impl cli_maintainability.evidence_index_submodule_split.behavior_preserved.focused_checks]
// r[impl cli_maintainability.evidence_index_submodule_split.behavior_preserved.repository_checks]
mod bundle;
mod issue;
mod options;
mod receipt;
mod verify;

use serde_json::{json, Value};
use std::{error::Error, path::PathBuf};

use crate::support::{print_json, CapabilityRoot, CapabilityRootKind};

use bundle::{evidence_index_bundle_relative, evidence_index_bundle_summary};
use issue::evidence_index_issue;
use options::{evidence_index_options, evidence_index_verify_options};
use receipt::evidence_index_final_receipt;

pub(crate) use options::{EvidenceIndexOptions, EvidenceIndexVerifyOptions};
pub(crate) use verify::evidence_index_verify_receipt;

pub(super) fn run_evidence_index(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    if let Some(first) = iter.next() {
        if first == "verify" {
            return run_evidence_index_verify(iter.collect());
        }
        let mut rest = vec![first];
        rest.extend(iter);
        let options = evidence_index_options(&rest)?;
        let receipt = evidence_index_receipt(&options);
        let valid = receipt
            .get("valid")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        print_json(receipt)?;
        if !valid {
            std::process::exit(1);
        }
        return Ok(());
    }
    Err("evidence index requires --bundle <dir> --out <file> or verify".into())
}

pub(super) fn run_evidence_index_verify(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = evidence_index_verify_options(&args)?;
    let receipt = evidence_index_verify_receipt(&options);
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

pub(crate) fn evidence_index_receipt(options: &EvidenceIndexOptions) -> Value {
    // r[impl cairn.supersede.evidence_publish_index_rail.command]
    // r[impl cairn.supersede.evidence_publish_index_rail.failures]
    let root = options.root.clone();
    let bundle = options.bundle.clone().unwrap_or_else(|| PathBuf::from("."));
    let out = options.out.clone().unwrap_or_else(|| PathBuf::from("."));
    let mut issues = Vec::new();
    let bundle_root = match CapabilityRoot::open(CapabilityRootKind::EvidenceBundle, &bundle) {
        Ok(capability) => Some(capability),
        Err(_) => {
            issues.push(evidence_index_issue(
                "evidence_index.bundle_unreadable",
                "bundle",
                "readable evidence bundle root",
                "unreadable",
            ));
            None
        }
    };
    let bundle_summary = bundle_root.as_ref().and_then(|bundle_root| {
        evidence_index_bundle_summary(&root, bundle_root, &bundle, "evidence_index", &mut issues)
    });
    let bundle_relative = evidence_index_bundle_relative(&bundle, &out);
    let mut receipt = evidence_index_final_receipt(
        &root,
        "evidence index",
        issues,
        bundle_summary,
        bundle_relative,
        json!({}),
    );
    if receipt.get("valid").and_then(Value::as_bool) == Some(true) {
        match CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, &out, true) {
            Ok((output_root, relative)) => {
                if output_root
                    .write(
                        &relative,
                        &serde_json::to_vec_pretty(&receipt).unwrap_or_default(),
                    )
                    .is_err()
                {
                    receipt = evidence_index_final_receipt(
                        &root,
                        "evidence index",
                        vec![evidence_index_issue(
                            "evidence_index.receipt_write_failed",
                            "out",
                            "writable index receipt",
                            "failed",
                        )],
                        None,
                        Value::Null,
                        json!({}),
                    );
                }
            }
            Err(_) => {
                receipt = evidence_index_final_receipt(
                    &root,
                    "evidence index",
                    vec![evidence_index_issue(
                        "evidence_index.output_create_failed",
                        "out",
                        "writable output parent",
                        "failed",
                    )],
                    None,
                    Value::Null,
                    json!({}),
                );
            }
        }
    }
    receipt
}
