use super::*;

const RETAINED_LOG_PREFIX: &str = "mutation-log-";
const RETAINED_DIFF_PREFIX: &str = "mutation-diff-";

pub(super) fn validate_artifacts(
    artifacts: &std::collections::BTreeMap<String, String>,
    profile: &Profile,
) -> Result<(), Vec<String>> {
    if artifacts.len() > profile.max_artifact_count {
        return Err(vec!["mutation artifact count exceeds profile limit".to_owned()]);
    }
    let mut total = 0_u64;
    for (path, text) in artifacts {
        if !safe_path(path) {
            return Err(vec![format!("mutation artifact path `{path}` is unsafe")]);
        }
        let bytes =
            u64::try_from(text.len()).map_err(|_| vec!["mutation artifact length does not fit u64".to_owned()])?;
        total = total.checked_add(bytes).ok_or_else(|| vec!["mutation artifact byte count overflow".to_owned()])?;
        if total > profile.max_artifact_bytes {
            return Err(vec!["mutation artifacts exceed total byte limit".to_owned()]);
        }
    }
    Ok(())
}

pub(super) fn baseline_record(
    outcome: &validation::NativeOutcome,
    artifacts: &std::collections::BTreeMap<String, String>,
    profile: &Profile,
) -> Result<BaselineRecord, String> {
    let normalized = if outcome.summary == validation::NATIVE_SUCCESS {
        "passed"
    } else {
        "failed"
    };
    Ok(BaselineRecord {
        outcome: normalized.to_owned(),
        log: retained_artifact(&outcome.log_path, format!("{RETAINED_LOG_PREFIX}baseline.txt"), artifacts, profile)?,
    })
}

pub(super) fn mutant_record(
    native: validation::NativeMutant,
    outcome: &validation::NativeOutcome,
    source_identity: String,
    artifacts: &std::collections::BTreeMap<String, String>,
    profile: &Profile,
) -> Result<MutantRecord, String> {
    if !safe_path(&native.file) || native.span.start.line == 0 || native.span.start.column == 0 {
        return Err(format!("mutation inventory entry `{}` has an unsafe location", native.name));
    }
    let stable_id = stable_id(&native, &source_identity);
    let log = retained_artifact(
        &outcome.log_path,
        format!("{RETAINED_LOG_PREFIX}{}.txt", stable_id.trim_start_matches("b3:")),
        artifacts,
        profile,
    )?;
    let diff = match outcome.diff_path.as_deref() {
        Some(path) => Some(retained_artifact(
            path,
            format!("{RETAINED_DIFF_PREFIX}{}.diff", stable_id.trim_start_matches("b3:")),
            artifacts,
            profile,
        )?),
        None => None,
    };
    Ok(MutantRecord {
        stable_id,
        name: native.name,
        package: native.package,
        file: native.file,
        line: native.span.start.line,
        column: native.span.start.column,
        replacement: native.replacement,
        genre: native.genre,
        source_identity,
        outcome: normalize_outcome(&outcome.summary).to_owned(),
        log,
        diff,
    })
}

pub(super) fn duplicate_id(mutants: &[MutantRecord]) -> Option<String> {
    mutants
        .windows(2)
        .find(|items| items[0].stable_id == items[1].stable_id)
        .map(|items| format!("duplicate normalized mutation identity `{}`", items[0].stable_id))
}

pub(super) fn outcome_counts(mutants: &[MutantRecord]) -> std::collections::BTreeMap<String, u64> {
    let mut counts = std::collections::BTreeMap::from([
        ("caught".to_owned(), 0_u64),
        ("missed".to_owned(), 0_u64),
        ("timeout".to_owned(), 0_u64),
        ("unviable".to_owned(), 0_u64),
        ("not_run".to_owned(), 0_u64),
        ("tool_failure".to_owned(), 0_u64),
    ]);
    for mutant in mutants {
        let count = counts.entry(mutant.outcome.clone()).or_insert(0_u64);
        *count = count.saturating_add(1);
    }
    counts
}

pub(super) fn gate_status(counts: &std::collections::BTreeMap<String, u64>, baseline: &str) -> String {
    if baseline != "passed"
        || present(counts, "missed")
        || present(counts, "not_run")
        || present(counts, "tool_failure")
    {
        "failed".to_owned()
    } else if present(counts, "timeout") || present(counts, "unviable") {
        "manual_review_required".to_owned()
    } else {
        "passed".to_owned()
    }
}

pub(super) fn policy_diagnostics(status: &str) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if status == "failed" {
        diagnostics.push("cargo-mutants policy observed a failed outcome class".to_owned());
    } else if status == "manual_review_required" {
        diagnostics.push("cargo-mutants policy requires review for incomplete outcomes".to_owned());
    }
    diagnostics
}

pub(super) fn projection_identity(mutants: &[MutantRecord]) -> Result<String, String> {
    let mut bytes = serde_json::to_vec_pretty(mutants)
        .map_err(|error| format!("serialize normalized mutation projection: {error}"))?;
    bytes.push(b'\n');
    Ok(prefixed_identity(&bytes))
}

pub(super) fn safe_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains(':')
        && path.split(['/', '\\']).all(|part| !matches!(part, "" | "." | ".."))
}

pub(super) fn is_b3(value: &str) -> bool {
    value.strip_prefix("b3:").is_some_and(|digest| {
        digest.len() == blake3::OUT_LEN * 2 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn retained_artifact(
    source_locator: &str,
    locator: String,
    artifacts: &std::collections::BTreeMap<String, String>,
    profile: &Profile,
) -> Result<RetainedArtifact, String> {
    if !safe_path(source_locator) {
        return Err(format!("mutation artifact locator `{source_locator}` is unsafe"));
    }
    let text = artifacts
        .get(source_locator)
        .ok_or_else(|| format!("mutation artifact `{source_locator}` is missing"))?;
    if text.len() > profile.max_log_bytes {
        return Err(format!("mutation artifact `{source_locator}` exceeds its byte limit"));
    }
    Ok(RetainedArtifact {
        source_locator: source_locator.to_owned(),
        locator,
        identity: prefixed_identity(text.as_bytes()),
        bytes: u64::try_from(text.len()).map_err(|_| "mutation artifact length does not fit u64".to_owned())?,
    })
}

fn stable_id(native: &validation::NativeMutant, source_identity: &str) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"octet-cargo-mutant-v1\0");
    for value in [
        native.package.as_str(),
        native.file.as_str(),
        native.name.as_str(),
        native.replacement.as_str(),
        native.genre.as_str(),
        source_identity,
    ] {
        hasher.update(&value.len().to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.update(&native.span.start.line.to_le_bytes());
    hasher.update(&native.span.start.column.to_le_bytes());
    format!("b3:{}", hasher.finalize().to_hex())
}

fn normalize_outcome(summary: &str) -> &str {
    match summary {
        "CaughtMutant" => "caught",
        "MissedMutant" => "missed",
        "Timeout" => "timeout",
        "Unviable" => "unviable",
        "NotRun" => "not_run",
        _ => "tool_failure",
    }
}

fn present(counts: &std::collections::BTreeMap<String, u64>, class: &str) -> bool {
    counts.get(class).is_some_and(|count| *count > 0)
}
