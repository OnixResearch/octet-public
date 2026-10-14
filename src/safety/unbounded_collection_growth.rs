use std::collections::HashMap;
use std::collections::HashSet;

use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::Res;
use rustc_hir::intravisit::Visitor;
use rustc_hir::intravisit::{self};
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty::Ty;
use rustc_middle::ty::{self};
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects narrow same-block cases where a local collection starts empty and
    /// then grows in a loop without a prior reservation or explicit local bound.
    /// Tiger Style prefers reviewable bounds on collection growth before a loop
    /// can keep appending forever.
    pub tigerstyle::UNBOUNDED_COLLECTION_GROWTH,
    Warn,
    "collection growth in loop without prior bound or reservation"
}

declare_lint_pass!(UnboundedCollectionGrowth => [UNBOUNDED_COLLECTION_GROWTH]);

#[derive(Clone, Copy, PartialEq, Eq)]
enum InitKind {
    Empty,
    Reserved,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MeasureKind {
    Len,
    Capacity,
}

fn match_def_path(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId, path: &[&str]) -> bool {
    cx.get_def_path(did).iter().copied().eq(path.iter().map(|segment| Symbol::intern(segment)))
}

fn path_ends_with(path: &[Symbol], suffix: &[&str]) -> bool {
    path.len() >= suffix.len()
        && path[path.len() - suffix.len()..]
            .iter()
            .copied()
            .eq(suffix.iter().map(|segment| Symbol::intern(segment)))
}

fn peel_transparent_expr<'tcx>(mut expr: &'tcx hir::Expr<'tcx>) -> &'tcx hir::Expr<'tcx> {
    loop {
        match &expr.kind {
            ExprKind::DropTemps(inner) => expr = inner,
            ExprKind::Block(block, _) if block.stmts.is_empty() => {
                let Some(inner) = block.expr else {
                    break expr;
                };
                expr = inner;
            }
            _ => break expr,
        }
    }
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

fn is_default_trait_function(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId) -> bool {
    match_def_path(cx, did, &["core", "default", "Default", "default"])
        || match_def_path(cx, did, &["std", "default", "Default", "default"])
}

fn is_supported_collection_ty(cx: &LateContext<'_>, ty: Ty<'_>) -> bool {
    let ty::Adt(adt, _) = ty.peel_refs().kind() else {
        return false;
    };
    let path = cx.get_def_path(adt.did());
    let Some(crate_name) = path.first() else {
        return false;
    };

    match crate_name.as_str() {
        "alloc" | "std" => {
            path_ends_with(&path, &["vec", "Vec"])
                || path_ends_with(&path, &["vec_deque", "VecDeque"])
                || path_ends_with(&path, &["btree", "map", "BTreeMap"])
                || path_ends_with(&path, &["hash", "map", "HashMap"])
        }
        "hashbrown" => path_ends_with(&path, &["map", "HashMap"]),
        _ => false,
    }
}

fn classify_collection_init(cx: &LateContext<'_>, local_ty: Ty<'_>, init: &hir::Expr<'_>) -> Option<InitKind> {
    if !is_supported_collection_ty(cx, local_ty) {
        return None;
    }

    let ExprKind::Call(callee, args) = &peel_transparent_expr(init).kind else {
        return None;
    };
    let did = resolved_callee_def_id(cx, callee)?;
    let def_path = cx.get_def_path(did);
    let name = def_path.last()?.as_str();

    if name == "with_capacity" && args.len() == 1 {
        Some(InitKind::Reserved)
    } else if args.is_empty() && (name == "new" || is_default_trait_function(cx, did)) {
        Some(InitKind::Empty)
    } else {
        None
    }
}

fn direct_local_hir_id(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<hir::HirId> {
    let expr = peel_transparent_expr(expr);
    let ExprKind::Path(qpath) = &expr.kind else {
        return None;
    };

    match cx.typeck_results().qpath_res(qpath, expr.hir_id) {
        Res::Local(local_id) => Some(local_id),
        _ => None,
    }
}

fn is_growth_method(name: &str) -> bool {
    matches!(name, "push" | "insert" | "extend")
}

fn is_reservation_method(name: &str) -> bool {
    matches!(name, "reserve" | "reserve_exact" | "try_reserve")
}

fn collection_growth_local(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<hir::HirId> {
    let expr = peel_transparent_expr(expr);
    let ExprKind::MethodCall(segment, receiver, _args, _) = &expr.kind else {
        return None;
    };
    if !is_growth_method(segment.ident.name.as_str()) {
        return None;
    }

    let local_id = direct_local_hir_id(cx, receiver)?;
    is_supported_collection_ty(cx, cx.typeck_results().expr_ty(receiver)).then_some(local_id)
}

fn measurement_kind(cx: &LateContext<'_>, expr: &hir::Expr<'_>, local_id: hir::HirId) -> Option<MeasureKind> {
    let expr = peel_transparent_expr(expr);
    let ExprKind::MethodCall(segment, receiver, args, _) = &expr.kind else {
        return None;
    };
    if !args.is_empty() || direct_local_hir_id(cx, receiver) != Some(local_id) {
        return None;
    }

    match segment.ident.name.as_str() {
        "len" => Some(MeasureKind::Len),
        "capacity" => Some(MeasureKind::Capacity),
        _ => None,
    }
}

fn comparison_allows_growth(
    cx: &LateContext<'_>,
    op: hir::BinOpKind,
    lhs: &hir::Expr<'_>,
    rhs: &hir::Expr<'_>,
    local_id: hir::HirId,
) -> bool {
    matches!(measurement_kind(cx, lhs, local_id), Some(MeasureKind::Len))
        && matches!(op, hir::BinOpKind::Lt | hir::BinOpKind::Le)
        || matches!(measurement_kind(cx, rhs, local_id), Some(MeasureKind::Len))
            && matches!(op, hir::BinOpKind::Gt | hir::BinOpKind::Ge)
}

fn comparison_blocks_growth(
    cx: &LateContext<'_>,
    op: hir::BinOpKind,
    lhs: &hir::Expr<'_>,
    rhs: &hir::Expr<'_>,
    local_id: hir::HirId,
) -> bool {
    matches!(measurement_kind(cx, lhs, local_id), Some(MeasureKind::Len))
        && matches!(op, hir::BinOpKind::Gt | hir::BinOpKind::Ge)
        || matches!(measurement_kind(cx, rhs, local_id), Some(MeasureKind::Len))
            && matches!(op, hir::BinOpKind::Lt | hir::BinOpKind::Le)
        || matches!(measurement_kind(cx, lhs, local_id), Some(MeasureKind::Len))
            && matches!(measurement_kind(cx, rhs, local_id), Some(MeasureKind::Capacity))
            && matches!(op, hir::BinOpKind::Eq)
        || matches!(measurement_kind(cx, lhs, local_id), Some(MeasureKind::Capacity))
            && matches!(measurement_kind(cx, rhs, local_id), Some(MeasureKind::Len))
            && matches!(op, hir::BinOpKind::Eq)
}

fn condition_allows_growth(cx: &LateContext<'_>, cond: &hir::Expr<'_>, local_id: hir::HirId) -> bool {
    let cond = peel_transparent_expr(cond);
    match &cond.kind {
        ExprKind::Binary(op, lhs, rhs) => match op.node {
            hir::BinOpKind::And => {
                condition_allows_growth(cx, lhs, local_id) || condition_allows_growth(cx, rhs, local_id)
            }
            hir::BinOpKind::Or => {
                condition_allows_growth(cx, lhs, local_id) && condition_allows_growth(cx, rhs, local_id)
            }
            _ => comparison_allows_growth(cx, op.node, lhs, rhs, local_id),
        },
        ExprKind::Unary(hir::UnOp::Not, inner) => condition_blocks_growth(cx, inner, local_id),
        _ => false,
    }
}

fn condition_blocks_growth(cx: &LateContext<'_>, cond: &hir::Expr<'_>, local_id: hir::HirId) -> bool {
    let cond = peel_transparent_expr(cond);
    match &cond.kind {
        ExprKind::Binary(op, lhs, rhs) => match op.node {
            hir::BinOpKind::And => {
                condition_blocks_growth(cx, lhs, local_id) || condition_blocks_growth(cx, rhs, local_id)
            }
            hir::BinOpKind::Or => {
                condition_blocks_growth(cx, lhs, local_id) && condition_blocks_growth(cx, rhs, local_id)
            }
            _ => comparison_blocks_growth(cx, op.node, lhs, rhs, local_id),
        },
        ExprKind::Unary(hir::UnOp::Not, inner) => condition_allows_growth(cx, inner, local_id),
        _ => false,
    }
}

fn stmt_expr<'tcx>(stmt: &'tcx hir::Stmt<'tcx>) -> Option<&'tcx hir::Expr<'tcx>> {
    match stmt.kind {
        hir::StmtKind::Expr(expr) | hir::StmtKind::Semi(expr) => Some(expr),
        _ => None,
    }
}

fn expr_definitely_exits(expr: &hir::Expr<'_>) -> bool {
    let expr = peel_transparent_expr(expr);
    match &expr.kind {
        ExprKind::Break(_, _) | ExprKind::Continue(_) | ExprKind::Ret(_) => true,
        ExprKind::Block(block, _) => block_definitely_exits(block),
        _ => false,
    }
}

fn block_definitely_exits(block: &hir::Block<'_>) -> bool {
    if let Some(expr) = block.expr {
        return expr_definitely_exits(expr);
    }

    if block.stmts.len() != 1 {
        return false;
    }

    let Some(expr) = stmt_expr(&block.stmts[0]) else {
        return false;
    };
    expr_definitely_exits(expr)
}

fn direct_break_guard_locals(
    cx: &LateContext<'_>,
    expr: &hir::Expr<'_>,
    candidates: &HashSet<hir::HirId>,
) -> HashSet<hir::HirId> {
    let expr = peel_transparent_expr(expr);
    let ExprKind::If(cond, then_expr, _else_expr) = &expr.kind else {
        return HashSet::new();
    };
    let ExprKind::Block(then_block, _) = &then_expr.kind else {
        return HashSet::new();
    };
    if !block_definitely_exits(then_block) {
        return HashSet::new();
    }

    candidates.iter().copied().filter(|local_id| condition_blocks_growth(cx, cond, *local_id)).collect()
}

struct GuaranteedReservationFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    candidates: &'a HashSet<hir::HirId>,
    found: HashSet<hir::HirId>,
}

impl<'tcx> Visitor<'tcx> for GuaranteedReservationFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        let expr = peel_transparent_expr(expr);
        match &expr.kind {
            ExprKind::Closure { .. } | ExprKind::If(..) | ExprKind::Match(..) | ExprKind::Loop(..) => return,
            ExprKind::MethodCall(segment, receiver, _args, _) => {
                if is_reservation_method(segment.ident.name.as_str())
                    && let Some(local_id) = direct_local_hir_id(self.cx, receiver)
                    && self.candidates.contains(&local_id)
                {
                    self.found.insert(local_id);
                }
            }
            _ => {}
        }

