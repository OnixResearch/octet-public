use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects direct calls to `Instant::now()`, `SystemTime::now()`,
    /// and `Utc::now()` (chrono) in production code. Reading the system
    /// clock is a side effect — core/pure crates should receive time as
    /// a parameter so the logic stays deterministic and testable.
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

/// Known clock-reading paths (final segment `now` on these types).
const CLOCK_TYPES: &[&str] = &[
    "Instant",
    "SystemTime",
    "Utc",   // chrono::Utc::now()
    "Local", // chrono::Local::now()
];

fn is_in_test_context(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    use rustc_span::sym;
    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        if let hir::Node::Item(item) = node {
            let attrs = cx.tcx.hir_attrs(item.hir_id());
            if attrs.iter().any(|a| a.has_name(sym::test)) {
                return true;
            }
            if let hir::ItemKind::Mod(..) = &item.kind {
                for attr in attrs {
                    if attr.has_name(sym::cfg) {
                        let source = cx.sess().source_map().span_to_snippet(attr.span()).unwrap_or_default();
                        if source.contains("test") {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Check if a resolved path ends with `SomeClockType::now`.
fn is_clock_now_path(qpath: &hir::QPath<'_>) -> bool {
    match qpath {
        hir::QPath::Resolved(_, path) => {
            let segments: Vec<&str> = path.segments.iter().map(|s| s.ident.name.as_str()).collect();
            // Match patterns like `Instant::now`, `std::time::Instant::now`
            if segments.last() == Some(&"now")
                && let Some(type_seg) = segments.iter().rev().nth(1)
            {
                return CLOCK_TYPES.contains(type_seg);
            }
            false
        }
        hir::QPath::TypeRelative(ty, segment) => {
            if segment.ident.name.as_str() != "now" {
                return false;
            }
            // Check if the type is one of our clock types
            if let hir::TyKind::Path(ref ty_qpath) = ty.kind {
                let type_name = match ty_qpath {
                    hir::QPath::Resolved(_, path) => path.segments.last().map(|segment| segment.ident.name.as_str()),
                    hir::QPath::TypeRelative(..) => None,
                };
                if let Some(name) = type_name {
                    return CLOCK_TYPES.contains(&name);
                }
            }
            false
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for AmbientClock {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        // Match function-style calls: `Instant::now()`, `Utc::now()`
        if let ExprKind::Call(func, _) = &expr.kind
            && let ExprKind::Path(ref qpath) = func.kind
            && is_clock_now_path(qpath)
            && !is_in_test_context(cx, expr)
        {
            emit(cx, expr);
            return;
        }

        // Match method calls where we can resolve the impl type:
        // `chrono::Utc::now()` sometimes desugars to a method call
        if let ExprKind::MethodCall(segment, receiver, _, _) = &expr.kind
            && segment.ident.name.as_str() == "now"
            && let ty::Adt(adt, _) = cx.typeck_results().expr_ty(receiver).kind()
        {
            let name = cx.tcx.item_name(adt.did());
            let type_name = name.as_str();
            if CLOCK_TYPES.contains(&type_name) && !is_in_test_context(cx, expr) {
                emit(cx, expr);
            }
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
