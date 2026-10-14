use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

mod support {
    pub mod consumer_fixture;
}

use support::consumer_fixture::ConsumerFixture;
use support::consumer_fixture::TempDir;
use support::consumer_fixture::bash_path;
use support::consumer_fixture::build_standalone_cargo_octet;
use support::consumer_fixture::built_cargo_octet_bin;
use support::consumer_fixture::expected_repo_toolchain;
use support::consumer_fixture::metadata;
use support::consumer_fixture::write_executable;
use support::consumer_fixture::write_file;

#[test]
fn cargo_style_argv_prefix_parses_like_bare_check() {
    let fixture = ConsumerFixture::new("cargo-style-argv", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_raw_os(
        vec![OsString::from("tigerstyle"), OsString::from("check")],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-style argv prefix should behave like bare check: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("ARGS:check"), "cargo-style argv prefix should still invoke cargo check, got: {log}");
    assert!(
        log.contains("-p consumer"),
        "cargo-style argv prefix should preserve metadata default scope, got: {log}"
    );
    assert!(
        log.contains("--lib"),
        "cargo-style argv prefix should preserve metadata cargo_check_args, got: {log}"
    );
}

#[test]
fn no_argument_check_uses_workspace_metadata_defaults() {
    let fixture = ConsumerFixture::new("metadata-defaults", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-octet should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("ARGS:check"), "expected cargo check invocation, got: {log}");
    assert!(log.contains("-p consumer"), "expected metadata default scope, got: {log}");
    assert!(log.contains("--lib"), "expected metadata cargo_check_args, got: {log}");
    assert!(log.contains("DYLINT_NO_DEPS=1"), "expected DYLINT_NO_DEPS=1, got: {log}");
}

#[test]
fn known_strict_profile_is_handed_to_cargo_and_unknown_profile_fails_before_cargo() {
    let selected = ConsumerFixture::new("strict-profile-selected", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let selected_output = selected.run_with_launcher_and_env(
        None,
        &[("OCTET_STRICT_PROFILE", "deterministic-core")],
        &["--profile", "robust"],
        EXIT_CODE_SUCCESS,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(selected.driver_path.as_path()),
        Some(selected.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        selected_output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "known profile should run: stdout={} stderr={}",
        String::from_utf8_lossy(&selected_output.stdout),
        String::from_utf8_lossy(&selected_output.stderr)
    );
    assert!(selected.log().contains("OCTET_STRICT_PROFILE=robust"));
    assert!(String::from_utf8_lossy(&selected_output.stderr).contains("Profile: robust"));

    let unknown = ConsumerFixture::new("strict-profile-unknown", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let unknown_output = unknown.run(
        &["--profile", "not-a-profile"],
        EXIT_CODE_SUCCESS,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(unknown.driver_path.as_path()),
        Some(unknown.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(unknown_output.status.code(), Some(EXIT_CODE_INTEGRATION_FAILURE));
    let unknown_stderr = String::from_utf8_lossy(&unknown_output.stderr);
    assert!(unknown_stderr.contains("unknown strict profile `not-a-profile`"));
    assert!(unknown_stderr.contains("bounded-systems, deterministic-core, robust"));
    assert!(!unknown.log().contains("ARGS:check"), "unknown profile must fail before Cargo");
}

#[test]
fn deterministic_core_requires_scopes_and_default_check_scrubs_ambient_profile_selection() {
    let missing_scope =
        ConsumerFixture::new("strict-profile-missing-core", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let missing_output = missing_scope.run(
        &["--profile", "deterministic-core"],
        EXIT_CODE_SUCCESS,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(missing_scope.driver_path.as_path()),
        Some(missing_scope.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(missing_output.status.code(), Some(EXIT_CODE_INTEGRATION_FAILURE));
    let missing_stderr = String::from_utf8_lossy(&missing_output.stderr);
    assert!(missing_stderr.contains("deterministic-core"));
    assert!(missing_stderr.contains("core_source_scopes"));
    assert!(!missing_scope.log().contains("ARGS:check"), "missing scope must fail before Cargo");

    let default = ConsumerFixture::new("strict-profile-env-scrub", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let default_output = default.run_with_launcher_and_env(
        None,
        &[("OCTET_STRICT_PROFILE", "deterministic-core")],
        &[],
        EXIT_CODE_SUCCESS,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(default.driver_path.as_path()),
        Some(default.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(default_output.status.code(), Some(EXIT_CODE_SUCCESS));
    assert!(default.log().contains("OCTET_STRICT_PROFILE=\n"));
    assert!(String::from_utf8_lossy(&default_output.stderr).contains("Profile: workspace-metadata"));
}

#[test]
fn deterministic_core_uses_reviewed_core_scope_function_address_baseline() {
    let fixture = ConsumerFixture::new(
        "strict-profile-deterministic-baseline",
        &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#),
    );
    write_file(&fixture.repo.join("dylint.toml"), "[octet]\ncore_source_scopes = [\"src\"]\n");
    let observed_dir = artifact_dir(&fixture, "deterministic-observe");
    let observed = run_with_check_args(
        &fixture,
        &[
            "--function-addresses",
            "observe",
            "--function-address-path",
            "src",
            "--artifact-dir",
            observed_dir.to_str().expect("observe artifact dir utf8"),
        ],
        &[cargo_build_finished(true)],
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(
        observed.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "observe bootstrap should pass: stdout={} stderr={}",
        String::from_utf8_lossy(&observed.stdout),
        String::from_utf8_lossy(&observed.stderr)
    );

    let baseline = observed_dir.join("function-addresses.json");
    let admitted_dir = artifact_dir(&fixture, "deterministic-admitted");
    let admitted = run_with_check_args(
        &fixture,
        &[
            "--profile",
            "deterministic-core",
            "--function-address-baseline",
            baseline.to_str().expect("baseline path utf8"),
            "--artifact-dir",
            admitted_dir.to_str().expect("admitted artifact dir utf8"),
        ],
        &[cargo_build_finished(true)],
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(
        admitted.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "reviewed deterministic core should pass: stdout={} stderr={}",
        String::from_utf8_lossy(&admitted.stdout),
        String::from_utf8_lossy(&admitted.stderr)
    );
    let status = read_json_artifact(&admitted_dir.join(STATUS_JSON_FILE));
    assert_eq!(status["metadata"]["profile_name"], "deterministic-core");
    let report = read_json_artifact(&admitted_dir.join("function-addresses.json"));
    assert_eq!(report["mode"], "deny");
    assert_eq!(report["source_envelopes"][0]["path"], "src/lib.rs");
    assert!(fixture.log().contains("OCTET_STRICT_PROFILE=deterministic-core"));
}

// r[verify octet.effect_closure.cli.check_failure]
#[test]
fn effect_closure_admission_stops_after_failed_direct_check() {
    let fixture =
        ConsumerFixture::new("effect-closure-blocked-check", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    write_file(&fixture.repo.join("dylint.toml"), "[octet]\ncore_source_scopes = [\"src/lib.rs\"]\n");
    write_file(&fixture.repo.join("function-addresses.json"), "{}\n");
    write_file(
        &fixture.repo.join("octet-effect-closure.json"),
        r#"{
  "schema_version": "octet-effect-closure-workspace-policy/v1",
  "profile_id": "deterministic-core",
  "function_address_baseline": "function-addresses.json",
  "module_roles": [{ "path": "src/lib.rs", "role": "core" }],
  "call_summaries": [],
  "non_claims": ["selected source-token closure only"]
}
"#,
    );
    let artifacts = artifact_dir(&fixture, "effect-closure-blocked");
    let output = fixture.run_raw_os(
        [
            "admit",
            "--profile",
            "deterministic-core",
            "--closure",
            "--artifact-dir",
            artifacts.to_str().expect("artifact path utf8"),
        ]
        .into_iter()
        .map(OsString::from)
        .collect(),
        EXIT_CODE_LINT_FAILURE,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );

    assert!(!output.status.success(), "failed direct check must deny admission");
    let status = read_json_artifact(&artifacts.join(STATUS_JSON_FILE));
    assert_eq!(status["status"], "denied");
    assert_eq!(status["phase"], "direct-check");
    assert!(!artifacts.join("effect-closure.json").exists());
    assert!(fixture.log().contains("ARGS:check"));
    assert!(!fixture.log().contains("ARGS:metadata"));
}

#[test]
fn no_argument_check_from_nested_workspace_subdirectory_uses_workspace_metadata_defaults() {
    let fixture = ConsumerFixture::new("metadata-defaults-subdir", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let nested_dir = fixture.repo.join("nested/deeper");
    fs::create_dir_all(&nested_dir).unwrap_or_else(|error| panic!("create {}: {error}", nested_dir.display()));

    let output = fixture.run_from_dir_with_launcher_and_env(
        &nested_dir,
        None,
        &[],
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-octet from a nested workspace subdirectory should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("ARGS:check"), "expected cargo check invocation, got: {log}");
    assert!(log.contains("-p consumer"), "expected metadata default scope from nested dir, got: {log}");
    assert!(log.contains("--lib"), "expected metadata cargo_check_args from nested dir, got: {log}");
    assert!(log.contains("DYLINT_NO_DEPS=1"), "expected DYLINT_NO_DEPS=1 from nested dir, got: {log}");
}

#[test]
fn temp_workspace_launch_uses_built_binary_and_preserves_consumer_cwd() {
    let fixture = ConsumerFixture::new("temp-workspace-launch", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let nested_dir = fixture.repo.join("nested/deeper");
    fs::create_dir_all(&nested_dir).unwrap_or_else(|error| panic!("create {}: {error}", nested_dir.display()));
    let launcher = built_cargo_octet_bin().expect("build cargo-octet binary");

    let output = fixture.run_from_dir_with_launcher_and_env(
        &nested_dir,
        Some(launcher.as_path()),
        &[],
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "built-binary launcher from a nested consumer subdirectory should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(
        log.contains(&format!("LAUNCHER:{}", launcher.display())),
        "expected explicit built-binary launcher path in log, got: {log}"
    );
    assert!(log.contains("ARGS:check"), "expected cargo check invocation, got: {log}");
    assert!(log.contains("-p consumer"), "expected metadata default scope from nested dir, got: {log}");
    assert!(log.contains("--lib"), "expected metadata cargo_check_args from nested dir, got: {log}");
}

#[test]
fn explicit_workspace_scope_override_beats_metadata_defaults() {
    let fixture = ConsumerFixture::new("workspace-override", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--workspace"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-octet should succeed with explicit --workspace: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("--workspace"), "expected explicit --workspace scope override, got: {log}");
    assert!(!log.contains("-p consumer"), "explicit --workspace should replace metadata default scope: {log}");
    assert!(
        log.contains("--lib"),
        "metadata cargo_check_args should still apply without explicit cargo tail args: {log}"
    );
}

#[test]
fn repeated_package_scope_flags_forward_in_order_and_replace_metadata_defaults() {
    let fixture =
        ConsumerFixture::new("repeated-package-overrides", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--package", "consumer", "-p", "other"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-octet should succeed with repeated package scope flags: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    let consumer_pos = log.find("--package consumer").expect("expected --package consumer in cargo log");
    let other_pos = log.find("-p other").expect("expected -p other in cargo log");
    assert!(consumer_pos < other_pos, "repeated package scope flags should preserve order: {log}");
    assert!(
        !log.contains("-p consumer"),
        "repeated package scope flags should replace metadata default scope: {log}"
    );
    assert!(
        log.contains("--lib"),
        "metadata cargo_check_args should still apply without explicit cargo tail args: {log}"
    );
}

#[test]
fn bare_double_dash_replaces_metadata_cargo_check_args_with_nothing() {
    let fixture = ConsumerFixture::new("bare-double-dash", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-octet should succeed with a bare -- override: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("-p consumer"), "bare -- should keep metadata default scope: {log}");
    assert!(
        !log.contains("--lib"),
        "bare -- should replace metadata cargo_check_args with an empty explicit tail: {log}"
    );
}

#[test]
fn explicit_workspace_scope_with_cargo_tail_replaces_metadata_scope_and_args() {
    let fixture = ConsumerFixture::new("workspace-tail-overrides", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--workspace", "--", "--all-targets"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-octet should succeed with explicit --workspace plus cargo tail: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("--workspace"), "expected explicit --workspace scope override, got: {log}");
    assert!(log.contains("--all-targets"), "expected explicit cargo tail args, got: {log}");
    assert!(!log.contains("-p consumer"), "explicit --workspace should replace metadata default scope: {log}");
    assert!(
        !log.contains("--lib"),
        "explicit cargo tail args should replace metadata defaults even with --workspace: {log}"
    );
}

#[test]
fn explicit_scope_and_check_args_override_metadata_defaults() {
    let fixture = ConsumerFixture::new("explicit-overrides", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["-p", "override-crate", "--", "--all-targets"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "cargo-octet should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("-p override-crate"), "expected explicit scope override, got: {log}");
    assert!(log.contains("--all-targets"), "expected explicit cargo tail args, got: {log}");
    assert!(!log.contains("--lib"), "explicit cargo tail args should replace metadata defaults: {log}");
}

#[test]
fn resolves_toolchain_from_repo_when_env_is_missing() {
    let fixture = ConsumerFixture::new("toolchain-fallback", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert!(
        output.status.success(),
        "toolchain fallback should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    let expected = expected_repo_toolchain();
    assert!(
        log.contains(&format!("RUSTUP_TOOLCHAIN={expected}")),
        "expected derived toolchain `{expected}` in cargo env log, got: {log}"
    );
}

#[test]
fn bootstraps_driver_when_explicit_path_is_missing() {
    let fixture = ConsumerFixture::new("driver-bootstrap", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        None,
        Some(fixture.lint_lib_path.as_path()),
        Some("test-rev"),
    );
    let log = fixture.log();
    assert!(
        output.status.success(),
        "driver bootstrap should succeed: stdout={} stderr={} log={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
        log,
    );

    let build_pos = log.find("ARGS:build").expect("saw bootstrap build in cargo log");
    let check_pos = log.find("ARGS:check").expect("saw consumer cargo check in cargo log");
    assert!(build_pos < check_pos, "bootstrap build should happen before cargo check: {log}");
    assert!(
        !log.contains(&format!("RUSTC_WORKSPACE_WRAPPER={}\n", fixture.driver_path.display())),
        "bootstrap path should not reuse the explicit fixture driver path: {log}"
    );
}

#[test]
fn invalid_metadata_exits_with_integration_failure_code() {
    let fixture = ConsumerFixture::new(
        "invalid-metadata",
        r#"[workspace.metadata.tigerstyle]
default_scope = ["-p", "consumer"]
"#,
    );

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "invalid metadata should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "invalid metadata should fail before invoking cargo: {log}");
}

#[test]
fn missing_metadata_block_exits_with_integration_failure_code() {
    let fixture = ConsumerFixture::new("missing-metadata", "");

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "missing workspace.metadata.octet should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "missing metadata should fail before invoking cargo: {log}");
}

#[test]
fn non_string_metadata_array_entries_exit_with_integration_failure_code() {
    let fixture = ConsumerFixture::new(
        "non-string-metadata",
        r#"[workspace.metadata.tigerstyle]
default_scope = ["-p", 7]
cargo_check_args = ["--lib"]
"#,
    );

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "non-string metadata entries should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "non-string metadata should fail before invoking cargo: {log}");
}

#[test]
fn equals_sign_package_form_is_rejected_as_unsupported() {
    let fixture = ConsumerFixture::new("equals-sign-package", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--package=consumer"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "equals-sign package form should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: unsupported argument before `--`: `--package=consumer`"),
        "equals-sign package form should preserve diagnostic stderr: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "equals-sign package form should fail before invoking cargo: {log}");
}

#[test]
fn unsupported_arguments_before_double_dash_exit_with_integration_failure_code() {
    let fixture = ConsumerFixture::new("unsupported-arg", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--all-targets"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "unsupported arguments before `--` should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "unsupported arguments should fail before invoking cargo: {log}");
}

#[test]
fn unsupported_arguments_before_double_dash_preserve_diagnostic_stderr() {
    let fixture = ConsumerFixture::new("unsupported-arg-stderr", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &["--all-targets"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: unsupported argument before `--`: `--all-targets`"),
        "unsupported-argument parse failure should preserve diagnostic stderr: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
}

#[test]
fn non_utf8_cli_arguments_exit_with_integration_failure_code_and_preserve_diagnostic() {
    let fixture = ConsumerFixture::new("non-utf8-arg", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_raw_os(
        vec![OsString::from("check"), OsString::from_vec(vec![b'f', 0x80, b'o'])],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "non-utf8 CLI arguments should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: non-utf8 arguments are not supported"),
        "non-utf8 CLI arguments should preserve diagnostic stderr: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    let log = fixture.log();
    assert!(!log.contains("ARGS:"), "non-utf8 CLI arguments should fail before invoking cargo: {log}");
}

#[test]
fn missing_and_unknown_subcommands_preserve_diagnostics_and_skip_cargo() {
    const EXPECTED_SUBCOMMANDS: &str = "expected `admit`, `artifact`, `check`, `evidence`, `evidence-index`, `fingerprint`, `migrate`, `object`, `provider-manifest`, `prove`, `proof-ref`, `roadmap`, or `transcript`";

    let assert_subcommand_failure = |label: &str, args: Vec<OsString>, expected_stderr: &str| {
        let fixture = ConsumerFixture::new(label, &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
        let output = fixture.run_raw_os(
            args,
            0,
            false,
            Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
            Some(fixture.driver_path.as_path()),
            Some(fixture.lint_lib_path.as_path()),
            None,
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "{label} should exit 2 for CLI misuse: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_stderr),
            "{label} should preserve exact diagnostic stderr: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
        let log = fixture.log();
        assert!(!log.contains("ARGS:"), "{label} should fail before invoking cargo: {log}");
    };

    assert_subcommand_failure(
        "missing-subcommand",
        vec![],
        &format!("error: missing subcommand; {EXPECTED_SUBCOMMANDS}"),
    );
    assert_subcommand_failure(
        "unknown-subcommand",
        vec![OsString::from("lint")],
        &format!("error: unknown subcommand `lint`; {EXPECTED_SUBCOMMANDS}"),
    );
}

#[test]
fn missing_package_value_after_scope_flag_exits_with_integration_failure_code_and_preserves_diagnostic() {
    let assert_missing_value_failure = |label: &str, args: &[&str], expected_stderr: &str| {
        let fixture = ConsumerFixture::new(label, &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
        let output = fixture.run(
            args,
            0,
            false,
            Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
            Some(fixture.driver_path.as_path()),
            Some(fixture.lint_lib_path.as_path()),
            None,
        );
        assert_eq!(
            output.status.code(),
            Some(2),
            "{label} should exit 2 for a dangling scope flag: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(expected_stderr),
            "{label} should preserve exact diagnostic stderr: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            stderr
        );
        let log = fixture.log();
        assert!(!log.contains("ARGS:"), "{label} should fail before invoking cargo: {log}");
    };

    assert_missing_value_failure("missing-package-short", &["-p"], "error: missing value after `-p`");
    assert_missing_value_failure("missing-package-long", &["--package"], "error: missing value after `--package`");
}

#[test]
fn cargo_termination_without_exit_code_exits_two_and_restores_lockfile() {
    let fixture = ConsumerFixture::new("signal-terminated-cargo", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_with_launcher_and_env(
        None,
        &[("TS_KILL_SELF", "1")],
        &[],
        0,
        true,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "signal-terminated cargo should exit 2 from cargo-octet: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: cargo check terminated without an exit code"),
        "signal-terminated cargo should preserve the no-exit-code diagnostic: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert!(
        stderr.contains("cargo octet check modified"),
        "signal-terminated cargo should also report fail-closed lockfile restoration: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert_eq!(
        fixture.lockfile(),
        fixture.original_lockfile,
        "signal-terminated cargo should restore the consumer lockfile"
    );
}

#[test]
fn cargo_termination_without_exit_code_and_without_lockfile_mutation_reports_only_root_cause() {
    let fixture =
        ConsumerFixture::new("signal-terminated-clean-lockfile", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_with_launcher_and_env(
        None,
        &[("TS_KILL_SELF", "1")],
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "clean signal-terminated cargo should exit 2 from cargo-octet: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("error: cargo check terminated without an exit code"),
        "clean signal-terminated cargo should preserve the no-exit-code diagnostic: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert!(
        !stderr.contains("cargo octet check modified"),
        "clean signal-terminated cargo should not claim lockfile restoration when no mutation happened: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        stderr
    );
    assert_eq!(
        fixture.lockfile(),
        fixture.original_lockfile,
        "clean signal-terminated cargo should leave the consumer lockfile unchanged"
    );
}

#[test]
fn explicit_empty_tigerstyle_env_vars_exit_with_integration_failure_code() {
    let assert_empty_env_failure = |label: &str,
                                    toolchain_env: Option<&str>,
                                    driver_env: Option<&Path>,
                                    lint_lib_env: Option<&Path>,
                                    dylint_rev_env: Option<&str>| {
        let fixture = ConsumerFixture::new(label, &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
        let output = fixture.run(&[], 0, false, toolchain_env, driver_env, lint_lib_env, dylint_rev_env);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{label} should exit 2 for an explicit empty TIGERSTYLE env var: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let log = fixture.log();
        assert!(!log.contains("ARGS:"), "{label} should fail before invoking cargo: {log}");
    };

    let toolchain_fixture =
        ConsumerFixture::new("empty-toolchain-fixture", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    assert_empty_env_failure(
        "empty-toolchain",
        Some(""),
        Some(toolchain_fixture.driver_path.as_path()),
        Some(toolchain_fixture.lint_lib_path.as_path()),
        None,
    );

    let driver_fixture =
        ConsumerFixture::new("empty-driver-fixture", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    assert_empty_env_failure(
        "empty-driver-path",
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(Path::new("")),
        Some(driver_fixture.lint_lib_path.as_path()),
        None,
    );

    let lint_fixture =
        ConsumerFixture::new("empty-lint-lib-fixture", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    assert_empty_env_failure(
        "empty-lint-lib",
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(lint_fixture.driver_path.as_path()),
        Some(Path::new("")),
        None,
    );

    let dylint_rev_fixture =
        ConsumerFixture::new("empty-dylint-rev-fixture", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    assert_empty_env_failure(
        "empty-dylint-rev",
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        None,
        Some(dylint_rev_fixture.lint_lib_path.as_path()),
        Some(""),
    );
}

#[test]
fn error_level_lint_findings_exit_with_code_one() {
    let fixture = ConsumerFixture::new("lint-failure", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![
        cargo_compiler_message_with_level(
            "consumer",
            "no_unwrap",
            ERROR_LEVEL,
            "use of `.unwrap()`",
            "src/lib.rs",
            10,
            5,
            None,
        ),
        cargo_build_finished(true),
    ];

    let output = run_with_output_format(&fixture, JSON_OUTPUT_FORMAT, &lines, EXIT_CODE_SUCCESS);
    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_LINT_FAILURE),
        "lint failures should exit 1: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(&format!("Status: {STATUS_LINT_FAILURE}")),
        "lint failure summary should report lint-failure: {stderr}"
    );
    assert!(stderr.contains("Errors: 1"), "lint failure summary should count one error: {stderr}");
    assert_eq!(fixture.lockfile(), fixture.original_lockfile, "lint failure should not dirty lockfile");
}

#[test]
fn warning_level_lint_findings_exit_with_code_zero() {
    let fixture = ConsumerFixture::new("lint-warning", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![
        cargo_compiler_message("consumer", "no_unwrap", "use of `.unwrap()`", "src/lib.rs", 10, 5, None),
        cargo_build_finished(true),
    ];

    let output = run_with_output_format(&fixture, JSON_OUTPUT_FORMAT, &lines, EXIT_CODE_SUCCESS);
    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "warning-only lint findings should stay non-fatal: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let findings: Vec<serde_json::Value> =
        stdout.lines().map(|line| serde_json::from_str(line).expect("parse warning finding")).collect();
    assert_eq!(findings.len(), 1, "warning-only run should emit exactly one finding: {stdout}");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Status: warning-only"), "warning-only summary should report warning-only: {stderr}");
    assert!(stderr.contains("Warnings: 1"), "warning-only summary should count one warning: {stderr}");
    assert_eq!(fixture.lockfile(), fixture.original_lockfile, "warning-only lint finding should not dirty lockfile");
}

#[test]
fn lockfile_mutation_restores_original_and_exits_two() {
    let fixture = ConsumerFixture::new("lockfile-restore", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        true,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "lockfile mutation should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(fixture.lockfile(), fixture.original_lockfile, "lockfile should be restored after mutation");
}

#[test]
fn missing_lockfile_mutation_removes_generated_lockfile_and_exits_two() {
    let fixture = ConsumerFixture::new("missing-lockfile-restore", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    fs::remove_file(fixture.repo.join("Cargo.lock")).expect("remove consumer Cargo.lock before test");
    assert!(
        !fixture.repo.join("Cargo.lock").exists(),
        "consumer Cargo.lock should start missing for the missing-lockfile contract"
    );

    let output = fixture.run(
        &[],
        0,
        true,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "missing-lockfile mutation should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !fixture.repo.join("Cargo.lock").exists(),
        "generated consumer Cargo.lock should be removed again after fail-closed restoration"
    );
}

#[test]
fn missing_lockfile_clean_success_preserves_absence() {
    let fixture = ConsumerFixture::new("missing-lockfile-success", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    fs::remove_file(fixture.repo.join("Cargo.lock")).expect("remove consumer Cargo.lock before clean-success test");
    assert!(
        !fixture.repo.join("Cargo.lock").exists(),
        "consumer Cargo.lock should start missing for the clean-success missing-lockfile contract"
    );

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        None,
        Some(fixture.lint_lib_path.as_path()),
        Some("test-rev"),
    );
    assert!(
        output.status.success(),
        "missing-lockfile clean success should stay green: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !fixture.repo.join("Cargo.lock").exists(),
        "clean successful run should preserve the absence of consumer Cargo.lock"
    );
}

#[test]
fn bootstrap_lockfile_mutation_restores_original_and_exits_two() {
    let fixture =
        ConsumerFixture::new("bootstrap-lockfile-restore", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_with_bootstrap_lockfile_mutation(
        &[],
        0,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        None,
        Some(fixture.lint_lib_path.as_path()),
        Some("test-rev"),
    );
    assert_eq!(
        output.status.code(),
        Some(2),
        "bootstrap-phase lockfile mutation should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        fixture.lockfile(),
        fixture.original_lockfile,
        "bootstrap mutation should restore the consumer lockfile"
    );
}

#[test]
fn bootstraps_lint_library_when_explicit_path_is_missing() {
    let fixture = ConsumerFixture::new("lint-lib-bootstrap", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run(
        &[],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        output.status.success(),
        "lint-library bootstrap should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    let build_pos = log.find("ARGS:build").expect("saw lint-library build in cargo log");
    let check_pos = log.find("ARGS:check").expect("saw consumer cargo check in cargo log");
    assert!(build_pos < check_pos, "lint-library build should happen before cargo check: {log}");
    assert!(
        !log.contains(&format!("DYLINT_LIBS=[\"{}\"]\n", fixture.lint_lib_path.display())),
        "lint-library bootstrap should not reuse the explicit fixture lint path: {log}"
    );
    assert!(
        log.contains("liboctet@nightly-2026-03-21-x86_64-unknown-linux-gnu.so"),
        "bootstrapped lint library should use the toolchain-suffixed dylint path: {log}"
    );
}

fn first_embedded_source_dir(cache_home: &Path) -> PathBuf {
    let sources_root = cache_home.join("octet/sources");
    let entries = fs::read_dir(&sources_root)
        .unwrap_or_else(|error| panic!("read embedded source cache {}: {error}", sources_root.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    assert_eq!(entries.len(), 1, "expected exactly one embedded source dir under {}", sources_root.display());
    entries.into_iter().next().expect("embedded source dir")
}

fn set_tree_permissions(root: &Path, dir_mode: u32, file_mode: u32) {
    let metadata = fs::metadata(root).unwrap_or_else(|error| panic!("metadata {}: {error}", root.display()));
    let mode = if metadata.is_dir() { dir_mode } else { file_mode };
    let mut perms = metadata.permissions();
    perms.set_mode(mode);
    fs::set_permissions(root, perms).unwrap_or_else(|error| panic!("chmod {}: {error}", root.display()));
    if metadata.is_dir() {
        for entry in fs::read_dir(root).unwrap_or_else(|error| panic!("read dir {}: {error}", root.display())) {
            let entry = entry.unwrap_or_else(|error| panic!("dir entry under {}: {error}", root.display()));
            set_tree_permissions(&entry.path(), dir_mode, file_mode);
        }
    }
}

#[test]
fn installed_cargo_subcommand_works_without_repo_checkout() {
    let standalone = build_standalone_cargo_octet();
    let fixture = ConsumerFixture::new("installed-subcommand", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));

    let output = fixture.run_with_launcher(
        Some(&standalone.bin_path),
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        output.status.success(),
        "installed cargo-octet should succeed without a repo checkout: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    let build_pos = log.find("ARGS:build").expect("saw embedded lint-library build in cargo log");
    let check_pos = log.find("ARGS:check").expect("saw consumer cargo check in cargo log");
    assert!(build_pos < check_pos, "lint-library build should happen before cargo check: {log}");
    assert!(
        !log.contains(&standalone.source_root.display().to_string()),
        "installed cargo-octet should not rely on the removed standalone source tree: {log}"
    );
}

#[test]
fn installed_cargo_subcommand_reuses_valid_embedded_source_cache_without_reextracting() {
    let standalone = build_standalone_cargo_octet();
    let fixture = ConsumerFixture::new("installed-cache-hit", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let cache_home_tmp = TempDir::new("installed-cache-hit-home");
    let cache_home = cache_home_tmp.path().join("cache-home");
    let cache_home_value = cache_home.display().to_string();

    let first_output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[("XDG_CACHE_HOME", cache_home_value.as_str())],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        first_output.status.success(),
        "first installed cargo-octet run should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&first_output.stdout),
        String::from_utf8_lossy(&first_output.stderr)
    );

    let source_dir = first_embedded_source_dir(&cache_home);
    set_tree_permissions(&source_dir, 0o555, 0o444);

    let second_output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[("XDG_CACHE_HOME", cache_home_value.as_str())],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        second_output.status.success(),
        "cached installed cargo-octet run should succeed without re-extracting: stdout={} stderr={}",
        String::from_utf8_lossy(&second_output.stdout),
        String::from_utf8_lossy(&second_output.stderr)
    );
    let marker_mode = fs::metadata(source_dir.join(".tigerstyle-source-blake3"))
        .expect("embedded source marker metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(marker_mode, 0o444, "embedded source marker should stay read-only when cache is reused");
    set_tree_permissions(&source_dir, 0o755, 0o644);
}

#[test]
fn installed_cargo_subcommand_recovers_corrupt_embedded_source_cache() {
    let standalone = build_standalone_cargo_octet();
    let fixture = ConsumerFixture::new("installed-cache-recovery", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let cache_home_tmp = TempDir::new("installed-cache-recovery-home");
    let cache_home = cache_home_tmp.path().join("cache-home");
    let cache_home_value = cache_home.display().to_string();

    let first_output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[("XDG_CACHE_HOME", cache_home_value.as_str())],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        first_output.status.success(),
        "first installed cargo-octet run should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&first_output.stdout),
        String::from_utf8_lossy(&first_output.stderr)
    );

    let source_dir = first_embedded_source_dir(&cache_home);
    let marker_path = source_dir.join(".tigerstyle-source-blake3");
    fs::write(&marker_path, "corrupt-marker\n")
        .unwrap_or_else(|error| panic!("write {}: {error}", marker_path.display()));

    let second_output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[("XDG_CACHE_HOME", cache_home_value.as_str())],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        second_output.status.success(),
        "installed cargo-octet should recover corrupt embedded source cache: stdout={} stderr={}",
        String::from_utf8_lossy(&second_output.stdout),
        String::from_utf8_lossy(&second_output.stderr)
    );
    let marker =
        fs::read_to_string(&marker_path).unwrap_or_else(|error| panic!("read {}: {error}", marker_path.display()));
    assert_ne!(
        marker.trim_end(),
        "corrupt-marker",
        "embedded source cache marker should be rewritten after recovery"
    );
}

#[test]
fn installed_cargo_subcommand_falls_back_to_temp_cache_when_home_caches_are_unwritable() {
    let standalone = build_standalone_cargo_octet();
    let fixture =
        ConsumerFixture::new("installed-temp-cache-fallback", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let blocked_roots = TempDir::new("installed-temp-cache-blocked");
    let blocked_xdg = blocked_roots.path().join("blocked-xdg");
    let blocked_home = blocked_roots.path().join("blocked-home");
    fs::create_dir_all(&blocked_xdg).unwrap_or_else(|error| panic!("create {}: {error}", blocked_xdg.display()));
    fs::create_dir_all(&blocked_home).unwrap_or_else(|error| panic!("create {}: {error}", blocked_home.display()));
    set_tree_permissions(&blocked_xdg, 0o555, 0o444);
    set_tree_permissions(&blocked_home, 0o555, 0o444);

    let temp_root_tmp = TempDir::new("installed-temp-cache-root");
    let temp_root = temp_root_tmp.path().join("tmp-root");
    fs::create_dir_all(&temp_root).unwrap_or_else(|error| panic!("create {}: {error}", temp_root.display()));

    let blocked_xdg_value = blocked_xdg.display().to_string();
    let blocked_home_value = blocked_home.display().to_string();
    let temp_root_value = temp_root.display().to_string();
    let output = fixture.run_with_launcher_and_env(
        Some(&standalone.bin_path),
        &[
            ("XDG_CACHE_HOME", blocked_xdg_value.as_str()),
            ("HOME", blocked_home_value.as_str()),
            ("TMPDIR", temp_root_value.as_str()),
        ],
        &[],
        0,
        false,
        None,
        Some(fixture.driver_path.as_path()),
        None,
        None,
    );
    assert!(
        output.status.success(),
        "installed cargo-octet should fall back to temp cache when home caches are unwritable: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let temp_cache_root = temp_root.join("octet-cache");
    let source_dir = first_embedded_source_dir(&temp_cache_root);
    assert!(
        source_dir.starts_with(&temp_cache_root),
        "embedded source cache should be materialized under temp fallback root {} but got {}",
        temp_cache_root.display(),
        source_dir.display()
    );
    assert!(
        !blocked_xdg.join("tigerstyle").exists(),
        "unwritable XDG cache root should stay unused: {}",
        blocked_xdg.display()
    );
    assert!(
        !blocked_home.join(".cache").exists(),
        "unwritable HOME cache root should stay unused: {}",
        blocked_home.display()
    );

    set_tree_permissions(&blocked_xdg, 0o755, 0o644);
    set_tree_permissions(&blocked_home, 0o755, 0o644);
}

#[test]
fn installed_cargo_subcommand_handles_concurrent_first_use_of_shared_embedded_source_cache() {
    let standalone = build_standalone_cargo_octet();
    let cache_home_tmp = TempDir::new("installed-concurrent-cache-home");
    let cache_home = cache_home_tmp.path().join("cache-home");
    fs::create_dir_all(&cache_home).unwrap_or_else(|error| panic!("create {}: {error}", cache_home.display()));

    let mut handles = Vec::new();
    for label in ["installed-concurrent-cache-a", "installed-concurrent-cache-b"] {
        let bin_path = standalone.bin_path.clone();
        let cache_home = cache_home.clone();
        handles.push(std::thread::spawn(move || {
            let fixture = ConsumerFixture::new(label, &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
            let cache_home_value = cache_home.display().to_string();
            let output = fixture.run_with_launcher_and_env(
                Some(bin_path.as_path()),
                &[("XDG_CACHE_HOME", cache_home_value.as_str())],
                &[],
                0,
                false,
                None,
                Some(fixture.driver_path.as_path()),
                None,
                None,
            );
            (label, output)
        }));
    }

    for handle in handles {
        let (label, output) = handle.join().expect("join concurrent installed cargo-octet run");
        assert!(
            output.status.success(),
            "{label} should succeed during concurrent first-use cache extraction: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    let source_dir = first_embedded_source_dir(&cache_home);
    let file_name = source_dir.file_name().and_then(|value| value.to_str()).expect("embedded source dir name");
    assert!(
        !file_name.starts_with('.'),
        "concurrent cache extraction should leave the final hash dir, not a staging dir: {}",
        source_dir.display()
    );
    assert!(
        source_dir.join(".tigerstyle-source-blake3").is_file(),
        "concurrent cache extraction should leave a valid hash marker in {}",
        source_dir.display()
    );
}

#[test]
fn checked_in_nix_consumer_fixture_is_reviewable() {
    let fixture_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/consumer-integration/nix-consumer");
    let cargo_toml = fs::read_to_string(fixture_root.join("Cargo.toml")).unwrap_or_else(|error| {
        panic!("read checked-in nix consumer Cargo.toml under {}: {error}", fixture_root.display())
    });
    let flake = fs::read_to_string(fixture_root.join("flake.nix"))
        .unwrap_or_else(|error| panic!("read checked-in nix consumer flake under {}: {error}", fixture_root.display()));

    assert!(
        cargo_toml.contains("[workspace.metadata.octet]"),
        "checked-in nix consumer fixture must carry workspace octet metadata: {}",
        fixture_root.display()
    );
    assert!(
        flake.contains("mkConsumerCheck"),
        "checked-in nix consumer fixture must exercise the first-party Nix helper: {}",
        fixture_root.display()
    );
}

#[test]
fn downstream_rollout_docs_point_to_first_class_cli_path() {
    let doc_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/downstream-rollout.md");
    let source = fs::read_to_string(&doc_path)
        .unwrap_or_else(|error| panic!("read downstream rollout doc {}: {error}", doc_path.display()));

    for required in [
        "cargo octet check",
        "[workspace.metadata.octet]",
        "nix run",
        "cargo-octet",
    ] {
        assert!(
            source.contains(required),
            "downstream rollout doc must mention `{required}`: {}",
            doc_path.display()
        );
    }
    assert!(
        !source.contains("Copy `templates/tigerstyle-check.sh`"),
        "downstream rollout doc should not keep the copied template as the active recommended path: {}",
        doc_path.display()
    );
}

#[test]
fn pre_commit_deny_all_hook_metadata_and_template_are_reviewable() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let metadata_path = repo_root.join(".pre-commit-hooks.yaml");
    let hook_path = repo_root.join("hooks/octet-deny-all.sh");
    let template_path = repo_root.join("templates/pre-commit-config.yaml");
    let inventory_path = repo_root.join("tests/fixtures/lint_inventory_names.txt");

    let metadata = fs::read_to_string(&metadata_path)
        .unwrap_or_else(|error| panic!("read pre-commit hook metadata {}: {error}", metadata_path.display()));
    let hook = fs::read_to_string(&hook_path)
        .unwrap_or_else(|error| panic!("read pre-commit hook script {}: {error}", hook_path.display()));
    let template = fs::read_to_string(&template_path)
        .unwrap_or_else(|error| panic!("read pre-commit config template {}: {error}", template_path.display()));
    let inventory = fs::read_to_string(&inventory_path)
        .unwrap_or_else(|error| panic!("read lint inventory {}: {error}", inventory_path.display()));

    for required in [
        "id: octet-deny-all",
        "id: tigerstyle-deny-all",
        "entry: hooks/octet-deny-all.sh",
        "language: script",
        "pass_filenames: false",
        "always_run: true",
    ] {
        assert!(
            metadata.contains(required),
            "pre-commit hook metadata must include `{required}` in {}",
            metadata_path.display()
        );
    }

    for lint_name in inventory.lines().filter(|line| !line.trim().is_empty()) {
        let expected_flag = format!("-D {lint_name}");
        assert!(
            hook.contains(&expected_flag),
            "deny-all pre-commit hook must promote `{lint_name}` with `{expected_flag}` in {}",
            hook_path.display()
        );
    }

    for required in [
        "#!/bin/sh",
        "-D unknown_lints",
        "OCTET_PRECOMMIT_USE_INSTALLED",
        "nix run \"${flake_ref}#cargo-octet\" -- check",
    ] {
        assert!(
            hook.contains(required),
            "deny-all pre-commit hook must include `{required}` in {}",
            hook_path.display()
        );
    }
    assert!(
        !hook.contains("/usr/bin/env bash"),
        "deny-all pre-commit hook must avoid /usr/bin/env for pure Nix compatibility: {}",
        hook_path.display()
    );
    assert!(
        !hook.contains("-D warnings"),
        "deny-all pre-commit hook should deny Octet lints without promoting unrelated rustc warnings: {}",
        hook_path.display()
    );

    for required in [
        "repo: https://github.com/OnixResearch/octet",
        "rev: 0123456789abcdef0123456789abcdef01234567",
        "id: octet-deny-all",
        "args: [\"--workspace\", \"--\", \"--all-targets\"]",
    ] {
        assert!(
            template.contains(required),
            "pre-commit config template must include `{required}` in {}",
            template_path.display()
        );
    }
}

#[test]
fn pre_commit_deny_all_hook_launches_pinned_runner_with_deny_flags() {
    let fixture = ConsumerFixture::new("pre-commit-deny-all", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let shell = bash_path().expect("find bash");
    write_executable(
        &fixture.fakebin.join("nix"),
        &format!(
            r#"#!{shell}
set -euo pipefail
printf 'NIX_ARGS:%s\n' "$*" >> "$TS_LOG"
printf 'DYLINT_RUSTFLAGS:%s\n' "${{DYLINT_RUSTFLAGS:-}}" >> "$TS_LOG"
"#,
            shell = shell.display(),
        ),
    );

    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let hook_path = repo_root.join("hooks/octet-deny-all.sh");
    let path = format!("{}:{}", fixture.fakebin.display(), env::var("PATH").expect("PATH"));
    let output = Command::new(&hook_path)
        .arg("--workspace")
        .current_dir(&fixture.repo)
        .env("PATH", path)
        .env("TS_LOG", &fixture.log_path)
        .output()
        .expect("run deny-all pre-commit hook");

    assert!(
        output.status.success(),
        "deny-all pre-commit hook should delegate successfully to fake nix: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let log = fixture.log();
    assert!(log.contains("NIX_ARGS:run path:"), "hook should use the pinned flake path: {log}");
    assert!(
        log.contains("#cargo-octet -- check --workspace"),
        "hook should run cargo-octet check and preserve explicit args: {log}"
    );
    for required in [
        "-D unknown_lints",
        "-D numeric_units",
        "-D raw_arithmetic_overflow",
        "-D no_recursion",
    ] {
        assert!(log.contains(required), "hook should inject deny flag `{required}` through DYLINT_RUSTFLAGS: {log}");
    }
}

#[test]
fn thin_wrapper_fixture_delegates_without_policy_logic() {
    let fixture_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/consumer-integration/justfile");
    let source = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|error| panic!("read thin-wrapper fixture {}: {error}", fixture_path.display()));

    assert!(
        source.contains("cargo octet check"),
        "thin-wrapper fixture must delegate to cargo octet check: {}",
        fixture_path.display()
    );
    for forbidden in [
        "RUSTC_WORKSPACE_WRAPPER",
        "DYLINT_LIBS",
        "DYLINT_NO_DEPS",
        "TIGERSTYLE_TOOLCHAIN",
        "cargo check",
        "cargo dylint",
    ] {
        assert!(
            !source.contains(forbidden),
            "thin-wrapper fixture must not reimplement rollout policy with `{forbidden}`: {}",
            fixture_path.display()
        );
    }
}

// --- Structured output integration tests ---

const EXIT_CODE_SUCCESS: i32 = 0;
const EXIT_CODE_LINT_FAILURE: i32 = 1;
const EXIT_CODE_INTEGRATION_FAILURE: i32 = 2;
const WARNING_LEVEL: &str = "warning";
const ERROR_LEVEL: &str = "error";
const JSON_OUTPUT_FORMAT: &str = "json";
const SARIF_OUTPUT_FORMAT: &str = "sarif";
const STATUS_CLEAN: &str = "clean";
const STATUS_WARNING_ONLY: &str = "warning-only";
const STATUS_LINT_FAILURE: &str = "lint-failure";
const STATUS_INTEGRATION_FAILURE: &str = "integration-failure";
const BASELINE_SCHEMA_VERSION: u64 = 1;
const FINDING_SCHEMA_VERSION: u64 = 2;
const STATUS_JSON_FILE: &str = "status.json";
const COMMAND_TXT_FILE: &str = "command.txt";
const SUMMARY_TXT_FILE: &str = "summary.txt";
const RESULTS_JSONL_FILE: &str = "results.jsonl";
const RESULTS_SARIF_FILE: &str = "results.sarif";
const PROVENANCE_JSONL_FILE: &str = "provenance.jsonl";

fn cargo_compiler_message(
    target_name: &str,
    lint_name: &str,
    message: &str,
    file: &str,
    line: u32,
    col: u32,
    suggestion: Option<(&str, &str)>,
) -> String {
    cargo_compiler_message_with_level(target_name, lint_name, WARNING_LEVEL, message, file, line, col, suggestion)
}

fn cargo_compiler_message_with_level(
    target_name: &str,
    lint_name: &str,
    level: &str,
    message: &str,
    file: &str,
    line: u32,
    col: u32,
    suggestion: Option<(&str, &str)>,
) -> String {
    let children = match suggestion {
        Some((replacement, applicability)) => format!(
            r#"[{{"message":"try this","code":null,"level":"help","spans":[{{"file_name":"{file}","byte_start":0,"byte_end":1,"line_start":{line},"line_end":{line},"column_start":{col},"column_end":{col},"is_primary":true,"text":[],"label":null,"suggested_replacement":"{replacement}","suggestion_applicability":"{applicability}","expansion":null}}],"children":[],"rendered":null}}]"#
        ),
        None => "[]".to_owned(),
    };
    format!(
        r#"{{"reason":"compiler-message","package_id":"path+file:///fake#{target_name}@0.1.0","manifest_path":"/fake/Cargo.toml","target":{{"kind":["lib"],"crate_types":["lib"],"name":"{target_name}","src_path":"/fake/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true}},"message":{{"$message_type":"diagnostic","message":"{message}","code":{{"code":"{lint_name}","explanation":null}},"level":"{level}","spans":[{{"file_name":"{file}","byte_start":0,"byte_end":1,"line_start":{line},"line_end":{line},"column_start":{col},"column_end":{col},"is_primary":true,"text":[],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}}],"children":{children},"rendered":"{level}[{lint_name}]: {message}\n --> {file}:{line}:{col}\n"}}}}"#
    )
}

fn cargo_compiler_message_with_source_text(
    target_name: &str,
    lint_name: &str,
    message: &str,
    file: &str,
    line: u32,
    col: u32,
    source_text: &str,
) -> String {
    format!(
        r#"{{"reason":"compiler-message","package_id":"path+file:///fake#{target_name}@0.1.0","manifest_path":"/fake/Cargo.toml","target":{{"kind":["lib"],"crate_types":["lib"],"name":"{target_name}","src_path":"/fake/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true}},"message":{{"$message_type":"diagnostic","message":"{message}","code":{{"code":"{lint_name}","explanation":null}},"level":"warning","spans":[{{"file_name":"{file}","byte_start":0,"byte_end":1,"line_start":{line},"line_end":{line},"column_start":{col},"column_end":{col},"is_primary":true,"text":[{{"text":"{source_text}","highlight_start":{col},"highlight_end":{col}}}],"label":null,"suggested_replacement":null,"suggestion_applicability":null,"expansion":null}}],"children":[],"rendered":"warning[{lint_name}]: {message}\n --> {file}:{line}:{col}\n"}}}}"#
    )
}

fn cargo_build_finished(success: bool) -> String {
    format!(r#"{{"reason":"build-finished","success":{success}}}"#)
}

fn cargo_compiler_artifact(target_name: &str, fresh: bool) -> String {
    format!(
        r#"{{"reason":"compiler-artifact","package_id":"path+file:///fake#{target_name}@0.1.0","manifest_path":"/fake/Cargo.toml","target":{{"kind":["lib"],"crate_types":["lib"],"name":"{target_name}","src_path":"/fake/src/lib.rs","edition":"2024","doc":true,"doctest":true,"test":true}},"profile":{{"opt_level":"0","debuginfo":2,"debug_assertions":true,"overflow_checks":true,"test":false}},"features":[],"filenames":["/fake/target/debug/lib{target_name}.rlib"],"executable":null,"fresh":{fresh}}}"#
    )
}

fn write_json_stdout_file(fixture: &ConsumerFixture, lines: &[String]) -> PathBuf {
    let path = fixture.repo.join("../.cargo-json-stdout");
    fs::write(&path, lines.join("\n") + "\n").expect("write JSON stdout file");
    path
}

fn run_with_check_args(
    fixture: &ConsumerFixture,
    check_args: &[&str],
    json_lines: &[String],
    cargo_exit: i32,
) -> std::process::Output {
    let stdout_file = write_json_stdout_file(fixture, json_lines);
    fixture.run_with_launcher_and_env(
        None,
        &[("TS_STDOUT_FILE", stdout_file.to_str().unwrap())],
        check_args,
        cargo_exit,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    )
}

fn run_with_output_format(
    fixture: &ConsumerFixture,
    output_format: &str,
    json_lines: &[String],
    cargo_exit: i32,
) -> std::process::Output {
    run_with_check_args(fixture, &["--output-format", output_format], json_lines, cargo_exit)
}

fn artifact_dir(fixture: &ConsumerFixture, label: &str) -> PathBuf {
    fixture.repo.join("artifacts").join(label)
}

fn read_artifact(path: &Path) -> String {
    fs::read_to_string(path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn read_json_artifact(path: &Path) -> serde_json::Value {
    let contents = read_artifact(path);
    serde_json::from_str(&contents)
        .unwrap_or_else(|error| panic!("parse {} as JSON: {error}\n{}", path.display(), contents))
}

fn artifact_file_names(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("read {}: {error}", dir.display()))
        .map(|entry| entry.expect("artifact dir entry").file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

fn sorted_object_keys(value: &serde_json::Value) -> Vec<String> {
    let mut keys: Vec<String> = value
        .as_object()
        .unwrap_or_else(|| panic!("expected JSON object, got: {value}"))
        .keys()
        .cloned()
        .collect();
    keys.sort();
    keys
}

fn assert_metadata_shape(metadata: &serde_json::Value) {
    assert_eq!(
        sorted_object_keys(metadata),
        vec![
            "config_hash".to_owned(),
            "profile_hash".to_owned(),
            "profile_name".to_owned(),
            "rustc_version".to_owned(),
            "tool_name".to_owned(),
            "tool_version".to_owned(),
            "toolchain".to_owned(),
        ],
        "metadata should expose deterministic replay fields"
    );
    assert_eq!(metadata["tool_name"], "cargo-octet");
    assert!(metadata["config_hash"].as_str().expect("config hash").starts_with("b3:"));
    assert!(metadata["profile_hash"].as_str().expect("profile hash").starts_with("b3:"));
    assert!(metadata["rustc_version"].as_str().expect("rustc version").starts_with("rustc "));
}

fn expected_artifact_files(output_format: &str) -> Vec<String> {
    let mut names = match output_format {
        "human" => vec![
            COMMAND_TXT_FILE.to_owned(),
            PROVENANCE_JSONL_FILE.to_owned(),
            STATUS_JSON_FILE.to_owned(),
            SUMMARY_TXT_FILE.to_owned(),
        ],
        JSON_OUTPUT_FORMAT => vec![
            COMMAND_TXT_FILE.to_owned(),
            PROVENANCE_JSONL_FILE.to_owned(),
            RESULTS_JSONL_FILE.to_owned(),
            STATUS_JSON_FILE.to_owned(),
            SUMMARY_TXT_FILE.to_owned(),
        ],
        SARIF_OUTPUT_FORMAT => vec![
            COMMAND_TXT_FILE.to_owned(),
            PROVENANCE_JSONL_FILE.to_owned(),
            RESULTS_SARIF_FILE.to_owned(),
            STATUS_JSON_FILE.to_owned(),
            SUMMARY_TXT_FILE.to_owned(),
        ],
        other => panic!("unexpected output format: {other}"),
    };
    names.sort();
    names
}

fn artifact_bundle_is_valid(dir: &Path, output_format: &str) -> bool {
    if !dir.is_dir() {
        return false;
    }

    let expected = expected_artifact_files(output_format);
    if artifact_file_names(dir) != expected {
        return false;
    }

    expected.iter().all(|name| dir.join(name).is_file())
}

fn assert_provenance_matches_artifacts(dir: &Path, output_format: &str) {
    let records = read_provenance_records(dir);
    let artifact_names = expected_artifact_files(output_format)
        .into_iter()
        .filter(|name| name != PROVENANCE_JSONL_FILE)
        .collect::<Vec<_>>();

    assert_eq!(
        records.len(),
        artifact_names.len(),
        "one provenance record should cover each non-provenance artifact"
    );

    for artifact_name in artifact_names {
        let record = records
            .iter()
            .find(|candidate| candidate["artifact_locator"] == artifact_name)
            .unwrap_or_else(|| panic!("missing provenance record for {artifact_name}: {records:#?}"));
        let artifact_bytes = fs::read(dir.join(&artifact_name)).expect("read provenanced artifact");
        let artifact_hash = blake3::hash(&artifact_bytes).to_hex().to_string();
        assert_eq!(record["schema_version"], "octet-provenance-record/v1");
        assert_eq!(record["checker_id"], "cargo-octet");
        assert_eq!(record["blake3"], artifact_hash);
        assert_eq!(record["checked_scope"], "profile:workspace-metadata");
        if artifact_name == RESULTS_JSONL_FILE || artifact_name == RESULTS_SARIF_FILE {
            assert_eq!(record["source_id"], "octet.runner.static-findings");
            assert_eq!(record["evidence_tier"], "static_lint");
        } else {
            assert_eq!(record["source_id"], "octet.runner.artifact-bundle");
            assert_eq!(record["evidence_tier"], "manual_review_required");
            assert!(
                record["human_action"].as_str().expect("manual review action").contains("review runner status"),
                "manual review provenance should carry a human action: {record:#?}"
            );
        }
    }
}

fn read_provenance_records(dir: &Path) -> Vec<serde_json::Value> {
    read_artifact(&dir.join(PROVENANCE_JSONL_FILE))
        .lines()
        .map(|line| serde_json::from_str(line).unwrap_or_else(|error| panic!("parse provenance line: {error}: {line}")))
        .collect()
}

fn launcher_path_from_log(log: &str) -> &str {
    log.lines()
        .find_map(|line| line.strip_prefix("LAUNCHER:"))
        .unwrap_or_else(|| panic!("missing launcher line in log: {log}"))
}

fn trailing_summary(stderr: &str) -> &str {
    let summary_header = "--- octet summary ---";
    let start = stderr.find(summary_header).unwrap_or_else(|| panic!("missing summary header in stderr: {stderr}"));
    &stderr[start..]
}

fn three_finding_json_lines() -> Vec<String> {
    vec![
        cargo_compiler_artifact("aspen-core", false),
        cargo_compiler_message(
            "aspen-core",
            "no_unwrap",
            "use of `.unwrap()`",
            "src/connection.rs",
            42,
            5,
            Some((".expect(\\\"reason\\\")", "MaybeIncorrect")),
        ),
        cargo_compiler_message(
            "aspen-core",
            "function_length",
            "function `handle_request` is 82 lines long (limit: 70)",
            "src/router.rs",
            15,
            1,
            None,
        ),
        cargo_compiler_artifact("aspen-dns", false),
        cargo_compiler_message(
            "aspen-dns",
            "unbounded_channel",
            "use of unbounded channel",
            "src/resolver.rs",
            28,
            9,
            None,
        ),
        cargo_build_finished(true),
    ]
}

fn four_finding_json_lines() -> Vec<String> {
    let mut lines = three_finding_json_lines();
    lines.insert(
        lines.len() - 1,
        cargo_compiler_message(
            "aspen-core",
            "no_unwrap",
            "use of `.unwrap()` in new path",
            "src/new_connection.rs",
            7,
            3,
            None,
        ),
    );
    lines
}

#[test]
fn structured_json_output_contains_all_required_fields() {
    let fixture = ConsumerFixture::new("json-output", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, JSON_OUTPUT_FORMAT, &three_finding_json_lines(), EXIT_CODE_SUCCESS);

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "warning-only findings should exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let findings: Vec<serde_json::Value> =
        stdout.lines().map(|line| serde_json::from_str(line).expect("parse JSON finding")).collect();

    assert_eq!(findings.len(), 3, "expected 3 findings, got: {stdout}");

    for (i, f) in findings.iter().enumerate() {
        assert_eq!(f["version"], FINDING_SCHEMA_VERSION, "finding {i} missing version");
        assert!(f["index"].as_str().is_some(), "finding {i} missing index");
        assert!(f["stable_id"].as_str().is_some(), "finding {i} missing stable_id");
        assert!(f["lint"].as_str().is_some(), "finding {i} missing lint");
        assert!(f["group"].as_str().is_some(), "finding {i} missing group");
        assert!(f["severity"].as_str().is_some(), "finding {i} missing severity");
        assert!(f["file"].as_str().is_some(), "finding {i} missing file");
        assert!(f["line"].is_number(), "finding {i} missing line");
        assert!(f["column"].is_number(), "finding {i} missing column");
        assert!(f["message"].as_str().is_some(), "finding {i} missing message");
        assert!(f["principle"].as_str().is_some(), "finding {i} missing principle");
        assert!(f["crate_name"].as_str().is_some(), "finding {i} missing crate_name");
    }

    assert_eq!(findings[0]["lint"], "no_unwrap");
    assert_eq!(findings[0]["group"], "tigerstyle::safety");
    assert_eq!(findings[0]["file"], "src/connection.rs");
    assert_eq!(findings[0]["line"], 42);
    assert_eq!(findings[0]["crate_name"], "aspen-core");

    assert_eq!(findings[1]["lint"], "unbounded_channel");
    assert_eq!(findings[1]["crate_name"], "aspen-dns");

    assert_eq!(findings[2]["lint"], "function_length");
    assert_eq!(findings[2]["group"], "tigerstyle::structure");
}

#[test]
fn semantic_identity_survives_file_rename() {
    let fixture = ConsumerFixture::new("semantic-identity-rename", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let original = vec![
        cargo_compiler_artifact("aspen-core", false),
        cargo_compiler_message("aspen-core", "no_unwrap", "use of `.unwrap()`", "src/old_name.rs", 10, 5, None),
        cargo_build_finished(true),
    ];
    let renamed = vec![
        cargo_compiler_artifact("aspen-core", false),
        cargo_compiler_message("aspen-core", "no_unwrap", "use of `.unwrap()`", "src/new_name.rs", 10, 5, None),
        cargo_build_finished(true),
    ];

    let first = run_with_output_format(&fixture, JSON_OUTPUT_FORMAT, &original, EXIT_CODE_SUCCESS);
    let second = run_with_output_format(&fixture, JSON_OUTPUT_FORMAT, &renamed, EXIT_CODE_SUCCESS);
    assert_eq!(first.status.code(), Some(EXIT_CODE_SUCCESS));
    assert_eq!(second.status.code(), Some(EXIT_CODE_SUCCESS));

    let first_finding: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&first.stdout).lines().next().expect("first finding"))
            .expect("json");
    let second_finding: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&second.stdout).lines().next().expect("second finding"))
            .expect("json");
    assert_eq!(first_finding["semantic_id"], second_finding["semantic_id"]);
    assert_eq!(first_finding["stable_id"], second_finding["stable_id"]);
    assert_eq!(first_finding["identity_status"], "unsupported");
    assert_eq!(first_finding["normalized_snippet_hash"], "unsupported");
}

#[test]
fn json_findings_report_effect_family_cache_status_and_unknown_lints() {
    let fixture = ConsumerFixture::new("effect-cache-output", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![
        cargo_compiler_artifact("aspen-core", false),
        cargo_compiler_message("aspen-core", "no_unwrap", "use of `.unwrap()`", "src/lib.rs", 10, 5, None),
        cargo_compiler_message("aspen-core", "mystery_lint", "unknown diagnostic", "src/lib.rs", 11, 5, None),
        cargo_build_finished(true),
    ];
    let output =
        run_with_check_args(&fixture, &["--output-format", JSON_OUTPUT_FORMAT, "--cache"], &lines, EXIT_CODE_SUCCESS);
    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    let findings: Vec<serde_json::Value> =
        stdout.lines().map(|line| serde_json::from_str(line).expect("parse JSON finding")).collect();
    assert_eq!(findings[0]["effect_family"], "failure");
    assert_eq!(findings[0]["cache_status"], "bypassed");
    assert_eq!(findings[1]["lint"], "mystery_lint");
    assert_eq!(findings[1]["group"], "tigerstyle::unknown");
    assert_eq!(findings[1]["effect_family"], "unknown");
}

#[test]
fn human_summary_aggregates_by_effect_family() {
    let fixture = ConsumerFixture::new("effect-summary", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, "human", &three_finding_json_lines(), EXIT_CODE_SUCCESS);
    assert_eq!(output.status.code(), Some(EXIT_CODE_SUCCESS));
    let stderr = String::from_utf8_lossy(&output.stderr);
    let summary = trailing_summary(&stderr);
    assert!(summary.contains("By effect:"), "summary should include effect aggregation: {summary}");
    assert!(summary.contains("failure"), "summary should include failure effect: {summary}");
    assert!(summary.contains("resource"), "summary should include resource effect: {summary}");
}

#[test]
fn cache_uses_content_identity_and_bypasses_unsupported_inputs() {
    let fixture = ConsumerFixture::new("content-cache", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let computed = vec![
        cargo_compiler_artifact("aspen-core", false),
        cargo_compiler_message_with_source_text(
            "aspen-core",
            "no_unwrap",
            "use of `.unwrap()`",
            "src/lib.rs",
            10,
            5,
            "let value = input.unwrap();",
        ),
        cargo_build_finished(true),
    ];

    let first = run_with_check_args(
        &fixture,
        &["--output-format", JSON_OUTPUT_FORMAT, "--cache"],
        &computed,
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(first.status.code(), Some(EXIT_CODE_SUCCESS));
    let first_finding: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&first.stdout).lines().next().expect("first finding"))
            .expect("json");
    assert_eq!(first_finding["identity_status"], "computed");
    assert_eq!(first_finding["cache_status"], "miss");

    let second = run_with_check_args(
        &fixture,
        &["--output-format", JSON_OUTPUT_FORMAT, "--cache"],
        &computed,
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(second.status.code(), Some(EXIT_CODE_SUCCESS));
    let second_finding: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&second.stdout).lines().next().expect("second finding"))
            .expect("json");
    assert_eq!(second_finding["cache_status"], "hit");

    write_file(&fixture.repo.join("dylint.toml"), "[octet]\n# changed config\n");
    let changed_config = run_with_check_args(
        &fixture,
        &["--output-format", JSON_OUTPUT_FORMAT, "--cache"],
        &computed,
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(changed_config.status.code(), Some(EXIT_CODE_SUCCESS));
    let changed_finding: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&changed_config.stdout).lines().next().expect("changed finding"))
            .expect("json");
    assert_eq!(changed_finding["cache_status"], "miss");
    assert_ne!(changed_finding["semantic_id"], first_finding["semantic_id"]);

    let unsupported = vec![
        cargo_compiler_artifact("aspen-core", false),
        cargo_compiler_message("aspen-core", "no_unwrap", "use of `.unwrap()`", "src/lib.rs", 10, 5, None),
        cargo_build_finished(true),
    ];
    let bypass = run_with_check_args(
        &fixture,
        &["--output-format", JSON_OUTPUT_FORMAT, "--cache"],
        &unsupported,
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(bypass.status.code(), Some(EXIT_CODE_SUCCESS));
    let bypass_finding: serde_json::Value =
        serde_json::from_str(String::from_utf8_lossy(&bypass.stdout).lines().next().expect("bypass finding"))
            .expect("json");
    assert_eq!(bypass_finding["identity_status"], "unsupported");
    assert_eq!(bypass_finding["cache_status"], "bypassed");
}

#[test]
fn baseline_matches_semantic_identity_after_rename() {
    let fixture = ConsumerFixture::new("baseline-semantic-rename", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let baseline_path = fixture.repo.join("octet-baseline.json");
    let baseline_path_str = baseline_path.to_str().expect("baseline path utf8");
    let original = vec![
        cargo_compiler_artifact("aspen-core", false),
        cargo_compiler_message("aspen-core", "no_unwrap", "use of `.unwrap()`", "src/old_name.rs", 10, 5, None),
        cargo_build_finished(true),
    ];
    let renamed = vec![
        cargo_compiler_artifact("aspen-core", false),
        cargo_compiler_message("aspen-core", "no_unwrap", "use of `.unwrap()`", "src/new_name.rs", 10, 5, None),
        cargo_build_finished(true),
    ];

    let write_output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--write-baseline",
            baseline_path_str,
        ],
        &original,
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(write_output.status.code(), Some(EXIT_CODE_SUCCESS));

    let accepted_output = run_with_check_args(
        &fixture,
        &["--output-format", JSON_OUTPUT_FORMAT, "--baseline", baseline_path_str],
        &renamed,
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(accepted_output.status.code(), Some(EXIT_CODE_SUCCESS));
    let baseline = read_json_artifact(&baseline_path);
    let stored = &baseline["findings"][0];
    assert!(stored["semantic_id"].as_str().is_some(), "baseline stores semantic_id: {stored}");
    assert_eq!(stored["identity_status"], "unsupported");
}

#[test]
fn migration_manifest_commands_are_deterministic_and_safe() {
    let fixture = ConsumerFixture::new("migration-manifest", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let manifest = fixture.repo.join(".octet/migrations/starter.json");
    let bin = built_cargo_octet_bin().expect("build cargo-octet binary");

    let start = Command::new(&bin)
        .args([
            "migrate",
            "start",
            "--profile",
            "starter",
            "--manifest",
            manifest.to_str().expect("manifest utf8"),
            "--format",
            JSON_OUTPUT_FORMAT,
        ])
        .current_dir(&fixture.repo)
        .output()
        .expect("run migrate start");
    assert_eq!(start.status.code(), Some(EXIT_CODE_SUCCESS), "stderr={}", String::from_utf8_lossy(&start.stderr));
    let manifest_json = read_json_artifact(&manifest);
    assert_eq!(manifest_json["schema_version"], 1);
    assert_eq!(manifest_json["profile"], "starter");
    assert!(manifest_json["config_hash"].as_str().expect("config hash").starts_with("b3:"));
    assert!(manifest_json["crate_graph_hash"].as_str().expect("crate graph hash").starts_with("b3:"));

    let status = Command::new(&bin)
        .args([
            "migrate",
            "status",
            "--manifest",
            manifest.to_str().expect("manifest utf8"),
            "--format",
            JSON_OUTPUT_FORMAT,
        ])
        .current_dir(&fixture.repo)
        .output()
        .expect("run migrate status");
    assert_eq!(status.status.code(), Some(EXIT_CODE_SUCCESS));
    let status_json: serde_json::Value = serde_json::from_slice(&status.stdout).expect("status json");
    assert_eq!(status_json["unresolved"], 0);
    assert_eq!(status_json["manual_review_required"], 0);

    let before = fs::read_to_string(&manifest).expect("read manifest before apply-safe");
    let apply = Command::new(&bin)
        .args([
            "migrate",
            "apply-safe",
            "--manifest",
            manifest.to_str().expect("manifest utf8"),
            "--format",
            JSON_OUTPUT_FORMAT,
        ])
        .current_dir(&fixture.repo)
        .output()
        .expect("run migrate apply-safe");
    assert_eq!(apply.status.code(), Some(EXIT_CODE_SUCCESS));
    let after = fs::read_to_string(&manifest).expect("read manifest after apply-safe");
    assert_eq!(before, after, "safe apply must not mutate manual-review state without machine-applicable fixes");
}

#[test]
fn transcript_reports_content_hash_and_fails_closed() {
    let fixture = ConsumerFixture::new("transcript-report", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let good = fixture.repo.join("docs/good.md");
    write_file(
        &good,
        r#"# Example

```octet-transcript
$ cargo octet check --output-format json
=> {"version":1
```
"#,
    );
    let bad = fixture.repo.join("docs/bad.md");
    write_file(&bad, "# Missing expected output\n\n```octet-transcript\n$ cargo octet check\n```\n");
    let bin = built_cargo_octet_bin().expect("build cargo-octet binary");

    let good_output = Command::new(&bin)
        .args([
            "transcript",
            good.to_str().expect("good utf8"),
            "--format",
            JSON_OUTPUT_FORMAT,
        ])
        .current_dir(&fixture.repo)
        .output()
        .expect("run transcript good");
    assert_eq!(good_output.status.code(), Some(EXIT_CODE_SUCCESS));
    let good_json: serde_json::Value = serde_json::from_slice(&good_output.stdout).expect("good transcript json");
    assert_eq!(good_json["status"], "passed");
    assert!(good_json["transcript_hash"].as_str().expect("transcript hash").starts_with("b3:"));
    assert_eq!(good_json["provenance"]["checker_id"], "octet-transcript");
    assert_eq!(good_json["provenance"]["source_id"], "octet.transcript.assertion");
    assert_eq!(good_json["provenance"]["evidence_tier"], "transcript_assertion");
    assert_eq!(good_json["provenance"]["artifact_locator"], good.display().to_string());

    let bad_output = Command::new(&bin)
        .args([
            "transcript",
            bad.to_str().expect("bad utf8"),
            "--format",
            JSON_OUTPUT_FORMAT,
        ])
        .current_dir(&fixture.repo)
        .output()
        .expect("run transcript bad");
    assert_eq!(bad_output.status.code(), Some(EXIT_CODE_LINT_FAILURE));
    let bad_json: serde_json::Value = serde_json::from_slice(&bad_output.stdout).expect("bad transcript json");
    assert_eq!(bad_json["status"], "failed");
    assert!(
        bad_json["failures"]
            .as_array()
            .expect("failures")
            .iter()
            .any(|failure| failure.as_str().unwrap_or_default().contains("expected-output"))
    );
}

#[test]
fn baseline_file_accepts_current_findings_and_gates_new_ids() {
    let fixture = ConsumerFixture::new("baseline-file", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let baseline_path = fixture.repo.join("octet-baseline.json");
    let baseline_path_str = baseline_path.to_str().expect("baseline path utf8");

    let write_output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--write-baseline",
            baseline_path_str,
        ],
        &three_finding_json_lines(),
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(
        write_output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "writing a warning-only baseline should keep the normal exit code: stderr={}",
        String::from_utf8_lossy(&write_output.stderr)
    );

    let baseline = read_json_artifact(&baseline_path);
    assert_eq!(baseline["schema_version"], BASELINE_SCHEMA_VERSION);
    assert_eq!(baseline["finding_schema_version"], FINDING_SCHEMA_VERSION);
    assert!(baseline["baseline_hash"].as_str().is_some(), "baseline should carry a deterministic hash");
    assert_metadata_shape(&baseline["metadata"]);
    assert_eq!(baseline["findings"].as_array().expect("baseline findings array").len(), 3);
    assert!(
        baseline["findings"]
            .as_array()
            .expect("baseline findings array")
            .iter()
            .all(|finding| finding["stable_id"].as_str().is_some()),
        "every accepted finding should store a stable_id"
    );

    let accepted_dir = artifact_dir(&fixture, "baseline-accepted");
    let accepted_output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--baseline",
            baseline_path_str,
            "--artifact-dir",
            accepted_dir.to_str().expect("accepted artifact dir utf8"),
        ],
        &three_finding_json_lines(),
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(
        accepted_output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "fully accepted warning-only findings should stay exit 0: stderr={}",
        String::from_utf8_lossy(&accepted_output.stderr)
    );
    let accepted_status = read_json_artifact(&accepted_dir.join(STATUS_JSON_FILE));
    assert_eq!(accepted_status["status"], STATUS_WARNING_ONLY);
    assert_eq!(accepted_status["baseline"]["accepted_total"], 3);
    assert_eq!(accepted_status["baseline"]["accepted_current"], 3);
    assert_eq!(accepted_status["baseline"]["new_total"], 0);
    assert_eq!(accepted_status["baseline"]["stale_total"], 0);
    assert_eq!(
        accepted_status["baseline"]["baseline_hash"], baseline["baseline_hash"],
        "evaluating a just-written baseline should report the stored baseline hash"
    );

    let drift_dir = artifact_dir(&fixture, "baseline-drift");
    let drift_output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--baseline",
            baseline_path_str,
            "--artifact-dir",
            drift_dir.to_str().expect("drift artifact dir utf8"),
        ],
        &four_finding_json_lines(),
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(
        drift_output.status.code(),
        Some(EXIT_CODE_LINT_FAILURE),
        "one unaccepted stable_id should fail the drift gate: stdout={} stderr={}",
        String::from_utf8_lossy(&drift_output.stdout),
        String::from_utf8_lossy(&drift_output.stderr)
    );
    let drift_stdout = String::from_utf8_lossy(&drift_output.stdout);
    assert_eq!(drift_stdout.lines().count(), 4, "drift run should still emit all current findings");
    let drift_status = read_json_artifact(&drift_dir.join(STATUS_JSON_FILE));
    assert_eq!(drift_status["status"], STATUS_LINT_FAILURE);
    assert_eq!(drift_status["baseline"]["accepted_total"], 3);
    assert_eq!(drift_status["baseline"]["accepted_current"], 3);
    assert_eq!(drift_status["baseline"]["new_total"], 1);
    assert_eq!(drift_status["baseline"]["stale_total"], 0);
    assert_eq!(drift_status["baseline"]["new_finding_ids"].as_array().expect("new finding ids").len(), 1);
}

#[test]
fn baseline_write_skips_incomplete_cargo_runs() {
    let fixture = ConsumerFixture::new("baseline-write-failed", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let baseline_path = fixture.repo.join("octet-baseline.json");
    let baseline_path_str = baseline_path.to_str().expect("baseline path utf8");
    let lines = vec![
        cargo_compiler_message("consumer", "no_unwrap", "use of `.unwrap()`", "src/lib.rs", 10, 5, None),
        cargo_build_finished(false),
    ];

    let output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--write-baseline",
            baseline_path_str,
        ],
        &lines,
        EXIT_CODE_LINT_FAILURE,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_INTEGRATION_FAILURE),
        "failed cargo runs should report integration failure: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(!baseline_path.exists(), "failed cargo runs must not create or overwrite acceptance baselines");
}

#[test]
fn json_output_coexists_with_summary_on_stderr() {
    let fixture = ConsumerFixture::new("json-stderr", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, JSON_OUTPUT_FORMAT, &three_finding_json_lines(), EXIT_CODE_SUCCESS);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // stdout is pure JSON-lines
    for line in stdout.lines() {
        assert!(serde_json::from_str::<serde_json::Value>(line).is_ok(), "stdout line is not valid JSON: {line}");
    }

    // stderr has summary but no human diagnostics
    assert!(stderr.contains("--- octet summary ---"), "stderr missing summary header: {stderr}");
    assert!(stderr.contains("Findings: 3"), "stderr missing finding count: {stderr}");
    assert!(
        !stderr.contains("warning[no_unwrap]"),
        "stderr should not contain human diagnostics in json mode: {stderr}"
    );
}

#[test]
fn sarif_output_has_required_structural_fields() {
    let fixture = ConsumerFixture::new("sarif-output", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, SARIF_OUTPUT_FORMAT, &three_finding_json_lines(), EXIT_CODE_SUCCESS);

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "warning-only findings should exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sarif: serde_json::Value = serde_json::from_str(&stdout).expect("parse SARIF JSON");

    assert_eq!(
        sarif["$schema"].as_str().unwrap(),
        "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/main/sarif-2.1/schema/sarif-schema-2.1.0.json"
    );
    assert_eq!(sarif["version"], "2.1.0");

    let run = &sarif["runs"][0];
    assert_eq!(run["tool"]["driver"]["name"], "cargo-octet");
    assert!(run["tool"]["driver"]["version"].as_str().is_some(), "missing driver version");

    let rules = run["tool"]["driver"]["rules"].as_array().expect("rules array");
    assert_eq!(rules.len(), 3, "expected 3 rules for 3 distinct lints");

    let invocations = run["invocations"].as_array().expect("invocations array");
    assert_eq!(invocations.len(), 1, "expected one SARIF invocation");
    assert!(invocations[0]["commandLine"].as_str().is_some(), "missing SARIF invocation command line");
    assert_eq!(invocations[0]["exitCode"], EXIT_CODE_SUCCESS);

    assert_eq!(run["properties"]["status"], STATUS_WARNING_ONLY);
    assert_eq!(run["properties"]["exitCode"], EXIT_CODE_SUCCESS);
    assert_eq!(run["properties"]["warningFindings"], 3);
    assert_eq!(run["properties"]["errorFindings"], 0);

    let results = run["results"].as_array().expect("results array");
    assert_eq!(results.len(), 3, "expected 3 results");

    for (i, r) in results.iter().enumerate() {
        assert!(r["ruleId"].as_str().is_some(), "result {i} missing ruleId");
        assert!(r["level"].as_str().is_some(), "result {i} missing level");
        assert!(r["message"]["text"].as_str().is_some(), "result {i} missing message text");
        assert!(
            r["locations"][0]["physicalLocation"]["artifactLocation"]["uri"].as_str().is_some(),
            "result {i} missing location uri"
        );
        assert!(r["properties"]["findingIndex"].as_str().is_some(), "result {i} missing findingIndex property");
        assert!(r["properties"]["stableId"].as_str().is_some(), "result {i} missing stableId property");
    }

    assert_eq!(results[0]["properties"]["findingIndex"], "F1");
    assert_eq!(results[2]["properties"]["findingIndex"], "F3");
}

#[test]
fn summary_format_for_noisy_run() {
    let fixture = ConsumerFixture::new("summary-noisy", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, JSON_OUTPUT_FORMAT, &three_finding_json_lines(), EXIT_CODE_SUCCESS);

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("--- octet summary ---"), "missing summary header: {stderr}");
    assert!(stderr.contains("Status: warning-only"), "missing warning-only status: {stderr}");
    assert!(stderr.contains("Findings: 3"), "missing total count: {stderr}");
    assert!(stderr.contains("Warnings: 3"), "missing warning count: {stderr}");
    assert!(stderr.contains("Errors: 0"), "missing error count: {stderr}");
    assert!(stderr.contains("Autofixable: 0"), "missing autofixable count: {stderr}");
    assert!(stderr.contains("By crate:"), "missing by-crate section: {stderr}");
    assert!(stderr.contains("aspen-core"), "missing aspen-core in summary: {stderr}");
    assert!(stderr.contains("aspen-dns"), "missing aspen-dns in summary: {stderr}");
    assert!(stderr.contains("By lint:"), "missing by-lint section: {stderr}");
    assert!(stderr.contains("no_unwrap"), "missing no_unwrap in summary: {stderr}");
    assert!(stderr.contains("Index:"), "missing index section: {stderr}");
    assert!(stderr.contains("F1"), "missing F1 in index: {stderr}");
    assert!(stderr.contains("F3"), "missing F3 in index: {stderr}");
    assert!(stderr.ends_with("---\n"), "missing summary closing delimiter: {stderr}");
}

#[test]
fn summary_for_clean_run() {
    let fixture = ConsumerFixture::new("summary-clean", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![cargo_build_finished(true)];
    let output = run_with_output_format(&fixture, JSON_OUTPUT_FORMAT, &lines, EXIT_CODE_SUCCESS);

    assert!(
        output.status.success(),
        "clean run should exit 0: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--- octet summary ---"), "missing summary header: {stderr}");
    assert!(stderr.contains("Status: clean"), "clean run should show clean status: {stderr}");
    assert!(stderr.contains("Findings: 0"), "clean run should show Findings: 0: {stderr}");
    assert!(stderr.contains("Warnings: 0"), "clean run should show zero warnings: {stderr}");
    assert!(stderr.contains("Errors: 0"), "clean run should show zero errors: {stderr}");
    assert!(stderr.contains("Autofixable: 0"), "clean run should show zero autofixable findings: {stderr}");
    assert!(!stderr.contains("By crate:"), "clean run should not have by-crate section: {stderr}");
    assert!(!stderr.contains("By lint:"), "clean run should not have by-lint section: {stderr}");
}

#[test]
fn clean_structured_run_stays_exit_zero() {
    let fixture = ConsumerFixture::new("clean-structured-run", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![cargo_build_finished(true)];
    let output = run_with_output_format(&fixture, SARIF_OUTPUT_FORMAT, &lines, EXIT_CODE_SUCCESS);

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "clean structured run should exit 0: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Status: clean"), "clean structured run should report clean status: {stderr}");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sarif: serde_json::Value = serde_json::from_str(&stdout).expect("parse clean SARIF");
    assert_eq!(sarif["runs"][0]["properties"]["status"], STATUS_CLEAN);
    assert_eq!(sarif["runs"][0]["invocations"][0]["exitCode"], EXIT_CODE_SUCCESS);
    assert_eq!(sarif["runs"][0]["results"].as_array().expect("clean SARIF results array").len(), 0);
}

#[test]
fn cargo_compilation_failure_uses_integration_failure_exit_code() {
    let fixture =
        ConsumerFixture::new("integration-failure-exit-code", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![
        cargo_compiler_message("consumer", "no_unwrap", "use of `.unwrap()`", "src/lib.rs", 10, 5, None),
        cargo_build_finished(false),
    ];

    for output_format in [JSON_OUTPUT_FORMAT, SARIF_OUTPUT_FORMAT] {
        let output = run_with_output_format(&fixture, output_format, &lines, EXIT_CODE_LINT_FAILURE);
        assert_eq!(
            output.status.code(),
            Some(EXIT_CODE_INTEGRATION_FAILURE),
            "compile failure should exit 2 in {output_format} mode: stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );

        let stdout = String::from_utf8_lossy(&output.stdout);
        match output_format {
            JSON_OUTPUT_FORMAT => {
                for line in stdout.lines() {
                    assert!(
                        serde_json::from_str::<serde_json::Value>(line).is_ok(),
                        "stdout line is not valid JSON during compile failure: {line}"
                    );
                }
            }
            SARIF_OUTPUT_FORMAT => {
                let sarif: serde_json::Value = serde_json::from_str(&stdout).expect("parse integration-failure SARIF");
                assert!(
                    sarif["runs"][0]["results"].as_array().is_some(),
                    "compile failure SARIF should keep results array"
                );
                assert_eq!(sarif["runs"][0]["properties"]["status"], STATUS_INTEGRATION_FAILURE);
            }
            _ => unreachable!("structured output formats are exhaustively covered"),
        }

        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains("Status: integration-failure"),
            "compile failure summary should report integration-failure in {output_format} mode: {stderr}"
        );
    }
}

#[test]
fn artifact_directory_captures_structured_run() {
    let fixture = ConsumerFixture::new("artifact-dir-structured", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let dir = artifact_dir(&fixture, "structured-run");
    let output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            SARIF_OUTPUT_FORMAT,
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
        ],
        &three_finding_json_lines(),
        EXIT_CODE_SUCCESS,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "structured artifact run should exit 0 because findings are warning-only: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        artifact_file_names(&dir),
        expected_artifact_files(SARIF_OUTPUT_FORMAT),
        "artifact directory should contain exactly the expected files"
    );
    assert_provenance_matches_artifacts(&dir, SARIF_OUTPUT_FORMAT);

    let status = read_json_artifact(&dir.join(STATUS_JSON_FILE));
    assert_eq!(
        sorted_object_keys(&status),
        vec![
            "autofixable_findings".to_owned(),
            "cargo_process_exit".to_owned(),
            "error_findings".to_owned(),
            "exit_code".to_owned(),
            "metadata".to_owned(),
            "output_format".to_owned(),
            "status".to_owned(),
            "total_findings".to_owned(),
            "warning_findings".to_owned(),
        ],
        "status.json should use the exact top-level key set"
    );
    assert_eq!(sorted_object_keys(&status["cargo_process_exit"]), vec![
        "classification".to_owned(),
        "code".to_owned()
    ]);
    assert_eq!(status["status"], STATUS_WARNING_ONLY);
    assert_eq!(status["exit_code"], output.status.code().expect("structured artifact exit code"));
    assert_eq!(status["output_format"], SARIF_OUTPUT_FORMAT);
    assert_eq!(status["total_findings"], 3);
    assert_eq!(status["warning_findings"], 3);
    assert_eq!(status["error_findings"], 0);
    assert_eq!(status["cargo_process_exit"]["classification"], "success");
    assert_eq!(status["cargo_process_exit"]["code"], EXIT_CODE_SUCCESS);
    assert_metadata_shape(&status["metadata"]);

    let summary = read_artifact(&dir.join(SUMMARY_TXT_FILE));
    assert_eq!(
        summary,
        String::from_utf8_lossy(&output.stderr),
        "summary.txt should byte-match stderr summary output"
    );
    assert!(summary.contains("Status: warning-only"), "artifact summary should report warning-only: {summary}");
    assert!(summary.contains("Findings: 3"), "artifact summary should report total findings: {summary}");

    let command = read_artifact(&dir.join(COMMAND_TXT_FILE));
    let fixture_log = fixture.log();
    let launcher = launcher_path_from_log(&fixture_log);
    let expected_command = format!("{launcher} check --output-format sarif --artifact-dir {}\n", dir.display());
    assert_eq!(command, expected_command, "command.txt should store the exact shell-ready command line");
    let sh_parse_status = Command::new("sh")
        .arg("-n")
        .arg("-c")
        .arg(command.trim_end())
        .status()
        .expect("parse command.txt with sh -n");
    assert!(sh_parse_status.success(), "command.txt should be valid POSIX sh syntax: {command}");

    let sarif = read_json_artifact(&dir.join(RESULTS_SARIF_FILE));
    assert_eq!(sarif["runs"][0]["properties"]["status"], status["status"]);
    assert_eq!(
        sarif["runs"][0]["properties"]["exitCode"],
        output.status.code().expect("structured artifact exit code")
    );
    assert_eq!(sarif["runs"][0]["properties"]["totalFindings"], status["total_findings"]);
    assert_eq!(sarif["runs"][0]["properties"]["cargoProcessExitCode"], status["cargo_process_exit"]["code"]);
}

#[test]
fn artifact_directory_captures_json_run() {
    let fixture = ConsumerFixture::new("artifact-dir-json", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let dir = artifact_dir(&fixture, "json-run");
    let stale_extra_file = dir.join("extra.txt");
    let stale_sarif_file = dir.join(RESULTS_SARIF_FILE);
    write_file(&stale_extra_file, "stale extra sidecar\n");
    write_file(&stale_sarif_file, "stale sarif sidecar\n");
    let output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
        ],
        &three_finding_json_lines(),
        EXIT_CODE_SUCCESS,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "json artifact run should exit 0 because findings are warning-only: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        artifact_file_names(&dir),
        expected_artifact_files(JSON_OUTPUT_FORMAT),
        "json artifact directory should contain exactly the expected files"
    );
    assert_provenance_matches_artifacts(&dir, JSON_OUTPUT_FORMAT);
    assert!(
        !stale_extra_file.exists(),
        "runner should remove stale extra sidecars before writing the final bundle"
    );
    assert!(
        !stale_sarif_file.exists(),
        "runner should remove stale non-selected structured sidecars before writing the final bundle"
    );

    let status = read_json_artifact(&dir.join(STATUS_JSON_FILE));
    assert_eq!(status["status"], STATUS_WARNING_ONLY);
    assert_eq!(status["exit_code"], output.status.code().expect("json artifact exit code"));
    assert_eq!(status["output_format"], JSON_OUTPUT_FORMAT);
    assert_eq!(status["total_findings"], 3);
    assert_eq!(status["warning_findings"], 3);
    assert_eq!(status["error_findings"], 0);
    assert_eq!(status["autofixable_findings"], 0);
    assert_eq!(status["cargo_process_exit"]["classification"], "success");
    assert_eq!(status["cargo_process_exit"]["code"], EXIT_CODE_SUCCESS);
    assert_metadata_shape(&status["metadata"]);

    let summary = read_artifact(&dir.join(SUMMARY_TXT_FILE));
    assert_eq!(
        summary,
        String::from_utf8_lossy(&output.stderr),
        "json summary.txt should byte-match stderr summary output"
    );
    assert!(
        summary.contains("Status: warning-only"),
        "json artifact summary should report warning-only: {summary}"
    );

    let json_lines = read_artifact(&dir.join(RESULTS_JSONL_FILE));
    assert_eq!(
        json_lines,
        String::from_utf8_lossy(&output.stdout),
        "results.jsonl should byte-match the emitted JSON stdout stream"
    );
    for line in json_lines.lines() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "artifact JSON line should stay valid on success: {line}"
        );
    }
}

#[test]
fn artifact_directory_records_integration_failure_cleanly() {
    let fixture = ConsumerFixture::new("artifact-dir-error", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let dir = artifact_dir(&fixture, "integration-failure");
    let lines = vec![
        cargo_compiler_message("consumer", "no_unwrap", "use of `.unwrap()`", "src/lib.rs", 10, 5, None),
        cargo_build_finished(false),
    ];
    let output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
        ],
        &lines,
        EXIT_CODE_LINT_FAILURE,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_INTEGRATION_FAILURE),
        "integration-failure artifact run should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        artifact_file_names(&dir),
        expected_artifact_files(JSON_OUTPUT_FORMAT),
        "integration-failure artifact directory should contain exactly the expected files"
    );
    assert_provenance_matches_artifacts(&dir, JSON_OUTPUT_FORMAT);

    let status = read_json_artifact(&dir.join(STATUS_JSON_FILE));
    assert_eq!(
        sorted_object_keys(&status),
        vec![
            "autofixable_findings".to_owned(),
            "cargo_process_exit".to_owned(),
            "error_findings".to_owned(),
            "exit_code".to_owned(),
            "metadata".to_owned(),
            "output_format".to_owned(),
            "status".to_owned(),
            "total_findings".to_owned(),
            "warning_findings".to_owned(),
        ],
        "integration-failure status.json should keep the exact top-level key set"
    );
    assert_eq!(sorted_object_keys(&status["cargo_process_exit"]), vec![
        "classification".to_owned(),
        "code".to_owned()
    ]);
    assert_eq!(status["status"], STATUS_INTEGRATION_FAILURE);
    assert_eq!(status["exit_code"], output.status.code().expect("integration-failure exit code"));
    assert_eq!(status["cargo_process_exit"]["classification"], "exit-code");
    assert_eq!(status["cargo_process_exit"]["code"], EXIT_CODE_LINT_FAILURE);
    assert_metadata_shape(&status["metadata"]);

    let summary = read_artifact(&dir.join(SUMMARY_TXT_FILE));
    assert_eq!(
        summary,
        String::from_utf8_lossy(&output.stderr),
        "integration-failure summary.txt should byte-match stderr summary output"
    );
    assert!(
        summary.contains("Status: integration-failure"),
        "integration-failure summary should report integration-failure: {summary}"
    );

    let json_lines = read_artifact(&dir.join(RESULTS_JSONL_FILE));
    assert_eq!(
        json_lines,
        String::from_utf8_lossy(&output.stdout),
        "results.jsonl should byte-match the emitted JSON stdout stream"
    );
    for line in json_lines.lines() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "artifact JSON line should stay valid on integration failure: {line}"
        );
    }
}

#[test]
fn artifact_directory_captures_human_run() {
    let fixture = ConsumerFixture::new("artifact-dir-human", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let dir = artifact_dir(&fixture, "human-run");
    let output = run_with_check_args(
        &fixture,
        &["--artifact-dir", dir.to_str().expect("artifact dir utf8")],
        &three_finding_json_lines(),
        EXIT_CODE_SUCCESS,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "human artifact run should exit 0 for warning-only findings: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        artifact_file_names(&dir),
        expected_artifact_files("human"),
        "human artifact directory should omit structured-output sidecars"
    );
    assert_provenance_matches_artifacts(&dir, "human");
    assert!(output.stdout.is_empty(), "human artifact run should not emit stdout: {:?}", output.stdout);

    let status = read_json_artifact(&dir.join(STATUS_JSON_FILE));
    assert_eq!(status["output_format"], "human");
    assert_eq!(status["exit_code"], output.status.code().expect("human artifact exit code"));

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("warning[no_unwrap]"),
        "human artifact run should preserve rendered diagnostics: {stderr}"
    );
    let summary = read_artifact(&dir.join(SUMMARY_TXT_FILE));
    assert_eq!(summary, trailing_summary(&stderr), "human summary.txt should match trailing stderr summary block");
}

#[test]
fn strict_profile_bundle_binds_profile_identity_and_passes_artifact_verification() {
    let fixture = ConsumerFixture::new("strict-profile-artifact", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let dir = artifact_dir(&fixture, "strict-profile-run");
    let output = run_with_check_args(
        &fixture,
        &[
            "--profile",
            "robust",
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
        ],
        &[cargo_build_finished(true)],
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "strict-profile bundle should be emitted: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let status = read_json_artifact(&dir.join(STATUS_JSON_FILE));
    assert_eq!(status["metadata"]["profile_name"], "robust");
    assert!(status["metadata"]["profile_hash"].as_str().expect("profile hash").starts_with("b3:"));
    assert!(status["metadata"]["config_hash"].as_str().expect("config hash").starts_with("b3:"));
    let summary = read_artifact(&dir.join(SUMMARY_TXT_FILE));
    assert!(summary.contains("Profile: robust"));
    let function_addresses = read_json_artifact(&dir.join("function-addresses.json"));
    assert_eq!(function_addresses["mode"], "observe");
    let provenance = read_artifact(&dir.join(PROVENANCE_JSONL_FILE));
    for line in provenance.lines() {
        let record: serde_json::Value = serde_json::from_str(line).expect("profile provenance record");
        assert_eq!(record["checked_scope"], "profile:robust");
    }

    let bin = built_cargo_octet_bin().expect("build cargo-octet binary");
    let verify = Command::new(&bin)
        .args([
            "artifact",
            "verify",
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
            "--output-format",
            JSON_OUTPUT_FORMAT,
        ])
        .current_dir(&fixture.repo)
        .output()
        .expect("verify strict-profile bundle");
    assert_eq!(
        verify.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "strict-profile bundle should verify: stdout={} stderr={}",
        String::from_utf8_lossy(&verify.stdout),
        String::from_utf8_lossy(&verify.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&verify.stdout).expect("artifact verification report");
    assert_eq!(report["status"], "valid");
}

#[test]
fn artifact_verifier_accepts_real_bundle_and_rejects_tamper_without_mutation() {
    let fixture = ConsumerFixture::new("artifact-verifier", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let dir = artifact_dir(&fixture, "verifier-json-run");
    let output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
        ],
        &three_finding_json_lines(),
        EXIT_CODE_SUCCESS,
    );
    assert_eq!(output.status.code(), Some(EXIT_CODE_SUCCESS));

    let bin = built_cargo_octet_bin().expect("build cargo-octet binary");
    let valence_export = fixture.repo.join("valence-provenance.json");
    let verify = Command::new(&bin)
        .args([
            "artifact",
            "verify",
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--emit-valence-provenance",
            valence_export.to_str().expect("valence export path utf8"),
        ])
        .current_dir(&fixture.repo)
        .output()
        .expect("run artifact verifier");
    assert_eq!(
        verify.status.code(),
        Some(EXIT_CODE_SUCCESS),
        "complete bundle should verify: stdout={} stderr={}",
        String::from_utf8_lossy(&verify.stdout),
        String::from_utf8_lossy(&verify.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&verify.stdout).expect("artifact verifier json");
    assert_eq!(report["status"], "valid");
    assert_eq!(report["output_format"], JSON_OUTPUT_FORMAT);
    assert!(report["diagnostics"].as_array().expect("diagnostics").is_empty());
    let export_json: serde_json::Value =
        serde_json::from_slice(&fs::read(&valence_export).expect("read Valence export")).expect("Valence export json");
    assert_eq!(export_json["schema_version"], "octet-valence-provenance-export/v1");
    assert!(
        export_json["provenance_records"]
            .as_array()
            .expect("provenance records")
            .iter()
            .any(|record| { record["artifact"] == RESULTS_JSONL_FILE && record["evidence_tier"] == "static_lint" })
    );

    let files_before_tamper = artifact_file_names(&dir);
    let tampered_path = dir.join(RESULTS_JSONL_FILE);
    let tampered_contents = format!("{}tamper\n", read_artifact(&tampered_path));
    write_file(&tampered_path, &tampered_contents);
    let tampered_before_verify = fs::read(&tampered_path).expect("read tampered artifact before verify");

    let tampered = Command::new(&bin)
        .args([
            "artifact",
            "verify",
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
        ])
        .current_dir(&fixture.repo)
        .output()
        .expect("run artifact verifier on tampered bundle");
    assert_eq!(
        tampered.status.code(),
        Some(EXIT_CODE_LINT_FAILURE),
        "tampered bundle should fail verification: stdout={} stderr={}",
        String::from_utf8_lossy(&tampered.stdout),
        String::from_utf8_lossy(&tampered.stderr)
    );
    let tampered_stdout = String::from_utf8_lossy(&tampered.stdout);
    assert!(
        tampered_stdout.contains("hash mismatch for artifact locator `results.jsonl`"),
        "tampered verifier output should name hash mismatch: {tampered_stdout}"
    );
    assert_eq!(artifact_file_names(&dir), files_before_tamper, "verifier must not mutate file set");
    assert_eq!(
        fs::read(&tampered_path).expect("read tampered artifact after verify"),
        tampered_before_verify,
        "verifier must not rewrite artifact contents"
    );
}

#[test]
fn partial_artifact_bundle_is_invalid() {
    let tmp = TempDir::new("partial-artifact-bundle");

    let human_dir = tmp.path().join("human");
    fs::create_dir_all(&human_dir).expect("create partial human dir");
    write_file(&human_dir.join(COMMAND_TXT_FILE), "cmd\n");
    write_file(&human_dir.join(STATUS_JSON_FILE), "{}\n");
    assert!(!artifact_bundle_is_valid(&human_dir, "human"), "human bundle missing summary must be invalid");
    write_file(&human_dir.join(SUMMARY_TXT_FILE), "summary\n");
    assert!(!artifact_bundle_is_valid(&human_dir, "human"), "human bundle missing provenance must be invalid");
    write_file(&human_dir.join(PROVENANCE_JSONL_FILE), "{}\n");
    assert!(artifact_bundle_is_valid(&human_dir, "human"), "complete human bundle should be valid");
    write_file(&human_dir.join("extra.txt"), "extra\n");
    assert!(!artifact_bundle_is_valid(&human_dir, "human"), "human bundle with extra sidecar must be invalid");

    let json_dir = tmp.path().join("json");
    fs::create_dir_all(&json_dir).expect("create partial json dir");
    write_file(&json_dir.join(COMMAND_TXT_FILE), "cmd\n");
    write_file(&json_dir.join(STATUS_JSON_FILE), "{}\n");
    write_file(&json_dir.join(SUMMARY_TXT_FILE), "summary\n");
    write_file(&json_dir.join(PROVENANCE_JSONL_FILE), "{}\n");
    assert!(
        !artifact_bundle_is_valid(&json_dir, JSON_OUTPUT_FORMAT),
        "json bundle missing results.jsonl must be invalid"
    );
    write_file(&json_dir.join(RESULTS_JSONL_FILE), "{}\n");
    assert!(artifact_bundle_is_valid(&json_dir, JSON_OUTPUT_FORMAT), "complete json bundle should be valid");
    write_file(&json_dir.join("extra.txt"), "extra\n");
    assert!(
        !artifact_bundle_is_valid(&json_dir, JSON_OUTPUT_FORMAT),
        "json bundle with extra sidecar must be invalid"
    );

    let sarif_dir = tmp.path().join("sarif");
    fs::create_dir_all(&sarif_dir).expect("create partial sarif dir");
    write_file(&sarif_dir.join(COMMAND_TXT_FILE), "cmd\n");
    write_file(&sarif_dir.join(STATUS_JSON_FILE), "{}\n");
    write_file(&sarif_dir.join(SUMMARY_TXT_FILE), "summary\n");
    write_file(&sarif_dir.join(PROVENANCE_JSONL_FILE), "{}\n");
    assert!(
        !artifact_bundle_is_valid(&sarif_dir, SARIF_OUTPUT_FORMAT),
        "sarif bundle missing results.sarif must be invalid"
    );
    write_file(&sarif_dir.join(RESULTS_SARIF_FILE), "{}\n");
    assert!(artifact_bundle_is_valid(&sarif_dir, SARIF_OUTPUT_FORMAT), "complete sarif bundle should be valid");
    write_file(&sarif_dir.join("extra.txt"), "extra\n");
    assert!(
        !artifact_bundle_is_valid(&sarif_dir, SARIF_OUTPUT_FORMAT),
        "sarif bundle with extra sidecar must be invalid"
    );
}

#[test]
fn artifact_directory_mid_write_failure_exits_two_and_invalidates_bundle() {
    let fixture =
        ConsumerFixture::new("artifact-dir-mid-write-failure", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let dir = artifact_dir(&fixture, "mid-write-failure");
    fs::create_dir_all(dir.join(RESULTS_SARIF_FILE)).expect("create blocking results.sarif directory");
    let lines = vec![
        cargo_compiler_message("consumer", "no_unwrap", "use of `.unwrap()`", "src/lib.rs", 10, 5, None),
        cargo_build_finished(true),
    ];
    let output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            SARIF_OUTPUT_FORMAT,
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
        ],
        &lines,
        EXIT_CODE_SUCCESS,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_INTEGRATION_FAILURE),
        "mid-write artifact failure should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let summary_index = stderr.find("--- octet summary ---").expect("summary header in stderr");
    let error_index = stderr.find("error:").expect("artifact write error in stderr");
    assert!(
        summary_index < error_index,
        "late artifact write error should happen after summary emission: {stderr}"
    );
    assert!(
        stderr.contains("Status: integration-failure"),
        "artifact write failure summary should report integration-failure: {stderr}"
    );
    assert!(
        !output.stdout.is_empty(),
        "structured stdout should still be emitted after artifact failure finalization"
    );
    assert!(
        !artifact_bundle_is_valid(&dir, SARIF_OUTPUT_FORMAT),
        "missing required results.sarif file must invalidate the bundle"
    );

    let status = read_json_artifact(&dir.join(STATUS_JSON_FILE));
    assert_eq!(status["status"], STATUS_INTEGRATION_FAILURE);
    assert_eq!(status["exit_code"], EXIT_CODE_INTEGRATION_FAILURE);
    assert_eq!(status["cargo_process_exit"]["classification"], "success");
    assert_eq!(status["cargo_process_exit"]["code"], EXIT_CODE_SUCCESS);

    let sarif: serde_json::Value = serde_json::from_slice(&output.stdout).expect("parse emitted SARIF stdout");
    assert_eq!(sarif["runs"][0]["invocations"][0]["exitCode"], EXIT_CODE_INTEGRATION_FAILURE);
    assert_eq!(sarif["runs"][0]["properties"]["status"], STATUS_INTEGRATION_FAILURE);
    assert_eq!(sarif["runs"][0]["properties"]["exitCode"], EXIT_CODE_INTEGRATION_FAILURE);
    assert_eq!(sarif["runs"][0]["properties"]["cargoProcessExitClassification"], "success");
    assert_eq!(sarif["runs"][0]["properties"]["cargoProcessExitCode"], EXIT_CODE_SUCCESS);
}

#[test]
fn artifact_directory_setup_failure_exits_two() {
    let fixture =
        ConsumerFixture::new("artifact-dir-setup-failure", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let artifact_target = fixture.repo.join("occupied-artifact-path");
    write_file(&artifact_target, "occupied\n");
    let lines = vec![cargo_build_finished(true)];
    let output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            SARIF_OUTPUT_FORMAT,
            "--artifact-dir",
            artifact_target.to_str().expect("artifact target utf8"),
        ],
        &lines,
        EXIT_CODE_SUCCESS,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_INTEGRATION_FAILURE),
        "artifact-dir setup failure should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    let summary_index = stderr.find("--- octet summary ---").expect("summary header in stderr");
    let error_index = stderr.find("error:").expect("artifact-dir error in stderr");
    assert!(summary_index < error_index, "artifact-dir error should be reported after the summary: {stderr}");
    assert!(
        stderr.contains("Status: integration-failure"),
        "artifact-dir setup failure should finalize as integration-failure: {stderr}"
    );
    assert!(
        !artifact_target.join(STATUS_JSON_FILE).exists(),
        "invalid artifact path must not produce a status sidecar path"
    );

    let sarif: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse emitted setup-failure SARIF stdout");
    assert_eq!(sarif["runs"][0]["invocations"][0]["exitCode"], EXIT_CODE_INTEGRATION_FAILURE);
    assert_eq!(sarif["runs"][0]["properties"]["status"], STATUS_INTEGRATION_FAILURE);
    assert_eq!(sarif["runs"][0]["properties"]["exitCode"], EXIT_CODE_INTEGRATION_FAILURE);
    assert_eq!(sarif["runs"][0]["properties"]["cargoProcessExitClassification"], "success");
    assert_eq!(sarif["runs"][0]["properties"]["cargoProcessExitCode"], EXIT_CODE_SUCCESS);
}

#[test]
fn terminated_child_artifact_records_terminated_classification() {
    let fixture = ConsumerFixture::new("terminated-artifact", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let dir = artifact_dir(&fixture, "terminated");
    let output = fixture.run_with_launcher_and_env(
        None,
        &[("TS_KILL_SELF", "1")],
        &[
            "--output-format",
            JSON_OUTPUT_FORMAT,
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
        ],
        EXIT_CODE_SUCCESS,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_INTEGRATION_FAILURE),
        "terminated cargo child should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let status = read_json_artifact(&dir.join(STATUS_JSON_FILE));
    assert_eq!(status["cargo_process_exit"]["classification"], "terminated");
    assert!(
        status["cargo_process_exit"]["code"].is_null(),
        "terminated cargo child should store null code: {status}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("cargo check terminated without an exit code"),
        "terminated cargo child should preserve the root-cause error: {stderr}"
    );
}

#[test]
fn sarif_carries_invocation_metadata() {
    let fixture = ConsumerFixture::new("sarif-invocation", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let dir = artifact_dir(&fixture, "sarif-invocation");
    let lines = vec![
        cargo_compiler_message_with_level(
            "consumer",
            "no_unwrap",
            ERROR_LEVEL,
            "use of `.unwrap()`",
            "src/lib.rs",
            10,
            5,
            None,
        ),
        cargo_build_finished(true),
    ];
    let output = run_with_check_args(
        &fixture,
        &[
            "--output-format",
            SARIF_OUTPUT_FORMAT,
            "--artifact-dir",
            dir.to_str().expect("artifact dir utf8"),
        ],
        &lines,
        EXIT_CODE_SUCCESS,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_LINT_FAILURE),
        "lint-failure SARIF run should exit 1: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let status = read_json_artifact(&dir.join(STATUS_JSON_FILE));
    let sarif = read_json_artifact(&dir.join(RESULTS_SARIF_FILE));
    let invocation = &sarif["runs"][0]["invocations"][0];
    assert!(
        invocation["commandLine"].as_str().unwrap().contains("--output-format sarif"),
        "SARIF invocation should record the command line"
    );
    assert_eq!(invocation["exitCode"], output.status.code().expect("sarif invocation exit code"));
    assert_eq!(
        sorted_object_keys(&sarif["runs"][0]["properties"]),
        vec![
            "autofixableFindings".to_owned(),
            "cargoProcessExitClassification".to_owned(),
            "cargoProcessExitCode".to_owned(),
            "errorFindings".to_owned(),
            "exitCode".to_owned(),
            "metadata".to_owned(),
            "status".to_owned(),
            "totalFindings".to_owned(),
            "warningFindings".to_owned(),
        ],
        "SARIF run properties should use the exact key set"
    );
    assert_eq!(sarif["runs"][0]["properties"]["status"], status["status"]);
    assert_eq!(sarif["runs"][0]["properties"]["exitCode"], output.status.code().expect("sarif property exit code"));
    assert_eq!(
        sarif["runs"][0]["properties"]["cargoProcessExitClassification"],
        status["cargo_process_exit"]["classification"]
    );
    assert_eq!(sarif["runs"][0]["properties"]["cargoProcessExitCode"], status["cargo_process_exit"]["code"]);
}

#[test]
fn finding_indices_are_unique_and_monotonic() {
    let fixture = ConsumerFixture::new("finding-index", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = run_with_output_format(&fixture, "json", &three_finding_json_lines(), 0);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let findings: Vec<serde_json::Value> =
        stdout.lines().map(|line| serde_json::from_str(line).expect("parse JSON finding")).collect();

    let indices: Vec<&str> = findings.iter().map(|f| f["index"].as_str().unwrap()).collect();
    assert_eq!(indices, vec!["F1", "F2", "F3"], "indices should be F1, F2, F3 in order");

    // Check uniqueness
    let mut unique = indices.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(unique.len(), indices.len(), "all indices must be unique");
}

#[test]
fn default_human_mode_preserves_diagnostics_on_stderr_with_summary() {
    let fixture = ConsumerFixture::new("default-human", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let json_lines = three_finding_json_lines();
    let stdout_file = write_json_stdout_file(&fixture, &json_lines);
    let output = fixture.run_with_launcher_and_env(
        None,
        &[("TS_STDOUT_FILE", stdout_file.to_str().unwrap())],
        &[], // no --output-format flag = human default
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );

    assert_eq!(output.status.code(), Some(EXIT_CODE_SUCCESS), "warning-only findings should exit 0");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // (a) stderr has human-readable diagnostics
    assert!(
        stderr.contains("warning[no_unwrap]"),
        "human mode should show rendered diagnostics on stderr: {stderr}"
    );

    // (b) stderr has trailing summary
    assert!(stderr.contains("--- octet summary ---"), "human mode should append summary: {stderr}");

    // (c) no structured output on stdout
    assert!(stdout.is_empty(), "human mode should produce no stdout output: {stdout}");
}

#[test]
fn invalid_output_format_rejected() {
    let fixture = ConsumerFixture::new("invalid-format", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = fixture.run(
        &["--output-format", "xml"],
        EXIT_CODE_SUCCESS,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_INTEGRATION_FAILURE),
        "invalid format should exit 2: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unsupported output format"), "should report unsupported format: {stderr}");
}

#[test]
fn missing_artifact_dir_value_is_rejected() {
    let fixture = ConsumerFixture::new("missing-artifact-dir", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = fixture.run(
        &["--artifact-dir"],
        EXIT_CODE_SUCCESS,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_INTEGRATION_FAILURE),
        "missing artifact dir should exit 2: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("missing value after `--artifact-dir`"),
        "missing artifact dir diagnostic should mention the flag: {stderr}"
    );
}

#[test]
fn compilation_error_during_json_output_produces_valid_json() {
    let fixture = ConsumerFixture::new("error-json", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![
        cargo_compiler_message("consumer", "no_unwrap", "use of `.unwrap()`", "src/lib.rs", 10, 5, None),
        cargo_build_finished(false),
    ];
    let output = run_with_output_format(&fixture, JSON_OUTPUT_FORMAT, &lines, EXIT_CODE_LINT_FAILURE);

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_INTEGRATION_FAILURE),
        "error path should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        assert!(
            serde_json::from_str::<serde_json::Value>(line).is_ok(),
            "stdout line is not valid JSON during error path: {line}"
        );
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--- octet summary ---"), "summary should print even during error: {stderr}");
    assert!(
        stderr.contains("Status: integration-failure"),
        "summary should report integration-failure on compile error: {stderr}"
    );
}

#[test]
fn compilation_error_during_sarif_output_produces_valid_sarif() {
    let fixture = ConsumerFixture::new("error-sarif", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let lines = vec![
        cargo_compiler_message("consumer", "no_unwrap", "use of `.unwrap()`", "src/lib.rs", 10, 5, None),
        cargo_build_finished(false),
    ];
    let output = run_with_output_format(&fixture, SARIF_OUTPUT_FORMAT, &lines, EXIT_CODE_LINT_FAILURE);

    assert_eq!(
        output.status.code(),
        Some(EXIT_CODE_INTEGRATION_FAILURE),
        "error path should exit 2: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let sarif: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("SARIF must be valid JSON even on error path: {error}\nstdout: {stdout}"));
    assert_eq!(sarif["version"], "2.1.0", "SARIF version must be present on error path");
    assert!(sarif["runs"][0]["results"].as_array().is_some(), "SARIF results must be present on error path");
    assert_eq!(sarif["runs"][0]["properties"]["status"], STATUS_INTEGRATION_FAILURE);
    assert_eq!(sarif["runs"][0]["invocations"][0]["exitCode"], EXIT_CODE_INTEGRATION_FAILURE);

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--- octet summary ---"), "summary should print even during error: {stderr}");
    assert!(
        stderr.contains("Status: integration-failure"),
        "summary should report integration-failure on compile error: {stderr}"
    );
}

#[test]
fn conflicting_message_format_in_passthrough_args_rejected() {
    let fixture = ConsumerFixture::new("msg-format-conflict", &metadata(r#"["-p", "consumer"]"#, r#"["--lib"]"#));
    let output = fixture.run(
        &["--", "--message-format=short"],
        0,
        false,
        Some("nightly-2026-03-21-x86_64-unknown-linux-gnu"),
        Some(fixture.driver_path.as_path()),
        Some(fixture.lint_lib_path.as_path()),
        None,
    );

    assert_eq!(
        output.status.code(),
        Some(2),
        "conflicting --message-format should exit 2: stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--message-format"), "should mention --message-format in error: {stderr}");
}
