//! `artifact verify` re-derives an integrated admitted effect-closure receipt from its bound
//! function-address report and workspace policy, so a denied closure edited and re-identified
//! without signatures cannot verify as admitted.
use octet_standards::project_architecture_gate as gate;

use super::ownership::insert_json;
use super::ownership::reseal_bundle;
use super::ownership::reseal_receipt;
use super::ownership::set_status_phase;
use super::ownership::store_receipt;
use super::ownership::synthetic_files;
use super::*;
use crate::effect_closure::AdmissionStatus;
use crate::effect_closure::CallSummary;
use crate::effect_closure::ClosureInput;
use crate::effect_closure::EffectClosureReceipt;
use crate::effect_closure::ModuleRole;
use crate::effect_closure::ModuleRoleAssignment;
use crate::effect_closure::{self as closure};

const SOURCE_PATH: &str = "src/lib.rs";
const FUNCTION_ADDRESS: &str = "b3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff";
const RECORD_ROOT: &str = "b3:1111111111111111111111111111111111111111111111111111111111111111";
const IDENTITY_HASH: &str = "b3:3333333333333333333333333333333333333333333333333333333333333333";
const CARGO_LOCK: &str = "b3:2222222222222222222222222222222222222222222222222222222222222222";
const TARGET: &str = "x86_64-unknown-linux-gnu";
const TOOLCHAIN: &str = "nightly-fixture";
const PARENT_STATUS: &[u8] = b"{\"phase\":\"integrated-parent-fixture\"}\n";
const ALLOCATING_SOURCE: &str = "fn root() { let _values: Vec<u8> = Vec::new(); }\n";

struct ClosureFixture {
    files: BTreeMap<String, Vec<u8>>,
    input: ClosureInput,
}

fn toolchain_summary(shape: &str) -> CallSummary {
    let provider = closure::ProviderIdentity {
        kind: closure::ProviderKind::Toolchain,
        name: "alloc".to_owned(),
        version: "rustc-fixture".to_owned(),
        source: "rust-toolchain".to_owned(),
        target: TARGET.to_owned(),
        features: Vec::new(),
    };
    let scope = closure::prelude_call_scope(SOURCE_PATH, shape);
    CallSummary {
        call_site_id: closure::call_site_identity(FUNCTION_ADDRESS, SOURCE_PATH, &scope, shape, 0),
        review_label: format!("reviewed `{shape}` toolchain call"),
        callee: shape.trim_start_matches("call:").to_owned(),
        effects: vec![closure::Effect::Alloc],
        evidence: closure::EvidenceReference {
            kind: "toolchain-api-review-v1".to_owned(),
            locator: "rustc-version#alloc".to_owned(),
            blake3: closure::provider_identity_blake3(&provider),
        },
        provider,
    }
}

