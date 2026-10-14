use rustc_hir as hir;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;

const LINT_NAME: &str = "raw_domain_value";
const MAX_SYMBOL_SEGMENTS: usize = 4;

// Detects registered core fields and API values that still use primitive
// storage. Compatibility scopes stay outside this lint's reportable set.
rustc_session::declare_tool_lint! {
    pub tigerstyle::RAW_DOMAIN_VALUE,
    Allow,
    "registered core domain value uses a primitive representation"
}

pub struct RawDomainValue {
    catalog: octet_lint_metadata::nominal_domain::NominalDomainCatalog,
}

rustc_session::impl_lint_pass!(RawDomainValue => [RAW_DOMAIN_VALUE]);

impl RawDomainValue {
    #[must_use]
    pub fn new(catalog: octet_lint_metadata::nominal_domain::NominalDomainCatalog) -> Self {
        assert!(!catalog.domains.is_empty());
        Self { catalog }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PrimitiveOccurrence {
    representation: octet_lint_metadata::nominal_domain::RawRepresentation,
    kind: octet_lint_metadata::nominal_domain::DescriptorKind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ContainerKind {
    Sequence,
    Optional,
    Result,
    Map,
    Other,
}

pub(super) fn container_kind(path: &str) -> ContainerKind {
    match path {
        "alloc::vec::Vec" | "std::vec::Vec" => ContainerKind::Sequence,
        "core::option::Option" | "std::option::Option" => ContainerKind::Optional,
        "core::result::Result" | "std::result::Result" => ContainerKind::Result,
        "alloc::collections::btree::map::BTreeMap" | "std::collections::BTreeMap" => ContainerKind::Map,
        _ => ContainerKind::Other,
    }
}

fn direct_representation(
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

fn primitive_occurrences<'tcx>(
    cx: &rustc_lint::LateContext<'tcx>,
    resolved: rustc_middle::ty::Ty<'tcx>,
    default_kind: octet_lint_metadata::nominal_domain::DescriptorKind,
) -> Vec<PrimitiveOccurrence> {
    if let Some(representation) = direct_representation(cx, resolved) {
        return vec![PrimitiveOccurrence {
            representation,
            kind: default_kind,
        }];
    }

    let rustc_middle::ty::Adt(definition, args) = resolved.kind() else {
        return Vec::new();
    };
    let path = cx.tcx.def_path_str(definition.did());
    match container_kind(&path) {
        ContainerKind::Sequence | ContainerKind::Optional => {
            args.types().next().map_or_else(Vec::new, |inner| primitive_occurrences(cx, inner, default_kind))
        }
        ContainerKind::Result => {
            args.types().next().map_or_else(Vec::new, |inner| primitive_occurrences(cx, inner, default_kind))
        }
        ContainerKind::Map => {
            let mut types = args.types();
            let key_occurrences = types.next().map_or_else(Vec::new, |key| {
                primitive_occurrences(cx, key, octet_lint_metadata::nominal_domain::DescriptorKind::MapKey)
            });
            let value_occurrences =
                types.next().map_or_else(Vec::new, |value| primitive_occurrences(cx, value, default_kind));
            key_occurrences.into_iter().chain(value_occurrences).collect()
        }
        ContainerKind::Other => Vec::new(),
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

fn field_descriptor_kind(
    cx: &rustc_lint::LateContext<'_>,
    field: &hir::FieldDef<'_>,
) -> octet_lint_metadata::nominal_domain::DescriptorKind {
    let parent = cx.tcx.parent(field.def_id.to_def_id());
    if matches!(cx.tcx.def_kind(parent), rustc_hir::def::DefKind::Variant) {
        octet_lint_metadata::nominal_domain::DescriptorKind::EnumPayload
    } else {
        octet_lint_metadata::nominal_domain::DescriptorKind::Field
    }
}

pub(super) fn symbol_candidates(path: &str) -> Vec<String> {
    let segments = path.split("::").collect::<Vec<_>>();
    let upper_bound = segments.len().min(MAX_SYMBOL_SEGMENTS);
    (1..=upper_bound).rev().map(|count| segments[segments.len() - count..].join("::")).collect()
}

fn parameter_symbol_candidates(function_path: &str, parameter: &str) -> Vec<String> {
    symbol_candidates(&format!("{function_path}::{parameter}"))
}

fn exact_domain<'a>(
    catalog: &octet_lint_metadata::nominal_domain::NominalDomainCatalog,
    source_path: &'a str,
    symbols: &'a [String],
    occurrence: PrimitiveOccurrence,
) -> Option<(&'a str, String)> {
    for symbol in symbols {
        let descriptor = octet_lint_metadata::nominal_domain::DomainDescriptor {
            symbol,
            raw_representation: occurrence.representation,
            source_path,
            kind: occurrence.kind,
        };
        match octet_lint_metadata::nominal_domain::classify_descriptor(catalog, descriptor) {
            octet_lint_metadata::nominal_domain::DomainDisposition::ExactCore { domain_id } => {
                return Some((symbol, domain_id));
            }
            octet_lint_metadata::nominal_domain::DomainDisposition::Compatibility { .. }
            | octet_lint_metadata::nominal_domain::DomainDisposition::ExactOutsideScope { .. } => return None,
            octet_lint_metadata::nominal_domain::DomainDisposition::HeuristicCandidate { .. }
            | octet_lint_metadata::nominal_domain::DomainDisposition::Unclassified => {}
        }
    }
    None
}

struct ReportContext<'a, 'tcx> {
    cx: &'a rustc_lint::LateContext<'tcx>,
    lint: &'static rustc_lint::Lint,
    catalog: &'a octet_lint_metadata::nominal_domain::NominalDomainCatalog,
    source: &'a str,
}

fn report_occurrences(
    context: &ReportContext<'_, '_>,
    span: rustc_span::Span,
    symbols: &[String],
    occurrences: &[PrimitiveOccurrence],
) {
    for occurrence in occurrences {
        let Some((symbol, domain_id)) = exact_domain(context.catalog, context.source, symbols, *occurrence) else {
            continue;
        };
        crate::diagnostics::emit_tiger_style_lint(
            context.cx,
            context.lint,
            span,
            format!(
                "registered domain value `{symbol}` uses raw `{}` storage in core scope `{domain_id}`",
                occurrence.representation.as_str()
            ),
            LINT_NAME,
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for RawDomainValue {
    fn check_field_def(&mut self, cx: &rustc_lint::LateContext<'tcx>, field: &'tcx hir::FieldDef<'tcx>) {
        if field.span.from_expansion() {
            return;
        }
        let resolved = cx.tcx.type_of(field.def_id).instantiate_identity();
        let occurrences = primitive_occurrences(cx, resolved, field_descriptor_kind(cx, field));
        if occurrences.is_empty() {
            return;
        }

        let source = source_path(cx, field.span);
        let path = cx.tcx.def_path_str(field.def_id.to_def_id());
        let symbols = symbol_candidates(&path);
        let context = ReportContext {
            cx,
            lint: RAW_DOMAIN_VALUE,
            catalog: &self.catalog,
            source: &source,
        };
        report_occurrences(&context, field.span, &symbols, &occurrences);
    }

    fn check_fn(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        kind: rustc_hir::intravisit::FnKind<'tcx>,
        decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: rustc_span::Span,
        def_id: hir::def_id::LocalDefId,
    ) {
        if matches!(kind, rustc_hir::intravisit::FnKind::Closure) || span.from_expansion() {
            return;
        }

        let source = source_path(cx, span);
        let function_path = cx.tcx.def_path_str(def_id.to_def_id());
        let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
        for (parameter, resolved) in body.params.iter().zip(signature.inputs()) {
            let hir::PatKind::Binding(_, _, ident, _) = parameter.pat.kind else {
                continue;
            };
            let occurrences =
                primitive_occurrences(cx, *resolved, octet_lint_metadata::nominal_domain::DescriptorKind::Parameter);
            if occurrences.is_empty() {
                continue;
            }
            let symbols = parameter_symbol_candidates(&function_path, ident.name.as_str());
            let context = ReportContext {
                cx,
                lint: RAW_DOMAIN_VALUE,
                catalog: &self.catalog,
                source: &source,
            };
            report_occurrences(&context, ident.span, &symbols, &occurrences);
        }

        let hir::FnRetTy::Return(output) = decl.output else {
            return;
        };
        let occurrences =
            primitive_occurrences(cx, signature.output(), octet_lint_metadata::nominal_domain::DescriptorKind::Result);
        if occurrences.is_empty() {
            return;
        }
        let symbols = parameter_symbol_candidates(&function_path, "return");
        let context = ReportContext {
            cx,
            lint: RAW_DOMAIN_VALUE,
            catalog: &self.catalog,
            source: &source,
        };
        report_occurrences(&context, output.span, &symbols, &occurrences);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_domain_rejects_compatibility_and_outside_scopes() {
        let catalog = octet_lint_metadata::nominal_domain::load_nominal_domain_catalog().expect("embedded catalog");
        let symbols = vec!["DynamicUnit::id".to_owned()];
        let occurrence = PrimitiveOccurrence {
            representation: octet_lint_metadata::nominal_domain::RawRepresentation::String,
            kind: octet_lint_metadata::nominal_domain::DescriptorKind::Field,
        };

        assert!(exact_domain(&catalog, "crates/crunch-build/src/dynamic_plan.rs", &symbols, occurrence).is_some());
        assert!(exact_domain(&catalog, "crates/other/src/lib.rs", &symbols, occurrence).is_none());
    }
}
