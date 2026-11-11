use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

use octet_standards::model::Classification;
use octet_standards::model::EnforcementLevel;
use octet_standards::model::EnforcementMode;
use octet_standards::model::EvidenceCommand;
use octet_standards::model::FixtureCoverage;
use octet_standards::model::FixtureStatus;
use octet_standards::model::Mechanism;
use octet_standards::model::MechanismKind;
use octet_standards::model::PolicyEntry;
use octet_standards::model::PolicyRegistry;
use octet_standards::model::ProofRail;
use octet_standards::model::ReceiptProvenance;
use octet_standards::model::ReceiptVerifier;
use octet_standards::model::Severity;
use octet_standards::model::SuppressionRequirements;
use octet_standards::report::CheckReport;
use pretty_assertions::assert_eq;
use tempfile::TempDir;

const FCIS_POLICY_ID: &str = "tigerstyle.architecture.fcis-crate-boundary";
const ROOT_THINNESS_POLICY_ID: &str = "tigerstyle.architecture.root-crate-thinness";
const PUBLIC_API_BUDGET_POLICY_ID: &str = "tigerstyle.architecture.public-api-budget";
const PROOF_POLICY_ID: &str = "tigerstyle.verification.deterministic-effect-handlers";
const ARCHITECTURE_FAMILY: &str = "architecture";
const VERIFICATION_FAMILY: &str = "verification";
const CHECKER_ID: &str = "octet-standards";
const REGISTRY_FILE_NAME: &str = "registry.json";
const POLICIES_JSON_FILE: &str = "octet-policies.json";
const CHECK_CONFIG_TOML_FILE: &str = "octet-check-config.toml";
const SEED_MANIFESTS_JSON_FILE: &str = "octet-seed-manifests.json";
const PROOF_BUNDLES_JSON_FILE: &str = "octet-proof-bundles.json";
const CONFIG_RECEIPT_FAMILIES_JSON_FILE: &str = "octet-config-receipt-families.json";
const ARTIFACT_MANIFEST_JSON_FILE: &str = "octet-artifact-manifest.json";
const PROOF_BUNDLE_SCHEMA_VERSION: &str = "octet-proof-bundle/v1";
const ARTIFACT_MANIFEST_SCHEMA_VERSION: &str = "octet-artifact-manifest/v1";
const ADVISORY_FIXTURE: &str = "standards/check-fixtures/architecture/advisory/public-api-budget";
const HARD_FAILURE_FIXTURE: &str = "standards/check-fixtures/architecture/noncompliant/missing-no-std";
const OCTET_LAYOUT_METADATA_FIXTURE: &str =
    "standards/check-fixtures/architecture/dependency-direction/octet-layout.json";
const EXPECTED_BINARY_DECLARATION_COUNT: usize = 1;
const STANDARDS_ALIAS_BINARY_COUNT: usize = 3;
const EXPECTED_STANDARDS_VERSION_PREFIX: &str = "octet-standards ";
const STANDARDS_HELP_NEEDLE: &str = "Run Octet policy checks";

#[test]
fn octet_identity_surfaces() {
    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let manifest = std::fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("read {}: {error}", manifest_path.display()));

    assert!(manifest.contains("[package]\nname = \"octet-standards\""));
    assert!(manifest.contains("[lib]\nname = \"octet_standards\""));
    assert_eq!(
        manifest.matches("[[bin]]\nname = \"octet-standards\"\npath = \"src/main.rs\"").count(),
        EXPECTED_BINARY_DECLARATION_COUNT,
        "canonical standards binary must be declared exactly once"
    );
    assert_eq!(
        manifest
            .matches("[[bin]]\nname = \"slotcar-standards\"\npath = \"src/bin/slotcar-standards.rs\"")
            .count(),
        EXPECTED_BINARY_DECLARATION_COUNT,
        "legacy Slotcar standards binary remains one explicit compatibility alias"
    );
    assert_eq!(
        manifest
            .matches("[[bin]]\nname = \"tigerstyle-standards\"\npath = \"src/bin/tigerstyle-standards.rs\"")
            .count(),
        EXPECTED_BINARY_DECLARATION_COUNT,
        "Tiger Style standards binary remains one explicit compatibility alias"
    );
    assert!(!manifest.contains("[package]\nname = \"slotcar-standards\""));
    assert!(!manifest.contains("[lib]\nname = \"slotcar_standards\""));

    assert!(env!("CARGO_BIN_EXE_octet-standards").ends_with("octet-standards"));
    assert!(env!("CARGO_BIN_EXE_slotcar-standards").ends_with("slotcar-standards"));
    assert!(env!("CARGO_BIN_EXE_tigerstyle-standards").ends_with("tigerstyle-standards"));
}

#[test]
fn standards_alias_bins_preserve_help_and_version() {
    for (name, binary) in standards_alias_bins() {
        let version = Command::new(binary).arg("--version").output().expect("version command should run");
        assert_success(&version);
        let version_stdout = String::from_utf8_lossy(&version.stdout);
        assert!(
            version_stdout.starts_with(EXPECTED_STANDARDS_VERSION_PREFIX),
            "{name} version output changed: {version_stdout}"
        );

        let help = Command::new(binary).arg("--help").output().expect("help command should run");
        assert_success(&help);
        assert!(String::from_utf8_lossy(&help.stdout).contains(STANDARDS_HELP_NEEDLE));
    }
}

#[test]
fn advisory_only_cli_report_exits_successfully() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);

    let metadata = write_octet_layout_metadata(&temp);
    let output = run_architecture_check(&registry, &[&repo_path(ADVISORY_FIXTURE), &metadata]);
    assert_success(&output);
    let report = parse_report(&output);

    assert_eq!(report.diagnostics.len(), 1);
    let diagnostic = &report.diagnostics[0];
    assert_eq!(diagnostic.policy_id, PUBLIC_API_BUDGET_POLICY_ID);
    assert_eq!(diagnostic.severity, Severity::Warning);
    assert!(diagnostic.source_section.contains("when-boundaries-are-too-tight"));
    assert!(report.is_advisory_only());
    assert!(!report.has_blocking_diagnostics());
}

#[test]
fn hard_policy_cli_report_exits_with_failure() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);

    let output = run_architecture_check(&registry, &[&repo_path(HARD_FAILURE_FIXTURE)]);
    assert_failure(&output);
    let report = parse_report(&output);

    assert!(report.has_blocking_diagnostics());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.policy_id == FCIS_POLICY_ID && diagnostic.severity == Severity::Error)
    );
}

#[test]
fn check_rejects_unknown_family() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(repo_path(ADVISORY_FIXTURE))
        .arg("--family")
        .arg("architecure")
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("unknown --family value(s): architecure"), "stderr was: {stderr}");
    assert!(stderr.contains(ARCHITECTURE_FAMILY), "stderr should list known families: {stderr}");
}

#[test]
fn check_rejects_selection_without_policies() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut empty_registry = proof_registry();
    empty_registry.policies.clear();
    let registry = write_registry_value(&temp, &empty_registry);

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(repo_path(ADVISORY_FIXTURE))
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("evaluates zero policies"), "stderr was: {stderr}");
}

