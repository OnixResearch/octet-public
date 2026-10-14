// r[impl octet.benchmark_evidence.resource_observation]
use super::*;

const BLAKE3_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
const COPY_INTRINSIC: &str = "@llvm.memcpy";
const LENGTH_TYPES: [&str; 2] = ["i64", "i32"];

pub(super) fn summarize(
    allocations: &[AllocationObservation],
    copies: &[CopyObservation],
) -> Result<ResourceSummary, Vec<String>> {
    let mut diagnostics = validate_allocations(allocations);
    diagnostics.extend(validate_copies(copies));
    let mut allocation_events = 0_u64;
    let mut allocated_bytes = 0_u64;
    for observation in allocations {
        allocation_events = match allocation_events.checked_add(observation.allocation_events) {
            Some(value) => value,
            None => {
                diagnostics.push("benchmark allocation event total overflowed".to_owned());
                allocation_events
            }
        };
        allocated_bytes = match allocated_bytes.checked_add(observation.allocated_bytes) {
            Some(value) => value,
            None => {
                diagnostics.push("benchmark allocated-byte total overflowed".to_owned());
                allocated_bytes
            }
        };
    }
    let mut known_copy_bytes = 0_u64;
    let mut unknown_copy_length_count = 0_usize;
    for observation in copies {
        match observation.constant_length_bytes {
            Some(length) => {
                known_copy_bytes = match known_copy_bytes.checked_add(length) {
                    Some(value) => value,
                    None => {
                        diagnostics.push("benchmark known copy-byte total overflowed".to_owned());
                        known_copy_bytes
                    }
                };
            }
            None => {
                unknown_copy_length_count = match unknown_copy_length_count.checked_add(1) {
                    Some(value) => value,
                    None => {
                        diagnostics.push("benchmark unknown copy-site count overflowed".to_owned());
                        unknown_copy_length_count
                    }
                };
            }
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Ok(ResourceSummary {
        allocation_events,
        allocated_bytes,
        copy_site_count: copies.len(),
        known_copy_bytes,
        unknown_copy_length_count,
    })
}

pub(super) fn parse_allocation_observations(text: &str) -> Result<Vec<AllocationObservation>, String> {
    serde_json::from_str(text).map_err(|error| format!("parse benchmark allocation observations: {error}"))
}

pub(super) fn scan_llvm_copy_sites(
    workload_id: &str,
    llvm_text: &str,
    compiler_identity: &str,
    target: &str,
    optimization_profile: &str,
) -> Result<Vec<CopyObservation>, String> {
    if workload_id.is_empty() || target.is_empty() || optimization_profile.is_empty() {
        return Err("LLVM copy census requires workload, target, and optimization profile".to_owned());
    }
    if !is_b3(compiler_identity) {
        return Err("LLVM copy census compiler identity is malformed".to_owned());
    }
    let llvm_artifact_identity = prefixed_identity(llvm_text.as_bytes());
    let mut observations = Vec::new();
    for (line_index, line) in llvm_text.lines().enumerate() {
        if !line.contains(COPY_INTRINSIC) {
            continue;
        }
        let site_id = copy_site_identity(&llvm_artifact_identity, line_index, line)?;
        observations.push(CopyObservation {
            workload_id: workload_id.to_owned(),
            llvm_artifact_identity: llvm_artifact_identity.clone(),
            compiler_identity: compiler_identity.to_owned(),
            target: target.to_owned(),
            optimization_profile: optimization_profile.to_owned(),
            site_id,
            context: line.trim().to_owned(),
            constant_length_bytes: copy_length(line),
        });
    }
    Ok(observations)
}

fn validate_allocations(observations: &[AllocationObservation]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut instrumentation_by_workload = std::collections::BTreeMap::new();
    for observation in observations {
        if observation.workload_id.is_empty()
            || observation.interval_id.is_empty()
            || observation.scope.is_empty()
            || !is_b3(&observation.instrumentation_identity)
        {
            diagnostics.push("benchmark allocation observation has missing scope or identity".to_owned());
        }
        if instrumentation_by_workload
            .insert(&observation.workload_id, &observation.instrumentation_identity)
            .is_some_and(|prior| prior != &observation.instrumentation_identity)
        {
            diagnostics.push(format!(
                "benchmark allocation instrumentation drifted for workload `{}`",
                observation.workload_id
            ));
        }
        if observation.zero_allocation_claim && (observation.allocation_events != 0 || observation.allocated_bytes != 0)
        {
            diagnostics.push(format!(
                "benchmark allocation interval `{}` contradicts its zero-allocation claim",
                observation.interval_id
            ));
        }
    }
    diagnostics
}

fn validate_copies(observations: &[CopyObservation]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut cohort_by_workload = std::collections::BTreeMap::new();
    for observation in observations {
        let cohort = (
            &observation.llvm_artifact_identity,
            &observation.compiler_identity,
            &observation.target,
            &observation.optimization_profile,
        );
        if cohort_by_workload.insert(&observation.workload_id, cohort).is_some_and(|prior| prior != cohort) {
            diagnostics.push(format!("benchmark LLVM copy cohort drifted for workload `{}`", observation.workload_id));
        }
        if observation.workload_id.is_empty()
            || observation.target.is_empty()
            || observation.optimization_profile.is_empty()
            || observation.context.is_empty()
            || !is_b3(&observation.llvm_artifact_identity)
            || !is_b3(&observation.compiler_identity)
            || !is_b3(&observation.site_id)
        {
            diagnostics.push("benchmark copy observation has missing scope or identity".to_owned());
        }
    }
    diagnostics
}

fn copy_length(line: &str) -> Option<u64> {
    let intrinsic = line.find(COPY_INTRINSIC)?;
    let suffix = line.get(intrinsic..)?;
    let tokens: Vec<&str> = suffix.split_ascii_whitespace().collect();
    for pair in tokens.windows(LENGTH_TYPES.len()) {
        if LENGTH_TYPES.contains(&pair[0]) {
            let value = pair[1].trim_matches(|character: char| !character.is_ascii_digit());
            if !value.is_empty() {
                return value.parse().ok();
            }
        }
    }
    None
}

fn copy_site_identity(artifact_identity: &str, line_index: usize, line: &str) -> Result<String, String> {
    let index = u64::try_from(line_index).map_err(|_| "LLVM copy-site line index does not fit u64".to_owned())?;
    let mut hasher = blake3::Hasher::new();
    update_frame(&mut hasher, b"octet-benchmark-copy-site-v1")?;
    update_frame(&mut hasher, artifact_identity.as_bytes())?;
    update_frame(&mut hasher, &index.to_le_bytes())?;
    update_frame(&mut hasher, line.as_bytes())?;
    Ok(format!("b3:{}", hasher.finalize().to_hex()))
}

fn update_frame(hasher: &mut blake3::Hasher, bytes: &[u8]) -> Result<(), String> {
    let length =
        u64::try_from(bytes.len()).map_err(|_| "benchmark copy-site frame length does not fit u64".to_owned())?;
    hasher.update(&length.to_le_bytes());
    hasher.update(bytes);
    Ok(())
}

pub(super) fn is_b3(value: &str) -> bool {
    value
        .strip_prefix(BLAKE3_PREFIX)
        .is_some_and(|hex| hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_hexdigit()))
}
