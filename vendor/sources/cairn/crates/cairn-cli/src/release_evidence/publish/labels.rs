use serde_json::{json, Value};

use super::options::ReleaseEvidencePublishOptions;
use crate::release_evidence::common::release_evidence_issue;

pub(super) fn release_evidence_publish_labels(options: &ReleaseEvidencePublishOptions) -> Value {
    let mut object = serde_json::Map::new();
    if let Some(channel) = &options.channel {
        object.insert("channel".to_string(), json!(channel));
    }
    if let Some(release) = &options.release {
        object.insert("release".to_string(), json!(release));
    }
    if let Some(retention) = &options.retention {
        object.insert("retention".to_string(), json!(retention));
    }
    Value::Object(object)
}

pub(in crate::release_evidence) fn validate_publish_label(
    field: &str,
    value: Option<&str>,
    issues: &mut Vec<Value>,
) {
    if let Some(text) = value {
        if text.is_empty()
            || text.contains("..")
            || text.contains('/')
            || text.contains('\\')
            || !text
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
        {
            issues.push(release_evidence_issue(
                "release_evidence_publish.label_invalid",
                field,
                "safe policy label text",
                "invalid",
            ));
        }
    }
}
