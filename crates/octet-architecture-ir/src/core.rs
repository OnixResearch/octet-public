use std::collections::BTreeMap;
use std::collections::BTreeSet;

use crate::identity::canonical_identity;
use crate::identity::canonical_json;
use crate::model::ArchitectureCollectionLimits;
use crate::model::COMPILER_ARCHITECTURE_COVERAGE_SCHEMA_VERSION;
use crate::model::COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION;
use crate::model::COMPILER_ARCHITECTURE_ISSUE_SCHEMA_VERSION;
use crate::model::COMPILER_ARCHITECTURE_RECEIPT_SCHEMA_VERSION;
use crate::model::COMPILER_ARCHITECTURE_SHARD_SCHEMA_VERSION;
use crate::model::CollectionStatus;
use crate::model::CompilationUnit;
use crate::model::CompilerArchitectureCoverage;
use crate::model::CompilerArchitectureFact;
use crate::model::CompilerArchitectureIr;
use crate::model::CompilerArchitectureIssue;
use crate::model::CompilerArchitectureReceipt;
use crate::model::CompilerArchitectureShard;
use crate::model::CoverageStatus;
use crate::model::ExpectedCompilationUnit;
use crate::model::NormalizationInput;
use crate::model::NormalizationOutcome;
use crate::model::SourceSpan;
use crate::model::UnknownCompilerFact;

const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;
const HEX_CHARS_PER_BYTE: usize = 2;
const UNIT_ID_DOMAIN: &str = "octet.compiler-architecture-ir.unit/v1";
const SPAN_ID_DOMAIN: &str = "octet.compiler-architecture-ir.span/v1";
const FACT_ID_DOMAIN: &str = "octet.compiler-architecture-ir.fact/v1";
const UNKNOWN_ID_DOMAIN: &str = "octet.compiler-architecture-ir.unknown/v1";
const SHARD_ID_DOMAIN: &str = "octet.compiler-architecture-ir.shard/v1";
const IR_ID_DOMAIN: &str = "octet.compiler-architecture-ir.normalized/v1";
const COVERAGE_ID_DOMAIN: &str = "octet.compiler-architecture-ir.coverage/v1";
const ISSUE_ID_DOMAIN: &str = "octet.compiler-architecture-ir.issue/v1";
const RECEIPT_ID_DOMAIN: &str = "octet.compiler-architecture-ir.receipt/v1";

#[must_use]
pub fn compilation_unit_identity(unit: &CompilationUnit) -> String {
    canonical_identity(
        UNIT_ID_DOMAIN,
        &(
            &unit.package_name,
            &unit.package_version,
            &unit.target_name,
            &unit.target_kind,
            &unit.crate_types,
            &unit.target_triple,
            &unit.features,
            unit.default_features,
            unit.role,
            &unit.toolchain_blake3,
            &unit.collector_blake3,
            &unit.policy_blake3,
            &unit.source_blake3,
            &unit.invocation_blake3,
        ),
    )
    .expect("compilation-unit identity fields must serialize")
}

#[must_use]
pub fn source_span_identity(span: &SourceSpan) -> String {
    canonical_identity(
        SPAN_ID_DOMAIN,
        &(
            &span.origin,
            span.byte_start,
            span.byte_end,
            span.line_start,
            span.column_start,
            span.from_expansion,
        ),
    )
    .expect("source-span identity fields must serialize")
}

