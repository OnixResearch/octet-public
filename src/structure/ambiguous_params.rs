use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;

const LINT_NAME: &str = "ambiguous_params";
const MIN_AMBIGUOUS_RUN: usize = 2;

declare_tool_lint! {
    /// Detects function signatures with consecutive parameters of the same
    /// primitive or raw boundary type. Callers can swap these arguments
    /// without a type error.
    pub tigerstyle::AMBIGUOUS_PARAMS,
    Allow,
    "repeated same-type primitive parameters are easy to swap"
}

declare_lint_pass!(AmbiguousParams => [AMBIGUOUS_PARAMS]);

fn ty_key(cx: &LateContext<'_>, resolved: ty::Ty<'_>) -> Option<String> {
    match resolved.kind() {
        ty::Bool => Some("bool".into()),
        ty::Char => Some("char".into()),
        ty::Int(integer) => Some(integer.name_str().to_string()),
        ty::Uint(integer) => Some(integer.name_str().to_string()),
        ty::Float(float) => Some(float.name_str().to_string()),
        ty::Ref(_, inner, _) if matches!(inner.kind(), ty::Str) => Some("&str".into()),
        ty::Adt(definition, _) => match cx.tcx.def_path_str(definition.did()).as_str() {
            "alloc::string::String" | "std::string::String" => Some("String".into()),
            "std::path::PathBuf" => Some("PathBuf".into()),
            _ => None,
        },
        _ => None,
    }
}

fn first_swap_risk_group(keys: &[Option<String>]) -> Option<(usize, String)> {
    for key in keys.iter().flatten() {
        let count = keys.iter().flatten().filter(|candidate| *candidate == key).count();
        if count >= MIN_AMBIGUOUS_RUN {
            return Some((count, key.clone()));
        }
    }
    None
}

fn first_swap_risk_parameters(cx: &LateContext<'_>, inputs: &[ty::Ty<'_>]) -> Option<(usize, String)> {
    let keys = inputs.iter().map(|resolved| ty_key(cx, *resolved)).collect::<Vec<_>>();
    first_swap_risk_group(&keys)
}

fn report_signature(cx: &LateContext<'_>, def_id: hir::def_id::LocalDefId, span: rustc_span::Span, has_self: bool) {
    let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
    let start = usize::from(has_self);
    let inputs = &signature.inputs()[start..];
    let Some((count, key)) = first_swap_risk_parameters(cx, inputs) else {
        return;
    };

    emit_tiger_style_lint(
        cx,
        AMBIGUOUS_PARAMS,
        span,
        format!("{count} `{key}` parameters are easy to swap by accident"),
        LINT_NAME,
    );
}

impl<'tcx> LateLintPass<'tcx> for AmbiguousParams {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { .. } = &item.kind else {
            return;
        };
        report_signature(cx, item.owner_id.def_id, item.span, false);
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(signature, _) = &impl_item.kind else {
            return;
        };
        let parent_hir_id = cx.tcx.local_def_id_to_hir_id(cx.tcx.local_parent(impl_item.owner_id.def_id));
        let hir::Node::Item(parent_item) = cx.tcx.hir_node(parent_hir_id) else {
            return;
        };
        let hir::ItemKind::Impl(impl_block) = &parent_item.kind else {
            return;
        };
        if impl_block.of_trait.is_some() {
            return;
        }

        let has_self = signature.decl.implicit_self != hir::ImplicitSelfKind::None;
        report_signature(cx, impl_item.owner_id.def_id, impl_item.span, has_self);
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, trait_item: &'tcx hir::TraitItem<'tcx>) {
        let hir::TraitItemKind::Fn(signature, _) = &trait_item.kind else {
            return;
        };
        let has_self = signature.decl.implicit_self != hir::ImplicitSelfKind::None;
        report_signature(cx, trait_item.owner_id.def_id, trait_item.span, has_self);
    }
}

#[cfg(test)]
mod tests {
    use super::first_swap_risk_group;

    const EXPECTED_REPEATED_COUNT: usize = 2;

    #[test]
    fn repeated_non_consecutive_raw_types_are_ambiguous() {
        let keys = vec![
            Some("String".to_owned()),
            Some("u32".to_owned()),
            Some("String".to_owned()),
        ];

        assert_eq!(first_swap_risk_group(&keys), Some((EXPECTED_REPEATED_COUNT, "String".to_owned())));
    }

    #[test]
    fn distinct_nominal_types_are_not_raw_candidates() {
        let keys = vec![None, None];

        assert_eq!(first_swap_risk_group(&keys), None);
    }
}
