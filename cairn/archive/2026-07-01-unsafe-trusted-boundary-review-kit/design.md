## Context

Octet's current surfaces already point in the right direction: `public_unsafe_api` flags public unsafe functions that leak caller obligations, Rust idiom guidance recommends contained unsafe modules, concurrency review asks for unsafe quarantine, and dynamic profiles name unsafe/FFI evidence. The missing capability is a single review record that ties these surfaces together.

Salt's `unsafe`/`@trusted` language is not transferable as syntax. The transferable idea is making unverified boundaries explicit, searchable, and evidence-backed.

## Approach

1. **Review record over new syntax.** Define an unsafe/trusted-boundary record in documentation and fixtures. The record names the boundary, invariant, public API shape, evidence, and non-claim.
2. **Use existing lint as entry point.** Link `public_unsafe_api` findings to the record shape: either hide unsafe behind a safe wrapper or document the exception and review action.
3. **FFI as trusted import boundary.** Treat FFI wrappers, raw handles, erased lifetimes, and external ownership as trusted imports that require stable handles, explicit error mapping, and ownership/lifetime notes.
4. **Dynamic evidence remains selected.** High-risk boundaries should select the unsafe/FFI dynamic evidence profile, but ordinary starter runs must not require heavyweight tools.
5. **Boundary wording checks.** Validation rejects claims that Octet verifies unsafe internals, provides Salt-style `@trusted`, or makes unsafe safe by documentation alone.

## Decisions

### 1. Keep the capability documentation-first

**Choice:** Start with docs and deterministic inventory checks, not lint expansion.

**Rationale:** The existing `public_unsafe_api` lint already catches the highest-signal source shape. Broader unsafe/FFI review requires human context and dynamic evidence selection.

### 2. Make exceptions reviewable

**Choice:** Public unsafe may remain only when a boundary record names why a safe wrapper is not appropriate, what obligations callers inherit, and which reviewer action is required.

**Rationale:** Some low-level crates intentionally expose unsafe APIs. Octet should make those obligations explicit rather than banning them without context.

### 3. Separate evidence from proof

**Choice:** Unsafe-boundary records may cite lint findings, dynamic evidence, transcript assertions, proof receipts, or manual review, but the record itself is not proof.

**Rationale:** This preserves Octet's evidence-tier model and avoids Salt-style verifier overclaims.

## Risks / Trade-offs

- **Too much process:** A heavy checklist could slow low-risk changes. Mitigate by requiring the full record only for new or changed unsafe/FFI boundaries and by allowing explicit non-applicability notes.
- **False confidence:** A completed checklist can look like proof. Mitigate with required non-claim and evidence-tier fields.
- **Duplicated guidance:** Unsafe guidance appears in multiple docs. Mitigate by creating one canonical record definition and linking from the other docs.

## Validation

- Positive inventory confirms the canonical record defines every required field.
- Positive link check confirms `public_unsafe_api`, Rust idiom guidance, concurrency review, and dynamic evidence docs point to or align with the record.
- Positive fixture check confirms at least one safe-wrapper example and one documented-exception example are present.
- Negative wording check rejects claims that Octet verifies unsafe code, provides Salt-style `@trusted`, or makes unsafe safe by documentation alone.
- Cairn `validate` and proposal/design/tasks gates pass.
