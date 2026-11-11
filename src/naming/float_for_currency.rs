use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ItemKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects struct fields, and function or method return types, using
    /// `f32` or `f64` whose name has a whole `cost`, `price`, `budget`,
    /// `balance`, `amount`, or `fee` word (so `total_cost` matches, while
    /// `load_balancer_weight` and `feedback_score` do not). Names with a whole
    /// `_`-separated `percentage`, `ratio`, `factor`, or `rate` token are
    /// skipped. IEEE 754 loses precision above 2^53 — accumulated rounding
    /// across thousands of operations adds up. Use integer types with
    /// explicit units (e.g., `cost_microdollars: u64`).
    pub tigerstyle::FLOAT_FOR_CURRENCY,
    Allow,
    "floating point type used for currency"
}

declare_lint_pass!(FloatForCurrency => [FLOAT_FOR_CURRENCY]);

/// Currency words, matched as whole identifier words; `total_cost` matches `cost`.
const CURRENCY_WORDS: &[&str] = &["cost", "price", "budget", "balance", "amount", "fee"];

const SKIP_WORDS: &[&str] = &["percentage", "ratio", "factor", "rate"];

fn is_float_ty(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Float(_))
}

fn is_currency_name(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    crate::naming::identifier_words::identifier_has_word(name, CURRENCY_WORDS)
        && !lower.split('_').any(|token| SKIP_WORDS.contains(&token))
}

fn check_currency_return(
    cx: &LateContext<'_>,
    def_id: rustc_span::def_id::LocalDefId,
    name: &str,
    decl: &hir::FnDecl<'_>,
    span: rustc_span::Span,
) {
    let hir::FnRetTy::Return(return_ty) = decl.output else {
        return;
    };
    if span.from_expansion() || !is_currency_name(name) {
        return;
    }

    let output = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder().output();
    if is_float_ty(output) {
        emit_tiger_style_lint(
            cx,
            FLOAT_FOR_CURRENCY,
            return_ty.span,
            format!("floating point return type for currency function `{name}`"),
            "float_for_currency",
        );
    }
}

fn currency_field_suggestion(cx: &LateContext<'_>, field: &hir::FieldDef<'_>, name: &str) -> Option<LintSuggestion> {
    let sm = cx.sess().source_map();
    let snippet = sm.span_to_snippet(field.span).ok()?;

    let suggested_name = format!("{name}_micros");
    let replacement = snippet.replace("f64", "i64").replace("f32", "i64").replace(name, &suggested_name);

    Some(LintSuggestion::Single {
        span: field.span,
        message: format!("use integer type with unit suffix, e.g. `{suggested_name}: i64`"),
        replacement,
        applicability: Applicability::HasPlaceholders,
    })
}

impl<'tcx> LateLintPass<'tcx> for FloatForCurrency {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let variant_data = match &item.kind {
            ItemKind::Struct(_, _, variant_data) => variant_data,
            ItemKind::Fn { ident, sig, .. } => {
                check_currency_return(cx, item.owner_id.def_id, ident.name.as_str(), sig.decl, item.span);
                return;
            }
            _ => return,
        };

        for field in variant_data.fields() {
            let name = field.ident.name.as_str();

            // Skip non-currency names and legitimately float names
            if !is_currency_name(name) {
                continue;
            }

            // Check if the field type is f32 or f64
            let field_ty = cx.tcx.type_of(field.def_id).instantiate_identity();
            if is_float_ty(field_ty) {
                let message = format!("floating point type for currency field `{name}`");
                if let Some(suggestion) = currency_field_suggestion(cx, field, name) {
                    emit_tiger_style_lint_with_suggestion(
                        cx,
                        FLOAT_FOR_CURRENCY,
                        field.span,
                        message,
                        "float_for_currency",
                        suggestion,
                    );
                } else {
                    emit_tiger_style_lint(cx, FLOAT_FOR_CURRENCY, field.span, message, "float_for_currency");
                }
            }
        }
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(sig, _) = &impl_item.kind else {
            return;
        };
        // Trait impls inherit their signature; the trait declaration is checked instead.
        if let hir::Node::Item(parent) = cx.tcx.parent_hir_node(impl_item.hir_id())
            && let ItemKind::Impl(impl_block) = &parent.kind
            && impl_block.of_trait.is_some()
        {
            return;
        }
        let name = impl_item.ident.name.as_str();
        check_currency_return(cx, impl_item.owner_id.def_id, name, sig.decl, impl_item.span);
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, trait_item: &'tcx hir::TraitItem<'tcx>) {
        let hir::TraitItemKind::Fn(sig, _) = &trait_item.kind else {
            return;
        };
        let name = trait_item.ident.name.as_str();
        check_currency_return(cx, trait_item.owner_id.def_id, name, sig.decl, trait_item.span);
    }
}

#[cfg(test)]
mod tests {
    use super::is_currency_name;

    #[test]
    fn currency_words_match_whole_words_only() {
        assert!(is_currency_name("cost_usd"));
        assert!(is_currency_name("total_cost"));
        assert!(is_currency_name("registration_fee"));
        assert!(is_currency_name("monthly_fees"));
        assert!(is_currency_name("accountBalance"));
        assert!(!is_currency_name("load_balancer_weight"));
        assert!(!is_currency_name("feedback_score"));
        assert!(!is_currency_name("priceless_rank"));
        assert!(!is_currency_name("fee_rate"));
    }
}
