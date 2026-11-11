//! Production/test partition controls. Fixture membership is explicit, never inferred from names.
use octet_architecture_ir::UnitFactMembership;

use super::*;
use crate::project_architecture_policy::ArchitectureNoStdObligation;

impl Fixture {
    fn add_test_unit(&mut self, package: &str) -> CompilationUnit {
        let scope = format!("crates/{package}/tests");
        self.policy
            .roles
            .push(role_assignment(ArchitectureScopeKind::Source, &scope, ArchitectureRole::Test));
        let mut test = unit(package, CompilerArchitectureRole::Test);
        test.target_name = "integration".to_owned();
        test.target_kind = "test+test".to_owned();
        test.source_path = format!("{scope}/integration.rs");
        test.crate_types = vec!["bin".to_owned()];
        test.unit_id = octet_architecture_ir::compilation_unit_identity(&test);
        self.ir.units.push(test.clone());
        self.ir.unit_memberships.push(UnitFactMembership {
            unit_id: test.unit_id.clone(),
            fact_ids: Vec::new(),
            unknown_ids: Vec::new(),
        });
        test
    }

    fn move_to_test(
        &mut self,
        test: &CompilationUnit,
        select: impl Fn(&CompilerArchitectureFact) -> bool,
    ) -> Vec<String> {
        let ids = self
            .ir
            .facts
            .iter()
            .filter(|fact| select(fact))
            .map(|fact| fact.fact_id().to_owned())
            .collect::<Vec<_>>();
        assert!(!ids.is_empty(), "negative control must move real facts");
        for row in &mut self.ir.unit_memberships {
            row.fact_ids.retain(|id| !ids.contains(id));
            if row.unit_id == test.unit_id {
                row.fact_ids.extend(ids.iter().cloned());
            }
        }
        ids
    }

