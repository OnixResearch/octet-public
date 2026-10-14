use rustc_hir as hir;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;

const LINT_NAME: &str = "primitive_domain_alias";

// Detects registered domain aliases that still resolve to primitive storage.
// Exact policy rows identify enforceable core domains. Name suffixes produce
// discovery output only while the catalog remains in inventory mode.
rustc_session::declare_tool_lint! {
    pub tigerstyle::PRIMITIVE_DOMAIN_ALIAS,
    Allow,
    "registered domain alias resolves to a primitive representation"
}

pub struct PrimitiveDomainAlias {
    catalog: octet_lint_metadata::nominal_domain::NominalDomainCatalog,
    mode: octet_lint_metadata::nominal_domain::EnforcementMode,
}

rustc_session::impl_lint_pass!(PrimitiveDomainAlias => [PRIMITIVE_DOMAIN_ALIAS]);

impl PrimitiveDomainAlias {
    #[must_use]
    pub fn new(
        catalog: octet_lint_metadata::nominal_domain::NominalDomainCatalog,
        mode: octet_lint_metadata::nominal_domain::EnforcementMode,
    ) -> Self {
        assert!(!catalog.domains.is_empty());
        assert!(!catalog.heuristic_suffixes.is_empty());
        Self { catalog, mode }
    }
}

fn raw_representation(
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
        rustc_middle::ty::Adt(definition, _) => raw_adt_representation(cx, definition.did()),
        _ => None,
    }
}

fn raw_adt_representation(
    cx: &rustc_lint::LateContext<'_>,
    definition: hir::def_id::DefId,
) -> Option<octet_lint_metadata::nominal_domain::RawRepresentation> {
    let path = cx.tcx.def_path_str(definition);
    match path.as_str() {
        "alloc::string::String" | "std::string::String" => {
            Some(octet_lint_metadata::nominal_domain::RawRepresentation::String)
        }
        "std::path::PathBuf" => Some(octet_lint_metadata::nominal_domain::RawRepresentation::Path),
        _ => None,
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

fn alias_message(symbol: &str, disposition: &octet_lint_metadata::nominal_domain::DomainDisposition) -> Option<String> {
    match disposition {
        octet_lint_metadata::nominal_domain::DomainDisposition::ExactCore { domain_id } => {
            Some(format!("domain alias `{symbol}` resolves to primitive storage declared by `{domain_id}`"))
        }
        octet_lint_metadata::nominal_domain::DomainDisposition::HeuristicCandidate { suffix } => Some(format!(
            "alias `{symbol}` ends with reviewed domain suffix `{suffix}` and needs policy classification"
        )),
        octet_lint_metadata::nominal_domain::DomainDisposition::Compatibility { .. }
        | octet_lint_metadata::nominal_domain::DomainDisposition::ExactOutsideScope { .. }
        | octet_lint_metadata::nominal_domain::DomainDisposition::Unclassified => None,
    }
}

fn disposition_is_reportable(
    mode: octet_lint_metadata::nominal_domain::EnforcementMode,
    disposition: &octet_lint_metadata::nominal_domain::DomainDisposition,
) -> bool {
    match disposition {
        octet_lint_metadata::nominal_domain::DomainDisposition::ExactCore { .. } => true,
        octet_lint_metadata::nominal_domain::DomainDisposition::HeuristicCandidate { .. } => {
            mode == octet_lint_metadata::nominal_domain::EnforcementMode::Inventory
        }
        octet_lint_metadata::nominal_domain::DomainDisposition::Compatibility { .. }
        | octet_lint_metadata::nominal_domain::DomainDisposition::ExactOutsideScope { .. }
        | octet_lint_metadata::nominal_domain::DomainDisposition::Unclassified => false,
    }
}

impl<'tcx> LateLintPass<'tcx> for PrimitiveDomainAlias {
    fn check_item(&mut self, cx: &rustc_lint::LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::TyAlias(ident, ..) = item.kind else {
            return;
        };
        if item.span.from_expansion() {
            return;
        }

        let resolved = cx.tcx.type_of(item.owner_id.def_id).instantiate_identity();
        let Some(raw_representation) = raw_representation(cx, resolved) else {
            return;
        };
        let symbol = ident.name.as_str();
        let source_path = source_path(cx, item.span);
        let descriptor = octet_lint_metadata::nominal_domain::DomainDescriptor {
            symbol,
            raw_representation,
            source_path: &source_path,
            kind: octet_lint_metadata::nominal_domain::DescriptorKind::TypeAlias,
        };
        let disposition = octet_lint_metadata::nominal_domain::classify_descriptor(&self.catalog, descriptor);
        if !disposition_is_reportable(self.mode, &disposition) {
            return;
        }
        let Some(message) = alias_message(symbol, &disposition) else {
            return;
        };

        crate::diagnostics::emit_tiger_style_lint(cx, PRIMITIVE_DOMAIN_ALIAS, ident.span, message, LINT_NAME);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_reports_exact_and_heuristic_candidates() {
        let exact = octet_lint_metadata::nominal_domain::DomainDisposition::ExactCore {
            domain_id: "fixture.unit-id".to_owned(),
        };
        let heuristic = octet_lint_metadata::nominal_domain::DomainDisposition::HeuristicCandidate {
            suffix: "Id".to_owned(),
        };

        assert!(disposition_is_reportable(octet_lint_metadata::nominal_domain::EnforcementMode::Inventory, &exact));
        assert!(disposition_is_reportable(
            octet_lint_metadata::nominal_domain::EnforcementMode::Inventory,
            &heuristic
        ));
        assert!(alias_message("UnitId", &exact).is_some());
        assert!(alias_message("SessionId", &heuristic).is_some());
    }

    #[test]
    fn promoted_modes_never_report_heuristic_candidates() {
        let heuristic = octet_lint_metadata::nominal_domain::DomainDisposition::HeuristicCandidate {
            suffix: "Id".to_owned(),
        };

        assert!(!disposition_is_reportable(octet_lint_metadata::nominal_domain::EnforcementMode::Warn, &heuristic));
        assert!(!disposition_is_reportable(octet_lint_metadata::nominal_domain::EnforcementMode::Deny, &heuristic));
        assert!(disposition_is_reportable(
            octet_lint_metadata::nominal_domain::EnforcementMode::Deny,
            &octet_lint_metadata::nominal_domain::DomainDisposition::ExactCore {
                domain_id: "fixture.unit-id".to_owned()
            }
        ));
    }

    #[test]
    fn compatibility_and_outside_scopes_are_silent() {
        let compatibility = octet_lint_metadata::nominal_domain::DomainDisposition::Compatibility {
            domain_id: "fixture.unit-id".to_owned(),
        };
        let outside = octet_lint_metadata::nominal_domain::DomainDisposition::ExactOutsideScope {
            domain_id: "fixture.unit-id".to_owned(),
        };

        assert!(!disposition_is_reportable(
            octet_lint_metadata::nominal_domain::EnforcementMode::Inventory,
            &compatibility
        ));
        assert!(!disposition_is_reportable(
            octet_lint_metadata::nominal_domain::EnforcementMode::Inventory,
            &outside
        ));
        assert!(alias_message("UnitId", &compatibility).is_none());
        assert!(alias_message("UnitId", &outside).is_none());
    }
}
