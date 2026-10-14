# Profiled Hegel property evidence

Octet provides one optional Hegel evidence profile. The profile runs selected laws for pure, deterministic cores.

This rail is not part of `cargo octet check`. A Hegel result is transcript evidence, not formal proof.

## Run the checked producer

```bash
cargo octet evidence collect \
  --rail hegel-property \
  --profile octet-pure-core-laws-v1 \
  --config standards/hegel/profiles.json \
  --workspace-root . \
  --run \
  --locked \
  --artifact-dir artifacts/hegel-property

cargo octet artifact verify \
  --artifact-dir artifacts/hegel-property
```

Do not supply `--command`. Octet constructs the fixed Cargo arguments from the checked profile.

The run uses a clear environment. Nix supplies the reviewed Cargo executable, vendor configuration, path, and library path.

## Producer output

The producer writes `octet-hegel-producer-summary/v2` to one fixed output path. Octet does not parse stdout or stderr as evidence.

Octet retains these artifacts:

- `hegel-producer-summary.json`;
- `hegel-bounded-log.txt`;
- `hegel-source-manifest.json`;
- `hegel-generator-manifest.json`;
- `hegel-counterexample.json`, when a counterexample exists;
- `hegel-projection.json`.

The receipt remeasures the profile, Cargo lock, package manifest, selected source files, generator catalog, summary, logs, and counterexample.

A stale value causes a deterministic failure. Unsafe paths, symbolic-link substitutions, missing output, and over-limit data also fail.

## Law catalog

`standards/hegel/laws.ncl` owns the selected law catalog. Its generated JSON is reviewable input to the Rust core.

Each law records:

- the pure core and stable law identifier;
- the bounded generator;
- positive and negative properties;
- the generated-input benefit;
- the case budget and replay method;
- a complementary concrete fixture;
- the effect-free boundary.

The first catalog law checks issue-code normalization. Generated duplicates, whitespace, ordering, and Unicode values extend the concrete fixture matrix.

## Counterexamples and replay

A failed law must bind one retained counterexample. The summary identity and counterexample identity must match the exact retained bytes.

Profiles retain only bounded issue codes and bounded values. They do not authorize arbitrary paths or environment values.

Accepted counterexamples stay replayable through the retained artifact or a checked regression fixture.

## Import compatibility

Legacy `hegel-property-summary/v1` files remain valid in import mode. Use the old import command without the hardened profile:

```bash
cargo octet evidence collect \
  --rail hegel-property \
  --input target/hegel-property-v1.json \
  --artifact-dir artifacts/hegel-property-import
```

Octet preserves the legacy evidence boundary. It does not invent profile, package, source, log, or counterexample measurements.

A hardened request can also be imported with the checked profile. Its measured fields must already be complete and consistent.

## Claim boundary

Hegel checks bounded generated inputs only. It does not prove correctness, semantic equivalence, verifier soundness, requirement satisfaction, release eligibility, or whole-program safety.

Hegel does not replace concrete tests, negative fixtures, cargo-fuzz, Kani, Miri, Loom, or formal verification.
