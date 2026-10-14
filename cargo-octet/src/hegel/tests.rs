use super::*;

const CASE_BUDGET: u64 = 64;
const COUNTEREXAMPLE_CASES: u64 = 7;
const PROPERTY_ID: &str = "octet.hegel.issue-code-normalization-idempotent";
const SOURCE_SCOPE: &str =
    "cargo-octet/src/hegel/mod.rs,cargo-octet/src/hegel/tests.rs,cargo-octet/src/hegel/validation.rs";
const PASSED_RESULT: &str = "passed";
const EXPECTED_ARTIFACT_COUNT: usize = 5;
const OUTPUT_STAGE_EXTENSION: &str = "octet-stage";
const TEST_FILE_BYTE_LIMIT: u64 = 64;
const MAX_GENERATED_TEXT_BYTES: usize = 32;
const MAX_GENERATED_VECTOR_ITEMS: usize = 16;

type InputMutation = Box<dyn Fn(&mut request::Input, &mut Profile)>;

#[hegel::standalone_function(test_cases = 64, derandomize = true, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
fn issue_code_normalization_is_idempotent(tc: hegel::TestCase) {
    let values = tc.draw(
        hegel::generators::vecs(hegel::generators::text().max_size(MAX_GENERATED_TEXT_BYTES))
            .max_size(MAX_GENERATED_VECTOR_ITEMS),
    );
    let once = normalize_issue_codes(&values);
    let twice = normalize_issue_codes(&once);
    assert_eq!(once, twice);
    assert!(once.windows(2).all(|pair| pair[0] < pair[1]));
    assert!(once.iter().all(|value| !value.is_empty() && value.trim() == value));
}

#[hegel::standalone_function(test_cases = 64, derandomize = true, phases = [hegel::Phase::Generate, hegel::Phase::Shrink])]
fn raw_projection_equivalence_is_rejected(tc: hegel::TestCase) {
    let value = tc.draw(hegel::generators::text().max_size(MAX_GENERATED_TEXT_BYTES));
    tc.assume(!value.trim().is_empty());
    let raw = vec![value.clone(), format!(" {value} ")];
    assert_ne!(raw, normalize_issue_codes(&raw));
}

// r[verify octet.law_tested_deterministic_cores.hegel_catalog]
// r[verify octet.dynamic_evidence.hegel_profiled_run]
#[test]
fn hegel_profile_adapter() {
    issue_code_normalization_is_idempotent();
    raw_projection_equivalence_is_rejected();
    let Some(output_path) = std::env::var_os("OCTET_HEGEL_OUTPUT") else {
        return;
    };
    let summary = producer_summary_from_environment();
    let bytes = pretty_bytes(&summary, "Hegel producer adapter summary").expect("serialize adapter summary");
    let output_path = std::path::PathBuf::from(output_path);
    let stage_path = output_path.with_extension(OUTPUT_STAGE_EXTENSION);
    std::fs::write(&stage_path, bytes).expect("stage adapter summary");
    std::fs::rename(&stage_path, &output_path).expect("publish adapter summary");
}

// r[verify octet.dynamic_evidence.hegel_measured_identity]
// r[verify octet.dynamic_evidence.hegel_hardening_verification]
#[cfg(unix)]
#[test]
fn capability_read_rejects_leaf_and_parent_symlink_substitution() {
    use std::os::unix::fs::symlink;

    let tempdir = tempfile::tempdir().expect("tempdir");
    let outside = tempdir.path().join("outside");
    std::fs::create_dir_all(tempdir.path().join("safe")).expect("safe directory");
    std::fs::create_dir_all(&outside).expect("outside directory");
    std::fs::write(tempdir.path().join("safe/input.json"), b"safe").expect("safe input");
    std::fs::write(outside.join("input.json"), b"outside").expect("outside input");
    symlink(outside.join("input.json"), tempdir.path().join("leaf.json")).expect("leaf symlink");
    symlink(&outside, tempdir.path().join("linked-dir")).expect("directory symlink");

    let valid =
        safe_read::read_checked_input(tempdir.path(), std::path::Path::new("safe/input.json"), TEST_FILE_BYTE_LIMIT)
            .expect("safe input");
    assert_eq!(valid, b"safe");
    for locator in ["leaf.json", "linked-dir/input.json"] {
        let error = safe_read::read_checked_input(tempdir.path(), std::path::Path::new(locator), TEST_FILE_BYTE_LIMIT)
            .expect_err("symlink substitution rejected");
        assert!(error.contains("symlink"));
    }
}

// r[verify octet.dynamic_evidence.hegel_output_contract]
#[test]
fn bounded_shell_rejects_mixed_process_output_without_dedicated_summary() {
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root");
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let mut tools = shell::production_tools().expect("Nix Hegel tools");
    tools.cargo = find_path_executable("true").expect("true executable");

    let error = shell::run_with(workspace, &profile, tools).expect_err("missing dedicated summary rejected");
    assert!(error.contains("summary.json"));
}

