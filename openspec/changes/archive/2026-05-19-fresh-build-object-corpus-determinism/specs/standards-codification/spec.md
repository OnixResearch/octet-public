## ADDED Requirements

### Requirement: Fresh-build object corpus determinism rail

Octet MUST provide a named fresh-build object corpus determinism rail that regenerates the expanded internal object corpus receipt with independently built Octet binaries and compares deterministic receipt identity while preserving bounded evidence claims.

#### Scenario: Fresh-build binaries reproduce expanded corpus receipt

- GIVEN the checked-in expanded internal object corpus taxonomy and receipt fixture
- WHEN the fresh-build object corpus rail runs independent Nix-built `cargo-octet` binaries over the selected corpus
- THEN the generated receipts match each other and the checked-in expanded receipt fixture for bytes and stable contract fields

#### Scenario: Fresh-build rail preserves bounded non-proof claims

- GIVEN a fresh-build generated object corpus receipt
- WHEN the rail inspects the receipt boundary fields
- THEN `claims_correctness` is false, `claims_semantic_equivalence` is false, and the rail describes the result as bounded local fresh-build evidence replay rather than correctness, semantic equivalence, cross-machine determinism, or exhaustive Rust coverage proof

#### Scenario: Contributor guide names the fresh-build replay check

- GIVEN contributors need to replay fresh-build object corpus evidence locally
- WHEN they read the contributor guide
- THEN it names `nix build .#checks.x86_64-linux.fresh-build-object-corpus-determinism --no-link -L` and preserves bounded local non-proof language
