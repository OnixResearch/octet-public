## Design: Valence provenance chain export

### Boundary

Octet exports adapter-ready provenance facts from Octet-owned artifacts. It does not verify Mantle releases and does not claim cross-stack traceability by itself.

```text
Octet artifact verify
  -> Octet Valence provenance export
  -> Valence stack-provenance adapter
  -> optional Mantle external evidence sidecar
```

### Export command

The export can be exposed either as a dedicated command or as an artifact verifier output flag:

```bash
cargo octet artifact verify \
  --artifact-dir artifacts/octet \
  --emit-valence-provenance target/octet-valence-provenance.json
```

or:

```bash
cargo octet provenance export \
  --artifact-dir artifacts/octet \
  --out target/octet-valence-provenance.json
```

The export is written only after Octet artifact verification succeeds for the referenced artifact directory.

### Export contents

The export should include normalized records for:

- `provenance.jsonl` source ID, checker ID, evidence tier, artifact locator, BLAKE3, checked scope, and parent artifacts;
- proof-card IDs, policy/requirement IDs, receipt paths, proof target paths, verifier identity summaries, and artifact identities;
- receipt identities and bounded log identities when present;
- producer registry decisions for checker IDs and source ID families;
- diagnostics for skipped or manual-review-only evidence.

The format may reference Valence chain field names, but Octet remains responsible for translating Octet-specific artifacts into that normalized export.

### Evidence-tier discipline

The exporter must preserve Octet's bounded evidence tiers:

- lint and policy evidence remain `static_lint` or `policy_check`;
- transcript evidence remains `transcript_assertion`;
- imported Trellis bundles remain `manual_review_required` unless Octet reruns and validates the proof with an Octet-owned verifier receipt;
- only validated verifier-backed receipts may export `formal_proof` facts.

### Validation

The export path fails closed when:

- artifact verification fails;
- a provenance record has a stale BLAKE3 digest;
- a parent artifact is missing or cyclic;
- a proof card references a missing receipt or provenance artifact;
- a checker ID is not registered;
- a manual-review or imported Trellis record is promoted to `formal_proof`.

### Non-claims

The export is input to Valence stack verification. It does not prove release eligibility, whole-program correctness, semantic equivalence, or that Mantle bundled the final sidecar.