#[test]
fn exported_policy_artifacts_drive_checker() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let artifact_dir = temp.path().join("generated");

    let export = Command::new(binary_path())
        .arg("export-artifacts")
        .arg("--registry")
        .arg(&registry)
        .arg("--out-dir")
        .arg(&artifact_dir)
        .output()
        .expect("artifact export should run");
    assert_success(&export);

    assert!(artifact_dir.join(POLICIES_JSON_FILE).is_file());
    assert!(artifact_dir.join(CHECK_CONFIG_TOML_FILE).is_file());
    assert!(artifact_dir.join(SEED_MANIFESTS_JSON_FILE).is_file());
    assert!(artifact_dir.join(PROOF_BUNDLES_JSON_FILE).is_file());
    assert!(artifact_dir.join(ARTIFACT_MANIFEST_JSON_FILE).is_file());

    let manifest = std::fs::read_to_string(artifact_dir.join(ARTIFACT_MANIFEST_JSON_FILE))
        .expect("artifact manifest should be readable");
    let manifest: serde_json::Value = serde_json::from_str(&manifest).expect("artifact manifest should parse as JSON");
    assert_eq!(manifest["schema_version"], ARTIFACT_MANIFEST_SCHEMA_VERSION);
    assert!(
        manifest["artifacts"].as_array().expect("manifest artifacts should be an array").iter().any(
            |entry| entry["path"] == POLICIES_JSON_FILE
                && entry["blake3"].as_str().is_some()
                && entry["bytes"].as_u64().is_some()
        )
    );

    let config =
        std::fs::read_to_string(artifact_dir.join(CHECK_CONFIG_TOML_FILE)).expect("check config should be readable");
    assert!(config.contains("hard_policy_ids"));
    assert!(config.contains(FCIS_POLICY_ID));

    let output = run_architecture_artifact_check(&artifact_dir, &repo_path(HARD_FAILURE_FIXTURE));
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.has_blocking_diagnostics());
}

#[test]
fn export_artifacts_emits_profile_bound_acceptance_receipt() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let artifact_dir = temp.path().join("generated-with-receipt");
    let receipt_path = temp.path().join("receipts/acceptance.json");
    let profile_path = temp.path().join("profile.json");
    let profile_json = test_profile_json();
    std::fs::write(&profile_path, profile_json).expect("profile JSON should be written");

    let output = Command::new(binary_path())
        .arg("export-artifacts")
        .arg("--registry")
        .arg(&registry)
        .arg("--out-dir")
        .arg(&artifact_dir)
        .arg("--profile-json")
        .arg(&profile_path)
        .arg("--receipt-out")
        .arg(&receipt_path)
        .arg("--verification-command")
        .arg("nickel export --format json standards/profiles/local.ncl")
        .output()
        .expect("artifact export should run");
    assert_success(&output);

    let receipt_text = std::fs::read_to_string(&receipt_path).expect("receipt should be readable");
    let receipt: serde_json::Value = serde_json::from_str(&receipt_text).expect("receipt should parse as JSON");
    assert_eq!(receipt["schema_version"], "octet-acceptance-receipt/v1");
    assert_eq!(receipt["status"], "passed");
    assert_eq!(receipt["config_digest"], blake3_hex(profile_json));
    assert_eq!(receipt["config_digest_detail"]["algorithm"], "blake3");
    assert_eq!(receipt["raw_log_path"], "");
    assert!(receipt.get("stdout").is_none());
    assert!(receipt.get("stderr").is_none());
    assert_receipt_artifact(&receipt, POLICIES_JSON_FILE, &artifact_dir);
    assert_receipt_artifact(&receipt, CHECK_CONFIG_TOML_FILE, &artifact_dir);
    assert_receipt_artifact(&receipt, CONFIG_RECEIPT_FAMILIES_JSON_FILE, &artifact_dir);
    assert_receipt_artifact(&receipt, ARTIFACT_MANIFEST_JSON_FILE, &artifact_dir);
    let verification_commands = receipt["verification_commands"].as_array().expect("commands should be an array");
    assert!(
        verification_commands[0]["command"]
            .as_str()
            .expect("command should be a string")
            .contains("nickel export")
    );
    assert_eq!(
        verification_commands[0]["status"], "recorded",
        "a caller-supplied command is recorded, not executed, by export-artifacts"
    );
    let provenance_records = receipt["provenance_records"].as_array().expect("provenance records array");
    assert!(!provenance_records.is_empty(), "receipt should carry generated-artifact provenance records");
    assert!(provenance_records.iter().any(|record| {
        record["checker_id"] == "octet-policy-export"
            && record["source_id"] == "octet.policy.generated-artifact"
            && record["evidence_tier"] == "policy_check"
            && record["artifact_locator"].as_str().unwrap_or_default().ends_with(ARTIFACT_MANIFEST_JSON_FILE)
            && !record["parent_artifacts"].as_array().expect("parent artifacts").is_empty()
    }));
}

#[test]
fn export_artifacts_receipt_without_recorded_commands_lists_its_own_passing_run() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let receipt_path = temp.path().join("receipts/acceptance.json");
    let profile_path = temp.path().join("profile.json");
    std::fs::write(&profile_path, test_profile_json()).expect("profile JSON should be written");

    let output = Command::new(binary_path())
        .arg("export-artifacts")
        .arg("--registry")
        .arg(&registry)
        .arg("--out-dir")
        .arg(temp.path().join("generated"))
        .arg("--profile-json")
        .arg(&profile_path)
        .arg("--receipt-out")
        .arg(&receipt_path)
        .output()
        .expect("artifact export should run");
    assert_success(&output);

    let receipt_text = std::fs::read_to_string(&receipt_path).expect("receipt should be readable");
    let receipt: serde_json::Value = serde_json::from_str(&receipt_text).expect("receipt should parse as JSON");
    let verification_commands = receipt["verification_commands"].as_array().expect("commands should be an array");
    assert_eq!(verification_commands.len(), 1);
    assert_eq!(verification_commands[0]["status"], "passed");
}

#[test]
fn export_artifacts_receipt_requires_profile_json() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let artifact_dir = temp.path().join("generated-without-profile");
    let receipt_path = temp.path().join("receipts/acceptance.json");

    let output = Command::new(binary_path())
        .arg("export-artifacts")
        .arg("--registry")
        .arg(&registry)
        .arg("--out-dir")
        .arg(&artifact_dir)
        .arg("--receipt-out")
        .arg(&receipt_path)
        .output()
        .expect("artifact export should run");
    assert_failure(&output);
    assert!(String::from_utf8_lossy(&output.stderr).contains("pass --profile-json when --receipt-out is set"));
}

#[test]
fn export_artifacts_receipt_rejects_non_blake3_profile_digest() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let artifact_dir = temp.path().join("generated-bad-profile");
    let receipt_path = temp.path().join("receipts/acceptance.json");
    let profile_path = temp.path().join("profile.json");
    std::fs::write(&profile_path, test_profile_json().replace("blake3", "sha256"))
        .expect("profile JSON should be written");

    let output = Command::new(binary_path())
        .arg("export-artifacts")
        .arg("--registry")
        .arg(&registry)
        .arg("--out-dir")
        .arg(&artifact_dir)
        .arg("--profile-json")
        .arg(&profile_path)
        .arg("--receipt-out")
        .arg(&receipt_path)
        .output()
        .expect("artifact export should run");
    assert_failure(&output);
    assert!(String::from_utf8_lossy(&output.stderr).contains("unsupported config digest algorithm `sha256`"));
}

#[test]
fn artifact_backed_manifest_rejects_tampered_generated_artifact() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let artifact_dir = temp.path().join("generated-tampered");
    export_artifacts(&registry, &artifact_dir);
    std::fs::write(artifact_dir.join(CHECK_CONFIG_TOML_FILE), "schema_version = \"tampered\"\n")
        .expect("tampered check config should be written");

    let output = run_architecture_artifact_check(&artifact_dir, &repo_path(HARD_FAILURE_FIXTURE));
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == "tigerstyle.generated-artifacts.integrity-manifest"
            && diagnostic.message.contains("artifact integrity mismatch")
            && diagnostic.message.contains(CHECK_CONFIG_TOML_FILE)
    }));
}

