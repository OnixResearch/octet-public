use super::*;

pub(super) fn evidence_registry_matching_entries(
    entries: &[Value],
    selectors: &EvidenceRegistryConsumeSelectors,
    issues: &mut Vec<Value>,
) -> Vec<Value> {
    if let Some(issue) = evidence_registry_selector_issue(selectors) {
        issues.push(issue);
        return Vec::new();
    }
    entries
        .iter()
        .filter(|entry| evidence_registry_entry_matches(entry, selectors))
        .cloned()
        .collect()
}

fn evidence_registry_entry_matches(
    entry: &Value,
    selectors: &EvidenceRegistryConsumeSelectors,
) -> bool {
    if !evidence_registry_match_string(entry, "bundle_kind", selectors.bundle_kind.as_deref()) {
        return false;
    }
    if !evidence_registry_match_string(entry, "policy_hash", selectors.policy_hash.as_deref()) {
        return false;
    }
    if !evidence_registry_match_label(entry, "channel", selectors.channel.as_deref()) {
        return false;
    }
    if !evidence_registry_match_label(entry, "release", selectors.release.as_deref()) {
        return false;
    }
    if !evidence_registry_match_label(entry, "platform", selectors.platform.as_deref()) {
        return false;
    }
    if !evidence_registry_match_label(
        entry,
        "compatibility_class",
        selectors.compatibility_class.as_deref(),
    ) {
        return false;
    }
    if !evidence_registry_match_string(
        entry,
        "index_receipt_hash",
        selectors.index_receipt_hash.as_deref(),
    ) {
        return false;
    }
    if !evidence_registry_match_string(
        entry,
        "index_verification_receipt_hash",
        selectors.index_verification_receipt_hash.as_deref(),
    ) {
        return false;
    }
    if !evidence_registry_match_string(
        entry,
        "bundle_receipt_hash",
        selectors.bundle_receipt_hash.as_deref(),
    ) {
        return false;
    }
    if !evidence_registry_match_receipt_chain(
        entry,
        "coverage_audit_receipt_hash",
        selectors.coverage_audit_receipt_hash.as_deref(),
    ) {
        return false;
    }
    if !evidence_registry_match_receipt_chain(
        entry,
        "release_readiness_receipt_hash",
        selectors.release_readiness_receipt_hash.as_deref(),
    ) {
        return false;
    }
    if !evidence_registry_match_artifact(entry, selectors) {
        return false;
    }
    true
}

fn evidence_registry_match_string(entry: &Value, key: &str, selector: Option<&str>) -> bool {
    match selector {
        Some(expected) => entry.get(key).and_then(Value::as_str) == Some(expected),
        None => true,
    }
}

fn evidence_registry_match_label(entry: &Value, key: &str, selector: Option<&str>) -> bool {
    match selector {
        Some(expected) => {
            entry
                .get("labels")
                .and_then(|labels| labels.get(key))
                .and_then(Value::as_str)
                == Some(expected)
        }
        None => true,
    }
}

fn evidence_registry_match_receipt_chain(entry: &Value, key: &str, selector: Option<&str>) -> bool {
    match selector {
        Some(expected) => {
            entry
                .get("receipt_chain")
                .and_then(|chain| chain.get(key))
                .and_then(Value::as_str)
                == Some(expected)
        }
        None => true,
    }
}

fn evidence_registry_match_artifact(
    entry: &Value,
    selectors: &EvidenceRegistryConsumeSelectors,
) -> bool {
    if selectors.artifact_role.is_none() && selectors.artifact_hash.is_none() {
        return true;
    }
    let Some(artifacts) = entry.get("artifacts").and_then(Value::as_array) else {
        return false;
    };
    artifacts.iter().any(|artifact| {
        let role_ok = selectors
            .artifact_role
            .as_deref()
            .is_none_or(|role| artifact.get("role").and_then(Value::as_str) == Some(role));
        let hash_ok = selectors
            .artifact_hash
            .as_deref()
            .is_none_or(|hash| artifact.get("hash").and_then(Value::as_str) == Some(hash));
        role_ok && hash_ok
    })
}

fn evidence_registry_selector_issue(selectors: &EvidenceRegistryConsumeSelectors) -> Option<Value> {
    let selector_values = [
        selectors.bundle_kind.as_deref(),
        selectors.policy_hash.as_deref(),
        selectors.channel.as_deref(),
        selectors.release.as_deref(),
        selectors.platform.as_deref(),
        selectors.compatibility_class.as_deref(),
        selectors.index_receipt_hash.as_deref(),
        selectors.index_verification_receipt_hash.as_deref(),
        selectors.bundle_receipt_hash.as_deref(),
        selectors.artifact_role.as_deref(),
        selectors.artifact_hash.as_deref(),
        selectors.coverage_audit_receipt_hash.as_deref(),
        selectors.release_readiness_receipt_hash.as_deref(),
    ];
    if selector_values.iter().flatten().any(|value| {
        value.is_empty() || value.starts_with('/') || value.contains("..") || value.contains('\\')
    }) {
        return Some(evidence_registry_issue(
            "evidence_registry_consume.selector_unsafe",
            "selectors",
            "stable non-path selector values",
            "unsafe",
        ));
    }
    None
}
