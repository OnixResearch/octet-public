# Vermilion proof rail evaluation

Status: **contract, fixtures, and live Vermilion u64 pilot implemented; executable and Aeneas refinement bridges open**.

This document records a bounded evaluation of [Vermilion](https://github.com/ilyasergey/vermilion)
alongside Octet's existing Verus and Aeneas/Lean proof rails. It reuses the
existing provider-profile, rails, receipt, proof-card, and artifact-verification
boundaries rather than creating a second Aeneas integration.

Octet owns this provider contract and its pure validators. Octet does **not**
claim that Vermilion, Verus, or Lean is sound, that the pilot subject is
verified, or that any cross-provider comparison is a merged proof.

## Implemented surface

| Surface | Location |
| --- | --- |
| Pinned provider profile | `standards/proof-rails/vermilion-lean/profile.ncl` |
| Generated profile bytes | `standards/proof-rails/vermilion-lean/generated/profile.json` |
| Typed Nickel contracts | `standards/proof-rails/vermilion-lean/contracts.ncl` |
| Compatibility matrix | `standards/proof-rails/vermilion-lean/compatibility-matrix.ncl` |
| Pilot semantic contract | `standards/proof-rails/vermilion-lean/pilot-contract.ncl` |
| Pilot subject | `standards/proof-rails/vermilion-lean/pilot/src/lib.rs` |
| Canonical statement bytes | `standards/proof-rails/vermilion-lean/pilot/semantic-spec.txt` |
| Positive receipt fixture | `standards/proof-rails/vermilion-lean/fixtures/valid/complete-receipt.json` |
| Negative-control matrix | `standards/proof-rails/vermilion-lean/fixtures/negative-matrix.json` |
| Typed receipt/validator/comparison | `crates/octet-standards/src/vermilion/` |

The module is opt-in and pure. It adds no work to starter `cargo octet check`,
does not execute a provider, and does not extend the default proof-rail path.
Wiring a Vermilion rail into `cargo octet prove`, proof cards, artifact
verification, or Valence export requires a separate reviewed change. The live
Vermilion cohort itself is now built and exercised; see the evidence links
below.

## Pilot contract

The subject is a small sequential bounded-increment executable contract:

```rust
pub fn increment_checked(value: u64) -> Option<u64> {
    if value < u64::MAX { Some(value + 1) } else { None }
}
```

The common statement is bound by BLAKE3 to `pilot/semantic-spec.txt`:

- width: `u64` (64-bit unsigned)
- precondition: `value < u64::MAX`
- postcondition: `result == Some(value + 1)`
- overflow behavior: checked; `None` at `u64::MAX`
- termination: single call, no loops

This deliberately does not treat an unrestricted `nat` lemma as a proof about
every `u64` execution.

## Provider pins

Observed from the Vermilion repository at
`7c83bbca96940538eb282a67ca923893e5aa0e06`:

| Component | Pin |
| --- | --- |
| Vermilion | `7c83bbca96940538eb282a67ca923893e5aa0e06` |
| Verus front/middle end | `ilyasergey/verus` branch `dev` at `48600836424a79420589c56d86e720236a029811` |
| vstd | `48600836424a79420589c56d86e720236a029811` |
| Lean | `leanprover/lean4:v4.33.0` |
| Mathlib | `db584cd6d46c92f209a44c0f1c829460d327499d` |
| `lake-manifest.json` | `b3:a18fdfb10b7b53cc0368b088cd5d505ef811293ce639112a57db3f9b93a79479` |
| cvc5 (via lean-cvc5) | `7e3365990661b697ccb30e92d6912f4cc6589322` |
| Checker | `vrml_check`, twin check required, `--skip-identical` |

Profile identity:
`b3:a95f705436b12445d3d3cb6b7f794cf37f1e610349c1cb24725e5fd0abf65fab`.
Compatibility-matrix identity:
`b3:285ca956a570428c2b748003c145868a20ccf33df645c66ab02200ccd942a9b7`.
Pilot contract identity:
`b3:0da2a0d4a04c259384a86b91bef1565aad103b6beb315e4a6377f396f0ce3839`.

Changing any pin requires a fresh compatibility run and regenerated receipts;
updating one identity alone is rejected by construction.

## Receipt contract

`crates/octet-standards/src/vermilion/model.rs` defines a typed receipt that
binds:

- exact provider, profile, compatibility-matrix, and contract identities;
- the semantic statement, revision, width, overflow behavior, precondition,
  and postcondition;
- the exact source path, item, span, and BLAKE3 bytes;
- Verus front-end, vstd, translator, backend, Lean, Mathlib, solver, and
  dependency-audit identities;
- command argv, environment allowlist, and resource limits;
- every generated artifact and proof-twin identity;
- every required obligation with its kind, theorem, assumptions, and status;
- refinement-bridge status, including open obligations;
- checker status, exit code, completeness, empty-output state, and required
  versus checked counts;
- theorem and axiom inventories;
- the declared outcome, evidence role, comparison metadata, and non-claims.

Validation fails closed on source substitution, spec substitution, weakened
preconditions, changed overflow semantics, `nat`/`u64` confusion, wrong
provider, stale generated output, stale proof twins, missing or unexpected
obligations, unexpected axioms, unresolved holes, interrupted checking, empty
checker output, unavailable checkers, missing dependency audits, command drift,
receipt tampering, non-claim drift, and evidence overclaim.

A synthetic fixture may report the `supported` contract outcome, but its
evidence disposition must be `recorded_only` and it can never admit property
evidence. Only a complete `provider_run` receipt can request the `property`
role; even then the receipt proves only its exact checked obligations.

## Outcome vocabulary

The receipt uses a closed outcome vocabulary that maps onto the existing
Aeneas consumer semantics:

| Vermilion outcome | Meaning | Aeneas equivalent |
| --- | --- | --- |
| `supported` | Every required obligation was checked and every binding matched. | `supported` |
| `blocked` | A reviewed toolchain or language boundary prevented execution; the blocker is complete. | `blocked` |
| `failed` | Execution or contract validation failed. | `failed` |

Timeout and resource exhaustion map to `failed`, matching the existing Aeneas
selector/classification. A tool error also maps to `failed`; an absent pinned
closure maps to `blocked`.

## Comparison vocabulary

`compare` produces one closed category and never merges authority:

| Category | Condition | Aeneas mapping | Vermilion mapping |
| --- | --- | --- | --- |
| `comparable_agreement` | All routes checked the same property, semantics, and bridges. | none | none |
| `semantic_mismatch` | Same property identity, different width/overflow/precondition/result. | `failed` | `failed` |
| `incomparable_claims` | Different property identities or an open refinement bridge. | none | none |
| `proof_failure` | An executed provider rejected an obligation. | `failed` | `failed` |
| `unsupported_or_blocked` | A provider was unsupported or its closure was unavailable. | `blocked` | `blocked` |
| `timeout_or_exhaustion` | A provider exhausted its budget. | `failed` | `failed` |
| `tool_error` | A provider or surrounding tool failed to execute. | `failed` | `failed` |

`comparable_agreement` sets `evidence_merge = false`. Verus and Vermilion share
a front end, and Vermilion and Aeneas share a Lean kernel, so agreement is not
independent soundness, majority-vote success, or release eligibility.

## Claim and trust matrix

| Route | What a checked result proves | Trusted boundary | Does not prove |
| --- | --- | --- | --- |
| Verus/Z3 | The named Verus property under the pinned verifier and solver. | Verus elaboration and VC placement, Z3, vstd contracts. | u64 execution semantics of the pilot subject, whole-program correctness. |
| Vermilion/Lean | The named Lean theorem for each generated obligation under its axioms. | Pinned Verus front end, Vermilion lowering and Lean generator, prelude conventions, Lean kernel, used axioms, completeness/freshness of checking and reporting. | Translation soundness, that the statement corresponds to the Rust source, refinement to the executable function, whole-program correctness. |
| Aeneas/Lean | The named theorem over the generated functional model. | Charon extraction, Aeneas translation, generated model conventions, maintained theorem, Lean kernel. | Extraction/translation soundness, executable Rust refinement, whole-program correctness. |

Hashes bind identity, not semantic correctness.

For the Aeneas route, the maintained theorem
`aeneas.consumer.increment-matches-model` proves only the generated model's
postcondition. No maintained theorem yet relates that model to the u64 checked
contract, and the shipped fixture models a `u32` increment. For the Vermilion
route, no maintained theorem yet relates the generated Lean obligations to the
executable Rust function. Charon extraction, Aeneas translation, Vermilion
lowering, the Lean generator, and the Lean kernel remain trusted translation or
checking edges, not proved refinements.

## Negative controls

The fixture matrix exercises one positive contract fixture and these negative
classes:

source substitution, spec substitution, weakened precondition, changed
overflow semantics, `nat`/`u64` confusion, wrong provider, stale generated
output, stale proof twin, missing obligation, unexpected obligation, unexpected
axiom, unresolved proof hole, interrupted checking, empty checker output,
missing checker, missing dependency audit, receipt tamper, non-claim drift,
evidence overclaim, and absent toolchain.

`nat`/`u64` confusion is also a route-level comparison test: the existing Verus
proof is a monotonicity lemma over `nat`, so its route is incomparable with the
u64 checked contract until a refinement bridge exists.

## Open obligations

1. **Vermilion/Lean/Verus/cvc5 and Aeneas/Charon cohorts are built.**
   The pinned Vermilion `7c83bbc`, Lean 4.33.0, Mathlib cache, Vermilion Lean
   libraries, Rust workspace, and Verus fork `4860083` built on 2026-11-09.
   The pinned Aeneas `bc2eb6f9` / Charon `cb50ff16` closure also built and
   passed four live compatibility tests; see
   `standards/proof-rails/vermilion-lean/evidence/2026-11-09-aeneas-charon-closure.md`.
2. **Measured Vermilion obligations exist for the u64 pilot.** The live
   `2026-11-09-u64-pilot` run produced two Lean-verified obligations with zero
   `sorry` and a kernel-checked proof twin. See
   `standards/proof-rails/vermilion-lean/evidence/2026-11-09-u64-pilot/`.
3. **Executable refinement is open** (`pilot.increment.refinement-to-spec`).
   The Verus-syntax pilot source is a parallel specification, not the plain
   Rust executable; that bridge has not been checked.
4. **Verus route refinement is open** (`pilot.verus.plain-rust-refinement`).
   The production Verus rail now checks the u64 checked contract model in
   `verus/proofs/checked_increment_bound.rs`; no formal plain-Rust-to-model
   refinement theorem is checked yet.
5. **Aeneas route refinement is open** (`pilot.aeneas.u32-to-u64-checked`). The
   shipped Aeneas external fixture models a `u32` increment.
6. **Downstream integration is designed only.** `cargo octet prove`, proof
   cards, artifact verification, Valence export, and release readiness do not
   yet consume Vermilion receipts.

## Measurement basis

The go/no-go decision below rests on an explicit local probe run during this
change, not on an assumption:

```console
for t in vermilion lean lake aeneas charon cvc5; do
  printf '%-10s ' "$t"; command -v "$t" 2>/dev/null || echo MISSING
done
```

Observed result after the 2026-11-09 builds: the pinned Vermilion, Lean, Lake,
Mathlib, cvc5, Verus fork, Aeneas, and Charon closures are present. The live `m1-pipeline` canary passed 3/3 obligations, and the u64
checked-increment pilot passed 2/2 obligations with zero `sorry` and a
kernel-checked proof twin. The synthetic positive receipt fixture remains a
contract fixture, not a substitute for the measured pilot receipt.

## Recorded-only measured canary rail

`cargo octet prove --rail octet.vermilion_canary_rail.recorded` validates the
exact measured u64 canary receipt against the reviewed Vermilion toolchain pins,
source BLAKE3, obligation counts, outcome, and non-claims. It does not execute a
provider. The emitted proof card is `manual_review_required`, carries no
`formal_proof_envelope`, and binds the canary receipt and source as
recorded-only provenance. `cargo octet artifact verify` accepts the complete
bundle, and the Valence export preserves the recorded-only tier.

The measured canary is not the typed `octet-vermilion-proof-receipt/v1`
contract, and the typed positive fixture remains synthetic. The rail therefore
does not promote either to formal proof or claim a fresh execution.

## Go/no-go recommendation

- **Go** for the reviewed profile, typed receipt contract, comparison
  vocabulary, negative-control fixtures, and the live Vermilion u64 checked
  route. The pinned closure and the measured u64 pilot now exist.
- **Go** for the measured canary as recorded-only manual-review evidence through
  `cargo octet prove`, artifact verification, and Valence export.
- **No-go** for cross-provider comparison, executable refinement, formal
  promotion of the canary or synthetic typed fixture, or any whole-program/property claim. The plain-Rust executable bridge, the Verus
  `nat`-to-`u64` bridge, and the Aeneas `u32`-to-`u64` bridge remain open, and
  Aeneas/Charon are absent.

The next discriminating step is to bridge the live Vermilion obligations to the
plain Rust executable and to build the Aeneas/Charon route, then rerun the
comparison.

## Reproduce the pure checks

```console
cargo test -p octet-standards --test vermilion_canary
cargo test -p cargo-octet --lib canary_rail_validates_measured_receipt
cargo test -p octet-standards vermilion --lib
cargo fmt --check
cargo clippy -p octet -p cargo-octet --all-targets -- -D warnings
nickel typecheck standards/proof-rails/vermilion-lean/contracts.ncl
nickel export --format json standards/proof-rails/vermilion-lean/profile.ncl \
  | cmp - standards/proof-rails/vermilion-lean/generated/profile.json
```

These commands exercise the contract and fixtures only. They do not execute
Vermilion and do not produce proof evidence.
