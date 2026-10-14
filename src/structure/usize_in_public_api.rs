use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ItemKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects inherent public APIs that expose platform-sized integers.
    /// Tiger Style prefers fixed-width types at boundaries so signatures stay
    /// stable across 32-bit and 64-bit targets.
    pub tigerstyle::USIZE_IN_PUBLIC_API,
    Warn,
    "platform-sized integer in public API"
}

declare_lint_pass!(UsizeInPublicApi => [USIZE_IN_PUBLIC_API]);

fn platform_sized_int_name(ty: ty::Ty<'_>) -> Option<&'static str> {
    match ty.kind() {
        ty::Uint(ty::UintTy::Usize) => Some("usize"),
        ty::Int(ty::IntTy::Isize) => Some("isize"),
        ty::Ref(_, inner, _) | ty::Slice(inner) | ty::Array(inner, _) => platform_sized_int_name(*inner),
        ty::Tuple(items) => items.iter().find_map(platform_sized_int_name),
        ty::Adt(_, args) => args.types().find_map(platform_sized_int_name),
        _ => None,
    }
}

fn lint_message(name: &str) -> String {
    if name == "usize" {
        "`usize` in public API — use `u32` or `u64` for cross-platform consistency".to_owned()
    } else {
        "`isize` in public API — use `i32` or `i64` for cross-platform consistency".to_owned()
    }
}

fn collect_platform_int_spans(ty: &hir::Ty<'_>) -> Vec<(rustc_span::Span, &'static str)> {
    let mut results = Vec::new();
    collect_platform_int_ty(ty, &mut results);
    results
}

fn collect_platform_int_ty(ty: &hir::Ty<'_>, out: &mut Vec<(rustc_span::Span, &'static str)>) {
    match &ty.kind {
        hir::TyKind::Path(hir::QPath::Resolved(_, path)) => {
            if let hir::def::Res::PrimTy(prim) = path.res {
                match prim {
                    hir::PrimTy::Uint(rustc_ast::UintTy::Usize) => out.push((ty.span, "usize")),
                    hir::PrimTy::Int(rustc_ast::IntTy::Isize) => out.push((ty.span, "isize")),
                    _ => {}
                }
            }
        }
        hir::TyKind::Ref(_, mut_ty) => collect_platform_int_ty(mut_ty.ty, out),
        hir::TyKind::Slice(inner) | hir::TyKind::Array(inner, _) => collect_platform_int_ty(inner, out),
        hir::TyKind::Tup(items) => items.iter().for_each(|t| collect_platform_int_ty(t, out)),
        _ => {}
    }
}

fn usize_suggestion(decl: &hir::FnDecl<'_>) -> Option<LintSuggestion> {
    let mut spans = Vec::new();
    for input in decl.inputs {
        spans.extend(collect_platform_int_spans(input));
    }
    if let hir::FnRetTy::Return(ret_ty) = &decl.output {
        spans.extend(collect_platform_int_spans(ret_ty));
    }

    if spans.is_empty() {
        return None;
    }

    let parts: Vec<(rustc_span::Span, String)> = spans
        .iter()
        .map(|(span, name)| {
            let replacement = if *name == "usize" { "u64" } else { "i64" };
            (*span, replacement.to_owned())
        })
        .collect();

    if parts.len() == 1 {
        let (span, replacement) = parts.into_iter().next().unwrap();
        Some(LintSuggestion::Single {
            span,
            message: format!("replace with `{replacement}` for cross-platform consistency"),
            replacement,
            applicability: Applicability::MaybeIncorrect,
        })
    } else {
        Some(LintSuggestion::Multipart {
            message: "replace platform-sized integers with fixed-width types".to_owned(),
            parts,
            applicability: Applicability::MaybeIncorrect,
        })
    }
}

fn emit_with_suggestion(cx: &LateContext<'_>, report_span: rustc_span::Span, decl: &hir::FnDecl<'_>, name: &str) {
    let message = lint_message(name);
    if let Some(suggestion) = usize_suggestion(decl) {
        emit_tiger_style_lint_with_suggestion(
            cx,
            USIZE_IN_PUBLIC_API,
            report_span,
            message,
            "usize_in_public_api",
            suggestion,
        );
    } else {
        emit_tiger_style_lint(cx, USIZE_IN_PUBLIC_API, report_span, message, "usize_in_public_api");
    }
}

impl<'tcx> LateLintPass<'tcx> for UsizeInPublicApi {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let ItemKind::Fn { sig, .. } = &item.kind else {
            return;
        };

        let def_id = item.owner_id.def_id;
        if !cx.tcx.visibility(def_id).is_public() {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(def_id).instantiate_identity();
        let mir_sig = fn_sig.skip_binder();

        if let Some(name) = mir_sig.inputs().iter().find_map(|ty| platform_sized_int_name(*ty)) {
            emit_with_suggestion(cx, item.span, sig.decl, name);
            return;
        }

        if let Some(name) = platform_sized_int_name(mir_sig.output()) {
            emit_with_suggestion(cx, item.span, sig.decl, name);
        }
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(sig, _) = &impl_item.kind else {
            return;
        };

        let def_id = impl_item.owner_id.def_id;
        let parent_hir_id = cx.tcx.local_def_id_to_hir_id(cx.tcx.local_parent(def_id));
        let hir::Node::Item(parent_item) = cx.tcx.hir_node(parent_hir_id) else {
            return;
        };
        let hir::ItemKind::Impl(impl_block) = &parent_item.kind else {
            return;
        };
        if impl_block.of_trait.is_some() || !cx.tcx.visibility(def_id).is_public() {
            return;
        }

        let fn_sig = cx.tcx.fn_sig(def_id).instantiate_identity();
        let mir_sig = fn_sig.skip_binder();

        if let Some(name) = mir_sig.inputs().iter().find_map(|ty| platform_sized_int_name(*ty)) {
            emit_with_suggestion(cx, impl_item.span, sig.decl, name);
            return;
        }

        if let Some(name) = platform_sized_int_name(mir_sig.output()) {
            emit_with_suggestion(cx, impl_item.span, sig.decl, name);
        }
    }
}