        intravisit::walk_expr(self, expr);
    }
}

fn guaranteed_reservation_locals<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    candidates: &HashSet<hir::HirId>,
) -> HashSet<hir::HirId> {
    let mut finder = GuaranteedReservationFinder {
        cx,
        candidates,
        found: HashSet::new(),
    };
    finder.visit_expr(expr);
    finder.found
}

struct AssignmentFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    candidates: &'a HashSet<hir::HirId>,
    found: HashSet<hir::HirId>,
}

impl<'tcx> Visitor<'tcx> for AssignmentFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        let expr = peel_transparent_expr(expr);
        if let ExprKind::Closure { .. } = &expr.kind {
            return;
        }

        match &expr.kind {
            ExprKind::Assign(lhs, _rhs, _) | ExprKind::AssignOp(_, lhs, _rhs) => {
                if let Some(local_id) = direct_local_hir_id(self.cx, lhs)
                    && self.candidates.contains(&local_id)
                {
                    self.found.insert(local_id);
                }
            }
            _ => {}
        }

        intravisit::walk_expr(self, expr);
    }
}

fn assigned_candidate_locals<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    candidates: &HashSet<hir::HirId>,
) -> HashSet<hir::HirId> {
    let mut finder = AssignmentFinder {
        cx,
        candidates,
        found: HashSet::new(),
    };
    finder.visit_expr(expr);
    finder.found
}

