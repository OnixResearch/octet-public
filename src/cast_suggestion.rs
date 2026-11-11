//! Shared rewrite for integer `as` casts: replace the cast with
//! `<target>::from(...)` when std converts losslessly, or with
//! `<target>::try_from(...)` plus the failure handling that compiles in the
//! enclosing body.

use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::def::DefKind;
use rustc_infer::infer::TyCtxtInferExt;
use rustc_lint::LateContext;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_span::Symbol;
use rustc_span::sym;
use rustc_trait_selection::infer::InferCtxtExt;

use crate::diagnostics::LintSuggestion;

/// How the enclosing body can propagate a failed `try_from` conversion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConversionFailurePath {
    /// Returns `Result<_, E>` with `E: From<TryFromIntError>`: `?` compiles.
    ResultQuestionMark,
    /// Returns `Option<_>`: `.ok()?` compiles.
    OptionQuestionMark,
    /// Returns `Result<_, E>` without a conversion from the cast error.
    ResultWithOtherError,
    /// `?` is not available in the enclosing body.
    Unavailable,
}

/// The `try_from` rewrite of `inner as target` for a failure path, and how
/// much of it the maintainer still has to write.
fn try_from_rewrite(path: ConversionFailurePath, target_name: &str, inner: &str) -> (String, Applicability) {
    match path {
        ConversionFailurePath::ResultQuestionMark => {
            (format!("{target_name}::try_from({inner})?"), Applicability::MaybeIncorrect)
        }
        ConversionFailurePath::OptionQuestionMark => {
            (format!("{target_name}::try_from({inner}).ok()?"), Applicability::MaybeIncorrect)
        }
        ConversionFailurePath::ResultWithOtherError => (
            format!("{target_name}::try_from({inner}).map_err(|_| /* error */)?"),
            Applicability::HasPlaceholders,
        ),
        ConversionFailurePath::Unavailable => (
            format!("{target_name}::try_from({inner}).unwrap_or_else(|_| /* handle out-of-range value */)"),
            Applicability::HasPlaceholders,
        ),
    }
}

/// Return type of the innermost body (function, closure, or async block)
/// that a `?` inside `cx.enclosing_body` would return from.
fn enclosing_return_ty<'tcx>(cx: &LateContext<'tcx>) -> Option<ty::Ty<'tcx>> {
    let owner = cx.tcx.hir_body_owner_def_id(cx.enclosing_body?);
    match cx.tcx.def_kind(owner) {
        DefKind::Fn | DefKind::AssocFn => {
            Some(cx.tcx.instantiate_bound_regions_with_erased(cx.tcx.fn_sig(owner).instantiate_identity().output()))
        }
        DefKind::Closure => match cx.typeck_results().node_type(cx.tcx.local_def_id_to_hir_id(owner)).kind() {
            ty::Closure(_, args) => {
                Some(cx.tcx.instantiate_bound_regions_with_erased(args.as_closure().sig().output()))
            }
            ty::Coroutine(_, args) => Some(args.as_coroutine().return_ty()),
            _ => None,
        },
        _ => None,
    }
}

/// `<target as TryFrom<source>>::Error`, normalized.
fn try_from_error_ty<'tcx>(
    cx: &LateContext<'tcx>,
    source_ty: ty::Ty<'tcx>,
    target_ty: ty::Ty<'tcx>,
) -> Option<ty::Ty<'tcx>> {
    let try_from = cx.tcx.get_diagnostic_item(sym::TryFrom)?;
    let error = cx
        .tcx
        .associated_items(try_from)
        .filter_by_name_unhygienic_and_kind(Symbol::intern("Error"), ty::AssocTag::Type)
        .next()?;
    let projection = ty::Ty::new_projection(cx.tcx, error.def_id, [target_ty, source_ty]);
    cx.tcx.try_normalize_erasing_regions(cx.typing_env(), projection).ok()
}

fn error_converts_from<'tcx>(cx: &LateContext<'tcx>, error_ty: ty::Ty<'tcx>, source_error_ty: ty::Ty<'tcx>) -> bool {
    let Some(from) = cx.tcx.get_diagnostic_item(sym::From) else {
        return false;
    };
    let error_ty = cx.tcx.erase_and_anonymize_regions(error_ty);
    let (infcx, param_env) = cx.tcx.infer_ctxt().build_with_typing_env(cx.typing_env());
    infcx
        .type_implements_trait(from, [error_ty, source_error_ty], param_env)
        .must_apply_modulo_regions()
}

