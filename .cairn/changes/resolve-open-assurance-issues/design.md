## Context

The reviewed issues at development revision `2bdd5819` pointed at real source
surfaces: selection-based policy metadata, unattributed negative-check status,
a test-only claim catalog, an API-only multihop traversal, supplied-observation
capsule replay, integrity-only admission, import-only SARIF, a same-command
benchmark, and an unqualified public export. The implementation reuses the
existing bounded executor, validation helpers, evidence index, proof-card DTOs,
Nickel export boundary, and Cairn policy rather than introducing parallel
mechanisms.

## Decisions

### Decision: Execution outcomes are derived from checkers, not selection

**Choice:** Add typed policy execution records and a checker-coverage plan in
`octet-standards`. Local checkers map to the policies they actually evaluate;
unimplemented local policies are `unsupported`; external and human policies are
`not_run` with explicit ownership. Strict mode additionally rejects required
external/human `not_run` outcomes.

**Rationale:** A registry entry is an obligation, not evidence. This preserves
the existing `evaluated_policy_ids` migration surface while giving strict
consumers a truthful verdict.

### Decision: Positive negative controls require attributable evidence

**Choice:** Add a typed `octet-proof-negative-check-evidence/v1` document,
measure source/deck bytes, bind both cards to one specification and obligation,
and recompute the verdict during proof-ref validation. An annotated failed
status without evidence is inconclusive.

**Rationale:** A process failure or unrelated proof failure is not
discrimination. Only an accepted baseline plus a mutant rejection attributed to
the expected obligation supports the bounded mutation claim.

### Decision: Reviewed requirements use one executable contract

**Choice:** A Nickel-authored `octet-requirement-contract/v1` names clause
dispositions, scopes, allowed evidence, assumptions, exclusions, and freshness
bindings. Rust evaluates it into a versioned decision with contract, evidence,
and source identities. `requirements verify` recomputes the decision from those
inputs.

**Rationale:** Completeness against reviewed intent needs an explicit inventory
that can fail closed when evidence is missing, stale, or for another clause.
The contract also becomes the expected-obligation inventory for freshness, so
there is no second obligation schema.

### Decision: Freshness uses typed bindings and bounded traversal

**Choice:** Match evidence records on declared source, policy, tool, artifact,
scope, and tier bindings. Report `current`, `stale_binding`, `missing`,
`failed`, `unsupported`, or `incomplete`; use the existing typed BFS for related
records and keep shared labels/content identity as relatedness only.

**Rationale:** Shared labels do not establish causal support. Explicit bindings
make old/new identity drift visible without executing providers.

### Decision: Executed replay uses closed profiles

**Choice:** Keep the `--observed` comparison and add `--execute` profiles with
fixed argv, allowlisted environment, bounded output, isolated output
directories, expected-output hashes, and a versioned execution receipt. Capsule
command text never selects an executable.

**Rationale:** Reproduction must come from an independently launched process,
not from a supplied observation file, while imported capsules remain untrusted
data.

### Decision: Authentication is an optional external trust layer

**Choice:** Load consumer-owned policy and detached evidence outside the
candidate bundle, construct the canonical statement scope from the bundle and
provenance identities, and call the pinned
`artifact_auth_ed25519::verify_and_evaluate`. Report integrity, signature,
authorization, currentness, checker, and overall outcomes separately.

**Rationale:** The existing mode proves local integrity only. External producer
admission needs a trust decision anchored outside the candidate bundle, without
turning the provider into a key-discovery or revocation service.

### Decision: Scanner execution and competitive measurement stay opt-in

**Choice:** Add fixed-argv Semgrep/CodeQL producer profiles and receipts above
the importer, and a separate cross-tool experiment contract above the
same-command comparator. Missing tools are unsupported, not clean; experiment
metrics use only declared comparable subsets.

**Rationale:** Completion and bounded scope are not complete vulnerability
coverage, and performance comparison of one workload is not a cross-tool
detection or adoption benchmark.

### Decision: Public release qualification is a blocking manifest

**Choice:** Author a Nickel release-cohort manifest with the exact reviewed
public revision, export mapping, inventory, dependencies, packages, notices, and
receipts. The current cohort is `blocked` because the public tree omits the
development notices and carries unresolved imported prose; the manifest names
each clearing action.

**Rationale:** Intentional public lag is acceptable only when explicit. A
transformed public tree cannot inherit development receipts, and a legal
conclusion is not asserted.

## Risks / Trade-offs

- Strict policy coverage will fail on registries with required external/human
  policies; this is intentional and opt-in through `--strict-policy-coverage`.
- Negative-control v1 records with `property_demonstrated` become inconclusive;
  re-recording with v2 evidence is required.
- Freshness depends on the index carrying the declared bindings; partial indexes
  report `incomplete` rather than guessing.
- Real Semgrep `1.180.0` and CodeQL `2.27.2` clean and violating reports are
  retained under `standards/sarif/fixtures/` and imported by the checked intake
  profiles. Semgrep 1.180.0's `Semgrep OSS` driver spelling is accepted for the
  checked `Semgrep` profile. The invalid `codeql/rust-queries@0.2.25` pin is
  corrected to `0.1.46`.
- The benchmark retains fixture-only observations for unavailable tools and one
  real local Clippy adoption measurement; no unavailable timing is invented.
- The public cohort remains blocked until the public export carries the required
  notices and resolves or excludes the imported prose.
