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

The policy source must import the contract companion, and the two files must not import anything else. Octet scans their `import` targets and rejects a policy whose import closure differs from that one contract file.

Generate the runtime export and BLAKE3 freshness manifest with the reviewed
Nickel evaluator. `nickel --version` must print exactly
`nickel-lang-cli nickel 1.17.0 (rev 1320a98)`; the manifest generator rejects
any other evaluator identity, including the `nickel:unspecified` default used
when `--nickel-identity` is omitted.

```bash
nickel export --format json octet-architecture.ncl > octet-architecture.json
nickel export --format json octet-architecture-export-spec.ncl > /tmp/octet-architecture-export-spec.json
octet-standards nickel-export-manifest \
  --spec /tmp/octet-architecture-export-spec.json \
  --output octet-architecture.json \
  --nickel-identity "$(nickel --version)" \
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

An active policy starts compiler architecture collection for the selected Cargo units. Explicit `--all-targets` selection covers libraries, binaries, tests, examples, benchmarks, procedural macros, and supporting build-script compilations.

Cargo owns target and feature resolution. The planner requests a bounded, locked `cargo build --unit-graph` with the same selectors. It does not execute that build. The pinned Cargo check graph omits the distinction between normal and test checks, so array order is not a valid test-mode signal.

Actual compilation still uses `cargo check`. Context v3 binds the selected unit's source root and role to its test configuration, features, target triple, and profile identity. The collector compares the session's Rust target tuple, such as `aarch64-apple-darwin` or `riscv64gc-unknown-linux-gnu`, with the planned target, not rustc's LLVM target name. Unknown sessions in selected packages fail closed. Equivalent metadata and linking compilations can share a content-identical shard. Each process reserves its own staging file, so concurrent writers do not share mutable staging bytes.

The policy must define positive limits for units, shards, facts, unknowns, spans, artifacts, and selected MIR traversal.

The Cargo invocation must cover all declared production packages. Its target, features, and default-feature selection must match one declared target. Matrix rows use explicit feature sets. Several rows can share one target triple when their features or `default_features` differ; Octet selects the one row that matches the full selection.

An invocation can use `--all-features` if Cargo resolves it to the explicit policy row. Octet compares two bounded locked unit plans. The second plan replaces only `--all-features` with the policy features. Both plans retain the same package, target, profile, and default-feature selectors. Neither plan executes a build or build script. When several rows share the triple and default-feature selection, Octet plans each of them and requires exactly one equivalent row; no equivalent row or more than one equivalent row is rejected.

The comparison binds complete unit descriptions, resolved features, dependency edges, and selected roots. It ignores array order and replaces dependency indices with content identities. Extra features, feature-gated targets, optional dependencies, and changed edges cause rejection. Missing or ambiguous graph facts also cause rejection.

Declare package-qualified features when the workspace needs them. Cargo owns feature unification and optional-dependency resolution. `--no-default-features` remains a separate exact policy choice. Because Cargo includes named `default` features in `--all-features`, a row with disabled automatic defaults can need explicit `package/default` features.

Mixed `--all-features` and `--features`, `-F` shorthand, and repeated feature or target selectors remain errors. The actual invocation stays bound to compiler identities and receipts. All-feature support does not weaken missing-shard checks or change architecture modes.

Every selected source root needs a role, including build scripts outside `src`. Existing `--lib` policies can require those additional source declarations. Test and generated source roles remain explicit. Old collector contexts are rejected rather than assigned a default normal-library identity. Source roots inside excluded build or evidence directories are rejected.

Octet writes one content-bound shard for each selected compilation unit. The shard records resolved items, types, traits, implementations, signatures, calls, constructors, effects, expansions, and active configuration. Constructor facts cover struct literals, unit struct and unit variant paths, and calls, so `adapters::Http { base }` and `adapters::SystemClock` count as adapter construction just as `adapters::Http::new(base)` does.

Each unit identity binds its package source envelope: the package `Cargo.toml` and every `.rs` file below the package directory, including nested packages such as `fuzz/`. Build and evidence directory names such as `target` and `.octet` are excluded only in a directory that holds a `Cargo.toml`, so `fuzz/target/` is excluded while a module directory such as `src/target/` stays in the envelope. Non-`.rs` inputs, such as files read through `include_str!`, are not part of the envelope. Before collection, Octet keeps a cached shard only when it is the single cached shard for an expected unit; when one unit has several cached shards, all of them are discarded.

The collector runs inside rustc, so it writes a shard only when Cargo compiles the unit. Cached shards therefore stay tied to Cargo freshness. Every Octet compilation records `OCTET_COMPILER_ARCHITECTURE_CONTEXT` in rustc dep-info, so switching between a lint-only run and an architecture run recompiles the workspace units. Each selected unit also records a stamp file under `.octet/compiler-architecture/unit-stamps/`. The stamp is named from the unit's stable locator (its unit fields other than `unit_id` and the policy, source, and invocation hashes) and holds the current `unit_id`. Before Cargo runs, Octet rewrites a stamp when its content differs or when the unit kept no cached shard, and it removes stamps of units that are no longer expected. Any other stamp is left untouched, so Cargo keeps that unit fresh beside its cached shard. As a result, a changed waiver-free policy hash, collector library, invocation, or deleted shard directory recompiles the affected units, while an unchanged rerun recompiles nothing. When Cargo recompiles a unit whose identity did not change, for example after a workspace dependency changed, the new shard supersedes that unit's cached shard before normalization.

Calls inside constant and static initializers retain their actual compiler-owned caller items.
Named, associated, inline, and anonymous constants use the existing constant item kind.
Only function item owners request function signatures. Expression owners have private visibility, without a named-item visibility query.
Closures keep their separate identities, and selected value-provenance facts observed inside a closure name that closure item as their caller. Production memberships and effect checks still apply to nested and expanded bodies.
Each selected value-provenance fact references a recorded type: a tracked type from another crate, including one passed by reference, is recorded with its defining crate.
This repair does not change targets, roles, exemptions, or schemas.

A selected value-provenance parameter takes the origins of the function's in-crate direct calls only when those are all of its calls: the function is a private free function or inherent method that no other crate can reach and that exports no symbol, and it is never named outside a direct call nor called from a constant or static initializer. Every other parameter has an unknown origin, so a function also reached through a function pointer or from another crate never reports an exact origin.
A mutable borrow or raw pointer of a local makes that local's origin unknown from the borrow on. A write to a field or through a pointer adds origins instead of replacing them, and the result of a call through a function pointer has an unknown origin.
Value flow analyses only bodies that hold a value of a tracked type or pass one to a call; any other body yields no provenance, so it spends none of the `max_value_flow_steps` budget and does not count toward `max_value_flow_bodies`.

Item identities, and the identities of types defined in the compiled crate, are package-scoped for a package's library target.
Binary, example, test, bench, and build-script targets compile as separate crates that can share the library's crate name, so their identities also bind the Cargo target kind and name.
A `src/main.rs` item therefore never shares an identity with a same-named `src/lib.rs` item. A target's test configuration keeps that target's identities.

The collector classifies the root `getrandom` functions `fill`, `fill_uninit`, `u32`, and `u64` as randomness effects.
It uses the defining crate and root-function shape from compiler metadata, because diagnostic paths can retain consumer aliases.
Recorded call and effect paths remain equal. Their callers, spans, and production unit memberships remain attached.
Local same-named helpers, nested definitions, and unrelated entry points do not qualify through name resemblance.
This bounded classification does not claim complete randomness-provider coverage or dependency safety.

Calls to the `std::io` standard streams, including the `print!`, `println!`, `eprint!`, `eprintln!`, and `dbg!` expansions, are observability effects.
Calls on `std::sync` locks, once cells, barriers, condition variables, channels, and atomics are concurrency effects; `Arc` shared ownership is not.
Constructors (`new`) and exclusive-access operations (`get_mut`, `into_inner`) on those types are not concurrency effects: a constructor creates a value that no other thread can see yet, and exclusive access requires ownership or `&mut`. Every other operation, including ones not named here, counts.
Calls in `const` and `static` initializers, including inline and anonymous consts, run during compilation and record no effect, so `static COUNTER: Mutex<u32> = Mutex::new(0);` is effect-free while `COUNTER.lock()` in a function body is a concurrency effect.
Calls in `const fn` bodies and closures still count, because they can run at runtime.
A function item passed as a value, such as `unwrap_or_else(Instant::now)` or `map(std::fs::read_to_string)`, is recorded as a call from the enclosing body with the same effect classification as a direct call.
An overloaded operator, such as `a + b`, `x[i]`, or `*guard`, and an implicit `Deref` or `DerefMut` call, such as `CONFIG.port` on a `static CONFIG: LazyLock<Config>`, are recorded as calls to the trait method. Their effect is classified by the `Self` type of the implementation the call resolves to, so `CONFIG.port` is a `std::sync::LazyLock::deref` concurrency effect.
Plain value types under those paths are not effects, so comparing addresses or reading `SocketAddr::port` is effect-free while `TcpStream::connect` is a network effect. The excluded types are the `std::net` address types (`IpAddr`, `Ipv4Addr`, `Ipv6Addr`, `SocketAddr`, `SocketAddrV4`, `SocketAddrV6`), `AddrParseError`, and `Shutdown`; `std::fs::{FileTimes, FileType, Metadata, Permissions}`; `std::process::{ExitStatus, ExitStatusError, Output}`; `std::env::{JoinPathsError, VarError}`; `std::thread::{AccessError, ThreadId}`; `std::sync::{BarrierWaitResult, OnceState}`; atomic `Ordering` in `std::sync::atomic` and `core::sync::atomic`; and the `std::sync::mpsc` error types `RecvError`, `RecvTimeoutError`, `SendError`, `TryRecvError`, and `TrySendError`.
Reading the clock through `Instant::elapsed` or `SystemTime::elapsed` is a time effect like `Instant::now`; `duration_since` only compares two given times and is not. The `std::path::Path` methods that query the filesystem (`canonicalize`, `exists`, `is_dir`, `is_file`, `is_symlink`, `metadata`, `read_dir`, `read_link`, `symlink_metadata`, and `try_exists`) are filesystem effects, including calls on a `PathBuf` that reach them through `Deref`. Path manipulation such as `join`, `parent`, `file_name`, `extension`, `components`, `strip_prefix`, `starts_with`, and `display` is not.

The collector records unresolved dispatch, FFI, missing bodies, unsupported expansions, and ambiguous selected provenance as typed unknown facts. A trait method call is dynamic when its compiler-resolved `Self` type is a trait object, whether the receiver is a reference, `Box`, `Arc`, or `Rc`, or the call uses a qualified path such as `Trait::method(object)`. It is also dynamic when `Self` is a trait object behind references, `Box`, `Rc`, `Arc`, or `Pin` and that trait object implements the called trait, because forwarding impls such as `impl<W: Write + ?Sized> Write for Box<W>` make the vtable call inside the standard library; `out.write_all(..)` on a `Box<dyn Write>` is dynamic, while `Clone` on an `Arc<dyn Trait>` and calls on a boxed concrete type are not. External generated spans use redacted BLAKE3 provenance, not workspace paths. Files below a build or evidence directory that the source envelope excludes, such as build-script output included from `OUT_DIR` under `target/`, are external generated sources too. The collector does not infer a clean result.

When `--artifact-dir` is set, a complete run adds these files:

- `compiler-architecture-ir.json`;
- `compiler-architecture-coverage.json`;
- `compiler-architecture-issues.json`;
- `compiler-architecture-receipt.json`.

Missing, duplicate, stale, malformed, mismatched, or over-limit shards make collection incomplete. Octet saves the incomplete coverage and receipt before it returns an error.

Architecture IR records bounded compiler observations for the selected policy, Cargo graph, invocation, target, features, toolchain, collector, and sources.
Its policy identity is the waiver binding identity described under [Waivers](#waivers), so editing only `waivers` does not change unit or finding identities.
It does not prove runtime behavior, whole-program correctness, or observation authenticity.

Normalized IR and raw shards use schema v2. Unit identities bind the required `source_path`.
Normalized IR retains a required `unit_memberships` relation before shared facts and unknowns are deduplicated.
Missing, orphan, noncanonical, or unreferenced memberships reject even when the outer IR identity is recomputed.
A structurally possible relation is not independently authenticated by its hash.

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

Project receipt schema v3 requires `test_only`, even when its value is null, and `waiver_policy_blake3`, the waiver binding identity.
After successful projection, this field lists validated test units and exclusively test-observed fact and unknown IDs.
Shared production observations remain in production evaluation. Null means integrity errors prevented projection, not that evidence was safely excluded.
The full input identities and unknown list remain bound. Protocol identity arrays remain input metadata, not production ownership proof.

Production evaluation checks identities and reference closure before and after exclusion. Each required production unit needs its own no-std evidence.
Test-only witnesses, ports, types, configuration facts, and unknowns cannot satisfy production obligations.
Full bundle replay must reproduce the complete receipt and disposition. Recomputed receipt and provenance hashes do not replace replay or authenticate observations.

JSON, SARIF, status, and human output keep lint and architecture phase records separate. Architecture findings use `policy-check` evidence. They are not lint findings.
In SARIF, a waived architecture finding has level `note`. An open `integration` finding has level `error` in every mode, because it blocks every mode. An open `policy` finding has level `warning` in `inventory` and `advisory` modes and `error` in `gate` and `admission` modes.

Use `cargo octet artifact verify --artifact-dir PATH` to check stored identities and cross-artifact links.

## Admission mode

Admission runs these phases in order:

1. the `deterministic-core` strict lint profile;
2. the project architecture gate;
3. the existing selected-source effect-closure rail.

The architecture gate still runs when the lint phase reports findings, as long as
compiler architecture facts are complete; its receipt records the lint status.
Effect closure runs only after the lint phase is clean and the architecture gate
has no open policy findings. Passing one phase does not change another phase
status or evidence tier.

The policy must contain this explicit binding:

```nickel
admission = {
  strict_profile = "deterministic-core",
  effect_closure_policy = "octet-effect-closure.json",
}
```

The path must stay inside the workspace. Octet does not guess a closure policy. An explicit `--profile` value must match the binding.
The run checks function addresses against the baseline that the bound closure policy names. A baseline given by `--function-address-baseline`, or named by the policy passed to `cargo octet admit --policy`, must be that same file. Otherwise the run stops with exit code 2 and names both baselines.

An admission run adds these separate closure artifacts:

- `effect-closure.json`;
- `effect-closure-policy.json`;
- `effect-closure-dependency-graph.json`;
- `effect-closure-parent-status.json`.

The parent status contains the successful lint and architecture prerequisites. The architecture receipt links the closure receipt without merging findings, evidence tiers, or non-claims.

Admission fails if closure is denied or if any stored source, graph, provider, receipt, or parent link is missing or mismatched.

`cargo octet artifact verify` checks an admitted closure the same way `cargo octet admit verify` does. It validates the stored closure policy, requires the receipt's record root and sources to match `function-addresses.json`, and re-checks nodes, roles, and reviewed summaries against those artifacts. A denied closure that is relabelled admitted and given a recomputed identity therefore fails verification.

## Roles

Assign each production package and source scope exactly one production role:

- `domain-core`: deterministic domain meaning, policy, invariants, and transitions;
- `application-shell`: orchestration and application-owned port contracts;
- `inbound-adapter`: converts external requests into application inputs;
- `outbound-adapter`: implements application-owned ports;
- `composition-root`: selects concrete adapters and starts the application.

Whole-package `test` and `generated` roles do not grant compilation-unit exemptions.
An independent integration test can receive `test` only through its explicit source scope and compiler-bound `test+test` target.
Libraries and binaries retain their production roles in test configurations. Examples, benchmarks, and test-looking names do not receive test exemptions.
A `generated` source declaration does not grant a compilation-unit exemption.

The collector preserves physical workspace source ownership for parent-relative imports. Source symlinks reject at the source envelope before collection.
Importing test-labelled source into a production root remains an integrity failure. Full lint and compilation target selection stays enabled.

Do not infer roles from directory names. Declare every package and source scope.

## Functional core and imperative shell

The domain core receives explicit facts, validated policy, state, actor context, identifiers, and prior observations.

It returns decisions, state, events, and effect plans. It does not invoke external capability ports.

The application shell interprets plans and invokes application-owned ports. Adapters implement those ports.

The composition root can construct concrete adapters. This exception does not give it domain-policy ownership.
An adapter package can also construct its own types, for example `Self { .. }` in its own constructor. Any other unit that constructs an adapter type gets `adapter-construction-outside-root`.
A call constructs what its definition declares it returns, so `SqlRepo::connect(&url)?` returning `Result<Self, Error>`, or a call returning `Option<Box<SqlRepo>>` or `Arc<SqlRepo>`, constructs `SqlRepo`. Only `Result`, `Option`, `Box`, `Arc`, `Rc`, tuples, and arrays hold a constructed value inside; a guard, iterator, `Ref`, `Pin`, or collection does not, and allocator or hasher type parameters are never constructed. A type parameter that the callee received, through an input or a caller-supplied closure's return, is passed through, not constructed: `Arc::clone(&repo)`, `repo.clone()`, `vec.pop()`, `Some(repo)`, `None`, `option.map(..)`, and `for repo in &repos` construct no `SqlRepo`. A type parameter the callee did not receive is one it produced, so `url.into()`, `url.parse::<SqlRepo>()`, and `make::<SqlRepo>()` construct `SqlRepo`. References and trait objects construct nothing. Each written call records its construction once: the `?`, `.await`, and `for` desugarings add none, and `SqlRepo::connect_async(&url).await?` records `SqlRepo` through its future's declared output.

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

An unprotected-only receipt can have empty authorization fact and type identity arrays.
If any effect is protected, both arrays must be nonempty, including in a mixed effect list.
The authority classification controls this requirement, not the optional protocol payload.
All receipt identity, non-claim, compiler coverage, and policy-binding checks still apply.
Receipt self-verification alone does not establish a passing architecture phase or authorize execution.

A protected protocol names the authority inputs, domain decision, authorized output, opaque witness, executor, plan, attempt, observation, receipt, constructors, scopes, cardinality, and non-claims.

The witness fields must be restricted to the witness owner module. The witness must not implement `Copy`, `Clone`, `Default`, `Deserialize`, broad `From`, or broad `Into`. A conversion is narrow only when its type argument is a declared authorized output type, as in `impl From<domain::AuthorizedPublication> for domain::PublicationAuthority`; a generic, wrapped, or other argument is broad. The collector records each implemented trait with its type arguments. Each function that returns the witness is checked against the approved safe constructor set.

A `consume` witness is one-shot. Its declared constructors must take the authorized output by value: a constructor that borrows it, as in `fn mint(output: &AuthorizedOutput) -> Witness`, lets one authorization mint several witnesses and gets the blocking `consume-authorization-borrowed` finding. A conversion such as `impl From<&AuthorizedOutput> for Witness` is broad in every mode. A `borrow` witness needs an explicit reusable scope and reason, and its constructors may borrow the authorized output. A borrow declaration does not grant wider authority.

The compiler traces only selected protocol types and calls through optimized MIR. It follows local moves, projections, parameters, returns, and wrappers. Empty, merged, unresolved, or over-limit provenance becomes a blocking typed unknown in strict modes. Exceeding a selected MIR body, step, or round limit also makes that unit's shard incomplete.

A protected executor must have the witness in its compiler-resolved input types. Signature inputs name the referent type and record whether each input is passed by reference. Under `borrow`, a `&Witness` or `&mut Witness` parameter satisfies this. Under `consume`, the executor must take the witness by value; a borrowed witness survives the call and could authorize another, so it gets the blocking `consume-witness-borrowed` finding. The effect site must remain in the declared executor method and outbound-adapter scope.
Policy validation therefore requires a protected effect's `executor_scope` to be an outbound-adapter source scope. Unprotected effects may also execute in application-shell or composition-root scopes.

Success-receipt constructors must receive the declared observed-outcome type from an approved success observation. Failure-receipt constructors require an approved failure observation. A plan, attempt, witness, missing observation, or failed observation cannot become a success receipt.

Denial remains application behavior. Test it with controlled ports: a denied decision must yield no witness and zero protected executor calls. Static policy evidence does not prove that an external system denied or succeeded.

A passing protocol does not prove credential validity, revocation freshness, actor trust, business-policy correctness, external success, or receipt authenticity.

## Optional `no_std`

`core_no_std` is a separate host-independence obligation.

A `domain-core` role does not imply `no_std`. A `no_std` crate does not prove functional purity.

The collector reads the active, parsed crate attribute from rustc. Source text and inactive `cfg_attr` branches cannot supply this fact.
Each required production unit needs its own observation, including library test configurations.
A package-scoped obligation binds the package's library units: the library and its test configuration. Build scripts, binaries, examples, and benches are separate crates with their own roots, so they are not required to carry the library's attribute; name such a root as a source scope to require it there. A package without a library target cannot satisfy a package-scoped obligation.
An integration-only fact cannot satisfy that obligation. An active `no_std` attribute does not prove that every dependency excludes `std`.

## Waivers

An architecture waiver must bind:

- one stable finding ID;
- the policy waiver binding identity;
- the exact source BLAKE3;
- an owner;
- a reason;
- a review condition;
- one affected claim.

Waivers live inside the exported policy, so they cannot bind the BLAKE3 of those export bytes. The waiver binding identity is the BLAKE3 of the canonical compact JSON of the typed policy with `waivers` emptied. Compiler units, policy-sourced findings, and waivers all bind it.

To write a waiver, copy `finding_id`, `source_blake3`, and `affected_claim` from the finding in `project-architecture-receipt.json`, and copy the receipt's `waiver_policy_blake3` into the waiver's `policy_blake3`. Adding or editing waivers leaves that identity unchanged. The receipt's `policy_blake3` still names the exact export bytes.

A change to any non-waiver policy field, the source, the finding, or the claim makes the waiver stale. A waived finding remains visible as `waived`; it is not `clean`.

A finding's `source_blake3` names the source it depends on:

- A finding about one compiler fact binds the source envelope of the unit that observed it.
- A finding about what a declared scope lacks or resolves ambiguously, such as `no-std-obligation`, `port-trait-resolution`, `port-implementation-missing`, or a protected-protocol `*-resolution` or `*-provenance-missing` finding, binds the source envelopes of every production unit that owns the scope. A unit owns a scope that names its package, contains its target root, lies under its target root directory, or contains one of its observations. When no unit owns the scope, the finding binds only the waiver binding identity.
- A finding about the policy alone, such as `missing-composition-root` or `missing-protected-effect-protocol`, binds only the waiver binding identity.

A source edit therefore makes stale every waiver whose finding binds that unit's source envelope.

When every policy finding is waived, the architecture status can still be `clean`. The receipt keeps each waived finding and lists it in `applied_waiver_ids`, `status.json` reports `phases.architecture.waived_finding_count`, and the summary's architecture phase line names the waived count.

Waivers apply only to policy findings. Input-integrity findings, such as incomplete compiler coverage, stay open; a waiver that targets one adds an `unwaivable-integration-finding` integration failure.

Rust `#[allow]` attributes do not suppress project architecture findings.

## Claim boundary

A valid policy proves only that the generated policy input passed its typed and cross-field checks.

It does not prove complete FCIS conformance, complete Hexagonal conformance, runtime determinism, whole-program correctness, or release eligibility.
