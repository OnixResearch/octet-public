# Coverage baseline evidence

Octet collects optional cargo-llvm-cov evidence through one checked profile. Starter `cargo octet check` does not run coverage.

## Profile

The `cargo-llvm-cov-octet-lib` profile selects these inputs:

- cargo-llvm-cov 0.8.3 from the pinned nixpkgs revision
- Rust `nightly-2026-03-21` with `llvm-tools-preview`
- the `cargo-octet` library with all features
- line, branch, function, and region counts
- fixed offline arguments and bounded process limits
- integer thresholds in basis points
- equal-denominator and no-regression policies

The checked profile identity is `b3:e221f5d35528e6979bb44ab58cc5e486109799cf99fb80fc4d85ab77a7082634`.

## Import

Prepare a native current report, an accepted baseline report, and one bounded source-manifest file. Octet gives the source manifest a BLAKE3 identity.

```console
cargo octet evidence collect \
  --rail coverage-baseline \
  --profile cargo-llvm-cov-octet-lib \
  --config standards/coverage/profiles.json \
  --input target/coverage-current.json \
  --baseline evidence/coverage-baseline.json \
  --external-report evidence/coverage-source-manifest.json \
  --workspace-root . \
  --artifact-dir artifacts/coverage
```

The first accepted baseline can be a native cargo-llvm-cov report. Octet wraps it with the selected profile and source-manifest identity.

Use `coverage-current-envelope.json` as the accepted baseline for a later comparison. This envelope retains the original raw report text and collection identities.

## Run

Use `--run` instead of `--input` to operate the pinned provider. Do not supply both options.

```console
cargo octet evidence collect \
  --rail coverage-baseline \
  --profile cargo-llvm-cov-octet-lib \
  --config standards/coverage/profiles.json \
  --baseline evidence/coverage-baseline-envelope.json \
  --external-report evidence/coverage-source-manifest.json \
  --workspace-root . \
  --run \
  --artifact-dir artifacts/coverage
```

The run uses Bounded Exec from `OnixResearch/bounded-exec` at revision `44eb74eb639194665ee820562d5b2ce2c3ff6a94`. The owner-authorized reuse covers the bounded process mechanism only.

The shell clears the child environment. The Nix composition supplies the exact Cargo program, provider path, vendor configuration, and runtime library path.

Octet rejects `--command`. The profile owns all cargo-llvm-cov arguments.

## Comparison

Octet reads native covered and total counts. It ignores producer percentages when it makes policy decisions.

The core calculates display percentages with integer basis points and floor rounding. Threshold and regression decisions use exact cross multiplication.

The baseline and current reports must match these identities:

- profile
- provider closure
- Rust toolchain
- LLVM tools component
- source scope
- path policy
- source identity

The selected denominator policy requires equal totals. A denominator change fails and remains visible in the comparison receipt.

## Artifacts

The evidence bundle contains these additional files:

- `coverage-source-manifest.txt`
- `coverage-current-native.json`
- `coverage-current-envelope.json`
- `coverage-baseline-envelope.json`
- `coverage-comparison.json`

The receipt and provenance records bind each file with BLAKE3. Artifact validation remeasures the bundle files and recalculates the comparison projection.

## Non-claims

Coverage counts show instrumented execution only. They do not prove behavioral correctness, path correctness, test completeness, or formal proof.

The source-manifest identity binds supplied bytes only. It does not prove that the manifest includes all source files.

The receipt does not prove whole-program correctness or release eligibility.
