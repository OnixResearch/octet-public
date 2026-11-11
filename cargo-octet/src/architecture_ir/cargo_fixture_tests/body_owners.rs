use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_architecture_ir::ItemKind;
use octet_standards::project_architecture_gate::ProjectArchitectureReceipt;

// r[verify octet.compiler_architecture_ir.body_owner_kinds]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_preserves_non_function_body_owners() {
    super::run_body_fixture(
        false,
        super::TestImport::None,
        super::NoStdMode::None,
        super::BodyFixture::NonFunctionOwners,
    );
}

// r[verify octet.compiler_architecture_ir.body_owner_kinds]
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_all_targets_preserves_non_function_body_owners_with_all_features() {
    super::run_body_fixture(
        true,
        super::TestImport::None,
        super::NoStdMode::None,
        super::BodyFixture::NonFunctionOwners,
    );
}

pub(super) fn assert_body_owner_facts(ir: &CompilerArchitectureIr) {
    for target in ["lib", "lib+test"] {
        let unit = ir.units.iter().find(|unit| unit.target_kind == target).expect("production library unit");
        let membership = ir.unit_memberships.iter().find(|row| row.unit_id == unit.unit_id).expect("unit membership");
        for (callee, owner, kind, minimum) in [
            ("::body_owners::scalar", "::NAMED", ItemKind::Constant, 1),
            ("::body_owners::scalar", "::STATIC", ItemKind::Static, 1),
            ("::body_owners::scalar", "::VALUE", ItemKind::Constant, 2),
            ("::body_owners::scalar", "::INHERENT", ItemKind::Constant, 1),
            ("::body_owners::scalar", "::inline_value::", ItemKind::Constant, 1),
            ("::body_owners::length", "::anonymous_length::", ItemKind::Constant, 2),
            ("::body_owners::scalar", "::macro_value::", ItemKind::Constant, 1),
            ("::body_owners::scalar", "::constant_with_nested_body::", ItemKind::Constant, 1),
            ("::body_owners::scalar", "::method", ItemKind::Function, 1),
            ("::body_owners::closure_scalar", "closure@src/body_owners.rs:", ItemKind::Other, 1),
            ("std::cell::Cell::<T>::new", "::LOCAL::__RUST_STD_INTERNAL_INIT", ItemKind::Constant, 1),
            ("std::cell::Cell::<T>::set", "closure@src/body_owners.rs:", ItemKind::Other, 1),
            ("std::mem::needs_drop", "::LOCAL::{constant#0}", ItemKind::Constant, 1),
        ] {
            let mut matched = 0;
            for fact in &ir.facts {
                let CompilerArchitectureFact::Call {
                    fact_id,
                    caller_item_id,
                    resolved_definition: Some(resolved),
                    ..
                } = fact
                else {
                    continue;
                };
                if !resolved.ends_with(callee) || !membership.fact_ids.contains(fact_id) {
                    continue;
                }
                let item = ir.facts.iter().find(|item| matches!(item, CompilerArchitectureFact::Item { item_id, .. } if item_id == caller_item_id)).expect("call must reference a retained item");
                let CompilerArchitectureFact::Item {
                    fact_id: item_fact,
                    qualified_path,
                    item_kind,
                    signature_id,
                    visibility,
                    ..
                } = item
                else {
                    unreachable!()
                };
                if !qualified_path.contains(owner) {
                    continue;
                }
                assert_eq!(*item_kind, kind, "{target}: {qualified_path}");
                assert_eq!(signature_id.is_some(), kind == ItemKind::Function, "{target}: {qualified_path}");
                if owner.ends_with("::") || owner == "::LOCAL::{constant#0}" || kind == ItemKind::Other {
                    assert_eq!(visibility, "private", "expression owner: {qualified_path}");
                } else if owner == "::LOCAL::__RUST_STD_INTERNAL_INIT" {
                    // thread_local! consumes `const { ... }` into a named
                    // initialization constant, not an anonymous expression.
                    assert_eq!(visibility, "restricted", "named initializer: {qualified_path}");
                }
                assert!(
                    membership.fact_ids.contains(item_fact),
                    "caller item must belong to the same production observer"
                );
                matched += 1;
            }
            if matched < minimum {
                for call in ir.facts.iter().filter(|fact| matches!(fact, CompilerArchitectureFact::Call { resolved_definition: Some(name), .. } if name.contains("Cell"))).take(8) {
                    if let CompilerArchitectureFact::Call { caller_item_id, resolved_definition, .. } = call {
                        let item = ir.facts.iter().find(|fact| matches!(fact, CompilerArchitectureFact::Item { item_id, .. } if item_id == caller_item_id));
                        eprintln!("Cell call diagnostic: {resolved_definition:?} owner={item:?}");
                    }
                }
            }
            assert!(
                matched >= minimum,
                "{target}: missing {owner} -> {callee}, got {matched}, expected at least {minimum}"
            );
            eprintln!("body-owner {target}: {owner} -> {callee}: {matched} matched {kind:?}");
        }
    }
}

pub(super) fn assert_nested_effects(ir: &CompilerArchitectureIr, receipt: &ProjectArchitectureReceipt) {
    assert_eq!(receipt.mode, octet_standards::project_architecture_policy::ArchitectureMode::Gate);
    assert_eq!(receipt.status, octet_standards::project_architecture_gate::ArchitecturePhaseStatus::Blocked);
    for target in ["lib", "lib+test"] {
        let unit = ir.units.iter().find(|unit| unit.target_kind == target).expect("production library unit");
        assert!(
            receipt.findings.iter().any(|finding| {
                finding.code == "domain-core-effect"
                    && finding.unit_id.as_ref() == Some(&unit.unit_id)
                    && finding.source_path.as_deref() == Some("src/body_owners.rs")
            }),
            "nested constant body effect must remain a production finding: {target}: {:?}",
            receipt.findings
        );
    }
}