    fn add_test_effect_and_unknown(&mut self) -> CompilationUnit {
        let test = self.add_test_unit("domain");
        let source = span(&test.source_path);
        let caller = item(&source.span_id, "domain::integration::run");
        let mut effect = CompilerArchitectureFact::Effect {
            fact_id: String::new(),
            caller_item_id: item_id(&caller).to_owned(),
            capability_family: CapabilityFamily::Filesystem,
            primitive_definition: "std::fs::read".to_owned(),
            span_id: source.span_id.clone(),
        };
        set_fact_id(&mut effect);
        let mut unknown = UnknownCompilerFact {
            unknown_id: String::new(),
            kind: UnknownFactKind::DynamicDispatch,
            owner_item_id: None,
            target_identity: Some("dyn TestPort".to_owned()),
            span_id: Some(source.span_id.clone()),
            detail: "test dispatch unresolved".to_owned(),
        };
        unknown.unknown_id = octet_architecture_ir::unknown_fact_identity(&unknown);
        let row = self.ir.unit_memberships.last_mut().expect("test row");
        row.fact_ids.extend([caller.fact_id().to_owned(), effect.fact_id().to_owned()]);
        row.unknown_ids.push(unknown.unknown_id.clone());
        self.ir.spans.push(source);
        self.ir.facts.extend([caller, effect]);
        self.ir.unknowns.push(unknown);
        test
    }
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn test_only_effects_and_unknowns_have_explicit_dispositions_and_full_input_identity() {
    let mut fixture = Fixture::new(ArchitectureMode::Gate);
    let test = fixture.add_test_effect_and_unknown();
    let ir = fixture.prepared_ir();
    let original = ir.clone();
    let receipt = fixture.evaluate_ir(&ir, None);
    assert_eq!(receipt.status, ArchitecturePhaseStatus::Clean, "{:?}", receipt.findings);
    assert_eq!(ir, original, "projection cannot mutate or reseal the full IR");
    assert_eq!(receipt.architecture_ir_id, ir.architecture_ir_id);
    let disposition = receipt.test_only.as_ref().expect("evaluated partition");
    assert_eq!(disposition.unit_ids, [test.unit_id]);
    assert_eq!(disposition.fact_ids, ir.facts.iter().map(|fact| fact.fact_id().to_owned()).collect::<Vec<_>>());
    assert_eq!(disposition.unknown_ids, receipt.unknown_ids);
    assert_eq!(verify_project_architecture_receipt(&receipt), Vec::<String>::new());
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn shared_production_effects_and_unknowns_remain_findings() {
    let mut fixture = Fixture::new(ArchitectureMode::Gate);
    fixture.add_domain_effect();
    fixture.add_domain_unknown();
    let before = fixture.evaluate(None);
    let test = fixture.add_test_unit("domain");
    let domain = fixture
        .ir
        .units
        .iter()
        .find(|unit| unit.package_name == "domain" && unit.role == CompilerArchitectureRole::DomainCore)
        .expect("domain");
    let shared = fixture
        .ir
        .unit_memberships
        .iter()
        .find(|row| row.unit_id == domain.unit_id)
        .expect("domain row")
        .clone();
    let row = fixture.ir.unit_memberships.iter_mut().find(|row| row.unit_id == test.unit_id).expect("test row");
    row.fact_ids = shared.fact_ids;
    row.unknown_ids = shared.unknown_ids;
    let receipt = fixture.evaluate(None);
    assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked);
    assert_eq!(receipt.findings, before.findings);
    let disposition = receipt.test_only.as_ref().expect("partition");
    assert!(disposition.fact_ids.is_empty());
    assert!(disposition.unknown_ids.is_empty());
    assert!(!receipt.unknown_ids.is_empty());
    fixture.ir.units.reverse();
    fixture.ir.unit_memberships.reverse();
    fixture.ir.facts.reverse();
    fixture.ir.unknowns.reverse();
    assert_eq!(fixture.evaluate(None), receipt, "canonical input order cannot hide shared findings");
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn test_only_witness_shape_and_provenance_cannot_discharge_production_protocol() {
    for (shape, expected) in [
        (true, "witness-shape-resolution"),
        (false, "protected-executor-provenance-missing"),
    ] {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        fixture.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
        assert_eq!(fixture.evaluate(None).status, ArchitecturePhaseStatus::Clean);
        let test = fixture.add_test_unit("domain");
        let ids = fixture.move_to_test(&test, |fact| if shape {
            matches!(fact, CompilerArchitectureFact::TypeShape { definition_path, .. } if definition_path == "domain::PublicationAuthority")
        } else {
            matches!(fact, CompilerArchitectureFact::CallArgumentProvenance { callee_definition, .. } if callee_definition == "outbound::FilesystemPort::publish")
        });
        let receipt = fixture.evaluate(None);
        assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked, "{:?}", receipt.findings);
        assert!(receipt.findings.iter().any(|finding| finding.code == expected));
        assert_eq!(receipt.test_only.as_ref().expect("partition").fact_ids, sorted_dedup(ids));
    }
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn test_only_port_declarations_cannot_discharge_production_port_ownership() {
    let mut fixture = Fixture::new(ArchitectureMode::Gate);
    fixture.add_port_facts("outbound");
    assert_eq!(fixture.evaluate(None).status, ArchitecturePhaseStatus::Clean);
    let test = fixture.add_test_unit("application");
    fixture.move_to_test(&test, |fact| matches!(fact, CompilerArchitectureFact::Trait { .. }));
    let receipt = fixture.evaluate(None);
    assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked, "{:?}", receipt.findings);
    assert!(receipt.findings.iter().any(|finding| finding.code == "port-trait-resolution"));
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn no_std_requires_each_production_unit_not_a_package_or_test_witness() {
    let mut fixture = Fixture::new(ArchitectureMode::Gate);
    fixture.policy.core_no_std.push(ArchitectureNoStdObligation {
        scope: "domain".to_owned(),
    });
    let mut configuration = CompilerArchitectureFact::Configuration {
        fact_id: String::new(),
        predicate: "crate:no-std=domain".to_owned(),
        active: true,
        span_id: None,
    };
    set_fact_id(&mut configuration);
    let id = configuration.fact_id().to_owned();
    fixture.add_unit_facts("domain", Vec::new(), vec![configuration], Vec::new());
    assert_eq!(fixture.evaluate(None).status, ArchitecturePhaseStatus::Clean);
    let test = fixture.add_test_unit("domain");
    fixture.move_to_test(&test, |fact| fact.fact_id() == id);
    assert!(fixture.evaluate(None).findings.iter().any(|finding| finding.code == "no-std-obligation"));
    let production = fixture
        .ir
        .units
        .iter()
        .find(|unit| unit.package_name == "domain" && unit.role == CompilerArchitectureRole::DomainCore)
        .expect("production")
        .clone();
    fixture
        .ir
        .unit_memberships
        .iter_mut()
        .find(|row| row.unit_id == production.unit_id)
        .expect("production row")
        .fact_ids
        .push(id.clone());
    let mut library_test = production;
    library_test.target_kind = "lib+test".to_owned();
    library_test.unit_id = octet_architecture_ir::compilation_unit_identity(&library_test);
    fixture.ir.units.push(library_test.clone());
    fixture.ir.unit_memberships.push(UnitFactMembership {
        unit_id: library_test.unit_id,
        fact_ids: Vec::new(),
        unknown_ids: Vec::new(),
    });
    assert!(fixture.evaluate(None).findings.iter().any(|finding| finding.code == "no-std-obligation"));
    fixture.ir.unit_memberships.last_mut().expect("library-test row").fact_ids.push(id);
    assert_eq!(fixture.evaluate(None).status, ArchitecturePhaseStatus::Clean);
}

// A package-scoped obligation binds the package's library units: the library and
// its test configuration. Build scripts, binaries, examples, and benches link
// `std` through their own roots and cannot carry the library's crate attribute.
#[test]
fn package_no_std_binds_library_units_not_build_scripts_or_binaries() {
    let mut fixture = Fixture::new(ArchitectureMode::Gate);
    fixture.policy.core_no_std.push(ArchitectureNoStdObligation {
        scope: "domain".to_owned(),
    });
    let mut configuration = CompilerArchitectureFact::Configuration {
        fact_id: String::new(),
        predicate: "crate:no-std=domain".to_owned(),
        active: true,
        span_id: None,
    };
    set_fact_id(&mut configuration);
    let id = configuration.fact_id().to_owned();
    fixture.add_unit_facts("domain", Vec::new(), vec![configuration], Vec::new());
    let library = fixture.ir.units.iter().find(|unit| unit.package_name == "domain").expect("domain library").clone();
    for (kind, name, root, crate_type) in [
        ("custom-build", "build-script-build", "crates/domain/src/build.rs", "bin"),
        ("bin", "domain-tool", "crates/domain/src/main.rs", "bin"),
        ("example", "sample", "crates/domain/src/examples/sample.rs", "bin"),
        ("bench", "throughput", "crates/domain/src/benches/throughput.rs", "bin"),
    ] {
        let mut other = library.clone();
        other.target_kind = kind.to_owned();
        other.target_name = name.to_owned();
        other.source_path = root.to_owned();
        other.crate_types = vec![crate_type.to_owned()];
        other.unit_id = octet_architecture_ir::compilation_unit_identity(&other);
        fixture.ir.units.push(other.clone());
        fixture.ir.unit_memberships.push(UnitFactMembership {
            unit_id: other.unit_id,
            fact_ids: Vec::new(),
            unknown_ids: Vec::new(),
        });
    }
    let receipt = fixture.evaluate(None);
    assert_eq!(receipt.status, ArchitecturePhaseStatus::Clean, "{:?}", receipt.findings);

    let mut library_test = library;
    library_test.target_kind = "lib+test".to_owned();
    library_test.unit_id = octet_architecture_ir::compilation_unit_identity(&library_test);
    fixture.ir.units.push(library_test.clone());
    fixture.ir.unit_memberships.push(UnitFactMembership {
        unit_id: library_test.unit_id,
        fact_ids: Vec::new(),
        unknown_ids: Vec::new(),
    });
    assert!(fixture.evaluate(None).findings.iter().any(|finding| finding.code == "no-std-obligation"));
    fixture.ir.unit_memberships.last_mut().expect("library-test row").fact_ids.push(id);
    assert_eq!(fixture.evaluate(None).status, ArchitecturePhaseStatus::Clean);
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn excluding_test_only_types_cannot_turn_production_reference_checks_into_skips() {
    let mut fixture = Fixture::new(ArchitectureMode::Gate);
    fixture.add_protected_effect_facts(ValueProvenanceStatus::Exact, "domain::authorize_publication", false);
    let test = fixture.add_test_unit("domain");
    fixture.move_to_test(&test, |fact| matches!(fact, CompilerArchitectureFact::Type { definition_path: Some(path), .. } if path == "domain::PublicationAuthority"));
    let receipt = fixture.evaluate(None);
    assert_eq!(receipt.status, ArchitecturePhaseStatus::IntegrationFailure);
    assert!(receipt.findings.iter().any(|finding| finding.message.contains("missing type")));
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn shared_effect_and_constructor_checks_visit_every_production_observation() {
    for effect in [true, false] {
        let mut fixture = Fixture::new(ArchitectureMode::Gate);
        let (id, expected) = if effect {
            fixture.add_outbound_effect_only();
            let id = fixture
                .ir
                .facts
                .iter()
                .find(|fact| matches!(fact, CompilerArchitectureFact::Effect { .. }))
                .expect("effect")
                .fact_id()
                .to_owned();
            (id, "domain-core-effect")
        } else {
            let source = span("crates/host/src/main.rs");
            let caller = item(&source.span_id, "host::compose");
            let adapter = type_fact("outbound", "outbound::Adapter");
            let mut constructor = CompilerArchitectureFact::Constructor {
                fact_id: String::new(),
                caller_item_id: item_id(&caller).to_owned(),
                constructed_type_id: type_id(&adapter).to_owned(),
                constructor_definition: Some("outbound::Adapter::new".to_owned()),
                span_id: source.span_id.clone(),
            };
            set_fact_id(&mut constructor);
            let id = constructor.fact_id().to_owned();
            fixture.add_unit_facts("host", vec![source], vec![caller, adapter, constructor], Vec::new());
            assert_eq!(fixture.evaluate(None).status, ArchitecturePhaseStatus::Clean);
            (id, "adapter-construction-outside-root")
        };
        assert!(!fixture.evaluate(None).findings.iter().any(|finding| finding.code == expected));
        let domain =
            fixture.ir.units.iter().find(|unit| unit.package_name == "domain").expect("domain").unit_id.clone();
        fixture
            .ir
            .unit_memberships
            .iter_mut()
            .find(|row| row.unit_id == domain)
            .expect("domain row")
            .fact_ids
            .push(id);
        let receipt = fixture.evaluate(None);
        assert_eq!(receipt.status, ArchitecturePhaseStatus::Blocked);
        assert!(
            receipt
                .findings
                .iter()
                .any(|finding| finding.code == expected && finding.unit_id.as_ref() == Some(&domain))
        );
        fixture.ir.units.reverse();
        fixture.ir.unit_memberships.reverse();
        assert_eq!(fixture.evaluate(None), receipt);
    }
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn malformed_and_noncanonical_memberships_reject_even_after_resealing() {
    let mut fixture = Fixture::new(ArchitectureMode::Gate);
    fixture.add_test_effect_and_unknown();
    let ir = fixture.prepared_ir();
    for case in 0..4 {
        let mut invalid = ir.clone();
        match case {
            0 => invalid.unit_memberships.reverse(),
            1 => invalid.unit_memberships.clear(),
            2 => invalid.unit_memberships[0].fact_ids.push(HASH_C.to_owned()),
            _ => {
                for row in &mut invalid.unit_memberships {
                    row.unknown_ids.clear();
                }
            }
        }
        invalid.architecture_ir_id = octet_architecture_ir::ir_identity(&invalid);
        let receipt = fixture.evaluate_ir(&invalid, None);
        assert_eq!(receipt.status, ArchitecturePhaseStatus::IntegrationFailure, "case {case}");
        assert!(receipt.test_only.is_none());
        assert!(receipt.findings.iter().any(|finding| finding.code == "compiler-unit-membership"));
    }
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn receipt_partition_is_required_identity_bound_and_structurally_checked() {
    let mut fixture = Fixture::new(ArchitectureMode::Gate);
    fixture.add_test_effect_and_unknown();
    let receipt = fixture.evaluate(None);
    let mut json = serde_json::to_value(&receipt).expect("receipt JSON");
    json.as_object_mut().expect("object").remove("test_only");
    assert!(serde_json::from_value::<ProjectArchitectureReceipt>(json).is_err());
    let mut stale = receipt.clone();
    stale.test_only.as_mut().expect("partition").fact_ids.clear();
    assert!(verify_project_architecture_receipt(&stale).iter().any(|error| error.contains("identity")));
    for case in 0..4 {
        let mut invalid = receipt.clone();
        match case {
            0 => invalid.test_only = None,
            1 => invalid.test_only.as_mut().expect("partition").unit_ids.clear(),
            2 => invalid.test_only.as_mut().expect("partition").fact_ids.reverse(),
            _ => invalid.test_only.as_mut().expect("partition").unknown_ids.push(HASH_C.to_owned()),
        }
        invalid.receipt_id = receipt_identity(&invalid);
        assert!(!verify_project_architecture_receipt(&invalid).is_empty(), "case {case}");
    }
    // Receipt self-verification is consistency, not an independent ownership observation.
    stale.receipt_id = receipt_identity(&stale);
    assert!(verify_project_architecture_receipt(&stale).is_empty());
    assert_ne!(stale, fixture.evaluate(None), "artifact replay must compare the full disposition");
}
