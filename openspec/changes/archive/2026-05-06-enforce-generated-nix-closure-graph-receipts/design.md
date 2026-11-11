## Context

The previous Nix closure slice introduced `proof-verifier-nix-derivation` and `proof-verifier-nix-closure` receipt lines plus a flake package that copies the Nix `exportReferencesGraph` output. The Rust checker still computes the configured Nix closure digest from source-root relative fixture paths using the generic path+file digest manifest algorithm.

## Decisions

### 1. Graph artifact digest is first-class receipt metadata

**Choice:** Add a `proof-verifier-nix-closure-graph-blake3` receipt line and a corresponding typed Nickel/Rust field.

**Rationale:** Keep the existing closure identity/digest lines stable while adding a direct obligation for the actual generated graph bytes.

### 2. Rust uses graph-aware canonicalization

**Choice:** For configured graph paths, parse the `exportReferencesGraph` text into records, sort records by store path, sort references inside each record, and hash canonical `path\nref-count\nrefs...` text with BLAKE3.

**Rationale:** This makes digest enforcement independent of incidental graph record/reference order while still binding to the complete recursive closure path set and edges emitted by Nix. Malformed graph input falls back to mismatch rather than silently accepting fixture text.

### 3. Nix manifest package exports graph digest sidecar

**Choice:** The flake package that uses `exportReferencesGraph` also writes a `tigerstyle-standards-closure.graph.blake3` sidecar and includes the digest line in `proof-nix-closure.txt`.

**Rationale:** Operators can inspect the generated graph and its digest without recomputing it manually, and the Nix check proves the sidecar and manifest stay aligned.

## Risks / Trade-offs

- The graph format is Nix-specific and line-oriented; tests include order-insensitive canonicalization and malformed graph rejection.
- Current fixture policy uses a committed small graph fixture for cargo tests while the Nix package proves the live generated graph path.
