# Octet

Octet is the evidence-backed checked path for agent-written and human-written Rust systems code.

Rust source flows through `cargo octet check` and checked-in policy to reviewable evidence artifacts and consumer CI or release gates.

Octet provides a Rust-compatible toolchain, lint checks, standards checks, policy, and evidence artifacts.

It applies [Tiger Style](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md) rules:

- explicit functional-core and imperative-shell boundaries
- bounded resource use
- positive names
- compact functions
- reviewable suppressions

The checked path is:

```text
Rust source → cargo octet check → standards and policy → evidence artifacts → consumer gates
```

Read [`docs/trust-boundary-evidence.md`](docs/trust-boundary-evidence.md) for evidence tiers, provenance, scope, and deployment boundaries.

Use [`docs/trust-boundary-evidence.md#checked-contract-ladder`](docs/trust-boundary-evidence.md#checked-contract-ladder) to review the evidence ladder.

Use [`docs/trust-boundary-evidence.md#verus-proof-path-lane`](docs/trust-boundary-evidence.md#verus-proof-path-lane) to map a requirement or policy ID to Verus proof rail evidence.

Read [`docs/adoption-story.md`](docs/adoption-story.md) for the card-oriented public story.

Read [`docs/examples/first-checked-rust-patch/README.md`](docs/examples/first-checked-rust-patch/README.md) for the first checked Rust patch walkthrough.

Octet treats code as closed by default across APIs, effects, capabilities, proofs, and review surfaces. Interaction requires a complete interface.

Octet ships 77 Dylint lints across six domains. A separate standards registry handles policy traceability and other source-shape rules.

A Nickel catalog defines nominal domains for primitive aliases, raw core values, and wrapper bypasses. Read [`docs/type-driven-domain-modeling.md`](docs/type-driven-domain-modeling.md).

Checked-path configuration uses the exact reviewed Nickel cohort. Read [`docs/nickel-toolchain.md`](docs/nickel-toolchain.md).

## Status and prerequisites

Octet is under active development. Nix pins Rust, Dylint, Verus, solvers, and support tools.

Direct Cargo use is an advanced path. It must reproduce the same pins.

Octet pins exact Valence and Bounded Exec revisions in Cargo and Nix. This public export resolves those pins anonymously from the public mirrors and carries `NOTICE`, `THIRD_PARTY_NOTICES.md`, and `LICENSES/`. Imported standards prose is excluded from the public source tree and retained only in the attribution inventory. Cached local builds are not accepted as anonymous checkout evidence.

The [Mantle producer handoff](docs/aeneas-consumer-contract.md#mantle-producer-handoff)
exports an immutable ordinary Nix job with an explicit legacy or Nix v4 wire
contract. Acquisition is not proof execution or external acceptance.

## Quick start

1. Copy `templates/dylint.toml` to the consumer root as `dylint.toml`.
2. Add runner metadata to the workspace `Cargo.toml`.

```toml
[workspace.metadata.octet]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

3. Run Octet.

Installed Cargo subcommand:

```bash
cargo octet check
```

Public Nix flake:

```bash
nix run github:OnixResearch/octet#cargo-octet -- check
```

Local checkout:

```bash
nix run path:/path/to/octet#cargo-octet -- check
```

Octet's own policy checks each workspace library with `--workspace --lib`.

Projects can opt in to explicit FCIS and Hexagonal policy with:

```toml
[workspace.metadata.octet]
architecture_policy = "octet-architecture.ncl"
```

Read [`docs/project-architecture-policy.md`](docs/project-architecture-policy.md) before activation. Without this pointer, Octet makes no project architecture claim.

An active `admission` policy makes `cargo octet check` run strict lint, the architecture gate, and its explicitly bound effect closure in order.

A self-check can be `warning-only`. The [warning budget](docs/self-check-warning-budget.md) records accepted finding IDs and rejects new IDs.

This scope does not cover tests, binaries, examples, dependencies, runtime behavior, whole-program correctness, or release eligibility.

Write SARIF and sidecars with:

```bash
cargo octet check --output-format sarif --artifact-dir artifacts/octet
```

## Strict Rust profiles

Named profiles set reviewed lint floors, function-address minimums, and BLAKE3 evidence.

```bash
cargo octet check --profile robust --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet
```

The profiles are:

- `robust`
- `bounded-systems`
- `deterministic-core`

These profiles enforce direct source-shape policy. They do not prove transitive effects, whole-program behavior, or release eligibility.

`deterministic-core` requires `core_source_scopes` and a reviewed deny-mode function-address baseline.

Read [`docs/strict-rust-profiles.md`](docs/strict-rust-profiles.md).

## Assurance profiles

An assurance profile links mechanisms, assumptions, trusted-base elements, evidence scopes, commands, and non-claims.

The OnixOS lifecycle pilot is available through:

```bash
nix build .#octet-assurance-profile-onixos-lifecycle --no-link --print-out-paths
nix build .#checks.x86_64-linux.assurance-profile-onixos-lifecycle --no-link -L
```

The package includes generated JSON, Nickel source, its contract, and a Nickel Export BLAKE3 manifest.

It is not a strict Rust profile, cumulative proof, security score, or release decision.

Read [`docs/assurance-profiles.md`](docs/assurance-profiles.md).

## Effect-closure admission

The opt-in closure rail extends a successful `deterministic-core` result. It uses conservative source-token propagation and reviewed provider summaries.

```bash
cargo octet admit --profile deterministic-core --closure \
  --artifact-dir target/octet-admission \
  -- --target x86_64-unknown-linux-gnu
cargo octet admit verify --artifact-dir target/octet-admission
```

Unknown effects, opaque constructs, stale identities, forbidden role edges, and provider mismatches fail closed.

The receipt is bounded source-token evidence. It is not complete name resolution, macro closure, MIR closure, dependency proof, runtime determinism, or release eligibility.

Read [`docs/effect-closure-admission.md`](docs/effect-closure-admission.md).

## External evidence rails

Optional rails collect external tool output into separate receipt bundles.

```bash
cargo octet evidence collect --rail dependency-audit --input target/deny.json --config deny.toml --locked --artifact-dir artifacts/dependency-audit
cargo octet evidence collect --rail clippy-co-reporting --run --profile production-safety --workspace-root . --artifact-dir artifacts/clippy-restrictions
cargo octet evidence collect --rail ast-grep --run --config standards/ast-grep/generated/profile.json --workspace-root . --artifact-dir artifacts/ast-grep
cargo octet evidence collect --rail wasm-artifact --input module.wasm --config standards/wasm-artifact/generated/profiles.json --profile portable-component-baseline --artifact-dir artifacts/wasm-artifact
cargo octet evidence collect --rail wasm-artifact --input module.wasm --config standards/wasm-artifact/generated/profiles.json --profile spacewasm-mvp --external-report chaoscontrol.json --artifact-dir artifacts/spacewasm
cargo octet evidence collect --rail charon-semantic --run --workspace-root . --config standards/charon-semantic/generated/profile.json --profile octet-charon-semantic-safe-add-v1 --external-report standards/charon-semantic/generated/comparison-parent.json --artifact-dir artifacts/charon-semantic
cargo octet evidence collect --rail hegel-property --profile octet-pure-core-laws-v1 --config standards/hegel/profiles.json --workspace-root . --run --locked --artifact-dir artifacts/hegel-property
cargo octet evidence collect --rail mutation-testing --profile cargo-mutants-octet-metadata --config standards/mutation/profiles.json --input .octet/mutants-output/mutants.out/outcomes.json --external-report evidence/mutation-source-manifest.json --workspace-root . --artifact-dir artifacts/mutation
cargo octet evidence collect --rail coverage-baseline --profile cargo-llvm-cov-octet-lib --config standards/coverage/profiles.json --input target/coverage.json --baseline baselines/coverage.json --external-report evidence/coverage-source-manifest.json --workspace-root . --artifact-dir artifacts/coverage
cargo octet evidence collect --rail benchmark-evidence --profile octet-metadata-benchmark --config standards/benchmark/profiles.json --input evidence/benchmark-input.json --workspace-root . --artifact-dir artifacts/benchmark
cargo octet artifact verify --artifact-dir artifacts/dependency-audit
```

For Mantle-produced Molten components, select `molten-component-baseline`
explicitly. It permits the declared Preserves carrier type import, bulk memory
and padded `call_indirect` table immediates. `call-indirect-overlong` enables
only that encoding feature; it does not enable reference types, SIMD or all of
Wasm 2. The import-free `portable-component-baseline` remains unchanged.
Profile and registry identities change with policy; downstream toolchains must
pin the new identities and rerun exact-byte evidence. Static admission grants
no runtime authority or execution-correctness claim.

The Clippy rail uses an embedded reviewed catalog. It does not enable the full `clippy::restriction` group.

Receipts bind catalog and profile identities, arguments, findings, and tool status. They remain complementary static-lint evidence.

Read [`docs/clippy-restriction-profiles.md`](docs/clippy-restriction-profiles.md).

Read [`docs/benchmark-evidence.md`](docs/benchmark-evidence.md) for typed workloads, bounded samples, resource observations, signal oracles, and public-claim limits.

Read [`docs/hegel-property-evidence.md`](docs/hegel-property-evidence.md) for the checked Hegel producer, law catalog, replay artifacts, and bounded claims.

Read [`docs/mirai-semantic-pilot.md`](docs/mirai-semantic-pilot.md) for the pinned source package, selected corpus, and blocked output-contract decision.

Read [`docs/mutation-testing-evidence.md`](docs/mutation-testing-evidence.md) for typed mutation profiles and complete native outcomes.

Read [`docs/coverage-baseline-evidence.md`](docs/coverage-baseline-evidence.md) for typed coverage profiles and exact-count comparison.

## Pre-commit and exit status

Copy `templates/pre-commit-config.yaml` to a consumer repository. Pin Octet as the hook source.

The `octet-deny-all` hook promotes each Octet lint to `deny`.

Exit status values are stable:

- `0`: clean or warning-only
- `1`: error-level findings, including denied Octet lints that make rustc fail the compile
- `2`: integration or Cargo failure, including any compiler error that is not an Octet finding

For direct Cargo use, install the runner once:

```bash
cargo install --path /path/to/octet/cargo-octet
```

Then use [`docs/consumer-guide.md`](docs/consumer-guide.md) for the pinned `OCTET_DYLINT_REV` setup.

Octet reports independent policy, manifest, and evidence-input errors in deterministic order. Invalid entries do not affect dependent evidence, and the command fails closed. `dylint.toml` is the exception: unknown keys, unknown lint names, and invalid levels print a warning and are ignored.

Run standards checks with:

```bash
nix run .#octet-policy-validate
nix run .#octet-standards -- --help
```

Policy reports include one execution record per registry policy. A local checker
that ran on an applicable scope reports `passed` or `failed` with its scope and
linked diagnostics; a local policy with no implemented checker reports
`unsupported`; externally owned tools and human review report `not_run` with
their ownership. `octet-standards check ... --strict-policy-coverage` fails when
a selected hard policy lacks a completed local check, including a required
external or human check that was not actually run. Default mode keeps the
existing diagnostics-only exit behavior.

## Formal proof rails

Proof rails are opt-in. `cargo octet prove` writes verifier receipts and `proof-cards.json`.

Formal-proof provenance applies only to the exact source, proof, verifier, solver, command, assumptions, and property scope.

Starter `cargo octet check` does not require Verus.

Octet includes fixture rails and production-mode pilots. Each pilot proves only its named property.

Imported Trellis evidence remains recorded-only until Octet reruns the proof and checks its own receipt.

The Verus toolchain profile is `standards/verus-toolchain/profile.ncl`. It also pins `octet-verusfmt`.

```bash
nix build .#octet-verus-toolchain-profile .#octet-verusfmt --no-link -L
nix run .#octet-verusfmt -- --identity
```

```bash
cargo octet prove --list \
  --policy-artifacts-dir artifacts/policy \
  --source src/verified
cargo octet prove --explain \
  --policy-artifacts-dir artifacts/policy \
  --source src/verified \
  --rail octet.verus_proof_rails.addition-bound-production#rail-0
cargo octet prove \
  --policy-artifacts-dir artifacts/policy \
  --source src/verified \
  --rail octet.verus_proof_rails.addition-bound-production#rail-0 \
  --artifact-dir artifacts/octet-proofs
cargo octet proof-ref scaffold \
  --proof-cards artifacts/octet-proofs/proof-cards.json \
  --proof-or-property-id octet.verus_proof_rails.addition-bound-production#rail-0 \
  --reference-id unchecked-narrowing-range-proof \
  --consumer-context 'allow tigerstyle::unchecked_narrowing at src/codec.rs:42' \
  --consumer-source src/codec.rs \
  --anchor 'octet-proof-ref: unchecked-narrowing-range-proof -> octet.verus_proof_rails.addition-bound-production#rail-0' \
  --output proof-ref.json
cargo octet proof-ref validate \
  --proof-cards artifacts/octet-proofs/proof-cards.json \
  --ref proof-ref.json
cargo octet artifact verify --artifact-dir artifacts/octet-proofs
```

`cargo octet prove --candidates` lists Tiger Style allowances and waivers in the
checked source that have no validated proof-ref, grouped by lint and ranked by
lint severity and trust-boundary membership. Use `--json` for the stable
`octet-proof-candidates/v1` report. Only existing rails are mapped; every other
candidate is reported as unsupported. A rail that still passes against a mutant
source is recorded as `property_not_demonstrated` by
`cargo octet prove --record-negative-check`; `property_demonstrated` requires
typed verifier evidence with a passing baseline and a mutant rejection
attributed to the named obligation, and `proof-ref validate` recomputes the
decision from the retained artifacts. The guided agent path lives in
[`docs/skills/prove-a-candidate/SKILL.md`](docs/skills/prove-a-candidate/SKILL.md).

`cargo octet check` and `cargo octet prove` write separate bundles, so give each
its own artifact directory; `cargo octet prove` refuses a directory that holds a
`check` bundle. Both commands, and `cargo octet evidence collect`, prune only
their own earlier outputs from a reused bundle and refuse an `--artifact-dir`
that is the workspace root or one of its ancestors.

The reusable Aeneas consumer profile binds exact downstream source, generated
artifacts, a maintained theorem, and the existing pinned provider. See
[`docs/aeneas-consumer-contract.md`](docs/aeneas-consumer-contract.md).

Legacy `cargo slotcar` and `cargo tigerstyle` commands remain compatibility aliases.

## Reviewed requirement contracts

`cargo octet requirements check` evaluates a reviewed Nickel-authored contract against retained
evidence. It requires a disposition for every clause, binds formal proofs to the exact source and
scope, rejects empirical evidence for formal-proof requirements, and records the recomputable
decision. `requirements verify` recomputes that decision from the contract, evidence, and source
bytes. `requirements freshness` compares the same contract with an evidence codebase index and
reports `current`, `stale_binding`, `missing`, `failed`, `unsupported`, and `incomplete` outcomes.

```bash
cargo octet requirements check \
  --contract standards/requirements/example-contract.json \
  --evidence standards/requirements/fixtures/example/evidence.json \
  --source-root . \
  --output artifacts/requirements/decision.json \
  --output-format json
cargo octet requirements verify \
  --contract standards/requirements/example-contract.json \
  --evidence standards/requirements/fixtures/example/evidence.json \
  --decision artifacts/requirements/decision.json \
  --source-root .
```

The contract surface does not authenticate reviewers or producers, prove verifier soundness, or
establish release eligibility. See [`docs/requirements-contracts.md`](docs/requirements-contracts.md).

## Release operator path

Use these surfaces for a bounded release smoke:

- `cargo octet check --cache --artifact-dir artifacts/octet`
- `cargo octet artifact verify --artifact-dir artifacts/octet`
- `cargo octet evidence collect --rail <rail> ...`
- `cargo octet fingerprint --check --review-manifest artifacts/fingerprint-review.json src/lib.rs`
- `cargo octet migrate start --manifest artifacts/octet-migration.json`
- `cargo octet transcript docs/examples/release-operator-smoke/transcript.md`

`cargo octet transcript` checks transcript structure only: fences holding `$ ` command lines must use a supported fence language, and the file must contain `$ ` command lines and `=> ` expected-output lines. It records the transcript BLAKE3 but runs no command and compares no output.

Function-address modes are `observe`, `warn`, and `deny`. They produce source-envelope records, function spans, review fields, diagnostics, and Kamacite handoff data.

A reviewed baseline entry whose function was deleted or renamed away reports `function_address_stale`. Entries outside the checked `--function-address-path` scope are not checked.

`--function-address-path` and `--function-address-baseline` need `--function-addresses observe|warn|deny` or a strict profile, and paths must stay inside the workspace root; otherwise `cargo octet check` exits 2 before Cargo runs.

Function-address markers share the `// octet-fingerprint:` syntax. A marker written by `cargo octet fingerprint` holds the fingerprint token hash, not a function address, so function-address checks ignore it; that function still needs a function-address baseline entry or a function-address marker.

Kamacite-owned Preserves bytes are the portable receipt authority. The JSON file is an operator projection.

<!--
r[verify octet.evidence_provenance.evidence_chain_handoff.projection]
r[verify octet.evidence_provenance.evidence_chain_handoff.positive]
r[verify octet.evidence_provenance.evidence_chain_handoff.validation]
r[verify octet.evidence_provenance.valence_export.boundary_hardened.visible]
-->

At Cairn revision `70db8fe114910feb5d162fea5db8ad4ffdb6f176`, the stack smoke checks one frozen Octet-to-Cairn fixture.

This result does not prove arbitrary-version compatibility, behavior, deployment, or release eligibility.

External rails can collect dependency, Clippy, ast-grep, Wasm, Charon, trust, API, runtime, supply-chain, mutation, coverage, Kani, Hegel, solver, and verifier evidence.

Each rail retains its own scope and non-claims. Agreement between rails does not merge their authority.

Run the complete documentation smoke:

```bash
nix build .#checks.x86_64-linux.docs-examples --no-link -L
```

The transcript is at `docs/examples/release-operator-smoke/transcript.md`.

## Naming map

Octet is the current product identity. Tiger Style remains the coding philosophy and current lint namespace.

Canonical attributes use `tigerstyle::<lint>`. Domain groups use `tigerstyle::<domain>`.

| Surface | Current name | Compatibility name |
|---|---|---|
| Public site | `octet.dev` | `slotcar.dev` is historical |
| Project | `Octet` or `octet` | `slotcar` is legacy |
| Repository | `github:OnixResearch/octet` | Old repository names are obsolete |
| Package | `octet` | `slotcar` and `tigerstyle` are aliases |
| Runner binary | `cargo-octet` | `cargo-slotcar` and `cargo-tigerstyle` are aliases |
| Runner command | `cargo octet check` | Legacy commands route to the same engine |
| Standards binary | `octet-standards` | Legacy binaries are aliases |
| Policy commands | `octet-policy-export` and `octet-policy-validate` | Old names are aliases |
| Lint attributes | `tigerstyle::<lint>` | Bare names remain compatibility aliases |
| Workspace config | `[workspace.metadata.octet]` | `[workspace.metadata.slotcar]` has lower precedence |
| Lint config | `[octet]` and `[octet.lint_levels]` | `[slotcar]` and `[tigerstyle]` have lower precedence; a shadowed section is ignored with a warning |
| Environment prefix | `OCTET_*` | `TIGERSTYLE_*` remains bootstrap compatibility |

When both forms exist, Octet names take precedence.

## Lint domains

| Domain | Count | Focus |
|---|---:|---|
| safety | 27 | panics, bounds, defaults, loops, queues, unsafe code, failures, and durability claims |
| naming | 9 | predicates, acronyms, units, currency, paths, and imports |
| performance | 2 | clones, allocation, and ownership |
| structure | 21 | size, nesting, parameters, casts, APIs, and cohesion |
| purity | 16 | clocks, environment, randomness, effects, allocation, and verified purity |
| concurrency | 2 | lock order and task lifecycle |

Read [`docs/lint-reference.md`](docs/lint-reference.md) for the full catalog.

## Documentation

Start with these guides:

- [`docs/consumer-guide.md`](docs/consumer-guide.md): setup and rollout
- [`docs/lint-reference.md`](docs/lint-reference.md): lint catalog and config
- [`docs/boolean-api-modeling.md`](docs/boolean-api-modeling.md): public boolean API migration and compatibility policy
- [`docs/trust-boundary-evidence.md`](docs/trust-boundary-evidence.md): evidence and claims
- [`docs/proof-implementation-gap.md`](docs/proof-implementation-gap.md): the named specification-assumption failure class at the proof-to-implementation boundary
- [`docs/strict-rust-profiles.md`](docs/strict-rust-profiles.md): named profiles
- [`docs/project-architecture-policy.md`](docs/project-architecture-policy.md): opt-in FCIS and Hexagonal project policy
- [`docs/assurance-profiles.md`](docs/assurance-profiles.md): cross-plane assurance profiles
- [`docs/durability-claims.md`](docs/durability-claims.md): declared durability boundaries and storage-fault assurance
- [`docs/downstream-rollout.md`](docs/downstream-rollout.md): consumer promotion
- [`docs/contributor-guide.md`](docs/contributor-guide.md): source map and checks
- [`docs/release-candidate.md`](docs/release-candidate.md): release checklist
- [`CHANGELOG.md`](CHANGELOG.md): consumer changes

The `docs/` and `templates/` directories contain focused guides and copyable checklists for assertions, safety, bounds, resilience, performance, concurrency, Kani, system comprehension, APIs, sources, and proof rails.

## License

Octet's original material uses `AGPL-3.0-or-later` (GNU Affero General Public License, version 3 or any later version). See [LICENSE](LICENSE) and [NOTICE](NOTICE). This preserves the existing grant.

Dependencies and imported material retain their own terms. [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) records bundled licenses and unresolved provenance; project metadata does not grant rights over somebody else's work.

Running Octet does not by itself relicense the code being checked. Redistributing covered binaries requires the corresponding-source arrangements in AGPL section 6. Modified versions supporting remote network interaction must offer their corresponding source to those users under section 13. The license text controls; check output and license metadata do not certify correctness or release readiness.

For contributions, see [CONTRIBUTING.md](CONTRIBUTING.md). For private vulnerability reporting, see [SECURITY.md](SECURITY.md).

## References

- [OnixResearch/trellis](https://github.com/OnixResearch/trellis) — Verus proof corpus and proof-artifact identity reference.
- [verus-lang/verus](https://github.com/verus-lang/verus) — pinned verifier and toolchain source.
- [verus-lang/verusfmt](https://github.com/verus-lang/verusfmt) — source formatter.
- [nickel-lang/json-schema-to-nickel](https://github.com/nickel-lang/json-schema-to-nickel) — generated contract reference.
- [oxidecomputer/dropshot](https://github.com/oxidecomputer/dropshot) — typed API reference.
- [oxidecomputer/progenitor](https://github.com/oxidecomputer/progenitor) and [oxidecomputer/typify](https://github.com/oxidecomputer/typify) — contract generator references.
- [joweeba/z4](https://github.com/joweeba/z4) — optional solver evidence reference.
- [nasa/spacewasm](https://github.com/nasa/spacewasm) at `e24cf09355a90497148eb5029fdb8e3400bd63e3` — execution-free Wasm profile source.
- [AeneasVerif/aeneas](https://github.com/AeneasVerif/aeneas) and [AeneasVerif/charon](https://github.com/AeneasVerif/charon) — opt-in proof and semantic extraction references.
- [AeneasVerif/eurydice](https://github.com/AeneasVerif/eurydice) and [AeneasVerif/scylla](https://github.com/AeneasVerif/scylla) — deferred translation references.
- [endorlabs/MIRAI](https://github.com/endorlabs/MIRAI) at `aae0a56f91d9a0a62073bb86e7b2c9793cefc24b` — bounded semantic-pilot source reference.
- [Radiant Computer/radiance](https://code.radiant.computer/radiance/tree/8c7552bb010a226d2686724e93e8e5d064ec01cdf46523185eebb9937588f7e4) at `8c7552bb010a226d2686724e93e8e5d064ec01cdf46523185eebb9937588f7e4` — its MIT-licensed [STYLE](https://code.radiant.computer/radiance/tree/8c7552bb010a226d2686724e93e8e5d064ec01cdf46523185eebb9937588f7e4/STYLE) is a reference for explicit boolean-callsite and state modeling only; not an Octet lint, compiler, or behavior proof. See [the bounded adaptation](docs/boolean-api-modeling.md#upstream-style-boundary).

These references do not transfer compiler, verifier, semantic, security, or release claims into Octet.
