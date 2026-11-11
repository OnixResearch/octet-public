## Context

The existing Valence object-evidence stack has several durable rails, but operators still need to remember multiple command names and test filters. A thin Nix aggregate check can package them without changing the underlying evidence semantics.

## Goals

- Provide one stable Nix check name for selected Valence object evidence.
- Reuse existing rails rather than reimplementing the checks.
- Keep the gate scope and non-proof boundaries visible in docs.

## Non-Goals

- Do not make object hashes correctness proofs.
- Do not broaden selected internal corpus coverage into exhaustive Rust coverage.
- Do not claim cross-machine or cross-toolchain determinism.
- Do not add HIR/MIR extraction.

## Decisions

### 1. Compose existing derivations

**Choice:** Implement `valence-object-evidence` as a `pkgs.runCommand` that depends on existing object corpus, compatibility, caveat taxonomy, and fresh-build derivations.

**Rationale:** This keeps the aggregate check cheap to reason about and avoids drifting from already tested rails.

**Alternative:** Add a large new Rust test filter that invokes every flow. Rejected because it would duplicate coverage and be harder to maintain than a Nix aggregate over existing checks.

### 2. Add focused compatibility/caveat derivations if needed

**Choice:** Use `mkFingerprintDocsCheck` filters for object compatibility and caveat taxonomy so the aggregate has explicit named components.

**Rationale:** The object corpus gate covers much of the stack, but explicit sub-rails make the aggregate receipt self-describing.

## Risks / Trade-offs

- **Build time:** The aggregate will run multiple rails. Mitigation: it is a named operator/CI check, not part of every focused edit loop unless explicitly invoked.
- **Overclaiming:** A single green gate can sound stronger than its components. Mitigation: docs and spec require non-proof boundary wording.

## Validation Plan

- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`
- `nix develop -c bash -c 'cargo fmt --check && cargo test -p cargo-octet && openspec validate --all --strict --json && git diff --check'`
