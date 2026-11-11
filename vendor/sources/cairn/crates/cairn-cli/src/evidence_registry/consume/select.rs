use super::matching::evidence_registry_matching_entries;
use super::receipts::evidence_registry_select_final_receipt;
use super::registry::evidence_registry_consume_registry;
use super::*;

pub(crate) fn evidence_registry_select_receipt(options: &EvidenceRegistrySelectOptions) -> Value {
    // r[impl cairn.supersede.evidence_registry_consume.select]
    let root = options.root.clone();
    let registry_path = options
        .registry
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    let out = options.out.clone().unwrap_or_else(|| PathBuf::from("."));
    let (registry, verification, mut issues) =
        evidence_registry_consume_registry(&root, &registry_path, "evidence_registry_select");
    let entries = registry
        .get("entries")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let matched_entries =
        evidence_registry_matching_entries(&entries, &options.selectors, &mut issues);
    let mut selected_entry = Value::Null;
    if matched_entries.is_empty() {
        issues.push(evidence_registry_issue(
            "evidence_registry_select.no_match",
            "selectors",
            "exactly one matching registry entry",
            "none",
        ));
    } else if matched_entries.len() > 1 {
        issues.push(evidence_registry_issue(
            "evidence_registry_select.ambiguous_match",
            "selectors",
            "exactly one matching registry entry",
            "multiple",
        ));
    } else {
        selected_entry = matched_entries[0].clone();
    }

    if issues.is_empty() {
        let base = registry_path.parent().unwrap_or_else(|| Path::new("."));
        let Some(index_path) = evidence_registry_resolve_index_path(base, &selected_entry) else {
            return evidence_registry_select_final_receipt(
                &root,
                vec![evidence_registry_issue(
                    "evidence_registry_select.index_path_unsafe",
                    "selected.index_relative",
                    "safe relative index path",
                    "unsafe",
                )],
                &options.selectors,
                verification,
                Value::Null,
            );
        };
        let source_bytes =
            CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, &index_path, false)
                .and_then(|(index_root, relative)| {
                    index_root.read_bounded(&relative, AGENT_RECEIPT_MAX_BYTES)
                });
        match source_bytes {
            Ok(bytes) => {
                match CapabilityRoot::open_file_parent(CapabilityRootKind::Registry, &out, true) {
                    Ok((output_root, relative)) => {
                        if output_root.write(&relative, &bytes).is_err() {
                            issues.push(evidence_registry_issue(
                                "evidence_registry_select.output_write_failed",
                                "out",
                                "writable selected index artifact",
                                "failed",
                            ));
                        }
                    }
                    Err(_) => issues.push(evidence_registry_issue(
                        "evidence_registry_select.output_create_failed",
                        "out",
                        "writable output parent",
                        "failed",
                    )),
                }
            }
            Err(_) => issues.push(evidence_registry_issue(
                "evidence_registry_select.index_unreadable",
                "selected.index_relative",
                "readable selected index",
                "unreadable",
            )),
        }
    }

    evidence_registry_select_final_receipt(
        &root,
        issues,
        &options.selectors,
        verification,
        selected_entry,
    )
}
