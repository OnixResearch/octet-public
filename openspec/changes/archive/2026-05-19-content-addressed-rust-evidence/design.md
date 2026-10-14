## Context

The content-addressed Rust evidence product boundary belongs in Octet. Octet already owns Rust source scanning, function fingerprints, markers, diffs, docs/examples gates, and content-addressed evidence surfaces. Cairn should be a downstream lifecycle/receipt consumer, not the place where Octet's object model is first defined.

Valence needs Octet to provide a portable Rust evidence layer: selected functions become deterministic, content-addressed review objects with caveated dependency/effect summaries, mutable name pointers, diffs, and export bundles. Trellis can later admit abstract object/name/closure/effect facts. Cairn can later validate replay receipts. Crunch/Mantle and Aspen can consume the exported identity material.

## Goals / Non-Goals

**Goals:**

- Define Octet-owned function-object v1 records.
- Preserve current fingerprint marker workflows while adding object markers.
- Define canonical BLAKE3 object hashing and entropy exclusions.
- Define name pointers, object diffs, caveats, and export bundles.
- Scope HIR/MIR as later supplemental compiler-grounded evidence.
- State downstream ownership boundaries clearly.

**Non-Goals:**

- Build a new Rust compiler, runtime, package manager, or Unison replacement.
- Prove correctness from hashes.
- Require rustc-private HIR/MIR integration for the first milestone.
- Put lifecycle policy, distributed execution, build scheduling, or formal admission proofs inside Octet.

## Decisions

### 1. Octet owns the object model

**Choice:** The authoritative product spec for content-addressed Rust evidence lives in Octet's OpenSpec tree.

**Rationale:** Octet is where extraction, markers, normalization, caveats, and diffs are implemented. If the core contract only lives in Cairn, implementation would be downstream of the wrong repo.

**Alternative:** Keep the primary change in Cairn. Rejected because Cairn should consume and validate receipts; it should not own Rust extraction semantics.

### 2. Function-object v1 starts from source-token evidence

**Choice:** v1 generalizes existing function fingerprints into canonical function objects without requiring HIR/MIR.

**Rationale:** This gives immediate value and keeps claims truthful: review/provenance identity, not compiler-grounded semantic identity.

**Alternative:** Start with MIR or HIR hashes. Rejected because rustc coupling and stability risks would delay the first usable milestone.

### 3. Canonical JSON is the initial wire format

**Choice:** Function objects and bundles use deterministic canonical JSON with explicit schema versions.

**Rationale:** It is easy for Cairn, Trellis adapters, Aspen, and Crunch/Mantle pilots to consume. Nickel contracts can be generated or added later where human-authored policy is needed.

**Alternative:** Start with a custom binary format. Rejected because it slows review and interop.

### 4. Unknown effects fail closed

**Choice:** Source-scanned or unknown dependency/effect information blocks pure-cache eligibility unless stronger reviewed evidence exists.

**Rationale:** Downstream systems need conservative evidence. False purity is more harmful than explicit caveats.

### 5. HIR/MIR are supplemental layers

**Choice:** HIR/THIR/MIR evidence becomes v2+ metadata with exact toolchain/extractor identity.

**Rationale:** Compiler IR is valuable for resolved dependencies, call paths, panic/drop/unsafe/async summaries, and macro expansion boundaries, but it cannot be treated as stable cross-toolchain identity.

**Implementation boundary:** `docs/content-addressed-rust-evidence-boundaries.md` is the Phase 7 contract. Future HIR records must reference the canonical `octet.function-object.v1` hash and name rustc/toolchain, extractor, schema, resolved dependency summaries, and unresolved/macro/generated/cfg caveats. Future MIR records must also name rustc/toolchain, extractor, schema, and explicit call/panic/drop/async/unsafe/allocation/effect summary support or unknown caveats. Neither layer replaces source-token v1 review identity.

### 6. Downstream consumers do not own extraction

**Choice:** Octet remains the Rust extraction/object authority; Trellis, Cairn, Crunch/Mantle, and Aspen consume bounded evidence at their own layers.

**Rationale:** Keeping extraction in Octet prevents downstream lifecycle, proof, build, or distributed-systems consumers from redefining Rust object semantics.

**Implementation boundary:** Cairn consumes Octet bundles for lifecycle/replay policy. Trellis owns abstract admission models. Crunch/Mantle and Aspen pilots may cite Octet object/bundle hashes as evidence identity, but Octet hashes alone do not prove build correctness, binary reproducibility, distributed execution correctness, or live-system safety.

## Risks / Trade-offs

**Object schema churn** → Include `schema`, `normalization.version`, and extractor/toolchain identity fields.

**Overclaiming semantics** → Require non-correctness language in object output, docs, and receipts.

**Macro opacity** → Emit machine-readable macro/generated-code caveats and fail closed for purity claims.

**Downstream confusion with Cairn ownership** → State that Cairn validates lifecycle/replay receipts while Octet owns extraction/object emission.

## Validation Plan

- `openspec validate content-addressed-rust-evidence --strict --json`
- Proposal/design/tasks gates for the change.
- Later implementation: focused `cargo test -p cargo-octet` cases for canonical hashing, markers, pointers, diffs, and bundles.
- Later docs/examples gate: `nix build .#checks.x86_64-linux.docs-examples --no-link -L`.
