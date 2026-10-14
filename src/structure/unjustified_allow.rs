use rustc_ast::Attribute;
use rustc_ast::Item;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::sym;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects `#[allow(clippy::...)]` attributes without an accompanying
    /// comment explaining why the lint is suppressed. Unjustified suppressions
    /// hide real bugs — verified-logic documents every `#[allow(...)]` with
    /// a rationale.
    pub tigerstyle::UNJUSTIFIED_ALLOW,
    Allow,
    "`#[allow]` attribute without justification comment"
}

declare_lint_pass!(UnjustifiedAllow => [UNJUSTIFIED_ALLOW]);

fn is_clippy_allow_attr(attr: &Attribute) -> bool {
    if !attr.has_name(sym::allow) {
        return false;
    }
    // Only fire on #[allow(clippy::...)] — check the nested tokens
    let source = format!("{attr:?}");
    source.contains("clippy")
}

fn justification_stub(cx: &EarlyContext<'_>, attr: &Attribute) -> String {
    let source_map = cx.sess().source_map();
    let indent_width = source_map.lookup_char_pos(attr.span.lo()).col.0;
    let indent = " ".repeat(indent_width);
    format!("// TODO(octet): explain why this allow is safe here\n{indent}")
}

fn has_comment_nearby(cx: &EarlyContext<'_>, attr: &Attribute) -> bool {
    let source_map = cx.sess().source_map();

    // Get the source line of the attribute and the line before it
    let Ok(snippet) = source_map.span_to_snippet(attr.span) else {
        return true; // assume justified if we can't read source
    };

    // Check if the attribute itself has an inline comment
    if snippet.contains("//") {
        return true;
    }

    // Check the line before the attribute for a comment.
    // Use span_to_lines to get the line number, then look at the previous line.
    let Ok(lines) = source_map.span_to_lines(attr.span) else {
        return true;
    };

    if let Some(first_line) = lines.lines.first() {
        let line_idx = first_line.line_index;
        if line_idx > 0
            && let Some(prev_line) = lines.file.get_line(line_idx - 1)
        {
            let prev = prev_line.trim();
            if prev.starts_with("//") {
                return true;
            }
        }
    }

    false
}

impl EarlyLintPass for UnjustifiedAllow {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        // Skip items in #[cfg(test)] blocks
        if item.attrs.iter().any(|a| a.has_name(sym::test)) {
            return;
        }

        for attr in &item.attrs {
            if is_clippy_allow_attr(attr) && !has_comment_nearby(cx, attr) {
                emit_tiger_style_lint_with_suggestion(
                    cx,
                    UNJUSTIFIED_ALLOW,
                    attr.span,
                    "`#[allow(...)]` without justification comment",
                    "unjustified_allow",
                    LintSuggestion::Single {
                        span: attr.span.shrink_to_lo(),
                        message: "add a local justification comment".to_owned(),
                        replacement: justification_stub(cx, attr),
                        applicability: Applicability::HasPlaceholders,
                    },
                );
            }
        }
    }
}
