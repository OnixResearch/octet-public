use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

const MIN_LOCK_FIELD_COUNT: usize = 2;
const LOCK_KIND_MUTEX: &str = "Mutex";
const LOCK_KIND_RWLOCK: &str = "RwLock";
const LOCK_COMMENT_SUBJECT: &str = "lock";
const LOCK_COMMENT_ORDER: &str = "order";
const LOCK_COMMENT_ORDERING: &str = "ordering";
const LOCK_COMMENT_ACQUISITION: &str = "acquisition";
const SUPPORTED_LOCK_CRATES: &[&str] = &["std", "tokio", "parking_lot"];

declare_tool_lint! {
    /// Detects struct definitions containing 2 or more `Mutex` or `RwLock`
    /// fields without a comment documenting lock acquisition order.
    /// Undocumented lock ordering leads to deadlocks that only appear
    /// under production concurrency.
    pub tigerstyle::MULTI_LOCK_ORDERING,
    Allow,
    "struct has multiple lock fields without documented ordering"
}

declare_lint_pass!(MultiLockOrdering => [MULTI_LOCK_ORDERING]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct LockOrderingFacts {
    lock_field_count: usize,
    has_order_specific_comment: bool,
}

fn classify_multi_lock_ordering(facts: LockOrderingFacts) -> bool {
    facts.lock_field_count >= MIN_LOCK_FIELD_COUNT && !facts.has_order_specific_comment
}

fn ty_lock_kind(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> Option<&'static str> {
    let ty::Adt(adt, _) = ty.peel_refs().kind() else {
        return None;
    };

    let def_path = cx.get_def_path(adt.did());
    let crate_name = def_path.first()?.as_str();
    if !SUPPORTED_LOCK_CRATES.contains(&crate_name) {
        return None;
    }

    match def_path.last()?.as_str() {
        LOCK_KIND_MUTEX => Some(LOCK_KIND_MUTEX),
        LOCK_KIND_RWLOCK => Some(LOCK_KIND_RWLOCK),
        _ => None,
    }
}

fn lock_field_count(cx: &LateContext<'_>, variant_data: &hir::VariantData<'_>) -> usize {
    variant_data
        .fields()
        .iter()
        .filter(|field| ty_lock_kind(cx, cx.tcx.type_of(field.def_id).instantiate_identity()).is_some())
        .count()
}

fn comment_line_is_lock_order(line: &str) -> bool {
    let trimmed = line.trim_start();
    let is_comment = trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("/*");
    if !is_comment {
        return false;
    }

    let lower = trimmed.to_ascii_lowercase();
    lower.contains(LOCK_COMMENT_SUBJECT)
        && (lower.contains(LOCK_COMMENT_ORDER)
            || lower.contains(LOCK_COMMENT_ORDERING)
            || lower.contains(LOCK_COMMENT_ACQUISITION))
}

fn has_order_specific_lock_comment(snippet: &str) -> bool {
    snippet.lines().any(comment_line_is_lock_order)
}

fn item_order_comment_fact(cx: &LateContext<'_>, item: &hir::Item<'_>) -> bool {
    cx.sess()
        .source_map()
        .span_to_snippet(item.span)
        .is_ok_and(|snippet| has_order_specific_lock_comment(&snippet))
}

impl<'tcx> LateLintPass<'tcx> for MultiLockOrdering {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Struct(ident, _, variant_data) = &item.kind else {
            return;
        };

        let facts = LockOrderingFacts {
            lock_field_count: lock_field_count(cx, variant_data),
            has_order_specific_comment: item_order_comment_fact(cx, item),
        };
        if !classify_multi_lock_ordering(facts) {
            return;
        }

        let message =
            format!("struct `{}` has {} lock fields without documented ordering", ident.name, facts.lock_field_count);
        if let Some(suggestion) = lock_ordering_suggestion(cx, item.span, facts.lock_field_count) {
            emit_tiger_style_lint_with_suggestion(
                cx,
                MULTI_LOCK_ORDERING,
                item.span,
                message,
                "multi_lock_ordering",
                suggestion,
            );
        } else {
            emit_tiger_style_lint(cx, MULTI_LOCK_ORDERING, item.span, message, "multi_lock_ordering");
        }
    }
}

fn lock_ordering_suggestion(
    cx: &LateContext<'_>,
    item_span: rustc_span::Span,
    lock_count: usize,
) -> Option<LintSuggestion> {
    const OPENING_BRACE_WIDTH_BYTES: usize = 1;

    let snippet = cx.sess().source_map().span_to_snippet(item_span).ok()?;
    let brace_pos = snippet.find('{')?;

    let insert_offset_bytes = u32::try_from(brace_pos + OPENING_BRACE_WIDTH_BYTES).ok()?;
    let insert_offset = rustc_span::BytePos(insert_offset_bytes);
    let insert_span = item_span.with_lo(item_span.lo() + insert_offset).with_hi(item_span.lo() + insert_offset);

    let replacement = format!("\n    // Lock order: <document acquisition order for {lock_count} locks>");

    Some(LintSuggestion::Single {
        span: insert_span,
        message: "add lock ordering documentation".to_owned(),
        replacement,
        applicability: Applicability::HasPlaceholders,
    })
}

#[cfg(test)]
mod tests {
    use super::LockOrderingFacts;
    use super::classify_multi_lock_ordering;
    use super::has_order_specific_lock_comment;

    #[test]
    fn classifies_multiple_locks_without_order_comment() {
        let facts = LockOrderingFacts {
            lock_field_count: 2,
            has_order_specific_comment: false,
        };

        assert!(classify_multi_lock_ordering(facts));
    }

    #[test]
    fn skips_single_lock_or_documented_order() {
        let single = LockOrderingFacts {
            lock_field_count: 1,
            has_order_specific_comment: false,
        };
        let documented = LockOrderingFacts {
            lock_field_count: 2,
            has_order_specific_comment: true,
        };

        assert!(!classify_multi_lock_ordering(single));
        assert!(!classify_multi_lock_ordering(documented));
    }

    #[test]
    fn comment_classifier_requires_lock_specific_ordering() {
        let unrelated = "struct UsesAtomics {\n    // memory ordering is SeqCst here\n}";
        let documented = "struct Locks {\n    // Lock ordering: acquire metadata before state\n}";

        assert!(!has_order_specific_lock_comment(unrelated));
        assert!(has_order_specific_lock_comment(documented));
    }
}
