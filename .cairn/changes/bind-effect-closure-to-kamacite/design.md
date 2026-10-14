## Context

Octet computes a conservative fixed point over selected Rust source and reviewed provider summaries. Its vocabulary names implementation effects such as clocks, randomness, files, networks, spawning, locks, FFI, and unknown behavior.

Kamacite effect rows name semantic operations with nominal family, schemas, continuation policy, replay class, dependencies, and behavior identity. These models answer different questions.

The integration must compare them without treating source tokens as canonical semantics or treating declared semantics as observed Rust behavior.

## Decisions

### Decision: Add an opt-in mapping profile

**Choice:** Add a typed Nickel profile that binds one Octet closure profile, one canonical Kamacite effect summary, and exact semantic-emission rules.

Each emission rule binds one function address or call-site identity, provider identity, exact Kamacite operation identity, evidence reference, and non-claims.

Ambient effect dispositions remain a separate closed policy section.

**Rationale:** Reviewers can inspect every explicit emission without converting ambient host access into application semantics.

### Decision: Keep ambient effects and semantic emissions separate

**Choice:** Ambient effect closure retains the existing strict-profile result. The mapping profile cannot discharge or rename a denied ambient effect.

A closed ambient disposition section permits only `profile-local` or `denied`. V1 permits only `alloc` as profile-local.

A separate `semantic-emission` rule names one exact operation identity at one exact reviewed emission site. Unknown, missing, duplicate, broad, or ambiguous rules fail closed.

**Rationale:** A clock, network, file, random, process, lock, FFI, panic, unsafe, or unknown source effect cannot masquerade as effect-request data.

### Decision: Consume a bound compatibility projection

**Choice:** Octet consumes a checked compatibility projection that names the canonical Kamacite Preserves profile and effect-summary identities.

Octet does not reimplement Kamacite canonicalization. A live Kamacite dependency is not part of the initial profile.

**Rationale:** Exact frozen projections preserve independent release cycles and canonical ownership.

### Decision: Compare evidence without merging authority

**Choice:** The pure comparison core produces these classes:

- `emitted-and-declared`;
- `profile-local`;
- `declared-not-emitted`;
- `emitted-not-declared`;
- `stale-identity`;
- `ambiguous-emission`;
- `unknown`.

Only `emitted-and-declared`, `profile-local`, and `declared-not-emitted` can appear in an admitted comparison.

`declared-not-emitted` remains visible because Octet cannot prove runtime reachability or complete semantic coverage.

**Rationale:** The result reports agreement and gaps without rewriting either parent artifact.

### Decision: Extend closure admission only when requested

**Choice:** Add an optional `--kamacite-effect-profile PATH` input to `cargo octet admit --profile deterministic-core --closure`.

The input takes effect only after direct checks and effect closure pass. Existing commands and receipts remain unchanged when the input is absent.

**Rationale:** The integration must not widen the default Octet trust base or change starter checks.

### Decision: Export a handoff after verification

**Choice:** A successful comparison emits a BLAKE3-bound handoff with parent closure, mapping profile, Kamacite profile, effect summary, operation, function-address, provider, comparison, and non-claim fields.

JSON is a compatibility projection. Kamacite remains the canonical portable receipt owner.

**Rationale:** Downstream systems can canonicalize the result without reparsing Rust or inferring operation identity from names.

### Decision: Preserve functional core and imperative shell

**Choice:** Pure code validates loaded profiles, checks identity domains, compares sets, sorts rows, and returns deterministic diagnostics.

The runner shell reads files, measures bytes, loads Cargo evidence, and writes artifacts.

**Rationale:** The comparison stays testable without filesystem, process, environment, clock, network, or ambient mutable state.

### Decision: Keep source-analysis claims bounded

**Choice:** Passing comparison proves only that selected Octet source-token evidence does not contradict the supplied mapping and declared row.

It does not prove effect completeness, runtime order, semantic determinism, handler correctness, or property satisfaction.

**Rationale:** Mapping two bounded artifacts cannot strengthen either producer beyond its original claim.

## Test Design

Positive fixtures cover exact semantic emissions, profile-local allocation, declared but unemitted operations, canonical input order, and unchanged default commands.

Negative fixtures cover ambient clock masquerade, malformed projections, stale identities, duplicate rules, broad scopes, denied effects, unknown effects, undeclared emissions, role promotion, and tampered parents.

Property tests cover order independence, duplicate rejection, identity sensitivity, and comparison determinism.

## Risks / Trade-offs

- Mapping profiles can become manual call graphs. Rules remain bounded to existing Octet source or provider identities.
- Profile-local effects can hide important implementation facts. V1 permits only allocation and keeps it visible.
- Declared but unemitted operations can look covered. Reports keep that class separate from emitted operations.
- Frozen projections can lag Kamacite. Stale identities fail closed and require explicit cohort updates.

## Claim Boundary

A passing comparison records bounded agreement between exact supplied artifacts. It does not prove that either artifact completely models program behavior.
