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

/// Def paths of the system-clock reads this lint recognizes.
const SYSTEM_CLOCK_NOW_PATHS: &[&[&str]] = &[&["std", "time", "Instant", "now"], &[
    "std",
    "time",
    "SystemTime",
    "now",
]];

/// Visitor that checks if a function body, including closures and `async`
/// bodies nested in it, contains `Instant::now()` or `SystemTime::now()` calls.
struct TimeCallFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    /// Typeck results of the typeck root, which also cover every nested body.
    typeck: &'tcx rustc_middle::ty::TypeckResults<'tcx>,
    found: Option<Span>,
}

fn calls_system_clock_now(
    cx: &LateContext<'_>,
    typeck: &rustc_middle::ty::TypeckResults<'_>,
    callee: &hir::Expr<'_>,
) -> bool {
    let ExprKind::Path(qpath) = &callee.kind else {
        return false;
    };
    let hir::def::Res::Def(_, did) = typeck.qpath_res(qpath, callee.hir_id) else {
        return false;
    };
    let def_path = cx.get_def_path(did);
    SYSTEM_CLOCK_NOW_PATHS
        .iter()
        .any(|path| def_path.iter().copied().eq(path.iter().map(|segment| rustc_span::Symbol::intern(segment))))
}

impl<'tcx> Visitor<'tcx> for TimeCallFinder<'_, 'tcx> {
    // `async fn` bodies lower to nested coroutine bodies and closures are
    // nested bodies too; a clock read in either is still this function's.
    type NestedFilter = rustc_middle::hir::nested_filter::OnlyBodies;

    fn maybe_tcx(&mut self) -> Self::MaybeTyCtxt {
        self.cx.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found.is_some() {
            return;
        }

        if let ExprKind::Call(func, _) = &expr.kind
            && calls_system_clock_now(self.cx, self.typeck, func)
        {
            self.found = Some(expr.span);
            return;
        }

        intravisit::walk_expr(self, expr);
    }
}

/// Names of the parameters of `body` that carry a time value.
fn temporal_param_names<'a>(body: &'a hir::Body<'_>) -> impl Iterator<Item = &'a str> {
    body.params.iter().filter_map(|param| {
        if let hir::PatKind::Binding(_, _, ident, _) = &param.pat.kind
            && is_time_param_name(ident.name.as_str())
        {
            return Some(ident.name.as_str());
        }
        None
    })
}

impl<'tcx> LateLintPass<'tcx> for ContradictoryTime {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: hir::intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        _span: Span,
        def_id: rustc_hir::def_id::LocalDefId,
    ) {
        let time_params: Vec<&str> = temporal_param_names(body).collect();
        if time_params.is_empty() {
            return;
        }

        // An enclosing function that takes a time parameter already searches
        // this closure's body; reporting here too would duplicate the finding.
        let root = cx.tcx.typeck_root_def_id(def_id.to_def_id()).expect_local();
        if matches!(kind, hir::intravisit::FnKind::Closure)
            && root != def_id
            && temporal_param_names(cx.tcx.hir_body_owned_by(root)).next().is_some()
        {
            return;
        }

        // Walk the body looking for `Instant::now()` / `SystemTime::now()` calls
        let mut finder = TimeCallFinder {
            cx,
            typeck: cx.tcx.typeck(root),
            found: None,
        };
        finder.visit_expr(body.value);

        if let Some(call_span) = finder.found {
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
