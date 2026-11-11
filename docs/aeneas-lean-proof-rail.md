# Aeneas-to-Lean proof rail

The `aeneas-lean` provider is an opt-in formal-proof rail. Ordinary
`cargo octet check` does not select it, require Aeneas, require Charon, or
change starter lint behavior. Select the rail through the checked proof-policy
artifacts and `cargo octet prove` only after the provider profile and package
closure have been reviewed.

## Provider selection

The reviewed provider profile is
`standards/proof-rails/aeneas-lean/profile.ncl`, with its generated runtime
export at `standards/proof-rails/aeneas-lean/generated/profile.json`. It binds
Aeneas, Charon, OCaml, Lean, the Aeneas Lean library, command argv, execution
limits, the environment allowlist, extraction policy, assumptions, semantic
reference, and non-claims.

The execution shell requires explicit absolute provider executable paths through
the shell-owned configuration:

- `OCTET_AENEAS_CHARON`;
- `OCTET_AENEAS_AENEAS`;
- `OCTET_AENEAS_LEAN`;
- `OCTET_AENEAS_LEAN_LIBRARY`; and
- `OCTET_AENEAS_NETWORK_SANDBOX`.

The child environment is cleared and rebuilt only from the profile allowlist; a
rail that declares a different `environment_allowlist` is rejected before any
stage runs, and the receipt records the profile allowlist that was applied.
Each rail limit (`timeout_ms`, `stdout_max_bytes`, `stderr_max_bytes`) falls
back to the profile default only when the rail leaves it at zero, and the
receipt records exactly the limits that were enforced.
The provider package must also supply the exact Rust driver/runtime closure
required by the pinned Charon build; the host rustc is not an implicit fallback.
Each stage runs with argv, a fresh isolated work root, bounded output capture,
a named timeout, a denied network namespace, and a private PID namespace. Each
stage runs in its own process group; on timeout, and after the stage exits,
the whole group is killed and output-reader teardown is bounded, so Cargo,
rustc, or Lean worker grandchildren cannot outlive the stage or stall the shell.
Before execution the work root receives the source package's Rust inputs and
the pinned Aeneas Lean library; the Nix store is mounted read-only so absolute
provider binaries remain available. The source copy leaves out `.git`,
`target`, and `.lake` directories and every `.lean` file, so the Lake package
holds no module other than the generated `Llbc`, the approved `Theorem`, and
the pinned library; a theorem that imports another local module fails to
build. A source root that contains its own `aeneas-lean-library` entry is
refused, because it would merge into the pinned library. The work root path is
a deterministic BLAKE3 of the source root, backend, and rail argv; it is removed
after the run on success and on failure, so rerunning a rail starts fresh. The
shell creates the root itself with an atomic create, so a root that already
exists belongs to a concurrent run, or to a process that was killed before
cleanup, and is refused rather than reused; remove it after confirming no run
is active.
Mathlib and every Lake dependency are fixed-output, path-only closure inputs;
Lake receives frozen root and nested manifests and cannot fetch missing code.
A failed or incomplete stage stops the pipeline.

Lean builds the fixed `Llbc` and `Theorem` modules. When it prepares the work
root, the shell writes the exact preflighted, approved bytes of the rail's first
`proof_paths` entry to the work root's `Theorem.lean`, so the checked theorem is
always the approved one even when `proof_paths` names another file.

## Stage gates

Each stage's outputs are measured before the next stage may run:

- After Charon, the effective extraction policy is observed from the produced
  LLBC (`has_errors`, the serialized `options`, and `target_information`) and
  compared with the reviewed profile's `extraction_policy`. Charon's options are
  mapped into the profile vocabulary: `mir = null` is `borrowck`; no precise or
  desugared drops is `include-drops`; no `start_from*` selection is `crate`;
  spans on every declaration is `preserve-spans`; and JSON-encoded structured
  LLBC is `llbc-json-v1`. Any other value fails the comparison. The receipt's
  `extraction_policy` and `charon-options.json` record this observed policy, not
  a copy of the profile.
