## Context

The structural-enforcement work established Nickel as the typed policy authoring surface and Rust as the checker/runtime. The next seam is a deterministic, consumer-facing artifact bundle: CI should be able to generate policy artifacts once, inspect the exact generated files, and pass the artifact directory to the standards checker.

## Goals / Non-Goals

**Goals:**

- Name the generated artifacts explicitly.
- Keep `tigerstyle-policies.json` as the canonical machine input for checker runs.
- Include a compact deterministic TOML summary for CI/operator review.
- Include seed manifests as a separate machine-readable artifact.
- Prove `tigerstyle-standards check` consumes the artifact directory.

**Non-Goals:**

- Commit generated artifacts into the repository in this slice.
- Replace Nickel contracts or move policy validation out of Nickel.
- Add a full external consumer rollout in this slice.

## Decisions

### 1. Export artifacts from registry JSON, not Nickel source

**Choice:** `tigerstyle-standards export-artifacts` accepts a JSON registry and writes the generated bundle.

**Rationale:** Nickel remains the typed authoring and validation surface, while Rust owns deterministic artifact layout and consumer checker semantics.

### 2. Let checks consume an artifact directory

**Choice:** `tigerstyle-standards check` accepts either `--registry` or `--policy-artifacts-dir`.

**Rationale:** Existing registry-based tests and scripts keep working, while consumer gates can depend on named generated artifacts.

### 3. Keep check config as minimal deterministic TOML

**Choice:** `tigerstyle-check-config.toml` lists schema metadata, checker IDs, families, hard policies, structural policies, and verified policies.

**Rationale:** This gives reviewers and CI a compact stable surface without introducing another parser dependency into the standards crate.

## Validation Plan

- Unit-test artifact rendering stability.
- Integration-test artifact export and artifact-backed checker execution.
- Run `nix run .#tigerstyle-policy-artifacts -- <dir>` against live Nickel export.
- Run the Nix `standards-fixtures` check, which must export artifacts and use `--policy-artifacts-dir` for source checks.
