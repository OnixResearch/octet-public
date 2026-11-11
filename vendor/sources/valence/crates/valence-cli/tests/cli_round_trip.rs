//! CLI round-trip integration test: build a valid artifact, write it to a
//! temp file, run `valence check` on it, and assert it passes. This is the
//! end-to-end smoke for the Evidence IR v1 spine.

use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

use valence_core::evidence_ir::EvidenceSpan;
use valence_core::evidence_ir::KANI_CONTRACT_ARTIFACT_KIND;
use valence_core::evidence_ir::KANI_NON_CLAIM_ALIASING;
use valence_core::evidence_ir::KANI_NON_CLAIM_BOUNDED_ONLY;
use valence_core::evidence_ir::KANI_NON_CLAIM_CONCURRENCY_ASYNC;
use valence_core::evidence_ir::KANI_NON_CLAIM_FFI_STUB;
use valence_core::evidence_ir::KANI_NON_CLAIM_RECURSIVE_TERMINATION;
use valence_core::evidence_ir::KANI_NON_CLAIM_VERIFIER_SOUNDNESS;
use valence_core::evidence_ir::KANI_NON_CLAIM_WHOLE_PROGRAM;
use valence_core::evidence_ir::KaniContractFacet;
use valence_core::evidence_ir::REQUIRED_NON_CLAIM;
use valence_core::evidence_ir::VerificationRole;
use valence_core::evidence_ir::build_evidence_artifact;
use valence_core::function_address_profile::FUNCTION_ADDRESS_PROFILE_VERSION;
use valence_core::function_address_profile::FunctionAddressDigest;
use valence_core::function_address_profile::FunctionAddressDigestDomain;
use valence_core::function_address_profile::FunctionAddressEvidenceEdge;
use valence_core::function_address_profile::FunctionAddressEvidenceInput;
use valence_core::function_address_profile::FunctionAddressEvidenceRecord;
use valence_core::function_address_profile::FunctionAddressEvidenceRole;
use valence_core::function_address_profile::function_address_policy_identity;
use valence_core::stack_provenance_adapter::BUILT_BINARY_THREAD_NON_CLAIM;
use valence_core::stack_provenance_adapter::MANTLE_STACK_PROVENANCE_ROLE;
use valence_core::stack_provenance_adapter::OCTET_VALENCE_EXPORT_SCHEMA;
use valence_core::stack_provenance_adapter::STACK_PROVENANCE_CLAIM_SCOPE;
use valence_core::stack_provenance_adapter::STACK_PROVENANCE_GRAPH_NON_CLAIM;
use valence_core::stack_provenance_adapter::STACK_PROVENANCE_NON_CLAIM;
use valence_core::stack_provenance_adapter::STACK_PROVENANCE_SIDECAR_SCHEMA;

const BINARY_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const STATUS_HASH: &str = "2222222222222222222222222222222222222222222222222222222222222222";
const RESULTS_HASH: &str = "3333333333333333333333333333333333333333333333333333333333333333";
const BUILT_BINARY_BYTES: &[u8] = b"valence-cli-built-binary-fixture";
const TAMPERED_BINARY_BYTES: &[u8] = b"valence-cli-built-binary-fixture-tampered";
const BLAKE3_HEX_LENGTH: usize = 64;
const FUNCTION_SOURCE_DIGEST: &str = "b3:1111111111111111111111111111111111111111111111111111111111111111";
const FUNCTION_ADDRESS_DIGEST: &str = "b3:2222222222222222222222222222222222222222222222222222222222222222";
const FUNCTION_DEPENDENCY_DIGEST: &str = "b3:3333333333333333333333333333333333333333333333333333333333333333";
const FUNCTION_RECEIPT_DIGEST: &str = "b3:4444444444444444444444444444444444444444444444444444444444444444";
const FUNCTION_LOCATOR: &str = "src/lib.rs::f";
const FUNCTION_SPAN_START: u64 = 0;
const FUNCTION_SPAN_END: u64 = 12;
const FUNCTION_SPAN_LINE: u32 = 1;
const FUNCTION_SPAN_START_COLUMN: u32 = 1;
const FUNCTION_SPAN_END_COLUMN: u32 = 13;

fn valence_binary() -> String {
    env!("CARGO_BIN_EXE_valence").to_owned()
}

fn function_address_digest(domain: FunctionAddressDigestDomain, digest: &str) -> FunctionAddressDigest {
    FunctionAddressDigest {
        domain,
        digest: digest.to_owned(),
    }
}

fn function_address_non_claims() -> Vec<String> {
    vec![
        "function-address evidence records source-shape identity only".to_owned(),
        "not Rust semantic correctness".to_owned(),
        "not release eligibility".to_owned(),
    ]
}

