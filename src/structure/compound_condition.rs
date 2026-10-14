use rustc_ast::BinOpKind;
use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects `if` conditions containing 3 or more `&&`/`||` operators
    /// at the same nesting level. Complex conditions should be decomposed
    /// into early returns, guard clauses, or named boolean bindings.
    pub tigerstyle::COMPOUND_CONDITION,
    Allow,
    "if condition has 3+ boolean operators"
}

declare_lint_pass!(CompoundCondition => [COMPOUND_CONDITION]);

fn count_logical_ops(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::Binary(op, lhs, rhs) if matches!(op.node, BinOpKind::And | BinOpKind::Or) => {
            1 + count_logical_ops(lhs) + count_logical_ops(rhs)
        }
        _ => 0,
    }
}

impl EarlyLintPass for CompoundCondition {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        let ExprKind::If(cond, _, _) = &expr.kind else {
            return;
        };

        // Don't fire on macro-expanded if expressions (e.g., from assert!)
        if expr.span.from_expansion() {
            return;
        }

        let ops = count_logical_ops(cond);
        if ops >= 3 {
            let message = format!("condition has {ops} boolean operators");
            if let Some(suggestion) = compound_condition_suggestion(cx, cond, expr) {
                emit_tiger_style_lint_with_suggestion(
                    cx,
                    COMPOUND_CONDITION,
                    cond.span,
                    message,
                    "compound_condition",
                    suggestion,
                );
            } else {
                emit_tiger_style_lint(cx, COMPOUND_CONDITION, cond.span, message, "compound_condition");
            }
        }
    }
}

fn collect_condition_leaves(expr: &Expr) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::Binary(op, lhs, rhs) if matches!(op.node, BinOpKind::And | BinOpKind::Or) => {
            let mut leaves = collect_condition_leaves(lhs);
            leaves.extend(collect_condition_leaves(rhs));
            leaves
        }
        _ => vec![expr],
    }
}

fn compound_condition_suggestion(cx: &EarlyContext<'_>, cond: &Expr, if_expr: &Expr) -> Option<LintSuggestion> {
    let sm = cx.sess().source_map();
    let leaves = collect_condition_leaves(cond);
    if leaves.len() < 3 {
        return None;
    }

    let indent_pos = sm.lookup_char_pos(if_expr.span.lo());
    let indent = " ".repeat(indent_pos.col.0);

    let mut bindings = Vec::new();
    let mut cond_names = Vec::new();
    for (i, leaf) in leaves.iter().enumerate() {
        let snippet = sm.span_to_snippet(leaf.span).ok()?;
        let name = format!("predicate_{}", i + 1);
        bindings.push(format!("let {name} = {snippet};"));
        cond_names.push(name);
    }

    // Determine the operator used (all top-level ops should be the same)
    let op = if let ExprKind::Binary(op, _, _) = &cond.kind {
        if matches!(op.node, BinOpKind::Or) { "||" } else { "&&" }
    } else {
        "&&"
    };

    let bindings_text = bindings.join(&format!("\n{indent}"));
    let simplified_cond = cond_names.join(&format!(" {op} "));

    let parts = vec![
        (if_expr.span.shrink_to_lo(), format!("{bindings_text}\n{indent}")),
        (cond.span, simplified_cond),
    ];

    Some(LintSuggestion::Multipart {
        message: "extract conditions into named predicates".to_owned(),
        parts,
        applicability: Applicability::HasPlaceholders,
    })
}
