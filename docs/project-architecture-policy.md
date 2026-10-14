# Project architecture policy

A project can select explicit FCIS and Hexagonal architecture policy.

Activation is optional. After activation, missing or stale policy inputs are errors.

## Activate the policy

Copy these files to the workspace root:

- `templates/octet-architecture.ncl`
- `templates/octet-architecture.contracts.ncl`
- `templates/octet-architecture-export-spec.ncl`

Rename them only if all companion file names and export paths stay consistent.

Add the source pointer to the Cargo workspace metadata:

```toml
[workspace.metadata.octet]
default_scope = ["--workspace"]
cargo_check_args = ["--lib"]
architecture_policy = "octet-architecture.ncl"
```

The pointer must be workspace-relative. It must name a `.ncl` file inside the workspace.

Octet derives these companion paths from the pointer:

- policy export: `octet-architecture.json`
- policy contract: `octet-architecture.contracts.ncl`
- freshness manifest: `octet-architecture.manifest.json`

Generate the runtime export and BLAKE3 freshness manifest:

```bash
nickel export --format json octet-architecture.ncl > octet-architecture.json
nickel export --format json octet-architecture-export-spec.ncl > /tmp/octet-architecture-export-spec.json
octet-standards nickel-export-manifest \
  --spec /tmp/octet-architecture-export-spec.json \
  --output octet-architecture.json \
  --manifest-out octet-architecture.manifest.json
```

Commit the Nickel source, contract, JSON export, and manifest together.

Octet runtime code does not invoke Nickel. It reads the generated JSON and verifies exact source, contract, output, evaluator, and BLAKE3 identities.

## No-policy behavior

If `architecture_policy` is absent, Octet does not load project architecture policy.

It also does not emit:

- project FCIS findings;
- project Hexagonal findings;
- project architecture receipts;
- architecture-clean claims.

Existing lint checks and strict Rust profiles still run. Selecting `deterministic-core` does not activate project architecture policy.

An inactive run does not create compiler, project-architecture, or effect-closure artifacts. Its existing output serializers stay unchanged.

## Compiler Architecture IR

An active policy starts compiler architecture collection for the exact production library units.

The policy must define positive limits for units, shards, facts, unknowns, spans, artifacts, and selected MIR traversal.

The Cargo invocation must cover all declared production packages. Its target, features, and default-feature selection must match one declared target.

Octet writes one content-bound shard for each selected compilation unit. The shard records resolved items, types, traits, implementations, signatures, calls, constructors, effects, expansions, and active configuration.

The collector records unresolved dispatch, FFI, missing bodies, unsupported expansions, and ambiguous selected provenance as typed unknown facts. External generated spans use redacted BLAKE3 provenance, not workspace paths. The collector does not infer a clean result.

When `--artifact-dir` is set, a complete run adds these files:

- `compiler-architecture-ir.json`;
- `compiler-architecture-coverage.json`;
- `compiler-architecture-issues.json`;
- `compiler-architecture-receipt.json`.

Missing, duplicate, stale, malformed, mismatched, or over-limit shards make collection incomplete. Octet saves the incomplete coverage and receipt before it returns an error.

The Architecture IR proves bounded compiler observations for the selected policy, Cargo graph, invocation, target, features, toolchain, collector, and sources. It does not prove runtime behavior or whole-program correctness.

## Modes

Start with `inventory`. Promote only after scope and role coverage are complete.

| Mode | Result |
|---|---|
| `inventory` | Records declared roles, targets, rules, ports, capabilities, waivers, and later compiler facts. Findings do not block promotion. |
| `advisory` | Reports architecture findings. Findings do not block promotion. |
| `gate` | Blocks declared architecture violations and required unknown facts. |
| `admission` | Requires the architecture gate and transitive effect-closure evidence. |

Every active mode rejects malformed policy, stale exports, escaping paths, ambiguous roles, and missing required inputs.

## Architecture gate output

After compiler collection succeeds, `cargo octet check` evaluates the activated policy against the exact resolved Cargo graph and Architecture IR.

The evaluator checks:

- complete production package and source-role coverage;
- permitted production dependency edges;
- explicit external provider classifications;
- domain-core effects and required compiler unknowns;
- application-owned port traits and adapter implementations;
- concrete adapter construction at composition roots;
- forbidden infrastructure types in inward contracts;
- capability executor roles and optional `no_std` obligations;
- explicit outbound effect authority classifications;
- protected witness opacity, constructor safety, and use cardinality;
- executor witness signatures and selected MIR origin provenance;
- distinct plan, attempt, observation, and receipt boundaries.

When `--artifact-dir` is set, an active run also adds:

- `project-architecture-policy-source.ncl`;
- `project-architecture-policy.json`;
- `project-architecture-cargo-graph.json`;
- `project-architecture-receipt.json`.

