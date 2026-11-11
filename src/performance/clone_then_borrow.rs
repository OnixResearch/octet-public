use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::def::Res;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::walk_expr;
use rustc_hir_typeck::expr_use_visitor::Delegate;
use rustc_hir_typeck::expr_use_visitor::ExprUseVisitor;
use rustc_hir_typeck::expr_use_visitor::PlaceBase;
use rustc_hir_typeck::expr_use_visitor::PlaceWithHirId;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::hir::nested_filter::OnlyBodies;
use rustc_middle::mir::FakeReadCause;
use rustc_middle::ty;
use rustc_middle::ty::adjustment::Adjust;
use rustc_middle::ty::adjustment::AutoBorrow;
use rustc_middle::ty::adjustment::AutoBorrowMutability;
use rustc_middle::ty::adjustment::DerefAdjustKind;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

const LINT_NAME: &str = "clone_then_borrow";

declare_tool_lint! {
    /// Detects cloning a value and immediately taking a shared borrow of the clone.
    /// Tiger Style treats clone-to-satisfy-borrow-checker patterns as performance
    /// and ownership-clarity smells when no owned value crosses a boundary.
    pub tigerstyle::CLONE_THEN_BORROW,
    Warn,
    "clone immediately borrowed instead of borrowing the original value"
}

declare_lint_pass!(CloneThenBorrow => [CLONE_THEN_BORROW]);

fn peel_transparent_expr<'tcx>(mut expr: &'tcx hir::Expr<'tcx>) -> &'tcx hir::Expr<'tcx> {
    loop {
        match &expr.kind {
            hir::ExprKind::DropTemps(inner) | hir::ExprKind::AddrOf(_, _, inner) => expr = inner,
            hir::ExprKind::Block(block, _) if block.stmts.is_empty() => {
                let Some(inner) = block.expr else {
                    break expr;
                };
                expr = inner;
            }
            _ => break expr,
        }
    }
}

fn def_path_looks_like_clone(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    let Some(def_id) = cx.typeck_results().type_dependent_def_id(expr.hir_id) else {
        return false;
    };
    let def_path = cx.get_def_path(def_id);
    def_path.last().is_some_and(|segment| *segment == Symbol::intern("clone"))
        && def_path.iter().any(|segment| *segment == Symbol::intern("Clone"))
}

fn borrowed_clone_receiver<'tcx>(cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    if expr.span.from_expansion() {
        return None;
    }

    let hir::ExprKind::AddrOf(_, hir::Mutability::Not, borrowed) = &expr.kind else {
        return None;
    };
    let borrowed = peel_transparent_expr(borrowed);
    if borrowed.span.from_expansion() {
        return None;
    }

    let hir::ExprKind::MethodCall(segment, receiver, args, _) = &borrowed.kind else {
        return None;
    };
    if segment.ident.name.as_str() != "clone" || !args.is_empty() || !def_path_looks_like_clone(cx, borrowed) {
        return None;
    }

    Some(*receiver)
}

fn borrow_original_replacement(cx: &LateContext<'_>, receiver: &hir::Expr<'_>, receiver_snippet: &str) -> String {
    if matches!(cx.typeck_results().expr_ty(receiver).kind(), ty::Ref(..)) {
        receiver_snippet.to_owned()
    } else {
        format!("&{receiver_snippet}")
    }
}

/// The `let` statement whose initializer stores `borrow`, directly or inside a
/// tuple, array, or struct literal, so the borrowed clone outlives the statement.
fn storing_let_stmt<'tcx>(cx: &LateContext<'tcx>, borrow: &hir::Expr<'_>) -> Option<&'tcx hir::Stmt<'tcx>> {
    let mut child = borrow.hir_id;
    let mut parents = cx.tcx.hir_parent_iter(borrow.hir_id);
    for (parent_id, node) in parents.by_ref() {
        match node {
            hir::Node::Expr(parent)
                if matches!(
                    parent.kind,
                    hir::ExprKind::Tup(..) | hir::ExprKind::Array(..) | hir::ExprKind::Struct(..)
                ) => {}
            hir::Node::ExprField(_) => {}
            hir::Node::LetStmt(local) if local.init.is_some_and(|init| init.hir_id == child) => break,
            _ => return None,
        }
        child = parent_id;
    }
    match parents.next() {
        Some((_, hir::Node::Stmt(stmt))) => Some(stmt),
        _ => None,
    }
}

/// The local variable that owns the place the clone was taken from.
fn receiver_root_local(mut receiver: &hir::Expr<'_>) -> Option<hir::HirId> {
    loop {
        match &receiver.kind {
            hir::ExprKind::Field(base, _)
            | hir::ExprKind::Index(base, _, _)
            | hir::ExprKind::Unary(hir::UnOp::Deref, base)
            | hir::ExprKind::MethodCall(_, base, _, _) => receiver = base,
            hir::ExprKind::Path(hir::QPath::Resolved(None, path)) => {
                return match path.res {
                    Res::Local(local) => Some(local),
                    _ => None,
                };
            }
            _ => return None,
        }
    }
}