#[must_use]
pub fn fact_identity(fact: &CompilerArchitectureFact) -> String {
    match fact {
        CompilerArchitectureFact::Module {
            module_path, span_id, ..
        } => canonical_identity(FACT_ID_DOMAIN, &("module", module_path, span_id)),
        CompilerArchitectureFact::Item {
            item_id,
            qualified_path,
            item_kind,
            origin_crate,
            visibility,
            signature_id,
            span_id,
            ..
        } => canonical_identity(
            FACT_ID_DOMAIN,
            &("item", item_id, qualified_path, item_kind, origin_crate, visibility, signature_id, span_id),
        ),
        CompilerArchitectureFact::Type {
            type_id,
            display,
            definition_path,
            origin_crate,
            type_kind,
            ..
        } => canonical_identity(FACT_ID_DOMAIN, &("type", type_id, display, definition_path, origin_crate, type_kind)),
        CompilerArchitectureFact::Trait {
            trait_id,
            qualified_path,
            origin_crate,
            span_id,
            ..
        } => canonical_identity(FACT_ID_DOMAIN, &("trait", trait_id, qualified_path, origin_crate, span_id)),
        CompilerArchitectureFact::Implementation {
            implementation_id,
            trait_id,
            trait_definition,
            self_type_id,
            span_id,
            ..
        } => canonical_identity(
            FACT_ID_DOMAIN,
            &("implementation", implementation_id, trait_id, trait_definition, self_type_id, span_id),
        ),
        CompilerArchitectureFact::Signature {
            signature_id,
            owner_item_id,
            input_type_ids,
            output_type_id,
            is_public,
            ..
        } => canonical_identity(
            FACT_ID_DOMAIN,
            &("signature", signature_id, owner_item_id, input_type_ids, output_type_id, is_public),
        ),
        CompilerArchitectureFact::Call {
            caller_item_id,
            callee_item_id,
            resolved_definition,
            dispatch,
            span_id,
            ..
        } => canonical_identity(
            FACT_ID_DOMAIN,
            &("call", caller_item_id, callee_item_id, resolved_definition, dispatch, span_id),
        ),
        CompilerArchitectureFact::Constructor {
            caller_item_id,
            constructed_type_id,
            constructor_definition,
            span_id,
            ..
        } => canonical_identity(
            FACT_ID_DOMAIN,
            &("constructor", caller_item_id, constructed_type_id, constructor_definition, span_id),
        ),
        CompilerArchitectureFact::Effect {
            caller_item_id,
            capability_family,
            primitive_definition,
            span_id,
            ..
        } => canonical_identity(
            FACT_ID_DOMAIN,
            &("effect", caller_item_id, capability_family, primitive_definition, span_id),
        ),
        CompilerArchitectureFact::Expansion {
            generated_span_id,
            callsite_span_id,
            macro_definition,
            ..
        } => canonical_identity(FACT_ID_DOMAIN, &("expansion", generated_span_id, callsite_span_id, macro_definition)),
        CompilerArchitectureFact::Configuration {
            predicate,
            active,
            span_id,
            ..
        } => canonical_identity(FACT_ID_DOMAIN, &("configuration", predicate, active, span_id)),
        CompilerArchitectureFact::TypeShape {
            type_id,
            definition_path,
            field_visibilities,
            field_type_definitions,
            implemented_traits,
            span_id,
            ..
        } => canonical_identity(
            FACT_ID_DOMAIN,
            &(
                "type-shape",
                type_id,
                definition_path,
                field_visibilities,
                field_type_definitions,
                implemented_traits,
                span_id,
            ),
        ),
        CompilerArchitectureFact::FunctionSafety {
            item_id,
            qualified_path,
            output_type_id,
            is_unsafe,
            span_id,
            ..
        } => canonical_identity(
            FACT_ID_DOMAIN,
            &("function-safety", item_id, qualified_path, output_type_id, is_unsafe, span_id),
        ),
        CompilerArchitectureFact::CallArgumentProvenance {
            caller_item_id,
            callee_definition,
            argument_position,
            argument_type_id,
            origin_definitions,
            status,
            span_id,
            ..
        } => canonical_identity(
            FACT_ID_DOMAIN,
            &(
                "call-argument-provenance",
                caller_item_id,
                callee_definition,
                argument_position,
                argument_type_id,
                origin_definitions,
                status,
                span_id,
            ),
        ),
    }
    .expect("compiler architecture fact identity fields must serialize")
}

#[must_use]
pub fn unknown_fact_identity(unknown: &UnknownCompilerFact) -> String {
    canonical_identity(
        UNKNOWN_ID_DOMAIN,
        &(unknown.kind, &unknown.owner_item_id, &unknown.target_identity, &unknown.span_id, &unknown.detail),
    )
    .expect("unknown compiler fact identity fields must serialize")
}

#[must_use]
pub fn shard_identity(shard: &CompilerArchitectureShard) -> String {
    canonical_identity(
        SHARD_ID_DOMAIN,
        &(
            &shard.schema_version,
            &shard.unit,
            shard.collection_status,
            &shard.spans,
            &shard.facts,
            &shard.unknowns,
            &shard.non_claims,
        ),
    )
    .expect("compiler architecture shard identity fields must serialize")
}

