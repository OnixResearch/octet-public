use rustc_hir as hir;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;

const LINT_NAME: &str = "newtype_invariant_bypass";
const FROM_TRAIT_NAME: &str = "From";
const FROM_TRAIT_ARG_COUNT: usize = 2;
// Detects source shapes that let callers bypass a registered refined newtype's
// constructor and validation boundary.
rustc_session::declare_tool_lint! {
    pub tigerstyle::NEWTYPE_INVARIANT_BYPASS,
    Allow,
    "registered refined newtype exposes an invariant bypass"
}

pub struct NewtypeInvariantBypass {
    catalog: octet_lint_metadata::nominal_domain::NominalDomainCatalog,
}

rustc_session::impl_lint_pass!(NewtypeInvariantBypass => [NEWTYPE_INVARIANT_BYPASS]);

impl NewtypeInvariantBypass {
    #[must_use]
    pub fn new(catalog: octet_lint_metadata::nominal_domain::NominalDomainCatalog) -> Self {
        assert!(!catalog.domains.is_empty());
        Self { catalog }
    }
}

fn source_path(cx: &rustc_lint::LateContext<'_>, span: rustc_span::Span) -> String {
    cx.sess()
        .source_map()
        .lookup_char_pos(span.lo())
        .file
        .name
        .prefer_local_unconditionally()
        .to_string_lossy()
        .into_owned()
}

fn refined_core_policy<'a>(
    catalog: &'a octet_lint_metadata::nominal_domain::NominalDomainCatalog,
    symbol: &str,
    source: &str,
) -> Option<&'a octet_lint_metadata::nominal_domain::NominalDomainPolicy> {
    catalog.domains.iter().find(|policy| {
        if policy.wrapper_kind != octet_lint_metadata::nominal_domain::WrapperKind::RefinedNewtype
            || policy.constructor_policy != octet_lint_metadata::nominal_domain::ConstructorPolicy::PrivateFallible
            || !policy.symbols.iter().any(|candidate| candidate == symbol)
        {
            return false;
        }
        let descriptor = octet_lint_metadata::nominal_domain::DomainDescriptor {
            symbol,
            raw_representation: policy.raw_representation,
            source_path: source,
            kind: octet_lint_metadata::nominal_domain::DescriptorKind::Field,
        };
        matches!(
            octet_lint_metadata::nominal_domain::classify_descriptor(catalog, descriptor),
            octet_lint_metadata::nominal_domain::DomainDisposition::ExactCore { .. }
        )
    })
}

fn direct_raw_representation(
    cx: &rustc_lint::LateContext<'_>,
    resolved: rustc_middle::ty::Ty<'_>,
) -> Option<octet_lint_metadata::nominal_domain::RawRepresentation> {
    match resolved.kind() {
        rustc_middle::ty::Uint(rustc_middle::ty::UintTy::U64) => {
            Some(octet_lint_metadata::nominal_domain::RawRepresentation::U64)
        }
        rustc_middle::ty::Uint(rustc_middle::ty::UintTy::Usize) => {
            Some(octet_lint_metadata::nominal_domain::RawRepresentation::Usize)
        }
        rustc_middle::ty::Adt(definition, _) => match cx.tcx.def_path_str(definition.did()).as_str() {
            "alloc::string::String" | "std::string::String" => {
                Some(octet_lint_metadata::nominal_domain::RawRepresentation::String)
            }
            "std::path::PathBuf" => Some(octet_lint_metadata::nominal_domain::RawRepresentation::Path),
            _ => None,
        },
        _ => None,
    }
}

fn self_type_symbol(self_ty: &hir::Ty<'_>) -> Option<String> {
    let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = self_ty.kind else {
        return None;
    };
    path.segments.last().map(|segment| segment.ident.name.to_string())
}

fn impl_trait_is_from(impl_block: &hir::Impl<'_>) -> bool {
    impl_block
        .of_trait
        .and_then(|trait_ref| trait_ref.trait_ref.path.segments.last())
        .is_some_and(|segment| segment.ident.name.as_str() == FROM_TRAIT_NAME)
}

