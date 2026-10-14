use std::path::Path;

use super::*;

const HASH_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const HASH_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const HASH_C: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";
const HASH_D: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";
const MAX_FIXTURE_COUNT: usize = 64;
const MAX_FIXTURE_BYTES: usize = 65_536;

fn limits() -> ArchitectureCollectionLimits {
    ArchitectureCollectionLimits {
        max_units: MAX_FIXTURE_COUNT,
        max_shards: MAX_FIXTURE_COUNT,
        max_modules: MAX_FIXTURE_COUNT,
        max_items: MAX_FIXTURE_COUNT,
        max_types: MAX_FIXTURE_COUNT,
        max_traits: MAX_FIXTURE_COUNT,
        max_implementations: MAX_FIXTURE_COUNT,
        max_signatures: MAX_FIXTURE_COUNT,
        max_calls: MAX_FIXTURE_COUNT,
        max_constructors: MAX_FIXTURE_COUNT,
        max_effects: MAX_FIXTURE_COUNT,
        max_expansions: MAX_FIXTURE_COUNT,
        max_configurations: MAX_FIXTURE_COUNT,
        max_type_shapes: MAX_FIXTURE_COUNT,
        max_function_safety_facts: MAX_FIXTURE_COUNT,
        max_value_provenance_facts: MAX_FIXTURE_COUNT,
        max_value_flow_bodies: MAX_FIXTURE_COUNT,
        max_value_flow_steps: MAX_FIXTURE_COUNT,
        max_value_flow_rounds: MAX_FIXTURE_COUNT,
        max_unknowns: MAX_FIXTURE_COUNT,
        max_spans: MAX_FIXTURE_COUNT,
        max_span_bytes: MAX_FIXTURE_BYTES,
        max_shard_bytes: MAX_FIXTURE_BYTES,
        max_artifact_bytes: MAX_FIXTURE_BYTES,
    }
}

fn unit() -> CompilationUnit {
    let mut unit = CompilationUnit {
        unit_id: String::new(),
        package_name: "fixture".to_owned(),
        package_version: "0.1.0".to_owned(),
        target_name: "fixture".to_owned(),
        target_kind: "lib".to_owned(),
        crate_types: vec!["lib".to_owned()],
        target_triple: "x86_64-unknown-linux-gnu".to_owned(),
        features: vec!["architecture".to_owned()],
        default_features: false,
        role: CompilerArchitectureRole::ApplicationShell,
        toolchain_blake3: HASH_A.to_owned(),
        collector_blake3: HASH_D.to_owned(),
        policy_blake3: HASH_B.to_owned(),
        source_blake3: HASH_C.to_owned(),
        invocation_blake3: HASH_D.to_owned(),
    };
    unit.unit_id = compilation_unit_identity(&unit);
    unit
}

fn span() -> SourceSpan {
    let mut span = SourceSpan {
        span_id: String::new(),
        origin: SourceOrigin::Workspace {
            source_path: "src/lib.rs".to_owned(),
        },
        byte_start: 0,
        byte_end: 20,
        line_start: 1,
        column_start: 1,
        from_expansion: false,
    };
    span.span_id = source_span_identity(&span);
    span
}

fn item_fact(span_id: &str) -> CompilerArchitectureFact {
    let mut fact = CompilerArchitectureFact::Item {
        fact_id: String::new(),
        item_id: HASH_A.to_owned(),
        qualified_path: "fixture::run".to_owned(),
        item_kind: ItemKind::Function,
        origin_crate: "fixture".to_owned(),
        visibility: "public".to_owned(),
        signature_id: None,
        span_id: span_id.to_owned(),
    };
    let id = fact_identity(&fact);
    match &mut fact {
        CompilerArchitectureFact::Item { fact_id, .. } => *fact_id = id,
        _ => unreachable!(),
    }
    fact
}

fn unknown(span_id: &str) -> UnknownCompilerFact {
    let mut unknown = UnknownCompilerFact {
        unknown_id: String::new(),
        kind: UnknownFactKind::DynamicDispatch,
        owner_item_id: Some(HASH_A.to_owned()),
        target_identity: Some("dyn fixture::Port".to_owned()),
        span_id: Some(span_id.to_owned()),
        detail: "trait-object call target is not statically unique".to_owned(),
    };
    unknown.unknown_id = unknown_fact_identity(&unknown);
    unknown
}

fn shard() -> CompilerArchitectureShard {
    let span = span();
    let mut shard = CompilerArchitectureShard {
        schema_version: COMPILER_ARCHITECTURE_SHARD_SCHEMA_VERSION.to_owned(),
        shard_id: String::new(),
        unit: unit(),
        collection_status: CollectionStatus::Complete,
        spans: vec![span.clone()],
        facts: vec![item_fact(&span.span_id)],
        unknowns: vec![unknown(&span.span_id)],
        non_claims: vec!["runtime-behavior".to_owned()],
    };
    shard.shard_id = shard_identity(&shard);
    shard
}