fn function_address_input() -> FunctionAddressEvidenceInput {
    FunctionAddressEvidenceInput {
        profile_version: FUNCTION_ADDRESS_PROFILE_VERSION.to_owned(),
        stable_id: "function-address-cli".to_owned(),
        verification_role: VerificationRole::RecordedOnly,
        records: vec![FunctionAddressEvidenceRecord {
            locator: FUNCTION_LOCATOR.to_owned(),
            role: FunctionAddressEvidenceRole::OctetProducer,
            function_address: function_address_digest(
                FunctionAddressDigestDomain::FunctionAddress,
                FUNCTION_ADDRESS_DIGEST,
            ),
            source_identity: function_address_digest(
                FunctionAddressDigestDomain::SourceEnvelope,
                FUNCTION_SOURCE_DIGEST,
            ),
            dependency_closure: Some(function_address_digest(
                FunctionAddressDigestDomain::DependencyClosure,
                FUNCTION_DEPENDENCY_DIGEST,
            )),
            source_span: Some(EvidenceSpan {
                source_id: FUNCTION_SOURCE_DIGEST.to_owned(),
                source_path: "src/lib.rs".to_owned(),
                start_byte: FUNCTION_SPAN_START,
                end_byte: FUNCTION_SPAN_END,
                start_line: FUNCTION_SPAN_LINE,
                start_column: FUNCTION_SPAN_START_COLUMN,
                end_line: FUNCTION_SPAN_LINE,
                end_column: FUNCTION_SPAN_END_COLUMN,
                label: Some(FUNCTION_LOCATOR.to_owned()),
            }),
            caveats: Vec::new(),
        }],
        dependency_edges: vec![FunctionAddressEvidenceEdge {
            from_locator: FUNCTION_LOCATOR.to_owned(),
            to_locator: FUNCTION_LOCATOR.to_owned(),
            edge_kind: "function-depends-on".to_owned(),
        }],
        declared_roles: vec![
            "octet-rust-source-shape-producer".to_owned(),
            "valence-evidence-linkage-adapter".to_owned(),
        ],
        octet_producer_receipt_identity: Some(function_address_digest(
            FunctionAddressDigestDomain::OctetProducerReceipt,
            FUNCTION_RECEIPT_DIGEST,
        )),
        kamacite_receipt_identity: None,
        supplied_kamacite_receipt_identity: None,
        valence_projection_identity: None,
        mantle_sidecar_digest: None,
        release_binary_identity: None,
        policy_identity: Some(function_address_policy_identity()),
        semantic_claims: Vec::new(),
        non_claims: function_address_non_claims(),
    }
}

fn kani_non_claims() -> Vec<String> {
    vec![
        "evidence artifact records identity and linkage only".to_owned(),
        REQUIRED_NON_CLAIM.to_owned(),
        KANI_NON_CLAIM_WHOLE_PROGRAM.to_owned(),
        KANI_NON_CLAIM_VERIFIER_SOUNDNESS.to_owned(),
        KANI_NON_CLAIM_ALIASING.to_owned(),
        KANI_NON_CLAIM_CONCURRENCY_ASYNC.to_owned(),
        KANI_NON_CLAIM_FFI_STUB.to_owned(),
        KANI_NON_CLAIM_RECURSIVE_TERMINATION.to_owned(),
        KANI_NON_CLAIM_BOUNDED_ONLY.to_owned(),
    ]
}

