## Context

The existing fresh-target rail exercises repeated receipt generation under distinct `CARGO_TARGET_DIR`s using the test binary. The expanded object corpus fixture broadens source coverage across Octet/Slotcar. This change adds a separate build-level rail using Nix-built `cargo-octet` binaries as the receipt producers.

## Goals / Non-Goals

**Goals:**

- Build separate fresh `cargo-octet` package derivations from the same checked source.
- Use each built binary to generate an expanded internal object corpus receipt.
- Assert byte-stable receipts and stable fields (`schema`, `schema_version`, `object_count`, `source_paths`, `object_set_hash`, `caveat_counts`, `pure_cache_blocked_count`, and `boundary`).
- Assert fixture parity with `expanded-internal-corpus-receipt.json`.

**Non-Goals:**

- No claim of correctness, semantic equivalence, or full CAF runtime support.
- No cross-machine, cross-toolchain, or cross-Nixpkgs determinism proof.
- No downstream Aspen/Crunch pilot.

## Decisions

### Nix check as the primary rail

**Choice:** Add a named flake check that depends on two Nix-built `cargo-octet` package outputs and runs their binaries against the expanded internal corpus.

**Rationale:** A flake check naturally proves the checked-in source, fixtures, and built binary agree in a fresh build context. It also avoids making ordinary `cargo test -p cargo-octet` spawn expensive nested package builds.

**Alternative:** Put `cargo build --target-dir ...` directly inside the Rust integration test. Rejected because it is slower, more fragile under Nix sandboxes, and less clearly a fresh-build operator rail.

## Risks / Trade-offs

**Build cost:** Two package derivations can add CI work. Mitigation: keep the check named and focused, and reuse existing cargo artifacts/dependency cache.

**Overclaim risk:** The spec and check output must call this bounded local fresh-build receipt repeatability only.
