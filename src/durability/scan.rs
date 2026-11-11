//! Bounded HIR scanning shared by the durability lints.
//!
//! The scanner resolves synchronization, configured commit, adapter, and flush
//! calls by resolved definition path and classifies how the immediate caller
//! consumes their `Result`. Unknown shapes stay unanalyzed instead of becoming
//! durability claims.

use octet_lint_metadata::durability::DurabilityBoundary;
use octet_lint_metadata::durability::DurabilityCatalog;
use rustc_hir as hir;
use rustc_hir::PatKind;
use rustc_hir::def::CtorKind;
use rustc_hir::def::DefKind;
use rustc_hir::def::Res;
use rustc_hir::intravisit;
use rustc_hir::intravisit::Visitor;
use rustc_lint::LateContext;
use rustc_lint::LintContext;
use rustc_middle::hir::nested_filter::OnlyBodies;
use rustc_middle::ty;
use rustc_span::Span;
use rustc_span::sym;

/// The call surfaces the durability family distinguishes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum SurfaceKind {
    Synchronization,
    Commit,
    Adapter,
    Flush,
}

impl SurfaceKind {
    pub(crate) fn is_observation(self) -> bool {
        matches!(self, Self::Synchronization | Self::Commit | Self::Adapter)
    }
}

/// How the immediate caller consumes a call's `Result`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Handling {
    /// Failure is observed and the caller continues: recorded as uncertainty or
    /// followed by an admitted recovery operation.
    Handled,
    /// Failure is observed by propagating it out of the boundary function.
    Propagated,
    /// The result is dropped in statement position or through `let _`.
    Discarded,
    /// An explicit error arm is empty.
    EmptyError,
    /// Failure is mapped to the success type without an explicit uncertain
    /// outcome.
    SuccessMapping,
    /// The shape is outside the bounded analysis.
    Unanalyzed,
}

/// One resolved durability surface call.
#[derive(Clone, Debug)]
pub(crate) struct SurfaceCall {
    pub(crate) kind: SurfaceKind,
    pub(crate) handling: Handling,
    pub(crate) span: Span,
    pub(crate) receiver: Option<String>,
}

/// Resolves the durability surface kind for one call expression.
pub(crate) fn surface_kind(
    cx: &LateContext<'_>,
    catalog: &DurabilityCatalog,
    boundary: &DurabilityBoundary,
    expr: &hir::Expr<'_>,
) -> Option<SurfaceKind> {
    let hir::ExprKind::MethodCall(segment, receiver, _args, _) = expr.kind else {
        return None;
    };
    let resolved = resolved_surface_path(cx, expr, receiver, segment.ident.name.as_str())?;
    if catalog.is_synchronization_surface(&resolved) {
        return Some(SurfaceKind::Synchronization);
    }
    if boundary.is_commit_surface(&resolved) {
        return Some(SurfaceKind::Commit);
    }
    if boundary.is_adapter_observation(&resolved) {
        return Some(SurfaceKind::Adapter);
    }
    catalog.is_flush_surface(&resolved).then_some(SurfaceKind::Flush)
}

/// Definition path of the called method, qualified by its trait or receiver
/// type so catalog surfaces can name it exactly.
fn resolved_surface_path(
    cx: &LateContext<'_>,
    expr: &hir::Expr<'_>,
    receiver: &hir::Expr<'_>,
    method_name: &str,
) -> Option<String> {
    let def_id = cx.typeck_results().type_dependent_def_id(expr.hir_id)?;
    if let Some(trait_id) = cx.tcx.trait_of_assoc(def_id) {
        return Some(format!("{}::{method_name}", cx.tcx.def_path_str(trait_id)));
    }
    let ty::Adt(adt, _) = cx.typeck_results().expr_ty(receiver).peel_refs().kind() else {
        return None;
    };
    Some(format!("{}::{method_name}", cx.tcx.def_path_str(adt.did())))
}

/// Whether `expr` calls an admitted recovery operation by method or free
/// function name.
pub(crate) fn is_recovery_call(cx: &LateContext<'_>, catalog: &DurabilityCatalog, expr: &hir::Expr<'_>) -> bool {
    match expr.kind {
        hir::ExprKind::MethodCall(segment, _, _, _) => catalog.is_recovery_operation(segment.ident.name.as_str()),
        hir::ExprKind::Call(callee, _) => call_names_recovery(cx, catalog, callee),
        _ => false,
    }
}