#[test]
fn function_address_validate_emits_receipt_and_graph() {
    let input_json = serde_json::to_string_pretty(&function_address_input()).expect("serialize input");
    let input_path = "/tmp/valence-function-address-valid.json";
    let bundle_path = "/tmp/valence-function-address-bundle.json";
    let receipt_path = "/tmp/valence-function-address-receipt.json";
    let graph_path = "/tmp/valence-function-address-graph.json";
    let mut tmp = fs::File::create(input_path).expect("create temp file");
    tmp.write_all(input_json.as_bytes()).expect("write temp file");

    let output = Command::new(valence_binary())
        .arg("function-address")
        .arg("validate")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("--input")
        .arg("valence-function-address-valid.json")
        .arg("--output-root")
        .arg("/tmp")
        .arg("--bundle")
        .arg("valence-function-address-bundle.json")
        .arg("--receipt")
        .arg("valence-function-address-receipt.json")
        .arg("--graph")
        .arg("valence-function-address-graph.json")
        .output()
        .expect("run valence function-address validate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "function-address validation should succeed: {stdout}");
    assert!(stdout.contains("\"valid\": true"), "stdout should report valid: {stdout}");
    assert!(stdout.contains("\"graph_identity\""), "stdout should include graph identity: {stdout}");
    let bundle = fs::read_to_string(bundle_path).expect("read bundle");
    let receipt = fs::read_to_string(receipt_path).expect("read receipt");
    let graph = fs::read_to_string(graph_path).expect("read graph");
    assert!(bundle.contains("valence.function-address-publication.v1"));
    assert!(receipt.contains("function-address-evidence-v1"));
    assert!(graph.contains("function-depends-on"));
}

#[test]
fn function_address_validate_rejects_role_promotion_and_overclaim() {
    let mut input = function_address_input();
    input.verification_role = VerificationRole::Property;
    input.semantic_claims = vec!["proves semantic correctness".to_owned()];
    let input_json = serde_json::to_string_pretty(&input).expect("serialize input");
    let input_path = "/tmp/valence-function-address-invalid.json";
    let bundle_path = "/tmp/valence-function-address-invalid-bundle.json";
    let receipt_path = "/tmp/valence-function-address-invalid-receipt.json";
    let graph_path = "/tmp/valence-function-address-invalid-graph.json";
    const SENTINEL: &str = "preserve-existing-output";
    let mut tmp = fs::File::create(input_path).expect("create temp file");
    tmp.write_all(input_json.as_bytes()).expect("write temp file");
    write_text(bundle_path, SENTINEL);
    write_text(receipt_path, SENTINEL);
    write_text(graph_path, SENTINEL);

    let output = Command::new(valence_binary())
        .arg("function-address")
        .arg("validate")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("--input")
        .arg("valence-function-address-invalid.json")
        .arg("--output-root")
        .arg("/tmp")
        .arg("--bundle")
        .arg("valence-function-address-invalid-bundle.json")
        .arg("--receipt")
        .arg("valence-function-address-invalid-receipt.json")
        .arg("--graph")
        .arg("valence-function-address-invalid-graph.json")
        .output()
        .expect("run valence function-address validate");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "function-address validation should fail: {stdout}");
    assert!(stdout.contains("formal_proof_promotion_denied"));
    assert!(stdout.contains("overclaim"));
    assert_eq!(fs::read_to_string(bundle_path).expect("bundle sentinel"), SENTINEL);
    assert_eq!(fs::read_to_string(receipt_path).expect("receipt sentinel"), SENTINEL);
    assert_eq!(fs::read_to_string(graph_path).expect("graph sentinel"), SENTINEL);
}

#[test]
fn function_address_bundle_staging_failure_preserves_existing_output() {
    // r[verify valence.evidence_ir.function_address_publication.fixtures.negative]
    let input_json = serde_json::to_string_pretty(&function_address_input()).expect("serialize input");
    let input_path = "/tmp/valence-function-address-stage-failure.json";
    let bundle_path = "/tmp/valence-function-address-stage-failure-bundle.json";
    let staging_path = "/tmp/.valence-function-address-stage-failure-bundle.json.valence-stage";
    const SENTINEL: &str = "prior-valid-bundle";
    write_text(input_path, &input_json);
    write_text(bundle_path, SENTINEL);
    let _ = fs::remove_file(staging_path);
    let _ = fs::remove_dir_all(staging_path);
    fs::create_dir(staging_path).expect("create staging blocker");

    let output = Command::new(valence_binary())
        .args([
            "function-address",
            "validate",
            "--artifact-root",
            "/tmp",
            "--input",
            "valence-function-address-stage-failure.json",
            "--output-root",
            "/tmp",
            "--bundle",
            "valence-function-address-stage-failure-bundle.json",
        ])
        .output()
        .expect("run bundle staging failure");

    assert!(!output.status.success());
    assert_eq!(fs::read_to_string(bundle_path).expect("prior bundle"), SENTINEL);
    fs::remove_dir(staging_path).expect("remove staging blocker");
}

#[test]
fn declared_roots_fail_closed_for_omission_absolute_locators_and_traversal() {
    // r[verify valence.stack_provenance_adapter.declared_capability_roots.fixtures.negative]
    let omitted_check = Command::new(valence_binary())
        .arg("check")
        .arg("artifact.json")
        .output()
        .expect("run check without root");
    assert!(!omitted_check.status.success());

    for locator in [
        "/tmp/valence-cli-valid-artifact.json",
        "../valence-cli-valid-artifact.json",
    ] {
        let output = Command::new(valence_binary())
            .arg("check")
            .arg("--artifact-root")
            .arg("/tmp")
            .arg(locator)
            .output()
            .expect("run check escape fixture");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(!output.status.success());
        assert!(stderr.contains("capability_locator_"), "missing locator diagnostic: {stderr}");
    }

    let omitted_explain = Command::new(valence_binary())
        .args(["explain", "--id", "id", "--manifest", "manifest.json"])
        .output()
        .expect("run explain without root");
    assert!(!omitted_explain.status.success());
    let omitted_stack = Command::new(valence_binary())
        .args([
            "stack-provenance",
            "build",
            "--octet-export",
            "export.json",
            "--binary-hash",
            BINARY_HASH,
            "--output",
            "sidecar.json",
        ])
        .output()
        .expect("run stack build without roots");
    assert!(!omitted_stack.status.success());
    let omitted_function = Command::new(valence_binary())
        .args(["function-address", "validate", "--input", "input.json"])
        .output()
        .expect("run function-address without root");
    assert!(!omitted_function.status.success());
}

