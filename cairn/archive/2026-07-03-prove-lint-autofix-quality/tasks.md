## Contract

- [x] [serial] Specify mandatory `.fixed` proof for every `MachineApplicable` suggestion. r[octet.lint_autofix_quality.machine_applicable]
- [x] [serial] Specify visible rationale and tier evidence for `MaybeIncorrect` and `HasPlaceholders` suggestions. r[octet.lint_autofix_quality.non_machine_tiers]

## Implementation

- [x] [serial] Extend library freshness tests to map each machine-applicable lint to a checked fixed fixture or named out-of-scope rationale. r[octet.lint_autofix_quality.machine_applicable]
- [x] [parallel] Add `.fixed` artifacts for existing machine-applicable suggestions that lack one. r[octet.lint_autofix_quality.machine_applicable]
- [x] [parallel] Update diagnostics/docs so non-machine tiers are explicit in stderr and user guidance. r[octet.lint_autofix_quality.non_machine_tiers]

## Verification

- [x] [serial] Run `nix develop -c bash -c 'cargo test --test ui autofix_machine_applicable -- --ignored --nocapture'`. r[octet.lint_autofix_quality.machine_applicable]
- [x] [serial] Run the regular UI rail and confirm non-machine suggestions keep tier notes. r[octet.lint_autofix_quality.non_machine_tiers]
- [x] [serial] Run `cargo test --lib` and Cairn validation/gates. r[octet.lint_autofix_quality.machine_applicable]
