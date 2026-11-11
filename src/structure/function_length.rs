use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects functions and methods whose body exceeds a configurable number
    /// of non-blank, non-comment lines (default: 70). Long functions are
    /// hard to reason about and typically indicate missing decomposition.
    ///
    /// Configure in `dylint.toml`:
    /// ```toml
    /// [octet]
    /// function_length_max = 100
    /// ```
    pub tigerstyle::FUNCTION_LENGTH,
    Warn,
    "function body exceeds line limit"
}

pub struct FunctionLength {
    max_lines: usize,
    harness_test_fns: std::collections::HashSet<rustc_ast::NodeId>,
}

impl FunctionLength {
    pub fn new(max_lines: usize) -> Self {
        Self {
            max_lines,
            harness_test_fns: std::collections::HashSet::new(),
        }
    }
}

impl_lint_pass!(FunctionLength => [FUNCTION_LENGTH]);

/// Counts the code lines between a body's delimiting braces; the brace lines
/// themselves are not body lines.
fn count_body_lines(body_source: &str) -> usize {
    let inner = body_source.strip_prefix('{').and_then(|source| source.strip_suffix('}')).unwrap_or(body_source);
    count_code_lines(inner)
}

fn count_code_lines(source: &str) -> usize {
    let mut count = 0;
    let mut in_block_comment = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if in_block_comment {
            if let Some(pos) = trimmed.find("*/") {
                in_block_comment = false;
                let after = trimmed[pos + 2..].trim();
                if !after.is_empty() && !after.starts_with("//") {
                    count += 1;
                }
            }
            continue;
        }

        if trimmed.is_empty() {
            continue;
        }

        if trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("/*") {
            if !trimmed.contains("*/") {
                in_block_comment = true;
            }
            continue;
        }

        count += 1;
    }

    count
}

impl FunctionLength {
    fn check_function(&self, cx: &EarlyContext<'_>, id: rustc_ast::NodeId, func: &rustc_ast::Fn) {
        if self.harness_test_fns.contains(&id) {
            return;
        }

        let Some(body) = &func.body else {
            return;
        };

        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(body.span) else {
            return;
        };

        let code_lines = count_body_lines(&snippet);
        let max = self.max_lines;

        if code_lines > max {
            let name = func.ident.name;
            emit_tiger_style_lint(
                cx,
                FUNCTION_LENGTH,
                body.span,
                format!("function `{name}` is {code_lines} lines (limit: {max})"),
                "function_length",
            );
        }
    }
}

impl EarlyLintPass for FunctionLength {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &rustc_ast::Crate) {
        self.harness_test_fns = crate::test_context::ast_harness_test_fns(cx.sess(), krate);
    }

    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if let ItemKind::Fn(func) = &item.kind {
            self.check_function(cx, item.id, func);
        }
    }

    fn check_impl_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let rustc_ast::AssocItemKind::Fn(func) = &item.kind {
            self.check_function(cx, item.id, func);
        }
    }

    fn check_trait_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let rustc_ast::AssocItemKind::Fn(func) = &item.kind {
            self.check_function(cx, item.id, func);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::count_body_lines;

    #[test]
    fn body_line_count_excludes_delimiting_braces() {
        assert_eq!(count_body_lines("{\n    let a = 1;\n    a\n}"), 2);
        assert_eq!(count_body_lines("{}"), 0);
    }

    #[test]
    fn body_line_count_keeps_code_sharing_a_brace_line() {
        assert_eq!(count_body_lines("{ 1 }"), 1);
        assert_eq!(count_body_lines("{ let a = 1;\n    a }"), 2);
    }
}