fn enclosing_block<'tcx>(cx: &LateContext<'tcx>, hir_id: hir::HirId) -> Option<&'tcx hir::Block<'tcx>> {
    for (_, node) in cx.tcx.hir_parent_iter(hir_id) {
        if let hir::Node::Block(block) = node {
            return Some(block);
        }
        if matches!(node, hir::Node::Item(_) | hir::Node::ImplItem(_) | hir::Node::TraitItem(_)) {
            break;
        }
    }

    None
}

fn collect_candidate_bindings_before_loop<'tcx>(
    cx: &LateContext<'tcx>,
    block: &'tcx hir::Block<'tcx>,
    loop_expr: &'tcx hir::Expr<'tcx>,
) -> (HashSet<hir::HirId>, HashSet<hir::HirId>, HashMap<hir::HirId, rustc_span::Span>) {
    let loop_callsite = loop_expr.span.source_callsite();
    let mut candidates = HashSet::new();
    let mut reserved = HashSet::new();
    let mut init_spans = HashMap::new();

    for stmt in block.stmts {
        let stmt_callsite = stmt.span.source_callsite();
        if stmt_callsite.hi() > loop_callsite.lo() {
            break;
        }

        match stmt.kind {
            hir::StmtKind::Let(local) => {
                let Some(init) = local.init else {
                    continue;
                };
                let hir::PatKind::Binding(_, local_id, _, None) = local.pat.kind else {
                    continue;
                };
                let local_ty = cx.typeck_results().pat_ty(local.pat);
                match classify_collection_init(cx, local_ty, init) {
                    Some(InitKind::Empty) => {
                        candidates.insert(local_id);
                        init_spans.insert(local_id, init.span);
                    }
                    Some(InitKind::Reserved) => {
                        candidates.remove(&local_id);
                        reserved.remove(&local_id);
                        init_spans.remove(&local_id);
                    }
                    None => {}
                }
            }
            hir::StmtKind::Expr(expr) | hir::StmtKind::Semi(expr) => {
                let reservation_hits = guaranteed_reservation_locals(cx, expr, &candidates);
                reserved.extend(reservation_hits);

                let assignment_hits = assigned_candidate_locals(cx, expr, &candidates);
                for local_id in assignment_hits {
                    candidates.remove(&local_id);
                    reserved.remove(&local_id);
                    init_spans.remove(&local_id);
                }
            }
            hir::StmtKind::Item(_) => {}
        }
    }

    (candidates, reserved, init_spans)
}

