use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Symbol;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;
use crate::test_context;

declare_tool_lint! {
    /// Detects function parameters that borrow owned standard containers.
    /// Tiger Style prefers the most general borrowed argument type when a
    /// signature does not need ownership, capacity, or exact owned type identity.
    pub tigerstyle::BORROWED_ARGUMENT_TYPES,
    Warn,
    "borrowed owned-container argument type"
}

declare_lint_pass!(BorrowedArgumentTypes => [BORROWED_ARGUMENT_TYPES]);

struct BorrowedArgumentCandidate {
    span: rustc_span::Span,
    found: String,
    replacement: String,
}

fn path_ends_with(path: &[Symbol], suffix: &[&str]) -> bool {
    path.len() >= suffix.len()
        && path[path.len() - suffix.len()..]
            .iter()
            .copied()
            .eq(suffix.iter().map(|segment| Symbol::intern(segment)))
}

fn borrowed_replacement(cx: &LateContext<'_>, ty: &hir::Ty<'_>) -> Option<BorrowedArgumentCandidate> {
    let hir::TyKind::Ref(_, borrowed) = &ty.kind else {
        return None;
    };
    let hir::TyKind::Path(hir::QPath::Resolved(_, path)) = &borrowed.ty.kind else {
        return None;
    };
    let hir::def::Res::Def(_, def_id) = path.res else {
        return None;
    };

    let def_path = cx.get_def_path(def_id);
    if path_ends_with(&def_path, &["string", "String"]) {
        return Some(BorrowedArgumentCandidate {
            span: borrowed.ty.span,
            found: "&String".to_owned(),
            replacement: "str".to_owned(),
        });
    }
    if path_ends_with(&def_path, &["vec", "Vec"]) {
        let snippet = cx.sess().source_map().span_to_snippet(borrowed.ty.span).ok()?;
        let inner_start = snippet.find("Vec<")? + "Vec<".len();
        let inner = snippet[inner_start..].strip_suffix('>')?;
        return Some(BorrowedArgumentCandidate {
            span: borrowed.ty.span,
            found: format!("&{snippet}"),
            replacement: format!("[{inner}]"),
        });
    }
    if path_ends_with(&def_path, &["path", "PathBuf"]) {
        return Some(BorrowedArgumentCandidate {
            span: borrowed.ty.span,
            found: "&PathBuf".to_owned(),
            replacement: "Path".to_owned(),
        });
    }

    None
}

fn signature_is_extern(cx: &LateContext<'_>, sig: &hir::FnSig<'_>) -> bool {
    cx.sess().source_map().span_to_snippet(sig.span).is_ok_and(|snippet| snippet.contains("extern \""))
}

fn lint_decl(cx: &LateContext<'_>, hir_id: hir::HirId, report_span: rustc_span::Span, sig: &hir::FnSig<'_>) {
    if signature_is_extern(cx, sig) || test_context::late_hir_id_is_in_test_context(cx, hir_id, report_span) {
        return;
    }

    for input in sig.decl.inputs {
        let Some(candidate) = borrowed_replacement(cx, input) else {
            continue;
        };
        let suggested_type = format!("&{}", candidate.replacement);
        let message = format!(
            "parameter uses `{}`; accept `{suggested_type}` unless ownership, capacity, or exact owned type identity is required",
            candidate.found
        );
        emit_tiger_style_lint_with_suggestion(
            cx,
            BORROWED_ARGUMENT_TYPES,
            report_span,
            message,
            "borrowed_argument_types",
            LintSuggestion::Single {
                span: candidate.span,
                message: format!("replace `{}` with `{suggested_type}`", candidate.found),
                replacement: candidate.replacement,
                applicability: Applicability::MaybeIncorrect,
            },
        );
        return;
    }
}

impl<'tcx> LateLintPass<'tcx> for BorrowedArgumentTypes {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { sig, .. } = &item.kind else {
            return;
        };
        lint_decl(cx, item.hir_id(), item.span, sig);
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(sig, _) = &impl_item.kind else {
            return;
        };
        let parent_hir_id = cx.tcx.local_def_id_to_hir_id(cx.tcx.local_parent(impl_item.owner_id.def_id));
        let hir::Node::Item(parent_item) = cx.tcx.hir_node(parent_hir_id) else {
            return;
        };
        let hir::ItemKind::Impl(impl_block) = &parent_item.kind else {
            return;
        };
        if impl_block.of_trait.is_some() {
            return;
        }

        lint_decl(cx, impl_item.hir_id(), impl_item.span, sig);
    }
}