fn borrows_mutably(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    cx.typeck_results().expr_adjustments(expr).iter().any(|adjustment| match adjustment.kind {
        Adjust::Borrow(AutoBorrow::Ref(mutability)) => matches!(mutability, AutoBorrowMutability::Mut { .. }),
        Adjust::Deref(DerefAdjustKind::Overloaded(deref)) => deref.mutbl == hir::Mutability::Mut,
        _ => false,
    })
}

/// Whether the place rooted at `path_expr` is assigned or mutably borrowed.
fn place_is_mutated(cx: &LateContext<'_>, path_expr: &hir::Expr<'_>) -> bool {
    let mut place = path_expr;
    for (_, node) in cx.tcx.hir_parent_iter(path_expr.hir_id) {
        if borrows_mutably(cx, place) {
            return true;
        }
        let hir::Node::Expr(parent) = node else {
            return false;
        };
        match &parent.kind {
            hir::ExprKind::Field(base, _)
            | hir::ExprKind::Index(base, _, _)
            | hir::ExprKind::Unary(hir::UnOp::Deref, base)
                if base.hir_id == place.hir_id =>
            {
                place = parent;
            }
            hir::ExprKind::AddrOf(_, hir::Mutability::Mut, _) => return true,
            hir::ExprKind::Assign(lhs, ..) | hir::ExprKind::AssignOp(_, lhs, _) => return lhs.hir_id == place.hir_id,
            _ => return false,
        }
    }
    false
}

/// Finds an assignment or mutable borrow of `local`, including inside closures.
struct LocalMutationFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    local: hir::HirId,
    found: bool,
}

impl<'tcx> Visitor<'tcx> for LocalMutationFinder<'_, 'tcx> {
    type NestedFilter = OnlyBodies;

    fn maybe_tcx(&mut self) -> ty::TyCtxt<'tcx> {
        self.cx.tcx
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found {
            return;
        }
        if let hir::ExprKind::Path(hir::QPath::Resolved(None, path)) = &expr.kind
            && path.res == Res::Local(self.local)
            && place_is_mutated(self.cx, expr)
        {
            self.found = true;
            return;
        }
        walk_expr(self, expr);
    }
}

/// Index of the statement in `block` that contains `hir_id`; the tail
/// expression counts as the statement after the last one.
fn block_position(cx: &LateContext<'_>, block: &hir::Block<'_>, hir_id: hir::HirId) -> Option<usize> {
    let mut child = hir_id;
    for (parent_id, _) in cx.tcx.hir_parent_iter(hir_id) {
        if parent_id == block.hir_id {
            if block.expr.is_some_and(|tail| tail.hir_id == child) {
                return Some(block.stmts.len());
            }
            return block.stmts.iter().position(|stmt| stmt.hir_id == child);
        }
        child = parent_id;
    }
    None
}

/// Finds a by-value use (move) of `local`, or of a place inside it, located in
/// `block` after the statement at `stored_at`.
struct LaterLocalMoveFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    local: hir::HirId,
    block: &'a hir::Block<'tcx>,
    stored_at: usize,
    found: bool,
}

impl<'tcx> Delegate<'tcx> for LaterLocalMoveFinder<'_, 'tcx> {
    fn consume(&mut self, place_with_id: &PlaceWithHirId<'tcx>, diag_expr_id: hir::HirId) {
        let moved_local = match place_with_id.place.base {
            PlaceBase::Local(local) => local,
            PlaceBase::Upvar(upvar) => upvar.var_path.hir_id,
            PlaceBase::Rvalue | PlaceBase::StaticItem => return,
        };
        if moved_local == self.local
            && block_position(self.cx, self.block, diag_expr_id).is_some_and(|position| position > self.stored_at)
        {
            self.found = true;
        }
    }