#[test]
fn check_accepts_library_emitted_artifact() {
    let artifact = build_evidence_artifact(
        "function-object-v1",
        "eq:bs_001",
        Some(EvidenceSpan {
            source_id: "eq:bs_001".to_owned(),
            source_path: "src/pricing.rs".to_owned(),
            start_byte: 142,
            end_byte: 168,
            start_line: 7,
            start_column: 12,
            end_line: 7,
            end_column: 38,
            label: Some("eq:bs".to_owned()),
        }),
        VerificationRole::RecordedOnly,
        serde_json::json!({"function": "black_scholes_call", "body_hash": "b3:abc123"}),
        vec![
            "evidence artifact records identity and linkage only".to_owned(),
            REQUIRED_NON_CLAIM.to_owned(),
        ],
    )
    .expect("build artifact");

    let json = serde_json::to_string_pretty(&artifact).expect("serialize artifact");

    let mut tmp = fs::File::create("/tmp/valence-cli-valid-artifact.json").expect("create temp file");
    tmp.write_all(json.as_bytes()).expect("write temp file");

    let output = Command::new(valence_binary())
        .arg("check")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("valence-cli-valid-artifact.json")
        .output()
        .expect("run valence check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "check should succeed: {stdout}");
    assert!(stdout.contains("valid: true"), "stdout should report valid: {stdout}");
    assert!(stdout.contains("issue_count=0"), "stdout should report zero issues: {stdout}");
}

#[test]
fn check_prints_kani_non_claim_boundaries() {
    let artifact = build_evidence_artifact(
        KANI_CONTRACT_ARTIFACT_KIND,
        "kani:contract:cli-boundary",
        None,
        VerificationRole::Property,
        serde_json::json!({
            "contract_facets": [
                KaniContractFacet::Postcondition.as_str(),
                KaniContractFacet::ContractHarness.as_str(),
            ],
            "target": {
                "crate_target": "valence-core",
                "source_id": "src/lib.rs::checked_add",
                "target_scope": "fn checked_add::<u32>",
                "monomorphization": "u32",
            },
            "toolchain": {
                "kani_version": "kani fixture",
                "rust_toolchain": "nightly fixture",
                "cbmc_version": "cbmc fixture",
                "solver": "minisat fixture",
            },
            "receipt": {
                "status": "passed",
                "receipt_hash": "b3:kani-contract-receipt",
            },
            "claim_classification": "passing_contract",
            "termination_claimed": false,
        }),
        kani_non_claims(),
    )
    .expect("build Kani artifact");
    let json = serde_json::to_string_pretty(&artifact).expect("serialize Kani artifact");

    let mut tmp = fs::File::create("/tmp/valence-cli-kani-artifact.json").expect("create temp file");
    tmp.write_all(json.as_bytes()).expect("write temp file");

    let output = Command::new(valence_binary())
        .arg("check")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("valence-cli-kani-artifact.json")
        .output()
        .expect("run valence check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "Kani check should succeed: {stdout}");
    assert!(stdout.contains("valid: true"), "stdout should report valid: {stdout}");
    assert!(stdout.contains("non_claim_count="), "stdout should report non-claim count: {stdout}");
    assert!(
        stdout.contains(KANI_NON_CLAIM_VERIFIER_SOUNDNESS),
        "stdout should report Kani verifier-soundness boundary: {stdout}"
    );
}

#[test]
fn check_rejects_overclaim_artifact() {
    let artifact = build_evidence_artifact(
        "test",
        "overclaim_test",
        None,
        VerificationRole::Property,
        serde_json::json!({"claim": "proves semantic equivalence"}),
        vec!["records identity only".to_owned(), REQUIRED_NON_CLAIM.to_owned()],
    )
    .expect("build artifact");

    let json = serde_json::to_string(&artifact).expect("serialize artifact");

    let mut tmp = fs::File::create("/tmp/valence-cli-overclaim-artifact.json").expect("create temp file");
    tmp.write_all(json.as_bytes()).expect("write temp file");

    let output = Command::new(valence_binary())
        .arg("check")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("valence-cli-overclaim-artifact.json")
        .output()
        .expect("run valence check");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "check should fail for overclaim: {stdout}");
    assert!(stdout.contains("valid: false"), "stdout should report invalid: {stdout}");
    assert!(stdout.contains("overclaim_detected"), "stdout should report overclaim_detected: {stdout}");
}

#[test]
fn explain_resolves_known_id_from_manifest() {
    let span = EvidenceSpan {
        source_id: "eq:bs_001".to_owned(),
        source_path: "src/pricing.rs".to_owned(),
        start_byte: 142,
        end_byte: 168,
        start_line: 7,
        start_column: 12,
        end_line: 7,
        end_column: 38,
        label: Some("eq:bs".to_owned()),
    };
    let manifest = serde_json::to_string_pretty(&[span]).expect("serialize manifest");

    let mut tmp = fs::File::create("/tmp/valence-cli-spans.json").expect("create temp file");
    tmp.write_all(manifest.as_bytes()).expect("write temp file");

    let output = Command::new(valence_binary())
        .arg("explain")
        .arg("--manifest-root")
        .arg("/tmp")
        .arg("--id")
        .arg("eq:bs_001")
        .arg("--manifest")
        .arg("valence-cli-spans.json")
        .output()
        .expect("run valence explain");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "explain should succeed: {stdout}");
    assert!(stdout.contains("source_id: eq:bs_001"), "stdout should report source_id: {stdout}");
    assert!(stdout.contains("source_path: src/pricing.rs"), "stdout should report path: {stdout}");
    assert!(stdout.contains("label: eq:bs"), "stdout should report label: {stdout}");
}