/// Visible receiver key for same-receiver retry comparison.
pub(crate) fn receiver_key(cx: &LateContext<'_>, receiver: &hir::Expr<'_>) -> Option<String> {
    let snippet = cx.sess().source_map().span_to_snippet(receiver.span).ok()?;
    let compact = snippet.split_whitespace().collect::<String>();
    let compact = compact.strip_prefix("&mut").unwrap_or(&compact);
    let compact = compact.strip_prefix('&').unwrap_or(compact);
    let compact = compact.strip_prefix('*').unwrap_or(compact);
    (!compact.is_empty()).then(|| compact.to_owned())
}

/// Classifies how the caller consumes the call's `Result`.
pub(crate) fn handling<'tcx>(
    cx: &LateContext<'tcx>,
    catalog: &DurabilityCatalog,
    boundary: &DurabilityBoundary,
    expr: &'tcx hir::Expr<'tcx>,
) -> Handling {
    let mut current = expr;
    for _ in 0..8 {
        match cx.tcx.parent_hir_node(current.hir_id) {
            hir::Node::Stmt(stmt) => return statement_handling(stmt, current),
            hir::Node::LetStmt(local) => return local_handling(local, current),
            hir::Node::Expr(parent) => {
                let Some(next) = parent_step(cx, parent, current) else {
                    return parent_handling(cx, catalog, boundary, parent, current);
                };
                current = next;
            }
            _ => return Handling::Unanalyzed,
        }
    }
    Handling::Unanalyzed
}

fn parent_step<'tcx>(
    cx: &LateContext<'tcx>,
    parent: &'tcx hir::Expr<'tcx>,
    current: &hir::Expr<'_>,
) -> Option<&'tcx hir::Expr<'tcx>> {
    match parent.kind {
        hir::ExprKind::DropTemps(inner) if inner.hir_id == current.hir_id => Some(parent),
        hir::ExprKind::Block(block, _) if block.expr.is_some_and(|expr| expr.hir_id == current.hir_id) => Some(parent),
        hir::ExprKind::Let(let_expr) if let_expr.init.hir_id == current.hir_id => Some(parent),
        hir::ExprKind::MethodCall(segment, receiver, _, _)
            if receiver.hir_id == current.hir_id && segment.ident.name.as_str() == "map_err" =>
        {
            Some(parent)
        }
        hir::ExprKind::Call(callee, args)
            if args.iter().any(|argument| argument.hir_id == current.hir_id) && is_try_wrapper(cx, callee) =>
        {
            Some(parent)
        }
        _ => None,
    }
}

fn parent_handling<'tcx>(
    cx: &LateContext<'tcx>,
    catalog: &DurabilityCatalog,
    boundary: &DurabilityBoundary,
    parent: &'tcx hir::Expr<'tcx>,
    current: &hir::Expr<'_>,
) -> Handling {
    match parent.kind {
        hir::ExprKind::MethodCall(segment, receiver, _, _) if receiver.hir_id == current.hir_id => {
            method_handling(segment.ident.name.as_str())
        }
        hir::ExprKind::Match(_scrutinee, _arms, hir::MatchSource::TryDesugar(_)) => Handling::Propagated,
        hir::ExprKind::Match(scrutinee, arms, source) if scrutinee.hir_id == current.hir_id => {
            match_handling(cx, catalog, boundary, scrutinee, arms, source)
        }
        hir::ExprKind::If(condition, then_branch, else_branch) => {
            if_handling(cx, catalog, boundary, condition, then_branch, else_branch, current)
        }
        _ => Handling::Unanalyzed,
    }
}

fn is_try_wrapper(cx: &LateContext<'_>, callee: &hir::Expr<'_>) -> bool {
    let hir::ExprKind::Path(path) = callee.kind else {
        return false;
    };
    let Some(def_id) = cx.qpath_res(&path, callee.hir_id).opt_def_id() else {
        return false;
    };
    cx.get_def_path(def_id)
        .last()
        .is_some_and(|name| matches!(name.as_str(), "branch" | "into_try" | "from_residual"))
}

fn statement_handling(stmt: &hir::Stmt<'_>, current: &hir::Expr<'_>) -> Handling {
    match stmt.kind {
        hir::StmtKind::Semi(expr) if expr.hir_id == current.hir_id => Handling::Discarded,
        _ => Handling::Unanalyzed,
    }
}

