# MIRAI semantic pilot

Octet records a bounded `blocked` decision for the MIRAI semantic pilot.
It does not add a production evidence rail.

## Pinned provider facts

The pilot records these exact provider facts:

| Fact | Value |
|---|---|
| Repository | `https://github.com/endorlabs/MIRAI` |
| Revision | `aae0a56f91d9a0a62073bb86e7b2c9793cefc24b` |
| Version | `1.1.12` |
| License | `MIT` |
| Rust toolchain | `nightly-2025-01-10` |
| Target | `x86_64-unknown-linux-gnu` |
| Feature | `bundled` |
| Nix package | `.#mirai-source-provider` |

`fetchFromGitHub` uses the recorded SHA-256 source hash for Nix compatibility.
Octet-owned policy, corpus, receipt, and fixture identities use BLAKE3.

The Nix package contains the exact source tree and a provider-fact record.
Its closure contains only the source package needed for review.
The package does not grant execution authority.

## Selected corpus

The corpus contains one nested Rust crate with these selected properties:

| Fixture | Expected property |
|---|---|
| `safe.rs` | Checked addition has no reachable overflow panic. |
| `panic_path.rs` | Zero reaches the selected assertion. |
| `precondition.rs` | A zero denominator is rejected before division. |
| `call_edge.rs` | The selected root calls the selected helper. |
| `effect.rs` | The selected function calls a filesystem read operation. |
| `unsupported.rs` | Macro expansion stays outside the pilot normalization contract. |

`standards/mirai/pilot.json` binds each file path, byte count, and BLAKE3 identity.
`standards/mirai/corpus.json` is the deterministic corpus projection.

## Blocked output contract

The reviewed command is fixed:

```text
cargo mirai --manifest-path cargo-octet/tests/fixtures/mirai-pilot/Cargo.toml --lib
```

The reviewed environment contains only this declared MIRAI setting:

```text
MIRAI_FLAGS=--diag=verify
```

The pinned CLI exposes human-formatted diagnostics for the selected facts.
It does not expose one stable machine-readable schema for this pilot contract.
Parsing those diagnostics would create an unstable authority boundary.

The shell therefore returns `stable-machine-output-unavailable` before process spawn.
It clears ambient environment authority and grants no network authority.

## Comparison and decision

The pure core supports these outcomes:

- `agree`
- `disagree`
- `mirai-only`
- `charon-only`
- `source-token-only`
- `unknown`

Each row retains the Charon profile and source-token catalog identities.
The blocked receipt reports `unknown` because no MIRAI observation was admitted.
It does not rewrite either parent.

All promotion measurements remain unavailable at this boundary.
The stable-output criterion has an observed value of zero.
The deterministic decision is `blocked`.

`standards/mirai/decision.json` contains the checked decision receipt.
Repeated evaluation produces the same receipt bytes and receipt identity.

## Validation

Run the focused validation:

```bash
nix develop -c cargo test -p cargo-octet --lib mirai_pilot::
nix build .#checks.x86_64-linux.mirai-semantic-pilot -L
nix build .#mirai-source-provider -L
```

The tests cover valid policy, corpus identity, fixed planning, and blocked execution.
They also cover malformed output, drift, limits, missing parents, disagreements, and overclaims.

## Non-claims

The pilot does not claim MIRAI semantic evidence.
It does not claim provider build success or diagnostic stability.
It does not prove compiler correctness, whole-program correctness, or verifier soundness.
It does not prove semantic equivalence between MIRAI, Charon, and source-token analysis.
It does not affect starter, strict-profile, Charon, Aeneas, Kani, Verus, or release paths.
