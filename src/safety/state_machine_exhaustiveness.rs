use std::collections::BTreeSet;

use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::PatKind;
use rustc_hir::intravisit;
use rustc_hir::intravisit::FnKind;
use rustc_hir::intravisit::Visitor;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;

const LINT_NAME: &str = "state_machine_exhaustiveness";
const UNSUPPORTED_SHAPE: &str = "unsupported-state-machine-shape";
const WILDCARD_FALLBACK: &str = "state-machine-wildcard-fallback";
const HIDDEN_INVALID_TRANSITION: &str = "hidden-invalid-transition";
const MISSING_ENUM_MARKER: &str = "missing-fsm-enum-marker";
const MUTATION_COUPLED_TRANSITION: &str = "mutation-coupled-transition";

const INVALID_TRANSITION_TOKENS: &[&str] = &["Err(", "Invalid", "invalid", "None"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransitionArmFact {
    pub state_variants: Vec<String>,
    pub event_variants: Vec<String>,
    pub uses_wildcard: bool,
    pub explicit_invalid: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransitionCoreFacts {
    pub state_type: String,
    pub event_type: String,
    pub state_variants: Vec<String>,
    pub event_variants: Vec<String>,
    pub state_enum_marked: bool,
    pub event_enum_marked: bool,
    pub arms: Vec<TransitionArmFact>,
    pub has_mutation: bool,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum TransitionIssueKind {
    UnsupportedShape,
    WildcardFallback,
    HiddenInvalidTransition,
    MissingEnumMarker,
    MutationCoupledTransition,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransitionIssue {
    pub kind: TransitionIssueKind,
    pub detail: String,
}

declare_tool_lint! {
    /// Checks marked state-machine transition cores for explicit state/event handling.
    /// This is bounded source-shape evidence: it rejects wildcard fallbacks,
    /// hidden invalid transitions, unsupported marked shapes, and mutation-coupled
    /// decisions, but it does not prove protocol correctness or liveness.
    pub tigerstyle::STATE_MACHINE_EXHAUSTIVENESS,
    Warn,
    "marked state-machine transition core must handle state/event pairs explicitly"
}

pub struct StateMachineExhaustiveness {
    state_machine_core_marker: String,
    state_machine_enum_marker: String,
    state_machine_core_scopes: Vec<String>,
    state_machine_generated_adapter_scopes: Vec<String>,
}

impl StateMachineExhaustiveness {
    pub fn new(
        state_machine_core_marker: String,
        state_machine_enum_marker: String,
        state_machine_core_scopes: Vec<String>,
        state_machine_generated_adapter_scopes: Vec<String>,
    ) -> Self {
        Self {
            state_machine_core_marker,
            state_machine_enum_marker,
            state_machine_core_scopes,
            state_machine_generated_adapter_scopes,
        }
    }
}

impl_lint_pass!(StateMachineExhaustiveness => [STATE_MACHINE_EXHAUSTIVENESS]);

pub(crate) fn evaluate_transition_core(facts: &TransitionCoreFacts) -> Vec<TransitionIssue> {
    let mut issues = Vec::new();

    if facts.state_variants.is_empty() || facts.event_variants.is_empty() || facts.arms.is_empty() {
        push_issue(&mut issues, TransitionIssueKind::UnsupportedShape, "missing typed state/event inventory");
        return issues;
    }

    if !facts.state_enum_marked || !facts.event_enum_marked {
        push_issue(
            &mut issues,
            TransitionIssueKind::MissingEnumMarker,
            "state and event enums must carry the configured FSM enum marker",
        );
    }

    if facts.has_mutation {
        push_issue(
            &mut issues,
            TransitionIssueKind::MutationCoupledTransition,
            "transition decision mutates state instead of returning a decision",
        );
    }

    let mut covered_pairs = BTreeSet::new();
    for arm in &facts.arms {
        if arm.uses_wildcard {
            push_issue(
                &mut issues,
                TransitionIssueKind::WildcardFallback,
                "wildcard or rest pattern covers one or more state/event pairs",
            );
            continue;
        }

        if arm.state_variants.is_empty() || arm.event_variants.is_empty() {
            push_issue(
                &mut issues,
                TransitionIssueKind::UnsupportedShape,
                "arm does not bind concrete state and event variants",
            );
            continue;
        }

        for state in &arm.state_variants {
            for event in &arm.event_variants {
                covered_pairs.insert((state.clone(), event.clone()));
            }
        }
    }

    let expected_pairs = pair_count(&facts.state_variants, &facts.event_variants);
    if covered_pairs.len() != expected_pairs {
        let missing_count = expected_pairs.saturating_sub(covered_pairs.len());
        push_issue(
            &mut issues,
            TransitionIssueKind::HiddenInvalidTransition,
            &format!("{missing_count} state/event pair(s) require explicit next-state or invalid-transition handling"),
        );
    }

    dedupe_issues(issues)
}

fn pair_count(state_variants: &[String], event_variants: &[String]) -> usize {
    state_variants.len().saturating_mul(event_variants.len())
}

fn push_issue(issues: &mut Vec<TransitionIssue>, kind: TransitionIssueKind, detail: &str) {
    issues.push(TransitionIssue {
        kind,
        detail: detail.to_owned(),
    });
}

fn dedupe_issues(issues: Vec<TransitionIssue>) -> Vec<TransitionIssue> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for issue in issues {
        if seen.insert((issue.kind, issue.detail.clone())) {
            deduped.push(issue);
        }
    }
    deduped
}

struct TransitionMatchFacts {
    facts: TransitionCoreFacts,
    span: Span,
}

struct TransitionMatchFinder<'a, 'tcx> {
    cx: &'a LateContext<'tcx>,
    state_machine_enum_marker: &'a str,
    found: Option<TransitionMatchFacts>,
}

impl<'tcx> Visitor<'tcx> for TransitionMatchFinder<'_, 'tcx> {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found.is_some() {
            return;
        }

        if let Some(facts) = transition_match_facts(self.cx, expr, self.state_machine_enum_marker) {
            self.found = Some(facts);
            return;
        }

        intravisit::walk_expr(self, expr);
    }
}

struct MutationFinder {
    found: bool,
}

impl<'tcx> Visitor<'tcx> for MutationFinder {
    fn visit_expr(&mut self, expr: &'tcx hir::Expr<'tcx>) {
        if self.found {
            return;
        }

        match &expr.kind {
            ExprKind::Assign(_, _, _) | ExprKind::AssignOp(_, _, _) => {
                self.found = true;
            }
            _ => intravisit::walk_expr(self, expr),
        }
    }
}

fn transition_match_facts<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    state_machine_enum_marker: &str,
) -> Option<TransitionMatchFacts> {
    let ExprKind::Match(scrutinee, arms, _) = &expr.kind else {
        return None;
    };

    let (state_adt, event_adt) = tuple_enum_types(cx, scrutinee)?;
    let state_variants = variant_names(state_adt);
    let event_variants = variant_names(event_adt);
    let state_type = cx.tcx.def_path_str(state_adt.did());
    let event_type = cx.tcx.def_path_str(event_adt.did());
    let arms = arms
        .iter()
        .map(|arm| transition_arm_fact(cx, arm, &state_variants, &event_variants))
        .collect::<Vec<_>>();

    Some(TransitionMatchFacts {
        facts: TransitionCoreFacts {
            state_type,
            event_type,
            state_variants,
            event_variants,
            state_enum_marked: enum_has_marker(cx, state_adt, state_machine_enum_marker),
            event_enum_marked: enum_has_marker(cx, event_adt, state_machine_enum_marker),
            arms,
            has_mutation: false,
        },
        span: expr.span,
    })
}

fn tuple_enum_types<'tcx>(
    cx: &LateContext<'tcx>,
    scrutinee: &'tcx hir::Expr<'tcx>,
) -> Option<(rustc_middle::ty::AdtDef<'tcx>, rustc_middle::ty::AdtDef<'tcx>)> {
    let scrutinee_ty = cx.typeck_results().expr_ty(scrutinee);
    let ty::Tuple(items) = scrutinee_ty.kind() else {
        return None;
    };

    let mut types = items.iter();
    let state_ty = types.next()?;
    let event_ty = types.next()?;
    if types.next().is_some() {
        return None;
    }

    Some((enum_adt(state_ty)?, enum_adt(event_ty)?))
}