#[test]
fn explain_reports_not_found_for_unknown_id() {
    let span = EvidenceSpan {
        source_id: "eq:bs_001".to_owned(),
        source_path: "src/pricing.rs".to_owned(),
        start_byte: 142,
        end_byte: 168,
        start_line: 7,
        start_column: 12,
        end_line: 7,
        end_column: 38,
        label: None,
    };
    let manifest = serde_json::to_string(&[span]).expect("serialize manifest");

    let mut tmp = fs::File::create("/tmp/valence-cli-spans-notfound.json").expect("create temp file");
    tmp.write_all(manifest.as_bytes()).expect("write temp file");

    let output = Command::new(valence_binary())
        .arg("explain")
        .arg("--manifest-root")
        .arg("/tmp")
        .arg("--id")
        .arg("nonexistent")
        .arg("--manifest")
        .arg("valence-cli-spans-notfound.json")
        .output()
        .expect("run valence explain");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "explain should fail for unknown id: {stdout}");
    assert!(stdout.contains("not_found: nonexistent"), "stdout should report not_found: {stdout}");
}

#[test]
fn explain_rejects_duplicate_and_invalid_manifest_without_selecting_span() {
    let span = EvidenceSpan {
        source_id: "eq:duplicate".to_owned(),
        source_path: "src/pricing.rs".to_owned(),
        start_byte: 10,
        end_byte: 5,
        start_line: 0,
        start_column: 1,
        end_line: 1,
        end_column: 1,
        label: None,
    };
    let manifest = serde_json::to_string(&[span.clone(), span]).expect("serialize manifest");
    let manifest_path = "/tmp/valence-cli-spans-invalid.json";
    write_text(manifest_path, &manifest);

    let output = Command::new(valence_binary())
        .arg("explain")
        .arg("--manifest-root")
        .arg("/tmp")
        .arg("--id")
        .arg("eq:duplicate")
        .arg("--manifest")
        .arg("valence-cli-spans-invalid.json")
        .output()
        .expect("run valence explain");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "invalid manifest must fail");
    assert!(stdout.is_empty(), "invalid manifest must not print a selected span: {stdout}");
    assert!(!stderr.contains("not_found"), "invalid manifest is distinct from not found: {stderr}");
    assert!(stderr.contains("manifest_duplicate_source_id"), "duplicate diagnostic missing: {stderr}");
    assert!(stderr.contains("span_bytes_reversed"), "coordinate diagnostic missing: {stderr}");
}

