# Octet

Octet is the evidence-backed checked path for agent-written and human-written
Rust systems code. It is a Rust-compatible toolchain and lint distribution for
systems code that should be stable through closed interfaces. It applies
[Tiger Style](https://github.com/tigerbeetle/tigerbeetle/blob/main/docs/TIGER_STYLE.md)
rules: explicit shell/core boundaries, bounded resource use, positive naming,
compact functions, and reviewable suppressions.

The checked path is concrete: Rust source flows through `cargo octet check` and
`octet-standards` checks, checked-in policy/config, reviewable evidence
artifacts, then consumer CI or release gates. See
`docs/trust-boundary-evidence.md` for the evidence stack, evidence tiers,
provenance chain, scope boundary, and controlled-deployment map.

The chemistry metaphor is intentional: an octet is a stable full valence shell.
Octet treats Rust-compatible code as inert by default across APIs, effects,
capabilities, proofs, and review surfaces; interaction happens only through
complete, explicit interfaces.

Octet currently ships 68 Dylint lints across 6 domains. The repo also owns a
separate standards registry/checker seam for policy traceability and
non-Dylint source-shape rules.

## Source-only public consumption

The independently rooted `terrane-toolchain-374bd16` branch of
[`OnixResearch/octet-public`](https://github.com/OnixResearch/octet-public)
publishes this older production Verus/tooling cohort, derived from source
`374bd16b26cee2af34211a29bfa531c016811f51`.
It is not a byte-identical copy of that Git revision.
Private local settings and archived evidence directories are withheld; retained
operator checkout paths are illustrative. Historical checkboxes, receipts, and
fingerprint evidence references do not attest to this derived source. The
withheld transcripts are not publicly replayable, and edited notes are not
verbatim evidence. License texts and upstream notices are retained.

This source does not replace the newer `890f424` Aeneas-provider cohort on the
mirror's independently rooted `main` branch. The cohorts are not merged; do not
substitute newer compiler code for this older consumer. Live dependency
selections use immutable public sources. Pin a full reviewed public commit;
fresh package checks are required before release admission. Frozen fixture and
generated-index bytes retain their historical labels, not fresh proof claims.

## Quick start

1. Copy `templates/dylint.toml` into the consumer workspace as `dylint.toml`.
2. Add runner metadata to the workspace `Cargo.toml`:

```toml
[workspace.metadata.octet]
default_scope = ["-p", "your-core-crate"]
cargo_check_args = ["--lib"]
```

3. Run Octet through the first-class runner.

Installed cargo subcommand:

```bash
cargo octet check
```

Nix, from the canonical repo flake without installing into the consumer toolchain:

```bash
nix run github:OnixResearch/octet-public/terrane-toolchain-374bd16#cargo-octet -- check
```

Local checkout alternative:

```bash
nix run path:/path/to/octet#cargo-octet -- check
```

Structured output and reviewable sidecars are first-class too:

```bash
cargo octet check --output-format sarif --artifact-dir artifacts/octet
```

Named strict-Rust profiles activate reviewed lint floors, function-address
minimums, and profile-bound BLAKE3 evidence:

```bash
cargo octet check --profile robust --artifact-dir artifacts/octet
cargo octet artifact verify --artifact-dir artifacts/octet
```

The shipped profiles are `robust`, `bounded-systems`, and
`deterministic-core`. They enforce bounded direct source-shape policy; they do
not claim transitive effect inference, whole-program proof, or release
eligibility. `deterministic-core` additionally requires configured
`core_source_scopes` and a reviewed deny-mode function-address baseline. See
[`docs/strict-rust-profiles.md`](docs/strict-rust-profiles.md) for the profile
matrix and bootstrap path.

A separate opt-in closure rail extends a successful `deterministic-core` check
with conservative selected source-token call/effect propagation and exact
reviewed provider summaries:

```bash
cargo octet admit --profile deterministic-core --closure \
  --artifact-dir target/octet-admission \
  -- --target x86_64-unknown-linux-gnu
cargo octet admit verify --artifact-dir target/octet-admission
```

Unknown effects, opaque constructs, stale function identities, forbidden role
edges, and unadmitted or provider-mismatched calls fail closed. The resulting
receipt is bounded source-token evidence under reviewed summaries—not complete
Rust name resolution, macro/trait/MIR closure, dependency implementation proof,
runtime determinism, whole-program correctness, or release eligibility. See
[`docs/effect-closure-admission.md`](docs/effect-closure-admission.md).

Optional external evidence rails collect selected non-starter tool output into separate receipt bundles:

```bash
cargo octet evidence collect --rail dependency-audit --input target/deny.json --config deny.toml --locked --artifact-dir artifacts/dependency-audit
cargo octet evidence collect --rail hegel-property --input target/hegel-property.json --artifact-dir artifacts/hegel-property
cargo octet artifact verify --artifact-dir artifacts/dependency-audit
```

For a strict local gate, copy `templates/pre-commit-config.yaml` to a consumer
repo and pin this repository as a pre-commit hook source. The `octet-deny-all`
hook runs the same runner with every Octet/Tiger Style lint promoted to `deny`.

Exit codes stay stable for automation:

- `0` clean or warning-only runs
- `1` error-level lint findings
- `2` integration or cargo-check failures

Outside Nix, install the cargo subcommand once, then follow
`docs/consumer-guide.md` for the repo-pinned `OCTET_DYLINT_REV` bootstrap
before running `cargo octet check`:

```bash
cargo install --path /path/to/octet/cargo-octet
```

4. Tune thresholds, disabled lints, and per-lint levels in `dylint.toml`.

After TOML syntax parses, Octet accumulates independent semantic config diagnostics (for example unknown lint names, invalid levels, and conflicting disabled/level entries) in deterministic order while still ignoring invalid entries and failing closed before dependent evidence promotion.

Standards registry checks live behind a separate command surface:

```bash
nix run .#octet-policy-validate
nix run .#octet-standards -- --help
```

Formal proof rails are opt-in. `cargo octet prove` lists or runs selected
standards proof rails, writes structured verifier receipts plus `proof-cards.json`
under an artifact directory, and emits `formal_proof` provenance only for scoped
receipts with verifier, solver/toolchain, command argv, source/proof path,
receipt path, and bounded log identity. Starter `cargo octet check` usage does
not require Verus.
Octet keeps the fixture `octet-verus` rail for plumbing tests and also ships
multiple tiny production-mode pilots through `octet-production-verus`, pinned to
the real Verus binary release, solver, Rust toolchain, and Nix closure graph
identity. Each pilot proves only its declared property and makes no
whole-program correctness claim. The canonical route is documented in
[`docs/trust-boundary-evidence.md#verus-proof-path-lane`](docs/trust-boundary-evidence.md#verus-proof-path-lane):
requirement or policy ID to Verus proof rail, checked Rust source scope, proof
target, verifier identity, structured receipt, proof-card review,
`formal_proof` provenance, and artifact verification. Imported Trellis references
and Trellis proof evidence IR are BLAKE3 byte-identity/manual-review or
recorded-only inputs until Octet reruns the proof and validates its own receipt.
Promotion requires matching source, proof IR, verifier, toolchain, solver,
command, assumptions, property scope, and passed result identity.
The typed compatibility source is `standards/verus-toolchain/profile.ncl`; its
fresh generated export drives the production verifier package and records
intentional downstream pins without transferring proof claims. Reviewed native
package consumers include the OnixOS lifecycle rail and Aspen node-replication
probe; their rows do not transfer source-model, proof, runtime, or readiness
authority. The same profile pins `octet-verusfmt`, whose wrapper disables
self-update and exposes bounded formatter identity.

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
  --artifact-dir artifacts/octet
cargo octet proof-ref scaffold \
  --proof-cards artifacts/octet/proof-cards.json \
  --proof-or-property-id octet.verus_proof_rails.addition-bound-production#rail-0 \
  --reference-id unchecked-narrowing-range-proof \
  --consumer-context 'allow tigerstyle::unchecked_narrowing at src/codec.rs:42' \
  --consumer-source src/codec.rs \
  --anchor 'octet-proof-ref: unchecked-narrowing-range-proof -> octet.verus_proof_rails.addition-bound-production#rail-0' \
  --output proof-ref.json
cargo octet proof-ref validate \
  --proof-cards artifacts/octet/proof-cards.json \
  --ref proof-ref.json
cargo octet artifact verify --artifact-dir artifacts/octet
```

Start with `docs/consumer-guide.md` for the full consumer setup and rollout
path. Legacy `cargo slotcar` / `cargo tigerstyle` command names remain
compatibility aliases only.

## Release/operator smoke path

For release readiness, operators should be able to find the critical surfaces from
this README plus command help, without reading archived OpenSpec changes:

- `cargo octet check --cache --artifact-dir artifacts/octet` records
  effect-summary output in `summary.txt`, sidecar findings, `provenance.jsonl`,
  and lint-cache `cache_status` fields. Add `--function-addresses observe|warn|deny`
  plus optional `--function-address-path PATH` entries to gradually content-address
  functions with lint-like diagnostics before promoting selected scopes to `deny`;
  when enabled with `--artifact-dir`, the check writes `function-addresses.json`
  beside the normal bundle sidecars with workspace-relative source-envelope and
  function record paths, zero-based `start_byte`/`end_byte` spans, marker review
  fields, duplicate/malformed/stale diagnostics, a Kamacite-compatible projection
  descriptor, an Octet-to-Kamacite Preserves handoff contract, and the generic
  Octet evidence-chain producer handoff envelope. The JSON file is an
  operator/debug compatibility projection; Kamacite-owned canonical Preserves
  bytes are the portable stack receipt authority. Dependency identity is
  labeled `source-token-set`: it records the deterministic source-token dependency
  set, not a resolved Rust call graph or semantic dependency closure. `observe` is the bootstrap mode:
  unreviewed or unbaselined functions produce renderer-valid warning findings
  while still emitting the artifact for review. Use `--function-address-baseline PATH` to ratchet
  gradually: unchanged baseline functions stay admitted while changed or new
  unaddressed functions report stale/missing diagnostics. These receipts are
  identity and source-shape evidence only; they do not claim Rust semantic
  correctness, compiler correctness, whole-program safety, verifier soundness,
  or release eligibility.
- `cargo octet artifact verify --artifact-dir artifacts/octet` recomputes sidecar
  BLAKE3 hashes and fails closed on missing, stale, malformed, or inconsistent
  provenance before the directory is treated as release evidence. With
  `--emit-valence-provenance PATH`, it writes a replayable, typed Valence adapter
  input only after verification succeeds; r[verify octet.evidence_provenance.valence_export.boundary_hardened.visible]
  the export is not release eligibility, Mantle bundle inclusion, whole-program
  correctness, or verifier-soundness proof.
- `cargo octet evidence collect --rail dependency-audit|clippy-co-reporting|dependency-trust|api-stability|dynamic-evidence|release-supply-chain|mutation-testing|coverage-baseline|kani-model-check|hegel-property`
  writes optional external evidence receipts with checker identity, source severity,
  selected scope, parent links, and explicit non-claims. Kani receipts are scoped
  model-check evidence for named harnesses/contracts/stubs; Hegel receipts are
  bounded generated-input property evidence for named pure-core laws. Neither rail
  is evidence of whole-program correctness, release eligibility, semantic
  equivalence, or verifier soundness.
- `cargo octet fingerprint --check --review-manifest artifacts/fingerprint-review.json src/lib.rs`
  verifies semantic identity fingerprints with BLAKE3-backed review records.
- `cargo octet migrate start --manifest artifacts/octet-migration.json`
  writes a deterministic migration manifest before safe application.
- `cargo octet transcript docs/examples/release-operator-smoke/transcript.md`
  validates executable transcript command blocks and expected-output assertions.

One bounded smoke artifact covers those surfaces together:

```bash
nix build .#checks.x86_64-linux.docs-examples --no-link -L
```

The smoke transcript lives at
`docs/examples/release-operator-smoke/transcript.md`.

## Naming map

Octet is the canonical current product identity. `Tiger Style` remains the
source philosophy. Current release lint source attrs, domain groups, and emitted
lint group metadata still use `tigerstyle::...` until the namespace migration
lands; Octet command and config names are the preferred product surfaces.

| Surface | Canonical Octet name | Legacy / compatibility classification |
| --- | --- | --- |
| public site | `octet.dev` | `slotcar.dev` is historical until a site decision lands |
| project name | `Octet` / `octet` | `slotcar` is legacy product identity |
| GitHub repo reference | `github:OnixResearch/octet-public/terrane-toolchain-374bd16` | Source-only older cohort; pin a full reviewed commit |
| crate and lint package | `octet` | `slotcar`/`tigerstyle` package aliases are compatibility only |
| lint runner binary | `cargo-octet` | `cargo-slotcar` and `cargo-tigerstyle` route to the same engine as aliases |
| lint runner command | `cargo octet check` | `cargo slotcar check` and `cargo tigerstyle check` are transition aliases |
| standards checker binary | `octet-standards` | `slotcar-standards` / `tigerstyle-standards` are compatibility aliases |
| standards policy commands | `octet-policy-export`, `octet-policy-validate` | old policy command names are compatibility aliases |
| lint attrs and domain groups | `tigerstyle::<lint>` and `tigerstyle::<domain>` | bare lint names remain compatibility aliases during migration |
| workspace runner config | `[workspace.metadata.octet]` | `[workspace.metadata.slotcar]` is legacy and lower precedence |
| lint tuning config | `[octet]` and `[octet.lint_levels]` | `[slotcar]` is legacy and lower precedence |
| support env prefix | `OCTET_*` | `TIGERSTYLE_*` is bootstrap compatibility while Dylint wiring migrates |

When both Octet and legacy config are present, Octet names win. Remaining
`Tiger Style` references are valid when they describe the coding philosophy or
the current release's live lint namespace; `settle-octet-namespace-migration`
tracks the source-attribute namespace decision.

## Lint domains at a glance

| Domain | Count | Focus |
| --- | ---: | --- |
| safety | 23 | panics, bounds, defaults, loops, queues, unsafe boundaries, explicit failure paths, enum evolution |
| naming | 9 | predicate names, acronym casing, units, currency, searchable paths and imports |
| performance | 2 | clone avoidance, hot-loop allocation, and ownership clarity |
| structure | 16 | function size, nesting, parameters, casts, public APIs, interface cohesion |
| purity | 16 | wall-clock, environment, randomness, core-effect boundaries, allocation, and verified-module purity |
| concurrency | 2 | lock ordering and task lifecycle |

See `docs/lint-reference.md` for the full catalog, defaults, suppression rules,
and config keys.

## Documentation

- [Assertion review guide](docs/assertion-review.md) and [checklist template](templates/assertion-checklist.md)
- [Safety-critical claims guide](docs/safety-critical.md)
- [Bounds and pacing review guide](docs/bounds-and-pacing-review.md) and [checklist template](templates/bounds-and-pacing-checklist.md)
- [Resilience review guide](docs/resilience-review.md) and [five-whys template](templates/five-whys-architecture.md)
- [Performance review guide](docs/performance-review.md) and [sketch template](templates/performance-sketch.md)
- [Concurrency and distributed-state review guide](docs/concurrency-state-review.md) and [checklist template](templates/concurrency-state-checklist.md)
- [Kani panic-freedom pilot guide](docs/kani-panic-freedom-pilots.md) and [harness template](templates/kani-panic-harness.rs)
- [Kani property checklist](docs/kani-property-checklist.md) and [checklist template](templates/kani-property-checklist.md)
- [System comprehension guide](docs/system-comprehension.md) and [walkthrough template](templates/system-walkthrough.md)
- `docs/consumer-guide.md` — first run, config split, Nix integration, pre-commit hook, output modes, artifact bundles, migration notes
- `docs/adoption-story.md` — card-oriented public story, terminal-first quickstart, examples, Built with Octet case-study format, methodology requirements, and claim boundaries
- `docs/examples/first-checked-rust-patch/README.md` — first checked Rust patch walkthrough from `unchecked_division` finding to fixed code and verified artifacts
- `docs/trust-boundary-evidence.md` — checked-path framing, evidence stack, evidence tiers, the [checked-contract ladder](docs/trust-boundary-evidence.md#checked-contract-ladder), provenance, scope boundary, and controlled-deployment map
- `docs/law-tested-cores.md` — bounded, replayable law catalog for deterministic pure cores
- `docs/strict-rust-profiles.md` — named lint floors, function-address bootstrap, profile evidence, and non-claim boundaries
- `templates/pre-commit-config.yaml` — copyable deny-all pre-commit hook configuration
- `docs/lint-reference.md` — full lint catalog, default levels, suppression paths, config reference
- `docs/api-contract-rail.md` — Oxide-inspired API contract rail references, compatibility classes, and non-claims
- `docs/source-boundary-core.md` — source descriptor kinds, shell/core ownership, and non-claim boundaries
- `docs/source-acquisition-receipts.md` — stack-owned source receipt kinds, private SSH boundaries, and local override guidance
- `docs/valence-core-cutover.md` — standalone Valence dependency target, Octet compatibility shim, downstream migration, and cutover non-claims
- `docs/preserves-receipt-export.md` — optional Preserves envelope adapter contract and non-claims
- `docs/formal-proof-valence-export-fixture.md` — frozen formal-proof Valence adapter fixture contract
- `docs/concurrency-performance-lint-candidates.md` — first-wave concurrency/performance lint candidate matrix
- `docs/contributor-guide.md` — source-of-truth map, doc-update crosswalk, verification rails
- `docs/downstream-rollout.md` — rollout profiles, promotion playbooks, and boundary guidance
- `docs/indexable-lint-signal-summary.md` — high-volume indexable lint signal-quality status and promotion blockers
- `docs/downstream-rollout-evidence.md` — Aspen-backed rollout evidence and historical replay paths
- `standards/policies/README.md` — typed policy registry and traceability contract
- `docs/standards/policy-contract.md` — standards rules mapped to verification rails
- `docs/examples/release-operator-smoke/transcript.md` — one bounded release smoke transcript covering operator surfaces
- `docs/release-candidate.md` — pre-tag release candidate checklist and local smoke receipt path
- `CHANGELOG.md` — breaking changes and notable updates for consumers

## License

AGPL-3.0-or-later

## References

- `../trellis/` — reference Verus proof corpus and BLAKE3 proof-artifact identity patterns for Octet proof-path, proof-card, and imported proof-bundle work.
- [verus-lang/verus](https://github.com/verus-lang/verus) — pinned production verifier, solver, and toolchain source; each receipt remains scoped to exact source, proof, assumptions, and tool identity.
- [verus-lang/verusfmt](https://github.com/verus-lang/verusfmt) — pinned Verus formatter used for source-shape checks only; formatting is not proof or semantic validation.
- `github:nickel-lang/json-schema-to-nickel` — reference pattern for generating Nickel contracts from JSON Schema inputs while preserving generator capability caveats.
- `github:nickel-lang/nickel-schemastore` — reference pattern for packaging many generated Nickel contracts behind a discoverable indexed library.
- `github:nickel-lang/rules_nickel` — reference pattern for a small declarative Nickel export surface with source, dependencies, format, and output.
- `github:nickel-lang/nickel-customs` — reference pattern for local sanity checks over contributed Nickel package metadata before promotion.
- [oxidecomputer/dropshot](https://github.com/oxidecomputer/dropshot) — reference pattern for explicit typed API surfaces and generated OpenAPI documents without hidden middleware authority.
- [oxidecomputer/progenitor](https://github.com/oxidecomputer/progenitor) and [oxidecomputer/typify](https://github.com/oxidecomputer/typify) — reference generators for contract consumer fixtures and Rust/schema interop boundaries.
- [oxidecomputer/drift](https://github.com/oxidecomputer/drift), [oxidecomputer/openapi-lint](https://github.com/oxidecomputer/openapi-lint), and [oxidecomputer/dropshot-api-manager](https://github.com/oxidecomputer/dropshot-api-manager) — reference patterns for API compatibility diffing, linting, and managed contract promotion.
- [joweeba/z4](https://github.com/joweeba/z4) — Rust-native solver, conservative `unknown`, proof-artifact, and progress-diagnostic reference for optional recorded solver evidence rails; Octet does not treat it as a default solver or Z3 replacement.
- [joweeba/zani](https://github.com/joweeba/zani), [joweeba/sunder](https://github.com/joweeba/sunder), and [joweeba/certus](https://github.com/joweeba/certus) — Rust verifier UX and fixture-shape references for bounded positive/negative proof-rail regression fixtures; Octet does not adopt their runtimes or transfer verifier soundness claims.
