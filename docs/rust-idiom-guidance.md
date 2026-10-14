# Rust idiom guidance

Octet imports Rust idioms only when they make safety, explicitness, boundedness, recoverability, performance evidence, or developer comprehension stronger. Generic pattern names are not enough; every imported pattern must state the Tiger Style outcome it protects.

## Immediate guidance

| Concept | Tiger Style outcome | First enforcement tier |
| --- | --- | --- |
| Newtype | Units, IDs, capabilities, and secrets stop being interchangeable primitives | Advisory until a domain-specific hard gate has fixtures |
| RAII guard | Resource access is mediated by a type that releases or rolls back on scope exit | Human checklist; lint only for narrow known guard shapes |
| Contained unsafe module | Unsafe invariants stay in the smallest auditable boundary with a safe outer API | Advisory/hard depending on configured module allowlist and fixtures |
| Borrowed argument type | APIs accept `&str`, `&[T]`, `Path`, or other borrowed forms when ownership is unnecessary | Shipped warning lint `borrowed_argument_types` with positive and negative fixtures |
| Temporary mutability | Mutable setup ends before read-only use starts | Advisory; avoid hard gating without precise lifetime/use analysis |
| Clone-avoidance | `mem::take`, `mem::replace`, `Option::take`, and narrower borrows replace clone-to-appease-borrow-checker patterns | Advisory until local false-positive rate is known |
| Recoverable consumed argument errors | Fallible APIs that consume expensive/stateful input return it in the error for retry/recovery | Human API review; future lint only for named API families |
| Constrained builder | Many options or same-type parameters get named setters, explicit required fields, and `build()` invariant checks | Already related to parameter/default lints; docs first |
| Command/effects-as-data | Core returns commands/effects, shell executes them, rollback/replay is explicit | Architecture guidance and OpenSpec review |
| FFI wrappers and errors | Foreign APIs see stable handles/codes while Rust keeps lifetimes and rich errors internally | Human checklist plus optional dynamic evidence profile |
| Restrained `Deref` | `Deref` is for smart-pointer/guard ergonomics, not inheritance-like API smuggling | Advisory `deref_polymorphism` lint plus fixtures |
| Small crates | Crates enforce real boundaries such as core/shell, unsafe, FFI, policy, or verifier seams | Architecture guidance; hard only for declared repo roles |

## Review prompts

- What primitive, unit, state, or capability is currently unnamed?
- What invalid operation could become a compile error through a newtype, marker type, sealed trait, or RAII guard?
- What ownership or lifetime rule is currently enforced by convention instead of a safe wrapper?
- Which `clone()` calls are compensating for ownership structure rather than expressing a real copy requirement?
- Does a builder hide safety-sensitive defaults, or does `build()` validate the resulting object?
- Does FFI expose lifetimes or iterator state that the foreign caller can invalidate?
- Is `Deref` making an API look smaller while actually exposing too much behavior?

## Unsafe/trusted-boundary records

Use an unsafe/trusted-boundary record for any new or changed unsafe block,
raw-pointer seam, public `unsafe fn`, FFI wrapper, lifetime-erasing API, foreign
allocation path, or external ownership transfer. The record is review evidence,
not proof by itself; it makes the boundary searchable and names the evidence or
human action still required.

Search label: prefix review records with `UNSAFE-TRUST-BOUNDARY:` followed by
the boundary name.

| Field | Required content |
| --- | --- |
| boundary name | stable label a reviewer can grep during audit |
| public entry point | Rust API, FFI symbol, callback, trait method, or module path that exposes the seam |
| unsafe operation | raw pointer dereference, foreign call, layout assumption, aliasing assumption, lifetime reconstruction, or ownership transfer |
| protected invariant | condition the wrapper or caller must preserve before and after the operation |
| caller obligation | what external callers must uphold when the API cannot be fully safe |
| safe wrapper or exception rationale | normal safe-wrapper path, or why a documented public-unsafe exception is intentional |
| evidence tier | `static_lint`, `policy_check`, `transcript_assertion`, `formal_proof`, or `manual_review_required` as appropriate |
| dynamic evidence selection | selected unsafe/FFI dynamic profile command, fixture, transcript, and acceptance signal, or a scoped non-applicability note |
| reviewer action | decision still required: wrapper accepted, exception accepted, dynamic evidence reviewed, proof card reviewed, or follow-up blocked |
| explicit non-claim | statement that the record is not proof of unsafe correctness, certification, verified runtime behavior, or whole-program safety |

