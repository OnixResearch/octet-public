use super::matching::evidence_registry_matching_entries;
use super::receipts::evidence_registry_fetch_plan_final_receipt;
use super::registry::evidence_registry_consume_registry;
use super::*;

pub(crate) fn evidence_registry_fetch_plan_receipt(
    options: &EvidenceRegistryFetchPlanOptions,
) -> Value {
    // r[impl cairn.supersede.evidence_registry_consume.fetch_plan]
    let root = options.root.clone();
    let registry_path = options
        .registry
        .clone()
        .unwrap_or_else(|| PathBuf::from("."));
    let out = options.out.clone().unwrap_or_else(|| PathBuf::from("."));
    let (registry, verification, mut issues) =
        evidence_registry_consume_registry(&root, &registry_path, "evidence_registry_fetch_plan");
    let entries = registry
        .get("entries")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut plan_entries =
        evidence_registry_matching_entries(&entries, &options.selectors, &mut issues);
    plan_entries.sort_by_key(canonical_json_hash);
    let mut receipt = evidence_registry_fetch_plan_final_receipt(
        &root,
        issues,
        &options.selectors,
        verification,
        plan_entries,
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
                    receipt = evidence_registry_fetch_plan_final_receipt(
                        &root,
                        vec![evidence_registry_issue(
                            "evidence_registry_fetch_plan.output_write_failed",
                            "out",
                            "writable fetch plan receipt",
                            "failed",
                        )],
                        &options.selectors,
                        Value::Null,
                        Vec::new(),
                    );
                }
            }
            Err(_) => {
                receipt = evidence_registry_fetch_plan_final_receipt(
                    &root,
                    vec![evidence_registry_issue(
                        "evidence_registry_fetch_plan.output_create_failed",
                        "out",
                        "writable output parent",
                        "failed",
                    )],
                    &options.selectors,
                    Value::Null,
                    Vec::new(),
                );
            }
        }
    }
    receipt
}
