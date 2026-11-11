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
/// Whole words that make a comment about locks; `clock`, `block`, `unlock`,
/// and `deadlock` do not count.
const LOCK_COMMENT_SUBJECTS: &[&str] = &["lock", "locking", "locked"];
/// Whole words that make a lock comment document acquisition order;
/// `recorder` and `border` do not count.
const LOCK_COMMENT_ORDER_WORDS: &[&str] = &["order", "ordering", "ordered", "acquisition"];
// `parking_lot::Mutex` and `parking_lot::RwLock` are type aliases of the
// `lock_api` types, so their resolved definition path starts with `lock_api`.
const SUPPORTED_LOCK_CRATES: &[&str] = &["std", "tokio", "lock_api"];

declare_tool_lint! {
    /// Detects struct definitions containing 2 or more `Mutex` or `RwLock`
    /// fields (`std`, `tokio`, or `parking_lot`/`lock_api`, including locks
    /// held behind `Arc`) without a comment documenting lock acquisition order.
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

/// Peels one shared-ownership `Arc` layer so `Arc<Mutex<T>>` counts as a lock field.
fn peel_shared_lock_owner<'tcx>(cx: &LateContext<'tcx>, ty: ty::Ty<'tcx>) -> ty::Ty<'tcx> {
    let ty = ty.peel_refs();
    match ty.kind() {
        ty::Adt(adt, args) if cx.tcx.is_diagnostic_item(rustc_span::sym::Arc, adt.did()) => args.type_at(0).peel_refs(),
        _ => ty,
    }
}

fn ty_lock_kind<'tcx>(cx: &LateContext<'tcx>, ty: ty::Ty<'tcx>) -> Option<&'static str> {
    let ty::Adt(adt, _) = peel_shared_lock_owner(cx, ty).kind() else {
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

fn text_documents_lock_order(text: &str) -> bool {
    crate::naming::identifier_words::identifier_has_word(text, LOCK_COMMENT_SUBJECTS)
        && crate::naming::identifier_words::identifier_has_word(text, LOCK_COMMENT_ORDER_WORDS)
}

fn comment_line_is_lock_order(line: &str) -> bool {
    let trimmed = line.trim_start();
    let is_comment = trimmed.starts_with("//") || trimmed.starts_with("///") || trimmed.starts_with("/*");
    is_comment && text_documents_lock_order(trimmed)
}

fn has_order_specific_lock_comment(snippet: &str) -> bool {
    snippet.lines().any(comment_line_is_lock_order)
}

fn item_order_comment_fact(cx: &LateContext<'_>, item: &hir::Item<'_>) -> bool {
    let attrs = cx.tcx.hir_attrs(item.hir_id());
    // Doc attributes (`///`, `/** */`, `#[doc = "..."]`) document the item.
    if attrs
        .iter()
        .filter_map(hir::Attribute::doc_str)
        .any(|doc| doc.as_str().lines().any(text_documents_lock_order))
    {
        return true;
    }

    // Comments inside the struct body.
    let source_map = cx.sess().source_map();
    if source_map.span_to_snippet(item.span).is_ok_and(|snippet| has_order_specific_lock_comment(&snippet)) {
        return true;
    }

    // Comments between the outer attributes and directly above them;
    // `item.span` excludes outer attributes.
    crate::source::any_leading_comment_line(source_map, item.span, comment_line_is_lock_order)
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

    #[test]
    fn comment_classifier_matches_whole_words_only() {
        assert!(!has_order_specific_lock_comment("// Clock ordering follows the wall-clock source."));
        assert!(!has_order_specific_lock_comment("// Unlock the block recorder at the border."));
        assert!(has_order_specific_lock_comment("// Locks: acquire in `count`, `state` order."));
        assert!(has_order_specific_lock_comment("// lock_order: count before state"));
        assert!(has_order_specific_lock_comment("// Locking order: count before state"));
        assert!(has_order_specific_lock_comment("// Both are locked as ordered: count, then state"));
        assert!(!has_order_specific_lock_comment("// Deadlock-free: state is ordered after count."));
    }
}