// r[verify octet.dynamic_evidence.hegel_profiled_run]
// r[verify octet.dynamic_evidence.hegel_output_contract]
// r[verify octet.dynamic_evidence.hegel_measured_identity]
#[test]
fn profiled_summary_binds_measured_inputs_and_replays_artifacts() {
    let (input, profile) = valid_input();
    assert_eq!(prefixed_identity(include_bytes!("../../../standards/hegel/profiles.json")), PROFILE_IDENTITY);
    assert_eq!(prefixed_identity(include_bytes!("../../../standards/hegel/laws.json")), LAW_CATALOG_IDENTITY);
    let text = request::render(&input).expect("render profiled input");
    let evaluation = request::evaluate(&text, PROFILE_ID);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);
    let details = evaluation.details.expect("profiled details");
    assert!(validate(&details).is_empty());
    let artifacts = request::artifact_payloads(&details, &text).expect("artifact replay");
    assert_eq!(artifacts.len(), EXPECTED_ARTIFACT_COUNT);
    assert_eq!(details.command_argv, profile.command_argv);

    let command = profile.command_argv.join(" ");
    let mut receipt = crate::evidence_rails::evaluate(crate::evidence_rails::EvaluationInput {
        rail: crate::evidence_rails::EvidenceRailKind::HegelProperty,
        mode: "run",
        import_text: &text,
        workspace_scope: ".",
        config_path: Some("standards/hegel/profiles.json"),
        baseline_identity: None,
        lockfile_identity: Some(input.cargo_lock_identity.clone()),
        locked: true,
        advisory_updated_by_prior_step: false,
        profile: Some(PROFILE_ID),
        requested_command: Some(&command),
        parent_artifacts: Vec::new(),
        tool_exit_code: Some(0),
    });
    receipt.config_identity = Some(PROFILE_IDENTITY.to_owned());
    assert!(crate::evidence_rails::validate_receipt(&receipt).is_empty());
    assert!(receipt.hegel_property.and_then(|value| value.hardened).is_some());
}

// r[verify octet.dynamic_evidence.hegel_hardening_verification]
#[test]
fn profiled_summary_rejects_authority_drift_stale_identities_limits_and_overclaims() {
    let (input, profile) = valid_input();
    let mutations: Vec<InputMutation> = vec![
        Box::new(|_, profile| profile.command_argv.push("--ignored".to_owned())),
        Box::new(|input, _| input.summary_text = "not-json".to_owned()),
        Box::new(|input, _| input.cargo_lock_identity = prefixed_identity(b"stale-lock")),
        Box::new(|input, _| input.source_manifest[0].locator = "../escape.rs".to_owned()),
        Box::new(|_input, profile| profile.limits.max_summary_bytes = 1),
        Box::new(|input, profile| {
            profile.limits.max_log_bytes = 1;
            input.bounded_log = b"too-long".to_vec();
        }),
        Box::new(|_input, profile| profile.limits.max_counterexample_bytes = 0),
        Box::new(|_input, profile| profile.limits.max_source_files = 0),
        Box::new(|_input, profile| profile.limits.max_generator_files = 0),
        Box::new(|_input, profile| profile.limits.max_issue_codes = 0),
        Box::new(|_input, profile| profile.limits.timeout_ms = 0),
        Box::new(|_input, profile| profile.limits.poll_interval_ms = 0),
        Box::new(|_input, profile| profile.limits.teardown_timeout_ms = 0),
    ];
    for mutate in mutations {
        let mut candidate = input.clone();
        let mut selected = profile.clone();
        mutate(&mut candidate, &mut selected);
        assert!(validation::evaluate(&candidate, &selected).details.is_none());
    }

    let mut promoted =
        request::evaluate(&request::render(&input).expect("render"), PROFILE_ID).details.expect("details");
    promoted.evidence_tier = "formal_proof".to_owned();
    assert!(validate(&promoted).iter().any(|diagnostic| diagnostic.contains("boundary")));
}

#[test]
fn counterexample_contract_requires_matching_retained_bytes() {
    let (mut input, profile) = valid_input();
    let counterexample =
        include_bytes!("../../tests/fixtures/evidence-rails/hegel-hardened-counterexample.json").to_vec();
    let mut summary: ProducerSummary = serde_json::from_str(&input.summary_text).expect("summary");
    summary.result = "counterexample".to_owned();
    summary.cases_completed = COUNTEREXAMPLE_CASES;
    summary.issue_codes = vec!["duplicate".to_owned()];
    summary.counterexample_identity = Some(prefixed_identity(&counterexample));
    input.counterexample = Some(counterexample);
    input.summary_text = String::from_utf8(pretty_bytes(&summary, "counterexample summary").expect("summary bytes"))
        .expect("summary UTF-8");
    let evaluation = validation::evaluate(&input, &profile);
    assert!(evaluation.diagnostics.is_empty(), "{:?}", evaluation.diagnostics);

    let mut stale = input;
    stale.counterexample = Some(b"substituted".to_vec());
    assert!(validation::evaluate(&stale, &profile).details.is_none());
}