pub fn normalize_architecture_shards(input: NormalizationInput) -> Result<NormalizationOutcome, String> {
    let mut issues = validate_limit_shape(&input.limits);
    let expected_by_id = validate_expected_units(&input, &mut issues);
    let mut valid_shards = Vec::with_capacity(input.shards.len());
    let invalid_capacity = input
        .invalid_shards
        .len()
        .checked_add(input.shards.len())
        .ok_or_else(|| "compiler architecture invalid-shard capacity overflowed usize".to_owned())?;
    let mut invalid_shard_ids = Vec::with_capacity(invalid_capacity);

    for invalid in &input.invalid_shards {
        invalid_shard_ids.push(invalid.shard_id.clone());
        issues.push(issue(&invalid.code, None, None, &invalid.message));
    }

    for shard in &input.shards {
        let before = issues.len();
        validate_shard(shard, &input.policy_blake3, &input.limits, &mut issues)?;
        if issues.len() == before {
            valid_shards.push(shard.clone());
        } else {
            invalid_shard_ids.push(shard.shard_id.clone());
        }
    }

    validate_count_limit("units", expected_by_id.len(), input.limits.max_units, None, &mut issues);
    let observed_shard_count = input.shards.len().saturating_add(input.invalid_shards.len());
    validate_count_limit("shards", observed_shard_count, input.limits.max_shards, None, &mut issues);

    let observed_counts = unit_counts(&valid_shards);
    let expected_ids = expected_by_id.keys().cloned().collect::<Vec<_>>();
    let observed_ids = observed_counts.keys().cloned().collect::<Vec<_>>();
    let missing_ids = difference(&expected_ids, &observed_ids);
    let unexpected_ids = difference(&observed_ids, &expected_ids);
    let duplicate_ids = observed_counts
        .iter()
        .filter_map(|(unit_id, count)| (*count > 1).then_some(unit_id.clone()))
        .collect::<Vec<_>>();

    for unit_id in &missing_ids {
        issues.push(issue(
            "coverage-missing-unit",
            Some(unit_id),
            None,
            "expected compilation unit has no valid shard",
        ));
    }
    for unit_id in &unexpected_ids {
        issues.push(issue(
            "coverage-unexpected-unit",
            Some(unit_id),
            None,
            "valid shard does not match an expected compilation unit",
        ));
    }
    for unit_id in &duplicate_ids {
        issues.push(issue(
            "coverage-duplicate-unit",
            Some(unit_id),
            None,
            "compilation unit has more than one valid shard",
        ));
    }

    invalid_shard_ids.sort();
    invalid_shard_ids.dedup();
    issues.sort();
    issues.dedup();

    let coverage_status = if missing_ids.is_empty()
        && unexpected_ids.is_empty()
        && duplicate_ids.is_empty()
        && invalid_shard_ids.is_empty()
        && issues.is_empty()
    {
        CoverageStatus::Complete
    } else {
        CoverageStatus::Incomplete
    };
    let mut coverage = CompilerArchitectureCoverage {
        schema_version: COMPILER_ARCHITECTURE_COVERAGE_SCHEMA_VERSION.to_owned(),
        coverage_id: String::new(),
        status: coverage_status,
        expected_unit_ids: expected_ids,
        observed_unit_ids: observed_ids,
        missing_unit_ids: missing_ids,
        unexpected_unit_ids: unexpected_ids,
        duplicate_unit_ids: duplicate_ids,
        invalid_shard_ids,
    };
    coverage.coverage_id = coverage_identity(&coverage);

    let architecture_ir = if coverage.status == CoverageStatus::Complete {
        let candidate = build_ir(&input, valid_shards, &mut issues)?;
        issues.is_empty().then_some(candidate)
    } else {
        None
    };
    issues.sort();
    issues.dedup();

    let status = if architecture_ir.is_some() && issues.is_empty() {
        CollectionStatus::Complete
    } else {
        CollectionStatus::Incomplete
    };
    let mut receipt = CompilerArchitectureReceipt {
        schema_version: COMPILER_ARCHITECTURE_RECEIPT_SCHEMA_VERSION.to_owned(),
        receipt_id: String::new(),
        status,
        policy_blake3: input.policy_blake3,
        coverage_id: coverage.coverage_id.clone(),
        architecture_ir_id: architecture_ir.as_ref().map(|ir| ir.architecture_ir_id.clone()),
        issue_ids: issues.iter().map(|current| current.issue_id.clone()).collect(),
        non_claims: sorted_unique(input.non_claims),
    };
    receipt.receipt_id = receipt_identity(&receipt);

    Ok(NormalizationOutcome {
        architecture_ir,
        coverage,
        issues,
        receipt,
    })
}

fn validate_limit_shape(limits: &ArchitectureCollectionLimits) -> Vec<CompilerArchitectureIssue> {
    let values = [
        ("max_units", limits.max_units),
        ("max_shards", limits.max_shards),
        ("max_modules", limits.max_modules),
        ("max_items", limits.max_items),
        ("max_types", limits.max_types),
        ("max_traits", limits.max_traits),
        ("max_implementations", limits.max_implementations),
        ("max_signatures", limits.max_signatures),
        ("max_calls", limits.max_calls),
        ("max_constructors", limits.max_constructors),
        ("max_effects", limits.max_effects),
        ("max_expansions", limits.max_expansions),
        ("max_configurations", limits.max_configurations),
        ("max_unknowns", limits.max_unknowns),
        ("max_spans", limits.max_spans),
        ("max_span_bytes", limits.max_span_bytes),
        ("max_shard_bytes", limits.max_shard_bytes),
        ("max_artifact_bytes", limits.max_artifact_bytes),
    ];
    values
        .into_iter()
        .filter(|(_, value)| *value == 0)
        .map(|(name, _)| issue("invalid-limit", None, None, &format!("collection limit `{name}` must be positive")))
        .collect()
}

