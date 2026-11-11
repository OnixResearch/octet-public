# Durability claims

Octet detects selected false durability claims in Rust source and links saved
storage-fault evidence. It does not prove filesystem, device, or hardware
durability, and it does not execute a fault producer.

## Declared boundaries

Consumers declare durability boundaries in Nickel under
`standards/durability-claims/` (or a copy in their own repository), export
deterministic JSON, and point `durability_profile_path` in `dylint.toml` at the
generated file:

```toml
[octet]
durability_profile_path = "durability-claims.json"
```

Each boundary names a workspace-relative `source_scope`, exact
`boundary_functions` (a `::`-separated trailing suffix of the resolved
definition path, where a leading `crate::` is dropped), a
`required_observation` class (`synchronization` or `adapter-observation`),
`uncertain_outcome_types`, and optional `commit_surfaces` or
`adapter_observations`. Copy `templates/durability-claims-profile.ncl` and
`standards/durability-claims/contracts.ncl` to author a profile.

An unreadable or invalid profile is reported as an error and the durability
lints stay inert. No declared boundary means no durability finding.

## Findings

| Lint | Reports |
| --- | --- |
| `discarded_sync_result` | direct discard, `let _ =`, empty error arms, and unconditional error-to-success mapping for `sync_all`, `sync_data`, and configured commit results |
| `flush_only_durability` | a declared boundary whose only observation is `Write::flush()` |
| `retry_sync_after_error` | a same-visible-receiver synchronization retried after a handled failure with no intervening write, reopen, replacement, quarantine, or reconciliation |

Diagnostics explain the bounded options: record an explicit uncertain outcome,
quarantine the artifact, reopen or replace the handle, rewrite from the source
of truth, or reconcile against storage. No single repair proves durability.

## Storage-fault assurance

`standards/policies/assurance-profiles/storage-fault/` declares required cases
(including at least one negative case) and the exact BLAKE3 identities of saved
producer and Cairn receipts. Validate saved evidence with:

```bash
octet-standards storage-fault-assurance \
  --profile storage-fault.json \
  --producer-receipt producer.json \
  --cairn-receipt cairn.json \
  --output json
```

Validation reads the supplied files only. It never starts, locates, or waits for
a producer executable. It rejects stale source, candidate, or receipt
identities, missing or failed required cases, broken Cairn linkage, missing
non-claims, and receipts that claim more than bounded observation.

## Claim boundary

Passing source checks proves only the absence of selected source shapes in
declared scopes. Passing assurance validation proves only linkage to selected
saved evidence. Neither proves that a synchronization call, configured commit,
adapter observation, or fault campaign succeeded.
