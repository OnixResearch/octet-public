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

Octet rejects `--command` for this profile. It binds the exact target, libFuzzer engine, nightly toolchain, address sanitizer, budgets, corpus roots, dictionary identity, crash input, minimized reproducer, replay command, and bounded log identity.

Corpus rows use safe relative paths, lengths, and BLAKE3 identities. The canonical root sorts rows and uses the `octet-cargo-fuzz-corpus-v1` framing domain. Hash-only rows reduce disclosure, but hashes can still reveal known sensitive inputs. Do not put credentials, production records, or personal data in a corpus.

A clean receipt means only that one target found no crash under the selected seed and budget. A crash receipt remains failed evidence. It must retain both the original crash input and minimized reproducer identity. Reviewers must reproduce the crash with the recorded replay argv before they use it in a decision.

The rail rejects unsupported profiles, arbitrary command text, unsafe paths, duplicate artifacts, stale corpus roots, missing crash roles, excessive counts or bytes, malformed receipts, and non-transcript evidence tiers.

## Non-goals

- Do not require cargo-fuzz, Valgrind, Miri, sanitizers, heap profilers, cache profilers, or Loom for ordinary consumer lint runs.
- Do not treat a profiler or fuzz result as design proof unless it is compared to its declared workload and budget.
- Do not accept "ran tool" as evidence without the acceptance signal and finding summary.
- Do not claim exhaustive coverage, memory safety, panic freedom, whole-program correctness, or release eligibility from cargo-fuzz evidence.
