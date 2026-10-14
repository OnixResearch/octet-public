# Design: Formal proof Valence export fixture

## Shape

Add a frozen Octet fixture for a scoped formal-proof artifact bundle:

```text
proof rail manifest + source/proof path identities
  -> Verus receipt
  -> proof-cards.json
  -> provenance.jsonl formal_proof record
  -> artifact verification report
  -> Valence provenance export
```

The fixture is static and hermetic. It is suitable for Valence stack-chain tests and Cairn readiness fixtures. It must not run Verus, inspect live sibling repos, or fetch dependencies during fixture validation.

## Export preconditions

The export validator accepts a fixture only when:

- the proof receipt status is passed;
- the proof card links to the receipt, proof artifact, verifier identity, and provenance record;
- `cargo octet artifact verify` evidence is present and current for the fixture identities;
- the Valence export preserves evidence tier, typed edges, parent links, proof-card summary, diagnostics, and non-claims;
- imported Trellis references remain manual-review unless an Octet-owned rerun receipt is present.

## Negative fixtures

Negative fixture rows cover:

- missing or failed proof receipt;
- stale proof-card BLAKE3 identity;
- Valence export produced without artifact verification evidence;
- manual-review imported Trellis record marked `formal_proof`;
- Hegel/Kani/lint/policy evidence promoted to `formal_proof`;
- missing required non-claim text;
- release-readiness or whole-program-correctness overclaim in export text.

## Functional core / shell boundary

The pure core validates parsed fixture rows and expected identity values. The shell may read fixture files and render JSON, but it must not decide proof eligibility from raw text without routing through the pure validator.

## Validation

Focused validation should run:

```bash
nix develop -c cargo test -p cargo-octet formal_proof_valence_export_fixture
nix run .#octet-policy-validate
nix run path:/path/to/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/path/to/git/OnixResearch/cairn#cairn -- gate proposal formal-proof-valence-export-fixture --root .
nix run path:/path/to/git/OnixResearch/cairn#cairn -- gate design formal-proof-valence-export-fixture --root .
nix run path:/path/to/git/OnixResearch/cairn#cairn -- gate tasks formal-proof-valence-export-fixture --root .
```

## Risks

- **Fixture becomes confused with live proof execution**: make all commands and docs state that it is a frozen adapter fixture.
- **Tier promotion through export**: reject any non-Verus formal-proof source unless backed by a scoped Octet-owned proof receipt.
- **Consumer overclaim**: preserve explicit non-claims in the export and fixture report.
