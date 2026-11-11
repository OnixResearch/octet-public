# Stack provenance Preserves corpus

`corpus.json` is the checked-in freshness corpus for canonical stack-provenance metadata values.

It records:

- positive Preserves projections for the Octet Valence export, Valence sidecar, Mantle manifest view, built-binary thread input envelope, and built-binary thread report;
- the BLAKE3 `preserves_content_hash` over each canonical Preserves text value;
- a report drift case proving that diagnostic changes produce a deterministic Preserves hash mismatch and changed-field list.

The corpus is checked by the `stack_provenance_preserves_*` Rust tests. Update this file only when the metadata value contract intentionally changes.

Boundary: this corpus proves canonical metadata value identity only. It does not measure executable bytes, replace Mantle sidecar byte digests, or prove behavioral correctness, semantic equivalence, requirement satisfaction, release eligibility, verifier soundness, or whole-program safety.