fn conversion_failure_path<'tcx>(
    cx: &LateContext<'tcx>,
    source_ty: ty::Ty<'tcx>,
    target_ty: ty::Ty<'tcx>,
) -> ConversionFailurePath {
    let Some(return_ty) = enclosing_return_ty(cx) else {
        return ConversionFailurePath::Unavailable;
    };
    let ty::Adt(adt, args) = return_ty.kind() else {
        return ConversionFailurePath::Unavailable;
    };
    if cx.tcx.is_diagnostic_item(sym::Option, adt.did()) {
        return ConversionFailurePath::OptionQuestionMark;
    }
    if !cx.tcx.is_diagnostic_item(sym::Result, adt.did()) {
        return ConversionFailurePath::Unavailable;
    }
    match try_from_error_ty(cx, source_ty, target_ty) {
        Some(source_error_ty) if error_converts_from(cx, args.type_at(1), source_error_ty) => {
            ConversionFailurePath::ResultQuestionMark
        }
        _ => ConversionFailurePath::ResultWithOtherError,
    }
}

fn ty_name(ty: ty::Ty<'_>) -> &'static str {
    match ty.kind() {
        ty::Int(i) => i.name_str(),
        ty::Uint(u) => u.name_str(),
        _ => "?",
    }
}

/// Suggestion replacing the cast `expr` (`inner as target`). With
/// `lossless_from`, std converts every value on every platform, so the rewrite
/// is a machine-applicable `<target>::from(...)`. Otherwise it is
/// `<target>::try_from(...)` with failure handling chosen from the enclosing
/// body's return type: `?` for a `Result` whose error converts from the cast
/// error, `.ok()?` for `Option`, and a placeholder otherwise.
pub(crate) fn cast_conversion_suggestion<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &hir::Expr<'_>,
    inner: &hir::Expr<'_>,
    source_ty: ty::Ty<'tcx>,
    target_ty: ty::Ty<'tcx>,
    lossless_from: bool,
) -> Option<LintSuggestion> {
    // Neither `From::from` nor `TryFrom::try_from` is callable in const contexts.
    if cx
        .enclosing_body
        .is_some_and(|body| cx.tcx.hir_body_const_context(cx.tcx.hir_body_owner_def_id(body)).is_some())
    {
        return None;
    }
    // A cast in a macro body would be rewritten with one call site's operand,
    // and an operand from a macro would be replaced by the macro body's text.
    if expr.span.from_expansion() || inner.span.from_expansion() {
        return None;
    }
    let inner = cx.sess().source_map().span_to_snippet(inner.span).ok()?;
    let target_name = ty_name(target_ty);
    if lossless_from {
        return Some(LintSuggestion::Single {
            span: expr.span,
            message: format!("replace with lossless `{target_name}::from(...)`"),
            replacement: format!("{target_name}::from({inner})"),
            applicability: Applicability::MachineApplicable,
        });
    }
    let (replacement, applicability) =
        try_from_rewrite(conversion_failure_path(cx, source_ty, target_ty), target_name, &inner);
    Some(LintSuggestion::Single {
        span: expr.span,
        message: format!("replace with `{target_name}::try_from(...)`"),
        replacement,
        applicability,
    })
}

#[cfg(test)]
mod tests {
    use rustc_errors::Applicability;

    use super::ConversionFailurePath;
    use super::try_from_rewrite;

    #[test]
    fn try_from_rewrite_matches_failure_path() {
        let cases = [
            (ConversionFailurePath::ResultQuestionMark, "u32::try_from(x)?", Applicability::MaybeIncorrect),
            (ConversionFailurePath::OptionQuestionMark, "u32::try_from(x).ok()?", Applicability::MaybeIncorrect),
            (
                ConversionFailurePath::ResultWithOtherError,
                "u32::try_from(x).map_err(|_| /* error */)?",
                Applicability::HasPlaceholders,
            ),
            (
                ConversionFailurePath::Unavailable,
                "u32::try_from(x).unwrap_or_else(|_| /* handle out-of-range value */)",
                Applicability::HasPlaceholders,
            ),
        ];
        for (path, replacement, applicability) in cases {
            assert_eq!(try_from_rewrite(path, "u32", "x"), (replacement.to_owned(), applicability));
        }
    }
}
