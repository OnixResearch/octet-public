use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;
use crate::test_context;

declare_tool_lint! {
    /// Detects reads of the system clock in production code: calls to
    /// `Instant::now()`, `SystemTime::now()`, and `Utc::now()`/`Local::now()`
    /// (chrono), those functions passed as values (`or_insert_with(Instant::now)`),
    /// and `Instant::elapsed`/`SystemTime::elapsed`, which read the clock
    /// internally. Reading the system clock is a side effect — core/pure
    /// crates should receive time as a parameter so the logic stays
    /// deterministic and testable.
    ///
    /// Boundary crates (e.g. a web shell or CLI entry point) that
    /// legitimately own the clock can suppress this with:
    ///
    /// ```rust
    /// #[allow(tigerstyle::ambient_clock, reason = "entry-point owns time capture")]
    /// ```
    pub tigerstyle::AMBIENT_CLOCK,
    Warn,
    "direct system clock read — inject time as a parameter instead"
}

declare_lint_pass!(AmbientClock => [AMBIENT_CLOCK]);

/// Known clock types; a call to their associated `now` reads the clock.
const CLOCK_TYPES: &[&str] = &[
    "Instant",
    "SystemTime",
    "Utc",   // chrono::Utc::now()
    "Local", // chrono::Local::now()
];

/// Clock types whose `elapsed` method reads the clock (`now() - self`).
const ELAPSED_CLOCK_TYPES: &[&str] = &["Instant", "SystemTime"];

/// Whether `def_id` is `<ClockType>::now` or `<Instant|SystemTime>::elapsed`.
/// The match uses the definition path rather than the spelled path, so
/// renamed imports (`use Instant as Clock`) and type aliases
/// (`type Clock = Instant`) are recognized.
fn is_clock_read_def(cx: &LateContext<'_>, def_id: hir::def_id::DefId) -> bool {
    let def_path = cx.get_def_path(def_id);
    let [.., owner, method] = def_path.as_slice() else {
        return false;
    };
    let owner = owner.as_str();
    match method.as_str() {
        "now" => CLOCK_TYPES.contains(&owner),
        "elapsed" => ELAPSED_CLOCK_TYPES.contains(&owner),
        _ => false,
    }
}

/// Whether `expr` is a path resolving to a clock read (see [`is_clock_read_def`]).
fn is_clock_read_path(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let ExprKind::Path(qpath) = &expr.kind else {
        return false;
    };
    let hir::def::Res::Def(_, def_id) = cx.typeck_results().qpath_res(qpath, expr.hir_id) else {
        return false;
    };
    is_clock_read_def(cx, def_id)
}

/// Whether `expr` is the callee of a call expression; the call itself is
/// reported instead.
fn is_call_callee(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    matches!(
        cx.tcx.parent_hir_node(expr.hir_id),
        hir::Node::Expr(hir::Expr { kind: ExprKind::Call(callee, _), .. }) if callee.hir_id == expr.hir_id
    )
}

/// Whether a method call reads the clock: `started.elapsed()` resolves to
/// `Instant::elapsed`, or a `now` method is called on a clock-typed receiver.
fn is_clock_read_method_call(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let ExprKind::MethodCall(segment, receiver, _, _) = &expr.kind else {
        return false;
    };
    if cx
        .typeck_results()
        .type_dependent_def_id(expr.hir_id)
        .is_some_and(|def_id| is_clock_read_def(cx, def_id))
    {
        return true;
    }
    segment.ident.name.as_str() == "now"
        && matches!(
            cx.typeck_results().expr_ty(receiver).kind(),
            ty::Adt(adt, _) if CLOCK_TYPES.contains(&cx.tcx.item_name(adt.did()).as_str())
        )
}

impl<'tcx> LateLintPass<'tcx> for AmbientClock {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let reads_clock = match &expr.kind {
            // `Instant::now()`, `Instant::elapsed(&started)`
            ExprKind::Call(callee, _) => is_clock_read_path(cx, callee),
            // `or_insert_with(Instant::now)`: the function value reads the
            // clock wherever it is called.
            ExprKind::Path(_) => !is_call_callee(cx, expr) && is_clock_read_path(cx, expr),
            // `started.elapsed()`
            ExprKind::MethodCall(..) => is_clock_read_method_call(cx, expr),
            _ => false,
        };
        if reads_clock && !test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span) {
            emit(cx, expr);
        }
    }
}

fn emit(cx: &LateContext<'_>, expr: &hir::Expr<'_>) {
    let suggestion = LintSuggestion::Single {
        span: expr.span,
        message: "inject time as a parameter instead of reading the system clock".to_owned(),
        replacement: "/* inject time parameter */".to_owned(),
        applicability: Applicability::HasPlaceholders,
    };
    emit_tiger_style_lint_with_suggestion(
        cx,
        AMBIENT_CLOCK,
        expr.span,
        "direct system clock read in production code",
        "ambient_clock",
        suggestion,
    );
}
