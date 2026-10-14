## Context

Octet already owns all component mechanisms needed for a first strict-Rust profile: per-lint severity overrides, purity/core-scope lints, a shared lint metadata catalog, function-address modes, BLAKE3 run metadata, checked-scope provenance, and artifact verification. The missing seam is a single reviewed profile selection shared by the runner and lint driver.

The implementation must avoid two false shortcuts. Post-processing emitted warnings cannot enable allow-by-default lints that never ran, and a runner-only profile label would not prove the lint driver used the selected policy. The runner and driver therefore need one generated catalog dependency and one explicit, runner-owned profile handoff.

## Decisions

### 1. Nickel owns profile policy; a shared Rust crate owns runtime validation

**Choice:** Add `standards/strict-profiles/profiles.ncl` and a typed contract, export it to checked-in deterministic JSON, and embed that JSON in `octet-lint-metadata`. The shared crate will deserialize and validate the catalog for both `cargo-octet` and the lint driver.

**Rationale:** Profiles are human-authored, composable policy and therefore belong in Nickel. Runtime Rust must not execute Nickel. Sharing the validated DTO and lookup logic prevents the runner and driver from drifting onto different profile definitions.

### 2. Ship three bounded profiles

**Choice:** The first catalog contains:

- `robust`: deny explicit failure-path, unchecked-conversion/arithmetic, ignored-result, unsafe-public-API, TODO, and suppression-governance lints; function-address floor `observe` over the workspace.
- `bounded-systems`: extend `robust` with bounded loop/recursion/collection/channel, allocation-in-loop, task-lifecycle, lock-order, and platform-width checks; function-address floor `warn` over the workspace.
- `deterministic-core`: extend `robust` with core-effect, ambient-input, state, logging, spawning, deterministic-output, allocation, and resource-boundary checks; require configured core scopes and function-address floor `deny` over those scopes.

**Rationale:** These profiles are useful points in a strictness lattice without presenting one misleading universal “safe” level. A proof-required profile is deferred until check/prove orchestration can bind proof obligations without treating every lint as formal evidence.

### 3. Runner-owned environment handoff is explicit and scrubbed by default

**Choice:** `cargo-octet` resolves `--profile` before Cargo, then sets one internal profile environment variable on the child Cargo process. When no profile is selected, the runner removes that variable from the child environment.

**Rationale:** The Dylint driver runs inside the Cargo/rustc process and needs the selection before lint registration. Explicitly removing ambient values on the default path prevents a user's shell environment from silently changing check semantics. Status metadata and the command line record the selected profile.

### 4. Profile lint floors strengthen workspace configuration

**Choice:** The lint-driver configuration core validates normal consumer config, then applies the selected profile by removing required lints from `disabled_lints` and setting their effective rustc level to `deny`. Profile-required suppression-governance lints are included in every profile. Other thresholds, allowlists, levels, and disabled lints remain consumer-owned.

**Rationale:** A selected profile must be authoritative for its minimum set. Failing every time a starter config still lists a newly required lint would make migration unnecessarily brittle; deterministic strengthening provides the requested semantics while preserving unrelated consumer choices. Explicit source-level suppressions remain visible Rust and are checked by existing justification lints.

### 5. Core-scope preparation is a pure core with a thin file-reading shell

**Choice:** A new `cargo-octet` strict-profile module will separate:

- pure catalog selection, TOML table interpretation, scope normalization/de-duplication, and function-address floor calculation;
- shell-owned reading of workspace `dylint.toml` and child-process environment wiring.

`deterministic-core` fails before Cargo when no canonical or compatibility Octet config declares a non-empty core scope.

**Rationale:** This provides deterministic unit tests without launching Cargo and ensures the profile cannot appear to check a core while core-only lints are effectively unscoped.

### 6. Function-address policy is a minimum, not an override

**Choice:** Derive an ordering over existing `off`, `observe`, `warn`, and `deny` modes. The effective mode is the stronger of operator selection and profile floor. Explicit paths and baseline always remain operator-owned; only an absent path set is defaulted from the profile's declared workspace/core scope.

**Rationale:** Profiles must not weaken explicit enforcement. `deterministic-core` intentionally requires a reviewed deny-mode baseline; operators bootstrap that baseline with an unprofiled observe run before requesting deterministic-core admission.

### 7. Reuse the existing artifact bundle as admission evidence

**Choice:** Extend run metadata construction to accept the selected profile and include the canonical selected-profile JSON identity in BLAKE3 configuration/profile hashes. Render the profile name in the summary and use the existing status/provenance/artifact-verifier path rather than inventing a certificate schema.

**Rationale:** The current bundle already binds command, status, findings, profile name/hash, configuration hash, and checked scope. Reusing it avoids a redundant receipt while making `profile:<ID>` reviewable. Passing means this bounded check passed; it is not a transferable proof or release certificate.

### 8. Default behavior remains byte-shape compatible where practical

**Choice:** When no strict profile is selected, retain `workspace-metadata`, do not apply profile lint floors, do not raise function-address mode, and preserve the existing configuration hash payload.

**Rationale:** Strict profiles are opt-in. Avoiding an unconditional `null` profile field in hash material prevents needless baseline churn for existing consumers.

## Data flow

```text
standards/strict-profiles/profiles.ncl
  -> deterministic generated JSON
  -> octet-lint-metadata catalog validation
  -> cargo-octet --profile selection
     -> workspace/core-scope preparation
     -> effective function-address check
     -> child Cargo profile handoff
  -> lint driver profile floor application
  -> findings + function-address evidence
  -> BLAKE3-bound status/summary/provenance bundle
  -> cargo octet artifact verify
```

## Risks / Trade-offs

- Source-level `allow` remains possible because Octet uses Rust lint semantics. Requiring `unjustified_allow` and related governance lints makes that escape hatch reviewable but does not eliminate it.
- Current lints observe direct source/HIR shapes, not complete monomorphized call closure. Documentation and profile non-claims must keep that boundary prominent.
- `deterministic-core` deny-mode function addressing requires a reviewed baseline and may expose unsupported macro contexts. That friction is intentional for the strongest initial profile; bootstrap guidance must be explicit.
- The shared catalog adds generated JSON to a small metadata crate. Freshness checks and positive/negative catalog tests must prevent stale or malformed embedded policy.
- Existing compatibility config namespaces remain readable for core-scope discovery, but new documentation uses canonical Octet names only.