fn validate_expected_units(
    input: &NormalizationInput,
    issues: &mut Vec<CompilerArchitectureIssue>,
) -> BTreeMap<String, ExpectedCompilationUnit> {
    let mut units = BTreeMap::new();
    for expected in &input.expected_units {
        validate_unit(&expected.unit, &input.policy_blake3, issues);
        if units.insert(expected.unit.unit_id.clone(), expected.clone()).is_some() {
            issues.push(issue(
                "duplicate-expected-unit",
                Some(&expected.unit.unit_id),
                None,
                "expected compilation units contain a duplicate identity",
            ));
        }
    }
    units
}

fn validate_shard(
    shard: &CompilerArchitectureShard,
    policy_blake3: &str,
    limits: &ArchitectureCollectionLimits,
    issues: &mut Vec<CompilerArchitectureIssue>,
) -> Result<(), String> {
    let unit_id = Some(shard.unit.unit_id.as_str());
    if shard.schema_version != COMPILER_ARCHITECTURE_SHARD_SCHEMA_VERSION {
        issues.push(issue("shard-schema", unit_id, None, "compiler architecture shard schema is not supported"));
    }
    validate_unit(&shard.unit, policy_blake3, issues);
    if !valid_blake3(&shard.shard_id) || shard.shard_id != shard_identity(shard) {
        issues.push(issue(
            "shard-identity",
            unit_id,
            None,
            "compiler architecture shard identity is stale or malformed",
        ));
    }
    if shard.collection_status != CollectionStatus::Complete {
        issues.push(issue(
            "shard-incomplete",
            unit_id,
            None,
            "collector marked the compiler architecture shard incomplete",
        ));
    }
    if shard.non_claims.is_empty() || shard.non_claims.iter().any(|value| value.trim().is_empty()) {
        issues.push(issue(
            "shard-non-claims",
            unit_id,
            None,
            "compiler architecture shard must contain non-empty non-claims",
        ));
    }

    let shard_bytes = canonical_json(shard)?.len();
    validate_count_limit("shard-bytes", shard_bytes, limits.max_shard_bytes, unit_id, issues);
    validate_count_limit("spans", shard.spans.len(), limits.max_spans, unit_id, issues);
    validate_count_limit("unknowns", shard.unknowns.len(), limits.max_unknowns, unit_id, issues);
    validate_fact_limits(&shard.facts, limits, unit_id, issues);
    validate_spans(&shard.spans, limits, unit_id, issues);
    validate_facts(&shard.facts, &shard.spans, unit_id, issues);
    validate_unknowns(&shard.unknowns, &shard.spans, &shard.facts, unit_id, issues);
    Ok(())
}

fn validate_unit(unit: &CompilationUnit, policy_blake3: &str, issues: &mut Vec<CompilerArchitectureIssue>) {
    let unit_id = Some(unit.unit_id.as_str());
    for (field, value) in [
        ("package_name", unit.package_name.as_str()),
        ("package_version", unit.package_version.as_str()),
        ("target_name", unit.target_name.as_str()),
        ("target_kind", unit.target_kind.as_str()),
        ("target_triple", unit.target_triple.as_str()),
    ] {
        if value.trim().is_empty() {
            issues.push(issue("unit-field", unit_id, None, &format!("compilation unit field `{field}` is empty")));
        }
    }
    for (field, value) in [
        ("toolchain_blake3", unit.toolchain_blake3.as_str()),
        ("collector_blake3", unit.collector_blake3.as_str()),
        ("policy_blake3", unit.policy_blake3.as_str()),
        ("source_blake3", unit.source_blake3.as_str()),
        ("invocation_blake3", unit.invocation_blake3.as_str()),
    ] {
        if !valid_blake3(value) {
            issues.push(issue(
                "unit-identity-field",
                unit_id,
                None,
                &format!("compilation unit field `{field}` is not BLAKE3 hex"),
            ));
        }
    }
    if unit.policy_blake3 != policy_blake3 {
        issues.push(issue(
            "unit-policy",
            unit_id,
            None,
            "compilation unit policy identity does not match the active policy",
        ));
    }
    if !valid_blake3(&unit.unit_id) || unit.unit_id != compilation_unit_identity(unit) {
        issues.push(issue("unit-identity", unit_id, None, "compilation unit identity is stale or malformed"));
    }
    if unit.crate_types.is_empty() {
        issues.push(issue("unit-crate-types", unit_id, None, "compilation unit has no crate type"));
    }
    if unit.features.windows(2).any(|pair| pair[0] >= pair[1]) {
        issues.push(issue("unit-features", unit_id, None, "compilation unit features are not sorted and unique"));
    }
}

