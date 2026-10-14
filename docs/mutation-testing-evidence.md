# Mutation testing evidence

Octet hardens the optional `mutation-testing` rail with the `cargo-mutants-octet-metadata` profile.

The profile pins cargo-mutants 27.0.0, Rust `nightly-2026-03-21`, one package scope, fixed arguments, limits, retention, and outcome policy.

The checked profile identity is `b3:a40fc0726aea38932f42d6096a84846778e832472767b3c3129de4f7e69d9176`.

## Run

```console
cargo octet evidence collect \
  --rail mutation-testing \
  --profile cargo-mutants-octet-metadata \
  --config standards/mutation/profiles.json \
  --external-report evidence/mutation-source-manifest.json \
  --workspace-root . \
  --run \
  --artifact-dir artifacts/mutation
```

The source manifest uses schema `octet-source-manifest/v1`. Each file record contains a safe relative path and one BLAKE3 identity.

The shell rejects `--command`. It uses Bounded Exec through the pinned Octet runtime and a cleared child environment.

The profile selects deterministic cargo-mutants output with `--no-times`. Octet reads `outcomes.json` and `mutants.json` instead of terminal text.

## Import

Use the checked native outcomes path for a compatible import:

```console
cargo octet evidence collect \
  --rail mutation-testing \
  --profile cargo-mutants-octet-metadata \
  --config standards/mutation/profiles.json \
  --input .octet/mutants-output/mutants.out/outcomes.json \
  --external-report evidence/mutation-source-manifest.json \
  --workspace-root . \
  --artifact-dir artifacts/mutation
```

Without the hardened profile, existing normalized v1 inputs remain lower-authority transcript evidence.

## Outcomes

A passing receipt requires a successful unmutated baseline and one terminal outcome for every inventory mutant.

Octet preserves these classes:

- caught
- missed
- timeout
- unviable
- not-run
- tool-failure

Missed, not-run, tool-failure, and failed-baseline observations fail. Timeout and unviable observations require manual review.

Each mutant binds its package, safe source location, replacement, genre, source identity, terminal outcome, log, and retained diff.

## Artifacts

The bundle contains the source manifest, native outcomes, native inventory, normalized projection, bounded logs, and retained diffs.

Artifact validation remeasures all retained bytes. Valence export keeps transcript roles and does not create formal-proof roles.

## Non-claims

Mutation outcomes sample test-suite sensitivity. They do not prove test completeness, source correctness, whole-program correctness, or release eligibility.
