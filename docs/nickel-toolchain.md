# Nickel evaluator cohort

Octet evaluates checked-path configuration with Nickel CLI `1.17.0` from `nickel-lang/nickel` revision `1320a983e6c3d1e2fb53dd2464b084b4903b1426`.

The Nix flake supplies this exact evaluator to policy, standards, proof-profile, evidence, fixture, and developer-tool commands. The cohort guard rejects ambient package-set references and checks the resolved executable and version.

Nickel Export manifests record evaluator identity `nickel-lang-cli nickel 1.17.0 (rev 1320a98)`. Octet rejects substituted identities before it accepts checked-path export evidence.

## Validation

```console
nix build .#checks.x86_64-linux.nickel-cohort-exact --no-link -L
nix build .#checks.x86_64-linux.nickel-toolchain-fixtures --no-link -L
nix run .#octet-policy-validate
```

A passing Nickel evaluation proves configuration shape only. It does not replace Octet lint policy, prove Rust correctness, prove proof-rail claims, establish evidence truth, or decide release eligibility.
