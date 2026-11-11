## Change Status
- Status: complete
- Open tasks: none

## Implementation

- [x] [serial] I1 Add deterministic invalid supplement receipt validation and typed `status = "invalid"` JSON reports for malformed/missing/future/wrong-family receipt inputs.
- [x] [depends:I1] I2 Add focused supplement compatibility coverage for positive v1 replay, invalid receipt inputs, and boundary-tamper mismatch.
- [x] [depends:I2] I3 Add a named `supplement-compatibility-gate` Nix check and compose it into `valence-object-evidence` with a visible PASS line.

## Verification

- [x] [depends:I1,I2] V1 Verify focused supplement compatibility with command=`TMPDIR=/tmp nix develop -c bash -c 'cargo fmt --check && cargo test -p cargo-octet supplement_compatibility -- --nocapture'` evidence=`evidence/supplement-compatibility-tests.txt`.
- [x] [depends:I3,V1] V2 Verify the named Nix rail with command=`TMPDIR=/tmp nix build .#checks.x86_64-linux.supplement-compatibility-gate --no-link -L` evidence=`evidence/supplement-compatibility-gate.txt`.
- [x] [depends:V2] V3 Verify aggregate composition with command=`TMPDIR=/tmp nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L` evidence=`evidence/valence-object-evidence.txt`.
- [x] [depends:V3] V4 Verify OpenSpec and diff hygiene with commands=`openspec validate add-supplement-compatibility-fixtures --strict --json`, `openspec validate --all --strict --json`, and `git diff --check` evidence=`evidence/openspec-and-diff.txt`.

## Hardening

- [x] [depends:V4] H1 Save/confirm evidence transcripts include positive v1 pass, typed invalid errors, boundary mismatch, named Nix PASS, aggregate PASS line, and non-proof boundaries.

## Verification Coverage

| Scenario | Tasks | Evidence |
| --- | --- | --- |
| Supplement compatibility accepts positive v1 receipt | I1, I2, V1 | `evidence/supplement-compatibility-tests.txt` |
| Supplement compatibility rejects invalid receipts | I1, I2, V1 | `evidence/supplement-compatibility-tests.txt` |
| Supplement boundary tamper fails closed | I2, V1 | `evidence/supplement-compatibility-tests.txt` |
| Aggregate gate composes supplement compatibility rail | I3, V2, V3 | `evidence/supplement-compatibility-gate.txt`, `evidence/valence-object-evidence.txt` |
