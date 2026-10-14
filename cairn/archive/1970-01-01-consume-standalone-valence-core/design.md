## Design

Octet treats the standalone Valence crate as the authority for Valence-owned pure evidence semantics. The cutover is staged rather than deleting the hosted crate immediately: first add a dependency/source manifest and compatibility fixtures, then move `cargo-octet` call sites to the standalone dependency, then retire or gate the hosted copy once downstream compatibility is proven.

### Decisions

1. **Dependency direction is one-way.** Octet MAY consume standalone Valence for Evidence IR and Function Object core logic. Valence MUST NOT depend on Octet CLI shells, source scanners, Dylint passes, proof rail orchestration, or artifact publication paths.

2. **Compatibility is fixture-backed.** Function Object v1 hashes, evidence-index output, and evidence-codebase-index reports must be compared against known Octet-hosted fixtures before the internal copy is removed or turned into a shim.

3. **Stale hosted code fails closed.** During transition, any remaining Octet-hosted `valence-core` files must either be byte-identical to the canonical Valence source or listed in an explicit migration exception with a non-claim boundary.

4. **No semantic overclaim.** Passing cutover checks proves dependency/source ownership and DTO compatibility only. It does not prove behavioral correctness, semantic equivalence of arbitrary Rust code, pure-cache eligibility, release eligibility, external consumer compatibility, or cross-toolchain determinism.

5. **Consumers get a migration map.** Aspen and other downstream users that currently depend on `octet` package `valence-core` need a documented target coordinate, compatibility status, and cutover validation command.

### Validation shape

Use a pure decision core over an in-memory manifest of modules, dependency coordinates, compatibility fixture hashes, and migration exceptions. A thin Octet shell may collect filesystem state and write receipts. Positive tests cover byte-compatible canonical modules and approved shims. Negative tests cover stale hosted files, wrong dependency source, missing fixture rows, missing non-claims, and claims that turn Valence compatibility into correctness or release-readiness proof.