fn validate_spans(
    spans: &[SourceSpan],
    limits: &ArchitectureCollectionLimits,
    unit_id: Option<&str>,
    issues: &mut Vec<CompilerArchitectureIssue>,
) {
    let mut ids = BTreeSet::new();
    for span in spans {
        if !ids.insert(span.span_id.as_str()) {
            issues.push(issue(
                "duplicate-span",
                unit_id,
                None,
                "compiler architecture shard contains a duplicate span",
            ));
        }
        match &span.origin {
            crate::model::SourceOrigin::Workspace { source_path } if !valid_workspace_path(source_path) => {
                issues.push(issue(
                    "span-path",
                    unit_id,
                    None,
                    "workspace source span path is not normalized and workspace-relative",
                ));
            }
            crate::model::SourceOrigin::ExternalGenerated { provenance_blake3 } if !valid_blake3(provenance_blake3) => {
                issues.push(issue(
                    "span-external-provenance",
                    unit_id,
                    None,
                    "external generated source span provenance is not BLAKE3 hex",
                ));
            }
            _ => {}
        }
        if span.byte_end < span.byte_start {
            issues.push(issue("span-range", unit_id, None, "source span byte range is reversed"));
        }
        if span.line_start == 0 || span.column_start == 0 {
            issues.push(issue("span-coordinate", unit_id, None, "source span line and column must be one-based"));
        }
        let byte_length = span.byte_end.saturating_sub(span.byte_start);
        match usize::try_from(byte_length) {
            Ok(byte_length) => {
                validate_count_limit("span-bytes", byte_length, limits.max_span_bytes, unit_id, issues);
            }
            Err(_) => issues.push(issue(
                "span-byte-range",
                unit_id,
                None,
                "source span byte length is not addressable on this host",
            )),
        }
        if !valid_blake3(&span.span_id) || span.span_id != source_span_identity(span) {
            issues.push(issue("span-identity", unit_id, None, "source span identity is stale or malformed"));
        }
    }
}

fn validate_facts(
    facts: &[CompilerArchitectureFact],
    spans: &[SourceSpan],
    unit_id: Option<&str>,
    issues: &mut Vec<CompilerArchitectureIssue>,
) {
    let span_ids = spans.iter().map(|span| span.span_id.as_str()).collect::<BTreeSet<_>>();
    let mut fact_ids = BTreeSet::new();
    let mut item_ids = BTreeSet::new();
    let mut type_ids = BTreeSet::new();
    let mut trait_ids = BTreeSet::new();
    let mut signature_ids = BTreeSet::new();

    for fact in facts {
        if !fact_ids.insert(fact.fact_id()) {
            issues.push(issue(
                "duplicate-fact",
                unit_id,
                Some(fact.fact_id()),
                "compiler architecture shard contains a duplicate fact",
            ));
        }
        if !valid_blake3(fact.fact_id()) || fact.fact_id() != fact_identity(fact) {
            issues.push(issue(
                "fact-identity",
                unit_id,
                Some(fact.fact_id()),
                "compiler architecture fact identity is stale or malformed",
            ));
        }
        match fact {
            CompilerArchitectureFact::Item { item_id, .. } => {
                item_ids.insert(item_id.as_str());
            }
            CompilerArchitectureFact::Type { type_id, .. } => {
                type_ids.insert(type_id.as_str());
            }
            CompilerArchitectureFact::Trait { trait_id, .. } => {
                trait_ids.insert(trait_id.as_str());
            }
            CompilerArchitectureFact::Signature { signature_id, .. } => {
                signature_ids.insert(signature_id.as_str());
            }
            _ => {}
        }
    }

    for fact in facts {
        validate_fact_references(fact, &span_ids, &item_ids, &type_ids, &trait_ids, &signature_ids, unit_id, issues);
    }
}