fn from_input_representation(
    cx: &rustc_lint::LateContext<'_>,
    item: &hir::Item<'_>,
    impl_block: &hir::Impl<'_>,
) -> Option<octet_lint_metadata::nominal_domain::RawRepresentation> {
    if !impl_trait_is_from(impl_block) {
        return None;
    }
    let trait_ref = cx.tcx.impl_trait_ref(item.owner_id.def_id).instantiate_identity();
    if trait_ref.args.len() < FROM_TRAIT_ARG_COUNT {
        return None;
    }
    direct_raw_representation(cx, trait_ref.args.type_at(1))
}

fn deref_target_representation(
    cx: &rustc_lint::LateContext<'_>,
    impl_block: &hir::Impl<'_>,
) -> Option<octet_lint_metadata::nominal_domain::RawRepresentation> {
    let deref_trait = cx.tcx.lang_items().deref_trait()?;
    if impl_block.of_trait?.trait_ref.trait_def_id() != Some(deref_trait) {
        return None;
    }
    for item_id in impl_block.items {
        let impl_item = cx.tcx.hir_expect_impl_item(item_id.owner_id.def_id);
        if impl_item.ident.name != rustc_span::sym::Target {
            continue;
        }
        let resolved = cx.tcx.type_of(impl_item.owner_id.def_id).instantiate_identity();
        return direct_raw_representation(cx, resolved);
    }
    None
}

fn violation_message(symbol: &str, violation: octet_lint_metadata::nominal_domain::WrapperViolation) -> String {
    match violation {
        octet_lint_metadata::nominal_domain::WrapperViolation::PublicInnerField => {
            format!("refined newtype `{symbol}` exposes its raw inner field; keep the field private")
        }
        octet_lint_metadata::nominal_domain::WrapperViolation::UncheckedPrimitiveConversion => format!(
            "refined newtype `{symbol}` implements unchecked conversion from raw storage; use `TryFrom` or a fallible constructor"
        ),
        octet_lint_metadata::nominal_domain::WrapperViolation::BoundaryErasingDeref => {
            format!("refined newtype `{symbol}` dereferences to raw storage; use an explicit borrowed accessor")
        }
        octet_lint_metadata::nominal_domain::WrapperViolation::MissingFallibleConstructor => {
            format!("refined newtype `{symbol}` needs a fallible constructor")
        }
    }
}

fn isolated_violations(
    policy: &octet_lint_metadata::nominal_domain::NominalDomainPolicy,
    violation: octet_lint_metadata::nominal_domain::WrapperViolation,
) -> Vec<octet_lint_metadata::nominal_domain::WrapperViolation> {
    let facts = match violation {
        octet_lint_metadata::nominal_domain::WrapperViolation::PublicInnerField => {
            octet_lint_metadata::nominal_domain::WrapperFacts {
                inner_field_public: true,
                unchecked_from_raw: false,
                has_fallible_constructor: true,
                boundary_erasing_deref: false,
            }
        }
        octet_lint_metadata::nominal_domain::WrapperViolation::UncheckedPrimitiveConversion => {
            octet_lint_metadata::nominal_domain::WrapperFacts {
                inner_field_public: false,
                unchecked_from_raw: true,
                has_fallible_constructor: true,
                boundary_erasing_deref: false,
            }
        }
        octet_lint_metadata::nominal_domain::WrapperViolation::BoundaryErasingDeref => {
            octet_lint_metadata::nominal_domain::WrapperFacts {
                inner_field_public: false,
                unchecked_from_raw: false,
                has_fallible_constructor: true,
                boundary_erasing_deref: true,
            }
        }
        octet_lint_metadata::nominal_domain::WrapperViolation::MissingFallibleConstructor => {
            octet_lint_metadata::nominal_domain::WrapperFacts {
                inner_field_public: false,
                unchecked_from_raw: false,
                has_fallible_constructor: false,
                boundary_erasing_deref: false,
            }
        }
    };
    octet_lint_metadata::nominal_domain::wrapper_contract_violations(policy, facts)
}

