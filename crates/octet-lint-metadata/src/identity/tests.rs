use super::*;

const MANTLE_PLAN_PATH: &str = "crates/crunch-build/src/dynamic_plan.rs";
const WIRE_PATH: &str = "crates/crunch-build/src/wire.rs";

fn unit_domain() -> NominalDomainPolicy {
    NominalDomainPolicy {
        domain_id: "fixture.unit-id".to_owned(),
        symbols: vec!["UnitId".to_owned(), "DynamicUnit::id".to_owned()],
        raw_representation: RawRepresentation::String,
        core_scopes: vec![MANTLE_PLAN_PATH.to_owned()],
        compatibility_scopes: vec![WIRE_PATH.to_owned()],
        compatibility_required: true,
        wrapper_kind: WrapperKind::RefinedNewtype,
        constructor_policy: ConstructorPolicy::PrivateFallible,
    }
}

fn catalog() -> NominalDomainCatalog {
    NominalDomainCatalog {
        schema_version: NOMINAL_DOMAIN_SCHEMA_VERSION.to_owned(),
        default_mode: EnforcementMode::Inventory,
        heuristic_suffixes: vec!["Id".to_owned(), "Digest".to_owned()],
        domains: vec![unit_domain()],
        non_claims: vec!["Source-shape policy only.".to_owned()],
    }
}

fn descriptor(symbol: &'static str, source_path: &'static str) -> DomainDescriptor<'static> {
    DomainDescriptor {
        symbol,
        raw_representation: RawRepresentation::String,
        source_path,
        kind: DescriptorKind::TypeAlias,
    }
}

#[test]
fn embedded_catalog_is_valid_and_inventory_only() {
    let catalog = load_nominal_domain_catalog().expect("embedded catalog must pass");

    assert_eq!(catalog.schema_version, NOMINAL_DOMAIN_SCHEMA_VERSION);
    assert_eq!(catalog.default_mode, EnforcementMode::Inventory);
    assert!(!catalog.domains.is_empty());
    assert!(!catalog.heuristic_suffixes.is_empty());
    assert!(nominal_domain_catalog_diagnostics(&catalog).is_empty());
}

#[test]
fn exact_core_compatibility_and_outside_scopes_are_distinct() {
    let catalog = catalog();

    assert_eq!(
        classify_descriptor(&catalog, descriptor("UnitId", MANTLE_PLAN_PATH)),
        DomainDisposition::ExactCore {
            domain_id: "fixture.unit-id".to_owned()
        }
    );
    assert_eq!(classify_descriptor(&catalog, descriptor("UnitId", WIRE_PATH)), DomainDisposition::Compatibility {
        domain_id: "fixture.unit-id".to_owned()
    });
    assert_eq!(
        classify_descriptor(&catalog, descriptor("UnitId", "src/other.rs")),
        DomainDisposition::ExactOutsideScope {
            domain_id: "fixture.unit-id".to_owned()
        }
    );
}

#[test]
fn heuristic_candidates_never_replace_exact_scope_classification() {
    let catalog = catalog();

    assert_eq!(
        classify_descriptor(&catalog, descriptor("SessionId", "src/session.rs")),
        DomainDisposition::HeuristicCandidate {
            suffix: "Id".to_owned()
        }
    );
    assert_eq!(
        classify_descriptor(&catalog, descriptor("message", "src/session.rs")),
        DomainDisposition::Unclassified
    );
    assert_eq!(
        classify_descriptor(&catalog, descriptor("UnitId", "src/session.rs")),
        DomainDisposition::ExactOutsideScope {
            domain_id: "fixture.unit-id".to_owned()
        }
    );
}

#[test]
fn malformed_catalog_fails_closed_with_deterministic_diagnostics() {
    let mut catalog = catalog();
    catalog.schema_version = "future".to_owned();
    catalog.domains.push(unit_domain());
    catalog.domains[0].compatibility_scopes = vec![MANTLE_PLAN_PATH.to_owned()];
    catalog.domains[0].constructor_policy = ConstructorPolicy::PrivateInfallible;

    let diagnostics = nominal_domain_catalog_diagnostics(&catalog);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("schema_version")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate domain_id")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate nominal-domain symbol")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("core and compatibility")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("constructor policy")));
    assert!(diagnostics.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn refined_wrapper_contract_accepts_checked_private_construction() {
    let domain = unit_domain();
    let facts = WrapperFacts {
        inner_field_public: false,
        unchecked_from_raw: false,
        has_fallible_constructor: true,
        boundary_erasing_deref: false,
    };

    assert!(wrapper_contract_violations(&domain, facts).is_empty());
}

#[test]
fn refined_wrapper_contract_reports_every_supported_bypass() {
    let domain = unit_domain();
    let facts = WrapperFacts {
        inner_field_public: true,
        unchecked_from_raw: true,
        has_fallible_constructor: false,
        boundary_erasing_deref: true,
    };

    assert_eq!(wrapper_contract_violations(&domain, facts), vec![
        WrapperViolation::PublicInnerField,
        WrapperViolation::UncheckedPrimitiveConversion,
        WrapperViolation::MissingFallibleConstructor,
        WrapperViolation::BoundaryErasingDeref,
    ]);
}

#[test]
fn frozen_pilot_receipts_are_valid_and_reviewed() {
    let mantle: InventoryReceipt = serde_json::from_str(include_str!(
        "../../../../.cairn/archive/1970-01-01-enforce-nominal-domain-types/evidence/pilots/mantle-dynamic-plan.json"
    ))
    .expect("parse Mantle pilot receipt");
    let valence: InventoryReceipt = serde_json::from_str(include_str!(
        "../../../../.cairn/archive/1970-01-01-enforce-nominal-domain-types/evidence/pilots/valence-shared-identity.json"
    ))
    .expect("parse Valence pilot receipt");

    let policy_blake3 = blake3::hash(admission::catalog_json().as_bytes()).to_hex().to_string();
    for receipt in [&mantle, &valence] {
        assert!(validate_inventory_receipt(receipt).is_empty());
        assert!(receipt.review.reviewed);
        assert_eq!(receipt.policy_blake3, policy_blake3);
    }
    assert!(!mantle.findings.is_empty());
    assert!(valence.findings.is_empty());
    assert!(!valence.review.coverage_gaps.is_empty());
}

#[test]
fn inventory_receipt_validation_rejects_stale_counts_hashes_and_order() {
    let mut receipt: InventoryReceipt = serde_json::from_str(include_str!(
        "../../../../.cairn/archive/1970-01-01-enforce-nominal-domain-types/evidence/pilots/mantle-dynamic-plan.json"
    ))
    .expect("parse Mantle pilot receipt");
    receipt.policy_blake3 = "stale".to_owned();
    receipt.summary.exact_findings = 0;
    receipt.findings.swap(0, 1);

    let diagnostics = validate_inventory_receipt(&receipt);

    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("policy_blake3")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("exact_findings")));
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("deterministic")));
}

#[test]
fn malformed_json_and_unknown_enum_values_are_rejected() {
    let malformed = parse_nominal_domain_catalog("{");
    let unknown_representation = admission::catalog_json().replace("\"string\"", "\"uuid\"");

    assert!(malformed.is_err());
    assert!(parse_nominal_domain_catalog(&unknown_representation).is_err());
}