fn local_handling(local: &hir::LetStmt<'_>, current: &hir::Expr<'_>) -> Handling {
    let initialized_here = local.init.is_some_and(|init| init.hir_id == current.hir_id);
    match local.pat.kind {
        PatKind::Wild if initialized_here => Handling::Discarded,
        _ => Handling::Unanalyzed,
    }
}

fn method_handling(method_name: &str) -> Handling {
    match method_name {
        "ok" => Handling::Discarded,
        "unwrap_or" | "unwrap_or_default" | "unwrap_or_else" => Handling::SuccessMapping,
        _ => Handling::Unanalyzed,
    }
}

fn match_handling<'tcx>(
    cx: &LateContext<'tcx>,
    catalog: &DurabilityCatalog,
    boundary: &DurabilityBoundary,
    scrutinee: &'tcx hir::Expr<'tcx>,
    arms: &'tcx [hir::Arm<'tcx>],
    source: hir::MatchSource,
) -> Handling {
    if matches!(source, hir::MatchSource::TryDesugar(_)) {
        return Handling::Propagated;
    }
    if !matches!(source, hir::MatchSource::Normal | hir::MatchSource::Postfix) {
        return Handling::Unanalyzed;
    }
    let Some(ok_type) = result_ok_type(cx, scrutinee) else {
        return Handling::Unanalyzed;
    };
    let error_arms = arms
        .iter()
        .filter(|arm| matches!(arm.pat.kind, PatKind::Wild) || pattern_is_result_error(cx, arm.pat))
        .collect::<Vec<_>>();
    if error_arms.is_empty() {
        return Handling::Unanalyzed;
    }
    let mut handling = Handling::Unanalyzed;
    for arm in error_arms {
        let arm_handling = error_body_handling(cx, catalog, boundary, arm.body, ok_type);
        if arm_handling == Handling::Handled {
            return Handling::Handled;
        }
        if arm_handling != Handling::Unanalyzed {
            handling = arm_handling;
        }
    }
    handling
}

fn if_handling<'tcx>(
    cx: &LateContext<'tcx>,
    catalog: &DurabilityCatalog,
    boundary: &DurabilityBoundary,
    condition: &'tcx hir::Expr<'tcx>,
    then_branch: &'tcx hir::Expr<'tcx>,
    else_branch: Option<&'tcx hir::Expr<'tcx>>,
    current: &hir::Expr<'_>,
) -> Handling {
    let hir::ExprKind::Let(let_expr) = condition.kind else {
        return Handling::Unanalyzed;
    };
    if condition.hir_id != current.hir_id {
        return Handling::Unanalyzed;
    }
    let Some(ok_type) = result_ok_type(cx, let_expr.init) else {
        return Handling::Unanalyzed;
    };
    if pattern_is_result_error(cx, let_expr.pat) {
        return error_body_handling(cx, catalog, boundary, then_branch, ok_type);
    }
    let Some(else_branch) = else_branch else {
        return Handling::Unanalyzed;
    };
    error_body_handling(cx, catalog, boundary, else_branch, ok_type)
}

fn error_body_handling<'tcx>(
    cx: &LateContext<'tcx>,
    catalog: &DurabilityCatalog,
    boundary: &DurabilityBoundary,
    body: &'tcx hir::Expr<'tcx>,
    ok_type: ty::Ty<'tcx>,
) -> Handling {
    let body = peel_transparent(body);
    if is_unit_expr(body) {
        return Handling::EmptyError;
    }
    if contains_uncertainty_or_recovery(cx, catalog, boundary, body) {
        return Handling::Handled;
    }
    if is_ok_constructor(cx, body) || cx.typeck_results().expr_ty(body) == ok_type {
        return Handling::SuccessMapping;
    }
    Handling::Handled
}

fn is_ok_constructor(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let hir::ExprKind::Call(callee, _) = expr.kind else {
        return false;
    };
    let hir::ExprKind::Path(path) = callee.kind else {
        return false;
    };
    let Res::Def(DefKind::Ctor(_, CtorKind::Fn), ctor_id) = cx.qpath_res(&path, callee.hir_id) else {
        return false;
    };
    let variant_id = cx.tcx.parent(ctor_id);
    let adt_id = cx.tcx.parent(variant_id);
    cx.tcx.is_diagnostic_item(sym::Result, adt_id)
        && cx.tcx.adt_def(adt_id).variant_index_with_id(variant_id).as_usize() == 0
}

