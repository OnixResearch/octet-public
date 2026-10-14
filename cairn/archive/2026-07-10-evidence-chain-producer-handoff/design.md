## Design

Octet emits a producer handoff envelope for any evidence rail that wants stack-chain treatment. The envelope is profile-neutral and contains typed identity/linkage fields; the profile payload remains owned by the rail. Function-address export becomes the first adopter and continues to provide Rust-specific locators, spans, and source-token dependency identity in its profile payload.

### Decisions

- **Producer handoff is generic.** The envelope names `evidence_kind`, producer tool identity, run identity, policy/profile hashes, canonical roots, parent links, compatibility projections, and non-claims.
- **Profile payloads stay bounded.** Function-address details remain under the function-address profile; proof/lint/dependency rails can add their own profile payloads later.
- **JSON remains projection.** Octet can emit JSON for operators, but stack authority moves through Kamacite canonical receipts.
- **Parent links are typed.** Source roots, proof roots, receipt roots, and compatibility projection hashes are not interchangeable.
- **No evidence truth promotion.** Handoff means Octet produced source/proof/check evidence; it does not make downstream lifecycle or release claims.

### Validation shape

Positive fixtures cover a function-address producer handoff that Kamacite can canonicalize. Negative fixtures cover missing evidence kind, untyped parent hash, malformed BLAKE3, JSON-only authority, missing policy hash, duplicate canonical root, and non-claim weakening.