fn emit_violation(
    cx: &rustc_lint::LateContext<'_>,
    span: rustc_span::Span,
    symbol: &str,
    policy: &octet_lint_metadata::nominal_domain::NominalDomainPolicy,
    violation: octet_lint_metadata::nominal_domain::WrapperViolation,
) {
    for confirmed in isolated_violations(policy, violation) {
        crate::diagnostics::emit_tiger_style_lint(
            cx,
            NEWTYPE_INVARIANT_BYPASS,
            span,
            violation_message(symbol, confirmed),
            LINT_NAME,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for NewtypeInvariantBypass {
    fn check_item(&mut self, cx: &rustc_lint::LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        if item.span.from_expansion() {
            return;
        }
        let source = source_path(cx, item.span);
        match &item.kind {
            hir::ItemKind::Struct(ident, _, fields) => {
                let symbol = ident.name.as_str();
                let Some(policy) = refined_core_policy(&self.catalog, symbol, &source) else {
                    return;
                };
                for field in fields.fields() {
                    let resolved = cx.tcx.type_of(field.def_id).instantiate_identity();
                    if cx.tcx.visibility(field.def_id).is_public()
                        && direct_raw_representation(cx, resolved) == Some(policy.raw_representation)
                    {
                        emit_violation(
                            cx,
                            field.span,
                            symbol,
                            policy,
                            octet_lint_metadata::nominal_domain::WrapperViolation::PublicInnerField,
                        );
                    }
                }
            }
            hir::ItemKind::Impl(impl_block) => {
                let Some(symbol) = self_type_symbol(impl_block.self_ty) else {
                    return;
                };
                let Some(policy) = refined_core_policy(&self.catalog, &symbol, &source) else {
                    return;
                };
                if from_input_representation(cx, item, impl_block) == Some(policy.raw_representation) {
                    emit_violation(
                        cx,
                        item.span,
                        &symbol,
                        policy,
                        octet_lint_metadata::nominal_domain::WrapperViolation::UncheckedPrimitiveConversion,
                    );
                }
                if deref_target_representation(cx, impl_block) == Some(policy.raw_representation) {
                    emit_violation(
                        cx,
                        item.span,
                        &symbol,
                        policy,
                        octet_lint_metadata::nominal_domain::WrapperViolation::BoundaryErasingDeref,
                    );
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolated_facts_report_only_the_selected_bypass() {
        let catalog = octet_lint_metadata::nominal_domain::load_nominal_domain_catalog().expect("embedded catalog");
        let policy = catalog
            .domains
            .iter()
            .find(|domain| domain.domain_id == "mantle.dynamic-plan.unit-id")
            .expect("refined fixture policy");
        for violation in [
            octet_lint_metadata::nominal_domain::WrapperViolation::PublicInnerField,
            octet_lint_metadata::nominal_domain::WrapperViolation::UncheckedPrimitiveConversion,
            octet_lint_metadata::nominal_domain::WrapperViolation::BoundaryErasingDeref,
            octet_lint_metadata::nominal_domain::WrapperViolation::MissingFallibleConstructor,
        ] {
            assert_eq!(isolated_violations(policy, violation), vec![violation]);
        }
    }

    #[test]
    fn refined_policy_selection_respects_scope_and_constructor_contract() {
        let catalog = octet_lint_metadata::nominal_domain::load_nominal_domain_catalog().expect("embedded catalog");

        assert!(refined_core_policy(&catalog, "UnitId", "crates/crunch-build/src/dynamic_plan.rs").is_some());
        assert!(refined_core_policy(&catalog, "UnitId", "crates/other/src/lib.rs").is_none());
        assert!(refined_core_policy(&catalog, "UnknownId", "crates/crunch-build/src/dynamic_plan.rs").is_none());
    }
}