fn input(shards: Vec<CompilerArchitectureShard>) -> NormalizationInput {
    NormalizationInput {
        policy_blake3: HASH_B.to_owned(),
        limits: limits(),
        expected_units: vec![ExpectedCompilationUnit { unit: unit() }],
        shards,
        invalid_shards: Vec::new(),
        non_claims: vec!["runtime-behavior".to_owned(), "whole-program-correctness".to_owned()],
    }
}

#[test]
fn complete_shard_normalizes_and_keeps_unknown_facts() {
    let outcome = normalize_architecture_shards(input(vec![shard()])).expect("normalize complete fixture");

    assert_eq!(outcome.coverage.status, CoverageStatus::Complete);
    assert_eq!(outcome.receipt.status, CollectionStatus::Complete);
    assert!(outcome.issues.is_empty());
    let ir = outcome.architecture_ir.expect("complete architecture IR");
    assert_eq!(ir.unknowns.len(), 1);
    assert_eq!(ir.unknowns[0].kind, UnknownFactKind::DynamicDispatch);
}

#[test]
fn missing_unit_fails_closed_without_architecture_ir() {
    let outcome = normalize_architecture_shards(input(Vec::new())).expect("normalize missing-unit fixture");

    assert_eq!(outcome.coverage.status, CoverageStatus::Incomplete);
    assert_eq!(outcome.receipt.status, CollectionStatus::Incomplete);
    assert!(outcome.architecture_ir.is_none());
    assert!(outcome.issues.iter().any(|issue| issue.code == "coverage-missing-unit"));
}

#[test]
fn external_generated_span_uses_provenance_without_workspace_path() {
    let mut current = shard();
    current.spans[0].origin = SourceOrigin::ExternalGenerated {
        provenance_blake3: HASH_C.to_owned(),
    };
    current.spans[0].span_id = source_span_identity(&current.spans[0]);
    let CompilerArchitectureFact::Item { span_id, .. } = &mut current.facts[0] else {
        unreachable!();
    };
    *span_id = current.spans[0].span_id.clone();
    let updated_fact_id = fact_identity(&current.facts[0]);
    let CompilerArchitectureFact::Item { fact_id, .. } = &mut current.facts[0] else {
        unreachable!();
    };
    *fact_id = updated_fact_id;
    current.unknowns[0].span_id = Some(current.spans[0].span_id.clone());
    current.unknowns[0].unknown_id = unknown_fact_identity(&current.unknowns[0]);
    current.shard_id = shard_identity(&current);

    let outcome = normalize_architecture_shards(input(vec![current])).expect("normalize external generated span");

    assert_eq!(outcome.coverage.status, CoverageStatus::Complete);
    assert!(outcome.issues.is_empty());
}

#[test]
fn invalid_external_generated_provenance_is_rejected() {
    let mut current = shard();
    current.spans[0].origin = SourceOrigin::ExternalGenerated {
        provenance_blake3: "invalid".to_owned(),
    };
    current.spans[0].span_id = source_span_identity(&current.spans[0]);
    let CompilerArchitectureFact::Item { span_id, .. } = &mut current.facts[0] else {
        unreachable!();
    };
    *span_id = current.spans[0].span_id.clone();
    let updated_fact_id = fact_identity(&current.facts[0]);
    let CompilerArchitectureFact::Item { fact_id, .. } = &mut current.facts[0] else {
        unreachable!();
    };
    *fact_id = updated_fact_id;
    current.unknowns[0].span_id = Some(current.spans[0].span_id.clone());
    current.unknowns[0].unknown_id = unknown_fact_identity(&current.unknowns[0]);
    current.shard_id = shard_identity(&current);

    let outcome = normalize_architecture_shards(input(vec![current])).expect("normalize invalid external provenance");

    assert_eq!(outcome.coverage.status, CoverageStatus::Incomplete);
    assert!(outcome.issues.iter().any(|issue| issue.code == "span-external-provenance"));
}

#[test]
fn zero_based_source_coordinate_is_rejected() {
    let mut current = shard();
    current.spans[0].column_start = 0;
    current.spans[0].span_id = source_span_identity(&current.spans[0]);
    let CompilerArchitectureFact::Item { span_id, .. } = &mut current.facts[0] else {
        unreachable!();
    };
    *span_id = current.spans[0].span_id.clone();
    let updated_fact_id = fact_identity(&current.facts[0]);
    let CompilerArchitectureFact::Item { fact_id, .. } = &mut current.facts[0] else {
        unreachable!();
    };
    *fact_id = updated_fact_id;
    current.unknowns[0].span_id = Some(current.spans[0].span_id.clone());
    current.unknowns[0].unknown_id = unknown_fact_identity(&current.unknowns[0]);
    current.shard_id = shard_identity(&current);

    let outcome = normalize_architecture_shards(input(vec![current])).expect("normalize zero-based coordinate");

    assert_eq!(outcome.coverage.status, CoverageStatus::Incomplete);
    assert!(outcome.issues.iter().any(|issue| issue.code == "span-coordinate"));
}