fn enum_adt<'tcx>(ty: ty::Ty<'tcx>) -> Option<rustc_middle::ty::AdtDef<'tcx>> {
    let ty::Adt(adt, _) = ty.kind() else {
        return None;
    };
    adt.is_enum().then_some(*adt)
}

fn variant_names(adt: rustc_middle::ty::AdtDef<'_>) -> Vec<String> {
    adt.variants().iter().map(|variant| variant.name.as_str().to_owned()).collect()
}

fn transition_arm_fact(
    cx: &LateContext<'_>,
    arm: &hir::Arm<'_>,
    state_variants: &[String],
    event_variants: &[String],
) -> TransitionArmFact {
    let pattern = snippet(cx, arm.pat.span);
    let body = snippet(cx, arm.body.span);
    TransitionArmFact {
        state_variants: mentioned_variants(&pattern, state_variants),
        event_variants: mentioned_variants(&pattern, event_variants),
        uses_wildcard: pattern_uses_wildcard(arm, &pattern),
        explicit_invalid: INVALID_TRANSITION_TOKENS.iter().any(|token| body.contains(token)),
    }
}

fn mentioned_variants(pattern: &str, variants: &[String]) -> Vec<String> {
    variants.iter().filter(|variant| pattern_mentions_variant(pattern, variant)).cloned().collect()
}

