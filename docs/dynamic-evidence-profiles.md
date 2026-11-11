# Dynamic evidence profiles

Tiger Style lints are the default guardrail, but some bug classes need runtime or profiling evidence. Use these profiles when a change touches the matching risk area. Do not add heavyweight tools to the default consumer path unless a separate OpenSpec promotes one profile into a required rail.

## Profile selection

| Profile | Run when | Acceptance signal | Evidence to save |
| --- | --- | --- | --- |
| Memory and allocation | Unsafe code, FFI ownership, allocator-sensitive code, or suspected leaks | The selected tool finishes without invalid access/leak findings, or the transcript names accepted third-party suppressions | Tool command, target binary/test, version, suppression file if any, and summary finding count |
| Heap growth | Queues, caches, batching, or retention behavior changes | Peak heap or retained bytes match the review budget, or the delta is explained against the napkin sketch | Heap profile summary plus the expected byte/item bound |
| Cache or hot path | Data-plane loops, parsers, encoders, schedulers, or code changed for speed | Profile identifies the claimed hot path and does not show unexpected branch/cache/allocation dominance | Workload description, profile command, top functions/lines, and comparison to the performance sketch |
| Race and concurrency | Locking, atomics, channels, task topology, or replay/idempotence changes | Loom/Miri/sanitizer/model run passes for the scoped fixture, or remaining risk is escalated to human review | Scenario name, scheduler/model bounds, failing seed if any, and pass/fail summary |
| Unsafe or FFI boundary | New `unsafe`, raw pointers, C ABI, lifetime erasure, or external ownership changes | Miri/sanitizer/Valgrind-style run covers the boundary fixture and the [unsafe/trusted-boundary record](rust-idiom-guidance.md#unsafe-trusted-boundary-records) is complete, or the record explains scoped non-applicability | Boundary fixture, exact command, `UNSAFE-TRUST-BOUNDARY:` record, unsafe invariant checklist, and diagnostic transcript |
| Release readiness | Before tagging or before enabling a hard gate for a high-risk crate | Static Octet check passes and the one selected dynamic profile for the highest-risk path has saved evidence | Release candidate revision, static check transcript, selected dynamic profile transcript |

## Run timing

- **Always/local default:** static Octet and standards checks only; starter `cargo octet check` does not require heavyweight unsafe/FFI dynamic tools unless a separate accepted requirement selects that rail.
- **After a big change:** select the profile for the changed risk surface.
- **When a bug is suspected:** select the profile that can reproduce or falsify the suspected class.
- **Before release:** run static checks plus one selected high-risk dynamic profile.
- **Focused CI or nightly:** use longer profiles only after the owning OpenSpec says which tool, workload, and acceptance signal are required.

## Evidence rules

Every saved dynamic evidence artifact must include:

1. The exact command and working directory.
2. Tool version or Nix/app derivation when available.
3. The target workload, fixture, or test name.
4. Expected acceptance signal before the run.
5. Observed result and whether suppressions were used.
6. Link back to the OpenSpec scenario or review checklist row.
7. For unsafe/FFI profiles, link the `UNSAFE-TRUST-BOUNDARY:` record and state the selected command, fixture, transcript or artifact, and acceptance signal.

## Miri and Loom profiles

`cargo octet evidence collect --rail dynamic-evidence` accepts the built-in `miri-unsafe-boundary` (alias `miri`) and `loom-concurrency` (alias `loom`) profiles, in both `--run` and `--input` modes. The allowed command (`cargo miri test` or `cargo test`) and the expected acceptance signal (`passed`) come from the built-in profile. The Loom profile runs `cargo test` with `RUSTFLAGS=--cfg loom` in the child environment, because Cargo takes `--cfg loom` from its rustflags environment, not from its own argv. Cargo reads `CARGO_ENCODED_RUSTFLAGS` before `RUSTFLAGS`, so the profile also sets `CARGO_ENCODED_RUSTFLAGS` to `--cfg loom` in its `0x1f`-separated form. Both values replace any inherited `RUSTFLAGS` or `CARGO_ENCODED_RUSTFLAGS`, so an inherited encoded value cannot drop the cfg and let Loom models go uncompiled. Octet ignores `known_profiles`, `allowed_commands`, and `expected_signal` fields in imported JSON.

With `--run`, `--profile` and `--command` select the profile and must name its allowed command. Octet does not parse the test output: exit status 0 is the `passed` acceptance signal, and any other status records one `dynamic-finding` error. The receipt's `bounded_log_identity` binds the combined stdout and stderr transcript.

With `--input`, the imported JSON must carry its own `profile` and `command` strings and a non-negative integer `finding_count`. `--profile` and `--command` are optional; when given, they must match the input's profile (aliases count as the same profile) and command rather than replace them. A missing or unknown profile, a missing or different command, a missing or mistyped `finding_count`, or any observed signal other than `passed` fails the receipt.

## Cargo-fuzz libFuzzer profile

The `cargo-fuzz-libfuzzer` profile is optional. Starter `cargo octet check` does not resolve or run cargo-fuzz.

Export the reviewed Nickel registry before an import:

```console
nickel export --format json templates/dynamic-evidence-profiles.ncl > dynamic-evidence-profiles.json
cargo octet evidence collect \
  --rail dynamic-evidence \
  --profile cargo-fuzz-libfuzzer \
  --config dynamic-evidence-profiles.json \
  --input cargo-fuzz-run.json \
  --artifact-dir artifacts/cargo-fuzz
```

The reviewed run command is fixed:

```console
cargo fuzz run octet-profile-intake -- -max_total_time=60 -runs=100000 -seed=1
```

Octet rejects `--command` for this profile. It binds the exact target, libFuzzer engine, nightly toolchain, observed compiler identity, address sanitizer, budgets, corpus roots, dictionary identity, crash input, minimized reproducer, replay command, and bounded log identity.

With `--run`, Octet executes that command through Bounded Exec in a fresh process group. The child starts from a cleared environment and receives only these variables when they are set: `PATH`, `HOME`, `CARGO_HOME`, `RUSTUP_HOME`, `LD_LIBRARY_PATH`, `TMPDIR`, `CC`, `CXX`, `SSL_CERT_FILE`, and `NIX_SSL_CERT_FILE`. The ambient `RUSTUP_TOOLCHAIN` and `RUSTC` are withheld. Octet resolves `cargo` and `rustc` from the absolute entries of that `PATH`.

Before the run, Octet runs that `rustc -vV` in the same cleared environment and working directory. It refuses to run unless the reported `commit-hash` equals the profile's `rustc_commit_hash`, the `nightly-2026-03-21` rustc commit. The receipt's `compiler_identity` is the BLAKE3 identity of that whole `rustc -vV` report.

Octet then gives the cargo-fuzz child `RUSTC` set to the absolute path of the probed compiler, plus empty `RUSTC_WRAPPER` and `RUSTC_WORKSPACE_WRAPPER`. Cargo ranks these variables above `build.rustc`, `build.rustc-wrapper`, and `build.rustc-workspace-wrapper` in any `.cargo/config.toml`, and an empty wrapper variable disables a configured wrapper. A workspace or home Cargo config, a relative `PATH` entry, or a toolchain override cannot swap the compiler after Octet observes it.

The run deadline is the 60-second libFuzzer budget plus a 14-minute build allowance. Stdout and stderr each retain at most half of the 1 MiB receipt log limit. A timeout or an output overflow stops the whole process group and fails the run without a receipt.

The artifact directory `fuzz/artifacts/octet-profile-intake` must be empty or absent before a run. Octet rejects the run otherwise, because libFuzzer reuses content-hash file names and earlier `crash-*`, `leak-*`, `oom-*`, `timeout-*`, or `minimized-*` files would decide the outcome. Move earlier artifacts aside before you rerun.

Corpus rows use safe relative paths, lengths, and BLAKE3 identities. The canonical root sorts rows and uses the `octet-cargo-fuzz-corpus-v1` framing domain. Hash-only rows reduce disclosure, but hashes can still reveal known sensitive inputs. Do not put credentials, production records, or personal data in a corpus.

A clean receipt means only that one target found no crash under the selected seed and budget. libFuzzer `crash-*`, `leak-*`, `oom-*`, and `timeout-*` artifacts are all findings: a failed run that leaves one gets a `crash` outcome with that artifact bound as the crash input. A crash receipt remains failed evidence. It must retain both the original crash input and minimized reproducer identity. Reviewers must reproduce the crash with the recorded replay argv before they use it in a decision.

The rail rejects unsupported profiles, arbitrary command text, unsafe paths, duplicate artifacts, stale corpus roots, missing crash roles, excessive counts or bytes, malformed receipts, and non-transcript evidence tiers.

## Non-goals

- Do not require cargo-fuzz, Valgrind, Miri, sanitizers, heap profilers, cache profilers, or Loom for ordinary consumer lint runs.
- Do not treat a profiler or fuzz result as design proof unless it is compared to its declared workload and budget.
- Do not accept "ran tool" as evidence without the acceptance signal and finding summary.
- Do not claim exhaustive coverage, memory safety, panic freedom, whole-program correctness, or release eligibility from cargo-fuzz evidence.