struct LoopBodyAnalyzer<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    candidates: &'a HashSet<hir::HirId>,
    warned: &'a mut HashSet<hir::HirId>,
    guarded: HashSet<hir::HirId>,
    reserved: HashSet<hir::HirId>,
    init_spans: &'a HashMap<hir::HirId, rustc_span::Span>,
}

impl<'tcx> Visitor<'tcx> for LoopBodyAnalyzer<'_, 'tcx> {
    fn visit_block(&mut self, block: &'tcx hir::Block<'tcx>) {
        let saved_guarded = self.guarded.clone();
        let saved_reserved = self.reserved.clone();
        let mut current_guarded = saved_guarded.clone();
        let mut current_reserved = saved_reserved.clone();

        for stmt in block.stmts {
            self.guarded.clone_from(&current_guarded);
            self.reserved.clone_from(&current_reserved);
            intravisit::walk_stmt(self, stmt);

            if let Some(expr) = stmt_expr(stmt) {
                current_reserved.extend(guaranteed_reservation_locals(self.cx, expr, self.candidates));
                current_guarded.extend(direct_break_guard_locals(self.cx, expr, self.candidates));
            }
        }

        if let Some(expr) = block.expr {
            self.guarded.clone_from(&current_guarded);
            self.reserved.clone_from(&current_reserved);
            self.visit_expr(expr);
        }

        self.guarded = saved_guarded;
        self.reserved = saved_reserved;
    }

    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        let expr = peel_transparent_expr(expr);
        match &expr.kind {
            ExprKind::Closure { .. } => return,
            ExprKind::Loop(..) if !expr.span.from_expansion() => return,
            ExprKind::If(cond, then_expr, else_expr) => {
                let saved_guarded = self.guarded.clone();
                let saved_reserved = self.reserved.clone();

                let mut then_guarded = saved_guarded.clone();
                then_guarded.extend(
                    self.candidates
                        .iter()
                        .copied()
                        .filter(|local_id| condition_allows_growth(self.cx, cond, *local_id)),
                );
                self.guarded = then_guarded;
                self.reserved = saved_reserved.clone();
                self.visit_expr(then_expr);

                if let Some(else_expr) = else_expr {
                    let mut else_guarded = saved_guarded.clone();
                    else_guarded.extend(
                        self.candidates
                            .iter()
                            .copied()
                            .filter(|local_id| condition_blocks_growth(self.cx, cond, *local_id)),
                    );
                    self.guarded = else_guarded;
                    self.reserved = saved_reserved.clone();
                    self.visit_expr(else_expr);
                }

                self.guarded = saved_guarded;
                self.reserved = saved_reserved;
                return;
            }
            _ => {}
        }

