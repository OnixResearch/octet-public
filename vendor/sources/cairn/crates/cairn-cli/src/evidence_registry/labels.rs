use std::path::Path;

use serde_json::{json, Value};

use crate::support::{default_policy_path, load_policy_file, read_string_option};
use cairn_core::canonical_json_hash;

use super::common::evidence_registry_issue;

#[derive(Default, Clone)]
pub(crate) struct EvidenceRegistryLabels {
    pub(super) channel: Option<String>,
    pub(super) release: Option<String>,
    pub(super) platform: Option<String>,
    pub(super) compatibility_class: Option<String>,
}
pub(super) fn evidence_registry_labels_from_args(args: &[String]) -> EvidenceRegistryLabels {
    EvidenceRegistryLabels {
        channel: read_string_option(args, "--channel"),
        release: read_string_option(args, "--release"),
        platform: read_string_option(args, "--platform"),
        compatibility_class: read_string_option(args, "--compatibility-class"),
    }
}

pub(super) fn evidence_registry_labels_json(labels: &EvidenceRegistryLabels) -> Value {
    json!({
        "channel": labels.channel.clone().unwrap_or_else(|| "unspecified".to_string()),
        "release": labels.release.clone().unwrap_or_else(|| "unspecified".to_string()),
        "platform": labels.platform.clone().unwrap_or_else(|| "unspecified".to_string()),
        "compatibility_class": labels
            .compatibility_class
            .clone()
            .unwrap_or_else(|| "unspecified".to_string()),
    })
}

pub(super) fn evidence_registry_validate_labels(
    root: &Path,
    labels: &EvidenceRegistryLabels,
    prefix: &'static str,
    issues: &mut Vec<Value>,
) {
    let Ok(policy) = load_policy_file(default_policy_path().into()) else {
        issues.push(evidence_registry_issue(
            &format!("{prefix}.label_policy_unreadable"),
            "registry_label_policy",
            "readable generated policy",
            "unreadable",
        ));
        return;
    };
    let policy_hash = canonical_json_hash(&policy.raw);
    if policy_hash.is_empty() || root.as_os_str().is_empty() {
        issues.push(evidence_registry_issue(
            &format!("{prefix}.label_policy_hash_invalid"),
            "registry_label_policy",
            "BLAKE3 policy hash",
            "invalid",
        ));
    }
    for (field, value, policy_key) in [
        ("channel", labels.channel.as_deref(), "channels"),
        ("platform", labels.platform.as_deref(), "platforms"),
        (
            "compatibility_class",
            labels.compatibility_class.as_deref(),
            "compatibility_classes",
        ),
    ] {
        if let Some(value) = value {
            if evidence_registry_label_value_unsafe(value) {
                issues.push(evidence_registry_issue(
                    &format!("{prefix}.label_unsafe"),
                    field,
                    "stable non-path label value",
                    "unsafe",
                ));
                continue;
            }
            let allowed = registry_label_policy_values(&policy.raw, policy_key);
            if !allowed.iter().any(|allowed| allowed == value) {
                issues.push(evidence_registry_issue(
                    &format!("{prefix}.label_not_allowed"),
                    field,
                    "policy-owned allowed label",
                    value,
                ));
            }
        }
    }
    if let Some(release) = labels.release.as_deref() {
        if evidence_registry_label_value_unsafe(release) {
            issues.push(evidence_registry_issue(
                &format!("{prefix}.label_unsafe"),
                "release",
                "stable non-path label value",
                "unsafe",
            ));
        } else if !evidence_registry_release_label_valid(release) {
            issues.push(evidence_registry_issue(
                &format!("{prefix}.release_label_invalid"),
                "release",
                "YYYY-MM-DD release label",
                release,
            ));
        }
    }
    let label_count = [
        labels.channel.as_ref(),
        labels.release.as_ref(),
        labels.platform.as_ref(),
        labels.compatibility_class.as_ref(),
    ]
    .iter()
    .flatten()
    .count();
    if label_count > 0 && labels.channel.is_none() {
        issues.push(evidence_registry_issue(
            &format!("{prefix}.channel_required_for_labeled_registry"),
            "channel",
            "channel label when registry labels are present",
            "missing",
        ));
    }
    // r[impl cairn.supersede.registry_channel_metadata.policy]
}

fn registry_label_policy_values(policy: &Value, key: &str) -> Vec<String> {
    policy
        .get("registry_label_policy")
        .and_then(|policy| policy.get(key))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn evidence_registry_label_value_unsafe(value: &str) -> bool {
    value.is_empty()
        || value.starts_with('/')
        || value.contains("..")
        || value.contains('\\')
        || value.contains(':')
        || value.chars().any(char::is_whitespace)
}

fn evidence_registry_release_label_valid(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(idx, byte)| idx == 4 || idx == 7 || byte.is_ascii_digit())
}

pub(super) fn entry_labels_from_registry_entry(entry: &Value) -> Option<EvidenceRegistryLabels> {
    let labels = entry.get("labels")?;
    Some(EvidenceRegistryLabels {
        channel: registry_entry_label(labels, "channel"),
        release: registry_entry_label(labels, "release"),
        platform: registry_entry_label(labels, "platform"),
        compatibility_class: registry_entry_label(labels, "compatibility_class"),
    })
}

fn registry_entry_label(labels: &Value, key: &str) -> Option<String> {
    labels
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| *value != "unspecified")
        .map(str::to_string)
}

fn evidence_registry_label_key(entry: &Value) -> Option<String> {
    let labels = entry.get("labels")?;
    let channel = labels
        .get("channel")
        .and_then(Value::as_str)
        .unwrap_or("unspecified");
    if channel == "unspecified" {
        return None;
    }
    let release = labels
        .get("release")
        .and_then(Value::as_str)
        .unwrap_or("unspecified");
    let platform = labels
        .get("platform")
        .and_then(Value::as_str)
        .unwrap_or("unspecified");
    let compatibility = labels
        .get("compatibility_class")
        .and_then(Value::as_str)
        .unwrap_or("unspecified");
    Some(format!("{channel}|{release}|{platform}|{compatibility}"))
}

pub(super) fn evidence_registry_reject_label_conflicts(
    entries: &[Value],
    prefix: &'static str,
    issues: &mut Vec<Value>,
) {
    let mut seen = std::collections::BTreeMap::<String, String>::new();
    for entry in entries {
        let Some(key) = evidence_registry_label_key(entry) else {
            continue;
        };
        let hash = entry
            .get("index_receipt_hash")
            .and_then(Value::as_str)
            .unwrap_or("<missing>")
            .to_string();
        if let Some(previous) = seen.insert(key, hash.clone()) {
            if previous != hash {
                issues.push(evidence_registry_issue(
                    &format!("{prefix}.label_conflict"),
                    "labels",
                    "unique label tuple per index hash",
                    "conflict",
                ));
            }
        }
    }
    // r[impl cairn.supersede.registry_channel_metadata.build]
}
