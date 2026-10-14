use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_ast::Variant;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects type names, enum variant names, and trait names containing
    /// multi-character uppercase acronyms like `RPCHandler` or `SQLQuery`.
    /// Tiger Style uses single-word capitalization: `RpcHandler`, `SqlQuery`.
    ///
    /// Configure in `dylint.toml`:
    /// ```toml
    /// [octet]
    /// acronym_allowlist = ["IO", "ID"]
    /// ```
    pub tigerstyle::ACRONYM_STYLE,
    Warn,
    "acronym in type name should use single-word capitalization"
}

pub struct AcronymStyle {
    allowlist: Vec<String>,
}

impl AcronymStyle {
    pub fn new(allowlist: Vec<String>) -> Self {
        Self { allowlist }
    }
}

impl_lint_pass!(AcronymStyle => [ACRONYM_STYLE]);

/// Check if a name contains an acronym-style uppercase run (2+ uppercase
/// letters followed by a lowercase letter). Returns the suggested fix.
fn suggest_acronym_fix(name: &str) -> Option<String> {
    let chars: Vec<char> = name.chars().collect();

    // First pass: check if there's any issue
    let mut i = 0;
    let mut has_issue = false;
    while i < chars.len() {
        if chars[i].is_ascii_uppercase() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_uppercase() {
                i += 1;
            }
            let upper_len = i - start;
            if upper_len >= 2 && i < chars.len() && chars[i].is_ascii_lowercase() {
                has_issue = true;
                break;
            }
        } else {
            i += 1;
        }
    }

    if !has_issue {
        return None;
    }

    // Second pass: build the fixed name
    let mut result = String::with_capacity(name.len());
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_uppercase() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_uppercase() {
                i += 1;
            }
            let upper_len = i - start;

            if upper_len >= 2 && i < chars.len() && chars[i].is_ascii_lowercase() {
                // Keep first char uppercase, lowercase the acronym body,
                // keep the last uppercase char (it starts the next word)
                result.push(chars[start]);
                for c in &chars[(start + 1)..(i - 1)] {
                    result.push(c.to_ascii_lowercase());
                }
                result.push(chars[i - 1]);
            } else {
                for c in &chars[start..i] {
                    result.push(*c);
                }
            }
        } else {
            result.push(chars[i]);
            i += 1;
        }
    }

    Some(result)
}

fn extract_acronym(name: &str) -> Option<String> {
    let chars: Vec<char> = name.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i].is_ascii_uppercase() {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_uppercase() {
                i += 1;
            }
            let upper_len = i - start;
            if upper_len >= 2 && i < chars.len() && chars[i].is_ascii_lowercase() {
                return Some(chars[start..(i - 1)].iter().collect());
            }
        } else {
            i += 1;
        }
    }
    None
}

fn check_name(cx: &EarlyContext<'_>, name: &str, span: Span, allowlist: &[String]) {
    // Skip if the acronym is in the allowlist
    if let Some(acronym) = extract_acronym(name)
        && allowlist.iter().any(|a| a == &acronym)
    {
        return;
    }

    if let Some(suggested_name) = suggest_acronym_fix(name) {
        emit_tiger_style_lint_with_suggestion(
            cx,
            ACRONYM_STYLE,
            span,
            format!("`{name}` has acronym-style capitalization"),
            "acronym_style",
            LintSuggestion::Single {
                span,
                message: format!("rename `{name}` to `{suggested_name}`"),
                replacement: suggested_name,
                applicability: Applicability::MaybeIncorrect,
            },
        );
    }
}

impl EarlyLintPass for AcronymStyle {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        match &item.kind {
            ItemKind::Struct(ident, ..) | ItemKind::Enum(ident, ..) => {
                check_name(cx, ident.name.as_str(), ident.span, &self.allowlist);
            }
            ItemKind::Trait(tr) => {
                check_name(cx, tr.ident.name.as_str(), tr.ident.span, &self.allowlist);
            }
            ItemKind::TyAlias(ta) => {
                check_name(cx, ta.ident.name.as_str(), ta.ident.span, &self.allowlist);
            }
            _ => {}
        }
    }

    fn check_variant(&mut self, cx: &EarlyContext<'_>, variant: &Variant) {
        check_name(cx, variant.ident.name.as_str(), variant.ident.span, &self.allowlist);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acronym_fix() {
        assert_eq!(suggest_acronym_fix("RPCHandler"), Some("RpcHandler".into()));
        assert_eq!(suggest_acronym_fix("SQLQuery"), Some("SqlQuery".into()));
        assert_eq!(suggest_acronym_fix("TLSConfig"), Some("TlsConfig".into()));
        assert_eq!(suggest_acronym_fix("HTTPSServer"), Some("HttpsServer".into()));
        assert_eq!(suggest_acronym_fix("IOError"), Some("IoError".into()));

        // Should NOT fire on these
        assert_eq!(suggest_acronym_fix("RpcHandler"), None);
        assert_eq!(suggest_acronym_fix("Config"), None);
        assert_eq!(suggest_acronym_fix("RPC"), None);
        assert_eq!(suggest_acronym_fix("ID"), None);
        assert_eq!(suggest_acronym_fix("Ok"), None);
        assert_eq!(suggest_acronym_fix("Eq"), None);
    }
}
