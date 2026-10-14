# Tiger Style Standards Contract

## Purpose

This repo enforces a reusable Rust safety profile through two explicit seams:

- the Dylint lint engine in `src/` and `cargo-octet/`
- the policy registry and standards checker in `standards/` and `crates/octet-standards/`

The machine-readable policy index lives in
[`standards/policies/registry.ncl`](../../standards/policies/registry.ncl). It
links Tiger Style source sections to stable policy IDs, enforcement levels,
current/target enforcement modes, structural mechanisms, proof rails, seed
manifests, checker ownership, suppression requirements, and planned fixture
evidence.

## Functional Core / Imperative Shell

- Standards policy logic MUST live in `crates/octet-standards/src/` as pure
  rule functions over plain inputs.
- Filesystem walking, JSON loading, CLI parsing, and exit-code mapping MUST stay
  in the `octet-standards` binary shell.
- Dylint lint implementation MUST stay in `src/`; standards policy checks MUST
  not be folded into the compiler-plugin crate unless they need rustc HIR.
- `cargo-octet/` MUST remain the lint runner seam for downstream projects.

## Policy Registry

- Policy IDs use the `octet.<pillar>.<rule>` namespace.
- Suppressions use `octet-allow:` comments and MUST cite policy ID, source
  section, owner, rationale, review trigger, and safer default for hard rules.
- Hard policies need positive and negative fixtures before becoming blocking.
- Policies that target `structural` MUST export at least one mechanism contract
  such as crate-boundary, capability-api, bounded-primitive, or type-state.
- Capability-api mechanisms export forbidden ambient authority APIs, declared
  scope classes, capability families, core scopes, shell constructor scopes,
  approved capability names, injected traits/tokens, shell interpreters, and
  deterministic handler evidence so the architecture checker can reject
  core-side ambient effects and emit capability reports from generated policy
  artifacts. The report validates declared policy and observed source shapes
  only; it is not a Rust effect system, transitive purity proof, or
  whole-program runtime-effect absence claim.
- Bounded-primitive mechanisms export forbidden unbounded APIs, approved bounded
  constructors/wrappers, capacity witnesses, and backpressure/overflow witnesses.
- Policies that target `verified` MUST export a proof rail with a command argv,
  exact or globbed source/proof/receipt path mappings, pass signal,
  trust-boundary accounting, verifier mode, receipt-binding mode, and receipt
  provenance obligations. Verification-family checks treat source mappings,
  proof artifact pass signals, and receipt paths as hard artifact obligations:
  by default, a selected verified policy fails unless the declared source, proof,
  and receipt files are present in the checked source tree and the proof artifact
  contains the exact configured pass signal. Rails with
  `receipt_binding = "blake3-command-and-source"` also require command and source
  BLAKE3 bindings matching the configured argv and checked source contents.
  Rails with `receipt_provenance` configured also require matching
  `proof-receipt-schema`, `proof-backend`, `proof-runner`, `proof-environment`,
  and configured `trust-boundary` entries. Rails with `receipt_verifier`
  configured also require exact verifier/toolchain accounting for verifier,
  solver, binary, build closure, Nix derivation, Nix closure, Nix closure graph,
  and toolchain identity. Fixture rails MUST mark `verifier_mode = "fixture"`;
  production Verus rails MUST mark `verifier_mode = "production"` and pin real
  verifier, solver, and toolchain identity. If verifier build-closure, Nix
  closure, or Nix closure graph paths are configured, the corresponding digest is
  derived from checked manifest or graph file contents instead of trusting
  ambient build state. `cargo octet prove` executes only configured rail argv,
  captures bounded stdout/stderr artifacts, writes structured JSON receipts at
  configured receipt paths, and then emits `formal_proof` provenance only after
  the same pass-signal, binding, provenance, verifier/toolchain, binary, and
  closure gates pass. Command execution is explicit opt-in; read-only receipt
  checking remains the default for consumer safety.
- Proof-relevant seed manifests MUST remain untrusted until a local acceptance
  command or human/oracle review records provenance and re-verification.
- Advisory and human-process-only policies stay visible without pretending static
  tooling can judge them.

## Testing Rules

- Test suites MUST include both positive and negative cases.
- Standards checker tests MUST exercise pure rule functions directly.
- Fixture rails MUST prove compliant and noncompliant examples separately.
- Error-path tests MUST prove the exact failure surface: returned error variant,
  message, and no hidden panic.

## Numeric and Unsafe Discipline

- Non-obvious numeric literals MUST be named constants, parameters, or config
  values.
- Every unsafe block MUST be documented, or unsafe MUST be absent.
- Core validation helpers SHOULD prefer named input structs over ambiguous
  same-typed parameter lists.

## Verification Commands

Run these inside `nix develop` or through `nix develop path:. -c ...`:

```bash
cargo test -p octet-standards
cargo check --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
nix run .#octet-policy-validate
nix run .#octet-standards -- --help
nix build .#checks.x86_64-linux.policy-registry --no-link
nix build .#checks.x86_64-linux.standards-fixtures --no-link
nix run .#cargo-octet -- check
```