- The reviewed profile's `include_rules` are recorded in the provider profile
  but not applied by the executed Charon argv. The receipt records the observed
  include rules, and Octet does not compare them with the profile until the
  profile owner resolves that mismatch.
- After Aeneas, `translation.json` is parsed and linked to the produced LLBC and
  the measured `Llbc.lean`: every local function must name the same LLBC
  declaration, Rust name, and source lines, and every named Lean file must be
  measured.
- Also after Aeneas, the maintained theorem source and the generated
  `Llbc.lean` are scanned as Lean tokens, skipping comments, string text, and
  character literals. Only strings that Lean 4.31 parses as interpolated have
  their `{…}` holes scanned as code: those after `s!`, `m!`, `f!`,
  `println!`, `dbg_trace`, `throwError`, `throwErrorAt`, the
  `throwNamedError` and `logNamed…` commands, and `trace[…]`-style macros. A
  string after any other name, such as `panic!` or `HashMap.get!`, is plain
  text. `«…»`-quoted names are never keywords. `axiom` declarations outside
  the profile's declared axioms and external models, `opaque` definitions
  outside its external models, `native_decide`, `ofReduceBool`, `ofReduceNat`,
  `implemented_by`, and `extern` are undeclared inputs, and `sorry`, `admit`,
  and `sorryAx` are admitted declarations. Either kind fails the rail before
  Lean runs. This scan is only an early filter: Lean can add trust that no
  token names, such as the `stop` tactic, which elaborates to `repeat sorry`,
  or a meta-level `addDecl` of an axiom.
- After `lake build` exits cleanly, its log must carry no
  ``declaration uses `sorry` `` warning (or the older `'sorry'` spelling) for
  `Theorem.lean` or `Llbc.lean`; `lake build` exits successfully on those
  warnings, so the shell fails the Lean stage itself. Such warnings from the
  reviewed Aeneas library, which Lake rebuilds in the same log, stay inside that
  library's trust boundary and surface only through the axiom report below.
- The shell then writes a separate check program to the work root and runs it
  with `lake env lean --run OctetAxiomCheck.lean <theorem>` in the same sandbox
  and limits. `<theorem>` is the name the adequacy review assesses: the first
  line of the approved source that starts with `theorem `. It must be a theorem
  declared at that name by the `Theorem` module, so a theorem inside a
  `namespace` fails the check, and a `private` or attributed theorem is never
  the approved name. The program imports the built modules as data and prints
  one JSON line (`octet-aeneas-lean-axioms/v1`) whose `axioms` are the union of
  what `Lean.collectAxioms` reports for every constant the `Theorem` module
  declares, not only the approved theorem. An admitted `@[simp]`, `private`,
  namespaced, or doc-commented lemma beside the approved theorem therefore
  still fails the rail. The approved theorem file is never modified, and no
  theorem-defined command or elaborator runs while the report is produced. The
  report is the `lean-axioms.json` artifact, and the trust closure the receipt
  records is the token scan plus these axioms: `propext`, `Classical.choice`,
  and `Quot.sound` are accepted; `sorryAx` is an admitted declaration;
  `Lean.ofReduceBool`, `Lean.ofReduceNat`, `Lean.trustCompiler`, and the
  `_native` auxiliary axioms `native_decide` declares are native-code trust,
  accepted only when named by the profile's `axioms`; any other axiom must be
  named by the profile's `axioms` or `external_models`. A missing, malformed,
  or foreign report, or any undeclared or admitted entry, fails the rail.

## Supported subset

The initial claim surface is safe, sequential Rust in the reviewed
Charon-supported subset. The following remain blocked or failed and never
become `formal_proof` evidence:

- unsafe Rust;
- concurrent Rust;
- unsupported nested-loop control transfer;
- unresolved external definitions;
- unsupported language or extraction features; and
- translation or checker failures.

