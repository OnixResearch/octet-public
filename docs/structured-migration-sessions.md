# Structured migration sessions

Structured migration sessions make larger Octet renames and artifact transitions reviewable before any files are rewritten. They separate mutable pointers (names, paths, config keys, command aliases, artifact locators) from stable content identities (BLAKE3 hashes, source fingerprints, policy IDs, receipt hashes).

## Session receipt

A session JSON receipt uses schema `octet-structured-migration-session/v1` and records:

- `session_id` and `migration_kind` (`namespace`, `lint-policy`, `config`, `command`, `artifact-layout`, or `evidence-schema`).
- `old_pointer` and `new_pointer` as mutable names or locators.
- `identity_evidence` with BLAKE3-backed identities when available.
- `compatibility` entries classified as `canonical`, `compatibility-alias`, `deprecated`, or `removal-target`.
- `affected_surfaces` split by `mutability`: `live`, `generated`, `archive-read-only`, or `mutable-archive`.
- `planned_operations` with `operation_kind` (`record-only`, `rewrite`, or `replace`) and `safety` (`safe` or `manual-review`), plus `verification_commands`, `todos`, `status`, and explicit `non_claims`.

Start from `templates/structured-migration-session.json` when creating a review fixture or saved migration receipt.

## Plan, status, verify

All inspection commands are non-mutating:

```sh
cargo octet migrate session plan --session migration-session.json --output-format json
cargo octet migrate session status --session migration-session.json --output-format json
cargo octet migrate session verify --session migration-session.json --output-format json
```

Verification fails closed for missing pointer identities, unknown migration kinds, unknown compatibility classes, unknown surface mutability, operation kinds, or safety classes, stale BLAKE3 pairs, missing alias coverage, missing removal boundaries, and archive/read-only rewrites. Each `affected_surfaces` locator may appear only once, so a `live` entry cannot shadow an `archive-read-only` entry for the same path. Every `rewrite` or `replace` operation must target a declared affected surface; a `mutable-archive` target also needs an `archive_mutation_rationale`.

## Guarded apply

Apply is intentionally narrow. It writes a pre-write operation manifest for selected safe `record-only` operations:

```sh
cargo octet migrate session apply \
  --session migration-session.json \
  --operation-manifest artifacts/migration/operation-manifest.json \
  --output-format json
```

Use `--dry-run` to render the same operation set without treating it as an apply decision. Repeated apply over the same session is deterministic and idempotent at the operation-manifest level.

The manifest lists only operations with `safety: "safe"` and `operation_kind: "record-only"`. Any other planned operation is reported as `apply.unsupported_operation`; when the session has any diagnostic (validation failure or unsupported operation), apply prints the report, exits 1, and does not write the operation manifest.

## Boundary

Structured migration sessions prove reviewed name-pointer and artifact linkage only. They do not prove behavioral correctness, semantic equivalence, source compatibility, or release safety without separate evidence.
