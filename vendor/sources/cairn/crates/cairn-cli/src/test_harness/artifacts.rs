use serde_json::{json, Value};
use std::{error::Error, fs, path::Path};

pub(super) const ARTIFACT_SCHEMA: &str = "cairn.test-harness.artifact.v1";

pub(super) fn write_artifact(
    path: &Path,
    case: &str,
    seed: &str,
    status: &str,
) -> Result<(), Box<dyn Error>> {
    fs::write(
        path,
        format!(
            "{}
",
            serde_json::to_string_pretty(
                &json!({"schema":ARTIFACT_SCHEMA,"case_id":case,"seed":seed,"status":status})
            )?
        ),
    )?;
    Ok(())
}

pub(super) fn artifact_schema(bytes: &[u8]) -> &'static str {
    serde_json::from_slice::<Value>(bytes)
        .ok()
        .and_then(|v| v.get("schema").and_then(Value::as_str).map(str::to_string))
        .filter(|s| s == ARTIFACT_SCHEMA)
        .map(|_| ARTIFACT_SCHEMA)
        .unwrap_or("invalid")
}