#[test]
fn artifact_backed_manifest_rejects_missing_manifest_file() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let artifact_dir = temp.path().join("generated-missing-manifest");
    export_artifacts(&registry, &artifact_dir);
    std::fs::remove_file(artifact_dir.join(ARTIFACT_MANIFEST_JSON_FILE)).expect("manifest should be removed");

    let output = run_architecture_artifact_check(&artifact_dir, &repo_path(HARD_FAILURE_FIXTURE));
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == "tigerstyle.generated-artifacts.integrity-manifest"
            && diagnostic.message.contains("missing or unreadable artifact integrity manifest")
    }));
}

#[test]
fn artifact_backed_manifest_rejects_missing_manifest_before_policy_json_parse() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let artifact_dir = temp.path().join("generated-corrupt-policy-missing-manifest");
    export_artifacts(&registry, &artifact_dir);
    std::fs::write(artifact_dir.join(POLICIES_JSON_FILE), "{ not valid policy json")
        .expect("corrupt policies artifact should be written");
    std::fs::remove_file(artifact_dir.join(ARTIFACT_MANIFEST_JSON_FILE)).expect("manifest should be removed");

    let output = run_architecture_artifact_check(&artifact_dir, &repo_path(HARD_FAILURE_FIXTURE));
    assert_failure(&output);
    assert!(
        output.stderr.is_empty(),
        "manifest-first validation should emit JSON diagnostics, not a CLI parse error: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == "tigerstyle.generated-artifacts.integrity-manifest"
            && diagnostic.message.contains("missing or unreadable artifact integrity manifest")
    }));
}

#[test]
fn exported_proof_bundle_preserves_verified_rail_schema() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let artifact_dir = temp.path().join("generated-proof-bundle");

    let export = Command::new(binary_path())
        .arg("export-artifacts")
        .arg("--registry")
        .arg(&registry)
        .arg("--out-dir")
        .arg(&artifact_dir)
        .output()
        .expect("artifact export should run");
    assert_success(&export);

    let proof_bundle =
        std::fs::read_to_string(artifact_dir.join(PROOF_BUNDLES_JSON_FILE)).expect("proof bundle should be readable");
    let value: serde_json::Value = serde_json::from_str(&proof_bundle).expect("proof bundle should parse as JSON");
    assert_eq!(value["schema_version"], PROOF_BUNDLE_SCHEMA_VERSION);
    assert_eq!(value["proof_bundles"][0]["policy_id"], PROOF_POLICY_ID);
    assert_eq!(value["proof_bundles"][0]["proof_rail"]["receipt_binding"], "blake3-command-and-source");
    assert_eq!(
        value["proof_bundles"][0]["proof_rail"]["receipt_verifier"]["verifier_nix_closure_graph_paths"][0],
        "*/proof-nix-closure.graph"
    );
}

#[test]
fn artifact_backed_proof_bundle_rejects_unsupported_schema_version() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let artifact_dir = temp.path().join("generated-proof-bundle");
    export_artifacts(&registry, &artifact_dir);
    let proof_bundle_path = artifact_dir.join(PROOF_BUNDLES_JSON_FILE);
    let proof_bundle = std::fs::read_to_string(&proof_bundle_path).expect("proof bundle should be readable");
    std::fs::write(&proof_bundle_path, proof_bundle.replace(PROOF_BUNDLE_SCHEMA_VERSION, "tigerstyle-proof-bundle/v0"))
        .expect("mutated proof bundle should be written");
    refresh_manifest_entry(&artifact_dir, PROOF_BUNDLES_JSON_FILE);
    let fixture = write_executable_proof_fixture(&temp, "unsupported-schema-proof-fixture");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--policy-artifacts-dir")
        .arg(&artifact_dir)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--execute-proof-rails")
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == "tigerstyle.verification.proof-bundle-schema"
            && diagnostic.message.contains("unsupported proof-bundle schema version")
            && diagnostic.message.contains(PROOF_BUNDLE_SCHEMA_VERSION)
    }));
}

#[test]
fn artifact_backed_proof_bundle_rejects_missing_bundle_file() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let artifact_dir = temp.path().join("generated-proof-bundle");
    export_artifacts(&registry, &artifact_dir);
    std::fs::remove_file(artifact_dir.join(PROOF_BUNDLES_JSON_FILE)).expect("proof bundle should be removed");
    let fixture = write_executable_proof_fixture(&temp, "missing-bundle-proof-fixture");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--policy-artifacts-dir")
        .arg(&artifact_dir)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--execute-proof-rails")
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == "tigerstyle.generated-artifacts.integrity-manifest"
            && diagnostic.message.contains("failed to read generated artifact")
            && diagnostic.message.contains(PROOF_BUNDLES_JSON_FILE)
    }));
}

#[test]
fn artifact_backed_proof_bundle_accepts_complete_bundle() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let artifact_dir = temp.path().join("generated-proof-bundle");
    export_artifacts(&registry, &artifact_dir);
    let fixture = write_executable_proof_fixture(&temp, "complete-bundle-proof-fixture");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--policy-artifacts-dir")
        .arg(&artifact_dir)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--execute-proof-rails")
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_success(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.is_empty());
    assert!(report.evaluated_policy_ids.iter().any(|policy_id| policy_id == PROOF_POLICY_ID));
}

#[test]
fn execute_proof_rails_uses_command_output_as_receipt() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = temp.path().join("proof-fixture");
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    std::fs::write(fixture.join("src/lib.rs"), "pub fn proof_witness() {}\n").expect("proof source should be written");
    std::fs::write(
        fixture.join("proof-build-closure.txt"),
        "verifier = octet-standards\nclosure-accounting = test manifest\n",
    )
    .expect("proof closure manifest should be written");
    std::fs::write(
        fixture.join("proof-nix-closure.txt"),
        "nix-derivation = octet-standards.drv\nclosure-source = exportReferencesGraph\n",
    )
    .expect("proof Nix closure manifest should be written");
    std::fs::write(fixture.join("proof-nix-closure.graph"), nix_closure_graph())
        .expect("proof Nix closure graph should be written");
    std::fs::write(
        fixture.join("proof-command.sh"),
        "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\n' 'proof-result: pass deterministic-effect-handlers'\n",
    )
    .expect("proof command should be written");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--execute-proof-rails")
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_success(&output);
    let report = parse_report(&output);
    assert_eq!(report.diagnostics.len(), 0);
    assert!(report.evaluated_policy_ids.iter().any(|policy_id| policy_id == PROOF_POLICY_ID));
}

#[test]
fn execute_proof_rails_rejects_missing_nix_closure_graph() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = write_executable_proof_fixture(&temp, "missing-graph-proof-fixture");
    std::fs::remove_file(fixture.join("proof-nix-closure.graph")).expect("proof Nix closure graph should be removed");

    let output = run_executed_proof_check(&registry, &fixture);

    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID
            && diagnostic.message.contains("verifier_nix_closure_graph_paths")
            && diagnostic.message.contains("matches no checked closure graph artifact")
    }));
}

#[test]
fn execute_proof_rails_rejects_malformed_nix_closure_graph() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = write_executable_proof_fixture(&temp, "malformed-graph-proof-fixture");
    std::fs::write(fixture.join("proof-nix-closure.graph"), "/nix/store/aaa-octet-standards\nnot-a-count\n")
        .expect("malformed proof Nix closure graph should be written");

    let output = run_executed_proof_check(&registry, &fixture);

    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID
            && diagnostic.message.contains("verifier_nix_closure_graph_paths")
            && diagnostic.message.contains("malformed closure graph")
    }));
}

#[test]
fn execute_proof_rails_rejects_missing_build_closure_manifest() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = write_executable_proof_fixture(&temp, "missing-build-closure-proof-fixture");
    std::fs::remove_file(fixture.join("proof-build-closure.txt")).expect("build closure manifest should be removed");

    let output = run_executed_proof_check(&registry, &fixture);

    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID
            && diagnostic.message.contains("verifier_build_closure_paths")
            && diagnostic.message.contains("matches no checked closure manifest")
    }));
}

