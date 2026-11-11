# Aeneas/Charon provider closure and live compatibility

Date: 2026-11-09

## Closure built

`nix build .#octet-aeneas-lean-provider-closure` completed successfully.

- Aeneas revision: `bc2eb6f941369fa3c695e30598b5758b11e4f0de`
- Charon revision: `cb50ff16b9f1066b8a97dc06da704de2da2fa41c`
- Provider status: `packaged`
- Mathlib lake hash: `sha256-wq4OBVgChJi59Widl+vuAPoq+MNZfFmdJwcsREW5RIo=`
- Closure path: `/nix/store/3lh4savplcckg4zzg0kpx9747md8g53w-octet-aeneas-lean-provider-closure-x86_64-linux`

The closure also contains the pinned Lean/Lake binaries, `leantar`, the
Aeneas Lean library, and the network sandbox.

## Live compatibility matrix

`nix build .#checks.x86_64-linux.aeneas-lean-compatibility-matrix` passed.
The live provider suite ran four tests with the closure environment:

```text
test aeneas_execution::tests::live_provider_executes_real_positive_fixture ... ok
test aeneas_execution::tests::live_provider_rejects_real_false_fixture ... ok
test aeneas_execution::tests::live_provider_rejects_real_vacuous_fixture ... ok
test prove::tests::live_provider_builds_external_consumer_binding ... ok

test result: ok. 4 passed; 0 failed
```

## Scope and non-claims

This proves that the pinned Aeneas/Charon/Lean provider closure builds and
that the existing live positive, false, vacuous, and external-consumer
fixtures execute through it. It does not check the Octet u64 checked-increment
pilot through Aeneas, does not prove the `u32`-to-`u64` refinement bridge, and
does not establish provider soundness, whole-program correctness, semantic
equivalence, or release eligibility.
