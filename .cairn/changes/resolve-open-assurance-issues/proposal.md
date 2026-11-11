## Why

Nine open issues identified correctness, evidence, trust, and reproducibility
gaps in Octet's assurance surfaces. Selection was being reported as execution,
negative controls could accept an unattributed failure, reviewed requirements
had no production evaluator, freshness queries could not consume a reviewed
inventory, capsule replay compared supplied observations instead of executing,
admission had no external trust anchor, scanners were import-only, the benchmark
compared one workload rather than tools, and the public source cohort was not
qualified against its own inventory and notices.

## What Changes

- Policy reports carry execution outcomes derived from checker results; missing
  local checkers are `unsupported`, external/human checks are `not_run`, and
  strict coverage cannot pass on a required `not_run`.
- Negative controls accept `property_demonstrated` only with typed verifier
  evidence that a passing baseline and a mutant rejection attributed to the
  named obligation; validation recomputes the decision from retained artifacts.
- `cargo octet requirements check|verify` evaluates a Nickel-authored reviewed
  requirement contract, binds source/spec/assumption identities, rejects
  empirical evidence for formal-proof clauses, and emits a recomputable
  decision.
- `cargo octet requirements freshness` consumes the same contract against an
  evidence codebase index and reports current, stale binding, missing, failed,
  unsupported, and incomplete outcomes with bounded related records.
- `roadmap capsule replay --execute` launches a closed admitted profile through
  bounded execution and retains an execution receipt; supplied-observation
  comparison remains available as a distinct evidence mode.
- `admit verify --auth-policy --auth-evidence` verifies detached Ed25519
  statements, producer authorization, and consumer-owned currentness through the
  pinned artifact-auth crates while the unsigned integrity mode remains intact.
- SARIF intake gains opt-in Semgrep/CodeQL producer execution above the existing
  importer, with executable/rule/query-pack/source/argv/outcome bindings and
  CodeQL database sidecars; missing tools are never clean evidence.
- A separate cross-tool experiment contract compares predeclared architecture,
  assurance, and adoption tasks without relaxing the same-command comparator.
- A Nickel release-cohort manifest binds the supported public channel to exact
  source, export, dependency, toolchain, package, notice, capability, and
  acceptance identities; the cohort is truthfully blocked on missing public
  notices and unresolved imported prose.

## Impact

- **Files**: `crates/octet-standards/src/{report,runner,architecture,capability_manifest,proof,resource}.rs`,
  `cargo-octet/src/{requirements,admit,proof_candidates,proof_ref,prove,roadmap,evidence_rails,valence_core_shim}.rs`,
  `cargo-octet/src/external/sarif/`, `cargo-octet/src/benchmark/experiment/`,
  `standards/{requirements,release,sarif,benchmark}/`, `docs/`, `README.md`,
  `CHANGELOG.md`, and this Cairn change.
- **Testing**: `cargo test -p cargo-octet --lib`, `cargo test -p octet-standards --lib`,
  `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`,
  Nickel typecheck/export freshness, and the repository policy/standards gates.
- **Boundary**: every surface keeps its non-claims. Coverage is not detector
  correctness, authentication is not evidence truth, executed replay is not
  whole-program reproduction, scanner completion is not complete vulnerability
  coverage, the experiment is bounded fixture evidence, and the release cohort
  is blocked rather than release-ready.