#[test]
fn execute_proof_rails_rejects_missing_nix_closure_manifest() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = write_executable_proof_fixture(&temp, "missing-nix-closure-proof-fixture");
    std::fs::remove_file(fixture.join("proof-nix-closure.txt")).expect("Nix closure manifest should be removed");

    let output = run_executed_proof_check(&registry, &fixture);

    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID
            && diagnostic.message.contains("verifier_nix_closure_paths")
            && diagnostic.message.contains("matches no checked closure manifest")
    }));
}

#[test]
fn receipt_only_proof_rail_rejects_unbound_stale_receipt() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = temp.path().join("unbound-proof-fixture");
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    std::fs::write(fixture.join("src/lib.rs"), "pub fn proof_witness() {}\n").expect("proof source should be written");
    std::fs::write(fixture.join("proof-receipt.txt"), "proof-result: pass deterministic-effect-handlers\n")
        .expect("unbound proof receipt should be written");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID && diagnostic.message.contains("mismatched receipt binding")
    }));
}

#[test]
fn receipt_only_proof_rail_rejects_hash_bound_receipt_without_provenance() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = temp.path().join("hash-bound-proof-fixture");
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    let source = "pub fn proof_witness() {}\n";
    std::fs::write(fixture.join("src/lib.rs"), source).expect("proof source should be written");
    std::fs::write(
        fixture.join("proof-receipt.txt"),
        format!(
            "proof-command-blake3: {}\nproof-source-blake3: {}:{}\nproof-result: pass deterministic-effect-handlers\n",
            blake3_hex("bash proof-command.sh"),
            fixture.join("src/lib.rs").display(),
            blake3_hex(source),
        ),
    )
    .expect("hash-bound proof receipt should be written");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID && diagnostic.message.contains("mismatched receipt provenance")
    }));
}

#[test]
fn receipt_only_proof_rail_rejects_provenance_receipt_without_verifier_digest() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = temp.path().join("provenance-bound-proof-fixture");
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    let source = "pub fn proof_witness() {}
";
    std::fs::write(fixture.join("src/lib.rs"), source).expect("proof source should be written");
    std::fs::write(
        fixture.join("proof-receipt.txt"),
        format!(
            "proof-receipt-schema: tigerstyle-proof-receipt/v1
proof-backend: local-command-receipt
proof-runner: octet-standards
proof-environment: local-source-root
trust-boundary: test command receipt
proof-command-blake3: {}
proof-source-blake3: {}:{}
proof-result: pass deterministic-effect-handlers
",
            blake3_hex("bash proof-command.sh"),
            fixture.join("src/lib.rs").display(),
            blake3_hex(source),
        ),
    )
    .expect("provenance-bound proof receipt should be written");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID && diagnostic.message.contains("mismatched receipt verifier metadata")
    }));
}

#[test]
fn receipt_only_proof_rail_rejects_verifier_receipt_without_binary_digest() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = temp.path().join("verifier-bound-proof-fixture");
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    let source = "pub fn proof_witness() {}
";
    std::fs::write(fixture.join("src/lib.rs"), source).expect("proof source should be written");
    std::fs::write(
        fixture.join("proof-receipt.txt"),
        format!(
            "proof-receipt-schema: tigerstyle-proof-receipt/v1
proof-backend: local-command-receipt
proof-runner: octet-standards
proof-environment: local-source-root
trust-boundary: test command receipt
proof-verifier: octet-standards@proof-receipt/v1
proof-verifier-blake3: 9fb0cd3ff1b2898eba4c6f71479f93ae3cdc80f869cd8ae5e6355235ec5f9aef
proof-toolchain: rust-toolchain.toml
proof-toolchain-blake3: 50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8
proof-command-blake3: {}
proof-source-blake3: {}:{}
proof-result: pass deterministic-effect-handlers
",
            blake3_hex("bash proof-command.sh"),
            fixture.join("src/lib.rs").display(),
            blake3_hex(source),
        ),
    )
    .expect("verifier-bound proof receipt should be written");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID && diagnostic.message.contains("mismatched receipt verifier metadata")
    }));
}

#[test]
fn receipt_only_proof_rail_rejects_verifier_receipt_without_build_closure_digest() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = temp.path().join("build-closure-bound-proof-fixture");
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    let source = "pub fn proof_witness() {}\n";
    std::fs::write(fixture.join("src/lib.rs"), source).expect("proof source should be written");
    std::fs::write(
        fixture.join("proof-receipt.txt"),
        format!(
            "proof-receipt-schema: tigerstyle-proof-receipt/v1\nproof-backend: local-command-receipt\nproof-runner: octet-standards\nproof-environment: local-source-root\ntrust-boundary: test command receipt\nproof-verifier: octet-standards@proof-receipt/v1\nproof-verifier-blake3: 9fb0cd3ff1b2898eba4c6f71479f93ae3cdc80f869cd8ae5e6355235ec5f9aef\nproof-verifier-binary: octet-standards\nproof-verifier-binary-blake3: {}\nproof-toolchain: rust-toolchain.toml\nproof-toolchain-blake3: 50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8\nproof-command-blake3: {}\nproof-source-blake3: {}:{}\nproof-result: pass deterministic-effect-handlers\n",
            blake3_file_hex(binary_path()),
            blake3_hex("bash proof-command.sh"),
            fixture.join("src/lib.rs").display(),
            blake3_hex(source),
        ),
    )
    .expect("build-closure-bound proof receipt should be written");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID && diagnostic.message.contains("mismatched receipt verifier metadata")
    }));
}

#[test]
fn receipt_only_proof_rail_rejects_verifier_receipt_without_nix_closure_digest() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = temp.path().join("nix-closure-bound-proof-fixture");
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    let source = "pub fn proof_witness() {}\n";
    let build_closure = "verifier = octet-standards\nclosure-accounting = test manifest\n";
    std::fs::write(fixture.join("src/lib.rs"), source).expect("proof source should be written");
    std::fs::write(fixture.join("proof-build-closure.txt"), build_closure)
        .expect("build closure manifest should be written");
    std::fs::write(
        fixture.join("proof-receipt.txt"),
        format!(
            "proof-receipt-schema: tigerstyle-proof-receipt/v1\nproof-backend: local-command-receipt\nproof-runner: octet-standards\nproof-environment: local-source-root\ntrust-boundary: test command receipt\nproof-verifier: octet-standards@proof-receipt/v1\nproof-verifier-blake3: 9fb0cd3ff1b2898eba4c6f71479f93ae3cdc80f869cd8ae5e6355235ec5f9aef\nproof-verifier-binary: octet-standards\nproof-verifier-binary-blake3: {}\nproof-verifier-build-closure: nix-octet-standards-build-closure/v1\nproof-verifier-build-closure-blake3: {}\nproof-toolchain: rust-toolchain.toml\nproof-toolchain-blake3: 50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8\nproof-command-blake3: {}\nproof-source-blake3: {}:{}\nproof-result: pass deterministic-effect-handlers\n",
            blake3_file_hex(binary_path()),
            build_closure_digest_line(&fixture.join("proof-build-closure.txt")),
            blake3_hex("bash proof-command.sh"),
            fixture.join("src/lib.rs").display(),
            blake3_hex(source),
        ),
    )
    .expect("Nix-closure-bound proof receipt should be written");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID && diagnostic.message.contains("mismatched receipt verifier metadata")
    }));
}

#[test]
fn receipt_only_proof_rail_rejects_verifier_receipt_without_nix_closure_graph_digest() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = temp.path().join("nix-closure-graph-bound-proof-fixture");
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    let source = "pub fn proof_witness() {}
";
    let build_closure = "verifier = octet-standards
closure-accounting = test manifest
";
    let nix_closure = "nix-derivation = octet-standards.drv