#[allow(clippy::too_many_arguments)]
fn validate_fact_references(
    fact: &CompilerArchitectureFact,
    spans: &BTreeSet<&str>,
    items: &BTreeSet<&str>,
    types: &BTreeSet<&str>,
    traits: &BTreeSet<&str>,
    signatures: &BTreeSet<&str>,
    unit_id: Option<&str>,
    issues: &mut Vec<CompilerArchitectureIssue>,
) {
    let fact_id = Some(fact.fact_id());
    match fact {
        CompilerArchitectureFact::Module { span_id, .. } | CompilerArchitectureFact::Trait { span_id, .. } => {
            require_ref("span", span_id, spans, unit_id, fact_id, issues)
        }
        CompilerArchitectureFact::Item {
            signature_id, span_id, ..
        } => {
            require_ref("span", span_id, spans, unit_id, fact_id, issues);
            if let Some(signature_id) = signature_id {
                require_ref("signature", signature_id, signatures, unit_id, fact_id, issues);
            }
        }
        CompilerArchitectureFact::Type { .. } => {}
        CompilerArchitectureFact::Implementation {
            trait_id,
            self_type_id,
            span_id,
            ..
        } => {
            require_ref("span", span_id, spans, unit_id, fact_id, issues);
            require_ref("type", self_type_id, types, unit_id, fact_id, issues);
            if let Some(trait_id) = trait_id {
                require_ref("trait", trait_id, traits, unit_id, fact_id, issues);
            }
        }
        CompilerArchitectureFact::Signature {
            owner_item_id,
            input_type_ids,
            output_type_id,
            ..
        } => {
            require_ref("item", owner_item_id, items, unit_id, fact_id, issues);
            for type_id in input_type_ids {
                require_ref("type", type_id, types, unit_id, fact_id, issues);
            }
            require_ref("type", output_type_id, types, unit_id, fact_id, issues);
        }
        CompilerArchitectureFact::Call {
            caller_item_id,
            span_id,
            ..
        } => {
            require_ref("item", caller_item_id, items, unit_id, fact_id, issues);
            require_ref("span", span_id, spans, unit_id, fact_id, issues);
        }
        CompilerArchitectureFact::Constructor {
            caller_item_id,
            constructed_type_id,
            span_id,
            ..
        } => {
            require_ref("item", caller_item_id, items, unit_id, fact_id, issues);
            require_ref("type", constructed_type_id, types, unit_id, fact_id, issues);
            require_ref("span", span_id, spans, unit_id, fact_id, issues);
        }
        CompilerArchitectureFact::Effect {
            caller_item_id,
            span_id,
            ..
        } => {
            require_ref("item", caller_item_id, items, unit_id, fact_id, issues);
            require_ref("span", span_id, spans, unit_id, fact_id, issues);
        }
        CompilerArchitectureFact::Expansion {
            generated_span_id,
            callsite_span_id,
            ..
        } => {
            require_ref("span", generated_span_id, spans, unit_id, fact_id, issues);
            require_ref("span", callsite_span_id, spans, unit_id, fact_id, issues);
        }
        CompilerArchitectureFact::Configuration { span_id, .. } => {
            if let Some(span_id) = span_id {
                require_ref("span", span_id, spans, unit_id, fact_id, issues);
            }
        }
        CompilerArchitectureFact::TypeShape { type_id, span_id, .. } => {
            require_ref("type", type_id, types, unit_id, fact_id, issues);
            require_ref("span", span_id, spans, unit_id, fact_id, issues);
        }
        CompilerArchitectureFact::FunctionSafety {
            item_id,
            output_type_id,
            span_id,
            ..
        } => {
            require_ref("item", item_id, items, unit_id, fact_id, issues);
            require_ref("type", output_type_id, types, unit_id, fact_id, issues);
            require_ref("span", span_id, spans, unit_id, fact_id, issues);
        }
        CompilerArchitectureFact::CallArgumentProvenance {
            caller_item_id,
            argument_type_id,
            span_id,
            ..
        } => {
            require_ref("item", caller_item_id, items, unit_id, fact_id, issues);
            require_ref("type", argument_type_id, types, unit_id, fact_id, issues);
            require_ref("span", span_id, spans, unit_id, fact_id, issues);
        }
    }
}