fn pattern_mentions_variant(pattern: &str, variant: &str) -> bool {
    pattern.contains(&format!("::{variant}"))
        || pattern.split(|ch: char| !ch.is_alphanumeric() && ch != '_').any(|token| token == variant)
}

fn pattern_uses_wildcard(arm: &hir::Arm<'_>, pattern: &str) -> bool {
    matches!(arm.pat.kind, PatKind::Wild) || pattern.contains('_') || pattern.contains("..")
}

fn snippet(cx: &LateContext<'_>, span: Span) -> String {
    cx.sess().source_map().span_to_snippet(span).unwrap_or_default()
}

fn enum_has_marker(cx: &LateContext<'_>, adt: rustc_middle::ty::AdtDef<'_>, marker: &str) -> bool {
    let Some(local_def_id) = adt.did().as_local() else {
        return false;
    };
    has_marker(cx, local_def_id, marker)
}

fn has_marker(cx: &LateContext<'_>, def_id: hir::def_id::LocalDefId, marker: &str) -> bool {
    let hir_id = cx.tcx.local_def_id_to_hir_id(def_id);
    let marker_tail = marker.rsplit("::").next().unwrap_or(marker);
    cx.tcx.hir_attrs(hir_id).iter().any(|attr| {
        let rendered = format!("{attr:?}");
        rendered.contains(marker) || rendered.contains(marker_tail)
    })
}

fn span_path(cx: &LateContext<'_>, span: Span) -> String {
    format!("{:?}", cx.sess().source_map().lookup_char_pos(span.lo()).file.name)
}

fn span_matches_any_scope(path: &str, scopes: &[String]) -> bool {
    scopes.iter().any(|scope| !scope.is_empty() && path.contains(scope))
}

fn function_name(kind: FnKind<'_>, def_id: hir::def_id::LocalDefId, cx: &LateContext<'_>) -> String {
    match kind {
        FnKind::ItemFn(ident, _, _) | FnKind::Method(ident, _) => ident.name.to_string(),
        FnKind::Closure => cx.tcx.def_path_str(def_id.to_def_id()),
    }
}

fn issue_message(function: &str, facts: Option<&TransitionCoreFacts>, issue: &TransitionIssue) -> String {
    let context =
        facts.map_or_else(String::new, |facts| format!(" over `{}` and `{}`", facts.state_type, facts.event_type));
    match issue.kind {
        TransitionIssueKind::UnsupportedShape => format!(
            "state machine transition core `{function}`{context} has unsupported source shape ({UNSUPPORTED_SHAPE}): {}",
            issue.detail
        ),
        TransitionIssueKind::WildcardFallback => format!(
            "state machine transition core `{function}`{context} uses wildcard fallback ({WILDCARD_FALLBACK}); handle every state/event pair explicitly"
        ),
        TransitionIssueKind::HiddenInvalidTransition => format!(
            "state machine transition core `{function}`{context} hides invalid transitions ({HIDDEN_INVALID_TRANSITION}); {}",
            issue.detail
        ),
        TransitionIssueKind::MissingEnumMarker => format!(
            "state machine transition core `{function}`{context} lacks closed FSM enum markers ({MISSING_ENUM_MARKER}); {}",
            issue.detail
        ),
        TransitionIssueKind::MutationCoupledTransition => format!(
            "state machine transition core `{function}`{context} mutates transition state ({MUTATION_COUPLED_TRANSITION}); return an explicit decision from a pure core"
        ),
    }
}

