// r[impl octet.runner_self_check.warning_budget.reduction.public_unsafe_abi]
use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context::late_hir_id_is_in_test_context;

const LINT_NAME: &str = "public_unsafe_api";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ApiVisibility {
    Public,
    Private,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ApiAbi {
    Rust,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ApiSafety {
    Unsafe,
    Safe,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PublicUnsafeApiFacts {
    visibility: ApiVisibility,
    abi: ApiAbi,
    safety: ApiSafety,
    is_trait_impl_method: bool,
    is_in_test_context: bool,
    is_from_expansion: bool,
}

declare_tool_lint! {
    /// Detects public Rust APIs that expose `unsafe fn` directly.
    /// Tiger Style prefers containing unsafe operations behind safe public wrappers
    /// and small private helpers so caller obligations stay localized and auditable.
    pub tigerstyle::PUBLIC_UNSAFE_API,
    Warn,
    "public unsafe Rust API exposes caller obligations"
}

declare_lint_pass!(PublicUnsafeApi => [PUBLIC_UNSAFE_API]);

fn signature_declares_unsafe_fn(sig: &hir::FnSig<'_>) -> bool {
    matches!(sig.header.safety, hir::HeaderSafety::Normal(hir::Safety::Unsafe))
}

fn signature_uses_rust_abi(sig: &hir::FnSig<'_>) -> bool {
    matches!(sig.header.abi, rustc_abi::ExternAbi::Rust)
}

fn classify_public_unsafe_api(facts: PublicUnsafeApiFacts) -> bool {
    facts.visibility == ApiVisibility::Public
        && facts.abi == ApiAbi::Rust
        && facts.safety == ApiSafety::Unsafe
        && !facts.is_trait_impl_method
        && !facts.is_in_test_context
        && !facts.is_from_expansion
}

fn lint_public_unsafe(cx: &LateContext<'_>, span: rustc_span::Span) {
    emit_tiger_style_lint(
        cx,
        PUBLIC_UNSAFE_API,
        span,
        "public unsafe API leaks caller obligations; prefer a safe public wrapper around a small private unsafe helper",
        LINT_NAME,
    );
}

fn impl_parent_is_trait_impl(cx: &LateContext<'_>, impl_item: &hir::ImplItem<'_>) -> bool {
    let def_id = impl_item.owner_id.def_id;
    let parent_hir_id = cx.tcx.local_def_id_to_hir_id(cx.tcx.local_parent(def_id));
    let hir::Node::Item(parent_item) = cx.tcx.hir_node(parent_hir_id) else {
        return false;
    };
    let hir::ItemKind::Impl(impl_block) = &parent_item.kind else {
        return false;
    };
    impl_block.of_trait.is_some()
}

impl<'tcx> LateLintPass<'tcx> for PublicUnsafeApi {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { sig, .. } = &item.kind else {
            return;
        };

        let facts = PublicUnsafeApiFacts {
            visibility: if cx.tcx.visibility(item.owner_id.def_id).is_public() {
                ApiVisibility::Public
            } else {
                ApiVisibility::Private
            },
            abi: if signature_uses_rust_abi(sig) {
                ApiAbi::Rust
            } else {
                ApiAbi::Other
            },
            safety: if signature_declares_unsafe_fn(sig) {
                ApiSafety::Unsafe
            } else {
                ApiSafety::Safe
            },
            is_trait_impl_method: false,
            is_in_test_context: late_hir_id_is_in_test_context(cx, item.hir_id(), item.span),
            is_from_expansion: item.span.from_expansion(),
        };

        if classify_public_unsafe_api(facts) {
            lint_public_unsafe(cx, item.span);
        }
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(sig, _) = &impl_item.kind else {
            return;
        };

        let def_id = impl_item.owner_id.def_id;
        let facts = PublicUnsafeApiFacts {
            visibility: if cx.tcx.visibility(def_id).is_public() {
                ApiVisibility::Public
            } else {
                ApiVisibility::Private
            },
            abi: if signature_uses_rust_abi(sig) {
                ApiAbi::Rust
            } else {
                ApiAbi::Other
            },
            safety: if signature_declares_unsafe_fn(sig) {
                ApiSafety::Unsafe
            } else {
                ApiSafety::Safe
            },
            is_trait_impl_method: impl_parent_is_trait_impl(cx, impl_item),
            is_in_test_context: late_hir_id_is_in_test_context(cx, impl_item.hir_id(), impl_item.span),
            is_from_expansion: impl_item.span.from_expansion(),
        };

        if classify_public_unsafe_api(facts) {
            lint_public_unsafe(cx, impl_item.span);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ApiAbi;
    use super::ApiSafety;
    use super::ApiVisibility;
    use super::PublicUnsafeApiFacts;
    use super::classify_public_unsafe_api;

    fn unsafe_public_rust_api() -> PublicUnsafeApiFacts {
        PublicUnsafeApiFacts {
            visibility: ApiVisibility::Public,
            abi: ApiAbi::Rust,
            safety: ApiSafety::Unsafe,
            is_trait_impl_method: false,
            is_in_test_context: false,
            is_from_expansion: false,
        }
    }

    #[test]
    fn classifies_public_unsafe_rust_api() {
        assert!(classify_public_unsafe_api(unsafe_public_rust_api()));
    }

    #[test]
    fn skips_extern_unsafe_api() {
        let facts = PublicUnsafeApiFacts {
            abi: ApiAbi::Other,
            ..unsafe_public_rust_api()
        };

        assert!(!classify_public_unsafe_api(facts));
    }

    #[test]
    fn skips_trait_impl_method() {
        let facts = PublicUnsafeApiFacts {
            is_trait_impl_method: true,
            ..unsafe_public_rust_api()
        };

        assert!(!classify_public_unsafe_api(facts));
    }

    #[test]
    fn skips_private_or_safe_api() {
        let private = PublicUnsafeApiFacts {
            visibility: ApiVisibility::Private,
            ..unsafe_public_rust_api()
        };
        let safe = PublicUnsafeApiFacts {
            safety: ApiSafety::Safe,
            ..unsafe_public_rust_api()
        };

        assert!(!classify_public_unsafe_api(private));
        assert!(!classify_public_unsafe_api(safe));
    }
}