        if let Some(local_id) = collection_growth_local(self.cx, expr)
            && self.candidates.contains(&local_id)
            && !self.guarded.contains(&local_id)
            && !self.reserved.contains(&local_id)
            && self.warned.insert(local_id)
        {
            let message = "collection grows in loop without a prior reservation or explicit local bound";
            let report_span = expr.span.source_callsite();
            if let Some(suggestion) = collection_growth_suggestion(self.cx, local_id, self.init_spans) {
                emit_tiger_style_lint_with_suggestion(
                    self.cx,
                    UNBOUNDED_COLLECTION_GROWTH,
                    report_span,
                    message,
                    "unbounded_collection_growth",
                    suggestion,
                );
            } else {
                emit_tiger_style_lint(
                    self.cx,
                    UNBOUNDED_COLLECTION_GROWTH,
                    report_span,
                    message,
                    "unbounded_collection_growth",
                );
            }
        }

        intravisit::walk_expr(self, expr);
    }
}

fn collection_growth_suggestion(
    cx: &LateContext<'_>,
    local_id: hir::HirId,
    init_spans: &HashMap<hir::HirId, rustc_span::Span>,
) -> Option<LintSuggestion> {
    let init_span = *init_spans.get(&local_id)?;
    let sm = cx.sess().source_map();
    let snippet = sm.span_to_snippet(init_span).ok()?;

    // Only suggest for ::new() calls — replace with ::with_capacity(/* LIMIT */)
    let replacement = if snippet.contains("::new()") {
        snippet.replace("::new()", "::with_capacity(/* LIMIT */)")
    } else {
        return None;
    };

    Some(LintSuggestion::Single {
        span: init_span,
        message: "pre-allocate with an explicit capacity before the loop".to_owned(),
        replacement,
        applicability: Applicability::HasPlaceholders,
    })
}

impl<'tcx> LateLintPass<'tcx> for UnboundedCollectionGrowth {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let ExprKind::Loop(loop_block, _, loop_source, _) = &expr.kind else {
            return;
        };
        if expr.span.from_expansion() && !matches!(loop_source, hir::LoopSource::ForLoop) {
            return;
        }

        let Some(parent_block) = enclosing_block(cx, expr.hir_id) else {
            return;
        };
        let (candidates, pre_reserved, init_spans) = collect_candidate_bindings_before_loop(cx, parent_block, expr);
        if candidates.is_empty() {
            return;
        }

        let mut warned = HashSet::new();
        let mut analyzer = LoopBodyAnalyzer {
            cx,
            candidates: &candidates,
            warned: &mut warned,
            guarded: HashSet::new(),
            reserved: pre_reserved,
            init_spans: &init_spans,
        };
        analyzer.visit_block(loop_block);
    }
}
