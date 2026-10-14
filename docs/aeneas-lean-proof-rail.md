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

The child environment is cleared and rebuilt only from the profile allowlist.
The provider package must also supply the exact Rust driver/runtime closure
required by the pinned Charon build; the host rustc is not an implicit fallback.
Each stage runs with argv, a fresh isolated work root, bounded output capture,
a named timeout, and a denied network namespace. The source package and pinned
Aeneas Lean library are copied into that work root before execution; the Nix
store is mounted read-only so absolute provider binaries remain available.
Mathlib and every Lake dependency are fixed-output, path-only closure inputs;
Lake receives frozen root and nested manifests and cannot fetch missing code.
A failed or incomplete stage stops the pipeline.

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
