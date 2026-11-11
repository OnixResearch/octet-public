use std::ops::Range;

use rustc_ast::BinOpKind;
use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_ast::UnOp;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Pos;
use rustc_span::Span;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects `assert!(a && b)` and `assert!(a || b)` where the macro
    /// argument contains a top-level `&&` or `||`. Compound assertions
    /// produce a single "assertion failed" message on failure, hiding
    /// which condition actually broke.
    pub tigerstyle::COMPOUND_ASSERTION,
    Warn,
    "compound assertion should be split into separate assertions"
}

#[derive(Default)]
pub struct CompoundAssertion {
    /// Expression of the statement entered most recently. The early lint visitor
    /// runs `check_stmt` immediately before `check_expr` on that expression, so a
    /// matching id means the expanded `assert!` sits in statement position.
    statement_expr: Option<rustc_ast::NodeId>,
}

impl_lint_pass!(CompoundAssertion => [COMPOUND_ASSERTION]);

#[derive(Clone, Debug, PartialEq, Eq)]
struct AssertionSplit {
    clauses: Vec<String>,
    has_trailing_semicolon: bool,
}

/// Classify the condition of an expanded `assert!`, which lowers to
/// `if !condition { panic(...) }`: return the condition when its top-level
/// operator is `&&` or `||`, together with that operator.
fn top_level_boolean_condition(cond: &Expr) -> Option<(&Expr, BinOpKind)> {
    let ExprKind::Unary(UnOp::Not, condition) = &cond.kind else {
        return None;
    };
    let ExprKind::Binary(op, _, _) = &condition.kind else {
        return None;
    };
    matches!(op.node, BinOpKind::And | BinOpKind::Or).then_some((&**condition, op.node))
}

/// Collect the operator spans of the `op` chain rooted at `expr` in source
/// order, so `a && b && c` yields both `&&` tokens. Operands written by a nested
/// macro are leaves: their operators are not in the user's source.
fn collect_operator_spans(expr: &Expr, op: BinOpKind, spans: &mut Vec<Span>) {
    if expr.span.from_expansion() {
        return;
    }
    if let ExprKind::Binary(bin_op, lhs, rhs) = &expr.kind
        && bin_op.node == op
    {
        collect_operator_spans(lhs, op, spans);
        spans.push(bin_op.span);
        collect_operator_spans(rhs, op, spans);
    }
}

/// Whether `text` holds a `//` or `/* */` comment outside string and char
/// literals.
fn contains_comment(text: &str) -> bool {
    rustc_lexer::tokenize(text, rustc_lexer::FrontmatterAllowed::No).any(|token| {
        matches!(token.kind, rustc_lexer::TokenKind::LineComment { .. } | rustc_lexer::TokenKind::BlockComment { .. })
    })
}

/// Split the source text of `assert!(condition)` into one clause per operand of
/// the top-level operator chain. `condition` and `operators` are byte ranges
/// into `call_text` taken from the parsed condition, so `&&` inside a clause
/// (a double reference such as `&&expected`) never splits it. Returns `None`
/// when the call carries a panic message, when a clause holds a comment (a
/// trailing `//` comment would swallow the clause's closing `)`), or when the
/// call is not the plain `assert!(...)` shape that a rewrite can reproduce.
fn split_assertion_call(
    call_text: &str,
    condition: Range<usize>,
    operators: &[Range<usize>],
) -> Option<AssertionSplit> {
    let prefix = call_text.get(..condition.start)?;
    if prefix.trim() != "assert!(" {
        return None;
    }

    let suffix = call_text.get(condition.end..)?.trim();
    let (suffix, has_trailing_semicolon) = match suffix.strip_suffix(';') {
        Some(rest) => (rest.trim_end(), true),
        None => (suffix, false),
    };
    // Anything but the closing parenthesis, such as `, "message"`, blocks the split.
    if suffix != ")" {
        return None;
    }

    let mut clauses = Vec::with_capacity(operators.len() + 1);
    let mut start = condition.start;
    for operator in operators {
        clauses.push(call_text.get(start..operator.start)?.trim().to_owned());
        start = operator.end;
    }
    clauses.push(call_text.get(start..condition.end)?.trim().to_owned());
    if clauses.len() < 2 || clauses.iter().any(|clause| clause.is_empty() || contains_comment(clause)) {
        return None;
    }

    Some(AssertionSplit {
        clauses,
        has_trailing_semicolon,
    })
}

/// Locate the condition and its operator chain inside the `assert!` callsite and
/// split the call's source text at those operators.
fn split_expanded_assertion(
    callsite: Span,
    call_text: &str,
    condition: &Expr,
    op: BinOpKind,
) -> Option<AssertionSplit> {
    let mut operator_spans = Vec::new();
    collect_operator_spans(condition, op, &mut operator_spans);
    let base = callsite.lo();
    let relative =
        |span: Span| callsite.contains(span).then(|| (span.lo() - base).to_usize()..(span.hi() - base).to_usize());
    let operators = operator_spans.into_iter().map(relative).collect::<Option<Vec<_>>>()?;
    split_assertion_call(call_text, relative(condition.span)?, &operators)
}