fn result_ok_type<'tcx>(cx: &LateContext<'tcx>, scrutinee: &hir::Expr<'_>) -> Option<ty::Ty<'tcx>> {
    let ty::Adt(adt, args) = cx.typeck_results().expr_ty(scrutinee).kind() else {
        return None;
    };
    cx.tcx.is_diagnostic_item(sym::Result, adt.did()).then(|| args.type_at(0))
}

fn pattern_is_result_error(cx: &LateContext<'_>, pat: &hir::Pat<'_>) -> bool {
    let PatKind::TupleStruct(path, _, _) = pat.kind else {
        return false;
    };
    let Res::Def(DefKind::Ctor(_, CtorKind::Fn), ctor_id) = cx.qpath_res(&path, pat.hir_id) else {
        return false;
    };
    let variant_id = cx.tcx.parent(ctor_id);
    if cx.tcx.def_kind(variant_id) != DefKind::Variant {
        return false;
    }
    let adt_id = cx.tcx.parent(variant_id);
    cx.tcx.is_diagnostic_item(sym::Result, adt_id)
        && cx.tcx.adt_def(adt_id).variant_index_with_id(variant_id).as_usize() == 1
}

fn peel_transparent<'tcx>(mut expr: &'tcx hir::Expr<'tcx>) -> &'tcx hir::Expr<'tcx> {
    loop {
        match expr.kind {
            hir::ExprKind::DropTemps(inner) => expr = inner,
            hir::ExprKind::Block(block, _) if block.stmts.is_empty() => match block.expr {
                Some(inner) => expr = inner,
                None => break expr,
            },
            _ => break expr,
        }
    }
}

fn is_unit_expr(expr: &hir::Expr<'_>) -> bool {
    match expr.kind {
        hir::ExprKind::Tup([]) => true,
        hir::ExprKind::Block(block, _) => block.stmts.is_empty() && block.expr.is_none(),
        _ => false,
    }
}

fn contains_uncertainty_or_recovery<'tcx>(
    cx: &LateContext<'tcx>,
    catalog: &DurabilityCatalog,
    boundary: &DurabilityBoundary,
    expr: &'tcx hir::Expr<'tcx>,
) -> bool {
    let mut finder = UncertaintyFinder {
        cx,
        catalog,
        boundary,
        found: false,
    };
    finder.visit_expr(expr);
    finder.found
}

/// Finds an explicit uncertain-outcome construction or admitted recovery call.
struct UncertaintyFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    catalog: &'a DurabilityCatalog,
    boundary: &'a DurabilityBoundary,
    found: bool,
}

impl<'tcx> Visitor<'tcx> for UncertaintyFinder<'_, 'tcx> {
    type NestedFilter = OnlyBodies;

    fn maybe_tcx(&mut self) -> rustc_middle::ty::TyCtxt<'tcx> {
        self.cx.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found || matches!(expr.kind, hir::ExprKind::Closure(..)) {
            return;
        }
        match expr.kind {
            hir::ExprKind::Path(path) => {
                let res = self.cx.qpath_res(&path, expr.hir_id);
                if path_names_uncertainty(self.cx, self.boundary, res) {
                    self.found = true;
                    return;
                }
            }
            hir::ExprKind::MethodCall(segment, _, _, _)
                if self.catalog.is_recovery_operation(segment.ident.name.as_str()) =>
            {
                self.found = true;
                return;
            }
            hir::ExprKind::Call(callee, _) if call_names_recovery(self.cx, self.catalog, callee) => {
                self.found = true;
                return;
            }
            _ => {}
        }
        intravisit::walk_expr(self, expr);
    }
}

fn path_names_uncertainty(cx: &LateContext<'_>, boundary: &DurabilityBoundary, res: Res) -> bool {
    let Some(def_id) = res.opt_def_id() else {
        return false;
    };
    let segments = cx.get_def_path(def_id);
    segments
        .iter()
        .any(|segment| boundary.uncertain_outcome_types.iter().any(|declared| declared == segment.as_str()))
}

fn call_names_recovery(cx: &LateContext<'_>, catalog: &DurabilityCatalog, callee: &hir::Expr<'_>) -> bool {
    let hir::ExprKind::Path(path) = callee.kind else {
        return false;
    };
    let Some(def_id) = cx.qpath_res(&path, callee.hir_id).opt_def_id() else {
        return false;
    };
    cx.get_def_path(def_id).last().is_some_and(|name| catalog.is_recovery_operation(name.as_str()))
}
