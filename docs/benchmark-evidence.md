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

Every `benchmark-evidence` import or run must name this checked profile with `--profile octet-metadata-benchmark` and its `--config`. A missing or unknown profile is refused before any artifact is written.

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

The copy scanner reads one bounded capability-relative LLVM artifact. It records each `call` or `invoke` of an `llvm.memcpy` intrinsic and its compiler, target, and optimization identities. Intrinsic declarations are not copy sites.

A constant length remains an exact constant only when the length operand is a decimal literal. A variable or unresolved length, such as `%5`, `%_7`, or `%src.1`, remains `unknown`.

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

## Cross-tool experiment

The performance comparator above remains the only same-command comparison rail. A separate opt-in experiment compares predeclared architecture, assurance, and adoption tasks across tools. It never relaxes the equal-argument performance rule.

The checked contract lives in `standards/benchmark/experiment-profiles.ncl` and its generated `standards/benchmark/experiment-profiles.json`. The profile freezes, before candidate output is read:

- the task list and task-to-tool applicability matrix
- the compiling architecture corpus in `standards/benchmark/corpus/`
- the assurance corpus in `standards/benchmark/assurance/`
- expected outcomes, reviewer roles, and rationale for every case
- tool versions, source pins, configuration pins, and setup sources
- adoption targets, sample plan, dependency differences, and unavailable metrics
- the required negative controls and the bounded-coverage non-claim

The corpus covers dependency boundaries, reexports and aliases, macro expansion, conditional compilation, trait and method references, workspace binary targets, and an empty selected scope. Each architecture task has compliant and violating variants; disputed and empty-scope cases stay explicit and are excluded from precision and recall. The assurance corpus covers stale evidence, changed policy, missing execution, and invalid proof/source linkage, plus one valid control.

Evaluate a retained observation file:

```bash
cargo octet benchmark experiment \
  --profile octet-cross-tool-architecture-v1 \
  --input standards/benchmark/benchmark-experiment-observations.json \
  --artifact-dir artifacts/benchmark-experiment
```

Probe available executables, run bounded adoption commands, and retain the results:

```bash
cargo octet benchmark experiment \
  --profile octet-cross-tool-architecture-v1 \
  --execute \
  --workspace-root standards/benchmark/corpus \
  --artifact-dir artifacts/benchmark-experiment
```

Recorded and execute modes share the same result DTOs. `--execute` probes each declared executable, records unavailable tools as `unavailable` or `unsupported`, and runs only declared adoption commands. A missing executable never becomes clean evidence. Finding extraction is not inferred from a probe: execute-mode case observations stay `unparsed` until a reviewed parser is supplied.

The evaluator writes one machine artifact and one source-linked report:

- `standards/benchmark/benchmark-experiment.json`
- `standards/benchmark/benchmark-experiment-report.md`
- retained input: `standards/benchmark/benchmark-experiment-observations.json`

Verify a retained artifact without rerunning any tool:

```bash
cargo octet benchmark experiment \
  --verify \
  --artifact standards/benchmark/benchmark-experiment.json
```

The artifact binds the embedded profile identity, the retained observations, the derived architecture and assurance metrics, the adoption summary, the measurement provenance, and every negative-control record. Verification recomputes the derived comparison from the retained inputs and rejects stale observation, adoption, derived, or control identities. It rejects a noncompiling violation counted as detection, a renamed or stale oracle, a hidden scope difference, a dropped timeout, stale results, a hidden toolchain or cache difference, an unsupported case counted as clean, and a bounded corpus presented as complete Rust coverage.

Precision and recall are computed only on declared comparable subsets. Unsupported, unavailable, timed-out, failed, unparsed, disputed, incomparable, and empty-scope cases are retained but excluded from those denominators. Architecture results, assurance results, and adoption costs remain separate result sets.

The shipped artifact is generated by the ignored `regenerate_experiment_artifacts` test. Its architecture and assurance observations are deterministic recorded fixtures. If clippy is available in the generating environment, the clean-consumer adoption rows are real bounded local measurements; otherwise those rows stay unavailable and the artifact says so. Semgrep, CodeQL, cargo-pup, ArchUnitRust, cargo-vet, and an unusable cargo-deny are recorded as unavailable in the fixture environment. The immutable real-consumer selection is `bounded-exec@44eb74eb`; it is not checked out here, so no real-consumer timing is invented.

The experiment is bounded fixture evidence. It does not prove universal detection, complete Rust coverage, correctness, or release eligibility.

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

`cargo octet artifact verify` remeasures the published artifacts and rejects stale links. It also parses the retained baseline, candidate, allocation, copy-site, oracle, and finding bytes and reproduces the receipt projection from them and the checked profile. A sample, resource observation, signal input, or projection that no longer reproduces its receipt fails with an identity or reproduction diagnostic.

## Public claims

A public metric must bind the projection identity, corpus, environment, sample range, and interpretation boundary.

A claim with a stale projection identity fails validation. A result for one selected corpus cannot represent all Rust projects or defects.

## Evidence boundary

<!-- r[impl octet.benchmark_evidence.boundary] -->
<!-- r[impl octet.benchmark_evidence.public_claims] -->

Benchmark receipts use `transcript_assertion` evidence. They do not prove correctness, universal performance, corpus representativeness, formal proof, whole-program behavior, or release eligibility.
