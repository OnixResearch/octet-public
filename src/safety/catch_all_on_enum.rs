use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::PatKind;
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
    /// Detects `match` over enum types where the last arm is a wildcard (`_`)
    /// returning a "deny" value: `false`, `None`, `Err(...)`, or `0`. When
    /// new variants are added, the wildcard silently handles them instead of
    /// forcing an explicit decision.
    pub tigerstyle::CATCH_ALL_ON_ENUM,
    Warn,
    "catch-all wildcard on enum match returns deny value"
}

declare_lint_pass!(CatchAllOnEnum => [CATCH_ALL_ON_ENUM]);

fn arm_has_comment(snippet: &str) -> bool {
    for line in snippet.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("//") {
            return true;
        }
        if let Some(pos) = trimmed.find("//") {
            let before = &trimmed[..pos];
            let quote_count = before.chars().filter(|&c| c == '"').count();
            if quote_count % 2 == 0 {
                return true;
            }
        }
    }

    false
}

fn has_justification_comment(cx: &LateContext<'_>, last_arm: &hir::Arm<'_>) -> bool {
    let source_map = cx.sess().source_map();

    if let Ok(snippet) = source_map.span_to_snippet(last_arm.span)
        && arm_has_comment(&snippet)
    {
        return true;
    }

    if let Ok(lines) = source_map.span_to_lines(last_arm.span) {
        let Some(first_line) = lines.lines.first() else {
            return false;
        };

        for line_idx in (0..first_line.line_index).rev() {
            let Some(line) = lines.file.get_line(line_idx) else {
                break;
            };
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            return trimmed.starts_with("//");
        }
    }

    false
}

fn is_deny_value(expr: &hir::Expr<'_>) -> bool {
    match &expr.kind {
        ExprKind::Lit(lit) => {
            matches!(lit.node, rustc_ast::LitKind::Bool(false))
                || matches!(lit.node, rustc_ast::LitKind::Int(v, _) if v.get() == 0)
        }
        ExprKind::Path(hir::QPath::Resolved(_, path)) => {
            let name = path.segments.last().map_or("", |segment| segment.ident.name.as_str());
            name == "None"
        }
        ExprKind::Call(func, _) => {
            // Check for Err(...)
            if let ExprKind::Path(hir::QPath::Resolved(_, path)) = &func.kind {
                path.segments.last().is_some_and(|segment| segment.ident.name.as_str() == "Err")
            } else {
                false
            }
        }
        _ => false,
    }
}

impl<'tcx> LateLintPass<'tcx> for CatchAllOnEnum {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Match(scrutinee, arms, _) = &expr.kind else {
            return;
        };

        // Check if the scrutinee is an enum type
        let scrutinee_ty = cx.typeck_results().expr_ty(scrutinee);
        if !matches!(scrutinee_ty.kind(), ty::Adt(adt, _) if adt.is_enum()) {
            return;
        }

        // Check the last arm
        let Some(last_arm) = arms.last() else {
            return;
        };

        // Is it a wildcard?
        if !matches!(last_arm.pat.kind, PatKind::Wild) {
            return;
        }

        // Does it return a deny value?
        if !is_deny_value(last_arm.body) {
            return;
        }

        if has_justification_comment(cx, last_arm) {
            return;
        }

        let message = "catch-all `_` on enum match returns deny value — new variants will be silently handled";
        if let Some(suggestion) = expand_wildcard_suggestion(cx, scrutinee, last_arm) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                CATCH_ALL_ON_ENUM,
                last_arm.span,
                message,
                "catch_all_on_enum",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(cx, CATCH_ALL_ON_ENUM, last_arm.span, message, "catch_all_on_enum");
        }
    }
}

fn expand_wildcard_suggestion(
    cx: &LateContext<'_>,
    scrutinee: &hir::Expr<'_>,
    last_arm: &hir::Arm<'_>,
) -> Option<LintSuggestion> {
    let scrutinee_ty = cx.typeck_results().expr_ty(scrutinee);
    let ty::Adt(adt, _) = scrutinee_ty.kind() else {
        return None;
    };

    let sm = cx.sess().source_map();
    let indent_pos = sm.lookup_char_pos(last_arm.span.lo());
    let indent = " ".repeat(indent_pos.col.0);

    let type_name = cx.tcx.def_path_str(adt.did());

    let variants: Vec<_> = adt.variants().iter().collect();
    let mut arm_lines = Vec::new();
    for (i, variant) in variants.iter().enumerate() {
        let variant_name = variant.name.as_str();
        let pattern = if variant.fields.is_empty() {
            format!("{type_name}::{variant_name}")
        } else if variant.ctor.is_none() {
            // Struct variant (no constructor fn — uses brace syntax)
            format!("{type_name}::{variant_name} {{ .. }}")
        } else {
            // Tuple variant
            format!("{type_name}::{variant_name}(..)")
        };
        // Skip trailing comma on last arm — the original span includes it
        let comma = if i + 1 < variants.len() { "," } else { "" };
        arm_lines.push(format!("{pattern} => todo!(\"handle {variant_name}\"){comma}"));
    }

    if arm_lines.is_empty() {
        return None;
    }

    let replacement = arm_lines.join(&format!("\n{indent}"));

    Some(LintSuggestion::Single {
        span: last_arm.span,
        message: "expand wildcard into explicit variant arms".to_owned(),
        replacement,
        applicability: Applicability::HasPlaceholders,
    })
}
