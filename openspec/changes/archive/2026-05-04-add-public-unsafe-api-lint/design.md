## Context

The broad external-source incorporation work selected contained unsafe modules as a Rust idiom worth importing into Tigerstyle. A broad lint for every unsafe block would be noisy and would fight the desired pattern: safe public wrappers often need a small private unsafe implementation seam.

This child OpenSpec intentionally scopes the first candidate to public Rust `unsafe fn` signatures. The signal is easy to explain and fixture: a public unsafe signature exports a caller obligation instead of containing it behind a safe API.

## Goals / Non-Goals

**Goals:**

- Warn on `pub unsafe fn` free functions and public inherent unsafe methods using the normal Rust ABI.
- Skip private unsafe helpers so unsafe implementation details can remain localized.
- Skip trait implementations where the signature is dictated by a trait contract.
- Keep FFI/export shims out of the first signal unless future evidence proves low-noise treatment.

**Non-Goals:**

- Do not lint every unsafe block in the first implementation.
- Do not prove that a safe wrapper is actually sound.
- Do not rewrite APIs automatically; changing unsafe public contracts requires human review.
- Do not hard-gate public unsafe APIs until downstream inventory shows acceptable precision.

## Decisions

### 1. Start with public Rust unsafe signatures

**Choice:** The first implementation should inspect public free functions and inherent methods whose HIR signature is `unsafe fn`, then skip source snippets that declare `extern` ABI boundaries.

**Rationale:** A public unsafe Rust function is a clear review seam. It exposes unsafe obligations to every caller and is a better first signal than trying to infer unsafe-block containment.

**Alternative:** Flag all unsafe blocks outside modules named `unsafe_boundary` or `ffi`. Rejected for the first slice because module naming conventions vary and safe wrappers legitimately contain unsafe blocks.

**Implementation:** Add a late lint pass under `src/safety/` that checks item and inherent impl-item signatures, uses rustc visibility for publicness, skips trait impl methods, skips macro spans, and skips snippets containing an `extern` ABI declaration.

### 2. Diagnostics are advisory and diagnostic-only

**Choice:** Emit a diagnostic without a structured code suggestion.

**Rationale:** The right fix is architectural: introduce a safe wrapper, move unsafety into a private helper, document invariants, or intentionally retain an unsafe API with review. There is no universally safe edit.

**Alternative:** Suggest removing `unsafe`. Rejected because that would make unsound APIs appear safe.

## Risks / Trade-offs

**FFI false positives** → Skip `extern` signatures in the first implementation and leave FFI-wrapper rules to a separate OpenSpec.

**Legitimate unsafe APIs** → Keep the lint advisory and rely on local `allow` with rationale for APIs that must expose caller obligations.

**Incomplete unsafe containment** → Document that this lint is a narrow public-boundary signal, not a full unsafe-code audit.
