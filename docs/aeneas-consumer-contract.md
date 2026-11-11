# Aeneas consumer contract

<!-- r[impl octet.aeneas_consumer_contract.boundaries] -->

This contract lets an external Rust project use the pinned Octet Aeneas rail.
It keeps provider authority and consumer authority separate.

The checked profile is:

- `standards/proof-rails/aeneas-lean/consumer-profile.ncl`
- `standards/proof-rails/aeneas-lean/generated/consumer-profile.json`
- profile identity `b3:62bb3c01a2037eef1410e35d3902d73fb5e7439e8856d721d0823e14c0af4879`

## Scope

The contract binds these inputs:

1. The exact provider profile and compatibility matrix.
2. One exact Rust source file and selected item.
3. The selected item span and durable source identity.
4. The LLBC output, translation manifest, and generated Lean source.
5. One maintained theorem and its requirement mapping.
6. The declared resource policy, trust closure, and non-claims.

A transient Charon `def_id` is recorded as an observation. It is not part of the
durable item identity.

The contract does not prove:

- Aeneas or Charon translation soundness.
- Lean or verifier soundness.
- Whole-program correctness.
- Semantic equivalence outside the named theorem.
- Release eligibility.

## Use the checked profile

Pin an immutable Octet revision. Do not discover an ambient sibling checkout.
Export Octet policy artifacts from that pinned revision:

```console
nix run .#octet-policy-artifacts -- artifacts/policy
```

A selected consumer keeps these files in one project root:

```text
Cargo.toml
src/lib.rs
Theorem.lean
```

The checked fixture is at:

`standards/proof-rails/aeneas-lean/fixtures/external-consumer/`

Run its exact rail with the pinned provider closure:

```console
cargo octet prove \
  --policy-artifacts-dir artifacts/policy \
  --source src/lib.rs \
  --rail octet.aeneas_consumer_contract.external-fixture#rail-0 \
  --artifact-dir artifacts/aeneas-consumer
cargo octet artifact verify --artifact-dir artifacts/aeneas-consumer
```

The proof receipt contains a typed `consumer` binding. The binding includes the
profile, provider, source, item, generated stages, theorem, requirement, resource
policy, trust closure, outcome, and receipt identity.

## Outcomes

The contract uses three closed outcomes:

- `supported`: all exact bindings match and the named theorem passes.
- `blocked`: a required provider capability or compatibility condition is absent.
- `failed`: execution completed but the theorem or checked contract failed.

A blocked result is not a successful proof. An incomplete blocker is invalid.
A failed result is not a blocked result.

## Reject drift

The checker rejects these changes:

- Provider profile or compatibility-matrix substitution.
- Source, selected-item, span, or theorem changes.
- Path escape or symbolic-link substitution.
- Ambiguous selected-item matches.
- Missing LLBC, translation, generated Lean, or theorem bindings.
- Resource-policy or environment-allowlist drift.
- Empty trust boundaries or non-claims.
- Unknown Nickel or JSON fields.
- Receipt identity or stage-artifact tampering.

Review each provider or profile upgrade as a new cohort. Run the compatibility
matrix and regenerate the consumer profile. Do not update one identity alone.

## Ownership and deferred translators

Octet owns the consumer rail, execution policy, and receipt construction.
The consumer owns theorem intent, requirement mapping, and theorem campaigns.
Trellis owns reusable verified logic and its proof corpus when a consumer uses it.
A Trellis proof does not gain Octet consumer-proof authority without an Octet rerun.

Valence owns evidence identity and linkage. Mantle owns bounded build evidence.
Cairn owns lifecycle gates and release-readiness decisions. Workspace membership
does not transfer authority between these owners.

Eurydice and Scylla remain deferred translation references. This consumer
contract does not execute them or claim compatibility with their outputs.

## Downstream profiles

A downstream project can define another closed Nickel profile with the same
contracts. Load its exact exported bytes with `aeneas_consumer::load_profile`.
Pass the returned measured identity to `build_binding` and
`validate_receipt_against`.

The profile must select its own source paths, items, theorems, requirements,
resource limits, trust boundaries, and non-claims.

Octet does not infer these declarations from repository membership. It does not
grant proof authority because a project uses the same workspace or provider.

## Mantle producer handoff

Octet owns the executable handoff in
`standards/proof-rails/aeneas-lean/mantle-job.ncl`
(`octet-mantle-producer-job/v1`). It selects this existing consumer/provider
profile and reviewed external-fixture rail, their resource bounds, exact
commands, denied-network execution and output contract. It introduces no
alternative theorem admission policy. This selected Lean rail has no SMT solver.

