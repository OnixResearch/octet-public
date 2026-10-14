## Context

Octet already models formal proof as a bounded evidence tier: a proof result must name its verifier, model or checked scope, receipt, and covered property. `octet-standards` also has `ProofRail` metadata with backend, command, source paths, proof paths, pass signal, trust boundaries, BLAKE3 command/source binding, receipt provenance, and verifier/toolchain obligations.

The current gap is product integration. The `verus/` directory is experimental, the verifier is not pinned by Octet, `cargo octet` has no proof command, and proof-rail execution can synthesize receipts for checking without giving operators a persistent artifact bundle to promote.

## Approach

1. Keep Verus as a standards/proof rail, not a Dylint lint. Dylint remains the static source-shape edge; Verus is evidence for explicitly selected properties.
2. Add a Nix-pinned verifier wrapper that is the only accepted Verus identity for `formal_proof` evidence. The wrapper prints deterministic verifier/toolchain identity material that can be recorded in receipts.
3. Extend the proof-rail execution shell to persist receipts. The pure core validates receipt text, configured pass signals, BLAKE3 command/source bindings, provenance lines, and verifier/toolchain lines over in-memory inputs. The CLI shell owns process execution, filesystem writes, artifact-dir layout, and stdout/stderr.
4. Add `cargo octet prove` as a thin orchestration shell over the existing standards verification family. It selects proof rails, invokes the pinned wrapper, writes receipts under `artifacts/octet/proofs/`, and emits provenance records with `evidence_tier = formal_proof` only when verifier and proof artifact identity are present.
5. Register Verus proof receipt families as Rust-derived because receipts are runtime outputs. Keep raw verifier internals, secrets, and environment-specific paths out of Nickel except as digests, labels, allowlists, or explicit non-candidate rationale.
6. Add a verified-module coverage check that treats `verified/` source paths and explicit formal-proof claims as obligations. A selected verification profile must find a matching declared proof rail, or record/fail a `manual_review_required` boundary instead of silently implying proof.
7. Replace the skeleton Verus proof with one tiny Octet-owned target that exercises the full route: source mapping, proof file, pinned command, pass signal, receipt, provenance, and artifact verification.

## Key Decisions

### Reuse ProofRail instead of inventing a Verus-specific registry

`ProofRail` already has the required fields. The Verus integration should tighten backend-specific validation for `backend = "verus"`, not create a parallel proof registry.

### Persist receipts before trusting promotion evidence

A proof run that only exists in process memory is useful for local checking but weak as release evidence. `cargo octet prove` should write receipts and provenance, then `cargo octet artifact verify` should be able to fail closed on stale, malformed, or incomplete proof artifacts.

### Keep formal proof scoped

A passing Verus rail supports only the configured source paths, proof paths, verifier identity, and property labels. Documentation and emitted summaries must continue to say that Octet does not prove whole-program correctness.

### Make ordinary lint adoption unaffected

Starter consumers should not need Verus installed. Verus runs only when a verification profile, proof command, or `cargo octet prove` explicitly selects proof rails.

## Risks

- **Verifier output churn**: Verus pass wording can change. Mitigate by pinning the verifier and making the pass signal part of the proof-rail contract and fixture.
- **False proof promotion**: Teams may cite a policy check as proof. Mitigate with `formal_proof` validation requiring verifier and proof artifact identity.
- **Ambient binary drift**: A local `verus` on `PATH` may differ from the pinned one. Mitigate by recording verifier digest/toolchain identity and rejecting mismatches.
- **Large scope creep**: Proving core Octet behavior is a major effort. Mitigate with one tiny proof target and proof-rail plumbing first.
- **Receipt over-capture**: Raw logs may leak environment or source excerpts. Mitigate by storing bounded stdout/stderr only where required and keeping acceptance receipts structured.

## Validation

- Positive: pinned Verus proof fixture produces a persistent receipt, exact pass signal, BLAKE3 command/source bindings, verifier identity, `formal_proof` provenance, and a clean artifact verification result.
- Negative: missing proof file, missing pass signal, stale source hash, wrong verifier identity, unknown backend, unbacked `verified/` module claim, and `formal_proof` provenance without proof artifact identity all fail closed.
- Regression: ordinary `cargo octet check` and starter profiles remain usable without running Verus.