fn validate_unknowns(
    unknowns: &[UnknownCompilerFact],
    spans: &[SourceSpan],
    facts: &[CompilerArchitectureFact],
    unit_id: Option<&str>,
    issues: &mut Vec<CompilerArchitectureIssue>,
) {
    let spans = spans.iter().map(|span| span.span_id.as_str()).collect::<BTreeSet<_>>();
    let items = facts
        .iter()
        .filter_map(|fact| match fact {
            CompilerArchitectureFact::Item { item_id, .. } => Some(item_id.as_str()),
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let mut ids = BTreeSet::new();
    for unknown in unknowns {
        if !ids.insert(unknown.unknown_id.as_str()) {
            issues.push(issue(
                "duplicate-unknown",
                unit_id,
                None,
                "compiler architecture shard contains a duplicate unknown fact",
            ));
        }
        if !valid_blake3(&unknown.unknown_id) || unknown.unknown_id != unknown_fact_identity(unknown) {
            issues.push(issue(
                "unknown-identity",
                unit_id,
                None,
                "unknown compiler fact identity is stale or malformed",
            ));
        }
        if unknown.detail.trim().is_empty() {
            issues.push(issue("unknown-detail", unit_id, None, "unknown compiler fact has no detail"));
        }
        if let Some(owner_item_id) = &unknown.owner_item_id {
            require_ref("item", owner_item_id, &items, unit_id, None, issues);
        }
        if let Some(span_id) = &unknown.span_id {
            require_ref("span", span_id, &spans, unit_id, None, issues);
        }
    }
}

fn validate_fact_limits(
    facts: &[CompilerArchitectureFact],
    limits: &ArchitectureCollectionLimits,
    unit_id: Option<&str>,
    issues: &mut Vec<CompilerArchitectureIssue>,
) {
    let count = |predicate: fn(&CompilerArchitectureFact) -> bool| facts.iter().filter(|fact| predicate(fact)).count();
    validate_count_limit(
        "modules",
        count(|fact| matches!(fact, CompilerArchitectureFact::Module { .. })),
        limits.max_modules,
        unit_id,
        issues,
    );
    validate_count_limit(
        "items",
        count(|fact| matches!(fact, CompilerArchitectureFact::Item { .. })),
        limits.max_items,
        unit_id,
        issues,
    );
    validate_count_limit(
        "types",
        count(|fact| matches!(fact, CompilerArchitectureFact::Type { .. })),
        limits.max_types,
        unit_id,
        issues,
    );
    validate_count_limit(
        "traits",
        count(|fact| matches!(fact, CompilerArchitectureFact::Trait { .. })),
        limits.max_traits,
        unit_id,
        issues,
    );
    validate_count_limit(
        "implementations",
        count(|fact| matches!(fact, CompilerArchitectureFact::Implementation { .. })),
        limits.max_implementations,
        unit_id,
        issues,
    );
    validate_count_limit(
        "signatures",
        count(|fact| matches!(fact, CompilerArchitectureFact::Signature { .. })),
        limits.max_signatures,
        unit_id,
        issues,
    );
    validate_count_limit(
        "calls",
        count(|fact| matches!(fact, CompilerArchitectureFact::Call { .. })),
        limits.max_calls,
        unit_id,
        issues,
    );
    validate_count_limit(
        "constructors",
        count(|fact| matches!(fact, CompilerArchitectureFact::Constructor { .. })),
        limits.max_constructors,
        unit_id,
        issues,
    );
    validate_count_limit(
        "effects",
        count(|fact| matches!(fact, CompilerArchitectureFact::Effect { .. })),
        limits.max_effects,
        unit_id,
        issues,
    );
    validate_count_limit(
        "expansions",
        count(|fact| matches!(fact, CompilerArchitectureFact::Expansion { .. })),
        limits.max_expansions,
        unit_id,
        issues,
    );
    validate_count_limit(
        "configurations",
        count(|fact| matches!(fact, CompilerArchitectureFact::Configuration { .. })),
        limits.max_configurations,
        unit_id,
        issues,
    );
    validate_count_limit(
        "type-shapes",
        count(|fact| matches!(fact, CompilerArchitectureFact::TypeShape { .. })),
        limits.max_type_shapes,
        unit_id,
        issues,
    );
    validate_count_limit(
        "function-safety-facts",
        count(|fact| matches!(fact, CompilerArchitectureFact::FunctionSafety { .. })),
        limits.max_function_safety_facts,
        unit_id,
        issues,
    );
    validate_count_limit(
        "value-provenance-facts",
        count(|fact| matches!(fact, CompilerArchitectureFact::CallArgumentProvenance { .. })),
        limits.max_value_provenance_facts,
        unit_id,
        issues,
    );
}

fn build_ir(
    input: &NormalizationInput,
    shards: Vec<CompilerArchitectureShard>,
    issues: &mut Vec<CompilerArchitectureIssue>,
) -> Result<CompilerArchitectureIr, String> {
    let mut units = shards.iter().map(|shard| shard.unit.clone()).collect::<Vec<_>>();
    let mut spans = shards.iter().flat_map(|shard| shard.spans.clone()).collect::<Vec<_>>();
    let mut facts = shards.iter().flat_map(|shard| shard.facts.clone()).collect::<Vec<_>>();
    let mut unknowns = shards.iter().flat_map(|shard| shard.unknowns.clone()).collect::<Vec<_>>();
    units.sort_by(|left, right| left.unit_id.cmp(&right.unit_id));
    spans.sort();
    facts.sort_by(|left, right| left.fact_id().cmp(right.fact_id()));
    unknowns.sort();

    let mut ir = CompilerArchitectureIr {
        schema_version: COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION.to_owned(),
        architecture_ir_id: String::new(),
        policy_blake3: input.policy_blake3.clone(),
        units,
        spans,
        facts,
        unknowns,
        non_claims: sorted_unique(input.non_claims.clone()),
    };
    ir.architecture_ir_id = ir_identity(&ir);
    let artifact_bytes = canonical_json(&ir)?.len();
    validate_count_limit("artifact-bytes", artifact_bytes, input.limits.max_artifact_bytes, None, issues);
    Ok(ir)
}

fn unit_counts(shards: &[CompilerArchitectureShard]) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for shard in shards {
        *counts.entry(shard.unit.unit_id.clone()).or_insert(0) += 1;
    }
    counts
}

fn difference(left: &[String], right: &[String]) -> Vec<String> {
    let right = right.iter().collect::<BTreeSet<_>>();
    left.iter().filter(|item| !right.contains(item)).cloned().collect()
}

fn require_ref<T: AsRef<str>>(
    kind: &str,
    reference: &str,
    available: &BTreeSet<T>,
    unit_id: Option<&str>,
    fact_id: Option<&str>,
    issues: &mut Vec<CompilerArchitectureIssue>,
) {
    if !available.iter().any(|value| value.as_ref() == reference) {
        issues.push(issue(
            "invalid-reference",
            unit_id,
            fact_id,
            &format!("compiler architecture fact references missing {kind} `{reference}`"),
        ));
    }
}

fn validate_count_limit(
    name: &str,
    observed: usize,
    limit: usize,
    unit_id: Option<&str>,
    issues: &mut Vec<CompilerArchitectureIssue>,
) {
    if observed > limit {
        issues.push(issue(
            "limit-overflow",
            unit_id,
            None,
            &format!("collection `{name}` count {observed} exceeds limit {limit}"),
        ));
    }
}

fn issue(code: &str, unit_id: Option<&str>, fact_id: Option<&str>, message: &str) -> CompilerArchitectureIssue {
    let issue_id = canonical_identity(ISSUE_ID_DOMAIN, &(code, unit_id, fact_id, message))
        .expect("compiler architecture issue identity fields must serialize");
    CompilerArchitectureIssue {
        schema_version: COMPILER_ARCHITECTURE_ISSUE_SCHEMA_VERSION.to_owned(),
        issue_id,
        code: code.to_owned(),
        unit_id: unit_id.map(str::to_owned),
        fact_id: fact_id.map(str::to_owned),
        message: message.to_owned(),
    }
}

/// Compute the canonical identity of a compiler coverage artifact.
#[must_use]
pub fn coverage_identity(coverage: &CompilerArchitectureCoverage) -> String {
    canonical_identity(
        COVERAGE_ID_DOMAIN,
        &(
            &coverage.schema_version,
            coverage.status,
            &coverage.expected_unit_ids,
            &coverage.observed_unit_ids,
            &coverage.missing_unit_ids,
            &coverage.unexpected_unit_ids,
            &coverage.duplicate_unit_ids,
            &coverage.invalid_shard_ids,
        ),
    )
    .expect("compiler architecture coverage identity fields must serialize")
}

/// Compute the canonical identity of a normalized compiler Architecture IR artifact.
#[must_use]
pub fn ir_identity(ir: &CompilerArchitectureIr) -> String {
    canonical_identity(
        IR_ID_DOMAIN,
        &(&ir.schema_version, &ir.policy_blake3, &ir.units, &ir.spans, &ir.facts, &ir.unknowns, &ir.non_claims),
    )
    .expect("compiler architecture IR identity fields must serialize")
}

/// Compute the canonical identity of a compiler architecture collection receipt.
#[must_use]
pub fn receipt_identity(receipt: &CompilerArchitectureReceipt) -> String {
    canonical_identity(
        RECEIPT_ID_DOMAIN,
        &(
            &receipt.schema_version,
            receipt.status,
            &receipt.policy_blake3,
            &receipt.coverage_id,
            &receipt.architecture_ir_id,
            &receipt.issue_ids,
            &receipt.non_claims,
        ),
    )
    .expect("compiler architecture receipt identity fields must serialize")
}

fn valid_blake3(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LENGTH && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_workspace_path(path: &str) -> bool {
    !path.is_empty()
        && !path.starts_with('/')
        && path.split('/').all(|component| !component.is_empty() && component != "." && component != "..")
}

fn sorted_unique(mut values: Vec<String>) -> Vec<String> {
    values.sort();
    values.dedup();
    values
}