#[test]
fn stack_provenance_builds_and_verifies_mantle_sidecar() {
    let export_path = "/tmp/valence-cli-octet-export.json";
    let sidecar_path = "/tmp/valence-cli-stack-sidecar.json";
    let manifest_path = "/tmp/valence-cli-mantle-manifest.json";
    write_text(export_path, &sample_octet_export_json());

    let build_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("build")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("--octet-export")
        .arg("valence-cli-octet-export.json")
        .arg("--binary-hash")
        .arg(BINARY_HASH)
        .arg("--output-root")
        .arg("/tmp")
        .arg("--output")
        .arg("valence-cli-stack-sidecar.json")
        .output()
        .expect("run stack provenance build");
    let build_stdout = String::from_utf8_lossy(&build_output.stdout);
    assert!(build_output.status.success(), "build should succeed: {build_stdout}");
    assert!(build_stdout.contains("valid: true"), "stdout should report valid: {build_stdout}");

    let sidecar_json = fs::read_to_string(sidecar_path).expect("read sidecar");
    let digest = blake3::hash(sidecar_json.as_bytes()).to_hex().to_string();
    write_text(manifest_path, &mantle_manifest_json(&digest));

    let verify_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("verify")
        .arg("--manifest-root")
        .arg("/tmp")
        .arg("--manifest")
        .arg("valence-cli-mantle-manifest.json")
        .arg("--sidecar-root")
        .arg("/tmp")
        .arg("--sidecar")
        .arg("valence-cli-stack-sidecar.json")
        .output()
        .expect("run stack provenance verify");
    let verify_stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(verify_output.status.success(), "verify should succeed: {verify_stdout}");
    assert!(verify_stdout.contains("issue_count=0"), "stdout should report zero issues: {verify_stdout}");
}

#[test]
fn stack_provenance_verify_rejects_tampered_sidecar_digest() {
    let export_path = "/tmp/valence-cli-octet-export-tamper.json";
    let manifest_path = "/tmp/valence-cli-mantle-manifest-tamper.json";
    write_text(export_path, &sample_octet_export_json());

    let build_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("build")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("--octet-export")
        .arg("valence-cli-octet-export-tamper.json")
        .arg("--binary-hash")
        .arg(BINARY_HASH)
        .arg("--output-root")
        .arg("/tmp")
        .arg("--output")
        .arg("valence-cli-stack-sidecar-tamper.json")
        .output()
        .expect("run stack provenance build");
    assert!(build_output.status.success(), "build should succeed before tamper");
    write_text(manifest_path, &mantle_manifest_json(STATUS_HASH));

    let verify_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("verify")
        .arg("--manifest-root")
        .arg("/tmp")
        .arg("--manifest")
        .arg("valence-cli-mantle-manifest-tamper.json")
        .arg("--sidecar-root")
        .arg("/tmp")
        .arg("--sidecar")
        .arg("valence-cli-stack-sidecar-tamper.json")
        .output()
        .expect("run stack provenance verify");
    let verify_stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(!verify_output.status.success(), "verify should reject digest: {verify_stdout}");
    assert!(
        verify_stdout.contains("sidecar_digest_mismatch"),
        "stdout should report digest mismatch: {verify_stdout}"
    );
}

#[test]
fn stack_provenance_verify_built_binary_accepts_measured_thread() {
    let binary_path = "/tmp/valence-cli-built-binary";
    let export_path = "/tmp/valence-cli-built-binary-octet-export.json";
    let sidecar_path = "/tmp/valence-cli-built-binary-sidecar.json";
    let manifest_path = "/tmp/valence-cli-built-binary-mantle-manifest.json";
    let binary_hash = blake3::hash(BUILT_BINARY_BYTES).to_hex().to_string();
    fs::write(binary_path, BUILT_BINARY_BYTES).expect("write binary fixture");
    write_text(export_path, &sample_octet_export_json());

    let build_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("build")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("--octet-export")
        .arg("valence-cli-built-binary-octet-export.json")
        .arg("--binary-hash")
        .arg(&binary_hash)
        .arg("--output-root")
        .arg("/tmp")
        .arg("--output")
        .arg("valence-cli-built-binary-sidecar.json")
        .output()
        .expect("run stack provenance build");
    assert!(build_output.status.success(), "build should succeed");

    let sidecar_json = fs::read_to_string(sidecar_path).expect("read sidecar");
    let digest = blake3::hash(sidecar_json.as_bytes()).to_hex().to_string();
    write_text(manifest_path, &mantle_release_manifest_json(&digest, &binary_hash));

    let verify_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("verify-built-binary")
        .arg("--binary-root")
        .arg("/tmp")
        .arg("--binary")
        .arg("valence-cli-built-binary")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("--octet-export")
        .arg("valence-cli-built-binary-octet-export.json")
        .arg("--sidecar-root")
        .arg("/tmp")
        .arg("--sidecar")
        .arg("valence-cli-built-binary-sidecar.json")
        .arg("--manifest-root")
        .arg("/tmp")
        .arg("--manifest")
        .arg("valence-cli-built-binary-mantle-manifest.json")
        .arg("--json")
        .output()
        .expect("run built binary verify");
    let stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(verify_output.status.success(), "verify should succeed: {stdout}");
    let report: serde_json::Value = serde_json::from_slice(&verify_output.stdout).expect("json report");
    assert_eq!(report["valid"], true);
    assert_eq!(report["binary_hash"], binary_hash);
    assert!(
        report["non_claims"]
            .as_array()
            .expect("non claims")
            .iter()
            .any(|claim| claim == BUILT_BINARY_THREAD_NON_CLAIM)
    );
    assert_eq!(report["preserves_content_hash"].as_str().expect("preserves content hash").len(), BLAKE3_HEX_LENGTH);
}