closure-source = exportReferencesGraph
";
    std::fs::write(fixture.join("src/lib.rs"), source).expect("proof source should be written");
    std::fs::write(fixture.join("proof-build-closure.txt"), build_closure)
        .expect("build closure manifest should be written");
    std::fs::write(fixture.join("proof-nix-closure.txt"), nix_closure).expect("Nix closure manifest should be written");
    std::fs::write(fixture.join("proof-nix-closure.graph"), nix_closure_graph())
        .expect("Nix closure graph should be written");
    std::fs::write(
        fixture.join("proof-receipt.txt"),
        format!(
            "proof-receipt-schema: tigerstyle-proof-receipt/v1
proof-backend: local-command-receipt
proof-runner: octet-standards
proof-environment: local-source-root
trust-boundary: test command receipt
proof-verifier: octet-standards@proof-receipt/v1
proof-verifier-blake3: 9fb0cd3ff1b2898eba4c6f71479f93ae3cdc80f869cd8ae5e6355235ec5f9aef
proof-verifier-binary: octet-standards
proof-verifier-binary-blake3: {}
proof-verifier-build-closure: nix-octet-standards-build-closure/v1
proof-verifier-build-closure-blake3: {}
proof-verifier-nix-derivation: octet-standards.drv
proof-verifier-nix-derivation-blake3: {}
proof-verifier-nix-closure: nix-store-recursive-closure/v1
proof-verifier-nix-closure-blake3: {}
proof-toolchain: rust-toolchain.toml
proof-toolchain-blake3: 50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8
proof-command-blake3: {}
proof-source-blake3: {}:{}
proof-result: pass deterministic-effect-handlers
",
            blake3_file_hex(binary_path()),
            build_closure_digest_line(&fixture.join("proof-build-closure.txt")),
            blake3_hex("octet-standards.drv"),
            build_closure_digest_line(&fixture.join("proof-nix-closure.txt")),
            blake3_hex("bash proof-command.sh"),
            fixture.join("src/lib.rs").display(),
            blake3_hex(source),
        ),
    )
    .expect("Nix-closure-graph-bound proof receipt should be written");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID && diagnostic.message.contains("mismatched receipt verifier metadata")
    }));
}

#[test]
fn execute_proof_rails_replaces_stale_committed_receipt() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry =
        write_registry_value(&temp, &proof_registry_with_modes(EnforcementMode::Verified, EnforcementMode::Gate));
    let fixture = temp.path().join("stale-proof-fixture");
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    std::fs::write(fixture.join("src/lib.rs"), "pub fn proof_witness() {}\n").expect("proof source should be written");
    std::fs::write(fixture.join("proof-receipt.txt"), "proof-result: pass deterministic-effect-handlers\n")
        .expect("stale proof receipt should be written");
    std::fs::write(
        fixture.join("proof-command.sh"),
        "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\n' 'proof-result: fail deterministic-effect-handlers'\n",
    )
    .expect("proof command should be written");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--execute-proof-rails")
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");
    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID && diagnostic.message.contains("missing pass signal")
    }));
}

#[test]
fn execute_proof_rails_rejects_failing_execution_despite_valid_committed_receipt() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &relocated_receipt_proof_registry());
    let fixture = write_relocated_receipt_fixture(&temp, "failing-relocated-proof-fixture", "fail");
    std::fs::write(fixture.join("receipts/proof-receipt.txt"), "proof-result: pass deterministic-effect-handlers\n")
        .expect("valid committed proof receipt should be written");

    let output = run_executed_proof_check(&registry, &fixture);

    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID
            && diagnostic.path.ends_with("receipts/proof-receipt.txt")
            && diagnostic.message.contains("missing pass signal")
    }));
}

#[test]
fn check_skips_non_utf8_files_in_source_directories_but_rejects_named_ones() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let source_dir = temp.path().join("binary-asset-fixture");
    std::fs::create_dir_all(source_dir.join("src")).expect("source dir should be created");
    std::fs::create_dir_all(source_dir.join("assets")).expect("asset dir should be created");
    std::fs::write(source_dir.join("src/lib.rs"), "pub fn small() {}\n").expect("source should be written");
    let binary_asset = source_dir.join("assets/logo.bin");
    std::fs::write(&binary_asset, [0xff, 0xfe, 0x00, 0x01]).expect("binary asset should be written");

    let metadata = write_octet_layout_metadata(&temp);
    let walked = run_architecture_check(&registry, &[&source_dir, &metadata]);

    assert_success(&walked);
    assert!(parse_report(&walked).diagnostics.is_empty());

    let named = run_architecture_check(&registry, &[&binary_asset, &metadata]);

    assert_failure(&named);
    let stderr = String::from_utf8_lossy(&named.stderr);
    assert!(stderr.contains("failed to read") && stderr.contains("logo.bin"), "stderr was: {stderr}");
}

#[test]
fn proof_artifact_dir_requires_execute_proof_rails() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &proof_registry());
    let fixture = write_executable_proof_fixture(&temp, "proof-artifact-dir-fixture");
    let proof_artifact_dir = temp.path().join("proof-artifacts");

    let output = Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg(&fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--proof-artifact-dir")
        .arg(&proof_artifact_dir)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run");

    assert_failure(&output);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("--execute-proof-rails"), "stderr was: {stderr}");
    assert!(output.stdout.is_empty(), "no report should be emitted for rejected arguments");
    assert!(!proof_artifact_dir.exists());
}

#[test]
fn execute_proof_rails_rejects_pass_signal_with_failing_exit_status() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &relocated_receipt_proof_registry());
    let fixture = write_relocated_receipt_fixture(&temp, "failing-exit-proof-fixture", "pass");
    std::fs::write(
        fixture.join("proof-command.sh"),
        "#!/usr/bin/env bash\nprintf '%s\\n' 'proof-result: pass deterministic-effect-handlers'\nexit 1\n",
    )
    .expect("failing proof command should be written");

    let output = run_executed_proof_check(&registry, &fixture);

    assert_failure(&output);
    let report = parse_report(&output);
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.policy_id == PROOF_POLICY_ID && diagnostic.message.contains("`exit-status: 1`")
    }));
}

#[test]
fn execute_proof_rails_replaces_committed_receipt_at_receipt_path() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry_value(&temp, &relocated_receipt_proof_registry());
    let fixture = write_relocated_receipt_fixture(&temp, "passing-relocated-proof-fixture", "pass");
    std::fs::write(fixture.join("receipts/proof-receipt.txt"), "proof-result: fail deterministic-effect-handlers\n")
        .expect("stale committed proof receipt should be written");

    let output = run_executed_proof_check(&registry, &fixture);

    assert_success(&output);
    let report = parse_report(&output);
    assert_eq!(report.diagnostics.len(), 0);
    assert!(report.evaluated_policy_ids.iter().any(|policy_id| policy_id == PROOF_POLICY_ID));
}

#[cfg(unix)]
#[test]
fn architecture_check_reports_cargo_metadata_that_cannot_run() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let workspace = write_cargo_workspace_fixture(&temp);
    let empty_path = temp.path().join("empty-path");
    std::fs::create_dir_all(&empty_path).expect("empty PATH dir should be created");

    let output = run_architecture_check_with_path(&registry, &workspace, &empty_path);

    assert_cargo_metadata_failure(&output, "`cargo metadata --format-version 1` could not run");
}

#[cfg(unix)]
#[test]
fn architecture_check_reports_cargo_metadata_nonzero_exit() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let workspace = write_cargo_workspace_fixture(&temp);
    let fake_bin = write_fake_cargo(&temp, "#!/bin/sh\necho 'error: failed to load manifest' >&2\nexit 101\n");

    let output = run_architecture_check_with_path(&registry, &workspace, &fake_bin);

    assert_cargo_metadata_failure(&output, "exit status: 101");
    assert!(
        parse_report(&output)
            .diagnostics
            .iter()
            .any(|diagnostic| { diagnostic.message.contains("error: failed to load manifest") })
    );
}