    fn use_cloned(&mut self, _place_with_id: &PlaceWithHirId<'tcx>, _diag_expr_id: hir::HirId) {}

    fn borrow(&mut self, _place_with_id: &PlaceWithHirId<'tcx>, _diag_expr_id: hir::HirId, _kind: ty::BorrowKind) {}

    fn mutate(&mut self, _assignee_place: &PlaceWithHirId<'tcx>, _diag_expr_id: hir::HirId) {}

    fn fake_read(&mut self, _place_with_id: &PlaceWithHirId<'tcx>, _cause: FakeReadCause, _diag_expr_id: hir::HirId) {}
}

fn local_moved_after<'tcx>(
    cx: &LateContext<'tcx>,
    local: hir::HirId,
    block: &hir::Block<'tcx>,
    stored_at: usize,
) -> bool {
    let Some(body_id) = cx.enclosing_body else {
        return false;
    };
    let mut finder = LaterLocalMoveFinder {
        cx,
        local,
        block,
        stored_at,
        found: false,
    };
    let Ok(()) = ExprUseVisitor::for_clippy(cx, cx.tcx.hir_body_owner_def_id(body_id), &mut finder)
        .consume_body(cx.tcx.hir_body(body_id));
    finder.found
}

/// The spec exempts clones stored for later and independently mutated or
/// detached from their source: when `let before = &x.clone();` is followed in
/// the same block by a mutation of `x` or a move of `x` (`drop(guard)`,
/// `consume(x)`), borrowing `x` instead would conflict with that use (E0502,
/// E0505).
fn stored_clone_outlives_mutation_or_move(
    cx: &LateContext<'_>,
    borrow: &hir::Expr<'_>,
    receiver: &hir::Expr<'_>,
) -> bool {
    let Some(local) = receiver_root_local(receiver) else {
        return false;
    };
    let Some(let_stmt) = storing_let_stmt(cx, borrow) else {
        return false;
    };
    let hir::Node::Block(block) = cx.tcx.parent_hir_node(let_stmt.hir_id) else {
        return false;
    };
    let Some(stored_at) = block.stmts.iter().position(|stmt| stmt.hir_id == let_stmt.hir_id) else {
        return false;
    };
    let mut finder = LocalMutationFinder {
        cx,
        local,
        found: false,
    };
    for stmt in &block.stmts[stored_at + 1..] {
        finder.visit_stmt(stmt);
    }
    if let Some(tail) = block.expr {
        finder.visit_expr(tail);
    }
    finder.found || local_moved_after(cx, local, block, stored_at)
}

/// Whether `expr` is a call result with drop glue that is dereferenced through
/// `Deref`, either by an adjustment or by the explicit `*` in `parent`.
fn derefs_droppable_call_result(cx: &LateContext<'_>, expr: &hir::Expr<'_>, parent: Option<&hir::Expr<'_>>) -> bool {
    if !matches!(expr.kind, hir::ExprKind::Call(..) | hir::ExprKind::MethodCall(..)) {
        return false;
    }
    let typeck_results = cx.typeck_results();
    if !typeck_results.expr_ty(expr).needs_drop(cx.tcx, cx.typing_env()) {
        return false;
    }
    let adjusted_deref = typeck_results
        .expr_adjustments(expr)
        .iter()
        .any(|adjustment| matches!(adjustment.kind, Adjust::Deref(DerefAdjustKind::Overloaded(_))));
    let explicit_deref = parent.is_some_and(|parent| {
        matches!(parent.kind, hir::ExprKind::Unary(hir::UnOp::Deref, _)) && typeck_results.is_method_call(parent)
    });
    adjusted_deref || explicit_deref
}

/// The spec exempts clones that detach from a lock guard: in
/// `m.lock().unwrap().clone()` the receiver chain dereferences a temporary
/// guard, and borrowing it instead would hold the lock while the borrow lives.
fn receiver_derefs_temporary_guard(cx: &LateContext<'_>, receiver: &hir::Expr<'_>) -> bool {
    let mut parent = None;
    let mut current = receiver;
    loop {
        if derefs_droppable_call_result(cx, current, parent) {
            return true;
        }
        match &current.kind {
            hir::ExprKind::Field(base, _)
            | hir::ExprKind::Index(base, _, _)
            | hir::ExprKind::Unary(hir::UnOp::Deref, base)
            | hir::ExprKind::MethodCall(_, base, _, _) => {
                parent = Some(current);
                current = base;
            }
            _ => return false,
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for CloneThenBorrow {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let Some(receiver) = borrowed_clone_receiver(cx, expr) else {
            return;
        };
        if receiver_derefs_temporary_guard(cx, receiver) || stored_clone_outlives_mutation_or_move(cx, expr, receiver) {
            return;
        }
        let Ok(receiver_snippet) = cx.sess().source_map().span_to_snippet(receiver.span) else {
            return;
        };
        let replacement = borrow_original_replacement(cx, receiver, &receiver_snippet);
        let message = "clone result is immediately borrowed; borrow the original value or use an explicit borrowed adapter instead";

        emit_tiger_style_lint_with_suggestion(
            cx,
            CLONE_THEN_BORROW,
            expr.span,
            message,
            LINT_NAME,
            LintSuggestion::Single {
                span: expr.span,
                message: "borrow the original value instead of cloning then borrowing".to_owned(),
                replacement,
                applicability: Applicability::MaybeIncorrect,
            },
        );
    }
}
