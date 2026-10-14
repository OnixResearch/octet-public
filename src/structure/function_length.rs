use rustc_ast::Attribute;
use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::sym;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects functions whose body exceeds a configurable number of
    /// non-blank, non-comment lines (default: 70). Long functions are
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
}

impl FunctionLength {
    pub fn new(max_lines: usize) -> Self {
        Self { max_lines }
    }
}

impl_lint_pass!(FunctionLength => [FUNCTION_LENGTH]);

fn is_test_function(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| attr.has_name(sym::test))
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

impl EarlyLintPass for FunctionLength {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        let ItemKind::Fn(func) = &item.kind else {
            return;
        };

        if is_test_function(&item.attrs) {
            return;
        }

        let Some(body) = &func.body else {
            return;
        };

        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(body.span) else {
            return;
        };

        let code_lines = count_code_lines(&snippet);
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
