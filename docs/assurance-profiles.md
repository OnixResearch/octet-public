# Assurance profiles

An Octet assurance profile is a typed declaration of required mechanisms and evidence for one component or candidate. It links owners, protected properties, assumptions, trusted-base elements, evidence scopes, validation commands, and non-claims.

A profile is not a security score, passing receipt, or release decision. Independent mechanisms do not become a universal proof when they appear in one profile.

## Difference from strict Rust profiles

`cargo octet check --profile deterministic-core` selects direct Rust source and HIR policy. An assurance profile can require that strict profile together with formal proofs, model checks, canonical identity, evidence linkage, and lifecycle gates.

Ordinary `cargo octet check` does not load or evaluate assurance-profile Nickel source.

## Source and generated identity

The first profile lives at:

- authored profile: `standards/policies/assurance-profiles/onixos-lifecycle-proof.ncl`
- closed contract: `standards/policies/assurance-profiles/contracts.ncl`
- export request: `standards/policies/assurance-profiles/export-spec.ncl`
- generated JSON: `standards/policies/assurance-profiles/generated/onixos-lifecycle-proof.json`
- BLAKE3 manifest: `standards/policies/assurance-profiles/generated/onixos-lifecycle-proof.manifest.json`

The manifest comes from Octet's existing Nickel Export compatibility command and the pinned `nickel-export-core`. It binds the exact authored source, contract dependency, evaluator identity, command, generated output bytes, byte lengths, and BLAKE3 digests.

The manifest output digest is the profile artifact identity. The package also exposes that digest as `generated/profile.blake3`.

## Profile fields

| Field | Meaning |
| --- | --- |
| `scope` | Repository, component, and exact source paths covered by the declaration. |
| `assumptions` | Named assumptions, owners, statements, and review methods. |
| `trusted_computing_base` | Components whose correct behavior is required, including failure impact. |
| `mechanisms` | Owners, assurance planes, protected properties, linked IDs, and mechanism non-claims. |
| `required_evidence` | Producer, artifact path and kind, verification role and scope, validation command, and artifact non-claims. |
| `non_claims` | Profile-wide limits that every consumer must retain. |

The contract rejects duplicate IDs, unresolved references, unused declarations, unsupported planes, empty required arrays, and unknown fields.

## OnixOS lifecycle pilot

`onixos-lifecycle-proof-v1` maps the active OnixOS lifecycle proof work across all five assurance planes:

| Plane | Pilot mechanism |
| --- | --- |
| construction | Strict Rust finite-core boundaries and Octet policy. |
| semantic design | Pure transition logic bound to one canonical Kamacite FSM identity. |
| verification | Named Verus lifecycle obligations and Kani physical-switch harnesses. |
| effect control | Exact packaged proof tools, explicit argv, selected source, and no accepted host fallback. |
| provenance and readiness | Kamacite identity, Valence linkage, Cairn gates, and OnixOS candidate policy. |

The profile requires Octet, Kamacite, Rust conformance, Verus, Kani, Valence, and Cairn artifacts. It does not report that those artifacts exist or pass. The active OnixOS change remains the authority for implementation and completion state.

## Build and validate

Build the consumer package:

```bash
nix build .#octet-assurance-profile-onixos-lifecycle --no-link --print-out-paths
```

Run the focused freshness and fixture check:

```bash
nix build .#checks.x86_64-linux.assurance-profile-onixos-lifecycle --no-link -L
nix run .#octet-policy-validate
```

The package installs under:

```text
share/octet/assurance-profiles/onixos-lifecycle-proof-v1/
```

A downstream repository must select a published immutable Octet revision before it treats the package as durable input. The downstream adoption, profile identity pin, evidence mapping, and rollback remain consumer-owned.

## Claim boundary

A passing profile check proves typed declaration conformance and exact generated identity only. It does not prove evidence success, model completeness, verifier soundness, compiler correctness, complete dependency safety, whole-program correctness, hardware isolation, zero trust, physical behavior, deployment success, or release eligibility.
