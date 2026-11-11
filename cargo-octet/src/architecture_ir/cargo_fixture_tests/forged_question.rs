use std::fs;
use std::path::Path;
use std::process::Command;

use octet_architecture_ir::CapabilityFamily;
use octet_architecture_ir::CollectionStatus;
use octet_architecture_ir::CompilerArchitectureCoverage;
use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_architecture_ir::CompilerArchitectureReceipt;
use octet_architecture_ir::CompilerArchitectureRole;
use octet_architecture_ir::CompilerArchitectureShard;
use octet_architecture_ir::CoverageStatus;
use octet_architecture_ir::SourceOrigin;
use octet_architecture_ir::UnknownFactKind;
use octet_standards::project_architecture_gate::ArchitectureFindingStatus;
use octet_standards::project_architecture_gate::ArchitecturePhaseStatus;
use octet_standards::project_architecture_gate::ProjectArchitectureGateInput;
use octet_standards::project_architecture_gate::evaluate_project_architecture;
use octet_standards::project_architecture_gate::verify_project_architecture_receipt;
use octet_standards::project_architecture_policy::ArchitectureMode;

use super::FIXTURE_FILES;
use super::TOOLCHAIN;
use super::policy_with_mode;
use super::required_artifact;

#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
// The ignored real-Cargo scenario builds, collects, and gates one forged fixture end to end.
#[allow(clippy::too_many_lines)]
fn real_cargo_forged_question_retains_unknown_call_effect_and_blocks_gate() {
    let library = required_artifact("OCTET_UI_LIBRARY");
    let driver = required_artifact("OCTET_UI_DRIVER");
    let owner = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("owner root");
    let source = owner.join("fixtures/architecture-targets");
    let temporary = tempfile::TempDir::new().expect("fixture workspace");
    let root = temporary.path();
    for relative in FIXTURE_FILES {
        let target = root.join(relative);
        fs::create_dir_all(target.parent().expect("fixture parent")).expect("create fixture parent");
        fs::copy(source.join(relative), target).expect("copy controlled fixture source");
    }
    let source_path = root.join("src/lib.rs");
    let client = include_str!("../../../../fixtures/architecture-targets/forged-question-client.rs");
    let source_text = format!("{}\n{client}", fs::read_to_string(&source_path).expect("library source"));
    fs::write(&source_path, &source_text).expect("append hostile client fixture");
    let macro_path = root.join("macro/src/lib.rs");
    let macro_source = include_str!("../../../../fixtures/architecture-targets/forged-question-proc-macro.rs");
    fs::write(&macro_path, format!("{}\n{macro_source}", fs::read_to_string(&macro_path).expect("macro source")))
        .expect("append hostile macro fixture");

    let (_, host) = super::super::toolchain_identity(TOOLCHAIN).expect("pinned host");
    let policy = policy_with_mode(root, &host, false, ArchitectureMode::Gate);
    let scope = vec!["--workspace".to_owned()];
    let args = vec!["--all-targets".to_owned(), "--all-features".to_owned()];
    let prepared = super::super::prepare_collector(root, &policy, TOOLCHAIN, &library, &scope, &args)
        .expect("plan selected source-bound Cargo units");
    let selected = prepared
        .expected_units
        .iter()
        .find(|expected| {
            expected.unit.package_name == "architecture-target-fixture"
                && expected.unit.target_kind == "lib"
                && expected.unit.source_path == "src/lib.rs"
        })
        .expect("selected production library unit");
    assert_eq!(selected.unit.role, CompilerArchitectureRole::DomainCore);
    let output = Command::new("cargo")
        .current_dir(root)
        .args(["check", "--locked", "--workspace"])
        .args(&args)
        .env("RUSTUP_TOOLCHAIN", TOOLCHAIN)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env("RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("DYLINT_NO_DEPS", "1")
        .env("DYLINT_LIBS", serde_json::to_string(&[library]).expect("selected lint library"))
        .env(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV, &prepared.context_path)
        .output()
        .expect("compile hostile fixture through the real collector");
    assert!(output.status.success(), "compiler failed: {}", String::from_utf8_lossy(&output.stderr));
    let artifacts = super::super::normalize_collected_shards(&prepared).expect("normalize actual shards");
    assert!(artifacts.incomplete_error.is_none(), "{:?}", artifacts.incomplete_error);
    let coverage: CompilerArchitectureCoverage = serde_json::from_str(&artifacts.coverage).expect("coverage");
    assert_eq!(coverage.status, CoverageStatus::Complete);
    assert_eq!(coverage.expected_unit_ids, coverage.observed_unit_ids);
    assert_eq!(coverage.expected_unit_ids.len(), prepared.expected_units.len());
    let ir: CompilerArchitectureIr = serde_json::from_str(&artifacts.architecture_ir).expect("IR");
    assert!(octet_architecture_ir::verify_ir_unit_bindings(&ir).is_empty());
    assert!(octet_architecture_ir::verify_ir_unit_memberships(&ir).is_empty());
    let compiler_receipt: CompilerArchitectureReceipt = serde_json::from_str(&artifacts.receipt).expect("receipt");
    assert_eq!(compiler_receipt.status, CollectionStatus::Complete);
    let shard: CompilerArchitectureShard = fs::read_dir(&prepared.shard_directory)
        .expect("real compiler shards")
        .map(|entry| entry.expect("shard entry").path())
        .map(|path| serde_json::from_slice(&fs::read(path).expect("shard bytes")).expect("valid shard"))
        .find(|shard: &CompilerArchitectureShard| shard.unit.unit_id == selected.unit.unit_id)
        .expect("actual selected production shard");
    assert_eq!(shard.collection_status, CollectionStatus::Complete);
    assert_eq!(shard.unit, selected.unit);

    let invocation = "architecture_target_macro::emit_stolen_question_token_span!(?)";
    let forged_offset = source_text.find(invocation).expect("authored hostile invocation")
        + invocation.find('?').expect("authored question token");
    let direct = "Ok(result?)";
    let direct_start = source_text.find(direct).expect("authored direct question");
    let direct_offset = direct_start + direct.find('?').expect("direct question token");
    assert_ne!(forged_offset, direct_offset);
    let source_span = |id: &str| shard.spans.iter().find(|span| span.span_id == id).expect("fact source span");
    let stolen_span = shard
        .spans
        .iter()
        .find(|span| {
            matches!(&span.origin, SourceOrigin::Workspace { source_path } if source_path == "src/lib.rs")
                && span.byte_start == forged_offset as u64
                && span.byte_end == forged_offset as u64 + 1
                && !span.from_expansion
        })
        .expect("forged punctuation keeps the authored input span");
    let forged_item = shard
        .facts
        .iter()
        .find_map(|fact| match fact {
            CompilerArchitectureFact::Item {
                item_id,
                qualified_path,
                span_id,
                ..
            } if qualified_path.ends_with("::forged_question_filesystem") && span_id == &stolen_span.span_id => {
                Some(item_id)
            }
            _ => None,
        })
        .expect("hostile macro-generated filesystem function");
    let direct_item = shard
        .facts
        .iter()
        .find_map(|fact| match fact {
            CompilerArchitectureFact::Item {
                item_id,
                qualified_path,
                ..
            } if qualified_path.ends_with("::direct_authored_question") => Some(item_id),
            _ => None,
        })
        .expect("direct-authored control function");
    assert_ne!(forged_item, direct_item);
    assert!(
        shard.facts.iter().any(|fact| match fact {
            CompilerArchitectureFact::Call {
                caller_item_id,
                resolved_definition: Some(definition),
                span_id,
                ..
            } =>
                caller_item_id == direct_item
                    && definition == "std::ops::Try::branch"
                    && source_span(span_id).byte_start == (direct_start + "Ok(".len()) as u64
                    && source_span(span_id).byte_end == direct_offset as u64 + 1,
            _ => false,
        }),
        "direct-authored question has a distinct compiler lowering"
    );

    assert!(
        shard.facts.iter().any(|fact| match fact {
            CompilerArchitectureFact::Call {
                caller_item_id,
                resolved_definition: Some(definition),
                span_id,
                ..
            } => caller_item_id == forged_item && definition == "std::fs::read" && span_id == &stolen_span.span_id,
            _ => false,
        }),
        "forged source span must not erase its compiler-resolved filesystem call"
    );
    let effect_id = shard
        .facts
        .iter()
        .find_map(|fact| match fact {
            CompilerArchitectureFact::Effect {
                fact_id,
                caller_item_id,
                capability_family,
                primitive_definition,
                span_id,
            } if caller_item_id == forged_item
                && *capability_family == CapabilityFamily::Filesystem
                && primitive_definition == "std::fs::read"
                && span_id == &stolen_span.span_id =>
            {
                Some(fact_id)
            }
            _ => None,
        })
        .expect("forged function retains its exact filesystem effect");
    let generated_span_id = shard
        .facts
        .iter()
        .find_map(|fact| match fact {
            CompilerArchitectureFact::Call {
                caller_item_id,
                resolved_definition: Some(definition),
                span_id,
                ..
            } if caller_item_id == forged_item && definition == "std::ops::Try::branch" => Some(span_id),
            _ => None,
        })
        .expect("forged question retains its generated Try::branch lowering");
    let generated_span = source_span(generated_span_id);
    assert!(generated_span.from_expansion);
    assert_eq!(generated_span.origin, stolen_span.origin);
    assert_eq!((generated_span.byte_start, generated_span.byte_end), (stolen_span.byte_start, stolen_span.byte_end));
    assert!(shard.facts.iter().any(|fact| matches!(fact,
        CompilerArchitectureFact::Expansion { generated_span_id: generated, callsite_span_id, macro_definition: None, .. }
            if generated == generated_span_id && callsite_span_id == &stolen_span.span_id
    )), "lowered forged question must retain the exact compiler expansion/callsite link");
    let unknown = shard
        .unknowns
        .iter()
        .find(|unknown| {
            unknown.kind == UnknownFactKind::UnsupportedExpansion
                && unknown.span_id.as_ref() == Some(generated_span_id)
                && unknown.detail.contains("Desugaring(QuestionMark)")
        })
        .expect("the actual forged question lowering must be a required unknown");
    assert!(ir.unknowns.iter().any(|item| item.unknown_id == unknown.unknown_id));
    assert!(ir.facts.iter().any(|fact| fact.fact_id() == effect_id));

    let command_id = blake3::hash(b"real forged question compiler gate evaluation").to_hex().to_string();
    let receipt = evaluate_project_architecture(&ProjectArchitectureGateInput {
        policy: &policy.policy,
        policy_source_blake3: &policy.source_blake3,
        policy_blake3: &policy.policy_blake3,
        waiver_policy_blake3: &policy.waiver_policy_blake3,
        lint_phase_status: "warning-only",
        command_blake3: &command_id,
        cargo_graph: &prepared.cargo_graph,
        architecture_ir: &ir,
        coverage: &coverage,
        compiler_receipt: &compiler_receipt,
        effect_closure: None,
    });
    assert!(verify_project_architecture_receipt(&receipt).is_empty());
    assert_eq!(receipt.mode, ArchitectureMode::Gate);
    assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked);
    assert!(receipt.applied_waiver_ids.is_empty());
    assert!(receipt.effect_closure.is_none());
    let required_unknown = receipt
        .findings
        .iter()
        .find(|finding| {
            finding.code == "required-compiler-unknown"
                && finding.status == ArchitectureFindingStatus::Open
                && finding.fact_id.as_ref() == Some(&unknown.unknown_id)
        })
        .expect("the actual forged unknown blocks Gate");
    let blocked_unit_id = required_unknown.unit_id.as_ref().expect("compiler-bound unknown unit");
    let blocked_unit = ir.units.iter().find(|unit| &unit.unit_id == blocked_unit_id).expect("selected blocked unit");
    assert_eq!(blocked_unit.package_name, shard.unit.package_name);
    assert_eq!(blocked_unit.role, CompilerArchitectureRole::DomainCore);
    assert!(matches!(blocked_unit.target_kind.as_str(), "lib" | "lib+test"));
    let membership = ir
        .unit_memberships
        .iter()
        .find(|row| &row.unit_id == blocked_unit_id)
        .expect("compiler-bound production membership");
    assert!(membership.unknown_ids.contains(&unknown.unknown_id));
    assert!(membership.fact_ids.iter().any(|id| id == effect_id));
    assert!(
        receipt.findings.iter().any(|finding| {
            finding.code == "domain-core-effect"
                && finding.status == ArchitectureFindingStatus::Open
                && finding.unit_id.as_ref() == Some(blocked_unit_id)
                && finding.fact_id.as_ref() == Some(effect_id)
        }),
        "the forged filesystem effect must block the same unit as its unknown: {:?}",
        receipt.findings
    );
    eprintln!(
        "forged source token: unit={} unknown={} effect={} gate={:?}",
        blocked_unit_id, unknown.unknown_id, effect_id, receipt.status
    );
}
