## Design

Octet should not become the canonical Preserves receipt authority for function-address evidence. Instead, Octet provides a complete and validated handoff envelope that Kamacite can encode as canonical Preserves bytes. The handoff envelope is pure source-shape data and carries the same non-claims as the JSON projection.

### Decisions

1. **JSON remains a projection.** `function-addresses.json` is useful for operators and compatibility but is not the portable authority.
2. **Kamacite owns canonical bytes.** Canonical Preserves binary bytes and their BLAKE3 identity are produced by Kamacite.
3. **Octet handoff is complete.** Octet emits all data needed to build the Kamacite receipt without reparsing Rust or recomputing spans downstream.
4. **Handoff rejects overclaims.** Any text or marker claiming semantic Rust equivalence, compiler correctness, whole-program safety, verifier soundness, or release eligibility fails closed.
5. **Functional core stays pure.** Handoff completeness validation is an in-memory check over function-address records; CLI/artifact writing stays in the shell.

### Validation shape

Positive fixtures cover a valid Octet report that Kamacite can encode as a Preserves receipt without adapter-side span recomputation. Negative fixtures cover missing spans, absolute paths, missing dependency kind, malformed BLAKE3, unsupported producer metadata, and forbidden claims.
