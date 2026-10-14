use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context::late_hir_id_is_in_test_context;

const LINT_NAME: &str = "deref_polymorphism";

declare_tool_lint! {
    /// Detects public `Deref` impls that make one nominal domain type masquerade
    /// as another. Tiger Style prefers explicit accessors over inheritance-like
    /// method-surface expansion.
    pub tigerstyle::DEREF_POLYMORPHISM,
    Warn,
    "public Deref to nominal target hides API boundaries"
}

declare_lint_pass!(DerefPolymorphism => [DEREF_POLYMORPHISM]);

fn impl_is_deref(cx: &LateContext<'_>, impl_block: &hir::Impl<'_>) -> bool {
    let Some(of_trait) = impl_block.of_trait else {
        return false;
    };
    let Some(deref_trait) = cx.tcx.lang_items().deref_trait() else {
        return false;
    };

    of_trait.trait_ref.trait_def_id() == Some(deref_trait)
}

fn self_type_is_public_local(cx: &LateContext<'_>, self_ty: &hir::Ty<'_>) -> bool {
    let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &self_ty.kind else {
        return false;
    };
    let hir::def::Res::Def(_, def_id) = path.res else {
        return false;
    };
    let Some(local_def_id) = def_id.as_local() else {
        return false;
    };

    cx.tcx.visibility(local_def_id).is_public()
}

fn target_type<'tcx>(cx: &LateContext<'tcx>, impl_block: &hir::Impl<'tcx>) -> Option<&'tcx hir::Ty<'tcx>> {
    for item_id in impl_block.items {
        let impl_item = cx.tcx.hir_expect_impl_item(item_id.owner_id.def_id);
        if impl_item.ident.name == sym::Target
            && let hir::ImplItemKind::Type(ty) = impl_item.kind
        {
            return Some(ty);
        }
    }
    None
}

fn target_is_accepted_borrowed_view(ty: &hir::Ty<'_>) -> bool {
    match &ty.kind {
        hir::TyKind::Path(hir::QPath::Resolved(_, path)) => {
            path.segments.last().is_some_and(|segment| segment.ident.name == sym::str)
        }
        hir::TyKind::Slice(_) => true,
        _ => false,
    }
}

fn target_is_nominal(ty: &hir::Ty<'_>) -> bool {
    matches!(ty.kind, hir::TyKind::Path(hir::QPath::Resolved(_, _)))
}

fn target_name(cx: &LateContext<'_>, ty: &hir::Ty<'_>) -> String {
    cx.sess().source_map().span_to_snippet(ty.span).unwrap_or_else(|_| "nominal target".to_owned())
}

impl<'tcx> LateLintPass<'tcx> for DerefPolymorphism {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Impl(impl_block) = &item.kind else {
            return;
        };
        if item.span.from_expansion()
            || late_hir_id_is_in_test_context(cx, item.hir_id(), item.span)
            || !impl_is_deref(cx, impl_block)
            || !self_type_is_public_local(cx, impl_block.self_ty)
        {
            return;
        }

        let Some(target) = target_type(cx, impl_block) else {
            return;
        };
        if target_is_accepted_borrowed_view(target) || !target_is_nominal(target) {
            return;
        }

        let target = target_name(cx, target);
        emit_tiger_style_lint(
            cx,
            DEREF_POLYMORPHISM,
            item.span,
            format!(
                "public `Deref<Target = {target}>` expands the wrapper's API surface implicitly; prefer explicit accessors for domain polymorphism"
            ),
            LINT_NAME,
        );
    }
}