#[cfg(unix)]
#[test]
fn architecture_check_reports_cargo_metadata_non_utf8_output() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let workspace = write_cargo_workspace_fixture(&temp);
    let fake_bin = write_fake_cargo(&temp, "#!/bin/sh\nprintf '\\377\\376'\n");

    let output = run_architecture_check_with_path(&registry, &workspace, &fake_bin);

    assert_cargo_metadata_failure(&output, "non-UTF-8 output");
}

#[cfg(unix)]
#[test]
fn architecture_check_runs_cargo_metadata_for_relative_source_paths() {
    let temp = TempDir::new().expect("temp dir should be created");
    let mut registry_value = test_registry();
    registry_value.policies[0].mechanisms[0].dependency_edge_kinds = vec!["normal".to_owned(), "build".to_owned()];
    let registry = write_registry_value(&temp, &registry_value);
    let workspace = write_cargo_workspace_fixture(&temp);
    let fake_bin = write_fake_cargo(
        &temp,
        r#"#!/bin/sh
printf '%s' '{"packages":[{"id":"core","name":"tigerstyle-core","manifest_path":"/w/crates/tigerstyle-core/Cargo.toml"},{"id":"tokio","name":"tokio","manifest_path":"/registry/tokio/Cargo.toml"}],"resolve":{"nodes":[{"id":"core","deps":[{"pkg":"tokio","dep_kinds":[{"kind":null}]}]}]},"workspace_members":["core"]}'
"#,
    );

    let output = Command::new(binary_path())
        .current_dir(&workspace)
        .arg("check")
        .arg("--registry")
        .arg(&registry)
        .arg("--source")
        .arg("crates")
        .arg("--family")
        .arg(ARCHITECTURE_FAMILY)
        .arg("--output")
        .arg("json")
        .env("PATH", &fake_bin)
        .output()
        .expect("standards checker should run");

    assert_failure(&output);
    let report = parse_report(&output);
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.policy_id == FCIS_POLICY_ID
                && diagnostic
                    .message
                    .contains("core crate `tigerstyle-core` depends on external infrastructure crate `tokio`")
        }),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn architecture_check_rejects_sources_outside_any_cargo_workspace() {
    let temp = TempDir::new().expect("temp dir should be created");
    let registry = write_registry(&temp);
    let loose_sources = temp.path().join("loose-sources");
    std::fs::create_dir_all(loose_sources.join("crates/tigerstyle-core/src")).expect("core dir should be created");
    std::fs::write(loose_sources.join("crates/tigerstyle-core/src/lib.rs"), "#![no_std]\n")
        .expect("core crate root should be written");

    let output = run_architecture_check(&registry, &[&loose_sources]);

    assert_failure(&output);
    let report = parse_report(&output);
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.policy_id == FCIS_POLICY_ID
                && diagnostic.severity == Severity::Error
                && diagnostic.message.contains("Cargo.toml")
        }),
        "{:?}",
        report.diagnostics
    );
}

fn write_cargo_workspace_fixture(temp: &TempDir) -> PathBuf {
    let workspace = temp.path().join("workspace");
    std::fs::create_dir_all(workspace.join("crates/tigerstyle-core/src")).expect("core crate dir should be created");
    std::fs::write(workspace.join("Cargo.toml"), "[workspace]\nmembers = [\"crates/*\"]\n")
        .expect("workspace manifest should be written");
    std::fs::write(workspace.join("crates/tigerstyle-core/src/lib.rs"), "#![no_std]\n")
        .expect("core crate root should be written");
    workspace
}

#[cfg(unix)]
fn write_fake_cargo(temp: &TempDir, script: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt as _;

    let fake_bin = temp.path().join("fake-bin");
    std::fs::create_dir_all(&fake_bin).expect("fake cargo dir should be created");
    let fake_cargo = fake_bin.join("cargo");
    std::fs::write(&fake_cargo, script).expect("fake cargo should be written");
    std::fs::set_permissions(&fake_cargo, std::fs::Permissions::from_mode(0o755))
        .expect("fake cargo should be executable");
    fake_bin
}

fn run_architecture_check_with_path(registry: &Path, source: &Path, path_env: &Path) -> Output {
    Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(registry)
        .arg("--source")
        .arg(source)
        .arg("--family")
        .arg(ARCHITECTURE_FAMILY)
        .arg("--output")
        .arg("json")
        .env("PATH", path_env)
        .output()
        .expect("standards checker should run")
}

fn assert_cargo_metadata_failure(output: &Output, expected_message: &str) {
    assert_failure(output);
    let report = parse_report(output);
    assert!(
        report.diagnostics.iter().any(|diagnostic| {
            diagnostic.policy_id == FCIS_POLICY_ID
                && diagnostic.severity == Severity::Error
                && diagnostic.path.ends_with("workspace/Cargo.toml")
                && diagnostic.message.contains("cargo metadata")
                && diagnostic.message.contains(expected_message)
        }),
        "expected cargo metadata failure `{expected_message}`, got {:?}",
        report.diagnostics
    );
}

fn export_artifacts(registry: &Path, artifact_dir: &Path) {
    let output = Command::new(binary_path())
        .arg("export-artifacts")
        .arg("--registry")
        .arg(registry)
        .arg("--out-dir")
        .arg(artifact_dir)
        .output()
        .expect("artifact export should run");
    assert_success(&output);
}

fn assert_receipt_artifact(receipt: &serde_json::Value, artifact_name: &str, artifact_dir: &Path) {
    let artifact_path = artifact_dir.join(artifact_name);
    let artifact_bytes = std::fs::read(&artifact_path).expect("receipt artifact should be readable");
    let artifact_digest = blake3::hash(&artifact_bytes).to_hex().to_string();
    let artifacts = receipt["artifacts"].as_array().expect("receipt artifacts should be an array");
    assert!(artifacts.iter().any(|artifact| {
        artifact["path"] == artifact_path.display().to_string()
            && artifact["blake3"] == artifact_digest
            && artifact["bytes"] == u64::try_from(artifact_bytes.len()).expect("artifact length fits u64")
    }));
}

fn test_profile_json() -> &'static str {
    r#"{
  "schema_version": "octet-run-profile/v1",
  "profile_id": "cli-receipt-test",
  "source_paths": ["standards/policies/registry.ncl"],
  "artifact_out_dir": "build/octet-artifacts",
  "receipt_out_dir": "build/octet-receipts",
  "config_digest_algorithm": "blake3",
  "enabled_policy_families": ["architecture"],
  "secret_refs": []
}
"#
}

fn write_executable_proof_fixture(temp: &TempDir, name: &str) -> PathBuf {
    let fixture = temp.path().join(name);
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture dirs should be created");
    std::fs::write(fixture.join("src/lib.rs"), "pub fn proof_witness() {}\n").expect("proof source should be written");
    std::fs::write(
        fixture.join("proof-build-closure.txt"),
        "verifier = octet-standards\nclosure-accounting = test manifest\n",
    )
    .expect("proof closure manifest should be written");
    std::fs::write(
        fixture.join("proof-nix-closure.txt"),
        "nix-derivation = octet-standards.drv\nclosure-source = exportReferencesGraph\n",
    )
    .expect("proof Nix closure manifest should be written");
    std::fs::write(fixture.join("proof-nix-closure.graph"), nix_closure_graph())
        .expect("proof Nix closure graph should be written");
    std::fs::write(
        fixture.join("proof-command.sh"),
        "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\n' 'proof-result: pass deterministic-effect-handlers'\n",
    )
    .expect("proof command should be written");
    fixture
}

