# Benchmark evidence

Octet provides an optional `benchmark-evidence` rail. The rail records performance, resource, and diagnostic-signal evidence.

The rail is not part of starter `cargo octet check`. It does not grant arbitrary command, environment, workspace, or network authority.

## Checked profile

Edit `standards/benchmark/profiles.ncl` to change the checked policy. Generate `standards/benchmark/profiles.json` from that Nickel source.

The checked profile binds these facts:

- provider and Rust toolchain identities
- workload and source-state identities
- fixed command IDs and argument arrays
- cold, warm, no-change, and incremental modes
- environment identities
- sample and artifact limits
- synthetic and immutable cross-project corpus boundaries
- practical and uncertainty thresholds
- required metrics and non-claims

The runner maps `cargo-metadata-no-deps` to one fixed argument array. It rejects free-form `--command` text.

Workspace membership, a target name, or an operating-system label grants no extra authority.

## Import a complete suite

Use one complete typed input for performance, allocation, copy-site, and signal evidence:

```bash
cargo octet evidence collect \
  --rail benchmark-evidence \
  --profile octet-metadata-benchmark \
  --config standards/benchmark/profiles.json \
  --input evidence/benchmark-input.json \
  --workspace-root . \
  --artifact-dir artifacts/benchmark-evidence
```

The input uses schema `octet-benchmark-input/v1`. It contains these parts:

- baseline and candidate run sets
- instrumented allocation observations
- LLVM copy-site observations
- an independently adjudicated signal oracle
- observed findings
- optional public claims
- one bounded process-log identity

Import mode rejects separate baseline and external-report arguments. This rule prevents ambiguous parent selection.

## Run the performance workload

Run mode requires one accepted baseline run set:

```bash
OCTET_BENCHMARK_CARGO=/nix/store/.../bin/cargo \
OCTET_BENCHMARK_PATH=/nix/store/.../bin \
OCTET_BENCHMARK_LD_LIBRARY_PATH=/nix/store/.../lib \
OCTET_BENCHMARK_KERNEL_IDENTITY=reviewed-kernel-identity \
OCTET_BENCHMARK_CPU_IDENTITY=reviewed-cpu-identity \
OCTET_BENCHMARK_GOVERNOR=performance \
OCTET_BENCHMARK_LOAD_CLASS=isolated \
cargo octet evidence collect \
  --rail benchmark-evidence \
  --profile octet-metadata-benchmark \
  --config standards/benchmark/profiles.json \
  --baseline evidence/benchmark-baseline-run.json \
  --run \
  --workspace-root . \
  --artifact-dir artifacts/benchmark-evidence
```

Run mode uses `bounded-exec` with a clear environment and process-group teardown. It records every selected sample and its outcome.

The shell records explicit kernel, CPU, governor, load, toolchain, Nix closure, host, and isolation facts. Missing facts stop production execution.

A failed, timed-out, limited, or cancelled sample remains visible. It cannot silently enter a passing summary.

The current shell records wall duration and output bytes. CPU duration and peak memory remain unsupported instead of receiving estimated values.

Run mode reads the profile-owned allocation and LLVM paths after command execution. Supply a complete typed import when signal-oracle inputs are required.

## Cache and source modes

Cold mode resets its dedicated cache before each measured sample.

Warm mode performs one unmeasured preparation run. It then records the declared samples.

No-change mode records one exact source-state identity without source mutation.

Incremental mode measures an already prepared source state. The shell does not edit source to create that state.

Results from different modes are incomparable.

## Performance comparison

The pure core computes a median from retained successful samples. It does not delete outliers.

The result is one of:

- `improved`
- `stable`
- `regressed`
- `inconclusive`
- `incomparable`

A comparison becomes incomparable when a required workload, command, mode, source, or environment identity differs.

A comparison becomes inconclusive when samples fail, sample counts are too small, or observed spread exceeds the uncertainty policy.

A small numerical difference remains stable until it crosses the practical threshold.

## Allocation observations

Allocation evidence applies only between declared interval markers. Each observation binds the allocator instrumentation, workload, interval, and scope.

A zero-allocation claim fails when the selected interval reports an allocation event or allocated byte.

Allocation evidence does not describe code outside the selected interval or instrumented build.

## LLVM copy-site census

The copy scanner reads one bounded capability-relative LLVM artifact. It records each `llvm.memcpy` site and its compiler, target, and optimization identities.

A constant length remains an exact constant. A variable or unresolved length remains `unknown`.

Static copy sites are not runtime copy counts. Octet does not convert unknown lengths to zero.

## Signal oracle

The profile binds one synthetic corpus and one immutable Bounded Exec selection. Neither corpus represents all Rust projects or defects.

The signal oracle is independent from Octet output. Each record binds these facts:

- case and source identities
- expected lint and location class
- expected outcome
- reviewer role and rationale

The pure alignment core reports true positives, false positives, false negatives, expected suppressions, unexpected suppressions, and inconclusive cases.

Duplicate, contradictory, stale, or incomplete oracle records fail before metrics are calculated.

## Artifacts and replay

A successful collection writes these additional artifacts:

- `benchmark-baseline-samples.json`
- `benchmark-candidate-samples.json`
- `benchmark-allocations.json`
- `benchmark-copy-sites.json`
- `benchmark-signal-oracle.json`
- `benchmark-signal-findings.json`
- `benchmark-projection.json`

The receipt binds every artifact with BLAKE3. Provenance records link the projection to all six raw parents.

`cargo octet artifact verify` remeasures the published artifacts and rejects stale links.

## Public claims

A public metric must bind the projection identity, corpus, environment, sample range, and interpretation boundary.

A claim with a stale projection identity fails validation. A result for one selected corpus cannot represent all Rust projects or defects.

## Evidence boundary

<!-- r[impl octet.benchmark_evidence.boundary] -->
<!-- r[impl octet.benchmark_evidence.public_claims] -->

Benchmark receipts use `transcript_assertion` evidence. They do not prove correctness, universal performance, corpus representativeness, formal proof, whole-program behavior, or release eligibility.
