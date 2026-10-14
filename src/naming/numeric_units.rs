use rustc_ast::Local;
use rustc_ast::PatKind;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects numeric variable bindings whose names contain time or size
    /// words (timeout, delay, size, length, etc.) without a unit suffix
    /// (_ms, _secs, _bytes, etc.). Bare names create ambiguity about the
    /// unit of measure.
    pub tigerstyle::NUMERIC_UNITS,
    Allow,
    "numeric variable name missing unit suffix"
}

declare_lint_pass!(NumericUnits => [NUMERIC_UNITS]);

/// Words that imply a measurable quantity.
const QUANTITY_WORDS: &[&str] = &[
    "timeout", "delay", "duration", "interval", "latency", "size", "length", "capacity", "limit", "offset", "port",
];

/// Unit suffixes that resolve ambiguity.
const UNIT_SUFFIXES: &[&str] = &[
    "_ms", "_millis", "_secs", "_sec", "_us", "_micros", "_ns", "_nanos", "_bytes", "_kb", "_mb", "_gb",
];

/// Names that are common loop/index variables, not quantities.
const SKIP_NAMES: &[&str] = &["i", "j", "k", "n", "count", "index", "idx", "len"];

fn has_quantity_word(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    QUANTITY_WORDS.iter().any(|w| lower.contains(w))
}

fn has_unit_suffix(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    UNIT_SUFFIXES.iter().any(|s| lower.ends_with(s))
}

fn should_skip(name: &str) -> bool {
    SKIP_NAMES.contains(&name)
}

fn placeholder_unit_name(name: &str) -> String {
    format!("{name}_<unit>")
}

impl EarlyLintPass for NumericUnits {
    fn check_local(&mut self, cx: &EarlyContext<'_>, local: &Local) {
        let PatKind::Ident(_, ident, _) = &local.pat.kind else {
            return;
        };

        let name = ident.name.as_str();

        if should_skip(name) {
            return;
        }

        if has_quantity_word(name) && !has_unit_suffix(name) {
            let placeholder = placeholder_unit_name(name);
            emit_tiger_style_lint_with_suggestion(
                cx,
                NUMERIC_UNITS,
                ident.span,
                format!("`{name}` looks like a quantity but has no unit suffix"),
                "numeric_units",
                LintSuggestion::Single {
                    span: ident.span,
                    message: "rename with an explicit unit suffix like `_ms`, `_secs`, or `_bytes`".to_owned(),
                    replacement: placeholder,
                    applicability: Applicability::HasPlaceholders,
                },
            );
        }
    }
}