### Public unsafe API path

The normal remediation for a `public_unsafe_api` finding is to expose a safe
public wrapper, keep the unsafe helper private or narrowly scoped, and state the
invariant the wrapper enforces. A public `unsafe fn` may remain only as a
documented exception when a low-level crate intentionally exports caller
obligations and the record names the exception rationale, inherited caller
obligations, evidence tier, reviewer action, and explicit non-claim.

Safe-wrapper example:

```text
UNSAFE-TRUST-BOUNDARY: read-word-wrapper
public entry point: read_word(word: &u32) -> u32
unsafe operation: none at the public boundary; private pointer read isolated in module tests
protected invariant: caller provides a valid shared reference
caller obligation: ordinary Rust borrow rules only
evidence tier: static_lint plus manual_review_required
reviewer action: confirm public API no longer exports unsafe obligations
explicit non-claim: this record is review evidence, not proof of every internal unsafe path
```

Documented-exception example:

```text
UNSAFE-TRUST-BOUNDARY: dma-register-window
public entry point: unsafe fn map_register_window(base: NonNull<u8>, len: NonZeroUsize)
unsafe operation: memory-mapped register projection from a caller-owned address
protected invariant: base and len identify one live device register window for the wrapper lifetime
caller obligation: caller owns the mapped region and prevents aliasing mutable access
safe wrapper or exception rationale: hardware HAL must expose this constructor to board-support crates
evidence tier: manual_review_required plus selected unsafe/FFI dynamic evidence
reviewer action: board-support maintainer accepts caller obligations and reviews transcript
explicit non-claim: the record does not verify unsafe internals or certify the device driver
```

### FFI trusted import prompts

Treat FFI wrappers, raw handles, erased lifetimes, foreign allocation, and
external ownership transfer as trusted import boundaries. The review record
should answer:

- How do handles remain stable across the boundary?
- How do foreign error codes map into Rust errors?
- Who owns allocation and deallocation on each side?
- Which lifetimes or aliasing assumptions are erased or reconstructed?
- Which Rust type prevents foreign callers from observing invalid internal state?

### Dynamic evidence selection

High-risk unsafe or FFI boundary changes should select the unsafe/FFI dynamic
evidence profile, name the exact command or fixture, save the transcript or
artifact, and state the acceptance signal. If dynamic evidence is not applicable
for the scoped change, record why. Starter `cargo octet check` runs remain
lightweight: the starter static check path must not require Miri, sanitizers,
Valgrind-style tools, or Loom unless a separate accepted requirement promotes a
specific dynamic rail.

### Boundary wording

An unsafe/trusted-boundary record does not verify unsafe internals, provide
Salt-style `@trusted` semantics, or make unsafe safe by documentation alone. If
the record cites a formal proof receipt, scope that claim to the declared
property, verifier identity, source and proof binding, receipt, and artifact
identity. Keep lint findings, dynamic evidence, transcript assertions, manual
review, and formal proof in their own evidence tiers.

## Lint promotion rule

Pattern-derived lints stay advisory until they have:

1. A policy entry with source link and Tiger Style rationale.
2. At least one compliant fixture and one noncompliant fixture.
3. A documented false-positive boundary.
4. A staged rollout profile that starts with inventory-only or warning-level reporting.

## Non-goals

- Do not import a pattern because it is common in another language.
- Do not replace clear branches and assertions with clever combinators in safety-critical code.
- Do not hard-gate broad style preferences without fixture-backed precision.