fn write_relocated_receipt_fixture(temp: &TempDir, name: &str, proof_result: &str) -> PathBuf {
    let fixture = temp.path().join(name);
    std::fs::create_dir_all(fixture.join("src")).expect("proof fixture source dir should be created");
    std::fs::create_dir_all(fixture.join("proofs")).expect("proof fixture proof dir should be created");
    std::fs::create_dir_all(fixture.join("receipts")).expect("proof fixture receipt dir should be created");
    std::fs::write(fixture.join("src/lib.rs"), "pub fn proof_witness() {}\n").expect("proof source should be written");
    std::fs::write(fixture.join("proofs/witness.rs"), "proof fn witness() {}\n")
        .expect("proof input should be written");
    std::fs::write(
        fixture.join("proof-command.sh"),
        format!(
            "#!/usr/bin/env bash\nset -euo pipefail\nprintf '%s\\n' 'proof-result: {proof_result} deterministic-effect-handlers'\n"
        ),
    )
    .expect("proof command should be written");
    fixture
}

fn run_executed_proof_check(registry: &Path, fixture: &Path) -> Output {
    Command::new(binary_path())
        .arg("check")
        .arg("--registry")
        .arg(registry)
        .arg("--source")
        .arg(fixture)
        .arg("--family")
        .arg(VERIFICATION_FAMILY)
        .arg("--execute-proof-rails")
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run")
}

fn refresh_manifest_entry(artifact_dir: &Path, artifact_name: &str) {
    let manifest_path = artifact_dir.join(ARTIFACT_MANIFEST_JSON_FILE);
    let artifact_path = artifact_dir.join(artifact_name);
    let artifact_bytes = std::fs::read(&artifact_path).expect("artifact should be readable for manifest refresh");
    let manifest_text = std::fs::read_to_string(&manifest_path).expect("manifest should be readable");
    let mut manifest: serde_json::Value = serde_json::from_str(&manifest_text).expect("manifest should parse as JSON");
    let entries = manifest["artifacts"].as_array_mut().expect("manifest artifacts should be an array");
    let entry = entries
        .iter_mut()
        .find(|entry| entry["path"] == artifact_name)
        .expect("manifest entry should exist");
    entry["blake3"] = serde_json::Value::String(blake3::hash(&artifact_bytes).to_hex().to_string());
    entry["bytes"] = serde_json::Value::from(u64::try_from(artifact_bytes.len()).expect("artifact bytes fit u64"));
    let text = serde_json::to_string_pretty(&manifest).expect("manifest should serialize");
    std::fs::write(&manifest_path, format!("{text}\n")).expect("manifest should be refreshed");
}

fn run_architecture_check(registry: &Path, sources: &[&Path]) -> Output {
    let mut command = Command::new(binary_path());
    command.arg("check").arg("--registry").arg(registry);
    for source in sources {
        command.arg("--source").arg(source);
    }
    command
        .arg("--family")
        .arg(ARCHITECTURE_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run")
}

/// Copies the reviewed Octet-layout crate graph to a `__octet/cargo-metadata.json` source path.
fn write_octet_layout_metadata(temp: &TempDir) -> PathBuf {
    let metadata_dir = temp.path().join("cargo-metadata/__octet");
    std::fs::create_dir_all(&metadata_dir).expect("metadata dir should be created");
    let metadata = metadata_dir.join("cargo-metadata.json");
    std::fs::copy(repo_path(OCTET_LAYOUT_METADATA_FIXTURE), &metadata).expect("metadata fixture should be copied");
    metadata
}

fn run_architecture_artifact_check(policy_artifacts_dir: &Path, source: &Path) -> Output {
    Command::new(binary_path())
        .arg("check")
        .arg("--policy-artifacts-dir")
        .arg(policy_artifacts_dir)
        .arg("--source")
        .arg(source)
        .arg("--family")
        .arg(ARCHITECTURE_FAMILY)
        .arg("--output")
        .arg("json")
        .output()
        .expect("standards checker should run")
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "expected success, stdout was: {}\nstderr was: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn assert_failure(output: &Output) {
    assert!(
        !output.status.success(),
        "expected failure, stdout was: {}",
        String::from_utf8_lossy(&output.stdout)
    );
}

fn parse_report(output: &Output) -> CheckReport {
    serde_json::from_slice(&output.stdout).expect("stdout should be a standards report")
}

fn binary_path() -> &'static str {
    env!("CARGO_BIN_EXE_octet-standards")
}

fn standards_alias_bins() -> [(&'static str, &'static str); STANDARDS_ALIAS_BINARY_COUNT] {
    [
        ("octet-standards", env!("CARGO_BIN_EXE_octet-standards")),
        ("slotcar-standards", env!("CARGO_BIN_EXE_slotcar-standards")),
        ("tigerstyle-standards", env!("CARGO_BIN_EXE_tigerstyle-standards")),
    ]
}

fn blake3_hex(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

fn blake3_file_hex(path: &str) -> String {
    let bytes = std::fs::read(path).expect("file should be readable for BLAKE3 hashing");
    blake3::hash(&bytes).to_hex().to_string()
}

fn build_closure_digest_line(path: &Path) -> String {
    let file_digest = blake3_file_hex(&path.display().to_string());
    blake3_hex(&format!("{}:{}\n", path.display(), file_digest))
}

fn nix_closure_graph() -> &'static str {
    "/nix/store/aaa-octet-standards\n\n2\n/nix/store/bbb-glibc\n/nix/store/ccc-libgcc\n\n/nix/store/bbb-glibc\n\n0\n\n/nix/store/ccc-libgcc\n\n1\n/nix/store/bbb-glibc\n"
}

fn nix_closure_graph_digest_line(path: &Path) -> String {
    let canonical = "/nix/store/aaa-octet-standards\n2\n/nix/store/bbb-glibc\n/nix/store/ccc-libgcc\n/nix/store/bbb-glibc\n0\n/nix/store/ccc-libgcc\n1\n/nix/store/bbb-glibc\n";
    let graph_digest = blake3_hex(canonical);
    blake3_hex(&format!("{}:{}\n", path.display(), graph_digest))
}

fn repo_path(relative_path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative_path)
        .canonicalize()
        .expect("repo fixture path should exist")
}

fn write_registry(temp: &TempDir) -> PathBuf {
    write_registry_value(temp, &test_registry())
}

fn write_registry_value(temp: &TempDir, registry: &PolicyRegistry) -> PathBuf {
    let registry_path = temp.path().join(REGISTRY_FILE_NAME);
    let registry_json = serde_json::to_string(&registry).expect("registry should serialize");
    std::fs::write(&registry_path, registry_json).expect("registry file should be written");
    registry_path
}

fn test_registry() -> PolicyRegistry {
    PolicyRegistry {
        schema_version: "test".to_owned(),
        checker_ids: vec![CHECKER_ID.to_owned()],
        policies: vec![
            hard_policy(FCIS_POLICY_ID, "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see"),
            hard_policy(
                ROOT_THINNESS_POLICY_ID,
                "docs/standards/source/architecture.adoc#boundaries-the-compiler-can-see",
            ),
            advisory_policy(
                PUBLIC_API_BUDGET_POLICY_ID,
                "docs/standards/source/architecture.adoc#when-boundaries-are-too-tight",
            ),
        ],
        seed_manifests: Vec::new(),
        config_receipt_families: Vec::new(),
    }
}

fn proof_registry() -> PolicyRegistry {
    proof_registry_with_modes(EnforcementMode::Verified, EnforcementMode::Verified)
}

fn proof_registry_with_modes(current: EnforcementMode, target: EnforcementMode) -> PolicyRegistry {
    PolicyRegistry {
        schema_version: "test".to_owned(),
        checker_ids: vec![CHECKER_ID.to_owned()],
        policies: vec![verified_policy(current, target)],
        seed_manifests: Vec::new(),
        config_receipt_families: Vec::new(),
    }
}