`checks.x86_64-linux.mantle-producer-export` exercises ordinary graph admission
and exports this typed job with the pinned Nickel evaluator. It does not execute
the proof. The existing `aeneas-consumer-contract` check also checks generated
consumer-profile freshness against its Nickel source.

`packages.x86_64-linux.octet-mantle-aeneas-consumer-job` realizes that contract
as an ordinary derivation: it runs actual `cargo-octet prove` and
`cargo-octet artifact verify`, requires successful owner verification, and
preserves the original report, envelope and full artifact payload tree. Cached
output bytes record their original acceptance; reuse is not fresh proof work.

The producer fails closed at each stage. On failure it prints the stage and
original exit status, emitted producer/verification reports, and original proof
stdout, stderr and receipts into the Nix build log. It also copies original
reports and the artifact tree to `$TMPDIR/producer-diagnostics` before failed
output cleanup. Use `nix build --keep-failed -L` for a diagnostic execution and
retain that directory from Nix's reported failed build directory. These bytes
are failure diagnostics, not an admitted output, a reconstructed receipt or
evidence of a successful proof. The consumer-profile byte comparison and both
owner verification predicates remain mandatory.

The job declares `limits.cores = 32` and restricts the builder to the first
32 CPUs in its inherited allowed CPU list, or fewer when positive
`NIX_BUILD_CORES` or the inherited list is smaller. Nix's zero-core setting
means all available cores; the job's explicit 32-CPU cap still applies.
Linux `taskset` applies this affinity before producer execution, and the
unchanged network sandbox and provider processes inherit it. Sparse CPU lists
are preserved rather than assuming CPUs start at zero or broadening affinity.

This is necessary because pinned Aeneas creates its worker pool from OCaml
5.2.1's `Domain.recommended_domain_count`, which reads CPU affinity rather than
`NIX_BUILD_CORES`. Unbounded host-wide recommendations caused an observed
`Failure "failed to allocate domain"` before translation completed. The pinned
runtime has no domain-count environment override. The cap bounds allocated CPUs
and the worker recommendation, not total domains or threads: Domainslib workers
are additional to the main domain. No `-sequential` argument, provider/profile
identity change, theorem substitution or proof-isolation relaxation is used.

The restricted producer `PATH` includes the pinned Nixpkgs `stdenv.cc` wrapper.
Lean's native compilation invokes `cc`; a Rust-only tool path caused the actual
external consumer build to fail with `could not execute external process 'cc'`.
The compiler is an explicit job dependency in the ordinary derivation closure,
not ambient host discovery or an added provider argument.

`tools/mantle-producer/export.py --help` describes acquisition-only export. It
requires `--revision` with the exact committed owner Git revision, an immutable
`--flake`, and an explicit `--graph-format`. There is no mutable `--root`
override. The exporter resolves the flake first, loads the Nickel job and all
its imports from the locked source, checks that its own bytes match that source,
and evaluates Nix's locked URL rather than resolving a mutable branch again.

The Nickel job declares `tools.graph_formats`. Its wire contracts are separate:

| Graph format | Original Nix wire shape |
| --- | --- |
| `nix-derivation-show-legacy` | Flat map of absolute `/nix/store/*.drv` paths; nodes contain absolute `inputDrvs` and `inputSrcs`. |
| `nix-derivation-show-v4` | Top-level integer `version: 4` and `derivations` map keyed by store basenames; each node has integer `version: 4` and basename references in `inputs.drvs` and `inputs.srcs`. Each dependency selection has `outputs` and empty `dynamicOutputs`. |

`--graph-format nix-derivation-show-v4` requires actual v4 output. It does not
convert legacy output into v4 or downgrade v4 to legacy. A mismatched requested
format, unknown/mixed version, missing root/source binding, incomplete recursive
edge, or dynamic output fails closed before the destination is created.
Consumers must implement the actual selected wire contract or refuse it; they
must not hash-relabel reconstructed legacy derivations as original v4 identities.
The graph's wire version is independent of the Octet job/profile/receipt schema
versions and the Nix executable's release number.

