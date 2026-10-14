## ADDED Requirements

### Requirement: Evidence log retention boundary

r[octet.evidence_log_storage.retention] Octet SHOULD keep concise review summaries, exact replay commands, checked scopes, and scoped claim boundaries in-tree while allowing bulky raw evidence logs to move to content-addressed storage.

#### Scenario: Summary remains reviewable

r[octet.evidence_log_storage.retention.summary]

- GIVEN a bulky raw evidence log is externalized, compressed, or moved into a content-addressed bundle
- WHEN a reviewer opens the in-tree summary
- THEN the summary SHOULD name the exact replay command, checked scope, pass/fail signal, BLAKE3 identity, byte count, and claim boundary.

#### Scenario: Historical claim is not strengthened

r[octet.evidence_log_storage.retention.no_overclaim]

- GIVEN an archived evidence log is transformed into summary-plus-manifest form
- WHEN the summary is written
- THEN the summary MUST preserve the original scoped claim boundary and MUST NOT imply broader proof or release readiness than the original evidence supported.

### Requirement: Content-addressed evidence manifest

r[octet.evidence_log_storage.manifest] Content-addressed evidence log manifests MUST record artifact locator, BLAKE3 digest, byte count, replay command, checked scope, capture source, and retention classification for each externalized raw log.

#### Scenario: Manifest identifies artifact bytes

r[octet.evidence_log_storage.manifest.identity]

- GIVEN a raw evidence artifact is listed in an evidence log manifest
- WHEN manifest verification runs
- THEN the verifier MUST recompute the artifact BLAKE3 digest and byte count and compare them to the manifest entry.

#### Scenario: Manifest keeps replay metadata

r[octet.evidence_log_storage.manifest.replay]

- GIVEN a raw evidence artifact is listed in an evidence log manifest
- WHEN a reviewer inspects the manifest
- THEN the entry MUST include the replay command and checked scope needed to understand the artifact's evidence boundary.

### Requirement: Evidence log verification fails closed

r[octet.evidence_log_storage.fail_closed] Evidence log verification MUST fail closed when a manifest references a missing artifact, wrong BLAKE3 digest, stale byte count, missing replay command, missing checked scope, or unsupported retention classification.

#### Scenario: Complete artifact passes

r[octet.evidence_log_storage.fail_closed.complete]

- GIVEN every manifest entry references present bytes with matching BLAKE3 digest, byte count, replay command, checked scope, and supported retention classification
- WHEN evidence log verification runs
- THEN verification MUST pass.

#### Scenario: Tampered artifact fails

r[octet.evidence_log_storage.fail_closed.tampered]

- GIVEN a manifest entry references missing or modified artifact bytes, omits replay metadata, or names an unsupported retention classification
- WHEN evidence log verification runs
- THEN verification MUST fail with a diagnostic naming the rejected entry and field.