#[test]
fn stack_provenance_verify_built_binary_rejects_tampered_bytes() {
    let binary_path = "/tmp/valence-cli-built-binary-tampered";
    let export_path = "/tmp/valence-cli-built-binary-octet-export-tampered.json";
    let sidecar_path = "/tmp/valence-cli-built-binary-sidecar-tampered.json";
    let manifest_path = "/tmp/valence-cli-built-binary-mantle-manifest-tampered.json";
    let binary_hash = blake3::hash(BUILT_BINARY_BYTES).to_hex().to_string();
    fs::write(binary_path, TAMPERED_BINARY_BYTES).expect("write binary fixture");
    write_text(export_path, &sample_octet_export_json());

    let build_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("build")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("--octet-export")
        .arg("valence-cli-built-binary-octet-export-tampered.json")
        .arg("--binary-hash")
        .arg(&binary_hash)
        .arg("--output-root")
        .arg("/tmp")
        .arg("--output")
        .arg("valence-cli-built-binary-sidecar-tampered.json")
        .output()
        .expect("run stack provenance build");
    assert!(build_output.status.success(), "build should succeed before tamper");

    let sidecar_json = fs::read_to_string(sidecar_path).expect("read sidecar");
    let digest = blake3::hash(sidecar_json.as_bytes()).to_hex().to_string();
    write_text(manifest_path, &mantle_release_manifest_json(&digest, &binary_hash));

    let verify_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("verify-built-binary")
        .arg("--binary-root")
        .arg("/tmp")
        .arg("--binary")
        .arg("valence-cli-built-binary-tampered")
        .arg("--artifact-root")
        .arg("/tmp")
        .arg("--octet-export")
        .arg("valence-cli-built-binary-octet-export-tampered.json")
        .arg("--sidecar-root")
        .arg("/tmp")
        .arg("--sidecar")
        .arg("valence-cli-built-binary-sidecar-tampered.json")
        .arg("--manifest-root")
        .arg("/tmp")
        .arg("--manifest")
        .arg("valence-cli-built-binary-mantle-manifest-tampered.json")
        .output()
        .expect("run built binary verify");
    let stdout = String::from_utf8_lossy(&verify_output.stdout);
    assert!(!verify_output.status.success(), "verify should reject tamper: {stdout}");
    assert!(stdout.contains("binary_digest_mismatch"), "stdout should report binary mismatch: {stdout}");
    assert!(
        stdout.contains("missing_mantle_binary_identity"),
        "stdout should report missing binary identity: {stdout}"
    );
}

#[test]
fn stack_provenance_graph_report_outputs_json_and_markdown() {
    let fixture = graph_fixture_path("positive/complete.json");
    let fixture_root = Path::new(&fixture).parent().expect("fixture root");
    let fixture_locator = Path::new(&fixture).file_name().expect("fixture locator");
    let json_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("graph-report")
        .arg("--artifact-root")
        .arg(fixture_root)
        .arg("--chain")
        .arg(fixture_locator)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run graph report json");
    let stdout = String::from_utf8_lossy(&json_output.stdout);
    assert!(json_output.status.success(), "graph report should pass: {stdout}");
    let report: serde_json::Value = serde_json::from_slice(&json_output.stdout).expect("graph report json");
    assert_eq!(report["valid"], true);
    assert_eq!(report["edge_count"], 6);
    assert_eq!(report["report_hash"].as_str().expect("report hash").len(), BLAKE3_HEX_LENGTH);
    assert!(
        report["non_claims"]
            .as_array()
            .expect("non claims")
            .iter()
            .any(|claim| claim == STACK_PROVENANCE_GRAPH_NON_CLAIM)
    );

    let markdown_output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("graph-report")
        .arg("--artifact-root")
        .arg(fixture_root)
        .arg("--chain")
        .arg(fixture_locator)
        .arg("--format")
        .arg("markdown")
        .output()
        .expect("run graph report markdown");
    let markdown = String::from_utf8_lossy(&markdown_output.stdout);
    assert!(markdown_output.status.success(), "markdown graph report should pass: {markdown}");
    assert!(markdown.contains("# Stack provenance graph report"), "markdown has heading: {markdown}");
    assert!(markdown.contains("Cairn lifecycle-gate nodes"), "markdown has Cairn boundary: {markdown}");
}

