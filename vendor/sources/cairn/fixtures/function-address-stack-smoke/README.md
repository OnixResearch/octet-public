# Function-address stack smoke fixture

This fixture commits one tiny Rust crate and expected bounded artifacts owned by Octet, Kamacite, Valence, Mantle, and Cairn.

## One-command Cairn smoke

```console
nix develop --option secret-key-files '' --option builders '' -c \
  cargo test -p cairn-cli --test function_address_stack_smoke
```

The positive test recomputes the source identity, validates the Octet function-address/span contract, checks receipt links, roles, claim scopes, graph counts, and non-claims, then invokes Cairn release readiness with the committed Kamacite, Valence, Mantle, and graph inputs. The negative test mutates stale function address, bad span, stale receipt, wrong role, wrong claim scope, missing evidence, and overclaim boundaries one at a time.

## Producer commands and ownership

- **Octet** owns Rust source-shape extraction. Regenerate from `source/` with `nix run path:/path/to/octet#cargo-octet -- check --function-addresses observe --function-address-path src --artifact-dir ../positive/octet-artifacts -- --lib`; the expected projection is `positive/octet-function-addresses.json`.
- **Kamacite** owns canonical Preserves receipt bytes. Its repository command is `nix run .#kamacite -- validate --root . --manifest fixtures/manifest.json`; `positive/kamacite.json` is the bounded compatibility projection consumed here.
- **Valence** owns Evidence IR linkage. Validate a producer artifact with `valence function-address validate --artifact-root <ROOT> --input <FILE>`; the expected Cairn-facing receipt and graph are `positive/valence.json` and `positive/graph.json`.
- **Mantle** owns the build/release binding. Its first-class command is `mantle --json release function-address-bind <BUNDLE> ... --receipt-out <FILE>`; the expected binding is `positive/mantle.json`.
- **Cairn** owns lifecycle readiness. `positive/cairn-readiness.json` records the current expected receipt from the command exercised by the integration test.

The Octet regeneration attempt on 2026-07-14 was blocked before execution because the Octet Nix package could not fetch its exact `https://github.com/OnixResearch/valence.git` dependency. The committed Octet projection was therefore copied from Octet's existing first-class `target/strict-profile-smoke` output whose source bytes exactly match `source/src/lib.rs`. This is an explicit environment blocker, not permission to weaken or fabricate producer validation.

## Expected identities

`expected.json` binds the source identity, function address, source path/name, Valence and Kamacite receipt links, required Valence role, claim scope, and non-claim. BLAKE3 identities use `b3:` where the producer schema requires an algorithm tag.

## Non-claims

A passing smoke proves only that the committed bounded identities and evidence links satisfy the selected fixture and Cairn readiness contracts. It does not prove Rust semantic correctness, compiler correctness, whole-program safety, verifier soundness, honest producer execution, build correctness, or release eligibility.
