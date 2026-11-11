use rustc_ast::BinOpKind;
use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

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

#[derive(Default)]
pub struct CompoundCondition {
    /// The `if` expression that is the statement most recently visited.
    /// The early visitor checks a statement immediately before its expression,
    /// so this identifies `if`s whose `let` hoisting stays in statement position.
    statement_if: Option<rustc_ast::NodeId>,
}

impl_lint_pass!(CompoundCondition => [COMPOUND_CONDITION]);

fn count_logical_ops(expr: &Expr) -> usize {
    match &expr.kind {
        ExprKind::Binary(op, lhs, rhs) if matches!(op.node, BinOpKind::And | BinOpKind::Or) => {
            1 + count_logical_ops(lhs) + count_logical_ops(rhs)
        }
        _ => 0,
    }
}

impl EarlyLintPass for CompoundCondition {
    fn check_stmt(&mut self, _cx: &EarlyContext<'_>, stmt: &rustc_ast::Stmt) {
        self.statement_if = match &stmt.kind {
            rustc_ast::StmtKind::Expr(expr) | rustc_ast::StmtKind::Semi(expr)
                if matches!(expr.kind, ExprKind::If(..)) =>
            {
                Some(expr.id)
            }
            _ => None,
        };
    }

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
            let in_statement_position = self.statement_if == Some(expr.id);
            if in_statement_position && let Some(suggestion) = compound_condition_suggestion(cx, cond, expr) {
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

/// Flattens a chain of one logical operator; operands joined by any other
/// operator stay whole as leaves.
fn collect_condition_leaves(expr: &Expr, chain_op: BinOpKind) -> Vec<&Expr> {
    match &expr.kind {
        ExprKind::Binary(op, lhs, rhs) if op.node == chain_op => {
            let mut leaves = collect_condition_leaves(lhs, chain_op);
            leaves.extend(collect_condition_leaves(rhs, chain_op));
            leaves
        }
        _ => vec![expr],
    }
}

/// Returns whether hoisting `expr` into a `let` cannot panic or run side
/// effects that the original short-circuiting condition would have skipped.
fn is_hoistable_leaf(expr: &Expr) -> bool {
    let mut pending = vec![expr];
    while let Some(expr) = pending.pop() {
        match &expr.kind {
            ExprKind::Path(..) | ExprKind::Lit(_) => {}
            ExprKind::Paren(inner) | ExprKind::Field(inner, _) | ExprKind::Unary(rustc_ast::UnOp::Not, inner) => {
                pending.push(inner);
            }
            ExprKind::Binary(op, lhs, rhs)
                if matches!(
                    op.node,
                    BinOpKind::And
                        | BinOpKind::Or
                        | BinOpKind::Eq
                        | BinOpKind::Ne
                        | BinOpKind::Lt
                        | BinOpKind::Le
                        | BinOpKind::Gt
                        | BinOpKind::Ge
                ) =>
            {
                pending.push(lhs);
                pending.push(rhs);
            }
            _ => return false,
        }
    }
    true
}

fn compound_condition_suggestion(cx: &EarlyContext<'_>, cond: &Expr, if_expr: &Expr) -> Option<LintSuggestion> {
    let sm = cx.sess().source_map();
    let ExprKind::Binary(chain_op, _, _) = &cond.kind else {
        return None;
    };
    let leaves = collect_condition_leaves(cond, chain_op.node);
    if leaves.len() < 3 {
        return None;
    }
    // A leaf that is itself `&&`/`||` mixes operators, so the chain cannot be
    // rejoined with one operator; `let` leaves cannot be hoisted at all.
    let rejoinable = leaves.iter().all(|leaf| match &leaf.kind {
        ExprKind::Binary(op, _, _) => !matches!(op.node, BinOpKind::And | BinOpKind::Or),
        ExprKind::Let(..) => false,
        _ => true,
    });
    // The first leaf always runs; later leaves are hoisted out of the
    // short circuit, so they must not panic or have side effects.
    if !rejoinable || !leaves.iter().skip(1).all(|leaf| is_hoistable_leaf(leaf)) {
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

    let op = if chain_op.node == BinOpKind::Or { "||" } else { "&&" };

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
