# Verus Proof Rails

Verus is an opt-in formal-proof evidence rail in Octet. It is not part of the
ordinary `cargo octet check` starter path, and it is not a Dylint lint.
The canonical review route lives in
[`docs/trust-boundary-evidence.md#verus-proof-path-lane`](../docs/trust-boundary-evidence.md#verus-proof-path-lane):
requirement or policy ID to Verus proof rail, checked Rust source scope, proof
target, verifier identity, structured receipt, `formal_proof` provenance, and
artifact verification. The broader checked-contract ladder is documented at
[`docs/trust-boundary-evidence.md#checked-contract-ladder`](../docs/trust-boundary-evidence.md#checked-contract-ladder), where Verus receipts occupy only the scoped formal-proof stage.

The repository-owned fixture rail is intentionally small:

- source scope: `src/verified/addition.rs`
- proof target: `verus/proofs/addition_bound.rs`
- verifier surface: `nix run .#octet-verus -- --identity`
- proof command argv: `octet-verus verus/proofs/addition_bound.rs`
- pass signal: `proof-result: pass octet-addition-bound`
- receipt path: `proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt`
- verifier mode: `fixture`

The production pilot rails run tiny proof targets with real Verus:

- verifier surface: `nix run .#octet-production-verus -- --identity`
- addition proof command argv: `octet-production-verus verus/proofs/addition_bound.rs`
- addition receipt path: `proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt`
- reflexive-order proof command argv: `octet-production-verus verus/proofs/ordering_reflexive.rs`
- reflexive-order receipt path: `proofs/octet.verus_proof_rails.ordering-reflexive-production-rail-0.receipt`
- checked-increment source scope: `src/verified/checked_increment.rs`
- checked-increment proof target: `verus/proofs/checked_increment_bound.rs`
- checked-increment proof command argv: `octet-production-verus verus/proofs/checked_increment_bound.rs`
- checked-increment receipt path: `proofs/octet.verus_proof_rails.checked-increment-production-rail-0.receipt`
- pass signal for each production pilot: `verification results:: 1 verified, 0 errors`
- verifier mode for each production pilot: `production`
- pinned release: Verus `0.2026.05.17.e479cce` at revision `e479cce36490b8fa4b0fd7755aa742aec354372c`, Rust `1.95.0-x86_64-unknown-linux-gnu`, bundled Z3, and an Octet-generated Nix closure graph digest

The proof-rail manifest is a typed Nickel map, so teams can add additional
repo-local rails under `standards/proof-rails/manifest.ncl` or a consumer-owned
imported manifest without changing the contract every time a new rail ID is
introduced. Each rail entry is still validated for backend, explicit argv,
source/proof/receipt paths, verifier identity, trust boundaries, and production
solver/toolchain/closure identity when `verifier_mode = "production"`.

The production negative fixture is failure-path evidence only:

- source selector: `verus/failure-fixtures/addition_bound_invalid_source.rs`
- invalid proof target: `verus/proofs/addition_bound_invalid.rs`
- proof command argv: `octet-production-verus verus/proofs/addition_bound_invalid.rs`
- expected result: real Verus exits non-zero, `cargo octet prove` reports `verifier-failure`, and no `formal_proof` provenance is emitted
- isolation: ordinary positive proof runs over `src/verified` do not select the invalid fixture

`cargo octet prove --list` shows selected rails without writing artifacts.
`cargo octet prove --rail <policy-id[#rail-N]>` narrows execution to a named
policy rail after the ordinary family/source selection gate, so a typo, wrong
family, or missing source binding reports a deterministic diagnostic instead of
running an unintended verifier. A malformed or unmatched selector exits
non-zero in both run and `--list` mode. `cargo octet prove --explain` adds
deterministic remediation details for stale bindings, verifier mismatches,
unmatched rails, and unbacked proof claims.

`cargo octet prove` executes configured argv only, captures bounded stdout/stderr
artifacts, writes structured JSON receipts under the selected artifact directory,
emits `proof-cards.json` as a human review index, and emits `formal_proof`
provenance only after the receipt validates its pass signal, source binding,
proof binding, verifier/solver/toolchain identity, trust-boundary entries,
checked scope, and proof artifact identity. Verifier output that exceeds the
rail's `stdout_max_bytes` or `stderr_max_bytes` fails the rail with a
`verifier-failure` diagnostic, because the retained log no longer shows
everything the verifier printed.

Before a production rail runs its proof, `cargo octet prove` resolves the
verifier program once, runs it with `--identity` under the same environment
allowlist and timeout, and requires the reported `proof-verifier-binary-blake3`,
`proof-solver-blake3`, and `proof-toolchain-blake3` to equal the rail's pinned
`verifier_binary_digest`, `solver_digest`, and `toolchain_digest`. This applies
to every production rail except an `aeneas-lean` rail, whatever its `backend`
label says, because `cargo octet prove` issues a structured Verus receipt for
each of them. A program on
`PATH` that does not report the pinned identity fails the rail with an
`unpinned-verifier` diagnostic and is not asked to run the proof. The identity
is reported by the verifier wrapper itself, so the check refuses wrong or
unpinned verifiers; it does not authenticate a program built to impersonate the
pinned wrapper. The `proof-rail-manifest` flake check compares the manifest pins
with the `octet-production-verus --identity` output of the built derivation.

The artifact directory must be empty or hold an earlier `cargo octet prove`
bundle. An earlier bundle is cleared before the new one is written, so no
receipt, log, or evidence-log manifest from a previous run survives; any other
non-empty directory, including a `cargo octet check` bundle, is refused. Give
check and prove bundles separate directories, such as `artifacts/octet` and
`artifacts/octet-proofs`.

Source locators, rail `proof_paths`, and the proof command's working directory
are relative to one root: the Cargo workspace root of the `--source` arguments,
found the way Cargo finds it (the nearest `[workspace]` that lists the source's
package as a member, otherwise that package). Every `--source` must resolve to
the same root regardless of argument order, a source outside it is rejected,
`--source .` at the root selects the whole root, and overlapping arguments load
each file once.

## Proof cards

`proof-cards.json` contains one generated card per selected rail. A card names
the proof or property ID, source requirement or policy ID, checked Rust source
scope, proof target path, verifier mode, verifier identity summary,
solver/toolchain summary, receipt path, artifact BLAKE3 identities, evidence
origin, evidence tier, status, trust-boundary notes, non-claims, and
`cargo octet artifact verify` command. The card is an index into
receipt/provenance evidence, not standalone evidence.

Current selected positive rails produce cards with these stable identities when
run through `cargo octet prove --policy-artifacts-dir artifacts/policy --source src/verified --artifact-dir artifacts/octet-proofs`:

| Proof/property ID | Mode | Source scope | Proof target | Receipt path |
| --- | --- | --- | --- | --- |
| `octet.verus_proof_rails.addition-bound#rail-0` | `fixture` | `src/verified/addition.rs` | `verus/proofs/addition_bound.rs` | `proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt` |
| `octet.verus_proof_rails.addition-bound-production#rail-0` | `production` | `src/verified/addition.rs` | `verus/proofs/addition_bound.rs` | `proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt` |
| `octet.verus_proof_rails.ordering-reflexive-production#rail-0` | `production` | `src/verified/ordering.rs` | `verus/proofs/ordering_reflexive.rs` | `proofs/octet.verus_proof_rails.ordering-reflexive-production-rail-0.receipt` |
| `octet.verus_proof_rails.checked-increment-production#rail-0` | `production` | `src/verified/checked_increment.rs` | `verus/proofs/checked_increment_bound.rs` | `proofs/octet.verus_proof_rails.checked-increment-production-rail-0.receipt` |

A generated card reports `evidence_tier = formal_proof` only if it matches a validated receipt and provenance record.

The source scope, proof target, verifier mode, and artifact BLAKE3 identity must match. Imported
Trellis cards use `evidence_origin = external_trellis_import`, name the Trellis
repository revision plus source/proof/log/receipt BLAKE3 identities when present,
and stay `manual_review_required` unless Octet reruns the proof and validates an
Octet-owned receipt. BLAKE3 identifies exact artifact bytes and does not by
itself prove proof correctness. If the receipt is missing, stale, failed, fixture
evidence is presented as production evidence, external evidence is presented as
Octet formal proof, or wording broadens the claim, card validation downgrades the
card to `manual_review_required` and records diagnostics. Ordinary
`cargo octet check` does not require proof cards, imported bundles, or Verus
artifacts.

## Proof evidence references

Use `ProofEvidenceRef` JSON records when source comments, docs, or future lint
suppressions need to point at a proof card without adding a consumer-implemented
`Verified` trait. The reference is linkage evidence only: it does not rerun
Verus, does not emit `formal_proof` provenance, and cannot upgrade a
`manual_review_required` card.

```json
{
  "schema_version": "octet-proof-evidence-ref/v1",
  "reference_id": "unchecked-narrowing-range-proof",
  "consumer_context": "allow tigerstyle::unchecked_narrowing at src/codec.rs:42",
  "consumer_source_path": "src/codec.rs",
  "consumer_source_blake3": "<BLAKE3 of src/codec.rs at review time>",
  "consumer_source_anchor": "octet-proof-ref: unchecked-narrowing-range-proof -> octet.verus_proof_rails.addition-bound-production#rail-0",
  "proof_or_property_id": "octet.verus_proof_rails.addition-bound-production#rail-0",
  "source_requirement_or_policy_id": "octet.verus_proof_rails.addition-bound-production",
  "checked_rust_source_scope": "formal-proof:production:octet.verus_proof_rails.addition-bound-production#rail-0",
  "expected_evidence_tier": "formal_proof",
  "required_status": "passed",
  "required_artifacts": [
    "proofs/octet.verus_proof_rails.addition-bound-production-rail-0.receipt",
    "verus/proofs/addition_bound.rs"
  ],
  "non_claims": [
    "not whole-program correctness",
    "not certification",
    "not business-domain validation",
    "not semantic equivalence",
    "not full safety"
  ]
}
```

Scaffold and validate a reference against a saved proof-card deck:

```bash
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
```

Scaffold computes the consumer source BLAKE3 and copies proof-card fields into valid JSON.

Validation checks the saved card, required artifacts, exact source identity, and exact anchor. It rejects a card that carries diagnostics or presents imported Trellis evidence (`evidence_origin: external_trellis_import`) as `formal_proof`. This links a comment, doc reference, or future
suppression to the proof card without rerunning Verus or emitting new
`formal_proof` provenance.

Passing the fixture rail is scoped evidence for the declared proof-plumbing
fixture only. Passing a production pilot is scoped evidence for that pilot's
declared property only: the addition-bound target or the reflexive-order target.
The negative fixture is not a proof. It shows that production Verus failures fail closed. No rail claims whole-program correctness, certification,
business-domain validation, semantic equivalence, or full safety. Every
production rail must declare `verifier_mode = "production"` and pin the real
Verus verifier, solver, Rust toolchain, and Nix closure or closure-graph digest.
