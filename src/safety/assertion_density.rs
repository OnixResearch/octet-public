use rustc_ast::Block;
use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_ast::StmtKind;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Warns on functions exceeding a configurable line count (default: 20)
    /// that contain fewer than a required number of assertions (default: 2).
    /// Functions without assertions lack documentation of their invariants.
    ///
    /// Configure in `dylint.toml`:
    /// ```toml
    /// [octet]
    /// assertion_density_min_lines = 30
    /// assertion_density_min_assertions = 3
    /// ```
    pub tigerstyle::ASSERTION_DENSITY,
    Allow,
    "function has low assertion density"
}

pub struct AssertionDensity {
    min_lines: usize,
    min_assertions: usize,
}

impl AssertionDensity {
    pub fn new(min_lines: usize, min_assertions: usize) -> Self {
        Self {
            min_lines,
            min_assertions,
        }
    }
}

impl_lint_pass!(AssertionDensity => [ASSERTION_DENSITY]);

fn count_code_lines(source: &str) -> usize {
    source
        .lines()
        .filter(|l| {
            let t = l.trim();
            !t.is_empty() && !t.starts_with("//")
        })
        .count()
}

const ASSERT_PATTERNS: &[&str] = &[
    "assert!(",
    "assert_eq!(",
    "assert_ne!(",
    "debug_assert!(",
    "debug_assert_eq!(",
    "debug_assert_ne!(",
];

fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut chars = source.chars().peekable();
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_block_comment {
            if ch == '*' && chars.peek() == Some(&'/') {
                let _ = chars.next();
                in_block_comment = false;
            }
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'/') {
            for next in chars.by_ref() {
                if next == '\n' {
                    out.push('\n');
                    break;
                }
            }
            continue;
        }

        if ch == '/' && chars.peek() == Some(&'*') {
            let _ = chars.next();
            in_block_comment = true;
            continue;
        }

        out.push(ch);
    }

    out
}

fn count_assertions(source: &str) -> usize {
    let stripped = strip_comments(source);
    ASSERT_PATTERNS.iter().map(|pattern| stripped.match_indices(pattern).count()).sum()
}

/// Returns true if the block is a single expression or a trivial delegation
/// like `self.inner.method(args)`.
fn is_single_expression_body(body: &Block) -> bool {
    // A block with just one expression (no statements, just the tail expr)
    if body.stmts.is_empty() {
        return true;
    }
    // A block with one statement that's an expression
    if body.stmts.len() == 1
        && let StmtKind::Expr(_) | StmtKind::Semi(_) = &body.stmts[0].kind
    {
        return true;
    }
    false
}

impl EarlyLintPass for AssertionDensity {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        let ItemKind::Fn(func) = &item.kind else {
            return;
        };

        // Skip test functions
        if item.attrs.iter().any(|a| a.has_name(sym::test)) {
            return;
        }

        // Skip #[cfg(test)] items
        if item.attrs.iter().any(|attr| {
            if attr.has_name(sym::cfg) {
                let source = format!("{attr:?}");
                source.contains("test")
            } else {
                false
            }
        }) {
            return;
        }

        let Some(body) = &func.body else {
            return;
        };

        // Skip single-expression bodies
        if is_single_expression_body(body) {
            return;
        }

        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(body.span) else {
            return;
        };

        let lines = count_code_lines(&snippet);
        if lines < self.min_lines {
            return;
        }

        let assertions = count_assertions(&snippet);
        if assertions >= self.min_assertions {
            return;
        }

        let min_a = self.min_assertions;
        emit_tiger_style_lint(
            cx,
            ASSERTION_DENSITY,
            body.span,
            format!("function `{}` has {assertions} assertion(s) in {lines} lines (need {min_a}+)", func.ident.name),
            "assertion_density",
        );
    }
}
