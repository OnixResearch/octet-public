## Context

Function Object v1 gives selected functions a stable review/provenance identity, and Valence now owns the pure Function Object core. The new rail is broader: every function in a configured Rust scope should appear in a content-addressed inventory unless a policy-declared exclusion applies. Octet should compute this inventory because filesystem walking, Rust parsing, macro/source-shape caveats, and cargo-facing diagnostics are Octet shell responsibilities.

The first slice should avoid pretending Rust becomes Unison. A function address is a reproducible identity for Octet's canonical function record. It is not a proof that two compiler versions assign identical semantics, not a proof of behavior, and not a release gate by itself.

## Decisions

### 1. Split local function address from dependency-closure address

**Choice:** Compute `function_address` from the canonical function record and compute a separate optional closure identity when dependency addresses are available. Unknown external, macro-expanded, generated, or unresolved dependencies become explicit caveats instead of being silently ignored.

**Rationale:** This preserves a useful local content identity for every supported function while leaving room for Unison-like dependency closure without overstating what source-shape extraction can prove today.

### 2. Canonicalize parsed function facts, not raw source bytes

**Choice:** Hash a typed canonical record that includes signature facts, body facts available to the parser/canonicalizer, source envelope identity, canonicalizer identity, span locator, function kind, visibility facts, generics/where-clause facts, attributes admitted by policy, and known dependency references. Comments, formatting, display path aliases, and review labels are metadata unless explicitly admitted to the canonical record.

**Rationale:** Raw text hashing makes formatting churn identity churn. Parsed canonical facts match the stack's content-addressed evidence model and keep identity changes reviewable.

### 3. Support gradual adoption modes

**Choice:** Configure the rail with explicit modes: `off` skips collection, `observe` builds inventory/baseline artifacts without failing, `warn` emits lint-like diagnostics without failing warning-only runs, and `deny` fails on missing functions, duplicate locators, stale addresses, unsupported syntax without an explicit exclusion, malformed BLAKE3 digests, stale source-envelope identities, and receipts that claim semantic correctness.

**Rationale:** The stack can become content-addressed gradually: first observe the function universe, then warn on drift, then deny for selected crates/modules/paths, and finally require the rail for release profiles.

### 4. Make scope and ratchet policy explicit

**Choice:** The rail supports scoped rollout by crate, package, module/path set, workspace member, or changed-function set. Baseline/ratchet mode may allow reviewed existing debt while failing on new unaddressed functions or stale addresses in touched scopes.

**Rationale:** Large codebases should not need a flag day. Reviewable ratchets let teams increase content-address coverage without hiding remaining gaps.

### 5. Export through Kamacite before Valence/Mantle use

**Choice:** Octet emits a Kamacite-compatible receipt artifact containing the function inventory for verified scopes. Octet may also write adapter-ready Valence input after verification, but Kamacite owns the portable receipt/interchange contract and Valence owns stack evidence linkage.

**Rationale:** This keeps Rust extraction in Octet while giving the stack a source-neutral receipt format for later non-Rust producers.

### 6. Preserve Octet's shell/core boundary

**Choice:** Pure canonical record construction and receipt validation should be deterministic in-memory cores. Filesystem walking, cargo metadata, parser invocation, artifact directory writes, stdout/stderr, and process exits stay in the cargo-octet shell.

**Rationale:** Function-address logic should be testable without standing up a workspace, parser process, or release bundle.

## Risks / Trade-offs

- Parser and compiler-version drift can change canonical facts. The receipt must bind canonicalizer/toolchain identity and fail closed on unsupported versions rather than hiding drift.
- Macro expansion and generated code can make a literal “all functions” claim expensive. The first supported universe should be explicit and policy-excludable.
- Baseline mode can become a hiding place for debt if it is not ratcheted. The receipt should record allowed debt explicitly and enforcing profiles should reject new debt.
- Dependency-closure identity is stronger but harder. Keeping it separate avoids blocking the basic per-function address invariant.
