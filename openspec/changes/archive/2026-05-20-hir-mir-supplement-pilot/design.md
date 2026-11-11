## Context

Function Object v1 is implemented as source-token review/provenance evidence with explicit caveats and no correctness claims. The current HIR/MIR contract metadata names planned supplement families but reports them as `unsupported-planned` until deterministic extractor fixtures exist. The repo also has an aggregate Valence object-evidence gate that composes the selected corpus rails.

## Goals / Non-Goals

**Goals:**

- Introduce the first bounded HIR/MIR supplement pilot without weakening Function Object v1 identity.
- Make supplement records deterministic, toolchain-scoped, and fixture-backed.
- Keep contract metadata honest by distinguishing `pilot-supported` families from `unsupported-planned` families.
- Add fail-closed negative fixtures for missing object references, mismatched toolchains, missing required fields, and unsupported families.

**Non-Goals:**

- Full Rust compiler-IR semantic modeling.
- Cross-rustc, cross-target, or optimization-level stability claims.
- Any correctness, safety, semantic-equivalence, or pure-cache proof.
- Replacing existing object corpus receipt/check behavior.

## Decisions

### 1. Supplements attach to Function Object v1 objects

**Choice:** Every pilot supplement record references an existing `octet.function-object.v1` `object_hash` and selected source path/function identity.

**Rationale:** This preserves the existing source-token object as canonical v1 review identity while allowing HIR/MIR to add review context.

**Alternative:** Emit standalone HIR/MIR identities. Rejected because it creates ambiguous identity ownership and invites replacement/semantic-equivalence overclaims.

### 2. Pilot evidence is exact-toolchain scoped

**Choice:** Records include rustc/toolchain identity, extractor name/version, schema version, target/config context where relevant, and explicit unsupported/unknown caveats.

**Rationale:** HIR/MIR are compiler-version-coupled; deterministic fixtures are meaningful only under named extraction context.

**Alternative:** Normalize HIR/MIR into a toolchain-independent hash. Rejected for the pilot because no verifier exists for that claim.

### 3. Contract metadata is status-driven

**Choice:** Contract metadata uses closed statuses such as `unsupported-planned` and `pilot-supported`; only fixture-backed pilot families can leave `unsupported-planned`.

**Rationale:** Existing fail-closed behavior is valuable and should degrade only when deterministic evidence exists.

**Alternative:** Treat the presence of an extractor command as support. Rejected because commands without replay fixtures are not durable review evidence.

### 4. Verification starts with small fixtures

**Choice:** The first implementation uses selected fixture functions and checked-in positive/negative supplement receipts before any broad corpus expansion.

**Rationale:** Small fixtures keep the pilot reviewable and prevent accidental broad support claims.

## Risks / Trade-offs

- **Rustc-private churn** → Keep the pilot pinned to exact toolchain identity and report unsupported status outside that context.
- **Overclaim wording drift** → Assert boundary fields and contributor-guide language in tests/docs checks.
- **Fixture brittleness** → Keep fixtures minimal and refresh them only with explicit evidence tasks.
- **Scope creep into downstream policy** → Limit this change to Octet-owned supplement evidence surfaces.