fn compound_assertion_suggestion(
    cx: &EarlyContext<'_>,
    callsite: Span,
    split: &AssertionSplit,
    is_statement: bool,
) -> LintSuggestion {
    let (replacement, tail) = if is_statement {
        let indent = " ".repeat(cx.sess().source_map().lookup_char_pos(callsite.lo()).col.0);
        let replacement = format!("assert!({})", split.clauses[0]);
        let mut tail = String::new();
        for (index, clause) in split.clauses.iter().enumerate().skip(1) {
            tail.push_str(";\n");
            tail.push_str(&indent);
            tail.push_str("assert!(");
            tail.push_str(clause);
            tail.push(')');
            if index + 1 == split.clauses.len() && split.has_trailing_semicolon {
                tail.push(';');
            }
        }
        (replacement, tail)
    } else {
        // Outside statement position (match arm, closure body, ...) no `;` may follow
        // the call, so keep a single unit-typed block expression.
        let replacement = format!("{{ assert!({})", split.clauses[0]);
        let mut tail = String::new();
        for clause in split.clauses.iter().skip(1) {
            tail.push_str("; assert!(");
            tail.push_str(clause);
            tail.push(')');
        }
        tail.push_str(" }");
        (replacement, tail)
    };

    LintSuggestion::Multipart {
        message: "split into separate assertions".to_owned(),
        parts: vec![(callsite, replacement), (callsite.shrink_to_hi(), tail)],
        applicability: Applicability::MachineApplicable,
    }
}

impl EarlyLintPass for CompoundAssertion {
    fn check_stmt(&mut self, _cx: &EarlyContext<'_>, stmt: &rustc_ast::Stmt) {
        self.statement_expr = match &stmt.kind {
            rustc_ast::StmtKind::Expr(expr) | rustc_ast::StmtKind::Semi(expr) => Some(expr.id),
            _ => None,
        };
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Only inspect expressions originating from macro expansion.
        if !expr.span.from_expansion() {
            return;
        }

        // `assert!(cond)` expands to `if !cond { panic(...) }`. Limiting to that
        // `if` avoids duplicate diagnostics on children sharing the callsite.
        let ExprKind::If(cond, _, _) = &expr.kind else {
            return;
        };

        let callsite = expr.span.source_callsite();
        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(callsite) else {
            return;
        };
        if !snippet.trim_start().starts_with("assert!(") {
            return;
        }

        let Some((condition, op)) = top_level_boolean_condition(cond) else {
            return;
        };
        // An operator produced by a nested macro is not in the user's source.
        if condition.span.from_expansion() || !callsite.contains(condition.span) {
            return;
        }

        let is_statement = self.statement_expr == Some(expr.id);
        // Only a conjunction splits into assertions that keep its meaning.
        let split = match op {
            BinOpKind::And => split_expanded_assertion(callsite, &snippet, condition, op),
            _ => None,
        };
        if let Some(split) = split {
            emit_tiger_style_lint_with_suggestion(
                cx,
                COMPOUND_ASSERTION,
                callsite,
                "compound assertion should be split into separate assertions",
                "compound_assertion",
                compound_assertion_suggestion(cx, callsite, &split, is_statement),
            );
        } else {
            emit_tiger_style_lint(
                cx,
                COMPOUND_ASSERTION,
                callsite,
                "compound assertion should be split into separate assertions",
                "compound_assertion",
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::AssertionSplit;
    use super::split_assertion_call;

    /// Split `call_text` with the condition located by text and each 2-byte
    /// operator given as an offset into the condition.
    fn split(call_text: &str, condition: &str, operator_offsets: &[usize]) -> Option<AssertionSplit> {
        let start = call_text.find(condition).expect("condition in call text");
        let operators: Vec<_> = operator_offsets.iter().map(|offset| start + offset..start + offset + 2).collect();
        split_assertion_call(call_text, start..start + condition.len(), &operators)
    }

    #[test]
    fn splits_conjunction_at_operator_offsets() {
        assert_eq!(
            split("assert!(a > 0 && b > 0 && c);", "a > 0 && b > 0 && c", &[6, 15]),
            Some(AssertionSplit {
                clauses: vec!["a > 0".into(), "b > 0".into(), "c".into()],
                has_trailing_semicolon: true,
            })
        );
    }

    #[test]
    fn keeps_double_reference_inside_its_clause() {
        assert_eq!(
            split("assert!(item == &&expected && ready)", "item == &&expected && ready", &[19]),
            Some(AssertionSplit {
                clauses: vec!["item == &&expected".into(), "ready".into()],
                has_trailing_semicolon: false,
            })
        );
    }

    #[test]
    fn rejects_message_bearing_assertions_for_machine_applicable_fix() {
        assert_eq!(split("assert!(a > 0 && b > 0, \"boom\")", "a > 0 && b > 0", &[6]), None);
    }

    #[test]
    fn rejects_a_condition_without_an_operator() {
        assert_eq!(split("assert!(ready)", "ready", &[]), None);
    }

    #[test]
    fn rejects_clauses_with_comments() {
        assert_eq!(
            split(
                "assert!(\n    ready // set by init\n        && done\n)",
                "ready // set by init\n        && done",
                &[29]
            ),
            None
        );
        assert_eq!(split("assert!(ready /* init */ && done)", "ready /* init */ && done", &[17]), None);
        assert_eq!(
            split("assert!(path == \"a//b\" && done)", "path == \"a//b\" && done", &[15]),
            Some(AssertionSplit {
                clauses: vec!["path == \"a//b\"".into(), "done".into()],
                has_trailing_semicolon: false,
            })
        );
    }
}
