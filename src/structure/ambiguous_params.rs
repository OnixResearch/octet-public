use rustc_hir as hir;
use rustc_hir::ItemKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects function signatures with 2 or more consecutive parameters
    /// of the same primitive type. Callers can accidentally swap arguments
    /// without a type error. Suggest wrapping in a named struct.
    pub tigerstyle::AMBIGUOUS_PARAMS,
    Allow,
    "consecutive same-type primitive parameters are easy to swap"
}

declare_lint_pass!(AmbiguousParams => [AMBIGUOUS_PARAMS]);

fn ty_key(ty: ty::Ty<'_>) -> Option<String> {
    match ty.kind() {
        ty::Bool => Some("bool".into()),
        ty::Char => Some("char".into()),
        ty::Int(i) => Some(i.name_str().to_string()),
        ty::Uint(u) => Some(u.name_str().to_string()),
        ty::Float(f) => Some(f.name_str().to_string()),
        ty::Ref(_, inner, _) if matches!(inner.kind(), ty::Str) => Some("&str".into()),
        _ => None,
    }
}

impl<'tcx> LateLintPass<'tcx> for AmbiguousParams {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let ItemKind::Fn { .. } = &item.kind else {
            return;
        };

        // Skip trait implementations (signature is dictated by the trait)
        // This check_item won't fire for impl items anyway

        let def_id = item.owner_id.def_id;
        let fn_sig = cx.tcx.fn_sig(def_id).instantiate_identity();
        let inputs = fn_sig.skip_binder().inputs();

        if inputs.len() < 2 {
            return;
        }

        // Look for 2+ consecutive params of the same primitive type
        let mut i = 0;
        while i < inputs.len() {
            let Some(key) = ty_key(inputs[i]) else {
                i += 1;
                continue;
            };

            let mut run = 1;
            while i + run < inputs.len() {
                if ty_key(inputs[i + run]).as_deref() == Some(key.as_str()) {
                    run += 1;
                } else {
                    break;
                }
            }

            if run >= 2 {
                emit_tiger_style_lint(
                    cx,
                    AMBIGUOUS_PARAMS,
                    item.span,
                    format!("{run} consecutive `{key}` parameters are easy to swap by accident"),
                    "ambiguous_params",
                );
                return;
            }

            i += run;
        }
    }
}
