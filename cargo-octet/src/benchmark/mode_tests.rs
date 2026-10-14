// r[verify octet.benchmark_evidence.verification]
use super::test_support::*;
use super::*;

const EXPECTED_MODE_COUNT: usize = 4;

#[test]
fn cold_warm_no_change_and_incremental_modes_remain_separate_bounded_groups() {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let directory = tempfile::tempdir().expect("temporary directory");
    std::fs::create_dir_all(directory.path().join(".octet")).expect("create benchmark fixture root");
    let tools = shell::RuntimeTools {
        cargo: "/bin/true".into(),
        path: "/bin".into(),
        library_path: "/lib".into(),
        kernel_identity: "kernel-fixture".to_owned(),
        cpu_identity: "cpu-fixture".to_owned(),
        governor: "performance".to_owned(),
        load_class: "isolated".to_owned(),
    };
    let mut observed_modes = std::collections::BTreeSet::new();
    for workload in &profile.workloads {
        write_resource_inputs(directory.path(), &profile, &workload.workload_id);
        let baseline = run_set_for_workload(&profile, workload, BASE_DURATION_NS);
        let observation =
            shell::run_with(directory.path(), &profile, baseline, tools.clone()).expect("bounded benchmark mode run");
        assert_eq!(observation.input.candidate.mode, workload.mode);
        assert_eq!(observation.input.candidate.samples.len(), SAMPLE_COUNT);
        assert!(observation.input.candidate.samples.iter().all(|sample| sample.outcome == "succeeded"));
        observed_modes.insert(observation.input.candidate.mode);
    }
    assert_eq!(observed_modes.len(), EXPECTED_MODE_COUNT);
}

fn write_resource_inputs(root: &std::path::Path, profile: &Profile, workload_id: &str) {
    let allocations = vec![AllocationObservation {
        workload_id: workload_id.to_owned(),
        instrumentation_identity: b3(b"mode-allocation-instrumentation"),
        interval_id: "mode-selected-interval".to_owned(),
        scope: "selected mode fixture".to_owned(),
        allocation_events: 0,
        allocated_bytes: 0,
        zero_allocation_claim: true,
    }];
    let allocation_bytes = serde_json::to_vec(&allocations).expect("serialize mode allocation fixture");
    std::fs::write(root.join(&profile.resource_observation.allocation_input_path), allocation_bytes)
        .expect("write mode allocation fixture");
    std::fs::write(
        root.join(&profile.resource_observation.llvm_input_path),
        "call void @llvm.memcpy.p0.p0.i64(ptr %a, ptr %b, i64 32, i1 false)\n",
    )
    .expect("write mode LLVM fixture");
}
