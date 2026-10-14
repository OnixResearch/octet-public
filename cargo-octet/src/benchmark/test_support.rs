use super::*;

pub(super) const BASE_DURATION_NS: u64 = 1_000;
pub(super) const SMALL_DURATION_DELTA_NS: u64 = 10;
pub(super) const IMPROVED_DURATION_NS: u64 = 800;
pub(super) const REGRESSED_DURATION_NS: u64 = 1_300;
pub(super) const NOISY_DURATION_NS: u64 = 5_000;
pub(super) const SAMPLE_COUNT: usize = 5;
pub(super) const KNOWN_COPY_BYTES: u64 = 32;
pub(super) const ALLOCATION_EVENTS: u64 = 2;
pub(super) const ALLOCATED_BYTES: u64 = 64;
pub(super) const EXPECTED_SIGNAL_CLASS_COUNT: u64 = 1;
pub(super) const COMPILER_IDENTITY_INPUT: &[u8] = b"compiler";
const OUTPUT_BYTES: u64 = 128;

pub(super) fn valid_input() -> request::Input {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let baseline = run_set(&profile, BASE_DURATION_NS);
    let candidate = run_set(&profile, BASE_DURATION_NS + SMALL_DURATION_DELTA_NS);
    let compiler_identity = b3(COMPILER_IDENTITY_INPUT);
    let copies = resources::scan_llvm_copy_sites(
        &baseline.workload_id,
        "call void @llvm.memcpy.p0.p0.i64(ptr %a, ptr %b, i64 32, i1 false)\n\
         call void @llvm.memcpy.p0.p0.i64(ptr %a, ptr %b, i64 %length, i1 false)\n",
        &compiler_identity,
        "x86_64-unknown-linux-gnu",
        "release",
    )
    .expect("copy census");
    request::Input {
        schema_version: INPUT_SCHEMA.to_owned(),
        baseline: baseline.clone(),
        candidate,
        allocations: vec![AllocationObservation {
            workload_id: baseline.workload_id.clone(),
            instrumentation_identity: b3(b"allocator-wrapper"),
            interval_id: "selected-hot-path".to_owned(),
            scope: "between explicit benchmark interval markers".to_owned(),
            allocation_events: ALLOCATION_EVENTS,
            allocated_bytes: ALLOCATED_BYTES,
            zero_allocation_claim: false,
        }],
        copies,
        oracle: oracle(),
        findings: findings(),
        public_claims: Vec::new(),
        bounded_log_identity: b3(b"bounded-log"),
    }
}

pub(super) fn run_set(profile: &Profile, duration: u64) -> RunSet {
    let workload = profile.workloads.iter().find(|workload| workload.mode == "no_change").expect("no-change workload");
    run_set_for_workload(profile, workload, duration)
}

pub(super) fn run_set_for_workload(profile: &Profile, workload: &WorkloadProfile, duration: u64) -> RunSet {
    RunSet {
        workload_id: workload.workload_id.clone(),
        workload_identity: workload.workload_identity.clone(),
        command_id: workload.command_id.clone(),
        command_argv: workload.command_argv.clone(),
        mode: workload.mode.clone(),
        source_state_identity: workload.source_state_identity.clone(),
        environment: EnvironmentObservation {
            environment_identity: workload.environment_identity.clone(),
            target: "x86_64-unknown-linux-gnu".to_owned(),
            host_os: "linux".to_owned(),
            host_arch: "x86_64".to_owned(),
            kernel_identity: "kernel-fixture".to_owned(),
            cpu_identity: "cpu-fixture".to_owned(),
            governor: "performance".to_owned(),
            load_class: "isolated".to_owned(),
            rust_toolchain: profile.rust_toolchain.clone(),
            nix_closure_identity: profile.provider.closure_identity.clone(),
            isolation: "controlled-local-fixture".to_owned(),
            cpu_time_supported: false,
            peak_memory_supported: false,
        },
        samples: (0..SAMPLE_COUNT)
            .map(|sample_index| Sample {
                sample_index,
                outcome: "succeeded".to_owned(),
                wall_duration_ns: duration,
                cpu_duration_ns: None,
                peak_memory_bytes: None,
                output_bytes: OUTPUT_BYTES,
                log_identity: b3(format!("sample-{sample_index}").as_bytes()),
            })
            .collect(),
    }
}

pub(super) fn set_all_durations(run: &mut RunSet, duration: u64) {
    for sample in &mut run.samples {
        sample.wall_duration_ns = duration;
    }
}

fn oracle() -> Vec<OracleRecord> {
    [
        ("tp", "finding"),
        ("fp", "clean"),
        ("fn", "finding"),
        ("expected-suppression", "suppressed"),
        ("unexpected-suppression", "clean"),
        ("inconclusive", "inconclusive"),
    ]
    .into_iter()
    .map(|(case_id, expected)| OracleRecord {
        case_id: case_id.to_owned(),
        source_identity: b3(case_id.as_bytes()),
        lint_id: "octet.test-lint".to_owned(),
        location_class: "expression".to_owned(),
        expected: expected.to_owned(),
        rationale: "independent fixture adjudication".to_owned(),
        reviewer_role: "benchmark-oracle-reviewer".to_owned(),
    })
    .collect()
}

fn findings() -> Vec<SignalFinding> {
    [
        ("tp", false),
        ("fp", false),
        ("expected-suppression", true),
        ("unexpected-suppression", true),
    ]
    .into_iter()
    .map(|(case_id, suppressed)| SignalFinding {
        case_id: case_id.to_owned(),
        source_identity: b3(case_id.as_bytes()),
        lint_id: "octet.test-lint".to_owned(),
        location_class: "expression".to_owned(),
        suppressed,
    })
    .collect()
}

pub(super) fn b3(bytes: &[u8]) -> String {
    prefixed_identity(bytes)
}
