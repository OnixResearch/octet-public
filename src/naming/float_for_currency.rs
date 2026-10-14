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
    /// Detects struct fields using `f32` or `f64` whose name contains
    /// `cost`, `price`, `budget`, `balance`, `amount`, `fee`, or `total_cost`.
    /// IEEE 754 loses precision above 2^53 — accumulated rounding across
    /// thousands of operations adds up. Use integer types with explicit
    /// units (e.g., `cost_microdollars: u64`).
    pub tigerstyle::FLOAT_FOR_CURRENCY,
    Allow,
    "floating point type used for currency"
}

declare_lint_pass!(FloatForCurrency => [FLOAT_FOR_CURRENCY]);

const CURRENCY_WORDS: &[&str] = &["cost", "price", "budget", "balance", "amount", "fee", "total_cost"];

const SKIP_WORDS: &[&str] = &["percentage", "ratio", "factor", "rate"];

fn is_float_ty(ty: ty::Ty<'_>) -> bool {
    matches!(ty.kind(), ty::Float(_))
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
        let ItemKind::Struct(_, _, variant_data) = &item.kind else {
            return;
        };

        for field in variant_data.fields() {
            let name = field.ident.name.as_str();
            let lower = name.to_ascii_lowercase();

            // Skip non-currency names
            if !CURRENCY_WORDS.iter().any(|w| lower.contains(w)) {
                continue;
            }

            // Skip legitimately float names
            if SKIP_WORDS.iter().any(|w| lower.contains(w)) {
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
}