impl<'tcx> LateLintPass<'tcx> for StateMachineExhaustiveness {
    fn check_fn(
        &mut self,
        cx: &LateContext<'tcx>,
        kind: FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: Span,
        def_id: hir::def_id::LocalDefId,
    ) {
        let function_path = span_path(cx, span);
        if span_matches_any_scope(&function_path, &self.state_machine_generated_adapter_scopes) {
            return;
        }

        let marker_selected = has_marker(cx, def_id, &self.state_machine_core_marker);
        let scope_selected = span_matches_any_scope(&function_path, &self.state_machine_core_scopes);
        if !marker_selected && !scope_selected {
            return;
        }

        let function = function_name(kind, def_id, cx);
        if body.value.span.from_expansion() {
            let issue = TransitionIssue {
                kind: TransitionIssueKind::UnsupportedShape,
                detail: "marked transition body came from macro expansion".to_owned(),
            };
            emit_tiger_style_lint(
                cx,
                STATE_MACHINE_EXHAUSTIVENESS,
                span,
                issue_message(&function, None, &issue),
                LINT_NAME,
            );
            return;
        }

        let mut finder = TransitionMatchFinder {
            cx,
            state_machine_enum_marker: &self.state_machine_enum_marker,
            found: None,
        };
        finder.visit_expr(body.value);

        let Some(mut transition) = finder.found else {
            let issue = TransitionIssue {
                kind: TransitionIssueKind::UnsupportedShape,
                detail: "expected a tuple match over typed state and event enums".to_owned(),
            };
            emit_tiger_style_lint(
                cx,
                STATE_MACHINE_EXHAUSTIVENESS,
                span,
                issue_message(&function, None, &issue),
                LINT_NAME,
            );
            return;
        };

        let mut mutation_finder = MutationFinder { found: false };
        mutation_finder.visit_expr(body.value);
        transition.facts.has_mutation = mutation_finder.found;

        for issue in evaluate_transition_core(&transition.facts) {
            emit_tiger_style_lint(
                cx,
                STATE_MACHINE_EXHAUSTIVENESS,
                transition.span,
                issue_message(&function, Some(&transition.facts), &issue),
                LINT_NAME,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn explicit_facts() -> TransitionCoreFacts {
        TransitionCoreFacts {
            state_type: "State".to_owned(),
            event_type: "Event".to_owned(),
            state_variants: vec!["Idle".to_owned(), "Running".to_owned()],
            event_variants: vec!["Start".to_owned(), "Stop".to_owned()],
            state_enum_marked: true,
            event_enum_marked: true,
            arms: vec![
                arm(["Idle"], ["Start"], false),
                arm(["Idle"], ["Stop"], false),
                arm(["Running"], ["Start"], false),
                arm(["Running"], ["Stop"], false),
            ],
            has_mutation: false,
        }
    }

    fn arm<const STATE_COUNT: usize, const EVENT_COUNT: usize>(
        state_variants: [&str; STATE_COUNT],
        event_variants: [&str; EVENT_COUNT],
        uses_wildcard: bool,
    ) -> TransitionArmFact {
        TransitionArmFact {
            state_variants: state_variants.iter().map(|variant| (*variant).to_owned()).collect(),
            event_variants: event_variants.iter().map(|variant| (*variant).to_owned()).collect(),
            uses_wildcard,
            explicit_invalid: false,
        }
    }

    #[test]
    fn explicit_state_event_matrix_passes() {
        let issues = evaluate_transition_core(&explicit_facts());
        assert_eq!(issues, Vec::new());
    }

    #[test]
    fn wildcard_fallback_and_hidden_pair_fail_closed() {
        let mut facts = explicit_facts();
        facts.arms.pop();
        facts.arms.push(arm([], [], true));

        let issues = evaluate_transition_core(&facts);
        assert!(issues.iter().any(|issue| issue.kind == TransitionIssueKind::WildcardFallback));
        assert!(issues.iter().any(|issue| issue.kind == TransitionIssueKind::HiddenInvalidTransition));
    }

    #[test]
    fn mutation_coupled_transition_fails_closed() {
        let mut facts = explicit_facts();
        facts.has_mutation = true;

        let issues = evaluate_transition_core(&facts);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, TransitionIssueKind::MutationCoupledTransition);
    }

    #[test]
    fn missing_fsm_enum_markers_fail_closed() {
        let mut facts = explicit_facts();
        facts.event_enum_marked = false;

        let issues = evaluate_transition_core(&facts);
        assert!(issues.iter().any(|issue| issue.kind == TransitionIssueKind::MissingEnumMarker));
    }

    #[test]
    fn missing_variant_inventory_is_unsupported() {
        let facts = TransitionCoreFacts {
            state_type: "State".to_owned(),
            event_type: "Event".to_owned(),
            state_variants: Vec::new(),
            event_variants: vec!["Start".to_owned()],
            state_enum_marked: false,
            event_enum_marked: true,
            arms: Vec::new(),
            has_mutation: false,
        };

        let issues = evaluate_transition_core(&facts);
        assert_eq!(issues[0].kind, TransitionIssueKind::UnsupportedShape);
    }
}
