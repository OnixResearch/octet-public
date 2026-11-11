use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::DefKind;
use rustc_hir::def::Res;
use rustc_hir::def_id::DefId;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects known unbounded channel constructors from std, crossbeam, and
    /// Tokio. Tiger Style prefers explicit queue bounds so memory growth and
    /// backpressure remain reviewable.
    pub tigerstyle::UNBOUNDED_CHANNEL,
    Warn,
    "unbounded channel constructor"
}

declare_lint_pass!(UnboundedChannel => [UNBOUNDED_CHANNEL]);

fn match_def_path(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId, path: &[&str]) -> bool {
    cx.get_def_path(did).iter().copied().eq(path.iter().map(|segment| Symbol::intern(segment)))
}

fn resolved_callee_def_id(cx: &LateContext<'_>, callee: &hir::Expr<'_>) -> Option<rustc_hir::def_id::DefId> {
    let ExprKind::Path(qpath) = &callee.kind else {
        return None;
    };

    match cx.typeck_results().qpath_res(qpath, callee.hir_id) {
        Res::Def(_, did) => Some(did),
        _ => None,
    }
}

fn is_unbounded_channel_ctor(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId) -> bool {
    match_def_path(cx, did, &["std", "sync", "mpsc", "channel"])
        || match_def_path(cx, did, &["crossbeam_channel", "channel", "unbounded"])
        || match_def_path(cx, did, &["tokio", "sync", "mpsc", "unbounded", "unbounded_channel"])
        || match_def_path(cx, did, &["tokio", "sync", "mpsc", "unbounded_channel"])
}

impl<'tcx> LateLintPass<'tcx> for UnboundedChannel {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Call(callee, _) = &expr.kind else {
            return;
        };

        let Some(did) = resolved_callee_def_id(cx, callee) else {
            return;
        };
        if !is_unbounded_channel_ctor(cx, did) {
            return;
        }

        let message = "unbounded channel — use a bounded channel with explicit capacity";
        if let Some(suggestion) = bounded_suggestion(cx, expr, did) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                UNBOUNDED_CHANNEL,
                expr.span,
                message,
                "unbounded_channel",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(cx, UNBOUNDED_CHANNEL, expr.span, message, "unbounded_channel");
        }
    }
}

/// An unbounded constructor's name and the bounded sibling its public module
/// also exports.
struct BoundedCounterpart {
    unbounded_name: &'static str,
    bounded_name: &'static str,
    bounded_def_path: &'static [&'static str],
}

fn bounded_counterpart(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId) -> Option<BoundedCounterpart> {
    if match_def_path(cx, did, &["std", "sync", "mpsc", "channel"]) {
        Some(BoundedCounterpart {
            unbounded_name: "channel",
            bounded_name: "sync_channel",
            bounded_def_path: &["std", "sync", "mpsc", "sync_channel"],
        })
    } else if match_def_path(cx, did, &["crossbeam_channel", "channel", "unbounded"]) {
        Some(BoundedCounterpart {
            unbounded_name: "unbounded",
            bounded_name: "bounded",
            bounded_def_path: &["crossbeam_channel", "channel", "bounded"],
        })
    } else if match_def_path(cx, did, &["tokio", "sync", "mpsc", "unbounded", "unbounded_channel"])
        || match_def_path(cx, did, &["tokio", "sync", "mpsc", "unbounded_channel"])
    {
        Some(BoundedCounterpart {
            unbounded_name: "unbounded_channel",
            bounded_name: "channel",
            bounded_def_path: &["tokio", "sync", "mpsc", "bounded", "channel"],
        })
    } else {
        None
    }
}