The first executable canary further narrows the staged Lean prelude to
`Aeneas.Data`, `Aeneas.Do`, and `Aeneas.Std.Scalar`. It excludes the broad
command, extraction, `Std`, and tactic umbrellas; this avoids importing
unrelated modules and does not promote excluded surfaces into the supported
claim.

A successful translation is still only a recorded stage observation until the
complete stage chain and the named theorem admission pass.

## Assumption and model review

Every external model, axiom, assumption, opaque definition, and admitted
declaration must be declared in the provider payload and bound into the stage
chain. The first provider profile has an explicit empty assumption inventory;
that emptiness is a reviewed fact, not an implicit default. Missing, stale, or
undeclared trust inputs fail closed.

The exact `mechanized-llbc` revision in the profile is recorded-only semantic
foundation metadata. It does not certify the pinned Aeneas or Charon binaries,
and it does not transfer translation soundness.

## Theorem requirements

Promotion requires all of the following:

1. a named theorem over the generated model;
2. an independently reviewed and configured theorem-statement BLAKE3 that matches the measured source bytes;
3. a requirement ID and explicit claim class;
4. an explicit generated-versus-maintained source boundary;
5. a non-vacuous adequacy decision;
6. a complete declared trust closure with no `sorry`, `admit`, undeclared
   model, or undeclared axiom; and
7. a passing Lean check for that exact theorem.

False, `True`-style vacuous, approval-hash-mismatched, stale, or unbound theorem selections remain recorded or
failed observations and do not receive `formal_proof` provenance.

## Rollout order

1. Run the Octet-owned positive fixture and its false/vacuous theorem negatives.
2. Review Charon extraction and `translation.json` identities, then verify the
   generated/maintained Lean boundary and theorem artifacts.
3. Verify the complete Octet receipt and artifact directory.
4. Export the frozen Valence adapter input only after that verification passes.
5. Treat Aeneas and Charon upgrades as compatibility-matrix changes, not as
   ordinary dependency updates.

The Valence adapter preserves intermediate translation stages as bounded
recorded or boundary evidence. The final Octet-owned named theorem receipt is
the only property candidate.

## Consumer contract status

The shipped policy registry selects only the Octet-owned `safe_add` fixture.
It does not publish arbitrary downstream adoption as a compatibility surface.

The active Cairn change
`.cairn/changes/generalize-aeneas-consumer-contract/` defines the reusable
consumer profile, exact-source selection, outcome classes, and revision-bound
handoff. Until that change archives, downstream consumers must treat the
provider as a reviewed dependency with consumer-specific integration work.

The [AeneasVerif integration roadmap](aeneasverif-integration-roadmap.md)
records stack ownership and the deferred Eurydice and Scylla stages.

## Explicit non-claims

An accepted Aeneas/Lean receipt is scoped to the named theorem, exact source,
exact generated model, assumptions, package, toolchain, and stage identities.
It does not claim:

- Aeneas, Charon, Lean, rustc, or verifier soundness;
- translation soundness or semantic equivalence;
- external-model soundness;
- compiler correctness;
- whole-program correctness;
- unsafe or concurrent Rust correctness; or
- release eligibility.

BLAKE3 identities establish exact bytes and linkage. They do not independently
establish the truth of a theorem or the soundness of a verifier.

## Review commands

Run the focused profile and provider checks inside the Nix development shell,
then inspect the generated receipt and artifact verification output. The
provider-specific evidence files under
`.cairn/archive/2026-07-17-add-aeneas-lean-proof-rail/evidence/` record the positive and
negative commands used for the current pins. The executable upgrade gate is:

```bash
nix build --option builders '' --option secret-key-files '' \
  .#checks.x86_64-linux.aeneas-lean-compatibility-matrix --no-link -L
```

It runs two live provider fixtures serially plus the pure admission/rejection
matrix. The empty builder and signing-key options disable unavailable host
remote/signing configuration; they do not weaken repository policy.
