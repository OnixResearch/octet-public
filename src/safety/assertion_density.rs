use rustc_ast::Block;
use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_ast::StmtKind;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Warns on functions and methods exceeding a configurable line count
    /// (default: 20) that contain fewer than a required number of assertions
    /// (default: 2). Functions without assertions lack documentation of their
    /// invariants.
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
    harness_test_fns: std::collections::HashSet<rustc_ast::NodeId>,
}

impl AssertionDensity {
    pub fn new(min_lines: usize, min_assertions: usize) -> Self {
        Self {
            min_lines,
            min_assertions,
            harness_test_fns: std::collections::HashSet::new(),
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

/// Removes comments from `source` with the Rust lexer, so `/*` or `//` inside
/// string and char literals stays code and nested block comments end where
/// rustc ends them. Each comment becomes one space to keep tokens apart.
fn strip_comments(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut cursor = rustc_lexer::Cursor::new(source, rustc_lexer::FrontmatterAllowed::No);
    loop {
        let rest = cursor.as_str();
        let token = cursor.advance_token();
        let text = rest.strip_suffix(cursor.as_str()).unwrap_or(rest);
        match token.kind {
            rustc_lexer::TokenKind::Eof => break,
            rustc_lexer::TokenKind::LineComment { .. } | rustc_lexer::TokenKind::BlockComment { .. } => out.push(' '),
            _ => out.push_str(text),
        }
    }
    out
}

/// Counts assertion macro invocations, matching each macro name only at an
/// identifier boundary so `debug_assert!(` is not also counted as `assert!(`.
fn count_assertions(source: &str) -> usize {
    let stripped = strip_comments(source);
    ASSERT_PATTERNS
        .iter()
        .map(|pattern| {
            stripped
                .match_indices(pattern)
                .filter(|(start, _)| {
                    !stripped[..*start].chars().next_back().is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
                })
                .count()
        })
        .sum()
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

impl AssertionDensity {
    fn check_function(&self, cx: &EarlyContext<'_>, attrs: &[rustc_ast::Attribute], func: &rustc_ast::Fn) {
        // Skip items whose `cfg` can only hold under `cfg(test)`
        if crate::test_context::ast_attrs_indicate_test(cx.sess(), attrs) {
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

impl EarlyLintPass for AssertionDensity {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &rustc_ast::Crate) {
        self.harness_test_fns = crate::test_context::ast_harness_test_fns(cx.sess(), krate);
    }

    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // `#[test]` functions keep no attribute after expansion; they are found
        // through the harness markers instead.
        if let ItemKind::Fn(func) = &item.kind
            && !self.harness_test_fns.contains(&item.id)
        {
            self.check_function(cx, &item.attrs, func);
        }
    }

    fn check_impl_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let rustc_ast::AssocItemKind::Fn(func) = &item.kind {
            self.check_function(cx, &item.attrs, func);
        }
    }

    fn check_trait_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let rustc_ast::AssocItemKind::Fn(func) = &item.kind {
            self.check_function(cx, &item.attrs, func);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::count_assertions;

    #[test]
    fn debug_assertions_count_once() {
        assert_eq!(count_assertions("{ debug_assert!(a > 0); }"), 1);
        assert_eq!(count_assertions("{ debug_assert_eq!(a, b); debug_assert_ne!(a, c); }"), 2);
    }

    #[test]
    fn plain_and_path_qualified_assertions_count() {
        assert_eq!(count_assertions("{ assert!(a); std::assert_eq!(a, b); assert_ne!(a, c); }"), 3);
        assert_eq!(count_assertions("{ my_assert!(a); }"), 0);
    }

    #[test]
    fn comment_markers_in_literals_are_code() {
        assert_eq!(count_assertions(r#"{ let s = "/*"; assert!(a); assert!(b); }"#), 2);
        assert_eq!(count_assertions(r#"{ let s = r"//"; let c = '"'; assert!(a); }"#), 1);
        assert_eq!(count_assertions("{ /* assert!(a); */ assert!(b); // assert!(c);\n }"), 1);
        assert_eq!(count_assertions("{ /* outer /* nested */ assert!(a); */ assert!(b); }"), 1);
    }
}
