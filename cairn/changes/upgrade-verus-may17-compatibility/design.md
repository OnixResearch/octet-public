# Design

## Decision boundary

The May 17 release remains a candidate until exact Octet and downstream proof commands pass. A version number, release asset hash, package build, source compilation, partial verification count, or verifier count followed by linker failure is not compatibility evidence.

The Nickel profile is the source of truth. Nix fetches the published ZIP by SHA-256, supplies Rust `1.95.0` from Octet's locked closure, and emits the full Verus revision plus BLAKE3 identities after materialization. Downstream repositories retain proof-source and receipt authority.

An adapted corpus pass is admissible only when the immutable upstream source and reviewable patch identities are both recorded, trust-marker inventory does not grow, and the native verifier command exits zero. Such a pass is scoped to the adapted source and does not certify the unmodified upstream revision.

## Functional core and shell

The typed profile and compatibility rows are the pure reviewed decision input. The existing Nix derivation is the shell that fetches, patches runtime linkage, wraps the verifier, and emits identities. Proof commands and receipt reducers consume the package without reconstructing its closure.

## Alternatives

- Keep April and port every old corpus: rejected because April retains a node-replication verifier internal error.
- Select a later release: rejected because tested later releases regress node replication's collection semantics.
- Select May 17 after package build only: rejected because availability is not proof compatibility.
- Select May 17 only after the closed portfolio passes: chosen.

## Non-claims

This change does not establish Verus, Rust, Z3, Vstd, or adapted-source semantic soundness. Passing adapted upstream source does not certify its unmodified revision, and consumer compatibility rows do not transfer proof authority.
