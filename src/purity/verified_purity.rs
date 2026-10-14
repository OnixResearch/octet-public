use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects async functions, IO operations, and system time calls in
    /// files under a `verified/` directory. Verified modules must be pure —
    /// no async, no IO, no system time — so they remain testable and
    /// formally verifiable.
    pub tigerstyle::VERIFIED_PURITY,
    Deny,
    "impure operation in verified module"
}

declare_lint_pass!(VerifiedPurity => [VERIFIED_PURITY]);

fn is_in_verified_module(cx: &EarlyContext<'_>, span: Span) -> bool {
    let source_map = cx.sess().source_map();
    let filename = source_map.span_to_filename(span);
    let display = filename.prefer_local_unconditionally();
    let path = format!("{display}");
    path.contains("/verified/") || path.contains("\\verified\\")
}

fn is_in_test_cfg(item: &Item) -> bool {
    item.attrs.iter().any(|attr| {
        if attr.has_name(rustc_span::sym::cfg) || attr.has_name(rustc_span::sym::cfg_attr) {
            let s = format!("{attr:?}");
            s.contains("test")
        } else {
            attr.has_name(rustc_span::sym::test)
        }
    })
}

/// Impure path prefixes that should not appear in verified modules.
const IMPURE_PATHS: &[&str] = &[
    "std::io::",
    "std::fs::",
    "std::net::",
    "std::process::",
    "std::time::SystemTime",
    "std::time::Instant",
    "tokio::",
    "async_std::",
];

impl EarlyLintPass for VerifiedPurity {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if !is_in_verified_module(cx, item.span) {
            return;
        }

        if is_in_test_cfg(item) {
            return;
        }

        // Check for async fn
        if let ItemKind::Fn(func) = &item.kind
            && func.sig.header.coroutine_kind.is_some()
        {
            emit_tiger_style_lint(
                cx,
                VERIFIED_PURITY,
                item.span,
                "async function in verified module",
                "verified_purity",
            );
        }
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        if !is_in_verified_module(cx, expr.span) {
            return;
        }

        // Check for impure path references in expressions
        if let ExprKind::Path(_, path) = &expr.kind {
            let path_str: String = path.segments.iter().map(|s| s.ident.name.as_str()).collect::<Vec<_>>().join("::");

            for impure in IMPURE_PATHS {
                if path_str.contains(impure) {
                    let msg = if path_str.contains("time") {
                        "system time access in verified module"
                    } else if path_str.contains("io") || path_str.contains("fs") {
                        "I/O operation in verified module"
                    } else if path_str.contains("net") {
                        "network operation in verified module"
                    } else if path_str.contains("process") {
                        "subprocess operation in verified module"
                    } else {
                        "impure operation in verified module"
                    };

                    emit_tiger_style_lint(cx, VERIFIED_PURITY, expr.span, msg, "verified_purity");
                    return;
                }
            }
        }
    }
}
