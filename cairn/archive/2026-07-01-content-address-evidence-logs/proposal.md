## Why

The repository carries many historical evidence logs under lifecycle archives. Those logs are useful for review, but large raw transcripts dominate repository size and make ordinary source checkout and review heavier than necessary. Octet already uses BLAKE3 identity and provenance records for evidence, so bulky logs should follow the same content-addressed discipline.

A content-addressed evidence log policy can keep review summaries in-tree while making large replay artifacts deduplicated, verifiable, and easy to archive.

## What Changes

- Define which evidence artifacts stay inline and which move to content-addressed storage or compressed manifest-backed bundles.
- Add BLAKE3 manifests for bulky evidence logs, with command, checked scope, bytes, and digest metadata.
- Keep short human-review summaries and exact replay commands in-tree.
- Add verification tooling or checks that confirm referenced evidence artifacts are present, match their BLAKE3 digest, and preserve required replay metadata.

## Impact

- **Files**: `cairn/archive/**`, `openspec/changes/archive/**`, evidence docs, artifact manifest tooling, `.gitignore` or retention docs, and release/evidence verification commands.
- **Testing**: positive manifest verification for existing artifacts, negative missing/tampered artifact tests, and docs checks proving summaries retain replay commands and digest identities.
