use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{self};
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Span;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects functions that both accept a time parameter (name matching
    /// `*_ms`, `*_secs`, `now`, `timestamp`, etc.) AND call
    /// `Instant::now()` or `SystemTime::now()` in their body. Two time
    /// sources in the same function is always a design error — verified-logic
    /// functions take time as an explicit parameter for purity and testability.
    pub tigerstyle::CONTRADICTORY_TIME,
    Warn,
    "function takes time parameter but also reads system time"
}

declare_lint_pass!(ContradictoryTime => [CONTRADICTORY_TIME]);

/// Time parameter name patterns.
const TIME_SUFFIXES: &[&str] = &["_ms", "_secs", "_us", "_ns"];
const TIME_NAMES: &[&str] = &["now", "timestamp", "current_time", "elapsed", "now_ms", "now_secs"];

fn is_time_param_name(name: &str) -> bool {
    TIME_NAMES.contains(&name) || TIME_SUFFIXES.iter().any(|s| name.ends_with(s))
}

/// Visitor that checks if a function body contains `Instant::now()` or
/// `SystemTime::now()` calls.
struct TimeCallFinder {
    found: Option<Span>,
}

fn qpath_calls_now(qpath: &hir::QPath<'_>) -> bool {
    match qpath {
        hir::QPath::Resolved(_, path) => {
            path.segments.last().is_some_and(|segment| segment.ident.name.as_str() == "now")
        }
        hir::QPath::TypeRelative(_, segment) => segment.ident.name.as_str() == "now",
    }
}

impl<'tcx> Visitor<'tcx> for TimeCallFinder {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found.is_some() {
            return;
        }

        if let ExprKind::Call(func, _) = &expr.kind
            && let ExprKind::Path(qpath) = &func.kind
            && qpath_calls_now(qpath)
        {
            self.found = Some(expr.span);
            return;
        }

        if let ExprKind::MethodCall(segment, _, _, _) = &expr.kind
            && segment.ident.name.as_str() == "now"
        {
            self.found = Some(expr.span);
            return;
        }

        intravisit::walk_expr(self, expr);
    }
}

impl<'tcx> LateLintPass<'tcx> for ContradictoryTime {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        _kind: hir::intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        _span: Span,
        _def_id: rustc_hir::def_id::LocalDefId,
    ) {
        // Check if any parameter has a time-related name
        let has_time_param = body.params.iter().any(|param| {
            if let hir::PatKind::Binding(_, _, ident, _) = &param.pat.kind {
                is_time_param_name(ident.name.as_str())
            } else {
                false
            }
        });

        if !has_time_param {
            return;
        }

        // Walk the body looking for .now() calls
        let mut finder = TimeCallFinder { found: None };
        finder.visit_expr(body.value);

        if let Some(call_span) = finder.found {
            let time_params: Vec<&str> = body
                .params
                .iter()
                .filter_map(|p| {
                    if let hir::PatKind::Binding(_, _, ident, _) = &p.pat.kind {
                        let name = ident.name.as_str();
                        if is_time_param_name(name) {
                            return Some(name);
                        }
                    }
                    None
                })
                .collect();

            let param_list = time_params.join("`, `");
            let message = format!("function takes time parameter `{param_list}` but also reads system time");

            let first_param = time_params.first().copied().unwrap_or("now");
            let suggestion = LintSuggestion::Single {
                span: call_span,
                message: format!("use the `{first_param}` parameter instead of reading system time"),
                replacement: first_param.to_owned(),
                applicability: Applicability::MaybeIncorrect,
            };
            emit_tiger_style_lint_with_suggestion(
                cx,
                CONTRADICTORY_TIME,
                call_span,
                message,
                "contradictory_time",
                suggestion,
            );
        }
    }
}