fn relocated_receipt_proof_registry() -> PolicyRegistry {
    let mut registry = proof_registry();
    let rail = &mut registry.policies[0].proof_rails[0];
    rail.proof_paths = vec!["*/proofs/witness.rs".to_owned()];
    rail.receipt_paths = vec!["*/receipts/proof-receipt.txt".to_owned()];
    rail.receipt_binding = String::new();
    rail.receipt_provenance = ReceiptProvenance::default();
    rail.receipt_verifier = ReceiptVerifier::default();
    registry
}

fn verified_policy(current: EnforcementMode, target: EnforcementMode) -> PolicyEntry {
    PolicyEntry {
        policy_id: PROOF_POLICY_ID.to_owned(),
        pillar: VERIFICATION_FAMILY.to_owned(),
        source_path: "docs/standards/source/testing.adoc".to_owned(),
        source_heading: "Deterministic Effect Handlers".to_owned(),
        source_anchor: "docs/standards/source/testing.adoc#deterministic-effect-handlers".to_owned(),
        summary: "test verified policy".to_owned(),
        classification: Classification::Hard,
        enforcement_level: EnforcementLevel::Hard,
        current_enforcement_mode: current,
        target_enforcement_mode: target,
        escalation_criteria: "proof command is executed".to_owned(),
        mechanisms: Vec::new(),
        deterministic_checker: EvidenceCommand::default(),
        proof_rails: vec![ProofRail {
            backend: "local-command-receipt".to_owned(),
            command: "bash proof-command.sh".to_owned(),
            command_argv: vec!["bash".to_owned(), "proof-command.sh".to_owned()],
            source_paths: vec!["*/src/lib.rs".to_owned()],
            proof_paths: vec!["proof-receipt.txt".to_owned()],
            receipt_paths: vec!["*/proof-receipt.txt".to_owned()],
            pass_signal: "proof-result: pass deterministic-effect-handlers".to_owned(),
            trust_boundaries: vec!["test command receipt".to_owned()],
            receipt_binding: "blake3-command-and-source".to_owned(),
            receipt_provenance: ReceiptProvenance {
                schema_version: "tigerstyle-proof-receipt/v1".to_owned(),
                runner: CHECKER_ID.to_owned(),
                environment: "local-source-root".to_owned(),
                requires_backend: true,
                requires_trust_boundary: true,
            },
            receipt_verifier: ReceiptVerifier {
                verifier_id: "octet-standards@proof-receipt/v1".to_owned(),
                verifier_digest: "9fb0cd3ff1b2898eba4c6f71479f93ae3cdc80f869cd8ae5e6355235ec5f9aef".to_owned(),
                verifier_binary_path: "octet-standards".to_owned(),
                verifier_binary_digest: blake3_file_hex(binary_path()),
                verifier_build_closure: "nix-octet-standards-build-closure/v1".to_owned(),
                verifier_build_closure_digest: "fixture-build-closure-digest".to_owned(),
                verifier_build_closure_paths: vec!["*/proof-build-closure.txt".to_owned()],
                verifier_nix_derivation: "octet-standards.drv".to_owned(),
                verifier_nix_derivation_digest: blake3_hex("octet-standards.drv"),
                verifier_nix_closure: "nix-store-recursive-closure/v1".to_owned(),
                verifier_nix_closure_digest: "fixture-nix-closure-digest".to_owned(),
                verifier_nix_closure_paths: vec!["*/proof-nix-closure.txt".to_owned()],
                verifier_nix_closure_graph_digest: nix_closure_graph_digest_line(Path::new("proof-nix-closure.graph")),
                verifier_nix_closure_graph_paths: vec!["*/proof-nix-closure.graph".to_owned()],
                toolchain_id: "rust-toolchain.toml".to_owned(),
                toolchain_digest: "50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8".to_owned(),
                solver_id: String::new(),
                solver_digest: String::new(),
            },
            ..ProofRail::default()
        }],
        checker_owner: VERIFICATION_FAMILY.to_owned(),
        checker_id: CHECKER_ID.to_owned(),
        default_severity: Severity::Error,
        suppression_requirements: hard_suppression_requirements(),
        fixture_coverage: FixtureCoverage {
            status: FixtureStatus::Covered,
            compliant: Vec::new(),
            noncompliant: Vec::new(),
        },
        verification_evidence_requirements: vec!["proof command receipt".to_owned()],
        classification_rationale: "local command receipt is deterministic".to_owned(),
        false_positive_risk: "low".to_owned(),
        existing_enforcement: Vec::new(),
        custom_checker_justification: "test".to_owned(),
    }
}

fn hard_policy(policy_id: &str, source_anchor: &str) -> PolicyEntry {
    let mechanisms = if policy_id == FCIS_POLICY_ID {
        vec![Mechanism {
            kind: MechanismKind::CrateBoundary,
            description: "explicit CLI fixture boundary".to_owned(),
            core_crates: vec!["crates/tigerstyle-core/src/lib.rs".to_owned()],
            shell_crates: vec!["crates/tigerstyle-std/Cargo.toml".to_owned()],
            core_no_std: true,
            ..Mechanism::default()
        }]
    } else {
        Vec::new()
    };
    PolicyEntry {
        policy_id: policy_id.to_owned(),
        pillar: ARCHITECTURE_FAMILY.to_owned(),
        source_path: "docs/standards/source/architecture.adoc".to_owned(),
        source_heading: "Boundaries the Compiler Can See".to_owned(),
        source_anchor: source_anchor.to_owned(),
        summary: "test hard policy".to_owned(),
        classification: Classification::Hard,
        enforcement_level: EnforcementLevel::Hard,
        current_enforcement_mode: EnforcementMode::Gate,
        target_enforcement_mode: EnforcementMode::Gate,
        escalation_criteria: "already gated".to_owned(),
        mechanisms,
        deterministic_checker: EvidenceCommand::default(),
        proof_rails: Vec::new(),
        checker_owner: ARCHITECTURE_FAMILY.to_owned(),
        checker_id: CHECKER_ID.to_owned(),
        default_severity: Severity::Error,
        suppression_requirements: hard_suppression_requirements(),
        fixture_coverage: FixtureCoverage {
            status: FixtureStatus::Planned,
            compliant: Vec::new(),
            noncompliant: Vec::new(),
        },
        verification_evidence_requirements: vec!["compliant fixture".to_owned(), "noncompliant fixture".to_owned()],
        classification_rationale: "deterministic hard policy".to_owned(),
        false_positive_risk: "low".to_owned(),
        existing_enforcement: Vec::new(),
        custom_checker_justification: "test".to_owned(),
    }
}

fn advisory_policy(policy_id: &str, source_anchor: &str) -> PolicyEntry {
    PolicyEntry {
        classification: Classification::Advisory,
        enforcement_level: EnforcementLevel::Advisory,
        default_severity: Severity::Warning,
        suppression_requirements: advisory_suppression_requirements(),
        fixture_coverage: FixtureCoverage {
            status: FixtureStatus::NotApplicable,
            compliant: Vec::new(),
            noncompliant: Vec::new(),
        },
        verification_evidence_requirements: vec![
            "deterministic sample input".to_owned(),
            "stable advisory report".to_owned(),
        ],
        classification_rationale: "surface growth is reviewable but contextual".to_owned(),
        false_positive_risk: "medium".to_owned(),
        ..hard_policy(policy_id, source_anchor)
    }
}

fn hard_suppression_requirements() -> SuppressionRequirements {
    SuppressionRequirements {
        requires_policy_id: true,
        requires_source_section: true,
        requires_owner: true,
        requires_rationale: true,
        requires_review_trigger: true,
        requires_safer_default: true,
    }
}

fn advisory_suppression_requirements() -> SuppressionRequirements {
    SuppressionRequirements {
        requires_policy_id: true,
        requires_source_section: true,
        requires_owner: false,
        requires_rationale: true,
        requires_review_trigger: false,
        requires_safer_default: false,
    }
}
