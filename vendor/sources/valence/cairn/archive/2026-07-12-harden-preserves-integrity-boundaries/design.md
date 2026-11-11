## Context

The current hash helper operates on rendered text and drops any line containing a self-hash field name. That is not equivalent to removing the typed top-level self-hash fields. It also makes payload bytes sensitive to formatting and permits attacker-controlled content to be omitted from the digest. Separately, required and forbidden non-claim phrases are checked as text rather than as authority-bearing types.

## Decisions

### Parse, transform, and canonically encode

**Choice:** Parse the Preserves envelope into its typed value, remove only the schema-declared top-level self-hash fields, and encode the remaining value with the canonical packed writer before hashing with BLAKE3.

Payload keys and string values remain hash-bearing even when they contain `receipt_hash`, `preserves_content_hash`, or similar text. Parsing, structural field selection, and canonical encoding form a pure deterministic core. File reads and writes remain in the CLI shell.

### Version the corrected profile

**Choice:** Introduce a new hash-profile identity. Validation rejects receipts that claim the corrected profile but carry a legacy digest. Legacy receipts may be migrated only by an explicit adapter that receives the original bytes, validates the legacy identity, computes the corrected identity, and emits a migration receipt binding both identities.

Existing receipts are not rewritten in place. Downstream consumers must select accepted profile versions explicitly.

### Make claim authority typed

**Choice:** Model supported claim class, verification role, prohibited claim capabilities, and required boundary identifiers as enums or registry-backed typed values. Human-readable prose is display metadata and cannot grant a stronger claim or satisfy a missing typed boundary.

A record that declares an identity/linkage-only class while asserting correctness, equivalence, authority, or release eligibility in another field fails deterministically. Unknown claim classes or capabilities fail closed.

### Keep profile work separate

This change fixes shared Preserves identity and claim-policy primitives. It does not add new evidence kinds, graph edges, or profile-specific payload semantics. Active evidence-chain profile changes consume the corrected primitive after this package archives.

### CI scope

The checked-in workflow runs only `nix flake check`. Focused Rust and fixture tests are part of implementation evidence but are not added as separate CI jobs in this change.

## Cross-repository ordering

Downstream Octet, Aspen, Mantle, and Cairn remediation packages must consume an archived receipt for this change before accepting corrected-profile evidence. Until Cairn's cross-repository dependency feature is available, those packages record this prerequisite in their designs and tasks rather than using an unsupported metadata field.

## Risks / Trade-offs

- Correctly invalidating legacy receipts creates migration work, but silent acceptance would preserve the collision.
- Typed claims require adapters for existing prose-heavy formats, but prevent text from acting as authority.
- Canonical packed encoding may change golden hashes; fixture updates must be explicit and reviewable.