fn valid_input() -> (request::Input, Profile) {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    let source_manifest = vec![
        file_identity(&profile.source_scope[0], include_bytes!("mod.rs")),
        file_identity(&profile.source_scope[1], include_bytes!("tests.rs")),
        file_identity(&profile.source_scope[2], include_bytes!("validation.rs")),
    ];
    let generator_manifest = vec![file_identity(
        &profile.generator_inputs[0],
        include_bytes!("../../../standards/hegel/laws.ncl"),
    )];
    let cargo_lock_identity = prefixed_identity(include_bytes!("../../../Cargo.lock"));
    let package_manifest_identity = prefixed_identity(include_bytes!("../../Cargo.toml"));
    let source_manifest_identity = request::manifest_identity(&source_manifest, "source manifest").expect("source");
    let generator_manifest_identity =
        request::manifest_identity(&generator_manifest, "generator manifest").expect("generator");
    let summary = ProducerSummary {
        schema_version: SUMMARY_SCHEMA.to_owned(),
        profile_id: PROFILE_ID.to_owned(),
        profile_identity: PROFILE_IDENTITY.to_owned(),
        package: profile.producer.package.clone(),
        package_version: profile.producer.package_version.clone(),
        hegel_version: profile.producer.hegel_version.clone(),
        backend: profile.producer.backend.clone(),
        property_catalog: profile.property_catalog.clone(),
        property_id: PROPERTY_ID.to_owned(),
        linked_ids: vec!["octet.dynamic_evidence.hegel_output_contract".to_owned()],
        source_scope: SOURCE_SCOPE.to_owned(),
        settings: profile.settings.clone(),
        generator: GeneratorSummary {
            family: "bounded-unicode-issue-code-vectors".to_owned(),
            content_hash: LAW_CATALOG_IDENTITY.to_owned(),
        },
        source_manifest_identity,
        generator_manifest_identity,
        cargo_lock_identity: cargo_lock_identity.clone(),
        package_manifest_identity: package_manifest_identity.clone(),
        result: PASSED_RESULT.to_owned(),
        cases_completed: CASE_BUDGET,
        issue_codes: Vec::new(),
        counterexample_identity: None,
    };
    let summary_text = String::from_utf8(pretty_bytes(&summary, "producer summary").expect("summary")).expect("UTF-8");
    (
        request::Input {
            schema_version: REQUEST_SCHEMA.to_owned(),
            summary_text,
            bounded_log: b"bounded Hegel log".to_vec(),
            source_manifest,
            generator_manifest,
            cargo_lock_identity,
            package_manifest_identity,
            counterexample: None,
        },
        profile,
    )
}

fn file_identity(locator: &str, bytes: &[u8]) -> FileIdentity {
    FileIdentity {
        locator: locator.to_owned(),
        identity: prefixed_identity(bytes),
        bytes: u64::try_from(bytes.len()).expect("fixture byte length"),
    }
}

fn producer_summary_from_environment() -> ProducerSummary {
    let profile = selected_profile(PROFILE_ID).expect("profile");
    ProducerSummary {
        schema_version: SUMMARY_SCHEMA.to_owned(),
        profile_id: PROFILE_ID.to_owned(),
        profile_identity: required_environment("OCTET_HEGEL_PROFILE_IDENTITY"),
        package: profile.producer.package.clone(),
        package_version: profile.producer.package_version.clone(),
        hegel_version: profile.producer.hegel_version.clone(),
        backend: profile.producer.backend.clone(),
        property_catalog: profile.property_catalog.clone(),
        property_id: PROPERTY_ID.to_owned(),
        linked_ids: vec!["octet.dynamic_evidence.hegel_output_contract".to_owned()],
        source_scope: SOURCE_SCOPE.to_owned(),
        settings: profile.settings,
        generator: GeneratorSummary {
            family: "bounded-unicode-issue-code-vectors".to_owned(),
            content_hash: LAW_CATALOG_IDENTITY.to_owned(),
        },
        source_manifest_identity: required_environment("OCTET_HEGEL_SOURCE_IDENTITY"),
        generator_manifest_identity: required_environment("OCTET_HEGEL_GENERATOR_IDENTITY"),
        cargo_lock_identity: required_environment("OCTET_HEGEL_LOCK_IDENTITY"),
        package_manifest_identity: required_environment("OCTET_HEGEL_PACKAGE_IDENTITY"),
        result: PASSED_RESULT.to_owned(),
        cases_completed: CASE_BUDGET,
        issue_codes: Vec::new(),
        counterexample_identity: None,
    }
}

fn required_environment(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("missing {name}"))
}

fn find_path_executable(name: &str) -> Option<std::path::PathBuf> {
    std::env::var_os("PATH").and_then(|path| {
        std::env::split_paths(&path)
            .map(|directory| directory.join(name))
            .find(|candidate| candidate.is_file())
    })
}
