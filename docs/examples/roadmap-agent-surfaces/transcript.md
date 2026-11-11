# Roadmap agent surfaces transcript

This executable transcript documents the Unison-inspired agent-facing roadmap surfaces. It is intentionally command-shaped: operators can copy each command into a scratch directory with the named JSON fixtures and expect deterministic JSON receipts.

```octet-transcript
$ cargo octet roadmap artifact store --kind prompt --input prompt.json --store-dir store
=> drops secret-like and local-only fields, replaces a leading workspace, HOME, or temp-dir root in string values with <workspace>, <home>, or <temp> while keeping the relative remainder and leaving other strings and identity fields verbatim, rejects non-hash dependencies, and returns a stable b3: artifact hash

$ cargo octet roadmap artifact inspect --hash b3:<artifact> --store-dir store
=> accepts only b3: plus 64 lowercase hex, re-hashes the stored record, and reports artifact_kind, dependencies, and replay prerequisites

$ cargo octet roadmap artifact replay --hash b3:<artifact> --store-dir store
=> verifies the full dependency closure the same way, fails closed with missing_dependency when one is absent, and exits 2 for a record that does not hash to its name

$ cargo octet roadmap ability enforce --manifest ability-manifest.json --trace effect-trace.json
=> denies effects not declared by the ability manifest and reports denied_effects; allowed_effects and effects must be arrays of strings

$ cargo octet roadmap pointer set --registry registry.json --name done-review --kind prompt --hash b3:<artifact> --receipt pointer-receipt.json
=> records name-to-hash updates with an old_hash rollback receipt

$ cargo octet roadmap pointer rollback --registry registry.json --receipt pointer-receipt.json
=> restores the prior immutable hash from the rollback receipt

$ cargo octet roadmap session check --input session.json
=> verifies typed/versioned session records with identity and redaction metadata

$ cargo octet roadmap session migrate --input session-v0.json --output session-v1.json
=> reads schema_version (or legacy original_schema_version), refuses newer schemas and sessions without a string identity, and writes a migration receipt without changing that identity

$ cargo octet roadmap session todo --input session.json
=> summarizes structured todo kinds, statuses, sources, next actions, and blockers

$ cargo octet roadmap pure-cache check --key cache-key.json --store-dir cache --output-hash b3:<output>
=> reports hit when command identity, input hashes, config hash, tool revision, and environment allowlist match; a miss names miss_reason new-key or the changed_key_components, comparing only b3_<64 hex>.json store files that hold a cache_key (other files, such as artifact records or the key itself, are skipped; an unparseable, non-object, or key_components-less entry fails closed with exit 2); a hit with a different output hash reports output-mismatch and exits 1
```
