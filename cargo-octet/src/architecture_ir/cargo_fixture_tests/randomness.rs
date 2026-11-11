use octet_architecture_ir::CapabilityFamily;
use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_standards::project_architecture_gate::ProjectArchitectureReceipt;

// r[verify octet.compiler_architecture_ir.randomness_effects]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_preserves_registry_randomness_effects() {
    super::run_body_fixture(false, super::TestImport::None, super::NoStdMode::None, super::BodyFixture::Randomness);
}

// r[verify octet.compiler_architecture_ir.randomness_effects]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_preserves_registry_randomness_effects_with_all_features() {
    super::run_body_fixture(true, super::TestImport::None, super::NoStdMode::None, super::BodyFixture::Randomness);
}

pub(super) fn assert_randomness_facts(ir: &CompilerArchitectureIr, receipt: &ProjectArchitectureReceipt) {
    assert_eq!(receipt.mode, octet_standards::project_architecture_policy::ArchitectureMode::Gate);
    assert_eq!(receipt.status, octet_standards::project_architecture_gate::ArchitecturePhaseStatus::Blocked);
    for target in ["lib", "lib+test"] {
        let unit = ir.units.iter().find(|unit| unit.target_kind == target).expect("production library unit");
        let membership = ir.unit_memberships.iter().find(|row| row.unit_id == unit.unit_id).expect("unit membership");
        // rustc's diagnostic definition path retains this extern-crate alias.
        for definition in [
            "randomness::entropy::fill",
            "randomness::entropy::fill_uninit",
            "randomness::entropy::u32",
            "randomness::entropy::u64",
        ] {
            let calls = ir
                .facts
                .iter()
                .filter(|fact| {
                    matches!(fact,
                        CompilerArchitectureFact::Call { fact_id, resolved_definition: Some(name), .. }
                        if name == definition && membership.fact_ids.contains(fact_id)
                    )
                })
                .collect::<Vec<_>>();
            if calls.len() != 1 {
                for fact in ir
                    .facts
                    .iter()
                    .filter(|fact| {
                        matches!(fact,
                            CompilerArchitectureFact::Call { resolved_definition: Some(name), .. }
                                if name.contains("entropy") || name.contains("getrandom") || name.contains("randomness")
                        )
                    })
                    .take(12)
                {
                    eprintln!("randomness call diagnostic: {fact:?}");
                }
            }
            assert_eq!(calls.len(), 1, "{target}: missing or repeated real registry call: {definition}");
            let CompilerArchitectureFact::Call {
                caller_item_id: call_owner,
                span_id: call_span,
                ..
            } = calls[0]
            else {
                unreachable!()
            };
            let effects = ir.facts.iter().filter(|fact| matches!(fact,
                CompilerArchitectureFact::Effect { fact_id, primitive_definition, capability_family: CapabilityFamily::Randomness, .. }
                if primitive_definition == definition && membership.fact_ids.contains(fact_id)
            )).collect::<Vec<_>>();
            assert_eq!(effects.len(), 1, "{target}: missing or repeated randomness effect: {definition}");
            let CompilerArchitectureFact::Effect {
                fact_id,
                caller_item_id,
                span_id,
                ..
            } = effects[0]
            else {
                unreachable!()
            };
            assert_eq!(caller_item_id, call_owner, "effect must retain the call's owner");
            assert_eq!(span_id, call_span, "effect must retain the call's span");
            assert!(ir.facts.iter().any(|fact| matches!(fact,
                CompilerArchitectureFact::Item { fact_id, item_id, qualified_path, .. }
                if item_id == caller_item_id && qualified_path == "architecture_target_fixture::randomness::random_effects"
                    && membership.fact_ids.contains(fact_id)
            )), "effect caller must remain in the same production observer");
            assert!(
                receipt.findings.iter().any(|finding| finding.code == "domain-core-effect"
                    && finding.unit_id.as_ref() == Some(&unit.unit_id)
                    && finding.fact_id.as_ref() == Some(fact_id)
                    && finding.source_path.as_deref() == Some("src/randomness.rs")),
                "{target}: randomness must retain its production finding: {definition}"
            );
            eprintln!(
                "randomness {target}: {definition}: call, effect, owner, span, membership, production finding matched"
            );
        }
        let local = "architecture_target_fixture::randomness::getrandom::fill";
        assert!(
            ir.facts.iter().any(|fact| matches!(fact,
                CompilerArchitectureFact::Call { fact_id, resolved_definition: Some(name), .. }
                if name == local && membership.fact_ids.contains(fact_id)
            )),
            "local same-named call must remain visible"
        );
        assert!(!ir.facts.iter().any(|fact| matches!(fact,
            CompilerArchitectureFact::Effect { fact_id, primitive_definition, capability_family: CapabilityFamily::Randomness, .. }
            if primitive_definition == local && membership.fact_ids.contains(fact_id)
        )), "local same-named call must not become a randomness primitive");
        eprintln!("randomness {target}: local same-named helper retained without randomness promotion");
    }
}