/// Whether `module` exports the bounded sibling under its own name, visible from
/// `scope`, so `<path to module>::<bounded_name>` resolves there.
fn module_exports_bounded(cx: &LateContext<'_>, module: DefId, scope: DefId, counterpart: &BoundedCounterpart) -> bool {
    let children = match module.as_local() {
        Some(local) => cx.tcx.module_children_local(local),
        None => cx.tcx.module_children(module),
    };
    children.iter().any(|child| {
        child.ident.name.as_str() == counterpart.bounded_name
            && child.vis.is_accessible_from(scope, cx.tcx)
            && matches!(child.res, Res::Def(DefKind::Fn, bounded) if match_def_path(cx, bounded, counterpart.bounded_def_path))
    })
}

/// Module path of a `use <crate>::...::<ctor> [as <alias>]` import of `did` that
/// starts at an extern crate root, so it names the same module from any scope,
/// and whose module exports the bounded sibling.
fn extern_import_module_path(
    cx: &LateContext<'_>,
    did: DefId,
    scope: DefId,
    counterpart: &BoundedCounterpart,
) -> Option<String> {
    cx.tcx.hir_free_items().find_map(|item_id| {
        let hir::ItemKind::Use(path, hir::UseKind::Single(_)) = cx.tcx.hir_item(item_id).kind else {
            return None;
        };
        if path.res.value_ns != Some(Res::Def(DefKind::Fn, did)) {
            return None;
        }
        let (_, modules) = path.segments.split_last()?;
        let crate_root = modules.first()?;
        if !matches!(crate_root.res, Res::Def(DefKind::Mod, root) if root.is_crate_root() && !root.is_local()) {
            return None;
        }
        let Res::Def(DefKind::Mod, module) = modules.last()?.res else {
            return None;
        };
        if !module_exports_bounded(cx, module, scope, counterpart) {
            return None;
        }
        Some(modules.iter().map(|segment| segment.ident.as_str()).collect::<Vec<_>>().join("::"))
    })
}

fn bounded_suggestion(
    cx: &LateContext<'_>,
    expr: &hir::Expr<'_>,
    did: rustc_hir::def_id::DefId,
) -> Option<LintSuggestion> {
    let ExprKind::Call(callee, _) = &expr.kind else {
        return None;
    };
    let ExprKind::Path(hir::QPath::Resolved(None, path)) = &callee.kind else {
        return None;
    };
    let name = path.segments.last()?.ident;
    let name_span = name.span;
    if !expr.span.contains(callee.span) || !callee.span.contains(name_span) {
        return None;
    }
    let counterpart = bounded_counterpart(cx, did)?;
    let scope = cx.tcx.parent_module(expr.hir_id).to_def_id();
    let sm = cx.sess().source_map();
    // Generic args like `::<()>` follow the renamed segment and stay intact.
    let generic_args = sm.span_to_snippet(callee.span.with_lo(name_span.hi())).ok()?;

    // The written module prefix is kept only when it resolves to a module that
    // exports the bounded sibling; a bare imported name or a local re-export
    // module may not reach it.
    let written_module = path.segments.iter().rev().nth(1).and_then(|segment| match segment.res {
        Res::Def(DefKind::Mod, module) => Some(module),
        _ => None,
    });
    let bounded_path = if name.name.as_str() == counterpart.unbounded_name
        && written_module.is_some_and(|module| module_exports_bounded(cx, module, scope, &counterpart))
    {
        let prefix = sm.span_to_snippet(callee.span.with_hi(name_span.lo())).ok()?;
        format!("{prefix}{}", counterpart.bounded_name)
    } else {
        // Otherwise spell the bounded sibling through an extern-crate import of
        // the constructor; without such an import, offer no rewrite.
        let module_path = extern_import_module_path(cx, did, scope, &counterpart)?;
        format!("{module_path}::{}", counterpart.bounded_name)
    };

    // Unbounded constructors take no arguments, so the argument list becomes the
    // capacity placeholder.
    let replacement = format!("{bounded_path}{generic_args}(/* CAPACITY */)");

    Some(LintSuggestion::Single {
        span: expr.span,
        message: "use bounded channel with explicit capacity".to_owned(),
        replacement,
        applicability: Applicability::HasPlaceholders,
    })
}
