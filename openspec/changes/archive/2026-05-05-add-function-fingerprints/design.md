## Context

Tigerstyle already emphasizes verified pure primitives, protocol state transitions, explicit evidence, and fixture-backed documentation. Existing evidence identifies files, line numbers, commands, and transcripts, but it does not mechanically prove that a reviewed function body is unchanged after refactors or follow-up edits.

Function fingerprints adapt content-addressed-code ideas to Rust by hashing selected function signatures and normalized token bodies. The attribute or marker is a review gate: it proves that the function body still matches the reviewed body, and it forces an intentional update when the function changes.

## Goals / Non-Goals

**Goals:**
- Provide a selective function-fingerprint workflow for safety-critical functions.
- Keep fingerprints stable across rustfmt, comments, line moves, and file moves where possible.
- Detect semantic source-token changes to fingerprinted functions.
- Support both first-class attributes and stable comment markers.
- Integrate with OpenSpec evidence and CI without imposing fingerprints on all consumers.

**Non-Goals:**
- Recreate Unison's global codebase model.
- Treat fingerprints as proof of correctness.
- Hash macro expansion output in the first implementation.
- Require nightly-only tool attributes for stable crates.

## Decisions

### 1. Use BLAKE3-prefixed normalized function fingerprints

**Choice:** Fingerprints use `b3:<hex>` values produced from a domain-separated normalized representation.

**Rationale:** BLAKE3 is already part of Tigerstyle standards vocabulary for audit integrity and is fast, deterministic, and review-friendly.

**Alternative:** SHA-256 is ubiquitous but slower and not aligned with existing Tigerstyle BLAKE3 references.

**Implementation:** Hash input format is versioned as `tigerstyle-function-fingerprint-v1`, followed by normalized visibility, signature, generics, where clause, and body tokens.

### 2. Ignore fingerprint metadata during hashing

**Choice:** The checker MUST ignore its own `#[fingerprint(...)]` attribute or adjacent marker comment when computing the function hash.

**Rationale:** Otherwise writing the hash would change the hash.

**Alternative:** Store hashes only in a sidecar manifest. Rejected as the only form because inline review gates are easier to see during code review.

**Implementation:** Normalization strips known fingerprint attributes and marker comments before tokenization.

### 3. Support attribute and comment-marker modes

**Choice:** Accept both:

```rust
#[fingerprint("b3:...", review = "protocol-transition")]
pub fn apply_event(...) -> ... { ... }
```

and:

```rust
// tigerstyle-fingerprint: b3:... review=protocol-transition
pub fn apply_event(...) -> ... { ... }
```

**Rationale:** Attributes are the best developer experience for first-class Tigerstyle crates, while comment markers avoid proc-macro and stable-library friction.

**Alternative:** Attribute only. Rejected because some published crates cannot accept custom attributes without additional dependency or compiler constraints.

### 4. Warn on macro calls inside fingerprinted functions by default

**Choice:** A fingerprinted function containing source macro invocations emits an advisory warning unless the marker explicitly allows macros.

**Rationale:** Source-token hashing will not detect changes inside macro definitions. Critical reviewed logic should avoid hidden macro bodies unless intentionally accepted.

**Alternative:** Hash expanded HIR. Deferred because it is more toolchain-coupled and may be noisy across compiler versions.

### 5. Make initial enforcement advisory except for explicit check mode

**Choice:** Normal `cargo tigerstyle check` may report fingerprint metadata findings when configured, but hard failure belongs to `cargo tigerstyle fingerprint --check` or a consumer-selected CI gate.

**Rationale:** Fingerprints are useful only when teams explicitly select critical functions and own the review/update workflow.

**Alternative:** Default hard gate for all fingerprints. Rejected as too disruptive during rollout.


### 6. Reuse parser/token crates only at the right boundary

**Choice:** Keep `blake3` as the digest implementation. For standalone CLI fingerprint write/check paths, evaluate `syn`/`proc-macro2` tokenization or rust-analyzer `ra_ap_syntax` only behind a small normalization adapter. For Dylint-time checks, prefer rustc HIR/token data already available to the lint pass.

**Rationale:** Function fingerprints need stable normalized tokens and must ignore their own marker. Parser crates can help outside rustc, but adding them to the hot lint path may duplicate data rustc already has.

**Implementation:** Define a normalization-version trait/module with fixture coverage before choosing one parser backend permanently. Macro-heavy bodies should remain warning-only unless the normalization backend includes explicit macro-expansion semantics.

**Alternative:** Hash raw source spans directly. Rejected because raw source hashing is brittle, self-referential around markers, and too sensitive to comments/formatting.

## Risks / Trade-offs

**Token normalization drift** → Version the hash format and make version changes explicit in diagnostics.

**False confidence** → Docs and diagnostics must say fingerprints prove source identity, not correctness.

**Macro blind spots** → Warn by default and require `allow_macros = true` / `allow-macros=true` for intentional exceptions.

**Attribute compatibility** → Provide comment fallback and avoid requiring nightly-only `register_tool` syntax.

**Bulk churn** → Keep scope selective and support `--diff` so reviewers see exactly which reviewed functions changed.

## Validation Plan

- Positive fixture: `--write` inserts or updates a missing fingerprint and `--check` passes immediately after.
- Stability fixture: rustfmt/comment/line-number-only changes preserve the fingerprint.
- Negative fixture: a token/body/signature change causes a mismatch with expected/actual hashes.
- Macro fixture: a macro call inside a fingerprinted function emits the macro caveat diagnostic unless explicitly allowed.
- Stable fallback fixture: comment markers validate without requiring a proc macro dependency.
- Documentation/policy verification: standards docs and registry name the policy, enforcement tier, metadata, and non-proof caveat.
