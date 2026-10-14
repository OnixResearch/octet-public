## Why

Octet still hosts an in-repository `crates/valence-core` copy even though standalone Valence now owns the richer Evidence IR, Function Object, provenance, stack-provenance, and non-claim semantics. That duplicate crate keeps Octet self-contained for historical gates, but it also lets downstream users bind to the older Octet-hosted package instead of the canonical Valence crate.

This change defines the Octet-side cutover from hosted readiness seam to external Valence dependency while keeping Octet responsible for source scanning, CLI orchestration, proof rails, artifact publication, and release evidence that remain Octet-owned.

## What Changes

- Replace Octet's internal `valence-core` workspace dependency with a reviewed standalone Valence dependency or compatibility shim.
- Preserve byte-compatible Function Object v1, evidence-index, and evidence-codebase-index behavior through positive fixtures.
- Add negative drift fixtures for stale hosted code, wrong dependency source, missing non-claims, and widened evidence claims.
- Keep Octet shell responsibilities explicit: filesystem reads, Rust source scanning, source-marker mutation, corpus orchestration, supplement orchestration, stdout/stderr, and process exits stay in Octet.
- Document migration guidance for Aspen and other consumers that currently depend on Octet's `valence-core` package name.

## Impact

- **Octet** stops being the canonical package source for Valence semantics.
- **Valence** becomes the dependency target for `valence-core` contracts and DTOs.
- **Consumers** retain a bounded compatibility path while moving to standalone Valence.
- **Testing** must prove positive compatibility and fail-closed stale-copy detection before any internal crate removal.