#[test]
fn malformed_shard_record_is_reviewable_and_incomplete() {
    let mut current = input(Vec::new());
    current.invalid_shards.push(InvalidCompilerArchitectureShard {
        shard_id: HASH_A.to_owned(),
        code: "shard-malformed".to_owned(),
        message: "compiler architecture shard `malformed.json` is malformed".to_owned(),
    });

    let outcome = normalize_architecture_shards(current).expect("normalize malformed shard record");

    assert_eq!(outcome.coverage.status, CoverageStatus::Incomplete);
    assert_eq!(outcome.coverage.invalid_shard_ids, [HASH_A]);
    assert!(outcome.issues.iter().any(|issue| issue.code == "shard-malformed"));
}

#[test]
fn duplicate_valid_shards_fail_closed() {
    let current = shard();
    let outcome =
        normalize_architecture_shards(input(vec![current.clone(), current])).expect("normalize duplicate shards");

    assert_eq!(outcome.coverage.status, CoverageStatus::Incomplete);
    assert_eq!(outcome.coverage.duplicate_unit_ids, [unit().unit_id]);
    assert!(outcome.issues.iter().any(|issue| issue.code == "coverage-duplicate-unit"));
}

#[test]
fn unexpected_valid_unit_fails_closed() {
    let mut current = shard();
    current.unit.package_name = "unexpected".to_owned();
    current.unit.unit_id = compilation_unit_identity(&current.unit);
    current.shard_id = shard_identity(&current);
    let unexpected_id = current.unit.unit_id.clone();

    let outcome = normalize_architecture_shards(input(vec![current])).expect("normalize unexpected unit");

    assert_eq!(outcome.coverage.status, CoverageStatus::Incomplete);
    assert_eq!(outcome.coverage.unexpected_unit_ids, [unexpected_id]);
    assert!(outcome.issues.iter().any(|issue| issue.code == "coverage-unexpected-unit"));
}

#[test]
fn stale_collector_unit_is_invalid_and_missing() {
    let mut current = shard();
    current.unit.collector_blake3 = HASH_C.to_owned();
    current.shard_id = shard_identity(&current);

    let outcome = normalize_architecture_shards(input(vec![current])).expect("normalize stale collector unit");

    assert_eq!(outcome.coverage.status, CoverageStatus::Incomplete);
    assert_eq!(outcome.coverage.missing_unit_ids, [unit().unit_id]);
    assert!(!outcome.coverage.invalid_shard_ids.is_empty());
    assert!(outcome.issues.iter().any(|issue| issue.code == "unit-identity"));
}

#[test]
fn stale_fact_identity_and_invalid_reference_are_rejected() {
    let mut current = shard();
    let CompilerArchitectureFact::Item { fact_id, span_id, .. } = &mut current.facts[0] else {
        unreachable!();
    };
    *fact_id = HASH_D.to_owned();
    *span_id = HASH_C.to_owned();
    current.shard_id = shard_identity(&current);

    let outcome = normalize_architecture_shards(input(vec![current])).expect("normalize malformed fixture");

    assert_eq!(outcome.coverage.status, CoverageStatus::Incomplete);
    assert!(outcome.issues.iter().any(|issue| issue.code == "fact-identity"));
    assert!(outcome.issues.iter().any(|issue| issue.code == "invalid-reference"));
}

#[test]
fn count_overflow_is_incomplete() {
    let mut input = input(vec![shard()]);
    input.limits.max_unknowns = 1;
    let mut second = unknown(&input.shards[0].spans[0].span_id);
    second.detail.push_str(" for a second call");
    second.unknown_id = unknown_fact_identity(&second);
    input.shards[0].unknowns.push(second);
    input.shards[0].shard_id = shard_identity(&input.shards[0]);

    let outcome = normalize_architecture_shards(input).expect("normalize overflow fixture");

    assert_eq!(outcome.receipt.status, CollectionStatus::Incomplete);
    assert!(outcome.issues.iter().any(|issue| issue.code == "limit-overflow"));
}

#[test]
fn unchanged_inputs_repeat_byte_identities() {
    let first = normalize_architecture_shards(input(vec![shard()])).expect("first normalization");
    let second = normalize_architecture_shards(input(vec![shard()])).expect("second normalization");

    assert_eq!(canonical_json(&first.coverage).unwrap(), canonical_json(&second.coverage).unwrap());
    assert_eq!(canonical_json(&first.receipt).unwrap(), canonical_json(&second.receipt).unwrap());
    assert_eq!(canonical_json(&first.architecture_ir).unwrap(), canonical_json(&second.architecture_ir).unwrap());
}

#[test]
fn local_compiler_indexes_do_not_enter_unit_identity() {
    let first = unit();
    let second = first.clone();

    assert_eq!(compilation_unit_identity(&first), compilation_unit_identity(&second));
}

#[test]
fn changed_collector_changes_unit_identity() {
    let first = unit();
    let mut second = first.clone();
    second.collector_blake3 = HASH_C.to_owned();

    assert_ne!(compilation_unit_identity(&first), compilation_unit_identity(&second));
}

#[test]
fn generated_nickel_contract_is_fresh() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("repo root");
    let path = repo_root.join(COMPILER_ARCHITECTURE_CONTRACT_PATH);
    let checked_in = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read generated contract {}: {error}", path.display()));

    assert_eq!(checked_in, render_nickel_contract());
}
