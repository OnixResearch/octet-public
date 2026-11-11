use rustc_hir as hir;
use rustc_lint::LateLintPass;

rustc_session::declare_tool_lint! {
    /// Rejects direct `bool` parameters on public crate-owned callables and
    /// multiple direct `bool` parameters on private callables.
    ///
    /// Public call sites cannot see parameter names, so one direct boolean
    /// already hides domain meaning. Private callables stay practical with one
    /// predicate flag; two or more positional flags are ambiguous.
    ///
    /// Only direct `bool` parameters are inspected. Boolean return values,
    /// fields, ordinary predicates, `Option<bool>`, generic aliases, generated
    /// expansion output, and externally owned trait implementation signatures
    /// are outside this rule.
    pub tigerstyle::RAW_BOOL_PARAMETER,
    Warn,
    "raw boolean parameters obscure the domain meaning of arguments"
}

rustc_session::declare_lint_pass!(RawBoolParameter => [RAW_BOOL_PARAMETER]);

/// Direct boolean parameters a non-public callable may take before positional
/// flags become ambiguous. One private predicate flag remains practical.
const MIN_AMBIGUOUS_PRIVATE_BOOL_PARAMETERS: usize = 2;

/// Whether `ty` is written as the primitive `bool` type.
fn is_primitive_flag_type(ty: &hir::Ty<'_>) -> bool {
    matches!(
        ty.kind,
        hir::TyKind::Path(hir::QPath::Resolved(_, hir::Path {
            res: hir::def::Res::PrimTy(hir::PrimTy::Bool),
            ..
        },))
    )
}

/// Direct boolean parameter spans that are written in this crate's source.
fn primitive_flag_spans(decl: &hir::FnDecl<'_>) -> Vec<rustc_span::Span> {
    decl.inputs
        .iter()
        .filter(|input| !input.span.from_expansion() && is_primitive_flag_type(input))
        .map(|input| input.span)
        .collect()
}

fn lint_signature(
    cx: &rustc_lint::LateContext<'_>,
    span: rustc_span::Span,
    name: &str,
    decl: &hir::FnDecl<'_>,
    is_public: bool,
) {
    if span.from_expansion() {
        return;
    }

    let parameter_spans = primitive_flag_spans(decl);
    if is_public {
        for parameter_span in parameter_spans {
            crate::diagnostics::emit_tiger_style_lint(
                cx,
                RAW_BOOL_PARAMETER,
                parameter_span,
                format!(
                    "public callable `{name}` has a direct `bool` parameter; callers cannot see its domain meaning"
                ),
                "raw_bool_parameter",
            );
        }
        return;
    }

    if parameter_spans.len() >= MIN_AMBIGUOUS_PRIVATE_BOOL_PARAMETERS
        && let Some(first_span) = parameter_spans.first()
    {
        crate::diagnostics::emit_tiger_style_lint(
            cx,
            RAW_BOOL_PARAMETER,
            *first_span,
            format!(
                "private callable `{name}` has {} direct `bool` parameters; positional flags are ambiguous",
                parameter_spans.len()
            ),
            "raw_bool_parameter",
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for RawBoolParameter {
    fn check_item(&mut self, cx: &rustc_lint::LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { sig, ident, .. } = &item.kind else {
            return;
        };
        lint_signature(
            cx,
            item.span,
            ident.name.as_str(),
            sig.decl,
            cx.effective_visibilities.is_exported(item.owner_id.def_id),
        );
    }

    fn check_impl_item(&mut self, cx: &rustc_lint::LateContext<'tcx>, item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(sig, _) = &item.kind else {
            return;
        };
        let parent = cx.tcx.local_parent(item.owner_id.def_id);
        let hir::Node::Item(parent_item) = cx.tcx.hir_node(cx.tcx.local_def_id_to_hir_id(parent)) else {
            return;
        };
        let hir::ItemKind::Impl(impl_block) = &parent_item.kind else {
            return;
        };
        // A trait implementation repeats a signature its owner already defines.
        // Crate-owned trait declarations are checked at the declaration, and an
        // externally owned trait fixes the implementation signature.
        if impl_block.of_trait.is_some() {
            return;
        }
        lint_signature(
            cx,
            item.span,
            item.ident.name.as_str(),
            sig.decl,
            cx.effective_visibilities.is_exported(item.owner_id.def_id),
        );
    }

    fn check_trait_item(&mut self, cx: &rustc_lint::LateContext<'tcx>, item: &'tcx hir::TraitItem<'tcx>) {
        let hir::TraitItemKind::Fn(sig, _) = &item.kind else {
            return;
        };
        let trait_id = cx.tcx.local_parent(item.owner_id.def_id);
        lint_signature(
            cx,
            item.span,
            item.ident.name.as_str(),
            sig.decl,
            cx.effective_visibilities.is_exported(trait_id),
        );
    }

    fn check_expr(&mut self, cx: &rustc_lint::LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let hir::ExprKind::Closure(closure) = &expr.kind else {
            return;
        };
        // Closures are never public API, so they use the private threshold.
        lint_signature(cx, expr.span, "closure", closure.fn_decl, false);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn private_threshold_names_the_ambiguous_count() {
        assert_eq!(super::MIN_AMBIGUOUS_PRIVATE_BOOL_PARAMETERS, 2);
    }
}
