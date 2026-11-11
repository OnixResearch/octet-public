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

pub(crate) fn unit() -> CompilationUnit {
    let mut unit = CompilationUnit {
        unit_id: String::new(),
        package_name: "fixture".to_owned(),
        package_version: "0.1.0".to_owned(),
        target_name: "fixture".to_owned(),
        target_kind: "lib".to_owned(),
        source_path: "src/lib.rs".to_owned(),
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

// r[verify octet.architecture.unit_membership]
#[test]
fn projected_observations_retain_identity_and_reference_checks() {
    let source = span();
    let fact = item_fact(&source.span_id);
    let unresolved = unknown(&source.span_id);
    let check = |facts: &[CompilerArchitectureFact], unknowns: &[UnknownCompilerFact], spans: &[SourceSpan]| {
        verify_observation_set(facts, unknowns, spans, &limits())
    };
    assert!(
        check(std::slice::from_ref(&fact), std::slice::from_ref(&unresolved), std::slice::from_ref(&source)).is_empty()
    );
    assert!(check(std::slice::from_ref(&fact), &[], &[]).iter().any(|error| error.contains("missing span")));
    let mut stale_source = source.clone();
    stale_source.byte_end += 1;
    assert!(
        check(std::slice::from_ref(&fact), &[], &[stale_source])
            .iter()
            .any(|error| error.contains("span-identity"))
    );
    let mut stale_unknown = unresolved;
    stale_unknown.detail.push_str(" changed");
    assert!(
        check(std::slice::from_ref(&fact), &[stale_unknown], std::slice::from_ref(&source))
            .iter()
            .any(|error| error.contains("unknown-identity"))
    );
    let mut stale = fact;
    if let CompilerArchitectureFact::Item { visibility, .. } = &mut stale {
        *visibility = "private".to_owned();
    }
    assert!(check(&[stale], &[], &[source]).iter().any(|error| error.contains("fact-identity")));
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

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn unit_roots_are_required_and_bound_in_the_unit_identity() {
    let original = unit();
    let mut value = serde_json::to_value(&original).expect("serialize selected unit");
    assert_eq!(
        value["source_path"].as_str().expect("selected source root must survive normalization"),
        "src/lib.rs"
    );
    value["source_path"] = "tests/independent.rs".into();
    let changed: CompilationUnit = serde_json::from_value(value.clone()).expect("changed root");
    assert_ne!(compilation_unit_identity(&changed), original.unit_id);
    value.as_object_mut().expect("unit object").remove("source_path");
    assert!(serde_json::from_value::<CompilationUnit>(value).is_err(), "missing roots must not default");
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn malformed_roots_and_legacy_shards_reject_after_resealing() {
    for path in ["", "/src/lib.rs", "../src/lib.rs", "src//lib.rs", "src/./lib.rs"] {
        let mut invalid = shard();
        invalid.unit.source_path = path.to_owned();
        invalid.unit.unit_id = compilation_unit_identity(&invalid.unit);
        invalid.shard_id = shard_identity(&invalid);
        let mut selected = input(vec![invalid.clone()]);
        selected.expected_units[0].unit = invalid.unit;
        let outcome = normalize_architecture_shards(selected).expect("invalid-root outcome");
        assert!(outcome.architecture_ir.is_none(), "{path}");
        assert!(outcome.issues.iter().any(|issue| issue.code == "unit-source-root"), "{path}");
    }
    let mut legacy = shard();
    legacy.schema_version = "octet-compiler-architecture-shard/v1".to_owned();
    legacy.shard_id = shard_identity(&legacy);
    let rejected = normalize_architecture_shards(input(vec![legacy])).expect("legacy outcome");
    assert!(rejected.architecture_ir.is_none());
    assert!(rejected.issues.iter().any(|issue| issue.code == "shard-schema"));

    let mut ir = normalize_architecture_shards(input(vec![shard()])).expect("normalize").architecture_ir.expect("IR");
    assert!(verify_ir_unit_bindings(&ir).is_empty());
    ir.units[0].source_path = "tests/unobserved.rs".to_owned();
    ir.architecture_ir_id = ir_identity(&ir);
    assert!(verify_ir_unit_bindings(&ir).iter().any(|message| message.contains("identity is stale")));
    // A fully resealed, structurally possible root remains consistency, not authenticated Cargo
    // evidence.
    ir.units[0].unit_id = compilation_unit_identity(&ir.units[0]);
    ir.unit_memberships[0].unit_id = ir.units[0].unit_id.clone();
    ir.architecture_ir_id = ir_identity(&ir);
    assert!(verify_ir_unit_bindings(&ir).is_empty());
    assert!(verify_ir_unit_memberships(&ir).is_empty());
}

fn shared_observation_input() -> NormalizationInput {
    let first = shard();
    let mut second = first.clone();
    second.unit.target_kind = "lib+test".to_owned();
    second.unit.unit_id = compilation_unit_identity(&second.unit);
    second.shard_id = shard_identity(&second);
    let mut selected = input(vec![first, second.clone()]);
    selected.expected_units.push(ExpectedCompilationUnit { unit: second.unit });
    selected
}

// r[verify octet.architecture.unit_membership]
#[test]
fn shared_observations_preserve_each_unit_membership() {
    let selected = shared_observation_input();
    let outcome = normalize_architecture_shards(selected.clone()).expect("normalize shared observations");
    assert_eq!(outcome.coverage.status, CoverageStatus::Complete);
    let ir = outcome.architecture_ir.expect("complete shared IR");
    let value = serde_json::to_value(&ir).expect("serialize normalized IR");
    let rows = value["unit_memberships"].as_array().expect("normalized IR must retain unit membership");
    assert_eq!(rows.len(), selected.expected_units.len());
    for observed in &selected.shards {
        let row = rows.iter().find(|row| row["unit_id"] == observed.unit.unit_id).expect("observed unit");
        assert_eq!(row["fact_ids"], serde_json::json!([observed.facts[0].fact_id()]));
        assert_eq!(row["unknown_ids"], serde_json::json!([observed.unknowns[0].unknown_id]));
    }
    assert_eq!(ir.facts.len(), 1, "shared fact remains canonical, with both memberships");
    assert_eq!(ir.unknowns.len(), 1, "shared unknown remains canonical, with both memberships");
    let mut reversed = selected;
    reversed.shards.reverse();
    reversed.expected_units.reverse();
    let replay = normalize_architecture_shards(reversed).expect("normalize reordered observations");
    assert_eq!(Some(ir), replay.architecture_ir);
}

// r[verify octet.architecture.unit_membership]
#[test]
fn malformed_memberships_are_rejected_after_resealing() {
    let ir = normalize_architecture_shards(shared_observation_input())
        .expect("normalize")
        .architecture_ir
        .expect("IR");
    assert!(verify_ir_unit_memberships(&ir).is_empty());
    for case in [
        "missing rows",
        "missing unit",
        "duplicate unit",
        "unsorted units",
        "orphan unit",
        "unreferenced fact",
        "unreferenced unknown",
        "orphan fact",
        "orphan unknown",
        "duplicate fact",
        "duplicate unknown",
    ] {
        let mut altered = ir.clone();
        match case {
            "missing rows" => altered.unit_memberships.clear(),
            "missing unit" => {
                altered.unit_memberships.pop();
            }
            "duplicate unit" => altered.unit_memberships.push(altered.unit_memberships[0].clone()),
            "unsorted units" => altered.unit_memberships.reverse(),
            "orphan unit" => altered.unit_memberships[0].unit_id = HASH_A.to_owned(),
            "unreferenced fact" => altered.unit_memberships.iter_mut().for_each(|row| row.fact_ids.clear()),
            "unreferenced unknown" => altered.unit_memberships.iter_mut().for_each(|row| row.unknown_ids.clear()),
            "orphan fact" => altered.unit_memberships[0].fact_ids.push(HASH_C.to_owned()),
            "orphan unknown" => altered.unit_memberships[0].unknown_ids.push(HASH_C.to_owned()),
            "duplicate fact" => {
                let id = altered.unit_memberships[0].fact_ids[0].clone();
                altered.unit_memberships[0].fact_ids.push(id);
            }
            "duplicate unknown" => {
                let id = altered.unit_memberships[0].unknown_ids[0].clone();
                altered.unit_memberships[0].unknown_ids.push(id);
            }
            _ => unreachable!("declared fixture case"),
        }
        altered.architecture_ir_id = ir_identity(&altered);
        assert!(!verify_ir_unit_memberships(&altered).is_empty(), "accepted {case}");
    }
}

// r[verify octet.architecture.unit_membership]
#[test]
fn legacy_membership_is_not_invented_and_shared_edge_changes_identity() {
    let ir = normalize_architecture_shards(shared_observation_input())
        .expect("normalize")
        .architecture_ir
        .expect("IR");
    let mut legacy = serde_json::to_value(&ir).expect("serialize");
    legacy["schema_version"] = "octet-compiler-architecture-ir/v1".into();
    legacy.as_object_mut().expect("IR object").remove("unit_memberships");
    assert!(serde_json::from_value::<CompilerArchitectureIr>(legacy).is_err());
    let mut altered = ir.clone();
    altered.schema_version = "octet-compiler-architecture-ir/v1".to_owned();
    altered.architecture_ir_id = ir_identity(&altered);
    assert!(!verify_ir_unit_memberships(&altered).is_empty());

    let mut changed_relation = ir.clone();
    changed_relation.unit_memberships[0].fact_ids.clear();
    assert_ne!(ir.architecture_ir_id, ir_identity(&changed_relation));
    // The remaining relation is structurally possible. The producer must retain
    // its observations; a digest does not authenticate a compiler invocation.
    assert!(verify_ir_unit_memberships(&changed_relation).is_empty());
}

// r[verify octet.architecture.unit_membership]
#[test]
fn member_order_is_canonical_and_shard_order_is_not_semantic() {
    let mut selected = shared_observation_input();
    let mut extra = CompilerArchitectureFact::Configuration {
        fact_id: String::new(),
        predicate: "fixture:additional".to_owned(),
        active: true,
        span_id: None,
    };
    let id = fact_identity(&extra);
    let CompilerArchitectureFact::Configuration { fact_id, .. } = &mut extra else {
        unreachable!()
    };
    *fact_id = id;
    for observed in &mut selected.shards {
        observed.facts.push(extra.clone());
        let mut additional = observed.unknowns[0].clone();
        additional.detail = "another bounded unknown observation".to_owned();
        additional.unknown_id = unknown_fact_identity(&additional);
        observed.unknowns.push(additional);
        observed.shard_id = shard_identity(observed);
    }
    let ir = normalize_architecture_shards(selected.clone())
        .expect("normalize two members")
        .architecture_ir
        .expect("IR");
    assert!(verify_ir_unit_memberships(&ir).is_empty());
    for case in [
        "facts",
        "unknowns",
        "fact subjects",
        "unknown subjects",
        "unit subjects",
        "empty subjects",
    ] {
        let mut altered = ir.clone();
        match case {
            "facts" => altered.unit_memberships[0].fact_ids.reverse(),
            "unknowns" => altered.unit_memberships[0].unknown_ids.reverse(),
            "fact subjects" => altered.facts.reverse(),
            "unknown subjects" => altered.unknowns.reverse(),
            "unit subjects" => altered.units.reverse(),
            "empty subjects" => {
                altered.units.clear();
                altered.unit_memberships.clear();
            }
            _ => unreachable!(),
        }
        altered.architecture_ir_id = ir_identity(&altered);
        let diagnostics = verify_ir_unit_memberships(&altered);
        assert!(diagnostics.iter().any(|message| message.contains("canonical")), "accepted {case}: {diagnostics:?}");
    }
    for observed in &mut selected.shards {
        observed.facts.reverse();
        observed.unknowns.reverse();
        observed.shard_id = shard_identity(observed);
    }
    assert_eq!(
        Some(ir),
        normalize_architecture_shards(selected).expect("permuted raw observations").architecture_ir
    );
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
fn signature_by_reference_flags_must_match_inputs_one_to_one() {
    let with_flags = |input_by_reference: Vec<bool>| {
        let mut current = shard();
        let mut witness = CompilerArchitectureFact::Type {
            fact_id: String::new(),
            type_id: HASH_C.to_owned(),
            display: "fixture::Witness".to_owned(),
            definition_path: Some("fixture::Witness".to_owned()),
            origin_crate: "fixture".to_owned(),
            type_kind: TypeKind::Adt,
        };
        let mut signature = CompilerArchitectureFact::Signature {
            fact_id: String::new(),
            signature_id: HASH_D.to_owned(),
            owner_item_id: HASH_A.to_owned(),
            input_type_ids: vec![HASH_C.to_owned(), HASH_C.to_owned()],
            input_by_reference,
            output_type_id: HASH_C.to_owned(),
            is_public: true,
        };
        for fact in [&mut witness, &mut signature] {
            let id = fact_identity(fact);
            match fact {
                CompilerArchitectureFact::Type { fact_id, .. }
                | CompilerArchitectureFact::Signature { fact_id, .. } => {
                    *fact_id = id;
                }
                _ => unreachable!(),
            }
        }
        current.facts.extend([witness, signature]);
        current.shard_id = shard_identity(&current);
        normalize_architecture_shards(input(vec![current])).expect("normalize signature fixture")
    };

    let paired = with_flags(vec![true, false]);
    assert_eq!(paired.coverage.status, CoverageStatus::Complete, "{:?}", paired.issues);
    for mismatched in [Vec::new(), vec![true], vec![true, false, false]] {
        let outcome = with_flags(mismatched);
        assert_eq!(outcome.coverage.status, CoverageStatus::Incomplete);
        assert!(outcome.issues.iter().any(|issue| issue.code == "invalid-signature-inputs"));
    }
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
fn unit_stamp_follows_the_stable_locator_not_volatile_hashes() {
    let context = Path::new("/workspace/.octet/compiler-architecture/collector-context.json");
    let first = unit();
    let mut volatile = first.clone();
    volatile.policy_blake3 = HASH_C.to_owned();
    volatile.source_blake3 = HASH_C.to_owned();
    volatile.invocation_blake3 = HASH_C.to_owned();
    volatile.unit_id = compilation_unit_identity(&volatile);
    let mut relocated = first.clone();
    relocated.target_kind = format!("{}+test", relocated.target_kind);

    assert_ne!(volatile.unit_id, first.unit_id);
    assert_eq!(compilation_unit_stamp_name(&volatile), compilation_unit_stamp_name(&first));
    assert_ne!(compilation_unit_stamp_name(&relocated), compilation_unit_stamp_name(&first));
    assert_eq!(
        compilation_unit_stamp_path(context, &first),
        Path::new("/workspace/.octet/compiler-architecture/unit-stamps").join(compilation_unit_stamp_name(&first))
    );
    assert_eq!(
        compilation_unit_stamp_directory(context),
        compilation_unit_stamp_path(context, &first).parent().unwrap()
    );
}

#[test]
fn generated_nickel_contract_is_fresh() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().and_then(Path::parent).expect("repo root");
    let path = repo_root.join(COMPILER_ARCHITECTURE_CONTRACT_PATH);
    let checked_in = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read generated contract {}: {error}", path.display()));

    assert_eq!(checked_in, render_nickel_contract());
}