#[test]
fn stack_provenance_graph_report_rejects_unsupported_edge_fixture() {
    let fixture = graph_fixture_path("negative/unsupported-edge.json");
    let fixture_root = Path::new(&fixture).parent().expect("fixture root");
    let fixture_locator = Path::new(&fixture).file_name().expect("fixture locator");
    let output = Command::new(valence_binary())
        .arg("stack-provenance")
        .arg("graph-report")
        .arg("--artifact-root")
        .arg(fixture_root)
        .arg("--chain")
        .arg(fixture_locator)
        .arg("--format")
        .arg("json")
        .output()
        .expect("run graph report negative");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!output.status.success(), "graph report should reject unsupported edge: {stdout}");
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("graph report json");
    assert_eq!(report["valid"], false);
    assert!(
        report["diagnostics"]
            .as_array()
            .expect("diagnostics")
            .iter()
            .any(|code| code == "unsupported_edge_kind")
    );
}

#[test]
fn version_command_succeeds() {
    let output = Command::new(valence_binary()).arg("version").output().expect("run valence version");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(output.status.success(), "version should succeed: {stdout}");
    assert!(stdout.contains("valence"), "stdout should contain 'valence': {stdout}");
}

fn graph_fixture_path(relative: &str) -> String {
    let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .join("../../fixtures/stack-provenance-chain")
        .join(relative)
        .to_string_lossy()
        .into_owned()
}

fn write_text(path: &str, content: &str) {
    let mut tmp = fs::File::create(path).expect("create temp file");
    tmp.write_all(content.as_bytes()).expect("write temp file");
}

fn sample_octet_export_json() -> String {
    serde_json::json!({
        "schema_version": OCTET_VALENCE_EXPORT_SCHEMA,
        "artifact_dir": "target/octet-artifact",
        "verification_status": "valid",
        "artifacts": [
            {
                "artifact": "status.json",
                "blake3": STATUS_HASH
            },
            {
                "artifact": "results.jsonl",
                "blake3": RESULTS_HASH
            }
        ],
        "provenance_records": [
            {
                "source_id": "octet.standards.formal-proof",
                "checker_id": "octet-artifact-verifier",
                "evidence_tier": "formal_proof",
                "artifact": "status.json",
                "blake3": STATUS_HASH,
                "checked_scope": "standards/proof-claims/catalog.ncl",
                "verifier_identity": "octet-verus-rail",
                "proof_artifact_identity": RESULTS_HASH,
                "parent_artifacts": ["results.jsonl"]
            }
        ],
        "proof_cards": [
            {
                "proof_or_property_id": "octet.no-unwrap-proof",
                "source_requirement_or_policy_id": "r[octet.policy.no_unwrap]",
                "policy_id": "octet.standards.formal-proof",
                "rail_index": 0,
                "receipt_path": "status.json",
                "artifact_blake3_identities": [
                    {
                        "artifact": "status.json",
                        "blake3": STATUS_HASH
                    }
                ],
                "evidence_tier": "formal_proof",
                "evidence_origin": "octet_verified_receipt",
                "status": "valid",
                "non_claims": [STACK_PROVENANCE_NON_CLAIM]
            }
        ],
        "diagnostics": [],
        "non_claims": [STACK_PROVENANCE_NON_CLAIM]
    })
    .to_string()
}

fn mantle_manifest_json(digest: &str) -> String {
    serde_json::json!({
        "external_evidence": [
            {
                "role": MANTLE_STACK_PROVENANCE_ROLE,
                "schema": STACK_PROVENANCE_SIDECAR_SCHEMA,
                "relative_path": "external-evidence/stack-provenance.json",
                "digest_blake3": digest,
                "claim_scope": STACK_PROVENANCE_CLAIM_SCOPE,
                "non_claims": [STACK_PROVENANCE_NON_CLAIM]
            }
        ]
    })
    .to_string()
}

fn mantle_release_manifest_json(digest: &str, binary_hash: &str) -> String {
    serde_json::json!({
        "binaries": [
            {
                "relative_path": "bin/valence-fixture",
                "digest_blake3": binary_hash
            }
        ],
        "external_evidence": [
            {
                "role": MANTLE_STACK_PROVENANCE_ROLE,
                "schema": STACK_PROVENANCE_SIDECAR_SCHEMA,
                "relative_path": "external-evidence/stack-provenance.json",
                "digest_blake3": digest,
                "claim_scope": STACK_PROVENANCE_CLAIM_SCOPE,
                "non_claims": [STACK_PROVENANCE_NON_CLAIM]
            }
        ],
        "provenance_coverage": {
            "binary_hash": binary_hash
        }
    })
    .to_string()
}
