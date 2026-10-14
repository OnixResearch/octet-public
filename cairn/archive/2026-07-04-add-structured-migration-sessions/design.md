## Context

Octet has ongoing namespace, command, config, and evidence-surface evolution. The README already documents canonical and compatibility names, and `cargo octet migrate start` writes a deterministic manifest before safe application. The next step is to make migrations session-oriented: a reviewer should be able to inspect the intended rename or artifact transition, see what identities are preserved, see which compatibility aliases remain, run deterministic checks, and decide whether a later removal boundary is acceptable.

## Decisions

### 1. Treat names as pointers to evidence identity

**Choice:** Migration sessions SHOULD model names, paths, config keys, lint attrs, command names, and artifact locators as mutable pointers. Content identities such as source fingerprints, artifact hashes, policy IDs, and receipt hashes remain separate fields.

**Rationale:** This adapts Unison's names-vs-hashes lesson without replacing Rust files or Git. It lets Octet rename surfaces while proving which reviewed identity each name maps to.

### 2. Session records are reviewable receipts

**Choice:** A session receipt MUST include session ID, migration kind, old pointer, new pointer, identity evidence when available, compatibility classification, affected surfaces, planned operations, verification commands, observed status, and explicit non-claims.

**Rationale:** Larger migrations need more than a diff. A receipt gives reviewers a stable checklist and lets future checks reject stale or incomplete transitions.

### 3. Mutation remains explicit and dry-run first

**Choice:** Plan/status/verify modes MUST be non-mutating. Apply modes, if implemented, MUST require explicit operator intent and emit an operation manifest before writing.

**Rationale:** Migration tools touch many files and can corrupt evidence if they rewrite archives or transcripts without review. Dry-run-first behavior preserves safety.

### 4. Compatibility aliases are first-class

**Choice:** Sessions MUST distinguish canonical, compatibility alias, deprecated, and removal-target pointers and MUST record the removal boundary or blocking reason for each non-canonical pointer.

**Rationale:** Octet has live consumers. Migrations should not silently break source attrs, config sections, runner aliases, or evidence readers.

### 5. Verification covers identity and behavior separately

**Choice:** Session verification SHOULD check stale hashes, missing affected surfaces, alias coverage, expected output stability, and artifact freshness. It MUST keep identity linkage separate from behavioral correctness or semantic equivalence.

**Rationale:** A migration receipt can prove that the planned surfaces were updated and replayed; it cannot prove the new behavior is semantically identical unless separate tests or proofs do so.

## Risks / Trade-offs

- A general rewrite engine can be dangerous. Start with manifests, status, and verification; keep broad apply support conservative.
- Some historical archive references should not be rewritten. Sessions need explicit archive/read-only classifications.
- Surface inventories can miss files. Negative tests should include missing docs/config/artifact references and require explicit allowlists for intentionally retained legacy strings.