The receipt binds the Nickel source, exported policy, normalized command, lint status, Cargo graph, compiler IR, coverage, toolchain, roles, ports, capabilities, outbound authority protocols, findings, unknowns, waivers, and phase status.

JSON, SARIF, status, and human output keep lint and architecture phase records separate. Architecture findings use `policy-check` evidence. They are not lint findings.

Use `cargo octet artifact verify --artifact-dir PATH` to check stored identities and cross-artifact links.

## Admission mode

Admission runs these phases in order:

1. the `deterministic-core` strict lint profile;
2. the project architecture gate;
3. the existing selected-source effect-closure rail.

A later phase does not run after an earlier phase fails. Passing one phase does not change another phase status or evidence tier.

The policy must contain this explicit binding:

```nickel
admission = {
  strict_profile = "deterministic-core",
  effect_closure_policy = "octet-effect-closure.json",
}
```

The path must stay inside the workspace. Octet does not guess a closure policy. An explicit `--profile` value must match the binding.

An admission run adds these separate closure artifacts:

- `effect-closure.json`;
- `effect-closure-policy.json`;
- `effect-closure-dependency-graph.json`;
- `effect-closure-parent-status.json`.

The parent status contains the successful lint and architecture prerequisites. The architecture receipt links the closure receipt without merging findings, evidence tiers, or non-claims.

Admission fails if closure is denied or if any stored source, graph, provider, receipt, or parent link is missing or mismatched.

## Roles

Assign each production package and source scope exactly one production role:

- `domain-core`: deterministic domain meaning, policy, invariants, and transitions;
- `application-shell`: orchestration and application-owned port contracts;
- `inbound-adapter`: converts external requests into application inputs;
- `outbound-adapter`: implements application-owned ports;
- `composition-root`: selects concrete adapters and starts the application.

Use `test` and `generated` for explicit exclusions. These roles are not production roles.

Do not infer roles from directory names. Declare every package and source scope.

## Functional core and imperative shell

The domain core receives explicit facts, validated policy, state, actor context, identifiers, and prior observations.

It returns decisions, state, events, and effect plans. It does not invoke external capability ports.

The application shell interprets plans and invokes application-owned ports. Adapters implement those ports.

The composition root can construct concrete adapters. This exception does not give it domain-policy ownership.

## Application-owned ports

Each port declares:

- a compiler-qualified trait name;
- one `application-shell` owner scope;
- permitted inbound or outbound adapter scopes;
- forbidden concrete infrastructure type origins;
- reserved status.

Legacy `core_owner_scope` is invalid. Use `application_owner_scope` and move the contract to the application shell.

## Authorized outbound effects

Each effectful outbound adapter family must have one explicit authority classification:

- `unprotected` requires a reviewable reason;
- `protected` requires an exact typed protocol.

A protected protocol names the authority inputs, domain decision, authorized output, opaque witness, executor, plan, attempt, observation, receipt, constructors, scopes, cardinality, and non-claims.

The witness fields must be restricted to the witness owner module. The witness must not implement `Copy`, `Clone`, `Default`, `Deserialize`, broad `From`, or broad `Into`. Each function that returns the witness is checked against the approved safe constructor set.

A `consume` witness is one-shot. A `borrow` witness needs an explicit reusable scope and reason. A borrow declaration does not grant wider authority.

The compiler traces only selected protocol types and calls through optimized MIR. It follows local moves, projections, parameters, returns, and wrappers. Empty, merged, unresolved, or over-limit provenance becomes a blocking typed unknown in strict modes.

A protected executor must have the witness in its compiler-resolved input types. The effect site must remain in the declared executor method and outbound-adapter scope.

Success-receipt constructors must receive the declared observed-outcome type from an approved success observation. Failure-receipt constructors require an approved failure observation. A plan, attempt, witness, missing observation, or failed observation cannot become a success receipt.

Denial remains application behavior. Test it with controlled ports: a denied decision must yield no witness and zero protected executor calls. Static policy evidence does not prove that an external system denied or succeeded.

A passing protocol does not prove credential validity, revocation freshness, actor trust, business-policy correctness, external success, or receipt authenticity.

## Optional `no_std`

`core_no_std` is a separate host-independence obligation.

A `domain-core` role does not imply `no_std`. A `no_std` crate does not prove functional purity.

## Waivers

An architecture waiver must bind:

- one stable finding ID;
- the exact policy BLAKE3;
- the exact source BLAKE3;
- an owner;
- a reason;
- a review condition;
- one affected claim.

A changed policy, source, finding, or claim makes the waiver stale. A waived finding remains visible as `waived`; it is not `clean`.

Rust `#[allow]` attributes do not suppress project architecture findings.

## Claim boundary

A valid policy proves only that the generated policy input passed its typed and cross-field checks.

It does not prove complete FCIS conformance, complete Hexagonal conformance, runtime determinism, whole-program correctness, or release eligibility.