/// An integrated check bundle (lint, architecture, and closure artifacts) whose closure inputs bind
/// one selected source with one `root` function.
fn closure_fixture(source: &str, role: ModuleRole, call_summaries: Vec<CallSummary>) -> ClosureFixture {
    let mut files = synthetic_files();
    let source_identity = crate::determinism::full_hash(source);
    let records = vec![closure::FunctionAddressRecord {
        path: SOURCE_PATH.to_owned(),
        name: "root".to_owned(),
        function_address: FUNCTION_ADDRESS.to_owned(),
        source_identity: source_identity.clone(),
    }];
    insert_json(
        &mut files,
        FUNCTION_ADDRESS_ARTIFACT_NAME,
        &serde_json::json!({
            "mode": "deny",
            "source_envelopes": [{"path": SOURCE_PATH, "source_identity": source_identity}],
            "records": records,
            "evidence_chain_handoff": {
                "canonical_roots": [{"kind": "function-address-record-root", "digest": RECORD_ROOT}]
            }
        }),
    );
    let mut providers = call_summaries.iter().map(|summary| summary.provider.clone()).collect::<Vec<_>>();
    providers.sort();
    providers.dedup();
    let policy = closure::WorkspacePolicy {
        schema_version: closure::WORKSPACE_POLICY_SCHEMA_VERSION.to_owned(),
        profile_id: closure::DETERMINISTIC_CORE_PROFILE.to_owned(),
        function_address_baseline: FUNCTION_ADDRESS_ARTIFACT_NAME.to_owned(),
        module_roles: vec![ModuleRoleAssignment {
            path: SOURCE_PATH.to_owned(),
            role,
        }],
        call_summaries,
        non_claims: vec!["bounded fixture policy".to_owned()],
    };
    insert_json(&mut files, EFFECT_CLOSURE_POLICY_ARTIFACT_NAME, &policy);
    insert_json(&mut files, EFFECT_CLOSURE_GRAPH_ARTIFACT_NAME, &crate::admit::DependencyGraphArtifact {
        schema_version: "octet-effect-closure-dependency-graph/v1".to_owned(),
        target: TARGET.to_owned(),
        toolchain: TOOLCHAIN.to_owned(),
        rustc_version: "rustc-fixture".to_owned(),
        cargo_lock_blake3: CARGO_LOCK.to_owned(),
        providers: providers.clone(),
    });
    files.insert(EFFECT_CLOSURE_PARENT_STATUS_ARTIFACT_NAME.to_owned(), PARENT_STATUS.to_vec());
    let identity = |name: &str| crate::evidence::artifact_blake3_hex(&files[name]);
    let input = ClosureInput {
        profile_id: closure::DETERMINISTIC_CORE_PROFILE.to_owned(),
        profile_hash: IDENTITY_HASH.to_owned(),
        config_hash: IDENTITY_HASH.to_owned(),
        command_blake3: identity(COMMAND_ARTIFACT_NAME),
        check_status_blake3: identity(EFFECT_CLOSURE_PARENT_STATUS_ARTIFACT_NAME),
        function_address_artifact_blake3: identity(FUNCTION_ADDRESS_ARTIFACT_NAME),
        function_address_record_root: RECORD_ROOT.to_owned(),
        workspace_policy_blake3: identity(EFFECT_CLOSURE_POLICY_ARTIFACT_NAME),
        cargo_graph_blake3: identity(EFFECT_CLOSURE_GRAPH_ARTIFACT_NAME),
        cargo_lock_blake3: CARGO_LOCK.to_owned(),
        target: TARGET.to_owned(),
        toolchain: TOOLCHAIN.to_owned(),
        sources: vec![closure::SourceEnvelope {
            path: SOURCE_PATH.to_owned(),
            source: source.to_owned(),
            source_identity,
        }],
        function_addresses: records,
        resolved_providers: providers,
        catalog: closure::embedded_catalog().expect("embedded catalog"),
        policy,
    };
    ClosureFixture { files, input }
}

/// Stores `receipt` the way the runner writes an integrated admission: the closure receipt, the
/// architecture receipt's closure link, `status.json`, and provenance all agree with it.
fn integrated_bundle(files: BTreeMap<String, Vec<u8>>, receipt: &EffectClosureReceipt) -> ArtifactBundleInput {
    integrated_bundle_storing(files, receipt, &serde_json::to_value(receipt).expect("receipt JSON"))
}

/// Like [`integrated_bundle`], but stores `stored` as the closure receipt artifact while linking
/// the architecture receipt and status to `receipt`.
fn integrated_bundle_storing(
    mut files: BTreeMap<String, Vec<u8>>,
    receipt: &EffectClosureReceipt,
    stored: &serde_json::Value,
) -> ArtifactBundleInput {
    insert_json(&mut files, EFFECT_CLOSURE_RECEIPT_ARTIFACT_NAME, stored);
    let mut parent: gate::ProjectArchitectureReceipt =
        serde_json::from_slice(&files[PROJECT_ARCHITECTURE_RECEIPT_ARTIFACT_NAME]).expect("architecture receipt");
    parent.effect_closure = Some(gate::EffectClosureParent {
        receipt_id: receipt.receipt_identity.clone(),
        admitted: receipt.status == AdmissionStatus::Admitted,
        evidence_tier: EvidenceTier::PolicyCheck.as_str().to_owned(),
        non_claims: receipt.non_claims.clone(),
    });
    reseal_receipt(&mut parent);
    store_receipt(&mut files, &parent);
    set_status_phase(
        &mut files,
        "effect_closure",
        serde_json::to_value(&parent.effect_closure).expect("closure link JSON"),
    );
    reseal_bundle(files)
}

fn forge_admitted(mut receipt: EffectClosureReceipt) -> EffectClosureReceipt {
    receipt.diagnostics.clear();
    receipt.status = AdmissionStatus::Admitted;
    closure::finalize_receipt(receipt)
}

fn assert_rejected(input: ArtifactBundleInput, expected: &str) {
    let report = verify_bundle(input);
    assert_eq!(report.status, STATUS_INVALID, "forged integrated closure verified");
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
        "expected `{expected}` in {:?}",
        report.diagnostics
    );
}

