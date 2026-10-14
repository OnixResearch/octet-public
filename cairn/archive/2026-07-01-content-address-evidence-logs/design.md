## Context

Repository stats show archived lifecycle evidence is the dominant size contributor, especially `.txt` transcripts in `openspec/changes/archive`. These files are legitimate evidence, but raw logs do not always need to be inline when a deterministic manifest, BLAKE3 digest, and compact review summary can preserve auditability.

## Decisions

### 1. Keep review summaries in-tree

**Choice:** Store concise summaries, exact replay commands, scoped claims, pass/fail signals, and BLAKE3 identities in-tree.

**Rationale:** Reviewers should not need to fetch bulky logs to understand what evidence claims. Inline summaries also preserve local review and grepability.

### 2. Content-address bulky raw logs

**Choice:** Move or compress raw transcripts above the configured retention threshold into content-addressed bundles with BLAKE3 digests and byte counts.

**Rationale:** This keeps repository size bounded while preserving deterministic integrity checks.

### 3. Verify manifests fail closed

**Choice:** Add a verifier that rejects missing, tampered, stale, or metadata-incomplete evidence artifacts.

**Rationale:** Externalizing or compressing evidence is safe only if artifact identity and replay metadata are checked.

### 4. Do not rewrite historical claims silently

**Choice:** When historical archive logs are transformed, preserve original command text, checked scope, and claim boundaries in the in-tree summary.

**Rationale:** Evidence compaction must not make old claims look stronger or lose replay context.

## Risks / Trade-offs

- Moving historical logs can make offline review harder. Mitigate by keeping summaries and making bundle verification deterministic.
- Compression can obscure text diffs. Mitigate by storing extracted headings/pass signals and exact replay commands in plaintext summaries.
- Existing references may point to raw paths. Mitigate with redirect manifests or compatibility path notes during migration.

## Validation

- Positive: manifest verification accepts present artifacts whose bytes match their BLAKE3 digests and whose summaries name replay commands and checked scope.
- Negative: missing artifact, wrong digest, stale byte count, missing command, or missing checked scope fails verification.
- Regression: archived evidence summaries remain grep-friendly for required rollout and release evidence claims.
