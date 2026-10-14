use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
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

declare_lint_pass!(CompoundAssertion => [COMPOUND_ASSERTION]);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TopLevelBooleanOp {
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct TopLevelAssertionSplit {
    op: TopLevelBooleanOp,
    clauses: Vec<String>,
    has_trailing_semicolon: bool,
}

/// Scan a macro-call snippet for top-level `&&` or `||` inside `assert!(...)`.
///
/// We work from the callsite source text because Dylint does not reliably run
/// pre-expansion passes. The expanded AST still carries the original macro
/// callsite span, which lets us re-read the user's source.
fn has_top_level_compound_assertion(snippet: &str) -> bool {
    let Some(rest) = snippet.strip_prefix("assert!(") else {
        return false;
    };

    let mut depth = 1usize;
    let mut chars = rest.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    break;
                }
            }
            '&' if depth == 1 && chars.peek() == Some(&'&') => return true,
            '|' if depth == 1 && chars.peek() == Some(&'|') => return true,
            _ => {}
        }
    }

    false
}

fn top_level_assertion_split(snippet: &str) -> Option<TopLevelAssertionSplit> {
    let trimmed = snippet.trim();
    let rest = trimmed.strip_prefix("assert!(")?;
    let rest = rest.trim_end();
    let has_trailing_semicolon = rest.ends_with(';');
    let rest = if has_trailing_semicolon {
        rest[..rest.len() - 1].trim_end()
    } else {
        rest
    };
    let body = rest.strip_suffix(')')?;
    let (op, clauses) = parse_top_level_boolean_clauses(body)?;

    Some(TopLevelAssertionSplit {
        op,
        clauses,
        has_trailing_semicolon,
    })
}

fn parse_top_level_boolean_clauses(body: &str) -> Option<(TopLevelBooleanOp, Vec<String>)> {
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut op = None;
    let mut clauses = Vec::new();
    let mut chars = body.char_indices().peekable();
    let mut in_string = false;
    let mut in_char = false;
    let mut escaped = false;

    while let Some((idx, ch)) = chars.next() {
        if in_string {
            if escaped {
                escaped = false;
            } else {
                match ch {
                    '\\' => escaped = true,
                    '"' => in_string = false,
                    _ => {}
                }
            }
            continue;
        }

        if in_char {
            if escaped {
                escaped = false;
            } else {
                match ch {
                    '\\' => escaped = true,
                    '\'' => in_char = false,
                    _ => {}
                }
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '\'' => in_char = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.checked_sub(1)?,
            ',' if depth == 0 => return None,
            '&' if depth == 0 => {
                if let Some((next_idx, '&')) = chars.peek().copied() {
                    chars.next();
                    if matches!(op, Some(TopLevelBooleanOp::Or)) {
                        return None;
                    }
                    op = Some(TopLevelBooleanOp::And);
                    clauses.push(body[start..idx].trim().to_owned());
                    start = next_idx + 1;
                }
            }
            '|' if depth == 0 => {
                if let Some((next_idx, '|')) = chars.peek().copied() {
                    chars.next();
                    if matches!(op, Some(TopLevelBooleanOp::And)) {
                        return None;
                    }
                    op = Some(TopLevelBooleanOp::Or);
                    clauses.push(body[start..idx].trim().to_owned());
                    start = next_idx + 1;
                }
            }
            _ => {}
        }
    }

    if in_string || in_char || depth != 0 {
        return None;
    }

    let op = op?;
    clauses.push(body[start..].trim().to_owned());
    if clauses.iter().any(String::is_empty) {
        return None;
    }

    Some((op, clauses))
}

fn compound_assertion_suggestion(cx: &EarlyContext<'_>, callsite: Span, snippet: &str) -> Option<LintSuggestion> {
    let split = top_level_assertion_split(snippet)?;
    if split.op != TopLevelBooleanOp::And || split.clauses.len() < 2 {
        return None;
    }

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

    Some(LintSuggestion::Multipart {
        message: "split into separate assertions".to_owned(),
        parts: vec![(callsite, replacement), (callsite.shrink_to_hi(), tail)],
        applicability: Applicability::MachineApplicable,
    })
}

impl EarlyLintPass for CompoundAssertion {
    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        // Only inspect expressions originating from macro expansion.
        if !expr.span.from_expansion() {
            return;
        }

        // Limit to expanded `if` nodes from `assert!` to avoid duplicate
        // diagnostics on children that share the same callsite.
        if !matches!(expr.kind, ExprKind::If(..)) {
            return;
        }

        let callsite = expr.span.source_callsite();
        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(callsite) else {
            return;
        };
        let trimmed = snippet.trim();

        if !trimmed.starts_with("assert!(") {
            return;
        }

        if !has_top_level_compound_assertion(trimmed) {
            return;
        }

        if let Some(suggestion) = compound_assertion_suggestion(cx, callsite, trimmed) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                COMPOUND_ASSERTION,
                callsite,
                "compound assertion should be split into separate assertions",
                "compound_assertion",
                suggestion,
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
    use super::TopLevelAssertionSplit;
    use super::TopLevelBooleanOp;
    use super::has_top_level_compound_assertion;
    use super::top_level_assertion_split;

    #[test]
    fn detects_top_level_compound_assertion() {
        assert!(has_top_level_compound_assertion("assert!(a && b)"));
        assert!(has_top_level_compound_assertion("assert!(a || b)"));
        assert!(!has_top_level_compound_assertion("assert!(a)"));
        assert!(!has_top_level_compound_assertion("assert!(items.iter().all(|x| x > 0 && x < 10))"));
        assert!(!has_top_level_compound_assertion("debug_assert!(a && b)"));
    }

    #[test]
    fn parses_machine_applicable_top_level_conjunction() {
        assert_eq!(
            top_level_assertion_split("assert!(a > 0 && b > 0);"),
            Some(TopLevelAssertionSplit {
                op: TopLevelBooleanOp::And,
                clauses: vec!["a > 0".into(), "b > 0".into()],
                has_trailing_semicolon: true,
            })
        );
    }

    #[test]
    fn parses_top_level_disjunction_without_machine_applicable_fix() {
        assert_eq!(
            top_level_assertion_split("assert!(a > 0 || b > 0)"),
            Some(TopLevelAssertionSplit {
                op: TopLevelBooleanOp::Or,
                clauses: vec!["a > 0".into(), "b > 0".into()],
                has_trailing_semicolon: false,
            })
        );
    }

    #[test]
    fn rejects_message_bearing_assertions_for_machine_applicable_fix() {
        assert_eq!(top_level_assertion_split("assert!(a > 0 && b > 0, \"boom\")"), None);
    }
}