// r[verify octet.effect_closure.evidence.tamper]
#[test]
fn integrated_admitted_closure_verifies_and_denied_relabel_is_rejected() {
    let admitted = closure_fixture(ALLOCATING_SOURCE, ModuleRole::Core, vec![toolchain_summary("call:Vec::new")]);
    let receipt = closure::compute_closure(admitted.input.clone());
    assert_eq!(receipt.status, AdmissionStatus::Admitted, "{:?}", receipt.diagnostics);
    let report = verify_bundle(integrated_bundle(admitted.files, &receipt));
    assert_eq!(report.status, STATUS_VALID, "{:?}", report.diagnostics);

    let stale = closure_fixture("fn root() {}\n", ModuleRole::Core, vec![toolchain_summary("call:gone")]);
    let denied = closure::compute_closure(stale.input.clone());
    assert_eq!(denied.status, AdmissionStatus::Denied);
    assert!(denied.diagnostics.iter().any(|diagnostic| diagnostic.code == "call_summary_stale"));
    let report = verify_bundle(integrated_bundle(stale.files.clone(), &denied));
    assert_eq!(report.status, STATUS_VALID, "{:?}", report.diagnostics);
    assert_rejected(integrated_bundle(stale.files, &forge_admitted(denied)), "is not used by the admitted receipt");

    let shell = closure_fixture("fn root() {}\n", ModuleRole::Shell, Vec::new());
    let denied = closure::compute_closure(shell.input.clone());
    assert_eq!(denied.status, AdmissionStatus::Denied);
    assert_rejected(
        integrated_bundle(shell.files, &forge_admitted(denied)),
        "exactly one workspace-policy role `core`",
    );
}

/// Rewrites `function-addresses.json` and rebinds the receipt to the new bytes, as an attacker who
/// can recompute unkeyed identities would.
fn forge_function_addresses(
    fixture: &ClosureFixture,
    receipt: &EffectClosureReceipt,
    edit: impl FnOnce(&mut serde_json::Value),
) -> ArtifactBundleInput {
    let mut files = fixture.files.clone();
    let mut report: serde_json::Value =
        serde_json::from_slice(&files[FUNCTION_ADDRESS_ARTIFACT_NAME]).expect("function-address report");
    edit(&mut report);
    insert_json(&mut files, FUNCTION_ADDRESS_ARTIFACT_NAME, &report);
    let mut forged = receipt.clone();
    forged.inputs.function_address_artifact_blake3 =
        crate::evidence::artifact_blake3_hex(&files[FUNCTION_ADDRESS_ARTIFACT_NAME]);
    integrated_bundle(files, &closure::finalize_receipt(forged))
}

// r[verify octet.effect_closure.evidence.tamper]
#[test]
fn integrated_admitted_closure_must_match_function_address_root_and_envelopes() {
    let fixture = closure_fixture(ALLOCATING_SOURCE, ModuleRole::Core, vec![toolchain_summary("call:Vec::new")]);
    let receipt = closure::compute_closure(fixture.input.clone());
    assert_eq!(receipt.status, AdmissionStatus::Admitted, "{:?}", receipt.diagnostics);

    assert_rejected(
        forge_function_addresses(&fixture, &receipt, |report| {
            report["evidence_chain_handoff"]["canonical_roots"][0]["digest"] = serde_json::json!(IDENTITY_HASH);
        }),
        "record root does not match the receipt",
    );
    assert_rejected(
        forge_function_addresses(&fixture, &receipt, |report| {
            report["source_envelopes"][0]["source_identity"] =
                serde_json::json!(crate::determinism::full_hash("fn root() { std::fs::remove_dir_all(\"/\"); }\n"));
        }),
        "receipt sources do not match function-address source envelopes",
    );
}

// r[verify octet.effect_closure.evidence.tamper]
#[test]
fn integrated_previous_receipt_schema_is_reported_unsupported() {
    let fixture = closure_fixture(ALLOCATING_SOURCE, ModuleRole::Core, vec![toolchain_summary("call:Vec::new")]);
    let receipt = closure::compute_closure(fixture.input.clone());
    assert!(!receipt.nodes[0].calls.is_empty());
    // The previous schema labelled receipts `v1` and wrote call edges without `scope`.
    let mut stored = serde_json::to_value(&receipt).expect("receipt JSON");
    stored["schema_version"] = serde_json::json!("octet-effect-closure-receipt/v1");
    for node in stored["nodes"].as_array_mut().expect("receipt nodes") {
        for call in node["calls"].as_array_mut().expect("node calls") {
            call.as_object_mut().expect("call object").remove("scope");
        }
    }

    let report = verify_bundle(integrated_bundle_storing(fixture.files, &receipt, &stored));

    assert_eq!(report.status, STATUS_INVALID);
    let expected = "unsupported effect-closure receipt schema `octet-effect-closure-receipt/v1`";
    assert!(
        report.diagnostics.iter().any(|diagnostic| diagnostic.contains(expected)),
        "{:?}",
        report.diagnostics
    );
    assert!(
        !report.diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed")),
        "{:?}",
        report.diagnostics
    );
}
