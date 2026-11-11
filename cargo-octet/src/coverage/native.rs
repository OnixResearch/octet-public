use super::Counts;
use super::MetricCount;
use super::NATIVE_SCHEMA;
use super::NativeReport;
use super::PROFILE_IDENTITY;
use super::Profile;
use super::SOURCE_IDENTITY_PREFIX;
use super::comparison;
use super::prefixed_identity;

const SUPPORTED_NATIVE_TYPE: &str = "llvm.coverage.json.export";
// 2.0.1 matched the archived cargo-llvm-cov 0.8.3 probe; 3.1.0 is what cargo-llvm-cov 0.8.3 emits
// with the pinned nightly-2026-03-21 LLVM tools. Both carry the same `data[0].files[].filename` and
// `data[0].totals.{lines,branches,functions,regions}.{covered,count}` fields that normalization
// reads.
const SUPPORTED_NATIVE_VERSIONS: [&str; 2] = ["2.0.1", "3.1.0"];
const ADJACENT_PATH_COUNT: usize = 2;
const HEX_CHARACTERS_PER_BYTE: usize = 2;
const BRANCH_METRIC: &str = "branches";
const NO_BRANCH_COUNTS_DIAGNOSTIC: &str = "coverage native report has no branch counts; run cargo llvm-cov with `--branch` so the branches metric is measured";

pub(super) fn normalize(text: &str, profile: &Profile, source_identity: &str) -> Result<NativeReport, Vec<String>> {
    let Ok(report_bytes) = u64::try_from(text.len()) else {
        return Err(vec!["coverage report byte count does not fit u64".to_owned()]);
    };
    if report_bytes > profile.max_report_bytes {
        return Err(vec!["coverage report exceeds profile byte limit".to_owned()]);
    }
    if !valid_source_identity(source_identity) {
        return Err(vec!["coverage source identity is missing or malformed".to_owned()]);
    }
    let value = serde_json::from_str::<serde_json::Value>(text)
        .map_err(|error| vec![format!("coverage native report is malformed JSON: {error}")])?;
    validate_header(&value)?;
    let data = value
        .get("data")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| vec!["coverage native report is missing data".to_owned()])?;
    if data.len() != 1 {
        return Err(vec!["coverage native report must contain one data entry".to_owned()]);
    }
    let file_count = validate_files(&data[0], profile)?;
    let totals = data[0].get("totals").ok_or_else(|| vec!["coverage native report is missing totals".to_owned()])?;
    let counts = Counts {
        lines: metric(totals, "lines")?,
        branches: metric(totals, BRANCH_METRIC)?,
        functions: metric(totals, "functions")?,
        regions: metric(totals, "regions")?,
    };
    Ok(NativeReport {
        schema_version: NATIVE_SCHEMA.to_owned(),
        profile_id: profile.profile_id.clone(),
        profile_identity: PROFILE_IDENTITY.to_owned(),
        provider_identity: provider_identity(profile),
        rust_toolchain: profile.rust_toolchain.clone(),
        llvm_tools_component: profile.llvm_tools_component.clone(),
        source_scope_identity: profile.source_scope_identity.clone(),
        path_policy_identity: profile.path_policy_identity.clone(),
        source_identity: source_identity.to_owned(),
        raw_report_identity: prefixed_identity(text.as_bytes()),
        file_count,
        counts,
    })
}

fn validate_header(value: &serde_json::Value) -> Result<(), Vec<String>> {
    let observed_type = value.get("type").and_then(serde_json::Value::as_str);
    if observed_type != Some(SUPPORTED_NATIVE_TYPE) {
        return Err(vec!["coverage native report type is unsupported".to_owned()]);
    }
    let observed_version = value.get("version").and_then(serde_json::Value::as_str);
    if !observed_version.is_some_and(|version| SUPPORTED_NATIVE_VERSIONS.contains(&version)) {
        return Err(vec!["coverage native report version is unsupported".to_owned()]);
    }
    Ok(())
}

fn validate_files(data: &serde_json::Value, profile: &Profile) -> Result<usize, Vec<String>> {
    let files = data
        .get("files")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| vec!["coverage native report is missing files".to_owned()])?;
    if files.is_empty() || files.len() > profile.max_files {
        return Err(vec!["coverage native report file count is outside profile limits".to_owned()]);
    }
    let mut paths = Vec::with_capacity(files.len());
    for file in files {
        let path = file
            .get("filename")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| vec!["coverage native report file is missing filename".to_owned()])?;
        paths.push(path);
    }
    paths.sort_unstable();
    if paths.windows(ADJACENT_PATH_COUNT).any(|window| window[0] == window[1]) {
        return Err(vec!["coverage native report contains duplicate file paths".to_owned()]);
    }
    if let Some(path) = paths.iter().find(|path| !safe_path(path, profile.max_path_bytes)) {
        return Err(vec![format!("coverage native report contains unsafe path `{path}`")]);
    }
    Ok(files.len())
}

fn metric(totals: &serde_json::Value, name: &str) -> Result<MetricCount, Vec<String>> {
    let metric = totals.get(name).ok_or_else(|| vec![format!("coverage native report is missing {name} counts")])?;
    let covered = metric
        .get("covered")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| vec![format!("coverage native report {name} covered count is missing")])?;
    let total = metric
        .get("count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| vec![format!("coverage native report {name} total count is missing")])?;
    if name == BRANCH_METRIC && total == 0 {
        return Err(vec![NO_BRANCH_COUNTS_DIAGNOSTIC.to_owned()]);
    }
    let percentage_basis_points =
        comparison::basis_points(covered, total).map_err(|error| vec![format!("{name}: {error}")])?;
    Ok(MetricCount {
        covered,
        total,
        percentage_basis_points,
    })
}

fn safe_path(path: &str, max_bytes: usize) -> bool {
    !path.is_empty()
        && path.len() <= max_bytes
        && !path.starts_with('/')
        && !path.starts_with('\\')
        && !path.contains(':')
        && path.split(['/', '\\']).all(|segment| !matches!(segment, "" | "." | ".."))
}

fn valid_source_identity(value: &str) -> bool {
    value.strip_prefix(SOURCE_IDENTITY_PREFIX).is_some_and(|digest| {
        digest.len() == blake3::OUT_LEN * HEX_CHARACTERS_PER_BYTE && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
    })
}

fn provider_identity(profile: &Profile) -> String {
    format!("{}@{}:{}", profile.provider.package, profile.provider.version, profile.provider.closure_identity)
}