Export retains exact `job.json`, `tool-closure.derivation-json.json`,
`flake-metadata.json`, `flake.lock`, `root-derivation`, `source-revision`, and
all declared source objects under `source-payloads/`. Original graph bytes are
never rewritten. `/nix/store/` is added to v4 basenames only to locate existing
source objects for copying. Graph size, derivation/source count and source bytes
are bounded. Acquisition does not build tools or execute a proof. Its
`proof_executed: false` observation is not a proof outcome or external acceptance.
The complete graph includes declared bootstrap/binary acquisition inputs; it
does not claim source-only or seed-free bootstrap.

Mantle may consume that frozen graph through its existing build/source/store
boundaries and preserve Octet's full external acceptance report. Mantle does
not gain profile, theorem, proof admission, signature trust or release authority
by doing so. Real execution/replay evidence must identify the actual route;
importing a prebuilt closure is not evidence of source-built tools.

## Final immutable-source verification

The integration owner supplies an accessible committed `OCTET_REVISION`, its
`OCTET_FLAKE`, absolute `NIX` and reviewed `NICKEL` executable paths, and a new
`EXPORT` destination. Run the exporter from that exact checked-out revision:

```bash
python3 -B tools/mantle-producer/export.py \
  --nix "$NIX" --nickel "$NICKEL" \
  --flake "$OCTET_FLAKE" --revision "$OCTET_REVISION" \
  --graph-format nix-derivation-show-v4 --out "$EXPORT"
"$NIX" build --no-link --no-write-lock-file -L \
  "$OCTET_FLAKE#checks.x86_64-linux.mantle-producer-export" \
  "$OCTET_FLAKE#checks.x86_64-linux.aeneas-consumer-contract" \
  "$OCTET_FLAKE#checks.x86_64-linux.policy-registry"
out=$("$NIX" build --no-link --print-out-paths --no-write-lock-file -L \
  "$OCTET_FLAKE#packages.x86_64-linux.octet-mantle-aeneas-consumer-job")
runner=$("$NIX" build --no-link --print-out-paths --no-write-lock-file \
  "$OCTET_FLAKE#cargo-octet")
test "$(cat "$out/source-revision")" = "$OCTET_REVISION"
cmp "$EXPORT/job.json" "$out/job.json"
cmp "$out/consumer-profile.json" "$out/external-profile.json"
cmp "$out/external-receipt.json" \
  "$out/artifacts/proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.receipt"
jq -e '.schema_version == 1 and .status == "valid" and .diagnostics == []' \
  "$out/producer-artifact-verification.json"
jq -e '.schema_version == "octet-formal-proof-envelope/v1" and .status == "passed" and .consumer.outcome == "supported" and .consumer.verification_role == "property"' \
  "$out/external-receipt.json"
"$runner/bin/cargo-octet" artifact verify --artifact-dir "$out/artifacts" --format json
```

For a fail-closed negative on the actual emitted payload, use a throwaway copy,
not a canned expected fixture or a store mutation:

```bash
negative=$(mktemp -d)
cp -R "$out/artifacts" "$negative/artifacts"
chmod -R u+w "$negative/artifacts"
printf '\n-- tampered after producer execution\n' >> "$negative/artifacts/proofs/octet.aeneas_consumer_contract.external-fixture-rail-0.aeneas/Llbc.lean"
rc=0
"$runner/bin/cargo-octet" artifact verify \
  --artifact-dir "$negative/artifacts" --format json > "$negative/report.json" || rc=$?
test "$rc" -eq 1
jq -e '.schema_version == 1 and .status == "invalid" and (.diagnostics | length > 0)' \
  "$negative/report.json"
rm -rf "$negative"
```

On the same actual Nix v4 producer, the incompatible legacy request must fail
before creating its unused destination:

```bash
incompatible="$EXPORT.incompatible-wire"
test ! -e "$incompatible"
rc=0
python3 -B tools/mantle-producer/export.py \
  --nix "$NIX" --nickel "$NICKEL" \
  --flake "$OCTET_FLAKE" --revision "$OCTET_REVISION" \
  --graph-format nix-derivation-show-legacy --out "$incompatible" \
  > "$EXPORT/incompatible-wire.stdout" 2> "$EXPORT/incompatible-wire.stderr" || rc=$?
test "$rc" -ne 0
grep -F 'exported graph format differs from explicit consumer contract' \
  "$EXPORT/incompatible-wire.stderr"
test ! -e "$incompatible"
```

Record real commands, the immutable revision, original reports and artifact
identities. A dry run, successful source export, passing graph fixtures, or
reused closure alone is not fresh proof execution, Mantle acceptance,
publication, or release approval.

